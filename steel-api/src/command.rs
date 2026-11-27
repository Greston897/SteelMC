//! Команды — тонкий ergonomic слой над brigadier в `steel-core::command`.
//! Плагин регистрирует `literal`/`argument` деревья, как в ванили/Paper.

use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("команда `{0}` уже зарегистрирована")]
    AlreadyRegistered(String),
    #[error("ошибка регистрации: {0}")]
    Registration(String),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

/// Источник команды — игрок, консоль, блок, entity. Плагин получает его в `executes`.
#[derive(Debug, Clone)]
pub struct CommandSource {
    pub name: String,
    pub is_player: bool,
    pub player_id: Option<uuid::Uuid>,
    pub permission_level: u8,
}

impl CommandSource {
    pub fn console() -> Self {
        Self { name: "Console".into(), is_player: false, player_id: None, permission_level: 4 }
    }
    pub fn player(name: impl Into<String>, id: uuid::Uuid) -> Self {
        Self { name: name.into(), is_player: true, player_id: Some(id), permission_level: 0 }
    }
    pub fn send_success(&self, msg: impl AsRef<str>) {
        log::info!("[cmd:{}] {}", self.name, msg.as_ref());
    }
    pub fn send_failure(&self, msg: impl AsRef<str>) {
        log::warn!("[cmd:{}] {}", self.name, msg.as_ref());
    }
    pub fn has_permission(&self, _perm: &str) -> bool {
        self.permission_level >= 2 || !self.is_player
    }
}

/// Контекст выполнения (аргументы уже распарсены).
pub struct CommandContext<'a> {
    pub source: &'a CommandSource,
    pub input: &'a str,
    // в реальном ядре тут ParsedArguments; упрощённо — строковый доступ
}

impl<'a> CommandContext<'a> {
    pub fn arg(&self, _name: &str) -> Option<&str> { None }
}

/// Узел команды — строится через `literal`/`argument`, как в brigadier.
/// Хранится type-erased, ядро конвертирует в свой dispatcher.
pub struct CommandNode {
    pub name: String,
    pub handler: Option<Arc<dyn Fn(&CommandSource, &CommandContext<'_>) -> Result<i32, CommandError> + Send + Sync>>,
    pub children: Vec<CommandNode>,
    pub permission: Option<String>,
    pub requires_op: bool,
}

impl CommandNode {
    pub fn literal(name: impl Into<String>) -> Self {
        Self { name: name.into(), handler: None, children: Vec::new(), permission: None, requires_op: false }
    }
    pub fn executes<F>(mut self, f: F) -> Self
    where F: Fn(&CommandSource, &CommandContext<'_>) -> Result<i32, CommandError> + Send + Sync + 'static
    {
        self.handler = Some(Arc::new(f));
        self
    }
    pub fn then(mut self, child: CommandNode) -> Self {
        self.children.push(child);
        self
    }
    pub fn requires_permission(mut self, perm: impl Into<String>) -> Self {
        self.permission = Some(perm.into());
        self
    }
    pub fn requires_op(mut self, op: bool) -> Self {
        self.requires_op = op;
        self
    }
}

pub fn literal(name: impl Into<String>) -> CommandNode { CommandNode::literal(name) }
pub fn argument(_name: impl Into<String>, _parser: &str) -> CommandNode { CommandNode::literal(_name) }

/// Хэндл реестра — ядро инжектит его в `PluginContext`.
pub struct CommandRegistryHandle {
    commands: RwLock<Vec<CommandNode>>,
}

impl Default for CommandRegistryHandle {
    fn default() -> Self { Self::new() }
}
impl CommandRegistryHandle {
    pub fn new() -> Self { Self { commands: RwLock::new(Vec::new()) } }

    pub fn register(&self, node: CommandNode) -> Result<(), CommandError> {
        let name = node.name.clone();
        let mut w = self.commands.write();
        if w.iter().any(|n| n.name == name) {
            return Err(CommandError::AlreadyRegistered(name));
        }
        w.push(node);
        Ok(())
    }
    pub fn unregister(&self, name: &str) -> bool {
        let mut w = self.commands.write();
        let before = w.len();
        w.retain(|n| n.name != name);
        w.len() != before
    }
    pub fn list(&self) -> Vec<String> {
        self.commands.read().iter().map(|n| n.name.clone()).collect()
    }
    pub fn len(&self) -> usize { self.commands.read().len() }
    pub fn is_empty(&self) -> bool { self.commands.read().is_empty() }
}
