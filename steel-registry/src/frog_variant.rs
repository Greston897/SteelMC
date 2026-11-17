use crate::shared_structs::{SpawnConditionEntry, insert_spawn_conditions};
use rustc_hash::FxHashMap;
use simdnbt::ToNbtTag;
use simdnbt::owned::NbtTag;
use steel_utils::Identifier;

/// Represents a full frog variant definition from a data pack JSON file.
#[derive(Debug)]
pub struct FrogVariant {
    pub key: Identifier,
    pub asset_id: Identifier,
    pub spawn_conditions: &'static [SpawnConditionEntry],
}

impl FrogVariant {
    /// Vanilla `FrogVariant.NETWORK_CODEC` — `{asset_id}` only. Frogs have no
    /// `baby_asset_id`, and `spawn_conditions` is server-only.
    #[must_use]
    pub fn to_network_nbt(&self) -> NbtTag {
        use simdnbt::owned::NbtCompound;
        let mut compound = NbtCompound::new();
        let asset_id = self.asset_id.to_string();
        compound.insert("asset_id", asset_id.as_str());
        NbtTag::Compound(compound)
    }

    /// Full direct codec including server-only `spawn_conditions`.
    #[must_use]
    pub fn to_direct_nbt_tag(&self) -> NbtTag {
        use simdnbt::owned::NbtTag;
        let NbtTag::Compound(mut compound) = self.to_network_nbt() else {
            unreachable!("frog variant network NBT is always a compound");
        };
        insert_spawn_conditions(&mut compound, self.spawn_conditions);
        NbtTag::Compound(compound)
    }
}

impl ToNbtTag for &FrogVariant {
    fn to_nbt_tag(self) -> NbtTag {
        self.to_network_nbt()
    }
}

pub type FrogVariantRef = &'static FrogVariant;

pub struct FrogVariantRegistry {
    frog_variants_by_id: Vec<FrogVariantRef>,
    frog_variants_by_key: FxHashMap<Identifier, usize>,
    allows_registering: bool,
}

impl FrogVariantRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            frog_variants_by_id: Vec::new(),
            frog_variants_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }
}

crate::impl_standard_methods!(
    FrogVariantRegistry,
    FrogVariantRef,
    frog_variants_by_id,
    frog_variants_by_key,
    allows_registering
);

crate::impl_registry!(
    FrogVariantRegistry,
    FrogVariant,
    frog_variants_by_id,
    frog_variants_by_key,
    frog_variants
);
