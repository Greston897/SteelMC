//! Block behaviors for the remaining block-entity containers and the shelf.
//!
//! Each of these blocks defers almost everything to its block entity: the
//! behavior only supplies the placement state, the menu (when the block stores
//! items), and the block entity factory. The `open` sound and state changes are
//! driven by the block entity where vanilla does that.

use std::sync::{Arc, Weak};

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, Direction, FrontAndTop};
use steel_registry::vanilla_block_entity_types;
use glam::DVec3;
use steel_registry::vanilla_entities;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_items;
use steel_utils::{BlockPos, BlockStateId, Downcast as _};
use text_components::TextComponent;

use crate::behavior::InventoryAccess;
use crate::behavior::block::{BlockBehavior, BlockEntityCreation};
use crate::block_entity::BlockEntityTicker;
use crate::block_entity::entities::crafter::CrafterBlockEntity;
use crate::behavior::context::{BlockHitResult, BlockPlaceContext, InteractionResult};
use crate::block_entity::BLOCK_ENTITIES;
use crate::inventory::container::calculate_redstone_signal_from_container;
use crate::inventory::lock::{ContainerLockGuard, ContainerRef};
use crate::inventory::menu::kinds::{chest, crafter};
use crate::entity::SharedEntity;
use crate::entity::entities::{
    ArrowEntity, EnderPearlEntity, ExperienceBottleEntity, FireworkRocketEntity, ItemEntity,
    LingeringPotionEntity, SnowballEntity, SpectralArrowEntity, SplashPotionEntity,
    ThrownEggEntity, next_entity_id,
};
use crate::player::Player;
use crate::world::{SignalGetter, World};


const HOPPER_FACING: &EnumProperty<Direction> = &BlockStateProperties::FACING_HOPPER;

/// Vanilla `DispenseItemBehavior` launch speed for dispensed projectiles.
const DISPENSE_POWER: f64 = 1.1;

/// Maps a dispensed item to the projectile entity vanilla fires for it.
///
/// Items without a `ProjectileItem` implementation return `None`, which makes
/// `dispense_from` drop them instead.
fn dispense_projectile(stack: &ItemStack, world: &Arc<World>, pos: DVec3) -> Option<SharedEntity> {
    let kind = stack.item();
    let weak = Arc::downgrade(world);
    let id = next_entity_id();

    let entity: SharedEntity = if kind == &vanilla_items::ARROW {
        let arrow = Arc::new(ArrowEntity::new(&vanilla_entities::ARROW, id, pos, weak));
        arrow.set_arrow_item(stack.copy_with_count(1));
        arrow
    } else if kind == &vanilla_items::TIPPED_ARROW {
        let arrow = Arc::new(ArrowEntity::new(&vanilla_entities::ARROW, id, pos, weak));
        arrow.set_arrow_item(stack.copy_with_count(1));
        arrow
    } else if kind == &vanilla_items::SPECTRAL_ARROW {
        Arc::new(SpectralArrowEntity::new(
            &vanilla_entities::SPECTRAL_ARROW,
            id,
            pos,
            weak,
        ))
    } else if kind == &vanilla_items::SPLASH_POTION {
        Arc::new(SplashPotionEntity::new(
            &vanilla_entities::SPLASH_POTION,
            id,
            pos,
            weak,
        ))
    } else if kind == &vanilla_items::LINGERING_POTION {
        Arc::new(LingeringPotionEntity::new(
            &vanilla_entities::LINGERING_POTION,
            id,
            pos,
            weak,
        ))
    } else if kind == &vanilla_items::EXPERIENCE_BOTTLE {
        Arc::new(ExperienceBottleEntity::new(
            &vanilla_entities::EXPERIENCE_BOTTLE,
            id,
            pos,
            weak,
        ))
    } else if kind == &vanilla_items::SNOWBALL {
        Arc::new(SnowballEntity::new(
            &vanilla_entities::SNOWBALL,
            id,
            pos,
            weak,
        ))
    } else if kind == &vanilla_items::EGG {
        Arc::new(ThrownEggEntity::new(&vanilla_entities::EGG, id, pos, weak))
    } else if kind == &vanilla_items::ENDER_PEARL {
        Arc::new(EnderPearlEntity::new(
            &vanilla_entities::ENDER_PEARL,
            id,
            pos,
            weak,
        ))
    } else if kind == &vanilla_items::FIREWORK_ROCKET {
        Arc::new(FireworkRocketEntity::new(
            &vanilla_entities::FIREWORK_ROCKET,
            id,
            pos,
            weak,
        ))
    } else {
        return None;
    };
    Some(entity)
}

/// Vanilla `HopperBlock`: a five-slot container that points at a direction.
#[block_behavior]
pub struct HopperBlock {
    block: BlockRef,
}

impl HopperBlock {
    /// Creates a new hopper block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for HopperBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        // Vanilla `HopperBlock.getStateForPlacement`: a hopper points down unless
        // it is placed against a horizontal face, in which case it points into
        // that face. Sneaking control is not exposed by the placement context yet,
        // so the horizontal case always applies.
        let clicked = context.clicked_face();
        let direction = if clicked.is_horizontal() {
            clicked.opposite()
        } else {
            Direction::Down
        };
        Some(self.block.default_state().set_value(HOPPER_FACING, direction))
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::HOPPER,
            level,
            pos,
            state,
        ))
    }

    fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
        true
    }

    fn get_analog_output_signal(
        &self,
        _state: BlockStateId,
        world: &World,
        pos: BlockPos,
        _direction: Direction,
    ) -> i32 {
        container_signal(world, pos)
    }

    fn get_block_entity_ticker(
        &self,
        _world: &Arc<World>,
        _state: BlockStateId,
        block_entity_type: steel_registry::block_entity_type::BlockEntityTypeRef,
    ) -> Option<BlockEntityTicker> {
        // Vanilla `HopperBlock.getTicker`: enabled hoppers tick, disabled ones do
        // not. The state is read per tick by the world, so the enabled gate is
        // expressed by the ticker always being present for the hopper type.
        BlockEntityTicker::for_matching_entity_tick(
            block_entity_type,
            &vanilla_block_entity_types::HOPPER,
        )
    }
}

/// Vanilla `DispenserBlock`: a nine-slot container fired by redstone.
#[block_behavior]
pub struct DispenserBlock {
    block: BlockRef,
}

impl DispenserBlock {
    /// Creates a new dispenser block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

/// Vanilla `DropperBlock`: identical to a dispenser, but drops items as entities.
#[block_behavior]
pub struct DropperBlock {
    block: BlockRef,
}

impl DropperBlock {
    /// Creates a new dropper block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

macro_rules! funnel_like {
    ($name:ident, $type_const:ident, $fire:expr) => {
        impl BlockBehavior for $name {
            fn handle_neighbor_changed(
                &self,
                state: BlockStateId,
                world: &Arc<World>,
                pos: BlockPos,
                _source_block: BlockRef,
                _moved_by_piston: bool,
            ) {
                // Vanilla `DispenserBlock.neighborChanged` triggers on the rising
                // edge of a redstone signal only, so a constant signal does not
                // empty the funnel every tick.
                let powered = world.get_best_neighbor_signal(pos) > 0;
                let was_powered = state.get_value(&BlockStateProperties::TRIGGERED);
                if powered != was_powered {
                    world.set_block(
                        pos,
                        state.set_value(&BlockStateProperties::TRIGGERED, powered),
                        steel_utils::types::UpdateFlags::UPDATE_CLIENTS,
                    );
                    if powered {
                        $fire(world, pos, state);
                    }
                }
            }

            fn get_state_for_placement(
                &self,
                context: &BlockPlaceContext<'_>,
            ) -> Option<BlockStateId> {
                // Vanilla `DispenserBlock.getStateForPlacement`: the funnel faces
                // the player, i.e. away from the clicked face.
                Some(
                    self.block
                        .default_state()
                        .set_value(
                            &BlockStateProperties::FACING,
                            context.clicked_face().opposite(),
                        ),
                )
            }

            fn new_block_entity(
                &self,
                level: Weak<World>,
                pos: BlockPos,
                state: BlockStateId,
            ) -> BlockEntityCreation {
                BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
                    &vanilla_block_entity_types::$type_const,
                    level,
                    pos,
                    state,
                ))
            }

            fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
                true
            }

            fn get_analog_output_signal(
                &self,
                _state: BlockStateId,
                world: &World,
                pos: BlockPos,
                _direction: Direction,
            ) -> i32 {
                container_signal(world, pos)
            }
        }
    };
}

funnel_like!(DispenserBlock, DISPENSER, |world: &Arc<World>, pos, state| {
    dispense_from(world, pos, state);
});
funnel_like!(DropperBlock, DROPPER, |world: &Arc<World>, pos, state| {
    drop_from(world, pos, state);
});

/// Removes the first non-empty stack from the funnel at `pos`.
fn take_front_stack(world: &Arc<World>, pos: BlockPos) -> Option<ItemStack> {
    let block_entity = world.get_block_entity(pos)?;
    let container = block_entity.container_ref()?;
    let mut guard = container.lock();
    let slot = (0..guard.get_container_size()).find(|slot| !guard.get_item(*slot).is_empty())?;
    Some(guard.remove_item(slot, 1))
}

/// Vanilla dispenser launch geometry: the entity appears just past the funnel
/// face, travelling along the facing axis.
fn launch_point(pos: BlockPos, facing: Direction) -> (DVec3, DVec3) {
    let (dx, dy, dz) = facing.offset();
    let (dx, dy, dz) = (f64::from(dx), f64::from(dy), f64::from(dz));
    let spawn_pos = DVec3::new(
        f64::from(pos.x()) + 0.5 + dx * 0.7,
        f64::from(pos.y()) + 0.5 + dy * 0.7,
        f64::from(pos.z()) + 0.5 + dz * 0.7,
    );
    (spawn_pos, DVec3::new(dx, dy, dz) * DISPENSE_POWER)
}

/// Vanilla `DispenserBlock.dispenseFrom`: fires whatever the front slot holds.
///
/// Projectiles keep their vanilla spawn point (slightly ahead of the funnel) and
/// velocity; anything without a `ProjectileItem` implementation is dropped as an
/// item entity, matching `DefaultDispenseItemBehavior`.
fn dispense_from(world: &Arc<World>, pos: BlockPos, state: BlockStateId) -> bool {
    let facing = state.get_value(&BlockStateProperties::FACING);
    let Some(stack) = take_front_stack(world, pos) else {
        return false;
    };

    let (spawn_pos, velocity) = launch_point(pos, facing);
    if let Some(projectile) = dispense_projectile(&stack, world, spawn_pos) {
        projectile.set_velocity(velocity);
        let _ = world.try_add_entity(projectile);
    } else {
        spawn_dropped_item(world, spawn_pos, velocity, stack);
    }

    world.play_sound(
        &sound_events::BLOCK_DISPENSER_DISPENSE,
        crate::sound::SoundSource::Blocks,
        pos,
        1.0,
        1.0,
        None,
    );
    true
}

/// Vanilla `DropperBlock.dispenseFrom`: always drops the front stack as an item.
fn drop_from(world: &Arc<World>, pos: BlockPos, state: BlockStateId) -> bool {
    let facing = state.get_value(&BlockStateProperties::FACING);
    let Some(stack) = take_front_stack(world, pos) else {
        return false;
    };

    let (spawn_pos, velocity) = launch_point(pos, facing);
    spawn_dropped_item(world, spawn_pos, velocity, stack);
    world.play_sound(
        &sound_events::BLOCK_DISPENSER_DISPENSE,
        crate::sound::SoundSource::Blocks,
        pos,
        1.0,
        1.0,
        None,
    );
    true
}

/// Spawns an item entity that travels along `velocity`.
fn spawn_dropped_item(world: &Arc<World>, spawn_pos: DVec3, velocity: DVec3, stack: ItemStack) {
    let entity = ItemEntity::new(
        &vanilla_entities::ITEM,
        next_entity_id(),
        spawn_pos,
        Arc::downgrade(world),
    );
    entity.set_item(stack);
    entity.set_velocity(velocity);
    let _ = world.try_add_entity(Arc::new(entity));
}

/// Vanilla `BrewingStandBlock`: a four-slot container with an `has_bottle` state.
#[block_behavior]
pub struct BrewingStandBlock {
    block: BlockRef,
}

impl BrewingStandBlock {
    /// Creates a new brewing stand block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for BrewingStandBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::BREWING_STAND,
            level,
            pos,
            state,
        ))
    }

    fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
        true
    }

    fn get_analog_output_signal(
        &self,
        _state: BlockStateId,
        world: &World,
        pos: BlockPos,
        _direction: Direction,
    ) -> i32 {
        container_signal(world, pos)
    }
    fn get_block_entity_ticker(
        &self,
        _world: &Arc<World>,
        _state: BlockStateId,
        block_entity_type: steel_registry::block_entity_type::BlockEntityTypeRef,
    ) -> Option<BlockEntityTicker> {
        // Vanilla `BrewingStandBlock.getTicker`.
        BlockEntityTicker::for_matching_entity_tick(
            block_entity_type,
            &vanilla_block_entity_types::BREWING_STAND,
        )
    }
}

/// Vanilla `LecternBlock`: a one-slot book container that opens a reading menu.
#[block_behavior]
pub struct LecternBlock {
    block: BlockRef,
}

impl LecternBlock {
    /// Creates a new lectern block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for LecternBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(
            self.block
                .default_state()
                .set_value(
                    &BlockStateProperties::HORIZONTAL_FACING,
                    context.horizontal_direction().opposite(),
                ),
        )
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::LECTERN,
            level,
            pos,
            state,
        ))
    }
}

/// Vanilla `CrafterBlock`: a nine-slot grid with a slot-disable mask.
#[block_behavior]
pub struct CrafterBlock {
    block: BlockRef,
}

impl CrafterBlock {
    /// Creates a new crafter block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for CrafterBlock {
    /// Vanilla `CrafterBlock.useWithoutItem`: opens the 3x3 grid menu so the
    /// player can fill it and toggle slots.
    fn open_menu(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        _state: BlockStateId,
        player: &Player,
    ) -> InteractionResult {
        let Some(block_entity) = world.get_block_entity(pos) else {
            return InteractionResult::Pass;
        };
        if block_entity.downcast_ref::<CrafterBlockEntity>().is_none() {
            return InteractionResult::Pass;
        }
        let Some(container_ref) = ContainerRef::from_block_entity(Arc::clone(&block_entity))
        else {
            return InteractionResult::Pass;
        };
        let menu_entity = Arc::clone(&block_entity);
        let inventory = Arc::clone(&player.inventory);
        player.open_menu(
            // Vanilla titles this menu with `container.crafter`.
            TextComponent::plain("Crafter"),
            move |context| {
                crafter(
                    inventory,
                    context.container_id,
                    pos,
                    container_ref,
                    menu_entity,
                )
            },
        );
        InteractionResult::Success
    }

    /// Vanilla `CrafterBlock.getStateForPlacement`: the front face points back
    /// at the player, expressed through the `orientation` property.
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(
            self.block
                .default_state()
                .set_value(
                    &BlockStateProperties::ORIENTATION,
                    FrontAndTop::from_front_and_top(
                        context.get_nearest_looking_direction().opposite(),
                    ),
                ),
        )
    }

    /// Vanilla `CrafterBlock.neighborChanged`: a rising redstone edge starts the
    /// crafting sequence; the falling edge clears `triggered` once the counter
    /// has run out.
    fn handle_neighbor_changed(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _source_block: BlockRef,
        _moved_by_piston: bool,
    ) {
        let powered = world.get_best_neighbor_signal(pos) > 0;
        if powered {
            let Some(block_entity) = world.get_block_entity(pos) else {
                return;
            };
            let Some(crafter) = block_entity.downcast_ref::<CrafterBlockEntity>() else {
                return;
            };
            // The block entity is ticked every game tick whenever the crafter
            // block entity ticker is registered, so starting the counter here
            // is enough; `serverTick` decrements it and fires the craft.
            let _ = crafter.start_crafting(world);
        } else if state.get_value(&BlockStateProperties::TRIGGERED) {
            // The sequence finished or the signal dropped: mirror the falling
            // edge into the block state.
            world.set_block(
                pos,
                state.set_value(&BlockStateProperties::TRIGGERED, false),
                steel_utils::types::UpdateFlags::UPDATE_CLIENTS,
            );
        }
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::CRAFTER,
            level,
            pos,
            state,
        ))
    }

    fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
        true
    }

    fn get_analog_output_signal(
        &self,
        _state: BlockStateId,
        world: &World,
        pos: BlockPos,
        _direction: Direction,
    ) -> i32 {
        container_signal(world, pos)
    }
    fn get_block_entity_ticker(
        &self,
        _world: &Arc<World>,
        _state: BlockStateId,
        block_entity_type: steel_registry::block_entity_type::BlockEntityTypeRef,
    ) -> Option<BlockEntityTicker> {
        // Vanilla `CrafterBlock.getTicker`.
        BlockEntityTicker::for_matching_entity_tick(
            block_entity_type,
            &vanilla_block_entity_types::CRAFTER,
        )
    }
}

/// Vanilla `DecoratedPotBlock`: no menu, but the block entity holds sherds and a
/// loot table, and it can be shattered by projectiles.
#[block_behavior]
pub struct DecoratedPotBlock {
    block: BlockRef,
}

impl DecoratedPotBlock {
    /// Creates a new decorated pot block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for DecoratedPotBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(
            self.block
                .default_state()
                .set_value(
                    &BlockStateProperties::HORIZONTAL_FACING,
                    context.horizontal_direction().opposite(),
                ),
        )
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::DECORATED_POT,
            level,
            pos,
            state,
        ))
    }
}

/// Vanilla `ShelfBlock`: three display slots and a redstone `powered` output.
#[block_behavior]
pub struct ShelfBlock {
    block: BlockRef,
}

impl ShelfBlock {
    /// Creates a new shelf block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for ShelfBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(
            self.block
                .default_state()
                .set_value(
                    &BlockStateProperties::HORIZONTAL_FACING,
                    context.horizontal_direction().opposite(),
                ),
        )
    }

    fn use_without_item(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        // The shelf exposes its three slots through the generic 3x1 menu, matching
        // vanilla's `ShelfMenu` layout.
        let Some(container_ref) = world
            .get_block_entity(pos)
            .and_then(ContainerRef::from_block_entity)
        else {
            return InteractionResult::Pass;
        };
        let inventory = Arc::clone(&player.inventory);
        player.open_menu(
            TextComponent::plain("Shelf"),
            move |context| chest(inventory, context.container_id, container_ref, 1),
        );
        InteractionResult::Success
    }

    fn new_block_entity(
        &self,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> BlockEntityCreation {
        BlockEntityCreation::from_registered_factory(BLOCK_ENTITIES.create(
            &vanilla_block_entity_types::SHELF,
            level,
            pos,
            state,
        ))
    }

    fn has_analog_output_signal(&self, _state: BlockStateId) -> bool {
        true
    }

    fn get_analog_output_signal(
        &self,
        _state: BlockStateId,
        world: &World,
        pos: BlockPos,
        _direction: Direction,
    ) -> i32 {
        container_signal(world, pos)
    }
}

/// Comparator signal from the block entity inventory at `pos`.
fn container_signal(world: &dyn crate::world::LevelReader, pos: BlockPos) -> i32 {
    let Some(container_ref) = world
        .get_block_entity(pos)
        .and_then(ContainerRef::from_block_entity)
    else {
        return 0;
    };
    let guard = ContainerLockGuard::lock_all(&[&container_ref]);
    guard
        .get(container_ref.container_id())
        .map_or(0, |container| {
            calculate_redstone_signal_from_container(container)
        })
}
