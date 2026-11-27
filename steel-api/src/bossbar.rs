//! Боссбар-API — как `org.bukkit.boss.BossBar` + Paper `BossBar` (Adventure).
//! Плагин создаёт боссбар, показывает игрокам, меняет прогресс/цвет/оверлей.

use std::sync::Arc;
use steel_utils::locks::SyncRwLock;
use rustc_hash::FxHashSet;

use crate::chat::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BossBarColor {
    Pink, Blue, Red, Green, Yellow, Purple, White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BossBarOverlay {
    Progress, Notched6, Notched10, Notched12, Notched20,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BossBarFlags(u8);
impl BossBarFlags {
    pub const DARKEN_SKY: Self = Self(1);
    pub const PLAY_BOSS_MUSIC: Self = Self(2);
    pub const CREATE_FOG: Self = Self(4);
    pub const NONE: Self = Self(0);
    pub fn contains(self, other: Self) -> bool { self.0 & other.0 != 0 }
    pub fn insert(&mut self, other: Self) { self.0 |= other.0; }
    pub fn remove(&mut self, other: Self) { self.0 &= !other.0; }
}

pub struct BossBar {
    title: SyncRwLock<Component>,
    progress: SyncRwLock<f32>,
    color: SyncRwLock<BossBarColor>,
    overlay: SyncRwLock<BossBarOverlay>,
    flags: SyncRwLock<BossBarFlags>,
    visible: SyncRwLock<bool>,
    viewers: SyncRwLock<FxHashSet<uuid::Uuid>>,
}

impl BossBar {
    pub fn new(title: impl Into<Component>, color: BossBarColor, overlay: BossBarOverlay) -> Arc<Self> {
        Arc::new(Self {
            title: SyncRwLock::new(title.into()),
            progress: SyncRwLock::new(1.0),
            color: SyncRwLock::new(color),
            overlay: SyncRwLock::new(overlay),
            flags: SyncRwLock::new(BossBarFlags::NONE),
            visible: SyncRwLock::new(true),
            viewers: SyncRwLock::new(FxHashSet::default()),
        })
    }

    pub fn title(&self) -> Component { self.title.read().clone() }
    pub fn set_title(&self, c: impl Into<Component>) { *self.title.write() = c.into(); }
    pub fn progress(&self) -> f32 { *self.progress.read() }
    pub fn set_progress(&self, p: f32) { *self.progress.write() = p.clamp(0.0, 1.0); }
    pub fn color(&self) -> BossBarColor { *self.color.read() }
    pub fn set_color(&self, c: BossBarColor) { *self.color.write() = c; }
    pub fn overlay(&self) -> BossBarOverlay { *self.overlay.read() }
    pub fn set_overlay(&self, o: BossBarOverlay) { *self.overlay.write() = o; }
    pub fn flags(&self) -> BossBarFlags { *self.flags.read() }
    pub fn add_flag(&self, f: BossBarFlags) { self.flags.write().insert(f); }
    pub fn remove_flag(&self, f: BossBarFlags) { self.flags.write().remove(f); }
    pub fn is_visible(&self) -> bool { *self.visible.read() }
    pub fn set_visible(&self, v: bool) { *self.visible.write() = v; }

    pub fn add_viewer(&self, id: uuid::Uuid) { self.viewers.write().insert(id); }
    pub fn remove_viewer(&self, id: uuid::Uuid) -> bool { self.viewers.write().remove(&id) }
    pub fn viewers(&self) -> Vec<uuid::Uuid> { self.viewers.read().iter().copied().collect() }
    pub fn show_to(&self, _player: &dyn crate::player::Player) { self.add_viewer(_player.uuid()); }
    pub fn hide_from(&self, id: uuid::Uuid) { self.remove_viewer(id); }
    pub fn remove_all(&self) { self.viewers.write().clear(); }
}
