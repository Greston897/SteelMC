use std::sync::Arc;

use glam::DVec3;
use rand::RngExt;
use steel_macros::item_behavior;
use steel_registry::particle_type::ParticleData;
use steel_registry::{
    blocks::{block_state_ext::BlockStateExt, shapes::is_offset_shape_full_block},
    data_components::vanilla_components::USE_EFFECTS,
    item_stack::ItemStack,
    REGISTRY, RegistryExt as _, level_events, vanilla_blocks, vanilla_game_events,
    vanilla_particle_types,
};
use steel_utils::{BlockPos, Direction, types::UpdateFlags};

use crate::{
    behavior::blocks::vegetation::bonemealable::Bonemealable as _,
    behavior::{BLOCK_BEHAVIORS, InteractionResult, ItemBehavior, UseOnContext},
    entity::Entity,
    world::{LevelReader as _, World},
};

/// Behavior for the Bonemeal item.
#[item_behavior]
pub struct BoneMealItem;

impl BoneMealItem {
    fn cause_finish_use_vibration(context: &UseOnContext<'_>) {
        let interact_vibrations = context.inv.with_item(|item| {
            item.get(USE_EFFECTS)
                .is_some_and(|effects| effects.interact_vibrations)
        });
        if interact_vibrations {
            context
                .player
                .game_event(&vanilla_game_events::ITEM_INTERACT_FINISH);
        }
    }

    fn grow(world: &Arc<World>, pos: BlockPos) -> bool {
        let state = world.get_block_state(pos);
        let Some(behavior) = BLOCK_BEHAVIORS.get_behavior_for_state(state) else {
            return false;
        };
        if let Some(bonemealable) = behavior.as_bonemealable() {
            if !bonemealable.is_valid_bonemeal_target(state, world.as_ref(), pos) {
                return false;
            }

            let mut rng = rand::rng();
            if bonemealable.is_bonemeal_success(state, world, &mut rng, pos) {
                bonemealable.perform_bonemeal(state, world, &mut rng, pos);
                // Vanilla `BoneMealItem.growCrop` emits the happy-villager burst at
                // the block-specific particle position once growth succeeds.
                let particle_pos = bonemealable.particle_pos(bonemealable.bonemeal_action_type(), pos);
                Self::emit_growth_particles(world, particle_pos);
            }

            return true;
        }
        false
    }

    /// Vanilla `minecraft:underwater_bonemeals`: the blocks underwater bonemeal
    /// may grow. The tag is loaded from the vanilla datapack at build time, so a
    /// datapack that extends it automatically widens the growth set.
    fn underwater_bonemeal_blocks() -> &'static [steel_registry::blocks::BlockRef] {
        static CANDIDATES: std::sync::OnceLock<Vec<steel_registry::blocks::BlockRef>> =
            std::sync::OnceLock::new();
        CANDIDATES.get_or_init(|| {
            let tag = steel_utils::Identifier::vanilla_static("underwater_bonemeals");
            REGISTRY
                .blocks
                .iter_tag(&tag)
                .map(|block| block as steel_registry::blocks::BlockRef)
                .collect()
        })
    }

    /// Vanilla `BoneMealItem.spawnParticles`.
    fn emit_growth_particles(world: &Arc<World>, pos: BlockPos) {
        world.send_particles(
            ParticleData::simple(&vanilla_particle_types::HAPPY_VILLAGER),
            DVec3::new(
                f64::from(pos.0.x) + 0.5,
                f64::from(pos.0.y) + 0.5,
                f64::from(pos.0.z) + 0.5,
            ),
            15,
            DVec3::new(0.5, 0.5, 0.5),
            0.0,
        );
    }

    fn grow_water_plant(world: &Arc<World>, pos: BlockPos, _clicked_face: Direction) -> bool {
        let state = world.get_block_state(pos);
        if state.get_block() != &vanilla_blocks::WATER || !state.get_fluid_state().is_full() {
            return false;
        }

        let Some(bonemealable) = BLOCK_BEHAVIORS
            .get_behavior(&vanilla_blocks::SEAGRASS)
            .as_bonemealable()
        else {
            return false;
        };

        // Vanilla `BoneMealItem.growWaterPlant` picks a random block from the
        // `minecraft:underwater_bonemeals` block tag, falling back to seagrass.
        // The tag holds every coral kind plus seagrass, kelp, and sea pickles, so
        // bonemeal grows a mixed patch instead of only seagrass.
        let candidates = Self::underwater_bonemeal_blocks();
        let mut rng = rand::rng();

        'outer: for i in 0..128 {
            let mut new_pos = pos;
            let new_block = if candidates.is_empty() {
                &vanilla_blocks::SEAGRASS
            } else {
                candidates[rng.random_range(0..candidates.len())]
            };
            let new_state = new_block.default_state();

            for _ in 0..(i / 16) {
                new_pos = new_pos.offset(
                    rng.random_range(0i32..3) - 1,
                    (rng.random_range(0i32..3) - 1) * rng.random_range(0i32..3) / 2,
                    rng.random_range(0i32..3) - 1,
                );

                if is_offset_shape_full_block(
                    world
                        .get_block_state(new_pos)
                        .get_collision_shape_at(new_pos),
                ) {
                    continue 'outer;
                }
            }

            let Some(behavior) = BLOCK_BEHAVIORS.get_behavior_for_state(new_state) else {
                return false;
            };

            if behavior.can_survive(new_state, world, new_pos) {
                let current_state = world.get_block_state(new_pos);
                if current_state.get_block() == &vanilla_blocks::WATER
                    && current_state.get_fluid_state().is_full()
                {
                    world.set_block(new_pos, new_state, UpdateFlags::UPDATE_ALL);
                } else if current_state.get_block() == &vanilla_blocks::SEAGRASS
                    && bonemealable.is_valid_bonemeal_target(current_state, world.as_ref(), new_pos)
                    && rng.random_range(0..10) == 0
                {
                    bonemealable.perform_bonemeal(current_state, world, &mut rng, new_pos);
                }
            }
        }

        true
    }
}

impl ItemBehavior for BoneMealItem {
    fn use_on(&self, context: &mut UseOnContext) -> InteractionResult {
        if Self::grow(context.world, context.hit_result.block_pos) {
            context.inv.with_item(ItemStack::shrink_one);
            Self::cause_finish_use_vibration(context);
            context.world.level_event(
                level_events::PARTICLES_AND_SOUND_PLANT_GROWTH,
                context.hit_result.block_pos,
                15,
                None,
            );
            return InteractionResult::Success;
        }
        let state = context.world.get_block_state(context.hit_result.block_pos);
        let is_clicked_face_sturdy = context.world.is_face_sturdy(
            state,
            context.hit_result.block_pos,
            context.hit_result.direction,
        );
        if is_clicked_face_sturdy
            && Self::grow_water_plant(
                context.world,
                context
                    .hit_result
                    .block_pos
                    .relative(context.hit_result.direction),
                context.hit_result.direction,
            )
        {
            context.inv.with_item(ItemStack::shrink_one);
            Self::cause_finish_use_vibration(context);
            context.world.level_event(
                level_events::PARTICLES_AND_SOUND_PLANT_GROWTH,
                context
                    .hit_result
                    .block_pos
                    .relative(context.hit_result.direction),
                15,
                None,
            );
            return InteractionResult::Success;
        }
        InteractionResult::Pass
    }
}
