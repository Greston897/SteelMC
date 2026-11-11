//! Vanilla polar bear entity.
//!
//! Polar bears are neutral: an adult stands up and turns hostile when a player
//! comes close with a baby nearby, and the standing flag drives both the pose and
//! the reach of its attack.

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
use steel_registry::sound_events;
use steel_registry::vanilla_entity_data::PolarBearEntityData;
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    FloatGoal, FollowParentGoal, LookAtPlayerGoal, MeleeAttackGoal, PanicGoal,
    RandomLookAroundGoal, WaterAvoidingRandomStrollGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::{
    AgeableMob, AgeableMobBase, Animal, AnimalBase, DefaultEntityPush, Entity, EntityBase,
    EntityBaseLoad, EntityPose, EntitySpawnReason, EntitySyncedData, LivingEntity,
    LivingEntityBase, Mob, MobBase, PathfinderMob, SharedEntity, SpawnGroupData, SpawnRule,
    sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::world::World;

const POLAR_BEAR_BABY_PASSENGER_ATTACHMENTS: [EntityAttachmentPoint; 1] =
    [EntityAttachmentPoint::new(0.0, 0.5, 0.0)];
const POLAR_BEAR_BABY_WIDTH: f32 = 0.7;
const POLAR_BEAR_BABY_HEIGHT: f32 = 0.7;
const POLAR_BEAR_BABY_EYE_HEIGHT: f32 = 0.455;

const POLAR_BEAR_BABY_DIMENSIONS: EntityDimensions = EntityDimensions::new_with_attachments(
    POLAR_BEAR_BABY_WIDTH,
    POLAR_BEAR_BABY_HEIGHT,
    POLAR_BEAR_BABY_EYE_HEIGHT,
    EntityAttachments::new(&POLAR_BEAR_BABY_PASSENGER_ATTACHMENTS, &[], &[], &[]),
);

const POLAR_BEAR_STROLL_SPEED: f64 = 0.5;
const POLAR_BEAR_PANIC_SPEED: f64 = 2.0;
const POLAR_BEAR_MELEE_SPEED: f64 = 1.25;
const POLAR_BEAR_FOLLOW_PARENT_SPEED: f64 = 1.25;
const POLAR_BEAR_LOOK_DISTANCE: f32 = 8.0;

#[entity_behavior(class = "PolarBear", spawn_rule)]
/// Vanilla polar bear with the synced standing flag.
pub struct PolarBearEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    animal_base: AnimalBase,
    entity_data: SyncMutex<PolarBearEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `PolarBearEntity`.
unsafe impl DowncastType for PolarBearEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/polar_bear");
}

impl PolarBearEntity {
    /// Vanilla `SpawnPlacements` predicate: `Animal.checkAnimalSpawnRules`.
    pub(crate) const SPAWN_RULE: SpawnRule = |entity_type, level, spawn_reason, pos, random| {
        <Self as Animal>::check_animal_spawn_rules(entity_type, level, spawn_reason, pos, random)
    };

    /// Creates a new polar bear at runtime.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Reconstructs a polar bear from persisted base entity state.
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
        let ageable_base = AgeableMobBase::new();
        let animal_base = AnimalBase::new();
        AnimalBase::initialize_pathfinding_malus(&mob_base);
        let mut entity_data = PolarBearEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            // Vanilla `PolarBear.registerGoals`.
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(1, MeleeAttackGoal::new(POLAR_BEAR_MELEE_SPEED, true));
            goal_selector.add_goal(2, PanicGoal::new(POLAR_BEAR_PANIC_SPEED));
            goal_selector.add_goal(4, FollowParentGoal::new(POLAR_BEAR_FOLLOW_PARENT_SPEED));
            goal_selector.add_goal(
                5,
                WaterAvoidingRandomStrollGoal::new(POLAR_BEAR_STROLL_SPEED),
            );
            goal_selector.add_goal(
                6,
                LookAtPlayerGoal::new(f64::from(POLAR_BEAR_LOOK_DISTANCE) / 2.0),
            );
            goal_selector.add_goal(7, RandomLookAroundGoal::new());
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            ageable_base,
            animal_base,
            entity_data: SyncMutex::new(entity_data),
        }
    }

    /// Whether the bear is standing on its hind legs.
    #[must_use]
    pub fn is_standing(&self) -> bool {
        *self.entity_data.lock().standing.get()
    }

    /// Sets the standing flag.
    pub fn set_standing(&self, standing: bool) {
        self.entity_data.lock().standing.set(standing);
    }

    /// Vanilla flips the flag when a player is close enough to threaten a cub.
    ///
    /// The check runs every tick on the server: an adult only stands while a
    /// player is within eight blocks and a baby is nearby.
    pub fn update_standing(&self, player_near_cub: bool) {
        let should_stand = !AgeableMob::is_baby(self) && player_near_cub;
        if should_stand != self.is_standing() {
            self.set_standing(should_stand);
        }
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        if let Some(display) = sync_dirty_mob_effects(&self.living_base, &self.entity_data) {
            self.entity_data
                .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
        }
    }
}

impl DefaultEntityPush for PolarBearEntity {}

impl Entity for PolarBearEntity {
    fn as_default_entity_push(&self) -> Option<&dyn DefaultEntityPush> {
        Some(self)
    }

    fn is_hard_collision_relevant(&self) -> bool {
        false
    }

    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn base_tick(&self) {
        Mob::base_tick_mob(self);
    }

    fn dimensions_for_pose(&self, _pose: EntityPose) -> EntityDimensions {
        let scale = LivingEntity::get_scale(self);
        if AgeableMob::is_baby(self) {
            POLAR_BEAR_BABY_DIMENSIONS.scale(scale)
        } else if self.entity_type.fixed {
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

    fn sound_source(&self) -> SoundSource {
        SoundSource::Neutral
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
        self.save_ageable_mob(nbt);
        self.save_animal(nbt);
        nbt.insert("Standing", self.is_standing());
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);
        self.load_animal(nbt);

        if let Some(standing) = nbt.byte("Standing") {
            self.set_standing(standing != 0);
        }
    }
}

impl LivingEntity for PolarBearEntity {
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

    fn sound_volume(&self) -> f32 {
        0.4
    }

    fn hurt_sound(&self, _source: &DamageSource) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_POLAR_BEAR_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_POLAR_BEAR_DEATH)
    }

    fn server_ai_step(&self, entity: &SharedEntity) {
        Mob::mob_server_ai_step(self, entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        let result = Mob::mob_ai_step(self, entity);

        AgeableMob::tick_ageable_mob(self);
        Animal::tick_animal_love(self);
        result
    }
}

impl AgeableMob for PolarBearEntity {
    fn ageable_base(&self) -> &AgeableMobBase {
        &self.ageable_base
    }

    fn is_age_locked(&self) -> bool {
        *self.entity_data.lock().ageable_mob().age_locked.get()
    }

    fn set_age_locked(&self, age_locked: bool) {
        self.entity_data
            .lock()
            .ageable_mob_mut()
            .age_locked
            .set(age_locked);
    }

    fn set_synced_baby(&self, baby: bool) {
        self.entity_data.lock().ageable_mob_mut().baby.set(baby);
    }

    fn age_boundary_changed(&self, _baby: bool) {
        self.refresh_dimensions();
    }
}

impl Mob for PolarBearEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn custom_server_ai_step(&self) {
        Animal::custom_server_ai_step_animal(self);
    }

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_POLAR_BEAR_AMBIENT)
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

impl PathfinderMob for PolarBearEntity {}

impl Animal for PolarBearEntity {
    fn animal_base(&self) -> &AnimalBase {
        &self.animal_base
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        // Vanilla polar bears do not eat: `isFood` returns false for every item.
        let _ = item_stack;
        false
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;

    use super::*;

    fn bear() -> PolarBearEntity {
        PolarBearEntity::new(&vanilla_entities::POLAR_BEAR, 1, DVec3::ZERO, Weak::new())
    }

    #[test]
    fn an_adult_stands_when_a_player_is_near_a_cub() {
        init_vanilla_registry();
        let bear = bear();
        assert!(!bear.is_standing());
        bear.update_standing(true);
        assert!(bear.is_standing(), "an adult stands up to a threat");
        bear.update_standing(false);
        assert!(!bear.is_standing(), "and sits back down once it is gone");
    }

    #[test]
    fn a_cub_never_stands() {
        init_vanilla_registry();
        let bear = bear();
        bear.ageable_base().set_age(-1);
        bear.update_standing(true);
        assert!(!bear.is_standing(), "cubs stay on four legs");
    }

    #[test]
    fn polar_bears_eat_nothing() {
        init_vanilla_registry();
        let bear = bear();
        assert!(!bear.is_food(&ItemStack::new(&steel_registry::vanilla_items::SALMON)));
    }
}
