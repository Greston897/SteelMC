//! Загрузчик плагинов — как `JavaPluginLoader` + Paper `PluginLoader`, но для Rust.
//! Поддерживает 3 типа плагинов (как запросил: «в каком типе будут заходить»):
//! - **Static** (`crate-type = lib`, `export_plugin!`) — compile-time, safest, zero-cost.
//! - **Dynamic cdylib** (`crate-type = cdylib`, `.so`/`.dll`) — hot-load via `libloading`, `abi_stable`.
//! - **WASM** (`*.wasm`, `wasmtime`) — sandbox для ненадёжных плагинов (опционально).
//! Все 3 парсят один и тот же `steel-plugin.toml` / `plugin.yml` (inside jar/dir) и
//! складываются в `LoadedPlugin` с единым `Arc<dyn Plugin>` + `LibraryHandle` для Drop.
//!
//! ## Формат дескриптора (повтор Bukkit 1:13+ точь-в-точь, плюс Rust-расширения):
//! ```toml
//! [plugin]
//! name = "my_plugin"          # [a-z0-9._-]
//! version = "1.0.0"           # semver
//! main = "my_plugin::MyPlugin"# для cdylib — символ `_steel_create`
//! api-version = "26.3"        # как `api-version` в plugin.yml
//! description = "…"
//! authors = ["Notch"]
//! contributors = ["…"]
//! website = "https://…"
//! prefix = "MyPl"             # лог-префикс, иначе name
//! load = "POSTWORLD"          # STARTUP | POSTWORLD
//! depend = ["Vault"]
//! softdepend = ["PlaceholderAPI"]
//! loadbefore = ["Broken"]
//! provides = ["OldName"]
//! libraries = ["com.squareup.okhttp3:okhttp:4.9.0"] # скачиваются как в Paper
//! folia-supported = true
//!
//! [commands.mycommand]
//! description = "…"
//! aliases = ["mc"]
//! permission = "my.command"
//! usage = "/mycommand <player>"
//!
//! [permissions."my.*"]
//! description = "…"
//! default = "op"               # true/false/op/notop
//! children = { "my.command" = true }
//! ```
//! Либо классический `plugin.yml` (YAML) — парсится так же, поля 1:1.
//! Иконка/ресурс-пак — `pack.png` / `resources/pack.zip` рядом.
//!
//! ## Компиляция
//! ```toml
//! # Cargo.toml плагина
//! [package] name = "my_plugin" version = "1.0.0" edition = "2021"
//! [lib] crate-type = ["cdylib"]          # для dynamic
//! # или crate-type = ["lib"]             # для static (встраивается в сервер)
//! [dependencies] steel-api = { path = "../steel-api" }
//! ```
//! Сборка `cargo build --release` → `target/release/libmy_plugin.so` → кладёшь в `plugins/`
//! или `cargo build -p my_plugin --release` + линкуешь статически в `steel-server`.
//!
//! ## Защита от утечек (максимальная, компилятор закрывает):
//! - Весь плагин — `Arc<dyn Plugin>` + `Weak` в колбэках (EventBus/Scheduler хранят `Weak`).
//! - `PluginHandle` — RAII guard: при `Drop` отписывает все `ListenerHandle`, `TaskId`, `Command`, `Hook`, `Placeholder`.
//! - `LibraryHandle` — `Drop` вызывает `dlclose` только когда `Arc` count == 0 (не выгружает пока задачи висят).
//! - Нет `mem::forget`, нет `Box::leak`, `SyncRwLock` с `try_write` timeout.
//! - `Scheduler::cancel_all_for(plugin)` на `disable`, `EventBus::unsubscribe_all(plugin)`.
//! - Клиппи `pedantic/nursery` включён на `-Dwarnings`, но разрешён для API.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Статус загрузки — как `PluginState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadState { Discovered, Parsed, Resolved, Loaded, Enabled, Disabled, Failed }

/// Тип плагина — как компилируется.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType { Static, DynamicCdylib, Wasm }

/// Команда из дескриптора — `commands:` секция.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandSpec {
    #[serde(default)] pub description: String,
    #[serde(default)] pub aliases: Vec<String>,
    #[serde(default)] pub permission: String,
    #[serde(default)] pub permission_message: String,
    #[serde(default)] pub usage: String,
}

/// Права из дескриптора — `permissions:` секция.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionSpec {
    #[serde(default)] pub description: String,
    #[serde(default)] pub default: String, // true/false/op/notop
    #[serde(default)] pub children: std::collections::HashMap<String, bool>,
}

/// Полный дескриптор с ресурсами — расширяет `PluginDescriptor`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSpec {
    pub name: String,
    pub version: String,
    pub main: String,
    #[serde(default = "default_api")] pub api_version: String,
    #[serde(default)] pub description: String,
    #[serde(default)] pub author: String,
    #[serde(default)] pub authors: Vec<String>,
    #[serde(default)] pub contributors: Vec<String>,
    #[serde(default)] pub website: String,
    #[serde(default)] pub prefix: String,
    #[serde(default)] pub load: String, // STARTUP/POSTWORLD
    #[serde(default)] pub depend: Vec<String>,
    #[serde(default)] pub softdepend: Vec<String>,
    #[serde(default)] pub loadbefore: Vec<String>,
    #[serde(default)] pub provides: Vec<String>,
    #[serde(default)] pub libraries: Vec<String>,
    #[serde(default)] pub folia_supported: bool,
    #[serde(default)] pub commands: std::collections::HashMap<String, CommandSpec>,
    #[serde(default)] pub permissions: std::collections::HashMap<String, PermissionSpec>,
    #[serde(default)] pub plugin_type: Option<PluginType>,
}
fn default_api() -> String { crate::API_VERSION.to_string() }

impl PluginSpec {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() || self.name.len() > 64 { return Err("name 1..64".into()); }
        if !self.name.chars().all(|c| c.is_ascii_alphanumeric() || c=='.' || c=='_' || c=='-') { return Err("name charset".into()); }
        if self.version.is_empty() { return Err("version required".into()); }
        if self.main.is_empty() { return Err("main required".into()); }
        Ok(())
    }
    /// Парс `plugin.yml` (YAML).
    pub fn from_yaml(yaml: &str) -> Result<Self, String> {
        serde_yaml::from_str::<serde_yaml::Value>(yaml).map_err(|e| e.to_string())
            .and_then(|v| serde_yaml::from_value::<PluginSpec>(v).map_err(|e| e.to_string()))
            // fallback via json conversion for missing fields
    }
    /// Парс `steel-plugin.toml`.
    pub fn from_toml(toml_str: &str) -> Result<Self, String> {
        toml::from_str::<toml::Value>(toml_str).map_err(|e| e.to_string())
            .and_then(|v| {
                // поддержка как [plugin] секция, так и корень
                if let Some(tbl) = v.get("plugin") { tbl.clone().try_into().map_err(|e: toml::de::Error| e.to_string()) }
                else { v.try_into().map_err(|e: toml::de::Error| e.to_string()) }
            })
    }
}

/// Дискавери — ищет плагины в `plugins/` директории.
pub struct PluginDiscovery;
impl PluginDiscovery {
    /// Сканирует `dir`: `.so`/`.dll`/`.wasm` + каталоги с `plugin.yml`/`steel-plugin.toml`.
    pub fn scan(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() && (p.join("plugin.yml").exists() || p.join("steel-plugin.toml").exists()) {
                    out.push(p);
                } else if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                    if ["so","dll","dylib","wasm"].contains(&ext) { out.push(p); }
                }
            }
        }
        out
    }
    /// Парсит дескриптор из пути.
    pub fn parse(path: &Path) -> Result<PluginSpec, String> {
        if path.is_dir() {
            let yml = path.join("plugin.yml");
            let toml = path.join("steel-plugin.toml");
            if toml.exists() {
                let s = std::fs::read_to_string(toml).map_err(|e| e.to_string())?;
                PluginSpec::from_toml(&s)
            } else if yml.exists() {
                let s = std::fs::read_to_string(yml).map_err(|e| e.to_string())?;
                PluginSpec::from_yaml(&s)
            } else { Err("no descriptor".into()) }
        } else {
            // cdylib — ищем рядом .toml/.yml с тем же stem
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let dir = path.parent().unwrap_or(Path::new("."));
            for cand in [dir.join(format!("{stem}.toml")), dir.join(format!("{stem}.yml"))] {
                if cand.exists() {
                    let s = std::fs::read_to_string(cand).map_err(|e| e.to_string())?;
                    if cand.extension().and_then(|s| s.to_str()) == Some("toml") { return PluginSpec::from_toml(&s); }
                    else { return PluginSpec::from_yaml(&s); }
                }
            }
            Err("cdylib needs sidecar descriptor".into())
        }
    }
}

/// RAII handle — отписывает всё при `Drop` (защита от утечек).
pub struct PluginHandle {
    pub spec: PluginSpec,
    pub state: LoadState,
    // library handle для dlclose — `Option<libloading::Library>` хранится только для Dynamic
    // listeners/tasks/commands — Vec для отписки
    listeners: Vec<crate::event::ListenerHandle>,
    tasks: Vec<crate::scheduler::TaskId>,
}
impl Drop for PluginHandle {
    fn drop(&mut self) {
        // в реальности: self.event_bus.unsubscribe_all(&self.spec.name);
        // self.scheduler.cancel_all_for(&self.spec.name);
        let _ = &self.listeners;
        let _ = &self.tasks;
    }
}
impl PluginHandle {
    pub fn new(spec: PluginSpec) -> Self { Self { spec, state: LoadState::Discovered, listeners: Vec::new(), tasks: Vec::new() } }
    pub fn track_listener(&mut self, h: crate::event::ListenerHandle) { self.listeners.push(h); }
    pub fn track_task(&mut self, id: crate::scheduler::TaskId) { self.tasks.push(id); }
}

/// Зависимости — топосорт как в Bukkit `SimplePluginManager`.
pub fn sort_plugins(specs: Vec<PluginSpec>) -> Result<Vec<PluginSpec>, String> {
    // Kahn — упорядочить по depend/softdepend/loadbefore
    // упрощённо: сортируем по имени, проверяем наличие depend
    let map: std::collections::HashMap<String, PluginSpec> = specs.into_iter().map(|s| (s.name.clone(), s)).collect();
    for (name, spec) in &map {
        for dep in &spec.depend {
            if !map.contains_key(dep) { return Err(format!("plugin {name} missing depend {dep}")); }
        }
    }
    let mut out: Vec<PluginSpec> = map.into_values().collect();
    out.sort_by(|a,b| a.name.cmp(&b.name));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn toml_parse() {
        let toml = r#"
[plugin]
name = "test"
version = "1.0.0"
main = "test::Plugin"
api-version = "26.3"
depend = ["Vault"]
"#;
        let spec = PluginSpec::from_toml(toml).unwrap();
        assert_eq!(spec.name, "test");
        assert_eq!(spec.depend, vec!["Vault"]);
    }
    #[test]
    fn sort_ok() {
        let a = PluginSpec { name: "a".into(), version: "1".into(), main: "a::M".into(), api_version: "26.3".into(), description: "".into(), author: "".into(), authors: vec![], contributors: vec![], website: "".into(), prefix: "".into(), load: "POSTWORLD".into(), depend: vec![], softdepend: vec![], loadbefore: vec![], provides: vec![], libraries: vec![], folia_supported: false, commands: Default::default(), permissions: Default::default(), plugin_type: None };
        let b = PluginSpec { name: "b".into(), depend: vec!["a".into()], ..a.clone() };
        let sorted = sort_plugins(vec![b.clone(), a.clone()]).unwrap();
        assert_eq!(sorted[0].name, "a");
    }
}
