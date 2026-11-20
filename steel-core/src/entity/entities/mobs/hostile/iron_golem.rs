//! Vanilla `net.minecraft.world.entity.animal.golem.IronGolem`.
//!
//! A neutral village golem that retaliates when attacked.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * poppy offering and the crack overlay are not implemented; village defence is
//!   modelled as vanilla's `#minecraft:raiders` target selector.

use std::iter::empty;
use std::sync::Weak;

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::registry::TaggedRegistryExt as _;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::IronGolemEntityData;
use steel_registry::vanilla_entity_type_tags::EntityTypeTag;
use steel_registry::{REGISTRY, sound_events, vanilla_attributes};
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    FloatGoal, HurtByTargetGoal, LookAtPlayerGoal, MeleeAttackGoal, NearestAttackableTargetGoal,
    RandomLookAroundGoal, WaterAvoidingRandomStrollGoal, MoveTowardsTargetGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::{
    DefaultEntityPush, Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntityPose,
    EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase, PathfinderMob, SharedEntity,
    sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::world::World;

const DEFAULT_STEP_HEIGHT: f32 = 0.6;

/// Vanilla `net.minecraft.world.entity.animal.golem.IronGolem`.
#[entity_behavior(class = "IronGolem")]
pub struct IronGolemEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<IronGolemEntityData>,
    player_created: std::sync::atomic::AtomicBool,
}

// SAFETY: This key is owned by Steel and uniquely identifies `IronGolemEntity`.
unsafe impl DowncastType for IronGolemEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/iron_golem");
}

impl IronGolemEntity {
    /// Creates a new entity of this vanilla type.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Loads a saved entity of this vanilla type.
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

        let mut entity_data = IronGolemEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(1, MeleeAttackGoal::new(1.0, true));
            goal_selector.add_goal(2, MoveTowardsTargetGoal::new(0.9, 32.0));
            goal_selector.add_goal(7, WaterAvoidingRandomStrollGoal::new(1.0));
            goal_selector.add_goal(7, LookAtPlayerGoal::new(6.0));
            goal_selector.add_goal(8, RandomLookAroundGoal::new());

            let mut target_selector = mob_base.target_selector().lock();
            target_selector.add_goal(1, HurtByTargetGoal::new().set_alert_others(empty()));
            // Vanilla `IronGolem.createAngerLevelTarget`: golems attack the
            // raiders that threaten their village.
            target_selector.add_goal(
                2,
                NearestAttackableTargetGoal::new(true, |entity, _| {
                    REGISTRY
                        .entity_types
                        .is_in_tag(entity.entity_type(), &EntityTypeTag::RAIDERS)
                }),
            );
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            entity_data: SyncMutex::new(entity_data),
            player_created: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Whether this iron golem was built by a player (`PlayerCreated` NBT tag).
    #[must_use]
    pub fn is_player_created(&self) -> bool {
        self.player_created
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Marks this iron golem as player-created (`PlayerCreated` NBT tag).
    pub fn set_player_created(&self, player_created: bool) {
        self.player_created
            .store(player_created, std::sync::atomic::Ordering::Relaxed);
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        sync_dirty_mob_effects(&self.living_base, &self.entity_data);
    }
}

impl DefaultEntityPush for IronGolemEntity {}

impl Entity for IronGolemEntity {
    fn is_hard_collision_relevant(&self) -> bool {
        false
    }

    fn as_default_entity_push(&self) -> Option<&dyn DefaultEntityPush> {
        Some(self)
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
            .unwrap_or(f64::from(DEFAULT_STEP_HEIGHT)) as f32
    }

    fn sound_source(&self) -> SoundSource {
        SoundSource::Neutral
    }

    fn fall_sounds(&self) -> (SoundEventRef, SoundEventRef) {
        (
            &sound_events::ENTITY_GENERIC_SMALL_FALL,
            &sound_events::ENTITY_GENERIC_BIG_FALL,
        )
    }

    fn swim_sound(&self) -> SoundEventRef {
        &sound_events::ENTITY_GENERIC_SWIM
    }

    fn swim_splash_sound(&self) -> SoundEventRef {
        &sound_events::ENTITY_GENERIC_SPLASH
    }

    fn play_step_sound(&self, _pos: BlockPos, _block_state: BlockStateId) {
        self.play_sound(&sound_events::ENTITY_IRON_GOLEM_STEP, 0.15, 1.0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
        nbt.insert("PlayerCreated", self.is_player_created());
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        if let Some(flag) = nbt.byte("PlayerCreated") {
            self.set_player_created(flag != 0);
        }
    }
}

impl LivingEntity for IronGolemEntity {
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
        Some(&sound_events::ENTITY_IRON_GOLEM_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_IRON_GOLEM_DEATH)
    }

    fn server_ai_step(&self, entity: &SharedEntity) {
        Mob::mob_server_ai_step(self, entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        Mob::mob_ai_step(self, entity)
    }

    /// Vanilla `IronGolem.causeFallDamage`: iron golems are immune to fall damage.
    fn is_invulnerable_to(&self, _world: &World, source: &DamageSource) -> bool {
        source.is(&steel_registry::vanilla_damage_type_tags::DamageTypeTag::IS_FALL)
    }
}

impl Mob for IronGolemEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    /// Vanilla `IronGolem.mobInteract`: using an iron ingot repairs 25 health.
    fn mob_interact(
        &self,
        player: &crate::player::Player,
        hand: steel_utils::types::InteractionHand,
    ) -> crate::behavior::InteractionResult {
        let item_stack = {
            let inventory = player.inventory.lock();
            let held = inventory.get_item_in_hand(hand);
            held.copy_with_count(held.count())
        };
        if !item_stack.is(&steel_registry::vanilla_items::IRON_INGOT) {
            return crate::behavior::InteractionResult::Pass;
        }
        let before = self.get_health();
        self.heal(25.0);
        if (self.get_health() - before).abs() < f32::EPSILON {
            return crate::behavior::InteractionResult::Pass;
        }
        self.play_sound(&sound_events::ENTITY_IRON_GOLEM_REPAIR, 1.0, 1.0);
        if !player.has_infinite_materials() {
            player.inventory.lock().shrink_item_in_hand(hand, 1);
        }
        crate::behavior::InteractionResult::SuccessServer
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl PathfinderMob for IronGolemEntity {}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::Weak;

    use glam::DVec3;
    use simdnbt::borrow::read_compound;
    use simdnbt::owned::NbtCompound;
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::IronGolemEntity;
    use crate::entity::{Entity, LivingEntity};

    #[test]
    fn iron_golem_uses_registry_attributes_and_round_trips_nbt() {
        init_vanilla_registry();
        let golem =
            IronGolemEntity::new(&vanilla_entities::IRON_GOLEM, 1, DVec3::ZERO, Weak::new());
        assert!((golem.get_max_health() - 100.0).abs() < 1e-4);
        assert!(!golem.is_player_created());
        golem.set_player_created(true);

        let mut nbt = NbtCompound::new();
        golem.save_additional(&mut nbt);

        let loaded =
            IronGolemEntity::new(&vanilla_entities::IRON_GOLEM, 2, DVec3::ZERO, Weak::new());
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let borrowed = read_compound(&mut Cursor::new(&bytes))
            .unwrap_or_else(|error| panic!("reborrow failed: {error}"));
        loaded.load_additional((&borrowed).into());
        assert!(loaded.get_health() > 0.0);
        assert!(loaded.is_player_created());
    }
}
