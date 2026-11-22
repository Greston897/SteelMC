//! Runtime directory and ZIP data packs: function bodies and function tags.
//!
//! Other data registries are still baked. This module does not claim to reload
//! recipes, worldgen, loot, or advancements merely because their pack is visible.
use arc_swap::ArcSwap;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use steel_utils::Identifier;
use tokio::task::spawn_blocking;
use zip::{ZipArchive, result::ZipError};

const MAX_RESOURCE_BYTES: u64 = 8 * 1024 * 1024;

/// Atomically replaceable runtime function resources.
#[derive(Debug, Default)]
pub struct FunctionPacks {
    root: Option<PathBuf>,
    state: ArcSwap<FunctionPackSnapshot>,
}

#[derive(Debug, Default)]
struct FunctionPackState {
    packs: Vec<String>,
    functions: BTreeMap<Identifier, Vec<String>>,
    tags: BTreeMap<Identifier, Vec<TagEntry>>,
}

/// Immutable function resources; readers retain their generation while a reload
/// publishes the next one. Tag expansion happens before publication, not on ticks.
#[derive(Debug, Default)]
pub(crate) struct FunctionPackSnapshot {
    packs: Vec<String>,
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

#[derive(Deserialize)]
struct FunctionTag {
    #[serde(default)]
    replace: bool,
    values: Vec<TagEntry>,
}

fn read_resource(path: &Path) -> Result<String, String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if !metadata.is_file() || metadata.len() > MAX_RESOURCE_BYTES {
        return Err(format!(
            "{}: not a regular resource or exceeds 8 MiB",
            path.display()
        ));
    }
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn entries(path: &Path) -> Result<Vec<fs::DirEntry>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    if fs::symlink_metadata(path)
        .map_err(|error| error.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err(format!(
            "{}: symbolic resource directories are not supported",
            path.display()
        ));
    }
    let mut result = fs::read_dir(path)
        .map_err(|error| format!("{}: {error}", path.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    result.sort_by_key(fs::DirEntry::file_name);
    Ok(result)
}

fn load_tree(
    state: &mut FunctionPackState,
    namespace: &str,
    base: &Path,
    path: &Path,
    tags: bool,
) -> Result<(), String> {
    for entry in entries(path)? {
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_symlink() {
            return Err(format!(
                "{}: symbolic resource links are not supported",
                entry.path().display()
            ));
        }
        if file_type.is_dir() {
            load_tree(state, namespace, base, &entry.path(), tags)?;
            continue;
        }
        let path = entry.path();
        let extension = if tags { "json" } else { "mcfunction" };
        if path.extension().and_then(|value| value.to_str()) != Some(extension) {
            continue;
        }
        let relative = path
            .strip_prefix(base)
            .map_err(|error| error.to_string())?
            .with_extension("");
        let key: Identifier = format!(
            "{namespace}:{}",
            relative.to_string_lossy().replace('\\', "/")
        )
        .parse()
        .map_err(|error| format!("{}: {error}", path.display()))?;
        let text = read_resource(&path)?;
        load_resource(state, key, &text, &path.display().to_string(), tags)?;
    }
    Ok(())
}

fn load_resource(
    state: &mut FunctionPackState,
    key: Identifier,
    text: &str,
    source: &str,
    tags: bool,
) -> Result<(), String> {
    if tags {
        let tag: FunctionTag =
            serde_json::from_str(text).map_err(|error| format!("{source}: {error}"))?;
        let values = state.tags.entry(key).or_default();
        if tag.replace {
            values.clear();
        }
        values.extend(tag.values);
    } else {
        let mut commands = Vec::new();
        for (line, command) in text.lines().enumerate() {
            let command = command.trim();
            if command.is_empty() || command.starts_with('#') {
                continue;
            }
            if command.starts_with('/') || command.starts_with('$') {
                return Err(format!(
                    "{source}:{}: slash-prefixed commands and function macros are not supported",
                    line + 1
                ));
            }
            commands.push(command.to_owned());
        }
        state.functions.insert(key, commands);
    }
    Ok(())
}

fn read_zip_text(reader: impl Read, source: &str) -> Result<String, String> {
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

fn validate_metadata(text: &str, source: &str) -> Result<(), String> {
    let metadata: serde_json::Value =
        serde_json::from_str(text).map_err(|error| format!("{source}: {error}"))?;
    if !metadata
        .get("pack")
        .is_some_and(serde_json::Value::is_object)
    {
        return Err(format!("{source}: missing pack metadata"));
    }
    // Overlay and filter sections change resource resolution. Until those are
    // implemented, fail rather than execute the wrong function set.
    if metadata.get("overlays").is_some() || metadata.get("filter").is_some() {
        return Err(format!(
            "{source}: pack overlays and filters are not supported"
        ));
    }
    Ok(())
}

/// Loads function resources from an archive without extracting its paths.
fn load_zip(state: &mut FunctionPackState, path: &Path) -> Result<bool, String> {
    let source = path.display();
    let file = fs::File::open(path).map_err(|error| format!("{source}: {error}"))?;
    let mut archive = ZipArchive::new(file).map_err(|error| format!("{source}: {error}"))?;
    let metadata = match archive.by_name("pack.mcmeta") {
        Ok(file) => read_zip_text(file, &format!("{source}!pack.mcmeta"))?,
        Err(ZipError::FileNotFound) => return Ok(false),
        Err(error) => return Err(format!("{source}: {error}")),
    };
    validate_metadata(&metadata, &format!("{source}!pack.mcmeta"))?;

    let mut resources = Vec::new();
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|error| format!("{source}: {error}"))?;
        if !entry.is_dir() {
            let name = entry.name().map_err(|error| format!("{source}: {error}"))?;
            resources.push((name.into_owned(), index));
        }
    }
    resources.sort_unstable();
    if resources.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(format!(
            "{source}: duplicate ZIP entry names are not supported"
        ));
    }
    for (name, index) in resources {
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
        let key: Identifier = format!("{namespace}:{relative}")
            .parse()
            .map_err(|error| format!("{source}!{name}: {error}"))?;
        let entry = archive
            .by_index(index)
            .map_err(|error| format!("{source}!{name}: {error}"))?;
        let label = format!("{source}!{name}");
        let text = read_zip_text(entry, &label)?;
        load_resource(state, key, &text, &label, tags)?;
    }
    Ok(true)
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
            let is_tag = id.starts_with('#');
            let id: Identifier = id
                .trim_start_matches('#')
                .parse()
                .map_err(|error| format!("{key}: {error}"))?;
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
    pub fn from_root(root: PathBuf) -> Result<Self, String> {
        let packs = Self {
            root: Some(root),
            state: ArcSwap::default(),
        };
        packs.reload()?;
        Ok(packs)
    }

    /// Build and validate a fresh snapshot, publishing only on success.
    pub fn reload(&self) -> Result<usize, String> {
        let Some(root) = &self.root else {
            return Ok(0);
        };
        let next = Self::prepare(root)?;
        Ok(self.publish(next))
    }

    /// Prepares resources off the async executor without publishing them.
    /// Cancelling the caller may leave the blocking read running, but cannot
    /// apply its result.
    pub(crate) async fn prepare_reload(&self) -> Result<Option<FunctionPackSnapshot>, String> {
        let Some(root) = &self.root else {
            return Ok(None);
        };
        let root = root.clone();
        let next = spawn_blocking(move || Self::prepare(&root))
            .await
            .map_err(|error| format!("function-pack loading task: {error}"))??;
        Ok(Some(next))
    }

    fn prepare(root: &Path) -> Result<FunctionPackSnapshot, String> {
        let mut next = FunctionPackState::default();
        for entry in entries(root)? {
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
            let pack = entry.path();
            if file_type.is_file() && pack.extension().is_some_and(|extension| extension == "zip") {
                if load_zip(&mut next, &pack)? {
                    next.packs
                        .push(format!("file/{}", entry.file_name().to_string_lossy()));
                }
                continue;
            }
            if !file_type.is_dir() {
                continue;
            }
            if !pack.join("pack.mcmeta").is_file() {
                continue;
            }
            let metadata_path = pack.join("pack.mcmeta");
            validate_metadata(&read_resource(&metadata_path)?, &metadata_path.display().to_string())?;
            for namespace in entries(&pack.join("data"))? {
                if !namespace
                    .file_type()
                    .map_err(|error| error.to_string())?
                    .is_dir()
                {
                    continue;
                }
                let name = namespace.file_name().to_string_lossy().into_owned();
                if !Identifier::validate_namespace(&name) {
                    return Err(format!("invalid namespace {name}"));
                }
                let functions = namespace.path().join("function");
                load_tree(&mut next, &name, &functions, &functions, false)?;
                let tags = namespace.path().join("tags/function");
                load_tree(&mut next, &name, &tags, &tags, true)?;
            }
            next.packs
                .push(format!("file/{}", entry.file_name().to_string_lossy()));
        }
        for key in next.tags.keys() {
            next.expand_tag(key, &mut BTreeSet::new(), &mut Vec::new())?;
        }
        let expand = |name| -> Result<Vec<Identifier>, String> {
            let key = Identifier::vanilla_static(name);
            let mut result = Vec::new();
            if next.tags.contains_key(&key) {
                next.expand_tag(&key, &mut BTreeSet::new(), &mut result)?;
            }
            Ok(result)
        };
        let load_functions = expand("load")?;
        let tick_functions = expand("tick")?;
        Ok(FunctionPackSnapshot {
            packs: next.packs,
            functions: next
                .functions
                .into_iter()
                .map(|(key, commands)| (key, commands.into()))
                .collect(),
            load_functions,
            tick_functions,
            load_pending: AtomicBool::new(true),
        })
    }

    pub(crate) fn publish(&self, next: FunctionPackSnapshot) -> usize {
        let count = next.functions.len();
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

    /// Names of directory and ZIP packs whose function resources were loaded.
    pub fn pack_names(&self) -> Vec<String> {
        self.state.load().packs.clone()
    }

    /// Resolve load once per successful publication, followed by tick functions.
    pub fn tick_functions(&self) -> Vec<Identifier> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::error::Error;
    use std::io::Write;
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
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn directory_functions_load_and_reload_without_leaking_failed_snapshots()
    -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        let function = fixture
            .0
            .join("test/data/example/function/hello.mcfunction");
        fs::write(&function, "# comment\n\nreturn 7\n")?;
        let packs = FunctionPacks::from_root(fixture.0.clone())?;
        let key = "example:hello".parse::<Identifier>()?;
        assert_eq!(
            packs.commands(&key).as_deref(),
            Some(&["return 7".to_owned()][..])
        );
        fs::write(&function, "/return 9")?;
        assert!(packs.reload().is_err());
        assert_eq!(
            packs.commands(&key).as_deref(),
            Some(&["return 7".to_owned()][..])
        );
        fs::write(&function, "return 9")?;
        assert_eq!(packs.reload()?, 1);
        assert_eq!(
            packs.commands(&key).as_deref(),
            Some(&["return 9".to_owned()][..])
        );
        Ok(())
    }

    #[test]
    fn zip_functions_override_earlier_directory_and_reload_transactionally()
    -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        fs::write(
            fixture
                .0
                .join("test/data/example/function/hello.mcfunction"),
            "return 1",
        )?;
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
        let packs = FunctionPacks::from_root(fixture.0.clone())?;
        let key = "example:hello".parse::<Identifier>()?;
        assert_eq!(packs.pack_names(), vec!["file/test", "file/z.zip"]);
        assert_eq!(
            packs.commands(&key).as_deref(),
            Some(&["return 2".to_owned()][..])
        );
        assert_eq!(packs.tick_functions(), vec![key.clone()]);
        write_zip(
            &zip,
            &[
                ("pack.mcmeta", metadata),
                ("data/example/function/hello.mcfunction", "/return 3"),
            ],
        )?;
        assert!(packs.reload().is_err());
        assert_eq!(
            packs.commands(&key).as_deref(),
            Some(&["return 2".to_owned()][..])
        );
        assert_eq!(packs.tick_functions(), vec![key]);
        Ok(())
    }

    #[test]
    fn directory_overlays_and_filters_fail_without_publishing() -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        let function = fixture.0.join("test/data/example/function/hello.mcfunction");
        fs::write(&function, "return 1")?;
        let packs = FunctionPacks::from_root(fixture.0.clone())?;
        let key = "example:hello".parse::<Identifier>()?;
        for metadata in [
            r#"{"pack":{},"overlays":{"entries":[]}}"#,
            r#"{"pack":{},"filter":{"block":[]}}"#,
        ] {
            fs::write(&function, "return 2")?;
            fs::write(fixture.0.join("test/pack.mcmeta"), metadata)?;
            assert!(packs.reload().is_err());
            assert_eq!(packs.commands(&key).as_deref(), Some(&["return 1".to_owned()][..]));
        }
        Ok(())
    }

    #[test]
    fn unsupported_zip_overlays_fail_without_publishing() -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        fs::write(
            fixture
                .0
                .join("test/data/example/function/hello.mcfunction"),
            "return 1",
        )?;
        let packs = FunctionPacks::from_root(fixture.0.clone())?;
        write_zip(
            &fixture.0.join("z.zip"),
            &[("pack.mcmeta", r#"{"pack":{},"overlays":{}}"#)],
        )?;
        assert!(packs.reload().is_err());
        assert_eq!(packs.pack_names(), vec!["file/test"]);
        let key = "example:hello".parse::<Identifier>()?;
        assert_eq!(
            packs.commands(&key).as_deref(),
            Some(&["return 1".to_owned()][..])
        );
        Ok(())
    }

    #[test]
    fn load_tag_runs_once_tick_tag_repeats_and_successful_reload_rearms_load()
    -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        fs::write(
            fixture
                .0
                .join("test/data/example/function/hello.mcfunction"),
            "return 7",
        )?;
        for name in ["load", "tick"] {
            fs::write(
                fixture
                    .0
                    .join(format!("test/data/minecraft/tags/function/{name}.json")),
                r#"{"values":["example:hello",{"id":"example:optional","required":false}]}"#,
            )?;
        }
        let packs = FunctionPacks::from_root(fixture.0.clone())?;
        let key = "example:hello".parse::<Identifier>()?;
        assert_eq!(packs.tick_functions(), vec![key.clone(), key.clone()]);
        assert_eq!(packs.tick_functions(), vec![key.clone()]);
        packs.reload()?;
        assert_eq!(packs.tick_functions(), vec![key.clone(), key]);
        Ok(())
    }

    #[test]
    fn tick_tag_expands_nested_tags_and_refreshes_after_reload() -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        let tags = fixture.0.join("test/data/example/tags/function");
        fs::create_dir_all(&tags)?;
        for name in ["one", "two"] {
            fs::write(
                fixture
                    .0
                    .join(format!("test/data/example/function/{name}.mcfunction")),
                "return 1",
            )?;
        }
        let nested = tags.join("nested.json");
        fs::write(&nested, r#"{"values":["example:one","example:two"]}"#)?;
        fs::write(
            fixture
                .0
                .join("test/data/minecraft/tags/function/tick.json"),
            r##"{"values":["#example:nested","example:one"]}"##,
        )?;
        let packs = FunctionPacks::from_root(fixture.0.clone())?;
        let one = "example:one".parse::<Identifier>()?;
        let two = "example:two".parse::<Identifier>()?;
        assert_eq!(packs.tick_functions(), vec![one.clone(), two.clone()]);
        assert_eq!(packs.tick_functions(), vec![one.clone(), two]);
        fs::write(&nested, r#"{"values":["example:one"]}"#)?;
        packs.reload()?;
        assert_eq!(packs.tick_functions(), vec![one]);
        Ok(())
    }

    #[tokio::test]
    async fn async_reload_preserves_old_snapshot_and_rejects_invalid_resources()
    -> Result<(), Box<dyn Error>> {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        let function = fixture
            .0
            .join("test/data/example/function/hello.mcfunction");
        fs::write(&function, "return 1")?;
        let packs = FunctionPacks::from_root(fixture.0.clone())?;
        let key = "example:hello".parse::<Identifier>()?;
        let old = packs.state.load_full();
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

        fs::write(&function, "/return 3")?;
        assert!(packs.prepare_reload().await.is_err());
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
            fs::write(
                fixture
                    .0
                    .join(format!("test/data/example/function/{name}.mcfunction")),
                "return 1",
            )?;
            fs::write(
                fixture
                    .0
                    .join(format!("test/data/minecraft/tags/function/{name}.json")),
                format!(r#"{{"values":["example:{name}"]}}"#),
            )?;
        }
        let packs = FunctionPacks::from_root(fixture.0.clone())?;
        let barrier = Barrier::new(8);
        let results = thread::scope(|scope| {
            let mut threads = Vec::new();
            for _ in 0..8 {
                threads.push(scope.spawn(|| {
                    barrier.wait();
                    packs.tick_functions()
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
    fn cyclic_and_missing_required_function_tags_reject_publication() -> Result<(), Box<dyn Error>>
    {
        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        let tag = fixture
            .0
            .join("test/data/minecraft/tags/function/tick.json");
        fs::write(&tag, r##"{"values":["#minecraft:tick"]}"##)?;
        assert!(FunctionPacks::from_root(fixture.0.clone()).is_err());
        fs::write(&tag, r#"{"values":["example:missing"]}"#)?;
        assert!(FunctionPacks::from_root(fixture.0.clone()).is_err());
        Ok(())
    }
}
