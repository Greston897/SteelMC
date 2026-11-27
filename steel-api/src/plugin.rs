//! Плагин: жизненный цикл, дескриптор, контекст.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::command::CommandRegistryHandle;
use crate::event::EventBus;
use crate::permission::PermissionRegistryHandle;
use crate::registry::RegistryAccess;
use crate::scheduler::Scheduler;

/// Ошибка плагина.
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin `{0}` уже загружен")]
    AlreadyLoaded(String),
    #[error("зависимость `{0}` не найдена для `{1}`")]
    MissingDependency(String, String),
    #[error("ошибка инициализации `{0}`: {1}")]
    InitFailed(String, String),
    #[error("io: {0}")]
    Io(String),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

pub type PluginResult<T> = Result<T, PluginError>;

/// Декларативный дескриптор плагина (аналог `plugin.yml` / `paper-plugin.yml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDescriptor {
    /// `namespace:name`, только `a-z0-9._-`, например `my_plugin`.
    pub name: String,
    /// semver.
    pub version: String,
    /// Человеко-читаемое описание.
    #[serde(default)]
    pub description: String,
    /// Авторы.
    #[serde(default)]
    pub authors: Vec<String>,
    /// Версия API, напр. `26.3`.
    #[serde(default = "default_api_version")]
    pub api_version: String,
    /// Зависимости (жёсткие).
    #[serde(default)]
    pub depend: Vec<String>,
    /// Мягкие зависимости.
    #[serde(default)]
    pub soft_depend: Vec<String>,
    /// Главный класс (для динамической загрузки, имя символа).
    #[serde(default)]
    pub main: String,
    /// Сайт / репозиторий.
    #[serde(default)]
    pub website: Option<String>,
}

fn default_api_version() -> String {
    crate::API_VERSION.to_string()
}

impl PluginDescriptor {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: String::new(),
            authors: Vec::new(),
            api_version: default_api_version(),
            depend: Vec::new(),
            soft_depend: Vec::new(),
            main: String::new(),
            website: None,
        }
    }

    pub fn validate(&self) -> PluginResult<()> {
        if self.name.is_empty() || self.name.len() > 64 {
            return Err(PluginError::InitFailed(
                self.name.clone(),
                "name must be 1..64 chars".into(),
            ));
        }
        Ok(())
    }
}

/// Провайдер дескриптора — реализуется рядом с `Plugin` для `export_plugin!`.
pub trait PluginDescriptorProvider {
    fn descriptor() -> PluginDescriptor;
}

/// Контекст, который ядро передаёт плагину. Через него — весь доступ к миру.
pub struct PluginContext {
    descriptor: PluginDescriptor,
    data_folder: PathBuf,
    event_bus: Arc<EventBus>,
    scheduler: Arc<Scheduler>,
    commands: Arc<CommandRegistryHandle>,
    permissions: Arc<PermissionRegistryHandle>,
    registry: Arc<dyn RegistryAccess>,
}

impl PluginContext {
    pub fn new(
        descriptor: PluginDescriptor,
        data_folder: PathBuf,
        event_bus: Arc<EventBus>,
        scheduler: Arc<Scheduler>,
        commands: Arc<CommandRegistryHandle>,
        permissions: Arc<PermissionRegistryHandle>,
        registry: Arc<dyn RegistryAccess>,
    ) -> Self {
        Self {
            descriptor,
            data_folder,
            event_bus,
            scheduler,
            commands,
            permissions,
            registry,
        }
    }

    pub fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }
    pub fn name(&self) -> &str {
        &self.descriptor.name
    }
    pub fn data_folder(&self) -> &Path {
        &self.data_folder
    }
    pub fn events(&self) -> &Arc<EventBus> {
        &self.event_bus
    }
    pub fn scheduler(&self) -> &Arc<Scheduler> {
        &self.scheduler
    }
    pub fn commands(&self) -> &Arc<CommandRegistryHandle> {
        &self.commands
    }
    pub fn permissions(&self) -> &Arc<PermissionRegistryHandle> {
        &self.permissions
    }
    pub fn registry(&self) -> &Arc<dyn RegistryAccess> {
        &self.registry
    }

    pub fn logger(&self) -> PluginLogger {
        PluginLogger {
            prefix: self.descriptor.name.clone(),
        }
    }
}

/// Лёгкий логгер с префиксом плагина.
#[derive(Clone, Debug)]
pub struct PluginLogger {
    prefix: String,
}
impl PluginLogger {
    pub fn info(&self, msg: impl AsRef<str>) {
        log::info!("[{}] {}", self.prefix, msg.as_ref());
    }
    pub fn warn(&self, msg: impl AsRef<str>) {
        log::warn!("[{}] {}", self.prefix, msg.as_ref());
    }
    pub fn error(&self, msg: impl AsRef<str>) {
        log::error!("[{}] {}", self.prefix, msg.as_ref());
    }
}

/// Главный трейт плагина. Реализуй его. Жизненный цикл как у Paper, но без checked exceptions.
pub trait Plugin: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    /// Вызывается при загрузке (до включения миров). Регистрируй тут behaviors, команды без мира.
    fn on_load(&self, _ctx: &PluginContext) -> PluginResult<()> {
        Ok(())
    }
    /// Включение — миры готовы, можно трогать `World`.
    fn on_enable(&self, _ctx: &PluginContext) -> PluginResult<()> {
        Ok(())
    }
    /// Выключение — отмени таймеры, сохрани данные.
    fn on_disable(&self, _ctx: &PluginContext) {}

    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor::new(self.name().to_string(), self.version().to_string())
    }
}

/// Состояние плагина в менеджере.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    Loaded,
    Enabled,
    Disabled,
    Failed,
}
