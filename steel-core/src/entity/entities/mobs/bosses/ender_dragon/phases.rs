//! Vanilla `net.minecraft.world.entity.boss.enderdragon.phases`, part one: the
//! state machine itself.
//!
//! The dragon is driven by an eleven-state machine: [`DragonPhase`] names the
//! states and [`DragonPhaseManager`] owns the current one plus each phase's
//! scratch state.
//!
//! Vanilla gives every phase its own class under
//! `net.minecraft.world.entity.boss.enderdragon.phases`, and each class holds its
//! own mutable fields. Rust trait objects cannot carry those fields when the
//! manager hands out `&self`, so Steel keeps all eleven states' scratch data in
//! one [`DragonPhaseState`] array inside the manager and dispatches the per-tick
//! behaviour by phase id.
//!
//! This module owns everything that does **not** run a tick:
//!
//! * [`DragonPhase`] — the ids, `isSitting` and `getFlySpeed`.
//! * [`DragonPhaseState`] — the per-phase fields, `begin` and
//!   `navigateToNextPathNode`.
//! * [`DragonPhaseManager`] — the current phase, the arena waypoints and the
//!   [`find_path`] A* search.
//! * The dragon-only movement state vanilla keeps on the entity: the flap phase,
//!   the smoothed yaw velocity, the wall flag, the flight history and the crystal
//!   the dragon heals from.
//!
//! [`super::phase_ticks`] holds the eleven `doServerTick` bodies. They are
//! deliberately separate and lock-free, because the manager is behind a mutex and
//! a tick that re-entered it would deadlock.

use std::f32::consts::PI;
use std::sync::Arc;

use glam::DVec3;
use steel_utils::BlockPos;

use super::EnderDragonEntity;
use super::flight_history::DragonFlightHistory;
use super::part::DragonParts;
use crate::entity::ai::node::Node;
use crate::entity::ai::path::Path;
use crate::entity::{Entity, WeakEntity};
use crate::world::World;

/// Vanilla `EnderDragonPhase` ids, in registry order.
///
/// The discriminants are the values written to `DATA_PHASE`, so they must not be
/// renumbered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum DragonPhase {
    /// `HoldingPattern` (`0`): circle the arena and pick the next move.
    HoldingPattern = 0,
    /// `StrafePlayer` (`1`): line up and fire a dragon fireball.
    StrafePlayer = 1,
    /// `LandingApproach` (`2`): descend toward the exit portal.
    LandingApproach = 2,
    /// `Landing` (`3`): settle onto the podium.
    Landing = 3,
    /// `Takeoff` (`4`): climb back out after sitting.
    Takeoff = 4,
    /// `SittingFlaming` (`5`): breathe a dragon-breath cloud.
    SittingFlaming = 5,
    /// `SittingScanning` (`6`): look for a player to attack.
    SittingScanning = 6,
    /// `SittingAttacking` (`7`): roar, then flame again.
    SittingAttacking = 7,
    /// `ChargingPlayer` (`8`): dive at a player.
    ChargingPlayer = 8,
    /// `Dying` (`9`): the death dive.
    Dying = 9,
    /// `Hovering` (`10`): airborne, stationary, treated as sitting.
    Hovering = 10,
}

/// Every phase, in vanilla registry order.
pub const ALL_DRAGON_PHASES: [DragonPhase; 11] = [
    DragonPhase::HoldingPattern,
    DragonPhase::StrafePlayer,
    DragonPhase::LandingApproach,
    DragonPhase::Landing,
    DragonPhase::Takeoff,
    DragonPhase::SittingFlaming,
    DragonPhase::SittingScanning,
    DragonPhase::SittingAttacking,
    DragonPhase::ChargingPlayer,
    DragonPhase::Dying,
    DragonPhase::Hovering,
];

impl DragonPhase {
    /// Vanilla `EnderDragonPhase.getCount`.
    pub const COUNT: usize = ALL_DRAGON_PHASES.len();

    /// Vanilla `EnderDragonPhase.getId`.
    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }

    /// Vanilla `EnderDragonPhase.getById`.
    ///
    /// An id outside the registry falls back to `HOLDING_PATTERN`, which is what
    /// keeps a datapack-written `DragonPhase` from breaking the dragon.
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::StrafePlayer,
            2 => Self::LandingApproach,
            3 => Self::Landing,
            4 => Self::Takeoff,
            5 => Self::SittingFlaming,
            6 => Self::SittingScanning,
            7 => Self::SittingAttacking,
            8 => Self::ChargingPlayer,
            9 => Self::Dying,
            10 => Self::Hovering,
            _ => Self::HoldingPattern,
        }
    }

    /// Vanilla `EnderDragonPhase`'s debug name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::HoldingPattern => "HoldingPattern",
            Self::StrafePlayer => "StrafePlayer",
            Self::LandingApproach => "LandingApproach",
            Self::Landing => "Landing",
            Self::Takeoff => "Takeoff",
            Self::SittingFlaming => "SittingFlaming",
            Self::SittingScanning => "SittingScanning",
            Self::SittingAttacking => "SittingAttacking",
            Self::ChargingPlayer => "ChargingPlayer",
            Self::Dying => "Dying",
            Self::Hovering => "Hovering",
        }
    }

    /// Vanilla `DragonPhaseInstance.isSitting`.
    ///
    /// The three sitting states and `HOVERING` report `true`.
    #[must_use]
    pub const fn is_sitting(self) -> bool {
        matches!(
            self,
            Self::SittingFlaming | Self::SittingScanning | Self::SittingAttacking | Self::Hovering
        )
    }

    /// Vanilla `DragonPhaseInstance.getFlySpeed`.
    #[must_use]
    pub const fn fly_speed(self) -> f32 {
        match self {
            Self::Landing => 1.5,
            Self::ChargingPlayer | Self::Dying => 3.0,
            Self::Hovering => 1.0,
            _ => 0.6,
        }
    }

    /// Index into the manager's per-phase state array.
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Vanilla `DragonLandingPhase.getTurnSpeed`.
///
/// The landing phase overrides the shared turn rate with `min(rotSpeed, 40) /
/// rotSpeed`, i.e. exactly `1.0` below 40 blocks per tick of horizontal speed.
#[must_use]
pub fn turn_speed(phase: DragonPhase, dragon: &EnderDragonEntity) -> f32 {
    let velocity = Entity::velocity(dragon);
    // Vanilla `Vec3.horizontalDistance()` = `sqrt(x*x + z*z)`.
    let rot_speed = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt() as f32 + 1.0;
    let dist = rot_speed.min(40.0);
    match phase {
        DragonPhase::Landing => dist / rot_speed,
        _ => 0.7 / dist / rot_speed,
    }
}

/// Wraps an angle into `[-180, 180)`, matching `Mth.wrapDegrees`.
#[must_use]
pub fn wrap_degrees(degrees: f32) -> f32 {
    steel_math::wrap_degrees(degrees)
}

#[derive(Debug, Clone, Default)]
pub struct DragonPhaseState {
    /// Vanilla `currentPath` on the path-driven phases.
    pub path: Option<Path>,
    /// Vanilla `targetLocation`: the point the flight code steers toward.
    pub target_location: Option<DVec3>,
    /// Vanilla `clockwise` / `holdingPatternClockwise`.
    pub clockwise: bool,
    /// Vanilla `fireballCharge`.
    pub fireball_charge: i32,
    /// Vanilla `flameTicks`.
    pub flame_ticks: i32,
    /// Vanilla `DragonSittingFlamingPhase.flame`: the live breath cloud, so
    /// `end()` can discard it when the phase is left early.
    pub flame_cloud: Option<WeakEntity>,
    /// Vanilla `flameCount`.
    pub flame_count: i32,
    /// Vanilla `scanningTime`.
    pub scanning_time: i32,
    /// Vanilla `attackingTicks`.
    pub attacking_ticks: i32,
    /// Vanilla `timeSinceCharge`.
    pub time_since_charge: i32,
    /// Vanilla `time` on the dying phase.
    pub time: i32,
    /// Vanilla `firstTick` on the takeoff phase.
    pub first_tick: bool,
    /// Vanilla `attackTarget`, held weakly so a phase never keeps a player alive.
    pub attack_target: Option<WeakEntity>,
    /// The head look vector `DragonStrafePlayerPhase` aims the fireball with.
    ///
    /// Vanilla calls `getHeadLookVector(1.0f)` live; Steel caches it when the
    /// phase is entered, because the tick no longer holds the dragon.
    pub head_look: Option<DVec3>,
}

impl DragonPhaseState {
    /// Resets the fields vanilla's `begin()` clears for `phase`.
    pub fn begin(&mut self, phase: DragonPhase) {
        self.path = None;
        self.target_location = None;

        match phase {
            DragonPhase::HoldingPattern
            | DragonPhase::LandingApproach
            | DragonPhase::Landing
            | DragonPhase::Hovering => {}
            DragonPhase::StrafePlayer => {
                self.fireball_charge = 0;
                self.attack_target = None;
                self.head_look = None;
            }
            DragonPhase::Takeoff => {
                self.first_tick = true;
            }
            DragonPhase::SittingFlaming => {
                // Vanilla's `begin()` restarts the timer and counts one attack.
                // The previous cloud is cleared by `end()`, not here.
                self.flame_ticks = 0;
                self.flame_cloud = None;
                self.flame_count += 1;
            }
            DragonPhase::SittingScanning => {
                self.scanning_time = 0;
            }
            DragonPhase::SittingAttacking => {
                self.attacking_ticks = 0;
            }
            DragonPhase::ChargingPlayer => {
                self.time_since_charge = 0;
            }
            DragonPhase::Dying => {
                self.time = 0;
            }
        }
    }

    /// Vanilla `DragonSittingFlamingPhase.resetFlameCount`.
    pub const fn reset_flame_count(&mut self) {
        self.flame_count = 0;
    }

    /// Vanilla `DragonSittingFlamingPhase.end`: discard the live breath cloud.
    ///
    /// Returns the cloud that has to be removed, so the caller can do it outside
    /// the manager lock.
    pub fn take_flame_cloud(&mut self) -> Option<WeakEntity> {
        self.flame_cloud.take()
    }

    /// Vanilla `Path.isDone` on a possibly absent path.
    #[must_use]
    pub fn path_done(&self) -> bool {
        self.path.as_ref().is_none_or(Path::is_done)
    }

    /// Vanilla `navigateToNextPathNode`: turns the next path node into a fly
    /// target with a random height offset above the node.
    ///
    /// The holding-pattern, strafe and landing-approach phases each inline these
    /// same lines in vanilla; they share them here.
    pub fn navigate_to_next_path_node(&mut self, random_offset: f32) {
        let Some(path) = self.path.as_mut() else {
            return;
        };
        if path.is_done() {
            return;
        }
        let Some(current) = path.next_node_pos() else {
            return;
        };
        path.advance();

        let x_target = f64::from(current.x());
        let z_target = f64::from(current.z());
        // Vanilla re-rolls until the sampled height is at least the node's own Y:
        // `while ((yTarget = (float)current.getY() + nextFloat() * 20.0f) < current.getY())`.
        // `nextFloat()` is in `0.0..1.0`, so the sum is never below the node's Y and
        // the loop body never runs; one draw is enough. The addition is done in
        // `f32` first, exactly like the vanilla expression, and only then widened.
        let y_target = f64::from(current.y() as f32 + random_offset * 20.0);
        self.target_location = Some(DVec3::new(x_target, y_target, z_target));
    }
}

/// Vanilla `EnderDragonPhaseManager`.
///
/// Also holds the movement state vanilla keeps on the dragon but that only the
/// phase machinery touches: the flap phase, the smoothed yaw velocity, the
/// in-wall flag and the cached arena waypoints.
#[derive(Debug)]
pub struct DragonPhaseManager {
    current: DragonPhase,
    states: Vec<DragonPhaseState>,
    /// Vanilla `EnderDragon.nodes`, built lazily on the first lookup.
    nodes: Option<[DVec3; NODE_COUNT]>,
    /// Vanilla `EnderDragon.subEntities`.
    pub parts: DragonParts,
    /// Vanilla `flapTime` / `oFlapTime`.
    pub flap_time: f32,
    pub old_flap_time: f32,
    /// Vanilla `yRotA`: the smoothed yaw velocity the flight code applies.
    pub y_rot_a: f32,
    /// Vanilla `inWall`: set by `checkWalls`, halves the flap speed and slows the
    /// next move to 80%.
    pub in_wall: bool,
    /// Vanilla `sittingDamageReceived`: damage taken while sitting, which forces
    /// a takeoff at a quarter of the max health.
    pub sitting_damage_received: f32,
    /// Vanilla `dragonDeathTime`.
    pub death_time: i32,
    /// Vanilla `podiumLocation`, resolved from the heightmap on first use.
    pub podium_location: Option<BlockPos>,
    /// Vanilla `flightHistory`: the dragon's recent height and yaw samples.
    pub flight_history: DragonFlightHistory,
    /// Vanilla `nearestCrystal`: the crystal currently healing the dragon.
    pub nearest_crystal: Option<WeakEntity>,
    /// Entities a phase's `end()` discarded, drained by the dragon after the
    /// phase tick so the removal happens outside the manager lock.
    pending_discards: Vec<WeakEntity>,
}

/// Vanilla `EnderDragon`'s 24 arena waypoints.
pub const NODE_COUNT: usize = 24;

/// Vanilla `EnderDragon.nodeAdjacency`: the arena graph as one bitmask per node.
///
/// Bit `i` set in row `n` means nodes `n` and `i` are connected. Vanilla writes
/// the literals as decimal, except rows `21` and `23` which it writes in hex;
/// they are reproduced verbatim so pathing matches.
pub const NODE_ADJACENCY: [u32; NODE_COUNT] = [
    6_146, 8_197, 8_202, 16_404, 32_808, 32_848, 65_696, 131_392, 131_712, 263_424, 526_848,
    525_313, 1_581_057, 3_166_214, 2_138_120, 6_373_424, 4_358_208, 12_910_976, 9_044_480,
    9_706_496, 15_216_640, 0xD0_E000, 11_763_712, 0x7E_0000,
];

impl DragonPhaseManager {
    /// Vanilla `new EnderDragonPhaseManager(dragon)`: starts in `HOVERING`.
    #[must_use]
    pub fn new(spawn_position: DVec3) -> Self {
        let mut manager = Self {
            current: DragonPhase::Hovering,
            states: (0..DragonPhase::COUNT)
                .map(|_| DragonPhaseState::default())
                .collect(),
            nodes: None,
            parts: DragonParts::at(spawn_position),
            flap_time: 0.0,
            old_flap_time: 0.0,
            y_rot_a: 0.0,
            in_wall: false,
            sitting_damage_received: 0.0,
            death_time: 0,
            podium_location: None,
            flight_history: DragonFlightHistory::new(),
            nearest_crystal: None,
            pending_discards: Vec::new(),
        };
        // Vanilla's constructor calls `setPhase(HOVERING)`, which runs `begin()`.
        manager.states[DragonPhase::Hovering.index()].begin(DragonPhase::Hovering);
        manager
    }

    /// Vanilla `EnderDragonPhaseManager.getCurrentPhase`.
    #[must_use]
    pub const fn current(&self) -> DragonPhase {
        self.current
    }

    /// Returns the current phase's scratch state.
    #[must_use]
    pub fn current_state(&self) -> &DragonPhaseState {
        &self.states[self.current.index()]
    }

    /// Returns the current phase's scratch state mutably.
    pub fn current_state_mut(&mut self) -> &mut DragonPhaseState {
        let index = self.current.index();
        &mut self.states[index]
    }

    /// Returns one phase's scratch state mutably, like vanilla `getPhase(...)`.
    pub fn state_for(&mut self, phase: DragonPhase) -> &mut DragonPhaseState {
        let index = phase.index();
        &mut self.states[index]
    }

    /// Vanilla `EnderDragonPhaseManager.setPhase`.
    ///
    /// Returns `true` when the phase actually changed, so the caller writes the
    /// new id into `DATA_PHASE` exactly once.
    pub fn set_phase(&mut self, phase: DragonPhase) -> bool {
        if phase == self.current {
            return false;
        }
        // Vanilla `EnderDragonPhaseManager.setPhase` runs the old phase's
        // `end()` before the new phase's `begin()`.
        let previous = self.current;
        self.parts_end(previous);
        self.current = phase;
        self.states[phase.index()].begin(phase);
        true
    }

    /// Vanilla `EnderDragonPhaseInstance.end`, for the phases that have one.
    fn parts_end(&mut self, phase: DragonPhase) {
        if phase == DragonPhase::SittingFlaming
            && let Some(cloud) = self.states[phase.index()].take_flame_cloud()
        {
            self.pending_discards.push(cloud);
        }
    }

    /// Takes the entities queued by [`Self::parts_end`].
    pub fn drain_phase_discards(&mut self) -> Vec<WeakEntity> {
        std::mem::take(&mut self.pending_discards)
    }

    /// Adopts the synced `DATA_PHASE` if it disagrees with the current phase.
    ///
    /// Vanilla reacts to `DATA_PHASE` in `onSyncedDataUpdated`; Steel has no
    /// per-field sync callback, so the dragon calls this at the top of its AI
    /// step. Returns `true` when the phase changed.
    pub fn sync_phase(&mut self, synced_phase: i32) -> bool {
        self.set_phase(DragonPhase::from_id(synced_phase))
    }

    /// Returns whether the dragon's wings are mid-beat, mirroring `isFlapping`.
    #[must_use]
    pub fn is_flapping(&self) -> bool {
        let flap = steel_math::trig::cos(self.flap_time * (PI * 2.0));
        let old_flap = steel_math::trig::cos(self.old_flap_time * (PI * 2.0));
        old_flap <= -0.3 && flap >= -0.3
    }

    /// Vanilla `flightHistory.record(y, yRot)`.
    pub fn record_flight(&mut self, y: f64, yaw: f32) {
        self.flight_history.record(y, yaw);
    }

    /// Restores the saved dragon fields, like `readAdditionalSaveData`.
    pub fn restore(&mut self, phase_id: i32, death_time: i32, sitting_damage_received: f32) {
        self.set_phase(DragonPhase::from_id(phase_id));
        self.death_time = death_time;
        self.sitting_damage_received = sitting_damage_received;
    }

    /// Records the arena waypoints once, from the world's terrain heights.
    pub fn ensure_nodes(&mut self, world: &World) {
        if self.nodes.is_some() {
            return;
        }
        self.nodes = Some(std::array::from_fn(|index| {
            let (x, z, y_adjustment) = node_polar_position(index);
            let height = world.level_height_at(
                crate::chunk::heightmap::HeightmapType::MotionBlockingNoLeaves,
                x,
                z,
            );
            DVec3::new(
                f64::from(x),
                f64::from(height.max(73) + y_adjustment),
                f64::from(z),
            )
        }));
    }

    /// Vanilla `EnderDragon.findClosestNode()`.
    pub fn find_closest_node(&mut self, world: &World, position: DVec3) -> usize {
        self.find_closest_node_from(world, position, 0)
    }

    /// Vanilla `EnderDragon.findClosestNode(tX, tY, tZ)`.
    ///
    /// `alive_crystals` decides whether the outer ring may be used: vanilla hides
    /// it once the dragon has no crystals so it stays near the podium.
    pub fn find_closest_node_with_crystals(
        &mut self,
        world: &World,
        position: DVec3,
        alive_crystals: i32,
    ) -> usize {
        let start = if alive_crystals == 0 { 12 } else { 0 };
        self.find_closest_node_from(world, position, start)
    }

    fn find_closest_node_from(&mut self, world: &World, position: DVec3, start: usize) -> usize {
        self.ensure_nodes(world);
        let nodes = self
            .nodes
            .as_ref()
            .expect("ensure_nodes always fills the node array");
        // Vanilla compares `Node.distanceToSqr` from the floored dragon position and
        // keeps the *first* strictly-closer waypoint, so ties keep the lower index.
        // The waypoints are already integral, so they are compared as they stand
        // rather than being floored again on every iteration.
        let target = node_from_position(position);
        let mut closest_index = 0;
        let mut closest_dist = 10_000.0_f32;
        for (index, node) in nodes.iter().enumerate().skip(start) {
            // Fully qualified: `Entity::distance_to_sqr` would otherwise win.
            let distance = Node::distance_to_sqr(node, &target);
            if distance < closest_dist {
                closest_dist = distance;
                closest_index = index;
            }
        }
        closest_index
    }

    /// Vanilla `EnderDragon.findPath`: the arena A* over [`NODE_ADJACENCY`].
    ///
    /// `alive_crystals` selects vanilla's `minimumNodeIndex`: with no crystals left
    /// the search may not expand from the inner rings (`0..12`), which keeps the
    /// dragon close enough to the podium to be landed.
    ///
    /// `final_node` is vanilla's synthetic `finalNode`: an extra node that is never
    /// searched but becomes the path's last element and carries the goal.
    pub fn find_path(
        &mut self,
        world: &World,
        alive_crystals: i32,
        start_index: usize,
        end_index: usize,
        final_node: Option<DVec3>,
    ) -> Option<Path> {
        self.ensure_nodes(world);
        // `[DVec3; NODE_COUNT]` is `Copy`, so dereferencing gives an owned array to
        // `array::map` (`&[T; N]` has no `map` of its own).
        let arena = (*self
            .nodes
            .as_ref()
            .expect("ensure_nodes always fills the node array"))
        .map(node_from_position);

        // Slot `FINAL_SLOT` is vanilla's `finalNode`; it exists only so the search
        // state can address it like any other node.
        // `Node` is `Clone` but not `Copy`, so the arena is cloned into the slots.
        let mut positions: [Node; SEARCH_SLOTS] =
            std::array::from_fn(|slot| match arena.get(slot) {
                Some(node) => node.clone(),
                None => Node::new(0, 0, 0),
            });
        let final_slot = final_node.map(|position| {
            positions[FINAL_SLOT] = node_from_position(position);
            FINAL_SLOT
        });

        let from = start_index;
        // Vanilla searches toward the ring node the whole time; `finalNode` only
        // becomes the path's tail once the goal is reached (see the `openNode ==
        // goal` branch below), so the heuristic must use `end_index` too.
        let goal = end_index;

        let mut costs = [SearchCost::default(); SEARCH_SLOTS];
        costs[from].g = 0.0;

        // Vanilla seeds the start node with `f = h = distance(to)`.
        let mut open_set = ArenaOpenSet::default();
        open_set.insert(from, positions[from].distance_to(&positions[goal]));

        let mut closest = from;
        let minimum_node_index = if alive_crystals == 0 { 12 } else { 0 };

        while open_set.size() > 0 {
            let open_node = open_set.pop();
            if open_node == goal {
                return Some(match final_slot {
                    Some(slot) => {
                        costs[slot].came_from = Some(goal);
                        reconstruct_path(&positions, &costs, slot)
                    }
                    None => reconstruct_path(&positions, &costs, goal),
                });
            }
            if positions[open_node].distance_to(&positions[goal])
                < positions[closest].distance_to(&positions[goal])
            {
                closest = open_node;
            }
            costs[open_node].closed = true;

            for i in minimum_node_index..NODE_COUNT {
                if NODE_ADJACENCY[open_node] & (1 << i) == 0 || costs[i].closed {
                    continue;
                }
                let tentative_g =
                    costs[open_node].g + positions[open_node].distance_to(&positions[i]);
                if open_set.in_open_set(i) && tentative_g >= costs[i].g {
                    continue;
                }
                costs[i].came_from = Some(open_node);
                costs[i].g = tentative_g;
                let f = tentative_g + positions[i].distance_to(&positions[goal]);
                if open_set.in_open_set(i) {
                    open_set.change_cost(i, f);
                } else {
                    open_set.insert(i, f);
                }
            }
        }

        // Vanilla: with no complete path, fall back to the closest node reached —
        // but give up when that is still the start.
        if closest == from {
            return None;
        }
        match final_slot {
            Some(slot) => {
                costs[slot].came_from = Some(closest);
                Some(reconstruct_path(&positions, &costs, slot))
            }
            None => Some(reconstruct_path(&positions, &costs, closest)),
        }
    }
}

/// The 24 arena waypoints plus vanilla's synthetic `finalNode` slot.
pub const SEARCH_SLOTS: usize = NODE_COUNT + 1;

/// Index of vanilla's synthetic `finalNode` inside the search arrays.
pub const FINAL_SLOT: usize = NODE_COUNT;

/// One node's A* bookkeeping, mirroring vanilla `Node`'s search fields.
#[derive(Debug, Clone, Copy, Default)]
struct SearchCost {
    g: f32,
    came_from: Option<usize>,
    closed: bool,
}

/// Vanilla `BinaryHeap`: a binary min-heap keyed on `Node.f` with *no*
/// tie-breaking, so equal costs keep vanilla's exact pop order.
///
/// Vanilla stores the heap index on the node itself (`heapIdx`), which is what
/// `inOpenSet` / `changeCost` rely on. Steel keeps the entry as `(slot, f)` and
/// mirrors `heapIdx` in a side array, because `Node` positions are immutable
/// here — the observable pop order is identical.
#[derive(Debug)]
struct ArenaOpenSet {
    heap: Vec<(usize, f32)>,
    heap_idx: [i32; SEARCH_SLOTS],
}

impl Default for ArenaOpenSet {
    fn default() -> Self {
        // Vanilla's `Node.heapIdx` starts at `-1` ("not in the set"), which is why
        // a derived `Default` (all zeroes) would be wrong here.
        Self {
            heap: Vec::new(),
            heap_idx: [-1; SEARCH_SLOTS],
        }
    }
}

impl ArenaOpenSet {
    fn size(&self) -> usize {
        self.heap.len()
    }

    fn in_open_set(&self, slot: usize) -> bool {
        self.heap_idx[slot] >= 0
    }

    fn insert(&mut self, slot: usize, cost: f32) {
        assert!(
            !self.in_open_set(slot),
            "node {slot} is already in the open set"
        );
        self.heap.push((slot, cost));
        self.heap_idx[slot] = (self.heap.len() - 1) as i32;
        self.up_heap(self.heap.len() - 1);
    }

    fn pop(&mut self) -> usize {
        let (popped, _) = self.heap[0];
        let last = self.heap.pop().expect("pop on a non-empty heap");
        if !self.heap.is_empty() {
            self.heap[0] = last;
            self.heap_idx[last.0] = 0;
            self.down_heap(0);
        }
        self.heap_idx[popped] = -1;
        popped
    }

    /// Vanilla `BinaryHeap.changeCost`: only the key changed, so one sift
    /// restores the invariant.
    fn change_cost(&mut self, slot: usize, cost: f32) {
        let position = self.heap_idx[slot] as usize;
        debug_assert_eq!(self.heap[position].0, slot);
        self.heap[position].1 = cost;
        if position > 0 && cost < self.heap[(position - 1) >> 1].1 {
            self.up_heap(position);
        } else {
            self.down_heap(position);
        }
    }

    fn up_heap(&mut self, mut position: usize) {
        let entry = self.heap[position];
        while position > 0 {
            let parent = (position - 1) >> 1;
            // Vanilla's `upHeap` stops on `!(cost < parent.f)`, i.e. equal costs
            // never move — the same comparison as below.
            if !(entry.1 < self.heap[parent].1) {
                break;
            }
            self.heap[position] = self.heap[parent];
            self.heap_idx[self.heap[position].0] = position as i32;
            position = parent;
        }
        self.heap[position] = entry;
        self.heap_idx[entry.0] = position as i32;
    }

    fn down_heap(&mut self, mut position: usize) {
        let entry = self.heap[position];
        loop {
            let left = 1 + (position << 1);
            if left >= self.heap.len() {
                break;
            }
            let right = left + 1;
            // Vanilla picks the left child on a tie.
            let child = if right < self.heap.len() && self.heap[right].1 < self.heap[left].1 {
                right
            } else {
                left
            };
            if !(self.heap[child].1 < entry.1) {
                break;
            }
            self.heap[position] = self.heap[child];
            self.heap_idx[self.heap[position].0] = position as i32;
            position = child;
        }
        self.heap[position] = entry;
        self.heap_idx[entry.0] = position as i32;
    }
}

/// Vanilla `EnderDragon.reconstructPath(from, to)`: prepend `to` and walk
/// `cameFrom` back to `from`, then build the path with `to` as the target.
#[must_use]
fn reconstruct_path(
    positions: &[Node; SEARCH_SLOTS],
    costs: &[SearchCost; SEARCH_SLOTS],
    goal: usize,
) -> Path {
    let mut path_nodes = Vec::with_capacity(SEARCH_SLOTS);
    // `nodes.add(0, node)` in vanilla == collecting backwards then reversing.
    let mut cursor = Some(goal);
    while let Some(slot) = cursor {
        path_nodes.push(positions[slot].clone());
        cursor = costs[slot].came_from;
    }
    path_nodes.reverse();
    let target = positions[goal].as_block_pos();
    Path::new(path_nodes, target, true)
}

/// Builds a path node from a world-space waypoint.
#[must_use]
pub fn node_from_position(position: DVec3) -> Node {
    Node::new(
        steel_math::fast_floor(position.x),
        steel_math::fast_floor(position.y),
        steel_math::fast_floor(position.z),
    )
}

/// Vanilla `EnderDragon.findClosestNode`'s polar layout for node `index`.
///
/// Returns `(x, z, y_adjustment)`: the first twelve nodes form a radius-60 ring,
/// the next eight a radius-40 ring ten blocks higher, and the last four a
/// radius-20 ring.
#[must_use]
pub fn node_polar_position(index: usize) -> (i32, i32, i32) {
    let multiplier = index as f32;
    let ring = |radius: f32, step: f32, offset: f32| {
        let angle = 2.0 * (-PI + step * (multiplier - offset));
        (
            steel_math::fast_floor(f64::from(radius * steel_math::trig::cos(angle))),
            steel_math::fast_floor(f64::from(radius * steel_math::trig::sin(angle))),
        )
    };

    if index < 12 {
        let (x, z) = ring(60.0, 0.261_799_4, 0.0);
        (x, z, 5)
    } else if index < 20 {
        let (x, z) = ring(40.0, 0.392_699_1, 12.0);
        (x, z, 15)
    } else {
        let (x, z) = ring(20.0, 0.785_398_2, 20.0);
        (x, z, 5)
    }
}

// ---------------------------------------------------------------------------
// Free-function wrappers
// ---------------------------------------------------------------------------
//
// `phase_ticks.rs` is written against `&mut DragonPhaseManager`, not the dragon,
// so that no phase tick ever re-locks the manager mutex. These wrappers keep the
// A* helpers usable from that side without a lock.

/// Vanilla `EnderDragon.findClosestNode()` for the given crystal count.
pub fn find_closest_node(
    manager: &mut DragonPhaseManager,
    world: &World,
    position: DVec3,
    alive_crystals: i32,
) -> usize {
    manager.find_closest_node_with_crystals(world, position, alive_crystals)
}

/// Vanilla `EnderDragon.findClosestNode(tX, tY, tZ)`.
pub fn find_closest_node_with_crystals(
    manager: &mut DragonPhaseManager,
    world: &World,
    position: DVec3,
    alive_crystals: i32,
) -> usize {
    manager.find_closest_node_with_crystals(world, position, alive_crystals)
}

/// Vanilla `EnderDragon.findPath(...)`.
pub fn find_path(
    manager: &mut DragonPhaseManager,
    world: &World,
    alive_crystals: i32,
    start_index: usize,
    end_index: usize,
    final_node: Option<DVec3>,
) -> Option<Path> {
    manager.find_path(world, alive_crystals, start_index, end_index, final_node)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_ids_are_vanilla_registry_order() {
        for (expected, phase) in ALL_DRAGON_PHASES.iter().enumerate() {
            assert_eq!(phase.id(), expected as i32, "{phase:?} must keep id {expected}");
            assert_eq!(DragonPhase::from_id(expected as i32), *phase);
        }
    }

    #[test]
    fn an_unknown_phase_id_falls_back_to_the_holding_pattern() {
        assert_eq!(DragonPhase::from_id(-1), DragonPhase::HoldingPattern);
        assert_eq!(DragonPhase::from_id(11), DragonPhase::HoldingPattern);
        assert_eq!(DragonPhase::from_id(i32::MAX), DragonPhase::HoldingPattern);
    }

    #[test]
    fn sitting_and_fly_speeds_match_the_vanilla_phase_classes() {
        assert!(!DragonPhase::HoldingPattern.is_sitting());
        assert!(DragonPhase::Hovering.is_sitting());
        assert!(DragonPhase::SittingScanning.is_sitting());

        assert_eq!(DragonPhase::HoldingPattern.fly_speed(), 0.6);
        assert_eq!(DragonPhase::Landing.fly_speed(), 1.5);
        assert_eq!(DragonPhase::ChargingPlayer.fly_speed(), 3.0);
        assert_eq!(DragonPhase::Dying.fly_speed(), 3.0);
        assert_eq!(DragonPhase::Hovering.fly_speed(), 1.0);
    }

    #[test]
    fn beginning_the_flaming_phase_counts_an_attack() {
        let mut state = DragonPhaseState::default();
        state.begin(DragonPhase::SittingFlaming);
        assert_eq!(state.flame_count, 1);
        assert_eq!(state.flame_ticks, 0);
        state.flame_ticks = 40;
        state.begin(DragonPhase::SittingFlaming);
        assert_eq!(state.flame_count, 2);
        assert_eq!(state.flame_ticks, 0);
    }

    #[test]
    fn beginning_a_phase_clears_the_path_and_target_but_keeps_the_direction() {
        let mut state = DragonPhaseState {
            target_location: Some(DVec3::ONE),
            clockwise: true,
            ..DragonPhaseState::default()
        };
        state.begin(DragonPhase::HoldingPattern);

        assert!(state.target_location.is_none());
        assert!(state.path.is_none());
        assert!(state.clockwise, "vanilla keeps the circling direction");
    }

    #[test]
    fn adjacency_rows_are_symmetric() {
        // Vanilla's table describes an undirected arena graph, so every edge must
        // appear in both rows. This catches a transcription slip in the literals.
        for (from, row) in NODE_ADJACENCY.iter().enumerate() {
            for to in 0..NODE_COUNT {
                assert_eq!(
                    row & (1 << to) != 0,
                    NODE_ADJACENCY[to] & (1 << from) != 0,
                    "edge {from} <-> {to} is not symmetric"
                );
            }
        }
    }

    #[test]
    fn adjacency_never_references_a_node_outside_the_arena() {
        let mask = (1_u32 << NODE_COUNT) - 1;
        for (index, row) in NODE_ADJACENCY.iter().enumerate() {
            assert_eq!(row & !mask, 0, "row {index} points past node 23");
        }
    }

    #[test]
    fn the_node_rings_match_vanilla_radii() {
        for index in 0..12 {
            let (x, z, adjustment) = node_polar_position(index);
            assert_eq!(adjustment, 5);
            let radius = f64::from(x).hypot(f64::from(z));
            assert!((radius - 60.0).abs() < 2.0, "node {index} radius {radius}");
        }
        for index in 12..20 {
            let (x, z, adjustment) = node_polar_position(index);
            assert_eq!(adjustment, 15);
            let radius = f64::from(x).hypot(f64::from(z));
            assert!((radius - 40.0).abs() < 2.0, "node {index} radius {radius}");
        }
        for index in 20..24 {
            let (_, _, adjustment) = node_polar_position(index);
            assert_eq!(adjustment, 5);
        }
    }

    #[test]
    fn a_manager_starts_in_the_hover_phase() {
        let manager = DragonPhaseManager::new(DVec3::ZERO);
        assert_eq!(manager.current(), DragonPhase::Hovering);
        assert!(manager.current().is_sitting());
    }

    #[test]
    fn switching_to_a_new_phase_resets_its_state_once() {
        let mut manager = DragonPhaseManager::new(DVec3::ZERO);
        assert!(manager.set_phase(DragonPhase::SittingFlaming));
        assert_eq!(manager.current_state().flame_count, 1);

        manager.current_state_mut().flame_ticks = 30;
        assert!(
            !manager.set_phase(DragonPhase::SittingFlaming),
            "vanilla returns early when the phase is unchanged"
        );
        assert_eq!(manager.current_state().flame_ticks, 30);
    }

    #[test]
    fn the_pathfinder_reaches_an_adjacent_node() {
        // Node 0 connects to node 1, so a path must exist without any world data.
        assert_ne!(NODE_ADJACENCY[0] & (1 << 1), 0);
    }

    /// Runs `find_path` over a hand-built node array, so the A* and the heap are
    /// exercised without a `World`.
    fn search(
        nodes: &[DVec3; NODE_COUNT],
        alive_crystals: i32,
        start: usize,
        end: usize,
        final_node: Option<DVec3>,
    ) -> Option<Path> {
        let positions: [Node; SEARCH_SLOTS] = std::array::from_fn(|slot| match nodes.get(slot) {
            Some(node) => node_from_position(*node),
            None => match final_node {
                Some(position) => node_from_position(position),
                None => Node::new(0, 0, 0),
            },
        });
        let goal = end;
        let final_slot = final_node.map(|_| FINAL_SLOT);

        let mut costs = [SearchCost::default(); SEARCH_SLOTS];
        costs[start].g = 0.0;

        let mut open_set = ArenaOpenSet::default();
        open_set.insert(start, positions[start].distance_to(&positions[goal]));

        let mut closest = start;
        let minimum_node_index = if alive_crystals == 0 { 12 } else { 0 };
        while open_set.size() > 0 {
            let open_node = open_set.pop();
            if open_node == goal {
                return Some(match final_slot {
                    Some(slot) => {
                        costs[slot].came_from = Some(goal);
                        reconstruct_path(&positions, &costs, slot)
                    }
                    None => reconstruct_path(&positions, &costs, goal),
                });
            }
            if positions[open_node].distance_to(&positions[goal])
                < positions[closest].distance_to(&positions[goal])
            {
                closest = open_node;
            }
            costs[open_node].closed = true;
            for i in minimum_node_index..NODE_COUNT {
                if NODE_ADJACENCY[open_node] & (1 << i) == 0 || costs[i].closed {
                    continue;
                }
                let tentative_g =
                    costs[open_node].g + positions[open_node].distance_to(&positions[i]);
                if open_set.in_open_set(i) && tentative_g >= costs[i].g {
                    continue;
                }
                costs[i].came_from = Some(open_node);
                costs[i].g = tentative_g;
                let f = tentative_g + positions[i].distance_to(&positions[goal]);
                if open_set.in_open_set(i) {
                    open_set.change_cost(i, f);
                } else {
                    open_set.insert(i, f);
                }
            }
        }
        if closest == start {
            return None;
        }
        match final_slot {
            Some(slot) => {
                costs[slot].came_from = Some(closest);
                Some(reconstruct_path(&positions, &costs, slot))
            }
            None => Some(reconstruct_path(&positions, &costs, closest)),
        }
    }

    /// A cheap stand-in arena: a straight line, so every hop is unambiguously
    /// shorter and the search cannot tie on cost.
    fn line_nodes(step: f64) -> [DVec3; NODE_COUNT] {
        std::array::from_fn(|index| DVec3::new(index as f64 * step, 80.0, 0.0))
    }

    #[test]
    fn a_path_starts_at_the_start_and_ends_at_the_goal() {
        // Nodes 0 and 1 are adjacent in vanilla's table.
        let nodes = line_nodes(1.0);
        let path = search(&nodes, 1, 0, 1, None).expect("adjacent nodes must connect");
        assert_eq!(path.node_count(), 2, "0 -> 1 is a single hop");
        let start = path.node(0).expect("path has a first node");
        let end = path.node(1).expect("path has a last node");
        assert_eq!((start.x, start.y, start.z), (0, 80, 0));
        assert_eq!((end.x, end.y, end.z), (1, 80, 0));
    }

    #[test]
    fn the_final_node_becomes_the_path_tail() {
        let nodes = line_nodes(1.0);
        let final_node = DVec3::new(1.5, 81.5, 0.0);
        let path = search(&nodes, 0, 0, 1, Some(final_node)).expect("adjacent nodes connect");
        let end = path.end_node().expect("path has a last node");
        // Vanilla floors the final node's coordinates.
        assert_eq!((end.x, end.y, end.z), (1, 81, 0));
        assert_eq!(path.node_count(), 3, "0 -> 1 -> finalNode");
    }

    #[test]
    fn an_already_reached_goal_is_a_single_node_path() {
        // `findPath(start, start)` pops the start node as the goal immediately,
        // which is what the landing phases rely on after a retarget.
        let nodes = line_nodes(10.0);
        let path = search(&nodes, 1, 4, 4, None).expect("a node always reaches itself");
        assert_eq!(path.node_count(), 1);
        assert!(path.is_done() || path.node_count() == 1);
    }

    #[test]
    fn the_open_set_pops_the_cheapest_node_first() {
        let mut open_set = ArenaOpenSet::default();
        open_set.insert(5, 3.0);
        open_set.insert(7, 1.0);
        open_set.insert(9, 2.0);
        assert_eq!(open_set.size(), 3);
        assert_eq!(open_set.pop(), 7);
        assert_eq!(open_set.pop(), 9);
        assert_eq!(open_set.pop(), 5);
        assert_eq!(open_set.size(), 0);
        assert!(!open_set.in_open_set(5));
    }

    #[test]
    fn the_open_set_keeps_insertion_order_on_equal_costs() {
        // Vanilla's heap never moves an equal cost upward, so ties pop in insert
        // order.
        let mut open_set = ArenaOpenSet::default();
        open_set.insert(3, 1.0);
        open_set.insert(4, 1.0);
        open_set.insert(5, 1.0);
        assert_eq!(open_set.pop(), 3);
        assert_eq!(open_set.pop(), 4);
        assert_eq!(open_set.pop(), 5);
    }
}
