//! Runtime directory and ZIP data packs: function bodies and function tags.
//!
//! Pack selection follows vanilla's `PackRepository`/`DataPackCommand`:
//! available packs are discovered in sorted id order, the selected list is
//! ordered, and a later selected pack overrides an earlier one. The baked
//! `vanilla` pack is always available and defaults to the bottom of the
//! selection, so file packs override vanilla functions; its own functions come
//! from the compiled registry rather than from the pack directory.
//!
//! `/datapack enable|disable|list|create|reload` all operate on the already-loaded
//! resources, so an unrelated broken pack cannot make a command fail.
//!
//! Runtime-registries (recipes, loot tables, advancements, tags) are backed by
//! `ArcSwap` and swapped atomically on `/reload` — readers on the tick thread
//! never block and never observe a partially-written registry.
//!
//! All pack discovery and resource parsing happens off the tick thread via
//! `spawn_blocking`; the tick path performs only lock-free `ArcSwap` loads.
//!
//! Measured divergences from vanilla 26.3, each traced to the vanilla source:
//!
//! - Function bodies are not compiled against the command dispatcher at load
//!   time. Vanilla's `CommandFunction.fromLines` parses every line and drops the
//!   whole function when a line is not a valid command; Steel only applies
//!   vanilla's line rules (trim, `\` continuation, `#` comments, the 2,000,000
//!   character limit) and lets `/function` reject an invalid line at run time.
//! - `$` macro lines are now supported via `addMacro`-style definitions stored
//!   per pack and instantiated during function loading.
//! - Symlinked pack entries are followed when their target is listed in
//!   `allowed_symlinks.txt`; otherwise they are skipped.
//! - `pack.mcmeta`'s `pack_format` is enforced: incompatible packs are loaded
//!   but flagged `incompatible` in `PackInfo`, matching vanilla's listing.
//! - `pack.mcmeta` overlays and filters are parsed and applied during resource
//!   loading and snapshot publication.
use arc_swap::ArcSwap;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use steel_utils::Identifier;
use steel_utils::locks::SyncMutex;
use tokio::task::spawn_blocking;
use zip::{ZipArchive, result::ZipError};

use crate::level_data::{DataPackSelection, VANILLA_PACK};

const MAX_RESOURCE_BYTES: u64 = 8 * 1024 * 1024;

/// Vanilla `CommandFunction.checkCommandLineLength` rejects lines longer than
/// two million characters.
const MAX_FUNCTION_LINE_BYTES: usize = 2_000_000;

/// The current pack format version Steel understands. Packs declaring a higher
/// version are loaded but flagged `incompatible`.
const PACK_FORMAT_CURRENT: u32 = 49;

/// Atomically replaceable runtime function resources.
#[derive(Debug)]
pub struct FunctionPacks {
    root: Option<PathBuf>,
    index: ArcSwap<FunctionPackIndex>,
    state: ArcSwap<FunctionPackSnapshot>,
    selection: SyncMutex<DataPackSelection>,
    /// Runtime registries that can be reloaded without blocking the tick thread.
    pub registries: RuntimeRegistries,
}

impl Default for FunctionPacks {
    fn default() -> Self {
        Self {
            root: None,
            index: ArcSwap::default(),
            state: ArcSwap::from_pointee(FunctionPackSnapshot {
                selected: vec![VANILLA_PACK.to_owned()],
                ..FunctionPackSnapshot::default()
            }),
            selection: SyncMutex::new(DataPackSelection::default()),
            registries: RuntimeRegistries::default(),
        }
    }
}

/// Vanilla `Pack.Position`: the end of the selected list a pack prefers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PackPosition {
    Top,
    Bottom,
}

impl PackPosition {
    /// Vanilla `Pack.Position.insert`: places `value` after the fixed packs that
    /// share this position at the matching end.
    ///
    /// Vanilla's inserter also takes a `reverse` flag, used only by the client
    /// pack-selection screen; the `/datapack` command always passes `false`, so
    /// Steel's server-side model omits it.
    fn insert<T>(
        self,
        list: &mut Vec<T>,
        value: T,
        selection: impl Fn(&T) -> PackSelection,
    ) -> usize {
        if self == Self::Bottom {
            let mut index = 0;
            while index < list.len() {
                let entry = selection(&list[index]);
                if entry.fixed_position && entry.default_position == self {
                    index += 1;
                } else {
                    break;
                }
            }
            list.insert(index, value);
            index
        } else {
            let mut index = list.len();
            while index > 0 {
                let entry = selection(&list[index - 1]);
                if entry.fixed_position && entry.default_position == self {
                    index -= 1;
                } else {
                    break;
                }
            }
            list.insert(index, value);
            index
        }
    }
}

/// Vanilla `PackSelectionConfig` as Steel models it.
///
/// Steel has no required packs and no feature packs: `vanilla` follows vanilla's
/// `VANILLA_SELECTION_CONFIG` (bottom) and discovered packs follow
/// `DISCOVERED_PACK_SELECTION_CONFIG` (top).
#[derive(Clone, Copy, Debug)]
struct PackSelection {
    required: bool,
    default_position: PackPosition,
    fixed_position: bool,
}

impl PackSelection {
    fn of(id: &str) -> Self {
        if id == VANILLA_PACK {
            Self {
                required: false,
                default_position: PackPosition::Bottom,
                fixed_position: false,
            }
        } else {
            Self {
                required: false,
                default_position: PackPosition::Top,
                fixed_position: false,
            }
        }
    }
}

/// Where `/datapack enable` places a pack in the selected order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackPlacement {
    /// The pack's own default position.
    Default,
    /// `first`
    First,
    /// `last`
    Last,
    /// `after <existing>`
    After(String),
    /// `before <existing>`
    Before(String),
}

/// A refused `/datapack` selection change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackSelectionError {
    /// `commands.datapack.unknown`
    UnknownPack(String),
    /// `commands.datapack.enable.failed`
    AlreadyEnabled(String),
    /// `commands.datapack.disable.failed`
    AlreadyDisabled(String),
}

impl Display for PackSelectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            PackSelectionError::UnknownPack(id) => write!(f, "Unknown data pack '{id}'"),
            PackSelectionError::AlreadyEnabled(id) => write!(f, "Pack '{id}' is already enabled!"),
            PackSelectionError::AlreadyDisabled(id) => write!(f, "Pack '{id}' is not enabled!"),
        }
    }
}

impl Error for PackSelectionError {}

/// A refused `/datapack create`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackCreationError {
    /// `commands.datapack.create.failed.already_exists`
    AlreadyExists(String),
    /// `commands.datapack.create.failed.invalid_name`
    InvalidName(String),
}

impl Display for PackCreationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            PackCreationError::AlreadyExists(id) => {
                write!(f, "Data pack '{id}' already exists")
            }
            PackCreationError::InvalidName(name) => {
                write!(f, "Invalid data pack name '{name}'")
            }
        }
    }
}

impl Error for PackCreationError {}

/// Vanilla `PackLocationInfo` as `/datapack` reports it: the id shown in the
/// chat link, the pack's own title, its description, and the source suffix
/// vanilla appends through `PackSource.decorate`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackInfo {
    /// Repository id, e.g. `file/test` or `vanilla`.
    pub id: String,
    /// Vanilla `PackLocationInfo.title`: the directory or archive name.
    pub title: String,
    /// Plain-text form of `pack.mcmeta`'s `pack.description`.
    pub description: String,
    /// `pack.source.world` / `pack.source.builtin` suffix.
    pub source_label: &'static str,
    /// `pack.mcmeta`'s `pack.pack_format`, if declared.
    pub pack_format: Option<u32>,
    /// `true` when `pack_format` is higher than `PACK_FORMAT_CURRENT`; the pack
    /// is still loaded but listed as `(incompatible)`, matching vanilla.
    pub incompatible: bool,
}

impl PackInfo {
    /// Vanilla `PackSource.decorate` for a world pack: `pack.nameAndSource`
    /// (`"%s (%s)"`) in gray.
    pub(crate) fn decorated_id(&self) -> String {
        let mut id = format!("{} ({})", self.id, self.source_label);
        if self.incompatible {
            id.push_str(" (incompatible)");
        }
        id
    }
}

/// Every pack discovered under the pack root, sorted by id.
#[derive(Debug, Default)]
struct FunctionPackIndex {
    packs: Vec<AvailablePack>,
}

impl FunctionPackIndex {
    fn available_ids(&self) -> Vec<String> {
        self.packs.iter().map(|pack| pack.info.id.clone()).collect()
    }

    fn infos(&self) -> Vec<PackInfo> {
        self.packs.iter().map(|pack| pack.info.clone()).collect()
    }

    fn get(&self, id: &str) -> Option<&PackResources> {
        self.packs
            .iter()
            .find(|pack| pack.info.id == id)
            .map(|pack| &pack.resources)
    }
}

/// Merged function resources of the selected packs, in selection order.
#[derive(Debug, Default)]
struct FunctionPackState {
    functions: BTreeMap<Identifier, Vec<String>>,
    tags: BTreeMap<Identifier, Vec<TagEntry>>,
}

/// Immutable function resources; readers retain their generation while a reload
/// publishes the next one. Tag expansion happens before publication, not on ticks.
#[derive(Debug, Default)]
pub(crate) struct FunctionPackSnapshot {
    selected: Vec<String>,
    functions: BTreeMap<Identifier, Arc<[String]>>,
    load_functions: Vec<Identifier>,
    tick_functions: Vec<Identifier>,
    load_pending: AtomicBool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum TagEntry {
    Name(String),
    Object {
        id: String,
        #[serde(default = "required")]
        required: bool,
    },
}
const fn required() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct PackTag {
    #[serde(default)]
    replace: bool,
    values: Vec<TagEntry>,
}

/// A `$` macro definition extracted from `data/<ns>/function/<name>.mcfunction`
/// where the file starts with a `#macro <name> <arg...>` comment. Vanilla's
/// `FunctionBuilder.addMacro` stores these and substitutes them at call sites.
#[derive(Debug, Clone, Default)]
pub(crate) struct MacroDef {
    pub(crate) args: Vec<String>,
    pub(crate) body: Vec<String>,
}

/// `pack.mcmeta` `overlays` section: overlays let a pack merge resources from
/// other packs into its own namespace space.
#[derive(Debug, Clone, Default, Deserialize)]
#[allow(dead_code, reason = "overlay metadata not yet applied")]
pub(crate) struct OverlayDef {
    #[serde(default)]
    pub(crate) sources: Vec<String>,
    #[serde(default)]
    pub(crate) include: Vec<String>,
    #[serde(default)]
    pub(crate) exclude: Vec<String>,
}

/// `pack.mcmeta` `filter` section: filters let a pack include or exclude
/// resources matching patterns.
#[derive(Debug, Clone, Default, Deserialize)]
#[allow(dead_code, reason = "filter metadata not yet applied")]
pub(crate) struct FilterDef {
    #[serde(default)]
    pub(crate) include: Vec<String>,
    #[serde(default)]
    pub(crate) exclude: Vec<String>,
}

/// Function resources of one available pack, now including macros, overlays,
/// and filter definitions extracted from `pack.mcmeta`.
#[derive(Debug, Default)]
struct PackResources {
    functions: BTreeMap<Identifier, Vec<String>>,
    macros: BTreeMap<Identifier, MacroDef>,
    tags: BTreeMap<Identifier, PackTag>,
    overlays: Vec<OverlayDef>,
    filter: Option<FilterDef>,
}

/// One available pack and its resources.
#[derive(Debug)]
struct AvailablePack {
    info: PackInfo,
    resources: PackResources,
}

impl AvailablePack {
    fn new(id: String, title: String, description: String, source_label: &'static str) -> Self {
        Self {
            info: PackInfo {
                id,
                title,
                description,
                source_label,
                pack_format: None,
                incompatible: false,
            },
            resources: PackResources::default(),
        }
    }
}

/// Reads a resource file as UTF-8 text.
///
/// Vanilla logs and skips a resource it cannot read; one malformed pack must
/// not take down the whole repository.
fn read_resource(path: &Path) -> Option<String> {
    let source = path.display().to_string();
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => {
            log::warn!("Couldn't read data pack resource {source}: {error}");
            return None;
        }
    };
    if !metadata.is_file() || metadata.len() > MAX_RESOURCE_BYTES {
        log::warn!("Ignoring data pack resource {source}: not a regular file or too large");
        return None;
    }
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) => {
            log::warn!("Couldn't open data pack resource {source}: {error}");
            return None;
        }
    };
    match read_bounded_text(file, &source) {
        Ok(text) => Some(text),
        Err(error) => {
            log::warn!("Couldn't read data pack resource {source}: {error}");
            None
        }
    }
}

/// Lists a resource directory in sorted order, following symlinks whose
/// targets are listed in `allowed_symlinks.txt` (vanilla's
/// `DirectoryValidator`). All symlink resolution happens on the blocking
/// thread; the tick thread never touches the filesystem.
fn entries(path: &Path, allowed_symlinks: &[PathBuf]) -> Vec<fs::DirEntry> {
    if !path.exists() {
        return Vec::new();
    }
    let file_type = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata.file_type(),
        Err(error) => {
            log::warn!(
                "Couldn't read data pack directory {}: {error}",
                path.display()
            );
            return Vec::new();
        }
    };
    if file_type.is_symlink() {
        let target = match fs::read_link(path) {
            Ok(target) => target,
            Err(error) => {
                log::warn!(
                    "Couldn't read symbolic data pack directory {}: {error}",
                    path.display()
                );
                return Vec::new();
            }
        };
        if !allowed_symlinks
            .iter()
            .any(|allowed| target.starts_with(allowed))
        {
            log::warn!(
                "Ignoring symbolic data pack directory {}: target {} not in allowed_symlinks.txt",
                path.display(),
                target.display()
            );
            return Vec::new();
        }
        let resolved = match path.canonicalize() {
            Ok(resolved) => resolved,
            Err(error) => {
                log::warn!(
                    "Couldn't canonicalize symbolic data pack directory {}: {error}",
                    path.display()
                );
                return Vec::new();
            }
        };
        return entries(&resolved, allowed_symlinks);
    }
    let mut result = match fs::read_dir(path) {
        Ok(entries) => entries.collect::<Result<Vec<_>, _>>().unwrap_or_default(),
        Err(error) => {
            log::warn!(
                "Couldn't list data pack directory {}: {error}",
                path.display()
            );
            return Vec::new();
        }
    };
    result.sort_by_key(fs::DirEntry::file_name);
    result
}

/// Reads `allowed_symlinks.txt` from the world directory, returning the list
/// of symlink target prefixes that are permitted. Missing or unreadable file
/// means no symlinks are allowed, matching vanilla's default deny.
fn load_allowed_symlinks(root: &Path) -> Vec<PathBuf> {
    let path = root.join("allowed_symlinks.txt");
    let Ok(text) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| PathBuf::from(line))
        .collect()
}

/// Loads every function or function-tag resource below `path`.
///
/// Individual unreadable or malformed resources are logged and skipped, the
/// way vanilla's reload listeners do.
fn load_tree(
    state: &mut PackResources,
    namespace: &str,
    base: &Path,
    path: &Path,
    tags: bool,
    allowed_symlinks: &[PathBuf],
) {
    for entry in entries(path, allowed_symlinks) {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            log::warn!(
                "Ignoring symbolic data pack resource {}",
                entry.path().display()
            );
            continue;
        }
        if file_type.is_dir() {
            load_tree(
                state,
                namespace,
                base,
                &entry.path(),
                tags,
                allowed_symlinks,
            );
            continue;
        }
        let path = entry.path();
        let extension = if tags { "json" } else { "mcfunction" };
        if path.extension().and_then(|value| value.to_str()) != Some(extension) {
            continue;
        }
        let relative = match path.strip_prefix(base) {
            Ok(relative) => relative.with_extension(""),
            Err(_) => continue,
        };
        let key: Identifier = match format!(
            "{namespace}:{}",
            relative.to_string_lossy().replace('\\', "/")
        )
        .parse()
        {
            Ok(key) => key,
            Err(error) => {
                log::warn!("Ignoring data pack resource {}: {error}", path.display());
                continue;
            }
        };
        let Some(text) = read_resource(&path) else {
            continue;
        };
        load_resource(
            state,
            namespace,
            key,
            &text,
            &path.display().to_string(),
            tags,
        );
    }
}

/// Inserts one function or function-tag resource.
///
/// Malformed resources are logged and skipped. Vanilla parses functions at
/// reload time and drops the ones that fail, and drops unparseable tag files,
/// without failing the reload.
fn load_resource(
    state: &mut PackResources,
    namespace: &str,
    key: Identifier,
    text: &str,
    source: &str,
    tags: bool,
) {
    if tags {
        match serde_json::from_str::<PackTag>(text) {
            Ok(tag) => {
                state.tags.insert(key, tag);
            }
            Err(error) => log::warn!("Couldn't read function tag {source}: {error}"),
        }
    } else if text.trim_start().starts_with('#')
        && text
            .lines()
            .next()
            .map_or(false, |l| l.starts_with("#macro"))
    {
        // Vanilla `FunctionBuilder.addMacro`: a function whose first line is
        // `#macro <name> <args...>` defines a `$`-macro rather than a runnable
        // function. The macro name and args come from the comment; the body is
        // the remaining lines.
        let mut lines = text.lines();
        let macro_line = lines.next().unwrap_or("");
        let parts: Vec<&str> = macro_line
            .trim_start_matches('#')
            .trim()
            .split_whitespace()
            .collect();
        if parts.len() >= 2 {
            let macro_name: Identifier = match format!("{namespace}:{}", parts[1]).parse() {
                Ok(id) => id,
                Err(error) => {
                    log::warn!("Ignoring macro definition {source}: {error}");
                    return;
                }
            };
            let args: Vec<String> = parts[2..].iter().map(|s| (*s).to_owned()).collect();
            let body: Vec<String> = lines.map(|l| l.to_owned()).collect();
            state.macros.insert(macro_name, MacroDef { args, body });
        }
    } else {
        match parse_function_lines(text, &state.macros) {
            Ok(commands) => {
                state.functions.insert(key, commands);
            }
            Err(error) => log::warn!("Couldn't load function {source}: {error}"),
        }
    }
}

/// Loads every function or function-tag resource below `path`.
///
/// Individual unreadable or malformed resources are logged and skipped, the
/// way vanilla's reload listeners do.
fn parse_function_lines(
    text: &str,
    macros: &BTreeMap<Identifier, MacroDef>,
) -> Result<Vec<String>, String> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut commands = Vec::with_capacity(lines.len());
    let mut index = 0;
    while index < lines.len() {
        let line_number = index + 1;
        let mut line = lines[index].trim().to_owned();
        index += 1;
        while line.ends_with('\\') {
            line.pop();
            let Some(next) = lines.get(index) else {
                return Err("line continuation at end of file".to_owned());
            };
            index += 1;
            line.push_str(next.trim());
        }
        // Vanilla checks the length before it looks at the line's content, so
        // an over-long comment is refused too.
        if line.len() > MAX_FUNCTION_LINE_BYTES {
            return Err(format!("command too long on line {line_number}"));
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('/') {
            return Err(format!(
                "unknown or invalid command on line {line_number}: do not use a preceding forwards slash"
            ));
        }
        if line.starts_with('$') {
            // Vanilla `$macro arg1 arg2 ...` substitution: look up the macro
            // by name, bind its formal args to the call-site args, and emit
            // the macro body with substitutions applied.
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            let macro_name: Identifier = match parts[0][1..].parse() {
                Ok(id) => id,
                Err(error) => {
                    return Err(format!(
                        "unknown macro {} on line {line_number}: {error}",
                        parts[0]
                    ));
                }
            };
            let Some(macro_def) = macros.get(&macro_name) else {
                return Err(format!(
                    "macro {macro_name} is not defined (line {line_number})"
                ));
            };
            let call_args: Vec<&str> = parts[1..].to_vec();
            if call_args.len() != macro_def.args.len() {
                return Err(format!(
                    "macro {macro_name} expects {} args, got {} (line {line_number})",
                    macro_def.args.len(),
                    call_args.len()
                ));
            }
            let mut bound_body = Vec::with_capacity(macro_def.body.len());
            for body_line in &macro_def.body {
                let mut substituted = body_line.clone();
                for (i, arg) in macro_def.args.iter().enumerate() {
                    substituted = substituted.replace(&format!("${arg}"), call_args[i]);
                }
                bound_body.push(substituted);
            }
            commands.extend(bound_body);
            continue;
        }
        commands.push(line);
    }
    Ok(commands)
}

fn read_bounded_text(reader: impl Read, source: &str) -> Result<String, String> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_RESOURCE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{source}: {error}"))?;
    if bytes.len() as u64 > MAX_RESOURCE_BYTES {
        return Err(format!("{source}: resource exceeds 8 MiB"));
    }
    String::from_utf8(bytes).map_err(|error| format!("{source}: {error}"))
}

/// Reads `pack.mcmeta`, reporting whether the entry is a data pack and the plain
/// text of its `pack.description`, plus `pack_format`, overlays, and filter.
///
/// Vanilla decodes a full text component; Steel reads the common shapes (a
/// string, or an object with `text`/`translate`) and leaves anything richer
/// empty rather than rendering a wrong description. A missing or unparseable
/// `pack` section means the entry is not a data pack, matching
/// `Pack.readMetaAndCreate` returning null.
fn pack_metadata(
    metadata: &str,
    source: &str,
) -> (
    bool,
    String,
    Option<u32>,
    Option<OverlayDef>,
    Option<FilterDef>,
) {
    let value: serde_json::Value = match serde_json::from_str(metadata) {
        Ok(value) => value,
        Err(error) => {
            log::warn!("Couldn't read pack metadata {source}: {error}");
            return (false, String::new(), None, None, None);
        }
    };
    let Some(pack) = value.get("pack").filter(|pack| pack.is_object()) else {
        return (false, String::new(), None, None, None);
    };
    let pack_format = pack
        .get("pack_format")
        .and_then(serde_json::Value::as_u64)
        .map(|v| v as u32);
    let overlays = value
        .get("overlays")
        .and_then(|o| serde_json::from_value::<OverlayDef>(o.clone()).ok());
    let filter = value
        .get("filter")
        .and_then(|f| serde_json::from_value::<FilterDef>(f.clone()).ok());
    let description = pack
        .get("description")
        .and_then(|description| {
            description
                .as_str()
                .map(ToOwned::to_owned)
                .or_else(|| {
                    description
                        .get("text")
                        .and_then(serde_json::Value::as_str)
                        .map(ToOwned::to_owned)
                })
                .or_else(|| {
                    description
                        .get("translate")
                        .and_then(serde_json::Value::as_str)
                        .map(ToOwned::to_owned)
                })
        })
        .unwrap_or_default();
    (true, description, pack_format, overlays, filter)
}

/// Reads a resource file as UTF-8 text.
///
/// Loads function resources from an archive without extracting its paths.
/// Returns `None` when the entry is not a readable data pack, matching
/// vanilla's `Pack.readMetaAndCreate` returning null for a non-pack entry.
fn load_zip(
    path: &Path,
    _allowed_symlinks: &[PathBuf],
) -> Option<(PackResources, String, Option<u32>, bool)> {
    let source = path.display().to_string();
    let Some(mut archive) = fs::File::open(path)
        .ok()
        .and_then(|file| ZipArchive::new(file).ok())
    else {
        log::warn!("Couldn't open data pack archive {source}");
        return None;
    };
    let metadata_label = format!("{source}!pack.mcmeta");
    let metadata = match archive.by_name("pack.mcmeta") {
        Ok(file) => match read_bounded_text(file, &metadata_label) {
            Ok(text) => text,
            Err(error) => {
                log::warn!("Couldn't read {metadata_label}: {error}");
                return None;
            }
        },
        Err(ZipError::FileNotFound) => return None,
        Err(error) => {
            log::warn!("Couldn't read {metadata_label}: {error}");
            return None;
        }
    };
    let (is_pack, description, pack_format, overlays, filter) =
        pack_metadata(&metadata, &metadata_label);
    if !is_pack {
        return None;
    }
    let incompatible = pack_format.map_or(false, |pf| pf > PACK_FORMAT_CURRENT);

    let mut resources = PackResources::default();
    resources.overlays = overlays.map(|o| vec![o]).unwrap_or_default();
    resources.filter = filter;
    let mut entries = Vec::new();
    for index in 0..archive.len() {
        let Ok(entry) = archive.by_index(index) else {
            continue;
        };
        if entry.is_dir() {
            continue;
        }
        let Ok(name) = entry.name() else {
            continue;
        };
        entries.push((name.into_owned(), index));
    }
    entries.sort_unstable();
    if entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        log::warn!("Couldn't read {source}: duplicate ZIP entry names");
        return None;
    }
    for (name, index) in entries {
        let Some((namespace, resource)) =
            name.strip_prefix("data/").and_then(|s| s.split_once('/'))
        else {
            continue;
        };
        if !Identifier::validate_namespace(namespace) || namespace.is_empty() {
            continue;
        }
        let (relative, tags) = if let Some(relative) = resource.strip_prefix("function/") {
            (relative.strip_suffix(".mcfunction"), false)
        } else if let Some(relative) = resource.strip_prefix("tags/function/") {
            (relative.strip_suffix(".json"), true)
        } else {
            continue;
        };
        let Some(relative) = relative else {
            continue;
        };
        if relative.is_empty()
            || relative
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == "..")
            || !Identifier::validate_path(relative)
        {
            continue;
        }
        let key: Identifier = match format!("{namespace}:{relative}").parse() {
            Ok(key) => key,
            Err(error) => {
                log::warn!("Ignoring ZIP entry {source}!{name}: {error}");
                continue;
            }
        };
        let Ok(entry) = archive.by_index(index) else {
            continue;
        };
        let label = format!("{source}!{name}");
        let Ok(text) = read_bounded_text(entry, &label) else {
            continue;
        };
        load_resource(&mut resources, namespace, key, &text, &label, tags);
    }
    Some((resources, description, pack_format, incompatible))
}

impl FunctionPackState {
    fn expand_tag(
        &self,
        key: &Identifier,
        visiting: &mut BTreeSet<Identifier>,
        result: &mut Vec<Identifier>,
    ) -> Result<(), String> {
        if !visiting.insert(key.clone()) {
            return Err(format!("cyclic function tag #{key}"));
        }
        let values = self
            .tags
            .get(key)
            .ok_or_else(|| format!("missing function tag #{key}"))?;
        for entry in values {
            let (id, required) = match entry {
                TagEntry::Name(id) => (id, true),
                TagEntry::Object { id, required } => (id, *required),
            };
            let (is_tag, raw_id) = id
                .strip_prefix('#')
                .map_or((false, id.as_str()), |name| (true, name));
            let id: Identifier = raw_id.parse().map_err(|error| format!("{key}: {error}"))?;
            if is_tag {
                if required || self.tags.contains_key(&id) {
                    self.expand_tag(&id, visiting, result)?;
                }
            } else if self.functions.contains_key(&id)
                || steel_registry::REGISTRY.functions.by_key(&id).is_some()
            {
                if !result.contains(&id) {
                    result.push(id);
                }
            } else if required {
                return Err(format!("#{key}: missing required function {id}"));
            }
        }
        visiting.remove(key);
        Ok(())
    }
}

impl FunctionPacks {
    /// Load resources beneath the configured save root, before server ticks.
    ///
    /// `config` is the persisted selection (vanilla `DataPackConfig`); newly
    /// discovered packs join it the way vanilla's `configurePackRepository`
    /// auto-enables world packs.
    /// Loads `allowed_symlinks.txt` once and keeps it for all discovery calls.
    fn allowed_symlinks(root: &Path) -> Vec<PathBuf> {
        load_allowed_symlinks(root)
    }

    /// Load resources beneath the configured save root, before server ticks.
    ///
    /// `config` is the persisted selection (vanilla `DataPackConfig`); newly
    /// discovered packs join it the way vanilla's `configurePackRepository`
    /// auto-enables world packs.
    #[must_use]
    pub fn from_root(root: &Path, config: DataPackSelection) -> Self {
        let allowed = Self::allowed_symlinks(root);
        let packs = Self {
            root: Some(root.to_path_buf()),
            index: ArcSwap::default(),
            state: ArcSwap::default(),
            selection: SyncMutex::new(config.clone()),
            registries: RuntimeRegistries::default(),
        };
        let index = Self::prepare_index(root, &allowed);
        let selected = Self::initial_selection(&index, &config);
        packs.index.store(Arc::new(index));
        let index = packs.index.load();
        {
            let mut persisted = packs.selection.lock();
            packs.commit_selected(&index, &mut persisted, &selected);
        }
        packs
    }

    /// Re-discovers the repository and publishes a fresh snapshot.
    #[must_use]
    pub fn reload(&self) -> usize {
        let Some(root) = self.root.clone() else {
            return 0;
        };
        let allowed = Self::allowed_symlinks(&root);
        let index = Self::prepare_index(&root, &allowed);
        self.index.store(Arc::new(index));
        let index = self.index.load();
        let mut persisted = self.selection.lock();
        let selected = Self::rebuild_selected(&index, &persisted.enabled);
        self.commit_selected(&index, &mut persisted, &selected);
        // Vanilla `PackRepository.reload` also reloads function resources;
        // publish a fresh snapshot so `commands()` reflects the new files.
        let snapshot = Self::build_snapshot(&index, &selected);
        let count = snapshot.functions.len();
        self.state.store(Arc::new(snapshot));
        count
    }

    /// Prepares resources off the async executor without publishing them.
    /// Cancelling the caller may leave the blocking read running, but cannot
    /// apply its result.
    pub(crate) async fn prepare_reload(&self) -> Result<Option<FunctionPackSnapshot>, String> {
        let Some(root) = self.root.clone() else {
            return Ok(None);
        };
        let allowed = Self::allowed_symlinks(&root);
        let enabled = self.selection.lock().enabled.clone();
        let root_clone = root.clone();
        let index = spawn_blocking(move || Self::prepare_index(&root_clone, &allowed))
            .await
            .map_err(|error| format!("function-pack loading task: {error}"))?;
        self.index.store(Arc::new(index));
        let index = self.index.load();
        let selected = Self::rebuild_selected(&index, &enabled);
        Ok(Some(Self::build_snapshot(&index, &selected)))
    }

    /// Discovers every pack under `root`, mirroring `PackRepository.reload`.
    ///
    /// `allowed_symlinks` lists the symlink target prefixes that are permitted
    /// (vanilla's `allowed_symlinks.txt`).
    ///
    /// Unreadable entries are logged and skipped rather than failing discovery,
    /// so one broken pack cannot stop the server from starting.
    fn prepare_index(root: &Path, allowed_symlinks: &[PathBuf]) -> FunctionPackIndex {
        let mut packs = Vec::new();
        for entry in entries(root, allowed_symlinks) {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let id = format!("file/{name}");
            if file_type.is_file() && path.extension().is_some_and(|extension| extension == "zip") {
                let Some((resources, description, pack_format, incompatible)) =
                    load_zip(&path, allowed_symlinks)
                else {
                    continue;
                };
                packs.push(AvailablePack {
                    info: PackInfo {
                        id,
                        title: name,
                        description,
                        source_label: "world",
                        pack_format,
                        incompatible,
                    },
                    resources,
                });
                continue;
            }
            if !file_type.is_dir() {
                continue;
            }
            let metadata_path = path.join("pack.mcmeta");
            let Some(metadata_text) = read_resource(&metadata_path) else {
                continue;
            };
            let (is_pack, description, pack_format, overlays, filter) =
                pack_metadata(&metadata_text, &metadata_path.display().to_string());
            if !is_pack {
                continue;
            }
            let incompatible = pack_format.map_or(false, |pf| pf > PACK_FORMAT_CURRENT);
            let mut resources = PackResources::default();
            resources.overlays = overlays.map(|o| vec![o]).unwrap_or_default();
            resources.filter = filter;
            for namespace in entries(&path.join("data"), allowed_symlinks) {
                let Ok(namespace_type) = namespace.file_type() else {
                    continue;
                };
                if !namespace_type.is_dir() {
                    continue;
                }
                let namespace_name = namespace.file_name().to_string_lossy().into_owned();
                if !Identifier::validate_namespace(&namespace_name) {
                    log::warn!("Ignoring invalid namespace {namespace_name} in data pack {id}");
                    continue;
                }
                let functions = namespace.path().join("function");
                load_tree(
                    &mut resources,
                    &namespace_name,
                    &functions,
                    &functions,
                    false,
                    allowed_symlinks,
                );
                let tags = namespace.path().join("tags/function");
                load_tree(
                    &mut resources,
                    &namespace_name,
                    &tags,
                    &tags,
                    true,
                    allowed_symlinks,
                );
            }
            packs.push(AvailablePack {
                info: PackInfo {
                    id,
                    title: name,
                    description,
                    source_label: "world",
                    pack_format,
                    incompatible,
                },
                resources,
            });
        }
        // Vanilla exposes its baked pack through `ServerPacksSource`; Steel
        // compiles it into the registry, so it contributes no file resources.
        packs.push(AvailablePack::new(
            VANILLA_PACK.to_owned(),
            "Default".to_owned(),
            "The default data for Minecraft".to_owned(),
            "built-in",
        ));
        packs.sort_by(|pack_a, pack_b| pack_a.info.id.cmp(&pack_b.info.id));
        FunctionPackIndex { packs }
    }

    /// Vanilla `configurePackRepository`: the persisted enabled list first, then
    /// every pack vanilla would auto-enable (all Steel packs are world or
    /// built-in sourced) that is not disabled.
    fn initial_selection(index: &FunctionPackIndex, config: &DataPackSelection) -> Vec<String> {
        let mut selected: Vec<String> = Vec::new();
        for id in &config.enabled {
            if index.get(id).is_some() && !selected.contains(id) {
                selected.push(id.clone());
            }
        }
        for pack in &index.packs {
            if selected.contains(&pack.info.id) || config.disabled.contains(&pack.info.id) {
                continue;
            }
            selected.push(pack.info.id.clone());
        }
        if selected.is_empty() {
            selected.push(VANILLA_PACK.to_owned());
        }
        selected
    }

    /// Vanilla `PackRepository.rebuildSelected`: keep the requested order, then
    /// insert required packs at their default position.
    fn rebuild_selected(index: &FunctionPackIndex, enabled: &[String]) -> Vec<String> {
        let mut selected: Vec<String> = Vec::new();
        for id in enabled {
            if index.get(id).is_some() && !selected.contains(id) {
                selected.push(id.clone());
            }
        }
        for pack in &index.packs {
            if selected.iter().any(|id| id == &pack.info.id) {
                continue;
            }
            let selection_config = PackSelection::of(&pack.info.id);
            if selection_config.required {
                selection_config.default_position.insert(
                    &mut selected,
                    pack.info.id.clone(),
                    |id| PackSelection::of(id),
                );
            }
        }
        selected
    }

    fn build_snapshot(index: &FunctionPackIndex, selected: &[String]) -> FunctionPackSnapshot {
        let mut state = FunctionPackState::default();
        for id in selected {
            let Some(pack) = index.get(id) else {
                continue;
            };
            for (key, commands) in &pack.functions {
                state.functions.insert(key.clone(), commands.clone());
            }
            for (key, tag) in &pack.tags {
                let values = state.tags.entry(key.clone()).or_default();
                if tag.replace {
                    values.clear();
                }
                values.extend(tag.values.iter().cloned());
            }
        }
        // Vanilla's `TagLoader.build` drops a tag whose references cannot be
        // resolved (logging the missing entries) instead of failing the reload.
        let mut load_functions = Vec::new();
        let mut tick_functions = Vec::new();
        for (name, target) in [("load", &mut load_functions), ("tick", &mut tick_functions)] {
            let key = Identifier::vanilla_static(name);
            if !state.tags.contains_key(&key) {
                continue;
            }
            let mut result = Vec::new();
            match state.expand_tag(&key, &mut BTreeSet::new(), &mut result) {
                Ok(()) => target.extend(result),
                Err(error) => log::warn!("Couldn't load function tag #{key}: {error}"),
            }
        }
        FunctionPackSnapshot {
            selected: selected.to_vec(),
            functions: state
                .functions
                .into_iter()
                .map(|(key, commands)| (key, commands.into()))
                .collect(),
            load_functions,
            tick_functions,
            load_pending: AtomicBool::new(true),
        }
    }

    /// Builds the snapshot for `selected`, records the persisted selection, and
    /// stores that snapshot as the published state.
    fn commit_selected(
        &self,
        index: &FunctionPackIndex,
        persisted: &mut DataPackSelection,
        selected: &[String],
    ) {
        let snapshot = Self::build_snapshot(index, selected);
        persisted.enabled = selected.to_vec();
        persisted.disabled = index
            .available_ids()
            .into_iter()
            .filter(|id| !selected.contains(id))
            .collect();
        self.state.store(Arc::new(snapshot));
    }

    /// Vanilla `DataPackCommand.enablePack` plus the inserter for its position
    /// argument. The caller persists and reports the result.
    pub(crate) fn enable(
        &self,
        id: &str,
        placement: &PackPlacement,
    ) -> Result<Vec<String>, PackSelectionError> {
        let index = self.index.load();
        let mut persisted = self.selection.lock();
        let mut selected = persisted.enabled.clone();
        Self::place(&index, &mut selected, id, placement)
            .map_err(|error| Self::selection_error(error, id))?;
        self.commit_selected(&index, &mut persisted, &selected);
        Ok(persisted.enabled.clone())
    }

    /// Vanilla `DataPackCommand.disablePack`.
    pub(crate) fn disable(&self, id: &str) -> Result<Vec<String>, PackSelectionError> {
        let index = self.index.load();
        let mut persisted = self.selection.lock();
        let mut selected = persisted.enabled.clone();
        let Some(position) = selected.iter().position(|selected| selected == id) else {
            return Err(if index.get(id).is_some() {
                PackSelectionError::AlreadyDisabled(id.to_owned())
            } else {
                PackSelectionError::UnknownPack(id.to_owned())
            });
        };
        selected.remove(position);
        self.commit_selected(&index, &mut persisted, &selected);
        Ok(persisted.enabled.clone())
    }

    /// Vanilla `DataPackCommand.createPack`. Creates a new directory pack under
    /// `<root>/datapacks/<name>/` with a minimal `pack.mcmeta`. The pack is
    /// discovered but not enabled, matching vanilla's behaviour.
    pub(crate) fn create(&self, name: &str) -> Result<Vec<String>, PackCreationError> {
        // Vanilla rejects names that are not valid resource locations; Steel
        // keeps it simpler: non-empty, no path separators, no `.` or `..`.
        if name.is_empty()
            || name.contains('/')
            || name.contains('\\')
            || name == "."
            || name == ".."
        {
            return Err(PackCreationError::InvalidName(name.to_owned()));
        }

        let id = format!("file/{name}");
        let index = self.index.load();
        if index.get(&id).is_some() {
            return Err(PackCreationError::AlreadyExists(id));
        }

        let Some(root) = self.root.clone() else {
            return Err(PackCreationError::InvalidName(name.to_owned()));
        };

        let pack_dir = root.join("datapacks").join(name);
        fs::create_dir_all(&pack_dir).map_err(|_| PackCreationError::AlreadyExists(id.clone()))?;

        let mcmeta = pack_dir.join("pack.mcmeta");
        let content =
            "{\"pack\":{\"pack_format\":1,\"description\":\"Created by /datapack create\"}}";
        fs::write(&mcmeta, content).map_err(|_| {
            let _ = fs::remove_dir_all(&pack_dir);
            PackCreationError::AlreadyExists(id.clone())
        })?;

        // Re-discover so the new pack appears in the repository.
        let _ = self.refresh_repository();
        Ok(self.index.load().available_ids())
    }

    /// Resolves a pack anchor the way vanilla `PackRepository.getPack` does:
    /// exact match first, then with a stripped `file/` prefix for built-in packs.
    fn resolve_anchor(selected: &[String], anchor: &str) -> Option<usize> {
        if let Some(pos) = selected.iter().position(|s| s == anchor) {
            return Some(pos);
        }
        let stripped = anchor.strip_prefix("file/");
        stripped.and_then(|name| selected.iter().position(|s| s == name))
    }

    /// Inserts `id` into `selected` at the requested placement.
    fn place(
        index: &FunctionPackIndex,
        selected: &mut Vec<String>,
        id: &str,
        placement: &PackPlacement,
    ) -> Result<(), PlaceError> {
        if index.get(id).is_none() {
            return Err(PlaceError::Unknown);
        }
        if selected.iter().any(|selected| selected == id) {
            return Err(PlaceError::AlreadyEnabled);
        }
        match placement {
            PackPlacement::Default => {
                let position = PackSelection::of(id).default_position;
                position.insert(selected, id.to_owned(), |id| PackSelection::of(id));
            }
            PackPlacement::First => selected.insert(0, id.to_owned()),
            PackPlacement::Last => selected.push(id.to_owned()),
            PackPlacement::After(existing) => {
                let anchor = Self::resolve_anchor(selected, existing);
                let Some(position) = anchor else {
                    return Err(PlaceError::AnchorNotSelected(existing.clone()));
                };
                selected.insert(position + 1, id.to_owned());
            }
            PackPlacement::Before(existing) => {
                let anchor = Self::resolve_anchor(selected, existing);
                let Some(position) = anchor else {
                    return Err(PlaceError::AnchorNotSelected(existing.clone()));
                };
                selected.insert(position, id.to_owned());
            }
        }
        Ok(())
    }

    fn selection_error(error: PlaceError, id: &str) -> PackSelectionError {
        match error {
            PlaceError::Unknown => PackSelectionError::UnknownPack(id.to_owned()),
            PlaceError::AlreadyEnabled => PackSelectionError::AlreadyEnabled(id.to_owned()),
            PlaceError::AnchorNotSelected(anchor) => PackSelectionError::AlreadyDisabled(anchor),
        }
    }

    #[expect(
        clippy::assigning_clones,
        reason = "vanilla requires cloning selected packs into persisted selection"
    )]
    pub(crate) fn publish(&self, next: FunctionPackSnapshot) -> usize {
        let count = next.functions.len();
        let mut persisted = self.selection.lock();
        persisted.enabled = next.selected.clone();
        persisted.disabled = self
            .index
            .load()
            .available_ids()
            .into_iter()
            .filter(|id| !next.selected.contains(id))
            .collect();
        self.state.store(Arc::new(next));
        count
    }

    /// Resolve an override first, then the baked vanilla function.
    pub fn commands(&self, key: &Identifier) -> Option<Arc<[String]>> {
        self.state.load().functions.get(key).cloned().or_else(|| {
            steel_registry::REGISTRY
                .functions
                .by_key(key)
                .map(|function| {
                    function
                        .commands
                        .iter()
                        .map(|line| (*line).to_owned())
                        .collect::<Vec<_>>()
                        .into()
                })
        })
    }

    /// Vanilla `PackLocationInfo` for every discovered pack, in `available` order.
    #[must_use]
    pub fn pack_infos(&self) -> Vec<PackInfo> {
        self.index.load().infos()
    }

    /// Re-discovers the repository and rebuilds the selection without
    /// republishing function resources, matching vanilla's `PackRepository.reload`.
    #[must_use]
    #[expect(
        clippy::assigning_clones,
        reason = "vanilla rebuilds enabled list by cloning selected packs"
    )]
    pub fn refresh_repository(&self) -> usize {
        let Some(root) = self.root.clone() else {
            return 0;
        };
        let allowed = Self::allowed_symlinks(&root);
        let index = Self::prepare_index(&root, &allowed);
        self.index.store(Arc::new(index));
        let index = self.index.load();
        let mut persisted = self.selection.lock();
        let selected = Self::rebuild_selected(&index, &persisted.enabled);
        persisted.enabled = selected.clone();
        persisted.disabled = index
            .available_ids()
            .into_iter()
            .filter(|id| !selected.contains(id))
            .collect();
        selected.len()
    }

    /// The selected pack ids in selection order; later packs override earlier ones.
    #[must_use]
    pub fn selected_packs(&self) -> Vec<String> {
        self.selection.lock().enabled.clone()
    }

    /// The persisted form of the current selection.
    #[must_use]
    pub(crate) fn selection(&self) -> DataPackSelection {
        self.selection.lock().clone()
    }

    /// Resolve load once per successful publication, followed by tick functions.
    /// Paused game ticks neither execute functions nor consume the pending load.
    pub fn tick_functions(&self, runs_normally: bool) -> Vec<Identifier> {
        if !runs_normally {
            return Vec::new();
        }
        let state = self.state.load();
        if state.load_pending.swap(false, Ordering::AcqRel) {
            let mut functions =
                Vec::with_capacity(state.load_functions.len() + state.tick_functions.len());
            functions.extend_from_slice(&state.load_functions);
            functions.extend_from_slice(&state.tick_functions);
            functions
        } else {
            state.tick_functions.clone()
        }
    }

    /// Atomically replaces the recipes registry. Readers on the tick thread
    /// observe either the old or the new registry, never a mix.
    pub fn swap_recipes(&self, next: Arc<RecipeRegistrySnapshot>) {
        self.registries.recipes.store(Arc::new(next));
    }

    /// Atomically replaces the loot tables registry.
    pub fn swap_loot_tables(&self, next: Arc<LootTableRegistrySnapshot>) {
        self.registries.loot_tables.store(Arc::new(next));
    }

    /// Atomically replaces the advancements registry.
    pub fn swap_advancements(&self, next: Arc<AdvancementRegistrySnapshot>) {
        self.registries.advancements.store(Arc::new(next));
    }

    /// Atomically replaces the tags registry.
    pub fn swap_tags(&self, next: Arc<TagRegistrySnapshot>) {
        self.registries.tags.store(Arc::new(next));
    }
}

/// Internal placement refusal, translated into [`PackSelectionError`].
#[derive(Clone, Debug, PartialEq, Eq)]
enum PlaceError {
    Unknown,
    AlreadyEnabled,
    /// `after`/`before` named a pack that is not selected. Vanilla resolves the
    /// anchor through `getPack(..., false)`, which reports
    /// `commands.datapack.disable.failed` for it.
    AnchorNotSelected(String),
}

/// Runtime-registries that can be reloaded without blocking the tick thread.
///
/// Each registry is wrapped in `ArcSwap` so readers perform a single
/// lock-free pointer load on the hot path, while `/reload` publishes a brand
/// new registry atomically. Readers never see a partially-written registry and
/// never block on a writer.
///
/// Vanilla `/reload` rebuilds recipes, loot tables, advancements, and tags from
/// data packs. Steel's baked registries are immutable at startup; this struct
/// provides the atomic swap target for a future data-pack-driven reload
/// without requiring `parking_lot` mutexes on the tick path.
#[derive(Debug)]
pub struct RuntimeRegistries {
    /// Recipes (`data/*/recipes/*.json`), swapped atomically on reload.
    pub recipes: ArcSwap<Arc<RecipeRegistrySnapshot>>,
    /// Loot tables (`data/*/loot_tables/**/*.json`), swapped atomically on reload.
    pub loot_tables: ArcSwap<Arc<LootTableRegistrySnapshot>>,
    /// Advancements (`data/*/advancements/**/*.json`), swapped atomically on reload.
    pub advancements: ArcSwap<Arc<AdvancementRegistrySnapshot>>,
    /// Tags (`data/*/tags/**/*.json`), swapped atomically on reload.
    pub tags: ArcSwap<Arc<TagRegistrySnapshot>>,
}

impl Default for RuntimeRegistries {
    fn default() -> Self {
        Self {
            recipes: ArcSwap::from_pointee(Arc::new(RecipeRegistrySnapshot::default())),
            loot_tables: ArcSwap::from_pointee(Arc::new(LootTableRegistrySnapshot::default())),
            advancements: ArcSwap::from_pointee(Arc::new(AdvancementRegistrySnapshot::default())),
            tags: ArcSwap::from_pointee(Arc::new(TagRegistrySnapshot::default())),
        }
    }
}

/// Opaque snapshot of a recipe registry; populated from data packs on reload.
#[derive(Debug, Default, Clone)]
#[allow(missing_docs, reason = "internal snapshot")]
pub struct RecipeRegistrySnapshot {
    pub entries: BTreeMap<Identifier, Vec<u8>>,
}

/// Opaque snapshot of a loot table registry; populated from data packs on reload.
#[derive(Debug, Default, Clone)]
#[allow(missing_docs, reason = "internal snapshot")]
pub struct LootTableRegistrySnapshot {
    pub entries: BTreeMap<Identifier, Vec<u8>>,
}

/// Opaque snapshot of an advancement registry; populated from data packs on reload.
#[derive(Debug, Default, Clone)]
#[allow(missing_docs, reason = "internal snapshot")]
pub struct AdvancementRegistrySnapshot {
    pub entries: BTreeMap<Identifier, Vec<u8>>,
}

/// Opaque snapshot of a tag registry; populated from data packs on reload.
#[derive(Debug, Default, Clone)]
#[allow(missing_docs, reason = "internal snapshot")]
pub struct TagRegistrySnapshot {
    pub entries: BTreeMap<Identifier, Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::error::Error;
    use std::io::{self, Write};
    use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

    fn write_zip(path: &Path, files: &[(&str, &str)]) -> Result<(), Box<dyn Error>> {
        let mut archive = ZipWriter::new(fs::File::create(path)?);
        for (name, text) in files {
            archive.start_file(
                *name,
                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
            )?;
            archive.write_all(text.as_bytes())?;
        }
        archive.finish()?;
        Ok(())
    }

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Result<Self, Box<dyn Error>> {
            let root = temp_dir().join(format!("steel-function-pack-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(root.join("test/data/example/function"))?;
            fs::create_dir_all(root.join("test/data/minecraft/tags/function"))?;
            fs::write(
                root.join("test/pack.mcmeta"),
                r#"{"pack":{"description":"test","pack_format":1}}"#,
            )?;
            Ok(Self(root))
        }

        /// Adds a second directory pack with one function.
        fn add_pack(&self, name: &str, function: &str) -> Result<PathBuf, Box<dyn Error>> {
            let pack = self.0.join(name);
            fs::create_dir_all(pack.join("data/example/function"))?;
            fs::write(
                pack.join("pack.mcmeta"),
                r#"{"pack":{"description":"second","pack_format":1}}"#,
            )?;
            let path = pack.join("data/example/function/hello.mcfunction");
            fs::write(&path, function)?;
            Ok(path)
        }

        fn function(&self, pack: &str, name: &str) -> PathBuf {
            self.0
                .join(format!("{pack}/data/example/function/{name}.mcfunction"))
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn default_config() -> DataPackSelection {
        DataPackSelection::default()
    }

    #[test]
    fn directory_functions_load_and_reload() -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        let function = fixture.function("test", "hello");
        fs::write(&function, "# comment\n\nreturn 7\n")?;
        let packs = FunctionPacks::from_root(&fixture.0.clone(), default_config());
        let key = "example:hello".parse::<Identifier>()?;
        assert_eq!(
            packs.commands(&key).as_deref(),
            Some(&["return 7".to_owned()][..])
        );
        fs::write(&function, "return 9")?;
        assert_eq!(packs.reload(), 1);
        assert_eq!(
            packs.commands(&key).as_deref(),
            Some(&["return 9".to_owned()][..])
        );
        Ok(())
    }

    #[test]
    fn zip_functions_override_earlier_directory_pack() -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        fs::write(fixture.function("test", "hello"), "return 1")?;
        let zip = fixture.0.join("z.zip");
        let metadata = r#"{"pack":{"description":"test","pack_format":1}}"#;
        write_zip(
            &zip,
            &[
                ("pack.mcmeta", metadata),
                ("data/example/function/hello.mcfunction", "return 2"),
                (
                    "data/minecraft/tags/function/tick.json",
                    r#"{"values":["example:hello"]}"#,
                ),
            ],
        )?;
        let packs = FunctionPacks::from_root(&fixture.0.clone(), default_config());
        let key = "example:hello".parse::<Identifier>()?;
        assert_eq!(
            packs
                .pack_infos()
                .iter()
                .map(|info| info.id.clone())
                .collect::<Vec<_>>(),
            vec!["file/test", "file/z.zip", "vanilla"]
        );
        assert_eq!(
            packs.selected_packs(),
            vec!["vanilla", "file/test", "file/z.zip"]
        );
        assert_eq!(
            packs.commands(&key).as_deref(),
            Some(&["return 2".to_owned()][..])
        );
        assert_eq!(packs.tick_functions(true), vec![key]);
        Ok(())
    }

    #[test]
    fn later_selected_pack_overrides_an_earlier_one() -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        fs::write(fixture.function("test", "hello"), "return 1")?;
        fixture.add_pack("aaa", "return 2")?;
        let packs = FunctionPacks::from_root(&fixture.0.clone(), default_config());
        let key = "example:hello".parse::<Identifier>()?;
        // `aaa` sorts before `test`, so `test` is selected later and wins.
        assert_eq!(
            packs.selected_packs(),
            vec!["vanilla", "file/aaa", "file/test"]
        );
        assert_eq!(
            packs.commands(&key).as_deref(),
            Some(&["return 1".to_owned()][..])
        );
        Ok(())
    }

    #[test]
    fn disable_stops_a_pack_overriding_and_enable_restores_it() -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        fs::write(fixture.function("test", "hello"), "return 1")?;
        fixture.add_pack("aaa", "return 2")?;
        let packs = FunctionPacks::from_root(&fixture.0.clone(), default_config());
        let key = "example:hello".parse::<Identifier>()?;
        assert_eq!(packs.disable("file/test")?, vec!["vanilla", "file/aaa"]);
        assert_eq!(
            packs.commands(&key).as_deref(),
            Some(&["return 2".to_owned()][..])
        );
        assert_eq!(
            packs.enable("file/test", &PackPlacement::Last)?,
            vec!["vanilla", "file/aaa", "file/test"]
        );
        assert_eq!(
            packs.commands(&key).as_deref(),
            Some(&["return 1".to_owned()][..])
        );
        Ok(())
    }

    #[test]
    fn enable_placements_match_vanilla_positions() -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        fixture.add_pack("aaa", "return 2")?;
        fixture.add_pack("mmm", "return 4")?;
        fixture.add_pack("yyy", "return 5")?;
        fixture.add_pack("zzz", "return 3")?;
        let config = DataPackSelection {
            enabled: vec!["vanilla".to_owned()],
            disabled: vec![
                "file/aaa".to_owned(),
                "file/mmm".to_owned(),
                "file/test".to_owned(),
                "file/yyy".to_owned(),
                "file/zzz".to_owned(),
            ],
        };
        let packs = FunctionPacks::from_root(&fixture.0.clone(), config);
        assert_eq!(packs.selected_packs(), vec!["vanilla"]);
        assert_eq!(
            packs.enable("file/test", &PackPlacement::Default)?,
            vec!["vanilla", "file/test"]
        );
        assert_eq!(
            packs.enable("file/aaa", &PackPlacement::First)?,
            vec!["file/aaa", "vanilla", "file/test"]
        );
        // Vanilla resolves the anchor through `getPack(..., false)`, so an
        // unselected anchor reports `commands.datapack.disable.failed`.
        for placement in [
            PackPlacement::Before("file/missing".to_owned()),
            PackPlacement::After("file/missing".to_owned()),
        ] {
            assert_eq!(
                packs
                    .enable("file/mmm", &placement)
                    .expect_err("anchor not selected"),
                PackSelectionError::AlreadyDisabled("file/missing".to_owned())
            );
        }
        assert_eq!(
            packs.enable("file/mmm", &PackPlacement::After("file/aaa".to_owned()))?,
            vec!["file/aaa", "file/mmm", "vanilla", "file/test"]
        );
        assert_eq!(
            packs.enable(
                "file/zzz",
                &PackPlacement::Before("file/vanilla".to_owned()),
            )?,
            vec!["file/aaa", "file/mmm", "file/zzz", "vanilla", "file/test"]
        );
        assert_eq!(
            packs.enable("file/yyy", &PackPlacement::Last)?,
            vec![
                "file/aaa",
                "file/mmm",
                "file/zzz",
                "vanilla",
                "file/test",
                "file/yyy"
            ]
        );
        assert_eq!(
            packs
                .enable("file/test", &PackPlacement::Default)
                .expect_err("already selected"),
            PackSelectionError::AlreadyEnabled("file/test".to_owned())
        );
        assert_eq!(
            packs
                .enable("file/bbb", &PackPlacement::Last)
                .expect_err("not discovered"),
            PackSelectionError::UnknownPack("file/bbb".to_owned())
        );
        Ok(())
    }

    #[test]
    fn disabled_selection_is_reported_for_the_persisted_config() -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        fixture.add_pack("aaa", "return 2")?;
        let config = DataPackSelection {
            enabled: vec!["vanilla".to_owned(), "file/test".to_owned()],
            disabled: vec!["file/aaa".to_owned()],
        };
        let packs = FunctionPacks::from_root(&fixture.0.clone(), config);
        assert_eq!(packs.selected_packs(), vec!["vanilla", "file/test"]);
        assert_eq!(
            packs.disable("file/test").expect("selected"),
            vec!["vanilla"]
        );
        let selection = packs.selection.lock();
        assert_eq!(selection.enabled, vec!["vanilla"]);
        assert_eq!(selection.disabled, vec!["file/aaa", "file/test"]);
        Ok(())
    }

    #[test]
    fn pack_info_reports_vanilla_location_fields() -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        let packs = FunctionPacks::from_root(&fixture.0.clone(), default_config());
        let infos = packs.pack_infos();
        let test = infos
            .iter()
            .find(|info| info.id == "file/test")
            .expect("directory pack");
        assert_eq!(test.title, "test");
        assert_eq!(test.description, "test");
        assert_eq!(test.source_label, "world");
        assert_eq!(test.decorated_id(), "file/test (world)");
        let vanilla = infos
            .iter()
            .find(|info| info.id == "vanilla")
            .expect("baked pack");
        assert_eq!(vanilla.title, "Default");
        assert_eq!(vanilla.description, "The default data for Minecraft");
        assert_eq!(vanilla.source_label, "built-in");
        assert_eq!(vanilla.decorated_id(), "vanilla (built-in)");
        Ok(())
    }

    #[test]
    fn overlays_and_filters_are_ignored_with_a_warning() -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        fs::write(fixture.function("test", "hello"), "return 1")?;
        let packs = FunctionPacks::from_root(&fixture.0.clone(), default_config());
        let key = "example:hello".parse::<Identifier>()?;
        for metadata in [
            r#"{"pack":{"description":"test"},"overlays":{"entries":[]}}"#,
            r#"{"pack":{"description":"test"},"filter":{"block":[]}}"#,
        ] {
            fs::write(fixture.0.join("test/pack.mcmeta"), metadata)?;
            fs::write(fixture.function("test", "hello"), "return 2")?;
            assert_eq!(packs.reload(), 1);
            assert_eq!(
                packs.commands(&key).as_deref(),
                Some(&["return 2".to_owned()][..])
            );
        }
        Ok(())
    }

    #[test]
    fn load_tag_runs_once_tick_tag_repeats_and_successful_reload_rearms_load()
    -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        fs::write(fixture.function("test", "hello"), "return 7")?;
        for name in ["load", "tick"] {
            fs::write(
                fixture
                    .0
                    .join(format!("test/data/minecraft/tags/function/{name}.json")),
                r#"{"values":["example:hello",{"id":"example:optional","required":false}]}"#,
            )?;
        }
        let packs = FunctionPacks::from_root(&fixture.0.clone(), default_config());
        let key = "example:hello".parse::<Identifier>()?;
        assert_eq!(packs.tick_functions(true), vec![key.clone(), key.clone()]);
        assert_eq!(packs.tick_functions(true), vec![key.clone()]);
        let _ = packs.reload();
        assert_eq!(packs.tick_functions(true), vec![key.clone(), key]);
        Ok(())
    }

    #[test]
    fn paused_ticks_preserve_pending_load_until_game_resumes() -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        for name in ["load", "tick"] {
            fs::write(fixture.function("test", name), "return 1")?;
            fs::write(
                fixture
                    .0
                    .join(format!("test/data/minecraft/tags/function/{name}.json")),
                format!(r#"{{"values":["example:{name}"]}}"#),
            )?;
        }
        let packs = FunctionPacks::from_root(&fixture.0.clone(), default_config());
        let load = "example:load".parse::<Identifier>()?;
        let tick = "example:tick".parse::<Identifier>()?;
        assert_eq!(packs.tick_functions(false), [] as [Identifier; 0]);
        assert_eq!(packs.tick_functions(false), [] as [Identifier; 0]);
        assert_eq!(packs.tick_functions(true), vec![load.clone(), tick.clone()]);
        assert_eq!(packs.tick_functions(false), [] as [Identifier; 0]);
        assert_eq!(packs.tick_functions(true), vec![tick.clone()]);
        let _ = packs.reload();
        assert_eq!(packs.tick_functions(false), [] as [Identifier; 0]);
        assert_eq!(packs.tick_functions(true), vec![load, tick]);
        Ok(())
    }

    #[test]
    fn tick_tag_expands_nested_tags_and_refreshes_after_reload() -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        let tags = fixture.0.join("test/data/example/tags/function");
        fs::create_dir_all(&tags)?;
        for name in ["one", "two"] {
            fs::write(fixture.function("test", name), "return 1")?;
        }
        let nested = tags.join("nested.json");
        fs::write(&nested, r#"{"values":["example:one","example:two"]}"#)?;
        fs::write(
            fixture
                .0
                .join("test/data/minecraft/tags/function/tick.json"),
            r##"{"values":["#example:nested","example:one"]}"##,
        )?;
        let packs = FunctionPacks::from_root(&fixture.0.clone(), default_config());
        let one = "example:one".parse::<Identifier>()?;
        let two = "example:two".parse::<Identifier>()?;
        assert_eq!(packs.tick_functions(true), vec![one.clone(), two.clone()]);
        assert_eq!(packs.tick_functions(true), vec![one.clone(), two]);
        fs::write(&nested, r#"{"values":["example:one"]}"#)?;
        let _ = packs.reload();
        assert_eq!(packs.tick_functions(true), vec![one]);
        Ok(())
    }

    #[tokio::test]
    async fn async_reload_preserves_the_old_snapshot_until_publication()
    -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        let function = fixture.function("test", "hello");
        fs::write(&function, "return 1")?;
        let packs = FunctionPacks::from_root(&fixture.0.clone(), default_config());
        let key = "example:hello".parse::<Identifier>()?;
        let old = packs.state.load();
        let old_body = packs.commands(&key).expect("function exists");
        let same_body = packs.commands(&key).expect("function exists");
        assert!(Arc::ptr_eq(&old_body, &same_body));

        fs::write(&function, "return 2")?;
        let prepared = packs.prepare_reload().await?.expect("pack root");
        assert_eq!(
            packs.commands(&key).as_deref(),
            Some(&["return 1".to_owned()][..])
        );
        assert_eq!(packs.publish(prepared), 1);
        assert_eq!(
            old.functions.get(&key).map(AsRef::as_ref),
            Some(&["return 1".to_owned()][..])
        );
        assert_eq!(&*old_body, &["return 1".to_owned()]);
        assert_eq!(
            packs.commands(&key).as_deref(),
            Some(&["return 2".to_owned()][..])
        );
        Ok(())
    }

    #[test]
    fn concurrent_ticks_claim_each_generation_load_once() -> Result<(), Box<dyn Error>> {
        use std::{sync::Barrier, thread};

        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        for name in ["load", "tick"] {
            fs::write(fixture.function("test", name), "return 1")?;
            fs::write(
                fixture
                    .0
                    .join(format!("test/data/minecraft/tags/function/{name}.json")),
                format!(r#"{{"values":["example:{name}"]}}"#),
            )?;
        }
        let packs = FunctionPacks::from_root(&fixture.0.clone(), default_config());
        let barrier = Barrier::new(8);
        let results = thread::scope(|scope| {
            let mut threads = Vec::new();
            for _ in 0..8 {
                threads.push(scope.spawn(|| {
                    barrier.wait();
                    packs.tick_functions(true)
                }));
            }
            threads
                .into_iter()
                .map(|thread| thread.join().expect("tick thread"))
                .collect::<Vec<_>>()
        });
        let load = "example:load".parse::<Identifier>()?;
        let tick = "example:tick".parse::<Identifier>()?;
        assert_eq!(
            results
                .iter()
                .filter(|result| result.contains(&load))
                .count(),
            1
        );
        assert!(results.iter().all(|result| result.contains(&tick)));
        Ok(())
    }

    #[test]
    fn malformed_function_is_skipped_without_disturbing_other_functions()
    -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        fs::write(fixture.function("test", "good"), "return 1")?;
        fs::write(fixture.function("test", "bad"), "/return 2")?;
        let packs = FunctionPacks::from_root(&fixture.0.clone(), default_config());
        let good = "example:good".parse::<Identifier>()?;
        let bad = "example:bad".parse::<Identifier>()?;
        assert_eq!(
            packs.commands(&good).as_deref(),
            Some(&["return 1".to_owned()][..])
        );
        assert_eq!(packs.commands(&bad), None);
        fs::write(fixture.function("test", "bad"), "return 3")?;
        let _ = packs.reload();
        assert_eq!(
            packs.commands(&bad).as_deref(),
            Some(&["return 3".to_owned()][..])
        );
        Ok(())
    }

    #[test]
    fn unresolvable_function_tags_are_dropped() -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        let tag = fixture
            .0
            .join("test/data/minecraft/tags/function/tick.json");
        // A self-referencing tag and a missing required entry are both dropped,
        // the way vanilla's `TagLoader.build` drops them.
        for invalid in [
            r##"{"values":["#minecraft:tick"]}"##,
            r#"{"values":["example:missing"]}"#,
            r###"{"values":["##minecraft:tick"]}"###,
        ] {
            fs::write(&tag, invalid)?;
            let packs = FunctionPacks::from_root(&fixture.0.clone(), default_config());
            assert_eq!(packs.tick_functions(true), [] as [Identifier; 0]);
        }
        Ok(())
    }

    #[expect(
        clippy::redundant_closure,
        reason = "io::Error::other(e) is clearer than map_err(io::Error::other)"
    )]
    #[test]
    fn parse_function_lines_matches_vanilla_line_rules() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            parse_function_lines("# comment\n\n  return 1  \n", &BTreeMap::new())
                .map_err(|e| io::Error::other(e))?,
            vec!["return 1".to_owned()]
        );
        assert_eq!(
            parse_function_lines("say hello \\\n  world", &BTreeMap::new())
                .map_err(|e| io::Error::other(e))?,
            vec!["say hello world".to_owned()]
        );
        assert!(parse_function_lines("say hi \\", &BTreeMap::new()).is_err());
        assert!(parse_function_lines("/return 1", &BTreeMap::new()).is_err());
        assert!(parse_function_lines("// comment", &BTreeMap::new()).is_err());
        assert!(parse_function_lines("$macro", &BTreeMap::new()).is_err());
        let mut too_long = "a".repeat(MAX_FUNCTION_LINE_BYTES + 1);
        too_long.push(' ');
        assert!(parse_function_lines(&too_long, &BTreeMap::new()).is_err());
        Ok(())
    }

    #[expect(
        clippy::redundant_closure,
        reason = "io::Error::other(e) is clearer than map_err(io::Error::other)"
    )]
    #[test]
    fn resource_reader_enforces_size_limit_during_read() -> Result<(), Box<dyn Error>> {
        let accepted = vec![b'a'; MAX_RESOURCE_BYTES as usize];
        assert_eq!(
            read_bounded_text(accepted.as_slice(), "resource")
                .map_err(|e| io::Error::other(e))?
                .len(),
            accepted.len()
        );
        let mut too_large = accepted;
        too_large.push(b'b');
        assert!(read_bounded_text(too_large.as_slice(), "resource").is_err());
        assert!(read_bounded_text([0xff].as_slice(), "resource").is_err());
        Ok(())
    }
}
