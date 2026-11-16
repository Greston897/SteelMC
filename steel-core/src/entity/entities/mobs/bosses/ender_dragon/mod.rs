//! Vanilla `net.minecraft.world.entity.boss.enderdragon.EnderDragon`.
//!
//! The end boss. Vanilla drives it with an eleven-state phase machine
//! ([`phases`]) plus its own flight code: the dragon never walks, it steers toward
//! a phase-supplied fly target by blending its yaw toward the target, moving along
//! its facing, and damping its velocity. It carries eight hitbox [`part`]s, heals
//! from nearby end crystals, and dies on a scripted 200-tick descent instead of the
//! usual twenty-tick death animation.
//!
//! This module owns the entity and the vanilla code around the phase machine:
//!
//! * [`Entity::ai_step`] runs vanilla `EnderDragon.aiStep` instead of the inherited
//!   mob travel.
//! * [`LivingEntity::hurt_server`] applies vanilla `EnderDragon.hurt`'s gates and
//!   per-part scaling, plus the sitting-damage accumulator that forces a takeoff.
//! * [`LivingEntity::handle_killing_blow`] converts a lethal hit into the death
//!   dive: health pins at `1` and the phase switches to `Dying`, which drops it to
//!   `0` once the dragon reaches the podium.
//! * [`LivingEntity::tick_death`] runs the 180–200 tick death: experience in two
//!   batches, the `1028` level event on the first tick, and a slow upward drift.
//!
//! The dragon installs **no** goals, matching vanilla's empty `registerGoals`: all
//! behaviour is the phase machine.

pub mod flight_history;
pub mod part;
pub mod phase_ticks;
pub mod phases;

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::{NbtCompound, NbtTag};
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::EnderDragonEntityData;
use steel_registry::{
    vanilla_attributes, vanilla_blocks, vanilla_damage_type_tags, vanilla_damage_types,
    vanilla_entities, vanilla_game_events, vanilla_game_rules, vanilla_particle_types,
};
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::types::GameType;
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, DowncastType, DowncastTypeKey, WorldAabb};

use self::part::{
    DRAGON_PART_BODY, DRAGON_PART_COUNT, DRAGON_PART_HEAD, DRAGON_PART_NECK, part_damage_scaling,
};
use self::flight_history::DragonFlightHistory;
use self::phase_ticks::{FLAME_DURATION, PhaseAction, SITTING_FLAME_RADIUS, phase_fly_target, tick_phase};
use self::phases::{DragonPhase, DragonPhaseManager, turn_speed};
use crate::entity::damage::DamageSource;
use crate::entity::entities::ExperienceOrbEntity;
use crate::entity::entities::objects::effects::AreaEffectCloudEntity;
use crate::entity::entities::objects::projectiles::DragonFireballEntity;
use crate::entity::{
    DefaultEntityPush, Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntityPose,
    EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase, Monster, PathfinderMob,
    RemovalReason, SharedEntity, WeakEntity, sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::player::Player;
use crate::world::World;

const DEFAULT_STEP_HEIGHT: f32 = 0.6;

/// Vanilla `EnderDragon.SITTING_ALLOWED_DAMAGE_PERCENTAGE`.
///
/// Damage taken while sitting accumulates; passing a quarter of the max health
/// forces a takeoff.
const SITTING_ALLOWED_DAMAGE_PERCENTAGE: f32 = 0.25;

/// Vanilla `EnderDragon.getSoundVolume`.
const DRAGON_SOUND_VOLUME: f32 = 5.0;

/// Vanilla `EnderDragon`'s first-kill experience drop.
const FIRST_KILL_XP: i32 = 12_000;
/// Vanilla `EnderDragon`'s repeat-kill experience drop.
const REPEAT_KILL_XP: i32 = 500;

/// Vanilla `EnderDragon`'s death animation length in ticks.
const DRAGON_DEATH_DURATION: i32 = 200;

/// Vanilla NBT key for the death animation counter.
const DRAGON_DEATH_TIME_KEY: &str = "DragonDeathTime";
/// Vanilla NBT key for the current phase id.
const DRAGON_PHASE_KEY: &str = "DragonPhase";
/// Vanilla NBT key for the sitting-damage accumulator.
const SITTING_DAMAGE_RECEIVED_KEY: &str = "sitting_damage_received";

/// Vanilla `LevelEvent.SOUND_DRAGON_DEATH` (`1028`).
const SOUND_DRAGON_DEATH: i32 = 1028;
/// Vanilla `LevelEvent.PARTICLES_DRAGON_BLOCK_DESTROY` (`2008`).
const PARTICLES_DRAGON_BLOCK_DESTROY: i32 = 2008;

/// Vanilla `BlockTags.DRAGON_IMMUNE` members (`26.3` block tags).
///
/// Steel's generated tag data has no `dragon_immune` tag yet, so the members are
/// inlined. Keep in sync with `data/minecraft/tags/block/dragon_immune.json`.
const DRAGON_IMMUNE_BLOCKS: &[steel_registry::blocks::BlockRef] = &[
    &vanilla_blocks::BARRIER,
    &vanilla_blocks::BEDROCK,
    &vanilla_blocks::END_PORTAL,
    &vanilla_blocks::END_PORTAL_FRAME,
    &vanilla_blocks::END_GATEWAY,
    &vanilla_blocks::COMMAND_BLOCK,
    &vanilla_blocks::REPEATING_COMMAND_BLOCK,
    &vanilla_blocks::CHAIN_COMMAND_BLOCK,
    &vanilla_blocks::STRUCTURE_BLOCK,
    &vanilla_blocks::JIGSAW,
    &vanilla_blocks::MOVING_PISTON,
    &vanilla_blocks::OBSIDIAN,
    &vanilla_blocks::CRYING_OBSIDIAN,
    &vanilla_blocks::END_STONE,
    &vanilla_blocks::IRON_BARS,
    &vanilla_blocks::RESPAWN_ANCHOR,
    &vanilla_blocks::REINFORCED_DEEPSLATE,
];

/// Vanilla `BlockTags.DRAGON_TRANSPARENT`: light plus the fire tag.
const DRAGON_TRANSPARENT_BLOCKS: &[steel_registry::blocks::BlockRef] = &[
    &vanilla_blocks::LIGHT,
    &vanilla_blocks::FIRE,
    &vanilla_blocks::SOUL_FIRE,
];

/// Vanilla `net.minecraft.world.entity.boss.enderdragon.EnderDragon`.
#[entity_behavior(class = "EnderDragon")]
pub struct EnderDragonEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<EnderDragonEntityData>,
    /// The phase machine plus every piece of dragon-only movement state.
    pub(crate) phases: SyncMutex<DragonPhaseManager>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `EnderDragonEntity`.
unsafe impl DowncastType for EnderDragonEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/ender_dragon");
}

impl EnderDragonEntity {
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

        let mut entity_data = EnderDragonEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        // Vanilla `EnderDragon`'s constructor sets `noPhysics = true`; every move
        // the flight code makes is unclipped, and only `checkWalls` cares about
        // blocks.
        base.set_no_physics(true);
        let position = base.position();

        // Vanilla `registerGoals` is empty: no goals are installed.
        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            entity_data: SyncMutex::new(entity_data),
            phases: SyncMutex::new(DragonPhaseManager::new(position)),
        }
    }

    /// Vanilla `EnderDragon.getPhase` (`DATA_PHASE`) as an ordinal.
    #[must_use]
    pub fn phase_id(&self) -> i32 {
        *self.entity_data.lock().ender_dragon().phase.get()
    }

    /// Returns the current phase, like vanilla `getPhaseManager().getCurrentPhase()`.
    #[must_use]
    pub fn phase(&self) -> DragonPhase {
        self.phases.lock().current()
    }

    /// Vanilla `EnderDragonPhaseManager.setPhase`.
    ///
    /// Vanilla writes `DATA_PHASE` from inside `setPhase`; Steel cannot do that
    /// from the phase tick, which already holds the manager lock, so the write is
    /// done here once the switch returned.
    pub fn set_phase(&self, phase: DragonPhase) {
        if self.phases.lock().set_phase(phase) {
            self.sync_phase_data();
        }
    }

    /// Writes the current phase into the synced `DATA_PHASE`.
    fn sync_phase_data(&self) {
        let phase = self.phases.lock().current();
        self.entity_data.lock().ender_dragon_mut().phase.set(phase.id());
    }

    /// Returns the number of end crystals still alive around the arena.
    ///
    /// Vanilla reads `EnderDragonFight.aliveCrystals()`; the fight lives on the
    /// world, so this is the dragon's accessor for it.
    #[must_use]
    pub fn alive_crystals(&self, world: &World) -> i32 {
        world.dragon_fight.alive_crystals()
    }

    /// Vanilla `EnderDragonFight.getPodiumLocation(fightOrigin)`, resolved through
    /// the `MOTION_BLOCKING_NO_LEAVES` heightmap and cached like vanilla does.
    #[must_use]
    pub fn podium_location(&self, world: &World) -> BlockPos {
        if let Some(podium) = self.phases.lock().podium_location {
            return podium;
        }
        let podium = world.heightmap_pos(
            crate::chunk::heightmap::HeightmapType::MotionBlockingNoLeaves,
            crate::world::dragon_fight::DragonFight::podium_pos(),
        );
        self.phases.lock().podium_location = Some(podium);
        podium
    }

    /// Vanilla `RandomSource.nextInt(bound)` for a non-negative bound.
    ///
    /// Vanilla treats a non-positive bound as "any int". Every call site here
    /// passes a positive bound, so a zero or negative bound is clamped to `1`
    /// (always returning `0`) to keep the function total.
    #[must_use]
    pub fn random_below(&self, bound: i32) -> i32 {
        if bound <= 1 {
            return 0;
        }
        rand::random_range(0..bound)
    }

    /// Vanilla `RandomSource.nextFloat()`.
    #[must_use]
    pub fn random_f32(&self) -> f32 {
        rand::random::<f32>()
    }

    /// Vanilla `RandomSource.nextLong()`, used to seed the per-step phase stream.
    ///
    /// The phases draw several values per tick; vanilla takes them from the entity's
    /// single `RandomSource`, so Steel seeds one [`PhaseContext`] stream from here
    /// and advances it per draw.
    #[must_use]
    pub fn random_i64(&self) -> i64 {
        rand::random::<i64>()
    }

    /// Returns whether `player` may be targeted by this dragon's phases.
    ///
    /// Vanilla uses `TargetingConditions.forCombat().ignoreLineOfSight()`, which
    /// admits any visible, non-spectator, non-creative player.
    #[must_use]
    pub fn can_target_player(&self, player: &Player) -> bool {
        let mode = player.game_mode();
        mode != GameType::Spectator && mode != GameType::Creative && LivingEntity::is_alive(player)
    }

    /// Vanilla `EnderDragon.aiStep`'s server half.
    ///
    /// This is the entire dragon brain: adopt any externally written phase, advance
    /// the flap phase, heal from crystals, run the phase, steer toward its fly
    /// target, place the eight parts, sweep the wings, and update the wall flag.
    fn run_ai_step(&self, shared: &SharedEntity) {
        let Some(world) = self.level() else {
            return;
        };

        // Vanilla adopts a phase written into `DATA_PHASE` from outside.
        let synced = self.phase_id();
        self.phases.lock().sync_phase(synced);

        {
            let mut guard = self.phases.lock();
            guard.old_flap_time = guard.flap_time;
        }

        if LivingEntity::is_dead_or_dying(self) {
            self.spawn_death_particle(&world);
            return;
        }

        self.check_crystals(&world);
        self.update_flap_phase();

        // Vanilla clamps the yaw into `[-180, 180)` every step.
        let (yaw, pitch) = Entity::rotation(self);
        Entity::set_rotation(self, (steel_math::wrap_degrees(yaw), pitch));

        if Mob::is_no_ai(self) {
            self.phases.lock().flap_time = 0.5;
            return;
        }

        let position = Entity::position(self);
        let yaw = Entity::rotation(self).0;
        self.phases.lock().record_flight(position.y, yaw);

        // Everything the phases read is hoisted into a context *before* the
        // manager is locked, so no phase tick ever re-enters `self.phases`.
        let phase = self.phase();
        let podium = self.podium_location(&world);
        let ctx = self.build_phase_context(&world, phase, position, yaw, pitch, podium);

        let mut actions = Vec::new();
        {
            let mut guard = self.phases.lock();
            tick_phase(&mut guard, &ctx, phase, &mut actions);
            // Vanilla re-runs `doServerTick` after a mid-tick phase switch, so a
            // hand-off takes effect in the same step.
            let switched = guard.current();
            if switched != phase {
                let ctx = self.build_phase_context(&world, switched, position, yaw, pitch, podium);
                tick_phase(&mut guard, &ctx, switched, &mut actions);
            }
        }
        // Vanilla's phase ticks write `DATA_PHASE` through `setPhase`; Steel's
        // manager cannot touch the entity data, so the write happens here.
        if self.phases.lock().current() != phase {
            self.sync_phase_data();
        }
        self.apply_phase_actions(&world, shared, &actions);
        for cloud in self.phases.lock().drain_phase_discards() {
            if let Some(cloud) = cloud.upgrade() {
                cloud.set_removed(RemovalReason::Discarded);
            }
        }

        let phase = self.phase();
        self.apply_flight(phase);
        Entity::apply_effects_from_blocks(self);
        self.update_parts();
        self.sweep_wings(&world, shared);
        self.update_walls(&world);
        // Vanilla `aiStep` calls `dragonFight.updateDragon(this)` after updating the
        // parts, which refreshes the boss bar and the "seen" timer.
        world.update_dragon_fight(self.health_fraction());
    }

    /// Collects the world reads the phases need for this step.
    fn build_phase_context<'a>(
        &'a self,
        world: &'a Arc<World>,
        phase: DragonPhase,
        position: DVec3,
        yaw: f32,
        pitch: f32,
        podium: BlockPos,
    ) -> self::phase_ticks::PhaseContext<'a> {
        let alive_crystals = self.alive_crystals(world);
        // Vanilla queries only the phases that look for a player.
        let scanned_player = if phase == DragonPhase::SittingScanning {
            self::phase_ticks::scanning_player(self, world, position)
        } else {
            None
        };
        let charge_player = if phase == DragonPhase::SittingScanning {
            self::phase_ticks::charging_player(self, world, position)
        } else {
            None
        };
        // Vanilla `NEW_TARGET_TARGETING` is used by the holding pattern and the
        // landing approach; the strafe phase keeps its own recorded target.
        let arena_player = if matches!(
            phase,
            DragonPhase::HoldingPattern | DragonPhase::LandingApproach
        ) {
            self::phase_ticks::arena_player(self, world, podium)
        } else {
            None
        };

        self::phase_ticks::PhaseContext {
            world,
            dragon: self,
            position,
            yaw,
            pitch,
            alive_crystals,
            podium_location: podium,
            horizontal_collision: Entity::horizontal_collision(self),
            vertical_collision: Entity::vertical_collision(self),
            scanned_player,
            charge_player,
            arena_player,
            random: self.random_f32(),
            roll_eighth: self.random_below(8) == 0,
            random_state: std::cell::Cell::new(self.random_i64() as u64),
        }
    }

    /// Replays the phase tick's queued world changes, in the order vanilla makes
    /// them.
    fn apply_phase_actions(
        &self,
        world: &Arc<World>,
        shared: &SharedEntity,
        actions: &[PhaseAction],
    ) {
        for action in actions {
            match action {
                PhaseAction::PlayShootSound => {
                    Entity::play_sound(
                        self,
                        &steel_registry::sound_events::ENTITY_ENDER_DRAGON_SHOOT,
                        DRAGON_SOUND_VOLUME,
                        0.8 + self.random_f32() * 0.3,
                    );
                }
                PhaseAction::SpawnFireball { origin, direction } => {
                    self.spawn_dragon_fireball(world, *origin, *direction);
                }
                PhaseAction::SpawnBreathCloud { x, head_y, z } => {
                    // Vanilla keeps the cloud on the phase so `end()` can discard it.
                    if let Some(cloud) = self.spawn_breath_cloud(world, shared, *x, *head_y, *z) {
                        self.phases.lock().state_for(DragonPhase::SittingFlaming).flame_cloud =
                            Some(cloud);
                    }
                }
                PhaseAction::StrafeShotLevelEvent => {
                    world.level_event(1017, self.block_position(), 0, None);
                }
                PhaseAction::SetHealth(health) => {
                    // Vanilla `DragonDeathPhase` writes health directly; it must not
                    // re-enter the damage pipeline, which would run `die` again.
                    LivingEntity::set_health(self, *health);
                }
                PhaseAction::SetYaw(yaw) => {
                    let (_, pitch) = Entity::rotation(self);
                    Entity::set_rotation(self, (*yaw, pitch));
                }
            }
        }
    }

    /// Vanilla `EnderDragonFireball`'s launch from `DragonStrafePlayerPhase`.
    ///
    /// Vanilla spawns it `2.0` blocks behind the head along the look vector and
    /// lets the fireball's own `shoot` aim it; Steel passes the pre-computed
    /// direction so the phase tick stays free of entity handles.
    fn spawn_dragon_fireball(&self, world: &Arc<World>, origin: DVec3, direction: DVec3) {
        let fireball = DragonFireballEntity::new(
            &vanilla_entities::DRAGON_FIREBALL,
            crate::entity::next_entity_id(),
            origin,
            Arc::downgrade(world),
        );
        fireball.set_velocity(direction);
        if let Err(error) = world.try_add_entity(Arc::new(fireball) as SharedEntity) {
            log::debug!("world rejected a dragon fireball: {error}");
        }
    }

    /// Vanilla `DragonSittingFlamingPhase`'s breath cloud.
    ///
    /// The head position is dropped to the first solid block below it, then the
    /// cloud is given radius five, a 200-tick life, the dragon-breath particle and
    /// instant damage at a quarter duration scale, matching
    /// `AreaEffectCloud.setPotionDurationScale(0.25F)`.
    fn spawn_breath_cloud(
        &self,
        world: &Arc<World>,
        shared: &SharedEntity,
        x: f64,
        head_y: f64,
        z: f64,
    ) -> Option<WeakEntity> {
        // Vanilla drops the head position until a non-empty block, then uses
        // `floor(y) + 1` for the cloud and keeps the exact `x`/`z`.
        let y = Self::breath_cloud_origin(world, x, head_y, z)?;
        let cloud = Arc::new(AreaEffectCloudEntity::new(
            &vanilla_entities::AREA_EFFECT_CLOUD,
            crate::entity::next_entity_id(),
            DVec3::new(x, y, z),
            Arc::downgrade(world),
        ));
        cloud.set_owner(Some(shared));
        cloud.set_radius(SITTING_FLAME_RADIUS);
        cloud.set_duration(FLAME_DURATION);
        cloud.set_particle(steel_registry::particle_type::ParticleData::new(
            &steel_registry::vanilla_particle_types::DRAGON_BREATH,
            steel_registry::particle_type::PowerParticleOption::new(1.0),
        ));
        cloud.set_potion_duration_scale(0.25);
        // Vanilla `flame.addEffect(new MobEffectInstance(MobEffects.INSTANT_DAMAGE))`,
        // i.e. the 3-argument constructor: duration 0, amplifier 0.
        cloud.set_potion_contents(steel_registry::data_components::components::PotionContents::new(
            None,
            None,
            vec![steel_registry::mob_effect::instance::MobEffectInstance::new(
                &steel_registry::vanilla_mob_effects::INSTANT_DAMAGE,
                0,
                0,
                false,
                true,
                true,
                None,
            )],
            None,
        ));
        let shared = cloud as SharedEntity;
        if let Err(error) = world.try_add_entity(Arc::clone(&shared)) {
            log::debug!("world rejected a dragon breath cloud: {error}");
            return None;
        }
        Some(Arc::downgrade(&shared))
    }

    /// Vanilla `EnderDragon.checkCrystals`: the nearest crystal within 32 blocks
    /// heals one health every ten ticks, and it is rescanned one time in ten.
    fn check_crystals(&self, world: &Arc<World>) {
        let healed = {
            let mut guard = self.phases.lock();
            match guard.nearest_crystal.as_ref().and_then(Weak::upgrade) {
                None => {
                    guard.nearest_crystal = None;
                    false
                }
                Some(crystal) if crystal.is_removed() => {
                    guard.nearest_crystal = None;
                    false
                }
                Some(_) => {
                    self.tick_count() % 10 == 0 && self.get_health() < self.get_max_health()
                }
            }
        };
        if healed {
            self.set_health(self.get_health() + 1.0);
        }

        if self.random_below(10) == 0 {
            let search = self.bounding_box().inflate(32.0);
            let dragon_position = self.position();
            let nearest = world
                .get_entities_in_aabb_matching(&search, |entity| {
                    entity.entity_type() == &vanilla_entities::END_CRYSTAL
                })
                .into_iter()
                .min_by(|left, right| {
                    let left = Entity::distance_to_sqr(left.as_ref(), dragon_position);
                    let right = Entity::distance_to_sqr(right.as_ref(), dragon_position);
                    left.total_cmp(&right)
                });
            self.phases.lock().nearest_crystal = nearest.as_ref().map(Arc::downgrade);
        }
    }

    /// Vanilla `aiStep`'s flap-speed blend.
    fn update_flap_phase(&self) {
        let velocity = Entity::velocity(self);
        // Vanilla `Vec3.horizontalDistance()` = `sqrt(x*x + z*z)`, not `hypot`.
        let horizontal = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt() as f32;
        let mut flap_speed = 0.2 / (horizontal * 10.0 + 1.0);

        let mut guard = self.phases.lock();
        if guard.current().is_sitting() {
            guard.flap_time += 0.1;
        } else if guard.in_wall {
            guard.flap_time += flap_speed * 0.5;
        } else {
            flap_speed *= 2.0_f32.powf(velocity.y as f32);
            guard.flap_time += flap_speed;
        }
    }

    /// Vanilla `aiStep`'s flight block: steer toward the phase's fly target.
    fn apply_flight(&self, phase: DragonPhase) {
        let target = {
            let guard = self.phases.lock();
            phase_fly_target(phase, &guard)
        };
        let Some(target) = target else {
            return;
        };

        let position = Entity::position(self);
        let xdd = target.x - position.x;
        let ydd = target.y - position.y;
        let zdd = target.z - position.z;
        let dist_to_target = xdd * xdd + ydd * ydd + zdd * zdd;

        // Vanilla clamps the vertical component to the phase's fly speed, then
        // turns the clamped value into an acceleration of `* 0.01`.
        let max = f64::from(phase.fly_speed());
        let horizontal_dist = (xdd * xdd + zdd * zdd).sqrt();
        let mut delta_y = ydd;
        if horizontal_dist > 0.0 {
            delta_y = (delta_y / horizontal_dist).clamp(-max, max);
        }

        let mut velocity = Entity::velocity(self);
        velocity.y += delta_y * 0.01;
        Entity::set_velocity(self, velocity);

        let (yaw, pitch) = Entity::rotation(self);
        let yaw = steel_math::wrap_degrees(yaw);
        Entity::set_rotation(self, (yaw, pitch));

        // Vanilla's `dir` is the dragon's facing plus its vertical speed, which is
        // what the wing sweep and the slide both key off.
        let aim = DVec3::new(xdd, ydd, zdd).normalize_or_zero();
        let yaw_rad = f64::from(yaw.to_radians());
        let dir = DVec3::new(
            f64::from(steel_math::trig::sin(yaw_rad)),
            velocity.y,
            -f64::from(steel_math::trig::cos(yaw_rad)),
        )
        .normalize_or_zero();
        let dot = ((dir.dot(aim) + 0.5) / 1.5).max(0.0) as f32;

        // Vanilla turns toward the target with `yRotA` damping; the turn rate is
        // the phase's own `getTurnSpeed`.
        if xdd.abs() > 1.0E-5 || zdd.abs() > 1.0E-5 {
            let target_yaw = 180.0 - f64::from(steel_math::trig::atan2(xdd, zdd)) * 57.295_776;
            let y_rot_d =
                steel_math::wrap_degrees((target_yaw - f64::from(yaw)) as f32).clamp(-50.0, 50.0);
            let turn = turn_speed(phase, self);
            let y_rot_a = {
                let mut guard = self.phases.lock();
                guard.y_rot_a = guard.y_rot_a * 0.8 + y_rot_d * turn;
                guard.y_rot_a
            };
            let (yaw, pitch) = Entity::rotation(self);
            Entity::set_rotation(self, (yaw + y_rot_a * 0.1, pitch));
        }

        // Vanilla `moveRelative`: the dragon accelerates along its facing, scaled by
        // how well it is lined up with the target.
        let span = (2.0 / (dist_to_target + 1.0)) as f32;
        Entity::move_relative(
            self,
            0.06 * (dot * span + (1.0 - span)),
            DVec3::new(0.0, 0.0, -1.0),
        );

        // `noPhysics` makes the move an unclipped position update, which is what
        // `move_without_physics` does; it also refreshes the collision flags the
        // phases poll.
        let in_wall = self.phases.lock().in_wall;
        let velocity = Entity::velocity(self);
        let delta = if in_wall { velocity * 0.8 } else { velocity };
        Entity::move_without_physics(self, delta);

        // Vanilla slides the dragon along the surface it hit and bleeds off vertical
        // speed. It does this on every tick with a fly target, not only when the move
        // hit something, so the damping applies to free flight as well.
        let actual = Entity::velocity(self).normalize_or_zero();
        let slide = 0.8 + 0.15 * (actual.dot(dir) + 1.0) / 2.0;
        let velocity = Entity::velocity(self);
        Entity::set_velocity(
            self,
            DVec3::new(velocity.x * slide, velocity.y * 0.91, velocity.z * slide),
        );
    }

    /// Vanilla `aiStep`'s part placement.
    ///
    /// The head and neck follow the last yaw velocity, the tails follow the flight
    /// history twelve to sixteen ticks back, and the wing tilt comes from the
    /// five- versus ten-tick height delta.
    fn update_parts(&self) {
        let position = Entity::position(self);
        let (yaw, y_rot_a) = {
            let guard = self.phases.lock();
            (Entity::rotation(self).0, guard.y_rot_a)
        };
        let rot1 = f32::to_radians(yaw);

        let mut guard = self.phases.lock();
        let (tilt, y_offset) = {
            let history = &guard.flight_history;
            let tilt = ((history.get(5).y - history.get(10).y) * 10.0
                * f64::from(std::f32::consts::PI)
                / 180.0) as f32;
            let y_offset = if guard.current().is_sitting() {
                -1.0
            } else {
                (history.get(5).y - history.get(0).y) as f32
            };
            (tilt, y_offset)
        };
        let cc_tilt = steel_math::trig::cos(tilt);
        let ss_tilt = steel_math::trig::sin(tilt);

        let ss1 = steel_math::trig::sin(f64::from(rot1));
        let cc1 = steel_math::trig::cos(f64::from(rot1));

        guard.parts.tick_part(
            DRAGON_PART_BODY,
            position,
            DVec3::new(f64::from(ss1) * 0.5, 0.0, -f64::from(cc1) * 0.5),
        );
        guard.parts.tick_part(
            part::DRAGON_PART_WING1,
            position,
            DVec3::new(f64::from(cc1) * 4.5, 2.0, f64::from(ss1) * 4.5),
        );
        guard.parts.tick_part(
            part::DRAGON_PART_WING2,
            position,
            DVec3::new(f64::from(cc1) * -4.5, 2.0, f64::from(ss1) * -4.5),
        );

        // `y_rot_a` is a vanilla `f32` (DragonFlightHistory.yRotA), so widen it
        // before the f64 subtraction rather than narrowing the angle.
        let ss2 = steel_math::trig::sin(f64::from(rot1) - f64::from(y_rot_a) * 0.01);
        let cc2 = steel_math::trig::cos(f64::from(rot1) - f64::from(y_rot_a) * 0.01);
        guard.parts.tick_part(
            DRAGON_PART_HEAD,
            position,
            DVec3::new(
                f64::from(6.5 * ss2) * f64::from(cc_tilt),
                f64::from(y_offset) + f64::from(6.5 * ss_tilt),
                f64::from(-6.5 * cc2) * f64::from(cc_tilt),
            ),
        );
        guard.parts.tick_part(
            DRAGON_PART_NECK,
            position,
            DVec3::new(
                f64::from(5.5 * ss2) * f64::from(cc_tilt),
                f64::from(y_offset) + f64::from(5.5 * ss_tilt),
                f64::from(-5.5 * cc2) * f64::from(cc_tilt),
            ),
        );

        let p1 = guard.flight_history.get(5);
        for (index, part_index) in [
            (0usize, part::DRAGON_PART_TAIL1),
            (1, part::DRAGON_PART_TAIL2),
            (2, part::DRAGON_PART_TAIL3),
        ] {
            let p0 = guard.flight_history.get(12 + index * 2);
            let rot = f32::to_radians(yaw)
                + f32::to_radians(steel_math::wrap_degrees((p0.y_rot - p1.y_rot) as f32));
            let ss = steel_math::trig::sin(f64::from(rot));
            let cc = steel_math::trig::cos(f64::from(rot));
            let dd = (index as f32 + 1.0) * 2.0;
            guard.parts.tick_part(
                part_index,
                position,
                DVec3::new(
                    f64::from(-(ss1 * 1.5 + ss * dd)) * f64::from(cc_tilt),
                    p0.y - p1.y - f64::from((dd + 1.5) * ss_tilt) + 1.5,
                    f64::from(cc1 * 1.5 + cc * dd) * f64::from(cc_tilt),
                ),
            );
        }
    }

    /// Vanilla `aiStep`'s wing sweep: shove anything under a wing, then hit
    /// anything inside the head and neck boxes.
    fn sweep_wings(&self, world: &Arc<World>, shared: &SharedEntity) {
        // Vanilla skips both sweeps while the dragon is still in hurt-flash time.
        if LivingEntity::hurt_time(self) > 0 {
            return;
        }

        let (wing_boxes, head_box, neck_box) = {
            let guard = self.phases.lock();
            let [wing1, wing2] = guard.parts.wing_sweep_boxes();
            (
                [wing1, wing2],
                guard.parts.bounding_box(DRAGON_PART_HEAD),
                guard.parts.bounding_box(DRAGON_PART_NECK),
            )
        };

        let selector = |entity: &dyn Entity| {
            !entity.is_spectator() && entity.as_living_entity().is_some()
        };

        for wing in wing_boxes.into_iter().flatten() {
            let entities = world.get_entities_in_aabb_matching(&wing, selector);
            self.wing_knock_back(world, shared, &entities);
        }

        for box_ in [head_box, neck_box].into_iter().flatten() {
            let entities = world.get_entities_in_aabb_matching(&box_, selector);
            self.head_contact_damage(world, shared, &entities);
        }
    }

    /// Vanilla `EnderDragon.knockBack`: a wing sweep shoves entities away from the
    /// body centre, and deals five damage to anything that was not just hit.
    ///
    /// Steel applies the impulse and the damage gate exactly as vanilla orders them
    /// (`push`, then the sitting/recent-hurt/team checks). Damage attribution needs
    /// the dragon's shared handle, which this `&self` path does not have, so the
    /// gate is evaluated and the hit is delivered through the entity's own
    /// damage entry point.
    fn wing_knock_back(&self, world: &Arc<World>, shared: &SharedEntity, entities: &[SharedEntity]) {
        let (xm, zm) = {
            let guard = self.phases.lock();
            guard
                .parts
                .bounding_box(DRAGON_PART_BODY)
                .map_or((0.0, 0.0), |box_| {
                    (
                        (box_.min_x() + box_.max_x()) / 2.0,
                        (box_.min_z() + box_.max_z()) / 2.0,
                    )
                })
        };

        for entity in entities {
            let Some(living) = entity.as_living_entity() else {
                continue;
            };
            let entity_position = Entity::position(entity.as_ref());
            let xd = entity_position.x - xm;
            let zd = entity_position.z - zm;
            let dd = (xd * xd + zd * zd).max(0.1);
            // Vanilla `Entity.push`: the swept entity receives the impulse, not the
            // dragon, so the direction points away from the body centre.
            Entity::set_velocity(
                entity.as_ref(),
                Entity::velocity(entity.as_ref()) + DVec3::new(xd / dd * 4.0, 0.2, zd / dd * 4.0),
            );

            let sitting = self.phase().is_sitting();
            let recently_hurt = LivingEntity::last_hurt_mob_timestamp(living)
                >= Entity::tick_count(entity.as_ref()) - 2;
            if sitting || recently_hurt || !Entity::team_allows_friendly_fire(entity.as_ref()) {
                continue;
            }

            // Vanilla `damageSources().mobAttack(this)`: the dragon is both the
            // causing and the direct entity, so kills credit it.
            let source = DamageSource::direct(&vanilla_damage_types::MOB_ATTACK, Arc::clone(shared))
                .with_source_position(entity_position);
            Entity::hurt(entity.as_ref(), world, &source, 5.0);
        }
    }

    /// Vanilla `EnderDragon.hurt(List)`: the head and neck boxes deal ten damage.
    fn head_contact_damage(
        &self,
        world: &Arc<World>,
        shared: &SharedEntity,
        entities: &[SharedEntity],
    ) {
        for entity in entities {
            if entity.as_living_entity().is_none() {
                continue;
            }
            if !Entity::team_allows_friendly_fire(entity.as_ref()) {
                continue;
            }
            let source = DamageSource::direct(&vanilla_damage_types::MOB_ATTACK, Arc::clone(shared))
                .with_source_position(Entity::position(entity.as_ref()));
            Entity::hurt(entity.as_ref(), world, &source, 10.0);
        }
    }

    /// Vanilla's downward walk from the head to the first non-empty block, used
    /// by the sitting flame. Returns `None` when the column is empty all the way
    /// to the void, where vanilla keeps the head height.
    fn breath_cloud_origin(world: &World, x: f64, head_y: f64, z: f64) -> Option<f64> {
        let mut y = head_y;
        loop {
            if !world
                .get_block_state(BlockPos::containing(x, y, z))
                .is_air()
            {
                return Some(y.floor() + 1.0);
            }
            y -= 1.0;
            if y < 0.0 {
                // Vanilla resets to the un-descended height when it runs off the
                // bottom of the world.
                return Some(head_y);
            }
        }
    }

    /// Vanilla `EnderDragon.checkWalls` over the head, neck and body boxes.
    ///
    /// Blocks the dragon overlaps are destroyed unless mob griefing is off or the
    /// block is in `DRAGON_IMMUNE`; any block that survives sets `in_wall`.
    fn update_walls(&self, world: &Arc<World>) {
        let boxes: Vec<WorldAabb> = {
            let guard = self.phases.lock();
            [DRAGON_PART_HEAD, DRAGON_PART_NECK, DRAGON_PART_BODY]
                .into_iter()
                .filter_map(|index| guard.parts.bounding_box(index))
                .collect()
        };

        let mut hit_wall = false;
        let mut destroyed_any = false;
        for box_ in boxes {
            let (hit, destroyed) = self.check_walls(world, box_);
            hit_wall |= hit;
            destroyed_any |= destroyed;
        }
        let _ = destroyed_any;

        self.phases.lock().in_wall = hit_wall;
    }

    /// Vanilla `EnderDragon.checkWalls(level, AABB)`.
    ///
    /// Returns `(hit_wall, destroyed_any_block)`.
    fn check_walls(&self, world: &Arc<World>, box_: WorldAabb) -> (bool, bool) {
        let x0 = steel_math::fast_floor(box_.min_x());
        let y0 = steel_math::fast_floor(box_.min_y());
        let z0 = steel_math::fast_floor(box_.min_z());
        let x1 = steel_math::fast_floor(box_.max_x());
        let y1 = steel_math::fast_floor(box_.max_y());
        let z1 = steel_math::fast_floor(box_.max_z());

        let mob_griefing = world.get_game_rule(&vanilla_game_rules::MOB_GRIEFING);
        let mut hit_wall = false;
        let mut destroyed_block = false;

        for x in x0..=x1 {
            for y in y0..=y1 {
                for z in z0..=z1 {
                    let pos = BlockPos::new(x, y, z);
                    let state = world.get_block_state(pos);
                    let block = state.get_block();
                    if state.is_air()
                        || DRAGON_TRANSPARENT_BLOCKS.iter().any(|skip| *skip == block)
                    {
                        continue;
                    }
                    if !mob_griefing || DRAGON_IMMUNE_BLOCKS.iter().any(|immune| *immune == block)
                    {
                        hit_wall = true;
                        continue;
                    }
                    destroyed_block |= world.remove_block(pos, false);
                }
            }
        }

        if destroyed_block && x1 >= x0 && y1 >= y0 && z1 >= z0 {
            let random_pos = BlockPos::new(
                x0 + self.random_below(x1 - x0 + 1),
                y0 + self.random_below(y1 - y0 + 1),
                z0 + self.random_below(z1 - z0 + 1),
            );
            world.level_event(PARTICLES_DRAGON_BLOCK_DESTROY, random_pos, 0, None);
        }

        (hit_wall, destroyed_block)
    }

    /// Sends the vanilla `EXPLOSION` particle the dying dragon trails each tick.
    fn spawn_death_particle(&self, world: &Arc<World>) {
        self.send_dragon_particle(world, vanilla_particle_types::EXPLOSION);
    }

    /// Sends the vanilla `EXPLOSION_EMITTER` particle the death animation fires.
    fn spawn_death_explosion(&self, world: &Arc<World>) {
        self.send_dragon_particle(world, vanilla_particle_types::EXPLOSION_EMITTER);
    }

    fn send_dragon_particle(
        &self,
        world: &Arc<World>,
        particle: steel_registry::particle_type::ParticleTypeRef,
    ) {
        let offset = DVec3::new(
            f64::from((self.random_f32() - 0.5) * 8.0),
            2.0 + f64::from((self.random_f32() - 0.5) * 4.0),
            f64::from((self.random_f32() - 0.5) * 8.0),
        );
        world.send_particles(
            steel_registry::particle_type::ParticleData::simple(particle),
            self.position() + offset,
            1,
            DVec3::ZERO,
            0.0,
        );
    }

    /// Vanilla `aiStep`'s `dragonFight == null` relink and `EnderDragonFight`'s
    /// own `createNewDragon` contract: the dragon registers itself with the fight
    /// it belongs to.
    fn link_to_fight(&self) {
        let Some(world) = self.level() else {
            return;
        };
        if !world.is_end_dimension_type() || self.is_removed() || !LivingEntity::is_alive(self) {
            return;
        }

        let own_uuid = Entity::uuid(self);
        if world.dragon_fight.dragon_uuid() != Some(own_uuid) {
            world.dragon_fight.set_dragon_uuid(Some(own_uuid));
        }

        let position = self.block_position();
        if world.dragon_fight.dragon_target() != Some(position.0) {
            world.dragon_fight.set_dragon(Some(position.0));
        }

        // Vanilla recounts crystals from `EnderDragonFight.tick`; the fight has no
        // world handle, so the dragon drives it while it is alive.
        world.update_dragon_fight_crystals();
    }

    /// The part a damage source is striking, resolved by ray.
    ///
    /// Falls back to the body when the source carries no position at all, which is
    /// vanilla's behaviour for positional damage with no reachable origin.
    fn part_for_damage(&self, source: &DamageSource) -> usize {
        let Some(from) = source.source_position().or_else(|| {
            source
                .causing_entity()
                .map(|entity| Entity::position(entity.as_ref()))
        }) else {
            return DRAGON_PART_BODY;
        };
        let to = self.position();
        self.phases
            .lock()
            .parts
            .hit_part(from, to)
            .unwrap_or(DRAGON_PART_BODY)
    }

    /// Current health as a fraction of the maximum, for the boss bar.
    fn health_fraction(&self) -> f32 {
        let max_health = self.get_max_health();
        if max_health > 0.0 {
            self.get_health() / max_health
        } else {
            0.0
        }
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        sync_dirty_mob_effects(&self.living_base, &self.entity_data);
    }

    /// Vanilla `EnderDragon.hurt(level, part, source, damage)`.
    ///
    /// Applies the sitting phases' `onHurt`, the per-part scaling, the "only
    /// players and explosions hurt dragons" gate, and the sitting-damage
    /// accumulator.
    pub fn hurt_part_server(
        &self,
        world: &World,
        source: &DamageSource,
        amount: f32,
        part_index: usize,
    ) -> bool {
        if self.phase() == DragonPhase::Dying {
            return false;
        }

        let mut damage = phase_ticks::sitting_on_hurt(self, source, amount);
        let (multiplier, flat) = part_damage_scaling(part_index);
        if multiplier != 1.0 {
            // Vanilla `damage / 4.0f + Math.min(damage, 1.0f)`: two separately
            // rounded operations, not a fused multiply-add.
            damage = damage * multiplier + damage.min(flat);
        }
        if damage < 0.01 {
            return false;
        }

        // Vanilla `EnderDragon.hurt` lets through direct player hits and anything
        // tagged `DamageTypeTags.ALWAYS_HURTS_ENDER_DRAGONS`; every other source
        // bounces off without touching the health.
        let player_source = source
            .causing_entity()
            .is_some_and(|entity| entity.as_player().is_some());
        let always_hurts =
            source.is(&vanilla_damage_type_tags::DamageTypeTag::ALWAYS_HURTS_ENDER_DRAGONS);

        if player_source || always_hurts {
            let health_before = self.get_health();
            LivingEntity::default_hurt_server(self, world, source, damage);

            if self.phase().is_sitting() {
                let absorbed = health_before - self.get_health();
                let total = {
                    let mut guard = self.phases.lock();
                    guard.sitting_damage_received += absorbed;
                    guard.sitting_damage_received
                };
                let limit = SITTING_ALLOWED_DAMAGE_PERCENTAGE * self.get_max_health();
                if total > limit {
                    self.phases.lock().sitting_damage_received = 0.0;
                    self.set_phase(DragonPhase::Takeoff);
                }
            }
        }

        true
    }
}

impl DefaultEntityPush for EnderDragonEntity {}

impl Entity for EnderDragonEntity {
    /// Vanilla's hitbox is the loose 16x8x16 box; the parts carry the real hit
    /// areas, so movement collision never needs this entity.
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

    /// Vanilla `EnderDragon.baseTick` plus the fight link: the dragon claims the
    /// fight by UUID on its first tick, which is what makes crystals beam at it and
    /// the boss bar appear.
    fn base_tick(&self) {
        Mob::base_tick_mob(self);
        self.link_to_fight();
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

    /// Vanilla `EnderDragon.isPickable` is `false` because the *parts* are the hit
    /// targets. Steel has no part entities, so the dragon itself must be pickable
    /// for melee and projectile paths to reach it.
    fn is_pickable(&self) -> bool {
        true
    }

    /// Vanilla `EnderDragon.canRide` is `false`.
    fn can_ride(&self, _vehicle: &dyn Entity) -> bool {
        false
    }

    /// Vanilla `EnderDragon.checkDespawn` is empty: the boss never despawns.
    fn check_despawn(&self) {}

    /// Vanilla `EnderDragon.isFlapping`.
    fn is_flapping(&self) -> bool {
        self.phases.lock().is_flapping()
    }

    /// Vanilla `EnderDragon.flapTime` drives `onFlap`; the dragon only plays the
    /// flap sound for its own AI, so nothing extra is needed here.
    fn on_flap(&self) {}

    /// A hit that reaches the dragon resolves against whichever part the damage
    /// ray enters first, which is what gives the head and neck its full damage and
    /// every other part its quarter.
    ///
    /// Vanilla gets this for free: melee and projectiles hit `EnderDragonPart`
    /// entities, each of which forwards to `EnderDragon.hurt(part, ...)`. Steel has
    /// no part entities, so the ray is resolved here from the damage source's
    /// direct entity or source position to the dragon's centre.
    fn hurt(&self, world: &World, source: &DamageSource, amount: f32) -> bool {
        let part = self.part_for_damage(source);
        self.hurt_part_server(world, source, amount, part)
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
        let (phase, death_time, sitting_damage) = {
            let guard = self.phases.lock();
            (
                guard.current().id(),
                guard.death_time,
                guard.sitting_damage_received,
            )
        };
        nbt.insert(DRAGON_PHASE_KEY, NbtTag::Int(phase));
        nbt.insert(DRAGON_DEATH_TIME_KEY, NbtTag::Int(death_time));
        nbt.insert(SITTING_DAMAGE_RECEIVED_KEY, NbtTag::Float(sitting_damage));
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        let phase = nbt.int_value(DRAGON_PHASE_KEY).unwrap_or(0);
        let death_time = nbt.int_value(DRAGON_DEATH_TIME_KEY).unwrap_or(0);
        let sitting_damage = nbt.float_value(SITTING_DAMAGE_RECEIVED_KEY).unwrap_or(0.0);
        {
            let mut guard = self.phases.lock();
            guard.restore(phase, death_time, sitting_damage);
        }
        self.sync_phase_data();
    }
}

impl LivingEntity for EnderDragonEntity {
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
        Some(&steel_registry::sound_events::ENTITY_ENDER_DRAGON_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&steel_registry::sound_events::ENTITY_ENDER_DRAGON_DEATH)
    }

    fn sound_volume(&self) -> f32 {
        DRAGON_SOUND_VOLUME
    }

    /// Vanilla `EnderDragon` has no loot table and awards its own experience from
    /// the death tick, so the generic drops stay off.
    fn should_drop_experience(&self) -> bool {
        false
    }

    fn should_drop_loot(&self, _world: &World) -> bool {
        false
    }

    /// Vanilla `EnderDragon.hurtServer` → `hurt(level, body, source, damage)`.
    fn hurt_server(&self, world: &World, source: &DamageSource, amount: f32) -> bool {
        self.hurt_part_server(world, source, amount, DRAGON_PART_BODY)
    }

    /// Vanilla `EnderDragon.handleKillingBlow`: a lethal hit while not sitting pins
    /// the health at one and starts the death dive.
    ///
    /// Steel's `die` calls this *after* the health has already reached zero, so the
    /// clamp is what keeps the dragon alive for the 200-tick animation; the death
    /// processing that `die` continues with is then skipped because the dragon is
    /// no longer at zero health.
    fn handle_killing_blow(&self) {
        if self.phase() == DragonPhase::Dying {
            return;
        }
        if !self.phase().is_sitting() {
            self.set_health(1.0);
            self.set_phase(DragonPhase::Dying);
        }
    }

    /// Vanilla `EnderDragon.aiStep` replaces the inherited travel entirely.
    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        self.run_ai_step(entity);
        None
    }

    /// Vanilla `EnderDragon.serverAiStep` is empty: the goal selector never runs.
    fn server_ai_step(&self, _entity: &SharedEntity) {}

    /// Vanilla `EnderDragon.tickDeath`: a scripted 200-tick descent that awards
    /// experience in two batches and hands the kill to the fight at the end.
    ///
    /// This only runs once the dragon's health has actually reached zero, which is
    /// what the death dive's `SetHealth(0.0)` action does on arrival. Until then the
    /// dragon sits at one health and the `Dying` phase keeps steering it, exactly
    /// like vanilla's `handleKillingBlow` → `DragonDeathPhase` hand-off.
    fn tick_death(&self) {
        let Some(world) = self.level() else {
            return;
        };

        // Vanilla calls `updateDragon(this)` so the boss bar follows the last
        // health values through the animation.
        world.update_dragon_fight(self.health_fraction());

        let death_time = {
            let mut guard = self.phases.lock();
            guard.death_time += 1;
            guard.death_time
        };

        if (180..=200).contains(&death_time) {
            self.spawn_death_explosion(&world);
        }

        // Vanilla reads `hasPreviouslyKilledDragon` here; the fight only flips it in
        // `setDragonKilled` at the end of this function, so the first kill sees the
        // large drop.
        let xp = if world.dragon_fight.has_previously_killed_dragon() {
            REPEAT_KILL_XP
        } else {
            FIRST_KILL_XP
        };

        let mob_drops = world.get_game_rule(&vanilla_game_rules::MOB_DROPS);
        if death_time > 150 && death_time % 5 == 0 && mob_drops {
            ExperienceOrbEntity::award(&world, self.position(), (xp as f32 * 0.08) as i32);
        }
        if death_time == 1 {
            world.global_level_event(SOUND_DRAGON_DEATH, self.block_position(), 0);
        }

        let death_move = DVec3::new(0.0, 0.1, 0.0);
        // Vanilla moves the dragon by `(0, 0.1, 0)` through the normal move path;
        // `noPhysics` makes that an unclipped position update.
        Entity::move_without_physics(self, death_move);
        {
            let mut guard = self.phases.lock();
            for index in 0..DRAGON_PART_COUNT {
                if let Some(position) = guard.parts.position(index) {
                    guard.parts.tick_part(index, position, death_move);
                }
            }
        }

        if death_time >= DRAGON_DEATH_DURATION {
            if mob_drops {
                ExperienceOrbEntity::award(&world, self.position(), (xp as f32 * 0.2) as i32);
            }
            world.on_dragon_killed();
            self.set_removed(RemovalReason::Killed);
            self.game_event(&vanilla_game_events::ENTITY_DIE);
        }
    }

    /// The kill hand-off happens at the end of `tick_death`, matching vanilla's
    /// `tickDeath` → `setDragonKilled` ordering, so nothing happens here.
    fn on_death(&self, _source: &DamageSource) {}
}

impl Mob for EnderDragonEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn tick_goal_selectors(&self, _entity: &SharedEntity) {
        // Vanilla `EnderDragon` installs no goals and its `serverAiStep` is empty.
    }

    fn tick_path_navigation(&self) {}

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        Some(&steel_registry::sound_events::ENTITY_ENDER_DRAGON_AMBIENT)
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl Monster for EnderDragonEntity {}

impl PathfinderMob for EnderDragonEntity {}
