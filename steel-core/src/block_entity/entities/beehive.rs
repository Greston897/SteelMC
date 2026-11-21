//! Beehive block entity implementation.

use std::mem::take;
use std::sync::Weak;

use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::{NbtCompound, NbtList};
use steel_registry::vanilla_entity_type_tags::EntityTypeTag;
use steel_registry::{vanilla_block_entity_types, vanilla_entities};
use steel_utils::geometry::WorldAabb;
use steel_utils::nbt::NbtNumeric as _;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex};

use std::sync::Arc;

use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::{
    BlockStateProperties, Direction, EnumProperty, IntProperty,
};

use crate::behavior::{BLOCK_BEHAVIORS, BlockCollisionContext};
use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::entity::{EntitySpawnReason, SharedEntity, load_entity_recursive_owned};
use crate::world::{LevelReader as _, World};

/// Maximum number of occupants in a vanilla beehive.
pub const BEEHIVE_MAX_OCCUPANTS: usize = 3;
/// Minimum occupation time for bees without nectar.
pub const BEEHIVE_MIN_OCCUPATION_TICKS_NECTARLESS: i32 = 600;
/// Radius in which a hive angers bees that are already outside it.
const BEEHIVE_ANGER_RADIUS: f64 = 8.0;
/// Vanilla `BeehiveBlock.MAX_HONEY_LEVEL`.
const MAX_HONEY_LEVEL: u8 = 5;
/// Vanilla `BeehiveBlock.HORIZONTAL_FACING`.
const FACING: &EnumProperty<Direction> = &BlockStateProperties::HORIZONTAL_FACING;
/// Vanilla `BeehiveBlock.HONEY_LEVEL`.
const LEVEL_HONEY: &IntProperty = &BlockStateProperties::LEVEL_HONEY;

struct BeeOccupant {
    entity_data: NbtCompound,
    ticks_in_hive: i32,
    min_ticks_in_hive: i32,
}

impl BeeOccupant {
    fn worldgen(ticks_in_hive: i32) -> Self {
        Self {
            entity_data: default_bee_entity_data(),
            ticks_in_hive,
            min_ticks_in_hive: BEEHIVE_MIN_OCCUPATION_TICKS_NECTARLESS,
        }
    }

    fn load(nbt: NbtCompoundView<'_, '_>) -> Self {
        let entity_data = nbt
            .compound("entity_data")
            .map_or_else(default_bee_entity_data, |entity_data| {
                entity_data.to_owned()
            });
        let ticks_in_hive = nbt
            .get("ticks_in_hive")
            .and_then(|tag| tag.codec_i32())
            .unwrap_or(0);
        let min_ticks_in_hive = nbt
            .get("min_ticks_in_hive")
            .and_then(|tag| tag.codec_i32())
            .unwrap_or(BEEHIVE_MIN_OCCUPATION_TICKS_NECTARLESS);

        Self {
            entity_data,
            ticks_in_hive,
            min_ticks_in_hive,
        }
    }

    /// Vanilla checks the previous tick count using a strict greater-than.
    const fn tick_and_ready(&mut self) -> bool {
        let ready = self.ticks_in_hive > self.min_ticks_in_hive;
        self.ticks_in_hive = self.ticks_in_hive.wrapping_add(1);
        ready
    }

    fn has_nectar(&self) -> bool {
        self.entity_data
            .byte("HasNectar")
            .is_some_and(|value| value != 0)
    }

    fn save(&self) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        nbt.insert("entity_data", self.entity_data.clone());
        nbt.insert("ticks_in_hive", self.ticks_in_hive);
        nbt.insert("min_ticks_in_hive", self.min_ticks_in_hive);
        nbt
    }
}

fn default_bee_entity_data() -> NbtCompound {
    let mut entity_data = NbtCompound::new();
    entity_data.insert("id", vanilla_entities::BEE.key.to_string());
    entity_data
}

struct BeehiveState {
    stored: Vec<BeeOccupant>,
}

impl BeehiveState {
    fn push_occupant(&mut self, occupant: BeeOccupant) -> bool {
        if self.stored.len() >= BEEHIVE_MAX_OCCUPANTS {
            return false;
        }

        self.stored.push(occupant);
        true
    }
}

/// Beehive and bee nest block entity.
///
/// Stores and ticks occupants. Release remains incomplete: saved bee data,
/// environmental attributes and several bee-specific behaviours are not yet
/// fully restored or simulated.
pub struct BeehiveBlockEntity {
    base: BlockEntityBase,
    state: SyncMutex<BeehiveState>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `BeehiveBlockEntity`.
unsafe impl DowncastType for BeehiveBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/beehive");
}

impl BeehiveBlockEntity {
    /// Creates a new beehive block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self {
            base: BlockEntityBase::new(&vanilla_block_entity_types::BEEHIVE, level, pos, state),
            state: SyncMutex::new(BeehiveState { stored: Vec::new() }),
        }
    }

    /// Stores a vanilla worldgen bee occupant.
    ///
    /// Mirrors `BeehiveBlockEntity.Occupant.create(ticksInHive)`.
    pub fn store_worldgen_bee(&self, ticks_in_hive: i32) {
        let stored = {
            self.state
                .lock()
                .push_occupant(BeeOccupant::worldgen(ticks_in_hive))
        };
        if stored {
            BlockEntity::set_changed(self);
        }
    }

    /// Returns the number of stored occupants.
    #[must_use]
    pub fn occupant_count(&self) -> usize {
        self.state.lock().stored.len()
    }

    /// Returns whether the hive currently stores no occupants.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.state.lock().stored.is_empty()
    }

    /// Releases stored bees, retaining occupants that could not be spawned.
    pub fn release_all_bees(&self, world: &Arc<World>, pos: BlockPos) {
        let facing = self.state_facing();
        let changed = {
            let mut guard = self.state.lock();
            let before = guard.stored.len();
            guard.stored.retain(|occupant| {
                Self::spawn_released_bee(world, pos, facing, occupant, true).is_none()
            });
            guard.stored.len() != before
        };
        if changed {
            BlockEntity::set_changed(self);
        }
    }

    /// Vanilla `BeehiveBlockEntity.angerBees`: every released bee targets the
    /// player that harvested without smoke.
    pub fn anger_bees(&self, world: &Arc<World>, player: &SharedEntity) {
        let pos = self.get_block_pos();
        let facing = self.state_facing();
        let had_occupants = !self.is_empty();
        if had_occupants {
            self.anger_nearby_bees(world, player);
        }
        let changed = {
            let mut guard = self.state.lock();
            let before = guard.stored.len();
            guard.stored.retain(|occupant| {
                let Some(entity) = Self::spawn_released_bee(world, pos, facing, occupant, true)
                else {
                    return true;
                };
                if entity.position().distance_squared(player.position()) <= 16.0 {
                    Self::anger_bee(&entity, player);
                }
                false
            });
            guard.stored.len() != before
        };
        if changed {
            BlockEntity::set_changed(self);
        }
    }

    /// Points one bee at its harvester through the shared mob target slot.
    fn anger_bee(bee: &SharedEntity, harvester: &SharedEntity) {
        let Some(mob) = bee.as_mob() else {
            return;
        };
        mob.set_target(Some(harvester));
    }

    /// Vanilla `BeehiveBlockEntity.angerBees` fallback for bees already outside.
    fn anger_nearby_bees(&self, world: &Arc<World>, player: &SharedEntity) {
        let pos = self.get_block_pos();
        let aabb = WorldAabb::new(
            f64::from(pos.x()) - BEEHIVE_ANGER_RADIUS,
            f64::from(pos.y()) - BEEHIVE_ANGER_RADIUS,
            f64::from(pos.z()) - BEEHIVE_ANGER_RADIUS,
            f64::from(pos.x()) + BEEHIVE_ANGER_RADIUS,
            f64::from(pos.y()) + BEEHIVE_ANGER_RADIUS,
            f64::from(pos.z()) + BEEHIVE_ANGER_RADIUS,
        );
        for entity in world.get_entities_in_aabb(&aabb) {
            if entity.entity_type() == &vanilla_entities::BEE {
                Self::anger_bee(&entity, player);
            }
        }
    }

    /// Vanilla `BeehiveBlockEntity.emptyAllLivingFromHive` for honey harvesting:
    /// releases the bees and updates the block's honey level.
    pub fn on_honey_harvested(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        new_honey_level: i32,
        release_bees: bool,
    ) {
        if release_bees {
            self.release_all_bees(world, pos);
        }
        let state = state.set_value(
            &BlockStateProperties::LEVEL_HONEY,
            u8::try_from(new_honey_level).unwrap_or(0),
        );
        world.set_block(pos, state, UpdateFlags::UPDATE_ALL);
    }

    /// Reads the hive's facing from its current block state.
    fn state_facing(&self) -> Direction {
        self.get_block_state()
            .try_get_value(&BlockStateProperties::HORIZONTAL_FACING)
            .unwrap_or(Direction::North)
    }
}

impl BeehiveBlockEntity {
    /// Vanilla `BeehiveBlockEntity.serverTick`.
    ///
    /// Every tick the hive reports whether it still has work: bees that have
    /// stayed long enough leave through the front face, unless it is night, it
    /// is raining, or the exit is blocked. This is also what lets a worldgen
    /// nest release its bees once morning arrives.
    fn tick_occupants(&self, world: &Arc<World>, pos: BlockPos) {
        let state = self.base.block_state();
        let front = state.get_value(FACING);
        let exit_blocked = {
            let exit_pos = pos.relative(front);
            let exit_state = world.get_block_state(exit_pos);
            !BLOCK_BEHAVIORS
                .get_behavior(exit_state.get_block())
                .get_collision_shape(
                    exit_state,
                    world.as_ref(),
                    exit_pos,
                    BlockCollisionContext::empty(),
                )
                .is_empty()
        };

        let can_release = world.is_day() && !world.is_raining();

        let honey_level = state.get_value(LEVEL_HONEY);
        let mut released_any = false;
        let mut honey_gain: u8 = 0;

        {
            let mut guard = self.state.lock();
            if guard.stored.is_empty() {
                return;
            }

            let mut remaining = Vec::with_capacity(guard.stored.len());
            for mut occupant in take(&mut guard.stored) {
                let ready = occupant.tick_and_ready() && can_release && !exit_blocked;
                if !ready {
                    remaining.push(occupant);
                    continue;
                }

                // Only nectar-bearing occupants deliver honey. Vanilla gives
                // a 1% chance of two levels, capped at five.
                if Self::spawn_released_bee(world, pos, front, &occupant, false).is_some() {
                    released_any = true;
                    if occupant.has_nectar()
                        && honey_level.saturating_add(honey_gain) < MAX_HONEY_LEVEL
                    {
                        honey_gain += if rand::random_range(0..100) == 0 {
                            2
                        } else {
                            1
                        };
                    }
                } else {
                    // A failed spawn keeps the bee inside, as vanilla does.
                    remaining.push(occupant);
                }
            }
            guard.stored = remaining;
        }

        if !released_any {
            return;
        }
        BlockEntity::set_changed(self);

        let new_level = honey_level.saturating_add(honey_gain).min(MAX_HONEY_LEVEL);
        if new_level != honey_level {
            let updated = state.set_value(LEVEL_HONEY, new_level);
            world.set_block(pos, updated, UpdateFlags::UPDATE_CLIENTS);
        }
    }

    /// Load sanitized occupant data before publishing the entity to the world.
    fn spawn_released_bee(
        world: &Arc<World>,
        pos: BlockPos,
        facing: Direction,
        occupant: &BeeOccupant,
        emergency: bool,
    ) -> Option<SharedEntity> {
        let exit = pos.relative(facing);
        let exit_state = world.get_block_state(exit);
        let blocked = !BLOCK_BEHAVIORS
            .get_behavior(exit_state.get_block())
            .get_collision_shape(
                exit_state,
                world.as_ref(),
                exit,
                BlockCollisionContext::empty(),
            )
            .is_empty();
        if blocked && !emergency {
            return None;
        }

        let mut data = occupant.entity_data.clone();
        // Vanilla deliberately discards position, UUID, passengers and transient
        // state before reconstructing a hive occupant.
        for tag in [
            "Air",
            "drop_chances",
            "equipment",
            "Brain",
            "CanPickUpLoot",
            "DeathTime",
            "fall_distance",
            "FallFlying",
            "Fire",
            "HurtTime",
            "LeftHanded",
            "Motion",
            "NoGravity",
            "OnGround",
            "PortalCooldown",
            "Pos",
            "Rotation",
            "sleeping_pos",
            "CannotEnterHiveTicks",
            "TicksSincePollination",
            "CropsGrownSincePollination",
            "hive_pos",
            "Passengers",
            "leash",
            "UUID",
        ] {
            while data.remove(tag).is_some() {}
        }
        let loaded = load_entity_recursive_owned(world, &data, EntitySpawnReason::Load, |_| {})?;
        let entity = &loaded.root;
        if !entity
            .entity_type()
            .has_tag(&EntityTypeTag::BEEHIVE_INHABITORS)
        {
            return None;
        }
        entity.set_no_gravity(true);
        if let Some(animal) = entity.as_animal() {
            if !animal.is_age_locked() {
                let age = animal.get_age();
                let ticks = occupant.ticks_in_hive;
                let updated = if age < 0 {
                    age.saturating_add(ticks).min(0)
                } else {
                    age.saturating_sub(ticks).max(0)
                };
                animal.set_age(updated);
            }
            animal.set_in_love_time(
                animal
                    .in_love_time()
                    .saturating_sub(occupant.ticks_in_hive)
                    .max(0),
            );
        }
        let dimensions = entity.base().dimensions();
        let distance = if blocked {
            0.0
        } else {
            0.55 + f64::from(dimensions.width / 2.0)
        };
        let (dx, _, dz) = facing.offset();
        let position = glam::DVec3::new(
            f64::from(pos.x()) + 0.5 + distance * f64::from(dx),
            f64::from(pos.y()) + 0.5 - f64::from(dimensions.height / 2.0),
            f64::from(pos.z()) + 0.5 + distance * f64::from(dz),
        );
        let (yaw, pitch) = entity.rotation();
        entity.snap_to(position, yaw, pitch);
        world.try_add_entity(Arc::clone(entity)).ok()?;
        Some(Arc::clone(entity))
    }
}

impl BlockEntity for BeehiveBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    /// Vanilla `BeehiveBlockEntity.serverTick` entry point.
    fn tick(&self, world: &Arc<World>) {
        self.tick_occupants(world, self.get_block_pos());
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let nbt: NbtCompoundView<'_, '_> = nbt.into();
        let mut stored = Vec::new();

        if let Some(bees) = nbt.list("bees")
            && let Some(compounds) = bees.compounds()
        {
            for compound in compounds {
                if stored.len() >= BEEHIVE_MAX_OCCUPANTS {
                    break;
                }
                stored.push(BeeOccupant::load(compound));
            }
        }

        self.state.lock().stored = stored;
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let bees = self
            .state
            .lock()
            .stored
            .iter()
            .map(BeeOccupant::save)
            .collect::<Vec<_>>();
        nbt.insert("bees", NbtList::Compound(bees));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bootstrap::init_globals;
    use crate::test_support::{fresh_test_world, insert_ready_full_chunk};
    use steel_registry::vanilla_blocks;
    use steel_utils::ChunkPos;
    use text_components::TextComponent;

    #[test]
    fn released_bee_restores_persistent_data_and_ages_in_the_hive() {
        init_globals();
        let fixture = fresh_test_world("hive_restored_bee");
        let world = &fixture.world;
        let pos = BlockPos::new(8, 64, 8);
        let _holder = insert_ready_full_chunk(world, ChunkPos::from_block_pos(pos));
        let mut bee = BeeOccupant::worldgen(100);
        let name = TextComponent::plain("Pollinator");
        bee.entity_data.insert("CustomName", name.to_codec_nbt());
        bee.entity_data.insert("Age", -1000);
        bee.entity_data.insert("InLove", 2000);
        bee.entity_data.insert("NoGravity", 0_i8);

        let released =
            BeehiveBlockEntity::spawn_released_bee(world, pos, Direction::North, &bee, false)
                .expect("stored bee should load into the ready chunk");
        assert_eq!(released.custom_name(), Some(name));
        assert!(released.is_no_gravity());
        let animal = released.as_animal().expect("bee is an animal");
        assert_eq!(animal.get_age(), -900);
        assert_eq!(animal.in_love_time(), 1900);
        assert!(released.position().z < f64::from(pos.z()));
    }

    #[test]
    fn non_hive_entity_data_is_not_published() {
        init_globals();
        let fixture = fresh_test_world("hive_invalid_occupant");
        let world = &fixture.world;
        let pos = BlockPos::new(8, 64, 8);
        let _holder = insert_ready_full_chunk(world, ChunkPos::from_block_pos(pos));
        let mut occupant = BeeOccupant::worldgen(0);
        while occupant.entity_data.remove("id").is_some() {}
        occupant.entity_data.insert("id", "minecraft:pig");
        assert!(
            BeehiveBlockEntity::spawn_released_bee(world, pos, Direction::North, &occupant, true)
                .is_none()
        );
    }

    #[test]
    fn occupants_are_capped_at_three() {
        let mut state = BeehiveState { stored: Vec::new() };
        for _ in 0..BEEHIVE_MAX_OCCUPANTS {
            assert!(state.push_occupant(BeeOccupant::worldgen(0)));
        }
        // Vanilla `BeehiveBlockEntity.isFull` refuses the fourth bee.
        assert!(!state.push_occupant(BeeOccupant::worldgen(0)));
        assert_eq!(state.stored.len(), BEEHIVE_MAX_OCCUPANTS);
    }

    #[test]
    fn nectarless_bees_wait_until_after_the_minimum_tick_count() {
        let mut bee = BeeOccupant::worldgen(0);
        assert!(!bee.has_nectar());
        for _ in 0..=600 {
            assert!(!bee.tick_and_ready());
        }
        assert!(bee.tick_and_ready());
        bee.entity_data.insert("HasNectar", 1_i8);
        assert!(bee.has_nectar());
    }

    #[test]
    fn releasing_a_nectarless_bee_does_not_create_honey() {
        init_globals();
        let fixture = fresh_test_world("hive_nectarless_release");
        let world = &fixture.world;
        let pos = BlockPos::new(8, 64, 8);
        let _holder = insert_ready_full_chunk(world, ChunkPos::from_block_pos(pos));
        let state = vanilla_blocks::BEE_NEST.default_state();
        assert!(world.set_block(pos, state, UpdateFlags::UPDATE_ALL));
        assert!(world.is_day());
        assert!(!world.is_raining());
        let hive = BeehiveBlockEntity::new(Arc::downgrade(world), pos, state);
        hive.store_worldgen_bee(601);

        hive.tick_occupants(world, pos);
        assert_eq!(hive.occupant_count(), 0);
        assert_eq!(world.get_block_state(pos).get_value(LEVEL_HONEY), 0);
    }

    #[test]
    fn nectar_delivery_increases_honey_with_vanilla_bounds() {
        init_globals();
        let fixture = fresh_test_world("hive_nectar_delivery");
        let world = &fixture.world;
        let pos = BlockPos::new(8, 64, 8);
        let _holder = insert_ready_full_chunk(world, ChunkPos::from_block_pos(pos));
        let state = vanilla_blocks::BEE_NEST.default_state();
        assert!(world.set_block(pos, state, UpdateFlags::UPDATE_ALL));
        assert!(world.is_day());
        assert!(!world.is_raining());
        let hive = BeehiveBlockEntity::new(Arc::downgrade(world), pos, state);
        let mut bee = BeeOccupant::worldgen(601);
        bee.entity_data.insert("HasNectar", 1_i8);
        assert!(hive.state.lock().push_occupant(bee));

        hive.tick_occupants(world, pos);
        assert_eq!(hive.occupant_count(), 0);
        assert!((1..=2).contains(&world.get_block_state(pos).get_value(LEVEL_HONEY)));
    }

    #[test]
    fn partial_collision_blocks_the_normal_hive_exit() {
        init_globals();
        let fixture = fresh_test_world("hive_partial_exit");
        let world = &fixture.world;
        let pos = BlockPos::new(8, 64, 8);
        let _holder = insert_ready_full_chunk(world, ChunkPos::from_block_pos(pos));
        let state = vanilla_blocks::BEE_NEST.default_state();
        assert!(world.set_block(pos, state, UpdateFlags::UPDATE_ALL));
        let exit = pos.relative(state.get_value(FACING));
        assert!(world.set_block(
            exit,
            vanilla_blocks::OAK_SLAB.default_state(),
            UpdateFlags::UPDATE_ALL
        ));
        let hive = BeehiveBlockEntity::new(Arc::downgrade(world), pos, state);
        hive.store_worldgen_bee(601);

        hive.tick_occupants(world, pos);
        assert_eq!(hive.occupant_count(), 1);
    }
}
