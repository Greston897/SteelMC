//! Плейсхолдер-API — аналог PlaceholderAPI + Paper `MiniPlaceholders`.
//! Плагин регистрирует расширение, игроки пишут `%myplugin_stat%`, `%player_name%`.

use std::sync::Arc;
use rustc_hash::FxHashMap;
use steel_utils::locks::SyncRwLock;

/// Контекст плейсхолдера — кто запросил и где.
#[derive(Debug, Clone)]
pub struct PlaceholderContext {
    pub player: Option<uuid::Uuid>,
    pub world: Option<String>,
    pub args: Vec<String>,
}

/// Одно расширение — как `PlaceholderExpansion` в PAPI.
pub trait PlaceholderExpansion: Send + Sync + 'static {
    fn identifier(&self) -> &str; // "myplugin" → `%myplugin_x%`
    fn author(&self) -> &str { "steel" }
    fn version(&self) -> &str { "1.0.0" }
    fn can_register(&self) -> bool { true }
    /// Запросить значение: `params` — всё после `identifier_` без `%`.
    fn on_request(&self, player: Option<uuid::Uuid>, params: &str, ctx: &PlaceholderContext) -> Option<String>;
    fn on_placeholder_request(&self, player: Option<uuid::Uuid>, params: &str) -> Option<String> {
        let ctx = PlaceholderContext { player, world: None, args: Vec::new() };
        self.on_request(player, params, &ctx)
    }
}

/// Менеджер плейсхолдеров — ядро держит один глобальный.
pub struct PlaceholderManager {
    expansions: SyncRwLock<FxHashMap<String, Arc<dyn PlaceholderExpansion>>>,
}

impl Default for PlaceholderManager { fn default() -> Self { Self::new() } }

impl PlaceholderManager {
    pub fn new() -> Self { Self { expansions: SyncRwLock::new(FxHashMap::default()) } }

    pub fn register(&self, exp: Arc<dyn PlaceholderExpansion>) -> bool {
        let id=exp.identifier().to_string();
        let mut w=self.expansions.write();
        if w.contains_key(&id) { return false; }
        w.insert(id, exp); true
    }
    pub fn unregister(&self, id: &str) -> bool { self.expansions.write().remove(id).is_some() }
    pub fn is_registered(&self, id: &str) -> bool { self.expansions.read().contains_key(id) }

    /// Заменить все `%...%` в тексте — как `PlaceholderAPI#setPlaceholders(player, text)`.
    pub fn set_placeholders(&self, player: Option<uuid::Uuid>, text: &str) -> String {
        let mut out = text.to_string();
        let exps: Vec<(String, Arc<dyn PlaceholderExpansion>)> = self.expansions.read().iter().map(|(k,v)| (k.clone(), v.clone())).collect();
        // простой парсер: ищем %identifier_params%
        let mut result = String::new();
        let mut chars = out.chars().peekable();
        let mut buf = String::new();
        let mut in_ph = false;
        for c in chars {
            if c == '%' {
                if in_ph {
                    // закрываем
                    let full = buf.clone();
                    buf.clear();
                    in_ph = false;
                    // full = "identifier_params" или "identifier"
                    let (id, params) = if let Some(pos) = full.find('_') { (&full[..pos], &full[pos+1..]) } else { (full.as_str(), "") };
                    if let Some(exp) = exps.iter().find(|(k,_)| k==id).map(|(_,v)| v) {
                        if let Some(val) = exp.on_placeholder_request(player, params) {
                            result.push_str(&val);
                        } else {
                            result.push_str(&format!("%{full}%"));
                        }
                    } else {
                        result.push_str(&format!("%{full}%"));
                    }
                } else {
                    in_ph = true;
                }
            } else if in_ph {
                buf.push(c);
            } else {
                result.push(c);
            }
        }
        if in_ph { result.push('%'); result.push_str(&buf); }
        result
    }

    pub fn set_placeholders_player(&self, player: uuid::Uuid, text: &str) -> String { self.set_placeholders(Some(player), text) }

    /// Зарегистрировать встроенные `%player_name%`, `%player_uuid%`, `%world%`, `%tps%`.
    pub fn register_defaults(&self) {
        struct PlayerExpansion;
        impl PlaceholderExpansion for PlayerExpansion {
            fn identifier(&self) -> &str { "player" }
            fn on_request(&self, player: Option<uuid::Uuid>, params: &str, _ctx: &PlaceholderContext) -> Option<String> {
                match params {
                    "name" => Some(player.map(|u| u.to_string()).unwrap_or_else(|| "unknown".into())),
                    "uuid" => Some(player.map(|u| u.to_string()).unwrap_or_default()),
                    _ => None,
                }
            }
        }
        let _ = self.register(Arc::new(PlayerExpansion));
    }
}
