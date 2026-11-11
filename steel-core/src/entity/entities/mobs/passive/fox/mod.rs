//! Vanilla fox entity.
//!
//! Foxes carry a coat variant (red, snow) and a flag byte that encodes sleeping,
//! sitting, crouching and interested states. They also track up to two trusted
//! players, which is what keeps a tamed-by-feeding fox from fleeing.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{
    EntityAttachmentPoint, EntityAttachments, EntityDimensions, EntityTypeRef,
};
use steel_registry::item_stack::ItemStack;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::FoxEntityData;
use steel_registry::vanilla_item_tags::ItemTag;
use steel_registry::{REGISTRY, TaggedRegistryExt, sound_events};
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey, Identifier};
use uuid::Uuid;

use crate::entity::ai::goal::{
    FloatGoal, FollowParentGoal, LookAtPlayerGoal, PanicGoal, RandomLookAroundGoal, TemptGoal,
    WaterAvoidingRandomStrollGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::{
    AgeableMob, AgeableMobBase, Animal, AnimalBase, DefaultEntityPush, Entity, EntityBase,
    EntityBaseLoad, EntityPose, EntitySpawnReason, EntitySyncedData, LivingEntity,
    LivingEntityBase, Mob, MobBase, PathfinderMob, SharedEntity, SpawnGroupData, SpawnRule,
    sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::world::World;

/// Vanilla `Fox.Type`: the two coat variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoxVariant {
    /// The common red fox.
    Red = 0,
    /// The snowy taiga fox.
    Snow = 1,
}

impl FoxVariant {
    /// Returns the variant for a numeric id, defaulting to red.
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        if id == 1 { Self::Snow } else { Self::Red }
    }

    /// Returns the numeric id written to entity data and NBT.
    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }

    /// Returns the asset name of the coat.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Snow => "snow",
        }
    }
}

/// The bits of vanilla `Fox.DATA_FLAGS_ID`.
pub mod flags {
    /// The fox is sleeping.
    pub const SLEEPING: i8 = 1;
    /// The fox is sitting.
    pub const SITTING: i8 = 1 << 1;
    /// The fox is crouching, which is how it pounces.
    pub const CROUCHING: i8 = 1 << 2;
    /// The fox has noticed something and is interested.
    pub const INTERESTED: i8 = 1 << 3;
    /// The fox is pouncing.
    pub const POUNCING: i8 = 1 << 4;
    /// The fox is defending itself.
    pub const DEFENDING: i8 = 1 << 5;
    /// The fox has picked up an item.
    pub const FACE_PLANTED: i8 = 1 << 6;
}

const FOX_BABY_PASSENGER_ATTACHMENTS: [EntityAttachmentPoint; 1] =
    [EntityAttachmentPoint::new(0.0, 0.4, 0.0)];
const FOX_BABY_WIDTH: f32 = 0.36;
const FOX_BABY_HEIGHT: f32 = 0.42;
const FOX_BABY_EYE_HEIGHT: f32 = 0.273;

const FOX_BABY_DIMENSIONS: EntityDimensions = EntityDimensions::new_with_attachments(
    FOX_BABY_WIDTH,
    FOX_BABY_HEIGHT,
    FOX_BABY_EYE_HEIGHT,
    EntityAttachments::new(&FOX_BABY_PASSENGER_ATTACHMENTS, &[], &[], &[]),
);

const FOX_STROLL_SPEED: f64 = 0.3;
const FOX_PANIC_SPEED: f64 = 2.2;
const FOX_FOLLOW_PARENT_SPEED: f64 = 1.25;
const FOX_TEMPT_SPEED: f64 = 1.0;
const FOX_LOOK_DISTANCE: f32 = 24.0;

#[entity_behavior(class = "Fox", spawn_rule)]
/// Vanilla fox with a coat variant, a state flag byte and trusted players.
pub struct FoxEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    animal_base: AnimalBase,
    entity_data: SyncMutex<FoxEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `FoxEntity`.
unsafe impl DowncastType for FoxEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/fox");
}

impl FoxEntity {
    /// Vanilla `SpawnPlacements` predicate: `Animal.checkAnimalSpawnRules`.
    pub(crate) const SPAWN_RULE: SpawnRule = |entity_type, level, spawn_reason, pos, random| {
        <Self as Animal>::check_animal_spawn_rules(entity_type, level, spawn_reason, pos, random)
    };

    /// Creates a new fox at runtime.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Reconstructs a fox from persisted base entity state.
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
        let ageable_base = AgeableMobBase::new();
        let animal_base = AnimalBase::new();
        AnimalBase::initialize_pathfinding_malus(&mob_base);
        let mut entity_data = FoxEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            // Vanilla `Fox.registerGoals`, priorities and speeds unchanged.
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(1, PanicGoal::new(FOX_PANIC_SPEED));
            goal_selector.add_goal(
                2,
                TemptGoal::new(FOX_TEMPT_SPEED, is_fox_food, false),
            );
            goal_selector.add_goal(3, FollowParentGoal::new(FOX_FOLLOW_PARENT_SPEED));
            goal_selector.add_goal(4, WaterAvoidingRandomStrollGoal::new(FOX_STROLL_SPEED));
            goal_selector.add_goal(
                9,
                LookAtPlayerGoal::new(f64::from(FOX_LOOK_DISTANCE) / 2.0),
            );
            goal_selector.add_goal(10, RandomLookAroundGoal::new());
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

    /// Sets the coat variant.
    pub fn set_variant(&self, variant: FoxVariant) {
        self.entity_data.lock().variant_type.set(variant.id());
    }

    /// Returns the coat variant, falling back to red for unknown ids.
    #[must_use]
    pub fn variant(&self) -> FoxVariant {
        FoxVariant::from_id(*self.entity_data.lock().variant_type.get())
    }

    /// Returns the raw flag byte.
    #[must_use]
    pub fn flags(&self) -> i8 {
        *self.entity_data.lock().flags.get()
    }

    /// Replaces the raw flag byte.
    pub fn set_flags(&self, value: i8) {
        self.entity_data.lock().flags.set(value);
    }

    /// Whether one of the flag bits is set.
    #[must_use]
    pub fn has_flag(&self, flag: i8) -> bool {
        self.flags() & flag != 0
    }

    /// Sets or clears one flag bit and returns the new flag byte.
    pub fn set_flag(&self, flag: i8, value: bool) -> i8 {
        let current = self.flags();
        let updated = if value { current | flag } else { current & !flag };
        self.set_flags(updated);
        updated
    }

    /// Whether the given player is one of this fox's trusted players.
    #[must_use]
    pub fn is_trusted(&self, uuid: Uuid) -> bool {
        let data = self.entity_data.lock();
        *data.trusted_0.get() == Some(uuid) || *data.trusted_1.get() == Some(uuid)
    }

    /// Adds a trusted player, replacing the first slot when both are taken.
    ///
    /// Vanilla fills the two slots in order and only replaces slot zero once the
    /// second is occupied, so a third trust simply overwrites the first.
    pub fn add_trusted(&self, uuid: Uuid) {
        let mut data = self.entity_data.lock();
        if data.trusted_0.get().is_none() {
            data.trusted_0.set(Some(uuid));
            return;
        }
        if data.trusted_1.get().is_none() {
            data.trusted_1.set(Some(uuid));
            return;
        }
        data.trusted_0.set(Some(uuid));
        data.trusted_1.set(None);
    }

    /// Forgets a trusted player in either slot.
    pub fn remove_trusted(&self, uuid: Uuid) {
        let mut data = self.entity_data.lock();
        if *data.trusted_0.get() == Some(uuid) {
            data.trusted_0.set(None);
        }
        if *data.trusted_1.get() == Some(uuid) {
            data.trusted_1.set(None);
        }
    }

    /// Number of trusted players, capped at two like vanilla.
    #[must_use]
    pub fn trusted_count(&self) -> usize {
        let data = self.entity_data.lock();
        usize::from(data.trusted_0.get().is_some()) + usize::from(data.trusted_1.get().is_some())
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        if let Some(display) = sync_dirty_mob_effects(&self.living_base, &self.entity_data) {
            self.entity_data
                .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
        }
    }
}

/// Whether a stack is a vanilla fox food item.
#[must_use]
pub fn is_fox_food(item_stack: &ItemStack) -> bool {
    REGISTRY
        .items
        .is_in_tag(item_stack.item(), &ItemTag::FOX_FOOD)
}

impl DefaultEntityPush for FoxEntity {}

impl Entity for FoxEntity {
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

    fn base_tick(&self) {
        Mob::base_tick_mob(self);
    }

    fn dimensions_for_pose(&self, _pose: EntityPose) -> EntityDimensions {
        let scale = LivingEntity::get_scale(self);
        if AgeableMob::is_baby(self) {
            FOX_BABY_DIMENSIONS.scale(scale)
        } else if self.entity_type.fixed {
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

    fn sound_source(&self) -> SoundSource {
        SoundSource::Neutral
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
        self.save_ageable_mob(nbt);
        self.save_animal(nbt);
        nbt.insert("Type", self.variant().name());
        nbt.insert("FoxFlags", i32::from(self.flags()));
        nbt.insert("TrustedCount", self.trusted_count() as i32);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);
        self.load_animal(nbt);

        if let Some(variant) = nbt.string("Type") {
            let variant = match variant.to_str().as_ref() {
                "snow" => FoxVariant::Snow,
                _ => FoxVariant::Red,
            };
            self.set_variant(variant);
        }
        if let Some(flags) = nbt.int("FoxFlags") {
            self.set_flags(i8::try_from(flags).unwrap_or(0));
        }
    }
}

impl LivingEntity for FoxEntity {
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

    fn sound_volume(&self) -> f32 {
        0.4
    }

    fn hurt_sound(&self, _source: &DamageSource) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_FOX_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_FOX_DEATH)
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

impl AgeableMob for FoxEntity {
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

    fn breed_variant_key(&self) -> Option<&Identifier> {
        None
    }

    fn set_breed_variant_key(&self, _key: &Identifier) -> bool {
        false
    }
}

impl Mob for FoxEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn custom_server_ai_step(&self) {
        Animal::custom_server_ai_step_animal(self);
    }

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_FOX_AMBIENT)
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
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

impl PathfinderMob for FoxEntity {}

impl Animal for FoxEntity {
    fn animal_base(&self) -> &AnimalBase {
        &self.animal_base
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        is_fox_food(item_stack)
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;
    use steel_registry::vanilla_items;

    use super::*;

    #[test]
    fn coat_variants_round_trip() {
        init_vanilla_registry();
        let fox = FoxEntity::new(&vanilla_entities::FOX, 1, DVec3::ZERO, Weak::new());
        fox.set_variant(FoxVariant::Snow);
        assert_eq!(fox.variant(), FoxVariant::Snow);
        assert_eq!(fox.variant().name(), "snow");
        fox.set_variant(FoxVariant::Red);
        assert_eq!(fox.variant().id(), 0);
    }

    #[test]
    fn flag_bits_are_set_and_cleared_independently() {
        init_vanilla_registry();
        let fox = FoxEntity::new(&vanilla_entities::FOX, 1, DVec3::ZERO, Weak::new());
        fox.set_flags(0);
        fox.set_flag(flags::SLEEPING, true);
        assert!(fox.has_flag(flags::SLEEPING));
        assert!(!fox.has_flag(flags::SITTING));
        fox.set_flag(flags::SITTING, true);
        assert_eq!(fox.flags(), flags::SLEEPING | flags::SITTING);
        fox.set_flag(flags::SLEEPING, false);
        assert!(!fox.has_flag(flags::SLEEPING));
        assert!(fox.has_flag(flags::SITTING));
    }

    #[test]
    fn only_two_players_stay_trusted() {
        init_vanilla_registry();
        let fox = FoxEntity::new(&vanilla_entities::FOX, 1, DVec3::ZERO, Weak::new());
        let first = Uuid::from_u128(10);
        let second = Uuid::from_u128(11);
        let third = Uuid::from_u128(12);
        fox.add_trusted(first);
        fox.add_trusted(second);
        assert_eq!(fox.trusted_count(), 2);
        assert!(fox.is_trusted(first));
        assert!(fox.is_trusted(second));

        fox.add_trusted(third);
        assert_eq!(fox.trusted_count(), 1, "the third trust evicts the first slot");
        assert!(fox.is_trusted(third));
        assert!(!fox.is_trusted(first));

        fox.remove_trusted(third);
        assert_eq!(fox.trusted_count(), 0);
    }

    #[test]
    fn sweet_berries_are_fox_food() {
        init_vanilla_registry();
        assert!(is_fox_food(&ItemStack::new(&vanilla_items::SWEET_BERRIES)));
        assert!(!is_fox_food(&ItemStack::new(&vanilla_items::STONE)));
    }
}
