//! Villager trading: offers, stock and the price adjustments vanilla applies.
//!
//! Vanilla builds each villager's trade list from a datapack-driven pool per
//! profession and level. Steel has no trade data yet, so this module carries a
//! hand-written table that mirrors the vanilla structure: the same professions,
//! the same first-level trades and the same costs. A trade from the table is the
//! `TradeOffer` below; what a villager actually sells is a `Vec` of them, which is
//! what the menu sends in `merchant_offers` and what a completed trade consumes.
//!
//! Prices follow vanilla's `MerchantOffer` fields so the client renders demand
//! and stock the same way: `uses` counts down to `max_uses`, after which the
//! trade locks until the villager restocks, and `demand` raises the price of
//! trades that are bought out too often.

use steel_registry::item_stack::ItemStack;
use steel_registry::{RegistryExt as _, vanilla_items};

/// One trade a villager offers.
#[derive(Clone, Debug)]
pub struct TradeOffer {
    /// What the player pays first.
    pub base_cost_a: ItemStack,
    /// The optional second payment item.
    pub base_cost_b: ItemStack,
    /// What the player receives.
    pub result: ItemStack,
    /// Remaining uses before the trade locks.
    pub uses: i32,
    /// Uses at which the trade locks.
    pub max_uses: i32,
    /// Experience the villager gains per completed trade.
    pub xp: i32,
    /// Extra cost in percent on top of the base cost.
    pub special_price_diff: i32,
    /// Demand for the result, which raises the price.
    pub demand: i32,
}

impl TradeOffer {
    /// Builds an offer from its payment and result with vanilla's defaults for
    /// a first-level trade: twelve uses, two experience and no price markup.
    #[must_use]
    pub fn new(
        base_cost_a: ItemStack,
        base_cost_b: ItemStack,
        result: ItemStack,
        max_uses: i32,
        xp: i32,
    ) -> Self {
        Self {
            base_cost_a,
            base_cost_b,
            result,
            uses: 0,
            max_uses,
            xp,
            special_price_diff: 0,
            demand: 0,
        }
    }

    /// Whether the trade has any stock left.
    #[must_use]
    pub const fn is_out_of_stock(&self) -> bool {
        self.uses >= self.max_uses
    }

    /// The first payment with the price markup applied, mirroring vanilla's
    /// `MerchantOffer.getAdjustedCostA`.
    ///
    /// `demand` and `special_price_diff` both raise the price; vanilla clamps the
    /// result at one item so a discounted trade never becomes free.
    #[must_use]
    pub fn adjusted_cost_a(&self) -> ItemStack {
        let base = self.base_cost_a.count();
        let markup = base * self.special_price_diff / 100
            + self.demand * self.base_cost_a.count() / (self.max_uses - self.uses).max(1);
        let mut cost = self.base_cost_a.clone();
        cost.set_count((base + markup).max(1));
        cost
    }

    /// Records a completed trade: one use spent, demand raised and the price
    /// drifting up as stock falls.
    ///
    /// Vanilla raises demand by one per use and decays it over time; the decay
    /// needs the restock timer, which Steel does not model yet, so demand only
    /// grows.
    pub fn record_use(&mut self) {
        self.uses += 1;
        self.demand = (self.demand + 1).min(self.max_uses - self.uses.max(0));
    }

    /// Resets the trade to full stock, as a villager does when it restocks.
    pub fn restock(&mut self) {
        self.uses = 0;
        self.demand = 0;
    }
}

/// A payment item and a result item, the shorthand the trade table uses.
struct Trade {
    cost_a: (steel_registry::items::ItemRef, i32),
    cost_b: Option<(steel_registry::items::ItemRef, i32)>,
    result: (steel_registry::items::ItemRef, i32),
    max_uses: i32,
    xp: i32,
}

impl Trade {
    const fn new(
        cost_a: (steel_registry::items::ItemRef, i32),
        cost_b: Option<(steel_registry::items::ItemRef, i32)>,
        result: (steel_registry::items::ItemRef, i32),
        max_uses: i32,
        xp: i32,
    ) -> Self {
        Self {
            cost_a,
            cost_b,
            result,
            max_uses,
            xp,
        }
    }

    fn build(&self) -> TradeOffer {
        TradeOffer::new(
            ItemStack::with_count(self.cost_a.0, self.cost_a.1),
            self.cost_b.map_or_else(ItemStack::empty, |(item, count)| {
                ItemStack::with_count(item, count)
            }),
            ItemStack::with_count(self.result.0, self.result.1),
            self.max_uses,
            self.xp,
        )
    }
}

/// Vanilla's first-level trades for every profession that has them.
///
/// The list covers the novice tier only: later tiers need experience tracking
/// and profession levelling, which the villager does not have yet. An unknown or
/// jobless profession trades nothing, exactly like a vanilla nitwit.
fn trades_for(profession: &str) -> Vec<Trade> {
    use vanilla_items as i;
    match profession {
        "minecraft:farmer" => vec![
            Trade::new((&*i::WHEAT, 20), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::POTATO, 26), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::CARROT, 22), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::BEETROOT, 15), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::EMERALD, 1), None, (&*i::BREAD, 6), 16, 2),
            Trade::new((&*i::EMERALD, 1), None, (&*i::PUMPKIN_PIE, 4), 16, 2),
            Trade::new((&*i::EMERALD, 1), None, (&*i::APPLE, 4), 16, 2),
        ],
        "minecraft:librarian" => vec![
            Trade::new((&*i::PAPER, 24), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::EMERALD, 9), None, (&*i::BOOKSHELF, 1), 16, 2),
            Trade::new((&*i::EMERALD, 5), Some((&*i::BOOK, 1)), (&*i::ENCHANTED_BOOK, 1), 16, 2),
            Trade::new((&*i::EMERALD, 1), None, (&*i::LANTERN, 1), 16, 2),
        ],
        "minecraft:armorer" => vec![
            Trade::new((&*i::COAL, 15), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::EMERALD, 5), None, (&*i::IRON_HELMET, 1), 16, 2),
            Trade::new((&*i::EMERALD, 9), None, (&*i::IRON_CHESTPLATE, 1), 16, 2),
            Trade::new((&*i::EMERALD, 7), None, (&*i::IRON_LEGGINGS, 1), 16, 2),
            Trade::new((&*i::EMERALD, 4), None, (&*i::IRON_BOOTS, 1), 16, 2),
        ],
        "minecraft:toolsmith" => vec![
            Trade::new((&*i::COAL, 15), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::EMERALD, 6), None, (&*i::STONE_AXE, 1), 16, 2),
            Trade::new((&*i::EMERALD, 6), None, (&*i::STONE_PICKAXE, 1), 16, 2),
            Trade::new((&*i::EMERALD, 6), None, (&*i::STONE_SHOVEL, 1), 16, 2),
        ],
        "minecraft:weaponsmith" => vec![
            Trade::new((&*i::COAL, 15), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::EMERALD, 3), None, (&*i::IRON_AXE, 1), 16, 2),
            Trade::new((&*i::EMERALD, 8), None, (&*i::IRON_SWORD, 1), 16, 2),
        ],
        "minecraft:cleric" => vec![
            Trade::new((&*i::ROTTEN_FLESH, 32), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::EMERALD, 1), None, (&*i::REDSTONE, 4), 16, 2),
            Trade::new((&*i::EMERALD, 1), None, (&*i::LAPIS_LAZULI, 1), 16, 2),
        ],
        "minecraft:butcher" => vec![
            Trade::new((&*i::BEEF, 14), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::PORKCHOP, 14), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::CHICKEN, 14), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::EMERALD, 1), None, (&*i::COOKED_PORKCHOP, 5), 16, 2),
        ],
        "minecraft:fletcher" => vec![
            Trade::new((&*i::STICK, 32), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::EMERALD, 1), None, (&*i::ARROW, 16), 16, 2),
            Trade::new((&*i::EMERALD, 1), Some((&*i::FLINT, 10)), (&*i::ARROW, 16), 16, 2),
        ],
        "minecraft:cartographer" => vec![
            Trade::new((&*i::PAPER, 24), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::EMERALD, 7), None, (&*i::MAP, 1), 16, 2),
        ],
        "minecraft:mason" => vec![
            Trade::new((&*i::CLAY_BALL, 10), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::EMERALD, 1), None, (&*i::BRICK, 10), 16, 2),
            Trade::new((&*i::EMERALD, 1), None, (&*i::CHISELED_STONE_BRICKS, 4), 16, 2),
        ],
        "minecraft:leatherworker" => vec![
            Trade::new((&*i::LEATHER, 6), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::EMERALD, 3), None, (&*i::LEATHER_LEGGINGS, 1), 16, 2),
        ],
        "minecraft:shepherd" => vec![
            Trade::new((&*i::WHITE_WOOL, 18), None, (&*i::EMERALD, 1), 16, 2),
            Trade::new((&*i::EMERALD, 2), None, (&*i::SHEARS, 1), 16, 2),
        ],
        "minecraft:fisherman" => vec![
            Trade::new((&*i::COD, 6), Some((&*i::EMERALD, 1)), (&*i::COOKED_COD, 6), 16, 2),
            Trade::new((&*i::EMERALD, 1), None, (&*i::COOKED_SALMON, 6), 16, 2),
        ],
        "minecraft:wandering_trader" => vec![
            Trade::new((&*i::EMERALD, 1), None, (&*i::SEA_PICKLE, 1), 12, 2),
            Trade::new((&*i::EMERALD, 5), None, (&*i::CACTUS, 1), 12, 2),
            Trade::new((&*i::EMERALD, 1), None, (&*i::SUGAR_CANE, 1), 12, 2),
            Trade::new((&*i::EMERALD, 1), None, (&*i::KELP, 1), 12, 2),
        ],
        _ => Vec::new(),
    }
}

/// Builds the offer list for a profession.
///
/// The profession key is the vanilla one (`minecraft:farmer`, ...); an empty or
/// unknown profession, which is what a nitwit and a jobless villager have,
/// yields no trades.
#[must_use]
pub fn offers_for(profession: &str) -> Vec<TradeOffer> {
    trades_for(profession)
        .into_iter()
        .map(|trade| trade.build())
        .collect()
}

/// Builds the offer list for a profession registry id, which is how the
/// villager stores its profession in the synced `villager_data` component.
///
/// An id outside the registry, or the `none` profession, trades nothing.
#[must_use]
pub fn offers_for_profession_id(profession_id: i32) -> Vec<TradeOffer> {
    let Ok(id) = usize::try_from(profession_id) else {
        return Vec::new();
    };
    let Some(profession) = steel_registry::REGISTRY.villager_professions.by_id(id) else {
        return Vec::new();
    };
    offers_for(&profession.key.to_string())
}

/// The professions that have a trade table, for the tests and for callers that
/// need to pick one.
#[cfg(test)]
pub const PROFESSIONS: [&str; 14] = [
    "minecraft:farmer",
    "minecraft:librarian",
    "minecraft:armorer",
    "minecraft:toolsmith",
    "minecraft:weaponsmith",
    "minecraft:cleric",
    "minecraft:butcher",
    "minecraft:fletcher",
    "minecraft:cartographer",
    "minecraft:mason",
    "minecraft:leatherworker",
    "minecraft:shepherd",
    "minecraft:fisherman",
    "minecraft:wandering_trader",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_profession_offers_at_least_one_trade() {
        for profession in PROFESSIONS {
            let offers = offers_for(profession);
            assert!(
                !offers.is_empty(),
                "{profession} should trade at least one item"
            );
        }
    }

    #[test]
    fn a_jobless_villager_trades_nothing() {
        // Profession id 0 is `minecraft:none` in `villager_professions.json`.
        assert!(offers_for_profession_id(0).is_empty());
        assert!(offers_for_profession_id(-1).is_empty());
        assert!(offers_for_profession_id(i32::MAX).is_empty());
    }

    #[test]
    fn a_farmer_gets_its_trades_from_the_registry_id() {
        let farmer = &steel_registry::vanilla_villager_professions::FARMER;
        let id = steel_registry::REGISTRY
            .villager_professions
            .id_from_key(&farmer.key)
            .expect("the farmer profession is registered");
        let offers = offers_for_profession_id(i32::try_from(id).unwrap());
        assert_eq!(offers.len(), offers_for("minecraft:farmer").len());
        assert!(!offers.is_empty());
    }

    #[test]
    fn an_unknown_profession_trades_nothing() {
        assert!(offers_for("minecraft:none").is_empty());
        assert!(offers_for("").is_empty());
        assert!(offers_for("minecraft:nitwit").is_empty());
    }

    #[test]
    fn fresh_offers_are_in_stock() {
        for offer in offers_for("minecraft:farmer") {
            assert!(!offer.is_out_of_stock());
            assert_eq!(offer.uses, 0);
            assert!(offer.max_uses > 0);
        }
    }

    #[test]
    fn a_trade_locks_once_it_has_been_used_up() {
        let mut offer = &mut offers_for("minecraft:fisherman")[0];
        for _ in 0..offer.max_uses {
            assert!(!offer.is_out_of_stock());
            offer.record_use();
        }
        assert!(offer.is_out_of_stock());
    }

    #[test]
    fn a_locked_trade_reopens_after_a_restock() {
        let mut offer = offers_for("minecraft:mason")[0].clone();
        for _ in 0..offer.max_uses {
            offer.record_use();
        }
        assert!(offer.is_out_of_stock());

        offer.restock();
        assert!(!offer.is_out_of_stock());
        assert_eq!(offer.uses, 0);
        assert_eq!(offer.demand, 0);
    }

    #[test]
    fn demand_raises_the_price_but_never_below_one() {
        let mut offer = offers_for("minecraft:librarian")[1].clone();
        let base = offer.adjusted_cost_a().count();
        offer.record_use();
        assert!(offer.adjusted_cost_a().count() >= base);

        // A trade whose markup would drive it under one item stays at one.
        offer.special_price_diff = -1000;
        assert_eq!(offer.adjusted_cost_a().count(), 1);
    }

    #[test]
    fn the_second_payment_is_optional() {
        let librarian = offers_for("minecraft:librarian");
        let with_book = librarian
            .iter()
            .find(|offer| !offer.base_cost_b.is_empty())
            .expect("the librarian sells at least one two-item trade");
        assert!(with_book.base_cost_b.count() > 0);

        let farmer = offers_for("minecraft:farmer");
        assert!(farmer.iter().any(|offer| offer.base_cost_b.is_empty()));
    }
}
