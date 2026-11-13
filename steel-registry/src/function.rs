//! Datapack functions (`data/<namespace>/function/*.mcfunction`).
//!
//! Vanilla loads each function as a list of commands and runs them in order
//! through `FunctionCommand`. Steel compiles the commands from the built-in
//! vanilla datapack at build time and stores them here, so `/function` and the
//! `function` line inside another function both resolve against the same
//! registry.
//!
//! Function bodies are split into lines at build time; comment lines (`#`) and
//! blank lines are dropped, exactly as vanilla's `CommandFunction` parser does.

use rustc_hash::FxHashMap;
use steel_utils::Identifier;

/// One command line of a function.
pub type FunctionLine = &'static str;

/// A loaded function.
#[derive(Debug)]
pub struct Function {
    /// The function's identifier, e.g. `minecraft:demo/hello`.
    pub key: Identifier,
    /// The command lines, in execution order.
    pub commands: &'static [FunctionLine],
}

/// A reference to a registered function.
pub type FunctionRef = &'static Function;

/// Registry of every function in the built-in datapack.
pub struct FunctionRegistry {
    functions_by_key: FxHashMap<Identifier, FunctionRef>,
    allows_registering: bool,
}

impl FunctionRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            functions_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }

    /// Registers a function and returns whether it replaced an existing one.
    pub fn register(&mut self, function: FunctionRef) -> bool {
        assert!(
            self.allows_registering,
            "Cannot register functions after the registry has been frozen"
        );
        self.functions_by_key
            .insert(function.key.clone(), function)
            .is_some()
    }

    /// Looks a function up by identifier.
    #[must_use]
    pub fn by_key(&self, key: &Identifier) -> Option<FunctionRef> {
        self.functions_by_key.get(key).copied()
    }

    /// Looks a function up by its textual identifier, accepting both the full
    /// `namespace:path` form and a bare `path` in the default namespace.
    #[must_use]
    pub fn by_name(&self, name: &str) -> Option<FunctionRef> {
        if let Some(key) = self.functions_by_key.keys().find(|key| key.to_string() == name) {
            return self.functions_by_key.get(key).copied();
        }
        let (namespace, path) = name.split_once(':').unwrap_or(("minecraft", name));
        let key = Identifier::new(namespace.to_owned(), path.to_owned());
        self.functions_by_key.get(&key).copied()
    }

    /// The number of loaded functions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.functions_by_key.len()
    }

    /// Whether no function is loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.functions_by_key.is_empty()
    }

    /// Every loaded function id, sorted.
    #[must_use]
    pub fn ids(&self) -> Vec<Identifier> {
        let mut ids: Vec<Identifier> = self.functions_by_key.keys().cloned().collect();
        ids.sort_by(|left, right| left.to_string().cmp(&right.to_string()));
        ids
    }

    /// Freezes the registry: no function may be added afterwards.
    pub fn freeze(&mut self) {
        self.allows_registering = false;
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static DEMO: Function = Function {
        key: Identifier::vanilla_static("demo/hello"),
        commands: &["say hello"],
    };

    fn registry() -> FunctionRegistry {
        let mut registry = FunctionRegistry::new();
        registry.register(&DEMO);
        registry
    }

    #[test]
    fn a_function_resolves_by_full_identifier() {
        let registry = registry();
        let found = registry
            .by_name("minecraft:demo/hello")
            .expect("the demo function is registered");
        assert_eq!(found.commands, &["say hello"]);
    }

    #[test]
    fn a_bare_path_resolves_in_the_default_namespace() {
        let registry = registry();
        assert!(registry.by_name("demo/hello").is_some());
    }

    #[test]
    fn an_unknown_function_is_not_found() {
        let registry = registry();
        assert!(registry.by_name("minecraft:nope").is_none());
        assert!(registry.by_name("other:demo/hello").is_none());
    }

    #[test]
    fn registering_the_same_id_twice_reports_the_replacement() {
        let mut registry = registry();
        assert!(registry.register(&DEMO));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn freezing_rejects_later_registrations() {
        let mut registry = registry();
        registry.freeze();
        let result = std::panic::catch_unwind(move || registry.register(&DEMO));
        assert!(result.is_err());
    }

    #[test]
    fn ids_are_sorted() {
        static ANOTHER: Function = Function {
            key: Identifier::vanilla_static("aaa/first"),
            commands: &[],
        };
        let mut registry = registry();
        registry.register(&ANOTHER);
        assert_eq!(
            registry
                .ids()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["minecraft:aaa/first", "minecraft:demo/hello"]
        );
    }
}
