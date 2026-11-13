//! Beehive block entity implementation.

use std::sync::Weak;

use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::{NbtCompound, NbtList};
use steel_registry::{vanilla_block_entity_types, vanilla_entities};
use steel_utils::nbt::NbtNumeric as _;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex};

use std::sync::Arc;

use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::{BlockStateProperties, Direction};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::entity::{
    Entity as _, EntitySpawnPlacement, EntitySpawnReason, EntitySpawnRequest, Mob as _,
    SharedEntity, spawn_entity,
};
use crate::player::Player;
use crate::world::World;

/// Maximum number of occupants in a vanilla beehive.
pub const BEEHIVE_MAX_OCCUPANTS: usize = 3;
/// Minimum occupation time for bees without nectar.
pub const BEEHIVE_MIN_OCCUPATION_TICKS_NECTARLESS: i32 = 600;
/// Radius in which a hive angers bees that are already outside it.
const BEEHIVE_ANGER_RADIUS: f64 = 8.0;

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
/// Currently stores and persists occupants for worldgen bee nests. Full vanilla
/// occupant ticking/release is blocked on bee entity support.
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

    /// Vanilla `BeehiveBlockEntity.releaseAllOccupants`: spawns every stored bee
    /// just outside the hive and clears it.
    pub fn release_all_bees(&self, world: &Arc<World>, pos: BlockPos) {
        let released = {
            let mut guard = self.state.lock();
            std::mem::take(&mut guard.stored)
        };
        if released.is_empty() {
            return;
        }
        BlockEntity::set_changed(self);

        // Vanilla `BeehiveBlockEntity.releaseOccupant` offsets each bee by the
        // hive's facing direction so they do not all spawn inside the block.
        let facing = self.state_facing();
        let offset = match facing {
            Direction::North => (0.0, 0.0, -0.5),
            Direction::South => (0.0, 0.0, 0.5),
            Direction::West => (-0.5, 0.0, 0.0),
            Direction::East => (0.5, 0.0, 0.0),
            _ => (0.0, 0.0, 0.0),
        };

        for _ in released {
            let request = EntitySpawnRequest {
                entity_type: &vanilla_entities::BEE,
                placement: EntitySpawnPlacement::Block {
                    pos,
                    try_move_down: false,
                    moved_up: false,
                },
                reason: EntitySpawnReason::Beehive,
                finalize_spawn: true,
                play_ambient_sound: false,
                item_stack: None,
                user_is_operator: false,
            };
            let Ok(entity) = spawn_entity(world, request) else {
                continue;
            };
            // Vanilla `BeehiveBlockEntity.releaseOccupant` nudges the bee out of
            // the hive block along the hive's facing.
            let position = entity.position() + glam::DVec3::new(offset.0, offset.1, offset.2);
            entity.base().set_position_local(position);
        }
    }

    /// Vanilla `BeehiveBlockEntity.angerBees`: every released bee targets the
    /// player that harvested without smoke.
    pub fn anger_bees(&self, world: &Arc<World>, player: &SharedEntity) {
        let released = {
            let mut guard = self.state.lock();
            std::mem::take(&mut guard.stored)
        };
        if released.is_empty() {
            // Vanilla still aggros bees that are already outside and bound here.
            self.anger_nearby_bees(world, player);
            return;
        }
        BlockEntity::set_changed(self);

        for _ in released {
            let request = EntitySpawnRequest {
                entity_type: &vanilla_entities::BEE,
                placement: EntitySpawnPlacement::Block {
                    pos: self.get_block_pos(),
                    try_move_down: false,
                    moved_up: false,
                },
                reason: EntitySpawnReason::Beehive,
                finalize_spawn: true,
                play_ambient_sound: false,
                item_stack: None,
                user_is_operator: false,
            };
            let Ok(entity) = spawn_entity(world, request) else {
                continue;
            };
            // Vanilla marks the released bee angry at the harvester.
            Self::anger_bee(&entity, player);
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
        let aabb = steel_utils::geometry::WorldAabb::new(
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
        world.set_block(pos, state, steel_utils::types::UpdateFlags::UPDATE_ALL);
    }

    /// Reads the hive's facing from its current block state.
    fn state_facing(&self) -> Direction {
        self.get_block_state()
            .try_get_value(&BlockStateProperties::HORIZONTAL_FACING)
            .unwrap_or(Direction::North)
    }
}

impl BlockEntity for BeehiveBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
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
