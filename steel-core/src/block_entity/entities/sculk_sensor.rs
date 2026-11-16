//! Sculk sensor and calibrated sculk sensor block entity & vibration system.
//!
//! Implements vanilla MC 26.3 `SculkSensorBlockEntity`,
//! `CalibratedSculkSensorBlockEntity`, and `VibrationSystem`:
//! - Exact 56-entry `VIBRATION_FREQUENCY_FOR_EVENT` mapping (`1..=15`,
//!   including `resonate_1`..`resonate_15`).
//! - Radius `8` (`SculkSensor`) / `16` (`CalibratedSculkSensor`).
//! - Calibrated back-face redstone frequency filter.
//! - Sneaking (`IGNORE_VIBRATIONS_SNEAKING`) + `AVOID_VIBRATION` criterion,
//!   spectator immunity, and `DAMPENS_VIBRATIONS` wool line-of-sight occlusion.
//! - Redstone signal strength `max(1, 15 - floor(15.0 / radius * distance))`,
//!   `last_vibration_frequency` comparator output, phase transitions
//!   (`Inactive -> Active -> Cooldown -> Inactive`), adjacent amethyst block
//!   resonance, and `sculk_sensor_tendrils_clicking` emission for shriekers.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::BaseNbtCompound as BorrowedNbtCompound;
use simdnbt::owned::NbtCompound;
use steel_protocol::packets::game::SoundSource;
use steel_registry::block_entity_type::BlockEntityTypeRef;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::{BlockStateProperties, SculkSensorPhase};
use steel_registry::game_events::GameEventRef;
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::{
    REGISTRY, RegistryExt as _, sound_events, vanilla_block_entity_types, vanilla_blocks,
    vanilla_criteria_triggers, vanilla_game_events,
};
use steel_utils::types::UpdateFlags;
use steel_utils::{
    BlockPos, BlockStateId, Direction, Downcast as _, DowncastType, DowncastTypeKey, Identifier,
    locks::SyncMutex,
};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::entity::Entity as _;
use crate::player::advancement::predicate::TriggerContext;
use crate::world::game_event::{GameEventContext, GameEventListener, SharedGameEventListener};
use crate::world::{SignalGetter as _, World};

/// Vanilla `SculkSensorBlockEntity.VibrationUser` radius.
pub const SCULK_SENSOR_RADIUS: i32 = 8;
/// Vanilla `CalibratedSculkSensorBlockEntity.VibrationUser` radius.
pub const CALIBRATED_SCULK_SENSOR_RADIUS: i32 = 16;
/// Vanilla `SculkSensorBlock.ACTIVE_TICKS`.
pub const SCULK_SENSOR_ACTIVE_TICKS: i32 = 30;
/// Vanilla `CalibratedSculkSensorBlock.ACTIVE_TICKS`.
pub const CALIBRATED_SCULK_SENSOR_ACTIVE_TICKS: i32 = 10;
/// Vanilla `SculkSensorBlock.COOLDOWN_TICKS`.
pub const SCULK_SENSOR_COOLDOWN_TICKS: i32 = 10;

/// Returns the vanilla MC 26.3 `VibrationSystem.getGameEventFrequency` (`1..=15`),
/// or `0` for game events that are not vibrations.
#[must_use]
pub fn vibration_frequency(event: GameEventRef) -> i32 {
    if event.key.namespace.as_ref() != "minecraft" {
        return 0;
    }
    match event.key.path.as_ref() {
        "step" | "swim" | "flap" | "resonate_1" => 1,
        "projectile_land" | "hit_ground" | "splash" | "bounce" | "resonate_2" => 2,
        "item_interact_finish" | "projectile_shoot" | "instrument_play" | "resonate_3" => 3,
        "entity_action" | "elytra_glide" | "unequip" | "resonate_4" => 4,
        "entity_dismount" | "equip" | "resonate_5" => 5,
        "entity_interact" | "shear" | "entity_mount" | "resonate_6" => 6,
        "entity_damage" | "resonate_7" => 7,
        "drink" | "eat" | "resonate_8" => 8,
        "container_close" | "block_close" | "block_deactivate" | "block_detach" | "resonate_9" => 9,
        "container_open" | "block_open" | "block_activate" | "block_attach" | "prime_fuse"
        | "note_block_play" | "resonate_10" => 10,
        "block_change" | "resonate_11" => 11,
        "block_destroy" | "fluid_pickup" | "resonate_12" => 12,
        "block_place" | "fluid_place" | "resonate_13" => 13,
        "entity_place" | "lightning_strike" | "teleport" | "resonate_14" => 14,
        "entity_die" | "explode" | "resonate_15" => 15,
        _ => 0,
    }
}

/// Vanilla `minecraft:ignore_vibrations_sneaking` tag entries.
#[must_use]
pub fn ignored_while_sneaking(event: GameEventRef) -> bool {
    event.key.namespace.as_ref() == "minecraft"
        && matches!(
            event.key.path.as_ref(),
            "hit_ground" | "projectile_shoot" | "step" | "swim"
        )
}

/// Vanilla `VibrationSystem.getRedstoneStrengthForDistance`.
#[must_use]
pub fn redstone_strength_for_distance(distance: f64, listener_radius: i32) -> i32 {
    if listener_radius <= 0 {
        return 1;
    }
    let scale = 15.0 / f64::from(listener_radius);
    (15 - (scale * distance).floor() as i32).clamp(1, 15)
}

/// Returns whether a line between `from` and `to` passes through a block tagged
/// `minecraft:dampens_vibrations` (wool / wool carpets).
#[must_use]
pub fn is_vibration_occluded(world: &World, from: DVec3, to: DVec3) -> bool {
    let delta = to - from;
    let length = delta.length();
    if length < 1.0e-4 {
        return false;
    }
    let steps = (length * 2.0).ceil() as i32;
    let from_block = BlockPos::new(
        from.x.floor() as i32,
        from.y.floor() as i32,
        from.z.floor() as i32,
    );
    let to_block = BlockPos::new(
        to.x.floor() as i32,
        to.y.floor() as i32,
        to.z.floor() as i32,
    );
    let mut last = from_block;
    for step in 1..steps {
        let t = f64::from(step) / f64::from(steps);
        let p = from + delta * t;
        let block_pos = BlockPos::new(p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32);
        if block_pos == last || block_pos == to_block {
            continue;
        }
        last = block_pos;
        if world
            .get_block_state(block_pos)
            .get_block()
            .has_tag(&BlockTag::DAMPENS_VIBRATIONS)
        {
            return true;
        }
    }
    false
}

fn resonance_event(frequency: i32) -> Option<GameEventRef> {
    let name = match frequency {
        1 => "resonate_1",
        2 => "resonate_2",
        3 => "resonate_3",
        4 => "resonate_4",
        5 => "resonate_5",
        6 => "resonate_6",
        7 => "resonate_7",
        8 => "resonate_8",
        9 => "resonate_9",
        10 => "resonate_10",
        11 => "resonate_11",
        12 => "resonate_12",
        13 => "resonate_13",
        14 => "resonate_14",
        15 => "resonate_15",
        _ => return None,
    };
    REGISTRY
        .game_events
        .by_key(&Identifier::vanilla_static(name))
}

/// Sculk sensor and calibrated sculk sensor block entity.
pub struct SculkSensorBlockEntity {
    base: Arc<BlockEntityBase>,
    calibrated: bool,
    /// Last received vibration frequency, matching vanilla `lastVibrationFrequency`.
    last_vibration_frequency: SyncMutex<i32>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `SculkSensorBlockEntity`.
unsafe impl DowncastType for SculkSensorBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/sculk_sensor");
}

impl SculkSensorBlockEntity {
    /// Creates a new sculk sensor block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self::new_with_type(
            &vanilla_block_entity_types::SCULK_SENSOR,
            false,
            level,
            pos,
            state,
        )
    }

    /// Creates a new calibrated sculk sensor block entity.
    #[must_use]
    pub fn new_calibrated(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self::new_with_type(
            &vanilla_block_entity_types::CALIBRATED_SCULK_SENSOR,
            true,
            level,
            pos,
            state,
        )
    }

    fn new_with_type(
        block_entity_type: BlockEntityTypeRef,
        calibrated: bool,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(block_entity_type, level, pos, state)),
            calibrated,
            last_vibration_frequency: SyncMutex::new(0),
        }
    }

    /// Whether this block entity backs a calibrated sculk sensor.
    #[must_use]
    pub const fn is_calibrated(&self) -> bool {
        self.calibrated
    }

    /// Last received vibration frequency (`0` when none was received yet).
    #[must_use]
    pub fn last_vibration_frequency(&self) -> i32 {
        *self.last_vibration_frequency.lock()
    }

    /// Records a received vibration frequency (`0..=15`).
    pub fn set_last_vibration_frequency(&self, frequency: i32) {
        *self.last_vibration_frequency.lock() = frequency.clamp(0, 15);
    }

    /// Activates the sensor block at `pos` (`SculkSensorBlock.activate`).
    pub fn activate(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        power: i32,
        frequency: i32,
        context: &GameEventContext<'_>,
    ) {
        self.set_last_vibration_frequency(frequency);
        self.set_changed();

        let next_state = state
            .set_value(
                &BlockStateProperties::SCULK_SENSOR_PHASE,
                SculkSensorPhase::Active,
            )
            .set_value(&BlockStateProperties::POWER, power.clamp(1, 15) as u8);
        let block = state.get_block();
        world.set_block(pos, next_state, UpdateFlags::UPDATE_ALL);
        let active_ticks = if self.calibrated {
            CALIBRATED_SCULK_SENSOR_ACTIVE_TICKS
        } else {
            SCULK_SENSOR_ACTIVE_TICKS
        };
        world.schedule_block_tick_default(pos, block, active_ticks);
        world.update_neighbors_at(pos, block);
        world.update_neighbors_at(pos.below(), block);
        world.update_neighbor_for_output_signal(pos, block);

        // Emit `sculk_sensor_tendrils_clicking` so nearby shriekers wake up.
        world.game_event(
            &vanilla_game_events::SCULK_SENSOR_TENDRILS_CLICKING,
            pos,
            context,
        );

        // Vanilla amethyst resonance: an adjacent amethyst block re-emits the
        // exact frequency so chains of sensors transmit frequency across a room.
        if !event_is_adjacent_amethyst_absent(world, pos)
            && let Some(resonance) = resonance_event(frequency)
        {
            for dir in Direction::ALL {
                let adj = pos.relative(dir);
                if world.get_block_state(adj).get_block() == &vanilla_blocks::AMETHYST_BLOCK {
                    world.game_event(
                        resonance,
                        adj,
                        &GameEventContext::new(context.source_entity(), None),
                    );
                    break;
                }
            }
        }

        let waterlogged = state
            .try_get_value(&BlockStateProperties::WATERLOGGED)
            .unwrap_or(false);
        if !waterlogged {
            let pitch = if self.calibrated { 1.5 } else { 1.0 };
            world.play_sound(
                &sound_events::BLOCK_SCULK_SENSOR_CLICKING,
                SoundSource::Blocks,
                pos,
                1.0,
                pitch,
                None,
            );
        }
    }
}

fn event_is_adjacent_amethyst_absent(world: &World, pos: BlockPos) -> bool {
    !Direction::ALL
        .iter()
        .any(|&dir| world.get_block_state(pos.relative(dir)).get_block() == &vanilla_blocks::AMETHYST_BLOCK)
}

/// Vanilla `CalibratedSculkSensorBlockEntity.getBackSignal`: reads the redstone
/// signal fed into the face opposite `HORIZONTAL_FACING`.
#[must_use]
fn calibrated_input_filter(world: &World, pos: BlockPos, state: BlockStateId) -> i32 {
    let Some(facing) = state.try_get_value(&BlockStateProperties::HORIZONTAL_FACING) else {
        return 0;
    };
    let input_dir = facing.opposite();
    world.get_signal(pos.relative(input_dir), input_dir)
}

struct SculkSensorListener {
    pos: BlockPos,
    calibrated: bool,
}

impl SculkSensorListener {
    fn center(&self) -> DVec3 {
        DVec3::new(
            f64::from(self.pos.x()) + 0.5,
            f64::from(self.pos.y()) + 0.5,
            f64::from(self.pos.z()) + 0.5,
        )
    }
}

impl GameEventListener for SculkSensorListener {
    fn listener_pos(&self) -> Option<DVec3> {
        Some(self.center())
    }

    fn listener_radius(&self) -> i32 {
        if self.calibrated {
            CALIBRATED_SCULK_SENSOR_RADIUS
        } else {
            SCULK_SENSOR_RADIUS
        }
    }

    fn handle_game_event(
        &self,
        world: &Arc<World>,
        event: GameEventRef,
        context: &GameEventContext<'_>,
        source_pos: DVec3,
    ) -> bool {
        let frequency = vibration_frequency(event);
        if frequency == 0 {
            return false;
        }

        if let Some(source) = context.source_entity() {
            if source.is_spectator() {
                return false;
            }
            if source.is_stepping_carefully() && ignored_while_sneaking(event) {
                if let Some(player) = source.as_player() {
                    player.trigger_advancement_with_context(
                        &vanilla_criteria_triggers::AVOID_VIBRATION,
                        &TriggerContext::at(&player.get_world(), player.position()),
                    );
                }
                return false;
            }
        }

        // Ignore block-change events emitted by the sensor updating itself.
        let source_block = BlockPos::new(
            source_pos.x.floor() as i32,
            source_pos.y.floor() as i32,
            source_pos.z.floor() as i32,
        );
        if source_block == self.pos
            && matches!(
                event.key.path.as_ref(),
                "block_change" | "block_activate" | "block_deactivate"
            )
        {
            return false;
        }

        let state = world.get_block_state(self.pos);
        let Some(phase) = state.try_get_value(&BlockStateProperties::SCULK_SENSOR_PHASE) else {
            return false;
        };
        if phase != SculkSensorPhase::Inactive {
            return false;
        }

        if self.calibrated {
            let filter = calibrated_input_filter(world, self.pos, state);
            if filter != 0 && filter != frequency {
                return false;
            }
        }

        let center = self.center();
        if is_vibration_occluded(world, source_pos, center) {
            return false;
        }

        let Some(be) = world.get_block_entity(self.pos) else {
            return false;
        };
        let Some(sensor) = be.downcast_ref::<SculkSensorBlockEntity>() else {
            return false;
        };

        let distance = source_pos.distance(center);
        let power = redstone_strength_for_distance(distance, self.listener_radius());
        sensor.activate(world, self.pos, state, power, frequency, context);
        true
    }
}

impl BlockEntity for SculkSensorBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn game_event_listener(&self) -> Option<SharedGameEventListener> {
        Some(Arc::new(SculkSensorListener {
            pos: self.base.pos(),
            calibrated: self.calibrated,
        }))
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let view: simdnbt::borrow::NbtCompound<'_, '_> = nbt.into();
        *self.last_vibration_frequency.lock() =
            view.int("last_vibration_frequency").unwrap_or(0).clamp(0, 15);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        nbt.insert("last_vibration_frequency", *self.last_vibration_frequency.lock());
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;

    use super::*;

    #[test]
    fn sensor_records_frequency() {
        init_vanilla_registry();
        let sensor = SculkSensorBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::SCULK_SENSOR.default_state(),
        );
        assert_eq!(sensor.last_vibration_frequency(), 0);
        sensor.set_last_vibration_frequency(7);
        assert_eq!(sensor.last_vibration_frequency(), 7);
    }

    #[test]
    fn vibration_frequencies_match_vanilla_26_3_table() {
        init_vanilla_registry();
        assert_eq!(vibration_frequency(&vanilla_game_events::STEP), 1);
        assert_eq!(vibration_frequency(&vanilla_game_events::PROJECTILE_LAND), 2);
        assert_eq!(vibration_frequency(&vanilla_game_events::PROJECTILE_SHOOT), 3);
        assert_eq!(vibration_frequency(&vanilla_game_events::ELYTRA_GLIDE), 4);
        assert_eq!(vibration_frequency(&vanilla_game_events::EQUIP), 5);
        assert_eq!(vibration_frequency(&vanilla_game_events::ENTITY_INTERACT), 6);
        assert_eq!(vibration_frequency(&vanilla_game_events::ENTITY_DAMAGE), 7);
        assert_eq!(vibration_frequency(&vanilla_game_events::EAT), 8);
        assert_eq!(vibration_frequency(&vanilla_game_events::CONTAINER_CLOSE), 9);
        assert_eq!(vibration_frequency(&vanilla_game_events::CONTAINER_OPEN), 10);
        assert_eq!(vibration_frequency(&vanilla_game_events::BLOCK_CHANGE), 11);
        assert_eq!(vibration_frequency(&vanilla_game_events::BLOCK_DESTROY), 12);
        assert_eq!(vibration_frequency(&vanilla_game_events::BLOCK_PLACE), 13);
        assert_eq!(vibration_frequency(&vanilla_game_events::TELEPORT), 14);
        assert_eq!(vibration_frequency(&vanilla_game_events::EXPLODE), 15);
        assert_eq!(vibration_frequency(&vanilla_game_events::SHRIEK), 0);
    }

    #[test]
    fn distance_attenuation_matches_vanilla_formula() {
        assert_eq!(redstone_strength_for_distance(0.0, 8), 15);
        assert_eq!(redstone_strength_for_distance(4.0, 8), 8);
        assert_eq!(redstone_strength_for_distance(7.9, 8), 1);
        assert_eq!(redstone_strength_for_distance(8.0, 16), 8);
    }
}
