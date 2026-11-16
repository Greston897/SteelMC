//! Custom dialog display packet (`ClientboundShowDialogPacket`).
//!
//! Vanilla uses `Dialog.STREAM_CODEC` (`ByteBufCodecs.holder(Registries.DIALOG,
//! ByteBufCodecs.fromCodecWithRegistriesTrusted(Dialog.DIRECT_CODEC))`):
//! - Configuration state (`CONTEXT_FREE_STREAM_CODEC`): direct unnamed NBT compound.
//! - Play state (`STREAM_CODEC` `Holder<Dialog>`):
//!   - `RegistryId(id)` → `VarInt(id + 1)`
//!   - `Direct(nbt)` → `VarInt(0)` followed by unnamed network NBT compound (`0x0A ... 0x00`).

use std::io::Write;

use simdnbt::owned::NbtCompound;
use steel_macros::ClientPacket;
use steel_registry::packets::{config, play};
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// Encodes a vanilla `Holder<Dialog>` on the play connection.
#[derive(Debug, Clone)]
pub enum DialogHolder {
    /// Reference to a synchronized `minecraft:dialog` registry entry (`VarInt(id + 1)`).
    RegistryId(i32),
    /// Inline `Dialog` definition serialized as a network NBT compound (`VarInt(0)` + NBT).
    Direct(NbtCompound),
}

impl DialogHolder {
    /// Writes an unnamed network NBT compound (`TAG_Compound = 0x0A`, fields, `TAG_End = 0x00`).
    pub fn write_nbt_compound(compound: &NbtCompound, writer: &mut impl Write) -> std::io::Result<()> {
        let mut bytes = Vec::new();
        bytes.push(0x0A);
        compound.write(&mut bytes);
        writer.write_all(&bytes)
    }
}

/// Clientbound play packet that opens a server-defined dialog (`Holder<Dialog>`).
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Play = play::C_SHOW_DIALOG)]
pub struct CShowDialog {
    /// Dialog holder (registry id or inline NBT definition).
    pub dialog: DialogHolder,
}

impl CShowDialog {
    /// Builds a simple inline `minecraft:notice` dialog with a title and plain body text.
    #[must_use]
    pub fn notice(title: &str, body: &str) -> Self {
        let mut root = NbtCompound::new();
        root.insert("type", "minecraft:notice");
        root.insert("title", title);
        if !body.is_empty() {
            let mut body_tag = NbtCompound::new();
            body_tag.insert("type", "minecraft:plain_message");
            body_tag.insert("contents", body);
            root.insert("body", body_tag);
        }
        Self {
            dialog: DialogHolder::Direct(root),
        }
    }
}

impl WriteTo for CShowDialog {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match &self.dialog {
            DialogHolder::RegistryId(id) => VarInt(id.saturating_add(1)).write(writer),
            DialogHolder::Direct(compound) => {
                VarInt(0).write(writer)?;
                DialogHolder::write_nbt_compound(compound, writer)
            }
        }
    }
}

/// Clientbound configuration packet (`ClientboundShowDialogPacket.CONTEXT_FREE_STREAM_CODEC`),
/// which writes the inline dialog NBT directly without a `Holder` id prefix.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Config = config::C_SHOW_DIALOG)]
pub struct CConfigShowDialog {
    /// Inline dialog NBT compound.
    pub dialog: NbtCompound,
}

impl WriteTo for CConfigShowDialog {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        DialogHolder::write_nbt_compound(&self.dialog, writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_holder_writes_id_plus_one() {
        let pkt = CShowDialog {
            dialog: DialogHolder::RegistryId(4),
        };
        let mut bytes = Vec::new();
        pkt.write(&mut bytes).unwrap();
        assert_eq!(bytes, vec![5]);
    }

    #[test]
    fn inline_notice_writes_zero_holder_then_network_nbt_compound() {
        let pkt = CShowDialog::notice("Rules", "Welcome!");
        let mut bytes = Vec::new();
        pkt.write(&mut bytes).unwrap();
        assert_eq!(bytes[0], 0); // Holder direct discriminator
        assert_eq!(bytes[1], 0x0A); // TAG_Compound (unnamed network NBT)
        assert_eq!(*bytes.last().unwrap(), 0x00); // TAG_End
    }
}
