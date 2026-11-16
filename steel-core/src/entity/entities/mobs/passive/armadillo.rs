//! Vanilla `net.minecraft.world.entity.animal.armadillo.Armadillo`.
//!
//! Shared `Animal` skeleton with the vanilla `checkAnimalSpawnRules`
//! placement predicate.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the armadillo rolling state machine (`ArmadilloState`) is not implemented;
//!   its synced enum is stored but never updated.
//! * scute shedding is not implemented.

use std::sync::Weak;

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::ArmadilloEntityData;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_item_tags::ItemTag;
use steel_registry::{sound_events, vanilla_attributes};
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    BreedGoal, FloatGoal, FollowParentGoal,
    LookAtPlayerGoal, PanicGoal, RandomLookAroundGoal, WaterAvoidingRandomStrollGoal,
};

use crate::entity::damage::DamageSource;
use crate::entity::{
    AgeableMob, AgeableMobBase, Animal, AnimalBase, DefaultEntityPush, Entity, EntityBase,
    EntityBaseLoad, EntityPose, EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase,
    PathfinderMob, SharedEntity, SpawnRule, sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::world::World;

const DEFAULT_STEP_HEIGHT: f32 = 0.6;

/// Vanilla `Armadillo.pickNextScuteDropTime`: 5 to 10 minutes (`6000..12000` ticks).
const SCUTE_DROP_MIN_TICKS: i32 = 6_000;
const SCUTE_DROP_MAX_TICKS: i32 = 12_000;
const BRUSH_DAMAGE_PER_SCUTE: i32 = 16;

#[entity_behavior(class = "Armadillo", spawn_rule)]
/// Vanilla armadillo entity.
pub struct ArmadilloEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    animal_base: AnimalBase,
    entity_data: SyncMutex<ArmadilloEntityData>,
    scute_time: SyncMutex<i32>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `ArmadilloEntity`.
unsafe impl DowncastType for ArmadilloEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/armadillo");
}

impl ArmadilloEntity {
    /// Vanilla `SpawnPlacements` predicate: `Animal.checkAnimalSpawnRules`.
    pub(crate) const SPAWN_RULE: SpawnRule = |entity_type, level, spawn_reason, pos, random| {
        <Self as Animal>::check_animal_spawn_rules(entity_type, level, spawn_reason, pos, random)
    };

    /// Creates a new armadillo at runtime.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Reconstructs an armadillo from persisted base entity state.
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
        let mut entity_data = ArmadilloEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            // Shared animal goal priorities and speeds in vanilla order.
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(1, PanicGoal::new(2.0));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(4, FollowParentGoal::new(1.25));
            goal_selector.add_goal(5, WaterAvoidingRandomStrollGoal::new(1.0));
            goal_selector.add_goal(6, LookAtPlayerGoal::new(6.0));
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
            scute_time: SyncMutex::new(Self::next_scute_time()),
        }
    }

    fn next_scute_time() -> i32 {
        use rand::RngExt as _;
        rand::rng().random_range(SCUTE_DROP_MIN_TICKS..SCUTE_DROP_MAX_TICKS)
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        if let Some(display) = sync_dirty_mob_effects(&self.living_base, &self.entity_data) {
            self.entity_data
                .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
        }
    }
}

impl DefaultEntityPush for ArmadilloEntity {}

impl Entity for ArmadilloEntity {
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
            .unwrap_or(f64::from(DEFAULT_STEP_HEIGHT)) as f32
    }

    fn sound_source(&self) -> SoundSource {
        SoundSource::Neutral
    }

    fn play_step_sound(&self, _pos: BlockPos, _block_state: BlockStateId) {
        self.play_sound(&sound_events::ENTITY_ARMADILLO_STEP, 0.15, 1.0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
        self.save_ageable_mob(nbt);
        self.save_animal(nbt);
        nbt.insert("scute_time", *self.scute_time.lock());
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);
        self.load_animal(nbt);
        if let Some(ticks) = nbt.int("scute_time") {
            *self.scute_time.lock() = ticks.max(0);
        }
    }
}

impl LivingEntity for ArmadilloEntity {
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
        Some(&sound_events::ENTITY_ARMADILLO_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_ARMADILLO_DEATH)
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

impl AgeableMob for ArmadilloEntity {
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

impl Animal for ArmadilloEntity {
    fn animal_base(&self) -> &AnimalBase {
        &self.animal_base
    }

    /// Vanilla `Armadillo.isFood`: the item must be in `#minecraft:armadillo_food`.
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item().has_tag(&ItemTag::ARMADILLO_FOOD)
    }
}

impl Mob for ArmadilloEntity {
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
        if LivingEntity::is_alive(self) && !AgeableMob::is_baby(self) {
            let should_shed = {
                let mut t = self.scute_time.lock();
                *t -= 1;
                if *t <= 0 {
                    *t = Self::next_scute_time();
                    true
                } else {
                    false
                }
            };
            if should_shed {
                let _ = self.spawn_at_location(
                    ItemStack::new(&*steel_registry::vanilla_items::ARMADILLO_SCUTE),
                    0.0,
                );
                self.play_sound(&sound_events::ENTITY_ARMADILLO_SCUTE_DROP, 1.0, 1.0);
                self.game_event(&steel_registry::vanilla_game_events::ENTITY_PLACE);
            }
        }
    }

    /// Vanilla `Armadillo.mobInteract`: brushing an adult sheds an armadillo
    /// scute and damages the brush by 16 durability; food falls through to `Animal`.
    fn mob_interact(
        &self,
        player: &crate::player::Player,
        hand: steel_utils::types::InteractionHand,
    ) -> crate::behavior::InteractionResult {
        let held = {
            let inv = player.inventory.lock();
            let stack = inv.get_item_in_hand(hand);
            stack.copy_with_count(stack.count())
        };
        if held.is(&*steel_registry::vanilla_items::BRUSH) && !AgeableMob::is_baby(self) {
            let _ = self.spawn_at_location(
                ItemStack::new(&*steel_registry::vanilla_items::ARMADILLO_SCUTE),
                0.0,
            );
            self.play_sound(&sound_events::ENTITY_ARMADILLO_BRUSH, 1.0, 1.0);
            self.game_event_with_source_entity(
                &steel_registry::vanilla_game_events::ENTITY_INTERACT,
                Some(player),
            );
            player.inventory.lock().hurt_item_in_hand(
                hand,
                BRUSH_DAMAGE_PER_SCUTE,
                player.has_infinite_materials(),
            );
            return crate::behavior::InteractionResult::SuccessServer;
        }
        Animal::mob_interact_animal(self, player, hand)
    }

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_ARMADILLO_AMBIENT)
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl PathfinderMob for ArmadilloEntity {}

