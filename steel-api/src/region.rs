//! Регион-API — аналог WorldGuard + GriefPrevention.
//! Плагин создаёт регионы, ставит флаги, приоритеты, проверяет `can_build`/`can_break`.

use steel_utils::{BlockPos, Identifier};
use rustc_hash::FxHashMap;
use steel_utils::locks::SyncRwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Flag {
    Build, Break, Place, Interact, Pvp, MobSpawning, CreeperExplosion, Tnt, FireSpread, EnderPearl, Invincible, Entry, Exit, Use, VehiclePlace, VehicleDestroy,
    Custom(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlagState {
    Allow, Deny, None,
}

#[derive(Debug, Clone)]
pub struct Region {
    pub id: String,
    pub world: String,
    pub min: BlockPos,
    pub max: BlockPos,
    pub priority: i32,
    pub owners: Vec<uuid::Uuid>,
    pub members: Vec<uuid::Uuid>,
    pub flags: FxHashMap<Flag, FlagState>,
    pub parent: Option<String>,
}

impl Region {
    pub fn new(id: impl Into<String>, world: impl Into<String>, min: BlockPos, max: BlockPos) -> Self {
        Self { id: id.into(), world: world.into(), min, max, priority: 0, owners: Vec::new(), members: Vec::new(), flags: FxHashMap::default(), parent: None }
    }
    pub fn contains(&self, pos: BlockPos) -> bool {
        pos.x >= self.min.x.min(self.max.x) && pos.x <= self.min.x.max(self.max.x) &&
        pos.y >= self.min.y.min(self.max.y) && pos.y <= self.min.y.max(self.max.y) &&
        pos.z >= self.min.z.min(self.max.z) && pos.z <= self.min.z.max(self.max.z)
    }
    pub fn volume(&self) -> i64 { let dx=(self.max.x-self.min.x).abs() as i64 +1; let dy=(self.max.y-self.min.y).abs() as i64 +1; let dz=(self.max.z-self.min.z).abs() as i64 +1; dx*dy*dz }
    pub fn set_flag(&mut self, f: Flag, s: FlagState) { self.flags.insert(f, s); }
    pub fn get_flag(&self, f: Flag) -> FlagState { self.flags.get(&f).cloned().unwrap_or(FlagState::None) }
    pub fn is_owner(&self, id: uuid::Uuid) -> bool { self.owners.contains(&id) }
    pub fn is_member(&self, id: uuid::Uuid) -> bool { self.members.contains(&id) || self.is_owner(id) }
}

pub struct RegionManager {
    regions: SyncRwLock<FxHashMap<String, Region>>,
}

impl Default for RegionManager { fn default() -> Self { Self::new() } }

impl RegionManager {
    pub fn new() -> Self { Self { regions: SyncRwLock::new(FxHashMap::default()) } }

    pub fn add(&self, r: Region) -> bool { let mut w=self.regions.write(); if w.contains_key(&r.id) { false } else { w.insert(r.id.clone(), r); true } }
    pub fn remove(&self, id: &str) -> bool { self.regions.write().remove(id).is_some() }
    pub fn get(&self, id: &str) -> Option<Region> { self.regions.read().get(id).cloned() }
    pub fn all(&self) -> Vec<Region> { self.regions.read().values().cloned().collect() }
    pub fn at(&self, world: &str, pos: BlockPos) -> Vec<Region> {
        let r=self.regions.read();
        let mut out: Vec<Region> = r.values().filter(|reg| reg.world==world && reg.contains(pos)).cloned().collect();
        out.sort_by_key(|x| -x.priority);
        out
    }
    pub fn can(&self, world: &str, pos: BlockPos, player: uuid::Uuid, flag: Flag) -> bool {
        let regions=self.at(world, pos);
        if regions.is_empty() { return true; }
        for reg in regions {
            match reg.get_flag(flag) {
                FlagState::Allow => return true,
                FlagState::Deny => return reg.is_member(player),
                FlagState::None => continue,
            }
        }
        true
    }
    pub fn can_build(&self, world: &str, pos: BlockPos, player: uuid::Uuid) -> bool { self.can(world, pos, player, Flag::Build) }
    pub fn can_break(&self, world: &str, pos: BlockPos, player: uuid::Uuid) -> bool { self.can(world, pos, player, Flag::Break) }
    pub fn can_pvp(&self, world: &str, pos: BlockPos) -> bool {
        let regions=self.at(world, pos);
        for r in regions { if r.get_flag(Flag::Pvp)==FlagState::Deny { return false; } }
        true
    }
}
