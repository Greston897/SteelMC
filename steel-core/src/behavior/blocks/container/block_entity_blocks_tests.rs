//! Registered block dispatch and scheduled-tick integration, not helper-only tests.
use super::*;
use crate::bootstrap::init_globals;
use crate::test_support::{TestWorld, fresh_test_world, insert_ready_full_chunk, tick_test_world};
use steel_registry::vanilla_blocks;
use steel_utils::ChunkPos;

const SOURCE: BlockPos = BlockPos::new(8, 64, 8);
const TARGET: BlockPos = BlockPos::new(9, 64, 8);

fn fixture(name: &str, stack: ItemStack) -> (TestWorld, ContainerRef) {
    init_globals();
    let fixture = fresh_test_world(name);
    let world = &fixture.world;
    insert_ready_full_chunk(world, ChunkPos::from_block_pos(SOURCE));
    world.set_block(
        SOURCE,
        vanilla_blocks::DISPENSER
            .default_state()
            .set_value(&BlockStateProperties::FACING, Direction::East),
        UpdateFlags::UPDATE_ALL,
    );
    let container = world
        .get_block_entity(SOURCE)
        .unwrap()
        .container_ref()
        .unwrap();
    container.lock().set_item(0, stack);
    (fixture, container)
}

fn pulse(world: &Arc<World>) {
    world.set_block(
        SOURCE.above(),
        vanilla_blocks::REDSTONE_BLOCK.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    for tick in 0..3 {
        tick_test_world(world, tick, true);
    }
    tick_test_world(world, 3, true);
}

#[test]
fn powered_dispenser_grows_wheat_and_consumes_exactly_one_bone_meal() {
    let (fixture, container) = fixture(
        "dispenser_bonemeal_wheat",
        ItemStack::with_count(&vanilla_items::BONE_MEAL, 3),
    );
    let world = &fixture.world;
    world.set_block(
        TARGET.below(),
        vanilla_blocks::FARMLAND.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    world.set_block(
        TARGET,
        vanilla_blocks::WHEAT.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    pulse(world);
    assert!(
        world
            .get_block_state(TARGET)
            .get_value(&BlockStateProperties::AGE_7)
            > 0
    );
    assert_eq!(container.lock().get_item(0).count(), 2);
}

#[test]
fn failed_bone_meal_dispense_keeps_inventory_intact() {
    let (fixture, container) = fixture(
        "dispenser_bonemeal_invalid",
        ItemStack::with_count(&vanilla_items::BONE_MEAL, 3),
    );
    pulse(&fixture.world);
    assert_eq!(container.lock().get_item(0).count(), 3);
}

#[test]
fn bottles_fill_from_source_flowing_water_and_waterlogged_blocks_without_draining() {
    use steel_registry::data_components::vanilla_components::POTION_CONTENTS;
    let targets = [
        vanilla_blocks::WATER.default_state(),
        vanilla_blocks::WATER
            .default_state()
            .set_value(&BlockStateProperties::LEVEL, 3),
        vanilla_blocks::OAK_SLAB
            .default_state()
            .set_value(&BlockStateProperties::WATERLOGGED, true),
    ];
    for target in targets {
        let (fixture, container) = fixture(
            "dispenser_bottle_water",
            ItemStack::new(&vanilla_items::GLASS_BOTTLE),
        );
        let world = &fixture.world;
        world.set_block(
            TARGET.below(),
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_ALL,
        );
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
    let (fixture, container) = fixture(
        "dispenser_bottle_honey",
        ItemStack::with_count(&vanilla_items::GLASS_BOTTLE, 2),
    );
    let world = &fixture.world;
    world.set_block(
        TARGET,
        vanilla_blocks::BEEHIVE
            .default_state()
            .set_value(&BlockStateProperties::LEVEL_HONEY, 5),
        UpdateFlags::UPDATE_ALL,
    );
    pulse(world);
    assert_eq!(
        world
            .get_block_state(TARGET)
            .get_value(&BlockStateProperties::LEVEL_HONEY),
        0
    );
    assert!(
        container
            .lock()
            .get_item(0)
            .is(&vanilla_items::GLASS_BOTTLE)
    );
    assert_eq!(container.lock().get_item(0).count(), 1);
    assert!(
        container
            .lock()
            .get_item(1)
            .is(&vanilla_items::HONEY_BOTTLE)
    );
    assert_eq!(container.lock().get_item(1).count(), 1);
}

#[test]
fn dispenser_flint_lights_candles_cakes_campfires_and_always_wears_on_blocked_targets() {
    use steel_registry::data_components::vanilla_components::DAMAGE;
    let targets = [
        vanilla_blocks::CANDLE.default_state(),
        vanilla_blocks::CANDLE_CAKE.default_state(),
        vanilla_blocks::CAMPFIRE
            .default_state()
            .set_value(&BlockStateProperties::LIT, false),
        vanilla_blocks::SOUL_CAMPFIRE
            .default_state()
            .set_value(&BlockStateProperties::LIT, false),
        vanilla_blocks::CANDLE
            .default_state()
            .set_value(&BlockStateProperties::WATERLOGGED, true),
        vanilla_blocks::STONE.default_state(),
    ];
    for (index, target) in targets.into_iter().enumerate() {
        let (fixture, container) = fixture(
            "dispenser_flint_targets",
            ItemStack::new(&vanilla_items::FLINT_AND_STEEL),
        );
        let world = &fixture.world;
        world.set_block(
            TARGET.below(),
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_ALL,
        );
        world.set_block(TARGET, target, UpdateFlags::UPDATE_ALL);
        pulse(world);
        assert_eq!(
            container.lock().get_item(0).get(DAMAGE),
            Some(&1),
            "target {index}"
        );
        let actual = world.get_block_state(TARGET);
        if index < 4 {
            assert_eq!(
                actual.get_value(&BlockStateProperties::LIT),
                true,
                "target {index}"
            );
        } else {
            assert_eq!(actual, target);
        }
    }
}

#[test]
fn hopper_power_edges_and_playing_jukebox_refresh_enabled_state_and_ticker() {
    use crate::block_entity::BlockEntity as _;
    use crate::block_entity::entities::{HopperBlockEntity, JukeboxBlockEntity};
    init_globals();
    for prepowered in [false, true] {
        let fixture = fresh_test_world("hopper_power_edges");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::from_block_pos(SOURCE));
        if prepowered {
            world.set_block(
                SOURCE.east(),
                vanilla_blocks::REDSTONE_BLOCK.default_state(),
                UpdateFlags::UPDATE_ALL,
            );
        }
        world.set_block(
            SOURCE,
            vanilla_blocks::HOPPER.default_state(),
            UpdateFlags::UPDATE_ALL,
        );
        assert_eq!(
            world
                .get_block_state(SOURCE)
                .get_value(&BlockStateProperties::ENABLED),
            !prepowered
        );
        world.set_block(
            SOURCE.east(),
            vanilla_blocks::REDSTONE_BLOCK.default_state(),
            UpdateFlags::UPDATE_ALL,
        );
        assert!(
            !world
                .get_block_state(SOURCE)
                .get_value(&BlockStateProperties::ENABLED)
        );
        world.set_block(
            SOURCE.east(),
            vanilla_blocks::AIR.default_state(),
            UpdateFlags::UPDATE_ALL,
        );
        assert!(
            world
                .get_block_state(SOURCE)
                .get_value(&BlockStateProperties::ENABLED)
        );
        world.set_block(
            SOURCE.above(),
            vanilla_blocks::JUKEBOX.default_state(),
            UpdateFlags::UPDATE_ALL,
        );
        let entity = world.get_block_entity(SOURCE.above()).unwrap();
        let jukebox = entity.downcast_ref::<JukeboxBlockEntity>().unwrap();
        jukebox.set_the_item(ItemStack::new(&vanilla_items::MUSIC_DISC_13));
        assert!(
            !world
                .get_block_state(SOURCE)
                .get_value(&BlockStateProperties::ENABLED)
        );
        for tick in 0..16 {
            tick_test_world(world, tick, true);
        }
        assert!(jukebox.is_record_playing());
        let entity = world.get_block_entity(SOURCE).unwrap();
        let hopper = entity.downcast_ref::<HopperBlockEntity>().unwrap();
        assert!(
            hopper
                .container_ref()
                .unwrap()
                .lock()
                .get_item(0)
                .is_empty()
        );
        jukebox.finish_hopper_extraction();
        assert!(
            world
                .get_block_state(SOURCE)
                .get_value(&BlockStateProperties::ENABLED)
        );
    }
}

#[test]
fn powered_dispenser_initializes_firework_lifetime_source_and_straight_angled_flight() {
    use steel_registry::data_components::{
        components::Fireworks,
        vanilla_components::{CUSTOM_NAME, FIREWORKS},
    };
    for facing in [Direction::East, Direction::Up] {
        let mut stack = ItemStack::with_count(&vanilla_items::FIREWORK_ROCKET, 2);
        stack.set(FIREWORKS, Fireworks::new(2, Vec::new()).unwrap());
        stack.set(CUSTOM_NAME, TextComponent::plain("Dispensed rocket"));
        init_globals();
        let fixture = fresh_test_world("dispenser_firework_initialized");
        let world = &fixture.world;
        let chunk_pos = ChunkPos::from_block_pos(SOURCE);
        let holder = insert_ready_full_chunk(world, chunk_pos);
        holder.set_simulation_level(Some(
            crate::chunk::chunk_ticket_manager::ChunkTicketLevel::ENTITY_TICKING_CHUNK,
        ));
        holder.transition_ticking_readiness(
            crate::chunk::chunk_holder::TickingReadiness::EntityTicking,
        );
        world.update_entity_chunk_visibility(chunk_pos, holder.entity_visibility());
        world.chunk_map.rebuild_ticking_chunk_snapshot();
        world.set_block(
            SOURCE,
            vanilla_blocks::DISPENSER
                .default_state()
                .set_value(&BlockStateProperties::FACING, facing),
            UpdateFlags::UPDATE_ALL,
        );
        let container = world
            .get_block_entity(SOURCE)
            .unwrap()
            .container_ref()
            .unwrap();
        container.lock().set_item(0, stack);
        world.set_block(
            SOURCE.below(),
            vanilla_blocks::REDSTONE_BLOCK.default_state(),
            UpdateFlags::UPDATE_ALL,
        );
        for tick in 0..8 {
            tick_test_world(world, tick, true);
        }
        assert_eq!(container.lock().get_item(0).count(), 1);
        let rockets: Vec<_> = world
            .get_entities_in_aabb(&steel_utils::WorldAabb::new(
                1.0, 60.0, 1.0, 15.0, 80.0, 15.0,
            ))
            .into_iter()
            .filter(|entity| entity.entity_type() == &vanilla_entities::FIREWORK_ROCKET)
            .collect();
        assert_eq!(
            rockets.len(),
            1,
            "rocket must survive more than its first tick"
        );
        let rocket = rockets[0].downcast_ref::<FireworkRocketEntity>().unwrap();
        assert!(rocket.is_shot_at_angle());
        assert_eq!(
            rocket.get_item().get(CUSTOM_NAME),
            Some(&TextComponent::plain("Dispensed rocket"))
        );
        assert_eq!(
            rocket.get_item().get(FIREWORKS).unwrap().flight_duration(),
            2
        );
        let axis_speed = if facing == Direction::East {
            rocket.velocity().x
        } else {
            rocket.velocity().y
        };
        assert!(
            (0.45..0.55).contains(&axis_speed),
            "{facing:?}: {}",
            rocket.velocity()
        );
        assert!(if facing == Direction::East {
            rocket.position().x > 10.0
        } else {
            rocket.position().y > 66.0
        });
    }
}

#[test]
fn ordinary_dispenser_items_use_vanilla_spawn_offset_and_triangular_velocity() {
    use steel_utils::random::legacy_random::LegacyRandom;
    for facing in [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ] {
        for seed in 0..128 {
            let mut random = LegacyRandom::from_seed(seed);
            let mut expected_random = LegacyRandom::from_seed(seed);
            let (actual_pos, actual_velocity) = default_item_launch(SOURCE, facing, &mut random);
            let (dx, dy, dz) = facing.offset();
            let vertical = dy != 0;
            let expected_pos = DVec3::new(
                8.5 + f64::from(dx) * 0.7,
                64.5 + f64::from(dy) * 0.7 - if vertical { 0.125 } else { 0.15625 },
                8.5 + f64::from(dz) * 0.7,
            );
            use steel_utils::random::Random as _;
            let power = expected_random.next_f64() * 0.1 + 0.2;
            let deviation = 0.0172275 * 6.0;
            let expected_velocity = DVec3::new(
                expected_random.triangle(f64::from(dx) * power, deviation),
                expected_random.triangle(0.2, deviation),
                expected_random.triangle(f64::from(dz) * power, deviation),
            );
            assert_eq!(actual_pos, expected_pos);
            assert_eq!(actual_velocity, expected_velocity);
            assert!(actual_velocity.x.abs() < 0.404);
            assert!(actual_velocity.z.abs() < 0.404);
            assert!((0.096..0.304).contains(&actual_velocity.y));
        }
    }
}

#[test]
fn jack_o_lantern_is_ejected_as_a_slow_item_not_a_projectile_or_golem() {
    let (fixture, container) = fixture(
        "dispenser_default_item_velocity",
        ItemStack::new(&vanilla_items::JACK_O_LANTERN),
    );
    let world = &fixture.world;
    assert!(dispense_from(world, SOURCE, world.get_block_state(SOURCE)));
    assert!(container.lock().get_item(0).is_empty());
    let entities = world.get_entities_in_aabb(&steel_utils::WorldAabb::new(
        7.0, 63.0, 7.0, 11.0, 67.0, 10.0,
    ));
    let items: Vec<_> = entities
        .iter()
        .filter_map(|entity| entity.downcast_ref::<ItemEntity>())
        .collect();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].position(), DVec3::new(9.2, 64.34375, 8.5));
    assert!(items[0].velocity().x < 0.404);
    assert_eq!(items[0].get_item().item(), &*vanilla_items::JACK_O_LANTERN);
}
