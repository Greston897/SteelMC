//! This module contains all things player-related.
mod abilities;
pub mod advancement;
pub mod chat;
pub mod chunk_sender;
/// This module contains the `PlayerConnection` trait that abstracts network connections.
pub mod connection;
mod container_counter;
mod entity_state;
/// Experience System
pub mod experience;
pub mod food_data;
/// Game mode specific logic for player interactions.
pub mod game_mode;
mod health_sync;
mod item_cooldowns;
mod lifecycle;
pub mod movement;
mod permissions;
pub mod player_data;
pub mod player_data_storage;
pub mod player_inventory;
mod post_effects;
mod profile;
pub mod recipe_book;
pub mod recipe_placement;
mod session;
mod sleep;
mod sleep_state;
mod spam_throttler;
pub mod stats_counter;
mod tick_state;
mod title;

use crate::block_entity::BlockEntity as _;
use crate::block_entity::entities::CommandBlockBlockEntity;
use crate::block_entity::entities::CommandBlockMode;
use crate::block_entity::entities::CommandBlockMode::Auto;
use crate::block_entity::entities::CommandBlockMode::Redstone;
use crate::block_entity::entities::CommandBlockMode::Sequence;
use crate::block_entity::entities::JigsawBlockEntity;
use crate::block_entity::entities::StructureBlockEntity;
use crate::block_entity::entities::TestBlockEntity;
use crate::block_entity::entities::TestInstanceBlockEntity;
use crate::block_entity::entities::WardenSpawnTracker;
use crate::entity::create_wither_rose;
use crate::entity::entities::CommandBlockMinecartEntity;
use crate::player::advancement::predicate::TriggerContext;
use crate::player::player_inventory::Prediction;
use crate::player::recipe_placement::{PlacementOutcome, PostPlaceAction};
use crate::scoreboard::ScoreHolder;
pub use abilities::{Abilities, DEFAULT_FLYING_SPEED};
use chat::ChatState;
pub use chat::{LastSeen, LastSeenMessagesValidator, MessageCache};
use connection::NetworkConnection as _;
pub use connection::{ClientInformation, PlayerConnection};
use container_counter::ContainerCounter;
use food_data::{FoodData, food_constants};
use game_mode::{BlockBreakingManager, PlayerGameModeState};
use glam::DVec3;
use health_sync::HealthSyncState;
use item_cooldowns::ItemCooldowns;
use lifecycle::PlayerLifecycleState;
pub use lifecycle::PlayerRespawnConfig;
pub(crate) use lifecycle::ResetReason;
pub(crate) use lifecycle::resolve_personal_respawn_for_world_change;
pub use movement::PlayerInput;
use movement::{MovementState, TeleportState};
use permissions::PlayerPermissionState;
pub use post_effects::PlayerPostEffects;
pub(crate) use profile::{GAME_PROFILE_CACHE_LIMIT, KnownPlayerNameLookup, lookup_online_profile};
pub use profile::{
    GameProfile, KnownPlayer, KnownPlayers, ProfileLookupError, is_valid_player_name, offline_uuid,
};
pub use session::PlayerSession;
pub(crate) use session::PlayerSessionId;
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use sleep_state::PlayerSleepState;
use std::io::Cursor;
use std::ptr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, LazyLock, Weak};
use std::time::{Duration, Instant};
use steel_math::RAD_TO_DEG_F64;
use steel_protocol::packets::game::CCookieRequest;
use steel_protocol::packets::game::CCustomChatCompletions;
use steel_protocol::packets::game::CGameTestHighlightPos;
use steel_protocol::packets::game::CPing;
use steel_protocol::packets::game::CStartConfiguration;
use steel_protocol::packets::game::CStoreCookie;
use steel_protocol::packets::game::CustomChatCompletionAction;
use steel_protocol::packets::game::SCookieResponse;
use steel_protocol::packets::game::SCustomClickAction;
use steel_protocol::packets::game::{
    CEntityEvent, CHurtAnimation, CPlaceGhostRecipe, CPlayerCombatEnd, CPlayerCombatEnter,
    CPlayerCombatKill, CPlayerLookAt, CPlayerRotation, CRespawn, CSetChunkCacheRadius,
    CSetDefaultSpawnPosition, CSetHealth, CSetHeldSlot, CSetPassengers, CSetSimulationDistance,
    CTagQuery, ClientCommandAction, LookAtAnchor, RelativeMovement, SBlockEntityTagQuery,
    SDebugSubscriptionRequest, SEntityTagQuery, SJigsawGenerate, SLockDifficulty, SPlaceRecipe,
    SRecipeBookChangeSettings, SRecipeBookSeenRecipe, SSetCommandBlock, SSetCommandMinecart,
    SSetGameRule, SSetJigsawBlock, SSetStructureBlock, SSetTestBlock, STeleportToEntity,
    STestInstanceBlockAction, SoundSource,
};
use steel_protocol::packets::game::{CLevelEvent, CSetEntityData, CSetExperience};
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::TestBlockMode;
use steel_registry::entity_data::{EntityPose, HumanoidArm, ParticleList};
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::game_rules::GameRuleRef;
use steel_registry::recipe::display_for_recipe;
use steel_registry::slot_display::RecipeDisplay::CraftingShaped;
use steel_registry::slot_display::RecipeDisplay::CraftingShapeless;
use steel_registry::slot_display::RecipeDisplay::Furnace;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::stat::custom::CustomStatRef;
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::vanilla_criteria_triggers::KILL_MOB_NEAR_SCULK_CATALYST;
use steel_registry::vanilla_criteria_triggers::LEVITATION;
use steel_registry::vanilla_criteria_triggers::PLAYER_KILLED_ENTITY;
use steel_registry::vanilla_criteria_triggers::RECIPE_CRAFTED;
use steel_registry::vanilla_criteria_triggers::RECIPE_UNLOCKED;
use steel_registry::vanilla_criteria_triggers::RIDE_ENTITY_IN_LAVA;
use steel_registry::vanilla_criteria_triggers::USING_ITEM;
use steel_registry::vanilla_entity_data::PlayerEntityData;
use steel_registry::vanilla_game_rules::COMMAND_BLOCKS_WORK;
use steel_registry::vanilla_game_rules::{
    DROWNING_DAMAGE, FALL_DAMAGE, FIRE_DAMAGE, FREEZE_DAMAGE, IMMEDIATE_RESPAWN, KEEP_INVENTORY,
    SHOW_DEATH_MESSAGES,
};
use steel_registry::{
    REGISTRY, level_events, sound_events, vanilla_attributes, vanilla_blocks, vanilla_custom_stats,
    vanilla_damage_type_tags, vanilla_entities, vanilla_game_events, vanilla_mob_effects,
};
use steel_utils::geometry::WorldAabb;
use steel_utils::serial::OptionalNbt;
use steel_utils::serial::ReadFrom;
use steel_utils::translations_registry::TRANSLATIONS;
use steel_utils::types::UpdateFlags;
use steel_utils::{
    Downcast as _, Identifier, entity_events::EntityStatus, locks::Shared, translations,
};
use tick_state::PlayerTickState;
use uuid::Uuid;

use arc_swap::ArcSwap;
use steel_utils::locks::SyncMutex;
use steel_utils::types::{Difficulty, GameType, InteractionHand};
use text_components::resolving::TextResolutor;
use text_components::translation::TranslatedMessage;
use text_components::{
    Modifier as _, TextComponent,
    interactivity::{ClickEvent, HoverEvent, Interactivity},
};
use text_components::{content::Resolvable, custom::CustomData};

use crate::behavior::{
    BlockStateBehaviorExt as _, ITEM_BEHAVIORS, InteractionResult, InventoryTickContext,
    ItemBehavior, apply_use_remainder,
};
use crate::chunk::chunk_request::{ChunkRequestHandle, ChunkRequestState};
use crate::config::RuntimeConfig;
use crate::enchantment_helper;
use crate::entity::LivingEntityRef;
use crate::entity::damage::DamageSource;
use crate::entity::entities::ExperienceOrbEntity;
use crate::entity::{
    ActiveItemUseState, DEATH_DURATION, Entity, EntityAnchor, EntityBase, EntityEventSource,
    EntityMoveError, EntityMovementEmission, EntitySyncedData, LivingEntity, LivingEntityBase,
    LivingEntitySyncedData, MobEffectSyncChange, MobEffectSyncPacket, RemovalReason, SharedEntity,
    apply_entity_look_at, get_kill_credit, start_riding_entities,
};
use crate::fluid::get_fluid_state;
use crate::inventory::equipment::{EntityEquipment, EquipmentSlot};
use crate::inventory::lock::{ContainerLockGuard, ContainerRef};
use crate::inventory::menu::Menu;
use crate::inventory::menu::kinds::inventory_menu;
use crate::level_data::RespawnData;
use crate::permission::{
    PermissionContext, PermissionExpr, PermissionMetadataSet, PermissionMetadataValue,
    PermissionSet, PermissionState,
};
use crate::physics::MoveResult;
use crate::player::experience::Experience;
use crate::player::player_data::{PersistentEnderPearl, PersistentRootVehicle};
use crate::player::player_inventory::{
    MenuItemDisposition, MenuRemovalStatus, PlayerInventory, PlayerInventorySyncState,
};
use crate::server::{
    PlayerPacketTransition, Server,
    jobs::{JobPoll, ServerJob, ServerJobContext},
};
use crate::world::player_spawn_finder::{
    POLL_TIME_BUDGET, PlayerSpawnSearch, PlayerSpawnSearchPoll,
};
use steel_registry::vanilla_damage_types;

use steel_protocol::packets::{
    common::SCustomPayload,
    game::{CContainerClose, CGameEvent, GameEventType},
};
use steel_registry::RegistryEntry;
use steel_registry::damage_type::DeathMessageType;
use steel_registry::item_stack::ItemStack;
use steel_registry::items::ItemRef;
use steel_registry::stat::vanilla_stat_types;
use steel_utils::{BlockPos, BlockStateId, ChunkPos, DowncastType, DowncastTypeKey, UuidExt as _};

use crate::inventory::container::Container;

const HAT_MODEL_PART_MASK: i8 = 0b0100_0000;

const DROP_SPAM_THROTTLER_INCREMENT_STEP: i32 = 20;
const DROP_SPAM_THROTTLER_THRESHOLD: i32 = 1480;

static INTENTIONAL_GAME_DESIGN_STYLE: LazyLock<Interactivity> = LazyLock::new(|| Interactivity {
    click: Some(ClickEvent::open_url(
        "https://bugs.mojang.com/browse/MCPE-28723",
    )),
    hover: Some(HoverEvent::show_text("MCPE-28723")),
    ..Interactivity::new()
});

use crate::behavior::items::filled_map;
use crate::chunk::player_chunk_view::PlayerChunkView;
use crate::entity::entities::objects::projectiles::FishingHookEntity;
use crate::inventory::ender_chest::{PlayerEnderChestContainer, SyncPlayerEnderChest};
use crate::player::advancement::PlayerAdvancement;
use crate::player::chunk_sender::ChunkSender;
use crate::player::stats_counter::StatsCounter;
use crate::portal::{
    PortalTicketTarget, TeleportPostAction, TeleportPostTransition, TeleportTransition,
};
use crate::world::World;

/// A struct representing a player.
pub struct Player {
    /// The player's game profile.
    pub gameprofile: GameProfile,
    /// The player's connection (abstracted for testing).
    pub connection: Arc<PlayerConnection>,
    /// Stable connection session shared by every incarnation of this player.
    pub(crate) session: Arc<PlayerSession>,

    /// The world the player is in.
    pub world: ArcSwap<World>,

    /// Reference to the server (for entity ID generation, etc.).
    pub(crate) server: Weak<Server>,
    /// Runtime configuration shared with the server.
    pub(crate) config: Arc<RuntimeConfig>,

    /// Common entity fields (id, uuid, position, rotation, removal, callback).
    base: EntityBase,

    /// Client lifecycle flags.
    lifecycle: SyncMutex<PlayerLifecycleState>,
    /// Position captured when bad omen becomes raid omen.
    pub(crate) raid_omen_position: SyncMutex<Option<BlockPos>>,
    /// Overworld position at the last successful entry to the Nether.
    pub(crate) entered_nether_position: SyncMutex<Option<DVec3>>,

    /// Movement tracking state
    pub(crate) movement: SyncMutex<MovementState>,

    /// Synchronized entity data (health, pose, flags, etc.) for network sync.
    entity_data: SyncMutex<PlayerEntityData>,

    /// The last chunk position of the player.
    pub last_chunk_pos: SyncMutex<ChunkPos>,
    /// The last chunk tracking view of the player.
    pub last_tracking_view: SyncMutex<Option<PlayerChunkView>>,
    /// Whether the chunk map currently skips this player's chunk ticket.
    ///
    /// Vanilla `ChunkMap` keeps the per-player `skippedPlayer` flag; it is set
    /// by `spectatorsGenerateChunks` together with the spectator game mode and
    /// decides whether the player loads terrain around itself.
    pub skip_player: AtomicBool,
    /// Entity the player's camera is attached to, if any.
    ///
    /// Vanilla `ServerPlayer.camera`/`setCamera`; a spectator's `set_camera`
    /// stores the target here so later packets and teardown can restore it.
    camera_entity: SyncMutex<Option<SharedEntity>>,
    /// The client's settings/information (language, view distance, chat visibility, etc.).
    /// Updated when the client sends `SClientInformation` during config or play phase.
    client_information: SyncMutex<ClientInformation>,

    /// Current and previous game mode.
    game_modes: SyncMutex<PlayerGameModeState>,

    /// The player's inventory container (shared with `inventory_menu`).
    pub inventory: Shared<PlayerInventory>,

    /// Logical inventory slots that must be resent directly to this player's client.
    inventory_sync: SyncMutex<PlayerInventorySyncState>,

    /// The player's ender chest inventory.
    pub ender_chest_inventory: SyncPlayerEnderChest,

    /// Last main-hand stack used for vanilla attack-strength reset checks.
    last_item_in_main_hand: SyncMutex<ItemStack>,

    /// The player's inventory menu (always open, even when `container_id` is 0).
    inventory_menu: SyncMutex<Menu>,

    /// The currently open menu (None if player inventory is open).
    /// This is separate from `inventory_menu` which is always present.
    open_menu: SyncMutex<player_inventory::OpenMenuState>,

    /// Counter for generating container IDs (1-100, wraps around).
    container_counter: SyncMutex<ContainerCounter>,

    /// Pending server-initiated teleport state (ID, position, timeout).
    teleport_state: SyncMutex<TeleportState>,
    /// Vanilla item use cooldown groups.
    item_cooldowns: SyncMutex<ItemCooldowns>,

    /// Local tick and once-per-tick packet state.
    tick_state: SyncMutex<PlayerTickState>,
    /// Vanilla sleep/wake animation counter.
    sleep_state: SyncMutex<PlayerSleepState>,
    /// Persisted personal bed or respawn-anchor target.
    respawn_config: SyncMutex<Option<PlayerRespawnConfig>>,

    /// Recipes this player has unlocked and the state of their recipe book tabs.
    pub recipe_book: SyncMutex<recipe_book::ServerRecipeBook>,

    /// Player abilities (flight, invulnerability, build permissions, speeds, etc.)
    pub abilities: SyncMutex<Abilities>,

    /// Block breaking state machine.
    pub block_breaking: SyncMutex<BlockBreakingManager>,

    /// Shared living-entity runtime fields (attributes, speed, damage/death state).
    /// Vanilla: `LivingEntity` (L230-232) + `Entity.invulnerableTime` (L256).
    living_base: LivingEntityBase,

    /// Player food/hunger state (food level, saturation, exhaustion).
    pub food_data: SyncMutex<FoodData>,

    /// Delta-tracking state for `CSetHealth` deduplication.
    health_sync: SyncMutex<HealthSyncState>,

    /// The Player's Experience
    pub experience: SyncMutex<Experience>,

    /// Assigned groups, direct overrides, and the effective permission set.
    permissions: SyncMutex<PlayerPermissionState>,

    /// Vanilla `ServerPlayer.requestedDebugSubscriptions`: the raw set the client
    /// asked for. [`Self::debug_subscriptions`] filters it by permission.
    requested_debug_subscriptions: SyncMutex<Vec<i32>>,

    /// Cookie keys requested by the server (`CCookieRequest`) awaiting a single
    /// client `SCookieResponse`, plus the latest verified cookie values received.
    cookie_state: SyncMutex<CookieState>,

    /// Whether the server has sent `CStartConfiguration` and is awaiting the
    /// client's `SConfigurationAcknowledged` packet.
    awaiting_configuration_ack: AtomicBool,

    /// Most recent ping transaction id echoed by the client via `SPong`.
    last_pong_id: SyncMutex<Option<i32>>,

    /// Most recent `SCustomClickAction` identifier sent by a custom dialog button.
    last_custom_click_action: SyncMutex<Option<Identifier>>,

    /// Whether the player has completed the vanilla End credits flow.
    seen_credits: SyncMutex<bool>,

    /// Vanilla `ServerPlayer.postEffects`: the ordered set of `post_effect`
    /// registry ids the client applies to the whole screen, plus the dirty flag
    /// that makes the play tick push them.
    post_effects: SyncMutex<PlayerPostEffects>,

    /// Vanilla `ServerPlayer.wonGame`; transient while the End credits screen is open.
    won_game: SyncMutex<bool>,

    /// Monotonic counter bumped on world teleport/reset. The chunk sending tick
    /// snapshots this before encoding and compares after to detect stale batches.
    pub chunk_send_epoch: SyncMutex<u32>,

    /// Domain-residence identity and persisted entities awaiting restoration.
    residence: SyncMutex<PlayerResidenceState>,
    /// In-flight ender pearls thrown by this player, kept weakly so they persist
    /// with the player and re-spawn on login (vanilla `ServerPlayer.enderPearls`).
    ender_pearls: SyncMutex<Vec<Weak<dyn Entity>>>,

    /// Active fishing hook, kept weakly because the world owns live entities.
    pub(crate) fishing: SyncMutex<Option<Weak<FishingHookEntity>>>,

    /// The counter keeping track of this player's statistics.
    stats: SyncMutex<StatsCounter>,
    /// Vanilla `ServerPlayer.wardenSpawnTracker`: the per-player warning state
    /// the sculk shrieker climbs before a warden emerges.
    warden_spawn_tracker: SyncMutex<WardenSpawnTracker>,

    /// The last action time of this player.
    last_action_time: SyncMutex<Instant>,

    /// Manage the advancements of the player
    pub advancements: SyncMutex<PlayerAdvancement>,

    /// Remaining ticks of an active riptide spin attack, plus the enchantment
    /// level that scales its knockback. Vanilla `LivingEntity.autoSpinAttackTicks`.
    riptide: SyncMutex<Option<RiptideSpin>>,

    /// Where and when the current levitation started, for the `levitation`
    /// advancement trigger. Vanilla `LevitationMobEffect` keeps the same pair in
    /// `startLevitation`.
    levitation_start: SyncMutex<Option<(glam::DVec3, i32)>>,
}

/// The cookie keys the server asked for, plus the payloads the client answered
/// with.
type CookieState = (
    rustc_hash::FxHashSet<Identifier>,
    rustc_hash::FxHashMap<Identifier, ReceivedCookie>,
);

/// The answer the client gave to a cookie request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceivedCookie {
    /// The client sent the cookie payload.
    Value(Vec<u8>),
    /// The client answered without a payload.
    Empty,
}

/// Vanilla `LivingEntity` spin-attack activation ticks (`startAutoSpinAttack(20, ...)`).
pub const RIPTIDE_ACTIVATION_TICKS: i32 = 20;

/// Base damage of a riptide spin attack (`TridentItem.doHurtTarget` passes `8.0f`).
pub const RIPTIDE_SPIN_ATTACK_DAMAGE: f32 = 8.0;

/// Horizontal push applied to the first entity a spin attack sweeps into.
///
/// Vanilla `Player.causeExtraKnockback` with the plain `0.4` base knockback of an
/// attack that carries no knockback enchantment.
pub const SPIN_ATTACK_KNOCKBACK: f64 = 0.4;

/// State of an in-progress riptide spin attack.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiptideSpin {
    /// Ticks of spin attack left; vanilla starts at 20 per activation.
    pub remaining_ticks: i32,
    /// Riptide level, which scales the launch speed and the sweep reach.
    pub level: i32,
    /// Damage a swept entity takes; `autoSpinAttackDmg` in vanilla.
    pub damage: f32,
}

// SAFETY: This key is owned by Steel and uniquely identifies `Player`.
unsafe impl DowncastType for Player {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/player");
}

#[derive(Clone)]
struct PendingRootVehicleRestore {
    world: Identifier,
    root_vehicle: PersistentRootVehicle,
}

/// Runtime identity for one continuous stay in a Steel domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DomainResidenceToken(u64);

#[derive(Clone)]
struct PlayerResidenceState {
    token: DomainResidenceToken,
    pending_root_vehicle: Option<PendingRootVehicleRestore>,
    pending_ender_pearls: Vec<PersistentEnderPearl>,
}

impl PlayerResidenceState {
    const fn new() -> Self {
        Self {
            token: DomainResidenceToken(1),
            pending_root_vehicle: None,
            pending_ender_pearls: Vec::new(),
        }
    }

    fn advance(&mut self) -> DomainResidenceToken {
        let Some(next_token) = self.token.0.checked_add(1) else {
            panic!("domain residence token space exhausted");
        };
        self.token = DomainResidenceToken(next_token);
        self.pending_root_vehicle = None;
        self.pending_ender_pearls.clear();
        self.token
    }
}

/// Vanilla `TestBlockMode` wire ordinal (`ByIdMap` maps unknown ids to `START`).
const fn test_mode_from_ordinal(ordinal: i32) -> TestBlockMode {
    match ordinal {
        1 => TestBlockMode::Log,
        2 => TestBlockMode::Fail,
        3 => TestBlockMode::Accept,
        _ => TestBlockMode::Start,
    }
}

/// Vanilla `StructureBlockEntity.UpdateType.SAVE_AREA` ordinal.
///
/// The enum is `UPDATE_DATA(0)`, `SAVE_AREA(1)`, `LOAD_AREA(2)`,
/// `SCAN_AREA(3)`; `UPDATE_DATA` needs no action because the fields above are
/// the whole of it.
const STATUS_UPDATE_SAVE_AREA: i32 = 1;
/// Vanilla `StructureBlockEntity.UpdateType.LOAD_AREA` ordinal.
const STATUS_UPDATE_LOAD_AREA: i32 = 2;
/// Vanilla `StructureBlockEntity.UpdateType.SCAN_AREA` ordinal.
const STATUS_UPDATE_SCAN_AREA: i32 = 3;

impl Player {
    /// Fires the `recipe_crafted` trigger for a recipe this player just used.
    ///
    /// Vanilla carries the crafted stack in the predicate; Steel only knows the
    /// recipe key, so every `recipe_crafted` criterion of that recipe progresses.
    /// Fires `recipe_unlocked`, mirroring vanilla `ServerPlayer.awardRecipes`.
    pub fn trigger_recipe_unlocked(&self, recipe: &Identifier) -> usize {
        self.trigger_advancement_with_context(&RECIPE_UNLOCKED, &TriggerContext::recipe(recipe))
    }

    /// Fires the `recipe_crafted` advancement criterion for `recipe`.
    pub fn trigger_recipe_crafted(&self, recipe: &Identifier) -> usize {
        let _ = recipe;
        self.trigger_advancement(&RECIPE_CRAFTED)
    }

    /// Returns the entity this player's camera is attached to.
    #[must_use]
    pub fn camera_entity(&self) -> Option<SharedEntity> {
        self.camera_entity.lock().clone()
    }

    /// Attaches the player's camera to `entity`, or restores the player's own
    /// camera when `None`.
    ///
    /// Vanilla `ServerPlayer.setCamera`; callers are responsible for sending the
    /// matching `SetCamera` packet.
    pub fn set_camera_entity(&self, entity: Option<SharedEntity>) {
        *self.camera_entity.lock() = entity;
    }

    /// Fires an advancement trigger for this player, granting every matching
    /// criterion. Mirrors vanilla's `CriteriaTriggers` dispatch.
    pub fn trigger_advancement(&self, trigger: &Identifier) -> usize {
        self.advancements.lock().trigger(self, trigger)
    }

    /// Fires a trigger together with the event that caused it.
    ///
    /// Criteria carry vanilla `conditions` objects; passing the event lets
    /// [`crate::player::advancement::predicate::matches`] decide which
    /// advancements actually apply instead of granting every criterion with a
    /// matching trigger id.
    pub fn trigger_advancement_with_context(
        &self,
        trigger: &Identifier,
        context: &TriggerContext,
    ) -> usize {
        self.advancements
            .lock()
            .trigger_with_context(self, trigger, context)
    }

    const USING_ITEM_FLAG: i8 = 1;
    const OFF_HAND_ACTIVE_ITEM_FLAG: i8 = 1 << 1;

    /// Returns the chunk sender owned by this player's connection session.
    pub(crate) fn chunk_sender(&self) -> &SyncMutex<ChunkSender> {
        &self.session.chunk_sender
    }

    /// Returns chat protocol state owned by this connection session.
    pub(crate) fn chat(&self) -> &SyncMutex<ChatState> {
        &self.session.chat
    }

    /// Returns the hand currently driving active item use.
    #[must_use]
    pub fn active_item_use_hand(&self) -> Option<InteractionHand> {
        self.living_base
            .active_item_use()
            .map(|active| active.hand())
    }

    /// Starts using the item currently held in `hand`.
    pub fn start_using_item(&self, hand: InteractionHand) {
        let item = {
            let inventory = self.inventory.lock();
            let item = inventory.get_item_in_hand(hand);
            item.clone()
        };
        let duration = ITEM_BEHAVIORS
            .get_behavior(item.item())
            .get_use_duration(&item, self);
        if self.living_base.start_using_item(hand, &item, duration) {
            let mut entity_data = self.entity_data.lock();
            let flags = entity_data.living_entity().living_entity_flags.get();
            let mut flags = *flags | Self::USING_ITEM_FLAG;
            if hand == InteractionHand::OffHand {
                flags |= Self::OFF_HAND_ACTIVE_ITEM_FLAG;
            } else {
                flags &= !Self::OFF_HAND_ACTIVE_ITEM_FLAG;
            }
            entity_data
                .living_entity_mut()
                .living_entity_flags
                .set(flags);
        }
    }

    fn stop_using_item(&self) {
        self.living_base.stop_using_item();
        let mut entity_data = self.entity_data.lock();
        let flags = *entity_data.living_entity().living_entity_flags.get();
        entity_data
            .living_entity_mut()
            .living_entity_flags
            .set(flags & !Self::USING_ITEM_FLAG);
    }

    /// Releases the currently used item and invokes its release hook.
    pub fn release_using_item(&self, entity: &SharedEntity) {
        let Some(active) = self.living_base.active_item_use() else {
            return;
        };
        let behavior = ITEM_BEHAVIORS.get_behavior(active.item());
        self.release_using_item_with_behavior(entity, behavior, active);
    }

    fn release_using_item_with_behavior(
        &self,
        entity: &SharedEntity,
        behavior: &dyn ItemBehavior,
        active: ActiveItemUseState,
    ) {
        let hand = active.hand();
        let item_matches = {
            let inventory = self.inventory.lock();
            inventory.get_item_in_hand(hand).item() == active.item()
        };
        if !item_matches {
            self.stop_using_item();
            return;
        }
        // Read a copy rather than clearing the slot,
        // so any inventory-touching side effect from
        // `release_using` never sees the hand as vacant.
        let mut item = {
            let inventory = self.inventory.lock();
            let current = inventory.get_item_in_hand(hand);
            current.clone()
        };
        let stack_before_using = item.clone();
        let world = self.get_world();
        let apply_side_effects =
            behavior.release_using(&mut item, &world, self, active.remaining_ticks());
        let use_on_release = behavior.use_on_release(&item);
        if apply_side_effects {
            item = apply_use_remainder(&stack_before_using, item, self);
            self.apply_item_use_cooldown(&stack_before_using);
        }
        self.inventory.lock().set_item_in_hand(hand, item);
        // we re-read active here since behavior.release_using might have already ended the use
        if use_on_release && let Some(active) = self.living_base.active_item_use() {
            self.tick_active_item_use_with_behavior(entity, behavior, active);
        }
        self.stop_using_item();
    }

    fn tick_active_item_use(&self, entity: &SharedEntity) {
        let Some(active) = self.living_base.active_item_use() else {
            return;
        };
        let behavior = ITEM_BEHAVIORS.get_behavior(active.item());
        self.tick_active_item_use_with_behavior(entity, behavior, active);
    }

    fn tick_active_item_use_with_behavior(
        &self,
        entity: &SharedEntity,
        behavior: &dyn ItemBehavior,
        active: ActiveItemUseState,
    ) {
        let hand = active.hand();
        let item_matches = {
            let inventory = self.inventory.lock();
            inventory.get_item_in_hand(hand).item() == active.item()
        };
        if !item_matches {
            self.stop_using_item();
            return;
        }
        let mut item = {
            let inventory = self.inventory.lock();
            let current = inventory.get_item_in_hand(hand);
            current.clone()
        };
        let world = self.get_world();
        let Some(user) = LivingEntityRef::new(entity) else {
            panic!("active item use requires a living entity");
        };
        // Vanilla `Item.use` fires `using_item` once per tick while the item is
        // held in use, which is how `adventure/salvage_sherd`-style criteria see
        // sustained use.
        self.trigger_advancement_with_context(
            &USING_ITEM,
            &TriggerContext::item(&item.item().key).with_item_stack(&item),
        );
        behavior.on_use_tick(&world, user, &mut item, active.remaining_ticks());

        if self.active_item_use_hand() != Some(hand) {
            self.inventory.lock().set_item_in_hand(hand, item);
            return;
        }
        let Some(active) = self.living_base.decrement_active_item_use() else {
            self.inventory.lock().set_item_in_hand(hand, item);
            return;
        };
        let use_on_release = behavior.use_on_release(&item);
        if active.remaining_ticks() == 0 && !use_on_release && !item.is_empty() {
            let stack_before_finish = item.clone();
            item = behavior.finish_using(&mut item, &world, user);
            self.apply_item_use_cooldown(&stack_before_finish);
            self.stop_using_item();
        }

        self.inventory.lock().set_item_in_hand(hand, item);
    }

    /// Vanilla `LivingEntity.isAutoSpinAttack()`.
    #[must_use]
    pub fn is_auto_spin_attack(&self) -> bool {
        self.riptide.lock().is_some()
    }

    /// Vanilla `Player.autoSpinAttackDmg`: the base damage an active spin uses
    /// instead of the `ATTACK_DAMAGE` attribute, or `None` outside a spin.
    #[must_use]
    pub fn riptide_spin_damage(&self) -> Option<f32> {
        self.riptide.lock().map(|spin| spin.damage)
    }

    /// Vanilla `EnchantmentHelper.getTridentSpinAttackStrength`.
    ///
    /// Riptide contributes `1.5 + 0.75 * (level - 1)` through the
    /// `minecraft:trident_spin_attack_strength` effect (`minecraft:add` of a
    /// `minecraft:linear` value) in `data/minecraft/enchantment/riptide.json`.
    /// The enchantment cannot stack, so only one level ever applies.
    #[must_use]
    #[expect(
        clippy::unused_self,
        reason = "public API mirrors the vanilla signature"
    )]
    pub fn trident_spin_attack_strength(&self, riptide_level: i32) -> f32 {
        if riptide_level <= 0 {
            return 0.0;
        }
        1.5 + 0.75 * (riptide_level - 1) as f32
    }

    /// Vanilla `LivingEntity.startAutoSpinAttack(20, 8.0f, stack)`.
    ///
    /// The launch impulse is applied by `TridentItem` *before* this call, exactly
    /// as vanilla orders it.
    pub fn start_auto_spin_attack(&self, level: i32) {
        *self.riptide.lock() = Some(RiptideSpin {
            remaining_ticks: RIPTIDE_ACTIVATION_TICKS,
            level: level.max(1),
            damage: RIPTIDE_SPIN_ATTACK_DAMAGE,
        });
        self.sync_pose_from_state();
    }

    /// Vanilla `LivingEntity.livingEntityTick`: one tick of the spin attack.
    ///
    /// Vanilla only decrements `autoSpinAttackTicks` and runs
    /// `checkAutoSpinAttack`; it never re-applies a launch impulse, so neither
    /// does this.
    pub(crate) fn tick_riptide(&self) {
        if self.riptide.lock().is_none() {
            return;
        }
        // Decrement first, then test contacts: vanilla's order.
        if let Some(spin) = self.riptide.lock().as_mut() {
            spin.remaining_ticks -= 1;
        }

        self.check_auto_spin_attack();

        let finished = self
            .riptide
            .lock()
            .as_ref()
            .is_none_or(|spin| spin.remaining_ticks <= 0);
        if finished {
            self.riptide.lock().take();
            self.sync_pose_from_state();
        }
    }

    /// Vanilla `LivingEntity.checkAutoSpinAttack`.
    ///
    /// The sweep box is the union of the bounding box before and after movement;
    /// the first living entity inside it ends the attack after taking the hit,
    /// and a horizontal block collision also ends it.
    fn check_auto_spin_attack(&self) {
        // Vanilla unions the box captured before travel with the current one, so
        // the sweep also catches entities the player moved past this tick.
        let current = self.bounding_box();
        let previous = current.translate(self.old_position() - self.position());
        let sweep = WorldAabb::encapsulating(&previous, &current);

        let mut swept = None;
        for target in self.get_world().get_entities_in_aabb(&sweep) {
            if target.id() == self.id() || !target.is_living_entity() {
                continue;
            }
            swept = Some(target);
            break;
        }

        if let Some(target) = swept {
            // Vanilla `Player.doAutoAttackOnTouch` → `attack`, which uses
            // `autoSpinAttackDmg` and the spin-attack item stack as the weapon.
            // `attack` needs the world's own `Arc<Player>`, so resolve it by UUID
            // instead of fabricating a second allocation.
            if let Some(arc) = self
                .level()
                .and_then(|world| world.players.get_by_uuid(&self.base().uuid()))
            {
                let _ = arc.attack(&target);
            }
            if let Some(spin) = self.riptide.lock().as_mut() {
                spin.remaining_ticks = 0;
            }
            // Vanilla: `setDeltaMovement(getDeltaMovement().scale(-0.2))`.
            self.set_velocity(self.velocity() * -0.2);
            self.mark_velocity_sync();
        } else if self.horizontal_collision()
            && let Some(spin) = self.riptide.lock().as_mut()
        {
            spin.remaining_ticks = 0;
        }
    }

    /// Stops an active spin attack and restores the normal pose.
    pub fn stop_auto_spin_attack(&self) {
        if self.riptide.lock().take().is_some() {
            self.sync_pose_from_state();
        }
    }

    /// Recomputes the synced pose, including the riptide spin pose.
    fn sync_pose_from_state(&self) {
        let pose = self.get_desired_pose();
        self.set_pose(pose);
    }

    /// Returns the player's configured main arm.
    #[must_use]
    pub fn main_arm(&self) -> HumanoidArm {
        self.client_information.lock().main_hand
    }

    #[must_use]
    pub(crate) fn shows_hat(&self) -> bool {
        let model_customization = *self
            .entity_data
            .lock()
            .avatar()
            .player_mode_customization
            .get();
        model_customization & HAT_MODEL_PART_MASK != 0
    }

    fn apply_client_information_to_entity_data(
        data: &mut PlayerEntityData,
        client_information: &ClientInformation,
    ) {
        let avatar = data.avatar_mut();
        avatar.player_main_hand.set(client_information.main_hand);
        avatar
            .player_mode_customization
            .set(client_information.model_customization.cast_signed());
    }

    /// Computes the start (eye position) and end positions for a raytrace.
    pub fn get_ray_endpoints(&self) -> (DVec3, DVec3) {
        let pos = self.position();
        let start_pos = DVec3::new(pos.x, self.get_eye_y(), pos.z);
        let block_interaction_range = self.block_interaction_range();
        let direction = self.look_angle() * block_interaction_range;

        let end_pos = start_pos + direction;
        (start_pos, end_pos)
    }

    /// Returns the player's current game mode.
    #[must_use]
    pub fn game_mode(&self) -> GameType {
        self.game_modes.lock().current()
    }

    /// Returns the player's previous game mode.
    #[must_use]
    pub fn previous_game_mode(&self) -> Option<GameType> {
        self.game_modes.lock().previous()
    }

    /// Restores current and previous game mode from persistent player data.
    pub(crate) fn restore_game_modes(&self, current: GameType, previous: Option<GameType>) {
        self.game_modes.lock().set_pair(current, previous);
    }

    /// Changes the current game mode and records the old current mode as previous.
    fn change_game_mode_state(&self, game_mode: GameType) -> bool {
        let was_spectator = self.game_modes.lock().current() == GameType::Spectator;
        let changed = self.game_modes.lock().change_current(game_mode);
        // Vanilla `ServerPlayer.setGameMode` drops a spectator camera when the
        // player leaves spectator mode.
        if changed && was_spectator && game_mode != GameType::Spectator {
            self.set_camera_entity(None);
        }
        changed
    }

    /// Creates a new player.
    #[expect(
        clippy::too_many_arguments,
        reason = "player construction requires explicit connection, session, world, and identity owners"
    )]
    pub fn new(
        gameprofile: GameProfile,
        connection: Arc<PlayerConnection>,
        session: Arc<PlayerSession>,
        world: Arc<World>,
        server: Weak<Server>,
        config: Arc<RuntimeConfig>,
        entity_id: i32,
        client_information: ClientInformation,
    ) -> Self {
        // Create a single shared inventory container used by both the player and inventory menu
        let inventory = Arc::new(SyncMutex::new(PlayerInventory::new()));
        let ender_chest_inventory = Arc::new(SyncMutex::new(PlayerEnderChestContainer::new()));

        let pos = DVec3::new(0.0, 0.0, 0.0);

        let equipment = Arc::clone(&inventory);
        let living_base = LivingEntityBase::with_equipment(&vanilla_entities::PLAYER, equipment);
        let player_uuid = gameprofile.id();
        let world_ref = Arc::downgrade(&world);
        Self {
            gameprofile,
            connection,
            session,

            world: ArcSwap::new(world),
            server,
            config,
            base: EntityBase::with_uuid(
                entity_id,
                player_uuid,
                pos,
                Self::dimensions_for_pose(EntityPose::Standing),
                world_ref,
            ),
            lifecycle: SyncMutex::new(PlayerLifecycleState::default()),
            raid_omen_position: SyncMutex::new(None),
            entered_nether_position: SyncMutex::new(None),
            movement: SyncMutex::new(MovementState::new()),
            entity_data: SyncMutex::new({
                let mut data = PlayerEntityData::new();
                living_base.initialize_synced_data(&mut data);
                Self::apply_client_information_to_entity_data(&mut data, &client_information);
                data
            }),
            last_chunk_pos: SyncMutex::new(ChunkPos::new(0, 0)),
            last_tracking_view: SyncMutex::new(None),
            skip_player: AtomicBool::new(false),
            camera_entity: SyncMutex::new(None),
            client_information: SyncMutex::new(client_information),
            game_modes: SyncMutex::new(PlayerGameModeState::new(GameType::Survival)),
            inventory: Arc::clone(&inventory),
            inventory_sync: SyncMutex::new(PlayerInventorySyncState::new()),
            ender_chest_inventory,
            last_item_in_main_hand: SyncMutex::new(ItemStack::empty()),
            inventory_menu: SyncMutex::new(inventory_menu(inventory)),
            open_menu: SyncMutex::new(player_inventory::OpenMenuState::new()),
            container_counter: SyncMutex::new(ContainerCounter::new()),
            teleport_state: SyncMutex::new(TeleportState::new()),
            levitation_start: SyncMutex::new(None),
            item_cooldowns: SyncMutex::new(ItemCooldowns::default()),
            tick_state: SyncMutex::new(PlayerTickState::new()),
            sleep_state: SyncMutex::new(PlayerSleepState::new()),
            respawn_config: SyncMutex::new(None),
            recipe_book: SyncMutex::new(recipe_book::ServerRecipeBook::new()),
            abilities: SyncMutex::new(Abilities::default()),
            block_breaking: SyncMutex::new(BlockBreakingManager::new()),
            living_base,
            food_data: SyncMutex::new(FoodData::new()),
            health_sync: SyncMutex::new(HealthSyncState::new()),
            experience: SyncMutex::new(Experience::default()),
            permissions: SyncMutex::new(PlayerPermissionState::default()),
            requested_debug_subscriptions: SyncMutex::new(Vec::new()),
            cookie_state: SyncMutex::new((
                rustc_hash::FxHashSet::default(),
                rustc_hash::FxHashMap::default(),
            )),
            awaiting_configuration_ack: AtomicBool::new(false),
            last_pong_id: SyncMutex::new(None),
            last_custom_click_action: SyncMutex::new(None),
            seen_credits: SyncMutex::new(false),
            post_effects: SyncMutex::new(PlayerPostEffects::new_pending()),
            won_game: SyncMutex::new(false),
            chunk_send_epoch: SyncMutex::new(0),
            residence: SyncMutex::new(PlayerResidenceState::new()),
            ender_pearls: SyncMutex::new(Vec::new()),
            fishing: SyncMutex::new(None),
            stats: SyncMutex::new(StatsCounter::new()),
            warden_spawn_tracker: SyncMutex::new(WardenSpawnTracker::new()),
            last_action_time: SyncMutex::new(Instant::now()),
            advancements: SyncMutex::new(PlayerAdvancement::default()),
            riptide: SyncMutex::new(None),
        }
    }

    /// Returns the active fishing hook, clearing a stale reference after removal.
    pub fn fishing_hook(&self) -> Option<Arc<FishingHookEntity>> {
        let mut fishing = self.fishing.lock();
        let hook = fishing.as_ref().and_then(Weak::upgrade);
        if hook.is_none() {
            *fishing = None;
        }
        hook
    }

    /// Records the hook currently owned by this player.
    pub fn set_fishing_hook(&self, hook: &Arc<FishingHookEntity>) {
        *self.fishing.lock() = Some(Arc::downgrade(hook));
    }

    /// Clears `hook` if it is still this player's active fishing hook.
    pub fn clear_fishing_hook(&self, hook: &FishingHookEntity) {
        let mut fishing = self.fishing.lock();
        if fishing
            .as_ref()
            .and_then(Weak::upgrade)
            .is_some_and(|active| ptr::eq(active.as_ref(), hook))
        {
            *fishing = None;
        }
    }

    fn send_experience_packet_if_dirty(&self) {
        let experience_packet = {
            let mut experience = self.experience.lock();
            if experience.dirty {
                experience.dirty = false;
                Some(CSetExperience {
                    progress: experience.progress(),
                    level: experience.level(),
                    total_experience: experience.total_points(),
                })
            } else {
                None
            }
        };
        if let Some(packet) = experience_packet {
            self.send_packet(packet);
        }
    }

    /// Ticks the death animation timer.
    /// Vanilla: `LivingEntity.tickDeath()` (not overridden by `ServerPlayer`).
    fn tick_death(&self) {
        let death_time = self.living_base.increment_death_time();

        if death_time >= DEATH_DURATION && !self.is_removed() {
            let world = self.get_world();
            let chunk_pos = *self.last_chunk_pos.lock();
            world.broadcast_to_nearby(
                chunk_pos,
                CEntityEvent {
                    entity_id: self.id(),
                    event: EntityStatus::Poof,
                },
                None,
            );

            world.unregister_player_entity(self);
            world.chunk_map.remove_player(self);
            world.entity_tracker().on_player_leave(self);
            world.player_area_map.remove_by_entity_id(self.id());
            self.set_removed(RemovalReason::Killed);
            assert_eq!(
                self.remove_all_menus_with_disposition(MenuItemDisposition::Drop),
                MenuRemovalStatus::Complete,
                "death removal menu cleanup must run outside a menu callback"
            );
        }
    }

    /// Returns the value of a custom statistic for this player.
    ///
    /// Vanilla `ServerPlayer.getStats().getValue(Stats.CUSTOM.get(...))`.
    #[must_use]
    pub fn custom_stat(&self, stat: CustomStatRef) -> i32 {
        self.stats
            .lock()
            .stats
            .get(&vanilla_stat_types::CUSTOM.get(stat))
            .map_or(0, |(count, _)| *count)
    }

    /// Returns a clone of the player's warden spawn tracker.
    ///
    /// Vanilla keeps `WardenSpawnTracker` on the `ServerPlayer` and mutates it
    /// through the optional; the shrieker reads, bumps and copies it.
    #[must_use]
    /// Vanilla `ServerPlayer.addPostEffect`; returns whether the list changed.
    pub fn add_post_effect(&self, effect: Identifier) -> bool {
        self.post_effects.lock().add(effect)
    }

    /// Vanilla `ServerPlayer.removePostEffect`.
    pub fn remove_post_effect(&self, effect: &Identifier) -> bool {
        self.post_effects.lock().remove(effect)
    }

    /// Vanilla `ServerPlayer.clearPostEffects`.
    pub fn clear_post_effects(&self) -> bool {
        self.post_effects.lock().clear()
    }

    /// Vanilla `ServerPlayer.getPostEffects`.
    #[must_use]
    /// Returns the ordered list of screen post-effect identifiers active on this player.
    pub fn post_effects(&self) -> Vec<Identifier> {
        self.post_effects.lock().effects().to_vec()
    }

    /// Replaces the list, used when a saved player is restored.
    pub fn replace_post_effects(&self, effects: Vec<Identifier>) {
        self.post_effects.lock().replace(effects);
    }

    /// Pushes the list when it changed since the last push.
    ///
    /// Vanilla does this at the end of `ServerPlayer.tick` and again from
    /// `PlayerList.placeNewPlayer`, which is why a fresh player always receives
    /// one packet.
    pub(crate) fn flush_post_effects(&self) {
        let packet = self.post_effects.lock().take_update();
        if let Some(packet) = packet {
            self.send_packet(packet);
        }
    }

    /// Returns a copy of this player's current Warden warning tracker (`getWardenSpawnTracker`).
    #[must_use]
    pub fn warden_spawn_tracker(&self) -> WardenSpawnTracker {
        *self.warden_spawn_tracker.lock()
    }

    /// Replaces the player's tracker, as vanilla `copyData` does.
    pub fn set_warden_spawn_tracker(&self, tracker: WardenSpawnTracker) {
        *self.warden_spawn_tracker.lock() = tracker;
    }

    /// Vanilla `WardenSpawnTracker.tick`, called from the player tick.
    pub fn tick_warden_spawn_tracker(&self) {
        self.warden_spawn_tracker.lock().tick();
    }

    /// Ticks to award stats that are awarded based on ticks.
    fn tick_stats(&self) {
        // These stats are expressed in ticks.
        // We batch the stats so that the mutex for stats is only
        // locked once.
        let mut stats = self.stats.lock();
        stats.increment(
            vanilla_stat_types::CUSTOM.get(&vanilla_custom_stats::PLAY_TIME),
            1,
        );
        stats.increment(
            vanilla_stat_types::CUSTOM.get(&vanilla_custom_stats::TOTAL_WORLD_TIME),
            1,
        );

        if Entity::is_alive(self) {
            stats.increment(
                vanilla_stat_types::CUSTOM.get(&vanilla_custom_stats::TIME_SINCE_DEATH),
                1,
            );
        }
        if self.is_discrete() {
            stats.increment(
                vanilla_stat_types::CUSTOM.get(&vanilla_custom_stats::SNEAK_TIME),
                1,
            );
        }
        if !self.is_sleeping() {
            stats.increment(
                vanilla_stat_types::CUSTOM.get(&vanilla_custom_stats::TIME_SINCE_REST),
                1,
            );
        }
    }

    /// Vanilla `LevitationMobEffect.applyEffectTick`: while levitation is active the
    /// effect remembers where it started and fires the `levitation` trigger with
    /// the distance traveled and the effect duration so far.
    fn tick_levitation_trigger(&self) {
        let Some(_effect) = self.mob_effect(vanilla_mob_effects::LEVITATION) else {
            *self.levitation_start.lock() = None;
            return;
        };
        let position = self.position();
        let mut start = self.levitation_start.lock();
        let (start_position, start_tick) = *start.get_or_insert((position, self.tick_count()));
        let distance = position.distance(start_position);
        let duration = self.tick_count().saturating_sub(start_tick);
        drop(start);
        let mut context = TriggerContext::nether_travel(distance);
        context.duration = Some(duration);
        self.trigger_advancement_with_context(&LEVITATION, &context);
    }

    /// Vanilla `Entity.baseTick`: a player riding a vehicle while submerged in
    /// lava fires `ride_entity_in_lava` for the vehicle's type.
    fn tick_ride_entity_in_lava_trigger(&self) {
        if !self.is_passenger() || !self.is_in_lava() {
            return;
        }
        let Some(vehicle) = self.vehicle() else {
            return;
        };
        self.trigger_advancement_with_context(
            &RIDE_ENTITY_IN_LAVA,
            &TriggerContext::entity(&vehicle.entity_type().key),
        );
    }

    /// Immediately flushes dirty player entity data to tracking players and self.
    fn sync_entity_data(&self) {
        let dirty_values = self.entity_data.lock().pack_dirty();
        if let Some(dirty_values) = dirty_values {
            let packet = CSetEntityData::new(self.id(), dirty_values);
            self.get_world()
                .broadcast_to_entity_trackers(self.id(), packet.clone(), None);
            self.send_packet(packet);
        }
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        if !self.living_base.take_effects_dirty() {
            return;
        }

        let mut display = self.living_base.mob_effect_display_state();
        if self.game_mode() == GameType::Spectator {
            display.particles = ParticleList::default();
            display.invisible = true;
        }

        {
            let mut entity_data = self.entity_data.lock();
            let living = entity_data.living_entity_mut();
            living.effect_particles.set(display.particles);
            living.effect_ambience.set(display.ambient);
        }

        self.entity_data.set_base_invisible_flag(display.invisible);
        self.entity_data
            .set_base_glowing_flag(self.has_glowing_tag() || display.glowing);
    }

    /// Vanilla `ServerPlayer.die` dummy objective: `deathCount` on this domain's
    /// scoreboard, created on demand and incremented by one.
    fn increment_death_count_objective(&self, world: &Arc<World>) {
        self.increment_dummy_objective(world, "deathCount");
    }

    /// Vanilla `ServerPlayer.awardKillScore` dummy objective: `playerKillCount`.
    /// Returns whether `self` and `other` belong to the same scoreboard team.
    ///
    /// Vanilla asks the scoreboard for both holder teams; unteamed holders are
    /// never teammates, matching `PlayerTeam == null` checks.
    fn is_on_same_team_as(&self, world: &Arc<World>, other: &dyn Entity) -> bool {
        let Some(server) = self.try_server() else {
            return false;
        };
        let Some(scoreboard) = server.scoreboards.get(world.domain()) else {
            return false;
        };
        let Some(other) = other.as_player() else {
            return false;
        };
        let own = ScoreHolder::new(self.scoreboard_name());
        let theirs = ScoreHolder::new(other.scoreboard_name());
        let own_team = scoreboard.holder_team_name(&own);
        let other_team = scoreboard.holder_team_name(&theirs);
        own_team.is_some() && own_team == other_team
    }

    fn increment_player_kill_count_objective(&self) {
        let world = self.get_world();
        self.increment_dummy_objective(&world, "playerKillCount");
    }

    /// Increments one vanilla dummy scoreboard objective for this player,
    /// creating the objective when a server-side scoreboard exists for the
    /// domain. Missing scoreboards are ignored, like vanilla when the objective
    /// is not registered.
    fn increment_dummy_objective(&self, world: &Arc<World>, objective_name: &str) {
        let Some(server) = self.try_server() else {
            return;
        };
        let Some(scoreboard) = server.scoreboards.get(world.domain()) else {
            return;
        };
        let objective = scoreboard
            .objective(objective_name)
            .or_else(|| scoreboard.add_objective(objective_name).ok());
        let Some(objective) = objective else {
            return;
        };
        let holder = ScoreHolder::new(self.scoreboard_name());
        let current = scoreboard.score(&holder, &objective).unwrap_or(0);
        let _ = scoreboard.set_score(&holder, &objective, current.saturating_add(1));
    }

    /// Handles a custom payload packet (`minecraft:brand` length-prefixed UTF-8).
    #[expect(clippy::unused_self, reason = "player API method")]
    pub fn handle_custom_payload(&self, packet: SCustomPayload) {
        if packet.identifier == Identifier::vanilla_static("brand") {
            let mut cursor = Cursor::new(packet.payload.0.as_slice());
            if let Ok(brand) = <String as ReadFrom>::read(&mut cursor)
                && brand.len() <= 128
            {
                log::debug!("Player reported client brand: {brand}");
            }
        }
    }

    /// Vanilla `ServerGamePacketListenerImpl.handleSetGameRule`: applies each
    /// operator-submitted game rule update, ignoring unauthorized callers,
    /// unknown rule keys, and malformed values.
    pub fn handle_set_game_rule(&self, packet: SSetGameRule) {
        use steel_registry::game_rules::{GameRuleType, GameRuleValue};

        if !self.is_operator() {
            log::warn!(
                "Player {} tried to set game rule without permission",
                self.gameprofile.name()
            );
            return;
        }
        let Some(world) = Entity::level(self) else {
            return;
        };
        for entry in packet.entries {
            let Some(rule) = REGISTRY.game_rules.by_key(&entry.key) else {
                continue;
            };
            let value = match rule.value_type() {
                GameRuleType::Bool => match entry.value.as_str() {
                    "true" => GameRuleValue::new(true),
                    "false" => GameRuleValue::new(false),
                    _ => continue,
                },
                GameRuleType::Int => match entry.value.parse::<i32>() {
                    Ok(value)
                        if value >= rule.min_value().unwrap_or(i32::MIN)
                            && value <= rule.max_value().unwrap_or(i32::MAX) =>
                    {
                        GameRuleValue::new(value)
                    }
                    _ => continue,
                },
            };
            world.set_erased_game_rule(rule, value);
        }
    }

    /// Vanilla `ServerGamePacketListenerImpl.handleSetCommandBlock`: stores the
    /// command the operator typed into the block.
    ///
    /// Vanilla only accepts the packet from a creative-level-two operator and
    /// only when the block at the position really is a command block; the packet
    /// carries the mode and the three flags, not the power state.
    pub fn handle_set_command_block(&self, packet: SSetCommandBlock) {
        use steel_registry::blocks::properties::BlockStateProperties;
        use steel_registry::vanilla_blocks;

        if !self.is_operator() {
            return;
        }
        let Some(world) = Entity::level(self) else {
            return;
        };
        let Some(block_entity) = world.get_block_entity(packet.pos) else {
            return;
        };
        let Some(command_block) = block_entity
            .downcast_ref::<CommandBlockBlockEntity>()
            .cloned()
        else {
            return;
        };

        // Vanilla switches the block family to match the requested mode and
        // keeps the facing, then rewrites the block in place.
        let old_mode = command_block.mode();
        let current_state = world.get_block_state(packet.pos);
        let facing: steel_utils::Direction = current_state.get_value(&BlockStateProperties::FACING);
        let new_mode = CommandBlockMode::from_packet_ordinal(packet.mode);
        let base_block = match new_mode {
            Sequence => &vanilla_blocks::CHAIN_COMMAND_BLOCK,
            Auto => &vanilla_blocks::REPEATING_COMMAND_BLOCK,
            Redstone => &vanilla_blocks::COMMAND_BLOCK,
        };
        let conditional = packet.flags & 2 != 0;
        let new_state = base_block
            .default_state()
            .set_value(&BlockStateProperties::FACING, facing)
            .set_value(&BlockStateProperties::CONDITIONAL, conditional);
        if new_state != current_state {
            // Flag 2 = update clients only, exactly like vanilla.
            world.set_block(packet.pos, new_state, UpdateFlags::UPDATE_CLIENTS);
        }

        command_block.set_command(packet.command.clone());
        command_block.set_track_output(packet.flags & 1 != 0);
        command_block.set_automatic_and_schedule(packet.flags & 4 != 0, &world);
        if old_mode != new_mode {
            command_block.on_mode_switch(old_mode, &world);
        }

        // Vanilla answers with `advMode.setCommand.success` (or `.disabled` when
        // command blocks are switched off) once the command is not empty.
        if !packet.command.trim().is_empty() {
            let works = world.get_game_rule(&COMMAND_BLOCKS_WORK);
            // `advMode.setCommand.success` / `.disabled`, both take the command.
            let message = if works {
                translations::ADV_MODE_SET_COMMAND_SUCCESS.message([packet.command.clone()])
            } else {
                translations::ADV_MODE_SET_COMMAND_DISABLED.message([packet.command.clone()])
            };
            self.send_message(&message.component());
        }

        // Vanilla calls `BaseCommandBlock.onUpdated`, which sends the block
        // entity data to every viewer.
        world.broadcast_block_entity_if_needed(packet.pos);
    }

    /// Vanilla `ServerGamePacketListenerImpl.handleSetCommandMinecart`.
    ///
    /// Only a game master may edit a command minecart; the cart itself keeps the
    /// command, the output tracking and (when tracking is turned off) drops the
    /// last output. A non-empty command is acknowledged with
    /// `advMode.setCommand.success` or `.disabled`.
    pub fn handle_set_command_minecart(&self, packet: SSetCommandMinecart) {
        if !self.is_operator() {
            self.send_message(&translations::ADV_MODE_NOT_ALLOWED.msg().component());
            return;
        }
        let Some(world) = Entity::level(self) else {
            return;
        };
        let Some(entity) = world.get_entity_by_id(packet.entity_id) else {
            return;
        };
        let Some(minecart) = entity.downcast_ref::<CommandBlockMinecartEntity>() else {
            return;
        };

        minecart.set_command(packet.command.clone());
        minecart.set_track_output(packet.track_output);
        if !packet.track_output {
            minecart.set_last_output(TextComponent::plain(""));
        }

        let enabled = world.get_game_rule(&COMMAND_BLOCKS_WORK);
        if !packet.command.trim().is_empty() {
            let message = if enabled {
                translations::ADV_MODE_SET_COMMAND_SUCCESS.message([packet.command])
            } else {
                translations::ADV_MODE_SET_COMMAND_DISABLED.message([packet.command])
            };
            self.send_message(&message.component());
        }
    }

    /// Vanilla `ServerGamePacketListenerImpl.handleSetStructureBlock`: a game
    /// master updates every field of the structure block editor.
    ///
    /// The block entity stores all of it and mirrors the mode into the block
    /// state. Vanilla then runs the requested update type (`SAVE_AREA`,
    /// `LOAD_AREA`, `SCAN_AREA`) and reports success or failure; Steel has no
    /// runtime structure manager, so the update types that need one answer with
    /// vanilla's `structure_block.` failure keys instead of doing nothing.
    pub fn handle_set_structure_block(&self, packet: SSetStructureBlock) {
        use steel_registry::blocks::properties::StructureMode;

        if !self.has_game_master_permissions() {
            return;
        }
        let Some(world) = Entity::level(self) else {
            return;
        };
        let Some(block_entity) = world.get_block_entity(packet.pos) else {
            return;
        };
        let Some(structure) = block_entity.downcast_ref::<StructureBlockEntity>() else {
            return;
        };

        let mode = StructureMode::from_ordinal(packet.mode);
        structure.set_mode(mode);
        structure.set_structure_name(&packet.name);
        structure.set_structure_pos(packet.offset());
        structure.set_structure_size(packet.size());
        structure.set_mirror(packet.mirror);
        structure.set_rotation(packet.rotation);
        structure.set_metadata(&packet.metadata);
        structure.set_ignore_entities(packet.ignore_entities());
        structure.set_strict(packet.strict());
        structure.set_show_air(packet.show_air());
        structure.set_show_bounding_box(packet.show_bounding_box());
        structure.set_integrity(packet.integrity_clamped());
        structure.set_seed(packet.seed);
        structure.set_changed();

        // Vanilla answers every update type with a chat message. Steel has no
        // runtime structure manager (`ServerLevel.getStructureManager`), so the
        // three area operations cannot run; each one reports the vanilla key it
        // would report on failure, which keeps the client's chat identical to a
        // vanilla world where the structure is missing.
        let name = structure.structure_name();
        if !structure.has_structure_name() {
            self.send_message(
                &translations::STRUCTURE_BLOCK_INVALID_STRUCTURE_NAME
                    .message([packet.name])
                    .component(),
            );
            return;
        }
        match packet.update_type {
            // `StructureBlockEntity.saveStructure` returns `false` without a
            // structure manager.
            STATUS_UPDATE_SAVE_AREA => {
                log::warn!(
                    "structure block at {:?} cannot save '{name}': Steel has no structure manager",
                    packet.pos
                );
                self.send_message(
                    &translations::STRUCTURE_BLOCK_SAVE_FAILURE
                        .message([name])
                        .component(),
                );
            }
            // `StructureBlockEntity.isStructureLoadable` is `false` without a
            // structure manager, which is vanilla's `load_not_found` branch.
            STATUS_UPDATE_LOAD_AREA => {
                log::warn!(
                    "structure block at {:?} cannot load '{name}': Steel has no structure manager",
                    packet.pos
                );
                self.send_message(
                    &translations::STRUCTURE_BLOCK_LOAD_NOT_FOUND
                        .message([name])
                        .component(),
                );
            }
            // `StructureBlockEntity.detectSize` needs to walk every corner block
            // entity within 80 blocks; Steel does not expose a block-entity box
            // query, so the scan reports vanilla's failure branch.
            STATUS_UPDATE_SCAN_AREA => {
                log::warn!(
                    "structure block at {:?} cannot scan '{name}': Steel has no block-entity box query",
                    packet.pos
                );
                self.send_message(&translations::STRUCTURE_BLOCK_SIZE_FAILURE.msg().component());
            }
            // `UPDATE_DATA` (0) and unknown ordinals only store the fields,
            // which happened above.
            _ => {}
        }
    }

    /// Vanilla `ServerGamePacketListenerImpl.handlePlaceRecipe`.
    ///
    /// The server alone decides what moves: `resetLastActionTime`, then the
    /// container-id gate, the menu validity check, the recipe-book membership
    /// check, and finally the menu's own `handlePlacement`. A ghost recipe is
    /// echoed back so the client can draw the outline.
    pub fn handle_place_recipe(&self, packet: SPlaceRecipe) {
        self.reset_last_action_time();
        if self.is_spectator() {
            return;
        }

        let mut open_menu = self.open_menu.lock();
        let mut menu = open_menu.take_menu();
        drop(open_menu);

        let container_id;
        let outcome = match menu.as_mut() {
            Some(active) => {
                container_id = i32::from(active.container_id());
                if container_id != packet.container_id {
                    None
                } else if !active.still_valid(self) {
                    log::debug!(
                        "Player {} interacted with invalid menu",
                        self.gameprofile.name()
                    );
                    None
                } else {
                    self.place_recipe_into(active, &packet)
                }
            }
            None if packet.container_id == 0 => {
                container_id = 0;
                let mut inventory_menu = self.inventory_menu.lock();
                self.place_recipe_into(&mut inventory_menu, &packet)
            }
            None => return,
        };

        if let Some(active) = menu {
            self.open_menu.lock().put_menu(active);
        }

        // Vanilla answers `PLACE_GHOST_RECIPE` with the full display of the
        // recipe it could not place, not with the id the client sent.
        if let Some(outcome) = outcome
            && outcome.action == PostPlaceAction::PlaceGhostRecipe
        {
            self.send_packet(CPlaceGhostRecipe {
                container_id,
                display: outcome.display,
            });
        }
    }

    /// The per-menu half of [`Self::handle_place_recipe`], with the menu taken
    /// out of the open-menu slot.
    fn place_recipe_into(
        &self,
        menu: &mut Menu,
        packet: &SPlaceRecipe,
    ) -> Option<PlacementOutcome> {
        use crate::player::recipe_placement::PlacementRequest;
        use steel_registry::REGISTRY;

        // Vanilla `getRecipeFromDisplay` + `getRecipeBook().contains`.
        let key = self.recipe_book.lock().recipe_at(packet.recipe)?.clone();
        let recipe = REGISTRY.recipes.by_key(&key)?;

        let properties = recipe.data().properties()?;
        let placement = properties.placement.as_ref()?;
        if placement.is_impossible_to_place() {
            log::debug!(
                "Player {} tried to place impossible recipe {key}",
                self.gameprofile.name()
            );
            return None;
        }

        let display = display_for_recipe(recipe)?;
        let recipe_size = match &display {
            CraftingShaped { width, height, .. } => (
                usize::try_from(*width).unwrap_or(0),
                usize::try_from(*height).unwrap_or(0),
            ),
            CraftingShapeless { ingredients, .. } => (ingredients.len(), 1),
            Furnace { .. } => (1, 1),
            _ => return None,
        };

        let request = PlacementRequest {
            recipe_key: &key,
            placement,
            recipe_size,
            use_max_items: packet.make_all,
            // Vanilla `handlePlaceRecipe` passes `player.isCreative()`.
            allow_dropping_items_to_clear: self.has_infinite_materials(),
        };

        let mut guard = menu.behavior().lock_all_containers();
        let action = menu.handle_placement(&mut guard, self, request);
        drop(guard);
        Some(PlacementOutcome {
            action: action?,
            display,
        })
    }

    /// Vanilla `ServerGamePacketListenerImpl.handleTestInstanceBlockAction`.
    ///
    /// Only a game master reaches the block entity. Actions other than `INIT`
    /// and `QUERY` store the client payload and then run the requested action;
    /// the read-only pair answers with a status packet instead.
    pub fn handle_test_instance_block_action(&self, packet: STestInstanceBlockAction) {
        use steel_protocol::packets::game::{CTestInstanceBlockStatus, TestInstanceBlockAction};

        if !self.has_game_master_permissions() {
            return;
        }
        let Some(world) = Entity::level(self) else {
            return;
        };
        let Some(block_entity) = world.get_block_entity(packet.pos) else {
            return;
        };
        let Some(test_instance) = block_entity.downcast_ref::<TestInstanceBlockEntity>() else {
            return;
        };

        if !packet.action.is_read_only() {
            test_instance.set(packet.data);
            match packet.action {
                TestInstanceBlockAction::Reset
                | TestInstanceBlockAction::Save
                | TestInstanceBlockAction::Export
                | TestInstanceBlockAction::Run => {
                    // The game-test framework owns these operations; Steel has no
                    // `TestInstanceBlockEntity.resetTest/saveTest/exportTest/runTest`
                    // equivalent yet, so the payload is stored and the action is
                    // reported instead of being silently dropped.
                    log::warn!(
                        "test instance block action {:?} at {:?} is not implemented",
                        packet.action,
                        packet.pos
                    );
                }
                TestInstanceBlockAction::Init
                | TestInstanceBlockAction::Query
                | TestInstanceBlockAction::Set => {}
            }
            // Vanilla `sendBlockUpdated(pos, Blocks.AIR.defaultBlockState(), state, 3)`.
            world.send_block_updated(packet.pos);
            return;
        }

        let status = match packet.data.test {
            // Vanilla renders `GameTestInstance.describe()`; when no data-pack
            // test instance table entry is present, emit the test identifier.
            Some(ref test) => TextComponent::plain(test.to_string()),
            None => TextComponent::translated(TranslatedMessage {
                key: "test_instance.description.no_test".into(),
                fallback: Some("No test".into()),
                args: None,
            }),
        };
        let size = if packet.action == TestInstanceBlockAction::Query {
            test_instance.structure_size()
        } else {
            None
        };
        self.send_packet(CTestInstanceBlockStatus { status, size });
        if matches!(
            packet.action,
            TestInstanceBlockAction::Query | TestInstanceBlockAction::Run
        ) {
            self.send_packet(CGameTestHighlightPos {
                absolute_pos: packet.pos,
                relative_pos: BlockPos::new(0, 0, 0),
            });
        }
    }

    /// Vanilla `ServerGamePacketListenerImpl.handleDebugSubscriptionRequest`
    /// → `ServerPlayer.requestDebugSubscriptions`.
    ///
    /// The set is stored verbatim; [`Self::debug_subscriptions`] applies the
    /// permission filter, exactly like vanilla's `ServerPlayer`.
    pub fn handle_debug_subscription_request(&self, packet: SDebugSubscriptionRequest) {
        *self.requested_debug_subscriptions.lock() = packet.subscriptions;
    }

    /// Vanilla `ServerPlayer.debugSubscriptions`: an operator sees the set it
    /// asked for, everyone else sees nothing.
    #[must_use]
    pub fn debug_subscriptions(&self) -> Vec<i32> {
        if !self.is_operator() {
            return Vec::new();
        }
        self.requested_debug_subscriptions.lock().clone()
    }

    /// Fast allocation-free check for whether this operator subscribed to `subscription_id`.
    #[must_use]
    pub fn has_debug_subscription(&self, subscription_id: i32) -> bool {
        self.is_operator()
            && self
                .requested_debug_subscriptions
                .lock()
                .contains(&subscription_id)
    }

    /// Instructs the client to persist a cookie up to vanilla's 5 KiB cap
    /// (`ClientboundStoreCookiePacket`). Returns `false` if `payload` exceeds 5,120 bytes.
    pub fn store_cookie(&self, key: Identifier, payload: Vec<u8>) -> bool {
        if payload.len() > 5_120 {
            return false;
        }
        self.send_packet(CStoreCookie { key, payload });
        true
    }

    /// Requests a previously stored cookie from the client (`ClientboundCookieRequestPacket`).
    pub fn request_cookie(&self, key: Identifier) {
        self.cookie_state.lock().0.insert(key.clone());
        self.send_packet(CCookieRequest { key });
    }

    /// Vanilla `ServerCommonPacketListenerImpl.handleCookieResponse`: accepts a cookie
    /// reply if and only if the server previously requested that key; disconnects on
    /// unsolicited responses.
    pub fn handle_cookie_response(&self, packet: SCookieResponse) {
        let accepted = {
            let mut state = self.cookie_state.lock();
            if state.0.remove(&packet.key) {
                state.1.insert(
                    packet.key,
                    match packet.payload {
                        Some(payload) => ReceivedCookie::Value(payload),
                        None => ReceivedCookie::Empty,
                    },
                );
                true
            } else {
                false
            }
        };
        if !accepted {
            self.disconnect(translations::MULTIPLAYER_DISCONNECT_UNEXPECTED_QUERY_RESPONSE.msg());
        }
    }

    /// Returns the most recent validated cookie value returned by the client for `key`.
    #[must_use]
    pub fn received_cookie(&self, key: &Identifier) -> Option<ReceivedCookie> {
        self.cookie_state.lock().1.get(key).cloned()
    }

    /// Initiates a Play → Configuration phase switch (`ClientboundStartConfigurationPacket`).
    pub fn start_configuration(&self) {
        self.awaiting_configuration_ack
            .store(true, Ordering::Release);
        self.send_packet(CStartConfiguration {});
    }

    /// Returns `true` if `CStartConfiguration` was sent and not yet acknowledged.
    #[must_use]
    pub fn is_awaiting_configuration(&self) -> bool {
        self.awaiting_configuration_ack.load(Ordering::Acquire)
    }

    /// Handles `ServerboundConfigurationAcknowledgedPacket`.
    pub fn handle_configuration_acknowledged(&self) {
        if !self
            .awaiting_configuration_ack
            .swap(false, Ordering::AcqRel)
        {
            log::debug!(
                "Player {} sent unexpected SConfigurationAcknowledged",
                self.gameprofile.name()
            );
        }
    }

    /// Sends a Play-state ping transaction (`ClientboundPingPacket`).
    pub fn send_ping(&self, id: i32) {
        self.send_packet(CPing { id });
    }

    /// Records the client's `ServerboundPongPacket` transaction id.
    pub fn handle_pong(&self, id: i32) {
        *self.last_pong_id.lock() = Some(id);
    }

    /// Returns the most recent ping transaction id echoed by `SPong`.
    #[must_use]
    pub fn last_pong_id(&self) -> Option<i32> {
        *self.last_pong_id.lock()
    }

    /// Sends custom chat tab-completion entries to this player
    /// (`ClientboundCustomChatCompletionsPacket`).
    pub fn send_custom_chat_completions(
        &self,
        action: CustomChatCompletionAction,
        entries: Vec<String>,
    ) {
        self.send_packet(CCustomChatCompletions { action, entries });
    }

    /// Handles `ServerboundCustomClickActionPacket` (vanilla `MinecraftServer.handleCustomClickAction`).
    pub fn handle_custom_click_action(&self, packet: SCustomClickAction) {
        log::debug!(
            "Player {} triggered custom click action {}",
            self.gameprofile.name(),
            packet.id
        );
        *self.last_custom_click_action.lock() = Some(packet.id);
    }

    /// Returns the identifier of the latest custom click action sent by this player.
    #[must_use]
    pub fn last_custom_click_action(&self) -> Option<Identifier> {
        self.last_custom_click_action.lock().clone()
    }

    /// Vanilla `ServerGamePacketListenerImpl.handleSetJigsawBlock`.
    ///
    /// A game master rewrites every jigsaw field; the block entity keeps them in
    /// NBT so the editor state survives a save/load.
    pub fn handle_set_jigsaw_block(&self, packet: SSetJigsawBlock) {
        if !self.has_game_master_permissions() {
            return;
        }
        let Some(world) = Entity::level(self) else {
            return;
        };
        let Some(block_entity) = world.get_block_entity(packet.pos) else {
            return;
        };
        let Some(jigsaw) = block_entity.downcast_ref::<JigsawBlockEntity>() else {
            return;
        };
        jigsaw.set_name(packet.name);
        jigsaw.set_target(packet.target);
        jigsaw.set_pool(packet.pool);
        jigsaw.set_final_state(packet.final_state);
        jigsaw.set_joint_name(&packet.joint);
        jigsaw.set_placement_priority(packet.placement_priority);
        jigsaw.set_selection_priority(packet.selection_priority);
        block_entity.set_changed();
        // Vanilla `sendBlockUpdated(pos, state, state, 3)`.
        world.send_block_updated(packet.pos);
    }

    /// Vanilla `ServerGamePacketListenerImpl.handleJigsawGenerate`.
    ///
    /// Generating a structure from a jigsaw block needs the runtime structure
    /// manager, which Steel does not have: world generation places bundled
    /// templates instead of generating from placed jigsaw blocks.
    pub fn handle_jigsaw_generate(&self, packet: SJigsawGenerate) {
        if !self.has_game_master_permissions() {
            return;
        }
        let Some(world) = Entity::level(self) else {
            return;
        };
        let Some(block_entity) = world.get_block_entity(packet.pos) else {
            return;
        };
        if block_entity.downcast_ref::<JigsawBlockEntity>().is_none() {
            return;
        }
        log::warn!(
            "jigsaw generation requested at {:?} (levels {}, keep jigsaws {}) but Steel has no runtime structure generator; the request was skipped",
            packet.pos,
            packet.levels,
            packet.keep_jigsaws
        );
    }

    /// Vanilla `ServerGamePacketListenerImpl.handleSetTestBlock`.
    ///
    /// The test blocks only exist for the game-test framework; Steel keeps the
    /// vanilla gate (game master plus a real test block entity) so a rogue
    /// client cannot write into arbitrary block entities.
    pub fn handle_set_test_block(&self, packet: SSetTestBlock) {
        if !self.has_game_master_permissions() {
            return;
        }
        let Some(world) = Entity::level(self) else {
            return;
        };
        let Some(block_entity) = world.get_block_entity(packet.pos) else {
            return;
        };
        let Some(test_block) = block_entity.downcast_ref::<TestBlockEntity>() else {
            return;
        };
        test_block.set_mode(test_mode_from_ordinal(packet.mode));
        test_block.set_message(packet.message);
        test_block.set_changed();
        // Vanilla `sendBlockUpdated(pos, initialState, testBlock.getBlockState(), 3)`.
        world.send_block_updated(packet.pos);
    }

    /// Vanilla `ServerGamePacketListenerImpl.handleTeleportToEntityPacket`.
    ///
    /// Only a spectator may use it, and only to reach an entity that is actually
    /// present. Vanilla searches every level; Steel looks in this player's own
    /// level, which is the only one whose entities can be reached without a
    /// cross-world teleport that would need an asynchronous chunk load.
    pub fn handle_teleport_to_entity(&self, packet: STeleportToEntity) {
        if !self.is_spectator() {
            return;
        }
        let Some(world) = Entity::level(self) else {
            return;
        };
        let Some(target) = world.get_entity_by_uuid(&packet.target) else {
            return;
        };
        let position = target.position();
        let (yaw, pitch) = target.rotation();
        self.teleport(position, yaw, pitch).ok();
    }

    /// Vanilla `ServerGamePacketListenerImpl.handleBlockEntityTagQuery`.
    ///
    /// A game master asks for the block entity's tags; the answer is the NBT
    /// produced by `saveWithoutMetadata` (or a `null` tag when the block entity
    /// is gone), echoed with the request's transaction id.
    pub fn handle_block_entity_tag_query(&self, packet: SBlockEntityTagQuery) {
        if !self.is_operator() {
            return;
        }
        let Some(world) = Entity::level(self) else {
            return;
        };
        let tag = world
            .get_block_entity(packet.pos)
            .map(|block_entity| block_entity.save_without_metadata());
        self.send_packet(CTagQuery {
            transaction_id: packet.transaction_id,
            tag: OptionalNbt(tag),
        });
    }

    /// Vanilla `ServerGamePacketListenerImpl.handleEntityTagQuery`.
    pub fn handle_entity_tag_query(&self, packet: SEntityTagQuery) {
        if !self.is_operator() {
            return;
        }
        let Some(world) = Entity::level(self) else {
            return;
        };
        let tag = world.get_entity_by_id(packet.entity_id).map(|entity| {
            // Vanilla `Entity.saveWithoutMetadata`: the type-specific NBT
            // with the entity `id` but without the world position metadata.
            let mut nbt = entity.nbt_for_data_compare();
            nbt.insert("id", entity.entity_type().key.to_string());
            nbt
        });
        self.send_packet(CTagQuery {
            transaction_id: packet.transaction_id,
            tag: OptionalNbt(tag),
        });
    }

    /// Vanilla `ServerGamePacketListenerImpl.handleLockDifficulty`.
    ///
    /// Only a game master (or the singleplayer owner, which Steel approximates
    /// with the operator check) may lock or unlock the difficulty.
    pub fn handle_lock_difficulty(&self, packet: SLockDifficulty) {
        if !self.is_operator() {
            return;
        }
        let Some(world) = Entity::level(self) else {
            return;
        };
        // Vanilla locks the difficulty for the whole server; Steel keeps one
        // level data per world, so the world the player stands in is the one
        // that changes and broadcasts `CChangeDifficulty`.
        world.set_difficulty_locked(packet.locked);
    }

    /// Vanilla `ServerGamePacketListenerImpl.handleRecipeBookChangeSettingsPacket`:
    /// stores the tab state and echoes it to other listeners, exactly as the
    /// server does for every tab change.
    pub fn handle_recipe_book_change_settings(&self, packet: SRecipeBookChangeSettings) {
        let Some(book_type) = recipe_book::RecipeBookType::from_ordinal(packet.book_type) else {
            return;
        };
        {
            let mut book = self.recipe_book.lock();
            book.set_settings(
                book_type,
                recipe_book::BookTypeSettings {
                    open: packet.open,
                    filtering: packet.filtering,
                },
            );
        }
        let settings = self.recipe_book.lock().settings_packet();
        self.send_packet(settings);
    }

    /// Vanilla `ServerGamePacketListenerImpl.handleRecipeBookSeenRecipePacket`:
    /// clears the "new" highlight badge for the viewed `RecipeDisplayId`.
    pub fn handle_recipe_book_seen_recipe(&self, packet: SRecipeBookSeenRecipe) {
        self.recipe_book
            .lock()
            .remove_highlight_by_display(packet.recipe);
    }

    /// Handles the end of a client tick.
    pub fn handle_client_tick_end(&self) {
        self.movement.lock().finish_client_tick();
    }

    /// Main entry point for dealing damage. Returns `true` if damage was applied.
    ///
    /// `world` is vanilla's explicit `ServerLevel` argument and controls
    /// difficulty scaling and damage gamerules.
    pub fn hurt(&self, world: &World, source: &DamageSource, amount: f32) -> bool {
        if LivingEntity::is_invulnerable_to(self, world, source) {
            return false;
        }

        {
            let abilities = self.abilities.lock();
            if abilities.invulnerable && !source.bypasses_invulnerability() {
                return false;
            }
        }

        // Vanilla `Player.hurtServer` clears the idle timer so mobs stop
        // treating a freshly damaged player as inactive.
        self.set_no_action_time(0);
        // Vanilla also drops both shoulder entities here; Steel has no
        // shoulder-riding entities (parrots) yet, so there is nothing to
        // detach.
        if self.get_health() <= 0.0 {
            return false;
        }

        // Difficulty scaling (vanilla: Player.hurtServer)
        let mut amount = amount;
        if source.scales_with_difficulty() {
            let difficulty = world.level_data.read().data().difficulty;
            match difficulty {
                Difficulty::Peaceful => {
                    amount = 0.0;
                }
                Difficulty::Easy => {
                    amount = (amount / 2.0 + 1.0).min(amount);
                }
                Difficulty::Hard => {
                    amount = amount * 3.0 / 2.0;
                }
                Difficulty::Normal => {}
            }
        }

        if amount == 0.0 {
            return false;
        }

        LivingEntity::hurt_server(self, world, source, amount)
    }

    fn disabled_damage_game_rule(source: &DamageSource) -> Option<GameRuleRef<bool>> {
        if source.is(&vanilla_damage_type_tags::DamageTypeTag::IS_DROWNING) {
            Some(&DROWNING_DAMAGE)
        } else if source.is(&vanilla_damage_type_tags::DamageTypeTag::IS_FALL) {
            Some(&FALL_DAMAGE)
        } else if source.is(&vanilla_damage_type_tags::DamageTypeTag::IS_FIRE) {
            Some(&FIRE_DAMAGE)
        } else if source.is(&vanilla_damage_type_tags::DamageTypeTag::IS_FREEZING) {
            Some(&FREEZE_DAMAGE)
        } else {
            None
        }
    }

    /// Applies vanilla player damage reductions and health loss.
    fn actually_hurt(&self, world: &World, source: &DamageSource, amount: f32) {
        if LivingEntity::is_invulnerable_to(self, world, source) {
            return;
        }

        let damage = LivingEntity::get_damage_after_armor_absorb(self, source, amount);
        let damage = LivingEntity::get_damage_after_magic_absorb(self, source, damage);
        let original_damage = damage;
        let damage = (damage - self.get_absorption_amount()).max(0.0);
        self.set_absorption_amount(self.get_absorption_amount() - (original_damage - damage));

        let absorbed_damage = original_damage - damage;
        if (0.0..f32::MAX).contains(&absorbed_damage) {
            self.award_custom_stat_with_count(
                &vanilla_custom_stats::DAMAGE_ABSORBED,
                (absorbed_damage * 10.0).round() as i32,
            );
        }

        // Vanilla `CombatTracker.recordDamage` keeps the last damage source so
        // the death message and `getLastDamageSource` see it. Steel records into
        // the shared `DamageHistory` instead of a per-player tracker.
        if let Some(server) = self.try_server() {
            let world = self.get_world();
            let clock = world.level_data.read().game_time_handle();
            server.damage_history.record(self.base(), source, &clock);
        }

        // Vanilla `CombatTracker.recordDamage` keeps the combat state that the
        // `PlayerCombatEnter/End` packets mirror on the client. The player has
        // its own tick count, so the bookkeeping uses it instead of the shared
        // entity-tick helper. Vanilla's `shouldEnterCombat` is
        // `source.getEntity() instanceof LivingEntity`.
        {
            let tick_count = self.tick_count();
            self.tick_combat_status(tick_count);
            let enters_combat = source
                .causing_entity()
                .is_some_and(|causer| causer.as_living_entity().is_some());
            if self.living_base().record_combat_damage(
                tick_count,
                enters_combat,
                !self.is_dead_or_dying(),
            ) {
                self.on_enter_combat();
            }
        }

        if damage != 0.0 {
            self.cause_food_exhaustion(source.damage_type.exhaustion);
            self.set_health(self.get_health() - damage);
            if damage < f32::MAX {
                self.award_custom_stat_with_count(
                    &vanilla_custom_stats::DAMAGE_TAKEN,
                    (damage * 10.0).round() as i32,
                );
            }
            self.game_event(&vanilla_game_events::ENTITY_DAMAGE);
        }
    }

    fn get_death_message(&self, source: &DamageSource) -> TextComponent {
        let death_key = format!("death.attack.{}", source.damage_type.message_id);
        match source.damage_type.death_message_type {
            DeathMessageType::IntentionalGameDesign => {
                let mut link = translations::CHAT_SQUARE_BRACKETS
                    .message([TranslatedMessage {
                        key: format!("{death_key}.link").into(),
                        fallback: None,
                        args: None,
                    }
                    .component()])
                    .component();
                link.interactions = INTENTIONAL_GAME_DESIGN_STYLE.clone();

                TranslatedMessage {
                    key: format!("{death_key}.message").into(),
                    fallback: None,
                    args: Some(Box::new([self.display_name(), link])),
                }
                .component()
            }
            DeathMessageType::Default | DeathMessageType::FallVariants => {
                // Vanilla `DamageSource.getLocalizedDeathMessage`: the message
                // gets the victim plus, when a living entity caused the damage,
                // that entity's display name.
                let mut args = vec![TextComponent::plain(self.gameprofile.name().to_owned())];
                if let Some(killer) = source.causing_entity()
                    && killer.as_living_entity().is_some()
                {
                    args.push(killer.display_name());
                }
                TranslatedMessage {
                    key: death_key.into(),
                    fallback: None,
                    args: Some(args.into_boxed_slice()),
                }
                .component()
            }
        }
    }

    /// Vanilla: `ServerPlayer.die()` (does NOT call `super.die()`).
    fn die(&self, source: &DamageSource) {
        if self.is_removed() {
            return;
        }
        if !self.living_base.mark_death_processed() {
            return;
        }

        self.game_event(&vanilla_game_events::ENTITY_DIE);

        self.sync_entity_data();

        // NOTE: Vanilla `ServerPlayer.die()` does NOT set Pose::Dying — only
        // `LivingEntity.die()` does (which ServerPlayer never calls via super).
        // The death screen covers the player model, so the pose is irrelevant.

        let world = self.get_world();

        // Broadcast entity event 3 (death sound) to all nearby players.
        let chunk_pos = *self.last_chunk_pos.lock();
        world.broadcast_to_nearby(
            chunk_pos,
            CEntityEvent {
                entity_id: self.id(),
                event: EntityStatus::Death,
            },
            None,
        );

        let show_death_messages = world.get_game_rule(&SHOW_DEATH_MESSAGES);

        let death_message = self.get_death_message(source);

        self.send_packet(CPlayerCombatKill {
            player_id: self.id(),
            message: if show_death_messages {
                death_message.clone()
            } else {
                TextComponent::const_plain("")
            },
        });

        if show_death_messages && let Some(server) = self.try_server() {
            // Vanilla routes death messages through `PlayerTeam.sendMessage`, so a
            // team's `death_message_visibility` decides who receives them.
            let holder = ScoreHolder::new(self.scoreboard_name());
            let team = server
                .scoreboards
                .get(world.domain())
                .and_then(|scoreboard| scoreboard.holder_team_name(&holder));

            server.broadcast_team_message(&death_message, team.as_deref(), None);
        }

        if !world.get_game_rule(&KEEP_INVENTORY) && self.game_mode() != GameType::Spectator {
            let drops = self.inventory.lock().take_death_drops();
            for item in drops {
                let _ = self.spawn_dropped_item(item, true, false);
            }

            let reward = self.experience.lock().death_xp_reward();
            if reward > 0 {
                ExperienceOrbEntity::award(&world, self.position(), reward);
            }
        }

        // Vanilla `ServerPlayer.die` → `awardStat(Stats.DEATHS)` and the
        // `deathCount` dummy scoreboard objective.
        self.increment_death_count_objective(&world);
        if let Some(killer) = get_kill_credit(self, &world) {
            self.award_stat(&vanilla_stat_types::ENTITY_KILLED_BY, killer.entity_type());
            killer.award_kill_score(self, source);
            create_wither_rose(self, source);
        }

        self.award_custom_stat(&vanilla_custom_stats::DEATHS);
        self.reset_custom_stat(&vanilla_custom_stats::TIME_SINCE_DEATH);
        self.reset_custom_stat(&vanilla_custom_stats::TIME_SINCE_REST);

        self.clear_fire();
        self.set_ticks_frozen(0);
        // Vanilla `ServerPlayer.die` ends with `getCombatTracker().recheckStatus()`
        // so the death always closes out combat before the respawn screen shows.
        self.recheck_combat_status();

        if world.get_game_rule(&IMMEDIATE_RESPAWN) {
            self.respawn();
        }
    }

    /// Returns whether the Player can eat
    pub fn can_eat(&self, can_always_eat: bool) -> bool {
        let invulnerable = { self.abilities.lock().invulnerable };
        let needs_foods = { self.food_data.lock().needs_food() };
        invulnerable || can_always_eat || needs_foods
    }

    /// Cleans up player resources.
    #[expect(clippy::unused_self, reason = "this is an api function")]
    pub const fn cleanup(&self) {}

    /// Returns the world the player is currently in.
    pub fn get_world(&self) -> Arc<World> {
        self.world.load_full()
    }

    /// Returns the server this player belongs to, if still attached.
    #[must_use]
    pub(crate) fn try_server(&self) -> Option<Arc<Server>> {
        self.server.upgrade()
    }

    /// Returns the identity of the player's current continuous domain stay.
    pub(crate) fn domain_residence_token(&self) -> DomainResidenceToken {
        self.residence.lock().token
    }

    /// Starts a new continuous domain stay and invalidates old restore work.
    pub(crate) fn advance_domain_residence(&self) -> DomainResidenceToken {
        self.residence.lock().advance()
    }

    /// Returns whether delayed work still belongs to the current domain stay.
    pub(crate) fn is_domain_residence_current(&self, token: DomainResidenceToken) -> bool {
        self.residence.lock().token == token
    }

    /// Installs both persisted restore payloads for a token-owned domain stay.
    pub(crate) fn install_pending_domain_restores(
        &self,
        token: DomainResidenceToken,
        world: &World,
        root_vehicle: Option<PersistentRootVehicle>,
        ender_pearls: Vec<PersistentEnderPearl>,
    ) -> bool {
        let mut residence = self.residence.lock();
        if residence.token != token {
            return false;
        }

        residence.pending_root_vehicle =
            root_vehicle.map(|root_vehicle| PendingRootVehicleRestore {
                world: world.key.clone(),
                root_vehicle,
            });
        residence.pending_ender_pearls = ender_pearls;
        true
    }

    pub(crate) fn pending_root_vehicle_for_current_world(&self) -> Option<PersistentRootVehicle> {
        let world_key = self.get_world().key.clone();
        self.residence
            .lock()
            .pending_root_vehicle
            .as_ref()
            .filter(|pending| pending.world == world_key)
            .map(|pending| pending.root_vehicle.clone())
    }

    pub(crate) fn take_matching_pending_root_vehicle(
        &self,
        token: DomainResidenceToken,
        world: &World,
        attach: [u8; 16],
        root_uuid: [u8; 16],
    ) -> Option<PersistentRootVehicle> {
        let mut residence = self.residence.lock();
        if residence.token != token {
            return None;
        }
        let matches = residence
            .pending_root_vehicle
            .as_ref()
            .is_some_and(|pending| {
                pending.world == world.key
                    && pending.root_vehicle.attach == attach
                    && pending.root_vehicle.entity.uuid == root_uuid
            });
        if matches {
            residence
                .pending_root_vehicle
                .take()
                .map(|pending| pending.root_vehicle)
        } else {
            None
        }
    }

    pub(crate) fn pending_ender_pearls(&self) -> Vec<PersistentEnderPearl> {
        self.residence.lock().pending_ender_pearls.clone()
    }

    pub(crate) fn remove_pending_ender_pearl(&self, uuid: Uuid) {
        self.residence
            .lock()
            .pending_ender_pearls
            .retain(|pearl| Uuid::from_bytes(pearl.entity.uuid) != uuid);
    }

    pub(crate) fn discard_pending_ender_pearl(
        &self,
        token: DomainResidenceToken,
        uuid: Uuid,
    ) -> bool {
        let mut residence = self.residence.lock();
        if residence.token != token {
            return false;
        }
        let old_len = residence.pending_ender_pearls.len();
        residence
            .pending_ender_pearls
            .retain(|pearl| Uuid::from_bytes(pearl.entity.uuid) != uuid);
        residence.pending_ender_pearls.len() != old_len
    }

    pub(crate) fn take_matching_pending_ender_pearl(
        &self,
        token: DomainResidenceToken,
        world: &World,
        uuid: Uuid,
    ) -> Option<PersistentEnderPearl> {
        let mut residence = self.residence.lock();
        if residence.token != token {
            return None;
        }
        let world_key = world.key.to_string();
        let index = residence.pending_ender_pearls.iter().position(|pearl| {
            pearl.world == world_key && Uuid::from_bytes(pearl.entity.uuid) == uuid
        })?;
        Some(residence.pending_ender_pearls.remove(index))
    }

    /// Registers a thrown ender pearl so it persists with this player and
    /// re-spawns on login (vanilla `ServerPlayer.registerEnderPearl`).
    pub fn register_ender_pearl(&self, pearl: &SharedEntity) {
        let uuid = pearl.uuid();
        let mut pearls = self.ender_pearls.lock();
        pearls.retain(|weak| {
            weak.upgrade()
                .is_some_and(|p| !p.is_removed() && p.uuid() != uuid)
        });
        pearls.push(Arc::downgrade(pearl));
        drop(pearls);
        self.remove_pending_ender_pearl(uuid);
    }

    /// Deregisters a thrown ender pearl once it hits, teleports, or is discarded
    /// (vanilla `ServerPlayer.deregisterEnderPearl`).
    pub fn deregister_ender_pearl(&self, uuid: Uuid) {
        self.ender_pearls
            .lock()
            .retain(|weak| weak.upgrade().is_some_and(|p| p.uuid() != uuid));
    }

    /// Returns this player's live, in-flight ender pearls, pruning dead entries.
    #[must_use]
    pub fn ender_pearls(&self) -> Vec<SharedEntity> {
        let mut pearls = self.ender_pearls.lock();
        pearls.retain(|weak| weak.upgrade().is_some_and(|p| !p.is_removed()));
        pearls.iter().filter_map(Weak::upgrade).collect()
    }

    /// Rebinds live pearls to a fresh respawn incarnation with the same player UUID.
    pub(crate) fn rebind_ender_pearls_to(&self, replacement: &Arc<Self>) {
        debug_assert_eq!(self.gameprofile.id(), replacement.gameprofile.id());
        let replacement_entity: SharedEntity = Arc::<Player>::clone(replacement);
        for pearl in self.ender_pearls() {
            if pearl.projectile_owner_uuid() == Some(self.gameprofile.id()) {
                pearl.restore_owner_reference(&replacement_entity);
            }
        }
    }

    /// Appends vanilla-shaped player state used by command NBT predicates.
    pub(crate) fn save_command_nbt(&self, nbt: &mut NbtCompound) {
        {
            let inventory = self.inventory.lock();
            nbt.insert("Inventory", inventory.to_vanilla_inventory_nbt());
            nbt.insert("SelectedItemSlot", i32::from(inventory.get_selected_slot()));
        }

        {
            let experience = self.experience.lock();
            nbt.insert("XpP", experience.progress());
            nbt.insert("XpLevel", experience.level());
            nbt.insert("XpTotal", experience.total_points());
            nbt.insert("XpSeed", experience.enchantment_seed());
        }
        nbt.insert("Score", self.score());

        {
            let food = self.food_data.lock();
            nbt.insert("foodLevel", food.food_level);
            nbt.insert("foodTickTimer", food.tick_timer);
            nbt.insert("foodSaturationLevel", food.saturation_level);
            nbt.insert("foodExhaustionLevel", food.exhaustion_level);
        }

        {
            let abilities = self.abilities.lock();
            let mut abilities_nbt = NbtCompound::new();
            abilities_nbt.insert(
                "invulnerable",
                NbtTag::Byte(i8::from(abilities.invulnerable)),
            );
            abilities_nbt.insert("flying", NbtTag::Byte(i8::from(abilities.flying)));
            abilities_nbt.insert("mayfly", NbtTag::Byte(i8::from(abilities.may_fly)));
            abilities_nbt.insert("instabuild", NbtTag::Byte(i8::from(abilities.instabuild)));
            abilities_nbt.insert("mayBuild", NbtTag::Byte(i8::from(abilities.may_build)));
            abilities_nbt.insert("flySpeed", abilities.flying_speed);
            abilities_nbt.insert("walkSpeed", abilities.walking_speed);
            nbt.insert("abilities", NbtTag::Compound(abilities_nbt));
        }

        nbt.insert("playerGameType", self.game_mode() as i32);
        if let Some(previous_game_mode) = self.previous_game_mode() {
            nbt.insert("previousPlayerGameType", previous_game_mode as i32);
        }
        nbt.insert(
            "seenCredits",
            NbtTag::Byte(i8::from(self.has_seen_credits())),
        );
        nbt.insert("Dimension", self.get_world().key.to_string());
        if let Some(position) = *self.entered_nether_position.lock() {
            nbt.insert("entered_nether_pos", NbtList::Double(position.to_array().to_vec()));
        }

        if let Some(vehicle) = self.vehicle()
            && let Some(root_vehicle) = self.root_vehicle()
            && root_vehicle.id() != self.id()
            && root_vehicle.has_exactly_one_player_passenger()
            && let Some(entity_nbt) = root_vehicle.nbt_for_passenger_save()
        {
            let mut root_vehicle_nbt = NbtCompound::new();
            root_vehicle_nbt.insert(
                "Attach",
                NbtTag::IntArray(vehicle.uuid().to_int_array().to_vec()),
            );
            root_vehicle_nbt.insert("Entity", NbtTag::Compound(entity_nbt));
            nbt.insert("RootVehicle", NbtTag::Compound(root_vehicle_nbt));
        }

        let ender_pearls = self
            .ender_pearls()
            .into_iter()
            .filter_map(|pearl| {
                let world = pearl.level()?;
                let mut pearl_nbt = pearl.nbt_for_passenger_save()?;
                pearl_nbt.insert("ender_pearl_dimension", world.key.to_string());
                Some(pearl_nbt)
            })
            .collect::<Vec<_>>();
        if !ender_pearls.is_empty() {
            nbt.insert("ender_pearls", NbtList::Compound(ender_pearls));
        }
    }

    /// Marks live ender pearls as stored with this player so chunk saves remove
    /// them from world storage and player data remains the sole owner.
    pub fn store_ender_pearls_with_player(&self) {
        for pearl in self.ender_pearls() {
            let world = pearl.level();
            let chunk = ChunkPos::from_entity_pos(pearl.position());
            pearl.set_removed(RemovalReason::StoredWithPlayer);
            if let Some(world) = world {
                world.mark_chunk_dirty(chunk);
            }
        }
    }

    /// Returns this player's local server tick count.
    #[must_use]
    pub fn tick_count(&self) -> i32 {
        self.tick_state.lock().tick_count()
    }

    #[must_use]
    pub(crate) fn take_xp_delay(&self) -> i32 {
        self.tick_state.lock().take_xp_delay()
    }

    pub(crate) fn set_take_xp_delay(&self, delay: i32) {
        self.tick_state.lock().set_take_xp_delay(delay);
    }

    fn primary_step_sound_block_pos(&self, affecting_pos: BlockPos) -> BlockPos {
        let above_pos = affecting_pos.above();
        let above_state = self.get_world().get_block_state(above_pos);
        let above_block = above_state.get_block();

        if above_block.has_tag(&BlockTag::INSIDE_STEP_SOUND_BLOCKS)
            || above_block.has_tag(&BlockTag::COMBINATION_STEP_SOUND_BLOCKS)
        {
            above_pos
        } else {
            affecting_pos
        }
    }

    /// Resets the last action time of the player (to the current time).
    pub fn reset_last_action_time(&self) {
        *self.last_action_time.lock() = Instant::now();
    }

    fn check_idle_timeout(&self) {
        if let Some(server) = self.server.upgrade() {
            let player_idle_timeout = server.player_idle_timeout.load(Ordering::Relaxed);
            if player_idle_timeout > 0
                && Instant::now().duration_since(*self.last_action_time.lock())
                    > Duration::from_mins(player_idle_timeout as u64)
                && !self.has_won_game()
            {
                self.disconnect(translations::MULTIPLAYER_DISCONNECT_IDLING.msg());
            }
        }
    }
}

impl Entity for Player {
    fn is_hard_collision_relevant(&self) -> bool {
        false
    }

    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        &vanilla_entities::PLAYER
    }

    fn base_tick(&self) {
        LivingEntity::base_tick_living_entity(self);
    }

    fn do_water_splash_effect(&self) {
        if !self.is_spectator() {
            self.default_do_water_splash_effect();
        }
    }

    fn swim_splash_sound(&self) -> SoundEventRef {
        &sound_events::ENTITY_PLAYER_SPLASH
    }

    fn swim_high_speed_splash_sound(&self) -> SoundEventRef {
        &sound_events::ENTITY_PLAYER_SPLASH_HIGH_SPEED
    }

    fn scoreboard_name(&self) -> String {
        self.gameprofile.name().to_owned()
    }

    fn name(&self) -> TextComponent {
        TextComponent::plain(self.gameprofile.name().to_owned())
    }

    fn display_name(&self) -> TextComponent {
        self.name()
            .click_event(ClickEvent::suggest_command(format!(
                "/tell {} ",
                self.gameprofile.name()
            )))
            .hover_event(HoverEvent::show_entity(
                "minecraft:player",
                self.uuid(),
                Some(self.name()),
            ))
            .insertion(self.gameprofile.name().to_owned())
    }

    fn plain_text_name(&self) -> String {
        self.gameprofile.name().to_owned()
    }

    fn force_set_rotation(&self, yaw: f32, relative_yaw: bool, pitch: f32, relative_pitch: bool) {
        self.default_force_set_rotation(yaw, relative_yaw, pitch, relative_pitch);
        // Vanilla `ServerPlayer.forceSetRotation` also pushes the new values so
        // the client does not snap back on its next movement packet.
        self.send_packet(CPlayerRotation {
            yaw,
            relative_yaw,
            pitch,
            relative_pitch,
        });
    }

    fn look_at(&self, from_anchor: EntityAnchor, target: DVec3) {
        apply_entity_look_at(self, from_anchor, target);
        self.send_packet(CPlayerLookAt::position(
            protocol_look_at_anchor(from_anchor),
            target,
        ));
    }

    fn look_at_entity(
        &self,
        from_anchor: EntityAnchor,
        target: &dyn Entity,
        target_anchor: EntityAnchor,
    ) {
        let target_position = target_anchor.position(target);
        apply_entity_look_at(self, from_anchor, target_position);
        self.send_packet(CPlayerLookAt::entity(
            protocol_look_at_anchor(from_anchor),
            target_position,
            target.id(),
            protocol_look_at_anchor(target_anchor),
        ));
    }

    fn is_always_ticking(&self) -> bool {
        true
    }

    fn update_swimming(&self) {
        if self.is_flying() {
            self.set_shared_swimming(false);
        } else {
            self.default_update_swimming();
        }
    }

    fn ride_tick(self: Arc<Self>) {
        let pre = self.position();
        if self.wants_to_stop_riding() && self.is_passenger() {
            self.stop_riding();
        } else {
            Arc::clone(&self).default_ride_tick();
            self.reset_fall_distance();
        }
        self.check_riding_statistics(self.position() - pre);
    }

    fn stop_riding(&self) {
        let old_vehicle = self.vehicle();
        self.base().stop_riding();
        self.base.set_boarding_cooldown(0);
        let Some(old_vehicle) = old_vehicle else {
            return;
        };

        self.remove_active_effects_for_vehicle(old_vehicle.as_ref());
        self.send_packet(CSetPassengers::new(
            old_vehicle.id(),
            Self::passenger_ids_for_packet(old_vehicle.as_ref()),
        ));
    }

    fn teleport_to(&self, pos: DVec3) -> Result<(), EntityMoveError> {
        let (yaw, pitch) = self.rotation();
        self.teleport(pos, yaw, pitch)
    }

    fn start_riding(&self, entity_to_ride: &SharedEntity) -> bool {
        let Some(world) = self.level() else {
            return false;
        };
        let Some(passenger) = world.get_entity_by_id(self.id()) else {
            return false;
        };
        if !start_riding_entities(&passenger, entity_to_ride) {
            return false;
        }

        entity_to_ride.position_rider(self.as_entity_event_source());
        let position = self.position();
        let (yaw, pitch) = self.rotation();
        if let Err(error) = self.teleport(position, yaw, pitch) {
            panic!(
                "failed to synchronize player {} mounted position: {error}",
                self.id()
            );
        }
        self.send_active_effects_for_vehicle(entity_to_ride.as_ref());
        self.send_packet(CSetPassengers::new(
            entity_to_ride.id(),
            Self::passenger_ids_for_packet(entity_to_ride.as_ref()),
        ));
        true
    }

    fn broadcast_to_player(&self, player: &Player) -> bool {
        if player.is_spectator() {
            true
        } else {
            !self.is_spectator()
        }
    }

    fn fall_sounds(&self) -> (SoundEventRef, SoundEventRef) {
        (
            &sound_events::ENTITY_PLAYER_SMALL_FALL,
            &sound_events::ENTITY_PLAYER_BIG_FALL,
        )
    }

    fn is_alive(&self) -> bool {
        !self.is_removed() && self.get_health() > 0.0
    }

    fn blocks_building(&self) -> bool {
        true
    }

    fn is_pickable(&self) -> bool {
        !self.is_spectator() && !self.is_removed()
    }

    fn is_pushable(&self) -> bool {
        self.get_health() > 0.0 && !self.is_spectator() && !self.on_climbable()
    }

    fn on_climbable(&self) -> bool {
        Player::on_climbable(self)
    }

    fn is_spectator(&self) -> bool {
        self.game_mode() == GameType::Spectator
    }

    fn is_flying_player(&self) -> bool {
        self.is_flying()
    }

    fn fire_immune_ticks(&self) -> i32 {
        20
    }

    fn remaining_fire_ticks_cap(&self) -> Option<i32> {
        self.abilities.lock().invulnerable.then_some(1)
    }

    fn get_default_gravity(&self) -> f64 {
        LivingEntity::get_attribute_gravity(self)
    }

    fn fire_ignite_extra_ticks(&self) -> i32 {
        rand::random_range(1..=2)
    }

    fn can_freeze(&self) -> bool {
        if self.is_spectator() {
            return false;
        }

        self.default_living_can_freeze()
    }

    fn make_stuck_in_block(&self, state: BlockStateId, speed_multiplier: DVec3) {
        if !self.is_flying() {
            self.default_make_stuck_in_block(state, speed_multiplier);
        }

        // Vanilla `Player.makeStuckInBlock` clears the impulse context so the
        // following movement ticks are exempt from the stuck-in-block speed cap.
        self.reset_current_impulse_context();
    }

    fn can_be_hit_by_projectile(&self) -> bool {
        self.get_health() > 0.0 && self.is_pickable()
    }

    fn uses_client_movement_packets(&self) -> bool {
        true
    }

    fn can_simulate_movement(&self) -> bool {
        true
    }

    fn is_effective_ai(&self) -> bool {
        true
    }

    fn known_movement(&self) -> DVec3 {
        if let Some(vehicle) = self.vehicle()
            && vehicle
                .controlling_passenger()
                .is_none_or(|controller| controller.id() != self.id())
        {
            return vehicle.known_movement();
        }

        self.movement.lock().last_known_client_movement()
    }

    fn known_speed(&self) -> DVec3 {
        if let Some(vehicle) = self.vehicle()
            && vehicle
                .controlling_passenger()
                .is_none_or(|controller| controller.id() != self.id())
        {
            return vehicle.known_speed();
        }

        self.movement.lock().last_known_client_movement()
    }

    fn is_suppressing_bounce(&self) -> bool {
        self.is_crouching()
    }

    fn cause_fall_damage(
        self: Arc<Self>,
        fall_distance: f64,
        damage_modifier: f32,
        source: &DamageSource,
    ) -> bool {
        if self.abilities.lock().may_fly {
            return false;
        }

        if fall_distance >= 2.0 {
            self.award_custom_stat_with_count(
                &vanilla_custom_stats::FALL_ONE_CM,
                (fall_distance * 100.0).round() as i32,
            );
        }

        LivingEntity::cause_living_fall_damage(
            self.as_ref(),
            fall_distance,
            damage_modifier,
            source,
        )
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
            .unwrap_or(0.6) as f32
    }

    fn backs_off_from_edge(&self) -> bool {
        self.is_crouching() && !self.is_flying()
    }

    fn is_pushed_by_fluid(&self) -> bool {
        !self.is_flying()
    }

    fn is_crouching(&self) -> bool {
        Player::is_crouching(self)
    }

    fn may_interact(&self, world: &World, pos: BlockPos) -> bool {
        world.may_interact(self, pos)
    }

    fn is_swimming(&self) -> bool {
        Player::is_swimming(self)
    }

    fn sound_source(&self) -> SoundSource {
        SoundSource::Players
    }

    /// Matches vanilla `Player.playSound`, which excludes the source player.
    fn play_sound(&self, sound: SoundEventRef, volume: f32, pitch: f32) {
        if let Some(world) = self.level() {
            world.play_sound_at(
                sound,
                self.sound_source(),
                self.position(),
                volume,
                pitch,
                Some(self.id()),
            );
        }
    }

    fn swim_sound(&self) -> SoundEventRef {
        &sound_events::ENTITY_PLAYER_SWIM
    }

    fn play_step_sound(&self, on_pos: BlockPos, on_state: BlockStateId) {
        if self.is_in_water() {
            self.water_swim_sound();
            self.play_muffled_step_sound(on_state);
            return;
        }

        let primary_step_sound_pos = self.primary_step_sound_block_pos(on_pos);
        if primary_step_sound_pos == on_pos {
            self.play_block_step_sound(on_state);
        } else {
            let primary_state = self.get_world().get_block_state(primary_step_sound_pos);
            if primary_state
                .get_block()
                .has_tag(&BlockTag::COMBINATION_STEP_SOUND_BLOCKS)
            {
                self.play_combination_step_sounds(primary_state, on_state);
            } else {
                self.play_block_step_sound(primary_state);
            }
        }
    }

    fn movement_emission(&self) -> EntityMovementEmission {
        if self.is_flying() || self.on_ground() && self.is_discrete() {
            EntityMovementEmission::None
        } else {
            EntityMovementEmission::All
        }
    }

    fn on_below_world(&self) {
        let world = self.get_world();
        self.hurt(
            &world,
            &DamageSource::environment(&vanilla_damage_types::OUT_OF_WORLD),
            4.0,
        );
    }

    fn dimensions_for_pose(&self, pose: EntityPose) -> EntityDimensions {
        let dimensions = Player::dimensions_for_pose(pose);
        if pose == EntityPose::Sleeping || self.entity_type().fixed {
            dimensions
        } else {
            dimensions.scale(LivingEntity::get_scale(self))
        }
    }

    fn hurt(&self, world: &World, source: &DamageSource, amount: f32) -> bool {
        // Delegates to Player's inherent hurt method which handles
        // player-specific prechecks before the shared living hurt path.
        Player::hurt(self, world, source, amount)
    }

    fn killed_entity(
        &self,
        _world: &World,
        entity: &dyn LivingEntity,
        _source: &DamageSource,
    ) -> bool {
        self.award_stat(&vanilla_stat_types::ENTITY_KILLED, entity.entity_type());
        true
    }

    fn award_kill_score(&self, victim: &dyn Entity, _killing_blow: &DamageSource) {
        if self.id() != victim.id() {
            // Vanilla `ServerPlayer.awardKillScore` → `playerKillCount` objective.
            if victim.as_player().is_some() {
                self.increment_player_kill_count_objective();
            }
            self.award_custom_stat(if victim.as_player().is_some() {
                &vanilla_custom_stats::PLAYER_KILLS
            } else {
                &vanilla_custom_stats::MOB_KILLS
            });
            // Vanilla `ServerPlayer.awardKillScore` fires
            // `CriteriaTriggers.PLAYER_KILLED_ENTITY`, which the `kill_a_mob` and
            // `monster_hunter` advancements listen for.
            self.trigger_advancement_with_context(
                &PLAYER_KILLED_ENTITY,
                &TriggerContext::entity(&victim.entity_type().key),
            );
            // Vanilla `ServerPlayer.awardKillScore` fires
            // `CriteriaTriggers.KILL_MOB_NEAR_SCULK_CATALYST` when the victim is
            // a mob killed within eight blocks of a sculk catalyst.
            if victim.as_player().is_none()
                && let Some(world) = self.level()
            {
                let victim_pos = victim.block_position();
                let near_catalyst = BlockPos::between_closed(
                    victim_pos.offset(-8, -8, -8),
                    victim_pos.offset(8, 8, 8),
                )
                .any(|pos| {
                    world.get_block_state(pos).get_block() == &vanilla_blocks::SCULK_CATALYST
                });
                if near_catalyst {
                    self.trigger_advancement_with_context(
                        &KILL_MOB_NEAR_SCULK_CATALYST,
                        &TriggerContext::entity(&victim.entity_type().key),
                    );
                }
            }
            // Vanilla `ServerPlayer.awardKillScore` also bumps `teamKillCount`
            // when killer and victim share a team.
            let world = self.get_world();
            if self.is_on_same_team_as(&world, victim) {
                self.increment_dummy_objective(&world, "teamKillCount");
            }
        }
    }

    /// Vanilla `ServerPlayer.tickCount` is the player's own counter, not the
    /// shared `EntityBase` one; the combat tracker compares damage times against
    /// it, so the trait default must be overridden here.
    ///
    /// Fully qualified on purpose: a bare `self.tick_count()` would resolve back
    /// to this very method and recurse forever.
    fn tick_count(&self) -> i32 {
        Player::tick_count(self)
    }
}

const fn protocol_look_at_anchor(anchor: EntityAnchor) -> LookAtAnchor {
    match anchor {
        EntityAnchor::Feet => LookAtAnchor::Feet,
        EntityAnchor::Eyes => LookAtAnchor::Eyes,
    }
}

impl LivingEntity for Player {
    fn living_synced_data(&self) -> Option<&dyn LivingEntitySyncedData> {
        Some(&self.entity_data)
    }

    /// Vanilla `ServerPlayer.onEnterCombat`: tell the client combat started, so
    /// its combat music and the "in combat" HUD agree with the server.
    fn on_enter_combat(&self) {
        self.send_packet(CPlayerCombatEnter {});
    }

    /// Vanilla `ServerPlayer.onLeaveCombat`: send the duration the tracker
    /// measured, which the client fades the combat state out over.
    fn on_leave_combat(&self, combat_duration: i32) {
        self.send_packet(CPlayerCombatEnd {
            combat_time: combat_duration,
        });
    }

    fn indicate_damage(&self, xd: f64, zd: f64) {
        let (y_rot, _) = self.rotation();
        let hurt_dir = zd.atan2(xd) * RAD_TO_DEG_F64 - f64::from(y_rot);
        self.send_packet(CHurtAnimation {
            entity_id: self.id(),
            yaw: hurt_dir as f32,
        });
    }

    #[expect(clippy::too_many_lines, reason = "splitting would hurt readability")]
    fn tick_living_entity(&self, entity: &SharedEntity) {
        self.advance_tick();
        // Vanilla `ServerPlayer.tick` runs `this.getCombatTracker().recheckStatus()`
        // through the living tick; the player has its own tick count, so the
        // shared hook is fed here.
        self.tick_combat_status(self.tick_count());
        self.tick_riptide();
        self.tick_item_cooldowns();
        self.tick_attack_strength();
        self.tick_throttlers();
        self.check_idle_timeout();
        self.tick_client_load_timeout();
        self.tick_sleep_counter();
        if self.is_sleeping() {
            let world = self.get_world();
            let state = self.sleeping_pos().map(|pos| world.get_block_state(pos));
            if state.is_some_and(|state| {
                !self.bed_rule_value_allows(Self::bed_rule_for(&world, state).can_sleep)
            }) {
                self.stop_sleep_in_bed(false, true);
            } else if !self.can_interact_with_level()
                || self
                    .sleeping_pos()
                    .is_none_or(|pos| !world.get_block_state(pos).is_bed())
            {
                self.stop_sleep_in_bed(true, true);
            }
        }

        self.set_no_physics(self.is_spectator());
        if self.is_spectator() || self.is_passenger() {
            self.set_on_ground(false);
        }

        let tick_position = self.position();

        self.movement.lock().reset_for_tick(tick_position);
        self.set_old_position_to_current();
        self.reset_vehicle_movement_for_tick();

        self.default_tick();
        self.detect_equipment_updates();
        // Vanilla `ServerPlayer.doTick` keeps ticking a dead player's connection,
        // but steering and AI only run for a live entity. Running `ai_step` on a
        // removed player would move a corpse and corrupt its last position.
        if !self.is_removed() {
            self.ai_step(entity);
        }

        // Vanilla snaps the player back to firstGood after ServerPlayer.doTick().
        if let Err(error) = self.try_set_position(tick_position) {
            panic!(
                "failed to restore player {} tick position after ai_step: {error}",
                self.id()
            );
        }
        self.refresh_fluid_contact();

        self.tick_ack_block_changes();

        self.living_base.decrement_invulnerable_time();
        self.tick_mob_effects();
        self.tick_active_item_use(entity);
        // Vanilla `ServerPlayer.tick` ticks the warden spawn tracker every tick:
        // the warning level decays after twelve thousand quiet ticks.
        self.tick_warden_spawn_tracker();
        // Vanilla `Player.tick` runs the stat tick before the death branch, so
        // `play_time` keeps counting while dead.
        self.tick_stats();

        if self.get_health() <= 0.0 {
            self.tick_death();
        } else {
            let world = self.get_world();
            self.block_breaking.lock().tick(self, &world);

            self.update_player_attributes();
            // Vanilla `Player.tick` calls `advancements.tick()`, which re-runs the
            // registered `location` predicates against the player's new position.
            let location_context = {
                let position = self.position();
                let mut context = TriggerContext::at(&world, position);
                context.player_sneaking = Some(self.is_discrete());
                context
            };
            {
                let mut advancements = self.advancements.lock();
                advancements.trigger_location(self, &location_context);
                advancements.trigger_tick(self, &location_context);
                advancements.flush_dirty(self, true);
            }
            // Vanilla `ServerPlayer.tick`: the post-effect push follows the
            // advancement flush and only runs for a live player.
            self.flush_post_effects();
            self.tick_levitation_trigger();
            self.tick_ride_entity_in_lava_trigger();
            self.living_base.refresh_speed_from_attributes();
            self.tick_food_data();

            if self.is_sprinting() && !self.food_data.lock().has_enough_food() {
                self.set_sprinting(false);
            }
        }

        if self.disconnect_if_floating_too_long() {
            return;
        }
        if self.disconnect_if_vehicle_floating_too_long() {
            return;
        }

        self.tick_living_state();

        self.tick_open_menu();
        self.flush_inventory_resync();
        self.broadcast_inventory_changes();
        self.update_pose();

        {
            let health = self.get_health();
            let (food, saturation) = {
                let food_data = self.food_data.lock();
                (food_data.food_level, food_data.saturation_level)
            };

            let saturation_zero = saturation == 0.0;

            let mut sync = self.health_sync.lock();
            if sync.needs_update(health, food, saturation_zero) {
                self.send_packet(CSetHealth {
                    health,
                    food,
                    food_saturation: saturation,
                });
                sync.record_sent(health, food, saturation_zero);
            }
        }

        self.send_experience_packet_if_dirty();

        self.connection.tick();
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

    fn living_base(&self) -> &LivingEntityBase {
        &self.living_base
    }

    fn is_using_item(&self) -> bool {
        self.living_base.is_using_item()
    }

    fn get_luck(&self) -> f32 {
        self.attributes()
            .lock()
            .required_value(vanilla_attributes::LUCK) as f32
    }

    fn can_be_seen_as_enemy(&self) -> bool {
        !self.abilities.lock().invulnerable
            && !self.is_invulnerable()
            && self.can_be_seen_by_anyone()
    }

    fn is_invulnerable_to(&self, world: &World, source: &DamageSource) -> bool {
        if self.default_is_invulnerable_to(source)
            || enchantment_helper::is_immune_to_damage(self, source)
        {
            return true;
        }

        if let Some(rule) = Self::disabled_damage_game_rule(source) {
            return !world.get_game_rule(rule);
        }

        !self.has_client_loaded()
    }

    fn actually_hurt(&self, world: &World, source: &DamageSource, amount: f32) {
        Player::actually_hurt(self, world, source, amount);
    }

    fn hurt_broadcast_chunk(&self) -> ChunkPos {
        *self.last_chunk_pos.lock()
    }

    fn die(&self, source: &DamageSource) {
        Player::die(self, source);
    }

    fn tick_equipment(&self) {
        // skip main hand because its already being ticked through player inventory
        InventoryTickContext::tick_equipment(
            &self.get_world(),
            self,
            EquipmentSlot::ALL
                .into_iter()
                .filter(|slot| *slot != EquipmentSlot::MainHand),
        );
    }

    fn with_equipment_slot(&self, slot: EquipmentSlot, visitor: &mut dyn FnMut(&ItemStack)) {
        let inventory = self.inventory.lock();
        visitor(inventory.get_ref(slot));
    }

    fn with_equipment_slot_mut(
        &self,
        slot: EquipmentSlot,
        visitor: &mut dyn FnMut(&mut ItemStack),
    ) {
        let mut inventory = self.inventory.lock();
        inventory.with_equipment_item_mut(slot, visitor);
    }

    fn interact_living_entity_with_equippable(
        &self,
        player: &Player,
        hand: InteractionHand,
    ) -> InteractionResult {
        let item_stack = {
            let inventory = player.inventory.lock();
            let item_stack = inventory.get_item_in_hand(hand);
            item_stack.clone()
        };
        let Some(equippable) = item_stack.get_equippable() else {
            return InteractionResult::Pass;
        };
        if !equippable.equip_on_interact {
            return InteractionResult::Pass;
        }

        let slot = equippable.slot;
        let can_equip = |stack: &ItemStack| {
            stack.get_equippable().is_some_and(|equippable| {
                equippable.equip_on_interact
                    && equippable.slot == slot
                    && self.is_equippable_in_slot(stack, slot)
            })
        };
        if !can_equip(&item_stack) || !Entity::is_alive(self) {
            return InteractionResult::Pass;
        }

        let source_ref = ContainerRef::from(Arc::clone(&player.inventory));
        let target_ref = ContainerRef::from(Arc::clone(&self.inventory));
        let source_id = source_ref.container_id();
        let target_id = target_ref.container_id();
        let mut guard = ContainerLockGuard::lock_all(&[source_ref, target_ref]);
        let source_slot = match hand {
            InteractionHand::MainHand => EquipmentSlot::MainHand,
            InteractionHand::OffHand => EquipmentSlot::OffHand,
        };

        let equipped = if source_id == target_id {
            let Some(inventory) = guard.get_typed_mut::<PlayerInventory>(source_id) else {
                unreachable!("player inventory container retains its concrete type");
            };
            if !can_equip(inventory.get_item_in_hand(hand)) || !inventory.get_ref(slot).is_empty() {
                return InteractionResult::Pass;
            }

            let equipped = inventory.get_mut(source_slot).split(1);
            if equipped.is_empty() {
                return InteractionResult::Pass;
            }
            let equipped_for_effects = equipped.copy_with_count(1);
            *inventory.get_mut(slot) = equipped;
            equipped_for_effects
        } else {
            let Some((source_inventory, target_inventory)) =
                guard.get_two_typed_mut::<PlayerInventory, PlayerInventory>(source_id, target_id)
            else {
                unreachable!("player inventory containers retain their concrete type");
            };
            if !can_equip(source_inventory.get_item_in_hand(hand))
                || !target_inventory.get_ref(slot).is_empty()
            {
                return InteractionResult::Pass;
            }

            let equipped = source_inventory.get_mut(source_slot).split(1);
            if equipped.is_empty() {
                return InteractionResult::Pass;
            }
            let equipped_for_effects = equipped.copy_with_count(1);
            *target_inventory.get_mut(slot) = equipped;
            equipped_for_effects
        };
        drop(guard);

        player.inventory.lock().set_changed();
        if source_id != target_id {
            self.inventory.lock().set_changed();
        }

        // `on_equip_item` runs the vanilla attribute refresh, the equip sound
        // and the `GameEvent.EQUIP` dispatch.
        self.on_equip_item(slot, &ItemStack::empty(), &equipped);
        InteractionResult::Success
    }

    fn has_infinite_materials(&self) -> bool {
        Player::has_infinite_materials(self)
    }

    fn handle_extra_items_created_on_use(&self, extra: ItemStack) {
        let leftover = self.inventory.lock().add_or_return(extra);
        if !leftover.is_empty() {
            let _ = self.drop_item(leftover, false, Prediction::Predicted);
        }
    }

    fn get_absorption_amount(&self) -> f32 {
        *self.entity_data.lock().player_absorption.get()
    }

    fn set_absorption_amount(&self, amount: f32) {
        let max_absorption = self
            .living_base
            .attributes()
            .lock()
            .required_value(vanilla_attributes::MAX_ABSORPTION) as f32;
        self.entity_data
            .lock()
            .player_absorption
            .set(amount.clamp(0.0, max_absorption));
    }

    fn is_affected_by_fluids(&self) -> bool {
        !self.is_flying()
    }

    fn can_glide(&self) -> bool {
        !self.is_flying() && self.default_can_glide()
    }

    fn is_immobile(&self) -> bool {
        self.default_is_immobile() || self.is_sleeping()
    }

    fn stop_sleeping(&self) {
        self.stop_sleep_in_bed(true, true);
    }

    fn jump_from_ground(&self) {
        self.default_jump_from_ground();
        self.award_custom_stat(&vanilla_custom_stats::JUMP);
        if self.is_sprinting() {
            self.cause_food_exhaustion(food_constants::EXHAUSTION_SPRINT_JUMP);
        } else {
            self.cause_food_exhaustion(food_constants::EXHAUSTION_JUMP);
        }
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        self.tick_regeneration();
        InventoryTickContext::tick_player_inventory(&self.get_world(), self);
        // Vanilla `ServerPlayerEntity.playerTick` sends map updates for every
        // carried map, independently of the per-item color update.
        filled_map::send_map_packets(self);
        if self.is_flying() && !self.is_passenger() {
            self.reset_fall_distance();
        }

        let result = self.default_ai_step(entity);
        self.set_y_head_rot(self.rotation().0);
        // Pickup uses the simulated position; `tick_living_entity` restores the
        // authoritative tick position after this returns.
        if self.get_health() > 0.0 {
            self.touch_nearby_items();
        }
        result
    }

    fn travel(&self, entity: &SharedEntity, input: DVec3) -> Option<MoveResult> {
        if self.is_passenger() {
            return self.default_travel(entity, input);
        }

        if self.is_swimming() {
            let look_angle_y = self.look_angle().y;
            let multiplier = if look_angle_y < -0.2 { 0.085 } else { 0.06 };
            let has_fluid_above = self.level().is_some_and(|world| {
                let position = self.position();
                let pos = BlockPos::containing(position.x, position.y + 0.9, position.z);
                !get_fluid_state(&world, pos).is_empty()
            });
            if look_angle_y <= 0.0 || self.is_jumping() || has_fluid_above {
                let velocity = self.velocity();
                self.set_velocity(
                    velocity + DVec3::new(0.0, (look_angle_y - velocity.y) * multiplier, 0.0),
                );
            }
        }

        if self.is_flying() {
            let original_movement_y = self.velocity().y;
            let result = self.default_travel(entity, input);
            let velocity = self.velocity();
            self.set_velocity(DVec3::new(
                velocity.x,
                original_movement_y * 0.6,
                velocity.z,
            ));
            result
        } else {
            self.default_travel(entity, input)
        }
    }

    fn get_flying_speed(&self) -> f32 {
        if self.is_flying() && !self.is_passenger() {
            let flying_speed = self.abilities.lock().flying_speed;
            if self.is_sprinting() {
                flying_speed * 2.0
            } else {
                flying_speed
            }
        } else if self.is_sprinting() {
            0.025_999_999
        } else {
            0.02
        }
    }

    fn on_equipped_item_broken(&self, item: ItemRef, slot: EquipmentSlot) {
        self.broadcast_entity_event(slot.into());
        self.refresh_equipment_attribute_modifiers(slot);
        self.award_stat(&vanilla_stat_types::ITEM_BROKEN, item);
    }
}

impl TextResolutor for Player {
    /// Vanilla resolves entity and score content against the viewer. The
    /// player is the only entity reachable here, so a selector resolves to its
    /// own display name and a score resolves for its own scoreboard holder.
    fn resolve_content(&self, resolvable: &Resolvable) -> TextComponent {
        match resolvable {
            Resolvable::Entity { .. } => self.display_name(),
            Resolvable::Scoreboard { objective, .. } => {
                let holder = ScoreHolder::new(self.scoreboard_name());
                let world = self.get_world();
                let score = self.try_server().and_then(|server| {
                    let scoreboard = server.scoreboards.get(world.domain())?;
                    let objective = scoreboard.objective(objective)?;
                    scoreboard.score(&holder, &objective)
                });
                TextComponent::plain(score.unwrap_or(0).to_string())
            }
            // NBT content needs a live entity NBT snapshot, which the player
            // has no viewer-scoped copy of.
            Resolvable::NBT { .. } => TextComponent::new(),
        }
    }

    /// Vanilla resolves `custom` content from the client's registered custom
    /// text data; the server has no client-side copy, so nothing resolves.
    fn resolve_custom(&self, _data: &CustomData) -> Option<TextComponent> {
        None
    }

    /// Vanilla `TranslatableContents.resolve` for a server-side view: the
    /// English translation table.
    fn translate(&self, key: &str) -> Option<String> {
        TRANSLATIONS.get(key).map(|value| (*value).to_owned())
    }
}

#[cfg(test)]
mod tests;
