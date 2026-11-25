//! Vanilla tadpole entity.
//!
//! Tadpoles are the juvenile form of frogs. They swim, flop on land like fish,
//! grow into a frog and accelerate that growth when fed slime balls.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{
    EntityAttachmentPoint, EntityAttachments, EntityDimensions, EntityTypeRef,
};
use steel_registry::item_stack::ItemStack;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_attributes;
use steel_registry::vanilla_entity_data::TadpoleEntityData;
use steel_registry::vanilla_item_tags::ItemTag;
use steel_registry::{REGISTRY, TaggedRegistryExt, sound_events};
use steel_utils::locks::SyncMutex;
use steel_utils::types::InteractionHand;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::behavior::InteractionResult;
use crate::entity::ai::goal::{PanicGoal, RandomSwimmingGoal};
use crate::entity::ai::path::PathType;
use crate::entity::damage::DamageSource;
use crate::entity::entities::mobs::passive::fish::{
    check_water_animal_spawn_rules, flop, tick_breathing, travel_in_water,
};
use crate::entity::{
    AgeableMob, AgeableMobBase, DefaultEntityPush, Entity, EntityBase, EntityBaseLoad,
    EntityMovementEmission, EntityPose, EntitySpawnReason, EntitySyncedData, LivingEntity,
    LivingEntityBase, Mob, MobBase, PathfinderMob, SharedEntity, SpawnGroupData, SpawnRule,
    sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::player::Player;
use crate::world::World;

const TADPOLE_PASSENGER_ATTACHMENTS: [EntityAttachmentPoint; 1] =
    [EntityAttachmentPoint::new(0.0, 0.15, 0.0)];
const TADPOLE_WIDTH: f32 = 0.4;
const TADPOLE_HEIGHT: f32 = 0.3;
const TADPOLE_EYE_HEIGHT: f32 = 0.195;

const TADPOLE_DIMENSIONS: EntityDimensions = EntityDimensions::new_with_attachments(
    TADPOLE_WIDTH,
    TADPOLE_HEIGHT,
    TADPOLE_EYE_HEIGHT,
    EntityAttachments::new(&TADPOLE_PASSENGER_ATTACHMENTS, &[], &[], &[]),
);

/// Vanilla `Tadpole` movement speed from `createAttributes`.
const TADPOLE_MOVEMENT_SPEED: f64 = 1.0;
/// Vanilla `Tadpole.registerGoals` panic speed (`1.25`).
const TADPOLE_PANIC_SPEED: f64 = 1.25;
/// Vanilla `Tadpole` swim-to-random-place speed modifier (`2.0`).
const TADPOLE_SWIM_SPEED: f64 = 2.0;
/// Vanilla `Tadpole` swim-to-random-place interval.
const TADPOLE_SWIM_INTERVAL: i32 = 100;
/// Vanilla `Tadpole` age-up duration (`24000` ticks, about 20 minutes).
pub const TADPOLE_GROW_UP_TICKS: i32 = 24000;
/// Vanilla `Tadpole` growth per slime ball (`2400` ticks).
pub const TADPOLE_FEED_AGE_UP_TICKS: i32 = 2400;

#[entity_behavior(class = "Tadpole", spawn_rule)]
/// Vanilla tadpole: a water animal that grows into a frog.
pub struct TadpoleEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    entity_data: SyncMutex<TadpoleEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `TadpoleEntity`.
unsafe impl DowncastType for TadpoleEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/tadpole");
}

impl TadpoleEntity {
    /// Vanilla `SpawnPlacements` predicate: `WaterAnimal.checkSurfaceWaterAnimalSpawnRules`.
    pub(crate) const SPAWN_RULE: SpawnRule = check_water_animal_spawn_rules;

    /// Creates a new tadpole at runtime, already a baby like vanilla.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        let entity = Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        );
        // Vanilla always spawns tadpoles as babies.
        entity.set_baby(true);
        entity
    }

    /// Reconstructs a tadpole from persisted base entity state.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self::new_with_base(
            EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
        )
    }

    fn new_with_base(base: EntityBase, entity_type: EntityTypeRef) -> Self {
        let living_base = LivingEntityBase::new(entity_type);
        living_base
            .attributes()
            .lock()
            .set_base_value(vanilla_attributes::MOVEMENT_SPEED, TADPOLE_MOVEMENT_SPEED);

        let mob_base = MobBase::new();
        // Vanilla `WaterAnimal` gives water path nodes a malus of `0.0`.
        mob_base
            .pathfinding_malus()
            .lock()
            .set(PathType::Water, 0.0);

        let mut entity_data = TadpoleEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            // Vanilla `Tadpole.registerGoals`.
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, PanicGoal::new(TADPOLE_PANIC_SPEED));
            goal_selector.add_goal(
                1,
                RandomSwimmingGoal::new(TADPOLE_SWIM_SPEED, TADPOLE_SWIM_INTERVAL),
            );
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            ageable_base: AgeableMobBase::new(),
            entity_data: SyncMutex::new(entity_data),
        }
    }

    fn handle_air_supply(&self, pre_tick_air_supply: i32) {
        tick_breathing(self, pre_tick_air_supply);
    }

    fn tick_flopping(&self) {
        if !self.is_in_water() && self.on_ground() && self.vertical_collision() {
            flop(self, &sound_events::ENTITY_TADPOLE_FLOP);
        }
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        if let Some(display) = sync_dirty_mob_effects(&self.living_base, &self.entity_data) {
            self.entity_data
                .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
        }
    }
}

/// Whether a stack speeds up a tadpole's growth (vanilla: the `frog_food` tag).
#[must_use]
pub fn is_tadpole_food(item_stack: &ItemStack) -> bool {
    REGISTRY
        .items
        .is_in_tag(item_stack.item(), &ItemTag::FROG_FOOD)
}

impl DefaultEntityPush for TadpoleEntity {}

impl Entity for TadpoleEntity {
    fn as_default_entity_push(&self) -> Option<&dyn DefaultEntityPush> {
        Some(self)
    }

    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn movement_emission(&self) -> EntityMovementEmission {
        EntityMovementEmission::Events
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn base_tick(&self) {
        let air_supply = self.air_supply();
        Mob::base_tick_mob(self);
        self.handle_air_supply(air_supply);
    }

    fn dimensions_for_pose(&self, _pose: EntityPose) -> EntityDimensions {
        let scale = LivingEntity::get_scale(self);
        TADPOLE_DIMENSIONS.scale(scale)
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(&self.entity_data)
    }

    fn update_data_before_sync(&self) {
        self.update_dirty_mob_effect_entity_data();
    }

    fn sound_source(&self) -> SoundSource {
        SoundSource::Neutral
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
        self.save_ageable_mob(nbt);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);
    }
}

impl LivingEntity for TadpoleEntity {
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
        Some(&sound_events::ENTITY_TADPOLE_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_TADPOLE_DEATH)
    }

    fn travel_in_water(
        &self,
        shared: &SharedEntity,
        input: DVec3,
        _base_gravity: f64,
        _is_falling: bool,
        _old_y: f64,
    ) -> Option<MoveResult> {
        let navigation_idle = self.mob_base.navigation().lock().is_done();
        travel_in_water(self, shared, input, navigation_idle)
    }

    fn server_ai_step(&self, entity: &SharedEntity) {
        Mob::mob_server_ai_step(self, entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        let result = Mob::mob_ai_step(self, entity);
        self.tick_flopping();
        AgeableMob::tick_ageable_mob(self);
        result
    }
}

impl AgeableMob for TadpoleEntity {
    fn ageable_base(&self) -> &AgeableMobBase {
        &self.ageable_base
    }

    fn is_age_locked(&self) -> bool {
        *self.entity_data.lock().age_locked.get()
    }

    fn set_age_locked(&self, age_locked: bool) {
        self.entity_data.lock().age_locked.set(age_locked);
    }

    fn set_synced_baby(&self, _baby: bool) {
        // Vanilla `Tadpole` has no synced baby flag; it is always a baby until
        // it converts into a frog.
    }

    fn age_boundary_changed(&self, _baby: bool) {
        self.refresh_dimensions();
    }

    fn mob_interact_ageable(&self, player: &Player, hand: InteractionHand) -> InteractionResult {
        let item_stack = {
            let inventory = player.inventory.lock();
            let item_stack = inventory.get_item_in_hand(hand);
            item_stack.copy_with_count(item_stack.count())
        };
        if !is_tadpole_food(&item_stack) {
            return InteractionResult::Pass;
        }

        // Vanilla `Tadpole.mobInteract` accelerates growth by 2400 ticks.
        let age = self.get_age();
        self.age_up(age - TADPOLE_FEED_AGE_UP_TICKS, false);
        Mob::use_player_item(self, player, hand);
        InteractionResult::SuccessServer
    }
}

impl Mob for TadpoleEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        None
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        spawn_reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        self.finalize_spawn_ageable_mob(world, spawn_reason, group_data)
    }
}

impl PathfinderMob for TadpoleEntity {}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;

    use super::*;

    fn tadpole() -> TadpoleEntity {
        TadpoleEntity::new(&vanilla_entities::TADPOLE, 1, DVec3::ZERO, Weak::new())
    }

    #[test]
    fn tadpoles_spawn_as_babies() {
        init_vanilla_registry();
        assert!(
            AgeableMob::is_baby(&tadpole()),
            "vanilla tadpoles are always babies"
        );
    }

    #[test]
    fn water_animal_constants_match_vanilla() {
        init_vanilla_registry();
        assert_eq!(TADPOLE_GROW_UP_TICKS, 24000);
        assert_eq!(TADPOLE_FEED_AGE_UP_TICKS, 2400);
    }
}
