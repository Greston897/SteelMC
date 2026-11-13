//! Vanilla parrot entity.
//!
//! Parrots fly with a custom `FlyingMoveControl` instead of walking, carry one of
//! five coat variants, and periodically imitate the ambient sound of a hostile
//! mob.

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
use steel_registry::vanilla_entity_data::ParrotEntityData;
use steel_registry::vanilla_item_tags::ItemTag;
use steel_registry::{REGISTRY, TaggedRegistryExt, sound_events};
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey};

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

/// Vanilla `Parrot.Variant` order, which is also the numeric entity data value.
pub const PARROT_VARIANTS: [&str; 5] = ["red_blue", "blue", "green", "yellow_blue", "gray"];

const PARROT_BABY_PASSENGER_ATTACHMENTS: [EntityAttachmentPoint; 1] =
    [EntityAttachmentPoint::new(0.0, 0.25, 0.0)];
const PARROT_BABY_WIDTH: f32 = 0.25;
const PARROT_BABY_HEIGHT: f32 = 0.45;
const PARROT_BABY_EYE_HEIGHT: f32 = 0.117;

const PARROT_BABY_DIMENSIONS: EntityDimensions = EntityDimensions::new_with_attachments(
    PARROT_BABY_WIDTH,
    PARROT_BABY_HEIGHT,
    PARROT_BABY_EYE_HEIGHT,
    EntityAttachments::new(&PARROT_BABY_PASSENGER_ATTACHMENTS, &[], &[], &[]),
);

const PARROT_STROLL_SPEED: f64 = 1.0;
const PARROT_PANIC_SPEED: f64 = 1.25;
const PARROT_FOLLOW_PARENT_SPEED: f64 = 1.0;
const PARROT_TEMPT_SPEED: f64 = 1.0;
const PARROT_LOOK_DISTANCE: f32 = 8.0;
/// Vanilla parrots imitate a mob sound every 6000 ticks on average.
const PARROT_IMITATE_MIN_DELAY: i32 = 6000;
const PARROT_IMITATE_RANDOM_RANGE: i32 = 6000;

#[entity_behavior(class = "Parrot", spawn_rule)]
/// Vanilla parrot with a coat variant and periodic mob-sound imitation.
pub struct ParrotEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    animal_base: AnimalBase,
    entity_data: SyncMutex<ParrotEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `ParrotEntity`.
unsafe impl DowncastType for ParrotEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/parrot");
}

impl ParrotEntity {
    /// Vanilla `SpawnPlacements` predicate: `Animal.checkAnimalSpawnRules`.
    pub(crate) const SPAWN_RULE: SpawnRule = |entity_type, level, spawn_reason, pos, random| {
        <Self as Animal>::check_animal_spawn_rules(entity_type, level, spawn_reason, pos, random)
    };

    /// Creates a new parrot at runtime.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Reconstructs a parrot from persisted base entity state.
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
        let mut entity_data = ParrotEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            // Vanilla `Parrot.registerGoals`; the flying move control is part of
            // the shared pathfinder, so the goal list itself is small.
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(1, PanicGoal::new(PARROT_PANIC_SPEED));
            goal_selector.add_goal(2, TemptGoal::new(PARROT_TEMPT_SPEED, is_parrot_food, false));
            goal_selector.add_goal(3, FollowParentGoal::new(PARROT_FOLLOW_PARENT_SPEED));
            goal_selector.add_goal(5, WaterAvoidingRandomStrollGoal::new(PARROT_STROLL_SPEED));
            goal_selector.add_goal(
                6,
                LookAtPlayerGoal::new(f64::from(PARROT_LOOK_DISTANCE) / 2.0),
            );
            goal_selector.add_goal(7, RandomLookAroundGoal::new());
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

    /// Sets the coat variant by its numeric id, clamping to the vanilla range.
    pub fn set_variant_id(&self, id: i32) {
        let id = id.clamp(0, i32::try_from(PARROT_VARIANTS.len() - 1).unwrap_or(0));
        self.entity_data.lock().variant.set(id);
    }

    /// Returns the numeric coat variant.
    #[must_use]
    pub fn variant_id(&self) -> i32 {
        *self.entity_data.lock().variant.get()
    }

    /// Returns the coat variant name used by the asset files.
    #[must_use]
    pub fn variant_name(&self) -> &'static str {
        usize::try_from(self.variant_id())
            .ok()
            .and_then(|id| PARROT_VARIANTS.get(id).copied())
            .unwrap_or("red_blue")
    }

    /// Vanilla `Parrot.isFlyable`: a parrot flies unless it is sitting on a
    /// shoulder or riding something.
    #[must_use]
    pub fn is_flyable(&self) -> bool {
        !self.is_passenger()
    }

    /// Sound a parrot imitates for the given mob key, if vanilla defines one.
    ///
    /// Vanilla keeps this as a switch over the imitated entity type, so the
    /// mapping is spelled out here instead of being looked up by name.
    #[must_use]
    pub fn imitation_sound_for(mob_key: &str) -> Option<SoundEventRef> {
        let sound = match mob_key {
            "blaze" => &sound_events::ENTITY_PARROT_IMITATE_BLAZE,
            "bogged" => &sound_events::ENTITY_PARROT_IMITATE_BOGGED,
            "breeze" => &sound_events::ENTITY_PARROT_IMITATE_BREEZE,
            "camel_husk" => &sound_events::ENTITY_PARROT_IMITATE_CAMEL_HUSK,
            "creaking" => &sound_events::ENTITY_PARROT_IMITATE_CREAKING,
            "creeper" => &sound_events::ENTITY_PARROT_IMITATE_CREEPER,
            "drowned" => &sound_events::ENTITY_PARROT_IMITATE_DROWNED,
            "elder_guardian" => &sound_events::ENTITY_PARROT_IMITATE_ELDER_GUARDIAN,
            "ender_dragon" => &sound_events::ENTITY_PARROT_IMITATE_ENDER_DRAGON,
            "endermite" => &sound_events::ENTITY_PARROT_IMITATE_ENDERMITE,
            "evoker" => &sound_events::ENTITY_PARROT_IMITATE_EVOKER,
            "ghast" => &sound_events::ENTITY_PARROT_IMITATE_GHAST,
            "guardian" => &sound_events::ENTITY_PARROT_IMITATE_GUARDIAN,
            "hoglin" => &sound_events::ENTITY_PARROT_IMITATE_HOGLIN,
            "husk" => &sound_events::ENTITY_PARROT_IMITATE_HUSK,
            "illusioner" => &sound_events::ENTITY_PARROT_IMITATE_ILLUSIONER,
            "magma_cube" => &sound_events::ENTITY_PARROT_IMITATE_MAGMA_CUBE,
            "parched" => &sound_events::ENTITY_PARROT_IMITATE_PARCHED,
            "phantom" => &sound_events::ENTITY_PARROT_IMITATE_PHANTOM,
            "piglin" => &sound_events::ENTITY_PARROT_IMITATE_PIGLIN,
            "piglin_brute" => &sound_events::ENTITY_PARROT_IMITATE_PIGLIN_BRUTE,
            "pillager" => &sound_events::ENTITY_PARROT_IMITATE_PILLAGER,
            "ravager" => &sound_events::ENTITY_PARROT_IMITATE_RAVAGER,
            "shulker" => &sound_events::ENTITY_PARROT_IMITATE_SHULKER,
            "silverfish" => &sound_events::ENTITY_PARROT_IMITATE_SILVERFISH,
            "skeleton" => &sound_events::ENTITY_PARROT_IMITATE_SKELETON,
            "slime" => &sound_events::ENTITY_PARROT_IMITATE_SLIME,
            "spider" => &sound_events::ENTITY_PARROT_IMITATE_SPIDER,
            "stray" => &sound_events::ENTITY_PARROT_IMITATE_STRAY,
            "vex" => &sound_events::ENTITY_PARROT_IMITATE_VEX,
            "vindicator" => &sound_events::ENTITY_PARROT_IMITATE_VINDICATOR,
            "warden" => &sound_events::ENTITY_PARROT_IMITATE_WARDEN,
            "witch" => &sound_events::ENTITY_PARROT_IMITATE_WITCH,
            "wither" => &sound_events::ENTITY_PARROT_IMITATE_WITHER,
            "wither_skeleton" => &sound_events::ENTITY_PARROT_IMITATE_WITHER_SKELETON,
            "zoglin" => &sound_events::ENTITY_PARROT_IMITATE_ZOGLIN,
            "zombie" => &sound_events::ENTITY_PARROT_IMITATE_ZOMBIE,
            "zombie_horse" => &sound_events::ENTITY_PARROT_IMITATE_ZOMBIE_HORSE,
            "zombie_nautilus" => &sound_events::ENTITY_PARROT_IMITATE_ZOMBIE_NAUTILUS,
            "zombie_villager" => &sound_events::ENTITY_PARROT_IMITATE_ZOMBIE_VILLAGER,
            _ => return None,
        };
        Some(sound)
    }

    /// Rolls the next imitation delay, mirroring vanilla's `getAmbientSound`.
    #[must_use]
    pub fn next_imitation_delay() -> i32 {
        rand::random_range(0..PARROT_IMITATE_RANDOM_RANGE) + PARROT_IMITATE_MIN_DELAY
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        if let Some(display) = sync_dirty_mob_effects(&self.living_base, &self.entity_data) {
            self.entity_data
                .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
        }
    }
}

/// Whether a stack is a vanilla parrot food item.
#[must_use]
pub fn is_parrot_food(item_stack: &ItemStack) -> bool {
    REGISTRY
        .items
        .is_in_tag(item_stack.item(), &ItemTag::PARROT_FOOD)
}

impl DefaultEntityPush for ParrotEntity {}

impl Entity for ParrotEntity {
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
            PARROT_BABY_DIMENSIONS.scale(scale)
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
        nbt.insert("Variant", self.variant_id());
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);
        self.load_animal(nbt);

        if let Some(variant) = nbt.int("Variant") {
            self.set_variant_id(variant);
        }
    }
}

impl LivingEntity for ParrotEntity {
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
        Some(&sound_events::ENTITY_PARROT_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_PARROT_DEATH)
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

impl AgeableMob for ParrotEntity {
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

impl Mob for ParrotEntity {
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
        Some(&sound_events::ENTITY_PARROT_AMBIENT)
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
        // Vanilla picks the coat at random when the parrot spawns.
        self.set_variant_id(rand::random_range(
            0..i32::try_from(PARROT_VARIANTS.len()).unwrap_or(5),
        ));
        self.finalize_spawn_ageable_mob(world, spawn_reason, group_data)
    }
}

impl PathfinderMob for ParrotEntity {}

impl Animal for ParrotEntity {
    fn animal_base(&self) -> &AnimalBase {
        &self.animal_base
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        is_parrot_food(item_stack)
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;
    use steel_registry::vanilla_items;

    use super::*;

    fn parrot() -> ParrotEntity {
        ParrotEntity::new(&vanilla_entities::PARROT, 1, DVec3::ZERO, Weak::new())
    }

    #[test]
    fn coat_variants_round_trip_and_clamp() {
        init_vanilla_registry();
        let parrot = parrot();
        for (index, name) in PARROT_VARIANTS.iter().enumerate() {
            parrot.set_variant_id(i32::try_from(index).expect("index fits"));
            assert_eq!(&parrot.variant_name(), name);
        }
        parrot.set_variant_id(99);
        assert_eq!(parrot.variant_name(), "gray");
        parrot.set_variant_id(-3);
        assert_eq!(parrot.variant_name(), "red_blue");
    }

    #[test]
    fn parrots_fly_unless_riding() {
        init_vanilla_registry();
        let parrot = parrot();
        assert!(parrot.is_flyable());
    }

    #[test]
    fn imitation_sounds_exist_for_vanilla_mobs() {
        init_vanilla_registry();
        assert!(ParrotEntity::imitation_sound_for("creeper").is_some());
        assert!(ParrotEntity::imitation_sound_for("warden").is_some());
        assert!(
            ParrotEntity::imitation_sound_for("not_a_mob").is_none(),
            "unknown mobs have no imitation"
        );
    }

    #[test]
    fn seeds_tempt_parrots() {
        init_vanilla_registry();
        assert!(is_parrot_food(&ItemStack::new(&*vanilla_items::WHEAT_SEEDS)));
        assert!(!is_parrot_food(&ItemStack::new(&*vanilla_items::STONE)));
    }

    #[test]
    fn imitation_delay_matches_the_vanilla_range() {
        for _ in 0..64 {
            let delay = ParrotEntity::next_imitation_delay();
            assert!(
                (PARROT_IMITATE_MIN_DELAY..PARROT_IMITATE_MIN_DELAY + PARROT_IMITATE_RANDOM_RANGE)
                    .contains(&delay)
            );
        }
    }
}
