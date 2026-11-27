//! Тэг-API — `Tag<Block>`, `Tag<Item>`, `Tag<EntityType>` как в Paper `Tag` + `Registry`.
//! Плагин читает ваниль-тэги (`minecraft:logs`, `minecraft:wool`) и создаёт свои.

use steel_utils::Identifier;
use rustc_hash::FxHashSet;

#[derive(Debug, Clone)]
pub struct Tag<T> {
    pub key: Identifier,
    pub values: FxHashSet<T>,
}

impl<T: std::hash::Hash + Eq + Clone> Tag<T> {
    pub fn new(key: Identifier) -> Self { Self { key, values: FxHashSet::default() } }
    pub fn contains(&self, v: &T) -> bool { self.values.contains(v) }
    pub fn add(&mut self, v: T) { self.values.insert(v); }
    pub fn remove(&mut self, v: &T) -> bool { self.values.remove(v) }
    pub fn len(&self) -> usize { self.values.len() }
    pub fn is_empty(&self) -> bool { self.values.is_empty() }
    pub fn iter(&self) -> impl Iterator<Item=&T> { self.values.iter() }
}

/// Реестр тэгов — как `Bukkit.getTag`.
pub struct TagRegistry {
    block_tags: steel_utils::locks::SyncRwLock<rustc_hash::FxHashMap<Identifier, Tag<Identifier>>>,
    item_tags: steel_utils::locks::SyncRwLock<rustc_hash::FxHashMap<Identifier, Tag<Identifier>>>,
    entity_tags: steel_utils::locks::SyncRwLock<rustc_hash::FxHashMap<Identifier, Tag<Identifier>>>,
    fluid_tags: steel_utils::locks::SyncRwLock<rustc_hash::FxHashMap<Identifier, Tag<Identifier>>>,
}

impl Default for TagRegistry { fn default() -> Self { Self::new() } }

impl TagRegistry {
    pub fn new() -> Self {
        Self {
            block_tags: steel_utils::locks::SyncRwLock::new(Default::default()),
            item_tags: steel_utils::locks::SyncRwLock::new(Default::default()),
            entity_tags: steel_utils::locks::SyncRwLock::new(Default::default()),
            fluid_tags: steel_utils::locks::SyncRwLock::new(Default::default()),
        }
    }

    pub fn register_block_tag(&self, tag: Tag<Identifier>) -> bool {
        let mut w=self.block_tags.write();
        if w.contains_key(&tag.key) { return false; }
        w.insert(tag.key.clone(), tag); true
    }
    pub fn get_block_tag(&self, key: &Identifier) -> Option<Tag<Identifier>> { self.block_tags.read().get(key).cloned() }
    pub fn is_block_tagged(&self, tag: &Identifier, block: &Identifier) -> bool {
        self.block_tags.read().get(tag).is_some_and(|t| t.contains(block))
    }

    pub fn register_item_tag(&self, tag: Tag<Identifier>) -> bool {
        let mut w=self.item_tags.write();
        if w.contains_key(&tag.key) { return false; }
        w.insert(tag.key.clone(), tag); true
    }
    pub fn get_item_tag(&self, key: &Identifier) -> Option<Tag<Identifier>> { self.item_tags.read().get(key).cloned() }

    pub fn register_entity_tag(&self, tag: Tag<Identifier>) -> bool {
        let mut w=self.entity_tags.write();
        if w.contains_key(&tag.key) { return false; }
        w.insert(tag.key.clone(), tag); true
    }

    // хэлперы для Paper `Tag.WOOL`, `Tag.LOGS` и т.д.
    pub fn is_log(&self, block: &Identifier) -> bool { self.is_block_tagged(&Identifier::new_static("minecraft", "logs"), block) }
    pub fn is_wool(&self, block: &Identifier) -> bool { self.is_block_tagged(&Identifier::new_static("minecraft", "wool"), block) }
}
