//! Vanilla `net.minecraft.world.entity.monster.warden.Warden`.
//!
//! Deep-dark melee horror. It pulses vanilla's darkness effect onto nearby
//! players while it is angry; the shrieker summoning rules are not implemented.
//!
//! The AI is the shared `Monster` skeleton (float, melee attack, random stroll,
//! look goals) plus the standard hostile target selector; attributes and
//! dimensions come from the extracted `entities.json`.
//!
use std::iter::empty;
use std::sync::Weak;

use crate::entity::MobEffectInstance;
use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::WardenEntityData;
use steel_registry::{sound_events, vanilla_attributes, vanilla_mob_effects};
use steel_utils::WorldAabb;
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey};
use steel_registry::vanilla_damage_types::SONIC_BOOM;

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

/// Vanilla `net.minecraft.world.entity.monster.Silverfish`.
#[entity_behavior(class = "Warden")]
pub struct WardenEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<WardenEntityData>,
    /// Ticks since the last darkness pulse, matching vanilla
    /// `Warden.tendrilAnimation` bookkeeping.
    darkness_pulse_ticks: SyncMutex<i32>,
    /// Remaining cooldown ticks before vanilla `SonicBoom` (100t cooldown) can fire again.
    sonic_boom_cooldown: SyncMutex<i32>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `WardenEntity`.
unsafe impl DowncastType for WardenEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/warden");
}

impl WardenEntity {
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

        let mut entity_data = WardenEntityData::new();
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
            darkness_pulse_ticks: SyncMutex::new(0),
            sonic_boom_cooldown: SyncMutex::new(40),
        }
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        sync_dirty_mob_effects(&self.living_base, &self.entity_data);
    }
}

impl DefaultEntityPush for WardenEntity {}

impl Entity for WardenEntity {
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
        self.play_sound(&sound_events::ENTITY_WARDEN_STEP, 0.15, 1.0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
    }
}

impl LivingEntity for WardenEntity {
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
        Some(&sound_events::ENTITY_WARDEN_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_WARDEN_DEATH)
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

/// Vanilla `Warden.getDarknessPulseInterval`: ticks between darkness pulses.
const WARDEN_DARKNESS_INTERVAL: i32 = 6;

/// Vanilla `Warden` darkness radius in blocks.
const WARDEN_DARKNESS_RADIUS: f64 = 20.0;

/// Vanilla darkness effect length per pulse, in ticks.
const WARDEN_DARKNESS_DURATION: i32 = 260;

/// Vanilla `SonicBoom` ranged attack geometry, damage, and cooldown (100 ticks).
const SONIC_BOOM_COOLDOWN_TICKS: i32 = 100;
const SONIC_BOOM_HORIZONTAL_RANGE: f64 = 15.0;
const SONIC_BOOM_VERTICAL_RANGE: f64 = 20.0;
const SONIC_BOOM_MIN_RANGE_SQ: f64 = 16.0;
const SONIC_BOOM_DAMAGE: f32 = 10.0;

impl Mob for WardenEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    /// Vanilla `Warden` pulses `MobEffects.DARKNESS` onto every player within
    /// twenty blocks while hunting and fires its armor-bypassing `SonicBoom`
    /// beam at out-of-melee-reach targets within 15 horizontal / 20 vertical blocks.
    fn custom_server_ai_step(&self) {
        let Some(world) = self.level() else {
            return;
        };

        // Tick & fire vanilla `SonicBoom` against the current target.
        {
            let mut cd = self.sonic_boom_cooldown.lock();
            if *cd > 0 {
                *cd -= 1;
            } else if let Some(target) = self.target()
                && target.is_alive()
            {
                let origin = self.position();
                let target_pos = target.position();
                let dx = target_pos.x - origin.x;
                let dy = target_pos.y - origin.y;
                let dz = target_pos.z - origin.z;
                let horiz_sq = dx * dx + dz * dz;
                let dist_sq = horiz_sq + dy * dy;
                if dist_sq >= SONIC_BOOM_MIN_RANGE_SQ
                    && horiz_sq <= SONIC_BOOM_HORIZONTAL_RANGE * SONIC_BOOM_HORIZONTAL_RANGE
                    && dy.abs() <= SONIC_BOOM_VERTICAL_RANGE
                {
                    *cd = SONIC_BOOM_COOLDOWN_TICKS;
                    drop(cd);
                    self.play_sound(&sound_events::ENTITY_WARDEN_SONIC_BOOM, 3.0, 1.0);
                    let source = DamageSource::environment(
                        &SONIC_BOOM,
                    );
                    if target.hurt(&world, &source, SONIC_BOOM_DAMAGE) {
                        let kb_resist = target.as_living_entity().map_or(0.0, |l| {
                            l.attributes()
                                .lock()
                                .get_value(vanilla_attributes::KNOCKBACK_RESISTANCE)
                                .unwrap_or(0.0)
                        });
                        let v_scale = 0.5 * (1.0 - kb_resist);
                        let h_scale = 2.5 * (1.0 - kb_resist);
                        let dir = DVec3::new(dx, dy, dz).normalize_or_zero();
                        let vel = target.velocity();
                        target.set_velocity(DVec3::new(
                            vel.x + dir.x * h_scale,
                            vel.y + dir.y * v_scale,
                            vel.z + dir.z * h_scale,
                        ));
                    }
                }
            }
        }

        let mut ticks = self.darkness_pulse_ticks.lock();
        *ticks += 1;
        if *ticks < WARDEN_DARKNESS_INTERVAL {
            return;
        }
        *ticks = 0;
        drop(ticks);

        if self.target().is_none() {
            return;
        }

        let position = self.position();
        let reach = WorldAabb::new(
            position.x - WARDEN_DARKNESS_RADIUS,
            position.y - WARDEN_DARKNESS_RADIUS,
            position.z - WARDEN_DARKNESS_RADIUS,
            position.x + WARDEN_DARKNESS_RADIUS,
            position.y + WARDEN_DARKNESS_RADIUS,
            position.z + WARDEN_DARKNESS_RADIUS,
        );
        for entity in world.get_entities_in_aabb(&reach) {
            let Some(player) = world.players.get_by_entity_id(entity.id()) else {
                continue;
            };
            player.add_mob_effect(MobEffectInstance::with_duration(
                vanilla_mob_effects::DARKNESS,
                WARDEN_DARKNESS_DURATION,
                0,
            ));
        }
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_WARDEN_AMBIENT)
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl Monster for WardenEntity {}

impl PathfinderMob for WardenEntity {}
