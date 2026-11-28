//! Мирген-API — кастомные генераторы, биомы, фичи, структуры.
//! Аналог Paper `ChunkGenerator`, `BiomeProvider`, `BlockPopulator` + Fabric `ModWorldGen`.
//! Плагин может полностью заменить генерацию чанка, добавить биом, фичу, структуру.

use rustc_hash::FxHashMap;
use steel_utils::{BlockPos, BlockStateId, ChunkPos, Identifier};

/// Контекст генерации чанка — даёт сид, шум, доступ к соседям.
pub struct ChunkGenContext {
    pub world_seed: i64,
    pub chunk_pos: ChunkPos,
    pub dimension: Identifier,
}

/// Хук генератора чанка — замени `VanillaGenerator` на свой.
pub trait ChunkGeneratorHook: Send + Sync + 'static {
    fn generate_chunk(&self, ctx: &ChunkGenContext, chunk: &mut ChunkBuffer) -> bool { let _=(ctx, chunk); true }
    fn generate_biomes(&self, ctx: &ChunkGenContext, biomes: &mut BiomeBuffer) { let _=(ctx, biomes); }
    fn should_generate(&self, chunk: ChunkPos) -> bool { let _=chunk; true }
}

/// Буфер блока — 16×384×16 (или сколько в мире).
pub struct ChunkBuffer {
    pub min_y: i32,
    pub height: i32,
    blocks: Vec<BlockStateId>,
}

impl ChunkBuffer {
    pub fn new(min_y: i32, height: i32) -> Self { Self { min_y, height, blocks: vec![BlockStateId(0); 16*height as usize*16] } }
    pub fn set(&mut self, x: i32, y: i32, z: i32, state: BlockStateId) {
        if !(0..16).contains(&x) || !(0..16).contains(&z) || y < self.min_y || y >= self.min_y+self.height { return; }
        let idx=((y-self.min_y) as usize*16 + z as usize)*16 + x as usize;
        if idx < self.blocks.len() { self.blocks[idx]=state; }
    }
    pub fn get(&self, x: i32, y: i32, z: i32) -> BlockStateId {
        if !(0..16).contains(&x) || !(0..16).contains(&z) || y < self.min_y || y >= self.min_y+self.height { return BlockStateId(0); }
        let idx=((y-self.min_y) as usize*16 + z as usize)*16 + x as usize;
        self.blocks.get(idx).copied().unwrap_or(BlockStateId(0))
    }
    pub fn fill(&mut self, min: BlockPos, max: BlockPos, state: BlockStateId) {
        for x in min.x()..=max.x() { for y in min.y()..=max.y() { for z in min.z()..=max.z() { self.set(x & 15, y, z & 15, state); } } }
    }
}

pub struct BiomeBuffer {
    pub biomes: Vec<Identifier>,
}

impl BiomeBuffer {
    pub fn new() -> Self { Self { biomes: vec![Identifier::new_static("minecraft","plains"); 4*4*4] } }
    pub fn set(&mut self, x: i32, y: i32, z: i32, biome: Identifier) {
        let idx=((y & 3) as usize*4 + (z &3) as usize)*4 + (x &3) as usize;
        if idx < self.biomes.len() { self.biomes[idx]=biome; }
    }
}

/// Хук биома — Paper `BiomeProvider`.
pub trait BiomeProviderHook: Send + Sync + 'static {
    fn get_biome(&self, x: i32, y: i32, z: i32, seed: i64) -> Identifier { let _=(x,y,z,seed); Identifier::new_static("minecraft","plains") }
    fn get_biome_list(&self) -> Vec<Identifier> { vec![Identifier::new_static("minecraft","plains")] }
}

/// Хук фичи — дерево, руда, структура.
pub trait FeatureHook: Send + Sync + 'static {
    fn place(&self, world: &dyn crate::world::World, pos: BlockPos, seed: i64) -> bool { let _=(world,pos,seed); false }
    fn can_place(&self, world: &dyn crate::world::World, pos: BlockPos) -> bool { let _=(world,pos); true }
}

/// Реестр миргена — в `PluginContext`.
pub struct WorldgenRegistry {
    generators: steel_utils::locks::SyncRwLock<rustc_hash::FxHashMap<String, std::sync::Arc<dyn ChunkGeneratorHook>>>,
    biomes: steel_utils::locks::SyncRwLock<rustc_hash::FxHashMap<String, std::sync::Arc<dyn BiomeProviderHook>>>,
    features: steel_utils::locks::SyncRwLock<rustc_hash::FxHashMap<Identifier, std::sync::Arc<dyn FeatureHook>>>,
}

impl Default for WorldgenRegistry { fn default() -> Self { Self::new() } }

impl WorldgenRegistry {
    pub fn new() -> Self { Self { generators: steel_utils::locks::SyncRwLock::new(Default::default()), biomes: steel_utils::locks::SyncRwLock::new(Default::default()), features: steel_utils::locks::SyncRwLock::new(Default::default()) } }
    pub fn register_generator(&self, dimension: impl Into<String>, hook: std::sync::Arc<dyn ChunkGeneratorHook>) -> bool {
        let key: String = dimension.into();
        let mut w=self.generators.write();
        if w.contains_key(key.as_str()) { return false; }
        w.insert(key, hook); true
    }
    pub fn get_generator(&self, dimension: &str) -> Option<std::sync::Arc<dyn ChunkGeneratorHook>> { self.generators.read().get(dimension).cloned() }
    pub fn register_biome_provider(&self, world: impl Into<String>, p: std::sync::Arc<dyn BiomeProviderHook>) -> bool {
        let key: String = world.into();
        let mut w=self.biomes.write();
        if w.contains_key(key.as_str()) { return false; }
        w.insert(key, p); true
    }
    pub fn register_feature(&self, id: Identifier, f: std::sync::Arc<dyn FeatureHook>) -> bool {
        let mut w=self.features.write();
        if w.contains_key(&id) { return false; }
        w.insert(id, f); true
    }
    pub fn get_feature(&self, id: &Identifier) -> Option<std::sync::Arc<dyn FeatureHook>> { self.features.read().get(id).cloned() }
}
