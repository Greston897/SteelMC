//! Runtime directory data packs: function bodies and function tags.
//!
//! Other data registries are still baked. This module does not claim to reload
//! recipes, worldgen, loot, or advancements merely because their pack is visible.
use arc_swap::ArcSwap;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use steel_utils::Identifier;

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
struct FunctionPackSnapshot {
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
        if tags {
            let tag: FunctionTag = serde_json::from_str(&text)
                .map_err(|error| format!("{}: {error}", path.display()))?;
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
                        "{}:{}: slash-prefixed commands and function macros are not supported",
                        path.display(),
                        line + 1
                    ));
                }
                commands.push(command.to_owned());
            }
            state.functions.insert(key, commands);
        }
    }
    Ok(())
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

    /// Reads and validates resources off the async executor. Cancellation may
    /// leave the blocking read running, but cannot publish its result.
    pub(crate) async fn reload_async(&self) -> Result<usize, String> {
        let Some(root) = &self.root else {
            return Ok(0);
        };
        let root = root.clone();
        let next = tokio::task::spawn_blocking(move || Self::prepare(&root))
            .await
            .map_err(|error| format!("function-pack loading task: {error}"))??;
        Ok(self.publish(next))
    }

    fn prepare(root: &Path) -> Result<FunctionPackSnapshot, String> {
        let mut next = FunctionPackState::default();
        for entry in entries(root)? {
            if !entry
                .file_type()
                .map_err(|error| error.to_string())?
                .is_dir()
            {
                continue;
            }
            let pack = entry.path();
            if !pack.join("pack.mcmeta").is_file() {
                continue;
            }
            let metadata: serde_json::Value =
                serde_json::from_str(&read_resource(&pack.join("pack.mcmeta"))?)
                    .map_err(|error| format!("{}: {error}", pack.display()))?;
            if !metadata
                .get("pack")
                .is_some_and(serde_json::Value::is_object)
            {
                return Err(format!("{}: missing pack metadata", pack.display()));
            }
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

    fn publish(&self, next: FunctionPackSnapshot) -> usize {
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

    /// Names of directory packs whose function resources were loaded.
    pub fn pack_names(&self) -> Vec<String> {
        self.state.load().packs.clone()
    }

    /// Resolve load once per successful publication, followed by tick functions.
    pub fn tick_functions(&self) -> Vec<Identifier> {
        let state = self.state.load();
        if state.load_pending.swap(false, Ordering::AcqRel) {
            let mut functions = Vec::with_capacity(
                state.load_functions.len() + state.tick_functions.len(),
            );
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
        assert_eq!(packs.commands(&key).as_deref(), Some(&["return 7".to_owned()][..]));
        fs::write(&function, "/return 9")?;
        assert!(packs.reload().is_err());
        assert_eq!(packs.commands(&key).as_deref(), Some(&["return 7".to_owned()][..]));
        fs::write(&function, "return 9")?;
        assert_eq!(packs.reload()?, 1);
        assert_eq!(packs.commands(&key).as_deref(), Some(&["return 9".to_owned()][..]));
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
        assert_eq!(packs.reload_async().await?, 1);
        assert_eq!(old.functions.get(&key).map(AsRef::as_ref), Some(&["return 1".to_owned()][..]));
        assert_eq!(&*old_body, &["return 1".to_owned()]);
        assert_eq!(packs.commands(&key).as_deref(), Some(&["return 2".to_owned()][..]));

        fs::write(&function, "/return 3")?;
        assert!(packs.reload_async().await.is_err());
        assert_eq!(packs.commands(&key).as_deref(), Some(&["return 2".to_owned()][..]));
        Ok(())
    }

    #[test]
    fn concurrent_ticks_claim_each_generation_load_once() -> Result<(), Box<dyn Error>> {
        use std::sync::Barrier;

        steel_registry::init_vanilla_registry();
        let fixture = Fixture::new()?;
        for name in ["load", "tick"] {
            fs::write(
                fixture.0.join(format!("test/data/example/function/{name}.mcfunction")),
                "return 1",
            )?;
            fs::write(
                fixture.0.join(format!("test/data/minecraft/tags/function/{name}.json")),
                format!(r#"{{"values":["example:{name}"]}}"#),
            )?;
        }
        let packs = FunctionPacks::from_root(fixture.0.clone())?;
        let barrier = Barrier::new(8);
        let results = std::thread::scope(|scope| {
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
        assert_eq!(results.iter().filter(|result| result.contains(&load)).count(), 1);
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
