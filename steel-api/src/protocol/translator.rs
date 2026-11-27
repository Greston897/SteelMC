//! Транслятор пакетов — сердце ViaVersion. Плагины могут регистрировать свои.

use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};
use steel_utils::locks::SyncRwLock;
use thiserror::Error;

use super::version::ProtocolVersion;

pub type PacketId = i32;
pub type PacketData = Vec<u8>;

#[derive(Debug, Error)]
pub enum ViaError {
    #[error("нет пути перевода {from} -> {to}")]
    NoPath { from: i32, to: i32 },
    #[error("транслятор вернул ошибку: {0}")]
    Translator(String),
    #[error("пакет {0} неизвестен для версии {1}")]
    UnknownPacket(i32, i32),
}

pub type TranslationResult = Result<(PacketId, PacketData), ViaError>;

/// Трейт одного шага перевода — `from -> to` (соседние версии).
/// Аналог `Protocol` в ViaVersion.
pub trait ProtocolTranslator: Send + Sync + 'static {
    fn from(&self) -> ProtocolVersion;
    fn to(&self) -> ProtocolVersion;
    fn name(&self) -> &'static str { "custom" }

    /// Перевод входящего (client -> server). По умолчанию — passthrough.
    fn translate_inbound(&self, id: PacketId, data: PacketData) -> TranslationResult {
        Ok((id, data))
    }
    /// Исходящего (server -> client).
    fn translate_outbound(&self, id: PacketId, data: PacketData) -> TranslationResult {
        Ok((id, data))
    }

    /// Ремап id пакета (если протокол поменял нумерацию). По умолчанию — тот же.
    fn remap_inbound_id(&self, id: PacketId) -> PacketId { id }
    fn remap_outbound_id(&self, id: PacketId) -> PacketId { id }
}

/// No-op транслятор — для тестов.
pub struct NoopTranslator {
    from: ProtocolVersion,
    to: ProtocolVersion,
}
impl NoopTranslator {
    pub fn new(from: ProtocolVersion, to: ProtocolVersion) -> Self { Self { from, to } }
}
impl ProtocolTranslator for NoopTranslator {
    fn from(&self) -> ProtocolVersion { self.from }
    fn to(&self) -> ProtocolVersion { self.to }
    fn name(&self) -> &'static str { "noop" }
}

/// Конвейер Via — хранит граф трансляторов и находит путь `client_version -> server_version`.
///
pub struct ViaPipeline {
    translators: SyncRwLock<FxHashMap<(i32, i32), Arc<dyn ProtocolTranslator>>>,
}

impl Default for ViaPipeline {
    fn default() -> Self { Self::new() }
}

impl ViaPipeline {
    pub fn new() -> Self {
        let p = Self { translators: SyncRwLock::new(FxHashMap::default()) };
        // дефолтные passthrough между соседними известными версиями
        for w in ProtocolVersion::ALL.windows(2) {
            let a = w[0]; let b = w[1];
            p.register(Arc::new(NoopTranslator::new(a, b)));
            p.register(Arc::new(NoopTranslator::new(b, a)));
        }
        p
    }

    pub fn register(&self, t: Arc<dyn ProtocolTranslator>) {
        let key = (t.from().0, t.to().0);
        self.translators.write().insert(key, t);
    }

    pub fn unregister(&self, from: ProtocolVersion, to: ProtocolVersion) -> bool {
        self.translators.write().remove(&(from.0, to.0)).is_some()
    }

    pub fn has_translator(&self, from: ProtocolVersion, to: ProtocolVersion) -> bool {
        self.translators.read().contains_key(&(from.0, to.0))
    }

    /// Найти путь BFS `from -> to` через зарегистрированные рёбра.
    fn find_path(&self, from: i32, to: i32) -> Option<Vec<Arc<dyn ProtocolTranslator>>> {
        if from == to { return Some(Vec::new()); }
        let map = self.translators.read();
        // BFS
        use std::collections::VecDeque;
        let mut queue = VecDeque::new();
        let mut visited = FxHashSet::default();
        let mut prev: FxHashMap<i32, (i32, Arc<dyn ProtocolTranslator>)> = FxHashMap::default();
        queue.push_back(from);
        visited.insert(from);
        while let Some(cur) = queue.pop_front() {
            if cur == to { break; }
            for ((f, t), tr) in map.iter() {
                if *f == cur && !visited.contains(t) {
                    visited.insert(*t);
                    prev.insert(*t, (cur, tr.clone()));
                    queue.push_back(*t);
                }
            }
        }
        if !visited.contains(&to) { return None; }
        // восстановить путь
        let mut path = Vec::new();
        let mut cur = to;
        while cur != from {
            let (p, tr) = prev.get(&cur)?.clone();
            path.push(tr);
            cur = p;
        }
        path.reverse();
        Some(path)
    }

    /// Перевести входящий пакет от клиента `client_version` к версии сервера `server_version`.
    pub fn translate_inbound(&self, client_version: ProtocolVersion, server_version: ProtocolVersion, id: PacketId, data: PacketData) -> TranslationResult {
        let path = self.find_path(client_version.0, server_version.0).ok_or(ViaError::NoPath { from: client_version.0, to: server_version.0 })?;
        let mut cur_id = id;
        let mut cur_data = data;
        for tr in path {
            cur_id = tr.remap_inbound_id(cur_id);
            (cur_id, cur_data) = tr.translate_inbound(cur_id, cur_data)?;
        }
        Ok((cur_id, cur_data))
    }

    pub fn translate_outbound(&self, server_version: ProtocolVersion, client_version: ProtocolVersion, id: PacketId, data: PacketData) -> TranslationResult {
        let path = self.find_path(server_version.0, client_version.0).ok_or(ViaError::NoPath { from: server_version.0, to: client_version.0 })?;
        let mut cur_id = id;
        let mut cur_data = data;
        for tr in path {
            cur_id = tr.remap_outbound_id(cur_id);
            (cur_id, cur_data) = tr.translate_outbound(cur_id, cur_data)?;
        }
        Ok((cur_id, cur_data))
    }

    /// Совместим ли клиент?
    pub fn is_supported(&self, client_version: ProtocolVersion, server_version: ProtocolVersion) -> bool {
        self.find_path(client_version.0, server_version.0).is_some()
    }

    pub fn translator_count(&self) -> usize { self.translators.read().len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::version::ProtocolVersion;

    struct AddOneTranslator { from: ProtocolVersion, to: ProtocolVersion }
    impl ProtocolTranslator for AddOneTranslator {
        fn from(&self) -> ProtocolVersion { self.from }
        fn to(&self) -> ProtocolVersion { self.to }
        fn translate_inbound(&self, id: PacketId, mut data: PacketData) -> TranslationResult {
            data.push(1);
            Ok((id, data))
        }
        fn translate_outbound(&self, id: PacketId, mut data: PacketData) -> TranslationResult {
            if data.last() == Some(&1) { data.pop(); }
            Ok((id, data))
        }
    }

    #[test]
    fn pipeline_passthrough() {
        let p = ViaPipeline::new();
        let (id, data) = p.translate_inbound(ProtocolVersion::V26_3, ProtocolVersion::V26_3, 5, vec![1,2]).unwrap();
        assert_eq!(id, 5);
        assert_eq!(data, vec![1,2]);
    }

    #[test]
    fn pipeline_custom() {
        let p = ViaPipeline::new();
        p.register(Arc::new(AddOneTranslator { from: ProtocolVersion::V1_21_8, to: ProtocolVersion::V26_3 }));
        p.register(Arc::new(AddOneTranslator { from: ProtocolVersion::V26_3, to: ProtocolVersion::V1_21_8 }));
        let (id, data) = p.translate_inbound(ProtocolVersion::V1_21_8, ProtocolVersion::V26_3, 10, vec![9]).unwrap();
        assert_eq!(data, vec![9,1]);
        let (id2, data2) = p.translate_outbound(ProtocolVersion::V26_3, ProtocolVersion::V1_21_8, id, data).unwrap();
        assert_eq!(data2, vec![9]);
        let _ = id2;
    }

    #[test]
    fn via_reject_unknown() {
        let p = ViaPipeline::new();
        let r = p.translate_inbound(ProtocolVersion::new(999), ProtocolVersion::V26_3, 1, vec![]);
        assert!(r.is_err());
    }
}
