//! Vanilla `net.minecraft.world.entity.monster.zombie.ZombieVillager`.
//!
//! A zombie that keeps its villager profession and type. Vanilla cures it back
//! into a villager with a golden apple and a weakness potion; the synced
//! `converting` flag and the villager data drive that process.
//!
//! Curing follows vanilla `ZombieVillager`: a golden apple plus a weakness
//! potion starts a conversion timer, and when it runs out the zombie turns into
//! a villager that keeps the profession and fires `cured_zombie_villager`.
//!
//! Documented deviations from vanilla 26.3 (see `PARITY-26.3-RU.md`):
//! * the profession-specific texture is client-side and not part of Steel.
//! * gossip and the reputation discount are not modelled; the cured villager
//!   starts with vanilla's converted-villager experience of zero.
use std::iter::empty;
use std::sync::Weak;

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::ZombieVillagerEntityData;
use steel_registry::{sound_events, vanilla_attributes, vanilla_items};
use steel_utils::locks::SyncMutex;
use steel_utils::random::legacy_random::LegacyRandom;
use steel_utils::types::InteractionHand;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey};

use crate::behavior::InteractionResult;
use crate::entity::ai::goal::{
    FloatGoal, HurtByTargetGoal, LookAtPlayerGoal, MeleeAttackGoal, NearestAttackableTargetGoal,
    RandomLookAroundGoal, WaterAvoidingRandomStrollGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::entities::mobs::passive::VillagerEntity;
use crate::entity::{
    DefaultEntityPush, Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntityPose,
    EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase, Monster, PathfinderMob,
    SharedEntity, sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::player::Player;
use crate::world::World;

const DEFAULT_STEP_HEIGHT: f32 = 0.6;

/// Vanilla `ZombieVillager.getConversionProgress` base: the roll is
/// `random.nextInt(2401) + 3600` ticks, i.e. three to five minutes.
const CONVERSION_MIN_TICKS: i32 = 3600;
/// Upper bound of the extra random ticks added to [`CONVERSION_MIN_TICKS`].
const CONVERSION_RANDOM_TICKS: i32 = 2401;

/// Vanilla `net.minecraft.world.entity.monster.Zombie`.
#[entity_behavior(class = "ZombieVillager")]
pub struct ZombieVillagerEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<ZombieVillagerEntityData>,
    /// Remaining ticks of a started cure, or `None` when not converting.
    conversion: SyncMutex<Option<ConversionState>>,
}

/// State of a cure in progress.
#[derive(Debug, Clone, Copy)]
pub struct ConversionState {
    /// Ticks left until the zombie becomes a villager.
    remaining_ticks: i32,
    /// The player who started the cure, credited by `cured_zombie_villager`.
    starter: Option<uuid::Uuid>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `ZombieVillagerEntity`.
unsafe impl DowncastType for ZombieVillagerEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/zombie_villager");
}

impl ZombieVillagerEntity {
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

        // Vanilla `ZombieVillager` rolls a random villager profession on spawn
        // (`DATA_VILLAGER_DATA` runtime initializer).
        let mut random = LegacyRandom::from_seed(rand::random());
        let mut entity_data = ZombieVillagerEntityData::new(&mut random);
        living_base.initialize_synced_data(&mut entity_data);

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(2, MeleeAttackGoal::new(1.0, false));
            goal_selector.add_goal(7, WaterAvoidingRandomStrollGoal::new(1.0));
            goal_selector.add_goal(8, LookAtPlayerGoal::new(8.0));
            goal_selector.add_goal(9, RandomLookAroundGoal::new());

            let mut target_selector = mob_base.target_selector().lock();
            target_selector.add_goal(1, HurtByTargetGoal::new().set_alert_others(empty()));
            target_selector.add_goal(
                2,
                NearestAttackableTargetGoal::new_for_players(true, |_, _| true),
            );
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            entity_data: SyncMutex::new(entity_data),
            conversion: SyncMutex::new(None),
        }
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        sync_dirty_mob_effects(&self.living_base, &self.entity_data);
    }
}

impl DefaultEntityPush for ZombieVillagerEntity {}

impl Entity for ZombieVillagerEntity {
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
        Monster::sound_source_monster(self)
    }

    fn fall_sounds(&self) -> (SoundEventRef, SoundEventRef) {
        Monster::fall_sounds_monster(self)
    }

    fn swim_sound(&self) -> SoundEventRef {
        Monster::swim_sound_monster(self)
    }

    fn swim_splash_sound(&self) -> SoundEventRef {
        Monster::swim_splash_sound_monster(self)
    }

    fn play_step_sound(&self, _pos: BlockPos, _block_state: BlockStateId) {
        self.play_sound(&sound_events::ENTITY_ZOMBIE_VILLAGER_STEP, 0.15, 1.0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
    }
}

impl ZombieVillagerEntity {
    /// Vanilla `ZombieVillager.isConverting` (`DATA_CONVERTING_ID`).
    #[must_use]
    pub fn is_converting(&self) -> bool {
        *self.entity_data.lock().converting.get()
    }

    /// Sets the vanilla conversion flag.
    pub fn set_converting(&self, converting: bool) {
        self.entity_data.lock().converting.set(converting);
    }

    /// Vanilla `ZombieVillager.getVillagerData`.
    #[must_use]
    pub fn villager_data(&self) -> steel_registry::entity_data::VillagerData {
        *self.entity_data.lock().villager_data.get()
    }

    /// Sets the vanilla villager data (profession, type, level).
    pub fn set_villager_data(&self, data: steel_registry::entity_data::VillagerData) {
        self.entity_data.lock().villager_data.set(data);
    }
}

impl ZombieVillagerEntity {
    /// Vanilla `ZombieVillager.startConverting`: arms the conversion timer,
    /// remembering the player to credit.
    fn start_converting(&self, starter: &Player) {
        let ticks = rand::random_range(0..CONVERSION_RANDOM_TICKS) + CONVERSION_MIN_TICKS;
        *self.conversion.lock() = Some(ConversionState {
            remaining_ticks: ticks,
            starter: Some(starter.base().uuid()),
        });
        self.set_converting(true);
        // Vanilla plays the zombie-villager cure sound and particles here.
        self.play_sound(&sound_events::ENTITY_ZOMBIE_VILLAGER_CURE, 1.0, 1.0);
    }

    /// Advances an active cure, converting into a villager when it finishes.
    ///
    /// Vanilla `ZombieVillager.tick` decrements the timer and calls
    /// `finishConversion` at zero.
    fn tick_conversion(&self) {
        let Some(state) = *self.conversion.lock() else {
            return;
        };
        if state.remaining_ticks > 1 {
            let mut conversion = self.conversion.lock();
            if let Some(active) = conversion.as_mut() {
                active.remaining_ticks -= 1;
            }
            return;
        }
        self.finish_conversion(&state);
    }

    /// Vanilla `ZombieVillager.finishConversion`.
    fn finish_conversion(&self, state: &ConversionState) {
        if self.is_removed() {
            return;
        }
        let villager_data = self.villager_data();
        let starter = state
            .starter
            .and_then(|uuid| self.level().and_then(|world| world.players.get_by_uuid(&uuid)).cloned());
        // Vanilla `ZombieVillager.finishConversion` copies the villager data and
        // the gossip onto the new villager; Steel carries the profession/type/level.
        let copy_villager_data = |mob: &dyn Mob| {
            if let Some(villager) = mob
                .as_entity_event_source()
                .downcast_ref::<VillagerEntity>()
            {
                villager.set_villager_data(villager_data);
            }
        };
        let converted = crate::entity::conversion::convert_to(
            self,
            &vanilla_entities::VILLAGER,
            crate::entity::conversion::ConversionParams::single(true, true),
            Some(&copy_villager_data),
        );
        *self.conversion.lock() = None;
        if converted.is_none() {
            return;
        }
        // Vanilla fires `cured_zombie_villager` on whoever started the cure.
        if let Some(player) = starter {
            player.trigger_advancement_with_context(
                &steel_registry::vanilla_criteria_triggers::CURED_ZOMBIE_VILLAGER,
                &crate::player::advancement::predicate::TriggerContext::entity(
                    &vanilla_entities::ZOMBIE_VILLAGER.key,
                ),
            );
        }
    }
}

impl LivingEntity for ZombieVillagerEntity {
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
        Some(&sound_events::ENTITY_ZOMBIE_VILLAGER_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_ZOMBIE_VILLAGER_DEATH)
    }

    fn should_drop_experience(&self) -> bool {
        Monster::should_drop_experience_monster(self)
    }

    fn should_drop_loot(&self, world: &World) -> bool {
        Monster::should_drop_loot_monster(self, world)
    }

    fn server_ai_step(&self, entity: &SharedEntity) {
        Mob::mob_server_ai_step(self, entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        Monster::monster_ai_step(self, entity)
    }
}

impl Mob for ZombieVillagerEntity {
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
        Some(&sound_events::ENTITY_ZOMBIE_VILLAGER_AMBIENT)
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }

    fn mob_interact(&self, player: &Player, hand: InteractionHand) -> InteractionResult {
        // Vanilla `ZombieVillager.mobInteract`: a golden apple fed to a zombie
        // villager that already has Weakness starts the cure and consumes both
        // the apple and the effect.
        if self.is_converting() {
            return InteractionResult::Pass;
        }
        let stack = {
            let inventory = player.inventory.lock();
            let stack = inventory.get_item_in_hand(hand);
            stack.copy_with_count(stack.count())
        };
        if !stack.is(&*vanilla_items::GOLDEN_APPLE) {
            return InteractionResult::Pass;
        }
        if !self.has_mob_effect(vanilla_mob_effects::WEAKNESS) {
            return InteractionResult::Pass;
        }
        self.remove_mob_effect(vanilla_mob_effects::WEAKNESS);
        player.inventory.lock().shrink_item_in_hand(hand, 1);
        self.start_converting(player);
        InteractionResult::Success
    }

    fn custom_server_ai_step(&self) {
        self.tick_conversion();
    }
}

impl Monster for ZombieVillagerEntity {}

impl PathfinderMob for ZombieVillagerEntity {}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::Weak;

    use glam::DVec3;
    use simdnbt::borrow::read_compound;
    use simdnbt::owned::NbtCompound;
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::ZombieVillagerEntity;
    use crate::entity::{Entity, LivingEntity};

    #[test]
    fn zombie_villager_uses_registry_attributes_and_round_trips_nbt() {
        init_vanilla_registry();
        let zombie = ZombieVillagerEntity::new(
            &vanilla_entities::ZOMBIE_VILLAGER,
            1,
            DVec3::ZERO,
            Weak::new(),
        );
        assert!(zombie.get_max_health() > 0.0);

        let mut nbt = NbtCompound::new();
        zombie.save_additional(&mut nbt);

        let loaded = ZombieVillagerEntity::new(
            &vanilla_entities::ZOMBIE_VILLAGER,
            2,
            DVec3::ZERO,
            Weak::new(),
        );
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let borrowed = read_compound(&mut Cursor::new(&bytes))
            .unwrap_or_else(|error| panic!("reborrow failed: {error}"));
        loaded.load_additional((&borrowed).into());
        assert!(loaded.get_health() > 0.0);
    }
}
