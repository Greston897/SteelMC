//! Мир-API: чтение/запись блоков, чанки, погода, время, геймрулы, частицы.

use glam::DVec3;
use steel_utils::{BlockPos, BlockStateId, ChunkPos, Identifier};
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;

/// Лёгкий `WorldRef` — идентификатор мира.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldRef {
    pub name: String,
    pub dimension: Identifier,
}

impl WorldRef {
    pub fn new(name: impl Into<String>, dimension: Identifier) -> Self {
        Self { name: name.into(), dimension }
    }
    pub fn overworld() -> Self {
        Self { name: "minecraft:overworld".into(), dimension: Identifier::new_static("minecraft","overworld") }
    }
}

/// Высокоуровневый доступ к миру (аналог `org.bukkit.World`).
/// Ядро реализует этот трейт; плагин получает `Arc<dyn World>`.
pub trait World: Send + Sync + 'static {
    fn world_ref(&self) -> WorldRef;
    fn name(&self) -> &str { "world" }

    // — блоки —
    fn get_block_state(&self, pos: BlockPos) -> BlockStateId;
    fn get_block(&self, pos: BlockPos) -> Option<BlockRef>;
    fn set_block(&self, pos: BlockPos, state: BlockStateId, flags: i32) -> bool;
    fn is_air(&self, pos: BlockPos) -> bool { self.get_block_state(pos).is_air() }
    fn height(&self, x: i32, z: i32) -> i32;

    // — чанки —
    fn is_chunk_loaded(&self, pos: ChunkPos) -> bool;
    fn load_chunk(&self, pos: ChunkPos) -> bool;
    fn unload_chunk(&self, pos: ChunkPos) -> bool;

    // — сущности —
    fn spawn_entity(&self, entity_type: &str, pos: DVec3) -> Option<i32>;

    // — погода/время —
    fn is_raining(&self) -> bool;
    fn set_raining(&self, raining: bool);
    fn thunder_level(&self) -> f32;
    fn set_thunder(&self, level: f32);
    fn time(&self) -> i64;
    fn set_time(&self, time: i64);
    fn game_rule(&self, rule: &str) -> Option<String>;
    fn set_game_rule(&self, rule: &str, value: &str) -> bool;

    // — эффекты —
    fn play_sound(&self, pos: DVec3, sound: &str, volume: f32, pitch: f32);
    fn spawn_particles(&self, particle: &str, pos: DVec3, count: i32, dx: f64, dy: f64, dz: f64, speed: f64);
    fn send_block_event(&self, pos: BlockPos, event_id: u8, param: u8);

    // — границы/мир —
    fn world_border_center(&self) -> DVec3;
    fn world_border_size(&self) -> f64;
    fn seed(&self) -> i64;
    fn difficulty(&self) -> String;
}
