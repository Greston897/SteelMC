//! Vanilla glow squid.
//!
//! Glow squids are the underground variant of `Squid`: they keep the same
//! movement, ink-squirting and air handling, but play the `glow_squid` sound
//! set and track `dark_ticks_remaining` so they start glowing after spending
//! 100 ticks in the dark.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::{borrow, owned};
use steel_macros::entity_behavior;
use steel_math::vector;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::{
    entity_data::{EntityPose, ParticleData},
    entity_type::{EntityAttachments, EntityDimensions, EntityTypeRef},
    sound_event::SoundEventRef,
    sound_events, vanilla_attributes, vanilla_damage_types,
    vanilla_entity_data::GlowSquidEntityData,
    vanilla_mob_effects::LEVITATION,
    vanilla_particle_types,
};
use steel_utils::{
    DowncastType, DowncastTypeKey, entity_events, locks::SyncMutex, types::InteractionHand,
};

use crate::{
    behavior::InteractionResult,
    entity::{
        AgeableMob, AgeableMobBase, Entity, EntityBase, EntityBaseLoad, EntitySyncedData,
        LivingEntity, LivingEntityBase, Mob, MobBase, PathfinderMob, SharedEntity,
        ai::goal::SquidRandomMovementGoal,
    },
    physics::{MoveResult, MoverType},
    world::World,
};
use crate::{
    entity::{
        EntityMovementEmission, EntitySpawnReason, SpawnGroupData,
        ai::{goal::SquidFleeGoal, path::PathType},
        damage::DamageSource,
    },
    player::Player,
};

const GLOW_SQUID_DEFAULT_HEALTH: f64 = 10.0;
const GLOW_SQUID_BABY_WIDTH: f32 = 0.5;
const GLOW_SQUID_BABY_HEIGHT: f32 = 0.5;
const GLOW_SQUID_BABY_EYE_HEIGHT: f32 = 0.37;
const GLOW_SQUID_BABY_PASSENGER_ATTACHMENTS: [steel_registry::entity_type::EntityAttachmentPoint;
    1] = [steel_registry::entity_type::EntityAttachmentPoint::new(
    0.0, 0.25, 0.0,
)];

const GLOW_SQUID_BABY_DIMENSIONS: EntityDimensions = EntityDimensions::new_with_attachments(
    GLOW_SQUID_BABY_WIDTH,
    GLOW_SQUID_BABY_HEIGHT,
    GLOW_SQUID_BABY_EYE_HEIGHT,
    EntityAttachments::new(&GLOW_SQUID_BABY_PASSENGER_ATTACHMENTS, &[], &[], &[]),
);

const GLOW_SQUID_GRAVITY: f64 = 0.08;
const GLOW_SQUID_SOUND_VOLUME: f32 = 0.4;
const GLOW_SQUID_INK_PARTICLE_COUNT: u32 = 30;
const GLOW_SQUID_ADULT_POSITION_SCALE: f32 = 0.3;
const GLOW_SQUID_BABY_POSITION_SCALE: f32 = 0.1;
const GLOW_SQUID_AMBIENT_SOUND_INTERVAL: i32 = 120;
/// Vanilla `GlowSquid.DARK_TICKS_REMAINING_DEFAULT`.
const GLOW_SQUID_DARK_TICKS_DEFAULT: i32 = 100;
/// Vanilla `GlowSquid` dark threshold: glow once the counter drops below it.
const GLOW_SQUID_DARK_THRESHOLD: i32 = 0;
const GLOW_SQUID_SQUIRT_SOUND: SoundEventRef = &sound_events::ENTITY_GLOW_SQUID_SQUIRT;

#[entity_behavior(class = "GlowSquid", spawn_rule = false)]
/// Vanilla glow squid with the dark-ticks glow counter.
pub struct GlowSquidEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    entity_data: SyncMutex<GlowSquidEntityData>,

    state: SyncMutex<GlowSquidState>,
}

struct GlowSquidState {
    x_body_rot: f32,
    // Required for inking on fleeing.
    x_body_rot_old: f32,
    movement_vector: DVec3,
    tentacle_speed: f32,
    tentacle_movement: f32,
}

// SAFETY: This key is owned by Steel and uniquely identifies `GlowSquidEntity`.
unsafe impl DowncastType for GlowSquidEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/glow_squid");
}

impl GlowSquidEntity {
    /// Creates a new glow squid.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Recreates a glow squid from saved entity state.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self::new_with_base(
            EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
        )
    }

    /// Ticks remaining until this glow squid starts glowing.
    #[must_use]
    pub fn dark_ticks_remaining(&self) -> i32 {
        *self.entity_data.lock().dark_ticks_remaining.get()
    }

    /// Sets the ticks remaining until this glow squid starts glowing.
    pub fn set_dark_ticks_remaining(&self, ticks: i32) {
        self.entity_data.lock().dark_ticks_remaining.set(ticks);
    }

    /// Whether the squid is currently glowing (vanilla `isInvisible` override).
    #[must_use]
    pub fn is_glowing_in_the_dark(&self) -> bool {
        self.dark_ticks_remaining() <= GLOW_SQUID_DARK_THRESHOLD
    }

    /// Vanilla `GlowSquid` spawn rule: any dark water block below sea level.
    pub(crate) const SPAWN_RULE: crate::entity::SpawnRule =
        |_entity_type, level, _spawn_reason, pos, _random| {
            let below = pos.below();
            level.get_block_state(below).get_fluid_state().is_water()
                && level.get_block_state(pos.above()).is_air()
                && pos.y() < level.sea_level()
        };

    fn tick_dark_ticks(&self) {
        if self.is_glowing_in_the_dark() {
            return;
        }
        // Vanilla decrements while the block above is opaque.
        let Some(world) = self.level() else {
            return;
        };
        let pos = steel_utils::BlockPos::from(self.position());
        let block_above = world.get_block_state(pos.above());
        if !block_above.is_air() {
            let remaining = self.dark_ticks_remaining();
            self.set_dark_ticks_remaining((remaining - 1).max(GLOW_SQUID_DARK_THRESHOLD));
        }
    }

    /// Vanilla `Squid.rotateVector`, squid-local space into world space.
    fn rotate_vector(&self, vector: DVec3) -> DVec3 {
        let x_rot = self.state.lock().x_body_rot_old.to_radians();
        let y_rot = -self.living_rotation_state().y_body_rot_o().to_radians();

        vector::y_rot(vector::x_rot(vector, x_rot), y_rot)
    }

    fn spawn_ink(&self) {
        let Some(world) = self.level() else {
            return;
        };

        self.make_sound(Some(GLOW_SQUID_SQUIRT_SOUND));

        let position = self.position() + self.rotate_vector(DVec3::new(0.0, -1.0, 0.0));
        let particle_position = position + DVec3::new(0.0, 0.5, 0.0);
        let particle = ParticleData::simple(&vanilla_particle_types::GLOW_SQUID_INK);
        let position_scale = if AgeableMob::is_baby(self) {
            GLOW_SQUID_BABY_POSITION_SCALE
        } else {
            GLOW_SQUID_ADULT_POSITION_SCALE
        };

        for _ in 0..GLOW_SQUID_INK_PARTICLE_COUNT {
            let direction = self.rotate_vector(DVec3::new(
                rand::random_range(-0.3..0.3),
                -1.0,
                rand::random_range(-0.3..0.3),
            ));
            let offset =
                direction * f64::from(position_scale + rand::random::<f32>() * 2.0_f32);
            world.send_particles(particle.clone(), particle_position, 0, offset, 0.1);
        }
    }

    fn new_with_base(base: EntityBase, entity_type: EntityTypeRef) -> Self {
        let living_base = LivingEntityBase::new(entity_type);
        living_base
            .attributes()
            .lock()
            .set_base_value(vanilla_attributes::MAX_HEALTH, GLOW_SQUID_DEFAULT_HEALTH);

        let mob_base = MobBase::new();
        mob_base
            .pathfinding_malus()
            .lock()
            .set(PathType::Water, 0.0);

        let ageable_base = AgeableMobBase::new();

        let mut entity_data = GlowSquidEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, SquidRandomMovementGoal::new());
            goal_selector.add_goal(1, SquidFleeGoal::new());
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            ageable_base,
            entity_data: SyncMutex::new(entity_data),
            state: SyncMutex::new(GlowSquidState {
                x_body_rot: 0.0,
                x_body_rot_old: 0.0,
                movement_vector: DVec3::ZERO,
                tentacle_speed: 1.0 / (rand::random::<f32>() + 1.0) * 0.2,
                tentacle_movement: 0.0,
            }),
        }
    }
}

impl Entity for GlowSquidEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn hurt(&self, world: &World, source: &DamageSource, amount: f32) -> bool {
        let hurt = LivingEntity::hurt_server(self, world, source, amount);

        if hurt && self.last_hurt_by_mob().is_some() {
            self.spawn_ink();
            return true;
        }

        false
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
        if LivingEntity::is_alive(self) && !self.is_in_water() {
            self.set_air_supply(air_supply - 1);
            if self.should_take_drowning_damage() {
                self.set_air_supply(0);
                if let Some(world) = self.level() {
                    self.hurt(
                        &world,
                        &DamageSource::environment(&vanilla_damage_types::DROWN),
                        2.0,
                    );
                }
            }
        } else {
            self.set_air_supply(300);
            // Vanilla resets the glow counter while submerged.
            self.set_dark_ticks_remaining(GLOW_SQUID_DARK_TICKS_DEFAULT);
        }
    }

    fn get_default_gravity(&self) -> f64 {
        GLOW_SQUID_GRAVITY
    }

    fn dimensions_for_pose(&self, _pose: EntityPose) -> EntityDimensions {
        let scale = LivingEntity::get_scale(self);
        if AgeableMob::is_baby(self) {
            GLOW_SQUID_BABY_DIMENSIONS.scale(scale)
        } else {
            self.entity_type.dimensions.scale(scale)
        }
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(&self.entity_data)
    }

    fn save_additional(&self, nbt: &mut owned::NbtCompound) {
        self.save_mob(nbt);
        self.save_ageable_mob(nbt);
    }

    fn load_additional(&self, nbt: borrow::NbtCompound<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);
    }
}

impl LivingEntity for GlowSquidEntity {
    fn living_base(&self) -> &LivingEntityBase {
        &self.living_base
    }

    fn sound_volume(&self) -> f32 {
        GLOW_SQUID_SOUND_VOLUME
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

    fn server_ai_step(&self, entity: &SharedEntity) {
        Mob::mob_server_ai_step(self, entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        let movement = self.velocity().length_squared();
        if let Some(world) = self.level()
            && movement > 0.0
            && rand::random::<f32>() < (25.0 * movement) as f32
        {
            let particle = ParticleData::simple(&vanilla_particle_types::GLOW_SQUID_INK);
            world.send_particles(particle, self.position(), 0, DVec3::ZERO, 0.0);
        }
        self.tick_dark_ticks();
        Mob::mob_ai_step(self, entity)
    }

    fn hurt_sound(&self, _source: &DamageSource) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_GLOW_SQUID_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_GLOW_SQUID_DEATH)
    }

    fn base_experience_reward(&self) -> i32 {
        1 + rand::random_range(0..3)
    }
}

impl AgeableMob for GlowSquidEntity {
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

impl Mob for GlowSquidEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn mob_interact(&self, player: &Player, hand: InteractionHand) -> InteractionResult {
        AgeableMob::mob_interact_ageable(self, player, hand)
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
        Some(&sound_events::ENTITY_GLOW_SQUID_AMBIENT)
    }

    fn ambient_sound_interval(&self) -> i32 {
        GLOW_SQUID_AMBIENT_SOUND_INTERVAL
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

impl PathfinderMob for GlowSquidEntity {}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;

    use super::*;

    fn glow_squid() -> GlowSquidEntity {
        GlowSquidEntity::new(&vanilla_entities::GLOW_SQUID, 1, DVec3::ZERO, Weak::new())
    }

    #[test]
    fn new_glow_squids_start_with_the_dark_counter() {
        init_vanilla_registry();
        let squid = glow_squid();
        assert_eq!(squid.dark_ticks_remaining(), GLOW_SQUID_DARK_TICKS_DEFAULT);
        assert!(
            !squid.is_glowing_in_the_dark(),
            "freshly spawned glow squids still have dark ticks left"
        );
    }

    #[test]
    fn reaching_zero_dark_ticks_starts_the_glow() {
        init_vanilla_registry();
        let squid = glow_squid();
        squid.set_dark_ticks_remaining(0);
        assert!(squid.is_glowing_in_the_dark());
    }

    #[test]
    fn glow_squids_use_the_glow_squid_sounds() {
        init_vanilla_registry();
        let squid = glow_squid();
        assert_eq!(
            squid.ambient_sound().map(|sound| sound.key.clone()),
            Some(sound_events::ENTITY_GLOW_SQUID_AMBIENT.key.clone())
        );
    }
}
