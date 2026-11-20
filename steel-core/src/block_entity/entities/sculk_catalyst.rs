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
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::BlockStateProperties;
use steel_registry::game_events::GameEventRef;
use steel_registry::{level_events, vanilla_game_rules};
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::vanilla_entities::PLAYER;
use steel_registry::vanilla_game_events::ENTITY_DIE;
use steel_registry::vanilla_game_events::SCULK_SENSOR_TENDRILS_CLICKING;
use steel_registry::vanilla_game_events::SHRIEK;
use steel_registry::{
    sound_events, vanilla_block_entity_types, vanilla_blocks, vanilla_entities, vanilla_mob_effects,
};
use steel_utils::types::Difficulty;
use steel_utils::types::UpdateFlags;
use steel_utils::{
    BlockPos, BlockStateId, Downcast as _, DowncastType, DowncastTypeKey, WorldAabb,
    locks::SyncMutex,
};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::entity::warden_spawn_tracker::MAX_WARNING_LEVEL;
use crate::block_entity::entities::sculk_sensor::is_vibration_occluded;
use crate::entity::{
    Entity, EntitySpawnReason, LivingEntity as _, MobEffectInstance, WardenSpawnTracker, spawn_util,
};
use crate::player::Player;
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

/// Vanilla `SculkCatalystBlockEntity.CatalystListener` radius in blocks.
const CATALYST_LISTENER_RADIUS: i32 = 8;
/// Vanilla `SculkCatalystBlock.PULSE_TICKS` before clearing `bloom`.
pub const CATALYST_BLOOM_TICKS: i32 = 8;

struct SculkCatalystListener {
    position: BlockPos,
}

impl GameEventListener for SculkCatalystListener {
    fn listener_pos(&self) -> Option<glam::DVec3> {
        Some(block_center(self.position))
    }

    fn listener_radius(&self) -> i32 {
        CATALYST_LISTENER_RADIUS
    }

    fn handle_game_event(
        &self,
        world: &Arc<World>,
        event: GameEventRef,
        context: &GameEventContext<'_>,
        source_pos: glam::DVec3,
    ) -> bool {
        if event.key != ENTITY_DIE.key {
            return false;
        }
        // Only living entities (not inanimate drops/projectiles) feed a catalyst.
        let active = context
            .source_entity()
            .is_some_and(|entity| entity.as_living_entity().is_none());
        if active {
            return false;
        }

        let catalyst_state = world.get_block_state(self.position);
        if !ptr::eq(
            catalyst_state.get_block(),
            &raw const vanilla_blocks::SCULK_CATALYST,
        ) {
            return false;
        }

        // Bloom the catalyst and schedule its 8-tick pulse reset.
        if !catalyst_state.get_value(&BlockStateProperties::BLOOM) {
            world.set_block(
                self.position,
                catalyst_state.set_value(&BlockStateProperties::BLOOM, true),
                UpdateFlags::UPDATE_ALL,
            );
            world.schedule_block_tick_default(
                self.position,
                &vanilla_blocks::SCULK_CATALYST,
                CATALYST_BLOOM_TICKS,
            );
            world.play_sound(
                &sound_events::BLOCK_SCULK_CATALYST_BLOOM,
                SoundSource::Blocks,
                self.position,
                2.0,
                1.0,
                None,
            );
        }

        // Spread sculk across nearby `minecraft:sculk_replaceable` blocks around
        // the slain entity's feet, mirroring vanilla's `SculkSpreader` cursor.
        let origin = BlockPos::new(
            source_pos.x.floor() as i32,
            source_pos.y.floor() as i32 - 1,
            source_pos.z.floor() as i32,
        );
        let sculk_state = vanilla_blocks::SCULK.default_state();
        let mut budget = 5;
        for dy in [-1, 0, 1] {
            for dx in -2_i32..=2 {
                for dz in -2_i32..=2 {
                    if budget <= 0 {
                        break;
                    }
                    if dx.abs() + dz.abs() > 3 {
                        continue;
                    }
                    let target = origin.offset(dx, dy, dz);
                    let current = world.get_block_state(target);
                    if current
                        .get_block()
                        .has_tag(&BlockTag::SCULK_REPLACEABLE)
                        && world.get_block_state(target.above()).is_air()
                    {
                        world.set_block(target, sculk_state, UpdateFlags::UPDATE_ALL);
                        budget -= 1;
                    }
                }
            }
        }
        true
    }
}

impl BlockEntity for SculkCatalystBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn game_event_listener(&self) -> Option<SharedGameEventListener> {
        Some(Arc::new(SculkCatalystListener {
            position: self.base.pos(),
        }))
    }

    fn load_additional(&self, _nbt: &BorrowedNbtCompound<'_>) {}

    fn save_additional(&self, _nbt: &mut NbtCompound) {}
}

/// Sculk shrieker block entity.
#[derive(Clone)]
pub struct SculkShriekerBlockEntity {
    base: Arc<BlockEntityBase>,
    /// Vanilla `SculkShriekerBlockEntity.warningLevel`: raised by
    /// `WardenSpawnTracker.tryWarn` on every shriek and persisted across saves.
    warning_level: Arc<SyncMutex<i32>>,
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
            warning_level: Arc::new(SyncMutex::new(0)),
        }
    }

    /// Current warning level.
    #[must_use]
    pub fn warning_level(&self) -> i32 {
        *self.warning_level.lock()
    }

    /// Sets the warning level, clamped exactly like vanilla `setWarningLevel`.
    pub fn set_warning_level(&self, level: i32) {
        *self.warning_level.lock() = level.clamp(0, MAX_WARNING_LEVEL);
    }
}

/// Vanilla `SculkShriekerBlockEntity.DARKNESS_RADIUS`.
const SHRIEK_DARKNESS_RADIUS: f64 = 40.0;
/// Vanilla `SculkShriekerBlockEntity.SOUND_BY_LEVEL` broadcast radius.
const WARNING_SOUND_RADIUS: i32 = 10;
/// Vanilla `SculkShriekerBlockEntity.WARDEN_SPAWN_ATTEMPTS`.
const WARDEN_SUMMON_ATTEMPTS: i32 = 20;
const WARDEN_SUMMON_HORIZONTAL_RANGE: i32 = 5;
const WARDEN_SUMMON_VERTICAL_RANGE: i32 = 6;

impl SculkShriekerBlockEntity {
    /// Vanilla `SculkShriekerBlockEntity.trySummonWarden`.
    ///
    /// Only a warning level of four asks for a warden; the caller has already
    /// checked `canRespond` (which covers `can_summon`, the difficulty and the
    /// `spawn_wardens` game rule).
    ///
    /// Returns whether a warden emerged.
    pub fn try_summon_warden(&self, world: &Arc<World>, pos: BlockPos) -> bool {
        if self.warning_level() < MAX_WARNING_LEVEL {
            return false;
        }
        spawn_util::try_spawn_mob_on_top_of_collider(
            world,
            &vanilla_entities::WARDEN,
            EntitySpawnReason::Triggered,
            pos,
            WARDEN_SUMMON_ATTEMPTS,
            WARDEN_SUMMON_HORIZONTAL_RANGE,
            WARDEN_SUMMON_VERTICAL_RANGE,
        )
        .is_some()
    }

    /// Vanilla reads `can_summon` from the block state, so a shrieker placed by a
    /// player (or harvested and replaced) can never summon, even at level 4.
    #[must_use]
    pub fn can_summon(&self, world: &Arc<World>, pos: BlockPos) -> bool {
        world
            .get_block_state(pos)
            .get_value(&BlockStateProperties::CAN_SUMMON)
    }

    /// Vanilla `SculkShriekerBlockEntity.tryGetPlayer`: the shriek is credited to
    /// a player, the player riding the source, the shooter of the projectile, or
    /// the thrower of the item.
    ///
    /// The returned handle has to be the world's own `Arc<Player>`:
    /// [`WardenSpawnTracker::try_warn`] identifies the triggering player with
    /// `Arc::ptr_eq`, so a fresh allocation would drop the credit. Vanilla has no
    /// such problem because it compares object identity.
    #[must_use]
    pub fn try_get_player(entity: &dyn Entity) -> Option<Arc<Player>> {
        // Vanilla checks the concrete player first, then the passenger, then the
        // projectile owner / item thrower, and stops at the first one that is a
        // player.
        let candidate = entity
            .as_player()
            .map(|player| player.base().uuid())
            .or_else(|| {
                [entity.controlling_passenger(), entity.projectile_owner()]
                    .into_iter()
                    .flatten()
                    .find_map(|owner| {
                        (owner.entity_type() == &PLAYER)
                            .then(|| owner.as_player().map(|player| player.base().uuid()))
                            .flatten()
                    })
            })?;
        entity.level()?.players.get_by_uuid(&candidate)
    }

    /// Vanilla `SculkShriekerBlockEntity.canRespond`.
    ///
    /// A shrieker responds when it may summon, the difficulty is not peaceful,
    /// and the `spawn_wardens` game rule is on.
    #[must_use]
    pub fn can_respond(&self, world: &Arc<World>, pos: BlockPos) -> bool {
        self.can_summon(world, pos)
            && world.difficulty() != Difficulty::Peaceful
            && world.get_game_rule(&vanilla_game_rules::SPAWN_WARDENS)
    }

    /// Vanilla `SculkShriekerBlockEntity.tryShriek`.
    ///
    /// Resets the warning level, asks `WardenSpawnTracker.tryWarn` whether the
    /// nearby players may be warned, and only then shrieks. A shriek that cannot
    /// respond (peaceful, player-placed shrieker, `spawn_wardens` off) still
    /// shrieks, but never raises a level.
    pub fn try_shriek(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        player: Option<&Arc<Player>>,
    ) -> bool {
        let Some(player) = player else {
            return false;
        };
        // Cheap type gate first: a shrieker's `SHRIEKING` state sits behind an
        // O(properties) lookup, and the vibration listener fires for every
        // vibration in range.
        let state = world.get_block_state(pos);
        if !ptr::eq(state.get_block(), &raw const vanilla_blocks::SCULK_SHRIEKER) {
            return false;
        }
        if state.get_value(&BlockStateProperties::SHRIEKING) {
            return false;
        }

        self.set_warning_level(0);
        if self.can_respond(world, pos) {
            if let Some(warned) = WardenSpawnTracker::try_warn(world, pos, player) {
                self.set_warning_level(warned);
            } else {
                return false;
            }
        }
        self.shriek(world, pos);
        true
    }

    /// Vanilla `SculkShriekerBlockEntity.shriek`: the animation, the tone and the
    /// game event, without touching the warning level.
    fn shriek(&self, world: &Arc<World>, pos: BlockPos) {
        let state = world.get_block_state(pos);
        if ptr::eq(state.get_block(), &raw const vanilla_blocks::SCULK_SHRIEKER) {
            world.set_block(
                pos,
                state.set_value(&BlockStateProperties::SHRIEKING, true),
                UpdateFlags::UPDATE_CLIENTS,
            );
            let () =
                world.schedule_block_tick_default(pos, state.get_block(), SHRIEK_ANIMATION_TICKS);
        }

        // Vanilla `shriek` uses `levelEvent(3007)`, which is the shriek tone
        // together with the ring of soul particles, not a bare sound.
        world.level_event(level_events::PARTICLES_SCULK_SHRIEK, pos, 0, None);
        // Vanilla `shriek` also emits the `SHRIEK` game event so sculk sensors
        // and catalysts react to it.
        world.game_event(
            &SHRIEK,
            pos,
            &GameEventContext::new(None, None),
        );
    }

    /// Vanilla `SculkShriekerBlockEntity.tryRespond`, called when the shrieking
    /// state ends (the block tick after ninety ticks clears it).
    ///
    /// Only a warning level above zero responds: it either summons a warden or
    /// plays the "listening" reply sound at a random point around the shrieker,
    /// and always blinds every player in a forty block sphere.
    pub fn try_respond(&self, world: &Arc<World>, pos: BlockPos) -> bool {
        if !self.can_respond(world, pos) || self.warning_level() <= 0 {
            return false;
        }

        if !self.try_summon_warden(world, pos) {
            self.play_warden_reply_sound(world, pos);
        }

        for player in players_near(world, block_center(pos), SHRIEK_DARKNESS_RADIUS) {
            player.add_mob_effect(MobEffectInstance::with_duration(
                vanilla_mob_effects::DARKNESS,
                SHRIEK_DARKNESS_DURATION,
                0,
            ));
        }
        true
    }

    /// Vanilla `playWardenReplySound`: one of the four "nearby" warden sounds,
    /// pitched at a random offset up to ten blocks away, at volume five.
    fn play_warden_reply_sound(&self, world: &Arc<World>, pos: BlockPos) {
        let sound = match self.warning_level() {
            1 => &sound_events::ENTITY_WARDEN_NEARBY_CLOSE,
            2 => &sound_events::ENTITY_WARDEN_NEARBY_CLOSER,
            3 => &sound_events::ENTITY_WARDEN_NEARBY_CLOSEST,
            4 => &sound_events::ENTITY_WARDEN_LISTENING_ANGRY,
            _ => return,
        };
        let mut rng = rand::rng();
        let mut offset = |value: i32| {
            value
                + rand::RngExt::random_range(&mut rng, -WARNING_SOUND_RADIUS..=WARNING_SOUND_RADIUS)
        };
        world.play_sound_at(
            sound,
            SoundSource::Hostile,
            glam::DVec3::new(
                f64::from(offset(pos.x())) + 0.5,
                f64::from(offset(pos.y())) + 0.5,
                f64::from(offset(pos.z())) + 0.5,
            ),
            5.0,
            1.0,
            None,
        );
    }
}

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
) -> Vec<Arc<Player>> {
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
        event: GameEventRef,
        context: &GameEventContext<'_>,
        source_pos: glam::DVec3,
    ) -> bool {
        // Vanilla `GameEventTags.SHRIEKER_CAN_LISTEN`: a shrieker only listens to
        // `sculk_sensor_tendrils_clicking` emitted by an activated sculk sensor.
        if event.key != SCULK_SENSOR_TENDRILS_CLICKING.key {
            return false;
        }
        if is_vibration_occluded(
            world,
            source_pos,
            block_center(self.position),
        ) {
            return false;
        }
        let Some(block_entity) = world.get_block_entity(self.position) else {
            return false;
        };
        let Some(shrieker) = block_entity.downcast_ref::<SculkShriekerBlockEntity>() else {
            return false;
        };
        let player = context
            .source_entity()
            .and_then(SculkShriekerBlockEntity::try_get_player);
        shrieker.try_shriek(world, self.position, player.as_ref())
    }
}

impl BlockEntity for SculkShriekerBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn game_event_listener(&self) -> Option<SharedGameEventListener> {
        Some(Arc::new(SculkShriekerListener {
            position: self.base.pos(),
        }))
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let view: simdnbt::borrow::NbtCompound<'_, '_> = nbt.into();
        *self.warning_level.lock() = view.int("warning_level").unwrap_or(0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        nbt.insert("warning_level", *self.warning_level.lock());
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use simdnbt::borrow::read_compound as read_borrowed_compound;
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
    fn the_warning_level_saturates_at_the_vanilla_maximum() {
        init_vanilla_registry();
        let shrieker = SculkShriekerBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::SCULK_SHRIEKER.default_state(),
        );
        shrieker.set_warning_level(2);
        shrieker.set_warning_level(shrieker.warning_level() + 5);
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
        let mut nbt = NbtCompound::new();
        shrieker.save_additional(&mut nbt);
        assert_eq!(nbt.int("warning_level"), Some(3));

        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let borrowed = read_borrowed_compound(&mut Cursor::new(bytes.as_slice()))
            .expect("test NBT should reborrow");
        shrieker.set_warning_level(0);
        shrieker.load_additional(&borrowed);
        assert_eq!(shrieker.warning_level(), 3, "the level survives a save");
    }
}
