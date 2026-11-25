//! Merchant offers packet (`ClientboundMerchantOffersPacket`).
//!
//! Vanilla wire format:
//! - `containerId`: `ByteBufCodecs.CONTAINER_ID` (`VarInt`)
//! - `offers`: `VarInt`-prefixed list of `MerchantOffer`:
//!   1. `baseCostA`: `ItemCost` (`VarInt item_id`, `VarInt count`, `DataComponentExactPredicate`)
//!   2. `result`: `ItemStack`
//!   3. `costB`: `Optional<ItemCost>` (`bool` + `ItemCost`)
//!   4. `outOfStock`: `bool`
//!   5. `uses`: `i32`
//!   6. `maxUses`: `i32`
//!   7. `xp`: `i32`
//!   8. `specialPriceDiff`: `i32`
//!   9. `priceMultiplier`: `f32`
//!   10. `demand`: `i32`
//! - `villagerLevel`: `VarInt`
//! - `villagerXp`: `VarInt`
//! - `showProgress`: `bool`
//! - `canRestock`: `bool`

use std::io::Write;

use steel_macros::ClientPacket;
use steel_registry::RegistryEntry;
use steel_registry::item_stack::ItemStack;
use steel_registry::packets::play::C_MERCHANT_OFFERS;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// Encodes a non-empty `ItemStack` as vanilla `ItemCost.STREAM_CODEC`.
fn write_item_cost(stack: &ItemStack, writer: &mut impl Write) -> std::io::Result<()> {
    let item_id = i32::try_from(stack.item().id())
        .map_err(|_| std::io::Error::other("merchant cost item id exceeds i32::MAX"))?;
    VarInt(item_id).write(writer)?;
    VarInt(stack.count()).write(writer)?;
    stack.patch().write_exact_predicate(writer)
}

/// One trade on a villager or wandering trader (`MerchantOffer`).
#[derive(Debug, Clone)]
pub struct MerchantOffer {
    /// First payment item (`ItemCost`, must be non-empty).
    pub base_cost_a: ItemStack,
    /// Result stack handed out on trade completion.
    pub result: ItemStack,
    /// Optional second payment item (`Optional<ItemCost>`; empty stack encodes as `None`).
    pub cost_b: ItemStack,
    /// Whether the offer is temporarily sold out.
    pub out_of_stock: bool,
    /// Number of times the offer has been used.
    pub uses: i32,
    /// Maximum uses before the trade locks.
    pub max_uses: i32,
    /// Villager XP granted per trade.
    pub xp: i32,
    /// Special price adjustment (in items, added to `base_cost_a`).
    pub special_price_diff: i32,
    /// Demand / hero-of-the-village multiplier (`0.05` default in vanilla).
    pub price_multiplier: f32,
    /// Current demand adjustment.
    pub demand: i32,
}

impl WriteTo for MerchantOffer {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write_item_cost(&self.base_cost_a, writer)?;
        self.result.write(writer)?;
        if self.cost_b.is_empty() {
            false.write(writer)?;
        } else {
            true.write(writer)?;
            write_item_cost(&self.cost_b, writer)?;
        }
        self.out_of_stock.write(writer)?;
        self.uses.write(writer)?;
        self.max_uses.write(writer)?;
        self.xp.write(writer)?;
        self.special_price_diff.write(writer)?;
        self.price_multiplier.write(writer)?;
        self.demand.write(writer)
    }
}

/// Clientbound packet that populates the trade list of a merchant screen.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Play = C_MERCHANT_OFFERS)]
pub struct CMerchantOffers {
    /// Container id of the merchant screen.
    pub container_id: i32,
    /// Offers displayed in order.
    pub offers: Vec<MerchantOffer>,
    /// Villager profession level (`1..=5`, `1` for wandering trader).
    pub villager_level: i32,
    /// Villager total trade experience (`0` for wandering trader).
    pub villager_xp: i32,
    /// Whether the XP bar is drawn (false for wandering traders).
    pub show_progress: bool,
    /// Whether the merchant can restock at a workstation.
    pub can_restock: bool,
}

impl WriteTo for CMerchantOffers {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        VarInt(self.container_id).write(writer)?;
        self.offers.write(writer)?;
        VarInt(self.villager_level).write(writer)?;
        VarInt(self.villager_xp).write(writer)?;
        self.show_progress.write(writer)?;
        self.can_restock.write(writer)
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_items};

    use super::*;

    #[test]
    fn encodes_merchant_offers_in_exact_vanilla_order() {
        init_vanilla_registry();
        let wheat = ItemStack::with_count(&vanilla_items::WHEAT, 20);
        let emerald = ItemStack::with_count(&vanilla_items::EMERALD, 1);
        let pkt = CMerchantOffers {
            container_id: 3,
            offers: vec![MerchantOffer {
                base_cost_a: wheat.clone(),
                result: emerald.clone(),
                cost_b: ItemStack::empty(),
                out_of_stock: false,
                uses: 1,
                max_uses: 16,
                xp: 2,
                special_price_diff: -3,
                price_multiplier: 0.05,
                demand: 4,
            }],
            villager_level: 2,
            villager_xp: 15,
            show_progress: true,
            can_restock: true,
        };

        let mut actual = Vec::new();
        pkt.write(&mut actual).unwrap();

        let mut expected = Vec::new();
        VarInt(3).write(&mut expected).unwrap();
        VarInt(1).write(&mut expected).unwrap();
        // ItemCost A: item id, count, 0 expected components
        VarInt(wheat.item().id() as i32)
            .write(&mut expected)
            .unwrap();
        VarInt(20).write(&mut expected).unwrap();
        VarInt(0).write(&mut expected).unwrap();
        // Result ItemStack
        emerald.write(&mut expected).unwrap();
        // costB absent
        false.write(&mut expected).unwrap();
        // outOfStock, uses, maxUses, xp, specialPriceDiff, priceMultiplier, demand
        false.write(&mut expected).unwrap();
        1_i32.write(&mut expected).unwrap();
        16_i32.write(&mut expected).unwrap();
        2_i32.write(&mut expected).unwrap();
        (-3_i32).write(&mut expected).unwrap();
        0.05_f32.write(&mut expected).unwrap();
        4_i32.write(&mut expected).unwrap();
        // villagerLevel, villagerXp, showProgress, canRestock
        VarInt(2).write(&mut expected).unwrap();
        VarInt(15).write(&mut expected).unwrap();
        true.write(&mut expected).unwrap();
        true.write(&mut expected).unwrap();

        assert_eq!(actual, expected);
    }
}
