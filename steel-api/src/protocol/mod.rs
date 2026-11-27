//! ViaVersion — трансляция протоколов. Своя, встроенная, без внешних jar.

pub mod translator;
pub mod version;

pub use translator::{ViaPipeline, ProtocolTranslator, ViaError, PacketId, PacketData, TranslationResult};
pub use version::ProtocolVersion;
