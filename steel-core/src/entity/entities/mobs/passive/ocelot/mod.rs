//! Vanilla ocelot entity.
//!
//! Ocelots are skittish cats that flee from players until they are fed fish, at
//! which point the synced `trusting` flag flips and they stop running away.

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
use steel_registry::vanilla_entity_data::OcelotEntityData;
use steel_registry::vanilla_item_tags::ItemTag;
use steel_registry::{REGISTRY, TaggedRegistryExt, sound_events};
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    AvoidEntityGoal, FloatGoal, FollowParentGoal, LookAtPlayerGoal, PanicGoal,
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

const OCELOT_BABY_PASSENGER_ATTACHMENTS: [EntityAttachmentPoint; 1] =
    [EntityAttachmentPoint::new(0.0, 0.375, 0.0)];
const OCELOT_BABY_WIDTH: f32 = 0.3;
const OCELOT_BABY_HEIGHT: f32 = 0.35;
const OCELOT_BABY_EYE_HEIGHT: f32 = 0.234;

const OCELOT_BABY_DIMENSIONS: EntityDimensions = EntityDimensions::new_with_attachments(
    OCELOT_BABY_WIDTH,
    OCELOT_BABY_HEIGHT,
    OCELOT_BABY_EYE_HEIGHT,
    EntityAttachments::new(&OCELOT_BABY_PASSENGER_ATTACHMENTS, &[], &[], &[]),
);

const OCELOT_STROLL_SPEED: f64 = 0.8;
const OCELOT_PANIC_SPEED: f64 = 1.33;
const OCELOT_FOLLOW_PARENT_SPEED: f64 = 1.0;
const OCELOT_TEMPT_SPEED: f64 = 0.6;
const OCELOT_AVOID_DISTANCE: f32 = 16.0;
const OCELOT_AVOID_SPEED: f64 = 1.33;
const OCELOT_AVOID_CLOSE_SPEED: f64 = 1.33;
const OCELOT_LOOK_DISTANCE: f32 = 10.0;

#[entity_behavior(class = "Ocelot", spawn_rule)]
/// Vanilla ocelot with the synced trusting flag.
pub struct OcelotEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    animal_base: AnimalBase,
    entity_data: SyncMutex<OcelotEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `OcelotEntity`.
unsafe impl DowncastType for OcelotEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/ocelot");
}

impl OcelotEntity {
    /// Vanilla `SpawnPlacements` predicate: `Animal.checkAnimalSpawnRules`.
    pub(crate) const SPAWN_RULE: SpawnRule = |entity_type, level, spawn_reason, pos, random| {
        <Self as Animal>::check_animal_spawn_rules(entity_type, level, spawn_reason, pos, random)
    };

    /// Creates a new ocelot at runtime.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Reconstructs an ocelot from persisted base entity state.
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
        let mut entity_data = OcelotEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            // Vanilla `Ocelot.registerGoals`.
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(1, PanicGoal::new(OCELOT_PANIC_SPEED));
            goal_selector.add_goal(3, TemptGoal::new(OCELOT_TEMPT_SPEED, is_ocelot_food, false));
            goal_selector.add_goal(4, FollowParentGoal::new(OCELOT_FOLLOW_PARENT_SPEED));
            goal_selector.add_goal(5, WaterAvoidingRandomStrollGoal::new(OCELOT_STROLL_SPEED));
            goal_selector.add_goal(
                6,
                AvoidEntityGoal::new(
                    OCELOT_AVOID_DISTANCE,
                    OCELOT_AVOID_SPEED,
                    OCELOT_AVOID_CLOSE_SPEED,
                ),
            );
            goal_selector.add_goal(
                7,
                LookAtPlayerGoal::new(f64::from(OCELOT_LOOK_DISTANCE) / 2.0),
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

    /// Whether the ocelot trusts players.
    #[must_use]
    pub fn is_trusting(&self) -> bool {
        *self.entity_data.lock().trusting.get()
    }

    /// Sets the synced trusting flag.
    pub fn set_trusting(&self, trusting: bool) {
        self.entity_data.lock().trusting.set(trusting);
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        if let Some(display) = sync_dirty_mob_effects(&self.living_base, &self.entity_data) {
            self.entity_data
                .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
        }
    }
}

/// Whether a stack is a vanilla ocelot food item.
#[must_use]
pub fn is_ocelot_food(item_stack: &ItemStack) -> bool {
    REGISTRY
        .items
        .is_in_tag(item_stack.item(), &ItemTag::OCELOT_FOOD)
}

impl DefaultEntityPush for OcelotEntity {}

impl Entity for OcelotEntity {
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
            OCELOT_BABY_DIMENSIONS.scale(scale)
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
        nbt.insert("Trusting", self.is_trusting());
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);
        self.load_animal(nbt);

        if let Some(trusting) = nbt.byte("Trusting") {
            self.set_trusting(trusting != 0);
        }
    }
}

impl LivingEntity for OcelotEntity {
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
        Some(&sound_events::ENTITY_OCELOT_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_OCELOT_DEATH)
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

impl AgeableMob for OcelotEntity {
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

impl Mob for OcelotEntity {
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
        Some(&sound_events::ENTITY_OCELOT_AMBIENT)
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

impl PathfinderMob for OcelotEntity {}

impl Animal for OcelotEntity {
    fn animal_base(&self) -> &AnimalBase {
        &self.animal_base
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        is_ocelot_food(item_stack)
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;
    use steel_registry::vanilla_items;

    use super::*;

    fn ocelot() -> OcelotEntity {
        OcelotEntity::new(&vanilla_entities::OCELOT, 1, DVec3::ZERO, Weak::new())
    }

    #[test]
    fn trusting_starts_false_and_flips() {
        init_vanilla_registry();
        let ocelot = ocelot();
        assert!(!ocelot.is_trusting(), "wild ocelots do not trust players");
        ocelot.set_trusting(true);
        assert!(ocelot.is_trusting());
    }

    #[test]
    fn raw_cod_is_ocelot_food() {
        init_vanilla_registry();
        assert!(is_ocelot_food(&ItemStack::new(&vanilla_items::COD)));
        assert!(!is_ocelot_food(&ItemStack::new(&vanilla_items::STONE)));
    }
}
