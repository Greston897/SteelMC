//! Debug stick item behavior (`DebugStickItem`).
//!
//! Mirrors vanilla `DebugStickItem`: right-clicking a block cycles the value of
//! the property selected for that block, and left-clicking cycles which property
//! is selected. The stick never breaks blocks: `canMine` always refuses and uses
//! that call to rotate the selection. Both paths require vanilla's
//! `isCreativeLevelTwoOp` check and report on the action bar.

use std::cmp::Ordering;
use std::sync::Arc;

use steel_macros::item_behavior;
use steel_registry::REGISTRY;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::properties::Property;
use steel_registry::data_components::vanilla_components::DEBUG_STICK_STATE;
use steel_utils::types::{GameType, UpdateFlags};
use steel_utils::{BlockPos, BlockStateId};
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;

use crate::behavior::context::{CanMineContext, InteractionResult, InventoryAccess, UseOnContext};
use crate::behavior::item::ItemBehavior;
use crate::player::Player;
use crate::world::World;

/// Vanilla `DebugStickItem.BLOCK_UPDATE_FLAGS` (`NOTIFY_LISTENERS | FORCE_STATE`).
fn block_update_flags() -> UpdateFlags {
    UpdateFlags::UPDATE_CLIENTS | UpdateFlags::UPDATE_KNOWN_SHAPE
}

/// Behavior for the operator-only debug stick.
#[item_behavior(class = "DebugStickItem")]
pub struct DebugStickItem;

impl DebugStickItem {
    /// Vanilla `Item#translationKey` for the debug stick.
    const TRANSLATION_KEY: &'static str = "item.minecraft.debug_stick";

    /// Vanilla `DebugStickItem#applyDebugAction`.
    ///
    /// `update` selects the property-value cycling (right click) instead of the
    /// property selection (left click). Returns whether the action happened.
    fn apply_debug_action(
        player: &Player,
        world: &Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        update: bool,
        inv: &InventoryAccess,
    ) -> bool {
        if !Self::is_creative_level_two_op(player) {
            return false;
        }

        Self::perform_debug_action(player, world, pos, state, update, inv)
    }

    /// The permission-checked body of [`Self::apply_debug_action`].
    ///
    /// Kept separate so the property-cycling mechanics stay testable without a
    /// full server (operator status needs a `Server` in Steel).
    fn perform_debug_action(
        player: &Player,
        world: &Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        update: bool,
        inv: &InventoryAccess,
    ) -> bool {
        let Some(block) = REGISTRY.blocks.by_state_id(state) else {
            return false;
        };

        if block.properties.is_empty() {
            Self::send_message(
                player,
                ".empty",
                Box::new([TextComponent::plain(block.key.to_string())]),
            );
            return false;
        }

        let Some(debug_state) = inv.with_item(|stack| stack.get(DEBUG_STICK_STATE).cloned()) else {
            return false;
        };

        let selected = debug_state.get(block);
        let inverse = player.is_secondary_use_active();

        if update {
            let Some(property_name) =
                selected.or_else(|| block.properties.first().map(|property| property.get_name()))
            else {
                return false;
            };
            let Some(property) = find_property(block, property_name) else {
                return false;
            };
            let Some(current) = REGISTRY.blocks.get_property_str(state, property_name) else {
                return false;
            };

            let values = ordered_value_names(property);
            let Some(value) = cycle(&values, Some(&current), inverse).copied() else {
                return false;
            };
            let Some(updated) = REGISTRY
                .blocks
                .try_set_property_by_name(state, property_name, value)
            else {
                return false;
            };

            world.set_block(pos, updated, block_update_flags());
            Self::send_message(
                player,
                ".update",
                Box::new([
                    TextComponent::plain(property_name.to_owned()),
                    TextComponent::plain(value.to_owned()),
                ]),
            );
        } else {
            let names = property_names(block);
            let Some(name) = cycle(&names, selected.as_ref(), inverse).copied() else {
                return false;
            };

            inv.with_item(|stack| stack.set(DEBUG_STICK_STATE, debug_state.with(block, name)));
            let value = REGISTRY
                .blocks
                .get_property_str(state, name)
                .unwrap_or_default()
                .to_owned();
            Self::send_message(
                player,
                ".select",
                Box::new([
                    TextComponent::plain(name.to_owned()),
                    TextComponent::plain(value),
                ]),
            );
        }

        true
    }

    /// Vanilla `PlayerEntity#isCreativeLevelTwoOp`.
    ///
    /// Steel models operator levels through its permission groups, so the
    /// creative requirement plus operator status stands in for level two.
    fn is_creative_level_two_op(player: &Player) -> bool {
        player.game_mode() == GameType::Creative && player.is_operator()
    }

    /// Sends a debug-stick action bar message (`sendMessageToClient(msg, true)`).
    fn send_message(player: &Player, suffix: &str, args: Box<[TextComponent]>) {
        player.send_overlay_message(
            &TranslatedMessage {
                key: format!("{}{suffix}", Self::TRANSLATION_KEY).into(),
                fallback: None,
                args: Some(args),
            }
            .component(),
        );
    }
}

impl ItemBehavior for DebugStickItem {
    fn use_on(&self, context: &mut UseOnContext) -> InteractionResult {
        let pos = context.hit_result.block_pos;
        let state = context.world.get_block_state(pos);

        if !Self::apply_debug_action(
            context.player,
            context.world,
            pos,
            state,
            true,
            &context.inv,
        ) {
            return InteractionResult::Fail;
        }

        InteractionResult::Success
    }

    fn can_mine(&self, context: &CanMineContext<'_>) -> bool {
        Self::apply_debug_action(
            context.player,
            context.world,
            context.pos,
            context.state,
            false,
            &context.inv,
        );

        false
    }
}

/// Looks up one property of `block` by name.
fn find_property(block: BlockRef, name: &str) -> Option<&'static dyn Property> {
    block
        .properties
        .iter()
        .copied()
        .find(|property| property.get_name() == name)
}

/// Property names in vanilla state-manager order.
fn property_names(block: BlockRef) -> Vec<&'static str> {
    block
        .properties
        .iter()
        .map(|property| property.get_name())
        .collect()
}

/// Possible values of `property` in vanilla `Comparable` order (`Property#getValues`).
fn ordered_value_names(property: &dyn Property) -> Vec<&str> {
    let mut names = property.get_possible_value_names().to_vec();
    names.sort_by(|left, right| {
        property
            .compare_value_names(left, right)
            .unwrap_or(Ordering::Equal)
    });
    names
}

/// Vanilla `Util#next` / `Util#previous`: wraps around the ends and falls back to
/// the first (or last) element when `current` is absent or unknown.
fn cycle<'a, T: PartialEq>(values: &'a [T], current: Option<&'a T>, inverse: bool) -> Option<&'a T> {
    let first = values.first()?;
    let last = values.last()?;

    let Some(current) = current else {
        return Some(if inverse { last } else { first });
    };

    let Some(index) = values.iter().position(|value| value == current) else {
        return Some(if inverse { last } else { first });
    };

    if inverse {
        Some(if index == 0 {
            last
        } else {
            &values[index - 1]
        })
    } else {
        Some(if index + 1 == values.len() {
            first
        } else {
            &values[index + 1]
        })
    }
}

/// Reads the debug stick's stored selection for `block`, if any.
#[cfg(test)]
fn selected_property(
    stack: &steel_registry::item_stack::ItemStack,
    block: BlockRef,
) -> Option<&'static str> {
    stack.get(DEBUG_STICK_STATE).and_then(|state| state.get(block))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use glam::DVec3;
    use steel_registry::blocks::properties::Direction;
    use steel_registry::data_components::components::DebugStickState;
    use steel_registry::items::item::BlockHitResult;
    use steel_registry::{
        REGISTRY, init_vanilla_registry, item_stack::ItemStack, vanilla_blocks, vanilla_items,
    };
    use steel_utils::types::{InteractionHand, UpdateFlags};
    use steel_utils::{BlockPos, ChunkPos};

    use super::{
        DebugStickItem, cycle, find_property, ordered_value_names, property_names,
        selected_property,
    };
    use crate::behavior::ItemBehavior as _;
    use crate::behavior::context::{
        CanMineContext, InteractionResult, InventoryAccess, UseOnContext,
    };
    use crate::behavior::init_behaviors;
    use crate::test_support::{TestPlayerBuilder, fresh_test_world, insert_ready_full_chunk};

    /// Places a lever (three properties: `face`, `facing`, `powered`) and returns
    /// the world plus the placed state.
    fn lever_fixture(
        name: &str,
    ) -> (
        crate::test_support::TestWorld,
        Arc<crate::player::Player>,
        BlockPos,
        steel_utils::BlockStateId,
    ) {
        init_vanilla_registry();
        init_behaviors();

        let world_fixture = fresh_test_world(name);
        let world = Arc::clone(&world_fixture.world);
        let pos = BlockPos::new(0, 64, 0);
        insert_ready_full_chunk(&world, ChunkPos::from_block_pos(pos));

        let state = vanilla_blocks::LEVER.default_state();
        assert!(world.set_block(pos, state, UpdateFlags::UPDATE_ALL));

        let player = TestPlayerBuilder::new(Arc::clone(&world), "DebugStickTester", 1).build();
        player.set_client_loaded(true);
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::new(&vanilla_items::DEBUG_STICK));

        (world_fixture, player, pos, state)
    }

    fn use_on_context<'a>(
        world: &'a Arc<crate::world::World>,
        player: &'a Arc<crate::player::Player>,
        pos: BlockPos,
    ) -> UseOnContext<'a> {
        UseOnContext::new(
            player,
            InteractionHand::MainHand,
            BlockHitResult {
                location: DVec3::new(0.5, 64.5, 0.5),
                direction: Direction::Up,
                block_pos: pos,
                miss: false,
                inside: false,
                world_border_hit: false,
            },
            world,
            Arc::clone(&player.inventory),
        )
    }

    #[test]
    fn debug_stick_refuses_non_operators_and_leaves_the_block_alone() {
        let (fixture, player, pos, state) = lever_fixture("debug_stick_non_operator");
        let world = &fixture.world;

        let mut context = use_on_context(world, &player, pos);
        assert_eq!(DebugStickItem.use_on(&mut context), InteractionResult::Fail);

        assert_eq!(world.get_block_state(pos), state);
        let stack = player.inventory.lock().get_selected_item().clone();
        assert_eq!(selected_property(&stack, &vanilla_blocks::LEVER), None);
    }

    #[test]
    fn debug_stick_never_breaks_blocks() {
        let (fixture, player, pos, state) = lever_fixture("debug_stick_never_breaks");
        let world = &fixture.world;

        let can_mine = DebugStickItem.can_mine(&CanMineContext::new(
            &player,
            world,
            pos,
            state,
            Arc::clone(&player.inventory),
            InteractionHand::MainHand,
        ));

        assert!(!can_mine);
        assert_eq!(world.get_block_state(pos), state);
    }

    #[test]
    fn debug_stick_cycle_wraps_in_both_directions() {
        let values = ["a", "b", "c"];

        assert_eq!(cycle(&values, Some(&"a"), false), Some(&"b"));
        assert_eq!(cycle(&values, Some(&"c"), false), Some(&"a"));
        assert_eq!(cycle(&values, Some(&"b"), true), Some(&"a"));
        assert_eq!(cycle(&values, Some(&"a"), true), Some(&"c"));
    }

    #[test]
    fn debug_stick_cycle_defaults_to_the_first_or_last_value() {
        let values = ["a", "b", "c"];

        assert_eq!(cycle(&values, None, false), Some(&"a"));
        assert_eq!(cycle(&values, None, true), Some(&"c"));
        assert_eq!(cycle(&values, Some(&"unknown"), false), Some(&"a"));
        assert_eq!(cycle(&values, Some(&"unknown"), true), Some(&"c"));
        assert_eq!(cycle::<&str>(&[], None, false), None);
    }

    /// Runs the permission-checked body directly so the cycling mechanics are
    /// covered without a `Server` (operator status requires one).
    fn run_action(
        player: &Arc<crate::player::Player>,
        world: &Arc<crate::world::World>,
        pos: BlockPos,
        state: steel_utils::BlockStateId,
        update: bool,
    ) -> bool {
        let inv = InventoryAccess::new(Arc::clone(&player.inventory), InteractionHand::MainHand);
        DebugStickItem::perform_debug_action(player, world, pos, state, update, &inv)
    }

    #[test]
    fn debug_stick_selects_a_property_then_cycles_its_value() {
        let (fixture, player, pos, state) = lever_fixture("debug_stick_cycle_value");
        let world = &fixture.world;
        let first = property_names(&vanilla_blocks::LEVER)[0];

        // Left click selects the first property and leaves the world untouched.
        assert!(run_action(&player, world, pos, state, false));
        let stack = player.inventory.lock().get_selected_item().clone();
        assert_eq!(selected_property(&stack, &vanilla_blocks::LEVER), Some(first));
        assert_eq!(world.get_block_state(pos), state);

        // Right click cycles that property to the next vanilla value.
        let current = REGISTRY
            .blocks
            .get_property_str(state, first)
            .expect("the selected property has a value");
        let values = ordered_value_names(
            find_property(&vanilla_blocks::LEVER, first).expect("property exists"),
        );
        let index = values
            .iter()
            .position(|value| *value == current)
            .expect("current value is one of the property values");
        let expected = values[(index + 1) % values.len()];

        assert!(run_action(&player, world, pos, state, true));
        let updated = world.get_block_state(pos);
        assert_eq!(REGISTRY.blocks.get_property_str(updated, first), Some(expected));

        // Unrelated properties keep their values.
        let other = property_names(&vanilla_blocks::LEVER)[1];
        assert_eq!(
            REGISTRY.blocks.get_property_str(updated, other),
            REGISTRY.blocks.get_property_str(state, other)
        );
    }

    #[test]
    fn debug_stick_cycles_backwards_while_crouching() {
        let (fixture, player, pos, state) = lever_fixture("debug_stick_cycle_backwards");
        let world = &fixture.world;

        assert!(run_action(&player, world, pos, state, false));
        let first = property_names(&vanilla_blocks::LEVER)[0];
        let current = REGISTRY
            .blocks
            .get_property_str(state, first)
            .expect("the selected property has a value");
        let values = ordered_value_names(
            find_property(&vanilla_blocks::LEVER, first).expect("property exists"),
        );
        let index = values
            .iter()
            .position(|value| *value == current)
            .expect("current value is one of the property values");
        let expected = values[(index + values.len() - 1) % values.len()];

        player.set_crouching(true);
        assert!(run_action(&player, world, pos, state, true));

        assert_eq!(
            REGISTRY
                .blocks
                .get_property_str(world.get_block_state(pos), first),
            Some(expected)
        );
    }

    #[test]
    fn debug_stick_reports_blocks_without_properties() {
        let (fixture, player, pos, _state) = lever_fixture("debug_stick_no_properties");
        let world = &fixture.world;
        let stone = vanilla_blocks::STONE.default_state();
        assert!(world.set_block(pos, stone, UpdateFlags::UPDATE_ALL));

        assert!(!run_action(&player, world, pos, stone, true));
        assert!(!run_action(&player, world, pos, stone, false));
        assert_eq!(world.get_block_state(pos), stone);
    }

    #[test]
    fn debug_stick_orders_boolean_values_like_vanilla() {
        init_vanilla_registry();

        let powered = find_property(&vanilla_blocks::LEVER, "powered")
            .expect("levers expose a powered property");
        assert_eq!(ordered_value_names(powered), vec!["false", "true"]);
    }

    #[test]
    fn debug_stick_state_remembers_and_replaces_selections() {
        let empty = DebugStickState::empty();
        assert_eq!(empty.get(&vanilla_blocks::LEVER), None);

        let selected = empty.with(&vanilla_blocks::LEVER, "powered");
        assert_eq!(selected.get(&vanilla_blocks::LEVER), Some("powered"));
        assert_eq!(selected.get(&vanilla_blocks::STONE), None);
        assert_eq!(selected.properties().count(), 1);

        let replaced = selected.with(&vanilla_blocks::LEVER, "face");
        assert_eq!(replaced.get(&vanilla_blocks::LEVER), Some("face"));
        assert_eq!(replaced.properties().count(), 1);
    }
}
