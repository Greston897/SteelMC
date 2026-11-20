//! Vanilla MC 26.3 `minecraft:dialog` synchronized registry.
//!
//! Covers all 5 `minecraft:dialog_type` variants (`confirmation`,
//! `dialog_list`, `multi_action`, `notice`, `server_links`), both
//! `minecraft:dialog_body_type` variants (`item`, `plain_message`), all 4
//! `minecraft:input_control_type` variants (`boolean`, `number_range`,
//! `single_option`, `text`), and all 9 `minecraft:dialog_action_type`
//! variants (`change_page`, `copy_to_clipboard`, `custom`, `dynamic/custom`,
//! `dynamic/run_command`, `open_url`, `run_command`, `show_dialog`,
//! `suggest_command`) with NBT serialization and runtime JSON parsing for
//! custom datapack dialogs.

use rustc_hash::FxHashMap;
use simdnbt::ToNbtTag;
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use steel_utils::Identifier;
use text_components::{EmbeddedNbtCodec, TextComponent};

use crate::RegistryTags;

/// Post-click behavior shared by all dialogs (`Dialog.AfterAction`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialogAfterAction {
    #[default]
    Close,
    None,
    WaitForResponse,
}

impl DialogAfterAction {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Close => "close",
            Self::None => "none",
            Self::WaitForResponse => "wait_for_response",
        }
    }

    #[must_use]
    pub const fn from_name(name: &str) -> Option<Self> {
        match name {
            "close" => Some(Self::Close),
            "none" => Some(Self::None),
            "wait_for_response" => Some(Self::WaitForResponse),
            _ => None,
        }
    }
}

/// Represents an exit action with a label and width.
#[derive(Debug, Clone)]
pub struct ExitAction {
    pub label: TextComponent,
    pub width: i32,
}

impl ExitAction {
    #[must_use]
    pub fn to_nbt_compound(&self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        compound.insert("label", self.label.to_codec_nbt());
        compound.insert("width", self.width);
        compound
    }
}

/// One entry of a dialog `body` list.
#[derive(Debug, Clone)]
pub enum DialogBody {
    /// `minecraft:plain_message` — wrapped body text.
    PlainMessage { contents: TextComponent, width: i32 },
    /// `minecraft:item` — item-stack preview with optional description.
    Item {
        items: String,
        description: Option<TextComponent>,
        show_decorations: bool,
        show_tooltip: bool,
        width: i32,
        height: i32,
    },
}

impl DialogBody {
    #[must_use]
    pub fn to_nbt_compound(&self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        match self {
            Self::PlainMessage { contents, width } => {
                compound.insert("type", "minecraft:plain_message");
                compound.insert("contents", contents.to_codec_nbt());
                compound.insert("width", *width);
            }
            Self::Item {
                items,
                description,
                show_decorations,
                show_tooltip,
                width,
                height,
            } => {
                compound.insert("type", "minecraft:item");
                compound.insert("items", items.as_str());
                if let Some(description) = description {
                    compound.insert("description", description.to_codec_nbt());
                }
                compound.insert("show_decorations", *show_decorations);
                compound.insert("show_tooltip", *show_tooltip);
                compound.insert("width", *width);
                compound.insert("height", *height);
            }
        }
        compound
    }
}

/// One entry of a dialog `inputs` list.
#[derive(Debug, Clone)]
pub enum DialogInput {
    /// `minecraft:boolean` — true/false toggle with clickable labels.
    Boolean {
        key: String,
        label: TextComponent,
        label_visible: bool,
        initial: bool,
        on_true: String,
        on_false: String,
    },
    /// `minecraft:number_range` — slider over `[start, end]`.
    NumberRange {
        key: String,
        label: TextComponent,
        label_format: String,
        width: i32,
        start: f32,
        end: f32,
        step: Option<f32>,
        initial: Option<f32>,
    },
    /// `minecraft:single_option` — radio list with an initial selection.
    SingleOption {
        key: String,
        label: TextComponent,
        label_visible: bool,
        width: i32,
        entries: Vec<SingleOptionEntry>,
    },
    /// `minecraft:text` — free-text field with length limits.
    Text {
        key: String,
        label: TextComponent,
        label_visible: bool,
        width: i32,
        max_length: i32,
        multiline: Option<TextMultiline>,
    },
}

/// One radio entry of a `single_option` input control.
#[derive(Debug, Clone)]
pub struct SingleOptionEntry {
    pub id: String,
    pub display: Option<TextComponent>,
    pub initial: bool,
}

/// Optional multi-line settings of a `text` input control.
#[derive(Debug, Clone)]
pub struct TextMultiline {
    pub max_lines: Option<i32>,
    pub height: Option<i32>,
}

impl DialogInput {
    #[must_use]
    pub fn to_nbt_compound(&self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        match self {
            Self::Boolean {
                key,
                label,
                label_visible,
                initial,
                on_true,
                on_false,
            } => {
                compound.insert("type", "minecraft:boolean");
                compound.insert("key", key.as_str());
                compound.insert("label", label.to_codec_nbt());
                compound.insert("label_visible", *label_visible);
                compound.insert("initial", *initial);
                compound.insert("on_true", on_true.as_str());
                compound.insert("on_false", on_false.as_str());
            }
            Self::NumberRange {
                key,
                label,
                label_format,
                width,
                start,
                end,
                step,
                initial,
            } => {
                compound.insert("type", "minecraft:number_range");
                compound.insert("key", key.as_str());
                compound.insert("label", label.to_codec_nbt());
                compound.insert("label_format", label_format.as_str());
                compound.insert("width", *width);
                compound.insert("start", *start);
                compound.insert("end", *end);
                if let Some(step) = step {
                    compound.insert("step", *step);
                }
                if let Some(initial) = initial {
                    compound.insert("initial", *initial);
                }
            }
            Self::SingleOption {
                key,
                label,
                label_visible,
                width,
                entries,
            } => {
                compound.insert("type", "minecraft:single_option");
                compound.insert("key", key.as_str());
                compound.insert("label", label.to_codec_nbt());
                compound.insert("label_visible", *label_visible);
                compound.insert("width", *width);
                let list: Vec<NbtCompound> = entries
                    .iter()
                    .map(|entry| {
                        let mut e = NbtCompound::new();
                        e.insert("id", entry.id.as_str());
                        if let Some(display) = &entry.display {
                            e.insert("display", display.to_codec_nbt());
                        }
                        if entry.initial {
                            e.insert("initial", true);
                        }
                        e
                    })
                    .collect();
                compound.insert("entries", NbtTag::List(NbtList::Compound(list)));
            }
            Self::Text {
                key,
                label,
                label_visible,
                width,
                max_length,
                multiline,
            } => {
                compound.insert("type", "minecraft:text");
                compound.insert("key", key.as_str());
                compound.insert("label", label.to_codec_nbt());
                compound.insert("label_visible", *label_visible);
                compound.insert("width", *width);
                compound.insert("max_length", *max_length);
                if let Some(multiline) = multiline {
                    let mut m = NbtCompound::new();
                    if let Some(max_lines) = multiline.max_lines {
                        m.insert("max_lines", max_lines);
                    }
                    if let Some(height) = multiline.height {
                        m.insert("height", height);
                    }
                    compound.insert("multiline", NbtTag::Compound(m));
                }
            }
        }
        compound
    }
}

/// Click behavior of a dialog action button (all 9 action types).
#[derive(Debug, Clone)]
pub enum DialogClickAction {
    /// `minecraft:open_url` — opens a URL in the browser.
    OpenUrl { url: String },
    /// `minecraft:run_command` — runs a command as the clicking player.
    RunCommand { command: String },
    /// `minecraft:suggest_command` — suggests a command in chat.
    SuggestCommand { command: String },
    /// `minecraft:change_page` — turns the dialog-list page.
    ChangePage { page: i32 },
    /// `minecraft:show_dialog` — opens another dialog by id or `#tag`.
    ShowDialog { dialog: String },
    /// `minecraft:custom` — fires a custom click action with an NBT payload.
    Custom {
        id: Identifier,
        payload: Option<NbtCompound>,
    },
    /// `minecraft:dynamic/run_command` — runs a templated command with input values.
    DynamicRunCommand { template: String },
    /// `minecraft:dynamic/custom` — custom click action with templated additions.
    DynamicCustom {
        id: Identifier,
        additions: Option<NbtCompound>,
    },
    /// `minecraft:copy_to_clipboard` — copies text to the clipboard.
    CopyToClipboard { value: String },
}

impl DialogClickAction {
    #[must_use]
    pub fn to_nbt_compound(&self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        match self {
            Self::OpenUrl { url } => {
                compound.insert("type", "minecraft:open_url");
                compound.insert("url", url.as_str());
            }
            Self::RunCommand { command } => {
                compound.insert("type", "minecraft:run_command");
                compound.insert("command", command.as_str());
            }
            Self::SuggestCommand { command } => {
                compound.insert("type", "minecraft:suggest_command");
                compound.insert("command", command.as_str());
            }
            Self::ChangePage { page } => {
                compound.insert("type", "minecraft:change_page");
                compound.insert("page", *page);
            }
            Self::ShowDialog { dialog } => {
                compound.insert("type", "minecraft:show_dialog");
                compound.insert("dialog", dialog.as_str());
            }
            Self::Custom { id, payload } => {
                compound.insert("type", "minecraft:custom");
                compound.insert("id", id.to_string().as_str());
                if let Some(payload) = payload {
                    compound.insert("payload", NbtTag::Compound(payload.clone()));
                }
            }
            Self::DynamicRunCommand { template } => {
                compound.insert("type", "minecraft:dynamic/run_command");
                compound.insert("template", template.as_str());
            }
            Self::DynamicCustom { id, additions } => {
                compound.insert("type", "minecraft:dynamic/custom");
                compound.insert("id", id.to_string().as_str());
                if let Some(additions) = additions {
                    compound.insert("additions", NbtTag::Compound(additions.clone()));
                }
            }
            Self::CopyToClipboard { value } => {
                compound.insert("type", "minecraft:copy_to_clipboard");
                compound.insert("value", value.as_str());
            }
        }
        compound
    }
}

/// A labeled action button inside `confirmation`, `multi_action`, and `notice` dialogs.
#[derive(Debug, Clone)]
pub struct DialogAction {
    pub label: TextComponent,
    pub tooltip: Option<TextComponent>,
    pub width: i32,
    pub action: DialogClickAction,
}

impl DialogAction {
    #[must_use]
    pub fn to_nbt_compound(&self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        compound.insert("label", self.label.to_codec_nbt());
        if let Some(tooltip) = &self.tooltip {
            compound.insert("tooltip", tooltip.to_codec_nbt());
        }
        compound.insert("width", self.width);
        compound.insert("action", NbtTag::Compound(self.action.to_nbt_compound()));
        compound
    }
}

/// Represents a dialog defined in data packs.
#[derive(Debug, Clone)]
pub struct Dialog {
    pub key: Identifier,
    pub title: TextComponent,
    pub external_title: TextComponent,
    pub body: Vec<DialogBody>,
    pub inputs: Vec<DialogInput>,
    pub can_close_with_escape: bool,
    pub pause: bool,
    pub after_action: DialogAfterAction,
    pub variant: DialogVariant,
}

/// The variant-specific data for a dialog (all 5 `dialog_type`s).
#[derive(Debug, Clone)]
pub enum DialogVariant {
    /// A two-button confirmation dialog.
    Confirmation {
        yes: DialogAction,
        no: Box<DialogAction>,
    },
    /// A dialog that displays a list of other dialogs.
    DialogList {
        dialogs: &'static str,
        exit_action: ExitAction,
        columns: i32,
        button_width: i32,
    },
    /// A dialog with a custom list of action buttons.
    MultiAction {
        actions: Vec<DialogAction>,
        exit_action: ExitAction,
        columns: i32,
        button_width: i32,
    },
    /// A single-action notice dialog.
    Notice { action: DialogAction },
    /// A dialog that displays server links.
    ServerLinks {
        exit_action: ExitAction,
        columns: i32,
        button_width: i32,
    },
}

impl Dialog {
    /// Serializes the full 26.3 dialog codec (shared by network sync and direct
    /// inline definitions — dialogs have no server-only fields).
    #[must_use]
    pub fn to_nbt_compound(&self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        compound.insert(
            "type",
            match &self.variant {
                DialogVariant::Confirmation { .. } => "minecraft:confirmation",
                DialogVariant::DialogList { .. } => "minecraft:dialog_list",
                DialogVariant::MultiAction { .. } => "minecraft:multi_action",
                DialogVariant::Notice { .. } => "minecraft:notice",
                DialogVariant::ServerLinks { .. } => "minecraft:server_links",
            },
        );
        compound.insert("title", self.title.to_codec_nbt());
        compound.insert("external_title", self.external_title.to_codec_nbt());
        if !self.body.is_empty() {
            let list: Vec<NbtCompound> =
                self.body.iter().map(DialogBody::to_nbt_compound).collect();
            compound.insert("body", NbtTag::List(NbtList::Compound(list)));
        }
        if !self.inputs.is_empty() {
            let list: Vec<NbtCompound> = self
                .inputs
                .iter()
                .map(DialogInput::to_nbt_compound)
                .collect();
            compound.insert("inputs", NbtTag::List(NbtList::Compound(list)));
        }
        if !self.can_close_with_escape {
            compound.insert("can_close_with_escape", false);
        }
        if !self.pause {
            compound.insert("pause", false);
        }
        if self.after_action != DialogAfterAction::Close {
            compound.insert("after_action", self.after_action.as_str());
        }
        match &self.variant {
            DialogVariant::Confirmation { yes, no } => {
                compound.insert("yes", NbtTag::Compound(yes.to_nbt_compound()));
                compound.insert("no", NbtTag::Compound(no.to_nbt_compound()));
            }
            DialogVariant::DialogList {
                dialogs,
                exit_action,
                columns,
                button_width,
            } => {
                compound.insert("dialogs", *dialogs);
                compound.insert(
                    "exit_action",
                    NbtTag::Compound(exit_action.to_nbt_compound()),
                );
                compound.insert("columns", *columns);
                compound.insert("button_width", *button_width);
            }
            DialogVariant::MultiAction {
                actions,
                exit_action,
                columns,
                button_width,
            } => {
                let list: Vec<NbtCompound> =
                    actions.iter().map(DialogAction::to_nbt_compound).collect();
                compound.insert("actions", NbtTag::List(NbtList::Compound(list)));
                compound.insert(
                    "exit_action",
                    NbtTag::Compound(exit_action.to_nbt_compound()),
                );
                compound.insert("columns", *columns);
                compound.insert("button_width", *button_width);
            }
            DialogVariant::Notice { action } => {
                compound.insert("action", NbtTag::Compound(action.to_nbt_compound()));
            }
            DialogVariant::ServerLinks {
                exit_action,
                columns,
                button_width,
            } => {
                compound.insert(
                    "exit_action",
                    NbtTag::Compound(exit_action.to_nbt_compound()),
                );
                compound.insert("columns", *columns);
                compound.insert("button_width", *button_width);
            }
        }
        compound
    }

    /// Parses a custom datapack dialog JSON definition (`data/<ns>/dialog/*.json`).
    ///
    /// # Errors
    ///
    /// Returns a human-readable error when a required field is missing or a
    /// `type` discriminator is unknown.
    pub fn from_json(key: Identifier, json: &serde_json::Value) -> Result<Self, String> {
        let obj = json
            .as_object()
            .ok_or_else(|| format!("dialog {key} must be a JSON object"))?;
        let raw_type = obj
            .get("type")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("dialog {key} is missing required string field 'type'"))?;
        let title = parse_component(obj, "title", &key)?;
        let external_title = parse_component(obj, "external_title", &key)?;
        let body = obj
            .get("body")
            .map(|v| parse_body_list(v, &key))
            .transpose()?
            .unwrap_or_default();
        let inputs = obj
            .get("inputs")
            .map(|v| parse_input_list(v, &key))
            .transpose()?
            .unwrap_or_default();
        let can_close_with_escape = obj
            .get("can_close_with_escape")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let pause = obj
            .get("pause")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let after_action = match obj.get("after_action").and_then(serde_json::Value::as_str) {
            None => DialogAfterAction::Close,
            Some(name) => DialogAfterAction::from_name(name)
                .ok_or_else(|| format!("dialog {key} has unknown after_action '{name}'"))?,
        };
        let type_name = raw_type.strip_prefix("minecraft:").unwrap_or(raw_type);
        let variant = match type_name {
            "confirmation" => DialogVariant::Confirmation {
                yes: parse_action(obj, "yes", &key)?,
                no: Box::new(parse_action(obj, "no", &key)?),
            },
            "dialog_list" => DialogVariant::DialogList {
                dialogs: leak_str(
                    obj.get("dialogs")
                        .and_then(serde_json::Value::as_str)
                        .ok_or_else(|| {
                            format!("dialog {key} is missing required field 'dialogs'")
                        })?,
                ),
                exit_action: parse_exit_action(obj, &key)?,
                columns: obj
                    .get("columns")
                    .and_then(serde_json::Value::as_i64)
                    .map_or(2, |v| v as i32),
                button_width: obj
                    .get("button_width")
                    .and_then(serde_json::Value::as_i64)
                    .map_or(150, |v| v as i32),
            },
            "multi_action" => DialogVariant::MultiAction {
                actions: parse_action_list(obj.get("actions"), &key)?,
                exit_action: parse_exit_action(obj, &key)?,
                columns: obj
                    .get("columns")
                    .and_then(serde_json::Value::as_i64)
                    .map_or(2, |v| v as i32),
                button_width: obj
                    .get("button_width")
                    .and_then(serde_json::Value::as_i64)
                    .map_or(150, |v| v as i32),
            },
            "notice" => DialogVariant::Notice {
                action: parse_action(obj, "action", &key)?,
            },
            "server_links" => DialogVariant::ServerLinks {
                exit_action: parse_exit_action(obj, &key)?,
                columns: obj
                    .get("columns")
                    .and_then(serde_json::Value::as_i64)
                    .map_or(2, |v| v as i32),
                button_width: obj
                    .get("button_width")
                    .and_then(serde_json::Value::as_i64)
                    .map_or(150, |v| v as i32),
            },
            unknown => return Err(format!("dialog {key} has unknown type '{unknown}'")),
        };
        Ok(Self {
            key,
            title,
            external_title,
            body,
            inputs,
            can_close_with_escape,
            pause,
            after_action,
            variant,
        })
    }
}

fn leak_str(value: &str) -> &'static str {
    Box::leak(value.to_owned().into_boxed_str())
}

fn parse_component(
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    key: &Identifier,
) -> Result<TextComponent, String> {
    let value = obj
        .get(field)
        .ok_or_else(|| format!("dialog {key} is missing required field '{field}'"))?;
    serde_json::from_value(value.clone())
        .map_err(|e| format!("dialog {key} has invalid text component in '{field}': {e}"))
}

fn parse_optional_component(
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    key: &Identifier,
) -> Result<Option<TextComponent>, String> {
    obj.get(field)
        .map(|value| {
            serde_json::from_value(value.clone())
                .map_err(|e| format!("dialog {key} has invalid text component in '{field}': {e}"))
        })
        .transpose()
}

fn parse_body_list(value: &serde_json::Value, key: &Identifier) -> Result<Vec<DialogBody>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("dialog {key} field 'body' must be a list"))?;
    items
        .iter()
        .map(|item| {
            let obj = item
                .as_object()
                .ok_or_else(|| format!("dialog {key} has a non-object body entry"))?;
            let raw_type = obj
                .get("type")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| format!("dialog {key} body entry is missing 'type'"))?;
            match raw_type.strip_prefix("minecraft:").unwrap_or(raw_type) {
                "plain_message" => Ok(DialogBody::PlainMessage {
                    contents: parse_component(obj, "contents", key)?,
                    width: obj
                        .get("width")
                        .and_then(serde_json::Value::as_i64)
                        .map_or(200, |v| v as i32),
                }),
                "item" => Ok(DialogBody::Item {
                    items: obj
                        .get("items")
                        .and_then(serde_json::Value::as_str)
                        .ok_or_else(|| format!("dialog {key} item body is missing 'items'"))?
                        .to_owned(),
                    description: parse_optional_component(obj, "description", key)?,
                    show_decorations: obj
                        .get("show_decorations")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(true),
                    show_tooltip: obj
                        .get("show_tooltip")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(true),
                    width: obj
                        .get("width")
                        .and_then(serde_json::Value::as_i64)
                        .map_or(16, |v| v as i32),
                    height: obj
                        .get("height")
                        .and_then(serde_json::Value::as_i64)
                        .map_or(16, |v| v as i32),
                }),
                unknown => Err(format!("dialog {key} has unknown body type '{unknown}'")),
            }
        })
        .collect()
}

fn parse_input_list(
    value: &serde_json::Value,
    key: &Identifier,
) -> Result<Vec<DialogInput>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("dialog {key} field 'inputs' must be a list"))?;
    items
        .iter()
        .map(|item| {
            let obj = item
                .as_object()
                .ok_or_else(|| format!("dialog {key} has a non-object input entry"))?;
            let raw_type = obj
                .get("type")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| format!("dialog {key} input entry is missing 'type'"))?;
            let input_key = obj
                .get("key")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| format!("dialog {key} input entry is missing 'key'"))?
                .to_owned();
            match raw_type.strip_prefix("minecraft:").unwrap_or(raw_type) {
                "boolean" => Ok(DialogInput::Boolean {
                    key: input_key,
                    label: parse_component(obj, "label", key)?,
                    label_visible: obj
                        .get("label_visible")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(true),
                    initial: obj
                        .get("initial")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false),
                    on_true: obj
                        .get("on_true")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("true")
                        .to_owned(),
                    on_false: obj
                        .get("on_false")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("false")
                        .to_owned(),
                }),
                "number_range" => Ok(DialogInput::NumberRange {
                    key: input_key,
                    label: parse_component(obj, "label", key)?,
                    label_format: obj
                        .get("label_format")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("options.generic_value")
                        .to_owned(),
                    width: obj
                        .get("width")
                        .and_then(serde_json::Value::as_i64)
                        .map_or(200, |v| v as i32),
                    start: obj
                        .get("start")
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(0.0) as f32,
                    end: obj
                        .get("end")
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(1.0) as f32,
                    step: obj
                        .get("step")
                        .and_then(serde_json::Value::as_f64)
                        .map(|v| v as f32),
                    initial: obj
                        .get("initial")
                        .and_then(serde_json::Value::as_f64)
                        .map(|v| v as f32),
                }),
                "single_option" => {
                    let entries = obj
                        .get("entries")
                        .and_then(serde_json::Value::as_array)
                        .ok_or_else(|| {
                            format!("dialog {key} single_option input is missing 'entries'")
                        })?;
                    let mut parsed = Vec::with_capacity(entries.len());
                    for entry in entries {
                        let entry_obj = entry.as_object().ok_or_else(|| {
                            format!("dialog {key} has a non-object single_option entry")
                        })?;
                        parsed.push(SingleOptionEntry {
                            id: entry_obj
                                .get("id")
                                .and_then(serde_json::Value::as_str)
                                .ok_or_else(|| {
                                    format!("dialog {key} single_option entry is missing 'id'")
                                })?
                                .to_owned(),
                            display: parse_optional_component(entry_obj, "display", key)?,
                            initial: entry_obj
                                .get("initial")
                                .and_then(serde_json::Value::as_bool)
                                .unwrap_or(false),
                        });
                    }
                    Ok(DialogInput::SingleOption {
                        key: input_key,
                        label: parse_component(obj, "label", key)?,
                        label_visible: obj
                            .get("label_visible")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(true),
                        width: obj
                            .get("width")
                            .and_then(serde_json::Value::as_i64)
                            .map_or(200, |v| v as i32),
                        entries: parsed,
                    })
                }
                "text" => Ok(DialogInput::Text {
                    key: input_key,
                    label: parse_component(obj, "label", key)?,
                    label_visible: obj
                        .get("label_visible")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(true),
                    width: obj
                        .get("width")
                        .and_then(serde_json::Value::as_i64)
                        .map_or(200, |v| v as i32),
                    max_length: obj
                        .get("max_length")
                        .and_then(serde_json::Value::as_i64)
                        .map_or(32, |v| v as i32),
                    multiline: obj
                        .get("multiline")
                        .and_then(serde_json::Value::as_object)
                        .map(|m| TextMultiline {
                            max_lines: m
                                .get("max_lines")
                                .and_then(serde_json::Value::as_i64)
                                .map(|v| v as i32),
                            height: m
                                .get("height")
                                .and_then(serde_json::Value::as_i64)
                                .map(|v| v as i32),
                        }),
                }),
                unknown => Err(format!("dialog {key} has unknown input type '{unknown}'")),
            }
        })
        .collect()
}

fn parse_nbt_payload(
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    key: &Identifier,
) -> Result<Option<NbtCompound>, String> {
    let Some(value) = obj.get(field) else {
        return Ok(None);
    };
    match steel_utils::nbt::json_to_nbt(value) {
        Ok(NbtTag::Compound(compound)) => Ok(Some(compound)),
        Ok(_) => Err(format!(
            "dialog {key} field '{field}' must be a JSON object"
        )),
        Err(e) => Err(format!("dialog {key} field '{field}' is invalid NBT: {e}")),
    }
}

fn parse_click_action(
    value: &serde_json::Value,
    key: &Identifier,
) -> Result<DialogClickAction, String> {
    let obj = value
        .as_object()
        .ok_or_else(|| format!("dialog {key} has a non-object action"))?;
    let raw_type = obj
        .get("type")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("dialog {key} action is missing 'type'"))?;
    let get_str = |field: &str| {
        obj.get(field)
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("dialog {key} action is missing '{field}'"))
            .map(str::to_owned)
    };
    match raw_type.strip_prefix("minecraft:").unwrap_or(raw_type) {
        "open_url" => Ok(DialogClickAction::OpenUrl {
            url: get_str("url")?,
        }),
        "run_command" => Ok(DialogClickAction::RunCommand {
            command: get_str("command")?,
        }),
        "suggest_command" => Ok(DialogClickAction::SuggestCommand {
            command: get_str("command")?,
        }),
        "change_page" => Ok(DialogClickAction::ChangePage {
            page: obj
                .get("page")
                .and_then(serde_json::Value::as_i64)
                .map_or(0, |v| v as i32),
        }),
        "show_dialog" => Ok(DialogClickAction::ShowDialog {
            dialog: get_str("dialog")?,
        }),
        "custom" => Ok(DialogClickAction::Custom {
            id: obj
                .get("id")
                .and_then(serde_json::Value::as_str)
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| format!("dialog {key} custom action is missing valid 'id'"))?,
            payload: parse_nbt_payload(obj, "payload", key)?,
        }),
        "dynamic/run_command" => Ok(DialogClickAction::DynamicRunCommand {
            template: get_str("template")?,
        }),
        "dynamic/custom" => Ok(DialogClickAction::DynamicCustom {
            id: obj
                .get("id")
                .and_then(serde_json::Value::as_str)
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| {
                    format!("dialog {key} dynamic custom action is missing valid 'id'")
                })?,
            additions: parse_nbt_payload(obj, "additions", key)?,
        }),
        "copy_to_clipboard" => Ok(DialogClickAction::CopyToClipboard {
            value: get_str("value")?,
        }),
        unknown => Err(format!("dialog {key} has unknown action type '{unknown}'")),
    }
}

fn parse_action(
    obj: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    key: &Identifier,
) -> Result<DialogAction, String> {
    let obj = obj
        .get(field)
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| format!("dialog {key} is missing required field '{field}'"))?;
    Ok(DialogAction {
        label: parse_component(obj, "label", key)?,
        tooltip: parse_optional_component(obj, "tooltip", key)?,
        width: obj
            .get("width")
            .and_then(serde_json::Value::as_i64)
            .map_or(150, |v| v as i32),
        action: parse_click_action(
            obj.get("action")
                .ok_or_else(|| format!("dialog {key} action button is missing 'action'"))?,
            key,
        )?,
    })
}

fn parse_action_list(
    value: Option<&serde_json::Value>,
    key: &Identifier,
) -> Result<Vec<DialogAction>, String> {
    let items = value
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("dialog {key} is missing required list field 'actions'"))?;
    items
        .iter()
        .map(|item| {
            let obj = item
                .as_object()
                .ok_or_else(|| format!("dialog {key} has a non-object action entry"))?;
            Ok(DialogAction {
                label: parse_component(obj, "label", key)?,
                tooltip: parse_optional_component(obj, "tooltip", key)?,
                width: obj
                    .get("width")
                    .and_then(serde_json::Value::as_i64)
                    .map_or(150, |v| v as i32),
                action: parse_click_action(
                    obj.get("action")
                        .ok_or_else(|| format!("dialog {key} action button is missing 'action'"))?,
                    key,
                )?,
            })
        })
        .collect()
}

fn parse_exit_action(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &Identifier,
) -> Result<ExitAction, String> {
    let obj = obj
        .get("exit_action")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| format!("dialog {key} is missing required field 'exit_action'"))?;
    Ok(ExitAction {
        label: parse_component(obj, "label", key)?,
        width: obj
            .get("width")
            .and_then(serde_json::Value::as_i64)
            .map_or(150, |v| v as i32),
    })
}

impl ToNbtTag for &Dialog {
    fn to_nbt_tag(self) -> NbtTag {
        NbtTag::Compound(self.to_nbt_compound())
    }
}

impl EmbeddedNbtCodec for &Dialog {
    type Error = std::convert::Infallible;

    fn encode_embedded_nbt(self) -> Result<NbtTag, Self::Error> {
        Ok(self.to_nbt_tag())
    }
}

pub type DialogRef = &'static Dialog;

pub struct DialogRegistry {
    dialogs_by_id: Vec<DialogRef>,
    dialogs_by_key: FxHashMap<Identifier, usize>,
    tags: RegistryTags,
    allows_registering: bool,
}

impl DialogRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            dialogs_by_id: Vec::new(),
            dialogs_by_key: FxHashMap::default(),
            tags: RegistryTags::default(),
            allows_registering: true,
        }
    }
}

crate::impl_standard_methods!(
    DialogRegistry,
    DialogRef,
    dialogs_by_id,
    dialogs_by_key,
    allows_registering
);

crate::impl_registry!(
    DialogRegistry,
    Dialog,
    dialogs_by_id,
    dialogs_by_key,
    dialogs
);
crate::impl_tagged_registry!(DialogRegistry, dialogs_by_key, "dialog");
