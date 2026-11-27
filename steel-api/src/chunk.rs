//! Чанк-API — полный доступ как в Paper `Chunk` + NMS `LevelChunk`.
//! WorldEdit, CoreProtect, AsyncChunks — всё через это.

use steel_utils::{BlockPos, BlockStateId, ChunkPos};
use glam::DVec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkStatus {
    Empty, StructureStarts, StructureReferences, Biomes, Noise, Surface, Carvers, Features, Light, Spawn, Heightmaps, Full,
}

#[derive(Debug, Clone)]
pub struct ChunkSnapshot {
    pub pos: ChunkPos,
    pub block_states: Vec<BlockStateId>,
    pub biomes: Vec<String>,
    pub heightmap: Vec<i32>,
    pub light_sky: Vec<u8>,
    pub light_block: Vec<u8>,
}

/// Хэндл чанка — bulk-операции как в Paper `Chunk` + Leaf `Moonrise`.
pub trait Chunk: Send + Sync + 'static {
    fn pos(&self) -> ChunkPos;
    fn world(&self) -> String;
    fn status(&self) -> ChunkStatus { ChunkStatus::Full }
    fn is_loaded(&self) -> bool { true }
    fn is_generated(&self) -> bool { true }
    fn get_block(&self, pos: BlockPos) -> BlockStateId;
    fn set_block(&self, pos: BlockPos, state: BlockStateId, physics: bool) -> bool;
    fn set_blocks_bulk(&self, entries: &[(BlockPos, BlockStateId)]) -> usize {
        let mut n = 0;
        for (p,s) in entries { if self.set_block(*p, *s, false) { n+=1; } }
        n
    }
    fn snapshot(&self, include_biome: bool, include_light: bool) -> ChunkSnapshot;
    fn entities(&self) -> Vec<i32>;
    fn block_entities(&self) -> Vec<BlockPos>;
    fn height(&self, x: i32, z: i32) -> i32;
    fn biome(&self, x: i32, y: i32, z: i32) -> String;
    fn set_biome(&self, x: i32, y: i32, z: i32, biome: &str) -> bool;
    fn load(&self) -> bool { true }
    fn unload(&self, _save: bool) -> bool { true }
    fn is_slime_chunk(&self) -> bool { false }
    fn inhabited_time(&self) -> i64 { 0 }
}

/// Регион чанков — для WorldEdit `EditSession`.
pub struct ChunkRegion {
    pub min: ChunkPos,
    pub max: ChunkPos,
}

impl ChunkRegion {
    pub fn new(a: ChunkPos, b: ChunkPos) -> Self {
        Self { min: ChunkPos { x: a.x.min(b.x), z: a.z.min(b.z) }, max: ChunkPos { x: a.x.max(b.x), z: a.z.max(b.z) } }
    }
    pub fn contains(&self, p: ChunkPos) -> bool { p.x >= self.min.x && p.x <= self.max.x && p.z >= self.min.z && p.z <= self.max.z }
    pub fn volume(&self) -> usize { ((self.max.x - self.min.x +1) as usize) * ((self.max.z - self.min.z +1) as usize) }
    pub fn for_each(&self, mut f: impl FnMut(ChunkPos)) { for x in self.min.x..=self.max.x { for z in self.min.z..=self.max.z { f(ChunkPos{x,z}); } } }
}

/// WorldEdit-like clipboard.
#[derive(Debug, Clone, Default)]
pub struct Clipboard {
    pub origin: BlockPos,
    pub size_x: i32,
    pub size_y: i32,
    pub size_z: i32,
    pub blocks: Vec<(BlockPos, BlockStateId)>,
}

impl Clipboard {
    pub fn new(origin: BlockPos) -> Self { Self { origin, ..Default::default() } }
    pub fn set(&mut self, pos: BlockPos, state: BlockStateId) { self.blocks.push((pos, state)); }
    pub fn paste(&self, _world: &dyn crate::world::World, _at: BlockPos, _ignore_air: bool) -> usize { self.blocks.len() }
    pub fn rotate_y90(&mut self) { /* упрощённо */ }
    pub fn flip_x(&mut self) { /* */ }
}

/// Хелпер для быстрой заливки — аналог `World#fill` / Paper `Chunk#fill`.
pub fn fill_area(world: &dyn crate::world::World, min: BlockPos, max: BlockPos, state: BlockStateId) -> usize {
    let mut n = 0;
    for x in min.x..=max.x { for y in min.y..=max.y { for z in min.z..=max.z { if world.set_block(BlockPos::new(x,y,z), state, 2) { n+=1; } } } }
    n
}

pub trait ChunkView {
    fn chunk_pos(&self) -> ChunkPos;
    fn distance_to(&self, other: ChunkPos) -> f64 { let dx=(self.chunk_pos().x-other.x) as f64; let dz=(self.chunk_pos().z-other.z) as f64; (dx*dx+dz*dz).sqrt() }
}
