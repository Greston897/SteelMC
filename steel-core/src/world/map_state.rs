//! Server-side map saved data.
//!
//! Port of vanilla `net.minecraft.item.map.MapState`: the 128×128 color grid,
//! scale, lock flag, decoration set and per-player update trackers that turn
//! color writes into small delta patches.
//!
//! Differences from vanilla, all deliberate:
//! * Every map of a loaded world lives in one versioned saved-data entry
//!   (`data/maps.bin`) instead of one `map_<id>.dat` per map, because Steel's
//!   [`SavedDataName`](steel_utils::saved_data::SavedDataName) requires static
//!   file names.
//! * Trackers are keyed by player UUID instead of holding a `PlayerEntity`
//!   reference, so a map never keeps a disconnected player in memory.
//! * Banner and item-frame markers are not ported yet (no banner block entity
//!   and no item-frame integration); player, component and cartography
//!   decorations are.
//! * Vanilla compares `RegistryKey<World> == World.NETHER` for the pseudo-random
//!   nether marker rotation; Steel compares the loaded world path because world
//!   keys are domain-scoped (`steel:the_nether`).

use rustc_hash::FxHashMap;

use steel_protocol::packets::game::{CMapItemData, MapDecorationEntry, MapPatch, OptionalMapPatch};
use steel_registry::data_components::components::MapDecorations;
use steel_registry::data_components::vanilla_components::{MAP_DECORATIONS, MAP_ID};
use steel_registry::map_decoration_type::MapDecorationTypeRef;
use steel_registry::{REGISTRY, RegistryExt as _, vanilla_map_decoration_types};
use steel_utils::Identifier;
use uuid::Uuid;
use wincode::{SchemaRead, SchemaWrite};

use crate::entity::Entity as _;
use crate::player::Player;
use crate::world::World;
use std::collections::hash_map::Entry;

/// Vanilla `MapState.SIZE`.
pub const MAP_SIZE: usize = 128;
/// Number of pixels in one map, `SIZE * SIZE`.
pub const MAP_AREA: usize = MAP_SIZE * MAP_SIZE;
/// Vanilla `MapState.SIZE_HALF`.
pub const MAP_SIZE_HALF: i32 = (MAP_SIZE / 2) as i32;
/// Vanilla `MapState.MAX_SCALE`.
pub const MAX_SCALE: u8 = 4;
/// Vanilla `MapState.MAX_DECORATIONS`.
pub const MAX_DECORATIONS: usize = 256;
/// Vanilla `MapState.DECORATION_SYNC_INTERVAL`.
const DECORATION_SYNC_INTERVAL: u32 = 5;
/// Vanilla `MapState.MAP_HALF_SIZE` used for the marker bounds check.
const MARKER_HALF_SIZE: f32 = 63.0;
/// Vanilla `MapState.PLAYER_OFF_MAP_RADIUS`.
const PLAYER_OFF_MAP_RADIUS: f32 = 320.0;
/// Vanilla `MapState.MARKER_MAX`.
const MARKER_MAX: i8 = 127;

/// Vanilla nether marker rotation constants (`MapState.getPlayerMarkerRotation`).
const NETHER_ROTATION_TICK_DIVISOR: i64 = 10;
const NETHER_ROTATION_MULTIPLIER_A: i32 = 34_187_121;
const NETHER_ROTATION_MULTIPLIER_B: i32 = 121;
const NETHER_ROTATION_SHIFT: u32 = 15;
const NETHER_ROTATION_MASK: i32 = 15;

/// One decoration drawn on a map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapDecoration {
    /// Vanilla `MapDecorationType`.
    pub kind: MapDecorationTypeRef,
    /// Pixel X in `-128..=127`.
    pub x: i8,
    /// Pixel Z in `-128..=127`.
    pub z: i8,
    /// Rotation in units of `360 / 16` degrees, stored as vanilla's signed byte.
    pub rotation: i8,
    /// Custom display name, stored as plain text (vanilla keeps a `Text`).
    pub name: Option<String>,
}

impl MapDecoration {
    /// Creates a decoration and applies vanilla's rotation masking.
    #[must_use]
    pub const fn new(
        kind: MapDecorationTypeRef,
        x: i8,
        z: i8,
        rotation: i8,
        name: Option<String>,
    ) -> Self {
        Self {
            kind,
            x,
            z,
            rotation,
            name,
        }
    }

    /// Converts the decoration to its network representation.
    #[must_use]
    pub fn to_packet_entry(&self) -> MapDecorationEntry {
        let type_id = REGISTRY
            .map_decoration_types
            .id_from_key(&self.kind.key)
            .and_then(|id| i32::try_from(id).ok())
            .unwrap_or(0);
        let mut entry = MapDecorationEntry::new(type_id, self.x, self.z, self.rotation);
        entry.name = self
            .name
            .as_ref()
            .map(|name| text_components::TextComponent::plain(name.clone()));
        entry
    }
}

/// Vanilla `MapState.PlayerUpdateTracker`.
#[derive(Debug, Clone)]
struct PlayerUpdateTracker {
    dirty: bool,
    start_x: u8,
    start_z: u8,
    end_x: u8,
    end_z: u8,
    decorations_dirty: bool,
    empty_packets_requested: u32,
    update_tick: u32,
}

impl PlayerUpdateTracker {
    const fn new() -> Self {
        Self {
            dirty: true,
            start_x: 0,
            start_z: 0,
            end_x: (MAP_SIZE - 1) as u8,
            end_z: (MAP_SIZE - 1) as u8,
            decorations_dirty: true,
            empty_packets_requested: 0,
            update_tick: 0,
        }
    }

    /// Vanilla `markDirty(int, int)`: widens the pending rectangle while the
    /// tracker is already dirty, otherwise starts a new one.
    fn mark_dirty(&mut self, x: u8, z: u8) {
        if self.dirty {
            self.start_x = self.start_x.min(x);
            self.start_z = self.start_z.min(z);
            self.end_x = self.end_x.max(x);
            self.end_z = self.end_z.max(z);
        } else {
            self.dirty = true;
            self.start_x = x;
            self.start_z = z;
            self.end_x = x;
            self.end_z = z;
        }
    }
}

/// Snapped center of a newly created map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapBounds {
    /// Snapped center X.
    pub center_x: i32,
    /// Snapped center Z.
    pub center_z: i32,
}

/// Server-side state of one map id.
#[derive(Debug, Clone)]
pub struct MapState {
    id: i32,
    center_x: i32,
    center_z: i32,
    dimension: Identifier,
    scale: u8,
    colors: Box<[u8; MAP_AREA]>,
    show_decorations: bool,
    unlimited_tracking: bool,
    locked: bool,
    /// Decorations in insertion order, matching vanilla's `LinkedHashMap`.
    decorations: Vec<(String, MapDecoration)>,
    decoration_count: usize,
    trackers: FxHashMap<Uuid, PlayerUpdateTracker>,
    /// Insertion order of tracked players, so decoration sync is deterministic.
    tracker_order: Vec<Uuid>,
    dirty: bool,
}

impl MapState {
    /// Vanilla `MapState.of(double, double, byte, boolean, boolean, RegistryKey)`.
    ///
    /// The center snaps to the grid implied by the scale so maps created close
    /// to each other share their pixel grid.
    #[must_use]
    pub fn of(
        center_x: f64,
        center_z: f64,
        scale: u8,
        show_decorations: bool,
        unlimited_tracking: bool,
        dimension: Identifier,
    ) -> (Self, MapBounds) {
        let scale = scale.min(MAX_SCALE);
        let block_size = MAP_SIZE as i32 * (1 << scale);
        let grid_x = ((center_x + 64.0) / f64::from(block_size)).floor() as i32;
        let grid_z = ((center_z + 64.0) / f64::from(block_size)).floor() as i32;
        let snapped_x = grid_x * block_size + block_size / 2 - MAP_SIZE_HALF;
        let snapped_z = grid_z * block_size + block_size / 2 - MAP_SIZE_HALF;
        let state = Self::new(
            0,
            snapped_x,
            snapped_z,
            scale,
            show_decorations,
            unlimited_tracking,
            false,
            dimension,
        );
        (
            state,
            MapBounds {
                center_x: snapped_x,
                center_z: snapped_z,
            },
        )
    }

    /// Vanilla `MapState.of(byte, boolean, RegistryKey)` used by exploration maps.
    #[must_use]
    pub fn empty(scale: u8, locked: bool, dimension: Identifier) -> Self {
        Self::new(
            0,
            0,
            0,
            scale.min(MAX_SCALE),
            false,
            false,
            locked,
            dimension,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the vanilla MapState constructor"
    )]
    fn new(
        id: i32,
        center_x: i32,
        center_z: i32,
        scale: u8,
        show_decorations: bool,
        unlimited_tracking: bool,
        locked: bool,
        dimension: Identifier,
    ) -> Self {
        Self {
            id,
            center_x,
            center_z,
            dimension,
            scale,
            colors: Box::new([0; MAP_AREA]),
            show_decorations,
            unlimited_tracking,
            locked,
            decorations: Vec::new(),
            decoration_count: 0,
            trackers: FxHashMap::default(),
            tracker_order: Vec::new(),
            dirty: false,
        }
    }

    /// Map id this state is stored under.
    #[must_use]
    pub const fn id(&self) -> i32 {
        self.id
    }

    /// Assigns the saved-data id. Called once when the state is registered.
    pub(crate) const fn set_id(&mut self, id: i32) {
        self.id = id;
    }

    /// Snapped center X.
    #[must_use]
    pub const fn center_x(&self) -> i32 {
        self.center_x
    }

    /// Snapped center Z.
    #[must_use]
    pub const fn center_z(&self) -> i32 {
        self.center_z
    }

    /// Dimension (`domain:world` key) this map was created in.
    #[must_use]
    pub const fn dimension(&self) -> &Identifier {
        &self.dimension
    }

    /// Map scale in `0..=4`.
    #[must_use]
    pub const fn scale(&self) -> u8 {
        self.scale
    }

    /// Whether the map is locked (cartography table copy).
    #[must_use]
    pub const fn locked(&self) -> bool {
        self.locked
    }

    /// Whether player markers are tracked.
    #[must_use]
    pub const fn show_decorations(&self) -> bool {
        self.show_decorations
    }

    /// Whether markers outside the map bounds are still drawn.
    #[must_use]
    pub const fn unlimited_tracking(&self) -> bool {
        self.unlimited_tracking
    }

    /// Raw color grid.
    #[must_use]
    pub fn colors(&self) -> &[u8; MAP_AREA] {
        &self.colors
    }

    /// Decorations in insertion order.
    pub fn decorations(&self) -> impl Iterator<Item = (&str, &MapDecoration)> {
        self.decorations
            .iter()
            .map(|(key, decoration)| (key.as_str(), decoration))
    }

    /// Number of decorations that vanilla counts towards `MAX_DECORATIONS`.
    #[must_use]
    pub const fn decoration_count(&self) -> usize {
        self.decoration_count
    }

    /// Whether the state has unsaved changes.
    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Vanilla `MapState.copy`: a locked, independent duplicate of this map.
    #[must_use]
    pub fn copy(&self) -> Self {
        let mut copy = Self::new(
            0,
            self.center_x,
            self.center_z,
            self.scale,
            self.show_decorations,
            self.unlimited_tracking,
            true,
            self.dimension.clone(),
        );
        copy.colors.copy_from_slice(&self.colors[..]);
        copy.decorations.clone_from(&self.decorations);
        copy.decoration_count = self.decoration_count;
        copy
    }

    /// Vanilla `MapState.zoomOut`: a fresh empty map one scale step coarser.
    #[must_use]
    pub fn zoom_out(&self) -> Self {
        Self::new(
            0,
            self.center_x,
            self.center_z,
            (self.scale + 1).min(MAX_SCALE),
            self.show_decorations,
            self.unlimited_tracking,
            false,
            self.dimension.clone(),
        )
    }

    /// Vanilla `MapState.putColor(int, int, byte)`.
    ///
    /// Returns whether the color actually changed; the renderer folds this into
    /// one `needsUpdate` flag per column.
    pub fn put_color(&mut self, x: usize, z: usize, color: u8) -> bool {
        let index = x + z * MAP_SIZE;
        if self.colors[index] == color {
            return false;
        }
        self.colors[index] = color;
        self.mark_dirty(x as u8, z as u8);
        self.dirty = true;
        true
    }

    /// Vanilla `MapState.setColor(int, int, byte)`.
    pub fn set_color(&mut self, x: usize, z: usize, color: u8) {
        self.colors[x + z * MAP_SIZE] = color;
        self.mark_dirty(x as u8, z as u8);
        self.dirty = true;
    }

    /// Marks the state for the next save.
    pub const fn mark_saved_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clears the saved-data dirty flag after a successful write.
    pub const fn clear_saved_dirty(&mut self) {
        self.dirty = false;
    }

    fn mark_dirty(&mut self, x: u8, z: u8) {
        for tracker in self.trackers.values_mut() {
            tracker.mark_dirty(x, z);
        }
    }

    fn mark_decorations_dirty(&mut self) {
        self.dirty = true;
        for tracker in self.trackers.values_mut() {
            tracker.decorations_dirty = true;
        }
    }

    /// Returns the tracker for a player, creating it on first use.
    fn player_sync_data(&mut self, uuid: Uuid) -> &mut PlayerUpdateTracker {
        if let Entry::Vacant(e) = self.trackers.entry(uuid) {
            e.insert(PlayerUpdateTracker::new());
            self.tracker_order.push(uuid);
        }
        self.trackers
            .get_mut(&uuid)
            .expect("tracker was just inserted")
    }

    /// Increments the player's color-update tick counter and returns it.
    pub fn advance_update_tick(&mut self, uuid: Uuid) -> u32 {
        let tracker = self.player_sync_data(uuid);
        tracker.update_tick = tracker.update_tick.wrapping_add(1);
        tracker.update_tick
    }

    /// Removes a player's tracker.
    pub fn remove_tracker(&mut self, uuid: Uuid) {
        if self.trackers.remove(&uuid).is_some() {
            self.tracker_order.retain(|tracked| *tracked != uuid);
        }
    }

    /// Every tracked player UUID in insertion order.
    #[must_use]
    pub fn tracked_players(&self) -> &[Uuid] {
        &self.tracker_order
    }

    /// Whether a player currently has a tracker.
    #[must_use]
    pub fn is_tracked(&self, uuid: Uuid) -> bool {
        self.trackers.contains_key(&uuid)
    }

    /// Builds the next packet for a tracked player, consuming the dirty
    /// rectangle and (every fifth request) the decoration set.
    ///
    /// Vanilla `MapState.PlayerUpdateTracker#getPacket`.
    pub fn take_packet(&mut self, uuid: Uuid) -> Option<CMapItemData> {
        let Self {
            id,
            scale,
            locked,
            colors,
            decorations,
            trackers,
            ..
        } = self;
        let tracker = trackers.get_mut(&uuid)?;

        let patch = if tracker.dirty {
            tracker.dirty = false;
            let width = tracker.end_x + 1 - tracker.start_x;
            let height = tracker.end_z + 1 - tracker.start_z;
            let mut patch_colors = Vec::with_capacity(usize::from(width) * usize::from(height));
            for row in 0..height {
                for column in 0..width {
                    let x = usize::from(tracker.start_x + column);
                    let z = usize::from(tracker.start_z + row);
                    patch_colors.push(colors[x + z * MAP_SIZE]);
                }
            }
            Some(MapPatch {
                width,
                height,
                start_x: tracker.start_x,
                start_z: tracker.start_z,
                colors: patch_colors,
            })
        } else {
            None
        };

        // Vanilla `emptyPacketsRequested++` only runs while the decorations are
        // dirty; the counter throttles icon resends to every fifth request.
        let decoration_list = if tracker.decorations_dirty {
            let should_send = tracker.empty_packets_requested % DECORATION_SYNC_INTERVAL == 0;
            tracker.empty_packets_requested = tracker.empty_packets_requested.wrapping_add(1);
            if should_send {
                tracker.decorations_dirty = false;
                Some(
                    decorations
                        .iter()
                        .map(|(_, decoration)| decoration.to_packet_entry())
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            }
        } else {
            None
        };

        if decoration_list.is_none() && patch.is_none() {
            return None;
        }

        Some(CMapItemData::new(
            *id,
            *scale as i8,
            *locked,
            decoration_list,
            OptionalMapPatch::new(patch),
        ))
    }

    /// Vanilla `MapState.update(PlayerEntity, ItemStack)` without the item-frame
    /// and banner branches.
    ///
    /// Refreshes every tracked player's marker and drops trackers whose player
    /// left, died or no longer carries the map. Markers are only drawn while the
    /// map and the tracked player share a dimension, like vanilla; the
    /// `#map_invisibility_equipment` rule is not ported because Steel has no such
    /// item tag yet.
    pub fn update(&mut self, world: &World, player: &Player) {
        let uuid = player.uuid();
        let key = player.gameprofile.name().to_owned();
        let holds_map = player_holds_map(player, self.id);
        self.player_sync_data(uuid);
        if !holds_map {
            self.remove_decoration(&key);
        }

        let game_time = world.game_time();
        // Vanilla only draws markers for players that share the map's dimension.
        let same_dimension = world.key == self.dimension;
        let tracked = self.tracker_order.clone();
        for tracked_uuid in tracked {
            let Some(tracked_player) = world.players.get_by_uuid(&tracked_uuid) else {
                self.remove_tracker(tracked_uuid);
                continue;
            };
            let tracked_key = tracked_player.gameprofile.name().to_owned();
            let still_holds = if tracked_uuid == uuid {
                holds_map
            } else {
                player_holds_map(tracked_player.as_ref(), self.id)
            };
            if tracked_player.is_removed() || !still_holds {
                self.remove_tracker(tracked_uuid);
                self.remove_decoration(&tracked_key);
                continue;
            }
            if self.show_decorations && same_dimension {
                let position = tracked_player.position();
                let yaw = tracked_player.rotation().0;
                self.add_decoration(
                    &vanilla_map_decoration_types::PLAYER,
                    &tracked_key,
                    position.x,
                    position.z,
                    f64::from(yaw),
                    None,
                    game_time,
                );
            }
        }

        if holds_map {
            self.add_component_decorations(player);
        }
    }

    /// Merges decorations carried by the `map_decorations` item component.
    ///
    /// Vanilla does this inside `MapState.update`; exploration maps receive
    /// their target markers through this component.
    fn add_component_decorations(&mut self, player: &Player) {
        let decorations = {
            let inventory = player.inventory.lock();
            inventory.get_items().iter().find_map(|stack| {
                let id = stack.get(MAP_ID)?;
                if id.id() != self.id {
                    return None;
                }
                stack.get(MAP_DECORATIONS).cloned()
            })
        };
        let Some(decorations) = decorations else {
            return;
        };
        self.merge_component_decorations(&decorations);
    }

    /// Merges every not-yet-present entry of a `map_decorations` component.
    pub fn merge_component_decorations(&mut self, decorations: &MapDecorations) {
        let mut pending = Vec::new();
        for (key, entry) in decorations.decorations() {
            if self.decorations.iter().any(|(id, _)| id == key) {
                continue;
            }
            pending.push((
                key.clone(),
                entry.decoration_type().value(),
                entry.x(),
                entry.z(),
                f64::from(entry.rotation()),
            ));
        }
        for (key, kind, x, z, rotation) in pending {
            self.add_decoration(kind, &key, x, z, rotation, None, 0);
        }
    }

    /// Vanilla `MapState.addDecoration` including marker resolution.
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the vanilla signature (type, world, key, x, z, rotation, text)"
    )]
    pub fn add_decoration(
        &mut self,
        kind: MapDecorationTypeRef,
        key: &str,
        x: f64,
        z: f64,
        rotation: f64,
        name: Option<String>,
        game_time: i64,
    ) {
        let scale_factor = f64::from(1_u32 << self.scale);
        let dx = (x - f64::from(self.center_x)) / scale_factor;
        let dz = (z - f64::from(self.center_z)) / scale_factor;
        let resolved = self.resolve_marker(kind, dx as f32, dz as f32, rotation, game_time);
        let Some((kind, marker_x, marker_z, marker_rotation)) = resolved else {
            self.remove_decoration(key);
            return;
        };
        let decoration = MapDecoration::new(kind, marker_x, marker_z, marker_rotation, name);

        if let Some(index) = self.decorations.iter().position(|(id, _)| id == key) {
            if self.decorations[index].1 == decoration {
                return;
            }
            if self.decorations[index].1.kind.track_count {
                self.decoration_count = self.decoration_count.saturating_sub(1);
            }
            self.decorations[index].1 = decoration;
            if self.decorations[index].1.kind.track_count {
                self.decoration_count += 1;
            }
        } else {
            if decoration.kind.track_count {
                self.decoration_count += 1;
            }
            self.decorations.push((key.to_owned(), decoration));
        }

        self.mark_decorations_dirty();
    }

    /// Vanilla `MapState.getMarker`.
    fn resolve_marker(
        &self,
        kind: MapDecorationTypeRef,
        dx: f32,
        dz: f32,
        rotation: f64,
        game_time: i64,
    ) -> Option<(MapDecorationTypeRef, i8, i8, i8)> {
        let marker_x = offset_to_marker(dx);
        let marker_z = offset_to_marker(dz);

        if kind.key == vanilla_map_decoration_types::PLAYER.key {
            if is_in_bounds(dx, dz) {
                return Some((
                    kind,
                    marker_x,
                    marker_z,
                    self.player_marker_rotation(rotation, game_time),
                ));
            }
            let off_map = self.player_marker(dx, dz)?;
            return Some((off_map, marker_x, marker_z, 0));
        }

        if is_in_bounds(dx, dz) || self.unlimited_tracking {
            Some((
                kind,
                marker_x,
                marker_z,
                self.player_marker_rotation(rotation, game_time),
            ))
        } else {
            None
        }
    }

    /// Vanilla `MapState.getPlayerMarker`.
    fn player_marker(&self, dx: f32, dz: f32) -> Option<MapDecorationTypeRef> {
        let is_nearby = dx.abs() < PLAYER_OFF_MAP_RADIUS && dz.abs() < PLAYER_OFF_MAP_RADIUS;
        if is_nearby {
            return Some(&vanilla_map_decoration_types::PLAYER_OFF_MAP);
        }
        self.unlimited_tracking
            .then_some(&vanilla_map_decoration_types::PLAYER_OFF_LIMITS)
    }

    /// Vanilla `MapState.getPlayerMarkerRotation`.
    fn player_marker_rotation(&self, rotation: f64, game_time: i64) -> i8 {
        if is_nether_dimension(&self.dimension) {
            // Vanilla wraps 32-bit overflow on purpose here.
            let tick = (game_time / NETHER_ROTATION_TICK_DIVISOR) as i32;
            let squared = tick.wrapping_mul(tick);
            let hashed = squared
                .wrapping_mul(NETHER_ROTATION_MULTIPLIER_A)
                .wrapping_add(tick.wrapping_mul(NETHER_ROTATION_MULTIPLIER_B));
            return (((hashed >> NETHER_ROTATION_SHIFT) & NETHER_ROTATION_MASK) & 0xFF) as u8 as i8;
        }

        // Vanilla `MapItemSavedData`: `(byte) MathHelper.floor(rotation * 16.0 / 360.0 + 0.5)`.
        (rotation * 16.0 / 360.0 + 0.5).floor() as i8
    }

    /// Vanilla `MapState.removeDecoration(String)`.
    pub fn remove_decoration(&mut self, key: &str) {
        let Some(index) = self.decorations.iter().position(|(id, _)| id == key) else {
            return;
        };
        let (_, removed) = self.decorations.remove(index);
        if removed.kind.track_count {
            self.decoration_count = self.decoration_count.saturating_sub(1);
        }
        self.mark_decorations_dirty();
    }

    /// Replaces the whole decoration set, matching vanilla `replaceDecorations`.
    pub fn replace_decorations(&mut self, decorations: Vec<MapDecoration>) {
        self.decorations.clear();
        self.decoration_count = 0;
        for (index, decoration) in decorations.into_iter().enumerate() {
            if decoration.kind.track_count {
                self.decoration_count += 1;
            }
            self.decorations.push((format!("icon-{index}"), decoration));
        }
        self.mark_decorations_dirty();
    }

    /// Vanilla `MapState.decorationCountNotLessThan`.
    #[must_use]
    pub const fn decoration_count_not_less_than(&self, count: usize) -> bool {
        self.decoration_count >= count
    }

    /// Converts this state into its persisted representation.
    #[must_use]
    pub fn to_persistent(&self) -> PersistentMapState {
        PersistentMapState {
            id: self.id,
            dimension: self.dimension.clone(),
            center_x: self.center_x,
            center_z: self.center_z,
            scale: self.scale,
            colors: self.colors.to_vec(),
            show_decorations: self.show_decorations,
            unlimited_tracking: self.unlimited_tracking,
            locked: self.locked,
            decorations: self
                .decorations
                .iter()
                .map(|(key, decoration)| PersistentDecoration {
                    key: key.clone(),
                    kind: decoration.kind.key.clone(),
                    x: decoration.x,
                    z: decoration.z,
                    rotation: decoration.rotation,
                    name: decoration.name.clone(),
                })
                .collect(),
        }
    }

    /// Restores a state from its persisted representation.
    ///
    /// Unknown decoration types are skipped with a warning, and a color array of
    /// the wrong length falls back to an empty map instead of failing the load.
    #[must_use]
    pub fn from_persistent(saved: PersistentMapState) -> Self {
        let mut state = Self::new(
            saved.id,
            saved.center_x,
            saved.center_z,
            saved.scale.min(MAX_SCALE),
            saved.show_decorations,
            saved.unlimited_tracking,
            saved.locked,
            saved.dimension,
        );
        if saved.colors.len() == MAP_AREA {
            state.colors.copy_from_slice(&saved.colors);
        } else {
            log::warn!(
                "Map {} has {} color bytes instead of {}; resetting",
                saved.id,
                saved.colors.len(),
                MAP_AREA
            );
        }
        for decoration in saved.decorations {
            let Some(kind) = REGISTRY.map_decoration_types.by_key(&decoration.kind) else {
                log::warn!(
                    "Map {} references unknown decoration type {}; skipping",
                    saved.id,
                    decoration.kind
                );
                continue;
            };
            let restored = MapDecoration::new(
                kind,
                decoration.x,
                decoration.z,
                decoration.rotation,
                decoration.name,
            );
            if restored.kind.track_count {
                state.decoration_count += 1;
            }
            state.decorations.push((decoration.key, restored));
        }
        state
    }
}

/// Vanilla `MapState.isInBounds`.
fn is_in_bounds(dx: f32, dz: f32) -> bool {
    dx >= -MARKER_HALF_SIZE
        && dz >= -MARKER_HALF_SIZE
        && dx <= MARKER_HALF_SIZE
        && dz <= MARKER_HALF_SIZE
}

/// Vanilla `MapState.offsetToMarkerPosition`.
fn offset_to_marker(offset: f32) -> i8 {
    if offset <= -MARKER_HALF_SIZE {
        return -128;
    }
    if offset >= MARKER_HALF_SIZE {
        return MARKER_MAX;
    }
    (offset * 2.0 + 0.5) as i8
}

/// Vanilla compares against `World.NETHER`; Steel world keys are domain-scoped.
fn is_nether_dimension(dimension: &Identifier) -> bool {
    dimension.path.as_ref() == "the_nether"
}

/// Whether the player's inventory still contains this map.
fn player_holds_map(player: &Player, map_id: i32) -> bool {
    let inventory = player.inventory.lock();
    inventory
        .get_items()
        .iter()
        .any(|stack| stack.get(MAP_ID).is_some_and(|id| id.id() == map_id))
}

/// All maps of one loaded world.
#[derive(Debug, Default)]
pub struct MapStates {
    next_id: i32,
    states: FxHashMap<i32, MapState>,
    dirty: bool,
}

impl MapStates {
    /// Creates an empty container.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Looks up a map by id.
    #[must_use]
    pub fn get(&self, id: i32) -> Option<&MapState> {
        self.states.get(&id)
    }

    /// Looks up a map mutably by id.
    pub fn get_mut(&mut self, id: i32) -> Option<&mut MapState> {
        self.states.get_mut(&id)
    }

    /// Runs `f` with mutable access to a map, returning `None` when unknown.
    pub fn with_state<R>(&mut self, id: i32, f: impl FnOnce(&mut MapState) -> R) -> Option<R> {
        let state = self.states.get_mut(&id)?;
        let result = f(state);
        self.dirty = true;
        Some(result)
    }

    /// Vanilla `ServerWorld.increaseAndGetMapId`.
    pub const fn allocate_id(&mut self) -> i32 {
        self.next_id += 1;
        self.dirty = true;
        self.next_id
    }

    /// Registers a new map and returns its id.
    pub fn insert(&mut self, mut state: MapState) -> i32 {
        let id = self.allocate_id();
        state.set_id(id);
        self.states.insert(id, state);
        id
    }

    /// Number of stored maps.
    #[must_use]
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Whether no maps are stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Whether anything changed since the last save.
    ///
    /// Maps are mutated through [`Self::get_mut`], so the container also has to
    /// look at the per-map flags that `put_color`, `set_color` and the
    /// decoration methods raise.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty || self.states.values().any(MapState::is_dirty)
    }

    /// Clears the dirty flags after a successful save.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
        for state in self.states.values_mut() {
            state.clear_saved_dirty();
        }
    }

    /// Next id that will be handed out.
    #[must_use]
    pub const fn next_id(&self) -> i32 {
        self.next_id
    }

    /// Converts every map into its persisted representation.
    #[must_use]
    pub fn to_persistent(&self) -> PersistentMapStates {
        let mut maps = self
            .states
            .values()
            .map(MapState::to_persistent)
            .collect::<Vec<_>>();
        maps.sort_by_key(|map| map.id);
        PersistentMapStates {
            next_id: self.next_id,
            maps,
        }
    }

    /// Restores every persisted map.
    #[must_use]
    pub fn from_persistent(persistent: PersistentMapStates) -> Self {
        let mut states =
            FxHashMap::with_capacity_and_hasher(persistent.maps.len(), rustc_hash::FxBuildHasher);
        for saved in persistent.maps {
            let id = saved.id;
            states.insert(id, MapState::from_persistent(saved));
        }
        Self {
            next_id: persistent.next_id,
            states,
            dirty: false,
        }
    }
}

/// Persisted form of one map.
#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
pub struct PersistentMapState {
    /// Saved-data id.
    pub id: i32,
    /// Dimension the map belongs to.
    pub dimension: Identifier,
    /// Snapped center X.
    pub center_x: i32,
    /// Snapped center Z.
    pub center_z: i32,
    /// Scale in `0..=4`.
    pub scale: u8,
    /// Exactly [`MAP_AREA`] color bytes.
    pub colors: Vec<u8>,
    /// Whether player markers are tracked.
    pub show_decorations: bool,
    /// Whether off-map markers are drawn.
    pub unlimited_tracking: bool,
    /// Whether the map is locked.
    pub locked: bool,
    /// Decorations in insertion order.
    pub decorations: Vec<PersistentDecoration>,
}

/// Persisted form of one decoration.
#[derive(Debug, Clone, SchemaWrite, SchemaRead)]
pub struct PersistentDecoration {
    /// Decoration key (player name, component id, ...).
    pub key: String,
    /// Decoration type key.
    pub kind: Identifier,
    /// Pixel X.
    pub x: i8,
    /// Pixel Z.
    pub z: i8,
    /// Masked rotation.
    pub rotation: i8,
    /// Optional plain-text name.
    pub name: Option<String>,
}

/// Persisted form of all maps of one world.
#[derive(Debug, Clone, Default, SchemaWrite, SchemaRead)]
pub struct PersistentMapStates {
    /// Next id to hand out.
    pub next_id: i32,
    /// Stored maps.
    pub maps: Vec<PersistentMapState>,
}

#[cfg(test)]
mod tests {
    use super::{
        MAP_AREA, MAP_SIZE, MAX_SCALE, MapDecoration, MapState, MapStates, PersistentDecoration,
        PersistentMapStates, is_nether_dimension,
    };
    use steel_registry::{init_vanilla_registry, vanilla_map_decoration_types};
    use steel_utils::Identifier;
    use uuid::Uuid;

    fn overworld() -> Identifier {
        Identifier::new_static("steel", "overworld")
    }

    fn state(scale: u8) -> MapState {
        let (state, _) = MapState::of(0.0, 0.0, scale, true, false, overworld());
        state
    }

    #[test]
    fn center_snaps_to_the_scale_grid() {
        // Scale 0: 128-block grid, so the center lands on multiples of 128
        // shifted by 64 (vanilla `MapState.of`).
        let (state, bounds) = MapState::of(0.0, 0.0, 0, true, false, overworld());
        assert_eq!((bounds.center_x, bounds.center_z), (0, 0));
        assert_eq!(state.scale(), 0);

        let (_, bounds) = MapState::of(200.0, -200.0, 0, true, false, overworld());
        // Vanilla `MapItemSavedData.createFresh`: `blockSize = 128 << scale`,
        // `grid = floor((coord + 64) / blockSize)`, snap = `grid * blockSize`.
        assert_eq!((bounds.center_x, bounds.center_z), (256, -256));

        // Scale 1: 256-block grid.
        let (state, bounds) = MapState::of(200.0, 200.0, 1, true, false, overworld());
        assert_eq!((bounds.center_x, bounds.center_z), (320, 320));
        assert_eq!(state.scale(), 1);

        // Scale is clamped to vanilla's maximum.
        let (state, _) = MapState::of(0.0, 0.0, 9, true, false, overworld());
        assert_eq!(state.scale(), MAX_SCALE);
    }

    #[test]
    fn zoom_out_keeps_center_and_clears_pixels() {
        let mut map = state(0);
        map.set_color(1, 1, 0x2A);
        let zoomed = map.zoom_out();
        assert_eq!(zoomed.scale(), 1);
        assert_eq!(zoomed.center_x(), map.center_x());
        assert_eq!(zoomed.center_z(), map.center_z());
        assert_eq!(zoomed.colors()[1 + MAP_AREA / MAP_SIZE], 0);
        assert!(!zoomed.locked());

        let mut locked = state(MAX_SCALE);
        locked.mark_saved_dirty();
        assert_eq!(locked.zoom_out().scale(), MAX_SCALE, "scale saturates");
    }

    #[test]
    fn copy_locks_the_map_and_keeps_an_independent_grid() {
        let mut state = state(0);
        state.set_color(4, 4, 0x2A);
        let mut copy = state.copy();
        assert!(copy.locked());
        assert_eq!(copy.colors()[4 + 4 * MAP_SIZE], 0x2A);

        copy.set_color(4, 4, 0x0B);
        assert_eq!(state.colors()[4 + 4 * MAP_SIZE], 0x2A);
    }

    #[test]
    fn put_color_only_reports_real_changes() {
        let mut state = state(0);
        assert!(state.put_color(10, 20, 0x2A));
        assert!(!state.put_color(10, 20, 0x2A));
        assert!(state.put_color(10, 20, 0x0B));
    }

    #[test]
    fn delta_patch_covers_the_dirty_rectangle() {
        let mut state = state(0);
        let uuid = Uuid::from_u128(7);
        state.player_sync_data(uuid);
        // Drain the initial whole-map patch.
        assert!(state.take_packet(uuid).is_some());
        assert!(state.take_packet(uuid).is_none());

        state.put_color(3, 4, 0x2A);
        state.put_color(10, 20, 0x0B);

        let packet = state.take_packet(uuid).expect("dirty patch");
        let patch = packet.patch.patch().expect("patch present");
        assert_eq!((patch.start_x, patch.start_z), (3, 4));
        assert_eq!((patch.width, patch.height), (8, 17));
        assert_eq!(patch.colors.len(), 8 * 17);
        // Row-major patch data.
        assert_eq!(patch.colors[0], 0x2A);
        assert_eq!(patch.colors[7 + 16 * 8], 0x0B);
        assert!(packet.decorations.is_none());
    }

    #[test]
    fn decoration_sync_is_throttled_to_every_fifth_request() {
        init_vanilla_registry();
        let mut state = state(0);
        let uuid = Uuid::from_u128(1);
        state.player_sync_data(uuid);

        let first = state.take_packet(uuid).expect("initial packet");
        assert!(first.decorations.is_some(), "decorations start dirty");
        assert!(state.take_packet(uuid).is_none());

        state.add_decoration(
            &vanilla_map_decoration_types::RED_MARKER,
            "marker",
            0.0,
            0.0,
            0.0,
            None,
            0,
        );
        // counter is 1 after the first request: the next four requests skip the
        // decoration list, the fifth sends it.
        for skipped in 0..4 {
            assert!(
                state.take_packet(uuid).is_none(),
                "request {skipped} should not send decorations"
            );
        }
        let sent = state.take_packet(uuid).expect("decoration refresh");
        assert!(sent.decorations.is_some());
    }

    #[test]
    fn off_map_players_use_the_fallback_markers() {
        init_vanilla_registry();
        let mut state = state(0);
        // 200 blocks east of the center is outside the 63-block drawing area but
        // inside vanilla's 320-block tracking radius.
        state.add_decoration(
            &vanilla_map_decoration_types::PLAYER,
            "p",
            200.0,
            0.0,
            0.0,
            None,
            0,
        );
        let (_, decoration) = state.decorations().next().expect("marker");
        assert_eq!(
            decoration.kind.key,
            vanilla_map_decoration_types::PLAYER_OFF_MAP.key
        );
        assert_eq!(decoration.rotation, 0);

        // Beyond the radius the marker disappears unless tracking is unlimited.
        state.add_decoration(
            &vanilla_map_decoration_types::PLAYER,
            "p",
            400.0,
            0.0,
            0.0,
            None,
            0,
        );
        assert_eq!(state.decorations().count(), 0);
    }

    #[test]
    fn unlimited_tracking_keeps_distant_markers_off_limits() {
        init_vanilla_registry();
        let (mut state, _) = MapState::of(0.0, 0.0, 0, true, true, overworld());
        state.add_decoration(
            &vanilla_map_decoration_types::PLAYER,
            "p",
            400.0,
            0.0,
            0.0,
            None,
            0,
        );
        let (_, decoration) = state.decorations().next().expect("marker");
        assert_eq!(
            decoration.kind.key,
            vanilla_map_decoration_types::PLAYER_OFF_LIMITS.key
        );
    }

    #[test]
    fn on_map_players_use_the_vanilla_rotation_offset() {
        init_vanilla_registry();
        let mut state = state(0);
        state.add_decoration(
            &vanilla_map_decoration_types::PLAYER,
            "p",
            0.0,
            0.0,
            90.0,
            None,
            0,
        );
        let (_, decoration) = state.decorations().next().expect("marker");
        // Vanilla adds 8 degrees before scaling: (90 + 8) * 16 / 360 = 4.35 -> 4.
        assert_eq!(decoration.rotation, 4);

        state.add_decoration(
            &vanilla_map_decoration_types::PLAYER,
            "p",
            0.0,
            0.0,
            -90.0,
            None,
            0,
        );
        let (_, decoration) = state.decorations().next().expect("marker");
        assert_eq!(decoration.rotation, -4);
    }

    #[test]
    fn nether_markers_rotate_from_hashed_game_time() {
        init_vanilla_registry();
        let nether = Identifier::new_static("steel", "the_nether");
        assert!(is_nether_dimension(&nether));
        let (mut state, _) = MapState::of(0.0, 0.0, 0, true, false, nether);
        // game_time 100 -> tick 10 -> wrapping hash -876253986 -> bits 10.
        state.add_decoration(
            &vanilla_map_decoration_types::PLAYER,
            "p",
            0.0,
            0.0,
            0.0,
            None,
            100,
        );
        let (_, decoration) = state.decorations().next().expect("marker");
        assert_eq!(decoration.rotation, 10);
    }

    #[test]
    fn decoration_count_only_tracks_counted_types() {
        init_vanilla_registry();
        let mut state = state(0);
        state.add_decoration(
            &vanilla_map_decoration_types::PLAYER,
            "a",
            0.0,
            0.0,
            0.0,
            None,
            0,
        );
        assert_eq!(state.decoration_count(), 1);
        // Re-adding the same key replaces instead of duplicating.
        state.add_decoration(
            &vanilla_map_decoration_types::PLAYER,
            "a",
            10.0,
            0.0,
            0.0,
            None,
            0,
        );
        assert_eq!(state.decoration_count(), 1);
        assert_eq!(state.decorations().count(), 1);

        state.remove_decoration("a");
        assert_eq!(state.decoration_count(), 0);
        assert_eq!(state.decorations().count(), 0);
    }

    #[test]
    fn replace_decorations_renumbers_keys() {
        init_vanilla_registry();
        let mut state = state(0);
        state.replace_decorations(vec![
            MapDecoration::new(&vanilla_map_decoration_types::RED_MARKER, 1, 2, 0, None),
            MapDecoration::new(&vanilla_map_decoration_types::BLUE_MARKER, 3, 4, 0, None),
        ]);
        let keys = state
            .decorations()
            .map(|(key, _)| key.to_owned())
            .collect::<Vec<_>>();
        assert_eq!(keys, ["icon-0", "icon-1"]);
        assert_eq!(state.decoration_count(), 2);
    }

    #[test]
    fn persisted_state_round_trips() {
        init_vanilla_registry();
        let mut state = state(1);
        state.set_color(5, 6, 0x2A);
        state.add_decoration(
            &vanilla_map_decoration_types::PLAYER,
            "Tester",
            0.0,
            0.0,
            90.0,
            Some("Tester".to_owned()),
            0,
        );

        let persistent = PersistentMapStates {
            next_id: 12,
            maps: vec![state.to_persistent()],
        };
        let restored = MapStates::from_persistent(persistent);
        assert_eq!(restored.len(), 1);
        assert_eq!(restored.next_id(), 12);

        let restored = restored.get(state.id()).expect("map restored");
        assert_eq!(restored.scale(), 1);
        assert_eq!(restored.center_x(), state.center_x());
        assert_eq!(restored.colors()[5 + 6 * MAP_SIZE], 0x2A);
        assert_eq!(restored.decoration_count(), 1);
        let (key, decoration) = restored.decorations().next().expect("decoration");
        assert_eq!(key, "Tester");
        assert_eq!(decoration.rotation, 4);
        assert_eq!(decoration.name.as_deref(), Some("Tester"));
    }

    #[test]
    fn unknown_decoration_types_are_skipped_on_load() {
        init_vanilla_registry();
        let saved = super::PersistentMapState {
            id: 3,
            dimension: overworld(),
            center_x: 0,
            center_z: 0,
            scale: 0,
            colors: vec![0; MAP_AREA],
            show_decorations: true,
            unlimited_tracking: false,
            locked: false,
            decorations: vec![PersistentDecoration {
                key: "ghost".to_owned(),
                kind: Identifier::new_static("steel", "not_a_decoration"),
                x: 0,
                z: 0,
                rotation: 0,
                name: None,
            }],
        };
        let restored = MapState::from_persistent(saved);
        assert_eq!(restored.decorations().count(), 0);
        assert_eq!(restored.decoration_count(), 0);
    }

    #[test]
    fn ids_are_allocated_monotonically() {
        let mut maps = MapStates::new();
        let first = maps.insert(state(0));
        let second = maps.insert(state(0));
        assert_eq!((first, second), (1, 2));
        assert_eq!(maps.get(1).map(MapState::id), Some(1));
        assert_eq!(maps.get(2).map(MapState::id), Some(2));
        assert_eq!(maps.len(), 2);
    }

    #[test]
    fn player_markers_are_only_drawn_in_the_maps_own_dimension() {
        use std::sync::Arc;

        use steel_registry::data_components::components::MapId;
        use steel_registry::data_components::vanilla_components::MAP_ID;
        use steel_registry::item_stack::ItemStack;
        use steel_registry::vanilla_items;
        use steel_utils::ChunkPos;

        use crate::test_support::{TestPlayerBuilder, fresh_test_world, insert_ready_full_chunk};

        init_vanilla_registry();
        let fixture = fresh_test_world("map_state_dimension_marker");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let player = TestPlayerBuilder::new(Arc::clone(world), "Marker", 1).build();
        assert!(world.players.insert(Arc::clone(&player)));

        let nether = Identifier::new_static("minecraft", "the_nether");
        let (mut state, _) = MapState::of(0.0, 0.0, 0, true, false, nether);
        state.set_id(7);
        let mut stack = ItemStack::new(&vanilla_items::FILLED_MAP);
        stack.set(MAP_ID, MapId::new(7));
        player.inventory.lock().set_selected_item(stack);

        state.update(world, &player);
        assert_eq!(
            state.decorations().count(),
            0,
            "no player marker across dimensions"
        );

        let (mut same_dimension, _) = MapState::of(0.0, 0.0, 0, true, false, world.key.clone());
        same_dimension.set_id(7);
        same_dimension.update(world, &player);
        let markers: Vec<_> = same_dimension.decorations().collect();
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].0, "Marker");
        assert_eq!(markers[0].1.kind.key.path.as_ref(), "player");
    }

    #[test]
    fn map_states_report_color_and_decoration_changes_as_dirty() {
        init_vanilla_registry();
        let mut maps = MapStates::new();
        let id = maps.insert(state(0));
        maps.clear_dirty();
        assert!(!maps.is_dirty());

        maps.get_mut(id).expect("map exists").put_color(1, 2, 0x2A);
        assert!(maps.is_dirty(), "color writes reach the saved-data flag");

        maps.clear_dirty();
        assert!(!maps.is_dirty());

        maps.get_mut(id).expect("map exists").add_decoration(
            &vanilla_map_decoration_types::RED_MARKER,
            "marker",
            0.0,
            0.0,
            0.0,
            None,
            0,
        );
        assert!(
            maps.is_dirty(),
            "decoration writes reach the saved-data flag"
        );
    }
}
