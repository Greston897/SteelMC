//! NBT-API — прямой доступ к `CompoundTag`, `ListTag`, энтити/item/world NBT.
//! Как Paper `PersistentDataContainer` + NMS `CompoundTag`, но безопасно.
//! Плагин может читать/писать любые теги, не ломая ваниль.

use rustc_hash::FxHashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum NbtValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List(Vec<NbtValue>),
    Compound(FxHashMap<String, NbtValue>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl NbtValue {
    pub fn as_byte(&self) -> Option<i8> { if let Self::Byte(v)=self { Some(*v)} else { None } }
    pub fn as_int(&self) -> Option<i32> { if let Self::Int(v)=self { Some(*v)} else { None } }
    pub fn as_string(&self) -> Option<&str> { if let Self::String(s)=self { Some(s)} else { None } }
    pub fn as_compound(&self) -> Option<&FxHashMap<String, NbtValue>> { if let Self::Compound(m)=self { Some(m)} else { None } }
    pub fn as_list(&self) -> Option<&Vec<NbtValue>> { if let Self::List(v)=self { Some(v)} else { None } }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompoundTag {
    inner: FxHashMap<String, NbtValue>,
}

impl CompoundTag {
    pub fn new() -> Self { Self::default() }
    pub fn put(&mut self, key: impl Into<String>, value: NbtValue) { self.inner.insert(key.into(), value); }
    pub fn put_byte(&mut self, k: impl Into<String>, v: i8) { self.put(k, NbtValue::Byte(v)); }
    pub fn put_int(&mut self, k: impl Into<String>, v: i32) { self.put(k, NbtValue::Int(v)); }
    pub fn put_string(&mut self, k: impl Into<String>, v: impl Into<String>) { self.put(k, NbtValue::String(v.into())); }
    pub fn put_compound(&mut self, k: impl Into<String>, v: CompoundTag) { self.put(k, NbtValue::Compound(v.inner)); }
    pub fn get(&self, key: &str) -> Option<&NbtValue> { self.inner.get(key) }
    pub fn get_byte(&self, k: &str) -> Option<i8> { self.get(k)?.as_byte() }
    pub fn get_int(&self, k: &str) -> Option<i32> { self.get(k)?.as_int() }
    pub fn get_string(&self, k: &str) -> Option<&str> { self.get(k)?.as_string() }
    pub fn get_compound(&self, k: &str) -> Option<CompoundTag> { self.get(k)?.as_compound().map(|m| CompoundTag { inner: m.clone() }) }
    pub fn contains(&self, k: &str) -> bool { self.inner.contains_key(k) }
    pub fn remove(&mut self, k: &str) -> Option<NbtValue> { self.inner.remove(k) }
    pub fn is_empty(&self) -> bool { self.inner.is_empty() }
    pub fn len(&self) -> usize { self.inner.len() }
    pub fn keys(&self) -> Vec<String> { self.inner.keys().cloned().collect() }
    pub fn to_bytes(&self) -> Vec<u8> { Vec::new() } // в ядре — реальный сериализатор
    pub fn from_bytes(_b: &[u8]) -> Option<Self> { Some(Self::default()) }
    pub fn merge(&mut self, other: &CompoundTag) { for (k,v) in &other.inner { self.inner.insert(k.clone(), v.clone()); } }
}

/// Трейт для энтити с NBT — как `Entity#save`/`load` в NMS.
pub trait NbtHolder: Send + Sync {
    fn get_nbt(&self) -> CompoundTag;
    fn set_nbt(&self, tag: CompoundTag) -> bool;
    fn has_tag(&self, key: &str) -> bool { self.get_nbt().contains(key) }
}

/// PDC — `PersistentDataContainer` как в Paper.
#[derive(Debug, Clone, Default)]
pub struct PersistentDataContainer {
    tag: CompoundTag,
}

impl PersistentDataContainer {
    pub fn new() -> Self { Self::default() }
    pub fn set<T: Into<NbtValue>>(&mut self, key: impl Into<String>, value: T) { self.tag.put(key, value.into()); }
    pub fn get(&self, key: &str) -> Option<&NbtValue> { self.tag.get(key) }
    pub fn has(&self, key: &str) -> bool { self.tag.contains(key) }
    pub fn remove(&mut self, key: &str) { self.tag.remove(key); }
    pub fn keys(&self) -> Vec<String> { self.tag.keys() }
}

impl From<i32> for NbtValue { fn from(v: i32) -> Self { Self::Int(v) } }
impl From<String> for NbtValue { fn from(v: String) -> Self { Self::String(v) } }
impl From<&str> for NbtValue { fn from(v: &str) -> Self { Self::String(v.to_string()) } }
