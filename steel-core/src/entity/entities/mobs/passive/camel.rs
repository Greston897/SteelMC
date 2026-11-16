//! Vanilla `net.minecraft.world.entity.animal.camel.Camel`.
//!
//! Built on the shared `Animal` skeleton with the vanilla `AbstractHorse` flag
//! byte mirrored into the generated entity data.
//!
//! The dash jump works: `START_RIDING_JUMP` fires `Camel.executeRidersJump`
//! (a long forward leap plus `entity.camel.dash`), the cooldown drives
//! `getJumpCooldown`, and the dash flag clears on landing.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * sitting poses and the two-passenger saddle are not implemented; the
//!   `last_pose_change_tick` synced field is kept.

use std::sync::Weak;

use glam::DVec3;
use simdnbt::{borrow, owned};
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_data::EntityPose;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::sound_events;
use steel_registry::vanilla_game_events;
use steel_registry::vanilla_attributes;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_item_tags::ItemTag;
use steel_registry::vanilla_entity_data::CamelEntityData;
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    FloatGoal, FollowParentGoal, LookAtPlayerGoal, PanicGoal, RandomLookAroundGoal,
    WaterAvoidingRandomStrollGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::{
    AgeableMob, AgeableMobBase, Animal, AnimalBase, DefaultEntityPush, Entity, EntityBase,
    EntityBaseLoad, EntityDimensions, EntityEventSource, EntityMovementEmission, EntitySyncedData,
    LivingEntity, LivingEntityBase, Mob, MobBase, PathfinderMob, SharedEntity,
    sync_dirty_mob_effects,
};
use crate::entity::mount::{SharedMountInventory, open_mount_menu};
use crate::physics::MoveResult;
use crate::player::Player;
use crate::world::World;
use crate::world::game_event::GameEventContext;

/// Vanilla `AbstractHorse.DATA_ID_FLAGS` bit: the horse is tamed.
const HORSE_FLAG_TAME: i8 = 0x02;
/// Vanilla `AbstractHorse.DATA_ID_FLAGS` bit: the horse is saddled.
const HORSE_FLAG_SADDLE: i8 = 0x04;
/// Vanilla `AbstractHorse` base movement speed.
const HORSE_MOVEMENT_SPEED: f64 = 0.225;

#[entity_behavior(class = "Camel")]
/// Vanilla `Horse`.
pub struct CamelEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    animal_base: AnimalBase,
    /// Vanilla `AbstractHorse.inventory`: saddle, body armor and storage.
    mount_inventory: SharedMountInventory,
    entity_data: SyncMutex<CamelEntityData>,
    /// Vanilla `Camel.dashCooldown`: ticks left before the next dash.
    dash_cooldown: SyncMutex<i32>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `CamelEntity`.
unsafe impl DowncastType for CamelEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/camel");
}

impl CamelEntity {
    /// Creates a new horse at runtime.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Reconstructs a horse from persisted base entity state.
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
            .set_base_value(vanilla_attributes::MOVEMENT_SPEED, HORSE_MOVEMENT_SPEED);

        let mob_base = MobBase::new();
        let ageable_base = AgeableMobBase::new();
        let animal_base = AnimalBase::new();
        AnimalBase::initialize_pathfinding_malus(&mob_base);

        let mut entity_data = CamelEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(1, PanicGoal::new(1.2));
            goal_selector.add_goal(4, FollowParentGoal::new(1.0));
            goal_selector.add_goal(6, WaterAvoidingRandomStrollGoal::new(0.7));
            goal_selector.add_goal(7, LookAtPlayerGoal::new(6.0));
            goal_selector.add_goal(8, RandomLookAroundGoal::new());
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            ageable_base,
            animal_base,
            mount_inventory: crate::entity::mount::shared_inventory(1),
            entity_data: SyncMutex::new(entity_data),
            dash_cooldown: SyncMutex::new(0),
        }
    }

    /// Vanilla `AbstractHorse.openCustomInventoryScreen`: the mount screen with
    /// the saddle slot, the body-armor slot and the storage rows.
    pub fn open_mount_inventory(&self, player: &Player) {
        if !self.is_tamed() {
            return;
        }
        let inventory = self.mount_inventory.lock();
        let container = std::sync::Arc::clone(inventory.container());
        let columns = inventory.columns();
        drop(inventory);
        open_mount_menu(self, player, &container, columns);
    }

    /// Vanilla `AbstractHorse` flag byte.
    #[must_use]
    pub fn horse_flags(&self) -> i8 {
        *self.entity_data.lock().abstract_horse().id_flags.get()
    }

    /// Sets the vanilla `AbstractHorse` flag byte.
    pub fn set_horse_flags(&self, flags: i8) {
        self.entity_data
            .lock()
            .abstract_horse_mut()
            .id_flags
            .set(flags);
    }

    /// Whether this horse is tamed.
    #[must_use]
    pub fn is_tamed(&self) -> bool {
        self.horse_flags() & HORSE_FLAG_TAME != 0
    }

    /// Marks this horse as tamed.
    pub fn set_tamed(&self, tamed: bool) {
        let flags = self.horse_flags();
        let flags = if tamed {
            flags | HORSE_FLAG_TAME
        } else {
            flags & !HORSE_FLAG_TAME
        };
        self.set_horse_flags(flags);
    }

    /// Whether this horse is saddled.
    #[must_use]
    pub fn is_saddled(&self) -> bool {
        self.horse_flags() & HORSE_FLAG_SADDLE != 0
    }

    /// Sets the saddled bit of the vanilla flag byte.
    pub fn set_saddled(&self, saddled: bool) {
        let flags = self.horse_flags();
        let flags = if saddled {
            flags | HORSE_FLAG_SADDLE
        } else {
            flags & !HORSE_FLAG_SADDLE
        };
        self.set_horse_flags(flags);
    }

    /// Vanilla `Camel.isDashing` (`DATA_DASH`).
    #[must_use]
    pub fn is_dashing(&self) -> bool {
        *self.entity_data.lock().camel().dash.get()
    }

    /// Sets the vanilla camel dash flag.
    pub fn set_dashing(&self, dash: bool) {
        self.entity_data.lock().camel_mut().dash.set(dash);
    }

    /// Vanilla `Camel.getLastPoseChangeTick` (`DATA_LAST_POSE_CHANGE_TICK`).
    #[must_use]
    pub fn last_pose_change_tick(&self) -> i64 {
        *self.entity_data.lock().camel().last_pose_change_tick.get()
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        if let Some(display) = sync_dirty_mob_effects(&self.living_base, &self.entity_data) {
            self.entity_data
                .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
        }
    }
}

impl DefaultEntityPush for CamelEntity {}

impl Entity for CamelEntity {
    fn as_default_entity_push(&self) -> Option<&dyn DefaultEntityPush> {
        Some(self)
    }

    /// Vanilla `AbstractHorse.openCustomInventoryScreen` reached through the
    /// spectator path.
    fn open_menu_for(&self, player: &Player) -> crate::behavior::InteractionResult {
        if !self.is_tamed() {
            return crate::behavior::InteractionResult::Pass;
        }
        self.open_mount_inventory(player);
        crate::behavior::InteractionResult::Success
    }

    fn is_hard_collision_relevant(&self) -> bool {
        false
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
        self.tick_dash();
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
            .unwrap_or(1.0) as f32
    }

    fn save_additional(&self, nbt: &mut owned::NbtCompound) {
        self.save_mob(nbt);
        self.save_ageable_mob(nbt);
        self.save_animal(nbt);
    }

    fn load_additional(&self, nbt: borrow::NbtCompound<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);
        self.load_animal(nbt);
    }
}

impl CamelEntity {
    /// Vanilla `Camel.getJumpCooldown`: the dash cooldown gates rider jumps.
    #[must_use]
    pub fn dash_cooldown(&self) -> i32 {
        *self.dash_cooldown.lock()
    }

    /// Sets the vanilla dash cooldown.
    pub fn set_dash_cooldown(&self, cooldown: i32) {
        *self.dash_cooldown.lock() = cooldown;
    }

    /// Vanilla `Camel.tick`: clear a stale dash flag and count the cooldown
    /// down, playing `entity.camel.dash_ready` when it runs out.
    pub fn tick_dash(&self) {
        if self.is_dashing() && self.dash_cooldown() < 50 && self.on_ground() {
            self.set_dashing(false);
        }
        let remaining = self.dash_cooldown();
        if remaining > 0 {
            self.set_dash_cooldown(remaining - 1);
            if remaining - 1 == 0
                && let Some(world) = self.level()
            {
                world.play_sound_at(
                    &sound_events::ENTITY_CAMEL_DASH_READY,
                    SoundSource::Neutral,
                    self.position(),
                    1.0,
                    1.0,
                    None,
                );
            }
        }
    }
}

impl LivingEntity for CamelEntity {
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
        Some(&sound_events::ENTITY_CAMEL_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_CAMEL_DEATH)
    }

    fn server_ai_step(&self, entity: &SharedEntity) {
        Mob::mob_server_ai_step(self, entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        let result = Mob::mob_ai_step(self, entity);
        AgeableMob::tick_ageable_mob(self);
        Animal::tick_animal_love(self);
        result
    }

    /// Vanilla `Camel.canJump`: a sitting camel and one on cooldown refuse jumps.
    fn can_jump_with_rider(&self) -> bool {
        self.is_saddled() && self.dash_cooldown() <= 0
    }

    /// Vanilla `Camel.onPlayerJump`: only a grounded camel takes the charge.
    fn on_player_jump(&self, jump_amount: i32) {
        if self.is_saddled() && self.dash_cooldown() <= 0 && self.on_ground() {
            let amount = jump_amount.max(0);
            self.living_base().set_jump_pending_scale(
                crate::entity::living_entity::player_jump_pending_scale(amount),
            );
        }
    }

    /// Vanilla `Camel.handleStartJump`: dash sound, action game event, dash flag.
    fn handle_start_jump(&self, _jump_scale: i32) {
        if let Some(world) = self.level() {
            world.play_sound_at(
                &sound_events::ENTITY_CAMEL_DASH,
                SoundSource::Neutral,
                self.position(),
                1.0,
                1.0,
                None,
            );
            world.game_event_at(
                &vanilla_game_events::ENTITY_ACTION,
                self.position(),
                &GameEventContext::new(Some(self.as_entity_event_source()), None),
            );
        }
        self.set_dashing(true);
    }

    /// Vanilla `Camel.executeRidersJump`: a 55-tick-cooldown dash that adds a
    /// long horizontal leap on top of the vertical jump power.
    fn execute_riders_jump(&self, amount: f32, _ridden_input: DVec3) {
        let movement_speed = self
            .attributes()
            .lock()
            .required_value(vanilla_attributes::MOVEMENT_SPEED) as f32;
        let jump_momentum = self.get_jump_power();
        let (yaw, _pitch) = self.rotation();
        let look = DVec3::new(
            -f64::from(steel_math::trig::sin(yaw.to_radians())),
            0.0,
            f64::from(steel_math::trig::cos(yaw.to_radians())),
        )
        .normalize_or_zero();
        let horizontal = 22.2222_f32 * amount * movement_speed * self.block_speed_factor();
        self.set_velocity(
            self.velocity()
                + look * f64::from(horizontal)
                + DVec3::new(0.0, f64::from(1.4285_f32 * amount * jump_momentum), 0.0),
        );
        self.mark_velocity_sync();
        self.set_dash_cooldown(55);
        self.set_dashing(true);
    }
}

impl AgeableMob for CamelEntity {
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

impl Animal for CamelEntity {
    fn animal_base(&self) -> &AnimalBase {
        &self.animal_base
    }

    /// Vanilla `Camel.isFood`: the item must be in `#minecraft:camel_food`.
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item().has_tag(&ItemTag::CAMEL_FOOD)
    }
}

impl Mob for CamelEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    /// Vanilla `AbstractHorse.mobInteract`: a tamed mount opens its inventory on
    /// a secondary use, everything else falls back to the shared animal
    /// interaction (feeding, saddling and riding).
    fn mob_interact(
        &self,
        player: &Player,
        hand: steel_utils::types::InteractionHand,
    ) -> crate::behavior::InteractionResult {
        if !Entity::is_vehicle(self) && !LivingEntity::is_baby(self) && self.is_tamed()
            && player.is_secondary_use_active()
        {
            self.open_mount_inventory(player);
            return crate::behavior::InteractionResult::Success;
        }
        Animal::mob_interact_animal(self, player, hand)
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_CAMEL_AMBIENT)
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl PathfinderMob for CamelEntity {}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::*;

    #[test]
    fn camel_tame_and_saddle_bits_are_independent() {
        init_vanilla_registry();
        let camel = CamelEntity::new(
            &vanilla_entities::CAMEL,
            1,
            DVec3::ZERO,
            Weak::new(),
        );
        camel.set_tamed(true);
        assert!(camel.is_tamed());
        assert!(!camel.is_saddled());
        camel.set_saddled(true);
        assert!(camel.is_tamed() && camel.is_saddled());
        camel.set_tamed(false);
        assert!(!camel.is_tamed() && camel.is_saddled());
    }
}
