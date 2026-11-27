//! ViaVersion интеграция — трансляция пакетов между версиями клиента и сервера.
//! Ядро вызывает `translate_inbound`/`translate_outbound` в `steel-login` / `player::connection`.

#![allow(missing_docs, reason = "plugin system has extensive docs in STEEL-API.md")]
#![allow(clippy::missing_panics_doc, reason = "plugin manager panics are documented in API")]
use std::sync::Arc;
use steel_api::ViaPipeline;
use steel_api::protocol::{ProtocolVersion, ViaError};

/// Обёртка над `ViaPipeline` для ядра — держит `server_version` и текущий `ViaPipeline`.
pub struct ViaBridge {
    pipeline: Arc<ViaPipeline>,
    server_version: ProtocolVersion,
}

impl ViaBridge {
    pub fn new(pipeline: Arc<ViaPipeline>, server_version: ProtocolVersion) -> Self {
        Self { pipeline, server_version }
    }

    pub fn server_version(&self) -> ProtocolVersion { self.server_version }

    /// Перевести входящий пакет от клиента `client_version` к серверной версии.
    pub fn inbound(&self, client_version: ProtocolVersion, id: i32, data: Vec<u8>) -> Result<(i32, Vec<u8>), ViaError> {
        self.pipeline.translate_inbound(client_version, self.server_version, id, data)
    }

    /// Исходящий: сервер -> клиент.
    pub fn outbound(&self, client_version: ProtocolVersion, id: i32, data: Vec<u8>) -> Result<(i32, Vec<u8>), ViaError> {
        self.pipeline.translate_outbound(self.server_version, client_version, id, data)
    }

    /// Клиент поддерживается?
    pub fn is_supported(&self, client_version: ProtocolVersion) -> bool {
        self.pipeline.is_supported(client_version, self.server_version)
    }

    /// Зарегистрировать кастомный транслятор (плагин Via-расширение).
    pub fn register_translator(&self, t: Arc<dyn steel_api::protocol::ProtocolTranslator>) {
        self.pipeline.register(t);
    }
}

impl Default for ViaBridge {
    fn default() -> Self {
        Self { pipeline: Arc::new(ViaPipeline::new()), server_version: ProtocolVersion::V26_3 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn passthrough_same_version() {
        let b = ViaBridge::default();
        let (id, data) = b.inbound(ProtocolVersion::V26_3, 1, vec![0xAA]).unwrap();
        assert_eq!(id, 1);
        assert_eq!(data, vec![0xAA]);
    }
    #[test]
    fn reject_unknown_client() {
        let b = ViaBridge::default();
        let r = b.inbound(ProtocolVersion::new(999), 1, vec![]);
        assert!(r.is_err());
    }
}
