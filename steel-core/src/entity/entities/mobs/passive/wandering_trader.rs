//! Vanilla `net.minecraft.world.entity.npc.wanderingtrader.WanderingTrader`.
//!
//! Deviations (deliberate, documented in `PARITY-26.3-RU.md`):
//! * the trade list is fixed at spawn from the wandering trader table; the
//!   despawn timer and the llama escorts are not implemented.
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
use steel_registry::vanilla_entity_data::WanderingTraderEntityData;
use steel_registry::{sound_events, vanilla_attributes};
use steel_utils::locks::SyncMutex;
use steel_utils::{Downcast as _, DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    BreedGoal, FloatGoal, FollowParentGoal,
    LookAtPlayerGoal, PanicGoal, RandomLookAroundGoal, WaterAvoidingRandomStrollGoal,
};

use crate::entity::damage::DamageSource;
use crate::entity::villager_trades;
use crate::inventory::menu::kinds::{SharedTrades, merchant};
use crate::entity::{
    AgeableMob, AgeableMobBase, DefaultEntityPush, Entity, EntityBase, EntityBaseLoad, EntityPose,
    EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase, PathfinderMob, RemovalReason,
    SharedEntity, sync_dirty_mob_effects,
};
use crate::behavior::InteractionResult;
use crate::physics::MoveResult;
use crate::player::Player;
use steel_utils::types::InteractionHand;
use crate::world::World;

const DEFAULT_STEP_HEIGHT: f32 = 0.6;

#[entity_behavior(class = "WanderingTrader")]
/// The vanilla wandering trader (a villager-like NPC with its own trades).
pub struct WanderingTraderEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    ageable_base: AgeableMobBase,
    entity_data: SyncMutex<WanderingTraderEntityData>,
    /// The wandering trader's trades, shared with the open merchant screen.
    trades: SharedTrades,
    /// Vanilla `AbstractVillager.tradingPlayer`: the player browsing the offers.
    ///
    /// Kept as an id so the trader does not hold a strong reference to a player
    /// that may have disconnected; the despawn timer only needs to know whether
    /// anybody is trading.
    trading_player: SyncMutex<Option<uuid::Uuid>>,
    /// Vanilla `WanderingTrader.despawnDelay`: ticks until the trader leaves.
    ///
    /// A naturally spawned trader starts at 48000 (40 minutes); zero means the
    /// trader was placed by a command or spawn egg and never despawns.
    despawn_delay: SyncMutex<i32>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `WanderingTraderEntity`.
unsafe impl DowncastType for WanderingTraderEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/wandering_trader");
}

impl WanderingTraderEntity {
    /// Wandering traders never spawn naturally: vanilla spawns them through
    /// the trader spawner event, which Steel does not implement yet.
    /// Creates a new entity at runtime.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Reconstructs a wandering trader from persisted base entity state.
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
        let mut entity_data = WanderingTraderEntityData::new();
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

        let trades = Arc::new(SyncMutex::new(villager_trades::offers_for(
            "minecraft:wandering_trader",
        )));

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            ageable_base,
            entity_data: SyncMutex::new(entity_data),
            trades,
            trading_player: SyncMutex::new(None),
            despawn_delay: SyncMutex::new(0),
        }
    }

    /// Vanilla `AbstractVillager#setTradingPlayer`.
    pub fn set_trading_player(&self, player: Option<uuid::Uuid>) {
        *self.trading_player.lock() = player;
    }

    /// Vanilla `AbstractVillager#isTrading`.
    #[must_use]
    pub fn is_trading(&self) -> bool {
        self.trading_player.lock().is_some()
    }

    /// Vanilla `WanderingTrader#setDespawnDelay`.
    pub fn set_despawn_delay(&self, delay: i32) {
        *self.despawn_delay.lock() = delay;
    }

    /// Vanilla `WanderingTrader#getDespawnDelay`.
    #[must_use]
    pub fn get_despawn_delay(&self) -> i32 {
        *self.despawn_delay.lock()
    }

    /// Vanilla `WanderingTrader#maybeDespawn`, called from `aiStep`.
    ///
    /// The delay only counts down outside a trade: a player browsing the offers
    /// keeps the trader in place, which vanilla implements by skipping the tick
    /// while `getTradingPlayer()` is set. Trading is tracked by player id, and
    /// closing the screen clears it, so a crashed client cannot pin a trader.
    fn maybe_despawn(&self) {
        let trading = self.is_trading();
        let mut delay = self.despawn_delay.lock();
        let Some(next) = despawn_delay_step(*delay, trading) else {
            return;
        };
        *delay = next;
        if next == 0 {
            drop(delay);
            // Vanilla `WanderingTrader.maybeDespawn` discards once the counter
            // reaches zero, which is why the wiki reports "leaves at 1".
            self.set_removed(RemovalReason::Discarded);
        }
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

impl DefaultEntityPush for WanderingTraderEntity {}

impl Entity for WanderingTraderEntity {
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

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
        self.save_ageable_mob(nbt);
        // Vanilla `WanderingTrader.addAdditionalSaveData` writes `DespawnDelay`.
        nbt.insert("DespawnDelay", self.get_despawn_delay());
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        self.load_ageable_mob(nbt);
        if let Some(delay) = nbt.int("DespawnDelay") {
            self.set_despawn_delay(delay);
        }
    }

    /// Vanilla `WanderingTrader.openMenu` through `MenuProvider`.
    fn open_menu_for(&self, player: &Player) -> InteractionResult {
        self.open_trading_screen(player)
    }
}

impl LivingEntity for WanderingTraderEntity {
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
        Some(&sound_events::ENTITY_WANDERING_TRADER_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_WANDERING_TRADER_DEATH)
    }

    fn server_ai_step(&self, entity: &SharedEntity) {
        Mob::mob_server_ai_step(self, entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        let result = Mob::mob_ai_step(self, entity);

        AgeableMob::tick_ageable_mob(self);
        // Vanilla `WanderingTrader.mobTick` → `maybeDespawn`.
        self.maybe_despawn();
        result
    }
}

impl AgeableMob for WanderingTraderEntity {
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

impl WanderingTraderEntity {
    /// Builds the merchant screen for `player` from the trader's offers.
    fn open_trading_screen(&self, player: &Player) -> InteractionResult {
        let trades = Arc::clone(&self.trades);
        if trades.lock().is_empty() {
            return InteractionResult::Pass;
        }
        let inventory = Arc::clone(&player.inventory);
        // Vanilla `AbstractVillager.startTrading` stores the player and stops it
        // again from `stopTrading` when the merchant screen is removed, so the
        // despawn timer only runs while nobody is browsing the offers.
        let world = self.level();
        let trader_id = self.id();
        player.open_menu(self.display_name(), move |context| {
            let on_close = move |_player: &Player| {
                // Look the trader up on close: a shared handle here would keep a
                // discarded trader alive, and the entity is cheap to find by id.
                let Some(world) = world.as_ref() else {
                    return;
                };
                if let Some(trader) = World::entity_manager(world).get_accessible_by_id(trader_id)
                    && let Some(trader) = trader
                        .as_ref()
                        .downcast_ref::<WanderingTraderEntity>()
                {
                    trader.set_trading_player(None);
                }
            };
            merchant(inventory, context.container_id, trades).with_on_close(on_close)
        });
        self.set_trading_player(Some(player.uuid()));
        InteractionResult::Success
    }
}

impl Mob for WanderingTraderEntity {
    /// Vanilla `WanderingTrader.mobInteract`: right-clicking opens the trade
    /// screen. The trader has no age gate in vanilla, so neither does this.
    fn mob_interact(&self, player: &Player, _hand: InteractionHand) -> InteractionResult {
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
        Some(&sound_events::ENTITY_WANDERING_TRADER_AMBIENT)
    }

    /// Vanilla `WanderingTrader.removeWhenFarAway` returns `false`: the trader is
    /// removed by its own despawn timer, never by the distance check.
    fn remove_when_far_away(&self, _dist_sqr: f64) -> bool {
        false
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl PathfinderMob for WanderingTraderEntity {}

/// One tick of vanilla `WanderingTrader.maybeDespawn`.
///
/// Vanilla decrements only while the trader is not being traded with and
/// discards when the counter reaches `0` (`--this.despawnDelay == 0`). Returns
/// `Some(next)` when the counter moved and `None` when it is paused or already
/// finished; `Some(0)` after a `1` means the trader leaves now.
const fn despawn_delay_step(delay: i32, trading: bool) -> Option<i32> {
    if trading || delay <= 0 {
        return None;
    }
    Some(delay - 1)
}

#[cfg(test)]
mod despawn_timer_tests {
    use super::despawn_delay_step;

    #[test]
    fn the_timer_holds_while_a_player_is_trading() {
        assert_eq!(despawn_delay_step(48_000, true), None);
        assert_eq!(despawn_delay_step(2, true), None);
    }

    #[test]
    fn an_untraded_trader_counts_down() {
        assert_eq!(despawn_delay_step(48_000, false), Some(47_999));
        assert_eq!(despawn_delay_step(2, false), Some(1));
        // The step that reaches zero is the one that discards the trader.
        assert_eq!(despawn_delay_step(1, false), Some(0));
    }

    #[test]
    fn a_command_summoned_trader_never_despawns() {
        // Vanilla `DespawnDelay` defaults to 0 for `/summon` and spawn eggs.
        assert_eq!(despawn_delay_step(0, false), None);
        assert_eq!(despawn_delay_step(0, true), None);
    }
}
