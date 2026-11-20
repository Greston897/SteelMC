//! Vanilla `Nautilus` and `ZombieNautilus`.
//!
//! Both extend `AbstractNautilus` (dash / swim speeds) on top of
//! `TamableAnimal` + `AgeableMob` + `Animal`. Steel does not model the
//! `TamableAnimal` layer as a trait yet, so the taming flags live directly in
//! the generated `TamableAnimal` part of the entity data and the species only
//! adds the vanilla dash behaviour.

use std::sync::Weak;

use glam::DVec3;
use simdnbt::{borrow, owned};
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_data::EntityPose;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::sound_events;
use steel_registry::vanilla_attributes;
use steel_registry::vanilla_entity_data::{NautilusEntityData, ZombieNautilusEntityData};
use steel_registry::vanilla_game_events;
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{PanicGoal, RandomSwimmingGoal};
use crate::entity::ai::path::PathType;
use crate::entity::damage::DamageSource;
use crate::entity::living_entity::player_jump_pending_scale;
use crate::entity::registry::SpawnRule;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntityDimensions, EntityEventSource,
    EntityMovementEmission, EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase,
    PathfinderMob, SharedEntity,
};
use crate::physics::MoveResult;
use crate::player::Player;
use crate::world::World;
use crate::world::game_event::GameEventContext;

use super::{
    WATER_ANIMAL_AMBIENT_SOUND_INTERVAL, WATER_ANIMAL_EXPERIENCE_MAX, WATER_ANIMAL_EXPERIENCE_MIN,
    check_water_animal_spawn_rules, flop, tick_breathing, travel_in_water,
};

/// Vanilla `AbstractNautilus` base speed (`0.5`).
const NAUTILUS_MOVEMENT_SPEED: f64 = 0.5;
/// Vanilla `AbstractNautilus.DASH_MOMENTUM_IN_WATER`.
pub const NAUTILUS_DASH_MOMENTUM_IN_WATER: f32 = 1.2;
/// Vanilla `AbstractNautilus.DASH_MOMENTUM_ON_LAND`.
pub const NAUTILUS_DASH_MOMENTUM_ON_LAND: f32 = 0.5;
/// Vanilla `AbstractNautilus` dash cooldown length in ticks.
pub const NAUTILUS_DASH_COOLDOWN_TICKS: i32 = 40;
/// Vanilla `AbstractNautilus.tick`: the dash flag clears once the cooldown has
/// counted down below this value, which is what makes a dash last 5 ticks.
pub const NAUTILUS_DASH_END_AT_COOLDOWN: i32 = 35;
/// Vanilla panic speed modifier.
const NAUTILUS_PANIC_SPEED: f64 = 1.25;
/// Vanilla `SwimToRandomPlaceGoal` interval for nautilus.
const NAUTILUS_SWIM_INTERVAL: i32 = 40;

/// Vanilla `AbstractNautilus`: the dash state machine both nautilus species share.
///
/// Vanilla keeps `DASH` in synched entity data and `dashCooldown` as a plain
/// field on the abstract base, with each species only choosing the dash and
/// dash-ready sounds. The trait mirrors that split.
/// Vanilla `AbstractNautilus` extends `AbstractFish`, which is a
/// `LivingEntity`; the methods below read attributes and raise game events, so
/// the bound has to be `LivingEntity`, not just `Entity`.
pub trait AbstractNautilus: LivingEntity + EntityEventSource {
    /// Vanilla `AbstractNautilus.isDashing`.
    fn is_dashing(&self) -> bool;

    /// Vanilla `AbstractNautilus.setDashing`.
    fn set_dashing(&self, dashing: bool);

    /// Vanilla `AbstractNautilus.dashCooldown`, in ticks.
    fn dash_cooldown(&self) -> &SyncMutex<i32>;

    /// Vanilla `AbstractNautilus.getJumpCooldown`.
    fn jump_cooldown(&self) -> i32 {
        *self.dash_cooldown().lock()
    }

    /// Vanilla `AbstractNautilus.tick` dash bookkeeping.
    ///
    /// The dash flag clears as soon as the cooldown has counted down below
    /// `DASH_MINIMUM_DURATION_TICKS` away from the top, the counter keeps
    /// decrementing every tick, and reaching zero plays the dash-ready sound.
    /// The sound is species- and water-dependent, so the caller supplies
    /// exactly what `getDashReadySound` returns.
    fn tick_dash(&self, dash_ready_sound: Option<SoundEventRef>) {
        let (remaining, just_finished) = {
            let mut cooldown = self.dash_cooldown().lock();
            if *cooldown > 0 {
                *cooldown -= 1;
                (*cooldown, *cooldown == 0)
            } else {
                (0, false)
            }
        };
        if just_finished && let Some(sound) = dash_ready_sound {
            self.play_sound(sound, 1.0, 1.0);
        }
        if self.is_dashing() && remaining < NAUTILUS_DASH_END_AT_COOLDOWN {
            self.set_dashing(false);
        }
    }

    /// Vanilla `AbstractNautilus.executeRidersJump`.
    ///
    /// The impulse is the *rider's* look vector scaled by
    /// `(in water ? 1.2 : 0.5) * amount * MOVEMENT_SPEED * blockSpeedFactor`;
    /// then the 40-tick cooldown starts and the dash flag turns on.
    fn execute_nautilus_jump(&self, amount: f32, look: DVec3, block_speed_factor: f64) {
        let momentum = if self.is_in_water() {
            NAUTILUS_DASH_MOMENTUM_IN_WATER
        } else {
            NAUTILUS_DASH_MOMENTUM_ON_LAND
        };
        let movement_speed = self
            .attributes()
            .lock()
            .required_value(vanilla_attributes::MOVEMENT_SPEED);
        let scale = f64::from(momentum * amount) * movement_speed * block_speed_factor;
        self.set_velocity(self.velocity() + look * scale);
        self.mark_velocity_sync();
        *self.dash_cooldown().lock() = NAUTILUS_DASH_COOLDOWN_TICKS;
        self.set_dashing(true);
    }
}

#[entity_behavior(class = "Nautilus", spawn_rule)]
/// Vanilla `Nautilus`: a tameable water animal that dashes away.
pub struct NautilusEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<NautilusEntityData>,
    /// Vanilla `AbstractNautilus.dashCooldown`: ticks until another dash is
    /// allowed. Set to `40` when a dash starts and counted down every tick.
    dash_cooldown: SyncMutex<i32>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `NautilusEntity`.
unsafe impl DowncastType for NautilusEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/nautilus");
}

impl NautilusEntity {
    /// Vanilla `WaterAnimal.canSpawn`.
    pub(crate) const SPAWN_RULE: SpawnRule = check_water_animal_spawn_rules;

    /// Creates a new nautilus.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Recreates a nautilus from saved entity state.
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
            .set_base_value(vanilla_attributes::MOVEMENT_SPEED, NAUTILUS_MOVEMENT_SPEED);

        let mob_base = MobBase::new();
        mob_base
            .pathfinding_malus()
            .lock()
            .set(PathType::Water, 0.0);

        let mut entity_data = NautilusEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, PanicGoal::new(NAUTILUS_PANIC_SPEED));
            goal_selector.add_goal(1, RandomSwimmingGoal::new(1.0, NAUTILUS_SWIM_INTERVAL));
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            entity_data: SyncMutex::new(entity_data),
            dash_cooldown: SyncMutex::new(0),
        }
    }

    /// Vanilla `AbstractNautilus.isDash`.
    #[must_use]
    pub fn is_dash(&self) -> bool {
        *self.entity_data.lock().abstract_nautilus().dash.get()
    }

    /// Vanilla `AbstractNautilus.setDash`.
    pub fn set_dash(&self, dash: bool) {
        self.entity_data
            .lock()
            .abstract_nautilus_mut()
            .dash
            .set(dash);
    }
}

impl AbstractNautilus for NautilusEntity {
    fn is_dashing(&self) -> bool {
        self.is_dash()
    }

    fn set_dashing(&self, dashing: bool) {
        self.set_dash(dashing);
    }

    fn dash_cooldown(&self) -> &SyncMutex<i32> {
        &self.dash_cooldown
    }
}

impl Entity for NautilusEntity {
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
        let air_supply = self.air_supply();
        Mob::base_tick_mob(self);
        tick_breathing(self, air_supply);
        // Vanilla `Nautilus.getDashReadySound`.
        let ready = if self.is_under_water() {
            &sound_events::ENTITY_NAUTILUS_DASH_READY
        } else {
            &sound_events::ENTITY_NAUTILUS_DASH_READY_LAND
        };
        self.tick_dash(Some(ready));
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

    fn save_additional(&self, nbt: &mut owned::NbtCompound) {
        self.save_mob(nbt);
    }

    fn load_additional(&self, nbt: borrow::NbtCompound<'_, '_>) {
        self.load_mob(nbt);
    }
}

impl LivingEntity for NautilusEntity {
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
        if !self.is_in_water() && self.on_ground() && self.vertical_collision() {
            flop(self, &sound_events::ENTITY_NAUTILUS_SWIM);
        }
        result
    }

    fn hurt_sound(&self, _source: &DamageSource) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_NAUTILUS_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_NAUTILUS_DEATH)
    }

    fn base_experience_reward(&self) -> i32 {
        rand::random_range(WATER_ANIMAL_EXPERIENCE_MIN..WATER_ANIMAL_EXPERIENCE_MAX)
    }

    /// Vanilla `AbstractNautilus.canJump`: only a saddled nautilus accepts a
    /// rider's jump charge.
    fn can_jump_with_rider(&self) -> bool {
        self.is_saddled()
    }

    /// Vanilla `AbstractNautilus.onPlayerJump`: a saddled nautilus that is not
    /// already dashing stores the charge for its next ridden tick.
    fn on_player_jump(&self, jump_amount: i32) {
        if !self.is_saddled() || self.jump_cooldown() > 0 {
            return;
        }
        let amount = jump_amount.max(0);
        self.living_base()
            .set_jump_pending_scale(player_jump_pending_scale(amount));
    }

    /// Vanilla `AbstractNautilus.tickRidden`: resolve a stored charge into
    /// `executeRidersJump` against the rider's look vector.
    fn tick_ridden(&self, controller: &Player, _ridden_input: DVec3) {
        let pending = self.jump_pending_scale();
        if pending > 0.0 && !self.is_jumping() {
            let look = controller.look_angle();
            self.execute_nautilus_jump(pending, look, f64::from(self.block_speed_factor()));
        }
        self.reset_jump_pending_scale();
    }

    /// Vanilla `AbstractNautilus.handleStartJump`: dash sound, action game
    /// event, and the dash flag turns on immediately.
    fn handle_start_jump(&self, _jump_scale: i32) {
        if let Some(world) = self.level() {
            let underwater = self.is_under_water();
            let sound = if underwater {
                &sound_events::ENTITY_NAUTILUS_DASH
            } else {
                &sound_events::ENTITY_NAUTILUS_DASH_LAND
            };
            world.play_sound_at(sound, SoundSource::Neutral, self.position(), 1.0, 1.0, None);
            world.game_event_at(
                &vanilla_game_events::ENTITY_ACTION,
                self.position(),
                &GameEventContext::new(Some(self.as_entity_event_source()), None),
            );
        }
        self.set_dash(true);
    }
}

impl Mob for NautilusEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_NAUTILUS_AMBIENT)
    }

    fn ambient_sound_interval(&self) -> i32 {
        WATER_ANIMAL_AMBIENT_SOUND_INTERVAL
    }
}

impl PathfinderMob for NautilusEntity {}

#[entity_behavior(class = "ZombieNautilus")]
/// Vanilla `ZombieNautilus`: the zombified nautilus variant.
pub struct ZombieNautilusEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<ZombieNautilusEntityData>,
    /// Vanilla `AbstractNautilus.dashCooldown` for the zombie species.
    dash_cooldown: SyncMutex<i32>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `ZombieNautilusEntity`.
unsafe impl DowncastType for ZombieNautilusEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/zombie_nautilus");
}

impl ZombieNautilusEntity {
    /// Creates a new zombie nautilus.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Recreates a zombie nautilus from saved entity state.
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
            .set_base_value(vanilla_attributes::MOVEMENT_SPEED, NAUTILUS_MOVEMENT_SPEED);

        let mob_base = MobBase::new();
        mob_base
            .pathfinding_malus()
            .lock()
            .set(PathType::Water, 0.0);

        let mut entity_data = ZombieNautilusEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, PanicGoal::new(NAUTILUS_PANIC_SPEED));
            goal_selector.add_goal(1, RandomSwimmingGoal::new(1.0, NAUTILUS_SWIM_INTERVAL));
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            entity_data: SyncMutex::new(entity_data),
            dash_cooldown: SyncMutex::new(0),
        }
    }

    /// Whether the zombie nautilus is currently dashing.
    #[must_use]
    pub fn is_dash(&self) -> bool {
        *self.entity_data.lock().abstract_nautilus().dash.get()
    }

    /// Sets the dash flag.
    pub fn set_dash(&self, dash: bool) {
        self.entity_data
            .lock()
            .abstract_nautilus_mut()
            .dash
            .set(dash);
    }
}

impl AbstractNautilus for ZombieNautilusEntity {
    fn is_dashing(&self) -> bool {
        self.is_dash()
    }

    fn set_dashing(&self, dashing: bool) {
        self.set_dash(dashing);
    }

    fn dash_cooldown(&self) -> &SyncMutex<i32> {
        &self.dash_cooldown
    }
}

impl Entity for ZombieNautilusEntity {
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
        let air_supply = self.air_supply();
        Mob::base_tick_mob(self);
        tick_breathing(self, air_supply);
        // Vanilla `ZombieNautilus.getDashReadySound`.
        let ready = if self.is_under_water() {
            &sound_events::ENTITY_ZOMBIE_NAUTILUS_DASH_READY
        } else {
            &sound_events::ENTITY_ZOMBIE_NAUTILUS_DASH_READY_LAND
        };
        self.tick_dash(Some(ready));
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

    fn save_additional(&self, nbt: &mut owned::NbtCompound) {
        self.save_mob(nbt);
    }

    fn load_additional(&self, nbt: borrow::NbtCompound<'_, '_>) {
        self.load_mob(nbt);
    }
}

impl LivingEntity for ZombieNautilusEntity {
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
        Mob::mob_ai_step(self, entity)
    }

    fn hurt_sound(&self, _source: &DamageSource) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_ZOMBIE_NAUTILUS_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_ZOMBIE_NAUTILUS_DEATH)
    }

    fn base_experience_reward(&self) -> i32 {
        rand::random_range(WATER_ANIMAL_EXPERIENCE_MIN..WATER_ANIMAL_EXPERIENCE_MAX)
    }

    /// Vanilla `AbstractNautilus.canJump`, inherited by the zombie species.
    fn can_jump_with_rider(&self) -> bool {
        self.is_saddled()
    }

    /// Vanilla `AbstractNautilus.onPlayerJump`, inherited by the zombie species.
    fn on_player_jump(&self, jump_amount: i32) {
        if !self.is_saddled() || self.jump_cooldown() > 0 {
            return;
        }
        let amount = jump_amount.max(0);
        self.living_base()
            .set_jump_pending_scale(player_jump_pending_scale(amount));
    }

    /// Vanilla `AbstractNautilus.tickRidden`, inherited by the zombie species.
    fn tick_ridden(&self, controller: &Player, _ridden_input: DVec3) {
        let pending = self.jump_pending_scale();
        if pending > 0.0 && !self.is_jumping() {
            let look = controller.look_angle();
            self.execute_nautilus_jump(pending, look, f64::from(self.block_speed_factor()));
        }
        self.reset_jump_pending_scale();
    }

    /// Vanilla `ZombieNautilus.getDashSound`: the dash sound is
    /// species-specific, so the zombie plays its own.
    fn handle_start_jump(&self, _jump_scale: i32) {
        if let Some(world) = self.level() {
            let sound = if self.is_under_water() {
                &sound_events::ENTITY_ZOMBIE_NAUTILUS_DASH
            } else {
                &sound_events::ENTITY_ZOMBIE_NAUTILUS_DASH_LAND
            };
            world.play_sound_at(sound, SoundSource::Neutral, self.position(), 1.0, 1.0, None);
            world.game_event_at(
                &vanilla_game_events::ENTITY_ACTION,
                self.position(),
                &GameEventContext::new(Some(self.as_entity_event_source()), None),
            );
        }
        self.set_dash(true);
    }
}

impl Mob for ZombieNautilusEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_ZOMBIE_NAUTILUS_AMBIENT)
    }

    fn ambient_sound_interval(&self) -> i32 {
        WATER_ANIMAL_AMBIENT_SOUND_INTERVAL
    }
}

impl PathfinderMob for ZombieNautilusEntity {}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::*;

    #[test]
    fn new_nautilus_is_not_dashing() {
        init_vanilla_registry();
        let nautilus =
            NautilusEntity::new(&vanilla_entities::NAUTILUS, 1, DVec3::ZERO, Weak::new());
        assert!(!nautilus.is_dash());
        nautilus.set_dash(true);
        assert!(nautilus.is_dash());
    }

    #[test]
    fn an_unsaddled_nautilus_refuses_a_riders_jump() {
        init_vanilla_registry();
        let nautilus =
            NautilusEntity::new(&vanilla_entities::NAUTILUS, 1, DVec3::ZERO, Weak::new());
        // Vanilla `canJump` is `isSaddled()`; without a saddle the charge must
        // never be stored, so the mount cannot be launched by a stray packet.
        assert!(!LivingEntity::can_jump_with_rider(&nautilus));
        nautilus.on_player_jump(100);
        assert_eq!(nautilus.jump_pending_scale(), 0.0);
    }

    #[test]
    fn a_fresh_nautilus_has_no_dash_cooldown() {
        init_vanilla_registry();
        let nautilus =
            NautilusEntity::new(&vanilla_entities::NAUTILUS, 1, DVec3::ZERO, Weak::new());
        assert_eq!(nautilus.jump_cooldown(), 0);
    }

    #[test]
    fn a_riders_jump_starts_the_vanilla_cooldown() {
        init_vanilla_registry();
        let nautilus =
            NautilusEntity::new(&vanilla_entities::NAUTILUS, 1, DVec3::ZERO, Weak::new());
        nautilus.execute_nautilus_jump(1.0, DVec3::X, 1.0);
        assert!(nautilus.is_dash());
        assert_eq!(nautilus.jump_cooldown(), NAUTILUS_DASH_COOLDOWN_TICKS);
    }

    #[test]
    fn the_dash_ends_five_ticks_before_the_cooldown_does() {
        init_vanilla_registry();
        let nautilus =
            NautilusEntity::new(&vanilla_entities::NAUTILUS, 1, DVec3::ZERO, Weak::new());
        nautilus.execute_nautilus_jump(1.0, DVec3::X, 1.0);
        // Vanilla decrements `dashCooldown` first and clears the flag when it
        // drops below 35: ticks 1..=5 keep dashing (40 -> 35), tick 6 drops to
        // 34 and ends the dash.
        for _ in 0..NAUTILUS_DASH_COOLDOWN_TICKS - NAUTILUS_DASH_END_AT_COOLDOWN {
            nautilus.tick_dash(None);
        }
        assert!(
            nautilus.is_dash(),
            "still dashing one tick before the cutoff"
        );
        nautilus.tick_dash(None);
        assert!(
            !nautilus.is_dash(),
            "the dash ends once the cooldown reaches {NAUTILUS_DASH_END_AT_COOLDOWN}"
        );
        assert_eq!(nautilus.jump_cooldown(), NAUTILUS_DASH_END_AT_COOLDOWN - 1);
    }

    #[test]
    fn the_cooldown_must_run_out_before_another_dash() {
        init_vanilla_registry();
        let nautilus =
            NautilusEntity::new(&vanilla_entities::NAUTILUS, 1, DVec3::ZERO, Weak::new());
        nautilus.execute_nautilus_jump(1.0, DVec3::X, 1.0);
        for _ in 0..NAUTILUS_DASH_COOLDOWN_TICKS {
            nautilus.tick_dash(None);
        }
        assert_eq!(nautilus.jump_cooldown(), 0);
    }

    #[test]
    fn new_zombie_nautilus_is_not_dashing() {
        init_vanilla_registry();
        let nautilus = ZombieNautilusEntity::new(
            &vanilla_entities::ZOMBIE_NAUTILUS,
            1,
            DVec3::ZERO,
            Weak::new(),
        );
        assert!(!nautilus.is_dash());
    }

    #[test]
    fn nautilus_movement_speed_matches_vanilla() {
        assert_eq!(NAUTILUS_MOVEMENT_SPEED, 0.5);
        assert_eq!(NAUTILUS_DASH_MOMENTUM_IN_WATER, 1.2);
        assert_eq!(NAUTILUS_DASH_MOMENTUM_ON_LAND, 0.5);
        assert_eq!(NAUTILUS_DASH_COOLDOWN_TICKS, 40);
    }
}
