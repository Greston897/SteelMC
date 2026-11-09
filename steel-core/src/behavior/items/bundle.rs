//! Bundle item behavior (`BundleItem`).
//!
//! Mirrors vanilla `BundleItem`: bundles hold up to one unit of item weight in
//! their `bundle_contents` component, accept items with a left click while the
//! bundle is on the cursor or in the clicked slot, hand the selected stack back
//! with a right click, and spill their contents while the use key is held.

use std::sync::Arc;

use steel_macros::item_behavior;
use steel_registry::data_components::components::BundleContents;
use steel_registry::data_components::vanilla_components::BUNDLE_CONTENTS;
use steel_registry::item_stack::ItemStack;
use glam::DVec3;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::{sound_events, vanilla_stat_types};

use crate::behavior::context::{InteractionResult, UseItemContext};
use crate::behavior::item::{ItemBehavior, ItemUseAnimation};
use crate::entity::entities::ItemEntity;
use crate::entity::{Entity as _, LivingEntity, LivingEntityRef};
use crate::inventory::click::MouseButton;
use crate::inventory::lock::ContainerLockGuard;
use crate::inventory::slots::Slot;
use crate::player::Player;
use crate::world::World;

/// Vanilla `BundleItem.MAX_USE_TIME`.
const MAX_USE_TIME: i32 = 200;

/// Vanilla `BundleItem.TOOLTIP_ITEM_COLUMNS`, reused as the delay before the
/// held bundle starts emptying itself tick by tick.
const DROP_CONTENTS_DELAY: i32 = 10;

/// Vanilla volume used by every bundle interaction sound.
const SOUND_VOLUME: f32 = 0.8;

/// Behavior for every bundle item (the plain and the sixteen dyed bundles).
#[item_behavior(class = "BundleItem")]
pub struct BundleItem;

impl BundleItem {
    fn contents(stack: &ItemStack) -> Option<BundleContents> {
        stack.get(BUNDLE_CONTENTS).cloned()
    }

    fn play(player: &Player, sound: SoundEventRef) {
        player.play_sound(sound, SOUND_VOLUME, 0.8 + rand::random::<f32>() * 0.4);
    }

    fn play_insert(player: &Player) {
        Self::play(player, &sound_events::ITEM_BUNDLE_INSERT);
    }

    fn play_remove_one(player: &Player) {
        Self::play(player, &sound_events::ITEM_BUNDLE_REMOVE_ONE);
    }

    fn play_insert_fail(player: &Player) {
        player.play_sound(&sound_events::ITEM_BUNDLE_INSERT_FAIL, 1.0, 1.0);
    }

    fn play_drop_contents(player: &Player) {
        Self::play(player, &sound_events::ITEM_BUNDLE_DROP_CONTENTS);
    }

    /// Vanilla `Builder.add(Slot, PlayerEntity)`: takes what still fits from the
    /// slot and stores it, returning how many items moved.
    fn insert_from_slot(
        contents: &mut BundleContents,
        slot: &dyn Slot,
        player: &Player,
        guard: &mut ContainerLockGuard,
    ) -> i32 {
        let slot_stack = slot.get_item(guard).clone();
        if !BundleContents::can_be_bundled(&slot_stack) {
            return 0;
        }

        let max_allowed = contents.max_allowed(&slot_stack);
        if max_allowed <= 0 {
            return 0;
        }

        let mut taken = slot.safe_take(guard, slot_stack.count(), max_allowed, player);
        if taken.is_empty() {
            return 0;
        }

        let inserted = contents.add_stack(&mut taken);
        if !taken.is_empty() {
            let _ = slot.safe_insert(guard, taken, i32::MAX);
        }
        inserted
    }

    /// Writes updated contents back into the bundle stack stored in `slot`.
    fn write_contents(slot: &dyn Slot, guard: &mut ContainerLockGuard, contents: BundleContents) {
        let previous = slot.get_item(guard).clone();
        let mut updated = previous.clone();
        updated.set(BUNDLE_CONTENTS, contents);
        slot.set_by_player(guard, updated, &previous);
    }

    fn clear_selection(slot: &dyn Slot, guard: &mut ContainerLockGuard) {
        let Some(mut contents) = slot.get_item(guard).get(BUNDLE_CONTENTS).cloned() else {
            return;
        };
        contents.set_selected_stack_index(BundleContents::NO_SELECTED_SLOT);
        Self::write_contents(slot, guard, contents);
    }

    /// Vanilla `dropFirstBundledStack`: pops one stack and throws it out.
    fn drop_first_stack(stack: &mut ItemStack, player: &Player) -> bool {
        let Some(mut contents) = Self::contents(stack) else {
            return false;
        };
        if contents.is_empty() {
            return false;
        }
        let Some(removed) = contents.remove_selected() else {
            return false;
        };

        Self::play_remove_one(player);
        stack.set(BUNDLE_CONTENTS, contents);
        let _ = player.spawn_dropped_item(removed, true, false);
        true
    }
}

impl ItemBehavior for BundleItem {
    fn use_item(&self, context: &mut UseItemContext) -> InteractionResult {
        // Vanilla `BundleItem.use`: only starts the held-use animation; the
        // stacks are dropped by `usageTick` while the key stays down.
        context.player.start_using_item(context.hand);
        InteractionResult::Success
    }

    fn get_use_animation(&self, _stack: &ItemStack) -> ItemUseAnimation {
        ItemUseAnimation::Bundle
    }

    fn get_use_duration(&self, _stack: &ItemStack, _user: &dyn LivingEntity) -> i32 {
        MAX_USE_TIME
    }

    fn on_use_tick(
        &self,
        _world: &Arc<World>,
        user: LivingEntityRef<'_>,
        stack: &mut ItemStack,
        ticks_remaining: i32,
    ) {
        let Some(player) = user.living().as_player() else {
            return;
        };

        // Vanilla `usageTick`: the first tick always drops, then every other
        // tick once the initial delay has elapsed.
        let max_use_ticks = self.get_use_duration(stack, user.living());
        let is_first_tick = ticks_remaining == max_use_ticks;
        if !is_first_tick
            && (ticks_remaining >= max_use_ticks - DROP_CONTENTS_DELAY || ticks_remaining % 2 != 0)
        {
            return;
        }

        if Self::drop_first_stack(stack, player) {
            Self::play_drop_contents(player);
            player.award_stat(&vanilla_stat_types::ITEM_USED, stack.item());
        }
    }

    fn on_item_entity_destroyed(&self, entity: &ItemEntity) {
        // Vanilla `BundleItem.onItemEntityDestroyed`: clear the component and
        // spill every stored stack at the dying entity's position.
        let Some(world) = entity.level() else {
            return;
        };
        let mut stack = entity.get_item();
        let Some(mut contents) = Self::contents(&stack) else {
            return;
        };

        let dropped = contents.drain_items();
        stack.set(BUNDLE_CONTENTS, contents);
        entity.set_item(stack);

        let position = entity.position();
        for item in dropped {
            if let Some(dropped) = world.spawn_item_with_velocity(position, item, DVec3::ZERO) {
                dropped.set_default_pickup_delay();
            }
        }
    }

    fn on_stack_clicked(
        &self,
        stack: &mut ItemStack,
        slot: &dyn Slot,
        click: MouseButton,
        player: &Player,
        guard: &mut ContainerLockGuard,
    ) -> bool {
        let Some(mut contents) = Self::contents(stack) else {
            return false;
        };
        let slot_stack = slot.get_item(guard).clone();

        if click == MouseButton::Left && !slot_stack.is_empty() {
            if Self::insert_from_slot(&mut contents, slot, player, guard) > 0 {
                Self::play_insert(player);
            } else {
                Self::play_insert_fail(player);
            }

            stack.set(BUNDLE_CONTENTS, contents);
            return true;
        }

        if click == MouseButton::Right && slot_stack.is_empty() {
            if let Some(removed) = contents.remove_selected() {
                let mut remainder = slot.safe_insert(guard, removed, i32::MAX);
                if remainder.is_empty() {
                    Self::play_remove_one(player);
                } else {
                    contents.add_stack(&mut remainder);
                }
            }

            stack.set(BUNDLE_CONTENTS, contents);
            return true;
        }

        false
    }

    fn on_clicked(
        &self,
        slot: &dyn Slot,
        click: MouseButton,
        player: &Player,
        guard: &mut ContainerLockGuard,
        cursor: &mut ItemStack,
    ) -> bool {
        if click == MouseButton::Left && cursor.is_empty() {
            // Vanilla clears the selection and lets the default pickup run.
            Self::clear_selection(slot, guard);
            return false;
        }

        let Some(mut contents) = slot.get_item(guard).get(BUNDLE_CONTENTS).cloned() else {
            return false;
        };

        if click == MouseButton::Left && !cursor.is_empty() {
            if slot.may_pickup(guard, player) && contents.add_stack(cursor) > 0 {
                Self::play_insert(player);
            } else {
                Self::play_insert_fail(player);
            }

            Self::write_contents(slot, guard, contents);
            return true;
        }

        if click == MouseButton::Right && cursor.is_empty() {
            if slot.may_pickup(guard, player)
                && let Some(removed) = contents.remove_selected()
            {
                Self::play_remove_one(player);
                *cursor = removed;
            }

            Self::write_contents(slot, guard, contents);
            return true;
        }

        contents.set_selected_stack_index(BundleContents::NO_SELECTED_SLOT);
        Self::write_contents(slot, guard, contents);
        false
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use glam::DVec3;
    use steel_registry::data_components::components::BundleContents;
    use steel_registry::data_components::vanilla_components::BUNDLE_CONTENTS;
    use steel_registry::{init_vanilla_registry, item_stack::ItemStack, vanilla_items};
    use steel_utils::{ChunkPos, Downcast as _, WorldAabb};

    use super::BundleItem;
    use crate::behavior::init_behaviors;
    use crate::entity::Entity as _;
    use crate::entity::entities::ItemEntity;
    use crate::inventory::click::{Click, MouseButton};
    use crate::inventory::container::Container as _;
    use crate::inventory::menu::kinds::inventory_menu;
    use crate::test_support::{TestPlayerBuilder, fresh_test_world, insert_ready_full_chunk};

    /// Menu slot index of the first hotbar slot in the player inventory menu.
    const HOTBAR_SLOT: usize = 36;

    fn bundle_stack(contents: BundleContents) -> ItemStack {
        let mut stack = ItemStack::new(&vanilla_items::BUNDLE);
        stack.set(BUNDLE_CONTENTS, contents);
        stack
    }

    fn contents_of(stack: &ItemStack) -> BundleContents {
        stack
            .get(BUNDLE_CONTENTS)
            .cloned()
            .expect("the cursor keeps its bundle contents component")
    }

    fn dirt(count: i32) -> ItemStack {
        ItemStack::with_count(&vanilla_items::DIRT, count)
    }

    #[test]
    fn left_click_with_a_cursor_bundle_absorbs_the_clicked_slot() {
        init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("bundle_insert_from_slot");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let player = TestPlayerBuilder::new(Arc::clone(world), "Bundler", 1).build();

        player.inventory.lock().set_item(0, dirt(64));

        let mut menu = inventory_menu(Arc::clone(&player.inventory));
        *menu.behavior_mut().carried_mut() = bundle_stack(BundleContents::empty());

        menu.clicked(
            Click::Pickup {
                slot: HOTBAR_SLOT,
                button: MouseButton::Left,
            },
            &player,
        );

        assert!(player.inventory.lock().get_item(0).is_empty());
        let contents = contents_of(menu.behavior().carried());
        assert_eq!(contents.items().len(), 1);
        assert_eq!(contents.items()[0].count(), 64);
        assert!((contents.occupancy() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn right_click_with_a_cursor_bundle_refills_an_empty_slot() {
        init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("bundle_extract_to_slot");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let player = TestPlayerBuilder::new(Arc::clone(world), "Bundler", 1).build();

        let mut contents = BundleContents::empty();
        let mut full = dirt(64);
        assert_eq!(contents.add_stack(&mut full), 64);

        let mut menu = inventory_menu(Arc::clone(&player.inventory));
        *menu.behavior_mut().carried_mut() = bundle_stack(contents);

        menu.clicked(
            Click::Pickup {
                slot: HOTBAR_SLOT,
                button: MouseButton::Right,
            },
            &player,
        );

        assert_eq!(player.inventory.lock().get_item(0).count(), 64);
        assert!(contents_of(menu.behavior().carried()).is_empty());
    }

    #[test]
    fn shulker_boxes_cannot_be_stuffed_into_a_bundle() {
        init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("bundle_rejects_shulker_box");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let player = TestPlayerBuilder::new(Arc::clone(world), "Bundler", 1).build();

        player
            .inventory
            .lock()
            .set_item(0, ItemStack::new(&vanilla_items::SHULKER_BOX));

        let mut menu = inventory_menu(Arc::clone(&player.inventory));
        *menu.behavior_mut().carried_mut() = bundle_stack(BundleContents::empty());

        menu.clicked(
            Click::Pickup {
                slot: HOTBAR_SLOT,
                button: MouseButton::Left,
            },
            &player,
        );

        assert!(
            player
                .inventory
                .lock()
                .get_item(0)
                .is(&vanilla_items::SHULKER_BOX)
        );
        assert!(contents_of(menu.behavior().carried()).is_empty());
    }

    #[test]
    fn dropping_a_bundled_stack_spawns_an_item_entity() {
        init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("bundle_drop_contents");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let player = TestPlayerBuilder::new(Arc::clone(world), "Bundler", 1).build();
        player.base().set_position_local(DVec3::new(0.5, 64.0, 0.5));

        let mut contents = BundleContents::empty();
        let mut seven = dirt(7);
        assert_eq!(contents.add_stack(&mut seven), 7);

        let mut stack = bundle_stack(contents);
        assert!(BundleItem::drop_first_stack(&mut stack, &player));
        assert!(contents_of(&stack).is_empty());

        let search = WorldAabb::new(-2.0, 62.0, -2.0, 2.0, 68.0, 2.0);
        let dropped = world.get_entities_in_aabb_matching(&search, |entity| {
            entity.as_ref().downcast_ref::<ItemEntity>().is_some()
        });
        assert_eq!(dropped.len(), 1);
        let Some(item) = dropped[0].as_ref().downcast_ref::<ItemEntity>() else {
            panic!("dropped entity should be an item entity");
        };
        assert!(item.get_item().is(&vanilla_items::DIRT));
        assert_eq!(item.get_item().count(), 7);
    }

    #[test]
    fn destroying_a_bundle_item_entity_spills_every_stored_stack() {
        init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("bundle_item_entity_destroyed");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let player = TestPlayerBuilder::new(Arc::clone(world), "Bundler", 1).build();
        player.base().set_position_local(DVec3::new(0.5, 64.0, 0.5));

        let mut contents = BundleContents::empty();
        let mut five = dirt(5);
        let mut stone = ItemStack::with_count(&vanilla_items::STONE, 3);
        assert_eq!(contents.add_stack(&mut five), 5);
        assert_eq!(contents.add_stack(&mut stone), 3);

        let position = DVec3::new(0.5, 64.0, 0.5);
        let bundle = world
            .spawn_item_with_velocity(position, bundle_stack(contents), DVec3::ZERO)
            .expect("bundle item entity spawns");
        assert_eq!(contents_of(&bundle.get_item()).items().len(), 2);

        BundleItem.on_item_entity_destroyed(&bundle);

        assert!(contents_of(&bundle.get_item()).is_empty());
        let spilled = world.get_entities_in_aabb_matching(
            &WorldAabb::new(-2.0, 62.0, -2.0, 2.0, 68.0, 2.0),
            |entity| {
                entity
                    .as_ref()
                    .downcast_ref::<ItemEntity>()
                    .is_some_and(|item| item.id() != bundle.id())
            },
        );
        assert_eq!(spilled.len(), 2);
    }
}
