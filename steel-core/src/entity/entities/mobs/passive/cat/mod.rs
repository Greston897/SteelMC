//! Vanilla cat entity.
//!
//! Cats are tamable felines with a registry-backed coat variant, a collar
//! colour, and a handful of synced pose flags (lying down and the intermediate
//! "relax" state that precedes sleep). They answer to the owner only, hiss at
//! creepers via their sound variant, and keep the vanilla temptation, panic and
//! breeding goal layout.

use std::str::FromStr;
use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::{NbtCompound, NbtTag};
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::cat_sound_variant::{CatAge, CatSoundVariantRef};
use steel_registry::cat_variant::CatVariantRef;
use steel_registry::dye_color::DyeColor;
use steel_registry::entity_type::{
    EntityAttachmentPoint, EntityAttachments, EntityDimensions, EntityTypeRef,
};
use steel_registry::item_stack::ItemStack;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::CatEntityData;
use steel_registry::vanilla_item_tags::ItemTag;
use steel_registry::{REGISTRY, RegistryReference, TaggedRegistryExt};
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::{DowncastType, DowncastTypeKey, Identifier, UuidExt};
use uuid::Uuid;

use crate::entity::ai::goal::{
    BreedGoal, FloatGoal, FollowParentGoal, LookAtPlayerGoal, MeleeAttackGoal, PanicGoal,
    RandomLookAroundGoal, TemptGoal, WaterAvoidingRandomStrollGoal,
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

/// Vanilla `TamableAnimal.DATA_FLAGS_ID` bits.
pub mod flags {
    /// The animal is sitting.
    pub const SITTING: i8 = 1;
    /// The animal is tamed and has an owner.
    pub const TAME: i8 = 1 << 2;
}

/// The default collar colour: vanilla stores red (id 14) on new cats.
const DEFAULT_COLLAR_COLOR: i32 = 14;

const CAT_BABY_PASSENGER_ATTACHMENTS: [EntityAttachmentPoint; 1] =
    [EntityAttachmentPoint::new(0.0, 0.34375, 0.0)];
const CAT_BABY_WIDTH: f32 = 0.3;
const CAT_BABY_HEIGHT: f32 = 0.35;
const CAT_BABY_EYE_HEIGHT: f32 = 0.34375;

const CAT_BABY_DIMENSIONS: EntityDimensions = EntityDimensions::new_with_attachments(
    CAT_BABY_WIDTH,
    CAT_BABY_HEIGHT,
    CAT_BABY_EYE_HEIGHT,
    EntityAttachments::new(&CAT_BABY_PASSENGER_ATTACHMENTS, &[], &[], &[]),
);

const CAT_STROLL_SPEED: f64 = 0.8;
const CAT_PANIC_SPEED: f64 = 1.33;
const CAT_FOLLOW_PARENT_SPEED: f64 = 1.0;
const CAT_TEMPT_SPEED: f64 = 0.6;
const CAT_ATTACK_SPEED: f64 = 1.33;
const CAT_BREED_SPEED: f64 = 1.0;
const CAT_LOOK_DISTANCE: f32 = 10.0;

#[entity_behavior(class = "Cat", spawn_rule)]
/// Vanilla tamable cat with a coat variant, collar colour and relax state.
pub struct CatEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    animal_base: AnimalBase,
    entity_data: SyncMutex<CatEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `CatEntity`.
unsafe impl DowncastType for CatEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/cat");
}

impl CatEntity {
    /// Vanilla `SpawnPlacements` predicate: `Animal.checkAnimalSpawnRules`.
    pub(crate) const SPAWN_RULE: SpawnRule = |entity_type, level, spawn_reason, pos, random| {
        <Self as Animal>::check_animal_spawn_rules(entity_type, level, spawn_reason, pos, random)
    };

    /// Creates a new cat at runtime.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Reconstructs a cat from persisted base entity state.
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
        let mut entity_data = CatEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            // Vanilla `Cat.registerGoals`.
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(1, FloatGoal::new(&mob_base));
            goal_selector.add_goal(1, PanicGoal::new(CAT_PANIC_SPEED));
            goal_selector.add_goal(1, TemptGoal::new(CAT_TEMPT_SPEED, is_cat_food, false));
            goal_selector.add_goal(2, BreedGoal::new(CAT_BREED_SPEED));
            goal_selector.add_goal(3, FollowParentGoal::new(CAT_FOLLOW_PARENT_SPEED));
            goal_selector.add_goal(
                5,
                WaterAvoidingRandomStrollGoal::new(CAT_STROLL_SPEED),
            );
            goal_selector.add_goal(6, LookAtPlayerGoal::new(f64::from(CAT_LOOK_DISTANCE) / 2.0));
            goal_selector.add_goal(7, RandomLookAroundGoal::new());
            goal_selector.add_goal(8, MeleeAttackGoal::new(CAT_ATTACK_SPEED, true));
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

    /// Sets the coat variant by registry entry.
    pub fn set_variant(&self, variant: CatVariantRef) {
        self.entity_data
            .lock()
            .variant
            .set(RegistryReference::new(variant));
    }

    /// Returns the active coat variant, falling back to the vanilla default.
    #[must_use]
    pub fn variant(&self) -> CatVariantRef {
        self.entity_data.lock().variant.get().value()
    }

    /// Sets the coat variant from its registry key, returning whether it existed.
    pub fn set_variant_by_key(&self, key: &Identifier) -> bool {
        let Some(variant) = REGISTRY.cat_variants.by_key(key) else {
            return false;
        };
        self.set_variant(variant);
        true
    }

    /// Sets the sound variant by registry entry.
    pub fn set_sound_variant(&self, sound_variant: CatSoundVariantRef) {
        self.entity_data
            .lock()
            .sound_variant
            .set(RegistryReference::new(sound_variant));
    }

    /// Returns the active sound variant, falling back to the vanilla default.
    #[must_use]
    pub fn sound_variant(&self) -> CatSoundVariantRef {
        self.entity_data.lock().sound_variant.get().value()
    }

    /// Sets the sound variant from its registry key.
    pub fn set_sound_variant_by_key(&self, key: &Identifier) {
        if let Some(sound_variant) = REGISTRY.cat_sound_variants.by_key(key) {
            self.set_sound_variant(sound_variant);
        }
    }

    /// Returns the vanilla per-age sound set of this cat.
    fn sounds(&self) -> &'static CatAge {
        let sound_variant = self.sound_variant();
        if AgeableMob::is_baby(self) {
            &sound_variant.baby_sounds
        } else {
            &sound_variant.adult_sounds
        }
    }

    /// Returns the collar colour, defaulting to vanilla red.
    #[must_use]
    pub fn collar_color(&self) -> DyeColor {
        let id = *self.entity_data.lock().collar_color.get();
        u8::try_from(id)
            .ok()
            .and_then(|index| DyeColor::VALUES.get(usize::from(index)).copied())
            .unwrap_or(DyeColor::Red)
    }

    /// Sets the collar colour.
    pub fn set_collar_color(&self, color: DyeColor) {
        self.entity_data.lock().collar_color.set(color.id());
    }

    /// Whether the cat is lying down (sleeping pose).
    #[must_use]
    pub fn is_lying(&self) -> bool {
        *self.entity_data.lock().is_lying.get()
    }

    /// Sets whether the cat lies down.
    pub fn set_lying(&self, lying: bool) {
        self.entity_data.lock().is_lying.set(lying);
    }

    /// Whether the cat is in the intermediate relax state before sleeping.
    #[must_use]
    pub fn is_relax_state_one(&self) -> bool {
        *self.entity_data.lock().relax_state_one.get()
    }

    /// Sets the intermediate relax state.
    pub fn set_relax_state_one(&self, relax: bool) {
        self.entity_data.lock().relax_state_one.set(relax);
    }

    /// Whether this cat is tamed.
    #[must_use]
    pub fn is_tame(&self) -> bool {
        self.tamable_flags() & flags::TAME != 0
    }

    fn tamable_flags(&self) -> i8 {
        *self.entity_data.lock().flags.get()
    }

    fn set_tamable_flags(&self, flags: i8) {
        self.entity_data.lock().flags.set(flags);
    }

    /// Whether the cat is sitting.
    #[must_use]
    pub fn is_sitting(&self) -> bool {
        self.tamable_flags() & flags::SITTING != 0
    }

    /// Sets the sitting flag.
    pub fn set_sitting(&self, sitting: bool) {
        let mut flags = self.tamable_flags();
        if sitting {
            flags |= flags::SITTING;
        } else {
            flags &= !flags::SITTING;
        }
        self.set_tamable_flags(flags);
    }

    /// Returns the owner UUID of this cat, if tamed.
    #[must_use]
    pub fn owner_uuid(&self) -> Option<Uuid> {
        *self.entity_data.lock().owneruuid.get()
    }

    /// Tames the cat to `owner`, which also lifts the tame flag.
    pub fn set_owner(&self, owner: Option<Uuid>) {
        {
            let mut data = self.entity_data.lock();
            data.owneruuid.set(owner);
        }
        let mut flags = self.tamable_flags();
        if owner.is_some() {
            flags |= flags::TAME;
        } else {
            flags &= !flags::TAME;
        }
        self.set_tamable_flags(flags);
    }

    /// Vanilla `Cat.isOwnedBy`.
    #[must_use]
    pub fn is_owned_by(&self, uuid: Option<Uuid>) -> bool {
        self.is_tame() && uuid.is_some() && self.owner_uuid() == uuid
    }

    /// Returns the vanilla ambient sound for this cat's variant and age.
    fn update_dirty_mob_effect_entity_data(&self) {
        if let Some(display) = sync_dirty_mob_effects(&self.living_base, &self.entity_data) {
            self.entity_data
                .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
        }
    }
}

/// Whether a stack is a vanilla cat food item.
#[must_use]
pub fn is_cat_food(item_stack: &ItemStack) -> bool {
    REGISTRY
        .items
        .is_in_tag(item_stack.item(), &ItemTag::CAT_FOOD)
}

impl DefaultEntityPush for CatEntity {}

impl Entity for CatEntity {
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
            CAT_BABY_DIMENSIONS.scale(scale)
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
        nbt.insert("CollarColor", self.collar_color().id() as i8);
        nbt.insert("variant", self.variant().key.to_string());
        nbt.insert("sound_variant", self.sound_variant().key.to_string());
        nbt.insert("CatFlags", i32::from(self.tamable_flags()));
        if let Some(owner) = self.owner_uuid() {
            nbt.insert("Owner", NbtTag::IntArray(owner.to_int_array().to_vec()));
        }
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);
        self.load_animal(nbt);

        if let Some(collar) = nbt.byte_value("CollarColor") {
            let id = i32::from(collar).clamp(0, 15);
            if let Ok(index) = usize::try_from(id)
                && let Some(color) = DyeColor::VALUES.get(index)
            {
                self.set_collar_color(*color);
            }
        }

        if let Some(variant) = nbt.string("variant")
            && let Ok(key) = Identifier::from_str(variant.to_str().as_ref())
        {
            self.set_variant_by_key(&key);
        }
        if let Some(sound_variant) = nbt.string("sound_variant")
            && let Ok(key) = Identifier::from_str(sound_variant.to_str().as_ref())
        {
            self.set_sound_variant_by_key(&key);
        }
        if let Some(cat_flags) = nbt.int_value("CatFlags") {
            self.set_tamable_flags(i8::try_from(cat_flags).unwrap_or(0));
        }

        if let Some(owner) = nbt.int_array("Owner")
            && let Some(uuid) = Uuid::from_int_array(&owner)
        {
            self.set_owner(Some(uuid));
        }
    }
}

impl LivingEntity for CatEntity {
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
        Some(self.sounds().hurt_sound)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(self.sounds().death_sound)
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

impl AgeableMob for CatEntity {
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

impl Mob for CatEntity {
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
        Some(self.sounds().ambient_sound)
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

impl PathfinderMob for CatEntity {}

impl Animal for CatEntity {
    fn animal_base(&self) -> &AnimalBase {
        &self.animal_base
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        is_cat_food(item_stack)
    }

    fn play_eating_sound(&self) {
        let _ = self.sounds().eat_sound;
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;
    use steel_registry::vanilla_items;

    use super::*;

    fn cat() -> CatEntity {
        CatEntity::new(&vanilla_entities::CAT, 1, DVec3::ZERO, Weak::new())
    }

    #[test]
    fn new_cats_start_untamed_with_a_red_collar() {
        init_vanilla_registry();
        let cat = cat();
        assert!(!cat.is_tame(), "wild cats start untamed");
        assert!(!cat.is_sitting(), "wild cats do not sit");
        assert!(!cat.is_lying(), "wild cats stay upright");
        assert!(!cat.is_relax_state_one());
        assert_eq!(cat.collar_color(), DyeColor::Red);
        assert_eq!(cat.owner_uuid(), None);
        assert!(
            !cat.is_owned_by(Some(Uuid::nil())),
            "untamed cats have no owner"
        );
    }

    #[test]
    fn taming_sets_the_tame_flag_and_an_owner() {
        init_vanilla_registry();
        let cat = cat();
        let owner = Uuid::from_u128(0x1234_5678_9abc_def0);
        cat.set_owner(Some(owner));
        assert!(cat.is_tame());
        assert_eq!(cat.owner_uuid(), Some(owner));
        assert!(cat.is_owned_by(Some(owner)));
        assert!(!cat.is_owned_by(Some(Uuid::nil())), "only the owner counts");

        cat.set_owner(None);
        assert!(!cat.is_tame(), "clearing the owner un-tames the cat");
        assert!(!cat.is_owned_by(Some(owner)));
    }

    #[test]
    fn sitting_toggles_independently_of_taming() {
        init_vanilla_registry();
        let cat = cat();
        cat.set_sitting(true);
        assert!(cat.is_sitting());
        assert!(!cat.is_tame(), "sitting alone does not tame a cat");

        cat.set_owner(Some(Uuid::from_u128(7)));
        assert!(cat.is_tame() && cat.is_sitting(), "taming keeps the pose");

        cat.set_sitting(false);
        assert!(!cat.is_sitting());
        assert!(cat.is_tame(), "standing up does not un-tame a cat");
    }

    #[test]
    fn collar_colour_round_trips_through_ids() {
        init_vanilla_registry();
        let cat = cat();
        for color in DyeColor::VALUES {
            cat.set_collar_color(color);
            assert_eq!(cat.collar_color(), color);
        }
    }

    #[test]
    fn lying_and_relax_flags_are_independent() {
        init_vanilla_registry();
        let cat = cat();
        cat.set_relax_state_one(true);
        assert!(cat.is_relax_state_one() && !cat.is_lying());
        cat.set_lying(true);
        assert!(cat.is_lying() && cat.is_relax_state_one());

        cat.set_relax_state_one(false);
        assert!(cat.is_lying() && !cat.is_relax_state_one());
        cat.set_lying(false);
        assert!(!cat.is_lying() && !cat.is_relax_state_one());
    }

    #[test]
    fn raw_cod_and_salmon_are_cat_food() {
        init_vanilla_registry();
        assert!(is_cat_food(&ItemStack::new(&vanilla_items::COD)));
        assert!(is_cat_food(&ItemStack::new(&vanilla_items::SALMON)));
        assert!(!is_cat_food(&ItemStack::new(&vanilla_items::STONE)));
    }

    #[test]
    fn variants_and_sounds_round_trip_through_nbt() {
        init_vanilla_registry();
        let cat = cat();
        let key = Identifier::from_str("minecraft:tabby").expect("valid identifier");
        assert!(cat.set_variant_by_key(&key), "tabby is a vanilla cat variant");
        assert_eq!(cat.variant().key, key);

        let mut nbt = NbtCompound::new();
        cat.save_additional(&mut nbt);
        assert_eq!(
            nbt.string("variant").map(|s| s.to_str().into_owned()),
            Some("minecraft:tabby".to_string())
        );
        assert_eq!(nbt.byte("CollarColor"), Some(14));
    }
}
