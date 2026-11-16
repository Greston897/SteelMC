//! Test instance block entity.
//!
//! Vanilla `TestInstanceBlockEntity` is the persisted editor state of a test
//! instance block: the selected test, structure size, rotation, entity flag and
//! the last run status. It is stored as a single `data` compound plus the list
//! of error markers, so a marker block entity would silently drop everything a
//! gamemaster configured.

use std::sync::{Arc, Weak};

use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::{NbtCompound, NbtList};
use steel_protocol::packets::game::{TestInstanceBlockData, TestInstanceBlockSetupStatus};
use steel_registry::vanilla_block_entity_types;
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;

use steel_utils::nbt::NbtNumeric as _;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::world::World;

/// Vanilla `TestInstanceBlockEntity.STRUCTURE_OFFSET`.
pub const STRUCTURE_OFFSET: glam::IVec3 = glam::IVec3::new(0, 1, 1);

/// Vanilla `TestInstanceBlockEntity`.
pub struct TestInstanceBlockEntity {
    base: Arc<BlockEntityBase>,
    data: SyncMutex<TestInstanceBlockData>,
}

// SAFETY: This key is owned by Steel and uniquely identifies this type.
unsafe impl DowncastType for TestInstanceBlockEntity {
    const TYPE_KEY: DowncastTypeKey =
        DowncastTypeKey::new("steel:block_entity/test_instance_block");
}

impl TestInstanceBlockEntity {
    /// Creates a new test instance block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(
                &vanilla_block_entity_types::TEST_INSTANCE_BLOCK,
                level,
                pos,
                state,
            )),
            data: SyncMutex::new(TestInstanceBlockData::default()),
        }
    }

    /// Vanilla `TestInstanceBlockEntity.getData`.
    #[must_use]
    pub fn data(&self) -> TestInstanceBlockData {
        self.data.lock().clone()
    }

    /// Vanilla `TestInstanceBlockEntity.set`: replaces the data and marks the
    /// block entity dirty.
    pub fn set(&self, data: TestInstanceBlockData) {
        *self.data.lock() = data;
        self.set_changed();
    }

    /// Vanilla `TestInstanceBlockEntity.set` for the status only.
    pub fn set_status(&self, status: TestInstanceBlockSetupStatus) {
        self.data.lock().status = status;
        self.set_changed();
    }

    /// Vanilla `TestInstanceBlockEntity.getStructureSize` needs the test
    /// instance registry and the structure template manager, neither of which
    /// Steel models yet. Returning `None` matches vanilla's "no size known"
    /// branch, which the client renders as an empty selection.
    #[must_use]
    pub const fn structure_size(&self) -> Option<glam::IVec3> {
        None
    }
}

impl BlockEntity for TestInstanceBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let nbt_view: NbtCompoundView<'_, '_> = nbt.into();
        let Some(data) = nbt_view.compound("data") else {
            return;
        };
        let mut data_out = TestInstanceBlockData::default();
        // Vanilla `Vec3i.CODEC` is `Codec.INT_STREAM.comapFlatMap(fixedSize(3))`,
        // i.e. a list of three ints — not a compound with `x`/`y`/`z` keys.
        if let Some(size) = data.list("size") {
            let tags = size.to_owned().as_nbt_tags();
            let mut axes = tags
                .iter()
                .filter_map(steel_utils::nbt::NbtNumeric::codec_i32);
            if let (Some(x), Some(y), Some(z)) = (axes.next(), axes.next(), axes.next()) {
                data_out.size = glam::IVec3 { x, y, z };
            }
        }
        data_out.ignore_entities = data.byte_value("ignore_entities").is_some_and(|v| v != 0);
        data_out.status = match data
            .string("status")
            .map(|v| v.to_str().into_owned())
            .as_deref()
        {
            Some("running") => TestInstanceBlockSetupStatus::Running,
            Some("finished") => TestInstanceBlockSetupStatus::Finished,
            _ => TestInstanceBlockSetupStatus::Cleared,
        };
        data_out.rotation = match data
            .string("rotation")
            .map(|v| v.to_str().into_owned())
            .as_deref()
        {
            Some("clockwise_90") => steel_utils::Rotation::Clockwise90,
            Some("180") => steel_utils::Rotation::Clockwise180,
            Some("counterclockwise_90") => steel_utils::Rotation::CounterClockwise90,
            _ => steel_utils::Rotation::None,
        };
        *self.data.lock() = data_out;
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let data = self.data.lock();
        let mut compound = NbtCompound::new();
        if let Some(test) = &data.test {
            compound.insert("test", test.to_string());
        }
        // Vanilla `Vec3i.CODEC` writes an int list.
        compound.insert(
            "size",
            NbtList::Int(vec![data.size.x, data.size.y, data.size.z]),
        );
        compound.insert(
            "rotation",
            match data.rotation {
                steel_utils::Rotation::None => "none",
                steel_utils::Rotation::Clockwise90 => "clockwise_90",
                steel_utils::Rotation::Clockwise180 => "180",
                steel_utils::Rotation::CounterClockwise90 => "counterclockwise_90",
            }
            .to_owned(),
        );
        compound.insert("ignore_entities", u8::from(data.ignore_entities));
        compound.insert(
            "status",
            match data.status {
                TestInstanceBlockSetupStatus::Cleared => "cleared",
                TestInstanceBlockSetupStatus::Running => "running",
                TestInstanceBlockSetupStatus::Finished => "finished",
            }
            .to_owned(),
        );
        if let Some(message) = &data.error_message {
            compound.insert("error_message", message.to_codec_nbt());
        }
        drop(data);
        nbt.insert("data", compound);
    }

    fn get_update_tag(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.save_additional(&mut nbt);
        Some(nbt)
    }
}
