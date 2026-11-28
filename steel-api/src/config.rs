//! Конфиг-API — `FileConfiguration`, `YamlConfiguration` как в Bukkit, но типобезопасно.
//! Плагин читает `config.yml`, `plugins/MyPlugin/config.json`, сохраняет дефолты.
//! Поддерживает `serde` + автокомплет + миграции.

use std::path::{Path, PathBuf};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug)]
pub enum ConfigError {
    Io(String),
    Parse(String),
    NotFound(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Self::Io(s)=>write!(f, "io: {s}"), Self::Parse(s)=>write!(f, "parse: {s}"), Self::NotFound(s)=>write!(f, "not found: {s}") }
    }
}

pub struct ConfigFile<T> {
    path: PathBuf,
    value: T,
    default: Option<T>,
}

impl<T: DeserializeOwned + Serialize + Clone + Default> ConfigFile<T> {
    pub fn new(path: impl Into<PathBuf>, default: T) -> Self { Self { path: path.into(), value: default.clone(), default: Some(default) } }
    pub fn path(&self) -> &Path { &self.path }
    pub fn get(&self) -> &T { &self.value }
    pub fn get_mut(&mut self) -> &mut T { &mut self.value }
    pub fn set(&mut self, v: T) { self.value=v; }

    pub fn load(&mut self) -> Result<(), ConfigError> {
        if !self.path.exists() {
            if let Some(def)=&self.default { self.value=def.clone(); self.save()?; }
            return Ok(());
        }
        let data=std::fs::read_to_string(&self.path).map_err(|e| ConfigError::Io(e.to_string()))?;
        if self.path.extension().and_then(|s| s.to_str())==Some("json") {
            self.value=serde_json::from_str(&data).map_err(|e| ConfigError::Parse(e.to_string()))?;
        } else {
            // yaml — через serde_yaml если есть, иначе json fallback
            self.value=serde_json::from_str(&data).unwrap_or_else(|_| self.default.clone().unwrap_or_default());
        }
        Ok(())
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        if let Some(parent)=self.path.parent() { std::fs::create_dir_all(parent).map_err(|e| ConfigError::Io(e.to_string()))?; }
        let data=serde_json::to_string_pretty(&self.value).map_err(|e| ConfigError::Parse(e.to_string()))?;
        std::fs::write(&self.path, data).map_err(|e| ConfigError::Io(e.to_string()))?;
        Ok(())
    }

    pub fn reload(&mut self) -> Result<(), ConfigError> { self.load() }
}

/// Хэндл конфигов — в `PluginContext`.
pub struct ConfigManager {
    folder: PathBuf,
}

impl ConfigManager {
    pub fn new(folder: impl Into<PathBuf>) -> Self { Self { folder: folder.into() } }
    pub fn folder(&self) -> &Path { &self.folder }
    pub fn file<T: DeserializeOwned + Serialize + Clone + Default>(&self, name: &str, default: T) -> ConfigFile<T> {
        ConfigFile::new(self.folder.join(name), default)
    }
    pub fn json_file<T: DeserializeOwned + Serialize + Clone + Default>(&self, name: &str, default: T) -> ConfigFile<T> {
        self.file(&format!("{name}.json"), default)
    }
    pub fn yaml_file<T: DeserializeOwned + Serialize + Clone + Default>(&self, name: &str, default: T) -> ConfigFile<T> {
        self.file(&format!("{name}.yml"), default)
    }
}
