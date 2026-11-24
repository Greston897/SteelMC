//! This module contains the `World` struct, which represents a world.

use std::{
    io, mem,
    path::Path,
    sync::{
        Arc, LazyLock, Weak,
        atomic::{AtomicBool, AtomicI8, Ordering},
    },
    time::Duration,
};

use crate::chunk::chunk_ticket_storage::ChunkTicketStorage;
use crate::chunk::full_chunk::{FullChunkBlockSetResult, FullChunkRef};
use crate::chunk::gameplay_chunk_lookup_cache::GameplayChunkLookupCacheScope;
use crate::chunk::light::{
    LightLayer, LightSectionEmptinessChange, MAX_LIGHT_LEVEL, has_different_light_properties,
};
use crate::chunk::status::ChunkStatus;
use crate::entity::damage::DamageHistory;
use crate::entity::damage::DamageSource;
use crate::entity::entities::EndCrystalEntity;
use crate::entity::entities::EnderDragonEntity;
use crate::entity::entities::mobs::bosses::DragonPhase::HoldingPattern;
use crate::entity::entities::mobs::bosses::ender_dragon::phase_ticks::on_crystal_destroyed;
use crate::entity::natural_spawner::tick_natural_spawning;
use crate::entity::next_entity_id;
use crate::entity::patrol_spawner::PatrolSpawner;
use crate::entity::phantom_spawner::PhantomSpawner;
use crate::poi::OccupationStatus;
use crate::portal::WorldChangeRequest;
use crate::scoreboard::Scoreboard;
use crate::server::Server;
use crate::server::boss_bars::BossBar;
use crate::server::boss_bars::DRAGON_BAR_ID;
use crate::server::boss_bars::dragon_bar_uuid;
use crate::world::game_event::{
    GameEventContext, GameEventDispatcher, GameEventListenerCount, GameEventListenerStorage,
    SharedGameEventListener,
};
use crate::worldgen::feature::FeatureDecorationRunner;
use crate::worldgen::feature::place_end_gateway_in_world;
use crate::worldgen::feature::place_end_podium_in_world;
use crate::worldgen::feature::rebuild_spike_crystal_in_world;
use crate::worldgen::feature::rebuild_spike_in_world;
use crate::{chunk::chunk_map::ChunkMapGameTickTimings, world::weather::Weather};
use steel_registry::feature::EndGatewayConfiguration;
use steel_registry::feature::EndSpike;
use steel_registry::vanilla_entities::END_CRYSTAL;
use steel_registry::vanilla_entities::ENDER_DRAGON;
use steel_registry::vanilla_entities::WANDERING_TRADER;
use steel_utils::saved_data::{SavedDataManager, names as saved_data_names};
use text_components::translation::TranslatedMessage;
use tokio::fs::create_dir_all;
use tokio::fs::write;

use glam::{DVec3, IVec3};
use sha2::{Digest, Sha256};
use steel_protocol::packets::game::{
    BossBarColor, BossBarOverlay, BossEventAction, CBlockDestruction, CBossEvent,
    CChangeDifficulty, CGameEvent, CInitializeBorder, CLevelEvent, CLevelParticles, CPlayerChat,
    CSetBorderCenter, CSetBorderLerpSize, CSetBorderSize, CSetBorderWarningDelay,
    CSetBorderWarningDistance, CSetEntityData, CSetEntityLink, CSetEquipment, CSound, CSoundEntity,
    CSystemChat, CUpdateAttributes, GameEventType, RandomizationType, SoundSource,
};
use steel_protocol::utils::ConnectionProtocol;
use steel_protocol::{
    packet_traits::{ClientPacket, CompressionInfo, EncodedPacket},
    packets::game::CSetTime,
};

use rustc_hash::FxHashSet;
use simdnbt::owned::NbtCompound;
use steel_registry::biome::{BiomeRef, TemperatureModifier};
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{Axis, BlockStateProperties, Direction};
use steel_registry::blocks::shapes::{
    BooleanOp, OffsetVoxelShape, SupportType, VoxelShape, is_offset_face_full, is_shape_full_block,
    join_is_not_empty,
};
use steel_registry::fluid::{FluidRef, FluidState};
use steel_registry::game_events::GameEventRef;
use steel_registry::game_rules::{ErasedGameRuleRef, GameRule, GameRuleValue, GameRuleValueType};
use steel_registry::item_stack::ItemStack;
use steel_registry::level_events;
use steel_registry::loot_table::LootContext;
use steel_registry::particle_type::ParticleData;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::vanilla_game_rules::{
    BLOCK_DROPS, GLOBAL_SOUND_EVENTS, PLAYERS_NETHER_PORTAL_DEFAULT_DELAY, RANDOM_TICK_SPEED,
};
use steel_registry::{REGISTRY, RegistryEntry, RegistryExt, dimension_type::DimensionTypeRef};
use steel_registry::{block_entity_type::BlockEntityTypeRef, vanilla_dimension_types};
use steel_registry::{
    blocks::BlockRef, vanilla_game_rules::ADVANCE_TIME, vanilla_game_rules::ADVANCE_WEATHER,
};
use steel_registry::{vanilla_blocks, vanilla_entities, vanilla_game_events, vanilla_poi_types};
use steel_utils::block_util::FoundRectangle;
use steel_utils::{
    Downcast as _,
    locks::{SyncMutex, SyncRwLock},
    random::{Random as _, RandomSource, legacy_random::LegacyRandom},
};
use steel_worldgen::{biomes::obfuscate_biome_seed, noise::PerlinSimplexNoise};

use steel_utils::{
    BlockLocalAabb, BlockPos, BlockStateId, ChunkPos, Identifier, PackedBlockPos, SectionPos,
    WorldAabb,
    types::{Difficulty, GameType, UpdateFlags},
};
use tokio::{runtime::Runtime, time::Instant};

use crate::{
    ChunkMap,
    behavior::{BLOCK_BEHAVIORS, BlockCollisionContext, BlockLootContext, FLUID_BEHAVIORS},
    block_entity::{
        BlockEntity, SharedBlockEntity,
        entities::{EndGatewayBlockEntity, EndPortalBlockEntity},
    },
    chunk::{
        chunk_ticket_manager::FullChunkStatus, heightmap::HeightmapType,
        player_chunk_view::PlayerChunkView,
    },
    chunk_saver::{ChunkStorage, RamOnlyStorage, RegionManager},
    entity::{
        AddEntityError, Entity, EntityChangeSenders, EntityChunkCallback, EntityLifecycleChanges,
        EntityMovementSyncPacket, EntityOwnership, EntityTracker, EntityVisibility,
        InactiveEntityCallback, MobEffectSyncPacket, RemovalReason, SharedEntity,
        WorldEntityManager,
        entities::{ExperienceOrbEntity, ItemEntity},
        entity_loot_ref,
    },
    fluid::{FluidStateExt as _, fluid_state_to_block},
    level_data::{
        GameTime, GameTimeSource, LevelDataManager, RespawnData, WorldGenerationSettings,
    },
    player::{LastSeen, Player, connection::NetworkConnection},
    poi::PointOfInterestStorage,
};

mod block_entity_ticker;
mod block_event;
mod block_region;
pub(crate) mod block_updates;
mod border;
mod broadcasts;
pub(crate) mod clock;
mod domain_entity_directory;
pub mod dragon_fight;
mod entity_management;
mod environment;
mod events;
pub mod explosion;
/// Vanilla game-event contexts, listeners, and dispatch storage.
pub mod game_event;
mod level_effects;
mod level_reader;
pub mod map_state;
mod player_index;
pub(crate) mod player_spawn_finder;
mod portals;
mod properties;
pub mod raid;
pub(crate) mod raycast;
mod redstone;
mod signal_getter;
mod sleep;
mod sleep_status;
pub(crate) mod spawn;
pub mod tick_scheduler;
pub mod wandering_trader_spawner;
pub(crate) mod waypoint;
mod weather;
mod world_entities;

#[cfg(test)]
mod tests;

pub use crate::config::WorldStorageConfig;
use crate::worldgen::generators::vanilla::fuzzed_biome_at_block;
use crate::worldgen::{ChunkGenerator, ChunkGeneratorType};
use block_event::BlockEventQueue;
pub(crate) use block_region::{BlockRegionBounds, BlockRegionRead, MAX_BLOCK_REGION_WORKSET_SLOTS};
use block_updates::CollectingNeighborUpdater;
pub use border::WorldBorderError;
pub(crate) use border::{MAX_CENTER_COORDINATE, MAX_SIZE};
use border::{WorldBorder, WorldBorderSnapshot};
pub(crate) use domain_entity_directory::DomainEntityDirectory;
use entity_management::NavigatingMobTracker;
#[cfg(test)]
use entity_management::nearest_player_distance_in_range;
pub use explosion::{
    BlockInteraction, DefaultExplosionDamageCalculator, EntityBasedExplosionDamageCalculator,
    Explosion, ExplosionDamageCalculator, ExplosionInteraction, ExplosionOptions, ExplosionOutcome,
};
pub(crate) use explosion::{ExplosionBlockReader, ImmutableExplosionBlockCalculator};
pub use level_reader::{LevelAccessor, LevelReader, ScheduledTickAccess};
pub use player_index::{PlayerAreaMap, PlayerMap};
pub use raycast::{ClipBlockShape, ClipFluid, ClipHitResult, RaytraceAction};
#[cfg(test)]
pub(crate) use signal_getter::get_best_neighbor_signal;
pub use signal_getter::{SignalGetter, SignalQueryContext};
pub(crate) use signal_getter::{get_control_input_signal, get_signal, is_redstone_conductor};
pub use tick_scheduler::ScheduledTick;

#[cfg(test)]
use level_effects::sound_is_within_range;
#[cfg(test)]
use portals::{
    closest_portal_candidate, nether_portal_creation_scan_origin, nether_portal_frame_offset_pos,
};
use std::sync::OnceLock;

const fn initialize_border_packet(snapshot: WorldBorderSnapshot) -> CInitializeBorder {
    CInitializeBorder {
        new_center_x: snapshot.center_x,
        new_center_z: snapshot.center_z,
        old_size: snapshot.old_size,
        new_size: snapshot.new_size,
        lerp_time: snapshot.lerp_time,
        new_absolute_max_size: snapshot.absolute_max_size,
        warning_blocks: snapshot.warning_blocks,
        warning_time: snapshot.warning_time,
    }
}

/// Timing information for a world game tick.
#[derive(Debug)]
pub struct WorldGameTickTimings {
    /// Total time for this world's tick.
    pub elapsed: Duration,
    /// Chunk map game tick timings.
    pub chunk_map: ChunkMapGameTickTimings,
    /// Time spent ticking entities.
    pub entity_tick: Duration,
}

/// Result of replacing a block only when its current state still matches a prior read.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConditionalBlockSetResult {
    /// The expected state was claimed and replaced.
    Changed,
    /// The expected state already equals the requested state, so no callbacks ran.
    Unchanged,
    /// The current state did not match the caller's expected state.
    Stale(BlockStateId),
    /// The position is invalid, the chunk is unavailable, or the update limit was exhausted.
    Unavailable,
}

/// Configuration for creating a new world.
#[derive(Clone)]
pub struct WorldConfig {
    /// Server this world belongs to, filled once during startup.
    pub server: OnceLock<Weak<Server>>,
    /// Server-owned history; the caller retains it while the world runs.
    pub damage_history: Arc<DamageHistory>,
    /// Domain game-time authority, bound during construction.
    pub game_time_source: GameTimeSource,
    /// Storage configuration for chunk persistence.
    pub storage: WorldStorageConfig,
    /// Directory for level data. `None` means level data is ephemeral.
    pub level_data_path: Option<String>,
    /// World generator.
    pub generator: Arc<ChunkGeneratorType>,
    /// Generator metadata persisted for startup compatibility checks.
    pub generation_settings: WorldGenerationSettings,
    /// Server view distance (maximum chunk radius).
    pub view_distance: u8,
    /// Server simulation distance.
    pub simulation_distance: u8,
    /// Maximum queued neighbor-update tasks in one chained run; negative means unlimited.
    pub max_chained_neighbor_updates: i32,
    /// Compression settings for encoding broadcast packets.
    pub compression: Option<CompressionInfo>,
    /// Whether the world should be marked as flat in login/respawn packets.
    pub is_flat: bool,
    /// Sea level sent in login/respawn packets.
    pub sea_level: i32,
    /// Default game mode for first-visit player data.
    pub default_gamemode: GameType,
    /// Difficulty used when creating new level data.
    pub difficulty: Difficulty,
}

/// A struct that represents a world.
pub struct World {
    pub(crate) damage_history: Weak<DamageHistory>,
    /// The chunk map of the world.
    pub chunk_map: Arc<ChunkMap>,
    /// All players in the world with dual indexing by UUID and entity ID.
    pub players: PlayerMap,
    /// Vanilla `WaypointManager` state for the locator bar.
    pub waypoints: waypoint::WaypointManager,
    /// Vanilla `Raid` list for bad/raid omen.
    pub raids: raid::RaidManager,
    /// Vanilla `WanderingTraderSpawner` schedule and the one-trader guard.
    wandering_traders: SyncMutex<wandering_trader_spawner::WanderingTraderSpawner>,
    /// Vanilla `PhantomSpawner` countdown for the next insomnia attempt.
    phantoms: SyncMutex<PhantomSpawner>,
    /// Vanilla `PatrolSpawner` countdown for the next pillager patrol attempt.
    patrols: SyncMutex<PatrolSpawner>,
    /// Vanilla `EnderDragonFight` state for this world (only meaningful in the End).
    pub dragon_fight: dragon_fight::DragonFight,
    /// Spatial index for player proximity queries.
    pub player_area_map: PlayerAreaMap,
    /// Server this world belongs to, filled once during startup.
    ///
    /// A `OnceLock` keeps the world constructible before the server exists while
    /// still letting world-local code reach server-owned systems.
    pub server: OnceLock<Weak<Server>>,
    /// Loaded world identifier (`domain:world`).
    pub key: Identifier,
    /// Vanilla dimension type for this loaded world.
    ///
    /// Vanilla often calls loaded worlds "dimensions". In Steel, `World` is the
    /// loaded world instance and `dimension_type` is the vanilla registry entry
    /// controlling height, skylight, ceiling, water evaporation, etc.
    pub dimension_type: DimensionTypeRef,
    /// Level data manager for persistent world state.
    pub level_data: SyncRwLock<LevelDataManager>,
    pub(crate) game_time: Arc<GameTime>,
    /// Per-world saved data storage.
    pub(crate) saved_data: SavedDataManager,
    /// Vanilla `MapState` saved data: color grids, decorations and next map id.
    pub map_states: SyncMutex<map_state::MapStates>,
    /// Runtime world border state.
    world_border: SyncMutex<WorldBorder>,
    /// Vanilla sleeping player counts for night-skip checks.
    sleep_status: SyncMutex<sleep_status::SleepStatus>,
    /// Server view distance (maximum chunk radius).
    pub view_distance: u8,
    /// Server simulation distance.
    pub simulation_distance: u8,
    /// Compression settings for encoding broadcast packets.
    pub compression: Option<CompressionInfo>,
    /// Whether the world should be marked as flat in login/respawn packets.
    pub is_flat: bool,
    /// Sea level sent in login/respawn packets.
    pub sea_level: i32,
    /// Default game mode for first-visit player data.
    default_gamemode: AtomicI8,
    /// Whether the tick rate is running normally (not frozen/paused).
    /// When false, movement validation checks are skipped.
    tick_runs_normally: AtomicBool,
    /// Whether vanilla's scheduled/chunk/block-event tick phase is active.
    handling_tick: AtomicBool,
    /// Ordered, duplicate-suppressing server block events awaiting execution.
    block_events: SyncMutex<BlockEventQueue>,
    /// Vanilla collecting neighbor updater shared by all live block mutations.
    neighbor_updater: CollectingNeighborUpdater,
    /// Central runtime entity ownership and lookup.
    entity_manager: WorldEntityManager,
    domain_entity_directory: SyncRwLock<Option<Arc<DomainEntityDirectory>>>,
    /// World-global ordered block-entity ticker phase.
    block_entity_tickers: block_entity_ticker::WorldBlockEntityTickers,
    /// Physical entries retained by this world's chunk-owned game-event registries.
    game_event_listener_count: Arc<GameEventListenerCount>,
    /// Entity tracker for managing which players can see which entities.
    entity_tracker: EntityTracker,
    /// Runtime IDs for pathfinder mobs currently visible to the active world.
    navigating_mobs: NavigatingMobTracker,
    /// Weather Data needed for animating starting and stopping of rain clientside
    pub weather: SyncMutex<Weather>,
    /// Per-level recent toggle history used by vanilla redstone-torch burnout.
    redstone_torch_toggles: SyncMutex<redstone::RedstoneTorchToggleTracker>,
    /// World registration and sparse head index for chunk-owned scheduled ticks.
    scheduled_ticks: tick_scheduler::WorldTickScheduler,
    /// Published block batch used by `willTickThisTick` queries during callbacks.
    scheduled_block_ticks_this_tick:
        SyncMutex<Option<Arc<tick_scheduler::ScheduledTickRunBatch<BlockRef>>>>,
    /// Published fluid batch used by `willTickThisTick` queries during callbacks.
    scheduled_fluid_ticks_this_tick:
        SyncMutex<Option<Arc<tick_scheduler::ScheduledTickRunBatch<FluidRef>>>>,
    /// Point of interest storage for efficient spatial queries of special blocks.
    pub poi_storage: SyncMutex<PointOfInterestStorage>,
    /// World-change requests queued by world-local ticks for server safe-point processing.
    pending_world_changes: SyncMutex<Vec<(SharedEntity, WorldChangeRequest)>>,
}

impl World {
    /// Creates a new world with custom configuration.
    ///
    /// This allows specifying storage backend (disk or RAM-only) and other options.
    /// Uses `Arc::new_cyclic` to create a cyclic reference between
    /// the World and its `ChunkMap`'s `WorldGenContext`.
    ///
    /// # Arguments
    /// * `chunk_runtime` - The Tokio runtime for chunk operations
    /// * `dimension_type` - Vanilla dimension type (overworld, nether, end)
    /// * `seed` - The world seed
    /// * `config` - World configuration including storage options
    pub async fn new_with_config(
        chunk_runtime: Arc<Runtime>,
        key: Identifier,
        dimension_type: DimensionTypeRef,
        seed: i64,
        config: WorldConfig,
        generation_pool: Arc<rayon::ThreadPool>,
    ) -> io::Result<Arc<Self>> {
        let chunk_encoding_pool = Arc::clone(&generation_pool);
        Self::new_with_config_and_encoding_pool(
            chunk_runtime,
            key,
            dimension_type,
            seed,
            config,
            generation_pool,
            chunk_encoding_pool,
        )
        .await
    }

    #[expect(
        clippy::too_many_lines,
        reason = "single-pass constructor mirroring the vanilla ServerWorld init order"
    )]
    pub(crate) async fn new_with_config_and_encoding_pool(
        chunk_runtime: Arc<Runtime>,
        key: Identifier,
        dimension_type: DimensionTypeRef,
        seed: i64,
        config: WorldConfig,
        generation_pool: Arc<rayon::ThreadPool>,
        chunk_encoding_pool: Arc<rayon::ThreadPool>,
    ) -> io::Result<Arc<Self>> {
        let view_distance = config.view_distance;
        let simulation_distance = config.simulation_distance;
        let max_chained_neighbor_updates = config.max_chained_neighbor_updates;
        let compression = config.compression;
        let is_flat = config.is_flat;
        let sea_level = config.sea_level;
        let default_gamemode = config.default_gamemode;
        // Create storage backend based on config
        let storage: Arc<ChunkStorage> = match &config.storage {
            WorldStorageConfig::Disk { path } => {
                Arc::new(ChunkStorage::Disk(RegionManager::new(path.clone())))
            }
            WorldStorageConfig::RamOnly => {
                Arc::new(ChunkStorage::RamOnly(RamOnlyStorage::empty_world()))
            }
        };

        // Create or skip level data based on config

        let path = config.level_data_path.as_deref().map(Path::new);
        let saved_data = SavedDataManager::new(path);
        let mut level_data = LevelDataManager::new(
            path,
            seed,
            config.difficulty,
            config.generation_settings,
            config.game_time_source,
        )
        .await?;
        if level_data.is_dirty() {
            level_data.save().await?;
        }
        let dragon_fight_state = {
            let fight = dragon_fight::DragonFight::new();
            fight.set_level_seed(seed);
            // Vanilla persists the fight through `SavedDataType` `ender_dragon_fight`
            // so a restart keeps the portal, gateways and respawn progress.
            match saved_data
                .load_or_default::<dragon_fight::DragonFightSavedData>(
                    saved_data_names::ENDER_DRAGON_FIGHT,
                )
                .await
            {
                Ok(state) => fight.apply_saved_data(&state),
                Err(error) => log::warn!(
                    "Could not load the ender dragon fight for world {key}; starting fresh: {error}"
                ),
            }
            // Vanilla `EnderDragonFight.init`: a fight without a saved gateway
            // order derives one from the level seed. Saved orders are kept as-is,
            // which is what makes the gateway ring stable across restarts.
            if fight.gateways().is_empty() {
                fight.init_gateways(seed);
            }
            fight
        };
        let ticket_storage = ChunkTicketStorage::load(&saved_data, &key).await;
        let raids = match saved_data
            .load_or_default::<raid::RaidSavedData>(saved_data_names::RAIDS)
            .await
        {
            Ok(data) => raid::RaidManager::from_saved_data(data),
            Err(error) => {
                log::warn!("Could not load raids for world {key}; starting empty: {error}");
                raid::RaidManager::new()
            }
        };
        let wandering_traders = match saved_data
            .load_or_default::<wandering_trader_spawner::WanderingTraderSchedule>(
                saved_data_names::WANDERING_TRADERS,
            )
            .await
        {
            Ok(schedule) => {
                wandering_trader_spawner::WanderingTraderSpawner::from_schedule(schedule)
            }
            Err(error) => {
                log::warn!(
                    "Could not load the wandering trader schedule for world {key}; starting a fresh day: {error}"
                );
                wandering_trader_spawner::WanderingTraderSpawner::from_schedule(
                    wandering_trader_spawner::WanderingTraderSchedule::default(),
                )
            }
        };
        let map_states = match saved_data
            .sync_load_wincode::<map_state::PersistentMapStates>(saved_data_names::MAPS)
        {
            Ok(Some(persistent)) => map_state::MapStates::from_persistent(persistent),
            Ok(None) => map_state::MapStates::new(),
            Err(error) => {
                log::warn!("Could not load map data for world {key}; starting empty: {error}");
                map_state::MapStates::new()
            }
        };
        let world_border = WorldBorder::new(level_data.data().world_border)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        // let generator = Arc::new(ChunkGeneratorType::Flat(FlatChunkGenerator::new(
        //     REGISTRY
        //         .blocks
        //         .get_default_state_id(vanilla_blocks::BEDROCK), // Bedrock
        //     REGISTRY.blocks.get_default_state_id(vanilla_blocks::DIRT), // Dirt
        //     REGISTRY
        //         .blocks
        //         .get_default_state_id(vanilla_blocks::GRASS_BLOCK), // Grass Block
        // )));

        let mut weather = Weather::default();
        if level_data.is_raining() {
            weather.rain_level = 1.0;
            if level_data.is_thundering() {
                weather.thunder_level = 1.0;
            }
        }

        Ok(Arc::new_cyclic(|weak_self: &Weak<World>| {
            let chunk_map = Arc::new(ChunkMap::new_with_storage_and_ticket_storage(
                chunk_runtime,
                Weak::clone(weak_self),
                dimension_type,
                sea_level,
                storage,
                config.generator,
                generation_pool,
                chunk_encoding_pool,
                view_distance,
                simulation_distance,
                ticket_storage,
            ));
            chunk_map.start_generation_refill_loop();

            Self {
                damage_history: Arc::downgrade(&config.damage_history),
                chunk_map,
                players: PlayerMap::new(),
                waypoints: waypoint::WaypointManager::new(),
                raids,
                wandering_traders: SyncMutex::new(wandering_traders),
                phantoms: SyncMutex::new(PhantomSpawner::new()),
                patrols: SyncMutex::new(PatrolSpawner::new()),
                dragon_fight: dragon_fight_state,
                player_area_map: PlayerAreaMap::new(),
                server: OnceLock::new(),
                key,
                dimension_type,
                game_time: level_data.game_time_handle(),
                level_data: SyncRwLock::new(level_data),
                saved_data,
                map_states: SyncMutex::new(map_states),
                world_border: SyncMutex::new(world_border),
                sleep_status: SyncMutex::new(sleep_status::SleepStatus::default()),
                view_distance,
                simulation_distance,
                compression,
                is_flat,
                sea_level,
                default_gamemode: AtomicI8::new(default_gamemode.into()),
                tick_runs_normally: AtomicBool::new(true),
                handling_tick: AtomicBool::new(false),
                block_events: SyncMutex::new(BlockEventQueue::default()),
                neighbor_updater: CollectingNeighborUpdater::new(max_chained_neighbor_updates),
                entity_manager: WorldEntityManager::new(),
                domain_entity_directory: SyncRwLock::new(None),
                block_entity_tickers: block_entity_ticker::WorldBlockEntityTickers::new(),
                game_event_listener_count: GameEventListenerCount::shared(),
                entity_tracker: EntityTracker::new(),
                navigating_mobs: NavigatingMobTracker::new(),
                weather: SyncMutex::new(weather),
                redstone_torch_toggles: SyncMutex::new(
                    redstone::RedstoneTorchToggleTracker::default(),
                ),
                scheduled_ticks: tick_scheduler::WorldTickScheduler::new(),
                scheduled_block_ticks_this_tick: SyncMutex::new(None),
                scheduled_fluid_ticks_this_tick: SyncMutex::new(None),
                poi_storage: SyncMutex::new(PointOfInterestStorage::new()),
                pending_world_changes: SyncMutex::new(Vec::new()),
            }
        }))
    }

    /// Vanilla `ServerLevel.save`: writes `level.dat`, the saved-data sets, and
    /// finally every dirty chunk.
    ///
    /// `/save-all` and the autosave use this same path — vanilla
    /// `MinecraftServer.saveEverything` iterates the levels and then saves the
    /// command-owned data, so a periodic save must not skip the chunks.
    ///
    /// A failing saved-data set is logged but does not stop the pass; only the
    /// chunk write is returned, because that is what vanilla reports.
    pub async fn save(&self) -> io::Result<usize> {
        self.sync_world_border_to_level_data();
        // Finish the serialization while the guard is held, then release it
        // before awaiting the write: a `parking_lot` guard is not `Send`, so
        // holding it across `.await` would make this future non-`Send` and break
        // every `tokio::spawn` that saves a world.
        let save_result = self.level_data.write().prepare_save();
        let prepared = match save_result {
            Ok(prepared) => prepared,
            Err(e) => {
                log::error!("Failed to serialize world level data: {e}");
                None
            }
        };
        if let Some((path, content)) = prepared {
            let written = async {
                if let Some(parent) = path.parent() {
                    create_dir_all(parent).await?;
                }
                write(&path, content).await
            }
            .await;
            match written {
                Ok(()) => log::info!("World {} level data saved successfully", self.key),
                Err(e) => log::error!("Failed to save world level data: {e}"),
            }
        }

        let chunk_tickets = self.chunk_map.persistent_chunk_tickets();
        match self
            .saved_data
            .save(saved_data_names::CHUNK_TICKETS, &chunk_tickets)
            .await
        {
            Ok(()) => log::info!("World {} saved chunk ticket data successfully", self.key),
            Err(e) => log::error!("Failed to save world chunk ticket data: {e}"),
        }

        {
            let map_states = self.map_states.lock();
            let has_changes = map_states.is_dirty();
            let persistent = has_changes.then(|| map_states.to_persistent());
            drop(map_states);
            if let Some(persistent) = persistent {
                match self
                    .saved_data
                    .sync_save_wincode(saved_data_names::MAPS, &persistent)
                {
                    Ok(()) => {
                        self.map_states.lock().clear_dirty();
                        log::info!("World {} saved map data successfully", self.key);
                    }
                    Err(e) => log::error!("Failed to save world map data: {e}"),
                }
            }
        }

        // Active raids must survive a restart, otherwise a raid in progress is
        // silently forgotten when the server stops.
        let raids = self.raids.to_saved_data();
        match self.saved_data.save(saved_data_names::RAIDS, &raids).await {
            Ok(()) => log::info!("World {} saved {} raid(s)", self.key, raids.raids.len()),
            Err(e) => log::error!("Failed to save world raids: {e}"),
        }

        // The dragon fight is vanilla saved data: keep the portal, the opened
        // gateways and any half-finished respawn across a restart.
        if self.is_end_dimension_type() {
            let state = self.dragon_fight.to_saved_data();
            if let Err(error) = self
                .saved_data
                .save(saved_data_names::ENDER_DRAGON_FIGHT, &state)
                .await
            {
                log::error!("Failed to save the ender dragon fight: {error}");
            }
        }

        // The wandering trader day counter is vanilla `level.dat` state: a
        // restart must continue the countdown instead of restarting the day.
        let schedule = self.wandering_traders.lock().schedule();
        if let Err(error) = self
            .saved_data
            .save(saved_data_names::WANDERING_TRADERS, &schedule)
            .await
        {
            log::error!("Failed to save the wandering trader schedule: {error}");
        }

        self.save_all_chunks().await
    }

    /// Shutdown hook: saves the world and adds the chunk count to the tally.
    pub async fn cleanup(&self, total_saved: &mut usize) {
        match self.save().await {
            Ok(count) => *total_saved += count,
            Err(e) => log::error!("Failed to save world chunks: {e}"),
        }
    }

    /// Attaches the owning server so world-scoped gameplay can reach
    /// server-owned state such as scoreboards.
    ///
    /// Worlds are constructed before the server (they are fields of it), so the
    /// link is filled once during startup; callers must tolerate `None` in tests.
    pub fn attach_server(&self, server: &Arc<Server>) {
        let _ = self.server.set(Arc::downgrade(server));
    }

    /// Returns the owning server, if this world has been attached to one.
    #[must_use]
    pub fn server(&self) -> Option<Arc<Server>> {
        self.server.get().and_then(Weak::upgrade)
    }

    /// Returns the scoreboard for this world's domain, if a server is attached.
    ///
    /// The scoreboard is built once from all worlds, so it borrows from the
    /// server; callers clone whatever slice of state they need.
    #[must_use]
    pub fn with_scoreboard<R>(&self, f: impl FnOnce(&Scoreboard) -> R) -> Option<R> {
        let server = self.server()?;
        server.scoreboards.get(self.domain()).map(f)
    }

    /// Returns the block position crystals should beam at, if a dragon is alive.
    ///
    /// Vanilla only links crystals to the dragon in the End; other dimensions have
    /// no dragon fight, so this yields `None` there.
    #[must_use]
    pub fn dragon_fight_target(&self) -> Option<BlockPos> {
        if !self.is_end_dimension_type() {
            return None;
        }
        self.dragon_fight
            .dragon_target()
            .map(|pos| BlockPos::new(pos.x, pos.y, pos.z))
    }

    /// Vanilla `EndCrystal` removal hook (`EnderDragonFight.onCrystalDestroyed`).
    pub fn on_end_crystal_destroyed(self: &Arc<Self>, pos: BlockPos) {
        self.on_end_crystal_destroyed_by(pos, None);
    }

    /// Vanilla `EnderDragonFight.onCrystalDestroyed(crystal, source)`.
    ///
    /// While a respawn is running any crystal loss aborts it; otherwise the alive
    /// count is refreshed, the live dragon is told (which is what turns a
    /// holding-pattern dragon into a strafe at whoever broke the crystal), and the
    /// spawn ring is rebuilt on the next fight tick.
    pub fn on_end_crystal_destroyed_by(
        self: &Arc<Self>,
        _pos: BlockPos,
        source: Option<&DamageSource>,
    ) {
        if !self.is_end_dimension_type() {
            return;
        }

        // Vanilla `onCrystalDestroyed`: a crystal destroyed *during* a respawn
        // aborts the sequence; otherwise the count is refreshed and the dragon is
        // told, so it can start a strafe at whoever broke it.
        if self.dragon_fight.respawn_stage().is_some() {
            self.abort_dragon_respawn();
            return;
        }
        self.dragon_fight.on_crystal_destroyed();

        // The dragon handle is bound before the downcast so the borrow lives on the
        // `Arc` instead of the closure parameter.
        let dragon_entity = self
            .dragon_fight
            .dragon_uuid()
            .and_then(|uuid| self.get_entity_by_uuid(&uuid));
        if let Some(entity) = dragon_entity
            && let Some(dragon) = entity.as_ref().downcast_ref::<EnderDragonEntity>()
        {
            let player = source.and_then(DamageSource::causing_entity).cloned();
            on_crystal_destroyed(dragon, player.as_ref());
        }
    }

    /// Vanilla `EnderDragonFight.tryRespawn`, driven by a player placing an end
    /// crystal on the exit portal's four sides.
    ///
    /// Vanilla looks for crystals in the four boxes at `centre.relative(dir, 3)`
    /// around the block above the portal and requires every direction to find at
    /// least one; only then does `respawnDragon` run.
    pub fn try_respawn_dragon(self: &Arc<Self>) {
        if !self.is_end_dimension_type()
            || !self.dragon_fight.is_dragon_killed()
            || self.dragon_fight.respawn_stage().is_some()
        {
            return;
        }

        let location = if let Some(location) = self.dragon_fight.exit_portal_location() {
            BlockPos::new(location.x, location.y, location.z)
        } else {
            // Vanilla makes one if the world does not have a portal yet.
            self.spawn_end_exit_portal(true);
            self.exit_portal_location()
        };
        let center = location.above();

        let mut crystals = Vec::with_capacity(4);
        for direction in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let anchor = BlockPos::new(
                center.x() + direction.0 * 3,
                center.y(),
                center.z() + direction.1 * 3,
            );
            let box_ = steel_utils::WorldAabb::new(
                f64::from(anchor.x()),
                f64::from(anchor.y()),
                f64::from(anchor.z()),
                f64::from(anchor.x() + 1),
                f64::from(anchor.y() + 1),
                f64::from(anchor.z() + 1),
            );
            let found = self.get_entities_in_aabb_matching(&box_, |entity| {
                entity.entity_type() == &END_CRYSTAL
            });
            if found.is_empty() {
                return;
            }
            crystals.extend(found);
        }

        let positions = crystals
            .iter()
            .map(|crystal| {
                let position = Entity::position(crystal.as_ref());
                IVec3::new(
                    position.x.floor() as i32,
                    position.y.floor() as i32,
                    position.z.floor() as i32,
                )
            })
            .collect();

        if self.dragon_fight.try_respawn(positions) {
            // `respawnDragon` replaces the active portal with the inactive one.
            self.spawn_end_exit_portal(false);
        }
    }

    /// Vanilla `EnderDragonFight.abortRespawnSequence`: stop the sequence, reset
    /// the spike crystals and put the active portal back.
    fn abort_dragon_respawn(self: &Arc<Self>) {
        self.dragon_fight.abort_respawn();
        self.dragon_fight.set_respawn_crystals(Vec::new());
        self.reset_end_spike_crystals();
        self.spawn_end_exit_portal(true);
    }

    /// Vanilla `EnderDragonFight.tick`, once per world tick in the End.
    ///
    /// The order is exactly vanilla's: refresh the bar visibility, rescan the
    /// watching players every twenty ticks, and only do any arena work when at least
    /// one player is in range. Then the optional state scan, the respawn stage and
    /// the live-dragon upkeep.
    pub fn tick_dragon_fight(self: &Arc<Self>) {
        if !self.is_end_dimension_type() {
            return;
        }

        // `dragonEvent.setVisible(!dragonKilled)` runs on every tick, even with no
        // players around, because the bar definition is broadcast separately.
        self.dragon_fight
            .set_bar_visible(!self.dragon_fight.is_dragon_killed());

        if self.dragon_fight.tick_player_scan() {
            self.update_dragon_fight_players();
        }

        if !self.dragon_fight.has_nearby_players() {
            // Vanilla drops the arena ticket once nobody is watching, so the arena
            // can unload again.
            self.chunk_map.remove_dragon_ticket();
            return;
        }

        // Vanilla keeps the `-8..=8` arena resident while the fight has players.
        self.chunk_map.add_dragon_ticket();

        // Vanilla bails out of the whole tick body until the arena is loaded.
        if !self.is_arena_loaded() {
            return;
        }

        // Vanilla resolves the world's dragon state once, before anything else, so
        // a world created before the fight existed still gets its portal and the
        // right "previously killed" flag.
        if self.dragon_fight.needs_state_scanning() {
            self.scan_dragon_fight_state();
        }

        if self.dragon_fight.respawn_stage().is_some() {
            // `respawnStage.tick` is skipped when every respawn crystal is gone;
            // that case aborts the sequence instead.
            if self.respawn_crystals().is_empty() {
                self.dragon_fight.abort_respawn();
                return;
            }
            let effects = self.dragon_fight.tick_respawn();
            for effect in effects {
                match effect {
                    dragon_fight::DragonRespawnEffect::AimCrystalsAt(target) => {
                        self.aim_respawn_crystals(target);
                    }
                    dragon_fight::DragonRespawnEffect::LevelEvent { event, pos } => {
                        self.level_event(event, pos, 0, None);
                    }
                    dragon_fight::DragonRespawnEffect::RebuildSpike(spike) => {
                        self.rebuild_end_spike(&spike);
                    }
                    dragon_fight::DragonRespawnEffect::ResetSpikeCrystals => {
                        self.reset_end_spike_crystals();
                    }
                    dragon_fight::DragonRespawnEffect::DetonateRespawnCrystals => {
                        self.detonate_respawn_crystals();
                    }
                    dragon_fight::DragonRespawnEffect::RespawnDragon => {
                        self.spawn_end_dragon();
                    }
                }
            }
        }

        if !self.dragon_fight.is_dragon_killed() {
            self.ensure_end_dragon();
            self.update_dragon_fight_crystals();
        }
    }

    /// Vanilla `EnderDragonFight.updatePlayers`: the bar's player set is every
    /// living player within 192 blocks of `(origin.x, 128 + origin.y, origin.z)`.
    fn update_dragon_fight_players(self: &Arc<Self>) {
        let origin = dragon_fight::DragonFight::podium_pos();
        let center = DVec3::new(
            f64::from(origin.x()),
            f64::from(dragon_fight::DRAGON_SPAWN_Y),
            f64::from(origin.z()),
        );
        let radius_squared = dragon_fight::PLAYER_SCAN_RADIUS * dragon_fight::PLAYER_SCAN_RADIUS;
        let mut has_players = false;
        self.players.iter_players(|_, player| {
            if player.is_alive()
                && Entity::position(player.as_ref()).distance_squared(center) <= radius_squared
            {
                has_players = true;
                return false;
            }
            true
        });
        self.dragon_fight.set_has_nearby_players(has_players);
    }

    /// Vanilla `EnderDragonFight.isArenaLoaded`: every chunk in the `-8..=8`
    /// square around the fight origin is `FULL` and at least `BLOCK_TICKING`.
    ///
    /// The fight skips its whole tick body until the arena is loaded, so a scan or
    /// a dragon spawn never runs against half-generated chunks.
    #[must_use]
    fn is_arena_loaded(&self) -> bool {
        for chunk_x in -8..=8_i32 {
            for chunk_z in -8..=8_i32 {
                let Some(status) = self
                    .chunk_map
                    .full_chunk_status(ChunkPos::new(chunk_x, chunk_z))
                else {
                    return false;
                };
                if !status.is_or_after(FullChunkStatus::BlockTicking) {
                    return false;
                }
            }
        }
        true
    }

    /// Vanilla `EnderDragonFight.scanState`: resolve whether this world has
    /// already been beaten and whether a dragon is still alive.
    ///
    /// Vanilla checks for a `TheEndPortalBlockEntity` anywhere in the `-8..=8`
    /// chunk square, which for the End means the exit portal itself.
    fn scan_dragon_fight_state(self: &Arc<Self>) {
        // Vanilla runs `hasActiveExitPortal()`, then only when it is `false`
        // `findExitPortal()`. Both are the same chunk-entity sweep, so Steel runs
        // it once.
        let has_active_portal = self.has_active_exit_portal();

        // Vanilla: an active portal means the dragon was already beaten. Without
        // one, the world is unbeaten and gets an inactive exit portal.
        if !has_active_portal {
            self.spawn_end_exit_portal(false);
        }

        // Vanilla `getDragons()` = `getEntities(ENDER_DRAGON, LivingEntity::isAlive)`,
        // so a dragon that is already dying does not count as still alive.
        let dragons = self.get_entities_in_aabb_matching(&self.arena_box(), |entity| {
            entity.entity_type() == &ENDER_DRAGON && Entity::is_alive(entity)
        });

        // Vanilla keeps a live dragon only when it also has a portal; otherwise
        // that dragon is a leftover of an interrupted fight and is discarded so a
        // fresh one can spawn.
        let mut dragon = dragons.first().map(|entity| Entity::uuid(entity.as_ref()));
        if let Some(first) = dragons.first()
            && !has_active_portal
        {
            first.set_removed(RemovalReason::Discarded);
            dragon = None;
        }

        self.dragon_fight.scan_state(has_active_portal, dragon);
    }

    /// Vanilla `EnderDragonFight.tick`'s dragon half:
    /// `if (dragonUUID == null || ++ticksSinceDragonSeen >= 1200) findOrCreateDragon()`.
    ///
    /// The whole-level dragon query only runs on those ticks, so the common case is
    /// two lock reads. `findOrCreateDragon` adopts a dragon already in the arena and
    /// otherwise restores the recorded UUID, or creates a new dragon.
    fn ensure_end_dragon(self: &Arc<Self>) {
        let known_uuid = self.dragon_fight.dragon_uuid();
        if let Some(uuid) = known_uuid {
            let seen = self.dragon_fight.tick_dragon_seen();
            if seen < dragon_fight::MAX_TICKS_BEFORE_DRAGON_RESPAWN {
                // Vanilla only reaches `findOrCreateDragon` when the timer expires;
                // until then the recorded UUID is authoritative, and a vanished
                // dragon is recovered on the 1200-tick boundary.
                let _ = uuid;
                return;
            }
        }

        // Vanilla `getDragons()` filters on `LivingEntity::isAlive`, so a dying
        // dragon is not adopted.
        let dragons = self.get_entities_in_aabb_matching(&self.arena_box(), |entity| {
            entity.entity_type() == &ENDER_DRAGON && Entity::is_alive(entity)
        });

        // `findOrCreateDragon`: adopt whatever dragon the level has.
        if let Some(dragon) = dragons.first() {
            self.dragon_fight
                .set_dragon_uuid(Some(Entity::uuid(dragon.as_ref())));
            let position = Entity::position(dragon.as_ref());
            self.dragon_fight.set_dragon(Some(IVec3::new(
                position.x.floor() as i32,
                position.y.floor() as i32,
                position.z.floor() as i32,
            )));
            self.dragon_fight.reset_ticks_since_dragon_seen();
            return;
        }

        self.spawn_end_dragon();
    }

    /// The box `EnderDragonFight.getDragons()` covers.
    ///
    /// Vanilla `ServerLevel.getDragons()` is a whole-level
    /// `getEntities(EntityTypes.ENDER_DRAGON, LivingEntity::isAlive)` — not an AABB
    /// query — so the box spans the world's full height and a generous radius. The
    /// fight only cares about the dragons it can adopt, and the End is a single
    /// arena, so the radius covers everything a dragon can reach.
    #[must_use]
    fn arena_box(&self) -> steel_utils::WorldAabb {
        let min_y = f64::from(self.get_min_y());
        let max_y = f64::from(self.get_max_y() + 1);
        steel_utils::WorldAabb::new(
            -30_000_000.0,
            min_y,
            -30_000_000.0,
            30_000_000.0,
            max_y,
            30_000_000.0,
        )
    }

    /// Vanilla `EndCrystal.setBeamTarget` for every respawn crystal.
    fn aim_respawn_crystals(self: &Arc<Self>, target: BlockPos) {
        for crystal in self.respawn_crystals() {
            Self::with_crystal(&crystal, |crystal| crystal.set_beam_target(Some(target)));
        }
    }

    /// Vanilla `DragonRespawnStage`'s `respawnCrystals` list, resolved against the
    /// live entities.
    ///
    /// Vanilla stores `EntityReference<EndCrystal>`s and re-resolves them every tick
    /// (`e.getEntity(level, EndCrystal.class)`), dropping the ones that died; a
    /// sequence with no crystals left aborts. Steel looks the recorded positions up
    /// in the same way.
    #[must_use]
    fn respawn_crystals(self: &Arc<Self>) -> Vec<SharedEntity> {
        let positions = self.dragon_fight.respawn_crystals();
        if positions.is_empty() {
            return Vec::new();
        }

        let mut found = Vec::with_capacity(positions.len());
        for position in positions {
            let search = steel_utils::WorldAabb::new(
                f64::from(position.x) - 0.5,
                f64::from(position.y) - 0.5,
                f64::from(position.z) - 0.5,
                f64::from(position.x) + 1.5,
                f64::from(position.y) + 1.5,
                f64::from(position.z) + 1.5,
            );
            found.extend(self.get_entities_in_aabb_matching(&search, |entity| {
                entity.entity_type() == &END_CRYSTAL
            }));
        }
        found
    }

    /// Calls `f` on the crystal behind `entity`, if it is one.
    fn with_crystal<R>(entity: &SharedEntity, f: impl FnOnce(&EndCrystalEntity) -> R) -> Option<R> {
        let crystal = entity.as_ref().downcast_ref::<EndCrystalEntity>()?;
        Some(f(crystal))
    }

    /// Vanilla `EnderDragonFight.resetSpikeCrystals`: rebuild every spike's
    /// crystal.
    fn reset_end_spike_crystals(self: &Arc<Self>) {
        let spikes = FeatureDecorationRunner::end_spikes_for_level(self.seed());
        for spike in spikes {
            rebuild_spike_crystal_in_world(self, &spike);
        }
    }

    /// Vanilla `DragonRespawnStage.SUMMONING_PILLARS`'s per-spike rebuild.
    fn rebuild_end_spike(self: &Arc<Self>, spike: &EndSpike) {
        rebuild_spike_in_world(self, spike);
        rebuild_spike_crystal_in_world(self, spike);
    }

    /// Vanilla `DragonRespawnStage.SUMMONING_DRAGON`'s crystal detonation.
    fn detonate_respawn_crystals(self: &Arc<Self>) {
        for crystal in self.respawn_crystals() {
            Self::with_crystal(&crystal, |crystal| crystal.set_beam_target(None));
            let position = Entity::position(crystal.as_ref());
            let mut options = ExplosionOptions::new(position, 6.0, ExplosionInteraction::None);
            options.source = None;
            self.explode(options);
            crystal.set_removed(RemovalReason::Killed);
        }
    }

    /// Vanilla `EnderDragonFight.createNewDragon`.
    ///
    /// The dragon spawns at `(origin.x, 128 + origin.y, origin.z)` with yaw `0`, and
    /// the fight adopts it immediately so crystals relink their beams the same tick.
    fn spawn_end_dragon(self: &Arc<Self>) {
        // Vanilla `createNewDragon` loads the spawn chunk first, so the dragon never
        // appears inside ungenerated terrain.
        self.chunk_map.force_load(ChunkPos::new(0, 0));

        let position = DVec3::new(0.0, f64::from(dragon_fight::DRAGON_SPAWN_Y), 0.0);
        let dragon = Arc::new(EnderDragonEntity::new(
            &ENDER_DRAGON,
            next_entity_id(),
            position,
            Arc::downgrade(self),
        ));

        // Vanilla `createNewDragon`: start in the holding pattern and snap to the
        // spawn point with a random yaw, so two dragons never fly in lockstep.
        dragon.set_phase(HoldingPattern);
        let yaw = rand::random::<f32>() * 360.0;
        Entity::snap_to(dragon.as_ref(), position, yaw, 0.0);

        let dragon = dragon as SharedEntity;
        self.dragon_fight
            .set_dragon_uuid(Some(Entity::uuid(dragon.as_ref())));
        self.dragon_fight
            .set_dragon(Some(IVec3::new(0, dragon_fight::DRAGON_SPAWN_Y, 0)));
        self.dragon_fight.reset_ticks_since_dragon_seen();
        if let Err(error) = self.try_add_entity(dragon) {
            log::debug!("world rejected the ender dragon spawn: {error}");
        }
    }

    /// Vanilla `EnderDragonFight.updateCrystalCount`: count the crystals standing
    /// on each spike's top box.
    ///
    /// Vanilla sums `getEntitiesOfClass(EndCrystal, spike.getTopBoundingBox())`
    /// over the ten spikes, which is why a crystal placed anywhere else in the
    /// arena does not count. Driven from the fight tick.
    pub fn update_dragon_fight_crystals(self: &Arc<Self>) {
        if !self.is_end_dimension_type() || !self.dragon_fight.should_rescan_crystals() {
            return;
        }

        let spikes = FeatureDecorationRunner::end_spikes_for_level(self.seed());
        let mut alive = 0;
        for spike in spikes {
            let radius = f64::from(spike.radius);
            let search = steel_utils::WorldAabb::new(
                f64::from(spike.center_x) - radius,
                f64::from(self.get_min_y()),
                f64::from(spike.center_z) - radius,
                f64::from(spike.center_x) + radius,
                f64::from(self.get_max_y() + 1),
                f64::from(spike.center_z) + radius,
            );
            alive += self
                .get_entities_in_aabb_matching(&search, |entity| {
                    entity.entity_type() == &END_CRYSTAL
                })
                .len() as i32;
        }
        self.dragon_fight.update_crystal_count(alive);
    }

    /// Vanilla `EnderDragonFight.updateDragon`: the live dragon feeds the boss bar.
    ///
    /// Called from the dragon's tick; returns whether this dragon owns the fight.
    pub fn update_dragon_fight(self: &Arc<Self>, health_fraction: f32) -> bool {
        if !self.is_end_dimension_type() {
            return false;
        }
        let owns = self.dragon_fight.update_dragon(health_fraction);
        if owns {
            self.sync_dragon_boss_bar();
        }
        owns
    }

    /// Publishes the fight state to the server's boss-bar registry, which is what
    /// actually reaches clients.
    ///
    /// Vanilla's `ServerPlayer.sendBossEvent` only writes the wire when a field
    /// actually changed, so this keeps the previous definition and skips the
    /// update when the dragon's health fraction is unchanged.
    fn sync_dragon_boss_bar(self: &Arc<Self>) {
        let Some(server) = self.server() else {
            return;
        };
        let bar = BossBar {
            uuid: dragon_bar_uuid(),
            name: text_components::TextComponent::translated(TranslatedMessage {
                key: "entity.minecraft.ender_dragon".into(),
                fallback: None,
                args: None,
            }),
            progress: self.dragon_fight.progress(),
            color: BossBarColor::Pink,
            overlay: BossBarOverlay::Progress,
            flags: 0,
            visible: self.dragon_fight.bar_visible(),
        };
        let previous = server
            .boss_bars
            .insert(DRAGON_BAR_ID.to_owned(), bar.clone());
        let Some(previous) = previous else {
            // A brand new bar has to be pushed in full before updates matter.
            self.broadcast_to_all::<CBossEvent>(bar.add_packet());
            return;
        };
        if previous.progress.to_bits() == bar.progress.to_bits() && previous.visible == bar.visible
        {
            return;
        }
        self.broadcast_to_all::<CBossEvent>(CBossEvent {
            uuid: bar.uuid,
            action: BossEventAction::UpdateProgress {
                progress: bar.progress,
            },
        });
    }

    /// Vanilla `EnderDragonFight.setDragonKilled`: empties the bar, opens the
    /// exit portal, drops the egg on the first kill and hides the bar.
    pub fn on_dragon_killed(self: &Arc<Self>) {
        if !self.is_end_dimension_type() {
            return;
        }
        let outcome = self.dragon_fight.set_dragon_killed();
        if outcome.spawn_exit_portal {
            self.spawn_end_exit_portal(true);
        }
        if outcome.drop_dragon_egg {
            self.spawn_dragon_egg();
        }
        if outcome.spawn_gateway
            && let Some(gateway) = self.dragon_fight.next_gateway()
        {
            self.spawn_end_gateway(IVec3::new(gateway.x, gateway.y, gateway.z));
        }
        self.hide_dragon_boss_bar();
    }

    /// Vanilla `EnderDragonFight.spawnNewGateway(pos)`: the `3000` level event and
    /// then the `end_gateway_delayed` feature at `pos`.
    ///
    /// The feature is the same code worldgen runs, so the bedrock capsule and the
    /// gateway block entity (which owns the teleport exit) come out identical.
    fn spawn_end_gateway(self: &Arc<Self>, position: IVec3) {
        let pos = BlockPos::new(position.x, position.y, position.z);
        self.global_level_event(dragon_fight::EVENT_GATEWAY_SPAWN, pos, 0);
        place_end_gateway_in_world(
            self,
            pos,
            &EndGatewayConfiguration {
                exit: None,
                exact: false,
            },
        );
    }

    /// Removes the dragon boss bar from the registry and the clients.
    fn hide_dragon_boss_bar(self: &Arc<Self>) {
        let Some(server) = self.server() else {
            return;
        };
        let Some(bar) = server.boss_bars.remove(DRAGON_BAR_ID) else {
            return;
        };
        self.broadcast_to_all::<CBossEvent>(bar.remove_packet());
    }

    /// Vanilla `EnderDragonFight.spawnExitPortal(activated)`: drop the
    /// `end_podium_active`/`end_podium_inactive` feature on the resolved portal
    /// location.
    ///
    /// The location is resolved the way vanilla does it: heightmap of the podium,
    /// one below, then walk down past bedrock until the world's minimum height.
    fn spawn_end_exit_portal(self: &Arc<Self>, activated: bool) {
        // Vanilla only resolves (and records) the location when it has none; an
        // already-known location is reused as-is, so a re-spawn never moves the
        // portal.
        let location = self.exit_portal_location();
        if self.dragon_fight.exit_portal_location().is_none() {
            self.dragon_fight.set_exit_portal_location(Some(IVec3::new(
                location.x(),
                location.y(),
                location.z(),
            )));
        }
        place_end_podium_in_world(self, location, activated);
    }

    /// Vanilla `EnderDragonFight.hasActiveExitPortal` / `findExitPortal`, reduced
    /// to the existence answer `scanState` needs.
    ///
    /// Vanilla scans the `-8..=8` chunk square for any `TheEndPortalBlockEntity`
    /// and then matches the full 9x9x4 exit-portal pattern around that block.
    /// Steel only needs the existence answer — `scanState` uses it solely to decide
    /// whether to open an inactive podium — so the block-entity sweep is the whole
    /// search. An inactive podium writes no `END_PORTAL` at all, so a pattern
    /// match could not succeed after a miss here either.
    #[must_use]
    fn has_active_exit_portal(&self) -> bool {
        // The fight origin is always chunk `(0, 0)` in the End.
        for chunk_x in -8..=8_i32 {
            for chunk_z in -8..=8_i32 {
                let found = self
                    .chunk_map
                    .with_full_chunk(ChunkPos::new(chunk_x, chunk_z), |chunk| {
                        chunk
                            .get_block_entities()
                            .into_iter()
                            .any(|block_entity| block_entity.is::<EndPortalBlockEntity>())
                    })
                    .unwrap_or(false);
                if found {
                    return true;
                }
            }
        }
        false
    }

    /// Vanilla `EnderDragonFight.exitPortalLocation`, resolved from the world the
    /// first time it is needed.
    #[must_use]
    fn exit_portal_location(self: &Arc<Self>) -> BlockPos {
        if let Some(pos) = self.dragon_fight.exit_portal_location() {
            return BlockPos::new(pos.x, pos.y, pos.z);
        }
        let podium = dragon_fight::DragonFight::podium_pos();
        let surface = self.height_at(
            HeightmapType::MotionBlockingNoLeaves,
            podium.x(),
            podium.z(),
        );
        let mut y = surface - 1;
        while self
            .get_block_state(BlockPos::new(podium.x(), y, podium.z()))
            .get_block()
            == &vanilla_blocks::BEDROCK
            && y > 63
        {
            y -= 1;
        }
        BlockPos::new(podium.x(), y.max(self.get_min_y() + 1), podium.z())
    }

    /// Vanilla `EnderDragonFight.setDragonKilled`: the first kill places the dragon
    /// egg at `getHeightmapPos(MOTION_BLOCKING, getPodiumLocation(origin))` — the
    /// first air block above the podium's surface, not a fixed offset from the
    /// podium point.
    fn spawn_dragon_egg(self: &Arc<Self>) {
        let podium = dragon_fight::DragonFight::podium_pos();
        self.set_block(
            self.heightmap_pos(HeightmapType::MotionBlocking, podium),
            steel_registry::REGISTRY
                .blocks
                .get_default_state_id(&vanilla_blocks::DRAGON_EGG),
            UpdateFlags::UPDATE_ALL,
        );
    }

    /// Returns the domain this loaded world belongs to.
    #[must_use]
    pub fn domain(&self) -> &str {
        self.key.namespace.as_ref()
    }

    pub(crate) fn set_domain_entity_directory(&self, directory: Arc<DomainEntityDirectory>) {
        *self.domain_entity_directory.write() = Some(directory);
    }

    /// Gets an entity by UUID, checking this world before other loaded worlds in its domain.
    #[must_use]
    pub fn get_entity_in_domain_by_uuid(&self, uuid: &uuid::Uuid) -> Option<SharedEntity> {
        self.get_entity_by_uuid(uuid).or_else(|| {
            self.domain_entity_directory
                .read()
                .as_ref()
                .and_then(|directory| directory.get_entity_by_uuid(uuid))
        })
    }

    /// Game tick: weather, time, chunk game tick (broadcasts + random/scheduled ticks),
    /// and player logic (without chunk sending).
    ///
    /// Returns whether this dimension is kept active without players.
    ///
    /// Mirrors vanilla `ServerLevel`'s `FLAG_KEEP_DIMENSION_ACTIVE` check: a
    /// dimension whose tickets carry the flag keeps its game time and chunk
    /// simulation running even with no players inside.
    #[must_use]
    pub fn is_kept_active(&self) -> bool {
        self.chunk_map.dimension_kept_active()
    }

    /// Vanilla keeps ticking removed players from their connection.
    ///
    /// Steel ticks players through the entity manager, which skips removed
    /// entities, so a dead player would freeze: death animation, respawn timer,
    /// and `play_time` would all stop. Ticking them here keeps that behaviour and
    /// must run before the entity tick so the removal tick is not processed
    /// twice. Unlike the entity tick this skips `commonTick`, so the tick count
    /// stays frozen like vanilla.
    fn tick_removed_players(self: &Arc<Self>) {
        self.players.iter_players(|_, player| {
            if player.is_removed() {
                Arc::clone(player).tick();
            }
            true
        });
    }

    /// Executes one full world game tick (time, weather, chunks, entities, block entities).
    #[tracing::instrument(level = "trace", skip(self), name = "world_game_tick")]
    #[expect(
        clippy::too_many_lines,
        reason = "world tick orchestration keeps vanilla subsystem order explicit"
    )]
    pub fn tick_game(
        self: &Arc<Self>,
        tick_count: u64,
        runs_normally: bool,
    ) -> WorldGameTickTimings {
        let world_start = Instant::now();
        let (is_overloaded, overload_ticks) = self
            .server
            .get()
            .and_then(Weak::upgrade)
            .map_or((false, 0), |s| {
                (
                    s.is_overloaded.load(Ordering::Relaxed),
                    s.overload_tick_count.load(Ordering::Relaxed),
                )
            });
        // Lightest + critical first: border/weather/time are tiny and affect
        // player position/time sync, so they always run.
        let lookup_cache_scope = GameplayChunkLookupCacheScope::enter(&self.chunk_map);
        self.handling_tick.store(true, Ordering::Relaxed);
        self.set_tick_runs_normally(runs_normally);
        if runs_normally {
            self.tick_world_border();
            self.tick_weather();
        }
        self.tick_sleeping_players();
        if runs_normally {
            self.tick_time();
        }

        let random_tick_speed = self.get_game_rule(&RANDOM_TICK_SPEED) as u32;

        let early_lookup_stats = lookup_cache_scope.finish();
        // Chunk source updates + random ticks are the heaviest part of the
        // world tick. If the server is overloaded we defer random ticks to
        // the next tick (they are stateless) while still advancing tickets.
        let mut chunk_map_timings = if is_overloaded && random_tick_speed > 0 {
            // Defer heavy random block ticks but keep precipitation and a reduced
            // budget of random ticks so the world does not desync. Ticket and
            // propagation work always runs. Lightest+important first: we keep
            // precipitation (player-visible) and do half the random ticks.
            let reduced_speed = (random_tick_speed / 2).max(1);
            self.chunk_map
                .tick_game(self, tick_count, reduced_speed, runs_normally)
        } else {
            self.chunk_map
                .tick_game(self, tick_count, random_tick_speed, runs_normally)
        };
        chunk_map_timings.lookup_cache.merge(early_lookup_stats);
        let lookup_cache_scope = GameplayChunkLookupCacheScope::enter(&self.chunk_map);

        if runs_normally {
            let _span = tracing::trace_span!("block_events").entered();
            self.run_block_events();
        }

        // Vanilla clears this before ticking entities and block entities.
        self.handling_tick.store(false, Ordering::Relaxed);

        self.tick_removed_players();

        let entity_tick = {
            let _span = tracing::trace_span!("entity_tick").entered();
            let start = Instant::now();
            let dirty_chunks = self.entity_manager.tick_entities(
                tick_count as i32,
                runs_normally,
                is_overloaded,
                overload_ticks,
            );
            for chunk in dirty_chunks {
                self.mark_chunk_dirty(chunk);
            }
            start.elapsed()
        };

        {
            let _span = tracing::trace_span!("block_entities").entered();
            let start = Instant::now();
            self.block_entity_tickers.tick(self, runs_normally);
            chunk_map_timings.tick_block_entities = start.elapsed();
        }

        // Natural spawning and custom spawners are heavy and stateless per tick:
        // lightest+important first: when overloaded we run them with reduced
        // frequency (every 2nd tick) instead of skipping entirely, so the
        // world does not desync and the queue is drained next tick.
        let should_spawn = !is_overloaded || tick_count.is_multiple_of(2);
        if runs_normally && should_spawn {
            let _span = tracing::trace_span!("natural_spawning").entered();
            tick_natural_spawning(self, tick_count as usize);
        }

        if runs_normally && should_spawn {
            // Vanilla runs the wandering trader spawner from `ServerLevel`'s
            // custom spawners, once per tick; its own 1200-tick schedule makes
            // the call cheap on the other 1199. Lightest+important first.
            self.tick_wandering_trader_spawner();
            self.tick_phantom_spawner();
            self.tick_patrol_spawner();
            self.tick_dragon_fight();
        } else if runs_normally && is_overloaded {
            // Even under overload, dragon fight is important in the End – run it
            // with reduced frequency (every 4th tick) to keep the fight in sync.
            if tick_count.is_multiple_of(4) {
                self.tick_dragon_fight();
            }
        }

        {
            let _span = tracing::trace_span!("entity_tracker_send_changes").entered();
            self.entity_tracker.send_changes(
                |chunk| self.get_packet_tracking_players(chunk),
                |player_id| self.players.get_by_entity_id(player_id),
                EntityChangeSenders {
                    movement: |entity_id, packet| {
                        self.broadcast_movement_sync_to_entity_trackers(entity_id, packet, None);
                    },
                    self_movement: |player_id, packet| {
                        let Some(encoded) = self.encode_movement_sync_packet(packet) else {
                            return;
                        };
                        let Some(player) = self.players.get_by_entity_id(player_id) else {
                            return;
                        };
                        player.connection.send_encoded(encoded);
                    },
                    entity_data: |entity_id, dirty_entity_data| {
                        let packet = CSetEntityData::new(entity_id, dirty_entity_data);
                        let Ok(encoded) = EncodedPacket::from_bare(
                            packet,
                            self.compression,
                            ConnectionProtocol::Play,
                        ) else {
                            return;
                        };
                        self.broadcast_to_entity_trackers_encoded(entity_id, encoded.clone(), None);
                        if let Some(player) = self.players.get_by_entity_id(entity_id) {
                            player.connection.send_encoded(encoded);
                        }
                    },
                    attributes: |entity_id, dirty_attributes| {
                        let packet = CUpdateAttributes::new(entity_id, dirty_attributes);
                        let Ok(encoded) = EncodedPacket::from_bare(
                            packet,
                            self.compression,
                            ConnectionProtocol::Play,
                        ) else {
                            return;
                        };
                        self.broadcast_to_entity_trackers_encoded(entity_id, encoded.clone(), None);
                        if let Some(player) = self.players.get_by_entity_id(entity_id) {
                            player.connection.send_encoded(encoded);
                        }
                    },
                    mob_effects: |player_id, packet| {
                        let Some(player) = self.players.get_by_entity_id(player_id) else {
                            return;
                        };
                        match packet {
                            MobEffectSyncPacket::Update(packet) => player.send_packet(packet),
                            MobEffectSyncPacket::Remove(packet) => player.send_packet(packet),
                        }
                    },
                    equipment: |entity_id, packet: CSetEquipment| {
                        let Ok(encoded) = EncodedPacket::from_bare(
                            packet,
                            self.compression,
                            ConnectionProtocol::Play,
                        ) else {
                            return;
                        };
                        self.broadcast_to_entity_trackers_encoded(entity_id, encoded, None);
                    },
                    passengers: |player_id, packet| {
                        if let Some(player) = self.players.get_by_entity_id(player_id) {
                            player.send_packet(packet);
                        }
                    },
                    entity_link: |entity_id, packet: CSetEntityLink| {
                        let Ok(encoded) = EncodedPacket::from_bare(
                            packet,
                            self.compression,
                            ConnectionProtocol::Play,
                        ) else {
                            return;
                        };
                        self.broadcast_to_entity_trackers_encoded(entity_id, encoded, None);
                    },
                },
            );
        }

        chunk_map_timings
            .lookup_cache
            .merge(lookup_cache_scope.finish());
        WorldGameTickTimings {
            elapsed: world_start.elapsed(),
            chunk_map: chunk_map_timings,
            entity_tick,
        }
    }

    /// Saves all dirty chunks in this world to disk.
    ///
    /// This should be called during graceful shutdown.
    /// Returns the number of chunks saved.
    pub async fn save_all_chunks(&self) -> io::Result<usize> {
        self.chunk_map.save_all_chunks().await
    }

    /// Runs vanilla `PhantomSpawner.tick` for this world.
    pub fn tick_phantom_spawner(self: &Arc<Self>) {
        let world = Arc::clone(self);
        let mut spawner = self.phantoms.lock();
        spawner.tick(&world);
    }

    /// Runs vanilla `PatrolSpawner.tick` for this world.
    pub fn tick_patrol_spawner(self: &Arc<Self>) {
        let world = Arc::clone(self);
        let mut spawner = self.patrols.lock();
        spawner.tick(&world);
    }

    /// Whether pillager patrols may spawn in this world at all.
    ///
    /// Vanilla registers `PatrolSpawner` for the Overworld only; Steel tests the
    /// sky, like the other custom spawners, so a datapack dimension with a day
    /// timeline gets patrols as well.
    #[must_use]
    pub const fn allows_patrols(&self) -> bool {
        self.dimension_type.has_skylight
    }

    /// Whether it is daytime, using vanilla's `Level.isDay`.
    ///
    /// `Level.isDay` is `!dimensionType().hasFixedTime() && isBrightOutside()`,
    /// and `isBrightOutside` is the sky darkness falling under the daylight
    /// threshold.
    #[must_use]
    pub fn is_day(&self) -> bool {
        !self.dimension_type.has_fixed_time && self.sky_darkening() < 4
    }

    /// Whether phantoms may spawn in this world at all.
    ///
    /// Vanilla registers `PhantomSpawner` for the Overworld only. Steel tests the
    /// sky instead of the dimension identity, so a datapack dimension with a sky
    /// behaves like the Overworld, as it does for the trader spawner.
    #[must_use]
    pub const fn allows_phantoms(&self) -> bool {
        self.dimension_type.has_skylight
    }

    /// Runs vanilla `WanderingTraderSpawner.tick` for this world.
    pub fn tick_wandering_trader_spawner(self: &Arc<Self>) {
        // Clone the handle out of the lock so the spawner can query the world
        // (entities, heightmaps, game rules) while it holds its own state.
        let world = Arc::clone(self);
        let mut spawner = self.wandering_traders.lock();
        spawner.tick(&world);
    }

    /// Whether wandering traders may exist in this world at all.
    ///
    /// Vanilla registers the spawner for the Overworld only. Steel has no
    /// dimension-type identity check that survives datapacks, so it uses the
    /// property that makes Overworld trading meaningful: the dimension has a sky
    /// and therefore surface positions to place a trader on.
    #[must_use]
    pub const fn allows_wandering_traders(&self) -> bool {
        self.dimension_type.has_skylight
    }

    /// Whether a natural wandering trader is already present.
    ///
    /// Vanilla keeps at most one per world and holds the day counter while one
    /// exists, so the check walks the live entities instead of tracking a flag
    /// that would go stale when the trader despawns or is killed.
    #[must_use]
    pub fn has_natural_wandering_trader(&self) -> bool {
        World::entity_manager(self)
            .any_accessible_entity(|entity| entity.entity_type() == &WANDERING_TRADER)
    }
}

impl LevelReader for World {
    fn get_block_state(&self, pos: BlockPos) -> BlockStateId {
        Self::get_block_state(self, pos)
    }

    fn get_block_entity(&self, pos: BlockPos) -> Option<SharedBlockEntity> {
        Self::get_block_entity(self, pos)
    }

    fn is_face_sturdy_for(
        &self,
        state: BlockStateId,
        pos: BlockPos,
        direction: Direction,
        support_type: SupportType,
    ) -> bool {
        BLOCK_BEHAVIORS
            .get_behavior(state.get_block())
            .is_face_sturdy(state, self, pos, direction, support_type)
    }

    fn sky_darkening(&self) -> u8 {
        World::sky_darkening(self)
    }

    fn raw_brightness(&self, pos: BlockPos, sky_darkening: u8) -> u8 {
        let sky_light = if self.dimension_type.has_skylight {
            self.light_value_at(LightLayer::Sky, pos)
                .saturating_sub(sky_darkening)
        } else {
            0
        };

        if sky_light == MAX_LIGHT_LEVEL {
            return MAX_LIGHT_LEVEL;
        }

        sky_light.max(self.light_value_at(LightLayer::Block, pos))
    }

    fn can_see_sky(&self, pos: BlockPos) -> bool {
        Self::can_see_sky(self, pos)
    }

    fn ambient_light(&self) -> f32 {
        self.dimension_type.ambient_light
    }

    fn min_y(&self) -> i32 {
        self.get_min_y()
    }

    fn height(&self) -> i32 {
        self.get_height()
    }

    fn sea_level(&self) -> i32 {
        self.sea_level
    }
}

impl LevelReader for Arc<World> {
    fn get_block_state(&self, pos: BlockPos) -> BlockStateId {
        self.as_ref().get_block_state(pos)
    }

    fn get_block_entity(&self, pos: BlockPos) -> Option<SharedBlockEntity> {
        self.as_ref().get_block_entity(pos)
    }

    fn is_face_sturdy_for(
        &self,
        state: BlockStateId,
        pos: BlockPos,
        direction: Direction,
        support_type: SupportType,
    ) -> bool {
        self.as_ref()
            .is_face_sturdy_for(state, pos, direction, support_type)
    }

    fn sky_darkening(&self) -> u8 {
        self.as_ref().sky_darkening()
    }

    fn raw_brightness(&self, pos: BlockPos, sky_darkening: u8) -> u8 {
        self.as_ref().raw_brightness(pos, sky_darkening)
    }

    fn can_see_sky(&self, pos: BlockPos) -> bool {
        self.as_ref().can_see_sky(pos)
    }

    fn ambient_light(&self) -> f32 {
        self.as_ref().ambient_light()
    }

    fn height_at(&self, heightmap_type: HeightmapType, x: i32, z: i32) -> i32 {
        let mapped_type = match heightmap_type {
            HeightmapType::WorldSurfaceWg => HeightmapType::WorldSurface,
            HeightmapType::OceanFloorWg => HeightmapType::OceanFloor,
            other => other,
        };
        self.as_ref()
            .height_at(mapped_type, x, z)
            .unwrap_or_else(|| self.min_y())
    }

    fn min_y(&self) -> i32 {
        self.as_ref().get_min_y()
    }

    fn height(&self) -> i32 {
        self.as_ref().get_height()
    }

    fn sea_level(&self) -> i32 {
        self.as_ref().sea_level
    }
}

impl ScheduledTickAccess for Arc<World> {
    fn fluid_tick_delay(&self, fluid: FluidRef) -> i32 {
        FLUID_BEHAVIORS.get_behavior(fluid).tick_delay(self)
    }

    fn schedule_block_tick_default(&self, pos: BlockPos, block: BlockRef, delay: i32) -> bool {
        self.as_ref().schedule_block_tick_default(pos, block, delay);
        true
    }

    fn has_scheduled_block_tick(&self, pos: BlockPos, block: BlockRef) -> bool {
        self.as_ref().has_scheduled_block_tick(pos, block)
    }

    fn will_tick_block_this_tick(&self, pos: BlockPos, block: BlockRef) -> bool {
        self.as_ref().will_tick_block_this_tick(pos, block)
    }

    fn schedule_fluid_tick_default(&self, pos: BlockPos, fluid: FluidRef, delay: i32) -> bool {
        self.as_ref().schedule_fluid_tick_default(pos, fluid, delay);
        true
    }

    fn will_tick_fluid_this_tick(&self, pos: BlockPos, fluid: FluidRef) -> bool {
        self.as_ref().will_tick_fluid_this_tick(pos, fluid)
    }
}

impl LevelAccessor for Arc<World> {
    fn set_block_state(&self, pos: BlockPos, state: BlockStateId, flags: UpdateFlags) -> bool {
        self.set_block(pos, state, flags)
    }

    fn can_write_to_chunk(&self, chunk_x: i32, chunk_z: i32) -> bool {
        self.chunk_map
            .with_full_chunk(ChunkPos::new(chunk_x, chunk_z), |_| ())
            .is_some()
    }

    fn requires_live_write_preflight(&self) -> bool {
        true
    }

    fn destroy_block(&self, pos: BlockPos, drop_items: bool) -> bool {
        World::destroy_block(self, pos, drop_items)
    }

    fn play_block_sound(
        &self,
        sound: SoundEventRef,
        pos: BlockPos,
        volume: f32,
        pitch: f32,
        exclude: Option<i32>,
    ) {
        self.as_ref()
            .play_block_sound(sound, pos, volume, pitch, exclude);
    }

    fn game_event(&self, event: GameEventRef, pos: BlockPos, context: &GameEventContext<'_>) {
        World::game_event(self, event, pos, context);
    }
}
