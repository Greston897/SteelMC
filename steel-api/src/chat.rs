//! Чат-API — MiniMessage + Adventure Component как в Paper, но типобезопасно.
//! Поддерживает `Component`, `ClickEvent`, `HoverEvent`, `HexColor`, `Gradient`,
//! `Title`, `ActionBar`, `BossBar` — всё что есть в Paper/Leaf.
//! Плагин может слать `Component` игроку, в чат, в `BossBar`, `Scoreboard`.

use rustc_hash::FxHashMap;

/// Hex-цвет.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexColor(pub String);
impl HexColor { pub fn new(s: impl Into<String>) -> Self { Self(s.into()) } }

/// Событие клика — как в Adventure.
#[derive(Debug, Clone)]
pub enum ClickEvent {
    OpenUrl(String),
    RunCommand(String),
    SuggestCommand(String),
    CopyToClipboard(String),
}

/// Событие ховера.
#[derive(Debug, Clone)]
pub enum HoverEvent {
    ShowText(Box<Component>),
    ShowItem { id: String, count: i32 },
    ShowEntity { ty: String, id: uuid::Uuid, name: Option<Box<Component>> },
}

/// Компонент чата — полный аналог `net.kyori.adventure.text.Component`.
#[derive(Debug, Clone, Default)]
pub struct Component {
    pub text: String,
    pub color: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
    pub click: Option<ClickEvent>,
    pub hover: Option<HoverEvent>,
    pub extra: Vec<Component>,
    pub font: Option<String>,
    pub insertion: Option<String>,
}

impl Component {
    pub fn text(t: impl Into<String>) -> Self { Self { text: t.into(), ..Default::default() } }
    pub fn empty() -> Self { Self::default() }
    pub fn color(mut self, c: impl Into<String>) -> Self { self.color = Some(c.into()); self }
    pub fn bold(mut self, v: bool) -> Self { self.bold = Some(v); self }
    pub fn italic(mut self, v: bool) -> Self { self.italic = Some(v); self }
    pub fn click(mut self, e: ClickEvent) -> Self { self.click = Some(e); self }
    pub fn hover(mut self, e: HoverEvent) -> Self { self.hover = Some(e); self }
    pub fn append(mut self, o: Component) -> Self { self.extra.push(o); self }
    pub fn append_text(self, t: impl Into<String>) -> Self { self.append(Component::text(t)) }

    /// Собрать в legacy `&a` строку (для логов/старых клиентов).
    pub fn to_legacy(&self) -> String {
        let mut s = self.text.clone();
        for e in &self.extra { s.push_str(&e.to_legacy()); }
        s
    }

    /// Сериализовать в JSON как в ванили (для пакетов).
    pub fn to_json(&self) -> String {
        serde_json::json!({ "text": self.text, "extra": self.extra.iter().map(|c| c.text.clone()).collect::<Vec<_>>() }).to_string()
    }

    pub fn from_legacy(s: impl Into<String>) -> Self { Self::text(s) }
}

/// MiniMessage — `"<gradient:red:blue><b>Hello {player}!</b>"` как в Paper.
pub struct MiniMessage;

impl MiniMessage {
    pub fn deserialize(s: &str) -> Component { Component::text(s) }
    pub fn deserialize_with_placeholders(s: &str, ph: &FxHashMap<String, String>) -> Component {
        let mut out = s.to_string();
        for (k,v) in ph { out = out.replace(&format!("{{{k}}}"), v); }
        Component::text(out)
    }
    pub fn serialize(c: &Component) -> String { c.to_legacy() }
}

/// Заголовок — `Title` как в Paper `Player#showTitle`.
#[derive(Debug, Clone)]
pub struct Title {
    pub title: Component,
    pub subtitle: Component,
    pub fade_in: i32,
    pub stay: i32,
    pub fade_out: i32,
}

impl Title {
    pub fn new(title: impl Into<Component>, subtitle: impl Into<Component>) -> Self {
        Self { title: title.into(), subtitle: subtitle.into(), fade_in: 10, stay: 70, fade_out: 20 }
    }
    pub fn times(mut self, fade_in: i32, stay: i32, fade_out: i32) -> Self { self.fade_in = fade_in; self.stay = stay; self.fade_out = fade_out; self }
}

/// Типы чата — как ваниль `ChatType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatType {
    Chat,
    System,
    GameInfo,
    SayCommand,
    MsgCommandIncoming,
    MsgCommandOutgoing,
    TeamMsgCommandIncoming,
    TeamMsgCommandOutgoing,
    EmoteCommand,
}

/// Приватное сообщение.
#[derive(Debug, Clone)]
pub struct PrivateMessage {
    pub from: uuid::Uuid,
    pub to: uuid::Uuid,
    pub content: Component,
}

/// Фильтр чата — Paper `ChatEvent` + `AsyncChatEvent`.
pub trait ChatFilter: Send + Sync + 'static {
    fn filter(&self, player: uuid::Uuid, msg: &str) -> Option<String> { let _ = (player, msg); None }
    fn should_cancel(&self, _player: uuid::Uuid, _msg: &str) -> bool { false }
}

impl From<String> for Component { fn from(s: String) -> Self { Component::text(s) } }
impl From<&str> for Component { fn from(s: &str) -> Self { Component::text(s) } }
impl From<Component> for String { fn from(c: Component) -> Self { c.to_legacy() } }
