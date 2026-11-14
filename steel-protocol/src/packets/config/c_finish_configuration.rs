use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::config::C_FINISH_CONFIGURATION;

#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Config = C_FINISH_CONFIGURATION)]
pub struct CFinishConfiguration {}
