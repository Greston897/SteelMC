//! Fire block behavior implementation.
//!
//! Vanilla splits fire into `BaseFireBlock` (portal logic, placement checks) and `FireBlock`
//! (spreading, aging). This combines the portal-relevant parts from `BaseFireBlock`.

use std::sync::Arc;

use glam::DVec3;
use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, IntProperty};
use steel_registry::level_events;
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::vanilla_blocks;
use steel_registry::vanilla_damage_types;
use steel_registry::vanilla_dimension_types;
use steel_registry::vanilla_game_rules::FIRE_SPREAD_RADIUS_AROUND_PLAYER;
use steel_utils::axis::Axis;
use steel_utils::types::Difficulty::Easy;
use steel_utils::types::Difficulty::Hard;
use steel_utils::types::Difficulty::Normal;
use steel_utils::types::Difficulty::Peaceful;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId, Direction};

use crate::behavior::block::BlockBehavior;
use crate::behavior::blocks::TntBlock;
use crate::behavior::flammable::burn_odds;
use crate::behavior::flammable::ignite_odds;

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
            Self::fire_state_for_placement(world, pos)
        }
    }

    /// Vanilla `FireBlock.getStateForPlacement(BlockGetter, BlockPos)`.
    ///
    /// Unlike `BaseFireBlock.getState` this never selects soul fire: it is
    /// the state the fire block itself takes when it is placed.
    fn fire_state_for_placement(world: &dyn LevelReader, pos: BlockPos) -> BlockStateId {
        let below = pos.below();
        let below_state = world.get_block_state(below);
        if Self::can_burn(below_state) || world.is_face_sturdy(below_state, below, Direction::Up) {
            return vanilla_blocks::FIRE.default_state();
        }
        let mut state = vanilla_blocks::FIRE.default_state();
        for (direction, property) in [
            (Direction::North, &BlockStateProperties::NORTH),
            (Direction::East, &BlockStateProperties::EAST),
            (Direction::South, &BlockStateProperties::SOUTH),
            (Direction::West, &BlockStateProperties::WEST),
            (Direction::Up, &BlockStateProperties::UP),
        ] {
            state = state.set_value(
                property,
                Self::can_burn(world.get_block_state(pos.relative(direction))),
            );
        }
        state
    }

    /// Vanilla `FireBlock.getFireTickDelay`: `30 + random.nextInt(10)`.
    fn tick_delay() -> i32 {
        30 + rand::random_range(0..10)
    }

    fn selected_fire_can_survive_at(world: &dyn LevelReader, pos: BlockPos) -> bool {
        SoulFireBlock::can_survive_at(world, pos) || Self::can_survive_at(world, pos)
    }

    /// Vanilla `FireBlock.isValidFireLocation`: at least one of the six
    /// neighbours can burn.
    fn is_valid_fire_location(world: &dyn LevelReader, pos: BlockPos) -> bool {
        Direction::ALL
            .into_iter()
            .any(|direction| Self::can_burn(world.get_block_state(pos.relative(direction))))
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
        Self::is_valid_fire_location(world, pos)
    }

    /// Vanilla `FireBlock.getIgniteOdds(BlockState)`: the raw ignite odds of
    /// the block, or zero when the state is waterlogged.
    fn ignite_odds_of_state(state: BlockStateId) -> i32 {
        if state.try_get_value(&BlockStateProperties::WATERLOGGED) == Some(true) {
            return 0;
        }
        i32::try_from(ignite_odds(state.get_block())).unwrap_or(0)
    }

    /// Vanilla `FireBlock.canBurn`: `getIgniteOdds(state) > 0`.
    fn can_burn(state: BlockStateId) -> bool {
        Self::ignite_odds_of_state(state) > 0
    }

    /// Vanilla `FireBlock.getBurnOdds(BlockState)`.
    fn burn_odds(state: BlockStateId) -> i32 {
        if state.try_get_value(&BlockStateProperties::WATERLOGGED) == Some(true) {
            return 0;
        }
        i32::try_from(burn_odds(state.get_block())).unwrap_or(0)
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
        let odds = Self::burn_odds(state);
        if rand::random_range(0..chance) >= odds {
            return;
        }
        if rand::random_range(0..i32::from(age) + 10) < 5 && !world.is_raining_at(pos) {
            let new_age = u8::min(15, age.saturating_add(rand::random_range(0..5) as u8 / 4));
            world.set_block(
                pos,
                Self::state_with_age(world.as_ref(), pos, new_age),
                UpdateFlags::UPDATE_ALL,
            );
        } else {
            world.set_block(
                pos,
                vanilla_blocks::AIR.default_state(),
                UpdateFlags::UPDATE_ALL,
            );
        }

        if state.get_block() == &vanilla_blocks::TNT {
            TntBlock::prime(world, pos);
        }
    }

    /// Vanilla `FireBlock.getIgniteOdds(LevelReader, BlockPos)`: an occupied
    /// position never ignites, and an empty one takes the highest raw ignite
    /// odds of its six neighbours.
    fn ignition_odds_at(world: &dyn LevelReader, pos: BlockPos) -> i32 {
        if !world.get_block_state(pos).is_air() {
            return 0;
        }
        let mut odds = 0;
        for direction in Direction::ALL {
            let neighbour = world.get_block_state(pos.relative(direction));
            odds = odds.max(Self::ignite_odds_of_state(neighbour));
        }
        odds
    }

    /// Vanilla `ServerLevel.canSpreadFireAround(BlockPos)`.
    ///
    /// The `fire_spread_radius_around_player` rule is `-1` for "always",
    /// otherwise a non-spectator player has to be strictly closer to the block
    /// than the configured radius.
    fn can_spread_fire_around(world: &Arc<World>, pos: BlockPos) -> bool {
        let radius = world.get_game_rule(&FIRE_SPREAD_RADIUS_AROUND_PLAYER);
        if radius == -1 {
            return true;
        }
        let target = DVec3::new(f64::from(pos.x()), f64::from(pos.y()), f64::from(pos.z()));
        let mut close_enough = false;
        world.players.iter_players(|_, player| {
            if player.is_spectator() {
                return true;
            }
            if player.position().distance(target) < f64::from(radius) {
                close_enough = true;
                return false;
            }
            true
        });
        close_enough
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
    /// Vanilla `FireBlock.tick`: the `canSpreadFireAround` gate, the survival
    /// check, the `infiniburn` check, rain extinguishing, age advancement,
    /// adjacent block burn-out (including TNT ignition), and 3D neighbour fire
    /// spread.
    #[expect(clippy::too_many_lines, reason = "splitting would hurt readability")]
    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        world.schedule_block_tick_default(pos, self.block, Self::tick_delay());

        // Vanilla checks `ServerLevel.canSpreadFireAround` first: when no
        // player is close enough the fire neither ages nor burns away.
        if !Self::can_spread_fire_around(world, pos) {
            return;
        }

        if !self.can_survive(state, world.as_ref(), pos) {
            // Vanilla removes the block but keeps running the rest of the tick.
            world.set_block(
                pos,
                vanilla_blocks::AIR.default_state(),
                UpdateFlags::UPDATE_ALL,
            );
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

        if !infiniburn {
            if !Self::is_valid_fire_location(world.as_ref(), pos) {
                let below_pos = pos.below();
                if !world.is_face_sturdy(below, below_pos, Direction::Up) || age > 3 {
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

        // Vanilla `EnvironmentAttributes.INCREASED_FIRE_BURNOUT` shortens the
        // burn-out window by 50 ticks and halves the spread odds.
        let increased_burnout = world.increased_fire_burnout();
        let extra = if increased_burnout { -50 } else { 0 };

        // Burn adjacent flammable blocks (horizontal 300, vertical 250).
        Self::check_burn_out(world, pos.east(), 300 + extra, age);
        Self::check_burn_out(world, pos.west(), 300 + extra, age);
        Self::check_burn_out(world, pos.below(), 250 + extra, age);
        Self::check_burn_out(world, pos.above(), 250 + extra, age);
        Self::check_burn_out(world, pos.north(), 300 + extra, age);
        Self::check_burn_out(world, pos.south(), 300 + extra, age);

        // Spread fire into the vanilla 3x6x3 neighborhood.
        let difficulty_bonus = match world.difficulty() {
            Peaceful => 0,
            Easy => 7,
            Normal => 14,
            Hard => 21,
        };
        for dx in -1..=1 {
            for dz in -1..=1 {
                for dy in -1..=4 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }
                    let target = pos.offset(dx, dy, dz);
                    let ignite_odds = Self::ignition_odds_at(world.as_ref(), target);
                    if ignite_odds <= 0 {
                        continue;
                    }
                    let rate = if dy > 1 { 100 + (dy - 1) * 100 } else { 100 };
                    let mut odds = (ignite_odds + 40 + difficulty_bonus) / (i32::from(age) + 30);
                    if increased_burnout {
                        odds /= 2;
                    }
                    if odds <= 0 {
                        continue;
                    }
                    if rand::random_range(0..rate) > odds {
                        continue;
                    }
                    if world.is_raining() && Self::is_near_rain(world, target) {
                        continue;
                    }
                    let spread_age =
                        u8::min(15, age.saturating_add(rand::random_range(0..5) as u8 / 4));
                    world.set_block(
                        target,
                        Self::state_with_age(world.as_ref(), target, spread_age),
                        UpdateFlags::UPDATE_ALL,
                    );
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
        _direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if !self.can_survive(state, world, pos) {
            return vanilla_blocks::AIR.default_state();
        }
        Self::state_with_age(world, pos, state.get_value(AGE))
    }
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        // Vanilla `FireBlock.getStateForPlacement(BlockPlaceContext)` uses the
        // non-soul placement state.
        Some(Self::fire_state_for_placement(
            context.world.as_ref(),
            context.place_pos(),
        ))
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
    fn wool_and_carpet_have_vanilla_fire_odds() {
        init_vanilla_registry();
        for block in [&vanilla_blocks::WHITE_WOOL, &vanilla_blocks::BLACK_WOOL] {
            let state = block.default_state();
            assert_eq!(FireBlock::ignite_odds_of_state(state), 30);
            assert_eq!(FireBlock::burn_odds(state), 60);
        }
        for block in [&vanilla_blocks::WHITE_CARPET, &vanilla_blocks::BLACK_CARPET] {
            let state = block.default_state();
            assert_eq!(FireBlock::ignite_odds_of_state(state), 60);
            assert_eq!(FireBlock::burn_odds(state), 20);
        }
        assert!(!FireBlock::can_burn(
            vanilla_blocks::CRIMSON_STEM.default_state()
        ));
    }

    #[test]
    fn waterlogged_flammable_blocks_never_attach_fire_faces() {
        use steel_registry::blocks::properties::BlockStateProperties;

        init_vanilla_registry();
        let dry = vanilla_blocks::OAK_SLAB.default_state();
        let wet = dry.set_value(&BlockStateProperties::WATERLOGGED, true);
        assert!(FireBlock::can_burn(dry));
        assert!(!FireBlock::can_burn(wet));
        let dry_level = TestLevel::default().with_block(POS.north(), dry);
        let wet_level = TestLevel::default().with_block(POS.north(), wet);
        assert!(FireBlock::get_state(&dry_level, POS).get_value(&BlockStateProperties::NORTH));
        assert!(!FireBlock::get_state(&wet_level, POS).get_value(&BlockStateProperties::NORTH));
        assert!(FireBlock::can_survive_at(&dry_level, POS));
        assert!(!FireBlock::can_survive_at(&wet_level, POS));
    }

    #[test]
    fn ignition_odds_are_raw_and_skip_occupied_positions() {
        init_vanilla_registry();

        // Vanilla `FireBlock.getIgniteOdds(LevelReader, BlockPos)` returns the
        // raw neighbour odds for an empty position and zero for an occupied
        // one — never the midpoint transform Steel used before.
        let leaves = vanilla_blocks::OAK_LEAVES.default_state();
        let level = TestLevel::default().with_block(POS.north(), leaves);
        assert_eq!(FireBlock::ignition_odds_at(&level, POS), 30);

        let occupied = TestLevel::default()
            .with_block(POS.north(), leaves)
            .with_block(POS, vanilla_blocks::STONE.default_state());
        assert_eq!(FireBlock::ignition_odds_at(&occupied, POS), 0);
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
    fn horizontal_support_removal_extinguishes_fire() {
        init_vanilla_registry();
        let level = TestLevel::default();
        let state = vanilla_blocks::FIRE.default_state();
        let behavior = FireBlock::new(&vanilla_blocks::FIRE);
        assert!(
            behavior
                .update_shape(
                    state,
                    &level,
                    POS,
                    Direction::West,
                    POS.west(),
                    vanilla_blocks::AIR.default_state()
                )
                .is_air()
        );
    }

    #[test]
    fn changing_floor_to_soul_sand_converts_existing_fire() {
        init_vanilla_registry();
        let level = level_with_support(vanilla_blocks::SOUL_SAND.default_state());
        let behavior = FireBlock::new(&vanilla_blocks::FIRE);
        let result = behavior.update_shape(
            vanilla_blocks::FIRE.default_state(),
            &level,
            POS,
            Direction::Down,
            POS.below(),
            vanilla_blocks::SOUL_SAND.default_state(),
        );
        assert_eq!(result.get_block(), &vanilla_blocks::SOUL_FIRE);
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
