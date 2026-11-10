//! Vanilla `net.minecraft.world.entity.monster.skeleton.Stray`.
//!
//! An icy `Skeleton` variant: identical ranged behaviour, but every arrow
//! carries a `slowness` effect for `600` ticks (`30` seconds), matching vanilla
//! `Stray#getArrow`.
//!
//! Goal layout mirrors vanilla `AbstractSkeleton`: sunlight avoidance
//! (`RestrictSunGoal`, `FleeSunGoal`), wolf avoidance, the vanilla
//! `RangedBowAttackGoal` equivalent (`RangedAttackGoal`, 20-tick interval,
//! 15-block radius), random strolling and look goals, plus the standard hostile
//! target selectors (players and iron golems). Daylight burning is driven by
//! the `minecraft:burn_in_daylight` entity type tag, not by this file.
//!
//! Deviations (deliberate, documented in `PARITY-26.3-RU.md`):
//! * natural spawning is not wired yet: `SpawnRule` hands a `&dyn LevelReader`,
//!   while `Monster::check_monster_spawn_rules` needs `&World`, so the entity
//!   type is registered without a spawn rule and only spawners/commands create it.

use std::iter::empty;
use std::sync::Weak;

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::StrayEntityData;
use steel_registry::{sound_events, vanilla_attributes, vanilla_entities, vanilla_mob_effects};
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    AvoidEntityGoal, FleeSunGoal, FloatGoal, HurtByTargetGoal, LookAtPlayerGoal,
    NearestAttackableTargetGoal, RandomLookAroundGoal, RangedAttackGoal, RestrictSunGoal,
    WaterAvoidingRandomStrollGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::entities::AbstractArrow;
use crate::entity::ranged_attack;
use crate::entity::{
    DefaultEntityPush, Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntityPose,
    EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase, MobEffectInstance, Monster,
    PathfinderMob, SharedEntity, sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::world::World;

const DEFAULT_STEP_HEIGHT: f32 = 0.6;
/// Vanilla `RangedBowAttackGoal` speed modifier (`1.0`).
const RANGED_ATTACK_SPEED_MODIFIER: f64 = 1.0;
/// Vanilla `RangedBowAttackGoal` attack interval (`20` ticks).
const RANGED_ATTACK_INTERVAL_TICKS: i32 = 20;
/// Vanilla `RangedBowAttackGoal` attack radius (`15` blocks).
const RANGED_ATTACK_RADIUS: f32 = 15.0;
/// Vanilla `AvoidEntityGoal` maximum wolf distance.
const WOLF_AVOID_DISTANCE: f32 = 6.0;
/// Vanilla `AvoidEntityGoal` walk speed modifier.
const WOLF_FLEE_WALK_SPEED: f64 = 1.0;
/// Vanilla `AvoidEntityGoal` sprint speed modifier.
const WOLF_FLEE_SPRINT_SPEED: f64 = 1.2;
/// Vanilla `LookAtPlayerGoal` search distance.
const LOOK_AT_PLAYER_DISTANCE: f32 = 8.0;

/// Vanilla `net.minecraft.world.entity.monster.skeleton.Stray`.
#[entity_behavior(class = "Stray")]
pub struct StrayEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<StrayEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `StrayEntity`.
unsafe impl DowncastType for StrayEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/stray");
}

impl StrayEntity {
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

        let mut entity_data = StrayEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(2, RestrictSunGoal::new());
            goal_selector.add_goal(3, FleeSunGoal::new(1.0));
            goal_selector.add_goal(
                3,
                AvoidEntityGoal::with_selector(
                    WOLF_AVOID_DISTANCE,
                    WOLF_FLEE_WALK_SPEED,
                    WOLF_FLEE_SPRINT_SPEED,
                    |entity, _| entity.entity_type() == &vanilla_entities::WOLF,
                ),
            );
            goal_selector.add_goal(
                4,
                RangedAttackGoal::new(
                    RANGED_ATTACK_SPEED_MODIFIER,
                    RANGED_ATTACK_INTERVAL_TICKS,
                    RANGED_ATTACK_RADIUS,
                ),
            );
            goal_selector.add_goal(5, WaterAvoidingRandomStrollGoal::new(1.0));
            goal_selector.add_goal(6, LookAtPlayerGoal::new(LOOK_AT_PLAYER_DISTANCE));
            goal_selector.add_goal(6, RandomLookAroundGoal::new());

            let mut target_selector = mob_base.target_selector().lock();
            target_selector.add_goal(1, HurtByTargetGoal::new().set_alert_others(empty()));
            target_selector.add_goal(
                2,
                NearestAttackableTargetGoal::new_for_players(true, |_, _| true),
            );
            target_selector.add_goal(
                3,
                NearestAttackableTargetGoal::new(true, |entity, _| {
                    entity.entity_type() == &vanilla_entities::IRON_GOLEM
                }),
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

impl DefaultEntityPush for StrayEntity {}

impl Entity for StrayEntity {
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
        self.play_sound(&sound_events::ENTITY_STRAY_STEP, 0.15, 1.0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
    }
}

impl LivingEntity for StrayEntity {
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
        Some(&sound_events::ENTITY_STRAY_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_STRAY_DEATH)
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

impl Mob for StrayEntity {
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
        Some(&sound_events::ENTITY_STRAY_AMBIENT)
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl Monster for StrayEntity {}

impl PathfinderMob for StrayEntity {
    fn perform_ranged_attack(&self, self_entity: &SharedEntity, target: &SharedEntity, _power: f32) {
        let Some(arrow) =
            ranged_attack::shoot_arrow(self_entity, target, ranged_attack::MOB_ARROW_DAMAGE)
        else {
            return;
        };
        arrow.add_effect(MobEffectInstance::with_duration(
            vanilla_mob_effects::SLOWNESS,
            600,
            0,
        ));
        // Vanilla `AbstractSkeleton#performRangedAttack` plays the shared
        // skeleton shoot sound with a randomized pitch for every variant.
        self.play_sound(
            &sound_events::ENTITY_SKELETON_SHOOT,
            1.0,
            1.0 / (rand::random::<f32>() * 0.4 + 0.8),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::Weak;

    use glam::DVec3;
    use simdnbt::borrow::read_compound;
    use simdnbt::owned::NbtCompound;
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::StrayEntity;
    use crate::entity::{Entity, LivingEntity};

    #[test]
    fn stray_uses_registry_attributes_and_round_trips_nbt() {
        init_vanilla_registry();
        let mob = StrayEntity::new(&vanilla_entities::STRAY, 1, DVec3::ZERO, Weak::new());
        assert!((mob.get_max_health() - 20.0).abs() < 1e-4);

        let mut nbt = NbtCompound::new();
        mob.save_additional(&mut nbt);

        let loaded = StrayEntity::new(&vanilla_entities::STRAY, 2, DVec3::ZERO, Weak::new());
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let borrowed = read_compound(&mut Cursor::new(&bytes))
            .unwrap_or_else(|error| panic!("reborrow failed: {error}"));
        loaded.load_additional((&borrowed).into());
        assert!(loaded.get_health() > 0.0);
    }
}
