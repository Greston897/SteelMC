//! Плагин: жизненный цикл, дескриптор, контекст — Paper `JavaPlugin` + `PluginManager`, но Rust.
//! Через `PluginContext` плагин получает доступ ко всему: мир, команды, события, экономика, регионы, скорборд, пакеты, Via.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::command::CommandRegistryHandle;
use crate::event::EventBus;
use crate::permission::PermissionRegistryHandle;
use crate::registry::RegistryAccess;
use crate::scheduler::Scheduler;

// Новые менеджеры
use crate::server::ServerHandle;
use crate::scoreboard::Scoreboard;
use crate::region::RegionManager;
use crate::economy::EconomyHandle;
use crate::placeholder::PlaceholderManager;
use crate::recipe::RecipeManager;
use crate::tag::TagRegistry;
use crate::packet::PacketBus;
use crate::worldgen::WorldgenRegistry;
use crate::config::ConfigManager;
use crate::metadata::MetadataStore;

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
    /// Загружать до мира? как Paper `loadbefore`.
    #[serde(default)]
    pub load_before: Vec<String>,
    /// Стартовый лоадер.
    #[serde(default)]
    pub folia_supported: bool,
}

fn default_api_version() -> String { crate::API_VERSION.to_string() }

impl PluginDescriptor {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self { name: name.into(), version: version.into(), description: String::new(), authors: Vec::new(), api_version: default_api_version(), depend: Vec::new(), soft_depend: Vec::new(), main: String::new(), website: None, load_before: Vec::new(), folia_supported: false }
    }
    pub fn validate(&self) -> PluginResult<()> {
        if self.name.is_empty() || self.name.len() > 64 { return Err(PluginError::InitFailed(self.name.clone(), "name must be 1..64 chars".into())); }
        Ok(())
    }
}

/// Провайдер дескриптора.
pub trait PluginDescriptorProvider { fn descriptor() -> PluginDescriptor; }

/// Контекст, который ядро передаёт плагину. Через него — весь доступ к миру.
/// Аналог Paper `JavaPlugin#getServer()` + `Bukkit` статик, но DI и без глобалок.
pub struct PluginContext {
    descriptor: PluginDescriptor,
    data_folder: PathBuf,
    event_bus: Arc<EventBus>,
    scheduler: Arc<Scheduler>,
    commands: Arc<CommandRegistryHandle>,
    permissions: Arc<PermissionRegistryHandle>,
    registry: Arc<dyn RegistryAccess>,
    // новые
    server: Arc<ServerHandle>,
    scoreboard: Arc<Scoreboard>,
    regions: Arc<RegionManager>,
    economy: Arc<EconomyHandle>,
    placeholders: Arc<PlaceholderManager>,
    recipes: Arc<RecipeManager>,
    tags: Arc<TagRegistry>,
    packets: Arc<PacketBus>,
    worldgen: Arc<WorldgenRegistry>,
    config_mgr: Arc<ConfigManager>,
    metadata: Arc<MetadataStore>,
}

impl PluginContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        descriptor: PluginDescriptor,
        data_folder: PathBuf,
        event_bus: Arc<EventBus>,
        scheduler: Arc<Scheduler>,
        commands: Arc<CommandRegistryHandle>,
        permissions: Arc<PermissionRegistryHandle>,
        registry: Arc<dyn RegistryAccess>,
    ) -> Self {
        let server: Arc<ServerHandle> = Arc::new(ServerHandle::new(Arc::new(crate::server::DummyServer)));
        Self {
            descriptor, data_folder: data_folder.clone(), event_bus, scheduler, commands, permissions, registry,
            server,
            scoreboard: Arc::new(Scoreboard::new()),
            regions: Arc::new(RegionManager::new()),
            economy: Arc::new(EconomyHandle::new()),
            placeholders: Arc::new(PlaceholderManager::new()),
            recipes: Arc::new(RecipeManager::new()),
            tags: Arc::new(TagRegistry::new()),
            packets: Arc::new(PacketBus::new()),
            worldgen: Arc::new(WorldgenRegistry::new()),
            config_mgr: Arc::new(ConfigManager::new(data_folder)),
            metadata: Arc::new(MetadataStore::new()),
        }
    }

    /// Полный конструктор для ядра (с сервером).
    #[allow(clippy::too_many_arguments)]
    pub fn new_full(
        descriptor: PluginDescriptor,
        data_folder: PathBuf,
        event_bus: Arc<EventBus>,
        scheduler: Arc<Scheduler>,
        commands: Arc<CommandRegistryHandle>,
        permissions: Arc<PermissionRegistryHandle>,
        registry: Arc<dyn RegistryAccess>,
        server: Arc<ServerHandle>,
    ) -> Self {
        Self {
            descriptor, data_folder: data_folder.clone(), event_bus, scheduler, commands, permissions, registry, server,
            scoreboard: Arc::new(Scoreboard::new()),
            regions: Arc::new(RegionManager::new()),
            economy: Arc::new(EconomyHandle::new()),
            placeholders: Arc::new(PlaceholderManager::new()),
            recipes: Arc::new(RecipeManager::new()),
            tags: Arc::new(TagRegistry::new()),
            packets: Arc::new(PacketBus::new()),
            worldgen: Arc::new(WorldgenRegistry::new()),
            config_mgr: Arc::new(ConfigManager::new(data_folder)),
            metadata: Arc::new(MetadataStore::new()),
        }
    }

    pub fn descriptor(&self) -> &PluginDescriptor { &self.descriptor }
    pub fn name(&self) -> &str { &self.descriptor.name }
    pub fn data_folder(&self) -> &Path { &self.data_folder }
    pub fn events(&self) -> &Arc<EventBus> { &self.event_bus }
    pub fn scheduler(&self) -> &Arc<Scheduler> { &self.scheduler }
    pub fn commands(&self) -> &Arc<CommandRegistryHandle> { &self.commands }
    pub fn permissions(&self) -> &Arc<PermissionRegistryHandle> { &self.permissions }
    pub fn registry(&self) -> &Arc<dyn RegistryAccess> { &self.registry }
    pub fn server(&self) -> &Arc<ServerHandle> { &self.server }
    pub fn scoreboard(&self) -> &Arc<Scoreboard> { &self.scoreboard }
    pub fn regions(&self) -> &Arc<RegionManager> { &self.regions }
    pub fn economy(&self) -> &Arc<EconomyHandle> { &self.economy }
    pub fn placeholders(&self) -> &Arc<PlaceholderManager> { &self.placeholders }
    pub fn recipes(&self) -> &Arc<RecipeManager> { &self.recipes }
    pub fn tags(&self) -> &Arc<TagRegistry> { &self.tags }
    pub fn packets(&self) -> &Arc<PacketBus> { &self.packets }
    pub fn worldgen(&self) -> &Arc<WorldgenRegistry> { &self.worldgen }
    pub fn configs(&self) -> &Arc<ConfigManager> { &self.config_mgr }
    pub fn metadata(&self) -> &Arc<MetadataStore> { &self.metadata }

    /// Заменить существующую ванильную команду — например, `/give` на свой обработчик.
    /// Пример из Essentials: `ctx.commands().register_override(literal("give").executes(|src, ctx| ...))`
    pub fn override_command(&self, node: crate::command::CommandNode) -> Option<crate::command::CommandNode> {
        self.commands.register_override(node)
    }

    pub fn logger(&self) -> PluginLogger { PluginLogger { prefix: self.descriptor.name.clone() } }
}

/// Лёгкий логгер с префиксом плагина.
#[derive(Clone, Debug)]
pub struct PluginLogger { prefix: String, }
impl PluginLogger {
    pub fn info(&self, msg: impl AsRef<str>) { log::info!("[{}] {}", self.prefix, msg.as_ref()); }
    pub fn warn(&self, msg: impl AsRef<str>) { log::warn!("[{}] {}", self.prefix, msg.as_ref()); }
    pub fn error(&self, msg: impl AsRef<str>) { log::error!("[{}] {}", self.prefix, msg.as_ref()); }
    pub fn debug(&self, msg: impl AsRef<str>) { log::debug!("[{}] {}", self.prefix, msg.as_ref()); }
}

/// Главный трейт плагина. Жизненный цикл как у Paper, но без checked exceptions.
/// Реализуй `name`/`version`, опционально `on_load`/`on_enable`/`on_disable`.
pub trait Plugin: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_load(&self, _ctx: &PluginContext) -> PluginResult<()> { Ok(()) }
    fn on_enable(&self, _ctx: &PluginContext) -> PluginResult<()> { Ok(()) }
    fn on_disable(&self, _ctx: &PluginContext) {}
    fn descriptor(&self) -> PluginDescriptor { PluginDescriptor::new(self.name().to_string(), self.version().to_string()) }
}

/// Состояние плагина в менеджере.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState { Loaded, Enabled, Disabled, Failed, }
