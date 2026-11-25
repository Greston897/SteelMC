//! Vanilla chest minecart (`MinecartChest`): rail movement, 27-slot container
//! screen, item NBT persistence, and lazy structure loot-table unpacking.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use steel_macros::entity_behavior;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_entity_data::MinecartEntityData;
use steel_registry::vanilla_game_events;
use steel_utils::axis::Axis;
use steel_utils::block_util::FoundRectangle;
use steel_utils::locks::{IntoShared as _, Shared, SyncMutex};
use steel_utils::types::InteractionHand;
use steel_utils::{BlockStateId, DowncastType, DowncastTypeKey, Identifier};

use super::abstract_minecart::{
    AbstractMinecart, AbstractMinecartBase, minecart_relative_portal_position,
};
use super::vehicle_entity::{VEHICLE_DIMENSION_CHANGING_DELAY, vehicle_ignores_explosion};
use crate::behavior::InteractionResult;
use crate::block_entity::randomizable_container::RandomizableContainer;
use crate::entity::{Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, SharedEntity};
use crate::inventory::container::{Container as _, SimpleContainer, load_all_items};
use crate::inventory::menu::kinds::chest_boat;
use crate::player::Player;
use crate::world::{Explosion, World};

/// Vanilla `MinecartChest.getContainerSize()` (3 rows of 9).
const CHEST_MINECART_SLOTS: usize = 27;

/// Vanilla chest minecart with a 27-slot inventory, rail movement, and lazy
/// structure loot-table unpacking (used by abandoned mineshafts and survival
/// rail placement).
#[entity_behavior(class = "MinecartChest")]
pub struct ChestMinecartEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<MinecartEntityData>,
    minecart_base: AbstractMinecartBase,
    inventory: Shared<SimpleContainer>,
    loot: SyncMutex<RandomizableContainer>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `ChestMinecartEntity`.
unsafe impl DowncastType for ChestMinecartEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/chest_minecart");
}

impl ChestMinecartEntity {
    /// Creates a new chest minecart entity.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            entity_data: SyncMutex::new(MinecartEntityData::new()),
            minecart_base: AbstractMinecartBase::new(),
            inventory: SimpleContainer::new(CHEST_MINECART_SLOTS).into_shared(),
            loot: SyncMutex::new(RandomizableContainer::none()),
        }
    }

    /// Creates a chest minecart entity from saved data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(MinecartEntityData::new()),
            minecart_base: AbstractMinecartBase::new(),
            inventory: SimpleContainer::new(CHEST_MINECART_SLOTS).into_shared(),
            loot: SyncMutex::new(RandomizableContainer::none()),
        }
    }

    /// Sets the deferred loot table used when the container is first opened.
    pub fn set_loot_table(&self, loot_table: Identifier, seed: i64) {
        self.loot.lock().set_loot_table(Some(loot_table), seed);
    }

    /// Unpacks any deferred structure loot table into the 27-slot inventory.
    pub fn unpack_loot(&self) -> bool {
        if !self.loot.lock().has_loot_table() {
            return false;
        }
        let world_seed = self.level().map_or(0, |world| world.seed());
        let mut container = self.inventory.lock();
        let mut loot = self.loot.lock();
        loot.unpack_loot_table(container.items_mut(), world_seed)
    }

    /// Shared handle to the 27-slot inventory container.
    #[must_use]
    pub fn container(&self) -> Shared<SimpleContainer> {
        Arc::clone(&self.inventory)
    }
}

impl Entity for ChestMinecartEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn movement_emission(&self) -> EntityMovementEmission {
        EntityMovementEmission::Events
    }

    fn is_pickable(&self) -> bool {
        !self.is_removed()
    }

    fn is_pushable(&self) -> bool {
        true
    }

    fn blocks_building(&self) -> bool {
        true
    }

    fn ignore_explosion(&self, explosion: &dyn Explosion) -> bool {
        vehicle_ignores_explosion(explosion)
    }

    fn dimension_changing_delay(&self) -> i32 {
        VEHICLE_DIMENSION_CHANGING_DELAY
    }

    fn get_relative_portal_position(&self, axis: Axis, portal_area: FoundRectangle) -> DVec3 {
        minecart_relative_portal_position(self, axis, portal_area)
    }

    fn interact(
        &self,
        player: &Player,
        _hand: InteractionHand,
        _location: DVec3,
    ) -> InteractionResult {
        let Some(world) = self.level() else {
            return InteractionResult::Pass;
        };
        let Some(shared) = world.get_entity_by_id(self.id()) else {
            return InteractionResult::Pass;
        };
        let _ = self.unpack_loot();
        let owner: Weak<dyn Entity> = Arc::downgrade(&shared);
        drop(shared);
        let player_inventory = Arc::clone(&player.inventory);
        let container = Arc::clone(&self.inventory);
        player.open_menu(self.display_name(), move |context| {
            chest_boat(player_inventory, context.container_id, container, owner)
        });
        self.game_event_with_source_entity(
            &vanilla_game_events::CONTAINER_OPEN,
            Some(player as &dyn Entity),
        );
        InteractionResult::Success
    }

    fn tick(self: Arc<Self>) {
        if let Some(minecart) = self.as_abstract_minecart() {
            minecart.tick_minecart(&(Arc::clone(&self) as SharedEntity));
        }
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_abstract_minecart_data(nbt);
        let container = self.inventory.lock();
        let loot = self.loot.lock();
        if loot.has_loot_table() {
            loot.save_to_nbt(nbt);
        } else {
            let mut items: Vec<NbtCompound> = Vec::new();
            for (slot, stack) in container.items().iter().enumerate() {
                if !stack.is_empty()
                    && let NbtTag::Compound(mut item_nbt) = stack.to_nbt_tag_ref()
                {
                    item_nbt.insert("Slot", i8::try_from(slot).unwrap_or(0));
                    items.push(item_nbt);
                }
            }
            if !items.is_empty() {
                nbt.insert("Items", NbtList::Compound(items));
            }
        }
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_abstract_minecart_data(nbt);
        RandomizableContainer::load_from_nbt(nbt, &self.loot);
        let mut container = self.inventory.lock();
        container.items_mut().fill(ItemStack::empty());
        if !self.loot.lock().has_loot_table() {
            load_all_items(nbt, container.items_mut());
        }
    }
}

impl AbstractMinecart for ChestMinecartEntity {
    fn minecart_base(&self) -> &AbstractMinecartBase {
        &self.minecart_base
    }

    fn is_rideable(&self) -> bool {
        false
    }

    fn custom_display_block(&self) -> Option<BlockStateId> {
        *self
            .entity_data
            .lock()
            .abstract_minecart()
            .id_custom_display_block
            .get()
    }

    fn set_custom_display_block(&self, value: Option<BlockStateId>) {
        self.entity_data
            .lock()
            .abstract_minecart_mut()
            .id_custom_display_block
            .set(value);
    }

    fn display_offset(&self) -> i32 {
        *self
            .entity_data
            .lock()
            .abstract_minecart()
            .id_display_offset
            .get()
    }

    fn set_display_offset(&self, value: i32) {
        self.entity_data
            .lock()
            .abstract_minecart_mut()
            .id_display_offset
            .set(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use steel_registry::vanilla_entities;
    use steel_utils::BlockPos;

    #[test]
    fn chest_minecart_saves_structure_loot_table_state() {
        let minecart = ChestMinecartEntity::new(
            &vanilla_entities::CHEST_MINECART,
            1,
            DVec3::new(1.5, 2.5, 3.5),
            Weak::new(),
        );
        minecart.set_loot_table(
            Identifier::new_static("minecraft", "chests/abandoned_mineshaft"),
            42,
        );

        let mut nbt = NbtCompound::new();
        minecart.save_additional(&mut nbt);

        assert_eq!(
            nbt.string("LootTable").map(ToString::to_string),
            Some("minecraft:chests/abandoned_mineshaft".to_owned())
        );
        assert_eq!(nbt.long("LootTableSeed"), Some(42));
        assert_eq!(nbt.byte("HasTicked"), Some(1));
        assert_eq!(nbt.byte("FlippedRotation"), Some(0));
    }

    #[test]
    fn chest_minecart_is_pickable_and_pushable_like_vanilla() {
        let minecart = ChestMinecartEntity::new(
            &vanilla_entities::CHEST_MINECART,
            1,
            DVec3::new(1.5, 2.5, 3.5),
            Weak::new(),
        );

        assert!(minecart.is_pickable());
        assert!(minecart.is_pushable());
        assert!(minecart.blocks_building());
    }

    #[test]
    fn chest_minecart_relative_portal_position_resets_forward_offset() {
        let minecart = ChestMinecartEntity::new(
            &vanilla_entities::CHEST_MINECART,
            1,
            DVec3::new(12.0, 66.0, 20.75),
            Weak::new(),
        );
        let portal_area = FoundRectangle {
            min_corner: BlockPos::new(10, 64, 20),
            axis1_size: 4,
            axis2_size: 5,
        };

        assert!(
            minecart
                .get_relative_portal_position(Axis::X, portal_area)
                .z
                .abs()
                < f64::EPSILON
        );
    }
}
