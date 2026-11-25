//! Code of conduct prompt packet (`ClientboundCodeOfConductPacket`).
//!
//! Vanilla wire format (`ByteBufCodecs.STRING_UTF8`): a single non-optional UTF-8
//! string containing the server's code of conduct text. The client replies with
//! [`crate::packets::config::SAcceptCodeOfConduct`].

use steel_macros::{ClientPacket, WriteTo};
use steel_registry::packets::config::C_CODE_OF_CONDUCT;

/// Clientbound configuration packet presenting the server code of conduct.
#[derive(ClientPacket, WriteTo, Debug, Clone, PartialEq, Eq)]
#[packet_id(Config = C_CODE_OF_CONDUCT)]
pub struct CCodeOfConduct {
    /// Code of conduct text (`ByteBufCodecs.STRING_UTF8`).
    pub code_of_conduct: String,
}

#[cfg(test)]
mod tests {
    use super::CCodeOfConduct;
    use steel_utils::serial::WriteTo as _;

    #[test]
    fn encodes_plain_utf8_without_optional_flag() {
        let pkt = CCodeOfConduct {
            code_of_conduct: "Be kind".to_owned(),
        };
        let mut bytes = Vec::new();
        pkt.write(&mut bytes).unwrap();
        assert_eq!(bytes, b"\x07Be kind");
    }
}
