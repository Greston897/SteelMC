//! Vanilla `net.minecraft.world.entity.boss.enderdragon.phases`, part two: the
//! per-tick behaviour of the eleven phases.
//!
//! Vanilla gives every phase its own class and its own `doServerTick`. Each class
//! only ever touches `this.dragon` and its own fields, so a phase switch inside a
//! tick is an ordinary Java field write. In Steel the manager sits behind one
//! mutex, which makes the same structure a deadlock trap: a tick that re-entered
//! `EnderDragonEntity::phases` while the mutex was held would hang the server.
//!
//! This module is therefore written against **plain state**, not the dragon:
//!
//! * [`PhaseContext`] carries everything the phases read from the world or the
//!   dragon, all borrowed or copied *before* the mutex is taken.
//! * [`PhaseAction`] is the only way a phase may change the world: phases push
//!   actions into a buffer and the caller replays them after unlocking.
//! * [`tick_phase`] takes `&mut DragonPhaseManager` plus the context, so any
//!   number of nested helpers can run without a second lock.
//!
//! That is vanilla's "each phase mutates its own fields" model, expressed with the
//! lock discipline Steel needs.

use std::sync::Arc;

use glam::DVec3;
use steel_utils::BlockPos;

use super::EnderDragonEntity;
use crate::entity::LivingEntity;
use crate::entity::ai::targeting::TargetingConditions;
use super::part::{DRAGON_PART_HEAD, DRAGON_PART_HEAD_HEIGHT};
use super::phases::{
    DragonPhase, DragonPhaseManager, find_closest_node_with_crystals,
    find_path, wrap_degrees,
};
use crate::entity::damage::DamageSource;
use crate::entity::Entity;
use crate::entity::ai::path::Path;
use crate::world::World;

/// Vanilla `DragonStrafePlayerPhase.FIREBALL_CHARGE_AMOUNT`.
pub const FIREBALL_CHARGE_AMOUNT: i32 = 5;
/// Vanilla `DragonSittingFlamingPhase.FLAME_DURATION`.
pub const FLAME_DURATION: i32 = 200;
/// Vanilla `DragonSittingFlamingPhase.SITTING_FLAME_ATTACKS_COUNT`.
pub const SITTING_FLAME_ATTACKS_COUNT: i32 = 4;
/// Vanilla `DragonSittingFlamingPhase.WARMUP_TIME`.
pub const WARMUP_TIME: i32 = 10;
/// Vanilla `DragonSittingFlamingPhase`'s breath-cloud radius.
pub const SITTING_FLAME_RADIUS: f32 = 5.0;
/// Vanilla `DragonSittingScanningPhase.SITTING_SCANNING_IDLE_TICKS`.
pub const SITTING_SCANNING_IDLE_TICKS: i32 = 100;
/// Vanilla `DragonSittingScanningPhase.SITTING_ATTACK_Y_VIEW_RANGE`.
pub const SITTING_ATTACK_Y_VIEW_RANGE: i32 = 10;
/// Vanilla `DragonSittingScanningPhase.SITTING_ATTACK_VIEW_RANGE`.
pub const SITTING_ATTACK_VIEW_RANGE: f64 = 20.0;
/// Vanilla `DragonSittingScanningPhase.SITTING_CHARGE_VIEW_RANGE`.
pub const SITTING_CHARGE_VIEW_RANGE: f64 = 150.0;
/// Vanilla `DragonSittingAttackingPhase.ROAR_DURATION`.
pub const ROAR_DURATION: i32 = 40;
/// Vanilla `DragonChargePlayerPhase.CHARGE_RECOVERY_TIME`.
pub const CHARGE_RECOVERY_TIME: i32 = 10;
/// Vanilla `DragonLandingPhase`'s arrival radius, squared.
pub const LANDING_ARRIVE_DIST_SQR: f64 = 1.0;
/// The lower bound of the distance window a path phase accepts for its target.
pub const PATH_TARGET_MIN_DIST_SQR: f64 = 100.0;
/// The upper bound of the distance window a path phase accepts for its target.
pub const PATH_TARGET_MAX_DIST_SQR: f64 = 22_500.0;

/// Everything the phases read that lives outside the phase manager.
///
/// Built once per AI step, before the manager is locked. Every field is either a
/// copy or a shared handle, so a phase can read it while holding the mutex.
pub struct PhaseContext<'a> {
    /// The world the dragon is in.
    pub world: &'a Arc<World>,
    /// The dragon, for the reads that must stay on the real entity (line of sight).
    ///
    /// Only methods that do not lock `EnderDragonEntity::phases` may be called
    /// through it: the mutex is already held while a phase runs.
    pub dragon: &'a EnderDragonEntity,
    /// The dragon's position at the start of the step.
    pub position: DVec3,
    /// The dragon's yaw in degrees.
    pub yaw: f32,
    /// The dragon's pitch in degrees.
    pub pitch: f32,
    /// Vanilla `EnderDragonFight.aliveCrystals()`.
    pub alive_crystals: i32,
    /// Vanilla `getPodiumLocation()`.
    pub podium_location: BlockPos,
    /// Vanilla `horizontalCollision`.
    pub horizontal_collision: bool,
    /// Vanilla `verticalCollision`.
    pub vertical_collision: bool,
    /// The player the sitting-scanning phase found this tick, if any.
    pub scanned_player: Option<Arc<crate::player::Player>>,
    /// The player the sitting-scanning phase would charge at, if any.
    pub charge_player: Option<Arc<crate::player::Player>>,
    /// The arena player the holding pattern and landing approach consider.
    pub arena_player: Option<Arc<crate::player::Player>>,
    /// Vanilla `dragon.getRandom().nextFloat()` for this step.
    pub random: f32,
    /// Vanilla `dragon.getRandom().nextInt(8) == 0`.
    pub roll_eighth: bool,
    /// The dragon's random state for this step.
    ///
    /// Vanilla draws as many values as a phase needs from one stream, so a phase
    /// that rolls twice must not get the same number twice. The seed is taken from
    /// the entity's random once per AI step and advanced by [`Self::random_below`].
    ///
    /// Visible to the parent module because the AI step builds the context there;
    /// nothing outside the dragon touches it.
    pub(crate) random_state: std::cell::Cell<u64>,
}

/// Vanilla `DragonHoldingPatternPhase.NEW_TARGET_TARGETING`: combat targeting
/// with line of sight ignored, so the dragon notices players through terrain.
#[must_use]
fn new_target_targeting() -> TargetingConditions {
    TargetingConditions::for_combat().ignore_line_of_sight()
}

/// Vanilla `Level.getNearestPlayer(TargetingConditions, LivingEntity, x, y, z)`.
///
/// Vanilla measures the range from the *search point* (`x, y, z`), not from the
/// targeter, and picks the smallest squared distance to that point.
#[must_use]
fn nearest_player_to(
    dragon: &EnderDragonEntity,
    world: &World,
    conditions: &TargetingConditions,
    point: DVec3,
) -> Option<Arc<crate::player::Player>> {
    let targeter: &dyn LivingEntity = dragon;
    let mut best: Option<(Arc<crate::player::Player>, f64)> = None;
    world.players.iter_players(|_, player| {
        let candidate: &dyn LivingEntity = player.as_ref();
        if conditions.test(world, Some(targeter), candidate) {
            let distance_sqr = player.position().distance_squared(point);
            if best
                .as_ref()
                .is_none_or(|(_, current)| distance_sqr < *current)
            {
                best = Some((Arc::clone(player), distance_sqr));
            }
        }
        true
    });
    best.map(|(player, _)| player)
}

/// Vanilla `DragonHoldingPatternPhase`'s `playerNearestToEgg`.
///
/// The search is centred on the exit-portal podium, which is what makes a player
/// standing near the egg pull the dragon out of its ring.
#[must_use]
pub fn arena_player(
    dragon: &EnderDragonEntity,
    world: &World,
    podium: BlockPos,
) -> Option<Arc<crate::player::Player>> {
    nearest_player_to(
        dragon,
        world,
        &new_target_targeting(),
        DVec3::new(
            f64::from(podium.x()),
            f64::from(podium.y()),
            f64::from(podium.z()),
        ),
    )
}

/// Vanilla `DragonSittingScanningPhase.scanTargeting`: combat targeting within 20
/// blocks plus the vertical selector, this time *with* line of sight.
#[must_use]
pub fn scanning_player(
    dragon: &EnderDragonEntity,
    world: &World,
    position: DVec3,
) -> Option<Arc<crate::player::Player>> {
    let dragon_y = position.y;
    let conditions = TargetingConditions::for_combat()
        .range(SITTING_ATTACK_VIEW_RANGE)
        .selector(move |target, _world| {
            (target.position().y - dragon_y).abs() <= f64::from(SITTING_ATTACK_Y_VIEW_RANGE)
        });
    nearest_player_to(dragon, world, &conditions, position)
}

/// Vanilla `DragonSittingScanningPhase.CHARGE_TARGETING`: combat targeting within
/// 150 blocks, no vertical restriction.
#[must_use]
pub fn charging_player(
    dragon: &EnderDragonEntity,
    world: &World,
    position: DVec3,
) -> Option<Arc<crate::player::Player>> {
    let conditions = TargetingConditions::for_combat().range(SITTING_CHARGE_VIEW_RANGE);
    nearest_player_to(dragon, world, &conditions, position)
}

/// Vanilla `AbstractDragonSittingPhase.onHurt`.
///
/// While the dragon is sitting an arrow or wind charge is 'swallowed': the
/// projectile is set alight for a second and deals no damage at all. Everything
/// else passes through unchanged.
///
/// `EnderDragon.hurt_part_server` calls this before applying the per-part scaling,
/// matching the order vanilla's `hurt` uses.
#[must_use]
pub fn sitting_on_hurt(dragon: &EnderDragonEntity, source: &DamageSource, damage: f32) -> f32 {
    if !dragon.phase().is_sitting() {
        return damage;
    }
    let Some(direct) = source.direct_entity() else {
        return damage;
    };
    // Vanilla `source.getDirectEntity() instanceof AbstractArrow || WindCharge`.
    // `is_abstract_arrow` is the generated flag for the `AbstractArrow` family,
    // so arrows, spectral arrows and thrown tridents all count.
    let entity_type = direct.entity_type();
    let is_arrow_or_charge =
        entity_type.is_abstract_arrow || entity_type == &steel_registry::vanilla_entities::WIND_CHARGE;

    if !is_arrow_or_charge {
        return damage;
    }

    Entity::set_remaining_fire_ticks(direct.as_ref(), ARROW_IGNITE_TICKS);
    0.0
}

/// Vanilla `AbstractDragonSittingPhase.onHurt` ignites the projectile for one
/// second.
const ARROW_IGNITE_TICKS: i32 = 20;

impl PhaseContext<'_> {
    /// Vanilla `BlockPos.closerToCenterThan(position, distance)` against the
    /// podium, which the takeoff hand-off uses. It compares per-axis deltas, not a
    /// euclidean distance.
    #[must_use]
    pub fn distance_to_podium(&self) -> f64 {
        let dx = f64::from(self.podium_location.x()) - self.position.x;
        let dy = f64::from(self.podium_location.y()) - self.position.y;
        let dz = f64::from(self.podium_location.z()) - self.position.z;
        dx.abs().max(dy.abs()).max(dz.abs())
    }

    /// Vanilla `EnderDragon.getHeadLookVector(1.0f)` for `phase`.
    ///
    /// Landing and takeoff look down 22.5 degrees, the sitting phases 45.
    #[must_use]
    pub fn head_look(&self, phase: DragonPhase) -> DVec3 {
        let pitch = match phase {
            DragonPhase::Landing | DragonPhase::Takeoff => -22.5,
            _ if phase.is_sitting() => -45.0,
            _ => self.pitch,
        };
        let pitch_rad = f64::from(pitch.to_radians());
        let yaw_rad = f64::from(-self.yaw.to_radians());
        let yaw_cos = steel_math::trig::cos(yaw_rad);
        let yaw_sin = steel_math::trig::sin(yaw_rad);
        let pitch_cos = steel_math::trig::cos(pitch_rad);
        let pitch_sin = steel_math::trig::sin(pitch_rad);
        DVec3::new(
            f64::from(yaw_sin * pitch_cos),
            f64::from(-pitch_sin),
            f64::from(yaw_cos * pitch_cos),
        )
        .normalize_or_zero()
    }

    /// The dragon's horizontal unit facing.
    #[must_use]
    pub fn facing(&self) -> DVec3 {
        let yaw = f64::from(self.yaw.to_radians());
        DVec3::new(
            f64::from(steel_math::trig::sin(yaw)),
            0.0,
            -f64::from(steel_math::trig::cos(yaw)),
        )
        .normalize_or_zero()
    }
}

/// A world change a phase asks for.
///
/// Phases must not touch the world while the manager mutex is held, so every side
/// effect is queued and replayed in order by the dragon after unlocking
/// (`EnderDragonEntity::apply_phase_actions`). The variants mirror the Java call
/// sites one for one, so ordering is preserved.
#[derive(Debug, Clone)]
pub enum PhaseAction {
    /// Vanilla `dragon.playSound(ENTITY_ENDER_DRAGON_SHOOT, ...)`.
    PlayShootSound,
    /// Vanilla `level.addFreshEntity(new DragonFireball(...))`.
    SpawnFireball { origin: DVec3, direction: DVec3 },
    /// Vanilla `level.addFreshEntity(new AreaEffectCloud(...))` for the sitting
    /// flame. `origin` is the head position; the cloud resolves its own ground.
    SpawnBreathCloud { x: f64, head_y: f64, z: f64 },
    /// Vanilla `level.levelEvent(null, 1017, blockPosition(), 0)`.
    StrafeShotLevelEvent,
    /// Vanilla `dragon.setHealth(...)` from the death dive.
    SetHealth(f32),
    /// Vanilla `dragon.setYRot(...)` from the sitting-scan turn.
    SetYaw(f32),
}

/// Runs one tick of `phase`, mutating only `manager` and appending to `actions`.
pub fn tick_phase(
    manager: &mut DragonPhaseManager,
    ctx: &PhaseContext<'_>,
    phase: DragonPhase,
    actions: &mut Vec<PhaseAction>,
) {
    match phase {
        DragonPhase::HoldingPattern => tick_holding_pattern(manager, ctx),
        DragonPhase::StrafePlayer => tick_strafe_player(manager, ctx, actions),
        DragonPhase::LandingApproach => tick_landing_approach(manager, ctx),
        DragonPhase::Landing => tick_landing(manager, ctx),
        DragonPhase::Takeoff => tick_takeoff(manager, ctx),
        DragonPhase::SittingFlaming => tick_sitting_flaming(manager, ctx, actions),
        DragonPhase::SittingScanning => tick_sitting_scanning(manager, ctx, actions),
        DragonPhase::SittingAttacking => tick_sitting_attacking(manager, actions),
        DragonPhase::ChargingPlayer => tick_charging_player(manager, ctx),
        DragonPhase::Dying => tick_dying(manager, ctx, actions),
        DragonPhase::Hovering => tick_hovering(manager, ctx),
    }
}

/// The fly target `phase` steers toward, like
/// `DragonPhaseInstance.getFlyTargetLocation`.
#[must_use]
pub fn phase_fly_target(phase: DragonPhase, manager: &DragonPhaseManager) -> Option<DVec3> {
    if phase.is_sitting() {
        return None;
    }
    manager.current_state().target_location
}

/// Vanilla `DragonPhaseInstance.getFlySpeed` for the current manager state.
#[must_use]
pub fn fly_speed(phase: DragonPhase) -> f32 {
    phase.fly_speed()
}

// ---------------------------------------------------------------------------
// Holding pattern and strafe
// ---------------------------------------------------------------------------

/// Vanilla `DragonHoldingPatternPhase.doServerTick`.
fn tick_holding_pattern(manager: &mut DragonPhaseManager, ctx: &PhaseContext<'_>) {
    let dist_to_target = manager
        .current_state()
        .target_location
        .map_or(0.0, |target| target.distance_squared(ctx.position));

    if dist_to_target < PATH_TARGET_MIN_DIST_SQR
        || dist_to_target > PATH_TARGET_MAX_DIST_SQR
        || ctx.horizontal_collision
        || ctx.vertical_collision
    {
        find_new_holding_target(manager, ctx);
    }
}

/// Vanilla `DragonHoldingPatternPhase.findNewTarget`.
///
/// This is the arena's decision point: it may hand off to the landing approach,
/// start a strafe at a player, or pick the next ring node.
fn find_new_holding_target(manager: &mut DragonPhaseManager, ctx: &PhaseContext<'_>) {
    if manager.current_state().path_done() {
        // One time in `crystals + 3` the dragon decides to come in for a landing.
        if random_below(ctx, ctx.alive_crystals + 3) == 0 {
            manager.set_phase(DragonPhase::LandingApproach);
            return;
        }

        // One time in `crystals + 2`, or more often for a player standing close to
        // the podium, the dragon strafes instead of circling.
        if let Some(player) = ctx.arena_player.clone() {
            let egg = ctx.podium_location;
            // Vanilla `egg.distToCenterSqr(player.position()) / 512.0`.
            let dist_sqr = egg.dist_to_center_sqr(Entity::position(player.as_ref())) / 512.0;
            if random_below(ctx, dist_sqr as i32 + 2) == 0
                || random_below(ctx, ctx.alive_crystals + 2) == 0
            {
                strafe_player(manager, ctx, &(player as crate::entity::SharedEntity));
                return;
            }
        }
    }

    if manager.current_state().path_done() {
        pick_ring_path(manager, ctx, RingWalk::HoldingPattern);
    }

    navigate_to_next_path_node(manager, ctx);
}

/// Vanilla `strafePlayer(Player)`.
///
/// The phase switch runs `begin()` on the strafe state before the target lands,
/// matching `setPhase` then `setTarget` in Java.
fn strafe_player(
    manager: &mut DragonPhaseManager,
    ctx: &PhaseContext<'_>,
    player: &crate::entity::SharedEntity,
) {
    manager.set_phase(DragonPhase::StrafePlayer);
    set_strafe_target(manager, ctx, player);
}

/// Vanilla `DragonStrafePlayerPhase.setTarget`.
fn set_strafe_target(
    manager: &mut DragonPhaseManager,
    ctx: &PhaseContext<'_>,
    target: &crate::entity::SharedEntity,
) {
    let target_position = Entity::position(target.as_ref());
    manager.state_for(DragonPhase::StrafePlayer).attack_target = Some(Arc::downgrade(target));

    // Vanilla uses the *no-arg* `findClosestNode()` for the start of the strafe
    // path, which always searches from node 0 — only the end node is filtered by
    // the crystal count.
    let current_node = manager.find_closest_node(ctx.world, ctx.position);
    let target_node = find_closest_node_with_crystals(
        manager,
        ctx.world,
        target_position,
        ctx.alive_crystals,
    );

    // Vanilla aims a little above the target, scaled by the horizontal distance,
    // and caps the lift at ten blocks. The constants are Java's: `(double)0.4f`
    // (not `0.4`) and a plain `sqrt(x*x + z*z)` (not `hypot`, which is more
    // accurate and would round differently).
    let final_x = steel_math::fast_floor(target_position.x);
    let final_z = steel_math::fast_floor(target_position.z);
    let xd = f64::from(final_x) - ctx.position.x;
    let zd = f64::from(final_z) - ctx.position.z;
    let sd = (xd * xd + zd * zd).sqrt();
    let height_offset = (f64::from(0.4_f32) + sd / 80.0 - 1.0).min(10.0);
    let final_y = steel_math::fast_floor(target_position.y + height_offset);
    let final_node = DVec3::new(f64::from(final_x), f64::from(final_y), f64::from(final_z));

    let path = find_path(
        manager,
        ctx.world,
        ctx.alive_crystals,
        current_node,
        target_node,
        Some(final_node),
    );
    set_current_path(manager, path);
}

/// Vanilla `DragonStrafePlayerPhase.doServerTick`.
fn tick_strafe_player(
    manager: &mut DragonPhaseManager,
    ctx: &PhaseContext<'_>,
    actions: &mut Vec<PhaseAction>,
) {
    let Some(target) = manager
        .current_state()
        .attack_target
        .as_ref()
        .and_then(std::sync::Weak::upgrade)
    else {
        // Vanilla logs the missing target and falls back to the holding pattern.
        manager.set_phase(DragonPhase::HoldingPattern);
        return;
    };

    if manager.current_state().path_done() {
        // Vanilla `DragonStrafePlayerPhase.doServerTick`: while the path is spent,
        // the fly target is the player's raw position plus a distance-scaled
        // height offset. `(double)0.4f` is Java's widened float constant.
        let target_position = Entity::position(target.as_ref());
        let x_target_dist = target_position.x - ctx.position.x;
        let z_target_dist = target_position.z - ctx.position.z;
        let dist = (x_target_dist * x_target_dist + z_target_dist * z_target_dist).sqrt();
        let height_offset = (f64::from(0.4_f32) + dist / 80.0 - 1.0).min(10.0);
        manager.current_state_mut().target_location = Some(DVec3::new(
            target_position.x,
            target_position.y + height_offset,
            target_position.z,
        ));
    }

    let dist_to_target = manager
        .current_state()
        .target_location
        .map_or(0.0, |location| location.distance_squared(ctx.position));
    if dist_to_target < PATH_TARGET_MIN_DIST_SQR || dist_to_target > PATH_TARGET_MAX_DIST_SQR {
        if manager.current_state().path_done() {
            pick_ring_path(manager, ctx, RingWalk::StrafePlayer);
        }
        navigate_to_next_path_node(manager, ctx);
    }

    // Vanilla only charges the shot within 64 blocks and with line of sight.
    let target_position = Entity::position(target.as_ref());
    if target_position.distance_squared(ctx.position) >= 4096.0 {
        decay_fireball_charge(manager);
        return;
    }
    if !crate::entity::LivingEntity::has_line_of_sight(ctx.dragon, target.as_ref()) {
        decay_fireball_charge(manager);
        return;
    }

    let charge = {
        let state = manager.current_state_mut();
        state.fireball_charge += 1;
        state.fireball_charge
    };

    let aim = DVec3::new(
        target_position.x - ctx.position.x,
        0.0,
        target_position.z - ctx.position.z,
    )
    .normalize_or_zero();
    let dot = ctx.facing().dot(aim) as f32;
    // Vanilla adds half a degree before the window test.
    let angle_degs = dot.clamp(-1.0, 1.0).acos().to_degrees() + 0.5;

    if charge < FIREBALL_CHARGE_AMOUNT || !(0.0..10.0).contains(&angle_degs) {
        return;
    }

    // Vanilla fires from just behind the head, aimed at the target's chest.
    let head = manager
        .parts
        .position(DRAGON_PART_HEAD)
        .unwrap_or(ctx.position);
    let look = ctx.head_look(DragonPhase::StrafePlayer);
    // Cache it so a re-entered strafe aims the same way vanilla's live call would.
    manager.current_state_mut().head_look = Some(look);
    let origin = DVec3::new(head.x - look.x, head.y + 0.5, head.z - look.z);
    let eye_height = Entity::position(target.as_ref()).y + 0.5 - origin.y;
    let direction = DVec3::new(
        target_position.x - origin.x,
        eye_height,
        target_position.z - origin.z,
    )
    .normalize_or_zero();

    actions.push(PhaseAction::PlayShootSound);
    actions.push(PhaseAction::SpawnFireball { origin, direction });
    actions.push(PhaseAction::StrafeShotLevelEvent);

    manager.current_state_mut().fireball_charge = 0;
    if let Some(path) = manager.current_state_mut().path.as_mut() {
        while !path.is_done() {
            path.advance();
        }
    }
    manager.set_phase(DragonPhase::HoldingPattern);
}

/// Vanilla's `else if (this.fireballCharge > 0) --this.fireballCharge`.
fn decay_fireball_charge(manager: &mut DragonPhaseManager) {
    let state = manager.current_state_mut();
    if state.fireball_charge > 0 {
        state.fireball_charge -= 1;
    }
}

/// Which vanilla phase is walking the ring, because their node rules differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RingWalk {
    /// `DragonHoldingPatternPhase.findNewTarget`.
    HoldingPattern,
    /// `DragonStrafePlayerPhase.findNewTarget`.
    StrafePlayer,
}

/// Picks the next arena ring node and stores the A* path that reaches it.
///
/// The holding pattern and the strafe phase walk the ring with the same twelve
/// steps, but they differ in one comparison, reproduced here:
///
/// * holding pattern — `aliveCrystals() < 0`, which vanilla can never satisfy
///   (a crystal count never goes negative), so the outer ring stays in play;
/// * strafe phase — `aliveCrystals() <= 0`, which swaps to the inner ring once
///   the last crystal is gone.
///
/// Vanilla also starts both paths at the *no-arg* `findClosestNode()`, which
/// always searches from node `0` regardless of the crystal count, so
/// `current_node` is not filtered here.
fn pick_ring_path(
    manager: &mut DragonPhaseManager,
    ctx: &PhaseContext<'_>,
    walk: RingWalk,
) {
    let current_node = manager.find_closest_node(ctx.world, ctx.position);

    let mut target_node = current_node;
    let mut clockwise = manager.current_state().clockwise;
    if ctx.roll_eighth {
        clockwise = !clockwise;
        target_node += 6;
    }
    manager.current_state_mut().clockwise = clockwise;
    target_node = if clockwise {
        target_node + 1
    } else {
        target_node.wrapping_sub(1)
    };

    // Vanilla's `targetNodeIndex` is a signed `int`; `current_node` is `0..24`, so
    // the only way to go negative is the `- 1` above from node `0`.
    let wraps_below_zero = target_node == usize::MAX;
    let swaps_to_inner_ring = match walk {
        // `aliveCrystals() < 0` — unreachable in vanilla.
        RingWalk::HoldingPattern => false,
        // `aliveCrystals() <= 0`.
        RingWalk::StrafePlayer => ctx.alive_crystals <= 0,
    };

    if swaps_to_inner_ring {
        // `targetNodeIndex -= 12; &= 7; += 12;` on the signed value. With a
        // non-negative index this is `((i - 12) & 7) + 12`, i.e. `i & 7 + 12`.
        target_node = if wraps_below_zero {
            // `-1 - 12 = -13`, `-13 & 7 == 3`, so the node is `15`.
            15
        } else {
            ((target_node.wrapping_sub(12)) & 7) + 12
        };
    } else if wraps_below_zero {
        // The `else if ((targetNodeIndex %= 12) < 0)` arm: `-1 % 12 == -1`.
        target_node = 11;
    } else {
        target_node %= 12;
    }

    let path = find_path(
        manager,
        ctx.world,
        ctx.alive_crystals,
        current_node,
        target_node,
        None,
    );
    set_current_path(manager, path);
}

/// Stores `path` in the current phase.
fn set_current_path(manager: &mut DragonPhaseManager, path: Option<Path>) {
    manager.current_state_mut().path = path;
}

/// Vanilla `navigateToNextPathNode`.
fn navigate_to_next_path_node(manager: &mut DragonPhaseManager, ctx: &PhaseContext<'_>) {
    let offset = ctx.random;
    manager.current_state_mut().navigate_to_next_path_node(offset);
}

// ---------------------------------------------------------------------------
// Landing and takeoff
// ---------------------------------------------------------------------------

/// Vanilla `DragonLandingApproachPhase.doServerTick`.
fn tick_landing_approach(manager: &mut DragonPhaseManager, ctx: &PhaseContext<'_>) {
    let dist_to_target = manager
        .current_state()
        .target_location
        .map_or(0.0, |target| target.distance_squared(ctx.position));

    if dist_to_target < PATH_TARGET_MIN_DIST_SQR
        || dist_to_target > PATH_TARGET_MAX_DIST_SQR
        || ctx.horizontal_collision
        || ctx.vertical_collision
    {
        find_new_landing_target(manager, ctx);
    }

    // Vanilla hands off once the path is spent: `findNewTarget` ends with
    // `if (currentPath != null && currentPath.isDone()) setPhase(LANDING)`.
    if manager.current_state().path_done() {
        manager.set_phase(DragonPhase::Landing);
    }
}

/// Vanilla `DragonLandingApproachPhase.findNewTarget`.
fn find_new_landing_target(manager: &mut DragonPhaseManager, ctx: &PhaseContext<'_>) {
    if manager.current_state().path_done() {
        // Vanilla's no-arg `findClosestNode()` for the landing approach's start.
        let current_node = manager.find_closest_node(ctx.world, ctx.position);
        let egg = ctx.podium_location;

        let aim_position = match ctx.arena_player.as_ref() {
            Some(player) => {
                // Vanilla aims at the ring node opposite the player's bearing.
                let player_position = Entity::position(player.as_ref());
                let aim =
                    DVec3::new(player_position.x, 0.0, player_position.z).normalize_or_zero();
                DVec3::new(-aim.x * 40.0, 105.0, -aim.z * 40.0)
            }
            None => DVec3::new(40.0, f64::from(egg.y()), 0.0),
        };
        let target_node = find_closest_node_with_crystals(
            manager,
            ctx.world,
            aim_position,
            ctx.alive_crystals,
        );

        let final_node = DVec3::new(f64::from(egg.x()), f64::from(egg.y()), f64::from(egg.z()));
        let path = find_path(
            manager,
            ctx.world,
            ctx.alive_crystals,
            current_node,
            target_node,
            Some(final_node),
        );
        set_current_path(manager, path);
    }

    navigate_to_next_path_node(manager, ctx);
}

/// Vanilla `DragonLandingPhase.doServerTick`.
fn tick_landing(manager: &mut DragonPhaseManager, ctx: &PhaseContext<'_>) {
    if manager.current_state().target_location.is_none() {
        // Vanilla anchors the landing target to the bottom centre of the egg.
        let egg = ctx.podium_location;
        manager.current_state_mut().target_location = Some(DVec3::new(
            f64::from(egg.x()) + 0.5,
            f64::from(egg.y()),
            f64::from(egg.z()) + 0.5,
        ));
    }

    let arrive = manager
        .current_state()
        .target_location
        .is_some_and(|target| target.distance_squared(ctx.position) < LANDING_ARRIVE_DIST_SQR);
    if !arrive {
        return;
    }

    // Vanilla restarts the flame counter and switches straight to scanning; the
    // breath cloud is only created by `SITTING_FLAMING` itself at tick 10.
    manager
        .state_for(DragonPhase::SittingFlaming)
        .reset_flame_count();
    manager.set_phase(DragonPhase::SittingScanning);
}

/// Vanilla `DragonTakeoffPhase.doServerTick`.
fn tick_takeoff(manager: &mut DragonPhaseManager, ctx: &PhaseContext<'_>) {
    if manager.current_state().first_tick || manager.current_state().path.is_none() {
        manager.current_state_mut().first_tick = false;
        find_new_takeoff_target(manager, ctx);
        return;
    }

    if ctx.distance_to_podium() > 10.0 {
        manager.set_phase(DragonPhase::HoldingPattern);
    }
}

/// Vanilla `DragonTakeoffPhase.findNewTarget`.
fn find_new_takeoff_target(manager: &mut DragonPhaseManager, ctx: &PhaseContext<'_>) {
    // Vanilla's no-arg `findClosestNode()`: always searches from node 0.
    let current_node = manager.find_closest_node(ctx.world, ctx.position);

    // Vanilla looks backward from the head to pick the takeoff ring node.
    let look = ctx.head_look(DragonPhase::Takeoff);
    let aim_position = DVec3::new(-look.x * 40.0, 105.0, -look.z * 40.0);
    let mut target_node = find_closest_node_with_crystals(
        manager,
        ctx.world,
        aim_position,
        ctx.alive_crystals,
    );

    if ctx.alive_crystals <= 0 {
        target_node = (target_node.wrapping_sub(12) & 7) + 12;
    } else {
        target_node %= 12;
    }

    let path = find_path(
        manager,
        ctx.world,
        ctx.alive_crystals,
        current_node,
        target_node,
        None,
    );
    set_current_path(manager, path);

    if !manager.current_state().path_done() {
        navigate_to_next_path_node(manager, ctx);
    }
}

// ---------------------------------------------------------------------------
// Sitting phases
// ---------------------------------------------------------------------------

/// Vanilla `DragonSittingFlamingPhase.doServerTick`.
fn tick_sitting_flaming(
    manager: &mut DragonPhaseManager,
    ctx: &PhaseContext<'_>,
    actions: &mut Vec<PhaseAction>,
) {
    let ticks = {
        let state = manager.current_state_mut();
        state.flame_ticks += 1;
        state.flame_ticks
    };

    if ticks >= FLAME_DURATION {
        let next = if manager.current_state().flame_count >= SITTING_FLAME_ATTACKS_COUNT {
            DragonPhase::Takeoff
        } else {
            DragonPhase::SittingScanning
        };
        manager.set_phase(next);
        return;
    }

    if ticks == WARMUP_TIME {
        let head = manager
            .parts
            .position(DRAGON_PART_HEAD)
            .unwrap_or(ctx.position);
        let look = DVec3::new(head.x - ctx.position.x, 0.0, head.z - ctx.position.z)
            .normalize_or_zero();
        actions.push(PhaseAction::SpawnBreathCloud {
            x: head.x + look.x * f64::from(SITTING_FLAME_RADIUS) / 2.0,
            // Vanilla uses `head.getY(0.5)`: half the part's height above its feet.
            // Vanilla `head.getY(0.5)` = `y + height * 0.5`, widened afterwards.
            head_y: head.y + f64::from(DRAGON_PART_HEAD_HEIGHT) * 0.5,
            z: head.z + look.z * f64::from(SITTING_FLAME_RADIUS) / 2.0,
        });
    }
}

/// Vanilla `DragonSittingScanningPhase.doServerTick`.
fn tick_sitting_scanning(
    manager: &mut DragonPhaseManager,
    ctx: &PhaseContext<'_>,
    actions: &mut Vec<PhaseAction>,
) {
    let scanning_time = {
        let state = manager.current_state_mut();
        state.scanning_time += 1;
        state.scanning_time
    };

    if let Some(target) = ctx.scanned_player.clone() {
        if scanning_time > 25 {
            manager.set_phase(DragonPhase::SittingAttacking);
            return;
        }

        // Turn the dragon toward the player when it is outside the 10-degree cone.
        let target_position = Entity::position(target.as_ref());
        let aim = DVec3::new(
            target_position.x - ctx.position.x,
            0.0,
            target_position.z - ctx.position.z,
        )
        .normalize_or_zero();
        let angle = (ctx.facing().dot(aim) as f32).clamp(-1.0, 1.0).acos().to_degrees() + 0.5;

        if !(0.0..=10.0).contains(&angle) {
            let head = manager
                .parts
                .position(DRAGON_PART_HEAD)
                .unwrap_or(ctx.position);
            let x_dist = target_position.x - head.x;
            let z_dist = target_position.z - head.z;
            let target_yaw =
                180.0 - f64::from(steel_math::trig::atan2(x_dist, z_dist)) * 57.295_776;
            let y_rot_delta =
                wrap_degrees((target_yaw - f64::from(ctx.yaw)) as f32).clamp(-100.0, 100.0);

            // Vanilla: `rotSpeed = sqrt(x*x + z*z) + 1`, `dist = min(rotSpeed, 40)`,
            // then `yRotA += yRotDelta * (0.7 / dist / rotSpeed)` — note the gain
            // divides by the clamped *and* the unclamped value.
            let rot_speed = (x_dist * x_dist + z_dist * z_dist).sqrt() as f32 + 1.0;
            let dist = rot_speed.min(40.0);
            let y_rot_a = manager.y_rot_a * 0.8 + y_rot_delta * (0.7 / dist / rot_speed);
            manager.y_rot_a = y_rot_a;
            actions.push(PhaseAction::SetYaw(wrap_degrees(ctx.yaw + y_rot_a)));
        }
        return;
    }

    if scanning_time < SITTING_SCANNING_IDLE_TICKS {
        return;
    }

    // Vanilla gives up scanning: take off, then dive at a nearby player if there
    // is one within the charge range. The order matters — `begin()` clears the
    // charge target, so `setTarget` has to come after `setPhase`, exactly as in
    // `DragonSittingScanningPhase`.
    manager.set_phase(DragonPhase::Takeoff);
    if let Some(target) = ctx.charge_player.clone() {
        let target_position = Entity::position(target.as_ref());
        manager.set_phase(DragonPhase::ChargingPlayer);
        manager.state_for(DragonPhase::ChargingPlayer).target_location = Some(target_position);
    }
}

/// Vanilla `DragonSittingAttackingPhase.doServerTick`.
fn tick_sitting_attacking(manager: &mut DragonPhaseManager, _actions: &mut Vec<PhaseAction>) {
    let ticks = {
        let state = manager.current_state_mut();
        state.attacking_ticks += 1;
        state.attacking_ticks
    };
    if ticks >= ROAR_DURATION {
        manager.set_phase(DragonPhase::SittingFlaming);
    }
}

// Vanilla's roar is played by `DragonSittingAttackingPhase.doClientTick`, which
// is client-only; the server tick only advances the 40-tick counter.

// ---------------------------------------------------------------------------
// Charge, death, hover
// ---------------------------------------------------------------------------

/// Vanilla `DragonChargePlayerPhase.doServerTick`.
fn tick_charging_player(manager: &mut DragonPhaseManager, ctx: &PhaseContext<'_>) {
    let Some(target) = manager.current_state().target_location else {
        manager.set_phase(DragonPhase::HoldingPattern);
        return;
    };

    let time_since_charge = manager.current_state().time_since_charge;
    if time_since_charge > 0 {
        manager.current_state_mut().time_since_charge += 1;
        if time_since_charge + 1 >= CHARGE_RECOVERY_TIME {
            manager.set_phase(DragonPhase::HoldingPattern);
            return;
        }
    }

    let dist_to_target = target.distance_squared(ctx.position);
    if dist_to_target < PATH_TARGET_MIN_DIST_SQR
        || dist_to_target > PATH_TARGET_MAX_DIST_SQR
        || ctx.horizontal_collision
        || ctx.vertical_collision
    {
        manager.current_state_mut().time_since_charge += 1;
    }
}

/// Vanilla `DragonDeathPhase.doServerTick`.
fn tick_dying(
    manager: &mut DragonPhaseManager,
    ctx: &PhaseContext<'_>,
    actions: &mut Vec<PhaseAction>,
) {
    manager.current_state_mut().time += 1;

    if manager.current_state().target_location.is_none() {
        // Vanilla uses `MOTION_BLOCKING` (leaves included) for the death dive.
        let egg = ctx.world.heightmap_pos(
            crate::chunk::heightmap::HeightmapType::MotionBlocking,
            ctx.podium_location,
        );
        manager.current_state_mut().target_location = Some(DVec3::new(
            f64::from(egg.x()) + 0.5,
            f64::from(egg.y()),
            f64::from(egg.z()) + 0.5,
        ));
    }

    let Some(target) = manager.current_state().target_location else {
        return;
    };
    let dist_to_target = target.distance_squared(ctx.position);
    if dist_to_target < PATH_TARGET_MIN_DIST_SQR
        || dist_to_target > PATH_TARGET_MAX_DIST_SQR
        || ctx.horizontal_collision
        || ctx.vertical_collision
    {
        actions.push(PhaseAction::SetHealth(0.0));
    } else {
        actions.push(PhaseAction::SetHealth(1.0));
    }
}

/// Vanilla `DragonHoverPhase.doServerTick`.
fn tick_hovering(manager: &mut DragonPhaseManager, ctx: &PhaseContext<'_>) {
    if manager.current_state().target_location.is_none() {
        manager.current_state_mut().target_location = Some(ctx.position);
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Vanilla `RandomSource.nextInt(bound)` for a positive bound.
///
/// Vanilla treats a non-positive bound as "any int"; every call site here derives
/// the bound from crystal counts, so a non-positive bound is clamped to `1` and
/// always returns `0`.
///
/// The rollout advances `ctx`'s per-step random state, so two rolls in one tick are
/// independent — deriving both from the single `nextFloat` would make them equal.
#[expect(
    clippy::cast_possible_wrap,
    reason = "the high 32 bits are reinterpreted as the signed result, as in Java"
)]
fn random_below(ctx: &PhaseContext<'_>, bound: i32) -> i32 {
    if bound <= 1 {
        return 0;
    }
    // SplitMix64: one step per draw, deterministic within a step and independent
    // between draws.
    let mut state = ctx.random_state.get().wrapping_add(0x9E37_79B9_7F4A_7C15);
    ctx.random_state.set(state);
    state = (state ^ (state >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    state ^= state >> 31;

    let value = (state >> 32) as u32;
    (i64::from(value) * i64::from(bound) >> 32) as i32
}

/// Vanilla `EnderDragon.onCrystalDestroyed`.
///
/// Runs when an end crystal in the arena is destroyed outside of a respawn: the
/// dragon turns to strafe whoever broke it. Vanilla only reacts while the dragon
/// is alive and holds off during the death animation, so the dying phase is left
/// untouched.
pub fn on_crystal_destroyed(
    dragon: &EnderDragonEntity,
    player: Option<&crate::entity::SharedEntity>,
) {
    // Vanilla `if (this.isDeadOrDying()) return;` -- a dying dragon ignores the
    // crystal, its phase is already `Dying` and must not be replaced.
    if dragon.phase() == DragonPhase::Dying {
        return;
    }
    let Some(player) = player else {
        // Vanilla falls back to `HoldingPattern` when nobody can be blamed.
        dragon.set_phase(DragonPhase::HoldingPattern);
        return;
    };
    // Vanilla `setPhase(StrafePlayer)` then `setTarget(player)`. The manager is
    // already under its own lock inside the phase tick, so the switch is made
    // here exactly like `EnderDragonEntity::set_phase` does.
    let mut manager = dragon.phases.lock();
    manager.set_phase(DragonPhase::StrafePlayer);
    manager.state_for(DragonPhase::StrafePlayer).attack_target = Some(Arc::downgrade(player));
    drop(manager);
    dragon.sync_phase_data();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_podium_distance_is_the_largest_axis_delta() {
        // `BlockPos.closerToCenterThan` compares per-axis deltas, so the takeoff
        // hand-off must not use a euclidean distance.
        let podium = BlockPos::new(5, 100, 0);
        let dx = f64::from(podium.x()) - 3.0;
        let dy = f64::from(podium.y()) - 100.0;
        let dz = f64::from(podium.z()) - 0.0;
        assert!((dx.abs().max(dy.abs()).max(dz.abs()) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn a_roll_from_the_step_float_stays_inside_the_bound() {
        let ctx_bound = 7;
        for step in 0..100 {
            let random = step as f32 / 100.0;
            let roll = (random * ctx_bound as f32) as i32 % ctx_bound;
            assert!((0..ctx_bound).contains(&roll), "roll {roll} left the bound");
        }
    }

    #[test]
    fn every_sitting_phase_reports_sitting_and_the_rest_do_not() {
        for phase in super::super::phases::ALL_DRAGON_PHASES {
            let expected = matches!(
                phase,
                DragonPhase::SittingFlaming
                    | DragonPhase::SittingScanning
                    | DragonPhase::SittingAttacking
                    | DragonPhase::Hovering
            );
            assert_eq!(phase.is_sitting(), expected, "{phase:?}");
        }
    }

    #[test]
    fn the_head_look_vector_is_unit_length_for_every_phase() {
        // The look vector feeds the fireball origin, so a non-unit vector would
        // move the spawn point.
        for phase in super::super::phases::ALL_DRAGON_PHASES {
            for yaw in [-180.0_f32, -90.0, 0.0, 45.0, 179.0] {
                let pitch: f32 = 12.0;
                let pitch_rad = match phase {
                    DragonPhase::Landing | DragonPhase::Takeoff => -22.5_f64.to_radians(),
                    _ if phase.is_sitting() => -45.0_f64.to_radians(),
                    _ => f64::from(pitch.to_radians()),
                };
                let yaw_rad = f64::from(-yaw.to_radians());
                let vector = DVec3::new(
                    f64::from(steel_math::trig::sin(yaw_rad) * steel_math::trig::cos(pitch_rad)),
                    f64::from(-steel_math::trig::sin(pitch_rad)),
                    f64::from(steel_math::trig::cos(yaw_rad) * steel_math::trig::cos(pitch_rad)),
                )
                .normalize_or_zero();
                assert!((vector.length() - 1.0).abs() < 1.0e-6, "{phase:?} {yaw}");
            }
        }
    }
}
