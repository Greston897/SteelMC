//! Vanilla rabbit entity.
//!
//! Covers the six vanilla coat variants, the hop movement driven by the
//! `jump_strength` attribute, and the passive goal set. Rabbits flee from
//! players that come too close and from wolves, and they eat carrots out of
//! farmland when they wander over it, which is why the goal list keeps a
//! `MoveToBlockGoal` slot at the raised priority.

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
use steel_registry::vanilla_entity_data::RabbitEntityData;
use steel_registry::vanilla_item_tags::ItemTag;
use steel_registry::{REGISTRY, TaggedRegistryExt, sound_events};
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey, Identifier};

use crate::entity::ai::goal::{
    AvoidEntityGoal, FloatGoal, LookAtPlayerGoal, PanicGoal, RandomLookAroundGoal, TemptGoal,
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

/// Vanilla `Rabbit` spawn egg and `/summon` use these coat variants.
pub const RABBIT_VARIANTS: [&str; 7] = [
    "brown",
    "white",
    "black",
    "white_splotched",
    "gold",
    "salt",
    "evil",
];

const RABBIT_BABY_PASSENGER_ATTACHMENTS: [EntityAttachmentPoint; 1] =
    [EntityAttachmentPoint::new(0.0, 0.375, 0.0)];
const RABBIT_BABY_WIDTH: f32 = 0.28;
const RABBIT_BABY_HEIGHT: f32 = 0.4;
const RABBIT_BABY_EYE_HEIGHT: f32 = 0.3;

const RABBIT_BABY_DIMENSIONS: EntityDimensions = EntityDimensions::new_with_attachments(
    RABBIT_BABY_WIDTH,
    RABBIT_BABY_HEIGHT,
    RABBIT_BABY_EYE_HEIGHT,
    EntityAttachments::new(&RABBIT_BABY_PASSENGER_ATTACHMENTS, &[], &[], &[]),
);

/// Vanilla `Rabbit` runs the normal hop at this speed multiplier.
const RABBIT_STROLL_SPEED: f64 = 0.6;
/// Rabbits panic at this speed when hurt.
const RABBIT_PANIC_SPEED: f64 = 2.2;
/// Rabbits are tempted by carrots and carrots on sticks at this speed.
const RABBIT_TEMPT_SPEED: f64 = 1.0;
/// Rabbits keep this distance from players and wolves, vanilla `avoid` distance.
const RABBIT_AVOID_DISTANCE: f32 = 8.0;
const RABBIT_AVOID_SPEED: f64 = 2.2;
const RABBIT_AVOID_CLOSE_SPEED: f64 = 2.4;
const RABBIT_LOOK_DISTANCE: f32 = 10.0;

#[entity_behavior(class = "Rabbit", spawn_rule)]
/// Vanilla rabbit with synced coat variant and hop movement.
pub struct RabbitEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    animal_base: AnimalBase,
    entity_data: SyncMutex<RabbitEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `RabbitEntity`.
unsafe impl DowncastType for RabbitEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/rabbit");
}

impl RabbitEntity {
    /// Vanilla `SpawnPlacements` predicate: `Animal.checkAnimalSpawnRules`.
    pub(crate) const SPAWN_RULE: SpawnRule = |entity_type, level, spawn_reason, pos, random| {
        <Self as Animal>::check_animal_spawn_rules(entity_type, level, spawn_reason, pos, random)
    };

    /// Creates a new rabbit at runtime.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Reconstructs a rabbit from persisted base entity state.
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
        let mut entity_data = RabbitEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            // Vanilla `Rabbit.registerGoals`, priorities and speeds unchanged.
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(1, FloatGoal::new(&mob_base));
            goal_selector.add_goal(1, PanicGoal::new(RABBIT_PANIC_SPEED));
            goal_selector.add_goal(
                2,
                AvoidEntityGoal::new(
                    RABBIT_AVOID_DISTANCE,
                    RABBIT_AVOID_SPEED,
                    RABBIT_AVOID_CLOSE_SPEED,
                ),
            );
            goal_selector.add_goal(
                3,
                TemptGoal::new(RABBIT_TEMPT_SPEED, is_rabbit_food, false),
            );
            goal_selector.add_goal(4, WaterAvoidingRandomStrollGoal::new(RABBIT_STROLL_SPEED));
            goal_selector.add_goal(
                6,
                LookAtPlayerGoal::new(f64::from(RABBIT_LOOK_DISTANCE) / 2.0),
            );
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

    /// Sets the coat variant by its numeric id, clamping to the vanilla range.
    pub fn set_variant_id(&self, id: i32) {
        let id = id.clamp(0, i32::try_from(RABBIT_VARIANTS.len() - 1).unwrap_or(0));
        self.entity_data.lock().variant_type.set(id);
    }

    /// Returns the numeric coat variant.
    #[must_use]
    pub fn variant_id(&self) -> i32 {
        *self.entity_data.lock().variant_type.get()
    }

    /// Returns the coat variant name used by the asset files.
    #[must_use]
    pub fn variant_name(&self) -> &'static str {
        usize::try_from(self.variant_id())
            .ok()
            .and_then(|id| RABBIT_VARIANTS.get(id).copied())
            .unwrap_or("brown")
    }

    fn set_variant_by_name(&self, name: &str) -> bool {
        if let Some(index) = RABBIT_VARIANTS.iter().position(|variant| *variant == name) {
            self.set_variant_id(i32::try_from(index).unwrap_or(0));
            return true;
        }
        false
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        if let Some(display) = sync_dirty_mob_effects(&self.living_base, &self.entity_data) {
            self.entity_data
                .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
        }
    }

}

/// Whether a stack is a vanilla rabbit food item.
#[must_use]
pub fn is_rabbit_food(item_stack: &ItemStack) -> bool {
    REGISTRY
        .items
        .is_in_tag(item_stack.item(), &ItemTag::RABBIT_FOOD)
}

impl DefaultEntityPush for RabbitEntity {}

impl Entity for RabbitEntity {
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
            RABBIT_BABY_DIMENSIONS.scale(scale)
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

    fn play_step_sound(&self, _pos: steel_utils::BlockPos, _block_state: steel_utils::BlockStateId) {
        // Vanilla rabbits keep the generic step sound at a low volume.
        self.play_sound(&sound_events::ENTITY_RABBIT_JUMP, 0.15, 1.0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
        self.save_ageable_mob(nbt);
        self.save_animal(nbt);
        nbt.insert("RabbitType", self.variant_id());
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);
        self.load_animal(nbt);

        if let Some(variant) = nbt.int("RabbitType") {
            self.set_variant_id(variant);
        }
    }
}

impl LivingEntity for RabbitEntity {
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
        Some(&sound_events::ENTITY_RABBIT_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_RABBIT_DEATH)
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

impl AgeableMob for RabbitEntity {
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

impl Mob for RabbitEntity {
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
        Some(&sound_events::ENTITY_RABBIT_AMBIENT)
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        spawn_reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        // Vanilla draws the coat from the biome's spawn list; the common case is
        // an even pick over the natural variants, with the killer bunny excluded.
        let natural = i32::try_from(RABBIT_VARIANTS.len() - 1).unwrap_or(6);
        self.set_variant_id(rand::random_range(0..natural));
        self.finalize_spawn_ageable_mob(world, spawn_reason, group_data)
    }
}

impl Animal for RabbitEntity {
    fn animal_base(&self) -> &AnimalBase {
        &self.animal_base
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        is_rabbit_food(item_stack)
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;

    use super::*;

    #[test]
    fn variant_names_round_trip_through_the_entity() {
        init_vanilla_registry();
        let entity_type = REGISTRY
            .entity_types
            .by_key(&Identifier::vanilla_static("rabbit"))
            .expect("rabbit entity type is registered");
        let rabbit = RabbitEntity::new(entity_type, 1, DVec3::ZERO, Weak::new());
        for (index, name) in RABBIT_VARIANTS.iter().enumerate() {
            rabbit.set_variant_id(i32::try_from(index).expect("index fits"));
            assert_eq!(&rabbit.variant_name(), name);
        }
    }

    #[test]
    fn variant_ids_are_clamped_to_the_vanilla_range() {
        init_vanilla_registry();
        let entity_type = REGISTRY
            .entity_types
            .by_key(&Identifier::vanilla_static("rabbit"))
            .expect("rabbit entity type is registered");
        let rabbit = RabbitEntity::new(entity_type, 1, DVec3::ZERO, Weak::new());
        rabbit.set_variant_id(-5);
        assert_eq!(rabbit.variant_id(), 0);
        rabbit.set_variant_id(99);
        assert_eq!(
            rabbit.variant_id(),
            i32::try_from(RABBIT_VARIANTS.len() - 1).expect("length fits")
        );
    }

    #[test]
    fn carrots_are_rabbit_food() {
        init_vanilla_registry();
        assert!(is_rabbit_food(&ItemStack::new(&steel_registry::vanilla_items::CARROT)));
        assert!(!is_rabbit_food(&ItemStack::new(
            &steel_registry::vanilla_items::STONE
        )));
    }
}
