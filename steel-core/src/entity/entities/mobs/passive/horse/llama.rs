//! Vanilla `net.minecraft.world.entity.animal.equine.Llama`.
//!
//! Built on the shared `Animal` skeleton with the vanilla `AbstractHorse` flags
//! (`id_flags`) mirrored into the generated entity data. Saddling and riding are
//! implemented; jumping, galloping and the chest inventory are not.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * jumping, galloping, the chest inventory and breeding stats are not
//!   implemented.

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
use steel_registry::vanilla_entity_data::LlamaEntityData;
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    FloatGoal, FollowParentGoal, LookAtPlayerGoal, PanicGoal, RandomLookAroundGoal,
    WaterAvoidingRandomStrollGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::mount::{SharedMountInventory, open_mount_menu};
use crate::entity::{
    AgeableMob, AgeableMobBase, Animal, AnimalBase, DefaultEntityPush, Entity, EntityBase,
    EntityBaseLoad, EntityDimensions, EntityMovementEmission, EntitySyncedData, LivingEntity,
    LivingEntityBase, Mob, MobBase, PathfinderMob, SharedEntity, sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::player::Player;
use crate::world::World;

/// Vanilla `AbstractHorse.DATA_ID_FLAGS` bit: the horse is tamed.
const HORSE_FLAG_TAME: i8 = 0x02;
/// Vanilla `AbstractHorse.DATA_ID_FLAGS` bit: the horse is saddled.
const HORSE_FLAG_SADDLE: i8 = 0x04;
/// Vanilla `AbstractHorse` base movement speed.
const HORSE_MOVEMENT_SPEED: f64 = 0.225;

#[entity_behavior(class = "Llama")]
/// Vanilla `Llama`.
pub struct LlamaEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    animal_base: AnimalBase,
    /// Vanilla `AbstractHorse.inventory`: saddle, body armor and storage.
    mount_inventory: SharedMountInventory,
    entity_data: SyncMutex<LlamaEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `LlamaEntity`.
unsafe impl DowncastType for LlamaEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/llama");
}

impl LlamaEntity {
    /// Creates a new llama at runtime.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Reconstructs a llama from persisted base entity state.
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

        let mut entity_data = LlamaEntityData::new();
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

    /// Vanilla `AbstractChestedHorse.hasChest`.
    #[must_use]
    pub fn has_chest(&self) -> bool {
        *self
            .entity_data
            .lock()
            .abstract_chested_horse()
            .id_chest
            .get()
    }

    /// Vanilla `Llama.getStrength`, the 1..5 storage-column count a chest opens.
    #[must_use]
    pub fn strength(&self) -> usize {
        (*self.entity_data.lock().llama().strength.get()).clamp(1, 5) as usize
    }

    /// Vanilla `AbstractChestedHorse.setChest`; the llama's column count comes
    /// from its strength instead of a fixed five.
    pub fn set_chest(&self, has_chest: bool) {
        self.entity_data
            .lock()
            .abstract_chested_horse_mut()
            .id_chest
            .set(has_chest);
        let columns = if has_chest { self.strength() } else { 1 };
        self.mount_inventory.lock().resize(columns);
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

    /// Vanilla `AbstractHorse.travel`: face the rider's look direction, damping
    /// the pitch so the mount does not nose-dive.
    fn set_ridden_rotation(&self, controller_yaw: f32, controller_pitch: f32) {
        self.set_rotation((controller_yaw, controller_pitch * 0.5));
        self.base.set_old_yaw_to_current();
        let yaw = self.rotation().0;
        self.set_y_body_rot(yaw);
        self.set_y_head_rot(yaw);
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        if let Some(display) = sync_dirty_mob_effects(&self.living_base, &self.entity_data) {
            self.entity_data
                .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
        }
    }
}

impl DefaultEntityPush for LlamaEntity {}

impl Entity for LlamaEntity {
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

    fn play_step_sound(
        &self,
        _pos: steel_utils::BlockPos,
        _block_state: steel_utils::BlockStateId,
    ) {
        self.play_sound(&sound_events::ENTITY_LLAMA_STEP, 0.15, 1.0);
    }

    fn save_additional(&self, nbt: &mut owned::NbtCompound) {
        self.save_mob(nbt);
        self.save_ageable_mob(nbt);
        self.save_animal(nbt);
        // Vanilla `AbstractChestedHorse.addAdditionalSaveData`.
        nbt.insert("ChestedHorse", self.has_chest());
        if self.has_chest() {
            self.mount_inventory.lock().save_items(nbt);
        }
    }

    fn load_additional(&self, nbt: borrow::NbtCompound<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);
        self.load_animal(nbt);
        let chest = nbt.byte("ChestedHorse").is_some_and(|b| b != 0);
        self.set_chest(chest);
        if chest {
            self.mount_inventory.lock().load_items(nbt);
        }
    }
}

impl LivingEntity for LlamaEntity {
    fn living_base(&self) -> &LivingEntityBase {
        &self.living_base
    }

    fn tick_ridden(&self, controller: &Player, _ridden_input: DVec3) {
        if !self.is_saddled() {
            return;
        }
        let (yaw, pitch) = controller.rotation();
        self.set_ridden_rotation(yaw, pitch);
    }

    fn ridden_speed(&self, _controller: &Player) -> f32 {
        self.living_base
            .attributes()
            .lock()
            .required_value(vanilla_attributes::MOVEMENT_SPEED) as f32
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
        Some(&sound_events::ENTITY_LLAMA_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_LLAMA_DEATH)
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
    /// Vanilla `AbstractHorse.canJump`: only a saddled mount accepts a charge.
    fn can_jump_with_rider(&self) -> bool {
        self.is_saddled()
    }

    /// Vanilla `AbstractHorse.handleStartJump`: the horse-family take-off sound.
    fn handle_start_jump(&self, _jump_scale: i32) {
        if !self.is_saddled() {
            return;
        }
        let Some(world) = self.level() else {
            return;
        };
        world.play_sound_at(
            &sound_events::ENTITY_HORSE_JUMP,
            SoundSource::Neutral,
            self.position(),
            0.4,
            1.0,
            None,
        );
    }
}

impl AgeableMob for LlamaEntity {
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

impl Animal for LlamaEntity {
    fn animal_base(&self) -> &AnimalBase {
        &self.animal_base
    }
}

impl Mob for LlamaEntity {
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
        if !Entity::is_vehicle(self)
            && !LivingEntity::is_baby(self)
            && self.is_tamed()
            && player.is_secondary_use_active()
        {
            self.open_mount_inventory(player);
            return crate::behavior::InteractionResult::Success;
        }

        // Vanilla `AbstractChestedHorse.mobInteract`: a chest opens the llama's
        // storage, whose width is its strength.
        if self.is_tamed() && !self.has_chest() {
            let held = {
                let inventory = player.inventory.lock();
                inventory.get_item_in_hand(hand).clone()
            };
            if held.is(&steel_registry::vanilla_items::CHEST) {
                self.set_chest(true);
                player.inventory.lock().shrink_item_in_hand(hand, 1);
                if let Some(world) = self.level() {
                    world.play_sound_at(
                        &steel_registry::sound_events::ENTITY_DONKEY_CHEST,
                        steel_protocol::packets::game::SoundSource::Neutral,
                        self.position(),
                        1.0,
                        1.0,
                        None,
                    );
                }
                return crate::behavior::InteractionResult::Success;
            }
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
        Some(&sound_events::ENTITY_LLAMA_AMBIENT)
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl PathfinderMob for LlamaEntity {}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::*;

    #[test]
    fn llama_tame_and_saddle_bits_are_independent() {
        init_vanilla_registry();
        let horse = LlamaEntity::new(&vanilla_entities::LLAMA, 1, DVec3::ZERO, Weak::new());
        horse.set_tamed(true);
        assert!(horse.is_tamed());
        assert!(!horse.is_saddled());
        horse.set_saddled(true);
        assert!(horse.is_tamed() && horse.is_saddled());
        horse.set_tamed(false);
        assert!(!horse.is_tamed() && horse.is_saddled());
    }
}
