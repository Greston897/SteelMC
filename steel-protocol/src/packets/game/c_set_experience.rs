use steel_macros::{ClientPacket, WriteTo};
use steel_utils::serial::WriteTo as _;
use steel_registry::packets::play::C_SET_EXPERIENCE;

#[derive(ClientPacket, WriteTo, Clone, Debug)]
#[packet_id(Play = C_SET_EXPERIENCE)]
pub struct CSetExperience {
    pub progress: f32,
    #[write(as = VarInt)]
    pub level: i32,
    #[write(as = VarInt)]
    pub total_experience: i32,
}
