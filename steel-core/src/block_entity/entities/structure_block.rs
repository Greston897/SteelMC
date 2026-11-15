//! Structure block entity.
//!
//! Vanilla `StructureBlockEntity` stores the editor settings for a structure
//! block: the structure name, author, metadata, offset, size, rotation, mirror,
//! mode and the flags that control saving and loading. The settings are written
//! to disk and synced to clients, which is why a marker block entity is not
//! enough — `/setblock` on a structure block writes data that must survive.

use std::sync::{Arc, Weak};

use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::NbtCompound;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, StructureMode};
use steel_registry::vanilla_block_entity_types;
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;

use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::world::World;

/// Vanilla `StructureBlockEntity.MAX_OFFSET_PER_AXIS`.
pub const MAX_OFFSET_PER_AXIS: i32 = 48;
/// Vanilla `StructureBlockEntity.MAX_SIZE_PER_AXIS`.
pub const MAX_SIZE_PER_AXIS: i32 = 48;
/// Vanilla default structure offset.
const DEFAULT_POS: BlockPos = BlockPos::new(0, 1, 0);

/// Mutable editor state of a structure block.
///
/// No `Eq`: the integrity is an `f32`, which has no total equality.
#[derive(Debug, Clone, PartialEq)]
struct StructureBlockState {
    /// Structure name, empty when unset.
    name: String,
    /// Author that last edited the block.
    author: String,
    /// Freeform metadata written into the saved structure.
    metadata: String,
    /// Offset from the block to the structure corner.
    structure_pos: BlockPos,
    /// Size of the structure box.
    structure_size: BlockPos,
    /// Mirror ordinal.
    mirror: i32,
    /// Rotation ordinal.
    rotation: i32,
    /// Current mode, mirrored into the block state.
    mode: StructureMode,
    ignore_entities: bool,
    strict: bool,
    powered: bool,
    show_air: bool,
    show_bounding_box: bool,
    integrity: f32,
    seed: i64,
}

impl StructureBlockState {
    fn new(state: BlockStateId) -> Self {
        let mode = state
            .try_get_value(&BlockStateProperties::STRUCTUREBLOCK_MODE)
            .unwrap_or(StructureMode::Load);
        Self {
            name: String::new(),
            author: String::new(),
            metadata: String::new(),
            structure_pos: DEFAULT_POS,
            structure_size: BlockPos::ZERO,
            mirror: 0,
            rotation: 0,
            mode,
            // Vanilla defaults: ignore entities, not strict, not powered, no
            // air display, bounding box shown, full integrity, seed zero.
            ignore_entities: true,
            strict: false,
            powered: false,
            show_air: false,
            show_bounding_box: true,
            integrity: 1.0,
            seed: 0,
        }
    }
}

/// Vanilla `StructureBlockEntity`.
pub struct StructureBlockEntity {
    base: Arc<BlockEntityBase>,
    state: SyncMutex<StructureBlockState>,
}

// SAFETY: This key is owned by Steel and uniquely identifies this type.
unsafe impl DowncastType for StructureBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/structure_block");
}

impl StructureBlockEntity {
    /// Creates a new structure block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(
                &vanilla_block_entity_types::STRUCTURE_BLOCK,
                level,
                pos,
                state,
            )),
            state: SyncMutex::new(StructureBlockState::new(state)),
        }
    }

    /// Returns the stored structure name, empty when the field is unset.
    #[must_use]
    pub fn structure_name(&self) -> String {
        self.state.lock().name.clone()
    }

    /// Vanilla `StructureBlockEntity.hasStructureName`.
    #[must_use]
    pub fn has_structure_name(&self) -> bool {
        !self.state.lock().name.is_empty()
    }

    /// Vanilla `StructureBlockEntity.setStructureName(String)`.
    pub fn set_structure_name(&self, name: &str) {
        let parsed = name.trim();
        self.state.lock().name = if parsed.is_empty() {
            String::new()
        } else {
            parsed.to_owned()
        };
    }

    /// Vanilla `StructureBlockEntity.createdBy`.
    pub fn created_by(&self, author: &str) {
        self.state.lock().author = author.to_owned();
    }

    /// Vanilla `StructureBlockEntity.getStructurePos`.
    #[must_use]
    pub fn structure_pos(&self) -> BlockPos {
        self.state.lock().structure_pos
    }

    /// Vanilla `StructureBlockEntity.setStructurePos`.
    pub fn set_structure_pos(&self, pos: BlockPos) {
        self.state.lock().structure_pos = clamp_offset_pos(pos);
    }

    /// Vanilla `StructureBlockEntity.getStructureSize`.
    #[must_use]
    pub fn structure_size(&self) -> BlockPos {
        self.state.lock().structure_size
    }

    /// Vanilla `StructureBlockEntity.setStructureSize`.
    pub fn set_structure_size(&self, size: BlockPos) {
        self.state.lock().structure_size = clamp_size_pos(size);
    }

    /// Vanilla `StructureBlockEntity.getMirror`.
    #[must_use]
    pub fn mirror(&self) -> i32 {
        self.state.lock().mirror
    }

    /// Vanilla `StructureBlockEntity.setMirror`.
    pub fn set_mirror(&self, mirror: i32) {
        self.state.lock().mirror = mirror;
    }

    /// Vanilla `StructureBlockEntity.getRotation`.
    #[must_use]
    pub fn rotation(&self) -> i32 {
        self.state.lock().rotation
    }

    /// Vanilla `StructureBlockEntity.setRotation`.
    pub fn set_rotation(&self, rotation: i32) {
        self.state.lock().rotation = rotation;
    }

    /// Vanilla `StructureBlockEntity.getMetaData`.
    #[must_use]
    pub fn metadata(&self) -> String {
        self.state.lock().metadata.clone()
    }

    /// Vanilla `StructureBlockEntity.setMetaData`.
    pub fn set_metadata(&self, metadata: &str) {
        self.state.lock().metadata = metadata.to_owned();
    }

    /// Vanilla `StructureBlockEntity.getMode`.
    #[must_use]
    pub fn mode(&self) -> StructureMode {
        self.state.lock().mode
    }

    /// Vanilla `StructureBlockEntity.isIgnoreEntities`.
    #[must_use]
    pub fn is_ignore_entities(&self) -> bool {
        self.state.lock().ignore_entities
    }

    /// Vanilla `StructureBlockEntity.setIgnoreEntities`.
    pub fn set_ignore_entities(&self, ignore_entities: bool) {
        self.state.lock().ignore_entities = ignore_entities;
    }

    /// Vanilla `StructureBlockEntity.isStrict`.
    #[must_use]
    pub fn is_strict(&self) -> bool {
        self.state.lock().strict
    }

    /// Vanilla `StructureBlockEntity.setStrict`.
    pub fn set_strict(&self, strict: bool) {
        self.state.lock().strict = strict;
    }

    /// Vanilla `StructureBlockEntity.getIntegrity`.
    #[must_use]
    pub fn integrity(&self) -> f32 {
        self.state.lock().integrity
    }

    /// Vanilla `StructureBlockEntity.setIntegrity`.
    pub fn set_integrity(&self, integrity: f32) {
        self.state.lock().integrity = integrity;
    }

    /// Vanilla `StructureBlockEntity.getSeed`.
    #[must_use]
    pub fn seed(&self) -> i64 {
        self.state.lock().seed
    }

    /// Vanilla `StructureBlockEntity.setSeed`.
    pub fn set_seed(&self, seed: i64) {
        self.state.lock().seed = seed;
    }

    /// Vanilla `StructureBlockEntity.isPowered`.
    #[must_use]
    pub fn is_powered(&self) -> bool {
        self.state.lock().powered
    }

    /// Vanilla `StructureBlockEntity.setPowered`.
    pub fn set_powered(&self, powered: bool) {
        self.state.lock().powered = powered;
    }

    /// Vanilla `StructureBlockEntity.setShowAir`.
    pub fn set_show_air(&self, show_air: bool) {
        self.state.lock().show_air = show_air;
    }

    /// Vanilla `StructureBlockEntity.setShowBoundingBox`.
    pub fn set_show_bounding_box(&self, show_bounding_box: bool) {
        self.state.lock().show_bounding_box = show_bounding_box;
    }

    /// Vanilla `StructureBlockEntity.setMode`: stores the mode and mirrors it
    /// into the block state so the client renders the right variant.
    pub fn set_mode(&self, mode: StructureMode) {
        self.state.lock().mode = mode;
        self.update_block_state();
    }

    /// Vanilla `StructureBlockEntity.updateBlockState`.
    fn update_block_state(&self) {
        // Inherent impl, so `level()` has to come from the base directly.
        let Some(world) = self.base.level() else {
            return;
        };
        let pos = self.base.pos();
        let state = world.get_block_state(pos);
        if state.get_block().key != steel_registry::vanilla_blocks::STRUCTURE_BLOCK.key {
            return;
        }
        let mode = self.state.lock().mode;
        let updated = state.set_value(&BlockStateProperties::STRUCTUREBLOCK_MODE, mode);
        if updated != state {
            world.set_block(
                pos,
                updated,
                steel_utils::types::UpdateFlags::UPDATE_CLIENTS,
            );
        }
    }

    /// Vanilla `StructureBlockEntity.usedBy`: only a game-master may open the
    /// editor. The menu itself is client-side, so `true` is the whole server
    /// response.
    #[must_use]
    pub fn used_by(&self, player: &crate::player::Player) -> bool {
        player.has_game_master_permissions()
    }
}

impl BlockEntity for StructureBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let nbt_view: NbtCompoundView<'_, '_> = nbt.into();
        let mut state = self.state.lock();
        state.name = nbt_view
            .string("name")
            .map_or_else(String::new, |name| name.to_str().into_owned());
        state.author = nbt_view
            .string("author")
            .map_or_else(String::new, |author| author.to_str().into_owned());
        state.metadata = nbt_view
            .string("metadata")
            .map_or_else(String::new, |metadata| metadata.to_str().into_owned());
        // Vanilla clamps the stored offset and size into the editor limits.
        state.structure_pos = clamp_offset_pos(BlockPos::new(
            nbt_view.int_value("posX").unwrap_or(DEFAULT_POS.x()),
            nbt_view.int_value("posY").unwrap_or(DEFAULT_POS.y()),
            nbt_view.int_value("posZ").unwrap_or(DEFAULT_POS.z()),
        ));
        state.structure_size = clamp_size_pos(BlockPos::new(
            nbt_view.int_value("sizeX").unwrap_or(0),
            nbt_view.int_value("sizeY").unwrap_or(0),
            nbt_view.int_value("sizeZ").unwrap_or(0),
        ));
        state.rotation = structure_rotation_from_name(
            nbt_view.string("rotation")
                .map_or("none", |value| value.to_str()),
        );
        state.mirror =
            structure_mirror_from_name(nbt_view.string("mirror").map_or("none", |value| value.to_str()));
        state.mode = structure_mode_from_name(
            nbt_view.string("mode")
                .map_or("data", |value| value.to_str()),
        );
        state.ignore_entities = nbt_view
            .byte_value("ignoreEntities")
            .is_none_or(|value| value != 0);
        state.strict = nbt_view.byte_value("strict").is_some_and(|value| value != 0);
        state.powered = nbt_view.byte_value("powered").is_some_and(|value| value != 0);
        state.show_air = nbt_view.byte_value("showair").is_some_and(|value| value != 0);
        state.show_bounding_box = nbt_view
            .byte_value("showboundingbox")
            .is_none_or(|value| value != 0);
        state.integrity = nbt_view.float_value("integrity").unwrap_or(1.0);
        state.seed = nbt_view.long_value("seed").unwrap_or(0);
    }

    /// Vanilla `StructureBlockEntity.getUpdateTag`: `saveCustomOnly`.
    fn get_update_tag(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.save_additional(&mut nbt);
        Some(nbt)
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let state = self.state.lock();
        nbt.insert("name", state.name.clone());
        nbt.insert("author", state.author.clone());
        nbt.insert("metadata", state.metadata.clone());
        nbt.insert("posX", state.structure_pos.x());
        nbt.insert("posY", state.structure_pos.y());
        nbt.insert("posZ", state.structure_pos.z());
        nbt.insert("sizeX", state.structure_size.x());
        nbt.insert("sizeY", state.structure_size.y());
        nbt.insert("sizeZ", state.structure_size.z());
        nbt.insert(
            "rotation",
            structure_rotation_name(state.rotation).to_owned(),
        );
        nbt.insert("mirror", structure_mirror_name(state.mirror).to_owned());
        nbt.insert("mode", structure_mode_name(state.mode).to_owned());
        nbt.insert("ignoreEntities", u8::from(state.ignore_entities));
        nbt.insert("strict", u8::from(state.strict));
        nbt.insert("powered", u8::from(state.powered));
        nbt.insert("showair", u8::from(state.show_air));
        nbt.insert("showboundingbox", u8::from(state.show_bounding_box));
        nbt.insert("integrity", state.integrity);
        nbt.insert("seed", state.seed);
    }
}

/// Vanilla `Mth.clamp` applied per axis to the structure offset.
fn clamp_offset_pos(pos: BlockPos) -> BlockPos {
    BlockPos::new(
        pos.x().clamp(-MAX_OFFSET_PER_AXIS, MAX_OFFSET_PER_AXIS),
        pos.y().clamp(-MAX_OFFSET_PER_AXIS, MAX_OFFSET_PER_AXIS),
        pos.z().clamp(-MAX_OFFSET_PER_AXIS, MAX_OFFSET_PER_AXIS),
    )
}

/// Vanilla `Mth.clamp` applied per axis to the structure size.
fn clamp_size_pos(pos: BlockPos) -> BlockPos {
    BlockPos::new(
        pos.x().clamp(0, MAX_SIZE_PER_AXIS),
        pos.y().clamp(0, MAX_SIZE_PER_AXIS),
        pos.z().clamp(0, MAX_SIZE_PER_AXIS),
    )
}

/// Vanilla `Rotation.LEGACY_CODEC` names, in ordinal order.
const ROTATION_NAMES: [&str; 4] = ["none", "clockwise_90", "180", "counterclockwise_90"];
/// Vanilla `Mirror.LEGACY_CODEC` names, in ordinal order.
const MIRROR_NAMES: [&str; 3] = ["none", "left_right", "front_back"];

const fn structure_rotation_name(rotation: i32) -> &'static str {
    if rotation < 0 || rotation as usize >= ROTATION_NAMES.len() {
        return ROTATION_NAMES[0];
    }
    ROTATION_NAMES[rotation as usize]
}

fn structure_rotation_from_name(name: &str) -> i32 {
    ROTATION_NAMES
        .iter()
        .position(|candidate| *candidate == name)
        .map_or(0, |index| index as i32)
}

const fn structure_mirror_name(mirror: i32) -> &'static str {
    if mirror < 0 || mirror as usize >= MIRROR_NAMES.len() {
        return MIRROR_NAMES[0];
    }
    MIRROR_NAMES[mirror as usize]
}

fn structure_mirror_from_name(name: &str) -> i32 {
    MIRROR_NAMES
        .iter()
        .position(|candidate| *candidate == name)
        .map_or(0, |index| index as i32)
}

const fn structure_mode_name(mode: StructureMode) -> &'static str {
    match mode {
        StructureMode::Save => "save",
        StructureMode::Load => "load",
        StructureMode::Corner => "corner",
        StructureMode::Data => "data",
    }
}

fn structure_mode_from_name(name: &str) -> StructureMode {
    match name {
        "save" => StructureMode::Save,
        "load" => StructureMode::Load,
        "corner" => StructureMode::Corner,
        _ => StructureMode::Data,
    }
}
