//! Player data storage for global and domain-scoped player state.

mod file_storage;
mod known_players;
mod memory_storage;
mod permissions;
mod stats;

#[cfg(test)]
mod tests;

use std::{io::Cursor, path::PathBuf};

use simdnbt::{ToNbtTag, borrow::read_compound as read_borrowed_compound, owned::NbtTag};
use tokio::io;
use uuid::Uuid;
use wincode::{SchemaRead, SchemaWrite};

#[cfg(test)]
use self::permissions::set_permission_subject;
use super::PlayerRespawnConfig;
use super::player_data::{
    PLAYER_DATA_VERSION, PersistentAbilities, PersistentEnderPearl, PersistentPlayerData,
    PersistentRootVehicle, PersistentSlot,
};
use crate::chunk_saver::PersistentEntity;
use crate::config::StorageSelection;
use crate::level_data::RespawnData;
use crate::permission::PermissionSubjectIndex;
#[cfg(test)]
use crate::permission::PermissionSubjectState;
use crate::player::KnownPlayers;
use crate::player::Player;
use crate::player::player_data_storage::file_storage::FilePlayerDataStorage;
use crate::player::player_data_storage::memory_storage::MemoryPlayerDataStorage;
use steel_registry::item_stack::ItemStack;
use steel_utils::{BlockPos, Identifier};

const PLAYER_MAGIC: [u8; 4] = *b"STLP";
const GLOBAL_MAGIC: [u8; 4] = *b"STLG";
/// Current on-disk layout version of a player data file.
const PLAYER_STORAGE_VERSION: u16 = 12;
/// Oldest on-disk layout this build can still read.
///
/// Version 10 lacked the vanilla `warden_spawn_tracker` block that version 11
/// appended, and version 12 appended `post_effects`. Instead of rejecting every
/// save written by an earlier build, the reader decodes the older layouts and
/// fills the missing fields with vanilla defaults. Bump this only together with
/// a reader.
const OLDEST_READABLE_PLAYER_STORAGE_VERSION: u16 = 10;
const GLOBAL_STORAGE_VERSION: u16 = 1;
const GLOBAL_PLAYER_DATA_VERSION: i32 = 1;

/// Server-wide player data.
#[derive(Debug, Clone)]
pub struct GlobalPlayerData {
    /// Last active domain for reconnects.
    pub last_active_domain: String,
}

/// Manages player data persistence.
pub struct PlayerDataStorage {
    backend: PlayerDataStorageBackend,
}

enum PlayerDataStorageBackend {
    File(FilePlayerDataStorage),
    Memory(MemoryPlayerDataStorage),
}

#[derive(SchemaWrite, SchemaRead)]
struct PlayerDataFile {
    data_version: i32,
    pos: [f64; 3],
    motion: [f64; 3],
    rotation: [f32; 2],
    on_ground: bool,
    fall_flying: bool,
    remaining_fire_ticks: i32,
    ticks_frozen: i32,
    is_in_powder_snow: bool,
    was_in_powder_snow: bool,
    has_visual_fire: bool,
    health: f32,
    game_mode: i32,
    prev_game_mode: Option<i32>,
    abilities: AbilitiesFile,
    inventory: Vec<SlotFile>,
    selected_slot: i32,
    world: String,
    food_level: i32,
    food_saturation_level: f32,
    food_exhaustion_level: f32,
    food_tick_timer: i32,
    experience_level: i32,
    experience_progress: f32,
    experience_total: i32,
    enchantment_seed: i32,
    score: i32,
    seen_credits: bool,
    root_vehicle: Option<RootVehicleFile>,
    respawn_config: Option<RespawnConfigFile>,
    ender_pearls: Vec<EnderPearlFile>,
    ender_items: Vec<SlotFile>,
    warden_spawn_tracker: WardenSpawnTrackerFile,
    /// Vanilla `post_effects` ids; empty for every save older than version 12.
    post_effects: Vec<String>,
}

/// Vanilla `warden_spawn_tracker` in the player save.
#[derive(SchemaWrite, SchemaRead)]
struct WardenSpawnTrackerFile {
    ticks_since_last_warning: i32,
    warning_level: i32,
    cooldown_ticks: i32,
}

#[derive(SchemaWrite, SchemaRead)]
struct RootVehicleFile {
    attach: [u8; 16],
    entity: PersistentEntity,
}

#[derive(SchemaWrite, SchemaRead)]
struct RespawnConfigFile {
    dimension: String,
    pos: [i32; 3],
    yaw: f32,
    pitch: f32,
    forced: bool,
}

#[derive(SchemaWrite, SchemaRead)]
struct EnderPearlFile {
    world: String,
    entity: PersistentEntity,
}

#[derive(SchemaWrite, SchemaRead)]
struct AbilitiesFile {
    invulnerable: bool,
    flying: bool,
    may_fly: bool,
    instabuild: bool,
    may_build: bool,
    flying_speed: f32,
    walking_speed: f32,
}

#[derive(SchemaWrite, SchemaRead)]
struct SlotFile {
    slot: i8,
    item_nbt: Vec<u8>,
}

#[derive(SchemaWrite, SchemaRead)]
struct GlobalPlayerDataFile {
    data_version: i32,
    last_active_domain: String,
}

impl PlayerDataStorage {
    /// Creates the player data storage a config selection asks for.
    ///
    /// `save_root` is only used by backends that persist.
    pub async fn from_selection(
        save_root: PathBuf,
        selection: &StorageSelection,
    ) -> io::Result<Self> {
        if selection.kind == Identifier::from_steel("disk") {
            Self::on_disk(save_root).await
        } else if selection.kind == Identifier::from_steel("ram") {
            Ok(Self::in_memory())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown player storage {}", selection.kind),
            ))
        }
    }

    /// Creates player data storage backed by files under `save_root`.
    pub async fn on_disk(save_root: PathBuf) -> io::Result<Self> {
        let backend = PlayerDataStorageBackend::File(FilePlayerDataStorage::new(save_root).await?);
        Ok(Self { backend })
    }

    /// Creates player data storage that keeps everything in RAM.
    #[must_use]
    pub fn in_memory() -> Self {
        Self {
            backend: PlayerDataStorageBackend::Memory(MemoryPlayerDataStorage::default()),
        }
    }

    /// Saves a player's current domain data and global last-active-domain.
    pub async fn save(&self, player: &Player) -> io::Result<()> {
        let domain = player.get_world().domain().to_owned();
        self.save_domain(&domain, player).await?;
        self.save_global(
            player.gameprofile.id(),
            &GlobalPlayerData {
                last_active_domain: domain,
            },
        )
        .await
    }

    /// Saves a player's data for a specific domain.
    pub async fn save_domain(&self, domain: &str, player: &Player) -> io::Result<()> {
        match &self.backend {
            PlayerDataStorageBackend::File(storage) => storage.save_domain(domain, player).await,
            PlayerDataStorageBackend::Memory(storage) => {
                storage.save_domain(domain, player);
                Ok(())
            }
        }
    }

    /// Saves an already captured player data snapshot for a specific domain.
    pub async fn save_domain_data(
        &self,
        domain: &str,
        uuid: Uuid,
        data: &PersistentPlayerData,
    ) -> io::Result<()> {
        match &self.backend {
            PlayerDataStorageBackend::File(storage) => {
                storage.save_domain_data(domain, uuid, data).await
            }
            PlayerDataStorageBackend::Memory(storage) => {
                storage.save_domain_data(domain, uuid, data);
                Ok(())
            }
        }
    }

    /// Loads a player's data for a specific domain.
    pub async fn load_domain(
        &self,
        domain: &str,
        uuid: Uuid,
    ) -> io::Result<Option<PersistentPlayerData>> {
        match &self.backend {
            PlayerDataStorageBackend::File(storage) => storage.load_domain(domain, uuid).await,
            PlayerDataStorageBackend::Memory(storage) => Ok(storage.load_domain(domain, uuid)),
        }
    }

    /// Loads server-wide player data.
    pub async fn load_global(&self, uuid: Uuid) -> io::Result<Option<GlobalPlayerData>> {
        match &self.backend {
            PlayerDataStorageBackend::File(storage) => storage.load_global(uuid).await,
            PlayerDataStorageBackend::Memory(storage) => Ok(storage.load_global(uuid)),
        }
    }

    /// Loads all persisted player permission snapshots.
    pub async fn load_permission_subjects(&self) -> io::Result<PermissionSubjectIndex> {
        match &self.backend {
            PlayerDataStorageBackend::File(storage) => storage.load_permission_subjects().await,
            PlayerDataStorageBackend::Memory(storage) => Ok(storage.load_permission_subjects()),
        }
    }

    /// Loads the rebuildable player identity cache, falling back to empty on failure.
    pub async fn load_known_players(&self) -> io::Result<KnownPlayers> {
        match &self.backend {
            PlayerDataStorageBackend::File(storage) => storage.load_known_players().await,
            PlayerDataStorageBackend::Memory(storage) => Ok(storage.load_known_players()),
        }
    }

    /// Persists the identity cache when the caller's snapshot is still current.
    pub async fn save_known_players_if_current(
        &self,
        players: &KnownPlayers,
        is_current: impl FnOnce() -> bool + Send,
    ) -> io::Result<bool> {
        match &self.backend {
            PlayerDataStorageBackend::File(storage) => {
                storage
                    .save_known_players_if_current(players, is_current)
                    .await
            }
            PlayerDataStorageBackend::Memory(storage) => {
                Ok(storage.save_known_players_if_current(players, is_current))
            }
        }
    }

    /// Saves server-wide player data.
    pub async fn save_global(&self, uuid: Uuid, data: &GlobalPlayerData) -> io::Result<()> {
        match &self.backend {
            PlayerDataStorageBackend::File(storage) => storage.save_global(uuid, data).await,
            PlayerDataStorageBackend::Memory(storage) => {
                storage.save_global(uuid, data);
                Ok(())
            }
        }
    }

    /// Persists the server's complete UUID-keyed permission snapshot.
    pub async fn save_permission_subjects(
        &self,
        subjects: &PermissionSubjectIndex,
    ) -> io::Result<()> {
        match &self.backend {
            PlayerDataStorageBackend::File(storage) => {
                storage.save_permission_subjects(subjects).await
            }
            PlayerDataStorageBackend::Memory(storage) => {
                storage.save_permission_subjects(subjects);
                Ok(())
            }
        }
    }
}

/// Player data layout written by builds before the warden spawn tracker.
///
/// Identical to [`PlayerDataFile`] minus the trailing `warden_spawn_tracker`
/// field, which is what makes version 10 decodable from the same byte stream
/// prefix.
#[derive(SchemaWrite, SchemaRead)]
struct PlayerDataFileV10 {
    data_version: i32,
    pos: [f64; 3],
    motion: [f64; 3],
    rotation: [f32; 2],
    on_ground: bool,
    fall_flying: bool,
    remaining_fire_ticks: i32,
    ticks_frozen: i32,
    is_in_powder_snow: bool,
    was_in_powder_snow: bool,
    has_visual_fire: bool,
    health: f32,
    game_mode: i32,
    prev_game_mode: Option<i32>,
    abilities: AbilitiesFile,
    inventory: Vec<SlotFile>,
    selected_slot: i32,
    world: String,
    food_level: i32,
    food_saturation_level: f32,
    food_exhaustion_level: f32,
    food_tick_timer: i32,
    experience_level: i32,
    experience_progress: f32,
    experience_total: i32,
    enchantment_seed: i32,
    score: i32,
    seen_credits: bool,
    root_vehicle: Option<RootVehicleFile>,
    respawn_config: Option<RespawnConfigFile>,
    ender_pearls: Vec<EnderPearlFile>,
    ender_items: Vec<SlotFile>,
}

impl PlayerDataFileV10 {
    /// Fills in the fields added in version 11.
    fn into_current(self) -> PlayerDataFile {
        PlayerDataFile {
            data_version: self.data_version,
            pos: self.pos,
            motion: self.motion,
            rotation: self.rotation,
            on_ground: self.on_ground,
            fall_flying: self.fall_flying,
            remaining_fire_ticks: self.remaining_fire_ticks,
            ticks_frozen: self.ticks_frozen,
            is_in_powder_snow: self.is_in_powder_snow,
            was_in_powder_snow: self.was_in_powder_snow,
            has_visual_fire: self.has_visual_fire,
            health: self.health,
            game_mode: self.game_mode,
            prev_game_mode: self.prev_game_mode,
            abilities: self.abilities,
            inventory: self.inventory,
            selected_slot: self.selected_slot,
            world: self.world,
            food_level: self.food_level,
            food_saturation_level: self.food_saturation_level,
            food_exhaustion_level: self.food_exhaustion_level,
            food_tick_timer: self.food_tick_timer,
            experience_level: self.experience_level,
            experience_progress: self.experience_progress,
            experience_total: self.experience_total,
            enchantment_seed: self.enchantment_seed,
            score: self.score,
            seen_credits: self.seen_credits,
            root_vehicle: self.root_vehicle,
            respawn_config: self.respawn_config,
            ender_pearls: self.ender_pearls,
            ender_items: self.ender_items,
            // Vanilla defaults: a player that never woke a shrieker.
            warden_spawn_tracker: WardenSpawnTrackerFile {
                ticks_since_last_warning: 0,
                warning_level: 0,
                cooldown_ticks: 0,
            },
            // Vanilla default: no screen effect was ever added.
            post_effects: Vec::new(),
        }
    }
}

/// Player data layout written before version 12 appended `post_effects`.
#[derive(SchemaWrite, SchemaRead)]
struct PlayerDataFileV11 {
    data_version: i32,
    pos: [f64; 3],
    motion: [f64; 3],
    rotation: [f32; 2],
    on_ground: bool,
    fall_flying: bool,
    remaining_fire_ticks: i32,
    ticks_frozen: i32,
    is_in_powder_snow: bool,
    was_in_powder_snow: bool,
    has_visual_fire: bool,
    health: f32,
    game_mode: i32,
    prev_game_mode: Option<i32>,
    abilities: AbilitiesFile,
    inventory: Vec<SlotFile>,
    selected_slot: i32,
    world: String,
    food_level: i32,
    food_saturation_level: f32,
    food_exhaustion_level: f32,
    food_tick_timer: i32,
    experience_level: i32,
    experience_progress: f32,
    experience_total: i32,
    enchantment_seed: i32,
    score: i32,
    seen_credits: bool,
    root_vehicle: Option<RootVehicleFile>,
    respawn_config: Option<RespawnConfigFile>,
    ender_pearls: Vec<EnderPearlFile>,
    ender_items: Vec<SlotFile>,
    warden_spawn_tracker: WardenSpawnTrackerFile,
}

impl PlayerDataFileV11 {
    /// Fills in the field added in version 12.
    fn into_current(self) -> PlayerDataFile {
        PlayerDataFile {
            data_version: self.data_version,
            pos: self.pos,
            motion: self.motion,
            rotation: self.rotation,
            on_ground: self.on_ground,
            fall_flying: self.fall_flying,
            remaining_fire_ticks: self.remaining_fire_ticks,
            ticks_frozen: self.ticks_frozen,
            is_in_powder_snow: self.is_in_powder_snow,
            was_in_powder_snow: self.was_in_powder_snow,
            has_visual_fire: self.has_visual_fire,
            health: self.health,
            game_mode: self.game_mode,
            prev_game_mode: self.prev_game_mode,
            abilities: self.abilities,
            inventory: self.inventory,
            selected_slot: self.selected_slot,
            world: self.world,
            food_level: self.food_level,
            food_saturation_level: self.food_saturation_level,
            food_exhaustion_level: self.food_exhaustion_level,
            food_tick_timer: self.food_tick_timer,
            experience_level: self.experience_level,
            experience_progress: self.experience_progress,
            experience_total: self.experience_total,
            enchantment_seed: self.enchantment_seed,
            score: self.score,
            seen_credits: self.seen_credits,
            root_vehicle: self.root_vehicle,
            respawn_config: self.respawn_config,
            ender_pearls: self.ender_pearls,
            ender_items: self.ender_items,
            warden_spawn_tracker: self.warden_spawn_tracker,
            post_effects: Vec::new(),
        }
    }
}

impl PlayerDataFile {
    fn from_persistent(data: &PersistentPlayerData) -> io::Result<Self> {
        let mut inventory = Vec::with_capacity(data.inventory.len());
        for slot in &data.inventory {
            inventory.push(SlotFile {
                slot: slot.slot,
                item_nbt: item_to_nbt_bytes(&slot.item)?,
            });
        }

        Ok(Self {
            data_version: data.data_version,
            pos: data.pos,
            motion: data.motion,
            rotation: data.rotation,
            on_ground: data.on_ground,
            fall_flying: data.fall_flying,
            remaining_fire_ticks: data.remaining_fire_ticks,
            ticks_frozen: data.ticks_frozen,
            is_in_powder_snow: data.is_in_powder_snow,
            was_in_powder_snow: data.was_in_powder_snow,
            has_visual_fire: data.has_visual_fire,
            health: data.health,
            game_mode: data.game_mode,
            prev_game_mode: data.prev_game_mode,
            abilities: AbilitiesFile {
                invulnerable: data.abilities.invulnerable,
                flying: data.abilities.flying,
                may_fly: data.abilities.may_fly,
                instabuild: data.abilities.instabuild,
                may_build: data.abilities.may_build,
                flying_speed: data.abilities.flying_speed,
                walking_speed: data.abilities.walking_speed,
            },
            inventory,
            selected_slot: data.selected_slot,
            world: data.world.clone(),
            food_level: data.food_level,
            food_saturation_level: data.food_saturation_level,
            food_exhaustion_level: data.food_exhaustion_level,
            food_tick_timer: data.food_tick_timer,
            experience_level: data.experience_level,
            experience_progress: data.experience_progress,
            experience_total: data.experience_total,
            enchantment_seed: data.enchantment_seed,
            score: data.score,
            seen_credits: data.seen_credits,
            root_vehicle: data
                .root_vehicle
                .clone()
                .map(|root_vehicle| RootVehicleFile {
                    attach: root_vehicle.attach,
                    entity: root_vehicle.entity,
                }),
            respawn_config: data
                .respawn_config
                .clone()
                .map(RespawnConfigFile::from_runtime),
            ender_pearls: data
                .ender_pearls
                .iter()
                .map(|pearl| EnderPearlFile {
                    world: pearl.world.clone(),
                    entity: pearl.entity.clone(),
                })
                .collect(),
            ender_items: data
                .ender_items
                .iter()
                .map(|slot| SlotFile {
                    slot: slot.slot,
                    item_nbt: item_to_nbt_bytes(&slot.item).unwrap_or_default(),
                })
                .collect(),
            warden_spawn_tracker: WardenSpawnTrackerFile {
                ticks_since_last_warning: data.warden_spawn_tracker.ticks_since_last_warning,
                warning_level: data.warden_spawn_tracker.warning_level,
                cooldown_ticks: data.warden_spawn_tracker.cooldown_ticks,
            },
            post_effects: data
                .post_effects
                .iter()
                .map(ToString::to_string)
                .collect(),
        })
    }

    fn into_persistent(self) -> io::Result<PersistentPlayerData> {
        // Older payload versions are forward-compatible: every bump so far only
        // added fields whose vanilla default is the "player never did this"
        // value, so reading them as the current shape is lossless. A payload
        // from a *newer* build is rejected instead of silently dropping fields
        // this binary cannot represent.
        if self.data_version > PLAYER_DATA_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "unsupported player data payload version {} (this build writes {PLAYER_DATA_VERSION})",
                    self.data_version
                ),
            ));
        }

        let mut inventory = Vec::with_capacity(self.inventory.len());
        for slot in self.inventory {
            inventory.push(PersistentSlot {
                slot: slot.slot,
                item: item_from_nbt_bytes(&slot.item_nbt)?,
            });
        }

        Ok(PersistentPlayerData {
            pos: self.pos,
            motion: self.motion,
            rotation: self.rotation,
            on_ground: self.on_ground,
            fall_flying: self.fall_flying,
            remaining_fire_ticks: self.remaining_fire_ticks,
            ticks_frozen: self.ticks_frozen,
            is_in_powder_snow: self.is_in_powder_snow,
            was_in_powder_snow: self.was_in_powder_snow,
            has_visual_fire: self.has_visual_fire,
            health: self.health,
            game_mode: self.game_mode,
            prev_game_mode: self.prev_game_mode,
            abilities: PersistentAbilities {
                invulnerable: self.abilities.invulnerable,
                flying: self.abilities.flying,
                may_fly: self.abilities.may_fly,
                instabuild: self.abilities.instabuild,
                may_build: self.abilities.may_build,
                flying_speed: self.abilities.flying_speed,
                walking_speed: self.abilities.walking_speed,
            },
            inventory,
            selected_slot: self.selected_slot,
            world: self.world,
            food_level: self.food_level,
            food_saturation_level: self.food_saturation_level,
            food_exhaustion_level: self.food_exhaustion_level,
            food_tick_timer: self.food_tick_timer,
            data_version: self.data_version,
            experience_level: self.experience_level,
            experience_progress: self.experience_progress,
            experience_total: self.experience_total,
            enchantment_seed: self.enchantment_seed,
            score: self.score,
            seen_credits: self.seen_credits,
            root_vehicle: self.root_vehicle.map(|root_vehicle| PersistentRootVehicle {
                attach: root_vehicle.attach,
                entity: root_vehicle.entity,
            }),
            respawn_config: self
                .respawn_config
                .map(RespawnConfigFile::into_runtime)
                .transpose()?,
            ender_pearls: self
                .ender_pearls
                .into_iter()
                .map(|pearl| PersistentEnderPearl {
                    world: pearl.world,
                    entity: pearl.entity,
                })
                .collect(),
            ender_items: {
                let mut items = Vec::with_capacity(self.ender_items.len());
                for slot in self.ender_items {
                    items.push(PersistentSlot {
                        slot: slot.slot,
                        item: item_from_nbt_bytes(&slot.item_nbt)?,
                    });
                }
                items
            },
            stats: Vec::new(),
            advancements: Vec::new(),
            warden_spawn_tracker: super::player_data::PersistentWardenSpawnTracker {
                ticks_since_last_warning: self.warden_spawn_tracker.ticks_since_last_warning,
                warning_level: self.warden_spawn_tracker.warning_level,
                cooldown_ticks: self.warden_spawn_tracker.cooldown_ticks,
            },
            post_effects: self
                .post_effects
                .iter()
                .filter_map(|id| id.parse::<Identifier>().ok())
                .collect(),
        })
    }
}

impl RespawnConfigFile {
    fn from_runtime(config: PlayerRespawnConfig) -> Self {
        let pos = config.respawn_data.pos();
        Self {
            dimension: config.respawn_data.dimension().to_string(),
            pos: [pos.x(), pos.y(), pos.z()],
            yaw: config.respawn_data.yaw,
            pitch: config.respawn_data.pitch,
            forced: config.forced,
        }
    }

    fn into_runtime(self) -> io::Result<PlayerRespawnConfig> {
        Ok(PlayerRespawnConfig {
            respawn_data: RespawnData::of(
                self.dimension.parse().map_err(|error| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("invalid respawn dimension: {error}"),
                    )
                })?,
                BlockPos::new(self.pos[0], self.pos[1], self.pos[2]),
                self.yaw,
                self.pitch,
            ),
            forced: self.forced,
        })
    }
}

fn item_to_nbt_bytes(item: &ItemStack) -> io::Result<Vec<u8>> {
    let NbtTag::Compound(compound) = item.clone().to_nbt_tag() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "item stack did not serialize to a compound",
        ));
    };
    let mut bytes = Vec::new();
    compound.write(&mut bytes);
    Ok(bytes)
}

fn item_from_nbt_bytes(bytes: &[u8]) -> io::Result<ItemStack> {
    let nbt = read_borrowed_compound(&mut Cursor::new(bytes)).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse item NBT: {e}"),
        )
    })?;
    let compound = simdnbt::borrow::NbtCompound::from(&nbt);
    ItemStack::from_borrowed_compound(&compound)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid item stack data"))
}

fn encode_player_file(file: &PlayerDataFile) -> io::Result<Vec<u8>> {
    encode_file(
        PLAYER_MAGIC,
        PLAYER_STORAGE_VERSION,
        wincode::serialize(file),
    )
}

fn decode_player_file(bytes: &[u8]) -> io::Result<PlayerDataFile> {
    let version = read_file_version(PLAYER_MAGIC, bytes)?;
    match version {
        PLAYER_STORAGE_VERSION => {
            let payload = decode_file(PLAYER_MAGIC, PLAYER_STORAGE_VERSION, bytes)?;
            wincode::deserialize(&payload)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
        }
        11 => {
            // Saves written before `post_effects` existed: the payload is a
            // strict prefix of the current layout.
            let payload = decode_file(PLAYER_MAGIC, 11, bytes)?;
            let old: PlayerDataFileV11 = wincode::deserialize(&payload)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            Ok(old.into_current())
        }
        10 => {
            // Saves written before the warden spawn tracker existed. The layout
            // is a strict prefix of the current one, so the same decoder can
            // read it and the migration only fills the new field.
            let payload = decode_file(PLAYER_MAGIC, 10, bytes)?;
            let old: PlayerDataFileV10 = wincode::deserialize(&payload)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            Ok(old.into_current())
        }
        other => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported player data storage version {other}"),
        )),
    }
}

fn encode_global_file(file: &GlobalPlayerDataFile) -> io::Result<Vec<u8>> {
    encode_file(
        GLOBAL_MAGIC,
        GLOBAL_STORAGE_VERSION,
        wincode::serialize(file),
    )
}

fn decode_global_file(bytes: &[u8]) -> io::Result<GlobalPlayerDataFile> {
    let payload = decode_file(GLOBAL_MAGIC, GLOBAL_STORAGE_VERSION, bytes)?;
    wincode::deserialize(&payload)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

/// Reads the layout version of a stored file, validating its magic first.
///
/// Separate from [`decode_file`] so a caller can pick a decoder for the layout
/// it finds instead of rejecting everything that is not the current version.
fn read_file_version(expected_magic: [u8; 4], bytes: &[u8]) -> io::Result<u16> {
    if bytes.len() < 6 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "player data file is too short",
        ));
    }
    if bytes[0..4] != expected_magic {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid player data magic",
        ));
    }
    Ok(u16::from_le_bytes([bytes[4], bytes[5]]))
}

fn encode_file(
    magic: [u8; 4],
    version: u16,
    serialized: wincode::WriteResult<Vec<u8>>,
) -> io::Result<Vec<u8>> {
    let payload =
        serialized.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let compressed = zstd::encode_all(&payload[..], 3)?;
    let mut bytes = Vec::with_capacity(6 + compressed.len());
    bytes.extend_from_slice(&magic);
    bytes.extend_from_slice(&version.to_le_bytes());
    bytes.extend_from_slice(&compressed);
    Ok(bytes)
}

fn decode_file(
    expected_magic: [u8; 4],
    expected_version: u16,
    bytes: &[u8],
) -> io::Result<Vec<u8>> {
    if bytes.len() < 6 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "player data file is too short",
        ));
    }
    if bytes[0..4] != expected_magic {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid player data magic",
        ));
    }
    let version = read_file_version(expected_magic, bytes)?;
    let readable = version == expected_version
        || (expected_magic == PLAYER_MAGIC
            && (OLDEST_READABLE_PLAYER_STORAGE_VERSION..=expected_version).contains(&version));
    if !readable {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported player data storage version {version}"),
        ));
    }
    zstd::decode_all(&bytes[6..])
}
