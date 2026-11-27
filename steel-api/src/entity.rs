//! Entity-API — живой доступ к любой сущности.

use glam::DVec3;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EntitySnapshot {
    pub id: i32,
    pub uuid: Uuid,
    pub entity_type: String,
    pub pos: DVec3,
    pub velocity: DVec3,
    pub health: Option<f32>,
    pub custom_name: Option<String>,
}

/// Трейт любого entity — плагин видит снапшот, ядро — живую ссылку.
pub trait Entity: Send + Sync {
    fn id(&self) -> i32;
    fn uuid(&self) -> Uuid;
    fn entity_type(&self) -> &str;
    fn pos(&self) -> DVec3;
    fn teleport(&self, pos: DVec3) -> bool;
    fn remove(&self);
    fn is_alive(&self) -> bool;
    fn snapshot(&self) -> EntitySnapshot;
}

pub trait LivingEntity: Entity {
    fn health(&self) -> f32;
    fn max_health(&self) -> f32;
    fn set_health(&self, h: f32);
    fn damage(&self, amount: f32, cause: &str) -> bool;
    fn add_effect(&self, effect: &str, duration: i32, amplifier: i32) -> bool;
    fn remove_effect(&self, effect: &str) -> bool;
}

pub trait Mob: LivingEntity {
    fn target(&self) -> Option<i32>;
    fn set_target(&self, target: Option<i32>);
    fn is_ai_enabled(&self) -> bool;
    fn set_ai_enabled(&self, enabled: bool);
}
