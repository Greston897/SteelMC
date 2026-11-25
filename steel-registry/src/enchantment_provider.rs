//! Vanilla MC 26.3 `minecraft:enchantment_provider` registry (`7/7`).
//!
//! Covers all 3 `minecraft:enchantment_provider_type` variants (`single`,
//! `by_cost`, `by_cost_with_difficulty`) with NBT serialization and runtime
//! enchantment rolling for mob spawns, raids, pillagers, and endermen.

use rustc_hash::FxHashMap;
use simdnbt::ToNbtTag;
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use steel_utils::Identifier;
use steel_utils::random::Random;
use steel_utils::value_providers::IntProvider;

use crate::enchantment::EnchantmentRef;
use crate::{REGISTRY, RegistryExt, TaggedRegistryExt};

/// Holder set of enchantments referenced by `by_cost` / `by_cost_with_difficulty`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnchantmentProviderSet {
    Tag(Identifier),
    List(Vec<Identifier>),
}

impl EnchantmentProviderSet {
    /// Resolves every registered enchantment belonging to this set.
    #[must_use]
    pub fn resolve(&self) -> Vec<EnchantmentRef> {
        match self {
            Self::Tag(tag) => REGISTRY.enchantments.iter_tag(tag).collect(),
            Self::List(keys) => keys
                .iter()
                .filter_map(|k| REGISTRY.enchantments.by_key(k))
                .collect(),
        }
    }

    fn to_nbt_tag(&self) -> NbtTag {
        match self {
            Self::Tag(tag) => NbtTag::String(format!("#{tag}").into()),
            Self::List(list) => {
                if list.len() == 1 {
                    NbtTag::String(list[0].to_string().into())
                } else {
                    NbtTag::List(NbtList::String(
                        list.iter().map(|id| id.to_string().into()).collect(),
                    ))
                }
            }
        }
    }
}

/// All 3 MC 26.3 `EnchantmentProvider` dispatch variants.
#[derive(Debug, Clone)]
pub enum EnchantmentProviderKind {
    /// `minecraft:single` — applies a single enchantment at a sampled level.
    Single {
        enchantment: Identifier,
        level: IntProvider,
    },
    /// `minecraft:by_cost` — rolls enchantments using a sampled cost.
    ByCost {
        enchantments: EnchantmentProviderSet,
        cost: IntProvider,
    },
    /// `minecraft:by_cost_with_difficulty` — rolls enchantments scaled by
    /// `special_multiplier` (`DifficultyInstance.getSpecialMultiplier`, `0.0..=1.0`).
    ByCostWithDifficulty {
        enchantments: EnchantmentProviderSet,
        min_cost: i32,
        max_cost_span: i32,
    },
}

/// Registered `minecraft:enchantment_provider` entry.
#[derive(Debug, Clone)]
pub struct EnchantmentProvider {
    pub key: Identifier,
    pub kind: EnchantmentProviderKind,
}

impl EnchantmentProvider {
    /// Samples `(enchantment, clamped_level)` pairs for `special_multiplier` (`0.0..=1.0`).
    #[must_use]
    pub fn sample<R: Random + ?Sized>(
        &self,
        special_multiplier: f32,
        rng: &mut R,
    ) -> Vec<(EnchantmentRef, i32)> {
        match &self.kind {
            EnchantmentProviderKind::Single { enchantment, level } => {
                let Some(ench) = REGISTRY.enchantments.by_key(enchantment) else {
                    return Vec::new();
                };
                let max = i32::try_from(ench.max_level).unwrap_or(1).max(1);
                let sampled = level.sample(rng).clamp(1, max);
                vec![(ench, sampled)]
            }
            EnchantmentProviderKind::ByCost { enchantments, cost } => {
                let rolled_cost = cost.sample(rng);
                select_by_cost(&enchantments.resolve(), rolled_cost, rng)
            }
            EnchantmentProviderKind::ByCostWithDifficulty {
                enchantments,
                min_cost,
                max_cost_span,
            } => {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "matches vanilla Mth.randomBetweenInclusive(random, minCost, minCost + (int)(specialMultiplier * maxCostSpan))"
                )]
                let span = (special_multiplier.clamp(0.0, 1.0) * (*max_cost_span as f32)) as i32;
                let cost = if span > 0 {
                    rng.next_i32_between(*min_cost, min_cost.saturating_add(span))
                } else {
                    *min_cost
                };
                select_by_cost(&enchantments.resolve(), cost, rng)
            }
        }
    }
}

fn select_by_cost<R: Random + ?Sized>(
    pool: &[EnchantmentRef],
    cost: i32,
    rng: &mut R,
) -> Vec<(EnchantmentRef, i32)> {
    let mut candidates: Vec<(EnchantmentRef, i32)> = Vec::new();
    for &ench in pool {
        let max_level = i32::try_from(ench.max_level).unwrap_or(1);
        for level in (1..=max_level).rev() {
            if cost >= ench.min_cost.calculate(level) && cost <= ench.max_cost.calculate(level) {
                candidates.push((ench, level));
                break;
            }
        }
    }
    if candidates.is_empty() {
        return Vec::new();
    }
    let total_weight: u64 = candidates.iter().map(|(e, _)| u64::from(e.weight)).sum();
    if total_weight == 0 {
        return Vec::new();
    }
    let bound = i32::try_from(total_weight).unwrap_or(i32::MAX);
    let mut roll = u64::try_from(rng.next_i32_bounded(bound)).unwrap_or(0);
    for (ench, level) in candidates {
        let w = u64::from(ench.weight);
        if roll < w {
            return vec![(ench, level)];
        }
        roll -= w;
    }
    Vec::new()
}

fn int_provider_to_nbt(provider: &IntProvider) -> NbtTag {
    match provider {
        IntProvider::Constant(v) => NbtTag::Int(*v),
        IntProvider::Uniform {
            min_inclusive,
            max_inclusive,
        } => {
            let mut c = NbtCompound::new();
            c.insert("type", "minecraft:uniform");
            c.insert("min_inclusive", *min_inclusive);
            c.insert("max_inclusive", *max_inclusive);
            NbtTag::Compound(c)
        }
        IntProvider::BiasedToBottom {
            min_inclusive,
            max_inclusive,
        } => {
            let mut c = NbtCompound::new();
            c.insert("type", "minecraft:biased_to_bottom");
            c.insert("min_inclusive", *min_inclusive);
            c.insert("max_inclusive", *max_inclusive);
            NbtTag::Compound(c)
        }
        IntProvider::Clamped {
            min_inclusive,
            max_inclusive,
            source,
        } => {
            let mut c = NbtCompound::new();
            c.insert("type", "minecraft:clamped");
            c.insert("min_inclusive", *min_inclusive);
            c.insert("max_inclusive", *max_inclusive);
            c.insert("source", int_provider_to_nbt(source));
            NbtTag::Compound(c)
        }
        IntProvider::ClampedNormal {
            mean,
            deviation,
            min_inclusive,
            max_inclusive,
        } => {
            let mut c = NbtCompound::new();
            c.insert("type", "minecraft:clamped_normal");
            c.insert("mean", *mean);
            c.insert("deviation", *deviation);
            c.insert("min_inclusive", *min_inclusive);
            c.insert("max_inclusive", *max_inclusive);
            NbtTag::Compound(c)
        }
        IntProvider::VeryBiasedToBottom {
            min_inclusive,
            max_inclusive,
        } => {
            let mut c = NbtCompound::new();
            c.insert("type", "minecraft:biased_to_bottom");
            c.insert("min_inclusive", *min_inclusive);
            c.insert("max_inclusive", *max_inclusive);
            NbtTag::Compound(c)
        }
        IntProvider::Trapezoid { min, max, plateau } => {
            let mut c = NbtCompound::new();
            c.insert("type", "minecraft:trapezoid");
            c.insert("min", *min);
            c.insert("max", *max);
            c.insert("plateau", *plateau);
            NbtTag::Compound(c)
        }
        IntProvider::WeightedList { distribution } => {
            let mut c = NbtCompound::new();
            c.insert("type", "minecraft:weighted_list");
            let list: Vec<NbtCompound> = distribution
                .iter()
                .map(|entry| {
                    let mut e = NbtCompound::new();
                    e.insert("data", int_provider_to_nbt(&entry.data));
                    e.insert("weight", entry.weight);
                    e
                })
                .collect();
            c.insert("distribution", NbtTag::List(NbtList::Compound(list)));
            NbtTag::Compound(c)
        }
    }
}

impl ToNbtTag for &EnchantmentProvider {
    fn to_nbt_tag(self) -> NbtTag {
        let mut compound = NbtCompound::new();
        match &self.kind {
            EnchantmentProviderKind::Single { enchantment, level } => {
                compound.insert("type", "minecraft:single");
                compound.insert("enchantment", enchantment.to_string().as_str());
                compound.insert("level", int_provider_to_nbt(level));
            }
            EnchantmentProviderKind::ByCost { enchantments, cost } => {
                compound.insert("type", "minecraft:by_cost");
                compound.insert("enchantments", enchantments.to_nbt_tag());
                compound.insert("cost", int_provider_to_nbt(cost));
            }
            EnchantmentProviderKind::ByCostWithDifficulty {
                enchantments,
                min_cost,
                max_cost_span,
            } => {
                compound.insert("type", "minecraft:by_cost_with_difficulty");
                compound.insert("enchantments", enchantments.to_nbt_tag());
                compound.insert("min_cost", *min_cost);
                compound.insert("max_cost_span", *max_cost_span);
            }
        }
        NbtTag::Compound(compound)
    }
}

pub type EnchantmentProviderRef = &'static EnchantmentProvider;

pub struct EnchantmentProviderRegistry {
    providers_by_id: Vec<EnchantmentProviderRef>,
    providers_by_key: FxHashMap<Identifier, usize>,
    allows_registering: bool,
}

impl EnchantmentProviderRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers_by_id: Vec::new(),
            providers_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }
}

crate::impl_standard_methods!(
    EnchantmentProviderRegistry,
    EnchantmentProviderRef,
    providers_by_id,
    providers_by_key,
    allows_registering
);

crate::impl_registry!(
    EnchantmentProviderRegistry,
    EnchantmentProvider,
    providers_by_id,
    providers_by_key,
    enchantment_providers
);
