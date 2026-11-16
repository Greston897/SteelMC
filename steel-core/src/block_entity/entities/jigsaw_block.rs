//! Jigsaw block entity.
//!
//! Vanilla `JigsawBlockEntity` stores the seven fields the jigsaw editor edits
//! and world generation reads back: name, target, pool, final state, joint type
//! and the two priorities. They live in NBT, so a marker block entity would lose
//! everything a `/setblock` or an editor session writes.

use std::sync::{Arc, Weak};

use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::NbtCompound;
use steel_registry::vanilla_block_entity_types;
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;

use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, Identifier};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::world::World;

/// Vanilla `JigsawBlockEntity.DEFAULT_FINAL_STATE`.
pub const DEFAULT_FINAL_STATE: &str = "minecraft:air";
/// Vanilla `JigsawBlockEntity.JointType.ROLLABLE` ordinal.
pub const JOINT_ROLLABLE: i32 = 0;
/// Vanilla `JigsawBlockEntity.JointType.ALIGNED` ordinal.
pub const JOINT_ALIGNED: i32 = 1;

/// Mutable jigsaw editor state.
#[derive(Debug, Clone, PartialEq, Eq)]
struct JigsawState {
    /// Vanilla defaults to `minecraft:empty` for both identifiers.
    name: String,
    target: String,
    pool: String,
    final_state: String,
    joint: i32,
    placement_priority: i32,
    selection_priority: i32,
}

impl JigsawState {
    fn new() -> Self {
        Self {
            // Vanilla `EMPTY_ID` / `Pools.EMPTY` serialize as `minecraft:empty`.
            name: "minecraft:empty".to_owned(),
            target: "minecraft:empty".to_owned(),
            pool: "minecraft:empty".to_owned(),
            final_state: DEFAULT_FINAL_STATE.to_owned(),
            joint: JOINT_ROLLABLE,
            placement_priority: 0,
            selection_priority: 0,
        }
    }
}

/// Vanilla `JigsawBlockEntity`.
pub struct JigsawBlockEntity {
    base: Arc<BlockEntityBase>,
    state: SyncMutex<JigsawState>,
}

// SAFETY: This key is owned by Steel and uniquely identifies this type.
unsafe impl DowncastType for JigsawBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/jigsaw");
}

impl JigsawBlockEntity {
    /// Creates a new jigsaw block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(
                &vanilla_block_entity_types::JIGSAW,
                level,
                pos,
                state,
            )),
            state: SyncMutex::new(JigsawState::new()),
        }
    }

    /// Vanilla `JigsawBlockEntity.getName`.
    #[must_use]
    pub fn name(&self) -> String {
        self.state.lock().name.clone()
    }

    /// Vanilla `JigsawBlockEntity.setName`.
    pub fn set_name(&self, name: Identifier) {
        self.state.lock().name = name.to_string();
    }

    /// Vanilla `JigsawBlockEntity.getTarget`.
    #[must_use]
    pub fn target(&self) -> String {
        self.state.lock().target.clone()
    }

    /// Vanilla `JigsawBlockEntity.setTarget`.
    pub fn set_target(&self, target: Identifier) {
        self.state.lock().target = target.to_string();
    }

    /// Vanilla `JigsawBlockEntity.getPool`.
    #[must_use]
    pub fn pool(&self) -> String {
        self.state.lock().pool.clone()
    }

    /// Vanilla `JigsawBlockEntity.setPool`.
    pub fn set_pool(&self, pool: Identifier) {
        self.state.lock().pool = pool.to_string();
    }

    /// Vanilla `JigsawBlockEntity.getFinalState`.
    #[must_use]
    pub fn final_state(&self) -> String {
        self.state.lock().final_state.clone()
    }

    /// Vanilla `JigsawBlockEntity.setFinalState`.
    pub fn set_final_state(&self, final_state: String) {
        self.state.lock().final_state = final_state;
    }

    /// Vanilla `JigsawBlockEntity.getJoint`.
    #[must_use]
    pub fn joint(&self) -> i32 {
        self.state.lock().joint
    }

    /// Vanilla `JigsawBlockEntity.setJoint`.
    pub fn set_joint(&self, joint: i32) {
        self.state.lock().joint = joint;
    }

    /// Sets the joint mode from its serialized vanilla name (`"aligned"` or `"rollable"`).
    pub fn set_joint_name(&self, name: &str) {
        self.set_joint(joint_from_name(name));
    }

    /// Vanilla `JigsawBlockEntity.getPlacementPriority`.
    #[must_use]
    pub fn placement_priority(&self) -> i32 {
        self.state.lock().placement_priority
    }

    /// Vanilla `JigsawBlockEntity.setPlacementPriority`.
    pub fn set_placement_priority(&self, priority: i32) {
        self.state.lock().placement_priority = priority;
    }

    /// Vanilla `JigsawBlockEntity.getSelectionPriority`.
    #[must_use]
    pub fn selection_priority(&self) -> i32 {
        self.state.lock().selection_priority
    }

    /// Vanilla `JigsawBlockEntity.setSelectionPriority`.
    pub fn set_selection_priority(&self, priority: i32) {
        self.state.lock().selection_priority = priority;
    }
}

impl BlockEntity for JigsawBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let nbt_view: NbtCompoundView<'_, '_> = nbt.into();
        let mut state = self.state.lock();
        state.name = nbt_view
            .string("name")
            .map_or_else(|| "minecraft:empty".to_owned(), |value| value.to_str().into_owned());
        state.target = nbt_view
            .string("target")
            .map_or_else(|| "minecraft:empty".to_owned(), |value| value.to_str().into_owned());
        state.pool = nbt_view
            .string("pool")
            .map_or_else(|| "minecraft:empty".to_owned(), |value| value.to_str().into_owned());
        state.final_state = nbt_view
            .string("final_state")
            .map_or_else(|| DEFAULT_FINAL_STATE.to_owned(), |value| value.to_str().into_owned());
        state.joint = nbt_view
            .string("joint")
            .map_or(JOINT_ROLLABLE, |value| joint_from_name(&value.to_str()));
        state.placement_priority = nbt_view.int_value("placement_priority").unwrap_or(0);
        state.selection_priority = nbt_view.int_value("selection_priority").unwrap_or(0);
    }

    /// Vanilla `JigsawBlockEntity.getUpdateTag`: `saveCustomOnly`.
    fn get_update_tag(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.save_additional(&mut nbt);
        Some(nbt)
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let state = self.state.lock();
        nbt.insert("name", state.name.clone());
        nbt.insert("target", state.target.clone());
        nbt.insert("pool", state.pool.clone());
        nbt.insert("final_state", state.final_state.clone());
        nbt.insert("joint", joint_name(state.joint).to_owned());
        nbt.insert("placement_priority", state.placement_priority);
        nbt.insert("selection_priority", state.selection_priority);
    }
}

/// Vanilla `JigsawBlockEntity.JointType.CODEC` names.
const fn joint_name(joint: i32) -> &'static str {
    if joint == JOINT_ALIGNED {
        "aligned"
    } else {
        "rollable"
    }
}

/// Parses a stored joint name, defaulting to `rollable`.
fn joint_from_name(name: &str) -> i32 {
    if name == "aligned" {
        JOINT_ALIGNED
    } else {
        JOINT_ROLLABLE
    }
}
