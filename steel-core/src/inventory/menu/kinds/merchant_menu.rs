//! Merchant menu (the villager and wandering trader trade screen).
//!
//! Vanilla `MerchantMenu` shows two payment slots and a result slot above the
//! player inventory. The offers themselves travel in a separate `merchant_offers`
//! packet, which is why the menu cannot be built from the slot layout alone.
//!
//! Steel differs from vanilla in one place: vanilla keeps the trade list on the
//! menu and syncs the payment from the slots every tick, while here the trades
//! come out of the villager and the menu mirrors them. The visible behaviour is
//! the same — pay, take the result, the stock drops.
//!
//! Slot layout, matching `merchant` (39 slots):
//! - Slots 0 and 1: the two payment slots
//! - Slot 2: the result, take-only
//! - Slots 3-38: the player inventory

use std::sync::Arc;

use steel_protocol::packets::game::{CMerchantOffers, MerchantOffer};
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_menu_types;
use steel_utils::locks::SyncMutex;

use crate::entity::villager_trades::TradeOffer;
use crate::inventory::click::{Click, ClickOutcome};
use crate::inventory::lock::ContainerLockGuard;
use crate::inventory::menu::MenuKind;
use crate::inventory::menu::behavior::MenuBehavior;
use crate::inventory::prelude::*;
use crate::player::Player;
use crate::player::player_inventory::{PlayerInventory, Prediction};

/// Index of the first payment slot.
const COST_SLOT_A: usize = 0;
/// Index of the second payment slot.
const COST_SLOT_B: usize = 1;
/// Index of the result slot.
const RESULT_SLOT: usize = 2;
/// Number of trade-specific slots, i.e. where the player inventory starts.
const TRADE_SLOTS: usize = 3;

/// Shared handle to a villager's trade list.
pub type SharedTrades = Arc<SyncMutex<Vec<TradeOffer>>>;

/// Merchant menu state.
pub struct MerchantKind {
    /// The villager's offers, shared with the entity so a completed trade
    /// updates the stock the next time the screen opens.
    trades: SharedTrades,
    /// Container backing the two payment slots and the result.
    trade_container: ContainerRef,
}

// SAFETY: This Steel-owned key uniquely identifies the concrete menu kind
// within the process.
unsafe impl steel_utils::DowncastType for MerchantKind {
    const TYPE_KEY: steel_utils::DowncastTypeKey =
        steel_utils::DowncastTypeKey::new("steel:menu/merchant");
}

impl MerchantKind {
    /// Creates the kind over a shared trade list.
    #[must_use]
    pub const fn new(trades: SharedTrades, trade_container: ContainerRef) -> Self {
        Self {
            trades,
            trade_container,
        }
    }

    /// The offers currently on sale.
    #[must_use]
    pub fn offers(&self) -> Vec<TradeOffer> {
        self.trades.lock().clone()
    }

    /// The offer the current payment slots pay for, if any.
    ///
    /// Vanilla picks the offer by matching the payment against every trade; the
    /// first match wins, which matters when a villager sells two trades with the
    /// same cost.
    fn matching_offer(&self, paid_a: &ItemStack, paid_b: &ItemStack) -> Option<usize> {
        let trades = self.trades.lock();
        trades.iter().position(|offer| {
            !offer.is_out_of_stock()
                && matches_payment(&offer.base_cost_a, paid_a)
                && (offer.base_cost_b.is_empty() || matches_payment(&offer.base_cost_b, paid_b))
        })
    }
}

/// Converts a trade into the wire form the client renders.
fn offer_to_packet(offer: &TradeOffer) -> MerchantOffer {
    MerchantOffer {
        base_cost_a: offer.adjusted_cost_a(),
        base_cost_b: offer.base_cost_b.clone(),
        result: offer.result.clone(),
        out_of_stock: offer.is_out_of_stock(),
        uses: offer.uses,
        max_uses: offer.max_uses,
        xp: offer.xp,
        special_price_diff: offer.special_price_diff,
        demand: offer.demand,
    }
}

/// Whether a payment slot holds enough of the item an offer asks for.
fn matches_payment(cost: &ItemStack, paid: &ItemStack) -> bool {
    !paid.is_empty() && ItemStack::is_same_item(cost, paid) && paid.count() >= cost.count()
}

/// Builds the merchant menu over a villager's trade list.
///
/// `closing` receives the villager's entity id so the menu can refuse to reopen
/// after the villager dies or walks away; callers that do not track that pass
/// `None`.
#[must_use]
pub fn merchant(
    inventory: Shared<PlayerInventory>,
    container_id: u8,
    trades: SharedTrades,
) -> Menu {
    let trade_container: ContainerRef = SimpleContainer::new(TRADE_SLOTS).into_shared().into();

    let mut builder = MenuBuilder::new(&vanilla_menu_types::MERCHANT, container_id);
    let trade = builder.section(&trade_container, TRADE_SLOTS);
    let player = builder.player_inventory(&inventory);

    builder.route(trade, player.all(), FillDirection::Backward);
    builder.route(player.all(), trade, FillDirection::Forward);

    builder.build(MerchantKind::new(trades, trade_container))
}

impl MenuKind for MerchantKind {
    /// Vanilla `MerchantMenu.setSelectionHint`: the selected index only decides
    /// which offer the result slot mirrors. Steel matches the offer from the
    /// payment slots instead, so a selection is accepted but has no lasting
    /// effect — the screen still shows the offer the payment pays for.
    fn select_trade(&mut self, _behavior: &mut MenuBehavior, index: i32, _player: &Player) -> bool {
        let count = self.trades.lock().len();
        let Ok(index) = usize::try_from(index) else {
            return false;
        };
        index < count
    }

    fn on_open(
        &mut self,
        behavior: &mut MenuBehavior,
        _guard: &mut ContainerLockGuard,
        player: &Player,
    ) {
        // Vanilla sends the offers right after the screen packet, before the
        // slot contents: the client needs them to draw the trade list and to
        // work out which recipe the payment slots match.
        let offers = self.offers();
        let villager_level = 1;
        player.send_packet(CMerchantOffers {
            container_id: i32::from(behavior.container_id()),
            offers: offers.iter().map(offer_to_packet).collect(),
            villager_level,
            show_progress: villager_level > 0,
            can_restock: true,
        });
    }

    fn on_tick(
        &mut self,
        _behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        _player: &Player,
    ) {
        // Vanilla recomputes the result slot from the payment before syncing.
        // Without this the client would show whatever the server last wrote,
        // which goes stale as soon as the player pays with a different item.
        let container_id = self.trade_container.container_id();
        let Some(container) = guard.get(container_id) else {
            return;
        };
        let paid_a = container.get_item(COST_SLOT_A);
        let paid_b = container.get_item(COST_SLOT_B);

        let result = if paid_a.is_empty() {
            ItemStack::empty()
        } else {
            self.matching_offer(&paid_a, &paid_b)
                .and_then(|index| self.trades.lock().get(index).map(|offer| offer.result.clone()))
                .unwrap_or_else(ItemStack::empty)
        };

        let current = container.get_item(RESULT_SLOT);
        if !ItemStack::is_same_item_same_components(current, &result) || current.count() != result.count()
        {
            let _ = guard.set_item(container_id, RESULT_SLOT, result);
        }
    }

    fn on_slot_clicked(
        &mut self,
        _behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        click: Click,
        player: &Player,
    ) -> ClickOutcome {
        let Some(slot) = click.slot() else {
            return ClickOutcome::Fallthrough;
        };
        if slot != RESULT_SLOT {
            return ClickOutcome::Fallthrough;
        }

        // Vanilla `MerchantResultSlot.onTake`: taking the result spends the
        // payment and one use of the selected offer. Only a pickup or a
        // shift-click pays; throwing or swapping the result does not.
        let (Click::Pickup { .. } | Click::QuickMove { .. }) = click else {
            return ClickOutcome::Consume;
        };

        let container_id = self.trade_container.container_id();
        let Some(container) = guard.get(container_id) else {
            return ClickOutcome::Consume;
        };
        let paid_a = container.get_item(COST_SLOT_A);
        let paid_b = container.get_item(COST_SLOT_B);
        if paid_a.is_empty() {
            return ClickOutcome::Consume;
        }

        let Some(index) = self.matching_offer(&paid_a, &paid_b) else {
            return ClickOutcome::Consume;
        };

        // The trade is only paid for when the player can hold the result.
        let mut trades = self.trades.lock();
        let Some(offer) = trades.get_mut(index) else {
            return ClickOutcome::Consume;
        };
        let result = offer.result.clone();
        let cost_a = offer.adjusted_cost_a();
        let cost_b = offer.base_cost_b.clone();

        let mut inventory = player.inventory.lock();
        let leftover = inventory.add_or_return(result.clone());
        if !leftover.is_empty() {
            return ClickOutcome::Consume;
        }
        drop(inventory);

        offer.record_use();
        drop(trades);

        let _ = guard.remove_item(container_id, COST_SLOT_A, cost_a.count());
        if !cost_b.is_empty() {
            let _ = guard.remove_item(container_id, COST_SLOT_B, cost_b.count());
        }

        // Vanilla `MerchantResultSlot.onTake` awards `Stats.TRADED_WITH_VILLAGER`
        // and fires `CriteriaTriggers.TRADE_WITH_VILLAGER` (`villager_trade`),
        // which `minecraft:adventure/trade` listens for.
        player.award_custom_stat(&steel_registry::vanilla_custom_stats::TRADED_WITH_VILLAGER);
        player.trigger_advancement_with_context(
            &steel_registry::vanilla_criteria_triggers::VILLAGER_TRADE,
            &crate::player::advancement::predicate::TriggerContext::item(&result.item().key),
        );
        ClickOutcome::Consume
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_merchant_menu_has_three_trade_slots_above_the_inventory() {
        assert_eq!(vanilla_menu_types::MERCHANT.slot_count, 39);
        assert_eq!(TRADE_SLOTS + 27 + 9, 39);
    }

    #[test]
    fn an_empty_trade_list_still_builds_a_menu() {
        let inventory = PlayerInventory::new().into_shared();
        let trades = Arc::new(SyncMutex::new(Vec::new()));
        let menu = merchant(inventory, 0, trades);
        assert_eq!(menu.slot_count(), 39);
    }

    #[test]
    fn a_payment_must_cover_the_whole_cost() {
        use steel_registry::vanilla_items;
        let cost = ItemStack::with_count(&*vanilla_items::EMERALD, 5);
        let exact = ItemStack::with_count(&*vanilla_items::EMERALD, 5);
        let short = ItemStack::with_count(&*vanilla_items::EMERALD, 4);
        let other = ItemStack::with_count(&*vanilla_items::WHEAT, 5);

        assert!(matches_payment(&cost, &exact));
        assert!(!matches_payment(&cost, &short));
        assert!(!matches_payment(&cost, &other));
        assert!(!matches_payment(&cost, &ItemStack::empty()));
    }

    #[test]
    fn an_offer_without_a_second_cost_ignores_the_second_slot() {
        let trades = Arc::new(SyncMutex::new(crate::entity::villager_trades::offers_for(
            "minecraft:farmer",
        )));
        let container: ContainerRef = SimpleContainer::new(TRADE_SLOTS).into_shared().into();
        let kind = MerchantKind::new(trades, container);

        let wheat = ItemStack::with_count(&steel_registry::vanilla_items::WHEAT, 20);
        // The first farmer trade is 20 wheat for an emerald and takes no second
        // payment, so a filled second slot must not block it.
        let bogus_second = ItemStack::with_count(&steel_registry::vanilla_items::STICK, 1);
        assert_eq!(kind.matching_offer(&wheat, &bogus_second), Some(0));
    }
}
