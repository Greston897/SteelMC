//! Boat entities (`Boat` and `Raft`).
//!
//! Port of 26.3 `AbstractBoatEntity`: water/land/air location detection,
//! buoyancy and drag per location, paddling from the rider's input, paddle
//! sounds, damage wobble/breaking, riding and the vanilla drop item (the item
//! whose id matches the entity id).
//!
//! Not yet ported (marked with TODOs in the code): the bubble-column wobble and
//! the passenger yaw clamp. The wobble needs the `onBubbleColumnSurfaceCollision`
//! hook from the bubble column block; leashing itself comes from the shared
//! `EntityBase` tick, so it needs nothing extra here.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::fluid::FluidStateExt as _;
use steel_registry::items::ItemRef;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::AbstractBoatEntityData;
use steel_registry::{REGISTRY, sound_events, vanilla_blocks, vanilla_items};
use steel_utils::locks::SyncMutex;
use steel_utils::types::InteractionHand;
use steel_utils::{DowncastType, DowncastTypeKey};

use super::vehicle_entity::{
    VEHICLE_DIMENSION_CHANGING_DELAY, VehicleEntity, vehicle_ignores_explosion,
};
use crate::behavior::InteractionResult;
use crate::entity::{
    DamageSource, Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntitySyncedData,
    Player, SharedEntity,
};
use crate::physics::collision::{WorldCollisionProvider, has_block_collision};
use crate::physics::MoverType;
use crate::world::{Explosion, World};

/// Vanilla `AbstractBoatEntity.UNDERWATER_TICKS_MAX`: riders are ejected after
/// a second fully underwater.
pub const UNDERWATER_TICKS_MAX: f32 = 60.0;

/// Vanilla `NEXT_PADDLE_PHASE`: `π / 8` per tick.
pub const NEXT_PADDLE_PHASE: f32 = std::f32::consts::PI / 8.0;

/// Vanilla `EMIT_SOUND_EVENT_PADDLE_ROTATION`: `π / 4`.
pub const EMIT_SOUND_EVENT_PADDLE_ROTATION: f32 = std::f32::consts::PI / 4.0;

/// Vanilla `PADDLE_TURN_SPEED`.
pub const PADDLE_TURN_SPEED: f32 = 0.005;

/// Vanilla `PADDLE_FORWARD_SPEED`.
pub const PADDLE_FORWARD_SPEED: f32 = 0.04;

/// Vanilla `PADDLE_BACK_SPEED`.
pub const PADDLE_BACK_SPEED: f32 = 0.005;

/// Vanilla `GRAVITY_IN_WATER`, also the boat's `getGravity()`.
pub const GRAVITY_IN_WATER: f64 = 0.04;

/// Vanilla `GRAVITY_UNDER_FLOWING_WATER`.
pub const GRAVITY_UNDER_FLOWING_WATER: f64 = -7.0e-4;

/// Vanilla `BUOYANCY_UNDER_WATER`.
pub const BUOYANCY_UNDER_WATER: f64 = 0.01;

/// Vanilla `DRAG_IN_WATER`.
pub const DRAG_IN_WATER: f32 = 0.9;

/// Vanilla `DRAG_UNDER_WATER`.
pub const DRAG_UNDER_WATER: f32 = 0.45;

/// Vanilla `DRAG_IN_AIR`.
pub const DRAG_IN_AIR: f32 = 0.9;

/// Vanilla `BUOYANCY_SCALE`.
pub const BUOYANCY_SCALE: f64 = 0.65;

/// Vanilla `BUOYANCY_DAMPING`.
pub const BUOYANCY_DAMPING: f64 = 0.75;

/// Vanilla `BUBBLE_WOBBLE_TICKS_MAX`.
pub const BUBBLE_WOBBLE_TICKS_MAX: i32 = 60;

/// Vanilla `BUBBLE_COLUMN_DRAG_VELOCITY`.
pub const BUBBLE_COLUMN_DRAG_VELOCITY: f64 = -0.7;

/// Vanilla `BUBBLE_COLUMN_LAUNCH_VELOCITY_PLAYER`.
pub const BUBBLE_COLUMN_LAUNCH_VELOCITY_PLAYER: f64 = 2.7;

/// Vanilla `BUBBLE_COLUMN_LAUNCH_VELOCITY_NO_PLAYER`.
pub const BUBBLE_COLUMN_LAUNCH_VELOCITY_NO_PLAYER: f64 = 0.6;

/// Vanilla `ENTITY_PUSH_EXPAND_H`.
pub const ENTITY_PUSH_EXPAND_H: f64 = 0.2;

/// Vanilla `ENTITY_PUSH_EXPAND_V`.
pub const ENTITY_PUSH_EXPAND_V: f64 = -0.01;

/// Vanilla `SPLASH_SOUND_PITCH_BASE`.
pub const SPLASH_SOUND_PITCH_BASE: f32 = 0.8;

/// Vanilla `SPLASH_SOUND_PITCH_RANGE`.
pub const SPLASH_SOUND_PITCH_RANGE: f32 = 0.4;

/// Vanilla `AbstractBoatEntity.getMaxPassengers`.
pub const MAX_PASSENGERS: usize = 2;

/// Vanilla boat `Location` enum (`AbstractBoatEntity.Location`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoatLocation {
    /// Floating on the surface, hull partly submerged.
    InWater,
    /// Fully below a still water surface.
    UnderWater,
    /// Below a flowing water surface.
    UnderFlowingWater,
    /// Resting on a solid block.
    OnLand,
    /// Falling.
    InAir,
}

impl BoatLocation {
    /// Whether the location counts as "the boat is in water".
    #[must_use]
    pub const fn is_water(self) -> bool {
        matches!(
            self,
            Self::InWater | Self::UnderWater | Self::UnderFlowingWater
        )
    }

    /// Whether the location counts as "the boat is fully submerged".
    #[must_use]
    pub const fn is_underwater(self) -> bool {
        matches!(self, Self::UnderWater | Self::UnderFlowingWater)
    }

    /// Vanilla `AbstractBoatEntity#getPaddleSound` for this location.
    #[must_use]
    pub fn paddle_sound(self) -> Option<SoundEventRef> {
        match self {
            Self::InWater | Self::UnderWater | Self::UnderFlowingWater => {
                Some(&sound_events::ENTITY_BOAT_PADDLE_WATER)
            }
            Self::OnLand => Some(&sound_events::ENTITY_BOAT_PADDLE_LAND),
            Self::InAir => None,
        }
    }
}

/// Non-synchronized boat state (`AbstractBoatEntity` fields).
#[derive(Debug, Clone)]
pub struct BoatState {
    /// Current vanilla boat location.
    pub location: BoatLocation,
    /// Location during the previous tick.
    pub last_location: BoatLocation,
    /// Paddle animation phases, left and right.
    pub paddle_phases: [f32; 2],
    /// Consecutive ticks spent fully underwater.
    pub ticks_underwater: f32,
    /// Yaw velocity damped by drag and driven by turning input.
    pub yaw_velocity: f32,
    /// Rider wants to turn left.
    pub pressing_left: bool,
    /// Rider wants to turn right.
    pub pressing_right: bool,
    /// Rider wants to move forward.
    pub pressing_forward: bool,
    /// Rider wants to move backward.
    pub pressing_back: bool,
    /// Height of the water surface around the boat.
    pub water_level: f64,
    /// Slipperiness of the blocks underneath the boat.
    pub nearby_slipperiness: f32,
    /// Vertical speed accumulated while falling, used by `getWaterHeightBelow`.
    pub fall_velocity: f64,
    /// Whether the boat sits on a bubble column surface.
    pub on_bubble_column_surface: bool,
    /// Whether the bubble column below the boat drags it down.
    pub bubble_column_is_drag: bool,
}

impl Default for BoatState {
    fn default() -> Self {
        Self {
            location: BoatLocation::InAir,
            last_location: BoatLocation::InAir,
            paddle_phases: [0.0; 2],
            ticks_underwater: 0.0,
            yaw_velocity: 0.0,
            pressing_left: false,
            pressing_right: false,
            pressing_forward: false,
            pressing_back: false,
            water_level: 0.0,
            nearby_slipperiness: 1.0,
            fall_velocity: 0.0,
            on_bubble_column_surface: false,
            bubble_column_is_drag: false,
        }
    }
}

/// Shared `AbstractBoatEntity` behaviour for the `Boat` and `Raft` classes.
pub trait AbstractBoat: VehicleEntity {
    /// Returns the boat's non-synchronized state.
    fn boat_state(&self) -> &SyncMutex<BoatState>;

    /// Returns the boat's synchronized data (paddle flags and bubble time).
    fn boat_data(&self) -> &SyncMutex<AbstractBoatEntityData>;

    /// Vanilla `isPaddleMoving(index)`; `0` is the left paddle, `1` the right.
    fn paddle_moving(&self, index: usize) -> bool {
        let data = self.boat_data().lock();
        match index {
            0 => *data.id_paddle_left.get(),
            _ => *data.id_paddle_right.get(),
        }
    }

    /// Vanilla `setPaddlesMoving(left, right)`.
    fn set_paddles_moving(&self, left: bool, right: bool) {
        let mut data = self.boat_data().lock();
        data.id_paddle_left.set(left);
        data.id_paddle_right.set(right);
    }

    /// Vanilla `setInputs(left, right, forward, back)`.
    fn set_inputs(&self, left: bool, right: bool, forward: bool, back: bool) {
        let mut state = self.boat_state().lock();
        state.pressing_left = left;
        state.pressing_right = right;
        state.pressing_forward = forward;
        state.pressing_back = back;
    }

    /// Vanilla `getBubbleWobbleTicks`.
    fn bubble_time(&self) -> i32 {
        *self.boat_data().lock().id_bubble_time.get()
    }

    /// Vanilla `setBubbleWobbleTicks`.
    fn set_bubble_time(&self, ticks: i32) {
        let mut data = self.boat_data().lock();
        data.id_bubble_time.set(ticks);
    }

    /// Vanilla `AbstractBoatEntity.onAboveBubbleColumn`: remember the column the
    /// boat floats above and arm the 60-tick bubble timer. Vanilla overrides the
    /// generic entity push because a boat is thrown out of the column instead.
    fn boat_above_bubble_column(&self, drag_down: bool, pos: BlockPos) {
        {
            let mut state = self.boat_state().lock();
            state.on_bubble_column_surface = true;
            state.bubble_column_is_drag = drag_down;
        }
        if self.bubble_time() <= 0 {
            self.set_bubble_time(BUBBLE_WOBBLE_TICKS_MAX);
        }
        if let Some(world) = self.level() {
            world.send_bubble_column_particles(pos);
        }
    }

    /// Vanilla `AbstractBoatEntity.handleBubbleColumnEffect` (entity events 71/72):
    /// a drag column pulls the boat down and lets the riders out, an up column
    /// throws the boat.
    fn boat_apply_bubble_column_effect(&self) {
        let drag = self.boat_state().lock().bubble_column_is_drag;
        if drag {
            let velocity = self.velocity();
            self.set_velocity(DVec3::new(
                velocity.x,
                velocity.y + BUBBLE_COLUMN_DRAG_VELOCITY,
                velocity.z,
            ));
            self.eject_passengers();
        } else {
            let carries_player = self
                .passengers()
                .first()
                .is_some_and(|passenger| passenger.as_player().is_some());
            let launch = if carries_player {
                BUBBLE_COLUMN_LAUNCH_VELOCITY_PLAYER
            } else {
                BUBBLE_COLUMN_LAUNCH_VELOCITY_NO_PLAYER
            };
            let velocity = self.velocity();
            self.set_velocity(DVec3::new(velocity.x, launch, velocity.z));
        }
    }

    /// Vanilla `getPaddleSound` for the boat's current location.
    fn paddle_sound(&self) -> Option<SoundEventRef> {
        self.boat_state().lock().location.paddle_sound()
    }

    /// Vanilla `checkLocation`.
    fn check_location(&self, world: &World) -> BoatLocation {
        if let Some(underwater) = self.underwater_location(world) {
            let max_y = self.bounding_box().max_y();
            self.boat_state().lock().water_level = max_y;
            return underwater;
        }

        if self.check_boat_in_water(world) {
            return BoatLocation::InWater;
        }

        let slipperiness = self.nearby_slipperiness(world);
        if slipperiness > 0.0 {
            self.boat_state().lock().nearby_slipperiness = slipperiness;
            return BoatLocation::OnLand;
        }

        BoatLocation::InAir
    }

    /// Vanilla `checkBoatInWater`: also refreshes `waterLevel`.
    fn check_boat_in_water(&self, world: &World) -> bool {
        let bb = self.bounding_box();
        let min_x = bb.min_x().floor() as i32;
        let max_x = bb.max_x().ceil() as i32;
        let min_y = bb.min_y().floor() as i32;
        let max_y = (bb.min_y() + 0.001).ceil() as i32;
        let min_z = bb.min_z().floor() as i32;
        let max_z = bb.max_z().ceil() as i32;

        let mut in_water = false;
        let mut water_level = f64::MIN;
        for x in min_x..max_x {
            for y in min_y..max_y {
                for z in min_z..max_z {
                    let pos = steel_utils::BlockPos::new(x, y, z);
                    let fluid = world.get_block_state(pos).get_fluid_state();
                    if !fluid.is_water() {
                        continue;
                    }
                    let fluid_top = f64::from(y) + f64::from(fluid.own_height());
                    water_level = water_level.max(fluid_top);
                    in_water |= bb.min_y() < fluid_top;
                }
            }
        }
        self.boat_state().lock().water_level = water_level;
        in_water
    }

    /// Vanilla `getUnderWaterLocation`.
    fn underwater_location(&self, world: &World) -> Option<BoatLocation> {
        let bb = self.bounding_box();
        let top_y = bb.max_y() + 0.001;
        let min_x = bb.min_x().floor() as i32;
        let max_x = bb.max_x().ceil() as i32;
        let min_y = bb.max_y().floor() as i32;
        let max_y = top_y.ceil() as i32;
        let min_z = bb.min_z().floor() as i32;
        let max_z = bb.max_z().ceil() as i32;

        let mut still_water = false;
        for x in min_x..max_x {
            for y in min_y..max_y {
                for z in min_z..max_z {
                    let pos = steel_utils::BlockPos::new(x, y, z);
                    let fluid = world.get_block_state(pos).get_fluid_state();
                    if !fluid.is_water() {
                        continue;
                    }
                    if top_y < f64::from(y) + f64::from(fluid.own_height()) {
                        if !fluid.is_source() {
                            return Some(BoatLocation::UnderFlowingWater);
                        }
                        still_water = true;
                    }
                }
            }
        }
        still_water.then_some(BoatLocation::UnderWater)
    }

    /// Vanilla `getWaterHeightBelow`: the water surface the boat would land on.
    fn water_height_below(&self, world: &World) -> f32 {
        let bb = self.bounding_box();
        let min_x = bb.min_x().floor() as i32;
        let max_x = bb.max_x().ceil() as i32;
        let min_y = bb.max_y().floor() as i32;
        let fall = self.boat_state().lock().fall_velocity;
        let max_y = (bb.max_y() - fall).ceil() as i32;
        let min_z = bb.min_z().floor() as i32;
        let max_z = bb.max_z().ceil() as i32;

        for y in min_y..max_y {
            let mut max_fluid_height = 0.0_f32;
            for x in min_x..max_x {
                for z in min_z..max_z {
                    let pos = steel_utils::BlockPos::new(x, y, z);
                    let fluid = world.get_block_state(pos).get_fluid_state();
                    if fluid.is_water() {
                        max_fluid_height = max_fluid_height.max(fluid.own_height());
                    }
                }
            }
            if max_fluid_height < 1.0 {
                return y as f32 + max_fluid_height;
            }
        }
        max_y as f32 + 1.0
    }

    /// Vanilla `getNearbySlipperiness`: averaged block friction under the boat.
    ///
    /// Vanilla intersects each block's exact collision shape with the thin
    /// ground box; Steel approximates that with a vertical overlap test, which
    /// is identical for full blocks (the common case under a boat).
    fn nearby_slipperiness(&self, world: &World) -> f32 {
        let bb = self.bounding_box();
        let ground_min_y = bb.min_y() - 0.001;
        let min_x = bb.min_x().floor() as i32 - 1;
        let max_x = bb.max_x().ceil() as i32 + 1;
        let min_y = ground_min_y.floor() as i32 - 1;
        let max_y = bb.min_y().ceil() as i32 + 1;
        let min_z = bb.min_z().floor() as i32 - 1;
        let max_z = bb.max_z().ceil() as i32 + 1;

        let mut total = 0.0_f32;
        let mut count = 0_u32;
        for x in min_x..max_x {
            for z in min_z..max_z {
                let edge_count = i32::from(x != min_x && x != max_x - 1)
                    + i32::from(z != min_z && z != max_z - 1);
                if edge_count == 2 {
                    continue;
                }
                for y in min_y..max_y {
                    if edge_count > 0 && (y == min_y || y == max_y - 1) {
                        continue;
                    }
                    let pos = steel_utils::BlockPos::new(x, y, z);
                    let state = world.get_block_state(pos);
                    let block_top = f64::from(y) + 1.0;
                    // Only blocks that actually reach the ground box count.
                    if block_top < ground_min_y || f64::from(y) > bb.min_y() {
                        continue;
                    }
                    if state.get_block() == &vanilla_blocks::LILY_PAD {
                        continue;
                    }
                    if state.get_collision_shape_at(pos).is_empty() {
                        continue;
                    }
                    total += state.get_block().config.friction;
                    count += 1;
                }
            }
        }
        if count == 0 { 0.0 } else { total / count as f32 }
    }

    /// Vanilla `updateVelocity`: buoyancy, drag and gravity per location.
    fn update_velocity(&self) {
        let mut gravity_y = -GRAVITY_IN_WATER;
        let mut buoyancy = 0.0_f64;

        let (location, last_location) = {
            let state = self.boat_state().lock();
            (state.location, state.last_location)
        };

        if last_location == BoatLocation::InAir
            && location != BoatLocation::InAir
            && location != BoatLocation::OnLand
        {
            let Some(world) = self.level() else {
                return;
            };
            let body_y = self.bounding_box().max_y();
            let target_y =
                f64::from(self.water_height_below(world.as_ref())) - self.boat_height() + 0.101;
            {
                let mut state = self.boat_state().lock();
                state.water_level = body_y;
            }
            let shifted = self
                .bounding_box()
                .translate(DVec3::new(0.0, target_y - self.position().y, 0.0));
            let collision = WorldCollisionProvider::new(&world);
            if !has_block_collision(&collision, shifted) {
                let _ = self.try_set_position(DVec3::new(
                    self.position().x,
                    target_y,
                    self.position().z,
                ));
                let velocity = self.velocity();
                self.set_velocity(DVec3::new(velocity.x, 0.0, velocity.z));
                self.boat_state().lock().fall_velocity = 0.0;
            }
            self.boat_state().lock().location = BoatLocation::InWater;
        } else {
            let drag;
            match location {
                BoatLocation::InWater => {
                    let (water_level, height) = {
                        let state = self.boat_state().lock();
                        (state.water_level, self.boat_height())
                    };
                    buoyancy = (water_level - self.position().y) / height;
                    drag = DRAG_IN_WATER;
                }
                BoatLocation::UnderFlowingWater => {
                    gravity_y = GRAVITY_UNDER_FLOWING_WATER;
                    drag = DRAG_IN_WATER;
                }
                BoatLocation::UnderWater => {
                    buoyancy = BUOYANCY_UNDER_WATER;
                    drag = DRAG_UNDER_WATER;
                }
                BoatLocation::InAir => drag = DRAG_IN_AIR,
                BoatLocation::OnLand => {
                    let mut state = self.boat_state().lock();
                    drag = state.nearby_slipperiness;
                    // Vanilla halves the *stored* slipperiness, not the drag used this tick.
                    let player_riding = self
                        .controlling_passenger()
                        .is_some_and(|passenger| passenger.as_player().is_some());
                    if player_riding {
                        state.nearby_slipperiness /= 2.0;
                    }
                }
            }

            let velocity = self.velocity();
            let drag = f64::from(drag);
            self.set_velocity(DVec3::new(
                velocity.x * drag,
                velocity.y + gravity_y,
                velocity.z * drag,
            ));
            {
                let mut state = self.boat_state().lock();
                state.yaw_velocity *= drag as f32;
            }

            if buoyancy > 0.0 {
                let velocity = self.velocity();
                let gravity = GRAVITY_IN_WATER;
                self.set_velocity(DVec3::new(
                    velocity.x,
                    (velocity.y + buoyancy * (gravity / BUOYANCY_SCALE)) * BUOYANCY_DAMPING,
                    velocity.z,
                ));
            }
        }
    }

    /// Vanilla `updatePaddles`: turns and accelerates the boat from rider input.
    fn update_paddles(&self) {
        if self.passengers().is_empty() {
            return;
        }

        let (left, right, forward, back) = {
            let state = self.boat_state().lock();
            (
                state.pressing_left,
                state.pressing_right,
                state.pressing_forward,
                state.pressing_back,
            )
        };

        let mut forward_force = 0.0_f32;
        {
            let mut state = self.boat_state().lock();
            if left {
                state.yaw_velocity -= 1.0;
            }
            if right {
                state.yaw_velocity += 1.0;
            }
            if right != left && !forward && !back {
                forward_force += PADDLE_TURN_SPEED;
            }
        }

        let (yaw, pitch) = self.rotation();
        let yaw_velocity = self.boat_state().lock().yaw_velocity;
        self.set_rotation((yaw + yaw_velocity, pitch));

        if forward {
            forward_force += PADDLE_FORWARD_SPEED;
        }
        if back {
            forward_force -= PADDLE_BACK_SPEED;
        }

        let (yaw, _) = self.rotation();
        let yaw_radians = yaw * std::f32::consts::PI / 180.0;
        let velocity = self.velocity();
        self.set_velocity(velocity + DVec3::new(
            f64::from(-yaw_radians.sin() * forward_force),
            0.0,
            f64::from(yaw_radians.cos() * forward_force),
        ));

        self.set_paddles_moving(
            (right && !left) || forward,
            (left && !right) || forward,
        );
    }

    /// Vanilla `tickPaddleSounds`.
    ///
    /// Both paddle phases advance every tick; a sound is queued when a phase
    /// crosses the emit threshold and is played after the state guard drops, so
    /// spawning the sound effect cannot re-enter this boat's locks.
    fn tick_paddle_sounds(&self, world: &World) {
        let (yaw, _) = self.rotation();
        let yaw_radians = yaw * std::f32::consts::PI / 180.0;
        let rot_x = -yaw_radians.sin();
        let rot_z = yaw_radians.cos();
        let silent = self.is_silent();

        let mut plays: Vec<(f32, f32, SoundEventRef)> = Vec::new();
        {
            let flags = {
                let data = self.boat_data().lock();
                [*data.id_paddle_left.get(), *data.id_paddle_right.get()]
            };
            let mut state = self.boat_state().lock();
            let full_turn = std::f32::consts::PI * 2.0;
            for (index, moving) in flags.into_iter().enumerate() {
                if !moving {
                    state.paddle_phases[index] = 0.0;
                    continue;
                }

                let phase = state.paddle_phases[index];
                let crossed = phase.rem_euclid(full_turn) <= EMIT_SOUND_EVENT_PADDLE_ROTATION
                    && (phase + NEXT_PADDLE_PHASE).rem_euclid(full_turn)
                        >= EMIT_SOUND_EVENT_PADDLE_ROTATION;
                if crossed
                    && !silent
                    && let Some(sound) = state.location.paddle_sound()
                {
                    let offset = if index == 1 {
                        (rot_x, -rot_z)
                    } else {
                        (-rot_x, rot_z)
                    };
                    plays.push((offset.0, offset.1, sound));
                }

                state.paddle_phases[index] += NEXT_PADDLE_PHASE;
            }
        }

        let position = self.position();
        for (offset_x, offset_z, sound) in plays {
            let pitch =
                SPLASH_SOUND_PITCH_BASE + SPLASH_SOUND_PITCH_RANGE * rand::random::<f32>();
            world.play_sound_at(
                sound,
                SoundSource::Neutral,
                DVec3::new(
                    position.x + f64::from(offset_x),
                    position.y,
                    position.z + f64::from(offset_z),
                ),
                1.0,
                pitch,
                None,
            );
        }
    }

    /// Drops stored container contents when the boat is destroyed.
    ///
    /// Vanilla `AbstractChestBoatEntity#remove` scatters on every destroying
    /// removal; plain boats have nothing to drop.
    fn scatter_boat_contents(&self, _world: &World) {}

    /// Persists boat-specific extras (chest boats store their inventory).
    fn save_boat_extra(&self, _nbt: &mut NbtCompound) {}

    /// Restores boat-specific extras.
    fn load_boat_extra(&self, _nbt: BorrowedNbtCompoundView<'_, '_>) {}

    /// Vanilla `getMaxPassengers`: two for a boat, one for a chest boat.
    fn max_passengers(&self) -> usize {
        MAX_PASSENGERS
    }

    /// Vanilla `AbstractBoatEntity#getHeight`, used by the buoyancy maths.
    fn boat_height(&self) -> f64 {
        self.bounding_box().max_y() - self.bounding_box().min_y()
    }

    /// The item dropped when the boat is destroyed.
    ///
    /// Boat entity ids match their item ids in vanilla (`oak_boat` entity →
    /// `oak_boat` item), so the drop is looked up by key.
    fn boat_drop_item(&self) -> ItemRef {
        let key = &self.entity_type().key;
        REGISTRY
            .items
            .iter()
            .find_map(|(_, item)| (&item.key == key).then_some(item))
            .unwrap_or(&vanilla_items::OAK_BOAT)
    }
}

/// Vanilla `Entity#isLogicalSideForUpdatingMovement` for boats.
///
/// A player-driven boat is simulated by that player's client, which reports its
/// position through `MoveVehicle`; everything else is simulated here.
fn logical_side_for_updating_movement<T: AbstractBoat + 'static>(boat: &Arc<T>) -> bool {
    boat.controlling_passenger()
        .is_none_or(|passenger| passenger.as_player().is_none())
}

/// Vanilla `AbstractBoatEntity#tick`, generic over the concrete boat type so the
/// `self: Arc<Self>` movement call stays available.
///
pub fn boat_tick<T: AbstractBoat + 'static>(boat: Arc<T>) {
    let Some(world) = boat.level() else {
        return;
    };

    {
        let mut state = boat.boat_state().lock();
        state.last_location = state.location;
        // Re-armed by `on_above_bubble_column` while the boat floats on a column.
        state.on_bubble_column_surface = false;
    }
    let location = boat.check_location(world.as_ref());
    {
        let mut state = boat.boat_state().lock();
        state.location = location;
        if location.is_underwater() {
            state.ticks_underwater += 1.0;
        } else {
            state.ticks_underwater = 0.0;
        }
    }

    if boat.boat_state().lock().ticks_underwater >= UNDERWATER_TICKS_MAX {
        boat.eject_passengers();
    }

    // Vanilla decays the damage wobble every tick.
    if boat.hurt_time() > 0 {
        boat.set_hurt_time(boat.hurt_time() - 1);
    }
    if boat.damage() > 0.0 {
        boat.set_damage(boat.damage() - 1.0);
    }

    boat.base_tick();

    if logical_side_for_updating_movement(&boat) {
        let first_is_player = boat
            .passengers()
            .first()
            .is_some_and(|passenger| passenger.as_player().is_some());
        if !first_is_player {
            boat.set_paddles_moving(false, false);
        }
        boat.update_velocity();
        // Vanilla runs `updatePaddles` on the client only, but Steel has no
        // client-side boat simulation, so mob-driven boats are still paddled
        // here. Player-driven boats return `false` from
        // [`logical_side_for_updating_movement`] and are moved by `MoveVehicle`.
        boat.update_paddles();
        let velocity = boat.velocity();
        let _ = Arc::clone(&boat).default_move_entity(MoverType::SelfMovement, velocity);
        // Vanilla records the vertical speed in `fall()`; Steel has no `fall`
        // hook, so the pre-move speed is stored instead.
        boat.boat_state().lock().fall_velocity = velocity.y;
    } else {
        boat.set_velocity(DVec3::ZERO);
    }

    boat.tick_paddle_sounds(world.as_ref());
    push_nearby_entities(&boat, world.as_ref());

    // Vanilla `AbstractBoatEntity.tickBubbleColumn`: the timer only runs while
    // the boat stays on a column surface; when it expires the boat is thrown.
    if boat.boat_state().lock().on_bubble_column_surface {
        let remaining = (boat.bubble_time() - 1).max(0);
        boat.set_bubble_time(remaining);
        if remaining == 0 {
            boat.boat_apply_bubble_column_effect();
        }
    } else {
        boat.set_bubble_time(0);
    }
}

/// Shared boat `interact` up to the point where chest boats open their container.
///
/// Vanilla chain: `AbstractBoatEntity#interact` → `VehicleEntity#interact` →
/// `PlayerEntity#startRiding`.
pub fn ride_boat_interact(entity: &dyn Entity, player: &Player) -> InteractionResult {
    if player.is_secondary_use_active() {
        return InteractionResult::Pass;
    }

    let underwater_too_long = entity.as_abstract_boat().is_some_and(|boat| {
        boat.boat_state().lock().ticks_underwater >= UNDERWATER_TICKS_MAX
    });
    if underwater_too_long {
        return InteractionResult::Pass;
    }

    let Some(world) = entity.level() else {
        return InteractionResult::Pass;
    };
    let Some(shared) = world.get_entity_by_id(entity.id()) else {
        return InteractionResult::Pass;
    };
    if player.start_riding(&shared) {
        InteractionResult::SuccessServer
    } else {
        // Vanilla returns `PASS` when the seat is already taken.
        InteractionResult::Pass
    }
}

/// Vanilla `pushNearbyEntities`: boards small animals and pushes the rest.
///
/// Divergence: vanilla skips entity types tagged
/// `#minecraft:cannot_be_pushed_onto_boats`; Steel's generated entity-type tags
/// are not available here, so players are excluded explicitly (they are the
/// only vanilla member of that tag).
pub fn push_nearby_entities<T: AbstractBoat + Entity + 'static>(boat: &Arc<T>, world: &World) {
    let box_with_margin = boat.bounding_box().inflate_xyz(
        ENTITY_PUSH_EXPAND_H,
        ENTITY_PUSH_EXPAND_V,
        ENTITY_PUSH_EXPAND_H,
    );
    let boat_id = boat.id();
    let nearby = world.get_entities_in_aabb_matching(&box_with_margin, |entity| {
        entity.id() != boat_id && !entity.is_removed() && !entity.is_spectator()
    });
    if nearby.is_empty() {
        return;
    }

    // Vanilla only picks passengers up when no player is driving.
    let can_pick_up = boat
        .controlling_passenger()
        .is_none_or(|passenger| passenger.as_player().is_none());
    let boat_width = boat.bounding_box().max_x() - boat.bounding_box().min_x();

    for entity in nearby {
        if entity.has_passenger(boat.as_ref()) {
            continue;
        }

        let pickup_candidate = can_pick_up
            && boat.passengers().len() < boat.max_passengers()
            && entity.vehicle().is_none()
            && entity.bounding_box().max_x() - entity.bounding_box().min_x() < boat_width
            && entity.as_living_entity().is_some()
            && entity.as_player().is_none();

        if pickup_candidate {
            // Reuse the manager's already-erased handle: unsizing `Arc<T>` to `Arc<dyn Entity>`
            // is not possible for a `?Sized` generic parameter here.
            if let Some(shared) = world.get_entity_by_id(boat.id()) {
                let _ = entity.start_riding(&shared);
            }
        } else {
            boat.push_entity(entity.as_ref());
        }
    }
}

/// Applies the shared boat implementation to one entity struct.
///
/// `interact` is deliberately *not* generated: plain boats ride, chest boats
/// first ride and then fall back to opening their container, and both split the
/// shared part through [`ride_boat_interact`].
macro_rules! boat_entity_impl {
    ($name:ident, $max_passengers:expr) => {
        boat_entity_impl!($name, $max_passengers, ride_boat_interact);
    };
    ($name:ident, $max_passengers:expr, $interact:expr) => {
        boat_entity_impl!($name, $max_passengers, $interact;);
    };
    ($name:ident, $max_passengers:expr, $interact:expr; $($extra_boat:tt)*) => {
        impl VehicleEntity for $name {
            fn hurt_time(&self) -> i32 {
                *self.entity_data.lock().vehicle_entity().id_hurt.get()
            }

            fn set_hurt_time(&self, value: i32) {
                self.entity_data
                    .lock()
                    .vehicle_entity_mut()
                    .id_hurt
                    .set(value);
            }

            fn hurt_dir(&self) -> i32 {
                *self.entity_data.lock().vehicle_entity().id_hurtdir.get()
            }

            fn set_hurt_dir(&self, value: i32) {
                self.entity_data
                    .lock()
                    .vehicle_entity_mut()
                    .id_hurtdir
                    .set(value);
            }

            fn damage(&self) -> f32 {
                *self.entity_data.lock().vehicle_entity().id_damage.get()
            }

            fn set_damage(&self, value: f32) {
                self.entity_data
                    .lock()
                    .vehicle_entity_mut()
                    .id_damage
                    .set(value);
            }

            fn vehicle_drop_item(&self) -> ItemRef {
                self.boat_drop_item()
            }

            fn vehicle_destroy(&self, world: &World, _source: &DamageSource) {
                self.scatter_boat_contents(world);
                self.vehicle_destroy_with_item(world, self.vehicle_drop_item());
            }
        }

        impl AbstractBoat for $name {
            fn boat_state(&self) -> &SyncMutex<BoatState> {
                &self.boat_state
            }

            fn boat_data(&self) -> &SyncMutex<AbstractBoatEntityData> {
                &self.entity_data
            }

            $($extra_boat)*
        }

        impl Entity for $name {
            fn base(&self) -> &EntityBase {
                &self.base
            }

            fn interact(
                &self,
                player: &Player,
                _hand: InteractionHand,
                _location: DVec3,
            ) -> InteractionResult {
                ($interact)(self, player)
            }

            fn entity_type(&self) -> EntityTypeRef {
                self.entity_type
            }

            fn on_above_bubble_column(&self, drag_down: bool, pos: BlockPos) {
                self.boat_above_bubble_column(drag_down, pos);
            }

            fn on_inside_bubble_column(&self, drag_down: bool) {
                // Vanilla keeps the generic clamped push for boats inside a column.
                self.default_on_inside_bubble_column(drag_down);
            }

            fn tick(self: Arc<Self>) {
                boat_tick(self);
            }

            fn controlling_passenger(&self) -> Option<SharedEntity> {
                self.passengers().into_iter().find(|passenger| {
                    passenger.as_living_entity().is_some() && passenger.can_control_vehicle()
                })
            }

            fn can_add_passenger(&self, _passenger: &dyn Entity) -> bool {
                self.passengers().len() < $max_passengers
            }

            fn get_default_gravity(&self) -> f64 {
                GRAVITY_IN_WATER
            }

            fn movement_emission(&self) -> EntityMovementEmission {
                EntityMovementEmission::Events
            }

            fn is_pickable(&self) -> bool {
                !self.is_removed()
            }

            fn is_pushable(&self) -> bool {
                true
            }

            fn blocks_building(&self) -> bool {
                true
            }

            fn can_collide_with(&self, other: &dyn Entity) -> bool {
                Self::boat_can_collide(other)
            }

            fn dimension_changing_delay(&self) -> i32 {
                VEHICLE_DIMENSION_CHANGING_DELAY
            }

            fn hurt(&self, world: &World, source: &DamageSource, amount: f32) -> bool {
                self.vehicle_hurt(world, source, amount)
            }

            fn ignore_explosion(&self, explosion: &dyn Explosion) -> bool {
                vehicle_ignores_explosion(explosion)
            }

            fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
                Some(&self.entity_data)
            }

            fn save_additional(&self, nbt: &mut NbtCompound) {
                self.save_boat_extra(nbt);
            }

            fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
                self.load_boat_extra(nbt);
            }
        }

        impl $name {
            /// Vanilla `AbstractBoatEntity.canCollide`: boats never collide with
            /// their own passengers and ignore spectators.
            fn boat_can_collide(other: &dyn Entity) -> bool {
                !other.is_spectator() && !other.is_vehicle()
            }
        }
    };
}

pub(crate) use boat_entity_impl;

/// A vanilla `Boat` (any wood type).
#[entity_behavior(class = "Boat")]
pub struct BoatEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<AbstractBoatEntityData>,
    boat_state: SyncMutex<BoatState>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `BoatEntity`.
unsafe impl DowncastType for BoatEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/boat");
}

impl BoatEntity {
    /// Creates a boat entity.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            entity_data: SyncMutex::new(AbstractBoatEntityData::new()),
            boat_state: SyncMutex::new(BoatState::default()),
        }
    }

    /// Creates a boat entity from saved data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(AbstractBoatEntityData::new()),
            boat_state: SyncMutex::new(BoatState::default()),
        }
    }
}

boat_entity_impl!(BoatEntity, MAX_PASSENGERS);

/// A vanilla `Raft` (bamboo raft).
#[entity_behavior(class = "Raft")]
pub struct RaftEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<AbstractBoatEntityData>,
    boat_state: SyncMutex<BoatState>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `RaftEntity`.
unsafe impl DowncastType for RaftEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/raft");
}

impl RaftEntity {
    /// Creates a raft entity.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            entity_data: SyncMutex::new(AbstractBoatEntityData::new()),
            boat_state: SyncMutex::new(BoatState::default()),
        }
    }

    /// Creates a raft entity from saved data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(AbstractBoatEntityData::new()),
            boat_state: SyncMutex::new(BoatState::default()),
        }
    }
}

boat_entity_impl!(RaftEntity, MAX_PASSENGERS);

#[cfg(test)]
mod tests {
    use std::sync::Weak;

    use glam::DVec3;
    use steel_registry::{init_vanilla_registry, vanilla_entities, vanilla_items};

    use super::{AbstractBoat, BoatEntity, BoatLocation, BoatState, RaftEntity};
    use crate::world::World;

    fn oak_boat() -> BoatEntity {
        BoatEntity::new(
            &vanilla_entities::OAK_BOAT,
            1,
            DVec3::new(0.5, 64.0, 0.5),
            Weak::<World>::new(),
        )
    }

    #[test]
    fn a_new_boat_starts_in_the_air_without_paddling() {
        init_vanilla_registry();
        let boat = oak_boat();
        assert_eq!(boat.boat_state().lock().location, BoatLocation::InAir);
        assert!(!boat.paddle_moving(0));
        assert!(!boat.paddle_moving(1));
    }

    #[test]
    fn paddle_flags_live_in_the_synchronized_data() {
        init_vanilla_registry();
        let boat = oak_boat();
        boat.set_paddles_moving(true, false);
        assert!(boat.paddle_moving(0));
        assert!(!boat.paddle_moving(1));

        boat.set_paddles_moving(false, true);
        assert!(!boat.paddle_moving(0));
        assert!(boat.paddle_moving(1));
    }

    #[test]
    fn rider_input_is_stored_on_the_boat() {
        init_vanilla_registry();
        let boat = oak_boat();
        boat.set_inputs(true, false, true, false);
        let state = boat.boat_state().lock();
        assert!(state.pressing_left);
        assert!(!state.pressing_right);
        assert!(state.pressing_forward);
        assert!(!state.pressing_back);
    }

    #[test]
    fn the_drop_item_is_derived_from_the_entity_id() {
        init_vanilla_registry();
        let boat = oak_boat();
        assert_eq!(boat.boat_drop_item().key, vanilla_items::OAK_BOAT.key);

        let raft = RaftEntity::new(
            &vanilla_entities::BAMBOO_RAFT,
            2,
            DVec3::new(0.5, 64.0, 0.5),
            Weak::<World>::new(),
        );
        assert_eq!(raft.boat_drop_item().key, vanilla_items::BAMBOO_RAFT.key);
    }

    #[test]
    fn paddle_sounds_follow_the_location() {
        init_vanilla_registry();
        let boat = oak_boat();
        boat.boat_state().lock().location = BoatLocation::InAir;
        assert!(boat.paddle_sound().is_none());
        boat.boat_state().lock().location = BoatLocation::OnLand;
        assert!(boat.paddle_sound().is_some());
        boat.boat_state().lock().location = BoatLocation::InWater;
        assert!(boat.paddle_sound().is_some());
    }

    #[test]
    fn locations_report_their_water_state() {
        assert!(BoatLocation::InWater.is_water());
        assert!(BoatLocation::UnderWater.is_water());
        assert!(BoatLocation::UnderFlowingWater.is_underwater());
        assert!(!BoatLocation::OnLand.is_water());
        assert!(!BoatLocation::InAir.is_water());
    }

    #[test]
    fn the_default_state_is_in_air_with_unit_friction() {
        let state = BoatState::default();
        assert_eq!(state.location, BoatLocation::InAir);
        assert_eq!(state.nearby_slipperiness, 1.0);
    }
}
