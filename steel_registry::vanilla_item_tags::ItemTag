//! Vanilla `net.minecraft.world.entity.animal.golem.CopperGolem`.
//!
//! Copper golem; it is a neutral golem in vanilla, so here it only retaliates.
//!
//! The AI is the shared `Monster` skeleton (float, melee attack, random stroll,
//! look goals) plus the standard hostile target selector; attributes and
//! dimensions come from the extracted `entities.json`.
//!
use std::iter::empty;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::BlockStateProperties;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::item_stack::ItemStack;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::CopperGolemEntityData;
use steel_registry::blocks::properties::Pose::Running;
use steel_registry::blocks::properties::Pose::Sitting;
use steel_registry::blocks::properties::Pose::Standing;
use steel_registry::blocks::properties::Pose::Star;
use steel_registry::sound_events::ITEM_AXE_SCRAPE;
use steel_registry::{
    level_events, sound_events, vanilla_attributes, vanilla_blocks, vanilla_game_rules,
    vanilla_items,
};
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::types::{InteractionHand, UpdateFlags};
use steel_utils::{
    BlockPos, BlockStateId, Direction, Downcast as _, DowncastType, DowncastTypeKey,
};

use crate::behavior::InteractionResult;
use crate::block_entity::entities::CopperGolemStatueBlockEntity;
use crate::player::Player;
use crate::entity::ai::goal::{
    FloatGoal, HurtByTargetGoal, LookAtPlayerGoal, MeleeAttackGoal, NearestAttackableTargetGoal,
    RandomLookAroundGoal, WaterAvoidingRandomStrollGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::leash::Leashable;
use crate::entity::{
    DefaultEntityPush, Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntityPose,
    EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase, Monster, PathfinderMob,
    RemovalReason, SharedEntity, sync_dirty_mob_effects,
};
use crate::inventory::equipment::EquipmentSlot;
use crate::physics::MoveResult;
use crate::world::World;

const DEFAULT_STEP_HEIGHT: f32 = 0.6;

/// Vanilla `WeatheringCopper.WeatherState.UNAFFECTED`.
const WEATHERING_UNAFFECTED: i32 = 0;
/// Vanilla `WeatheringCopper.WeatherState.OXIDIZED`.
const WEATHERING_OXIDIZED: i32 = 3;

/// Vanilla `WeatheringCopper.WeatherState.CODEC` names.
const WEATHERING_NAMES: [&str; 4] = ["unaffected", "exposed", "weathered", "oxidized"];

/// Parses a stored weathering state name, defaulting to `unaffected`.
fn weathering_state_from_name(name: &str) -> i32 {
    WEATHERING_NAMES
        .iter()
        .position(|candidate| *candidate == name)
        .map_or(0, |index| index as i32)
}

/// Vanilla `Direction.fromYRot`: the nearest horizontal direction to a yaw.
fn direction_from_y_rot(y_rot: f32) -> Direction {
    let quarter_turns = (y_rot / 90.0).round() as i32;
    match quarter_turns.rem_euclid(4) {
        0 => Direction::South,
        1 => Direction::West,
        2 => Direction::North,
        _ => Direction::East,
    }
}

/// Vanilla `net.minecraft.world.entity.monster.Silverfish`.
#[entity_behavior(class = "CopperGolem")]
pub struct CopperGolemEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<CopperGolemEntityData>,
    /// Vanilla `nextWeatheringTick`: `-1` = never scheduled yet, `-2` = waxed,
    /// anything else = absolute game time of the next oxidation attempt.
    next_weathering_tick: AtomicI64,
}

// SAFETY: This key is owned by Steel and uniquely identifies `CopperGolemEntity`.
unsafe impl DowncastType for CopperGolemEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/copper_golem");
}

impl CopperGolemEntity {
    /// Creates a new entity of this vanilla type.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Loads a saved entity of this vanilla type.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self::new_with_base(
            EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
        )
    }

    fn new_with_base(base: EntityBase, entity_type: EntityTypeRef) -> Self {
        let living_base = LivingEntityBase::new(entity_type);
        let mob_base = MobBase::new();

        let mut entity_data = CopperGolemEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(2, MeleeAttackGoal::new(1.0, false));
            goal_selector.add_goal(7, WaterAvoidingRandomStrollGoal::new(1.0));
            goal_selector.add_goal(8, LookAtPlayerGoal::new(8.0));
            goal_selector.add_goal(9, RandomLookAroundGoal::new());

            let mut target_selector = mob_base.target_selector().lock();
            target_selector.add_goal(1, HurtByTargetGoal::new().set_alert_others(empty()));
            target_selector.add_goal(
                2,
                NearestAttackableTargetGoal::new_for_players(true, |_, _| true),
            );
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            entity_data: SyncMutex::new(entity_data),
            // Vanilla `CopperGolem.nextWeatheringTick` starts at `-1`.
            next_weathering_tick: AtomicI64::new(-1),
        }
    }

    /// Applies the oxidation stage inherited from a copper construction block.
    pub(crate) fn set_spawn_weather_state(&self, state: i32) {
        self.set_weather_state(state);
    }

    /// Vanilla `CopperGolem.getWeatherState`.
    fn weather_state(&self) -> i32 {
        *self.entity_data.lock().weather_state.get()
    }

    /// Vanilla `WeatheringCopper.WeatherState.getSerializedName`.
    fn weather_state_name(&self) -> &'static str {
        let state = self.weather_state();
        if state < 0 || state as usize >= WEATHERING_NAMES.len() {
            return WEATHERING_NAMES[0];
        }
        WEATHERING_NAMES[state as usize]
    }

    /// Vanilla `CopperGolem.setWeatherState`.
    fn set_weather_state(&self, state: i32) {
        self.entity_data.lock().weather_state.set(state);
    }

    /// Vanilla `CopperGolem.nextWeatheringTick` accessors.
    fn next_weathering_tick(&self) -> i64 {
        self.next_weathering_tick.load(Ordering::Relaxed)
    }

    fn set_next_weathering_tick(&self, tick: i64) {
        self.next_weathering_tick.store(tick, Ordering::Relaxed);
    }

    /// Vanilla `CopperGolem.tick` -> `updateWeathering`.
    ///
    /// The first tick after a spawn schedules an oxidation attempt 504000..552000
    /// game ticks out (~7..7.7 in-game days). Once the golem is fully oxidized it
    /// tries to turn into a statue: the block under it must be air and the roll
    /// must be under `0.0058` every tick.
    fn update_weathering(&self, world: &Arc<World>) {
        let next = self.next_weathering_tick();
        if next == -2 {
            // Waxed: vanilla stops weathering entirely.
            return;
        }
        let game_time = world.game_time();
        if next == -1 {
            self.set_next_weathering_tick(game_time + rand::random_range(504_000..=552_000));
            return;
        }
        if game_time < next {
            return;
        }

        let state = self.weather_state();
        let fully_oxidized = state == WEATHERING_OXIDIZED;
        if !fully_oxidized {
            let new_state = (state + 1).min(WEATHERING_OXIDIZED);
            self.set_weather_state(new_state);
            self.set_next_weathering_tick(if new_state == WEATHERING_OXIDIZED {
                0
            } else {
                next + rand::random_range(504_000..=552_000)
            });
        }

        if fully_oxidized && self.can_turn_to_statue(world) {
            self.turn_to_statue(world);
        }
    }

    /// Vanilla `CopperGolem.canTurnToStatue`.
    fn can_turn_to_statue(&self, world: &Arc<World>) -> bool {
        world.get_block_state(Entity::block_position(self)).is_air()
            && rand::random::<f32>() <= 0.0058
    }

    /// Vanilla `CopperGolem.turnToStatue`: replace the golem's block with an
    /// oxidized statue carrying its name, then drop its equipment.
    fn turn_to_statue(&self, world: &Arc<World>) {
        let pos = Entity::block_position(self);
        let pose = match rand::random_range(0..4) {
            0 => Standing,
            1 => Sitting,
            2 => Running,
            _ => Star,
        };
        let state = vanilla_blocks::OXIDIZED_COPPER_GOLEM_STATUE
            .default_state()
            .set_value(&BlockStateProperties::COPPER_GOLEM_POSE, pose)
            .set_value(
                &BlockStateProperties::HORIZONTAL_FACING,
                direction_from_y_rot(Entity::rotation(self).0),
            );
        // `3` = UPDATE_NEIGHBORS | UPDATE_CLIENTS, matching vanilla `setBlock`.
        world.set_block(pos, state, UpdateFlags::UPDATE_ALL);

        if let Some(block_entity) = world.get_block_entity(pos)
            && let Some(statue) = block_entity
                .downcast_ref::<CopperGolemStatueBlockEntity>(
            )
        {
            statue.create_statue(self);
        }

        Mob::drop_preserved_equipment(self);
        self.set_removed(RemovalReason::Discarded);
        self.play_sound(&sound_events::ENTITY_COPPER_GOLEM_BECOME_STATUE, 1.0, 1.0);
        if Leashable::is_leashed(self) {
            if world.get_game_rule(&vanilla_game_rules::ENTITY_DROPS) {
                Leashable::drop_leash(self);
            } else {
                Leashable::remove_leash(self);
            }
        }
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        sync_dirty_mob_effects(&self.living_base, &self.entity_data);
    }
    /// Vanilla `itemStack.hurtAndBreak(1, player, hand.asEquipmentSlot())`.
    fn hurt_player_held_axe(player: &Player, hand: InteractionHand) {
        let mut inventory = player.inventory.lock();
        let stack = inventory.get_item_in_hand_mut(hand);
        if stack.is_empty() {
            return;
        }
        stack.hurt_and_break(1, player.has_infinite_materials());
    }
}

impl DefaultEntityPush for CopperGolemEntity {}

impl Entity for CopperGolemEntity {
    fn is_hard_collision_relevant(&self) -> bool {
        false
    }

    fn as_default_entity_push(&self) -> Option<&dyn DefaultEntityPush> {
        Some(self)
    }

    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn movement_emission(&self) -> EntityMovementEmission {
        EntityMovementEmission::Events
    }

    fn base_tick(&self) {
        Mob::base_tick_mob(self);
        // Vanilla `CopperGolem.tick` calls `updateWeathering` on the server every
        // tick. Steel has no client side, so the check collapses to "has a
        // level", which is also what keeps a detached entity from ticking.
        if let Some(world) = Entity::level(self) {
            self.update_weathering(&world);
        }
    }

    fn dimensions_for_pose(&self, _pose: EntityPose) -> EntityDimensions {
        let scale = LivingEntity::get_scale(self);
        if self.entity_type.fixed {
            self.entity_type.dimensions
        } else {
            self.entity_type.dimensions.scale(scale)
        }
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(&self.entity_data)
    }

    fn update_data_before_sync(&self) {
        self.update_dirty_mob_effect_entity_data();
    }

    fn max_up_step(&self) -> f32 {
        self.attributes()
            .lock()
            .get_value(vanilla_attributes::STEP_HEIGHT)
            .unwrap_or(f64::from(DEFAULT_STEP_HEIGHT)) as f32
    }

    fn sound_source(&self) -> SoundSource {
        Monster::sound_source_monster(self)
    }

    fn fall_sounds(&self) -> (SoundEventRef, SoundEventRef) {
        Monster::fall_sounds_monster(self)
    }

    fn swim_sound(&self) -> SoundEventRef {
        Monster::swim_sound_monster(self)
    }

    fn swim_splash_sound(&self) -> SoundEventRef {
        Monster::swim_splash_sound_monster(self)
    }

    fn play_step_sound(&self, _pos: BlockPos, _block_state: BlockStateId) {
        self.play_sound(&sound_events::ENTITY_COPPER_GOLEM_STEP, 0.15, 1.0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
        // Vanilla `CopperGolem.addAdditionalSaveData`.
        nbt.insert("next_weather_age", self.next_weathering_tick());
        nbt.insert("weather_state", self.weather_state_name());
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        // Vanilla `CopperGolem.readAdditionalSaveData`.
        self.set_next_weathering_tick(nbt.long_value("next_weather_age").unwrap_or(-1));
        let state = nbt
            .string("weather_state")
            .map_or(WEATHERING_UNAFFECTED, |value| {
                weathering_state_from_name(&value.to_str())
            });
        self.set_weather_state(state);
    }
}

impl LivingEntity for CopperGolemEntity {
    fn living_base(&self) -> &LivingEntityBase {
        &self.living_base
    }

    fn get_health(&self) -> f32 {
        *self.entity_data.lock().living_entity().health.get()
    }

    fn set_health(&self, health: f32) {
        let max_health = self.get_max_health();
        let clamped = health.clamp(0.0, max_health);
        self.entity_data
            .lock()
            .living_entity_mut()
            .health
            .set(clamped);
    }

    fn hurt_sound(&self, _source: &DamageSource) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_COPPER_GOLEM_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_COPPER_GOLEM_DEATH)
    }

    fn should_drop_experience(&self) -> bool {
        Monster::should_drop_experience_monster(self)
    }

    fn should_drop_loot(&self, world: &World) -> bool {
        Monster::should_drop_loot_monster(self, world)
    }

    fn server_ai_step(&self, entity: &SharedEntity) {
        Mob::mob_server_ai_step(self, entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        Monster::monster_ai_step(self, entity)
    }
}

impl Mob for CopperGolemEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    /// Vanilla `CopperGolem.mobInteract`: shears drop the held item, honeycomb
    /// waxes the golem permanently, and an axe scrapes one oxidation stage back
    /// off an oxidized golem.
    fn mob_interact(
        &self,
        player: &Player,
        hand: InteractionHand,
    ) -> InteractionResult {
        use steel_registry::vanilla_item_tags::ItemTag;

        let stack = {
            let inventory = player.inventory.lock();
            let stack = inventory.get_item_in_hand(hand);
            stack.copy_with_count(stack.count())
        };

        // An empty hand takes the item the golem is carrying.
        if stack.is_empty() {
            let held = self
                .living_base
                .equipment()
                .lock()
                .get_ref(EquipmentSlot::MainHand)
                .clone();
            if !held.is_empty() {
                self.living_base
                    .equipment()
                    .lock()
                    .set(EquipmentSlot::MainHand, ItemStack::empty());
                self.spawn_at_location(held, 0.0);
                return InteractionResult::Success;
            }
        }

        if stack.is(&vanilla_items::HONEYCOMB) && self.next_weathering_tick() != -2 {
            if let Some(world) = Entity::level(self) {
                world.level_event(
                    level_events::PARTICLES_WAX_ON,
                    Entity::block_position(self),
                    0,
                    None,
                );
            }
            self.set_next_weathering_tick(-2);
            self.use_player_item(player, hand);
            return InteractionResult::SuccessServer;
        }

        if stack.item().has_tag(&ItemTag::AXES) {
            if self.next_weathering_tick() == -2 {
                // Scraping the wax off restarts weathering from "never scheduled".
                if let Some(world) = Entity::level(self) {
                    world.play_sound(
                        &ITEM_AXE_SCRAPE,
                        SoundSource::Neutral,
                        Entity::block_position(self),
                        1.0,
                        1.0,
                        None,
                    );
                    world.level_event(
                        level_events::PARTICLES_WAX_OFF,
                        Entity::block_position(self),
                        0,
                        None,
                    );
                }
                self.set_next_weathering_tick(-1);
                Self::hurt_player_held_axe(player, hand);
                return InteractionResult::SuccessServer;
            }

            if self.weather_state() != WEATHERING_UNAFFECTED {
                if let Some(world) = Entity::level(self) {
                    world.play_sound(
                        &ITEM_AXE_SCRAPE,
                        SoundSource::Neutral,
                        Entity::block_position(self),
                        1.0,
                        1.0,
                        None,
                    );
                    world.level_event(
                        level_events::PARTICLES_SCRAPE,
                        Entity::block_position(self),
                        0,
                        None,
                    );
                }
                self.set_next_weathering_tick(-1);
                self.set_weather_state((self.weather_state() - 1).max(WEATHERING_UNAFFECTED));
                Self::hurt_player_held_axe(player, hand);
                return InteractionResult::SuccessServer;
            }
        }

        InteractionResult::Pass
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl Monster for CopperGolemEntity {}

impl PathfinderMob for CopperGolemEntity {}
