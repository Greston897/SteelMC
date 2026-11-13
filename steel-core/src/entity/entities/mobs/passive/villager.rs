//! Vanilla `net.minecraft.world.entity.npc.villager.Villager`.
//!
//! Deviations (deliberate, documented in `PARITY-26.3-RU.md`):
//! * trading covers the novice tier of every profession; levelling, restocking
//!   and gossip-driven prices are not modelled, so the offer list is fixed for
//!   the villager's lifetime.
//! * the `villager_data` synced component is stored as-is.
//! * no natural spawning: vanilla villages place villagers through structure
//!   generation and the wandering trader arrives with the trader event, both of
//!   which Steel does not run yet.
use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::VillagerEntityData;
use steel_registry::{sound_events, vanilla_attributes};
use steel_utils::locks::SyncMutex;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{BreedGoal, FloatGoal, FollowParentGoal, LookAtPlayerGoal, PanicGoal, RandomLookAroundGoal,
    WaterAvoidingRandomStrollGoal,
;
use crate::entity::damage::DamageSource;
use crate::entity::villager_trades;
use crate::inventory::menu::kinds::{SharedTrades, merchant};
use crate::behavior::InteractionResult;
use crate::player::Player;
use steel_utils::types::InteractionHand;
use crate::entity::{
    AgeableMob, AgeableMobBase, DefaultEntityPush, Entity, EntityBase, EntityBaseLoad, EntityPose,
    EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase, PathfinderMob, SharedEntity,
    sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::world::World;

const DEFAULT_STEP_HEIGHT: f32 = 0.6;

#[entity_behavior(class = "Villager")]
/// The vanilla villager (profession, type and level live in the synced data).
pub struct VillagerEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    entity_data: SyncMutex<VillagerEntityData>,
    /// The villager's trades, shared with the open merchant screen so a
    /// completed trade lowers the stock the entity reports on the next open.
    trades: SharedTrades,
}

// SAFETY: This key is owned by Steel and uniquely identifies `VillagerEntity`.
unsafe impl DowncastType for VillagerEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/villager");
}

impl VillagerEntity {
    /// Vanilla `SpawnPlacements` predicate for villagers: they only spawn
    /// inside a village, which Steel's world generation does not build yet.

    /// Creates a new entity at runtime.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Reconstructs a villager from persisted base entity state.
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
        let mut entity_data = VillagerEntityData::new();
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

        // The trade list is built once from the profession; vanilla rebuilds it
        // when the villager levels up, which needs experience tracking that the
        // villager does not have yet.
        let profession = *entity_data.villager().villager_data.get();
        let trades = Arc::new(SyncMutex::new(villager_trades::offers_for_profession_id(
            profession.profession,
        )));

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            ageable_base,
            entity_data: SyncMutex::new(entity_data),
            trades,
        }
    }

    /// Vanilla `Villager.getVillagerData`.
    #[must_use]
    pub fn villager_data(&self) -> steel_registry::entity_data::VillagerData {
        *self.entity_data.lock().villager().villager_data.get()
    }

    /// Sets the vanilla villager data (profession, type, level).
    pub fn set_villager_data(&self, data: steel_registry::entity_data::VillagerData) {
        self.entity_data.lock().villager_mut().villager_data.set(data);
    }

    /// Vanilla `AbstractVillager.getUnhappyCounter`.
    #[must_use]
    pub fn unhappy_counter(&self) -> i32 {
        *self.entity_data.lock().abstract_villager().unhappy_counter.get()
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        if let Some(display) = sync_dirty_mob_effects(&self.living_base, &self.entity_data) {
            self.entity_data
                .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
        }
    }
}

impl VillagerEntity {
    /// Builds the merchant screen for `player` from this villager's trades.
    fn open_trading_screen(&self, player: &Player) -> InteractionResult {
        let trades = Arc::clone(&self.trades);
        if trades.lock().is_empty() {
            // Vanilla `AbstractVillager.mobInteract`: a villager with no trades
            // (a nitwit, or a baby) cannot be traded with.
            return InteractionResult::Pass;
        }

        let inventory = Arc::clone(&player.inventory);
        player.open_menu(self.display_name(), move |context| {
            merchant(inventory, context.container_id, trades)
        });
        InteractionResult::Success
    }
}

impl DefaultEntityPush for VillagerEntity {}

impl Entity for VillagerEntity {
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

    /// Vanilla `Villager.openMenu` through `MenuProvider`: spectators reach the
    /// trade screen through this path.
    fn open_menu_for(&self, player: &Player) -> InteractionResult {
        self.open_trading_screen(player)
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

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
        self.save_ageable_mob(nbt);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);

    }
}

impl LivingEntity for VillagerEntity {
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
        Some(&sound_events::ENTITY_VILLAGER_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_VILLAGER_DEATH)
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

impl AgeableMob for VillagerEntity {
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

impl Mob for VillagerEntity {
    /// Vanilla `AbstractVillager.mobInteract`: right-clicking an adult villager
    /// opens the trade screen.
    fn mob_interact(&self, player: &Player, _hand: InteractionHand) -> InteractionResult {
        if AgeableMob::is_baby(self) {
            return InteractionResult::Pass;
        }
        self.open_trading_screen(player)
    }

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
        Some(&sound_events::ENTITY_VILLAGER_AMBIENT)
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl PathfinderMob for VillagerEntity {}

