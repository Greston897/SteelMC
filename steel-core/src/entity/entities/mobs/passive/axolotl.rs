//! Vanilla axolotl entity.
//!
//! Axolotls are amphibious predators with five colour variants (`lucy`, `wild`,
//! `gold`, `cyan`, `blue`), a `playing_dead` play-dead pose that halves incoming
//! damage and stops AI, a `from_bucket` flag and an attack buff applied after
//! killing a target. They hunt fish and drown-only mobs, and dry out on land.

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
use steel_registry::item_stack::ItemStack;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_attributes;
use steel_registry::vanilla_damage_types;
use steel_registry::vanilla_entity_data::AxolotlEntityData;
use steel_registry::vanilla_item_tags::ItemTag;
use steel_registry::{REGISTRY, TaggedRegistryExt, sound_events};
use steel_utils::locks::SyncMutex;
use steel_utils::types::InteractionHand;
use steel_utils::{DowncastType, DowncastTypeKey, Identifier};

use crate::behavior::InteractionResult;
use crate::entity::ai::goal::{MeleeAttackGoal, PanicGoal, RandomSwimmingGoal};
use crate::entity::ai::path::PathType;
use crate::entity::damage::DamageSource;
use crate::entity::entities::mobs::passive::fish::{
    check_water_animal_spawn_rules, tick_breathing, travel_in_water,
};
use crate::entity::{
    AgeableMob, AgeableMobBase, DefaultEntityPush, Entity, EntityBase, EntityBaseLoad,
    EntityMovementEmission, EntityPose, EntitySpawnReason, EntitySyncedData, LivingEntity,
    LivingEntityBase, Mob, MobBase, PathfinderMob, SharedEntity, SpawnGroupData, SpawnRule,
    sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::player::Player;
use crate::world::World;

/// Vanilla `Axolotl.Variant` colours, in registry-id order.
pub const AXOLOTL_VARIANTS: [&str; 5] = ["lucy", "wild", "gold", "cyan", "blue"];

const AXOLOTL_PASSENGER_ATTACHMENTS: [EntityAttachmentPoint; 1] =
    [EntityAttachmentPoint::new(0.0, 0.25, 0.0)];
const AXOLOTL_BABY_PASSENGER_ATTACHMENTS: [EntityAttachmentPoint; 1] =
    [EntityAttachmentPoint::new(0.0, 0.125, 0.0)];

const AXOLOTL_DIMENSIONS: EntityDimensions = EntityDimensions::new_with_attachments(
    0.75,
    0.42,
    0.2751,
    EntityAttachments::new(&AXOLOTL_PASSENGER_ATTACHMENTS, &[], &[], &[]),
);

const AXOLOTL_BABY_DIMENSIONS: EntityDimensions = EntityDimensions::new_with_attachments(
    0.375,
    0.21,
    0.09375,
    EntityAttachments::new(&AXOLOTL_BABY_PASSENGER_ATTACHMENTS, &[], &[], &[]),
);

/// Vanilla `Axolotl` movement speed from `createAttributes`.
const AXOLOTL_MOVEMENT_SPEED: f64 = 1.0;
/// Vanilla `Axolotl.registerGoals` attack speed (`MeleeAttackGoal(1.2, true)`).
const AXOLOTL_ATTACK_SPEED: f64 = 1.2;
/// Vanilla `Axolotl` swim-to-random-place speed modifier (`1.0`).
const AXOLOTL_SWIM_SPEED: f64 = 1.0;
/// Vanilla `Axolotl` swim-to-random-place interval.
const AXOLOTL_SWIM_INTERVAL: i32 = 20;
/// Vanilla `Axolotl` panic speed modifier (`1.25`).
const AXOLOTL_PANIC_SPEED: f64 = 1.25;
/// Vanilla `Axolotl` dry-out damage (`2.0` every 60 ticks on land).
const AXOLOTL_DRY_DAMAGE: f32 = 2.0;
/// Vanilla `Axolotl` regen while in water (`600` ticks per 1 HP).
const AXOLOTL_WATER_REGEN_INTERVAL: i32 = 600;
/// Vanilla `Axolotl` play-dead damage reduction (`0.5`).
const AXOLOTL_PLAY_DEAD_DAMAGE_REDUCTION: f32 = 0.5;

#[entity_behavior(class = "Axolotl", spawn_rule)]
/// Vanilla axolotl with variants, play-dead pose and dry-out damage.
pub struct AxolotlEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    entity_data: SyncMutex<AxolotlEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `AxolotlEntity`.
unsafe impl DowncastType for AxolotlEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/axolotl");
}

impl AxolotlEntity {
    /// Vanilla `SpawnPlacements` predicate: `WaterAnimal.checkSurfaceWaterAnimalSpawnRules`.
    pub(crate) const SPAWN_RULE: SpawnRule = check_water_animal_spawn_rules;

    /// Creates a new axolotl at runtime.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Reconstructs an axolotl from persisted base entity state.
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
            .set_base_value(vanilla_attributes::MOVEMENT_SPEED, AXOLOTL_MOVEMENT_SPEED);

        let mob_base = MobBase::new();
        mob_base
            .pathfinding_malus()
            .lock()
            .set(PathType::Water, 0.0);

        let mut entity_data = AxolotlEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            // Vanilla `Axolotl.registerGoals`.
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(1, PanicGoal::new(AXOLOTL_PANIC_SPEED));
            goal_selector.add_goal(2, MeleeAttackGoal::new(AXOLOTL_ATTACK_SPEED, true));
            goal_selector.add_goal(
                3,
                RandomSwimmingGoal::new(AXOLOTL_SWIM_SPEED, AXOLOTL_SWIM_INTERVAL),
            );
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            ageable_base: AgeableMobBase::new(),
            entity_data: SyncMutex::new(entity_data),
        }
    }

    /// Returns the numeric colour variant, clamped to the vanilla range.
    #[must_use]
    pub fn variant_id(&self) -> i32 {
        (*self.entity_data.lock().variant.get()).clamp(0, 4)
    }

    /// Returns the colour variant name used by the asset files.
    #[must_use]
    pub fn variant_name(&self) -> &'static str {
        usize::try_from(self.variant_id())
            .ok()
            .and_then(|index| AXOLOTL_VARIANTS.get(index).copied())
            .unwrap_or("lucy")
    }

    /// Sets the colour variant by its numeric id.
    pub fn set_variant_id(&self, id: i32) {
        self.entity_data.lock().variant.set(id.clamp(0, 4));
    }

    /// Whether the axolotl is playing dead.
    #[must_use]
    pub fn is_playing_dead(&self) -> bool {
        *self.entity_data.lock().playing_dead.get()
    }

    /// Sets the play-dead pose.
    pub fn set_playing_dead(&self, playing_dead: bool) {
        self.entity_data.lock().playing_dead.set(playing_dead);
    }

    /// Whether this axolotl came out of a bucket.
    #[must_use]
    pub fn is_from_bucket(&self) -> bool {
        *self.entity_data.lock().from_bucket.get()
    }

    /// Sets the bucket flag.
    pub fn set_from_bucket(&self, from_bucket: bool) {
        self.entity_data.lock().from_bucket.set(from_bucket);
    }

    fn handle_air_supply(&self, pre_tick_air_supply: i32) {
        tick_breathing(self, pre_tick_air_supply);
    }

    /// Vanilla `Axolotl.tick` dry-out and water regeneration.
    fn tick_dry_out(&self) {
        if self.is_in_water() {
            if self.get_health() < self.get_max_health()
                && self.tick_count() % AXOLOTL_WATER_REGEN_INTERVAL == 0
            {
                self.set_health(self.get_health() + 1.0);
            }
            return;
        }

        // On land, vanilla only dries out during rain-free weather.
        if self.tick_count() % 60 == 0
            && let Some(world) = self.level()
        {
            self.hurt(
                &world,
                &DamageSource::environment(&vanilla_damage_types::DRY_OUT),
                AXOLOTL_DRY_DAMAGE,
            );
        }
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        if let Some(display) = sync_dirty_mob_effects(&self.living_base, &self.entity_data) {
            self.entity_data
                .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
        }
    }
}

/// Whether a stack is a vanilla axolotl food item (the `axolotl_food` tag).
#[must_use]
pub fn is_axolotl_food(item_stack: &ItemStack) -> bool {
    REGISTRY
        .items
        .is_in_tag(item_stack.item(), &ItemTag::AXOLOTL_FOOD)
}

impl DefaultEntityPush for AxolotlEntity {}

impl Entity for AxolotlEntity {
    fn as_default_entity_push(&self) -> Option<&dyn DefaultEntityPush> {
        Some(self)
    }

    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn movement_emission(&self) -> EntityMovementEmission {
        EntityMovementEmission::Events
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn base_tick(&self) {
        let air_supply = self.air_supply();
        Mob::base_tick_mob(self);
        self.handle_air_supply(air_supply);
    }

    fn dimensions_for_pose(&self, _pose: EntityPose) -> EntityDimensions {
        let scale = LivingEntity::get_scale(self);
        if AgeableMob::is_baby(self) {
            AXOLOTL_BABY_DIMENSIONS.scale(scale)
        } else {
            AXOLOTL_DIMENSIONS.scale(scale)
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
        nbt.insert("Variant", self.variant_id());
        nbt.insert("FromBucket", u8::from(self.is_from_bucket()));
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);

        if let Some(variant) = nbt.int("Variant") {
            self.set_variant_id(variant);
        }
        if let Some(from_bucket) = nbt.byte("FromBucket") {
            self.set_from_bucket(from_bucket != 0);
        }
    }
}

impl LivingEntity for AxolotlEntity {
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
        Some(&sound_events::ENTITY_AXOLOTL_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_AXOLOTL_DEATH)
    }

    fn travel_in_water(
        &self,
        shared: &SharedEntity,
        input: DVec3,
        _base_gravity: f64,
        _is_falling: bool,
        _old_y: f64,
    ) -> Option<MoveResult> {
        let navigation_idle = self.mob_base.navigation().lock().is_done();
        travel_in_water(self, shared, input, navigation_idle)
    }

    fn server_ai_step(&self, entity: &SharedEntity) {
        Mob::mob_server_ai_step(self, entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        let result = Mob::mob_ai_step(self, entity);
        self.tick_dry_out();
        AgeableMob::tick_ageable_mob(self);
        result
    }
}

impl AgeableMob for AxolotlEntity {
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

    fn mob_interact_ageable(&self, player: &Player, hand: InteractionHand) -> InteractionResult {
        let item_stack = {
            let inventory = player.inventory.lock();
            let item_stack = inventory.get_item_in_hand(hand);
            item_stack.copy_with_count(item_stack.count())
        };
        if !is_axolotl_food(&item_stack) {
            return InteractionResult::Pass;
        }

        // Vanilla `Axolotl.mobInteract` breeds adults and speeds up babies.
        if self.can_age_up() {
            Mob::use_player_item(self, player, hand);
            let age = self.get_age();
            self.age_up(age - 6000, false);
            return InteractionResult::SuccessServer;
        }

        InteractionResult::Pass
    }
}

impl Mob for AxolotlEntity {
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
        if self.is_in_water() {
            Some(&sound_events::ENTITY_AXOLOTL_IDLE_WATER)
        } else {
            Some(&sound_events::ENTITY_AXOLOTL_IDLE_AIR)
        }
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

impl PathfinderMob for AxolotlEntity {}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;

    use super::*;

    fn axolotl() -> AxolotlEntity {
        AxolotlEntity::new(&vanilla_entities::AXOLOTL, 1, DVec3::ZERO, Weak::new())
    }

    #[test]
    fn new_axolotls_are_lucy_and_alive_on_land() {
        init_vanilla_registry();
        let axolotl = axolotl();
        assert_eq!(axolotl.variant_id(), 0);
        assert_eq!(axolotl.variant_name(), "lucy");
        assert!(!axolotl.is_playing_dead());
        assert!(!axolotl.is_from_bucket());
    }

    #[test]
    fn variants_round_trip_and_clamp() {
        init_vanilla_registry();
        let axolotl = axolotl();
        for (index, name) in AXOLOTL_VARIANTS.iter().enumerate() {
            axolotl.set_variant_id(i32::try_from(index).expect("index fits"));
            assert_eq!(&axolotl.variant_name(), name);
        }
        axolotl.set_variant_id(99);
        assert_eq!(axolotl.variant_name(), "blue");
        axolotl.set_variant_id(-5);
        assert_eq!(axolotl.variant_name(), "lucy");
    }

    #[test]
    fn play_dead_and_bucket_flags_are_independent() {
        init_vanilla_registry();
        let axolotl = axolotl();
        axolotl.set_playing_dead(true);
        assert!(axolotl.is_playing_dead() && !axolotl.is_from_bucket());

        axolotl.set_from_bucket(true);
        assert!(axolotl.is_from_bucket() && axolotl.is_playing_dead());

        axolotl.set_playing_dead(false);
        assert!(!axolotl.is_playing_dead() && axolotl.is_from_bucket());
    }

    #[test]
    fn play_dead_halves_incoming_damage() {
        assert!((AXOLOTL_PLAY_DEAD_DAMAGE_REDUCTION - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn baby_axolotls_are_half_sized() {
        init_vanilla_registry();
        let axolotl = axolotl();
        let adult = axolotl.dimensions_for_pose(EntityPose::Standing);
        axolotl.set_age(-1);
        let baby = axolotl.dimensions_for_pose(EntityPose::Standing);
        assert!(baby.width < adult.width && baby.height < adult.height);
    }
}
