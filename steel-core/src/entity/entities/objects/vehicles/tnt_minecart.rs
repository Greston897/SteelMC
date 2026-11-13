//! Vanilla TNT minecart.
//!
//! A `MinecartTNT` carries a fuse that triggers on activator rails and on
//! damage. Its explosion power depends on how fast the cart was moving
//! (`4` for the first speed tier, `+4` per extra tier, capped at `16` when the
//! cart is ignited while moving fast).

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_registry::entity_type::EntityTypeRef;
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::{DowncastType, DowncastTypeKey};

use super::abstract_minecart::{AbstractMinecart, AbstractMinecartBase};
use crate::entity::damage::DamageSource;
use crate::entity::{Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, SharedEntity};
use crate::world::{
    DefaultExplosionDamageCalculator, ExplosionInteraction, ExplosionOptions, World,
};

/// Vanilla `MinecartTNT` base explosion power.
const TNT_MINECART_BASE_POWER: f32 = 4.0;
/// Vanilla `MinecartTNT` maximum explosion power.
const TNT_MINECART_MAX_POWER: f32 = 16.0;
/// Vanilla `MinecartTNT` power growth per speed tier.
const TNT_MINECART_POWER_STEP: f32 = 4.0;
/// Vanilla `MinecartTNT` speed tier width.
const TNT_MINECART_SPEED_STEP: f64 = 5.0;
/// Vanilla `MinecartTNT` fuse length in ticks.
const TNT_MINECART_FUSE_TICKS: i32 = 80;

#[entity_behavior(class = "MinecartTNT")]
/// Vanilla TNT minecart with a speed-dependent explosion.
pub struct TntMinecartEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    minecart_base: AbstractMinecartBase,
    state: SyncMutex<TntMinecartState>,
}

#[derive(Debug)]
struct TntMinecartState {
    fuse: i32,
    /// Whether the fuse was lit by a player or a redstone pulse.
    ignited: bool,
}

impl Default for TntMinecartState {
    fn default() -> Self {
        // Vanilla `MinecartTNT.fuse = -1`: a freshly placed cart is not primed.
        Self {
            fuse: -1,
            ignited: false,
        }
    }
}

// SAFETY: This key is owned by Steel and uniquely identifies `TntMinecartEntity`.
unsafe impl DowncastType for TntMinecartEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/tnt_minecart");
}

impl TntMinecartEntity {
    /// Creates a new TNT minecart entity.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            minecart_base: AbstractMinecartBase::new(),
            state: SyncMutex::new(TntMinecartState::default()),
        }
    }

    /// Creates a TNT minecart entity from saved data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            minecart_base: AbstractMinecartBase::new(),
            state: SyncMutex::new(TntMinecartState::default()),
        }
    }

    /// Remaining fuse ticks; `-1` when the cart is not primed.
    #[must_use]
    pub fn fuse(&self) -> i32 {
        self.state.lock().fuse
    }

    /// Sets the remaining fuse ticks; `-1` defuses the cart.
    pub fn set_fuse(&self, fuse: i32) {
        let mut state = self.state.lock();
        state.fuse = fuse;
        state.ignited = fuse >= 0;
    }

    /// Whether the cart is currently primed.
    #[must_use]
    pub fn is_primed(&self) -> bool {
        self.fuse() >= 0
    }

    /// Vanilla `MinecartTNT.primeFuse`: light the cart with the standard fuse.
    pub fn prime_fuse(&self, ignited: bool) {
        let mut state = self.state.lock();
        if state.fuse >= 0 {
            return;
        }
        state.fuse = TNT_MINECART_FUSE_TICKS;
        state.ignited = ignited;
    }

    /// Vanilla `MinecartTNT.getExplosionPower`: `4` plus `4` per `5` blocks/s
    /// of horizontal speed, capped at `16`.
    #[must_use]
    pub fn explosion_power(&self) -> f32 {
        let horizontal_speed =
            f64::hypot(self.velocity().x, self.velocity().z) / 0.05;
        let power = (TNT_MINECART_BASE_POWER
            + (horizontal_speed / TNT_MINECART_SPEED_STEP) as f32 * TNT_MINECART_POWER_STEP)
            .floor();
        power.clamp(TNT_MINECART_BASE_POWER, TNT_MINECART_MAX_POWER)
    }

    /// Vanilla `MinecartTNT.explode`: detonate with the current power.
    pub fn explode(&self, world: &Arc<World>) {
        if !world.get_game_rule(&steel_registry::vanilla_game_rules::TNT_EXPLODES) {
            return;
        }
        let power = self.explosion_power();
        // Vanilla detonates at the cart's centre with a small upward offset.
        let center = self.position() + DVec3::new(0.0, 0.0625, 0.0);
        let mut options = ExplosionOptions::new(center, power, ExplosionInteraction::Tnt);
        if let Some(world_entity) = self.level().and_then(|level| level.get_entity_by_id(self.id()))
        {
            options.source = Some(world_entity);
        }
        options.immutable_block_calculator = Some(&DefaultExplosionDamageCalculator);
        world.explode(options);
        self.set_removed(crate::entity::RemovalReason::Killed);
    }
}

impl Entity for TntMinecartEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn movement_emission(&self) -> EntityMovementEmission {
        EntityMovementEmission::Events
    }

    fn tick(self: Arc<Self>) {
        if let Some(minecart) = self.as_abstract_minecart() {
            minecart.tick_minecart(&(self.clone() as SharedEntity));
        }

        let fuse = {
            let mut state = self.state.lock();
            if state.fuse > 0 {
                state.fuse -= 1;
            }
            state.fuse
        };

        if fuse == 0 {
            if let Some(world) = self.level() {
                self.explode(&world);
            }
        }
    }

    fn hurt(&self, world: &World, source: &DamageSource, amount: f32) -> bool {
        // Vanilla primes the fuse when the cart takes damage from a source
        // that is not fire, and only if the cart was not already primed.
        let ignites =
            source.damage_type.key != steel_registry::vanilla_damage_types::IN_FIRE.key
                && source.damage_type.key != steel_registry::vanilla_damage_types::ON_FIRE.key;
        if ignites && !self.is_primed() {
            self.prime_fuse(true);
        }
        // Minecarts are not living entities, so there is no health to subtract;
        // the fuse is the only damage reaction.
        let _ = (world, source, amount);
        false
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let fuse = self.fuse();
        if fuse >= 0 {
            nbt.insert("fuse", fuse);
        }
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        if let Some(fuse) = nbt.int_value("fuse") {
            self.set_fuse(fuse);
        }
    }
}

impl AbstractMinecart for TntMinecartEntity {
    fn minecart_base(&self) -> &AbstractMinecartBase {
        &self.minecart_base
    }

    fn activate_minecart(&self, _world: &World, _pos: steel_utils::BlockPos, powered: bool) {
        if powered {
            self.prime_fuse(false);
        }
    }
    fn custom_display_block(&self) -> Option<BlockStateId> {
        *self
            .entity_data
            .lock()
            .abstract_minecart()
            .id_custom_display_block
            .get()
    }

    fn set_custom_display_block(&self, value: Option<BlockStateId>) {
        self.entity_data
            .lock()
            .abstract_minecart_mut()
            .id_custom_display_block
            .set(value);
    }

    fn display_offset(&self) -> i32 {
        *self
            .entity_data
            .lock()
            .abstract_minecart()
            .id_display_offset
            .get()
    }

    fn set_display_offset(&self, value: i32) {
        self.entity_data
            .lock()
            .abstract_minecart_mut()
            .id_display_offset
            .set(value);
    }

}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;

    use super::*;

    fn cart() -> TntMinecartEntity {
        TntMinecartEntity::new(
            &vanilla_entities::TNT_MINECART,
            1,
            DVec3::new(0.5, 4.0, 0.5),
            Weak::new(),
        )
    }

    #[test]
    fn new_tnt_minecarts_are_defused() {
        init_vanilla_registry();
        let cart = cart();
        assert_eq!(cart.fuse(), -1);
        assert!(!cart.is_primed());
    }

    #[test]
    fn priming_uses_the_vanilla_fuse_length() {
        init_vanilla_registry();
        let cart = cart();
        cart.prime_fuse(true);
        assert!(cart.is_primed());
        assert_eq!(cart.fuse(), TNT_MINECART_FUSE_TICKS);
    }

    #[test]
    fn explosion_power_grows_with_speed_and_is_capped() {
        init_vanilla_registry();
        let cart = cart();
        assert!((cart.explosion_power() - 4.0).abs() < f32::EPSILON);

        cart.set_velocity(DVec3::new(5.0, 0.0, 0.0));
        let fast = cart.explosion_power();
        assert!(fast > 4.0, "moving carts explode harder: {fast}");

        cart.set_velocity(DVec3::new(1000.0, 0.0, 0.0));
        assert!(cart.explosion_power() <= TNT_MINECART_MAX_POWER);
    }
}
