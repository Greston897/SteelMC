//! Fire block behavior implementation.
//!
//! Vanilla splits fire into `BaseFireBlock` (portal logic, placement checks) and `FireBlock`
//! (spreading, aging). This combines the portal-relevant parts from `BaseFireBlock`.

use std::sync::Arc;
use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, IntProperty};
use steel_registry::level_events;
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::vanilla_blocks;
use steel_registry::vanilla_damage_types;
use steel_registry::vanilla_dimension_types;
use steel_utils::axis::Axis;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId, Direction};

use crate::behavior::block::BlockBehavior;

const AGE: &IntProperty = &BlockStateProperties::AGE_15;
use crate::behavior::context::BlockPlaceContext;
use crate::entity::damage::DamageSource;
use crate::entity::{Entity, InsideBlockEffectCollector, InsideBlockEffectType};
use crate::player::Player;
use crate::portal::portal_shape::{PortalShape, nether_portal_config};
use crate::world::{LevelReader, ScheduledTickAccess, World};
/// Behavior for fire blocks.
#[block_behavior]
pub struct FireBlock {
    block: BlockRef,
}

impl FireBlock {
    /// Creates a new fire block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    /// Returns true if the world supports nether portal creation.
    ///
    /// Vanilla expresses this in terms of dimensions; Steel checks the loaded
    /// world's vanilla dimension type.
    pub(crate) fn in_portal_world(world: &World) -> bool {
        world.dimension_type == &vanilla_dimension_types::OVERWORLD
            || world.dimension_type == &vanilla_dimension_types::THE_NETHER
    }

    /// Checks if fire can be placed at `pos`, matching vanilla's `BaseFireBlock.canBePlacedAt`.
    /// Position must be air AND (fire can survive there OR it's a valid portal location).
    pub(crate) fn can_be_placed_at(
        world: &Arc<World>,
        pos: BlockPos,
        forward_dir: Direction,
    ) -> bool {
        if !world.get_block_state(pos).is_air() {
            return false;
        }
        Self::selected_fire_can_survive_at(world.as_ref(), pos)
            || Self::is_portal(world, pos, forward_dir)
    }

    /// Steel equivalent of vanilla's `BaseFireBlock.getState` for selecting
    /// between soul fire and regular fire.
    pub(crate) fn get_state(world: &dyn LevelReader, pos: BlockPos) -> BlockStateId {
        if SoulFireBlock::can_survive_at(world, pos) {
            vanilla_blocks::SOUL_FIRE.default_state()
        } else {
            vanilla_blocks::FIRE.default_state()
        }
    }

    /// Vanilla `FireBlock.getFireTickDelay`: `30 + random.nextInt(10)`.
    fn tick_delay() -> i32 {
        30 + rand::random_range(0..10)
    }

    fn selected_fire_can_survive_at(world: &dyn LevelReader, pos: BlockPos) -> bool {
        SoulFireBlock::can_survive_at(world, pos) || Self::can_survive_at(world, pos)
    }

    /// Matches vanilla's `FireBlock.canSurvive`: block below has a sturdy top face,
    /// or an adjacent block is flammable.
    fn can_survive_at(world: &dyn LevelReader, pos: BlockPos) -> bool {
        let below_pos = pos.below();
        if world.is_face_sturdy(world.get_block_state(below_pos), below_pos, Direction::Up) {
            return true;
        }
        // Vanilla `FireBlock.canSurvive`: fire also lives next to a block whose
        // `igniteOdds` is above zero, which is what lets fire climb a tree.
        Direction::ALL.into_iter().any(|direction| {
            Self::is_flammable(world.get_block_state(pos.relative(direction)))
        })
    }

    /// Vanilla `FireBlock.getIgniteOdds(BlockState)` on a live state: a
    /// waterlogged block never catches fire.
    fn is_flammable(state: BlockStateId) -> bool {
        if state.get_block() == &vanilla_blocks::FIRE
            || state.get_block() == &vanilla_blocks::SOUL_FIRE
        {
            return false;
        }
        if state.try_get_value(&BlockStateProperties::WATERLOGGED) == Some(true) {
            return false;
        }
        crate::behavior::flammable::ignite_odds(state.get_block()) > 0
    }

    /// Vanilla `FireBlock.canBurn`.
    fn can_burn(state: BlockStateId) -> bool {
        crate::behavior::flammable::burn_odds(state.get_block()) > 0
            || Self::is_flammable(state)
    }

    /// Vanilla `FireBlock.getBurnOdds(BlockState)`.
    fn burn_odds(state: BlockStateId) -> i32 {
        if state.try_get_value(&BlockStateProperties::WATERLOGGED) == Some(true) {
            return 0;
        }
        i32::try_from(crate::behavior::flammable::burn_odds(state.get_block())).unwrap_or(0)
    }

    /// Vanilla `FireBlock.isNearRain`.
    fn is_near_rain(world: &Arc<World>, pos: BlockPos) -> bool {
        world.is_raining_at(pos)
            || world.is_raining_at(pos.west())
            || world.is_raining_at(pos.east())
            || world.is_raining_at(pos.north())
            || world.is_raining_at(pos.south())
    }

    /// Vanilla `FireBlock.getStateWithAge`: soul fire keeps its own state.
    fn state_with_age(world: &dyn LevelReader, pos: BlockPos, age: u8) -> BlockStateId {
        let state = Self::get_state(world, pos);
        if state.get_block() == &vanilla_blocks::FIRE {
            state.set_value(AGE, age)
        } else {
            state
        }
    }

    /// Vanilla `FireBlock.checkBurnOut`: burn or age the neighbour that a fire is
    /// touching.
    fn check_burn_out(world: &Arc<World>, pos: BlockPos, chance: i32, age: u8) {
        let state = world.get_block_state(pos);
        if rand::random_range(0..chance) >= Self::burn_odds(state) {
            return;
        }
        let i32_age = i32::from(age);
        if rand::random_range(0..i32_age + 10) < 5 && !world.is_raining_at(pos) {
            let new_age = u8::min(15, age.saturating_add(rand::random_range(0..5) as u8 / 4));
            let aged = Self::state_with_age(world.as_ref(), pos, new_age);
            world.set_block(pos, aged, UpdateFlags::UPDATE_CLIENTS);
        } else {
            world.set_block(pos, vanilla_blocks::AIR.default_state(), UpdateFlags::UPDATE_ALL);
        }

        if state.get_block() == &vanilla_blocks::TNT {
            crate::behavior::blocks::TntBlock::prime(world, pos);
        }
    }

    /// Vanilla `FireBlock.getIgniteOdds(LevelReader, BlockPos)`: an occupied
    /// position picks up odds from its own block, an empty one looks down for
    /// something that can burn.
    fn ignition_odds_at(world: &dyn LevelReader, pos: BlockPos) -> i32 {
        let state = world.get_block_state(pos);
        if state.get_block() != &vanilla_blocks::AIR {
            if Self::is_flammable(state) {
                return 0;
            }
            if state.try_get_value(&BlockStateProperties::WATERLOGGED) == Some(true) {
                return 0;
            }
            return i32::try_from(crate::behavior::flammable::ignite_odds(state.get_block()))
                .unwrap_or(0);
        }

        // Vanilla walks the neighbouring blocks for the highest ignite odds.
        let mut best = 0;
        for direction in Direction::ALL {
            let neighbour = world.get_block_state(pos.relative(direction));
            let odds = i32::try_from(crate::behavior::flammable::ignite_odds(
                neighbour.get_block(),
            ))
            .unwrap_or(0);
            best = best.max(odds);
        }
        if best > 0 { (best + 10) / 2 } else { 0 }
    }


    /// Matches vanilla's `BaseFireBlock.isPortal`: checks if placing fire here could form a portal.
    /// Requires a portal-capable world, adjacent obsidian, and a valid empty portal shape.
    fn is_portal(world: &Arc<World>, pos: BlockPos, forward_dir: Direction) -> bool {
        if !Self::in_portal_world(world) {
            return false;
        }

        let has_obsidian = Direction::ALL.iter().any(|&dir| {
            world.get_block_state(pos.relative(dir)).get_block() == &vanilla_blocks::OBSIDIAN
        });
        if !has_obsidian {
            return false;
        }

        let preferred_axis = if forward_dir.get_axis().is_horizontal() {
            forward_dir.rotate_y_counter_clockwise().get_axis()
        } else if rand::random::<bool>() {
            Axis::X
        } else {
            Axis::Z
        };

        let config = nether_portal_config();
        PortalShape::find_empty_portal_shape_with_axis(world, pos, preferred_axis, &config)
            .is_some()
    }

    fn queue_entity_contact_effects(
        effect_collector: &mut InsideBlockEffectCollector,
        fire_damage: f32,
    ) {
        effect_collector.apply(InsideBlockEffectType::ClearFreeze);
        effect_collector.apply(InsideBlockEffectType::FireIgnite);
        effect_collector.run_after(
            InsideBlockEffectType::FireIgnite,
            Box::new(move |entity| {
                if !entity.fire_immune()
                    && let Some(entity_world) = entity.level()
                {
                    entity.hurt(
                        &entity_world,
                        &DamageSource::environment(&vanilla_damage_types::IN_FIRE),
                        fire_damage,
                    );
                }
            }),
        );
    }
}

impl BlockBehavior for FireBlock {
    /// Vanilla `FireBlock.tick`: survival check, `infiniburn` check, rain
    /// extinguishing, age advancement, adjacent block burn-out (including TNT
    /// ignition), and 3D neighbor fire spread respecting
    /// `fire_spread_radius_around_player`.
    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        world.schedule_block_tick_default(pos, self.block, Self::tick_delay());

        if !self.can_survive(state, world.as_ref(), pos) {
            world.set_block(
                pos,
                vanilla_blocks::AIR.default_state(),
                UpdateFlags::UPDATE_ALL,
            );
            return;
        }

        let below = world.get_block_state(pos.below());
        let infiniburn = world.is_infiniburn(below);
        let age: u8 = state.try_get_value(AGE).unwrap_or(0);

        if !infiniburn
            && world.is_raining()
            && Self::is_near_rain(world, pos)
            && rand::random::<f32>() < 0.2 + f32::from(age) * 0.03
        {
            world.set_block(
                pos,
                vanilla_blocks::AIR.default_state(),
                UpdateFlags::UPDATE_ALL,
            );
            return;
        }

        let next_age = u8::min(15, age.saturating_add(rand::random_range(0..3) as u8 / 2));
        if next_age != age {
            world.set_block(
                pos,
                state.set_value(AGE, next_age),
                UpdateFlags::UPDATE_NONE,
            );
        }

        let spread_radius = world
            .get_game_rule(&steel_registry::vanilla_game_rules::FIRE_SPREAD_RADIUS_AROUND_PLAYER);
        if spread_radius == 0 {
            return;
        }

        if !infiniburn {
            let has_valid_neighbor = Direction::ALL
                .iter()
                .any(|&dir| Self::can_burn(world.get_block_state(pos.relative(dir))));
            if !has_valid_neighbor {
                let below_pos = pos.below();
                if !below.is_face_sturdy(world.as_ref(), below_pos, Direction::Up) || age > 3 {
                    world.set_block(
                        pos,
                        vanilla_blocks::AIR.default_state(),
                        UpdateFlags::UPDATE_ALL,
                    );
                }
                return;
            }
            if age == 15 && rand::random_range(0..4) == 0 && !Self::can_burn(below) {
                world.set_block(
                    pos,
                    vanilla_blocks::AIR.default_state(),
                    UpdateFlags::UPDATE_ALL,
                );
                return;
            }
        }

        if spread_radius < 0 {
            return;
        }

        // Burn adjacent flammable blocks (horizontal 300, vertical 250).
        Self::check_burn_out(world, pos.east(), 300, next_age);
        Self::check_burn_out(world, pos.west(), 300, next_age);
        Self::check_burn_out(world, pos.below(), 250, next_age);
        Self::check_burn_out(world, pos.above(), 250, next_age);
        Self::check_burn_out(world, pos.north(), 300, next_age);
        Self::check_burn_out(world, pos.south(), 300, next_age);

        // Spread fire into the vanilla 3x6x3 neighborhood.
        let difficulty_bonus = match world.difficulty() {
            steel_utils::types::Difficulty::Peaceful => 0,
            steel_utils::types::Difficulty::Easy => 7,
            steel_utils::types::Difficulty::Normal => 14,
            steel_utils::types::Difficulty::Hard => 21,
        };
        for dx in -1..=1 {
            for dz in -1..=1 {
                for dy in -1..=4 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }
                    let target = pos.offset(dx, dy, dz);
                    if world.get_block_state(target).get_block() != &vanilla_blocks::AIR {
                        continue;
                    }
                    let odds = Self::ignition_odds_at(world.as_ref(), target);
                    if odds <= 0 {
                        continue;
                    }
                    let rate = if dy > 1 { 100 + (dy - 1) * 100 } else { 100 };
                    let mut max_odds = (odds + 40 + difficulty_bonus) / (i32::from(next_age) + 30);
                    if world.is_raining() && Self::is_near_rain(world, target) {
                        max_odds /= 2;
                    }
                    if max_odds > 0 && rand::random_range(0..rate) <= max_odds {
                        let spread_age = u8::min(
                            15,
                            next_age.saturating_add(rand::random_range(0..5) as u8 / 4),
                        );
                        world.set_block(
                            target,
                            Self::state_with_age(world.as_ref(), target, spread_age),
                            UpdateFlags::UPDATE_ALL,
                        );
                    }
                }
            }
        }
    }

    /// Removes fire when its supporting block no longer allows it to survive
    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if direction == Direction::Down && !self.can_survive(state, world, pos) {
            return vanilla_blocks::AIR.default_state();
        }
        state
    }
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        if SoulFireBlock::can_survive_at(context.world.as_ref(), context.place_pos()) {
            Some(vanilla_blocks::SOUL_FIRE.default_state())
        } else {
            Some(self.block.default_state())
        }
    }

    fn can_survive(&self, _state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        Self::can_survive_at(world, pos)
    }
    fn entity_inside(
        &self,
        _state: BlockStateId,
        _world: &Arc<World>,
        _pos: BlockPos,
        _entity: &dyn Entity,
        effect_collector: &mut InsideBlockEffectCollector,
        _is_precise: bool,
    ) {
        Self::queue_entity_contact_effects(effect_collector, 1.0);
    }

    fn on_place(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        old_state: BlockStateId,
        _moved_by_piston: bool,
    ) {
        // Only attempt portal creation when fire is newly placed, not when replacing itself
        if old_state.get_block() == state.get_block() {
            return;
        }

        if Self::in_portal_world(world)
            && let Some(shape) =
                PortalShape::find_empty_portal_shape(world, pos, &nether_portal_config())
        {
            shape.place_portal_blocks(world);
            return;
        }

        if !self.can_survive(state, world, pos) {
            world.set_block(
                pos,
                vanilla_blocks::AIR.default_state(),
                UpdateFlags::UPDATE_ALL,
            );
            return;
        }

        // Vanilla `FireBlock.onPlace` ends with
        // `level.scheduleTick(pos, this, getFireTickDelay(level.random))`.
        // Without that line a freshly placed fire never enters the block-tick
        // queue, so it would never age or burn out: fire has
        // `isRandomlyTicking() == false` and therefore no random-tick fallback.
        world.schedule_block_tick_default(pos, self.block, Self::tick_delay());
    }
    fn player_will_destroy(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _player: &Player,
    ) -> BlockStateId {
        world.level_event(level_events::SOUND_EXTINGUISH_FIRE, pos, 0, None);
        state
    }
}

/// Behavior for soul fire survival.
///
/// Vanilla keeps this as `SoulFireBlock`, separate from normal `FireBlock`.

#[block_behavior]
pub struct SoulFireBlock {
    block: BlockRef,
}

impl SoulFireBlock {
    /// Creates a new soul fire block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }

    fn can_survive_at(world: &dyn LevelReader, pos: BlockPos) -> bool {
        let block_below = world.get_block_state(pos.below()).get_block();
        block_below.has_tag(&BlockTag::SOUL_FIRE_BASE_BLOCKS)
    }
}

impl BlockBehavior for SoulFireBlock {
    /// Removes soul fire when its supporting block no longer allows it to survive
    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if direction == Direction::Down && !self.can_survive(state, world, pos) {
            return vanilla_blocks::AIR.default_state();
        }
        state
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let state = self.block.default_state();
        self.can_survive(state, context.world, context.place_pos())
            .then_some(state)
    }

    fn can_survive(&self, _state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        Self::can_survive_at(world, pos)
    }

    // Vanilla `SoulFireBlock` overrides neither `tick` nor `randomTick`: it
    // inherits `BaseFireBlock`, which has no scheduled-tick behaviour at all.
    // Only `FireBlock` re-queues, ages, and spreads. Giving soul fire a tick
    // would burn it down in rain and age it, which vanilla never does.

    fn entity_inside(
        &self,
        _state: BlockStateId,
        _world: &Arc<World>,
        _pos: BlockPos,
        _entity: &dyn Entity,
        effect_collector: &mut InsideBlockEffectCollector,
        _is_precise: bool,
    ) {
        FireBlock::queue_entity_contact_effects(effect_collector, 2.0);
    }
    fn player_will_destroy(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _player: &Player,
    ) -> BlockStateId {
        world.level_event(level_events::SOUND_EXTINGUISH_FIRE, pos, 0, None);
        state
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{
        blocks::block_state_ext::BlockStateExt, init_vanilla_registry, vanilla_blocks,
    };
    use steel_utils::{BlockPos, BlockStateId, Direction};

    use crate::behavior::block::BlockBehavior;

/// Vanilla `FireBlock.AGE`.
    use crate::test_support::TestLevel;

    use super::{FireBlock, SoulFireBlock};

    const POS: BlockPos = BlockPos::new(0, 64, 0);

    fn level_with_support(support_state: BlockStateId) -> TestLevel {
        TestLevel::default()
            .with_min_y(0)
            .with_block(POS.below(), support_state)
    }

    #[test]
    fn get_state_selects_soul_fire_on_soul_fire_base_block() {
        init_vanilla_registry();

        let level = level_with_support(vanilla_blocks::SOUL_SAND.default_state());

        assert_eq!(
            FireBlock::get_state(&level, POS).get_block(),
            &vanilla_blocks::SOUL_FIRE
        );
        assert!(FireBlock::selected_fire_can_survive_at(&level, POS));
    }

    #[test]
    fn get_state_selects_regular_fire_otherwise() {
        init_vanilla_registry();

        let level = level_with_support(vanilla_blocks::STONE.default_state());

        assert_eq!(
            FireBlock::get_state(&level, POS).get_block(),
            &vanilla_blocks::FIRE
        );
    }
    #[test]
    fn update_shape_removes_unsupported_fire() {
        init_vanilla_registry();
        let behavior = FireBlock::new(&vanilla_blocks::FIRE);
        let state = vanilla_blocks::FIRE.default_state();
        let level = TestLevel::default();
        let result = behavior.update_shape(
            state,
            &level,
            POS,
            Direction::Down,
            POS.below(),
            vanilla_blocks::AIR.default_state(),
        );
        assert_eq!(result.get_block(), &vanilla_blocks::AIR);
    }

    #[test]
    fn update_shape_removes_unsupported_soul_fire() {
        init_vanilla_registry();
        let behavior = SoulFireBlock::new(&vanilla_blocks::SOUL_FIRE);
        let state = vanilla_blocks::SOUL_FIRE.default_state();
        let level = TestLevel::default();
        let result = behavior.update_shape(
            state,
            &level,
            POS,
            Direction::Down,
            POS.below(),
            vanilla_blocks::AIR.default_state(),
        );
        assert_eq!(result.get_block(), &vanilla_blocks::AIR);
    }
}
