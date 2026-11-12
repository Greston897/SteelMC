//! Vanilla dolphin entity.
//!
//! Dolphins are water creatures with two synced fields: `got_fish` (the reward
//! state after a player feeds them) and `moistness_level` (a 2400-tick counter
//! that drains on land and damages the dolphin at zero). They hunt fish, leap
//! out of water and follow boats.

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
use steel_registry::vanilla_entity_data::DolphinEntityData;
use steel_registry::vanilla_item_tags::ItemTag;
use steel_registry::{REGISTRY, TaggedRegistryExt, sound_events};
use steel_utils::locks::SyncMutex;
use steel_utils::types::InteractionHand;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::behavior::InteractionResult;
use crate::entity::ai::goal::{LookAtPlayerGoal, RandomLookAroundGoal, RandomSwimmingGoal};
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

const DOLPHIN_PASSENGER_ATTACHMENTS: [EntityAttachmentPoint; 1] =
    [EntityAttachmentPoint::new(0.0, 0.5, 0.0)];
const DOLPHIN_BABY_PASSENGER_ATTACHMENTS: [EntityAttachmentPoint; 1] =
    [EntityAttachmentPoint::new(0.0, 0.325, 0.0)];

const DOLPHIN_DIMENSIONS: EntityDimensions = EntityDimensions::new_with_attachments(
    0.9,
    0.6,
    0.3,
    EntityAttachments::new(&DOLPHIN_PASSENGER_ATTACHMENTS, &[], &[], &[]),
);

const DOLPHIN_BABY_DIMENSIONS: EntityDimensions = EntityDimensions::new_with_attachments(
    0.585,
    0.39,
    0.09375,
    EntityAttachments::new(&DOLPHIN_BABY_PASSENGER_ATTACHMENTS, &[], &[], &[]),
);

/// Vanilla `Dolphin` movement speed from `createAttributes`.
const DOLPHIN_MOVEMENT_SPEED: f64 = 1.2;
/// Vanilla `Dolphin` swim-to-random-place speed (`SwimToRandomPlaceGoal(1.3)`).
const DOLPHIN_SWIM_SPEED: f64 = 1.3;
/// Vanilla `Dolphin` swim interval.
const DOLPHIN_SWIM_INTERVAL: i32 = 10;
/// Vanilla `Dolphin.MOISTNESS_LEVEL_DEFAULT`.
pub const DOLPHIN_MOISTNESS_DEFAULT: i32 = 2400;
/// Vanilla `Dolphin` moistness drain per tick on land.
const DOLPHIN_MOISTNESS_DRAIN: i32 = 1;
/// Vanilla `Dolphin` moistness refill per tick in water.
const DOLPHIN_MOISTNESS_REFILL: i32 = 4;
/// Vanilla `Dolphin` dry damage per tick once moistness hits zero.
const DOLPHIN_DRY_DAMAGE: f32 = 1.0;

#[entity_behavior(class = "Dolphin", spawn_rule)]
/// Vanilla dolphin with moistness tracking and a fish reward flag.
pub struct DolphinEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    entity_data: SyncMutex<DolphinEntityData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `DolphinEntity`.
unsafe impl DowncastType for DolphinEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/dolphin");
}

impl DolphinEntity {
    /// Vanilla `SpawnPlacements` predicate: `WaterAnimal.checkSurfaceWaterAnimalSpawnRules`.
    pub(crate) const SPAWN_RULE: SpawnRule = check_water_animal_spawn_rules;

    /// Creates a new dolphin at runtime.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Reconstructs a dolphin from persisted base entity state.
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
            .set_base_value(vanilla_attributes::MOVEMENT_SPEED, DOLPHIN_MOVEMENT_SPEED);

        let mob_base = MobBase::new();
        mob_base
            .pathfinding_malus()
            .lock()
            .set(PathType::Water, 0.0);

        let mut entity_data = DolphinEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            // Vanilla `Dolphin.registerGoals` (the water-relevant subset).
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(
                0,
                RandomSwimmingGoal::new(DOLPHIN_SWIM_SPEED, DOLPHIN_SWIM_INTERVAL),
            );
            goal_selector.add_goal(6, LookAtPlayerGoal::new(6.0));
            goal_selector.add_goal(7, RandomLookAroundGoal::new());
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

    /// Whether this dolphin has been rewarded with a fish recently.
    #[must_use]
    pub fn got_fish(&self) -> bool {
        *self.entity_data.lock().got_fish.get()
    }

    /// Sets the vanilla `got_fish` reward flag.
    pub fn set_got_fish(&self, got_fish: bool) {
        self.entity_data.lock().got_fish.set(got_fish);
    }

    /// Returns the ticks of moisture left before the dolphin dries out.
    #[must_use]
    pub fn moistness_level(&self) -> i32 {
        *self.entity_data.lock().moistness_level.get()
    }

    /// Sets the ticks of moisture left.
    pub fn set_moistness_level(&self, level: i32) {
        self.entity_data.lock().moistness_level.set(level);
    }


    /// Vanilla `Dolphin.tickMoistness`.
    fn tick_moistness(&self) {
        if self.is_in_water_or_rain() {
            let refilled = self.moistness_level() + DOLPHIN_MOISTNESS_REFILL;
            self.set_moistness_level(refilled.min(DOLPHIN_MOISTNESS_DEFAULT));
            return;
        }

        let remaining = self.moistness_level() - DOLPHIN_MOISTNESS_DRAIN;
        self.set_moistness_level(remaining);
        if remaining <= -20
            && self.tick_count() % 10 == 0
            && let Some(world) = self.level()
        {
            self.hurt(
                &world,
                &DamageSource::environment(&vanilla_damage_types::DROWN),
                DOLPHIN_DRY_DAMAGE,
            );
        }
    }

    fn is_in_water_or_rain(&self) -> bool {
        self.is_in_water()
    }

    fn handle_air_supply(&self, pre_tick_air_supply: i32) {
        tick_breathing(self, pre_tick_air_supply);
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        if let Some(display) = sync_dirty_mob_effects(&self.living_base, &self.entity_data) {
            self.entity_data
                .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
        }
    }
}

/// Whether a stack is a vanilla dolphin food item (the `#minecraft:fishes` tag).
#[must_use]
pub fn is_dolphin_food(item_stack: &ItemStack) -> bool {
    REGISTRY
        .items
        .is_in_tag(item_stack.item(), &ItemTag::FISHES)
}

impl DefaultEntityPush for DolphinEntity {}

impl Entity for DolphinEntity {
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
            DOLPHIN_BABY_DIMENSIONS.scale(scale)
        } else {
            DOLPHIN_DIMENSIONS.scale(scale)
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
        nbt.insert("GotFish", u8::from(self.got_fish()));
        nbt.insert("Moistness", self.moistness_level());
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);

        if let Some(got_fish) = nbt.byte("GotFish") {
            self.set_got_fish(got_fish != 0);
        }
        if let Some(moistness) = nbt.int("Moistness") {
            self.set_moistness_level(moistness);
        }
    }
}

impl LivingEntity for DolphinEntity {
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
        Some(&sound_events::ENTITY_DOLPHIN_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_DOLPHIN_DEATH)
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
        self.tick_moistness();
        AgeableMob::tick_ageable_mob(self);
        result
    }
}

impl AgeableMob for DolphinEntity {
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
        if !is_dolphin_food(&item_stack) {
            return InteractionResult::Pass;
        }

        // Vanilla `Dolphin.mobInteract` rewards the player with `got_fish`.
        self.set_got_fish(true);
        if !player.abilities.lock().instabuild {
            Mob::use_player_item(self, player, hand);
        }
        InteractionResult::SuccessServer
    }
}

impl Mob for DolphinEntity {
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
            Some(&sound_events::ENTITY_DOLPHIN_AMBIENT_WATER)
        } else {
            Some(&sound_events::ENTITY_DOLPHIN_AMBIENT)
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

impl PathfinderMob for DolphinEntity {}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use simdnbt::borrow::read_compound;
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;
    use steel_registry::vanilla_items;

    use super::*;

    fn dolphin() -> DolphinEntity {
        DolphinEntity::new(&vanilla_entities::DOLPHIN, 1, DVec3::ZERO, Weak::new())
    }

    #[test]
    fn new_dolphins_start_moist_and_unrewarded() {
        init_vanilla_registry();
        let dolphin = dolphin();
        assert_eq!(dolphin.moistness_level(), DOLPHIN_MOISTNESS_DEFAULT);
        assert!(!dolphin.got_fish());
    }

    #[test]
    fn got_fish_flag_round_trips() {
        init_vanilla_registry();
        let dolphin = dolphin();
        dolphin.set_got_fish(true);
        assert!(dolphin.got_fish());
        dolphin.set_got_fish(false);
        assert!(!dolphin.got_fish());
    }

    #[test]
    fn moisture_level_round_trips_through_nbt() {
        init_vanilla_registry();
        let dolphin = dolphin();
        dolphin.set_moistness_level(1234);
        dolphin.set_got_fish(true);

        let mut nbt = NbtCompound::new();
        dolphin.save_additional(&mut nbt);
        assert_eq!(nbt.int("Moistness"), Some(1234));
        assert_eq!(nbt.byte("GotFish"), Some(1));

        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let borrowed = read_compound(&mut Cursor::new(&bytes))
            .unwrap_or_else(|error| panic!("reborrow failed: {error}"));

        let fresh = DolphinEntity::new(&vanilla_entities::DOLPHIN, 2, DVec3::ZERO, Weak::new());
        fresh.load_additional((&borrowed).into());
        assert_eq!(fresh.moistness_level(), 1234);
        assert!(fresh.got_fish());
    }

    #[test]
    fn raw_cod_is_dolphin_food() {
        init_vanilla_registry();
        assert!(is_dolphin_food(&ItemStack::new(&vanilla_items::COD)));
        assert!(!is_dolphin_food(&ItemStack::new(&vanilla_items::STONE)));
    }
}
