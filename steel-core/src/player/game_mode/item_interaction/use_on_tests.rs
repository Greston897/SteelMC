use std::sync::Arc;

use glam::DVec3;
use steel_registry::block_transformer::BlockTransformerComponent;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::data_components::UseRemainder;
use steel_registry::data_components::vanilla_components::{
    BLOCK_TRANSFORMER, USE_COOLDOWN, USE_REMAINDER, UseCooldown,
};
use steel_registry::item_stack::ItemStack;
use steel_registry::items::ItemRef;
use steel_registry::stat::vanilla_stat_types;
use steel_registry::{ItemStackTemplate, init_vanilla_registry, vanilla_blocks, vanilla_items};
use steel_utils::types::{GameType, InteractionHand, SignTextSlot, UpdateFlags};
use steel_utils::{BlockPos, BlockStateId, ChunkPos, Direction, Downcast as _};
use text_components::TextComponent;
use uuid::Uuid;

use crate::behavior::{
    BlockHitResult, InteractionResult, blocks::facing_text_slot, init_behaviors,
};
use crate::block_entity::SharedBlockEntity;
use crate::block_entity::entities::{SignBlockEntity, SignText};
use crate::entity::{Entity as _, entities::ItemEntity};
use crate::inventory::container::Container as _;
use crate::player::Player;
use crate::test_support::{
    TestPlayerBuilder, TestWorld, fresh_test_world, insert_ready_full_chunk,
};

use super::use_item_on;

struct Interaction {
    world_fixture: TestWorld,
    player: Arc<Player>,
    pos: BlockPos,
}

impl Interaction {
    fn new(name: &'static str) -> Self {
        init_vanilla_registry();
        init_behaviors();
        let world_fixture = fresh_test_world(name);
        let world = &world_fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let player = TestPlayerBuilder::new(Arc::clone(world), "ItemUse", 1).build();
        player.base().set_position_local(DVec3::new(8.5, 64.0, 6.5));
        Self {
            world_fixture,
            player,
            pos: BlockPos::new(8, 64, 8),
        }
    }

    fn block(&self, state: BlockStateId) {
        assert!(
            self.world_fixture
                .world
                .set_block(self.pos, state, UpdateFlags::UPDATE_ALL)
        );
    }

    fn sign(&self, state: BlockStateId, with_text: bool, glowing: bool) -> SharedBlockEntity {
        for direction in [
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ] {
            self.world_fixture.world.set_block(
                direction.relative(self.pos),
                vanilla_blocks::STONE.default_state(),
                UpdateFlags::UPDATE_ALL,
            );
        }
        self.block(state);
        let entity = self
            .world_fixture
            .world
            .get_block_entity(self.pos)
            .expect("sign entity");
        let sign = entity.downcast_ref::<SignBlockEntity>().expect("sign");
        for slot in [SignTextSlot::Front, SignTextSlot::Back] {
            let mut text = SignText::new();
            if with_text {
                text.set_message(0, TextComponent::plain("Read me"));
            }
            text.has_glowing_text = glowing;
            sign.set_text(text, slot);
        }
        entity
    }

    fn hold(&self, hand: InteractionHand, stack: ItemStack) {
        self.player.inventory.lock().set_item_in_hand(hand, stack);
    }

    fn use_on(&self, hand: InteractionHand) -> InteractionResult {
        use_item_on(
            &self.player,
            &self.world_fixture.world,
            hand,
            &BlockHitResult {
                block_pos: self.pos,
                direction: Direction::Up,
                location: DVec3::new(8.5, 64.5, 8.5),
                miss: false,
                inside: false,
                world_border_hit: false,
            },
        )
    }

    fn used(&self, item: ItemRef) -> i32 {
        let stat = vanilla_stat_types::ITEM_USED.get(item);
        self.player
            .stats()
            .into_iter()
            .find_map(|(key, count)| (key == stat).then_some(count))
            .unwrap_or(0)
    }

    fn count(&self, item: ItemRef) -> i32 {
        let inv = self.player.inventory.lock();
        (0..inv.get_container_size())
            .map(|slot| inv.get_item(slot))
            .filter(|stack| stack.is(item))
            .map(ItemStack::count)
            .sum()
    }
}

fn consuming_transformer(count: i32) -> ItemStack {
    let mut stack = ItemStack::with_count(&vanilla_items::STICK, count);
    let transformer = vanilla_items::WOODEN_SHOVEL
        .components
        .get_ref(BLOCK_TRANSFORMER)
        .expect("shovel transformer");
    stack.set(
        BLOCK_TRANSFORMER,
        BlockTransformerComponent::new(transformer.block_transformer),
    );
    stack
}

#[test]
fn extra_remainder_merges_into_the_other_hand_without_losing_inventory_changes() {
    let interaction = Interaction::new("use_on_remainder_merge");
    interaction.hold(
        InteractionHand::MainHand,
        ItemStack::with_count(&vanilla_items::DIAMOND, 63),
    );
    let mut stack = consuming_transformer(3);
    stack.set(
        USE_REMAINDER,
        UseRemainder::new(ItemStackTemplate::new(&vanilla_items::DIAMOND)),
    );
    interaction.hold(InteractionHand::OffHand, stack);
    interaction.block(vanilla_blocks::DIRT.default_state());
    assert_eq!(
        interaction.use_on(InteractionHand::OffHand),
        InteractionResult::Success
    );
    let inv = interaction.player.inventory.lock();
    assert_eq!(inv.get_selected_item().count(), 64);
    assert_eq!(inv.get_item_in_hand(InteractionHand::OffHand).count(), 2);
}

#[test]
fn extra_remainder_drops_when_the_inventory_is_full() {
    let interaction = Interaction::new("use_on_remainder_full_inventory");
    {
        let mut inv = interaction.player.inventory.lock();
        for slot in 0..36 {
            inv.set_item(slot, ItemStack::with_count(&vanilla_items::STONE, 64));
        }
    }
    let mut stack = consuming_transformer(3);
    stack.set(
        USE_REMAINDER,
        UseRemainder::new(ItemStackTemplate::new(&vanilla_items::DIAMOND)),
    );
    interaction.hold(InteractionHand::MainHand, stack);
    interaction.block(vanilla_blocks::DIRT.default_state());
    assert_eq!(
        interaction.use_on(InteractionHand::MainHand),
        InteractionResult::Success
    );
    assert_eq!(interaction.count(&vanilla_items::DIAMOND), 0);
    let entities = interaction
        .world_fixture
        .world
        .entity_manager()
        .get_accessible_entities();
    let drops: Vec<_> = entities
        .iter()
        .filter_map(|entity| entity.downcast_ref::<ItemEntity>())
        .collect();
    assert_eq!(drops.len(), 1);
    assert!(drops[0].get_item().is(&vanilla_items::DIAMOND));
    assert_eq!(drops[0].get_item().count(), 1);
    assert_eq!(
        interaction
            .player
            .inventory
            .lock()
            .get_selected_item()
            .count(),
        2
    );
}

#[test]
fn broken_tool_uses_the_original_statistics_remainder_and_cooldown() {
    let interaction = Interaction::new("use_on_broken_tool_remainder");
    let mut axe = ItemStack::new(&vanilla_items::WOODEN_AXE);
    axe.set_damage_value(axe.get_max_damage() - 1);
    axe.set(
        USE_REMAINDER,
        UseRemainder::new(ItemStackTemplate::new(&vanilla_items::DIAMOND)),
    );
    axe.set(USE_COOLDOWN, UseCooldown::new(1.0, None));
    let before_use = axe.clone();
    interaction.hold(InteractionHand::OffHand, axe);
    interaction.block(vanilla_blocks::OAK_LOG.default_state());
    assert_eq!(
        interaction.use_on(InteractionHand::OffHand),
        InteractionResult::Success
    );
    assert!(
        interaction
            .player
            .inventory
            .lock()
            .get_item_in_hand(InteractionHand::OffHand)
            .is(&vanilla_items::DIAMOND)
    );
    assert_eq!(interaction.used(&vanilla_items::WOODEN_AXE), 1);
    assert_eq!(interaction.used(&vanilla_items::DIAMOND), 0);
    assert!(interaction.player.is_item_on_cooldown(&before_use));
}

#[test]
fn sign_applicators_count_once_without_item_use_component_side_effects() {
    let applicators: [ItemRef; 3] = [
        &*vanilla_items::GLOW_INK_SAC,
        &*vanilla_items::INK_SAC,
        &*vanilla_items::HONEYCOMB,
    ];
    for block in [
        &vanilla_blocks::OAK_SIGN,
        &vanilla_blocks::OAK_WALL_SIGN,
        &vanilla_blocks::OAK_HANGING_SIGN,
        &vanilla_blocks::OAK_WALL_HANGING_SIGN,
    ] {
        for (item, hand) in applicators.into_iter().flat_map(|item| {
            [InteractionHand::MainHand, InteractionHand::OffHand]
                .into_iter()
                .map(move |hand| (item, hand))
        }) {
            let interaction = Interaction::new("use_on_sign_side_effects");
            let entity = interaction.sign(
                block.default_state(),
                true,
                item == &*vanilla_items::INK_SAC,
            );
            let sign = entity.downcast_ref::<SignBlockEntity>().expect("sign");
            let mut stack = ItemStack::with_count(item, 1);
            stack.set(USE_COOLDOWN, UseCooldown::new(1.0, None));
            stack.set(
                USE_REMAINDER,
                UseRemainder::new(ItemStackTemplate::new(&vanilla_items::DIAMOND)),
            );
            let before_use = stack.clone();
            interaction.hold(hand, stack);

            let result = interaction.use_on(hand);

            assert_eq!(result, InteractionResult::Success);
            assert!(result.should_swing());
            assert_eq!(sign.get_player_who_may_edit(), None);
            if item == &*vanilla_items::HONEYCOMB {
                assert!(sign.is_waxed());
            } else {
                let slot =
                    facing_text_slot(block.default_state(), interaction.pos, &interaction.player);
                assert_eq!(
                    sign.get_text(slot).has_glowing_text,
                    item == &*vanilla_items::GLOW_INK_SAC
                );
            }
            assert_eq!(interaction.used(item), 1);
            assert_eq!(interaction.count(item), 0);
            assert_eq!(interaction.count(&vanilla_items::DIAMOND), 0);
            assert!(!interaction.player.is_item_on_cooldown(&before_use));
        }
    }
}

#[test]
fn sneaking_bypasses_sign_applicators_in_both_hands() {
    for (item, hand) in [
        &*vanilla_items::GLOW_INK_SAC,
        &*vanilla_items::INK_SAC,
        &*vanilla_items::HONEYCOMB,
    ]
    .into_iter()
    .flat_map(|item| {
        [InteractionHand::MainHand, InteractionHand::OffHand]
            .into_iter()
            .map(move |hand| (item, hand))
    }) {
        let interaction = Interaction::new("sneaking_sign_applicator");
        let glowing = item == &*vanilla_items::INK_SAC;
        let entity = interaction.sign(vanilla_blocks::OAK_SIGN.default_state(), true, glowing);
        let sign = entity.downcast_ref::<SignBlockEntity>().expect("sign");
        interaction.player.set_crouching(true);
        interaction.hold(hand, ItemStack::with_count(item, 2));

        assert_eq!(interaction.use_on(hand), InteractionResult::Pass);

        assert_eq!(interaction.count(item), 2);
        assert_eq!(interaction.used(item), 0);
        assert!(!sign.is_waxed());
        assert_eq!(sign.get_player_who_may_edit(), None);
        for slot in [SignTextSlot::Front, SignTextSlot::Back] {
            assert_eq!(sign.get_text(slot).has_glowing_text, glowing);
        }
    }
}

#[test]
fn glow_ink_rejections_do_not_consume_or_change_the_sign() {
    for (with_text, waxed, other_editor) in [
        (false, false, false),
        (true, true, false),
        (true, false, true),
    ] {
        let interaction = Interaction::new("rejected_sign_applicator");
        let entity = interaction.sign(vanilla_blocks::OAK_SIGN.default_state(), with_text, false);
        let sign = entity.downcast_ref::<SignBlockEntity>().expect("sign");
        if waxed {
            assert!(sign.wax());
        }
        if other_editor {
            sign.set_player_who_may_edit(Some(Uuid::nil()));
        }
        interaction.hold(
            InteractionHand::MainHand,
            ItemStack::with_count(&vanilla_items::GLOW_INK_SAC, 2),
        );

        interaction.use_on(InteractionHand::MainHand);

        assert_eq!(interaction.count(&vanilla_items::GLOW_INK_SAC), 2);
        assert_eq!(interaction.used(&vanilla_items::GLOW_INK_SAC), 0);
        for slot in [SignTextSlot::Front, SignTextSlot::Back] {
            assert!(!sign.get_text(slot).has_glowing_text);
        }
    }
}

#[test]
fn honeycomb_can_wax_blank_signs_without_consuming_in_creative() {
    let interaction = Interaction::new("creative_blank_sign_waxing");
    let entity = interaction.sign(vanilla_blocks::OAK_SIGN.default_state(), false, false);
    let sign = entity.downcast_ref::<SignBlockEntity>().expect("sign");
    interaction
        .player
        .restore_game_modes(GameType::Creative, None);
    interaction.hold(
        InteractionHand::MainHand,
        ItemStack::new(&vanilla_items::HONEYCOMB),
    );

    let result = interaction.use_on(InteractionHand::MainHand);

    assert_eq!(result, InteractionResult::Success);
    assert!(result.should_swing());
    assert!(sign.is_waxed());
    assert_eq!(sign.get_player_who_may_edit(), None);
    assert_eq!(interaction.count(&vanilla_items::HONEYCOMB), 1);
    assert_eq!(interaction.used(&vanilla_items::HONEYCOMB), 1);
}

#[test]
fn water_cauldron_bottle_dispatch_lowers_every_level_and_preserves_creative_bottles() {
    use steel_registry::blocks::properties::BlockStateProperties;
    use steel_registry::data_components::vanilla_components::POTION_CONTENTS;
    for hand in [InteractionHand::MainHand, InteractionHand::OffHand] {
        for mode in [GameType::Survival, GameType::Creative] {
            for level in 1..=3 {
                let fixture = Interaction::new("cauldron_bottle_dispatch");
                fixture.player.set_game_mode(mode);
                fixture.block(vanilla_blocks::WATER_CAULDRON.default_state()
                    .set_value(&BlockStateProperties::LEVEL_CAULDRON, level));
                fixture.hold(hand, ItemStack::with_count(&vanilla_items::GLASS_BOTTLE, 2));
                assert!(fixture.use_on(hand).consumes_action());
                let world = &fixture.world_fixture.world;
                let next = world.get_block_state(fixture.pos);
                if level == 1 { assert_eq!(next.get_block(), &vanilla_blocks::CAULDRON); }
                else { assert_eq!(next.get_value(&BlockStateProperties::LEVEL_CAULDRON), level - 1); }
                assert_eq!(fixture.count(&vanilla_items::GLASS_BOTTLE), if mode == GameType::Creative { 2 } else { 1 });
                assert_eq!(fixture.count(&vanilla_items::POTION), 1);
                assert_eq!(fixture.used(&vanilla_items::GLASS_BOTTLE), 1);
                let inventory = fixture.player.inventory.lock();
                assert!((0..inventory.get_container_size()).any(|slot| {
                    let item = inventory.get_item(slot);
                    item.is(&vanilla_items::POTION) && item.get(POTION_CONTENTS).is_some()
                }));
            }
        }
    }
}

#[test]
fn bottle_does_not_extract_water_from_powder_snow_cauldron() {
    use steel_registry::blocks::properties::BlockStateProperties;
    let fixture = Interaction::new("powder_snow_rejects_bottle");
    let state = vanilla_blocks::POWDER_SNOW_CAULDRON.default_state()
        .set_value(&BlockStateProperties::LEVEL_CAULDRON, 3);
    fixture.block(state);
    fixture.hold(InteractionHand::MainHand, ItemStack::new(&vanilla_items::GLASS_BOTTLE));
    fixture.use_on(InteractionHand::MainHand);
    assert_eq!(fixture.world_fixture.world.get_block_state(fixture.pos), state);
    assert_eq!(fixture.count(&vanilla_items::GLASS_BOTTLE), 1);
    assert_eq!(fixture.count(&vanilla_items::POTION), 0);
}

#[test]
fn cauldron_filled_buckets_replace_every_cauldron_type_in_both_hands_and_modes() {
    use steel_registry::blocks::properties::BlockStateProperties;
    for hand in [InteractionHand::MainHand, InteractionHand::OffHand] {
        for mode in [GameType::Survival, GameType::Creative] {
            for original in [vanilla_blocks::CAULDRON.default_state(), vanilla_blocks::WATER_CAULDRON.default_state(),
                vanilla_blocks::LAVA_CAULDRON.default_state(), vanilla_blocks::POWDER_SNOW_CAULDRON.default_state()] {
                for (bucket, expected) in [
                    (&vanilla_items::WATER_BUCKET, vanilla_blocks::WATER_CAULDRON.default_state().set_value(&BlockStateProperties::LEVEL_CAULDRON, 3)),
                    (&vanilla_items::LAVA_BUCKET, vanilla_blocks::LAVA_CAULDRON.default_state()),
                    (&vanilla_items::POWDER_SNOW_BUCKET, vanilla_blocks::POWDER_SNOW_CAULDRON.default_state().set_value(&BlockStateProperties::LEVEL_CAULDRON, 3)),
                ] {
                    let fixture = Interaction::new("cauldron_bucket_fill");
                    fixture.player.set_game_mode(mode);
                    fixture.block(original);
                    fixture.hold(hand, ItemStack::new(bucket));
                    assert!(fixture.use_on(hand).consumes_action());
                    assert_eq!(fixture.world_fixture.world.get_block_state(fixture.pos), expected);
                    assert_eq!(fixture.count(bucket), if mode == GameType::Creative { 1 } else { 0 });
                    assert_eq!(fixture.count(&vanilla_items::BUCKET), 1);
                    assert_eq!(fixture.used(bucket), 1);
                }
            }
        }
    }
}

#[test]
fn bucket_extraction_requires_full_water_or_snow_but_accepts_lava() {
    use steel_registry::blocks::properties::BlockStateProperties;
    for (block, bucket) in [(&vanilla_blocks::WATER_CAULDRON, &vanilla_items::WATER_BUCKET),
        (&vanilla_blocks::POWDER_SNOW_CAULDRON, &vanilla_items::POWDER_SNOW_BUCKET)] {
        for level in 1..=3 {
            let fixture = Interaction::new("cauldron_bucket_extract");
            let state = block.default_state().set_value(&BlockStateProperties::LEVEL_CAULDRON, level);
            fixture.block(state);
            fixture.hold(InteractionHand::MainHand, ItemStack::new(&vanilla_items::BUCKET));
            fixture.use_on(InteractionHand::MainHand);
            assert_eq!(fixture.count(bucket), i32::from(level == 3));
            assert_eq!(fixture.world_fixture.world.get_block_state(fixture.pos), if level == 3 { vanilla_blocks::CAULDRON.default_state() } else { state });
        }
    }
    let fixture = Interaction::new("lava_bucket_extract");
    fixture.block(vanilla_blocks::LAVA_CAULDRON.default_state());
    fixture.hold(InteractionHand::OffHand, ItemStack::new(&vanilla_items::BUCKET));
    assert!(fixture.use_on(InteractionHand::OffHand).consumes_action());
    assert_eq!(fixture.count(&vanilla_items::LAVA_BUCKET), 1);
    assert_eq!(fixture.world_fixture.world.get_block_state(fixture.pos), vanilla_blocks::CAULDRON.default_state());
}

#[test]
fn water_potion_refills_only_empty_or_nonfull_water_cauldron() {
    use steel_registry::{data_components::PotionContents, vanilla_potions};
    use steel_registry::blocks::properties::BlockStateProperties;
    for level in 0..=3 {
        let fixture = Interaction::new("cauldron_potion_refill");
        let state = if level == 0 { vanilla_blocks::CAULDRON.default_state() }
            else { vanilla_blocks::WATER_CAULDRON.default_state().set_value(&BlockStateProperties::LEVEL_CAULDRON, level) };
        fixture.block(state);
        fixture.hold(InteractionHand::MainHand, PotionContents::create_item_stack(&vanilla_items::POTION, &vanilla_potions::WATER));
        fixture.use_on(InteractionHand::MainHand);
        assert_eq!(fixture.count(&vanilla_items::GLASS_BOTTLE), i32::from(level < 3));
        assert_eq!(fixture.count(&vanilla_items::POTION), i32::from(level == 3));
        assert_eq!(fixture.world_fixture.world.get_block_state(fixture.pos).get_value(&BlockStateProperties::LEVEL_CAULDRON), (level + 1).min(3));
    }
    let fixture = Interaction::new("cauldron_rejects_healing");
    fixture.block(vanilla_blocks::CAULDRON.default_state());
    fixture.hold(InteractionHand::MainHand, PotionContents::create_item_stack(&vanilla_items::POTION, &vanilla_potions::HEALING));
    fixture.use_on(InteractionHand::MainHand);
    assert_eq!(fixture.count(&vanilla_items::POTION), 1);
    assert_eq!(fixture.world_fixture.world.get_block_state(fixture.pos), vanilla_blocks::CAULDRON.default_state());
}

#[test]
fn underwater_cauldron_rejects_lava_and_snow_without_consuming_bucket() {
    for bucket in [&vanilla_items::LAVA_BUCKET, &vanilla_items::POWDER_SNOW_BUCKET] {
        let fixture = Interaction::new("cauldron_underwater");
        fixture.block(vanilla_blocks::CAULDRON.default_state());
        fixture.world_fixture.world.set_block(fixture.pos.above(), vanilla_blocks::WATER.default_state(), UpdateFlags::UPDATE_ALL);
        fixture.hold(InteractionHand::MainHand, ItemStack::new(bucket));
        assert!(fixture.use_on(InteractionHand::MainHand).consumes_action());
        assert_eq!(fixture.count(bucket), 1);
        assert_eq!(fixture.world_fixture.world.get_block_state(fixture.pos), vanilla_blocks::CAULDRON.default_state());
    }
}

#[test]
fn washing_colored_shulker_preserves_components_and_consumes_one_water_level() {
    use steel_registry::data_components::vanilla_components::{CUSTOM_NAME, CONTAINER};
    use steel_registry::data_components::ItemContainerContents;
    use steel_registry::blocks::properties::BlockStateProperties;
    for mode in [GameType::Survival, GameType::Creative] {
        let fixture = Interaction::new("cauldron_shulker_wash");
        fixture.player.set_game_mode(mode);
        fixture.block(vanilla_blocks::WATER_CAULDRON.default_state().set_value(&BlockStateProperties::LEVEL_CAULDRON, 1));
        let mut shulker = ItemStack::new(&vanilla_items::RED_SHULKER_BOX);
        shulker.set(CUSTOM_NAME, TextComponent::plain("Stored cargo"));
        let cargo = ItemContainerContents::new(vec![Some(steel_registry::ItemStackTemplate::from_stack(
            &ItemStack::with_count(&vanilla_items::DIAMOND, 17)).unwrap())]).unwrap();
        shulker.set(CONTAINER, cargo.clone());
        fixture.hold(InteractionHand::MainHand, shulker);
        assert!(fixture.use_on(InteractionHand::MainHand).consumes_action());
        assert_eq!(fixture.count(&vanilla_items::SHULKER_BOX), 1);
        assert_eq!(fixture.world_fixture.world.get_block_state(fixture.pos), vanilla_blocks::CAULDRON.default_state());
        let inventory = fixture.player.inventory.lock();
        assert!((0..inventory.get_container_size()).any(|slot| {
            let item = inventory.get_item(slot);
            item.is(&vanilla_items::SHULKER_BOX) && item.get(CUSTOM_NAME) == Some(&TextComponent::plain("Stored cargo"))
                && item.get(CONTAINER) == Some(&cargo)
        }));
    }
}

#[test]
fn water_cauldron_washes_dyed_equipment_in_place_including_creative_and_offhand() {
    use steel_registry::data_components::{DyedItemColor, vanilla_components::{DYED_COLOR, CUSTOM_NAME}};
    use steel_registry::blocks::properties::BlockStateProperties;
    for hand in [InteractionHand::MainHand, InteractionHand::OffHand] {
        for mode in [GameType::Survival, GameType::Creative] {
            for item in [&vanilla_items::LEATHER_HELMET, &vanilla_items::LEATHER_CHESTPLATE,
                &vanilla_items::LEATHER_LEGGINGS, &vanilla_items::LEATHER_BOOTS, &vanilla_items::LEATHER_HORSE_ARMOR] {
                let fixture = Interaction::new("cauldron_equipment_wash");
                fixture.player.set_game_mode(mode);
                fixture.block(vanilla_blocks::WATER_CAULDRON.default_state().set_value(&BlockStateProperties::LEVEL_CAULDRON, 2));
                let mut stack = ItemStack::new(item);
                stack.set(DYED_COLOR, DyedItemColor::new(0xff00aa));
                stack.set(CUSTOM_NAME, TextComponent::plain("Keep name"));
                fixture.hold(hand, stack);
                assert!(fixture.use_on(hand).consumes_action());
                assert_eq!(fixture.count(item), 1);
                let inventory = fixture.player.inventory.lock();
                let washed = inventory.get_item_in_hand(hand);
                assert!(washed.get(DYED_COLOR).is_none());
                assert_eq!(washed.get(CUSTOM_NAME), Some(&TextComponent::plain("Keep name")));
                assert_eq!(fixture.world_fixture.world.get_block_state(fixture.pos).get_value(&BlockStateProperties::LEVEL_CAULDRON), 1);
            }
        }
    }
}

#[test]
fn cauldron_dye_washing_rejects_uncolored_equipment_wrong_items_and_snow() {
    use steel_registry::data_components::{DyedItemColor, vanilla_components::DYED_COLOR};
    for (item, colored, block) in [(&vanilla_items::LEATHER_HELMET, false, &vanilla_blocks::WATER_CAULDRON),
        (&vanilla_items::STICK, true, &vanilla_blocks::WATER_CAULDRON),
        (&vanilla_items::LEATHER_HELMET, true, &vanilla_blocks::POWDER_SNOW_CAULDRON)] {
        let fixture = Interaction::new("cauldron_wash_reject");
        let state = block.default_state();
        fixture.block(state);
        let mut stack = ItemStack::new(item);
        if colored { stack.set(DYED_COLOR, DyedItemColor::new(123)); }
        fixture.hold(InteractionHand::MainHand, stack.clone());
        fixture.use_on(InteractionHand::MainHand);
        assert_eq!(fixture.world_fixture.world.get_block_state(fixture.pos), state);
        assert_eq!(fixture.player.inventory.lock().get_item_in_hand(InteractionHand::MainHand), &stack);
    }
}

#[test]
fn cauldron_banner_washing_removes_only_top_layer_from_one_banner() {
    use steel_registry::{DyeColor, RegistryHolder, vanilla_banner_patterns};
    use steel_registry::data_components::{BannerPatternLayer, BannerPatternLayers, vanilla_components::{BANNER_PATTERNS, CUSTOM_NAME}};
    use steel_registry::blocks::properties::BlockStateProperties;
    for mode in [GameType::Survival, GameType::Creative] {
        for hand in [InteractionHand::MainHand, InteractionHand::OffHand] {
            let fixture = Interaction::new("cauldron_banner_wash");
            fixture.player.set_game_mode(mode);
            fixture.block(vanilla_blocks::WATER_CAULDRON.default_state().set_value(&BlockStateProperties::LEVEL_CAULDRON, 3));
            let bottom = BannerPatternLayer::new(RegistryHolder::reference(&vanilla_banner_patterns::STRIPE_BOTTOM), DyeColor::Red);
            let top = BannerPatternLayer::new(RegistryHolder::reference(&vanilla_banner_patterns::CROSS), DyeColor::Blue);
            let mut stack = ItemStack::with_count(&vanilla_items::WHITE_BANNER, 2);
            stack.set(BANNER_PATTERNS, BannerPatternLayers::new(vec![bottom.clone(), top]));
            stack.set(CUSTOM_NAME, TextComponent::plain("Flag"));
            fixture.hold(hand, stack);
            assert!(fixture.use_on(hand).consumes_action());
            let inventory = fixture.player.inventory.lock();
            assert_eq!(inventory.get_item_in_hand(hand).count(), if mode == GameType::Creative { 2 } else { 1 });
            assert_eq!(inventory.get_item_in_hand(hand).get(BANNER_PATTERNS).unwrap().layers().len(), 2);
            let cleaned: Vec<_> = (0..inventory.get_container_size()).map(|slot| inventory.get_item(slot))
                .filter(|item| item.is(&vanilla_items::WHITE_BANNER) && item.get(BANNER_PATTERNS).is_some_and(|p| p.layers().len() == 1)).collect();
            assert_eq!(cleaned.len(), 1);
            assert_eq!(cleaned[0].count(), 1);
            assert_eq!(cleaned[0].get(BANNER_PATTERNS).unwrap().layers(), &[bottom]);
            assert_eq!(cleaned[0].get(CUSTOM_NAME), Some(&TextComponent::plain("Flag")));
            assert_eq!(fixture.world_fixture.world.get_block_state(fixture.pos).get_value(&BlockStateProperties::LEVEL_CAULDRON), 2);
        }
    }
}

// Reproduce independent Flint campfire/hay scenarios through the real player
// ray and full game-mode interaction path, not just a placement-state helper.
fn ray_use(interaction: &Interaction) -> InteractionResult {
    let (start, end) = interaction.player.get_ray_endpoints();
    let hit = interaction.world_fixture.world.clip(
        start, end, crate::world::ClipBlockShape::Outline, crate::world::ClipFluid::None,
    );
    assert!(!hit.is_miss(), "ray {start:?} -> {end:?} must hit the fixture");
    use_item_on(
        &interaction.player, &interaction.world_fixture.world, InteractionHand::MainHand,
        &BlockHitResult {
            block_pos: hit.block_pos, direction: hit.direction, location: hit.location,
            miss: hit.miss, inside: hit.inside, world_border_hit: hit.world_border_hit,
        },
    )
}

#[test]
fn survival_shovel_ray_extinguishes_campfire_preserving_properties() {
    use steel_registry::blocks::properties::BlockStateProperties as P;
    let interaction = Interaction::new("ray_campfire_shovel");
    interaction.player.set_game_mode(GameType::Survival);
    interaction.player.base().set_position_local(DVec3::new(8.5, 66.0, 8.5));
    interaction.player.set_rotation([0.0, 90.0].into());
    let state = vanilla_blocks::CAMPFIRE.default_state()
        .set_value(&P::LIT, true)
        .set_value(&P::SIGNAL_FIRE, false)
        .set_value(&P::WATERLOGGED, false)
        .set_value(&P::HORIZONTAL_FACING, Direction::North);
    interaction.world_fixture.world.set_block(
        interaction.pos.below(), vanilla_blocks::STONE.default_state(), UpdateFlags::UPDATE_ALL,
    );
    interaction.block(state);
    interaction.hold(InteractionHand::MainHand, ItemStack::new(&vanilla_items::IRON_SHOVEL));
    assert!(ray_use(&interaction).consumes_action());
    assert_eq!(interaction.world_fixture.world.get_block_state(interaction.pos), state.set_value(&P::LIT, false));
    assert_eq!(interaction.count(&vanilla_items::IRON_SHOVEL), 1);
    {
        let inventory = interaction.player.inventory.lock();
        assert_eq!(inventory.get_item_in_hand(InteractionHand::MainHand)
            .get(steel_registry::data_components::vanilla_components::DAMAGE), Some(&1));
    }
    // Repeating the click on an already unlit campfire must neither consume
    // another use nor damage the shovel again.
    assert!(!ray_use(&interaction).consumes_action());
    let inventory = interaction.player.inventory.lock();
    assert_eq!(inventory.get_item_in_hand(InteractionHand::MainHand)
        .get(steel_registry::data_components::vanilla_components::DAMAGE), Some(&1));
}

#[test]
fn survival_hay_east_face_ray_places_x_axis_and_consumes_stack() {
    use steel_registry::blocks::properties::BlockStateProperties as P;
    let interaction = Interaction::new("ray_hay_east_placement");
    interaction.player.set_game_mode(GameType::Survival);
    interaction.player.base().set_position_local(DVec3::new(11.5, 63.0, 8.5));
    interaction.player.set_rotation([90.0, 0.0].into());
    interaction.block(vanilla_blocks::STONE.default_state());
    interaction.hold(InteractionHand::MainHand, ItemStack::new(&vanilla_items::HAY_BLOCK));
    assert!(ray_use(&interaction).consumes_action());
    assert_eq!(interaction.world_fixture.world.get_block_state(interaction.pos.east()),
        vanilla_blocks::HAY_BLOCK.default_state().set_value(&P::AXIS, steel_utils::axis::Axis::X));
    assert_eq!(interaction.count(&vanilla_items::HAY_BLOCK), 0);
}

#[test]
fn creative_soul_campfire_dousing_preserves_tool_durability() {
    use steel_registry::blocks::properties::BlockStateProperties as P;
    let interaction = Interaction::new("ray_soul_campfire_creative");
    interaction.player.set_game_mode(GameType::Creative);
    interaction.player.base().set_position_local(DVec3::new(8.5, 66.0, 8.5));
    interaction.player.set_rotation([0.0, 90.0].into());
    interaction.world_fixture.world.set_block(
        interaction.pos.below(), vanilla_blocks::STONE.default_state(), UpdateFlags::UPDATE_ALL,
    );
    interaction.block(vanilla_blocks::SOUL_CAMPFIRE.default_state().set_value(&P::LIT, true));
    interaction.hold(InteractionHand::MainHand, ItemStack::new(&vanilla_items::IRON_SHOVEL));
    assert!(ray_use(&interaction).consumes_action());
    assert!(!interaction.world_fixture.world.get_block_state(interaction.pos).get_value(&P::LIT));
    let inventory = interaction.player.inventory.lock();
    assert_eq!(inventory.get_item_in_hand(InteractionHand::MainHand)
        .get(steel_registry::data_components::vanilla_components::DAMAGE), Some(&0));
}
