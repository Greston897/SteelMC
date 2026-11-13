//! Vanilla `net.minecraft.world.entity.animal.equine.Horse`.
//!
//! Built on the shared `Animal` skeleton with the vanilla `AbstractHorse` flags
//! (`id_flags`) mirrored into the generated entity data. Horse-specific
//! mechanics are intentionally not wired yet.
//!
//! Deviations (deliberate, documented in `PARITY-26.3-RU.md`):
//! * jumping, galloping, chest inventory and breeding stats are not implemented;
//!   taming by repeated mounting and saddling are.

use std::sync::Weak;

use glam::DVec3;
use simdnbt::{borrow, owned};
use steel_macros::entity_behavior;
use steel_registry::entity_data::EntityPose;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::sound_events;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_attributes;
use steel_registry::vanilla_item_tags::ItemTag;
use steel_registry::REGISTRY;
use steel_registry::vanilla_entity_data::HorseEntityData;
use steel_utils::locks::SyncMutex;
use steel_utils::types::{EquipmentSlot, InteractionHand};
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    FloatGoal, FollowParentGoal, LookAtPlayerGoal, PanicGoal, RandomLookAroundGoal,
    WaterAvoidingRandomStrollGoal,
};
use crate::behavior::InteractionResult;
use crate::entity::damage::DamageSource;
use crate::entity::{
    AgeableMob, AgeableMobBase, Animal, AnimalBase, DefaultEntityPush, Entity, EntityBase,
    EntityBaseLoad, EntityDimensions, EntityMovementEmission, EntitySyncedData, LivingEntity,
    LivingEntityBase, Mob, MobBase, PathfinderMob, SharedEntity, sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::world::World;

/// Vanilla `AbstractHorse.DATA_ID_FLAGS` bit: the horse is tamed.
const HORSE_FLAG_TAME: i8 = 0x02;
/// Vanilla `AbstractHorse.DATA_ID_FLAGS` bit: the horse is saddled.
const HORSE_FLAG_SADDLE: i8 = 0x04;
/// Vanilla `AbstractHorse` base movement speed.
const HORSE_MOVEMENT_SPEED: f64 = 0.225;

#[entity_behavior(class = "Horse")]
/// Vanilla `Horse`.
pub struct HorseEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    animal_base: AnimalBase,
    entity_data: SyncMutex<HorseEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `HorseEntity`.
unsafe impl DowncastType for HorseEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/horse");
}

impl HorseEntity {
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

        let mut entity_data = HorseEntityData::new();
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
            entity_data: SyncMutex::new(entity_data),
        }
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

    fn update_dirty_mob_effect_entity_data(&self) {
        if let Some(display) = sync_dirty_mob_effects(&self.living_base, &self.entity_data) {
            self.entity_data
                .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
        }
    }
}

impl DefaultEntityPush for HorseEntity {}

impl Entity for HorseEntity {
    fn as_default_entity_push(&self) -> Option<&dyn DefaultEntityPush> {
        Some(self)
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
        self.play_sound(&sound_events::ENTITY_HORSE_STEP, 0.15, 1.0);
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

impl LivingEntity for HorseEntity {
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
        Some(&sound_events::ENTITY_HORSE_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_HORSE_DEATH)
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
}

impl AgeableMob for HorseEntity {
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

impl Animal for HorseEntity {
    fn animal_base(&self) -> &AnimalBase {
        &self.animal_base
    }

    /// Vanilla `AbstractHorse.isFood`: `#minecraft:horse_food`.
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        REGISTRY
            .items
            .is_in_tag(item_stack.item(), &ItemTag::HORSE_FOOD)
    }
}

impl Mob for HorseEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    /// Vanilla `AbstractHorse.mobInteract`, reduced to the parts Steel models:
    /// an untamed horse is mounted and tamed by the player, a tamed one is
    /// saddled or ridden, and food still feeds it like any animal.
    fn mob_interact(&self, player: &Player, hand: InteractionHand) -> InteractionResult {
        let item_stack = {
            let inventory = player.inventory.lock();
            let item_stack = inventory.get_item_in_hand(hand);
            item_stack.copy_with_count(item_stack.count())
        };

        // Vanilla: feeding wins over every other interaction.
        if self.is_food(&item_stack) {
            return Animal::mob_interact_animal(self, player, hand);
        }

        if LivingEntity::is_equippable_in_slot(self, &item_stack, EquipmentSlot::Saddle) {
            let result = LivingEntity::interact_living_entity_with_equippable(self, player, hand);
            if result.consumes_action() {
                return result;
            }
        }

        if !AgeableMob::is_baby(self)
            && !player.is_secondary_use_active()
            && !self.is_vehicle()
            && let Some(world) = self.level()
            && let Some(vehicle) = world.get_entity_by_id(self.id())
        {
            // Vanilla `AbstractHorse.doPlayerRide`: mounting an untamed horse
            // tames it. Steel has no horse-stat minigame, so the roll is skipped.
            if !self.is_tamed() {
                self.set_tamed(true);
            }
            player.start_riding(&vehicle);
            return InteractionResult::Success;
        }

        InteractionResult::Pass
    }

    fn can_use_slot(&self, slot: EquipmentSlot) -> bool {
        slot != EquipmentSlot::Saddle || (Entity::is_alive(self) && !AgeableMob::is_baby(self))
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_HORSE_AMBIENT)
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl PathfinderMob for HorseEntity {}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::*;

    #[test]
    fn horse_tame_and_saddle_bits_are_independent() {
        init_vanilla_registry();
        let horse = HorseEntity::new(
            &vanilla_entities::HORSE,
            1,
            DVec3::ZERO,
            Weak::new(),
        );
        horse.set_tamed(true);
        assert!(horse.is_tamed());
        assert!(!horse.is_saddled());
        horse.set_saddled(true);
        assert!(horse.is_tamed() && horse.is_saddled());
        horse.set_tamed(false);
        assert!(!horse.is_tamed() && horse.is_saddled());
    }
}
