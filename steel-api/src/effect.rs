//! Эффект-API — зелья, частицы, звуки, `MobEffect`, `SoundEvent`, `Particle`.
//! Как `org.bukkit.potion`, `org.bukkit.Sound`, `org.bukkit.Particle` + Paper `PotionEffect`.

use glam::DVec3;

#[derive(Debug, Clone)]
pub struct MobEffect {
    pub id: String,
    pub amplifier: i32,
    pub duration: i32,
    pub ambient: bool,
    pub visible: bool,
    pub show_icon: bool,
}

impl MobEffect {
    pub fn new(id: impl Into<String>, duration: i32, amplifier: i32) -> Self {
        Self { id: id.into(), amplifier, duration, ambient: false, visible: true, show_icon: true }
    }
    pub fn ambient(mut self, v: bool) -> Self { self.ambient=v; self }
    pub fn with_duration(mut self, d: i32) -> Self { self.duration=d; self }
}

#[derive(Debug, Clone)]
pub enum Particle {
    Flame, Heart, Smoke, Crit, Enchant, Portal, Dust { r: u8, g: u8, b: u8, scale: f32 }, Block { block: String }, Item { item: String },
    Custom(String),
}

impl Particle {
    pub fn dust(r: u8, g: u8, b: u8, scale: f32) -> Self { Self::Dust { r,g,b,scale } }
}

#[derive(Debug, Clone)]
pub struct SoundEvent {
    pub id: String,
    pub volume: f32,
    pub pitch: f32,
    pub pos: Option<DVec3>,
}

impl SoundEvent {
    pub fn new(id: impl Into<String>, volume: f32, pitch: f32) -> Self { Self { id: id.into(), volume, pitch, pos: None } }
    pub fn at(mut self, pos: DVec3) -> Self { self.pos=Some(pos); self }
}

pub trait EffectHandler: Send + Sync + 'static {
    fn on_effect_add(&self, _entity: i32, _effect: &MobEffect) -> bool { true }
    fn on_effect_remove(&self, _entity: i32, _effect_id: &str) -> bool { true }
    fn on_particle_spawn(&self, _particle: &Particle, _pos: DVec3, _count: i32) -> bool { true }
    fn on_sound_play(&self, _sound: &SoundEvent) -> bool { true }
}

pub struct EffectRegistry {
    handlers: steel_utils::locks::SyncRwLock<Vec<std::sync::Arc<dyn EffectHandler>>>,
}

impl Default for EffectRegistry { fn default() -> Self { Self::new() } }

impl EffectRegistry {
    pub fn new() -> Self { Self { handlers: steel_utils::locks::SyncRwLock::new(Vec::new()) } }
    pub fn register(&self, h: std::sync::Arc<dyn EffectHandler>) { self.handlers.write().push(h); }
    pub fn fire_particle(&self, p: &Particle, pos: DVec3, count: i32) -> bool {
        let hs=self.handlers.read().clone();
        for h in hs { if !h.on_particle_spawn(p, pos, count) { return false; } }
        true
    }
}
