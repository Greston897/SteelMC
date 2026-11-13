//! Merchant offers packet (villager and wandering trader trade lists).

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::item_stack::ItemStack;
use steel_registry::packets::play::C_MERCHANT_OFFERS;
use steel_utils::codec::VarInt;

/// A single trade slot pair and its rewards.
#[derive(WriteTo, Clone, Debug)]
pub struct MerchantOffer {
    /// The item the player pays with.
    pub base_cost_a: ItemStack,
    /// The optional second payment item.
    pub base_cost_b: ItemStack,
    /// The item the player receives.
    pub result: ItemStack,
    /// Whether the trade is currently out of stock.
    pub out_of_stock: bool,
    /// Remaining uses before the trade locks.
    #[write(as = VarInt)]
    pub uses: i32,
    /// Maximum uses.
    #[write(as = VarInt)]
    pub max_uses: i32,
    /// Experience granted per use.
    #[write(as = VarInt)]
    pub xp: i32,
    /// Price multiplier in percent (`100` = no change).
    #[write(as = VarInt)]
    pub special_price_diff: i32,
    /// Demand, which raises the price over time.
    #[write(as = VarInt)]
    pub demand: i32,
}

/// Clientbound packet sent when a merchant screen opens.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_MERCHANT_OFFERS)]
pub struct CMerchantOffers {
    /// Open container id.
    #[write(as = VarInt)]
    pub container_id: i32,
    /// Available trades.
    #[write(as = Prefixed(VarInt))]
    pub offers: Vec<MerchantOffer>,
    /// Villager level shown in the header.
    #[write(as = VarInt)]
    pub villager_level: i32,
    /// Whether experience progress is shown.
    pub show_progress: bool,
    /// Whether the merchant is currently restocking.
    pub can_restock: bool,
}
