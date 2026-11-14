//! Enabled features packet.

use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::PrefixedWrite as _;
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::config::C_UPDATE_ENABLED_FEATURES;
use steel_utils::Identifier;

/// Clientbound packet that lists the active vanilla feature flags.
#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Config = C_UPDATE_ENABLED_FEATURES)]
pub struct CUpdateEnabledFeatures {
    /// Enabled feature flag identifiers.
    #[write(as = Prefixed(VarInt))]
    pub features: Vec<Identifier>,
}
