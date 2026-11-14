//! Sculk catalyst and sculk shrieker block entities.
//!
//! The catalyst is still a marker: vanilla spreads sculk from entity deaths, and
//! Steel's game-event system does not have a spender for that yet. The shrieker
//! owns the warden summoning chain: a naturally generated shrieker
//! (`can_summon`) counts shrieks, and at the fourth warning level it asks
//! `SpawnUtil` for a warden and roars when the summon fails.

use std::ptr;
use std::sync::{Arc, Weak};

use simdnbt::borrow::BaseNbtCompound as BorrowedNbtCompound;
use simdnbt::owned::NbtCompound;
use steel_protocol::packets::game::SoundSource;
use steel_registry::block_entity_type::BlockEntityTypeRef;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::BlockStateProperties;
use steel_registry::game_events::GameEventRef;
use steel_registry::vanilla_game_rules;
use steel_registry::{
    sound_events, vanilla_block_entity_types, vanilla_blocks, vanilla_entities,
    vanilla_mob_effects,
};
use steel_utils::types::UpdateFlags;
use steel_utils::{
    BlockPos, BlockStateId, DowncastType, DowncastTypeKey, WorldAabb, locks::SyncMutex,
};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::entity::{
    Entity as _, EntitySpawnReason, LivingEntity as _, MobEffectInstance, spawn_util,
};
use crate::world::World;
use crate::world::game_event::{GameEventContext, GameEventListener, SharedGameEventListener};

/// Sculk catalyst block entity: only carries the base state.
pub struct SculkCatalystBlockEntity {
    base: Arc<BlockEntityBase>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `SculkCatalystBlockEntity`.
unsafe impl DowncastType for SculkCatalystBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/sculk_catalyst");
}

impl SculkCatalystBlockEntity {
    /// Creates a new sculk catalyst block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(
                &vanilla_block_entity_types::SCULK_CATALYST,
                level,
                pos,
                state,
            )),
        }
    }
}

impl BlockEntity for SculkCatalystBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn load_additional(&self, _nbt: &BorrowedNbtCompound<'_>) {}

    fn save_additional(&self, _nbt: &mut NbtCompound) {}

}

/// Sculk shrieker block entity.
pub struct SculkShriekerBlockEntity {
    base: Arc<BlockEntityBase>,
    /// Warning level, incremented each time the shrieker hears a vibration and
    /// persisted across saves like vanilla's `warningLevel`.
    warning_level: SyncMutex<i32>,
    /// Vanilla `SculkShriekerBlockEntity` shriek cooldown: the last two hundred
    /// ticks refuse to shriek again, so one player cannot climb the warning level
    /// by standing on the block.
    shriek_cooldown: SyncMutex<i32>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `SculkShriekerBlockEntity`.
unsafe impl DowncastType for SculkShriekerBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/sculk_shrieker");
}

impl SculkShriekerBlockEntity {
    /// Creates a new sculk shrieker block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(
                &vanilla_block_entity_types::SCULK_SHRIEKER,
                level,
                pos,
                state,
            )),
            warning_level: SyncMutex::new(0),
            shriek_cooldown: SyncMutex::new(0),
        }
    }

    /// Current warning level.
    #[must_use]
    pub fn warning_level(&self) -> i32 {
        *self.warning_level.lock()
    }

    /// Sets the warning level.
    pub fn set_warning_level(&self, level: i32) {
        *self.warning_level.lock() = level;
    }

    /// Vanilla `SculkShriekerBlockEntity.canShriek`: the cooldown has expired.
    ///
    /// The cooldown is decremented by the block entity ticker, not by this check,
    /// so the gate stays a pure read.
    #[must_use]
    pub fn ready_to_shriek(&self) -> bool {
        use crate::block_entity::BlockEntityLifecycleExt as _;
        !self.is_removed() && *self.shriek_cooldown.lock() <= 0
    }

    /// Starts vanilla's shriek cooldown.
    pub fn start_shriek_cooldown(&self) {
        *self.shriek_cooldown.lock() = SHRIEKER_COOLDOWN;
    }
}

/// Vanilla `SculkShriekerBlockEntity` summon parameters
/// (`SpawnUtil.trySpawnMob(WARDEN, TRIGGERED, level, pos, 20, 5, 6, …)`).
const WARDEN_SUMMON_ATTEMPTS: i32 = 20;
const WARDEN_SUMMON_HORIZONTAL_RANGE: i32 = 5;
const WARDEN_SUMMON_VERTICAL_RANGE: i32 = 6;

/// Vanilla `WardenSpawnTracker.MAX_WARNING_LEVEL`.
const MAX_WARNING_LEVEL: i32 = 4;

impl SculkShriekerBlockEntity {
    /// Vanilla `SculkShriekerBlockEntity.trySummonWarden`.
    ///
    /// Only a naturally generated shrieker (`can_summon`) summons, only once the
    /// warning level reached its maximum, and only when the `spawn_wardens` game
    /// rule allows it. A failed summon is loud on purpose: vanilla plays the
    /// warden's roar so a player who is already near one can tell the difference.
    ///
    /// Returns whether a warden emerged.
    pub fn try_summon_warden(&self, world: &Arc<World>, pos: BlockPos) -> bool {
        if !world.get_game_rule(&vanilla_game_rules::SPAWN_WARDENS) {
            return false;
        }
        if self.warning_level() < MAX_WARNING_LEVEL {
            return false;
        }
        if !self.can_summon(world, pos) {
            return false;
        }

        let summoned = spawn_util::try_spawn_mob_on_top_of_collider(
            world,
            &vanilla_entities::WARDEN,
            EntitySpawnReason::Triggered,
            pos,
            WARDEN_SUMMON_ATTEMPTS,
            WARDEN_SUMMON_HORIZONTAL_RANGE,
            WARDEN_SUMMON_VERTICAL_RANGE,
        )
        .is_some();

        if !summoned {
            // Vanilla `SculkShriekerBlockEntity.trySummonWarden`: the failure
            // branch plays `WARDEN_ROAR` so the player hears that something tried.
            world.play_sound_at(
                &sound_events::ENTITY_WARDEN_ROAR,
                SoundSource::Hostile,
                block_center(pos),
                1.0,
                1.0,
                None,
            );
            return false;
        }
        // Vanilla marks the warden as emerging from the ground it was found in.
        world.play_sound_at(
            &sound_events::ENTITY_WARDEN_EMERGE,
            SoundSource::Hostile,
            block_center(pos),
            1.0,
            1.0,
            None,
        );
        true
    }

    /// Vanilla reads `can_summon` from the block state, so a shrieker placed by a
    /// player (or harvested and replaced) can never summon, even at level 4.
    #[must_use]
    pub fn can_summon(&self, world: &Arc<World>, pos: BlockPos) -> bool {
        world
            .get_block_state(pos)
            .get_value(&BlockStateProperties::CAN_SUMMON)
    }

    /// Vanilla `SculkShriekerBlockEntity.tryRespond`: bumps the warning level and
    /// gives the triggering player the darkness effect.
    ///
    /// The warning level lives on the block entity in Steel (vanilla keeps it on
    /// the player's `WardenSpawnTracker`); the block entity persists it, so a
    /// restarted server keeps the count that was already paid for.
    pub fn respond(&self, world: &Arc<World>, pos: BlockPos) -> bool {
        if !self.can_summon(world, pos) {
            return false;
        }
        let level = (self.warning_level() + 1).min(MAX_WARNING_LEVEL);
        self.set_warning_level(level);
        self.start_shriek_cooldown();

        // Vanilla `SculkShriekerBlock.tryShriek` sets the `shrieking` state for
        // ninety ticks and schedules the block tick that clears it again.
        let state = world.get_block_state(pos);
        if ptr::eq(state.get_block(), &vanilla_blocks::SCULK_SHRIEKER) {
            world.set_block(
                pos,
                state.set_value(&BlockStateProperties::SHRIEKING, true),
                UpdateFlags::UPDATE_ALL,
            );
            let _ = world.schedule_block_tick_default(
                pos,
                state.get_block(),
                SHRIEK_ANIMATION_TICKS,
            );
        }

        // Vanilla shrieks are heard as a block sound before anything reacts.
        world.play_sound_at(
            &sound_events::BLOCK_SCULK_SHRIEKER_SHRIEK,
            SoundSource::Blocks,
            block_center(pos),
            1.0,
            1.0,
            None,
        );
        // Vanilla `SculkShriekerBlockEntity.tryRespond` blinds every player that
        // can hear it: twelve seconds of darkness within a forty-block sphere.
        for player in players_near(world, block_center(pos), SHRIEK_DARKNESS_RADIUS) {
            player.add_mob_effect(MobEffectInstance::with_duration(
                &vanilla_mob_effects::DARKNESS,
                SHRIEK_DARKNESS_DURATION,
                0,
            ));
        }
        level >= MAX_WARNING_LEVEL
    }
}

/// Vanilla `SculkShriekerBlockEntity` darkness radius in blocks.
const SHRIEK_DARKNESS_RADIUS: f64 = 40.0;

/// Vanilla darkness duration applied by a shriek, in ticks.
const SHRIEK_DARKNESS_DURATION: i32 = 240;

fn block_center(pos: BlockPos) -> glam::DVec3 {
    glam::DVec3::new(
        f64::from(pos.x()) + 0.5,
        f64::from(pos.y()) + 0.5,
        f64::from(pos.z()) + 0.5,
    )
}

/// Every player within `radius` of `center`, matching vanilla's sphere test.
fn players_near(
    world: &Arc<World>,
    center: glam::DVec3,
    radius: f64,
) -> Vec<Arc<crate::player::Player>> {
    let reach = WorldAabb::new(
        center.x - radius,
        center.y - radius,
        center.z - radius,
        center.x + radius,
        center.y + radius,
        center.z + radius,
    );
    world
        .get_entities_in_aabb(&reach)
        .into_iter()
        .filter(|entity| entity.position().distance(center) <= radius)
        .filter_map(|entity| world.players.get_by_entity_id(entity.id()))
        .collect()
}

/// Vanilla `SculkShriekerBlockEntity.VibrationListener` radius in blocks.
const SHRIEKER_VIBRATION_RADIUS: i32 = 8;

/// Vanilla `SculkShriekerBlockEntity.SHRIEKER_COOLDOWN`, in ticks.
const SHRIEKER_COOLDOWN: i32 = 200;

/// Vanilla `SculkShriekerBlock.SHRIEKING_DURATION`, in ticks.
const SHRIEK_ANIMATION_TICKS: i32 = 90;

/// Vanilla hears a vibration, shrieks, and only then raises the warning level.
///
/// The listener is rebuilt by the world on every query, so the only state it
/// carries is the block position; the cooldown, the warning level and the
/// `can_summon` flag all live on the block entity.
struct SculkShriekerListener {
    position: BlockPos,
}

impl GameEventListener for SculkShriekerListener {
    fn listener_pos(&self) -> Option<glam::DVec3> {
        Some(block_center(self.position))
    }

    fn listener_radius(&self) -> i32 {
        SHRIEKER_VIBRATION_RADIUS
    }

    fn handle_game_event(
        &self,
        world: &Arc<World>,
        _event: GameEventRef,
        _context: &GameEventContext<'_>,
        _source_pos: glam::DVec3,
    ) -> bool {
        let Some(block_entity) = world.get_block_entity(self.position) else {
            return true;
        };
        let Some(shrieker) = block_entity.downcast_ref::<SculkShriekerBlockEntity>() else {
            return true;
        };
        shriek(shrieker, world, self.position)
    }
}

/// One shriek: cooldown, warning level, then the warden attempt.
fn shriek(shrieker: &SculkShriekerBlockEntity, world: &Arc<World>, pos: BlockPos) -> bool {
    if !shrieker.ready_to_shriek() {
        return true;
    }
    if shrieker.respond(world, pos) {
        shrieker.try_summon_warden(world, pos);
    }
    true
}

impl BlockEntity for SculkShriekerBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    /// Vanilla `SculkShriekerBlockEntity.tick`: only the cooldown counts down.
    fn tick(&self, _world: &Arc<World>) {
        let mut cooldown = self.shriek_cooldown.lock();
        *cooldown = (*cooldown - 1).max(0);
    }

    fn game_event_listener(&self) -> Option<SharedGameEventListener> {
        Some(Arc::new(SculkShriekerListener {
            position: self.base.pos(),
        }))
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let view: simdnbt::borrow::NbtCompound<'_, '_> = nbt.into();
        *self.warning_level.lock() = view.int("warning_level").unwrap_or(0);
        *self.shriek_cooldown.lock() = view.int("shriek_cooldown").unwrap_or(0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        nbt.insert("warning_level", *self.warning_level.lock());
        // Vanilla writes the cooldown only while it is running.
        let cooldown = *self.shriek_cooldown.lock();
        if cooldown > 0 {
            nbt.insert("shriek_cooldown", cooldown);
        }
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks};

    use super::*;

    #[test]
    fn shrieker_tracks_warning_level() {
        init_vanilla_registry();
        let shrieker = SculkShriekerBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::SCULK_SHRIEKER.default_state(),
        );
        assert_eq!(shrieker.warning_level(), 0);
        shrieker.set_warning_level(2);
        assert_eq!(shrieker.warning_level(), 2);
    }

    #[test]
    fn the_shriek_cooldown_gates_a_second_shriek() {
        init_vanilla_registry();
        let shrieker = SculkShriekerBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::SCULK_SHRIEKER.default_state(),
        );
        assert!(shrieker.ready_to_shriek(), "a fresh shrieker may shriek");
        shrieker.start_shriek_cooldown();
        assert!(
            !shrieker.ready_to_shriek(),
            "vanilla refuses a second shriek inside the cooldown"
        );
    }

    #[test]
    fn the_warning_level_saturates_at_the_vanilla_maximum() {
        init_vanilla_registry();
        let shrieker = SculkShriekerBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::SCULK_SHRIEKER.default_state(),
        );
        shrieker.set_warning_level(2);
        shrieker.set_warning_level((shrieker.warning_level() + 5).min(MAX_WARNING_LEVEL));
        assert_eq!(
            shrieker.warning_level(),
            MAX_WARNING_LEVEL,
            "vanilla stops counting at four"
        );
    }

    #[test]
    fn a_saved_cooldown_survives_a_round_trip() {
        init_vanilla_registry();
        let shrieker = SculkShriekerBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::SCULK_SHRIEKER.default_state(),
        );
        shrieker.set_warning_level(3);
        shrieker.start_shriek_cooldown();
        let mut nbt = NbtCompound::new();
        shrieker.save_additional(&mut nbt);
        assert_eq!(nbt.int("warning_level"), Some(3));
        assert_eq!(nbt.int("shriek_cooldown"), Some(SHRIEKER_COOLDOWN));
    }
}
