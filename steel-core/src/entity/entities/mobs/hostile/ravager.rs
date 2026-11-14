//! Vanilla `net.minecraft.world.entity.monster.Ravager`.
//!
//! A large illager beast that charges and knocks targets back.
//!
use std::iter::empty;
use std::sync::Weak;

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::RavagerEntityData;
use steel_registry::{sound_events, vanilla_attributes};
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    FloatGoal, HurtByTargetGoal, LookAtPlayerGoal, MeleeAttackGoal, NearestAttackableTargetGoal,
    RandomLookAroundGoal, WaterAvoidingRandomStrollGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::{
    DefaultEntityPush, Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntityPose,
    EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase, Monster, PathfinderMob,
    SharedEntity, sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::world::World;

const DEFAULT_STEP_HEIGHT: f32 = 0.6;

/// Vanilla `net.minecraft.world.entity.monster.Ravager`.
#[entity_behavior(class = "Ravager")]
pub struct RavagerEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<RavagerEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `RavagerEntity`.
unsafe impl DowncastType for RavagerEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/ravager");
}

impl RavagerEntity {
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

        let mut entity_data = RavagerEntityData::new();
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
        }
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        sync_dirty_mob_effects(&self.living_base, &self.entity_data);
    }
}

impl DefaultEntityPush for RavagerEntity {}

impl Entity for RavagerEntity {
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
        self.play_sound(&sound_events::ENTITY_RAVAGER_STEP, 0.15, 1.0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
    }
}

impl LivingEntity for RavagerEntity {
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
        Some(&sound_events::ENTITY_RAVAGER_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_RAVAGER_DEATH)
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

impl Mob for RavagerEntity {
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
        Some(&sound_events::ENTITY_RAVAGER_AMBIENT)
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl Monster for RavagerEntity {}

impl PathfinderMob for RavagerEntity {}
