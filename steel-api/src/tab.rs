//! Таб-лист API — header/footer, player-info, completions, как в Paper `PlayerList`.
//! Плагин может менять табы пингом, хедером, кастомными скинами в списке.

use crate::chat::Component;

#[derive(Debug, Clone)]
pub struct TabEntry {
    pub uuid: uuid::Uuid,
    pub name: String,
    pub display: Option<Component>,
    pub ping: i32,
    pub gamemode: String,
    pub listed: bool,
}

impl TabEntry {
    pub fn new(uuid: uuid::Uuid, name: impl Into<String>) -> Self {
        Self { uuid, name: name.into(), display: None, ping: 0, gamemode: "survival".into(), listed: true }
    }
    pub fn display(mut self, c: impl Into<Component>) -> Self { self.display = Some(c.into()); self }
    pub fn ping(mut self, p: i32) -> Self { self.ping = p; self }
}

/// Хэндл таб-листа — на игрока или глобально.
pub struct TabList {
    header: steel_utils::locks::SyncRwLock<Component>,
    footer: steel_utils::locks::SyncRwLock<Component>,
}

impl Default for TabList { fn default() -> Self { Self::new() } }

impl TabList {
    pub fn new() -> Self { Self { header: steel_utils::locks::SyncRwLock::new(Component::empty()), footer: steel_utils::locks::SyncRwLock::new(Component::empty()) } }
    pub fn header(&self) -> Component { self.header.read().clone() }
    pub fn footer(&self) -> Component { self.footer.read().clone() }
    pub fn set_header(&self, c: impl Into<Component>) { *self.header.write() = c.into(); }
    pub fn set_footer(&self, c: impl Into<Component>) { *self.footer.write() = c.into(); }
    pub fn set(&self, header: impl Into<Component>, footer: impl Into<Component>) { self.set_header(header); self.set_footer(footer); }
    pub fn clear(&self) { self.set(Component::empty(), Component::empty()); }
}

/// Провайдер таб-комплишена — аналог `TabCompleter` + Brigadier `SuggestionProvider`.
pub trait TabCompleter: Send + Sync + 'static {
    fn complete(&self, _player: uuid::Uuid, _buffer: &str, _pos: usize) -> Vec<String> { Vec::new() }
    fn complete_with_tooltip(&self, _player: uuid::Uuid, _buffer: &str) -> Vec<(String, Option<Component>)> { Vec::new() }
}
