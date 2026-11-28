//! WorldCreator / WorldBorder / Multiworld — полный аналог `org.bukkit.WorldCreator`
//! + Paper `WorldCreator` + Multiverse/MultiWorld-плагинов, но без хаков.
//! В ванили мир = `ServerLevel` с `Dimension`, `Seed`, `GameRules`, `Border`, `Spawn`.
//! Paper добавляет `createWorld(creator)`, `unloadWorld`, `getWorld`, `WorldBorder`.
//! Steel повторяет 1:1 + даёт хуки для каждого мини-часта (генератор, биом, структура).
//! MultiWorld-плагин теперь — 30 строк, а не 3000.

use steel_utils::{BlockPos, Identifier};
use steel_utils::locks::SyncRwLock;
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// Сложность как в ванили.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Difficulty { Peaceful, Easy, Normal, #[default] Hard }
impl std::fmt::Display for Difficulty { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }

/// Тип мира — overworld/nether/end/custom.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorldEnv { Normal, Nether, TheEnd, Custom(Identifier) }
impl Default for WorldEnv { fn default() -> Self { Self::Normal } }

/// Тип генерации — как в `level.dat:generator`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WorldType { #[default] Normal, Flat, LargeBiomes, Amplified, SingleBiome }

/// Геймрулы ванили (часть, остальные — строкой).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameRules(pub FxHashMap<String, String>);
impl GameRules {
    pub fn set(&mut self, k: impl Into<String>, v: impl Into<String>) { self.0.insert(k.into(), v.into()); }
    pub fn get(&self, k: &str) -> Option<&String> { self.0.get(k) }
}

/// Бордер — `WorldBorder` 1:1.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldBorder {
    pub center_x: f64,
    pub center_z: f64,
    pub size: f64,
    pub damage_buffer: f64,
    pub damage_per_block: f64,
    pub warning_blocks: i32,
    pub warning_time: i32,
}
impl Default for WorldBorder {
    fn default() -> Self {
        Self { center_x: 0.0, center_z: 0.0, size: 59_999_968.0, damage_buffer: 5.0, damage_per_block: 0.2, warning_blocks: 5, warning_time: 15 }
    }
}
impl WorldBorder {
    pub fn new(center_x: f64, center_z: f64, size: f64) -> Self { Self { center_x, center_z, size, ..Default::default() } }
    pub fn set_center(&mut self, x: f64, z: f64) { self.center_x = x; self.center_z = z; }
    pub fn set_size(&mut self, s: f64) { self.size = s.clamp(1.0, 59_999_968.0); }
    /// Проверка внутри ли бордера — ваниль формула.
    pub fn is_inside(&self, x: f64, z: f64) -> bool {
        let r = self.size / 2.0;
        x >= self.center_x - r && x <= self.center_x + r && z >= self.center_z - r && z <= self.center_z + r
    }
}

/// Полный дескриптор мира — то, что в `bukkit.yml` / `paper-world.yml` + `level.dat`.
#[derive(Debug, Clone)]
pub struct WorldCreator {
    pub name: String,
    pub env: WorldEnv,
    pub world_type: WorldType,
    pub seed: Option<i64>,
    pub structures: bool,
    pub hardcore: bool,
    pub keep_spawn_loaded: bool,
    pub generate: bool,
    pub generator: Option<String>, // "my_plugin:my_gen" — имя зарегистрированного `ChunkGeneratorHook`
    pub biome_provider: Option<String>, // "my_plugin:my_biome"
    pub gamerules: GameRules,
    pub difficulty: Difficulty,
    pub spawn: BlockPos,
    pub border: WorldBorder,
    pub dimension: Identifier,
}
impl WorldCreator {
    pub fn new(name: impl Into<String>) -> Self {
        let n = name.into();
        Self {
            dimension: Identifier::new("minecraft", n.clone()),
            name: n,
            env: WorldEnv::Normal,
            world_type: WorldType::Normal,
            seed: None,
            structures: true,
            hardcore: false,
            keep_spawn_loaded: true,
            generate: true,
            generator: None,
            biome_provider: None,
            gamerules: GameRules::default(),
            difficulty: Difficulty::Normal,
            spawn: BlockPos::new(0, 64, 0),
            border: WorldBorder::default(),
        }
    }
    pub fn env(mut self, e: WorldEnv) -> Self { self.env = e; self }
    pub fn seed(mut self, s: i64) -> Self { self.seed = Some(s); self }
    pub fn generator(mut self, id: impl Into<String>) -> Self { self.generator = Some(id.into()); self }
    pub fn biome(mut self, id: impl Into<String>) -> Self { self.biome_provider = Some(id.into()); self }
    pub fn spawn(mut self, pos: BlockPos) -> Self { self.spawn = pos; self }
    pub fn border(mut self, b: WorldBorder) -> Self { self.border = b; self }
    pub fn hardcore(mut self, v: bool) -> Self { self.hardcore = v; self }
    pub fn gamerule(mut self, k: impl Into<String>, v: impl Into<String>) -> Self { self.gamerules.set(k, v); self }
}

/// Менеджер миров — хранит все `WorldCreator` + живые `World`.
pub struct WorldManager {
    creators: SyncRwLock<FxHashMap<String, WorldCreator>>,
    // хуки для проверки canLoad/canUnload — аналог `WorldLoadEvent`/`WorldUnloadEvent`
}
impl Default for WorldManager { fn default() -> Self { Self::new() } }
impl WorldManager {
    pub fn new() -> Self { Self { creators: SyncRwLock::new(FxHashMap::default()) } }

    pub fn register(&self, creator: WorldCreator) -> bool {
        let mut w = self.creators.write();
        if w.contains_key(&creator.name) { return false; }
        w.insert(creator.name.clone(), creator);
        true
    }
    pub fn get(&self, name: &str) -> Option<WorldCreator> { self.creators.read().get(name).cloned() }
    pub fn all(&self) -> Vec<WorldCreator> { self.creators.read().values().cloned().collect() }
    pub fn remove(&self, name: &str) -> bool { self.creators.write().remove(name).is_some() }

    /// Создать мир синхронно — вызывает `SteelServer::create_world`.
    pub fn create<S: crate::server::SteelServer>(&self, server: &S, creator: WorldCreator) -> Option<Arc<dyn crate::world::World>> {
        let name = creator.name.clone();
        self.register(creator.clone());
        server.create_world(&name, creator.seed)
    }
}

/// Трейт для плагина MultiWorld — 1:1 с `MultiverseCore`.
pub trait MultiWorldHook: Send + Sync + 'static {
    fn on_pre_create(&self, creator: &mut WorldCreator) -> bool { let _ = creator; true }
    fn on_post_create(&self, world: &dyn crate::world::World) { let _ = world; }
    fn on_pre_unload(&self, name: &str) -> bool { let _ = name; true }
}

/// Пример MultiWorld-плагина на 30 строк:
/*
use steel_api::prelude::*;
pub struct MyMultiWorld;
impl Plugin for MyMultiWorld {
  fn on_enable(&self, ctx: &PluginContext) -> PluginResult<()> {
    let wm = ctx.world_manager(); // или ctx.server() + WorldManager
    wm.register(WorldCreator::new("lobby").seed(123).env(WorldEnv::Normal));
    wm.register(WorldCreator::new("pvp_nether").env(WorldEnv::Nether).hardcore(true));
    ctx.server().create_world("lobby", Some(123));
    Ok(())
  }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn border_inside() {
        let b = WorldBorder::new(0.0, 0.0, 100.0);
        assert!(b.is_inside(10.0, 10.0));
        assert!(!b.is_inside(60.0, 0.0));
    }
    #[test]
    fn creator_builder() {
        let c = WorldCreator::new("test").seed(42).env(WorldEnv::Nether).gamerule("keepInventory", "true");
        assert_eq!(c.seed, Some(42));
        assert_eq!(c.env, WorldEnv::Nether);
        assert_eq!(c.gamerules.get("keepInventory").map(|s| s.as_str()), Some("true"));
    }
    #[test]
    fn manager_crud() {
        let m = WorldManager::new();
        assert!(m.register(WorldCreator::new("a")));
        assert!(!m.register(WorldCreator::new("a")));
        assert!(m.get("a").is_some());
        assert!(m.remove("a"));
    }
}
