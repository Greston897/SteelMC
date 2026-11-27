//! Метадата-API — `Metadatable`, `PersistentDataContainer`, `MetadataValue` как в Paper.
//! Плагин вешает `setMetadata("myplugin:flag", FixedMetadataValue(true))` на любую сущность/блок/мир.


use steel_utils::locks::SyncRwLock;
use rustc_hash::FxHashMap;

#[derive(Debug, Clone)]
pub enum MetadataValue {
    Bool(bool),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Bytes(Vec<u8>),
}

impl From<bool> for MetadataValue { fn from(v: bool) -> Self { Self::Bool(v) } }
impl From<i32> for MetadataValue { fn from(v: i32) -> Self { Self::Int(v) } }
impl From<String> for MetadataValue { fn from(v: String) -> Self { Self::String(v) } }
impl From<&str> for MetadataValue { fn from(v: &str) -> Self { Self::String(v.to_string()) } }

/// Хранилище метадаты — на энтити, блок, мир, игрок.
#[derive(Default)]
pub struct MetadataStore {
    map: SyncRwLock<FxHashMap<String, FxHashMap<String, MetadataValue>>>, // owner -> (key -> value)
}

impl MetadataStore {
    pub fn new() -> Self { Self::default() }
    pub fn set(&self, owner: &str, key: impl Into<String>, value: MetadataValue) {
        let mut w=self.map.write();
        w.entry(owner.to_string()).or_default().insert(key.into(), value);
    }
    pub fn get(&self, owner: &str, key: &str) -> Option<MetadataValue> { self.map.read().get(owner)?.get(key).cloned() }
    pub fn has(&self, owner: &str, key: &str) -> bool { self.map.read().get(owner).is_some_and(|m| m.contains_key(key)) }
    pub fn remove(&self, owner: &str, key: &str) -> bool { self.map.write().get_mut(owner).is_some_and(|m| m.remove(key).is_some()) }
    pub fn clear(&self, owner: &str) { self.map.write().remove(owner); }
    pub fn keys(&self, owner: &str) -> Vec<String> { self.map.read().get(owner).map(|m| m.keys().cloned().collect()).unwrap_or_default() }
}

pub trait Metadatable: Send + Sync {
    fn metadata_store(&self) -> &MetadataStore;
    fn set_metadata(&self, key: impl Into<String>, value: impl Into<MetadataValue>) { self.metadata_store().set(&self.metadata_key(), key, value.into()); }
    fn get_metadata(&self, key: &str) -> Option<MetadataValue> { self.metadata_store().get(&self.metadata_key(), key) }
    fn has_metadata(&self, key: &str) -> bool { self.metadata_store().has(&self.metadata_key(), key) }
    fn remove_metadata(&self, key: &str) -> bool { self.metadata_store().remove(&self.metadata_key(), key) }
    fn metadata_key(&self) -> String;
}

/// PDC-трейт — уже есть в `nbt.rs`, ре-экспорт для удобства.
pub use crate::nbt::PersistentDataContainer;
