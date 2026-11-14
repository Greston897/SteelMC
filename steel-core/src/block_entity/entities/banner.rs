//! Banner block entity.
//!
//! Vanilla `BannerBlockEntity` stores the banner base colour (derived from the
//! block) and a list of pattern layers. Each layer is a pattern registry key plus
//! an optional dye colour. Banners are also `ItemSink`s for banner items, which
//! lets the loom produce patterned banners.

use std::sync::{Arc, Weak};

use simdnbt::borrow::{BaseNbtCompound as BorrowedNbtCompound, NbtCompound as NbtCompoundView};
use simdnbt::owned::{NbtCompound, NbtList};
use steel_registry::block_entity_type::BlockEntityTypeRef;
use steel_registry::vanilla_block_entity_types;
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, Identifier};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::world::World;

/// One banner pattern layer: the pattern key and an optional dye colour.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BannerPatternLayer {
    /// Registry key of the pattern (`minecraft:base`, `minecraft:border`, ...).
    pub pattern: Identifier,
    /// Dye colour name (`white`, `orange`, ...); `None` means the base colour.
    pub color: Option<Identifier>,
}

/// Banner block entity.
pub struct BannerBlockEntity {
    base: Arc<BlockEntityBase>,
    layers: SyncMutex<Vec<BannerPatternLayer>>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `BannerBlockEntity`.
unsafe impl DowncastType for BannerBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/banner");
}

impl BannerBlockEntity {
    /// Creates a new banner block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self::new_with_type(&vanilla_block_entity_types::BANNER, level, pos, state)
    }

    fn new_with_type(
        block_entity_type: BlockEntityTypeRef,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(block_entity_type, level, pos, state)),
            layers: SyncMutex::new(Vec::new()),
        }
    }

    /// Returns a snapshot of the pattern layers.
    #[must_use]
    pub fn layers(&self) -> Vec<BannerPatternLayer> {
        self.layers.lock().clone()
    }

    /// Replaces the pattern layers, matching vanilla's `setItemValues`.
    pub fn set_layers(&self, layers: Vec<BannerPatternLayer>) {
        *self.layers.lock() = layers;
    }
}

impl BlockEntity for BannerBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let nbt_view: NbtCompoundView<'_, '_> = nbt.into();
        let mut layers = Vec::new();
        if let Some(list) = nbt_view.list("patterns") {
            let tags = list.to_owned().as_nbt_tags();
            for entry in tags.iter() {
                let Some(compound) = entry.compound() else {
                    continue;
                };
                let Some(pattern) = compound
                    .string("pattern")
                    .and_then(|value| value.to_str().parse().ok())
                else {
                    continue;
                };
                let color = compound
                    .string("color")
                    .and_then(|value| value.to_str().parse::<Identifier>().ok());
                layers.push(BannerPatternLayer { pattern, color });
            }
        }
        *self.layers.lock() = layers;
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let layers = self.layers.lock();
        let mut patterns: Vec<NbtCompound> = Vec::with_capacity(layers.len());
        for layer in layers.iter() {
            let mut pattern = NbtCompound::new();
            pattern.insert("pattern", layer.pattern.to_string().as_str());
            if let Some(color) = &layer.color {
                pattern.insert("color", color.to_string().as_str());
            }
            patterns.push(pattern);
        }
        drop(layers);
        nbt.insert("patterns", NbtList::Compound(patterns));
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks};

    use super::*;

    #[test]
    fn banner_starts_without_layers() {
        init_vanilla_registry();
        let banner = BannerBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::WHITE_BANNER.default_state(),
        );
        assert!(banner.layers().is_empty());
    }
}
