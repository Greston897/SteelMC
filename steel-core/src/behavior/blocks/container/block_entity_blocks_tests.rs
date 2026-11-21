//! Registered block dispatch and scheduled-tick integration, not helper-only tests.
use super::*;
use crate::bootstrap::init_globals;
use crate::chunk::chunk_holder::TickingReadiness::EntityTicking;
use crate::chunk::chunk_ticket_manager::ChunkTicketLevel;
use crate::test_support::{TestWorld, fresh_test_world, insert_ready_full_chunk, tick_test_world};
use steel_registry::vanilla_blocks;
use steel_registry::vanilla_game_rules::TNT_EXPLODES;
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
        .expect("the block entity should exist")
        .container_ref()
        .expect("the container should be present");
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
fn dispenser_keeps_brush_when_no_armadillo_is_present() {
    let (fixture, container) = fixture(
        "dispenser_brush_without_armadillo",
        ItemStack::new(&vanilla_items::BRUSH),
    );
    fixture.world.set_block(
        TARGET.below(),
        vanilla_blocks::STONE.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    fixture.world.set_block(
        TARGET,
        vanilla_blocks::SUSPICIOUS_SAND.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    pulse(&fixture.world);
    let guard = container.lock();
    assert_eq!(guard.get_item(0).item(), &*vanilla_items::BRUSH);
    assert_eq!(guard.get_item(0).count(), 1);
    assert_eq!(
        fixture.world.get_block_state(TARGET).get_block(),
        &vanilla_blocks::SUSPICIOUS_SAND
    );
}

#[test]
fn dispenser_brushes_an_adult_armadillo() {
    use crate::entity::entities::mobs::passive::ArmadilloEntity;
    use crate::entity::next_entity_id;
    use steel_registry::data_components::vanilla_components::DAMAGE;

    let (fixture, container) = fixture(
        "dispenser_brush_armadillo",
        ItemStack::new(&vanilla_items::BRUSH),
    );
    let world = &fixture.world;
    let armadillo = Arc::new(ArmadilloEntity::new(
        &vanilla_entities::ARMADILLO,
        next_entity_id(),
        DVec3::new(9.5, 64.0, 8.5),
        Arc::downgrade(world),
    ));
    world.try_add_entity(armadillo).expect("add armadillo");
    // Exercise the dispenser directly so the mob cannot wander before the pulse.
    assert!(dispense_from(world, SOURCE, world.get_block_state(SOURCE)));
    assert_eq!(container.lock().get_item(0).get(DAMAGE).copied(), Some(16));
    let drops = world.get_entities_in_aabb(&steel_utils::WorldAabb::new(
        9.0, 63.0, 8.0, 10.0, 66.0, 9.0,
    ));
    assert!(drops.iter().any(|entity| {
        entity
            .as_ref()
            .downcast_ref::<ItemEntity>()
            .is_some_and(|item| item.get_item().item() == &*vanilla_items::ARMADILLO_SCUTE)
    }));
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
            assert!(
                actual.get_value(&BlockStateProperties::LIT),
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
        let entity = world
            .get_block_entity(SOURCE.above())
            .expect("the block entity should exist");
        let jukebox = entity
            .downcast_ref::<JukeboxBlockEntity>()
            .expect("the block entity should have the expected type");
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
        let entity = world
            .get_block_entity(SOURCE)
            .expect("the block entity should exist");
        let hopper = entity
            .downcast_ref::<HopperBlockEntity>()
            .expect("the block entity should have the expected type");
        assert!(
            hopper
                .container_ref()
                .expect("the container should be present")
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
        stack.set(
            FIREWORKS,
            Fireworks::new(2, Vec::new()).expect("the firework should be created"),
        );
        stack.set(CUSTOM_NAME, TextComponent::plain("Dispensed rocket"));
        init_globals();
        let fixture = fresh_test_world("dispenser_firework_initialized");
        let world = &fixture.world;
        let chunk_pos = ChunkPos::from_block_pos(SOURCE);
        let holder = insert_ready_full_chunk(world, chunk_pos);
        holder.set_simulation_level(Some(ChunkTicketLevel::ENTITY_TICKING_CHUNK));
        holder.transition_ticking_readiness(EntityTicking);
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
            .expect("the block entity should exist")
            .container_ref()
            .expect("the container should be present");
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
        let rocket = rockets[0]
            .downcast_ref::<FireworkRocketEntity>()
            .expect("the block entity should have the expected type");
        assert!(rocket.is_shot_at_angle());
        assert_eq!(
            rocket.get_item().get(CUSTOM_NAME),
            Some(&TextComponent::plain("Dispensed rocket"))
        );
        assert_eq!(
            rocket
                .get_item()
                .get(FIREWORKS)
                .expect("the component should be present")
                .flight_duration(),
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
            let power = expected_random.next_f64() * 0.1 + 0.2;
            let deviation = 0.017_227_5 * 6.0;
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

/// Vanilla `ProjectileDispenseBehavior` for the fire charge: the projectile
/// spawns at `DispenserBlock.getDispensePosition(source, 1.0, Vec3.ZERO)` and
/// flies along the normalized, noised facing direction (`FireChargeItem.shoot`
/// is a no-op, so the velocity comes from `asProjectile`).
#[test]
fn dispenser_fire_charge_launches_a_small_fireball_with_vanilla_position_and_speed() {
    use crate::entity::Entity as _;
    use crate::entity::ThrowableItemProjectile as _;
    use crate::entity::entities::SmallFireballEntity;

    let (fixture, container) = fixture(
        "dispenser_fire_charge_launch",
        ItemStack::with_count(&vanilla_items::FIRE_CHARGE, 2),
    );
    let world = &fixture.world;
    assert!(dispense_from(world, SOURCE, world.get_block_state(SOURCE)));

    assert_eq!(
        container.lock().get_item(0).count(),
        1,
        "the dispenser consumes exactly one fire charge"
    );

    let entities = world.get_entities_in_aabb(&steel_utils::WorldAabb::new(
        7.0, 62.0, 7.0, 12.0, 68.0, 10.0,
    ));
    let fireballs: Vec<_> = entities
        .iter()
        .filter_map(|entity| entity.downcast_ref::<SmallFireballEntity>())
        .collect();
    assert_eq!(fireballs.len(), 1);

    let fireball = fireballs[0];
    let position = fireball.position();
    assert!(
        (position.x - 9.5).abs() < 1.0e-6
            && (position.y - 64.5).abs() < 1.0e-6
            && (position.z - 8.5).abs() < 1.0e-6,
        "the fireball spawns one block in front of the dispenser: {position:?}"
    );

    let velocity = fireball.velocity();
    let speed = velocity.length();
    assert!(
        (speed - 1.0).abs() < 1.0e-6,
        "the fireball flies at unit speed: {speed}"
    );
    assert!(
        velocity.x > 0.88,
        "the fireball flies along the dispenser facing: {velocity:?}"
    );

    let item = fireball.get_item();
    assert_eq!(item.item(), &*vanilla_items::FIRE_CHARGE);
    assert_eq!(item.count(), 1);
}

/// Vanilla `WindChargeItem.asProjectile` keeps the raw noised facing vector as
/// the delta movement; it is deliberately *not* normalized.
#[test]
fn dispenser_wind_charge_keeps_the_unnormalized_launch_velocity() {
    use crate::entity::Entity as _;
    use crate::entity::entities::WindChargeEntity;

    let (fixture, container) = fixture(
        "dispenser_wind_charge_launch",
        ItemStack::with_count(&vanilla_items::WIND_CHARGE, 1),
    );
    let world = &fixture.world;
    assert!(dispense_from(world, SOURCE, world.get_block_state(SOURCE)));
    assert!(container.lock().get_item(0).is_empty());

    let entities = world.get_entities_in_aabb(&steel_utils::WorldAabb::new(
        7.0, 62.0, 7.0, 12.0, 68.0, 10.0,
    ));
    let charges: Vec<_> = entities
        .iter()
        .filter_map(|entity| entity.downcast_ref::<WindChargeEntity>())
        .collect();
    assert_eq!(charges.len(), 1);

    let velocity = charges[0].velocity();
    assert!(
        (velocity.x - 1.0).abs() <= 0.114_850_000_000_000_01,
        "the wind charge keeps the raw facing step: {velocity:?}"
    );
    assert!(
        (velocity.y).abs() <= 0.114_850_000_000_000_01
            && (velocity.z).abs() <= 0.114_850_000_000_000_01,
        "the wind charge spread only affects the facing axis: {velocity:?}"
    );
    assert!(
        velocity.length() > 0.88 && velocity.length() < 1.12,
        "the wind charge speed matches the unnormalized facing vector: {velocity:?}"
    );
}

/// Vanilla `ProjectileItem.DispenseConfig.DEFAULT`: the projectile spawns at
/// `getDispensePosition(source, 0.7, (0, 0.1, 0))` and is shot with power
/// `1.1` and uncertainty `6.0`.
#[test]
fn dispenser_arrow_uses_the_default_vanilla_launch_config() {
    use crate::entity::Entity as _;
    use crate::entity::entities::ArrowEntity;
    use crate::entity::entities::objects::projectiles::AbstractArrow as _;

    let (fixture, container) = fixture(
        "dispenser_arrow_launch",
        ItemStack::with_count(&vanilla_items::ARROW, 1),
    );
    let world = &fixture.world;
    assert!(dispense_from(world, SOURCE, world.get_block_state(SOURCE)));
    assert!(container.lock().get_item(0).is_empty());

    let entities = world.get_entities_in_aabb(&steel_utils::WorldAabb::new(
        7.0, 62.0, 7.0, 12.0, 68.0, 10.0,
    ));
    let arrows: Vec<_> = entities
        .iter()
        .filter_map(|entity| entity.downcast_ref::<ArrowEntity>())
        .collect();
    assert_eq!(arrows.len(), 1);

    let arrow = arrows[0];
    let position = arrow.position();
    assert!(
        (position.x - 9.2).abs() < 1.0e-6
            && (position.y - 64.6).abs() < 1.0e-6
            && (position.z - 8.5).abs() < 1.0e-6,
        "the arrow spawns at the vanilla default dispense position: {position:?}"
    );

    let velocity = arrow.velocity();
    let speed = velocity.length();
    // Vanilla `Projectile.getMovementToShoot` adds a triangular deviation of
    // `0.0172275 * uncertainty` to the normalized facing direction, so the
    // launch speed and direction stay inside a known envelope around the
    // default power instead of hitting an exact value.
    let deviation = 0.017_227_5_f64 * 6.0_f64;
    let lowest = 1.1 * (1.0 - deviation);
    let highest =
        1.1 * ((1.0 + deviation) * (1.0 + deviation) + 2.0 * deviation * deviation).sqrt();
    assert!(
        speed >= lowest && speed <= highest,
        "the arrow is shot with the default power of 1.1: {velocity:?}"
    );
    assert!(velocity.x >= lowest, "the arrow flies east: {velocity:?}");

    let item = arrow.pickup_item();
    assert_eq!(item.item(), &*vanilla_items::ARROW);
    assert_eq!(item.count(), 1);
}

/// Vanilla `EnderpearlItem` is a plain `Item`, not a `ProjectileItem`, so the
/// dispenser ejects it as an ordinary item instead of launching a projectile.
#[test]
fn dispenser_ender_pearl_is_ejected_as_an_item_not_a_projectile() {
    let (fixture, container) = fixture(
        "dispenser_ender_pearl",
        ItemStack::with_count(&vanilla_items::ENDER_PEARL, 1),
    );
    let world = &fixture.world;
    assert!(dispense_from(world, SOURCE, world.get_block_state(SOURCE)));
    assert!(container.lock().get_item(0).is_empty());

    let entities = world.get_entities_in_aabb(&steel_utils::WorldAabb::new(
        7.0, 62.0, 7.0, 12.0, 68.0, 10.0,
    ));
    let items: Vec<_> = entities
        .iter()
        .filter_map(|entity| entity.downcast_ref::<ItemEntity>())
        .collect();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].get_item().item(), &*vanilla_items::ENDER_PEARL);
}

/// Vanilla `FlintAndSteelDispenseItemBehavior` primes TNT and only removes the
/// block when the prime was accepted. The tool only wears when the behavior
/// succeeded.
#[test]
fn dispenser_flint_and_steel_priming_tnt_removes_the_block_and_wears_the_tool() {
    use steel_registry::data_components::vanilla_components::DAMAGE;

    let (fixture, container) = fixture(
        "dispenser_flint_tnt",
        ItemStack::new(&vanilla_items::FLINT_AND_STEEL),
    );
    let world = &fixture.world;
    world.set_block(
        TARGET,
        vanilla_blocks::TNT.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    assert!(dispense_from(world, SOURCE, world.get_block_state(SOURCE)));

    assert!(
        world.get_block_state(TARGET).is_air(),
        "a successfully primed TNT block is removed"
    );
    assert_eq!(container.lock().get_item(0).get(DAMAGE), Some(&1));
}

/// When the TNT prime is rejected the TNT stays, the tool is not damaged and
/// the dispenser plays the failure event.
#[test]
fn dispenser_flint_and_steel_keeps_the_tnt_when_priming_is_rejected() {
    use steel_registry::data_components::vanilla_components::DAMAGE;

    let (fixture, container) = fixture(
        "dispenser_flint_tnt_rejected",
        ItemStack::new(&vanilla_items::FLINT_AND_STEEL),
    );
    let world = &fixture.world;
    world.set_block(
        TARGET,
        vanilla_blocks::TNT.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    world.set_game_rule(&TNT_EXPLODES, false);

    assert!(!dispense_from(world, SOURCE, world.get_block_state(SOURCE)));
    assert!(
        !world.get_block_state(TARGET).is_air(),
        "a rejected TNT prime keeps the block"
    );
    // Vanilla only damages the tool when the behavior succeeded, so the stack
    // keeps the `minecraft:damage` value of the vanilla item prototype.
    assert_eq!(container.lock().get_item(0).get(DAMAGE), Some(&0));
}

#[test]
fn dispenser_places_shulker_boxes_with_the_expected_facing_and_keeps_blocked_items() {
    for (support, expected_facing) in [(true, Direction::Up), (false, Direction::East)] {
        let (fixture, container) = fixture(
            "dispenser_shulker_placement",
            ItemStack::new(&vanilla_items::SHULKER_BOX),
        );
        let world = &fixture.world;
        if support {
            world.set_block(
                TARGET.below(),
                vanilla_blocks::STONE.default_state(),
                UpdateFlags::UPDATE_ALL,
            );
        }
        pulse(world);
        let state = world.get_block_state(TARGET);
        assert_eq!(state.get_block(), &vanilla_blocks::SHULKER_BOX);
        assert_eq!(
            state.get_value(&BlockStateProperties::FACING),
            expected_facing
        );
        assert!(container.lock().get_item(0).is_empty());
    }

    let (fixture, container) = fixture(
        "dispenser_shulker_blocked",
        ItemStack::new(&vanilla_items::SHULKER_BOX),
    );
    fixture.world.set_block(
        TARGET,
        vanilla_blocks::STONE.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    pulse(&fixture.world);
    assert_eq!(
        fixture.world.get_block_state(TARGET).get_block(),
        &vanilla_blocks::STONE
    );
    assert_eq!(container.lock().get_item(0).count(), 1);
}
