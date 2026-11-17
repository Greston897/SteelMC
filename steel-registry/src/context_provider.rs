//! Vanilla MC 26.3 `minecraft:context_int_provider`,
//! `minecraft:context_float_provider`, and `minecraft:predicate` reloadable
//! registries (`RegistryDataLoader.RELOADABLE_REGISTRIES`).
//!
//! Powers data-driven `minecraft:cooking_fuel`, `minecraft:brewing_fuel`,
//! `minecraft:compostable`, and loot/tool predicates (`block/fast_cooking`,
//! `tool/can_shear`, `tool/can_silk_touch`).

use rustc_hash::FxHashMap;
use steel_utils::Identifier;
use steel_utils::random::Random;

use crate::{REGISTRY, RegistryExt, TaggedRegistryExt};

const MAX_PROVIDER_DEPTH: usize = 32;

/// Evaluation context supplied when sampling a context provider or predicate.
#[derive(Debug, Clone, Default)]
pub struct ProviderEvalContext<'a> {
    /// Block identifier at the interaction position (e.g. `minecraft:blast_furnace`,
    /// `minecraft:composter`).
    pub block: Option<&'a Identifier>,
    /// Block-state property `(name, value)` pairs (e.g. `[("level", "0")]`).
    pub block_state: &'a [(&'a str, &'a str)],
    /// Tool item identifier, if any (e.g. `minecraft:shears`).
    pub tool_item: Option<&'a Identifier>,
    /// `(enchantment_key, level)` pairs present on the tool item.
    pub tool_enchantments: &'a [(Identifier, i32)],
}

impl<'a> ProviderEvalContext<'a> {
    /// Convenience context with only a target block identifier.
    #[must_use]
    pub const fn for_block(block: &'a Identifier) -> Self {
        Self {
            block: Some(block),
            block_state: &[],
            tool_item: None,
            tool_enchantments: &[],
        }
    }

    /// Convenience context with a target block and state property pairs.
    #[must_use]
    pub const fn for_block_state(
        block: &'a Identifier,
        block_state: &'a [(&'a str, &'a str)],
    ) -> Self {
        Self {
            block: Some(block),
            block_state,
            tool_item: None,
            tool_enchantments: &[],
        }
    }

    /// Convenience context with a held tool and its active enchantments.
    #[must_use]
    pub const fn for_tool(
        tool_item: &'a Identifier,
        tool_enchantments: &'a [(Identifier, i32)],
    ) -> Self {
        Self {
            block: None,
            block_state: &[],
            tool_item: Some(tool_item),
            tool_enchantments,
        }
    }
}

/// Enchantment level constraint inside a `match_tool` predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantmentRequirement {
    /// Enchantment key or `#tag` (without `#` when `is_tag` is true).
    pub enchantment: Identifier,
    pub is_tag: bool,
    pub min_level: Option<i32>,
    pub max_level: Option<i32>,
}

/// Condition expression used in `minecraft:predicate` and inline inside
/// `context_int_provider` / `context_float_provider`.
#[derive(Debug, Clone)]
pub enum ContextCondition {
    /// Reference to a registered `minecraft:predicate` entry.
    Reference(Identifier),
    /// Constant boolean value.
    Constant(bool),
    /// `minecraft:inverted` — logical negation.
    Inverted(Box<ContextCondition>),
    /// `minecraft:all_of` — conjunction of child terms.
    AllOf(Vec<ContextCondition>),
    /// `minecraft:any_of` — disjunction of child terms.
    AnyOf(Vec<ContextCondition>),
    /// `minecraft:random_chance` — Bernoulli trial with `chance` in `0.0..=1.0`.
    RandomChance(f32),
    /// `minecraft:match_block` — matches block ID/tag plus optional state map.
    MatchBlock {
        blocks: Vec<Identifier>,
        block_tag: Option<Identifier>,
        state: Vec<(String, String)>,
    },
    /// `minecraft:match_tool` — matches tool item IDs/tag plus enchantment predicates.
    MatchTool {
        items: Vec<Identifier>,
        item_tag: Option<Identifier>,
        enchantments: Vec<EnchantmentRequirement>,
    },
}

impl ContextCondition {
    /// Evaluates this condition against `ctx` and `rng`.
    #[must_use]
    pub fn evaluate<R: Random + ?Sized>(
        &self,
        ctx: &ProviderEvalContext<'_>,
        rng: &mut R,
    ) -> bool {
        self.eval_bounded(ctx, rng, 0)
    }

    fn eval_bounded<R: Random + ?Sized>(
        &self,
        ctx: &ProviderEvalContext<'_>,
        rng: &mut R,
        depth: usize,
    ) -> bool {
        if depth > MAX_PROVIDER_DEPTH {
            return false;
        }
        match self {
            Self::Reference(id) => REGISTRY
                .predicates
                .by_key(id)
                .is_some_and(|pred| pred.condition.eval_bounded(ctx, rng, depth + 1)),
            Self::Constant(value) => *value,
            Self::Inverted(inner) => !inner.eval_bounded(ctx, rng, depth + 1),
            Self::AllOf(terms) => terms.iter().all(|t| t.eval_bounded(ctx, rng, depth + 1)),
            Self::AnyOf(terms) => terms.iter().any(|t| t.eval_bounded(ctx, rng, depth + 1)),
            Self::RandomChance(chance) => rng.next_f32() < *chance,
            Self::MatchBlock {
                blocks,
                block_tag,
                state,
            } => {
                let Some(block_key) = ctx.block else {
                    return false;
                };
                if !blocks.is_empty() && !blocks.iter().any(|b| b == block_key) {
                    return false;
                }
                if let Some(tag) = block_tag {
                    let Some(block) = REGISTRY.blocks.by_key(block_key) else {
                        return false;
                    };
                    if !REGISTRY.blocks.is_in_tag(block, tag) {
                        return false;
                    }
                }
                for (prop, expected) in state {
                    if !ctx
                        .block_state
                        .iter()
                        .any(|(k, v)| k == prop && v == expected)
                    {
                        return false;
                    }
                }
                true
            }
            Self::MatchTool {
                items,
                item_tag,
                enchantments,
            } => {
                let Some(tool_key) = ctx.tool_item else {
                    return false;
                };
                if !items.is_empty() && !items.iter().any(|it| it == tool_key) {
                    return false;
                }
                if let Some(tag) = item_tag {
                    let Some(item) = REGISTRY.items.by_key(tool_key) else {
                        return false;
                    };
                    if !REGISTRY.items.is_in_tag(item, tag) {
                        return false;
                    }
                }
                for req in enchantments {
                    let matched = ctx.tool_enchantments.iter().any(|(ench_key, level)| {
                        let key_ok = if req.is_tag {
                            REGISTRY
                                .enchantments
                                .by_key(ench_key)
                                .is_some_and(|e| REGISTRY.enchantments.is_in_tag(e, &req.enchantment))
                        } else {
                            ench_key == &req.enchantment
                        };
                        let min_ok = req.min_level.is_none_or(|min| *level >= min);
                        let max_ok = req.max_level.is_none_or(|max| *level <= max);
                        key_ok && min_ok && max_ok
                    });
                    if !matched {
                        return false;
                    }
                }
                true
            }
        }
    }
}

/// Registered `minecraft:predicate` entry.
#[derive(Debug, Clone)]
pub struct LootPredicate {
    pub key: Identifier,
    pub condition: ContextCondition,
}

pub type LootPredicateRef = &'static LootPredicate;

pub struct LootPredicateRegistry {
    predicates_by_id: Vec<LootPredicateRef>,
    predicates_by_key: FxHashMap<Identifier, usize>,
    allows_registering: bool,
}

impl LootPredicateRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            predicates_by_id: Vec::new(),
            predicates_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }

    /// Evaluates a registered predicate by key.
    #[must_use]
    pub fn evaluate<R: Random + ?Sized>(
        &self,
        key: &Identifier,
        ctx: &ProviderEvalContext<'_>,
        rng: &mut R,
    ) -> Option<bool> {
        Some(self.by_key(key)?.condition.evaluate(ctx, rng))
    }
}

crate::impl_standard_methods!(
    LootPredicateRegistry,
    LootPredicateRef,
    predicates_by_id,
    predicates_by_key,
    allows_registering
);

crate::impl_registry!(
    LootPredicateRegistry,
    LootPredicate,
    predicates_by_id,
    predicates_by_key,
    predicates
);

/// Expression tree of a `minecraft:context_int_provider`.
#[derive(Debug, Clone)]
pub enum ContextIntExpr {
    Constant(i32),
    Reference(Identifier),
    Uniform {
        min_inclusive: Box<ContextIntExpr>,
        max_inclusive: Box<ContextIntExpr>,
    },
    Clamped {
        source: Box<ContextIntExpr>,
        min: i32,
        max: i32,
    },
    WeightedList(Vec<(ContextIntExpr, u32)>),
    Add(Box<ContextIntExpr>, Box<ContextIntExpr>),
    Sub(Box<ContextIntExpr>, Box<ContextIntExpr>),
    Mul(Box<ContextIntExpr>, Box<ContextIntExpr>),
    Div(Box<ContextIntExpr>, Box<ContextIntExpr>),
    Conditional {
        condition: ContextCondition,
        on_true: Box<ContextIntExpr>,
        on_false: Box<ContextIntExpr>,
    },
    NumberDispatcher {
        cases: Vec<(ContextCondition, ContextIntExpr)>,
        default: Box<ContextIntExpr>,
    },
}

impl ContextIntExpr {
    /// Samples this integer expression in `ctx`.
    #[must_use]
    pub fn sample<R: Random + ?Sized>(&self, ctx: &ProviderEvalContext<'_>, rng: &mut R) -> i32 {
        self.sample_bounded(ctx, rng, 0)
    }

    fn sample_bounded<R: Random + ?Sized>(
        &self,
        ctx: &ProviderEvalContext<'_>,
        rng: &mut R,
        depth: usize,
    ) -> i32 {
        if depth > MAX_PROVIDER_DEPTH {
            return 0;
        }
        match self {
            Self::Constant(v) => *v,
            Self::Reference(id) => REGISTRY
                .context_int_providers
                .by_key(id)
                .map_or(0, |p| p.expr.sample_bounded(ctx, rng, depth + 1)),
            Self::Uniform {
                min_inclusive,
                max_inclusive,
            } => {
                let min = min_inclusive.sample_bounded(ctx, rng, depth + 1);
                let max = max_inclusive.sample_bounded(ctx, rng, depth + 1);
                if max <= min {
                    min
                } else {
                    rng.next_i32_between(min, max)
                }
            }
            Self::Clamped { source, min, max } => source
                .sample_bounded(ctx, rng, depth + 1)
                .clamp(*min, *max),
            Self::WeightedList(entries) => {
                let total: u64 = entries.iter().map(|(_, w)| u64::from(*w)).sum();
                if total == 0 {
                    return 0;
                }
                let bound = i32::try_from(total).unwrap_or(i32::MAX);
                let mut roll = u64::try_from(rng.next_i32_bounded(bound)).unwrap_or(0);
                for (expr, weight) in entries {
                    let w = u64::from(*weight);
                    if roll < w {
                        return expr.sample_bounded(ctx, rng, depth + 1);
                    }
                    roll -= w;
                }
                entries
                    .last()
                    .map_or(0, |(expr, _)| expr.sample_bounded(ctx, rng, depth + 1))
            }
            Self::Add(l, r) => l
                .sample_bounded(ctx, rng, depth + 1)
                .saturating_add(r.sample_bounded(ctx, rng, depth + 1)),
            Self::Sub(l, r) => l
                .sample_bounded(ctx, rng, depth + 1)
                .saturating_sub(r.sample_bounded(ctx, rng, depth + 1)),
            Self::Mul(l, r) => l
                .sample_bounded(ctx, rng, depth + 1)
                .saturating_mul(r.sample_bounded(ctx, rng, depth + 1)),
            Self::Div(l, r) => {
                let lhs = l.sample_bounded(ctx, rng, depth + 1);
                let rhs = r.sample_bounded(ctx, rng, depth + 1);
                if rhs == 0 { 0 } else { lhs.wrapping_div(rhs) }
            }
            Self::Conditional {
                condition,
                on_true,
                on_false,
            } => {
                if condition.eval_bounded(ctx, rng, depth + 1) {
                    on_true.sample_bounded(ctx, rng, depth + 1)
                } else {
                    on_false.sample_bounded(ctx, rng, depth + 1)
                }
            }
            Self::NumberDispatcher { cases, default } => {
                for (condition, value) in cases {
                    if condition.eval_bounded(ctx, rng, depth + 1) {
                        return value.sample_bounded(ctx, rng, depth + 1);
                    }
                }
                default.sample_bounded(ctx, rng, depth + 1)
            }
        }
    }
}

/// Registered `minecraft:context_int_provider` entry.
#[derive(Debug, Clone)]
pub struct ContextIntProvider {
    pub key: Identifier,
    pub expr: ContextIntExpr,
}

pub type ContextIntProviderRef = &'static ContextIntProvider;

pub struct ContextIntProviderRegistry {
    providers_by_id: Vec<ContextIntProviderRef>,
    providers_by_key: FxHashMap<Identifier, usize>,
    allows_registering: bool,
}

impl ContextIntProviderRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers_by_id: Vec::new(),
            providers_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }

    /// Evaluates a registered `minecraft:context_int_provider` by key.
    #[must_use]
    pub fn sample<R: Random + ?Sized>(
        &self,
        key: &Identifier,
        ctx: &ProviderEvalContext<'_>,
        rng: &mut R,
    ) -> Option<i32> {
        Some(self.by_key(key)?.expr.sample(ctx, rng))
    }
}

crate::impl_standard_methods!(
    ContextIntProviderRegistry,
    ContextIntProviderRef,
    providers_by_id,
    providers_by_key,
    allows_registering
);

crate::impl_registry!(
    ContextIntProviderRegistry,
    ContextIntProvider,
    providers_by_id,
    providers_by_key,
    context_int_providers
);

/// Expression tree of a `minecraft:context_float_provider`.
#[derive(Debug, Clone)]
pub enum ContextFloatExpr {
    Constant(f32),
    Reference(Identifier),
    Uniform {
        min_inclusive: Box<ContextFloatExpr>,
        max_exclusive: Box<ContextFloatExpr>,
    },
    Add(Box<ContextFloatExpr>, Box<ContextFloatExpr>),
    Sub(Box<ContextFloatExpr>, Box<ContextFloatExpr>),
    Mul(Box<ContextFloatExpr>, Box<ContextFloatExpr>),
    Div(Box<ContextFloatExpr>, Box<ContextFloatExpr>),
    Conditional {
        condition: ContextCondition,
        on_true: Box<ContextFloatExpr>,
        on_false: Box<ContextFloatExpr>,
    },
    NumberDispatcher {
        cases: Vec<(ContextCondition, ContextFloatExpr)>,
        default: Box<ContextFloatExpr>,
    },
}

impl ContextFloatExpr {
    /// Samples this float expression in `ctx`.
    #[must_use]
    pub fn sample<R: Random + ?Sized>(&self, ctx: &ProviderEvalContext<'_>, rng: &mut R) -> f32 {
        self.sample_bounded(ctx, rng, 0)
    }

    fn sample_bounded<R: Random + ?Sized>(
        &self,
        ctx: &ProviderEvalContext<'_>,
        rng: &mut R,
        depth: usize,
    ) -> f32 {
        if depth > MAX_PROVIDER_DEPTH {
            return 0.0;
        }
        match self {
            Self::Constant(v) => *v,
            Self::Reference(id) => REGISTRY
                .context_float_providers
                .by_key(id)
                .map_or(0.0, |p| p.expr.sample_bounded(ctx, rng, depth + 1)),
            Self::Uniform {
                min_inclusive,
                max_exclusive,
            } => {
                let min = min_inclusive.sample_bounded(ctx, rng, depth + 1);
                let max = max_exclusive.sample_bounded(ctx, rng, depth + 1);
                min + rng.next_f32() * (max - min)
            }
            Self::Add(l, r) => {
                l.sample_bounded(ctx, rng, depth + 1) + r.sample_bounded(ctx, rng, depth + 1)
            }
            Self::Sub(l, r) => {
                l.sample_bounded(ctx, rng, depth + 1) - r.sample_bounded(ctx, rng, depth + 1)
            }
            Self::Mul(l, r) => {
                l.sample_bounded(ctx, rng, depth + 1) * r.sample_bounded(ctx, rng, depth + 1)
            }
            Self::Div(l, r) => {
                let lhs = l.sample_bounded(ctx, rng, depth + 1);
                let rhs = r.sample_bounded(ctx, rng, depth + 1);
                if rhs == 0.0 { 0.0 } else { lhs / rhs }
            }
            Self::Conditional {
                condition,
                on_true,
                on_false,
            } => {
                if condition.eval_bounded(ctx, rng, depth + 1) {
                    on_true.sample_bounded(ctx, rng, depth + 1)
                } else {
                    on_false.sample_bounded(ctx, rng, depth + 1)
                }
            }
            Self::NumberDispatcher { cases, default } => {
                for (condition, value) in cases {
                    if condition.eval_bounded(ctx, rng, depth + 1) {
                        return value.sample_bounded(ctx, rng, depth + 1);
                    }
                }
                default.sample_bounded(ctx, rng, depth + 1)
            }
        }
    }
}

/// Registered `minecraft:context_float_provider` entry.
#[derive(Debug, Clone)]
pub struct ContextFloatProvider {
    pub key: Identifier,
    pub expr: ContextFloatExpr,
}

pub type ContextFloatProviderRef = &'static ContextFloatProvider;

pub struct ContextFloatProviderRegistry {
    providers_by_id: Vec<ContextFloatProviderRef>,
    providers_by_key: FxHashMap<Identifier, usize>,
    allows_registering: bool,
}

impl ContextFloatProviderRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers_by_id: Vec::new(),
            providers_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }

    /// Evaluates a registered `minecraft:context_float_provider` by key.
    #[must_use]
    pub fn sample<R: Random + ?Sized>(
        &self,
        key: &Identifier,
        ctx: &ProviderEvalContext<'_>,
        rng: &mut R,
    ) -> Option<f32> {
        Some(self.by_key(key)?.expr.sample(ctx, rng))
    }
}

crate::impl_standard_methods!(
    ContextFloatProviderRegistry,
    ContextFloatProviderRef,
    providers_by_id,
    providers_by_key,
    allows_registering
);

crate::impl_registry!(
    ContextFloatProviderRegistry,
    ContextFloatProvider,
    providers_by_id,
    providers_by_key,
    context_float_providers
);
