//! Crossbow (`CrossbowItem`).
//!
//! Vanilla crossbow: right-clicking an uncharged crossbow starts the `25`-tick
//! charge (shortened by Quick Charge), which loads arrows — or a firework
//! rocket — into the `charged_projectiles` data component. Right-clicking a
//! loaded crossbow fires every stored projectile in one shot, applying the
//! Multishot spread when the enchantment is present.

use std::sync::Arc;

use glam::DVec3;
use steel_macros::item_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::data_components::components::ChargedProjectiles;
use steel_registry::data_components::vanilla_components;
use steel_registry::item_stack::ItemStack;
use steel_registry::item_stack_template::ItemStackTemplate;
use steel_registry::stat::vanilla_stat_types;
use steel_registry::{sound_events, vanilla_entities, vanilla_items};
use steel_utils::Identifier;
use steel_utils::types::GameType;

use crate::behavior::items::{ArrowItem, SpectralArrowItem};
use crate::behavior::{InteractionResult, ItemBehavior, ItemUseAnimation, UseItemContext};
use crate::entity::LivingEntity;
use crate::entity::entities::{AbstractArrow, ArrowPickup, FireworkRocketEntity};
use crate::entity::{Entity, LivingEntityRef, Projectile as _, SharedEntity, next_entity_id};
use crate::player::Player;
use crate::world::World;

/// Vanilla `CrossbowItem.DEFAULT_CHARGE_DURATION`.
pub const DEFAULT_CHARGE_DURATION: i32 = 25;

/// Ticks removed from the charge time per Quick Charge level.
pub const QUICK_CHARGE_STEP: i32 = 5;

/// Vanilla shot velocity for arrows and other item projectiles.
pub const ARROW_SHOT_VELOCITY: f32 = 3.15;

/// Vanilla shot velocity for firework rockets.
pub const FIREWORK_SHOT_VELOCITY: f32 = 1.6;

/// A firework rocket must always be slower than a plain arrow.
const _: () = assert!(FIREWORK_SHOT_VELOCITY < ARROW_SHOT_VELOCITY);

/// Multishot loads (and fires) this many projectiles.
pub const MULTISHOT_COUNT: i32 = 3;

/// Half-angle between Multishot projectiles, in degrees.
pub const MULTISHOT_SPREAD: f32 = 10.0;

const QUICK_CHARGE: &str = "quick_charge";
const MULTISHOT: &str = "multishot";

/// Vanilla crossbow behavior.
#[item_behavior]
pub struct CrossbowItem;

impl CrossbowItem {
    /// Vanilla `CrossbowItem#getChargeDuration`, with Quick Charge applied.
    #[must_use]
    pub fn charge_duration(stack: &ItemStack) -> i32 {
        let level = stack.get_enchantment_level(&Identifier::vanilla_static(QUICK_CHARGE));
        if level <= 0 {
            DEFAULT_CHARGE_DURATION
        } else {
            (DEFAULT_CHARGE_DURATION - QUICK_CHARGE_STEP * level).max(1)
        }
    }

    /// Vanilla `EnchantmentHelper.processProjectileCount` for Multishot.
    #[must_use]
    pub fn projectile_count(stack: &ItemStack) -> i32 {
        if stack.get_enchantment_level(&Identifier::vanilla_static(MULTISHOT)) > 0 {
            MULTISHOT_COUNT
        } else {
            1
        }
    }

    /// Returns the loaded projectiles, treating an empty list as uncharged.
    #[must_use]
    pub fn charged(stack: &ItemStack) -> Option<&ChargedProjectiles> {
        stack
            .get(vanilla_components::CHARGED_PROJECTILES)
            .filter(|charged| !charged.items().is_empty())
    }

    /// Whether the crossbow currently holds at least one projectile.
    #[must_use]
    pub fn is_charged(stack: &ItemStack) -> bool {
        Self::charged(stack).is_some()
    }

    /// Vanilla Multishot spread: `(yaw offset, pitch offset)` in degrees.
    #[must_use]
    pub fn spread(index: i32, count: i32) -> (f32, f32) {
        if count <= 1 {
            return (0.0, 0.0);
        }
        let offset = index as f32 - (count - 1) as f32 / 2.0;
        (offset * MULTISHOT_SPREAD, offset * 2.0)
    }

    /// Vanilla `CrossbowItem#getShotVelocity`.
    #[must_use]
    pub fn shot_velocity(projectile: &ItemStack) -> f32 {
        if projectile.is(&*vanilla_items::FIREWORK_ROCKET) {
            FIREWORK_SHOT_VELOCITY
        } else {
            ARROW_SHOT_VELOCITY
        }
    }

    /// Vanilla `CrossbowItem#tryLoadProjectiles`.
    ///
    /// Prefers arrows over firework rockets, draws one item per loaded
    /// projectile and leaves the inventory untouched in creative mode.
    fn try_load_projectiles(stack: &mut ItemStack, player: &Player, creative: bool) -> bool {
        let count = usize::try_from(Self::projectile_count(stack)).unwrap_or(1);
        let mut templates: Vec<ItemStackTemplate> = Vec::with_capacity(count);

        {
            let inventory = Arc::clone(&player.inventory);
            let mut inventory = inventory.lock();
            let Some(slot) = inventory
                .get_items()
                .iter()
                .position(ArrowItem::is_arrow)
                .or_else(|| {
                    inventory
                        .get_items()
                        .iter()
                        .position(|candidate| candidate.is(&*vanilla_items::FIREWORK_ROCKET))
                })
            else {
                return false;
            };

            for _ in 0..count {
                let candidate = inventory.get_items()[slot].clone();
                if candidate.is_empty() {
                    break;
                }
                let Ok(template) = ItemStackTemplate::from_stack(&candidate.copy_with_count(1))
                else {
                    break;
                };
                templates.push(template);
                if !creative {
                    inventory.with_item_mut(slot, ItemStack::shrink_one);
                }
            }
        }

        let Ok(charged) = ChargedProjectiles::new(templates) else {
            return false;
        };
        if charged.items().is_empty() {
            return false;
        }
        stack.set(vanilla_components::CHARGED_PROJECTILES, charged);
        true
    }

    /// Fires every loaded projectile.
    fn shoot(context: &mut UseItemContext) -> InteractionResult {
        let world = context.world;
        let player = context.player;
        let creative = player.game_mode() == GameType::Creative;

        let projectiles: Vec<ItemStack> = context.inv.with_item(|stack| {
            Self::charged(stack).map_or_else(Vec::new, |charged| {
                charged
                    .items()
                    .iter()
                    .map(ItemStackTemplate::create)
                    .collect()
            })
        });
        if projectiles.is_empty() {
            return InteractionResult::Fail;
        }

        let Some(shooter) = world.get_entity_by_id(player.id()) else {
            return InteractionResult::Fail;
        };

        let position = player.position() + DVec3::new(0.0, 1.5, 0.0);
        let count = i32::try_from(projectiles.len()).unwrap_or(1);
        let pickup = if creative {
            ArrowPickup::CreativeOnly
        } else {
            ArrowPickup::Allowed
        };

        for (index, projectile) in projectiles.iter().enumerate() {
            let (yaw_offset, pitch_offset) = Self::spread(i32::try_from(index).unwrap_or(0), count);
            let (yaw, pitch) = shooter.rotation();
            let velocity = Self::shot_velocity(projectile);

            if projectile.is(&*vanilla_items::FIREWORK_ROCKET) {
                let rocket = FireworkRocketEntity::launched(
                    &vanilla_entities::FIREWORK_ROCKET,
                    next_entity_id(),
                    position,
                    Arc::downgrade(world),
                    projectile.clone(),
                );
                rocket.set_owner_entity(Some(&shooter));
                rocket.shoot_from_rotation(
                    shooter.as_ref(),
                    pitch + pitch_offset,
                    yaw + yaw_offset,
                    0.0,
                    velocity,
                    1.0,
                );
                Self::spawn(world, Arc::new(rocket));
                continue;
            }

            if projectile.is(&*vanilla_items::SPECTRAL_ARROW) {
                let arrow = SpectralArrowItem::create_arrow(
                    projectile,
                    position,
                    world,
                    Some(&shooter),
                    pickup,
                );
                Self::launch(
                    arrow,
                    world,
                    &shooter,
                    yaw + yaw_offset,
                    pitch + pitch_offset,
                );
            } else {
                let arrow =
                    ArrowItem::create_arrow(projectile, position, world, Some(&shooter), pickup);
                Self::launch(
                    arrow,
                    world,
                    &shooter,
                    yaw + yaw_offset,
                    pitch + pitch_offset,
                );
            }
        }

        // Vanilla `CrossbowItem.fireProjectiles` fires `shot_crossbow` with the
        // crossbow stack, which `adventure/arbalistic` requires.
        context.player.trigger_advancement_with_context(
            &steel_registry::vanilla_criteria_triggers::SHOT_CROSSBOW,
            &crate::player::advancement::predicate::TriggerContext::item(
                &context.inv.with_item(|stack| stack.item().key.clone()),
            ),
        );
        let item = context.inv.with_item(|stack| stack.item());
        context.inv.with_item(|stack| {
            stack.remove(vanilla_components::CHARGED_PROJECTILES);
            stack.hurt_and_break(1, creative);
        });
        player.award_stat(&vanilla_stat_types::ITEM_USED, item);
        world.play_sound(
            &sound_events::ITEM_CROSSBOW_SHOOT,
            SoundSource::Players,
            player.block_position(),
            1.0,
            1.0,
            None,
        );

        InteractionResult::Consume
    }

    fn launch<A: AbstractArrow + 'static>(
        arrow: Arc<A>,
        world: &Arc<World>,
        shooter: &SharedEntity,
        yaw: f32,
        pitch: f32,
    ) {
        arrow.set_arrow_damage(2.0);
        arrow.set_shot_from_crossbow();
        arrow.shoot_from_rotation(shooter.as_ref(), pitch, yaw, 0.0, ARROW_SHOT_VELOCITY, 1.0);
        Self::spawn(world, arrow);
    }

    fn spawn<E: Entity + 'static>(world: &Arc<World>, entity: Arc<E>) {
        let shared: SharedEntity = entity;
        if let Err(error) = world.try_add_entity(shared) {
            log::debug!("failed to spawn crossbow projectile: {error}");
        }
    }
}

impl ItemBehavior for CrossbowItem {
    fn use_item(&self, context: &mut UseItemContext) -> InteractionResult {
        if context.inv.with_item(|stack| Self::is_charged(stack)) {
            return Self::shoot(context);
        }
        context.player.start_using_item(context.hand);
        InteractionResult::Consume
    }

    fn get_use_animation(&self, _stack: &ItemStack) -> ItemUseAnimation {
        ItemUseAnimation::Crossbow
    }

    fn get_use_duration(&self, stack: &ItemStack, _user: &dyn LivingEntity) -> i32 {
        Self::charge_duration(stack)
    }

    fn use_on_release(&self, _stack: &ItemStack) -> bool {
        true
    }

    fn on_use_tick(
        &self,
        _world: &Arc<World>,
        user: LivingEntityRef<'_>,
        stack: &mut ItemStack,
        ticks_remaining: i32,
    ) {
        if Self::is_charged(stack) {
            return;
        }
        let Some(player) = user.living().as_player() else {
            return;
        };
        let duration = Self::charge_duration(stack);
        let elapsed = duration - ticks_remaining;

        // Vanilla plays the loading sounds while the charge advances.
        if elapsed == 1 {
            player.play_sound(&sound_events::ITEM_CROSSBOW_LOADING_START, 0.5, 1.0);
        } else if elapsed == duration / 2 {
            player.play_sound(&sound_events::ITEM_CROSSBOW_LOADING_MIDDLE, 0.5, 1.0);
        }

        if ticks_remaining > 1 {
            return;
        }

        let creative = player.game_mode() == GameType::Creative;
        if Self::try_load_projectiles(stack, player, creative) {
            player.play_sound(&sound_events::ITEM_CROSSBOW_LOADING_END, 1.0, 1.0);
        }
    }

    fn finish_using(
        &self,
        stack: &mut ItemStack,
        _world: &Arc<World>,
        user: LivingEntityRef<'_>,
    ) -> ItemStack {
        if !Self::is_charged(stack)
            && let Some(player) = user.living().as_player()
        {
            let creative = player.game_mode() == GameType::Creative;
            if Self::try_load_projectiles(stack, player, creative) {
                player.play_sound(&sound_events::ITEM_CROSSBOW_LOADING_END, 1.0, 1.0);
            }
        }
        stack.copy_with_count(stack.count())
    }

    fn release_using(
        &self,
        _stack: &mut ItemStack,
        _world: &Arc<World>,
        _user: &dyn LivingEntity,
        _time_left: i32,
    ) -> bool {
        // Releasing an uncharged crossbow only cancels the charge.
        false
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::data_components::components::ChargedProjectiles;
    use steel_registry::data_components::vanilla_components;
    use steel_registry::item_stack::ItemStack;
    use steel_registry::item_stack_template::ItemStackTemplate;
    use steel_registry::{init_vanilla_registry, vanilla_items};

    use super::{ARROW_SHOT_VELOCITY, CrossbowItem, FIREWORK_SHOT_VELOCITY};

    #[test]
    fn charge_duration_shrinks_with_quick_charge() {
        init_vanilla_registry();
        let stack = ItemStack::new(&*vanilla_items::CROSSBOW);
        assert_eq!(CrossbowItem::charge_duration(&stack), 25);

        let mut quick = ItemStack::new(&*vanilla_items::CROSSBOW);
        quick.set_enchantments(
            &[(steel_utils::Identifier::vanilla_static("quick_charge"), 2)],
            false,
        );
        assert_eq!(CrossbowItem::charge_duration(&quick), 15);
    }

    #[test]
    fn multishot_loads_three_projectiles() {
        init_vanilla_registry();
        let stack = ItemStack::new(&*vanilla_items::CROSSBOW);
        assert_eq!(CrossbowItem::projectile_count(&stack), 1);

        let mut multishot = ItemStack::new(&*vanilla_items::CROSSBOW);
        multishot.set_enchantments(
            &[(steel_utils::Identifier::vanilla_static("multishot"), 1)],
            false,
        );
        assert_eq!(CrossbowItem::projectile_count(&multishot), 3);
    }

    #[test]
    fn multishot_spread_is_symmetric() {
        assert_eq!(CrossbowItem::spread(0, 1), (0.0, 0.0));
        let (left_yaw, left_pitch) = CrossbowItem::spread(0, 3);
        let (center_yaw, center_pitch) = CrossbowItem::spread(1, 3);
        let (right_yaw, right_pitch) = CrossbowItem::spread(2, 3);
        assert_eq!((center_yaw, center_pitch), (0.0, 0.0));
        assert!((left_yaw + right_yaw).abs() < f32::EPSILON);
        assert!((left_pitch + right_pitch).abs() < f32::EPSILON);
        assert!(left_yaw < 0.0 && right_yaw > 0.0);
    }

    #[test]
    fn charged_projectiles_round_trip_through_the_component() {
        init_vanilla_registry();
        let mut stack = ItemStack::new(&*vanilla_items::CROSSBOW);
        assert!(!CrossbowItem::is_charged(&stack));

        let arrow = ItemStackTemplate::new(&*vanilla_items::ARROW);
        let charged = ChargedProjectiles::new(vec![arrow]).expect("one arrow fits");
        stack.set(vanilla_components::CHARGED_PROJECTILES, charged);

        assert!(CrossbowItem::is_charged(&stack));
        assert_eq!(
            CrossbowItem::charged(&stack).map(|charged| charged.items().len()),
            Some(1)
        );

        stack.remove(vanilla_components::CHARGED_PROJECTILES);
        assert!(!CrossbowItem::is_charged(&stack));
    }

    #[test]
    fn firework_rockets_use_the_slower_shot_velocity() {
        init_vanilla_registry();
        let arrow = ItemStack::new(&*vanilla_items::ARROW);
        let rocket = ItemStack::new(&*vanilla_items::FIREWORK_ROCKET);
        assert_eq!(CrossbowItem::shot_velocity(&arrow), ARROW_SHOT_VELOCITY);
        assert_eq!(CrossbowItem::shot_velocity(&rocket), FIREWORK_SHOT_VELOCITY);
    }

    #[test]
    fn charge_duration_never_drops_below_one_tick() {
        init_vanilla_registry();
        let mut stack = ItemStack::new(&*vanilla_items::CROSSBOW);
        stack.set_enchantments(
            &[(steel_utils::Identifier::vanilla_static("quick_charge"), 5)],
            false,
        );
        assert_eq!(CrossbowItem::charge_duration(&stack), 1);
    }
}
