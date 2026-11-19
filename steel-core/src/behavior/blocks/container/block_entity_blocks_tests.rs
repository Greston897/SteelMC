//! Registered block dispatch and scheduled-tick integration, not helper-only tests.
use super::*;
use crate::bootstrap::init_globals;
use crate::test_support::{fresh_test_world, insert_ready_full_chunk, tick_test_world, TestWorld};
use steel_utils::ChunkPos;
use steel_registry::vanilla_blocks;

const SOURCE: BlockPos = BlockPos::new(8, 64, 8);
const TARGET: BlockPos = BlockPos::new(9, 64, 8);

fn fixture(name: &str, stack: ItemStack) -> (TestWorld, ContainerRef) {
    init_globals();
    let fixture = fresh_test_world(name);
    let world = &fixture.world;
    insert_ready_full_chunk(world, ChunkPos::from_block_pos(SOURCE));
    world.set_block(SOURCE, vanilla_blocks::DISPENSER.default_state().set_value(&BlockStateProperties::FACING, Direction::East), UpdateFlags::UPDATE_ALL);
    let container = world.get_block_entity(SOURCE).unwrap().container_ref().unwrap();
    container.lock().set_item(0, stack);
    (fixture, container)
}

fn pulse(world: &Arc<World>) {
    world.set_block(SOURCE.above(), vanilla_blocks::REDSTONE_BLOCK.default_state(), UpdateFlags::UPDATE_ALL);
    for tick in 0..3 { tick_test_world(world, tick, true); }
    tick_test_world(world, 3, true);
}

#[test]
fn powered_dispenser_grows_wheat_and_consumes_exactly_one_bone_meal() {
    let (fixture, container) = fixture("dispenser_bonemeal_wheat", ItemStack::with_count(&vanilla_items::BONE_MEAL, 3));
    let world = &fixture.world;
    world.set_block(TARGET.below(), vanilla_blocks::FARMLAND.default_state(), UpdateFlags::UPDATE_ALL);
    world.set_block(TARGET, vanilla_blocks::WHEAT.default_state(), UpdateFlags::UPDATE_ALL);
    pulse(world);
    assert!(world.get_block_state(TARGET).get_value(&BlockStateProperties::AGE_7) > 0);
    assert_eq!(container.lock().get_item(0).count(), 2);
}

#[test]
fn failed_bone_meal_dispense_keeps_inventory_intact() {
    let (fixture, container) = fixture("dispenser_bonemeal_invalid", ItemStack::with_count(&vanilla_items::BONE_MEAL, 3));
    pulse(&fixture.world);
    assert_eq!(container.lock().get_item(0).count(), 3);
}

#[test]
fn bottles_fill_from_source_flowing_water_and_waterlogged_blocks_without_draining() {
    use steel_registry::data_components::vanilla_components::POTION_CONTENTS;
    let targets = [
        vanilla_blocks::WATER.default_state(),
        vanilla_blocks::WATER.default_state().set_value(&BlockStateProperties::LEVEL, 3),
        vanilla_blocks::OAK_SLAB.default_state().set_value(&BlockStateProperties::WATERLOGGED, true),
    ];
    for target in targets {
        let (fixture, container) = fixture("dispenser_bottle_water", ItemStack::new(&vanilla_items::GLASS_BOTTLE));
        let world = &fixture.world;
        world.set_block(TARGET.below(), vanilla_blocks::STONE.default_state(), UpdateFlags::UPDATE_ALL);
        world.set_block(TARGET, target, UpdateFlags::UPDATE_ALL);
        pulse(world);
        let filled = container.lock().get_item(0).clone();
        assert!(filled.is(&vanilla_items::POTION));
        assert!(filled.get(POTION_CONTENTS).is_some());
        assert_eq!(world.get_block_state(TARGET), target);
    }
}

#[test]
fn bottle_collects_honey_resets_hive_and_stores_stack_remainder() {
    let (fixture, container) = fixture("dispenser_bottle_honey", ItemStack::with_count(&vanilla_items::GLASS_BOTTLE, 2));
    let world = &fixture.world;
    world.set_block(TARGET, vanilla_blocks::BEEHIVE.default_state().set_value(&BlockStateProperties::LEVEL_HONEY, 5), UpdateFlags::UPDATE_ALL);
    pulse(world);
    assert_eq!(world.get_block_state(TARGET).get_value(&BlockStateProperties::LEVEL_HONEY), 0);
    assert!(container.lock().get_item(0).is(&vanilla_items::GLASS_BOTTLE));
    assert_eq!(container.lock().get_item(0).count(), 1);
    assert!(container.lock().get_item(1).is(&vanilla_items::HONEY_BOTTLE));
    assert_eq!(container.lock().get_item(1).count(), 1);
}

#[test]
fn dispenser_flint_lights_candles_cakes_campfires_and_always_wears_on_blocked_targets() {
    use steel_registry::data_components::vanilla_components::DAMAGE;
    let targets = [
        vanilla_blocks::CANDLE.default_state(),
        vanilla_blocks::CANDLE_CAKE.default_state(),
        vanilla_blocks::CAMPFIRE.default_state().set_value(&BlockStateProperties::LIT, false),
        vanilla_blocks::SOUL_CAMPFIRE.default_state().set_value(&BlockStateProperties::LIT, false),
        vanilla_blocks::CANDLE.default_state().set_value(&BlockStateProperties::WATERLOGGED, true),
        vanilla_blocks::STONE.default_state(),
    ];
    for (index, target) in targets.into_iter().enumerate() {
        let (fixture, container) = fixture("dispenser_flint_targets", ItemStack::new(&vanilla_items::FLINT_AND_STEEL));
        let world = &fixture.world;
        world.set_block(TARGET.below(), vanilla_blocks::STONE.default_state(), UpdateFlags::UPDATE_ALL);
        world.set_block(TARGET, target, UpdateFlags::UPDATE_ALL);
        pulse(world);
        assert_eq!(container.lock().get_item(0).get(DAMAGE), Some(&1), "target {index}");
        let actual = world.get_block_state(TARGET);
        if index < 4 {
            assert_eq!(actual.get_value(&BlockStateProperties::LIT), true, "target {index}");
        } else {
            assert_eq!(actual, target);
        }
    }
}

#[test]
fn hopper_power_edges_and_playing_jukebox_refresh_enabled_state_and_ticker() {
    use crate::block_entity::entities::{HopperBlockEntity, JukeboxBlockEntity};
    use crate::block_entity::BlockEntity as _;
    for prepowered in [false, true] {
        let fixture = fresh_test_world("hopper_power_edges");
        init_globals();
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::from_block_pos(SOURCE));
        if prepowered { world.set_block(SOURCE.east(), vanilla_blocks::REDSTONE_BLOCK.default_state(), UpdateFlags::UPDATE_ALL); }
        world.set_block(SOURCE, vanilla_blocks::HOPPER.default_state(), UpdateFlags::UPDATE_ALL);
        assert_eq!(world.get_block_state(SOURCE).get_value(&BlockStateProperties::ENABLED), !prepowered);
        world.set_block(SOURCE.east(), vanilla_blocks::REDSTONE_BLOCK.default_state(), UpdateFlags::UPDATE_ALL);
        assert!(!world.get_block_state(SOURCE).get_value(&BlockStateProperties::ENABLED));
        world.set_block(SOURCE.east(), vanilla_blocks::AIR.default_state(), UpdateFlags::UPDATE_ALL);
        assert!(world.get_block_state(SOURCE).get_value(&BlockStateProperties::ENABLED));
        world.set_block(SOURCE.above(), vanilla_blocks::JUKEBOX.default_state(), UpdateFlags::UPDATE_ALL);
        let entity = world.get_block_entity(SOURCE.above()).unwrap();
        let jukebox = entity.downcast_ref::<JukeboxBlockEntity>().unwrap();
        jukebox.set_the_item(ItemStack::new(&vanilla_items::MUSIC_DISC_13));
        assert!(!world.get_block_state(SOURCE).get_value(&BlockStateProperties::ENABLED));
        for tick in 0..16 { tick_test_world(world, tick, true); }
        assert!(jukebox.is_record_playing());
        let entity = world.get_block_entity(SOURCE).unwrap();
        let hopper = entity.downcast_ref::<HopperBlockEntity>().unwrap();
        assert!(hopper.container_ref().unwrap().lock().get_item(0).is_empty());
        jukebox.finish_hopper_extraction();
        assert!(world.get_block_state(SOURCE).get_value(&BlockStateProperties::ENABLED));
    }
}
