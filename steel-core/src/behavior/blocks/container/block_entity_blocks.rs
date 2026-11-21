//! Block behaviors for the remaining block-entity containers and the shelf.
//!
//! Each of these blocks defers almost everything to its block entity: the
//! behavior only supplies the placement state, the menu (when the block stores
//! items), and the block entity factory. The `open` sound and state changes are
//! driven by the block entity where vanilla does that.

use crate::world::game_event::GameEventContext;
use std::sync::{Arc, Weak};

use glam::DVec3;
use steel_macros::block_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::block_entity_type::BlockEntityTypeRef;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{
    BlockStateProperties, Direction, EnumProperty, FrontAndTop,
};
use steel_registry::data_components::PotionContents;
use steel_registry::item_stack::ItemStack;
use steel_registry::items::ItemRef;
use steel_registry::level_events::PARTICLES_AND_SOUND_PLANT_GROWTH;
use steel_registry::level_events::PARTICLES_WAX_ON;
use steel_registry::level_events::SOUND_BLAZE_FIREBALL;
use steel_registry::level_events::SOUND_DISPENSER_PROJECTILE_LAUNCH;
use steel_registry::level_events::SOUND_FIREWORK_SHOOT;
use steel_registry::level_events::SOUND_WIND_CHARGE_SHOOT;
use steel_registry::vanilla_block_entity_types;
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::vanilla_game_events::BLOCK_CHANGE;
use steel_registry::vanilla_game_events::BLOCK_PLACE;
use steel_registry::vanilla_game_events::ENTITY_INTERACT;
use steel_registry::vanilla_game_events::FLUID_PICKUP;
use steel_registry::vanilla_items;
use steel_registry::vanilla_potions::WATER;
use steel_registry::{sound_events, vanilla_entities};
use steel_utils::axis::Axis::Y;
use steel_utils::random::Random;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId, Downcast as _};
use text_components::TextComponent;

use crate::behavior::block::{BlockBehavior, BlockEntityCreation};
use crate::behavior::blocks::FireBlock;
use crate::behavior::blocks::TntBlock;
use crate::behavior::context::{BlockHitResult, BlockPlaceContext, InteractionResult};
use crate::behavior::items::BoneMealItem;
use crate::behavior::items::FlintAndSteelItem;
use crate::behavior::waxables::get_waxed_from_normal_variant;
use crate::behavior::{BlockItem, InventoryAccess};
use crate::block_entity::BLOCK_ENTITIES;
use crate::block_entity::BlockEntityTicker;
use crate::block_entity::entities::CrafterBlockEntity;
use crate::entity::entities::objects::projectiles::AbstractArrow as _;
use crate::entity::entities::{
    ArrowEntity, ExperienceBottleEntity, FireworkRocketEntity, ItemEntity, LingeringPotionEntity,
    SmallFireballEntity, SnowballEntity, SpectralArrowEntity, SplashPotionEntity, ThrownEggEntity,
    WindChargeEntity,
};
use crate::entity::projectile::triangle_random;
use crate::entity::{Entity, SharedEntity, ThrowableItemProjectile as _, next_entity_id};
use crate::inventory::container::calculate_redstone_signal_from_container;
use crate::inventory::lock::{ContainerLockGuard, ContainerRef};
use crate::inventory::menu::kinds::{brewing_stand, chest, crafter, dispenser, hopper, lectern};
use crate::player::Player;
use crate::world::LevelReader;
use crate::world::{SignalGetter, World};

const HOPPER_FACING: &EnumProperty<Direction> = &BlockStateProperties::FACING_HOPPER;

/// Vanilla `FireChargeItem.asProjectile` / `WindChargeItem.asProjectile`
/// direction spread.
///
/// Vanilla builds the launch direction from the dispenser facing plus a
/// triangular random deviation of `0.11485` per axis
/// (`RandomSource.triangle(mean, spread)`).
fn as_projectile_direction(facing: Direction) -> DVec3 {
    const SPREAD: f64 = 0.114_850_000_000_000_01;
    let (dx, dy, dz) = facing.offset();
    DVec3::new(
        triangle_random(f64::from(dx), SPREAD),
        triangle_random(f64::from(dy), SPREAD),
        triangle_random(f64::from(dz), SPREAD),
    )
}

/// Vanilla `Vec3.normalize`: returns the zero vector for near-zero lengths.
fn vanilla_normalize(vector: DVec3) -> DVec3 {
    let length = vector.length();
    if length < 1.0e-4 {
        DVec3::ZERO
    } else {
        vector / length
    }
}

/// Maps a dispensed item to the projectile entity vanilla fires for it.
///
/// Items without a `ProjectileItem` implementation return `None`, which makes
/// `dispense_from` drop them instead.
fn dispense_projectile(
    stack: &ItemStack,
    world: &Arc<World>,
    pos: DVec3,
    facing: Direction,
) -> Option<SharedEntity> {
    let kind = stack.item();
    let weak = Arc::downgrade(world);
    let id = next_entity_id();

    let entity: SharedEntity = if kind == &*vanilla_items::ARROW
        || kind == &*vanilla_items::TIPPED_ARROW
    {
        // Vanilla `ArrowItem.asProjectile`.
        let arrow = Arc::new(ArrowEntity::new(&vanilla_entities::ARROW, id, pos, weak));
        arrow.set_arrow_item(stack.copy_with_count(1));
        arrow
    } else if kind == &*vanilla_items::SPECTRAL_ARROW {
        // Vanilla `SpectralArrowItem.asProjectile`.
        let arrow = Arc::new(SpectralArrowEntity::new(
            &vanilla_entities::SPECTRAL_ARROW,
            id,
            pos,
            weak,
        ));
        arrow.set_arrow_item(stack.copy_with_count(1));
        arrow
    } else if kind == &*vanilla_items::SPLASH_POTION {
        let potion = SplashPotionEntity::new(&vanilla_entities::SPLASH_POTION, id, pos, weak);
        potion.set_item_clamped(stack.copy_with_count(1));
        Arc::new(potion)
    } else if kind == &*vanilla_items::LINGERING_POTION {
        let potion = LingeringPotionEntity::new(&vanilla_entities::LINGERING_POTION, id, pos, weak);
        potion.set_item_clamped(stack.copy_with_count(1));
        Arc::new(potion)
    } else if kind == &*vanilla_items::EXPERIENCE_BOTTLE {
        let bottle =
            ExperienceBottleEntity::new(&vanilla_entities::EXPERIENCE_BOTTLE, id, pos, weak);
        bottle.set_item_clamped(stack.copy_with_count(1));
        Arc::new(bottle)
    } else if kind == &*vanilla_items::SNOWBALL {
        let snowball = SnowballEntity::new(&vanilla_entities::SNOWBALL, id, pos, weak);
        snowball.set_item_clamped(stack.copy_with_count(1));
        Arc::new(snowball)
    } else if kind == &*vanilla_items::EGG
        || kind == &*vanilla_items::BROWN_EGG
        || kind == &*vanilla_items::BLUE_EGG
    {
        // Vanilla `EggItem.asProjectile`.
        let egg = ThrownEggEntity::new(&vanilla_entities::EGG, id, pos, weak);
        egg.set_item_clamped(stack.copy_with_count(1));
        Arc::new(egg)
    } else if kind == &*vanilla_items::WIND_CHARGE {
        // Vanilla `WindChargeItem.asProjectile` sets the delta movement to
        // the noised facing direction. The direction is *not* normalized
        // and `WindChargeItem.shoot` is a no-op, so the projectile keeps
        // this velocity.
        let charge = WindChargeEntity::new(&vanilla_entities::WIND_CHARGE, id, pos, weak);
        charge.set_velocity(as_projectile_direction(facing));
        Arc::new(charge)
    } else if kind == &*vanilla_items::FIRE_CHARGE {
        // Vanilla `FireChargeItem.asProjectile` copies the source stack
        // into the fireball's synced item (count one) and flies along the
        // normalized noised facing direction. `FireChargeItem.shoot` is a
        // no-op, so the fireball keeps this velocity.
        let fireball = SmallFireballEntity::new(&vanilla_entities::SMALL_FIREBALL, id, pos, weak);
        fireball.set_item_clamped(stack.copy_with_count(1));
        fireball.set_velocity(vanilla_normalize(as_projectile_direction(facing)));
        Arc::new(fireball)
    } else if kind == &*vanilla_items::FIREWORK_ROCKET {
        // Vanilla `FireworkRocketItem.asProjectile` creates the rocket with
        // the source stack and the `shotAtAngle` flag set.
        let rocket = FireworkRocketEntity::launched(
            &vanilla_entities::FIREWORK_ROCKET,
            id,
            pos,
            weak,
            stack.copy_with_count(1),
        );
        rocket.set_shot_at_angle(true);
        Arc::new(rocket)
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
    fn on_place(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        old_state: BlockStateId,
        _moved_by_piston: bool,
    ) {
        if old_state.get_block() != state.get_block() {
            self.handle_neighbor_changed(state, world, pos, self.block, false);
        }
    }

    fn handle_neighbor_changed(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _source: BlockRef,
        _moved_by_piston: bool,
    ) {
        let enabled = !world.has_neighbor_signal(pos);
        if state.get_value(&BlockStateProperties::ENABLED) != enabled {
            world.set_block(
                pos,
                state.set_value(&BlockStateProperties::ENABLED, enabled),
                UpdateFlags::UPDATE_CLIENTS,
            );
        }
    }

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
        Some(
            self.block
                .default_state()
                .set_value(HOPPER_FACING, direction),
        )
    }

    /// Vanilla `HopperBlock.useWithoutItem`: opens the five-slot hopper screen.
    fn use_without_item(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        let Some(container) = world
            .get_block_entity(pos)
            .and_then(ContainerRef::from_block_entity)
        else {
            return InteractionResult::Pass;
        };
        let inventory = Arc::clone(&player.inventory);
        player.open_menu(TextComponent::plain("Item Hopper"), move |context| {
            hopper(inventory, context.container_id, container)
        });
        InteractionResult::Success
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
        state: BlockStateId,
        block_entity_type: BlockEntityTypeRef,
    ) -> Option<BlockEntityTicker> {
        // Vanilla `HopperBlock.getTicker`:
        //
        //     return state.getValue(ENABLED)
        //         ? (level, pos, state, hopper) -> HopperBlockEntity.pushItemsTick(...)
        //         : null;
        //
        // A powered hopper keeps its contents and cooldown frozen, so the gate has
        // to live here. Returning a ticker for a disabled hopper let a redstone-
        // powered hopper keep moving items.
        if !state.get_value(&BlockStateProperties::ENABLED) {
            return None;
        }
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
                // Vanilla `DispenserBlock.neighborChanged`: checks direct and
                // quasi-connectivity (`pos.above()`) power, scheduling a 4-tick
                // dispense on the rising edge and clearing `TRIGGERED` on the
                // falling edge.
                let powered =
                    world.has_neighbor_signal(pos) || world.has_neighbor_signal(pos.above());
                let was_powered = state.get_value(&BlockStateProperties::TRIGGERED);
                if powered && !was_powered {
                    world.schedule_block_tick_default(pos, self.block, 4);
                    world.set_block(
                        pos,
                        state.set_value(&BlockStateProperties::TRIGGERED, true),
                        UpdateFlags::UPDATE_CLIENTS,
                    );
                } else if !powered && was_powered {
                    world.set_block(
                        pos,
                        state.set_value(&BlockStateProperties::TRIGGERED, false),
                        UpdateFlags::UPDATE_CLIENTS,
                    );
                }
            }

            fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
                $fire(world, pos, state);
            }

            fn get_state_for_placement(
                &self,
                context: &BlockPlaceContext<'_>,
            ) -> Option<BlockStateId> {
                // Vanilla `DispenserBlock.getStateForPlacement`: the funnel faces
                // the player, i.e. away from the clicked face.
                Some(self.block.default_state().set_value(
                    &BlockStateProperties::FACING,
                    context.clicked_face().opposite(),
                ))
            }

            /// Vanilla `DispenserBlock.useWithoutItem` (inherited from
            /// `BaseContainerBlockEntity`): opens the 3x3 screen.
            fn use_without_item(
                &self,
                _state: BlockStateId,
                world: &Arc<World>,
                pos: BlockPos,
                player: &Player,
                _hit_result: &BlockHitResult,
                _inv: &mut InventoryAccess,
            ) -> InteractionResult {
                let Some(container) = world
                    .get_block_entity(pos)
                    .and_then(ContainerRef::from_block_entity)
                else {
                    return InteractionResult::Pass;
                };
                let inventory = Arc::clone(&player.inventory);
                player.open_menu(TextComponent::plain("Dispenser"), move |context| {
                    dispenser(inventory, context.container_id, container)
                });
                InteractionResult::Success
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

funnel_like!(
    DispenserBlock,
    DISPENSER,
    |world: &Arc<World>, pos, state| {
        dispense_from(world, pos, state);
    }
);
funnel_like!(DropperBlock, DROPPER, |world: &Arc<World>, pos, state| {
    drop_from(world, pos, state);
});

/// Vanilla `DispenserBlockEntity.getRandomSlot`: selects one of the non-empty
/// slots uniformly at random using reservoir sampling.
fn pick_random_non_empty_slot(
    world: &Arc<World>,
    pos: BlockPos,
) -> Option<(ContainerRef, usize, ItemStack)> {
    let block_entity = world.get_block_entity(pos)?;
    let container = block_entity.container_ref()?;
    let guard = container.lock();
    let mut chosen: Option<usize> = None;
    let mut seen: u32 = 0;
    for slot in 0..guard.get_container_size() {
        if !guard.get_item(slot).is_empty() {
            seen += 1;
            if rand::random_range(0..seen) == 0 {
                chosen = Some(slot);
            }
        }
    }
    let slot = chosen?;
    let snapshot = guard.get_item(slot).clone();
    drop(guard);
    Some((container, slot, snapshot))
}

/// Puts a replacement item back into `slot` (or into any open slot / drops it
/// if `slot` still carries remaining items from a stack > 1).
fn replace_or_store_dispensed_result(
    world: &Arc<World>,
    pos: BlockPos,
    facing: Direction,
    container: &ContainerRef,
    slot: usize,
    replacement: ItemStack,
) {
    let mut guard = container.lock();
    let current = guard.get_item(slot);
    if current.count() <= 1 {
        guard.set_item(slot, replacement);
        return;
    }
    let _ = guard.remove_item(slot, 1);
    for idx in 0..guard.get_container_size() {
        let existing = guard.get_item(idx);
        if existing.is_empty() {
            guard.set_item(idx, replacement);
            return;
        }
        if ItemStack::is_same_item_same_components(existing, &replacement)
            && existing.count() < existing.max_stack_size()
        {
            guard.get_item_mut(idx).grow(1);
            return;
        }
    }
    drop(guard);
    spawn_dropped_item(world, pos, facing, replacement);
}

/// Tries to insert a single item stack into `destination` on `side`.
fn try_insert_into_container(
    destination: &ContainerRef,
    side: Direction,
    stack: &ItemStack,
) -> bool {
    let dest_id = destination.container_id();
    let mut guard = ContainerLockGuard::lock_all(&[destination]);
    let slots: Vec<usize> = {
        let Some(container) = guard.get(dest_id) else {
            return false;
        };
        match container.slots_for_face(side) {
            Some(s) => s.to_vec(),
            None => (0..container.get_container_size()).collect(),
        }
    };
    let Some(container) = guard.get_mut(dest_id) else {
        return false;
    };
    // First pass: merge into compatible non-full stacks.
    for &idx in &slots {
        if !container.can_place_item_through_face(idx, stack, side) {
            continue;
        }
        let existing = container.get_item(idx);
        let limit = existing
            .max_stack_size()
            .min(container.get_max_stack_size());
        if !existing.is_empty()
            && ItemStack::is_same_item_same_components(existing, stack)
            && existing.count() < limit
        {
            container.get_item_mut(idx).grow(1);
            guard.set_changed(dest_id);
            return true;
        }
    }
    // Second pass: place into first empty allowed slot.
    for &idx in &slots {
        if !container.can_place_item_through_face(idx, stack, side) {
            continue;
        }
        if container.get_item(idx).is_empty() {
            container.set_item(idx, stack.copy_with_count(1));
            guard.set_changed(dest_id);
            return true;
        }
    }
    false
}

/// Vanilla `DispenserBlock.getDispensePosition(source)`: the block center
/// offset by `0.7` along the facing axis.
fn launch_point(pos: BlockPos, facing: Direction) -> DVec3 {
    let (dx, dy, dz) = facing.offset();
    DVec3::new(
        f64::from(pos.x()) + 0.5 + f64::from(dx) * 0.7,
        f64::from(pos.y()) + 0.5 + f64::from(dy) * 0.7,
        f64::from(pos.z()) + 0.5 + f64::from(dz) * 0.7,
    )
}

/// Vanilla `ProjectileItem.DispenseConfig.PositionFunction`.
///
/// Vanilla positions the projectile relative to the dispenser block center,
/// either along the facing axis (`DispenserBlock.getDispensePosition`) or just
/// outside of the block (`FireworkRocketItem.getEntityJustOutsideOfBlockPos`).
#[derive(Clone, Copy)]
enum DispensePosition {
    /// `DispenserBlock.getDispensePosition(source, scale, offset)`.
    AxisOffset { scale: f64, offset: DVec3 },
    /// `FireworkRocketItem.getEntityJustOutsideOfBlockPos`.
    JustOutsideOfBlock,
}

/// Vanilla `ProjectileItem.DispenseConfig`.
#[derive(Clone, Copy)]
struct DispenseConfig {
    position: DispensePosition,
    /// The launch accuracy; vanilla adds `0.0172275 * uncertainty` of noise.
    uncertainty: f32,
    /// The launch speed multiplier.
    power: f32,
    /// `overrideDispenseEvent`: the level event played instead of the default
    /// projectile-launch event.
    override_dispense_event: Option<i32>,
}

impl DispenseConfig {
    /// Vanilla `ProjectileItem.DispenseConfig.DEFAULT`.
    const DEFAULT: Self = Self {
        position: DispensePosition::AxisOffset {
            scale: 0.7,
            offset: DVec3::new(0.0, 0.1, 0.0),
        },
        uncertainty: 6.0,
        power: 1.1,
        override_dispense_event: None,
    };

    /// Vanilla `ProjectileItem.DispenseConfig` for the dispensed item.
    fn for_item(item: ItemRef) -> Self {
        if item == &*vanilla_items::FIRE_CHARGE || item == &*vanilla_items::WIND_CHARGE {
            // Vanilla `FireChargeItem.createDispenseConfig` and
            // `WindChargeItem.createDispenseConfig`.
            return Self {
                position: DispensePosition::AxisOffset {
                    scale: 1.0,
                    offset: DVec3::ZERO,
                },
                uncertainty: 6.666_666_5,
                power: 1.0,
                override_dispense_event: Some(if item == &*vanilla_items::FIRE_CHARGE {
                    SOUND_BLAZE_FIREBALL
                } else {
                    SOUND_WIND_CHARGE_SHOOT
                }),
            };
        }
        if item == &*vanilla_items::FIREWORK_ROCKET {
            // Vanilla `FireworkRocketItem.createDispenseConfig`.
            return Self {
                position: DispensePosition::JustOutsideOfBlock,
                uncertainty: 1.0,
                power: 0.5,
                override_dispense_event: Some(SOUND_FIREWORK_SHOOT),
            };
        }
        if item == &*vanilla_items::SPLASH_POTION
            || item == &*vanilla_items::LINGERING_POTION
            || item == &*vanilla_items::EXPERIENCE_BOTTLE
        {
            // Vanilla `ThrowablePotionItem.createDispenseConfig` and
            // `ExperienceBottleItem.createDispenseConfig` halve the default
            // accuracy and raise the default power by a quarter.
            return Self {
                uncertainty: Self::DEFAULT.uncertainty * 0.5,
                power: Self::DEFAULT.power * 1.25,
                ..Self::DEFAULT
            };
        }
        Self::DEFAULT
    }

    /// The position the projectile is spawned at.
    fn spawn_position(self, pos: BlockPos, facing: Direction) -> DVec3 {
        let (dx, dy, dz) = facing.offset();
        let center = DVec3::new(
            f64::from(pos.x()) + 0.5,
            f64::from(pos.y()) + 0.5,
            f64::from(pos.z()) + 0.5,
        );
        match self.position {
            DispensePosition::AxisOffset { scale, offset } => {
                center
                    + DVec3::new(
                        scale * f64::from(dx) + offset.x,
                        scale * f64::from(dy) + offset.y,
                        scale * f64::from(dz) + offset.z,
                    )
            }
            DispensePosition::JustOutsideOfBlock => {
                let offset = 0.500_009_999_999_747_4;
                center
                    + DVec3::new(
                        f64::from(dx) * offset,
                        f64::from(dy) * offset,
                        f64::from(dz) * offset,
                    )
            }
        }
    }
}

/// Vanilla `DispenserBlock.dispenseFrom`: selects a random non-empty slot and
/// executes the registered `DispenseItemBehavior` (projectiles, TNT priming,
/// water/lava bucket placement, empty bucket fluid pickup, flint & steel fire /
/// TNT / campfire ignition with durability, or default item ejection). Plays
/// `BLOCK_DISPENSER_FAIL` when empty.
#[expect(
    clippy::too_many_lines,
    reason = "single table dispatching every vanilla DispenseItemBehavior branch"
)]
fn dispense_from(world: &Arc<World>, pos: BlockPos, state: BlockStateId) -> bool {
    use steel_registry::vanilla_blocks;

    let facing = state.get_value(&BlockStateProperties::FACING);
    let Some((container, slot, snapshot)) = pick_random_non_empty_slot(world, pos) else {
        world.play_sound(
            &sound_events::BLOCK_DISPENSER_FAIL,
            SoundSource::Blocks,
            pos,
            1.0,
            1.2,
            None,
        );
        return false;
    };

    let target_pos = pos.relative(facing);
    let target_state = world.get_block_state(target_pos);
    let target_block = target_state.get_block();
    let item = snapshot.item();

    // A dispenser uses a brush only on an adult armadillo in the target
    // block; without one it retains the brush rather than ejecting it.
    if item == &*vanilla_items::BRUSH {
        use crate::entity::AgeableMob;
        use crate::entity::entities::mobs::passive::ArmadilloEntity;
        use steel_utils::WorldAabb;

        let x = f64::from(target_pos.x());
        let y = f64::from(target_pos.y());
        let z = f64::from(target_pos.z());
        let entities =
            world.get_entities_in_aabb(&WorldAabb::new(x, y, z, x + 1.0, y + 1.0, z + 1.0));
        let armadillo = entities.iter().find_map(|entity| {
            let armadillo = entity.as_ref().downcast_ref::<ArmadilloEntity>()?;
            (Entity::is_alive(armadillo) && !AgeableMob::is_baby(armadillo)).then_some(armadillo)
        });
        if let Some(armadillo) = armadillo {
            let _ =
                armadillo.spawn_at_location(ItemStack::new(&vanilla_items::ARMADILLO_SCUTE), 0.0);
            armadillo.play_sound(&sound_events::ENTITY_ARMADILLO_BRUSH, 1.0, 1.0);
            armadillo.game_event(&ENTITY_INTERACT);
            let mut guard = container.lock();
            if guard.get_item_mut(slot).hurt_and_break(16, false) {
                guard.set_item(slot, ItemStack::empty());
            }
        }
        let sound = if armadillo.is_some() {
            &sound_events::BLOCK_DISPENSER_DISPENSE
        } else {
            &sound_events::BLOCK_DISPENSER_FAIL
        };
        world.play_sound(sound, SoundSource::Blocks, pos, 1.0, 1.0, None);
        return armadillo.is_some();
    }

    // Shulker boxes use a directional placement context, preserving item
    // components and retaining the stack when placement is blocked.
    if let Some(block) = item.block
        && block.has_tag(&BlockTag::SHULKER_BOXES)
    {
        let clicked_face = if world.get_block_state(target_pos.below()).is_air() {
            facing
        } else {
            Direction::Up
        };
        let mut stack = snapshot.clone();
        let context =
            BlockPlaceContext::directional(world, target_pos, facing, &mut stack, clicked_face);
        let success = BlockItem::new(block).place(context).consumes_action();
        if success {
            container.lock().set_item(slot, stack);
        }
        let sound = if success {
            &sound_events::BLOCK_DISPENSER_DISPENSE
        } else {
            &sound_events::BLOCK_DISPENSER_FAIL
        };
        world.play_sound(sound, SoundSource::Blocks, pos, 1.0, 1.0, None);
        return success;
    }

    // Honeycomb consumes one item only when the facing block can be waxed.
    if item == &*vanilla_items::HONEYCOMB
        && let Some(waxed) = get_waxed_from_normal_variant(target_block)
    {
        let waxed_state = steel_registry::REGISTRY
            .blocks
            .copy_matching_properties(target_state, waxed);
        world.set_block(target_pos, waxed_state, UpdateFlags::UPDATE_ALL);
        let _ = container.lock().remove_item(slot, 1);
        world.level_event(PARTICLES_WAX_ON, target_pos, 0, None);
        world.game_event(
            &BLOCK_CHANGE,
            target_pos,
            &GameEventContext::new(None, Some(waxed_state)),
        );
        return true;
    }

    // Optional bone-meal dispensing keeps the item if neither a crop nor an
    // underwater growth target accepts it. Reuse the player-side growth logic.
    if item == &*vanilla_items::BONE_MEAL {
        let success = BoneMealItem::grow(world, target_pos)
            || BoneMealItem::grow_water_plant(world, target_pos, facing);
        if success {
            let _ = container.lock().remove_item(slot, 1);
            world.level_event(PARTICLES_AND_SOUND_PLANT_GROWTH, target_pos, 15, None);
        }
        let sound = if success {
            &sound_events::BLOCK_DISPENSER_DISPENSE
        } else {
            &sound_events::BLOCK_DISPENSER_FAIL
        };
        world.play_sound(sound, SoundSource::Blocks, pos, 1.0, 1.0, None);
        return success;
    }

    if item == &*vanilla_items::GLASS_BOTTLE {
        use crate::block_entity::entities::BeehiveBlockEntity;
        use crate::fluid::FluidStateExt as _;
        let filled = if target_state
            .try_get_value(&BlockStateProperties::LEVEL_HONEY)
            .is_some_and(|level| level >= 5)
        {
            world.get_block_entity(target_pos).and_then(|entity| {
                let hive = entity.downcast_ref::<BeehiveBlockEntity>()?;
                hive.on_honey_harvested(target_state, world, target_pos, 0, true);
                Some(ItemStack::new(&vanilla_items::HONEY_BOTTLE))
            })
        } else if target_state.get_fluid_state().is_water() {
            Some(PotionContents::create_item_stack(
                &vanilla_items::POTION,
                &WATER,
            ))
        } else {
            None
        };
        if let Some(filled) = filled {
            replace_or_store_dispensed_result(world, pos, facing, &container, slot, filled);
            world.game_event(&FLUID_PICKUP, pos, &GameEventContext::new(None, None));
            world.play_sound(
                &sound_events::BLOCK_DISPENSER_DISPENSE,
                SoundSource::Blocks,
                pos,
                1.0,
                1.0,
                None,
            );
            return true;
        }
        // Invalid bottle targets use ordinary item ejection, not a no-op.
    }

    // 1. TNT: prime TNT in front if space allows.
    if item == &*vanilla_items::TNT && target_state.is_air() {
        let _ = container.lock().remove_item(slot, 1);
        TntBlock::prime(world, target_pos);
        world.play_sound(
            &sound_events::BLOCK_DISPENSER_DISPENSE,
            SoundSource::Blocks,
            pos,
            1.0,
            1.0,
            None,
        );
        return true;
    }

    // 2. Water / Lava / Powder-Snow buckets: place block and leave an empty bucket.
    if (item == &*vanilla_items::WATER_BUCKET
        || item == &*vanilla_items::LAVA_BUCKET
        || item == &*vanilla_items::POWDER_SNOW_BUCKET)
        && target_state.is_air()
    {
        let placed = if item == &*vanilla_items::WATER_BUCKET {
            vanilla_blocks::WATER.default_state()
        } else if item == &*vanilla_items::LAVA_BUCKET {
            vanilla_blocks::LAVA.default_state()
        } else {
            vanilla_blocks::POWDER_SNOW.default_state()
        };
        world.set_block(target_pos, placed, UpdateFlags::UPDATE_ALL);
        replace_or_store_dispensed_result(
            world,
            pos,
            facing,
            &container,
            slot,
            ItemStack::new(&vanilla_items::BUCKET),
        );
        world.play_sound(
            &sound_events::BLOCK_DISPENSER_DISPENSE,
            SoundSource::Blocks,
            pos,
            1.0,
            1.0,
            None,
        );
        return true;
    }

    // 3. Empty bucket: scoop water/lava source block or powder snow in front.
    if item == &*vanilla_items::BUCKET {
        let is_source = target_state
            .try_get_value(&BlockStateProperties::LEVEL)
            .is_some_and(|lvl| lvl == 0);
        let filled = if target_block == &vanilla_blocks::WATER && is_source {
            Some(ItemStack::new(&vanilla_items::WATER_BUCKET))
        } else if target_block == &vanilla_blocks::LAVA && is_source {
            Some(ItemStack::new(&vanilla_items::LAVA_BUCKET))
        } else if target_block == &vanilla_blocks::POWDER_SNOW {
            Some(ItemStack::new(&vanilla_items::POWDER_SNOW_BUCKET))
        } else {
            None
        };
        if let Some(filled_bucket) = filled {
            world.set_block(
                target_pos,
                vanilla_blocks::AIR.default_state(),
                UpdateFlags::UPDATE_ALL,
            );
            replace_or_store_dispensed_result(world, pos, facing, &container, slot, filled_bucket);
            world.play_sound(
                &sound_events::BLOCK_DISPENSER_DISPENSE,
                SoundSource::Blocks,
                pos,
                1.0,
                1.0,
                None,
            );
            return true;
        }
    }

    // 3b. Glowstone charges a facing Respawn Anchor up to 4 charges.
    if item == &*vanilla_items::GLOWSTONE
        && target_block == &vanilla_blocks::RESPAWN_ANCHOR
        && let Some(charges) =
            target_state.try_get_value(&BlockStateProperties::RESPAWN_ANCHOR_CHARGES)
        && charges < 4
    {
        let _ = container.lock().remove_item(slot, 1);
        world.set_block(
            target_pos,
            target_state.set_value(&BlockStateProperties::RESPAWN_ANCHOR_CHARGES, charges + 1),
            UpdateFlags::UPDATE_ALL,
        );
        world.play_sound(
            &sound_events::BLOCK_RESPAWN_ANCHOR_CHARGE,
            SoundSource::Blocks,
            target_pos,
            1.0,
            1.0,
            None,
        );
        return true;
    }

    // 4. Flint and steel: vanilla `FlintAndSteelDispenseItemBehavior`.
    //
    // The optional behavior starts out successful, so the tool is damaged even
    // when the target cannot be ignited. Only a rejected TNT prime flips the
    // result to a failure.
    if item == &*vanilla_items::FLINT_AND_STEEL {
        let mut success = true;
        if FireBlock::can_be_placed_at(world, target_pos, facing) {
            world.set_block(
                target_pos,
                FireBlock::get_state(world.as_ref(), target_pos),
                UpdateFlags::UPDATE_ALL,
            );
            world.game_event(&BLOCK_PLACE, target_pos, &GameEventContext::new(None, None));
        } else if FlintAndSteelItem::can_light(target_state) {
            let next = target_state.set_value(&BlockStateProperties::LIT, true);
            world.set_block(target_pos, next, UpdateFlags::UPDATE_ALL);
            world.game_event(
                &BLOCK_CHANGE,
                target_pos,
                &GameEventContext::new(None, Some(next)),
            );
        } else if target_block == &vanilla_blocks::TNT {
            // Vanilla primes the TNT first and only removes the block when the
            // prime was accepted.
            if TntBlock::prime(world, target_pos) {
                world.set_block(
                    target_pos,
                    vanilla_blocks::AIR.default_state(),
                    UpdateFlags::UPDATE_ALL,
                );
            } else {
                success = false;
            }
        }
        if success {
            let mut guard = container.lock();
            if guard.get_item_mut(slot).hurt_and_break(1, false) {
                guard.set_item(slot, ItemStack::empty());
            }
            drop(guard);
            world.play_sound(
                &sound_events::BLOCK_DISPENSER_DISPENSE,
                SoundSource::Blocks,
                pos,
                1.0,
                1.0,
                None,
            );
        } else {
            world.play_sound(
                &sound_events::BLOCK_DISPENSER_FAIL,
                SoundSource::Blocks,
                pos,
                1.0,
                1.0,
                None,
            );
        }
        return success;
    }

    // 5. Projectiles & DefaultDispenseItemBehavior.
    let stack = container.lock().remove_item(slot, 1);
    let config = DispenseConfig::for_item(stack.item());
    let spawn_pos = config.spawn_position(pos, facing);
    let sound = if let Some(projectile) = dispense_projectile(&stack, world, spawn_pos, facing) {
        // Vanilla `ProjectileDispenseBehavior.execute`: the projectile is
        // launched with `Projectile.shoot`, which normalizes the facing
        // direction, adds the triangular accuracy noise and scales by the
        // configured power.
        //
        // `FireChargeItem.shoot` and `WindChargeItem.shoot` are no-ops in
        // vanilla: those projectiles keep the velocity their `asProjectile`
        // implementation gave them.
        let shoots_with_config = stack.item() != &*vanilla_items::FIRE_CHARGE
            && stack.item() != &*vanilla_items::WIND_CHARGE;
        if shoots_with_config {
            let (dx, dy, dz) = facing.offset();
            let direction = DVec3::new(f64::from(dx), f64::from(dy), f64::from(dz));
            if let Some(projectile) = projectile.as_projectile() {
                projectile.shoot(direction, config.power, config.uncertainty);
            }
        }
        let _ = world.try_add_entity(projectile);
        // Vanilla `ProjectileDispenseBehavior.playSound` uses the configured
        // dispense event, defaulting to the projectile-launch event.
        let event = config
            .override_dispense_event
            .unwrap_or(SOUND_DISPENSER_PROJECTILE_LAUNCH);
        world.level_event(event, pos, 0, None);
        return true;
    } else {
        spawn_dropped_item(world, pos, facing, stack);
        &sound_events::BLOCK_DISPENSER_DISPENSE
    };

    world.play_sound(sound, SoundSource::Blocks, pos, 1.0, 1.0, None);
    true
}

/// Vanilla `DropperBlock.dispenseFrom`: picks a random non-empty slot and
/// transfers one item into a facing `Container` if present, otherwise drops it
/// into the world. Plays `BLOCK_DISPENSER_FAIL` when empty.
fn drop_from(world: &Arc<World>, pos: BlockPos, state: BlockStateId) -> bool {
    let facing = state.get_value(&BlockStateProperties::FACING);
    let Some((container, slot, snapshot)) = pick_random_non_empty_slot(world, pos) else {
        world.play_sound(
            &sound_events::BLOCK_DISPENSER_FAIL,
            SoundSource::Blocks,
            pos,
            1.0,
            1.2,
            None,
        );
        return false;
    };

    let single = snapshot.copy_with_count(1);
    let target_pos = pos.relative(facing);
    if let Some(destination) = world
        .get_block_entity(target_pos)
        .and_then(ContainerRef::from_block_entity)
    {
        if try_insert_into_container(&destination, facing.opposite(), &single) {
            let _ = container.lock().remove_item(slot, 1);
        }
        return true;
    }

    let stack = container.lock().remove_item(slot, 1);
    spawn_dropped_item(world, pos, facing, stack);
    world.play_sound(
        &sound_events::BLOCK_DISPENSER_DISPENSE,
        SoundSource::Blocks,
        pos,
        1.0,
        1.0,
        None,
    );
    true
}

/// Vanilla `DefaultDispenseItemBehavior.spawnItem`, separate from projectile
/// launching: lower the spawn point and apply triangular noise at accuracy 6.
fn default_item_launch(
    pos: BlockPos,
    facing: Direction,
    random: &mut impl Random,
) -> (DVec3, DVec3) {
    let mut spawn_pos = launch_point(pos, facing);
    spawn_pos.y -= if facing.get_axis() == Y {
        0.125
    } else {
        0.15625
    };
    let (dx, _, dz) = facing.offset();
    let power = random.next_f64() * 0.1 + 0.2;
    let deviation = 0.017_227_5 * 6.0;
    let velocity = DVec3::new(
        random.triangle(f64::from(dx) * power, deviation),
        random.triangle(0.2, deviation),
        random.triangle(f64::from(dz) * power, deviation),
    );
    (spawn_pos, velocity)
}

/// Spawns an ordinary dispensed item, never with the projectile launch speed.
fn spawn_dropped_item(world: &Arc<World>, pos: BlockPos, facing: Direction, stack: ItemStack) {
    use steel_utils::random::legacy_random::LegacyRandom;

    let mut random = LegacyRandom::from_seed(rand::random());
    let (spawn_pos, velocity) = default_item_launch(pos, facing, &mut random);
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

    /// Vanilla `BrewingStandBlock.useWithoutItem`: opens the brewing screen.
    fn use_without_item(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        let Some(container) = world
            .get_block_entity(pos)
            .and_then(ContainerRef::from_block_entity)
        else {
            return InteractionResult::Pass;
        };
        let inventory = Arc::clone(&player.inventory);
        player.open_menu(TextComponent::plain("Brewing Stand"), move |context| {
            brewing_stand(inventory, context.container_id, container)
        });
        InteractionResult::Success
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
        block_entity_type: BlockEntityTypeRef,
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
        Some(self.block.default_state().set_value(
            &BlockStateProperties::HORIZONTAL_FACING,
            context.horizontal_direction().opposite(),
        ))
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

    /// Vanilla `LecternBlock.useWithoutItem`: opens the single book slot.
    ///
    /// The reading screen only appears while the lectern holds a book; an empty
    /// lectern is interacted with through `use_item_on` instead.
    fn use_without_item(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        if !state.get_value(&BlockStateProperties::HAS_BOOK) {
            return InteractionResult::Pass;
        }
        let Some(container) = world
            .get_block_entity(pos)
            .and_then(ContainerRef::from_block_entity)
        else {
            return InteractionResult::Pass;
        };
        player.open_menu(TextComponent::plain("Lectern"), move |context| {
            lectern(context.container_id, container)
        });
        InteractionResult::Success
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
    fn use_without_item(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        let Some(block_entity) = world.get_block_entity(pos) else {
            return InteractionResult::Pass;
        };
        if block_entity.downcast_ref::<CrafterBlockEntity>().is_none() {
            return InteractionResult::Pass;
        }
        let Some(container_ref) = ContainerRef::from_block_entity(Arc::clone(&block_entity)) else {
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
        Some(self.block.default_state().set_value(
            &BlockStateProperties::ORIENTATION,
            FrontAndTop::from_front_and_top(context.get_nearest_looking_direction().opposite()),
        ))
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
            //
            // Redstone carries no player attribution in Steel, so the crafter
            // credits the nearest player within reach: the one whose
            // `crafter_recipe_crafted` vanilla would fire when the crafter is
            // powered next to them.
            let mut powering_player = None;
            world.players.iter_players(|_uuid, player| {
                // Within eight blocks on every axis, which covers vanilla's
                // four-block interaction range with slack for the sequence delay.
                let player_pos = player.block_position();
                let within = (player_pos.x() - pos.x()).abs() <= 8
                    && (player_pos.y() - pos.y()).abs() <= 8
                    && (player_pos.z() - pos.z()).abs() <= 8;
                if within {
                    powering_player = Some(player.gameprofile.id());
                    false
                } else {
                    true
                }
            });
            crafter.set_triggering_player(powering_player);
            let _ = crafter.start_crafting(world);
        } else if state.get_value(&BlockStateProperties::TRIGGERED) {
            // The sequence finished or the signal dropped: mirror the falling
            // edge into the block state.
            world.set_block(
                pos,
                state.set_value(&BlockStateProperties::TRIGGERED, false),
                UpdateFlags::UPDATE_CLIENTS,
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
        block_entity_type: BlockEntityTypeRef,
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
        Some(self.block.default_state().set_value(
            &BlockStateProperties::HORIZONTAL_FACING,
            context.horizontal_direction().opposite(),
        ))
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
        Some(self.block.default_state().set_value(
            &BlockStateProperties::HORIZONTAL_FACING,
            context.horizontal_direction().opposite(),
        ))
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
        player.open_menu(TextComponent::plain("Shelf"), move |context| {
            chest(inventory, context.container_id, container_ref, 1)
        });
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
fn container_signal(world: &dyn LevelReader, pos: BlockPos) -> i32 {
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

#[cfg(test)]
#[path = "block_entity_blocks_tests.rs"]
mod regression_tests;
