//! Copper golem statue block entity.
//!
//! Vanilla `CopperGolemStatueBlockEntity` keeps nothing beyond the stored data
//! components (`CUSTOM_NAME`, written by [`CopperGolemStatueBlockEntity::create_statue`]).
//! The block entity exists so a statue remembers the golem's name and hands it
//! back when an unwaxed, unaffected statue is scraped into a live golem.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::BaseNbtCompound as BorrowedNbtCompound;
use simdnbt::owned::NbtCompound;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::BlockStateProperties;
use steel_registry::data_components::vanilla_components::CUSTOM_NAME;
use steel_registry::vanilla_block_entity_types;
use steel_registry::vanilla_entities;
use steel_utils::Direction;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::entity::{ENTITIES, Entity, SharedEntity, next_entity_id};
use crate::world::World;

/// Vanilla `CopperGolemStatueBlockEntity`.
pub struct CopperGolemStatueBlockEntity {
    base: Arc<BlockEntityBase>,
}

// SAFETY: This key is owned by Steel and uniquely identifies this type.
unsafe impl DowncastType for CopperGolemStatueBlockEntity {
    const TYPE_KEY: DowncastTypeKey =
        DowncastTypeKey::new("steel:block_entity/copper_golem_statue");
}

impl CopperGolemStatueBlockEntity {
    /// Creates a new block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(
                &vanilla_block_entity_types::COPPER_GOLEM_STATUE,
                level,
                pos,
                state,
            )),
        }
    }

    /// Vanilla `CopperGolemStatueBlockEntity.createStatue`: stores the golem's
    /// custom name so a later `removeStatue` can restore it.
    pub fn create_statue(&self, copper_golem: &dyn Entity) {
        let mut components = self.base.stored_components();
        // `ComponentMap::set` removes the entry when handed `None`, which is
        // exactly what vanilla's `remove` branch does.
        let name = copper_golem.base().custom_name();
        components.set(CUSTOM_NAME, name);
        self.base.set_components(components);
        self.base.set_changed();
    }

    /// Vanilla `CopperGolemStatueBlockEntity.removeStatue`: turns the statue back
    /// into a copper golem at the statue's position and facing. Returns `None`
    /// when vanilla would get a `null` entity from `EntityType.create`.
    pub fn remove_statue(&self, state: BlockStateId) -> Option<SharedEntity> {
        let world = self.base.level()?;
        let golem = ENTITIES.create(
            &vanilla_entities::COPPER_GOLEM,
            next_entity_id(),
            self.golem_position(),
            Arc::downgrade(&world),
        )?;

        if let Some(name) = self.base.stored_components().get(CUSTOM_NAME) {
            golem.set_custom_name(Some(name));
        }
        Some(self.init_copper_golem(state, golem))
    }

    /// Position vanilla `snapTo` uses: block center in x/z, block bottom in y.
    fn golem_position(&self) -> DVec3 {
        let pos = self.base.pos();
        DVec3::new(
            f64::from(pos.x()) + 0.5,
            f64::from(pos.y()),
            f64::from(pos.z()) + 0.5,
        )
    }

    /// Vanilla `CopperGolemStatueBlockEntity.initCopperGolem`.
    fn init_copper_golem(&self, state: BlockStateId, golem: SharedEntity) -> SharedEntity {
        let facing = state.get_value(&BlockStateProperties::HORIZONTAL_FACING);
        let y_rot = direction_to_y_rot(facing);
        golem.snap_to(self.golem_position(), y_rot, 0.0);
        if let Some(living) = golem.as_living_entity() {
            living.set_y_head_rot(y_rot);
            living.set_y_body_rot(y_rot);
        }
        golem.play_sound(
            &steel_registry::sound_events::ENTITY_COPPER_GOLEM_SPAWN,
            1.0,
            1.0,
        );
        golem
    }
}

impl BlockEntity for CopperGolemStatueBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn load_additional(&self, _nbt: &BorrowedNbtCompound<'_>) {}

    fn save_additional(&self, _nbt: &mut NbtCompound) {}
}

/// Vanilla `Direction.toYRot`.
const fn direction_to_y_rot(direction: Direction) -> f32 {
    match direction {
        Direction::South => 0.0,
        Direction::West => 90.0,
        Direction::North => 180.0,
        Direction::East => 270.0,
        Direction::Down | Direction::Up => 0.0,
    }
}
