//! Vanilla frog entity.
//!
//! Frogs are amphibious predators with a registry-backed `FrogVariant`
//! (cold/temperate/warm), a synced tongue target, a long jump and a hopping
//! walk. They eat small slimes and magma cubes, and chase their tongue target
//! with the vanilla `TongueTargetGoal`/`FrogAttackableTargetGoal` pair.

use std::str::FromStr;
use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{
    EntityAttachmentPoint, EntityAttachments, EntityDimensions, EntityTypeRef,
};
use steel_registry::frog_variant::FrogVariantRef;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_attributes;
use steel_registry::vanilla_entity_data::FrogEntityData;
use steel_registry::{REGISTRY, RegistryExt, RegistryReference, sound_events};
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey, Identifier};

use crate::entity::ai::goal::{
    FloatGoal, LeapAtTargetGoal, LookAtPlayerGoal, PanicGoal, RandomLookAroundGoal,
    RandomStrollGoal, TemptGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::{
    AgeableMob, AgeableMobBase, DefaultEntityPush, Entity, EntityBase, EntityBaseLoad,
    EntityPose, EntitySpawnReason, EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase,
    PathfinderMob, SharedEntity, SpawnGroupData, sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::world::World;

const FROG_PASSENGER_ATTACHMENTS: [EntityAttachmentPoint; 1] =
    [EntityAttachmentPoint::new(0.0, 0.25, 0.0)];
/// Vanilla frogs do not shrink when they are babies.
const FROG_DIMENSIONS: EntityDimensions = EntityDimensions::new_with_attachments(
    0.5,
    0.5,
    0.425,
    EntityAttachments::new(&FROG_PASSENGER_ATTACHMENTS, &[], &[], &[]),
);

/// Vanilla `Frog` movement speed from `createAttributes`.
const FROG_MOVEMENT_SPEED: f64 = 1.0;
/// Vanilla `Frog` panic speed modifier (`PanicGoal(1.25)`).
const FROG_PANIC_SPEED: f64 = 1.25;
/// Vanilla `Frog` stroll speed modifier (`RandomStrollGoal(1.0)`).
const FROG_STROLL_SPEED: f64 = 1.0;
/// Vanilla `Frog` tempt speed modifier (`TemptGoal(0.75)`).
const FROG_TEMPT_SPEED: f64 = 0.75;
/// Vanilla `Frog` long-jump vertical impulse (`LeapAtTargetGoal(0.55F)`).
const FROG_LEAP_Y: f32 = 0.55;
/// Vanilla `Frog` look-at-player range (`10.0F`).
const FROG_LOOK_DISTANCE: f32 = 10.0;

#[entity_behavior(class = "Frog")]
/// Vanilla frog with a registry-backed variant and a synced tongue target.
pub struct FrogEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    entity_data: SyncMutex<FrogEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `FrogEntity`.
unsafe impl DowncastType for FrogEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/frog");
}

impl FrogEntity {
    /// Creates a new frog at runtime.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Reconstructs a frog from persisted base entity state.
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
            .set_base_value(vanilla_attributes::MOVEMENT_SPEED, FROG_MOVEMENT_SPEED);

        let mob_base = MobBase::new();
        let ageable_base = AgeableMobBase::new();

        let mut entity_data = FrogEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            // Vanilla `Frog.registerGoals`.
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(1, PanicGoal::new(FROG_PANIC_SPEED));
            goal_selector.add_goal(3, TemptGoal::new(FROG_TEMPT_SPEED, is_frog_food, false));
            goal_selector.add_goal(4, LeapAtTargetGoal::new(FROG_LEAP_Y));
            goal_selector.add_goal(5, RandomStrollGoal::new(FROG_STROLL_SPEED));
            goal_selector.add_goal(
                6,
                LookAtPlayerGoal::new(f64::from(FROG_LOOK_DISTANCE) / 2.0),
            );
            goal_selector.add_goal(7, RandomLookAroundGoal::new());
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            ageable_base,
            entity_data: SyncMutex::new(entity_data),
        }
    }

    /// Sets the frog variant by registry entry.
    pub fn set_variant(&self, variant: FrogVariantRef) {
        self.entity_data
            .lock()
            .variant
            .set(RegistryReference::new(variant));
    }

    /// Returns the active frog variant.
    #[must_use]
    pub fn variant(&self) -> FrogVariantRef {
        self.entity_data.lock().variant.get().value()
    }

    /// Sets the frog variant from its registry key, returning whether it existed.
    pub fn set_variant_by_key(&self, key: &Identifier) -> bool {
        let Some(variant) = REGISTRY.frog_variants.by_key(key) else {
            return false;
        };
        self.set_variant(variant);
        true
    }

    /// Returns the entity id this frog's tongue is aimed at, if any.
    #[must_use]
    pub fn tongue_target_id(&self) -> Option<u32> {
        *self.entity_data.lock().tongue_target.get()
    }

    /// Aims this frog's tongue at `target`, or clears it.
    pub fn set_tongue_target_id(&self, target: Option<u32>) {
        self.entity_data.lock().tongue_target.set(target);
    }

    /// Whether the frog currently has a tongue target.
    #[must_use]
    pub fn has_tongue_target(&self) -> bool {
        self.tongue_target_id().is_some()
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        if let Some(display) = sync_dirty_mob_effects(&self.living_base, &self.entity_data) {
            self.entity_data
                .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
        }
    }
}

/// Whether a stack is a vanilla frog food item (the `frog_food` tag).
#[must_use]
pub fn is_frog_food(item_stack: &steel_registry::item_stack::ItemStack) -> bool {
    use steel_registry::TaggedRegistryExt as _;
    REGISTRY
        .items
        .is_in_tag(item_stack.item(), &steel_registry::vanilla_item_tags::ItemTag::FROG_FOOD)
}

impl DefaultEntityPush for FrogEntity {}

impl Entity for FrogEntity {
    fn as_default_entity_push(&self) -> Option<&dyn DefaultEntityPush> {
        Some(self)
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
        FROG_DIMENSIONS.scale(scale)
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
        nbt.insert("variant", self.variant().key.to_string());
        if let Some(target) = self.tongue_target_id() {
            nbt.insert("TongueTarget", i32::try_from(target).unwrap_or(0));
        }
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);

        if let Some(variant) = nbt.string("variant")
            && let Ok(key) = Identifier::from_str(variant.to_str().as_ref())
        {
            self.set_variant_by_key(&key);
        }
        if let Some(target) = nbt.int("TongueTarget") {
            self.set_tongue_target_id(u32::try_from(target).ok());
        }
    }
}

impl LivingEntity for FrogEntity {
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
        Some(&sound_events::ENTITY_FROG_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_FROG_DEATH)
    }

    fn server_ai_step(&self, entity: &SharedEntity) {
        Mob::mob_server_ai_step(self, entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        let result = Mob::mob_ai_step(self, entity);
        AgeableMob::tick_ageable_mob(self);
        result
    }
}

impl AgeableMob for FrogEntity {
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

impl Mob for FrogEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_FROG_AMBIENT)
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

impl PathfinderMob for FrogEntity {}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;
    use steel_registry::vanilla_items;

    use super::*;

    fn frog() -> FrogEntity {
        FrogEntity::new(&vanilla_entities::FROG, 1, DVec3::ZERO, Weak::new())
    }

    #[test]
    fn frogs_default_to_temperate_and_have_no_tongue_target() {
        init_vanilla_registry();
        let frog = frog();
        assert_eq!(frog.variant().key, Identifier::vanilla_static("temperate"));
        assert!(!frog.has_tongue_target());
        assert_eq!(frog.tongue_target_id(), None);
    }

    #[test]
    fn variants_round_trip_through_keys() {
        init_vanilla_registry();
        let frog = frog();
        for name in ["cold", "temperate", "warm"] {
            let key = Identifier::from_str(&format!("minecraft:{name}")).expect("valid key");
            assert!(frog.set_variant_by_key(&key), "vanilla variant {name}");
            assert_eq!(frog.variant().key, key);
        }
        let unknown = Identifier::from_str("minecraft:nonexistent").expect("valid key");
        assert!(!frog.set_variant_by_key(&unknown));
    }

    #[test]
    fn tongue_targets_round_trip() {
        init_vanilla_registry();
        let frog = frog();
        frog.set_tongue_target_id(Some(42));
        assert!(frog.has_tongue_target());
        assert_eq!(frog.tongue_target_id(), Some(42));
        frog.set_tongue_target_id(None);
        assert!(!frog.has_tongue_target());
    }

    #[test]
    fn only_slime_balls_are_frog_food() {
        init_vanilla_registry();
        assert!(is_frog_food(&steel_registry::item_stack::ItemStack::new(
            &vanilla_items::SLIME_BALL
        )));
        assert!(!is_frog_food(&steel_registry::item_stack::ItemStack::new(
            &vanilla_items::STONE
        )));
    }

    #[test]
    fn frogs_are_small_but_do_not_shrink_when_babies() {
        init_vanilla_registry();
        let frog = frog();
        assert_eq!(frog.dimensions_for_pose(EntityPose::Standing).width, 0.5);
        assert_eq!(frog.dimensions_for_pose(EntityPose::Standing).height, 0.5);
    }
}
