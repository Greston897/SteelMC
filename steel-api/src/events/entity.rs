//! Сущности — мобы, живые, снаряды, AI.
use glam::DVec3;
use steel_utils::BlockPos;
use crate::event::Cancellable;

#[derive(Debug, Clone)]
pub struct EntitySpawnEvent {
    pub entity_type: String,
    pub pos: DVec3,
    pub reason: String,
    pub cancelled: bool,
}
impl Cancellable for EntitySpawnEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct EntityDespawnEvent {
    pub entity_id: i32,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct EntityDamageEvent {
    pub entity_id: i32,
    pub damage: f32,
    pub cause: String,
    pub cancelled: bool,
}
impl Cancellable for EntityDamageEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct EntityDeathEvent {
    pub entity_id: i32,
    pub drops: Vec<String>,
    pub exp: i32,
    pub cancelled: bool,
}
impl Cancellable for EntityDeathEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct EntityTargetEvent {
    pub entity_id: i32,
    pub target_id: Option<i32>,
    pub reason: String,
    pub cancelled: bool,
}
impl Cancellable for EntityTargetEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct EntityTeleportEvent {
    pub entity_id: i32,
    pub from: DVec3,
    pub to: DVec3,
    pub cancelled: bool,
}
impl Cancellable for EntityTeleportEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct EntityExplodeEvent {
    pub entity_id: i32,
    pub pos: DVec3,
    pub yield_rate: f32,
    pub blocks: Vec<BlockPos>,
    pub cancelled: bool,
}
impl Cancellable for EntityExplodeEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct ProjectileLaunchEvent {
    pub shooter_id: Option<i32>,
    pub projectile_type: String,
    pub pos: DVec3,
    pub cancelled: bool,
}
impl Cancellable for ProjectileLaunchEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct ProjectileHitEvent {
    pub projectile_id: i32,
    pub hit_pos: Option<BlockPos>,
    pub hit_entity: Option<i32>,
    pub cancelled: bool,
}
impl Cancellable for ProjectileHitEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct MobEffectAddEvent {
    pub entity_id: i32,
    pub effect: String,
    pub amplifier: i32,
    pub duration: i32,
    pub cancelled: bool,
}
impl Cancellable for MobEffectAddEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct MobEffectRemoveEvent {
    pub entity_id: i32,
    pub effect: String,
}

#[derive(Debug, Clone)]
pub struct EntityTameEvent {
    pub entity_id: i32,
    pub player_id: uuid::Uuid,
    pub cancelled: bool,
}
impl Cancellable for EntityTameEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct EntityBreedEvent {
    pub child_type: String,
    pub mother_id: i32,
    pub father_id: i32,
    pub breeder: uuid::Uuid,
    pub exp: i32,
    pub cancelled: bool,
}
impl Cancellable for EntityBreedEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct VillagerTradeEvent {
    pub villager_id: i32,
    pub player_id: uuid::Uuid,
    pub cancelled: bool,
}
impl Cancellable for VillagerTradeEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}

#[derive(Debug, Clone)]
pub struct EntityPortalEvent {
    pub entity_id: i32,
    pub from_world: String,
    pub to_world: Option<String>,
    pub cancelled: bool,
}
impl Cancellable for EntityPortalEvent {
    fn is_cancelled(&self) -> bool { self.cancelled }
    fn set_cancelled(&mut self, c: bool) { self.cancelled = c; }
}
