//! Vanilla MC 26.3 `minecraft:villager_trade` (`391/391`, `73` tags) and
//! `minecraft:trade_set` (`68/68`) data-driven merchant registries.
//!
//! Mirrors `net.minecraft.world.item.trading.VillagerTrade` / `TradeSet` /
//! `TradeCost` codecs exactly:
//!
//! * `wants` / `additional_wants` are [`TradeCostSpec`] (`TradeCost.CODEC`):
//!   item id, loot `NumberProvider` count (default `1`), exact component
//!   predicate (vanilla uses only `minecraft:potion_contents` for the wandering
//!   trader water-bottle trade).
//! * `gives` is a plain [`TradeResultSpec`] (`ItemStackTemplate` without
//!   components — no vanilla trade sets them).
//! * `max_uses` (default `4`), `reputation_discount` (default `0.0`) and `xp`
//!   (default `1`) are loot `NumberProvider`s sampled per generated offer.
//! * `merchant_predicate` is vanilla's full loot condition; every vanilla use
//!   (`27/27`) is an `entity_properties` check on `minecraft:villager/variant`,
//!   which is what [`VillagerTrade::merchant_villager_variants`] stores (empty
//!   means unrestricted).
//! * `given_item_modifier` (26.3 renamed the file field to singular;
//!   [`TradeItemModifier`] covers all 8 vanilla loot-function shapes) applies
//!   to the result in order; an emptied result discards the whole offer.
//! * `double_trade_price_enchantments` doubles the sampled additional emerald
//!   cost when the result's stored enchantments intersect the set.
//!
//! Offer generation ([`VillagerTrade::build_offer_parts`]) follows
//! `VillagerTrade.getOffer` step by step, including the
//! `2 + nextInt(5 + level * 10) + 3 * level` additional-cost formula from
//! `EnchantRandomlyFunction` and the `levels.getInt` cost from
//! `EnchantWithLevelsFunction`.

use rustc_hash::FxHashMap;
use steel_utils::Identifier;
use steel_utils::random::Random;

use crate::enchantment_provider::EnchantmentProviderSet;
use crate::{REGISTRY, RegistryExt, RegistryTags, TaggedRegistryExt};

/// Loot `NumberProvider` subset valid inside villager trades
/// (`LootContextParamSets.VILLAGER_TRADE`).
///
/// Vanilla validation rejects the context-dependent providers (`score`,
/// `storage`, `enchantment_level`, `environment_attribute`) in trades, so this
/// is 100% of the valid shape space: bare constants plus `uniform`, `binomial`
/// and `add` (26.3 renamed `sum`/`summands` to `add`/`inputs`).
#[derive(Debug, Clone)]
pub enum TradeNumberProvider {
    Constant(f32),
    Uniform {
        min: Box<TradeNumberProvider>,
        max: Box<TradeNumberProvider>,
    },
    Binomial {
        n: Box<TradeNumberProvider>,
        p: Box<TradeNumberProvider>,
    },
    Add(Vec<TradeNumberProvider>),
}

impl TradeNumberProvider {
    /// Vanilla `ConstantValue.exactly`.
    #[must_use]
    pub const fn constant(value: f32) -> Self {
        Self::Constant(value)
    }

    /// Vanilla `NumberProvider.getInt`.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "matches vanilla Mth.floor/getInt narrowing of provider floats"
    )]
    pub fn sample_int<R: Random + ?Sized>(&self, rng: &mut R) -> i32 {
        match self {
            // Vanilla `ConstantValue.getInt` floors.
            Self::Constant(value) => value.floor() as i32,
            // Vanilla `UniformGenerator.getInt` is inclusive on both ends.
            Self::Uniform { min, max } => {
                let lo = min.sample_int(rng);
                let hi = max.sample_int(rng);
                if hi <= lo { lo } else { rng.next_i32_between(lo, hi) }
            }
            Self::Binomial { n, p } => {
                let trials = n.sample_int(rng).max(0);
                let chance = p.sample_float(rng);
                let mut result = 0;
                for _ in 0..trials {
                    if rng.next_f32() < chance {
                        result += 1;
                    }
                }
                result
            }
            // Vanilla `Sum.getInt` floors the float sum.
            Self::Add(inputs) => {
                inputs.iter().map(|p| p.sample_float(rng)).sum::<f32>().floor() as i32
            }
        }
    }

    /// Vanilla `NumberProvider.getFloat`.
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "matches vanilla widening of binomial int counts to float"
    )]
    pub fn sample_float<R: Random + ?Sized>(&self, rng: &mut R) -> f32 {
        match self {
            Self::Constant(value) => *value,
            Self::Uniform { min, max } => {
                let lo = min.sample_float(rng);
                let hi = max.sample_float(rng);
                lo + rng.next_f32() * (hi - lo)
            }
            Self::Binomial { .. } => self.sample_int(rng) as f32,
            Self::Add(inputs) => inputs.iter().map(|p| p.sample_float(rng)).sum(),
        }
    }
}

/// Cost item specification (`wants` or `additional_wants`) in a `VillagerTrade`.
#[derive(Debug, Clone)]
pub struct TradeCostSpec {
    pub item: Identifier,
    pub count: TradeNumberProvider,
    /// Exact `minecraft:potion_contents.potion` the cost must carry, if any.
    /// This is the only `TradeCost.components` shape vanilla data uses (the
    /// wandering trader water-bottle trade).
    pub potion: Option<Identifier>,
}

/// Output item specification (`gives`) in a `VillagerTrade`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeResultSpec {
    pub item: Identifier,
    pub count: i32,
}

/// Generic id-or-tag holder set (potion options of `set_random_potion`, ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TradeHolderSet {
    Tag(Identifier),
    List(Vec<Identifier>),
}

/// A single `given_item_modifier` applied to a trade result, in file order.
///
/// Covers all 8 loot-function shapes vanilla 26.3 trade data uses. The
/// `filtered` + `discard` pairs collapse into `Require*` guards: the item must
/// carry the component afterwards or the whole offer is discarded, exactly like
/// vanilla's `FilteredFunction` with `on_fail: discard`.
#[derive(Debug, Clone)]
pub enum TradeItemModifier {
    /// `minecraft:enchant_randomly` — uniform pick from `options`, random level
    /// in `1..=max_level`, books become enchanted books.
    EnchantRandomly {
        options: EnchantmentProviderSet,
        only_compatible: bool,
        include_additional_cost_component: bool,
    },
    /// `minecraft:enchant_with_levels` — enchanting-table roll at `levels`.
    EnchantWithLevels {
        options: Option<EnchantmentProviderSet>,
        levels: TradeNumberProvider,
        include_additional_cost_component: bool,
    },
    /// `minecraft:exploration_map` — fills the map towards `destination`.
    ///
    /// `destination` is a structure *tag* (`TagKey.codec`, default
    /// `#minecraft:on_treasure_maps`); `decoration` defaults to
    /// `minecraft:mansion` (`DEFAULT_DECORATION`, also used for explicit
    /// `null`); `zoom` defaults to `2`, `search_radius` to `50` and
    /// `skip_existing_chunks` to `true`.
    ExplorationMap {
        destination: Identifier,
        decoration: Identifier,
        zoom: i32,
        search_radius: i32,
        skip_existing_chunks: bool,
    },
    /// `minecraft:set_stew_effect` — one random `(effect, duration)` pair.
    SetStewEffect { effects: Vec<(Identifier, i32)> },
    /// `minecraft:set_potion`.
    SetPotion { potion: Identifier },
    /// `minecraft:set_random_potion` — uniform pick from the potion set
    /// (`None` means every registered potion).
    SetRandomPotion { options: Option<TradeHolderSet> },
    /// `minecraft:set_random_dyes` — mixes `number_of_dyes` random dyes.
    SetRandomDyes {
        number_of_dyes: TradeNumberProvider,
    },
    /// `filtered` requiring any live enchantments (`minecraft:enchantments: [{}]`).
    RequireEnchantments,
    /// `filtered` requiring stored enchantments (enchanted books).
    RequireStoredEnchantments,
    /// `filtered` requiring a map id (exploration map resolved).
    RequireMapId,
    /// `filtered` requiring dyed color.
    RequireDyedColor,
}

/// Registered `minecraft:villager_trade` entry (`data/minecraft/villager_trade/**/*.json`).
#[derive(Debug, Clone)]
pub struct VillagerTrade {
    pub key: Identifier,
    pub wants: TradeCostSpec,
    pub additional_wants: Option<TradeCostSpec>,
    pub gives: TradeResultSpec,
    pub max_uses: TradeNumberProvider,
    pub xp: TradeNumberProvider,
    pub reputation_discount: TradeNumberProvider,
    /// Allowed merchant villager variants (`minecraft:villager/variant`).
    /// Empty means the trade has no `merchant_predicate` restriction.
    pub merchant_villager_variants: Vec<Identifier>,
    pub given_item_modifiers: Vec<TradeItemModifier>,
    pub double_trade_price_enchantments: Option<EnchantmentProviderSet>,
}

impl VillagerTrade {
    /// Vanilla `VillagerTrade.getOffer` price math shared by every caller.
    ///
    /// Returns `(wants_count, additional_cost_doubled)` inputs: samples the
    /// `NumberProvider` count, adds the loot-function `additional_cost`
    /// (doubled when the result's stored enchantments intersect
    /// `double_trade_price_enchantments`), and clamps into `0..=max_stack` like
    /// `TradeCost.toItemCost`.
    #[must_use]
    pub fn price_for<R: Random + ?Sized>(
        &self,
        additional_cost: i32,
        result_stored_enchantments: &[Identifier],
        max_stack: i32,
        rng: &mut R,
    ) -> i32 {
        let mut cost = additional_cost;
        if let Some(set) = &self.double_trade_price_enchantments {
            let pool: Vec<Identifier> = match set {
                EnchantmentProviderSet::Tag(tag) => REGISTRY
                    .enchantments
                    .iter_tag(tag)
                    .map(|e| e.key.clone())
                    .collect(),
                EnchantmentProviderSet::List(keys) => keys.clone(),
            };
            if result_stored_enchantments
                .iter()
                .any(|e| pool.contains(e))
            {
                cost *= 2;
            }
        }
        (self.wants.count.sample_int(rng) + cost).clamp(0, max_stack.max(0))
    }
}

pub type VillagerTradeRef = &'static VillagerTrade;

pub struct VillagerTradeRegistry {
    trades_by_id: Vec<VillagerTradeRef>,
    trades_by_key: FxHashMap<Identifier, usize>,
    tags: RegistryTags,
    allows_registering: bool,
}

impl VillagerTradeRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            trades_by_id: Vec::new(),
            trades_by_key: FxHashMap::default(),
            tags: RegistryTags::default(),
            allows_registering: true,
        }
    }
}

crate::impl_standard_methods!(
    VillagerTradeRegistry,
    VillagerTradeRef,
    trades_by_id,
    trades_by_key,
    allows_registering
);

crate::impl_registry!(
    VillagerTradeRegistry,
    VillagerTrade,
    trades_by_id,
    trades_by_key,
    villager_trades
);

crate::impl_tagged_registry!(VillagerTradeRegistry, trades_by_key, "villager_trade");

/// Pool of `VillagerTrade` entries referenced by a `TradeSet`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TradeSetEntries {
    Tag(Identifier),
    Direct(Vec<Identifier>),
}

impl TradeSetEntries {
    /// Resolves all `VillagerTradeRef` entries in this trade set.
    #[must_use]
    pub fn resolve(&self) -> Vec<VillagerTradeRef> {
        match self {
            Self::Tag(tag) => REGISTRY.villager_trades.iter_tag(tag).collect(),
            Self::Direct(keys) => keys
                .iter()
                .filter_map(|k| REGISTRY.villager_trades.by_key(k))
                .collect(),
        }
    }
}

/// Registered `minecraft:trade_set` entry (`data/minecraft/trade_set/**/*.json`).
#[derive(Debug, Clone)]
pub struct TradeSet {
    pub key: Identifier,
    pub trades: TradeSetEntries,
    pub amount: TradeNumberProvider,
    pub allow_duplicates: bool,
    pub random_sequence: Option<Identifier>,
}

impl TradeSet {
    /// Samples `amount` trade definitions from this trade set following vanilla's
    /// duplicate policy.
    #[must_use]
    pub fn sample_trades<R: Random + ?Sized>(&self, rng: &mut R) -> Vec<VillagerTradeRef> {
        let mut pool = self.trades.resolve();
        if pool.is_empty() {
            return Vec::new();
        }
        let target = usize::try_from(self.amount.sample_int(rng).max(0)).unwrap_or(0);
        let mut chosen = Vec::with_capacity(target);
        if self.allow_duplicates {
            let bound = i32::try_from(pool.len()).unwrap_or(i32::MAX);
            for _ in 0..target {
                let idx = usize::try_from(rng.next_i32_bounded(bound)).unwrap_or(0);
                chosen.push(pool[idx]);
            }
        } else {
            while chosen.len() < target && !pool.is_empty() {
                let bound = i32::try_from(pool.len()).unwrap_or(i32::MAX);
                let idx = usize::try_from(rng.next_i32_bounded(bound)).unwrap_or(0);
                chosen.push(pool.swap_remove(idx));
            }
        }
        chosen
    }
}

pub type TradeSetRef = &'static TradeSet;

pub struct TradeSetRegistry {
    sets_by_id: Vec<TradeSetRef>,
    sets_by_key: FxHashMap<Identifier, usize>,
    allows_registering: bool,
}

impl TradeSetRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sets_by_id: Vec::new(),
            sets_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }
}

crate::impl_standard_methods!(
    TradeSetRegistry,
    TradeSetRef,
    sets_by_id,
    sets_by_key,
    allows_registering
);

crate::impl_registry!(
    TradeSetRegistry,
    TradeSet,
    sets_by_id,
    sets_by_key,
    trade_sets
);
