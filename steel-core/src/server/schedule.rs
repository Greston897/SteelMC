//! Scheduled commands backing the `/schedule` command.
//!
//! Vanilla `ServerFunctionManager` keeps a list of `ScheduledFunction`s: a
//! command to run after a delay in ticks, on a clock (`schedule function` for
//! the game clock, `schedule clear`) and optionally keyed so a later `schedule`
//! can replace it. Steel keeps the same list here and drains the entries whose
//! time has come from the server tick loop.
//!
//! Two kinds of schedule exist in vanilla and both are modelled:
//!
//! * `replace` — a named schedule replaces any earlier one with the same key.
//! * `append` — the new schedule is queued behind the existing one, and the
//!   name keeps referring to the earliest entry.

use steel_utils::Identifier;
use steel_utils::locks::SyncMutex;

/// How a new schedule combines with an existing one of the same name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleMode {
    /// Replace any pending schedule with the same name.
    Replace,
    /// Queue behind the pending schedule with the same name.
    Append,
}

impl ScheduleMode {
    /// Parses the command's `mode` argument.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "replace" => Some(Self::Replace),
            "append" => Some(Self::Append),
            _ => None,
        }
    }
}

/// One command waiting for its tick.
#[derive(Debug, Clone)]
pub struct ScheduledCommand {
    /// Schedules sharing this name were replaced or appended together.
    pub key: String,
    /// The command to run when the tick arrives.
    pub command: String,
    /// `tick_count` value at which the command runs.
    pub due_tick: u64,
}

/// The server's schedule list.
#[derive(Debug, Default)]
pub struct ScheduleRegistry {
    entries: SyncMutex<Vec<ScheduledCommand>>,
}

impl ScheduleRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Schedules `command` to run at `due_tick`.
    ///
    /// Returns how many earlier schedules were replaced or removed by this
    /// call, which is what `/schedule` reports back to the caller.
    pub fn schedule(
        &self,
        key: String,
        command: String,
        due_tick: u64,
        mode: ScheduleMode,
    ) -> usize {
        let mut entries = self.entries.lock();
        let replaced = match mode {
            ScheduleMode::Replace => {
                let before = entries.len();
                entries.retain(|entry| entry.key != key);
                before - entries.len()
            }
            ScheduleMode::Append => 0,
        };
        entries.push(ScheduledCommand {
            key,
            command,
            due_tick,
        });
        replaced
    }

    /// Schedules `command` on the game clock.
    ///
    /// `function` and `schedule` both address schedules by the function
    /// identifier, so the name is the identifier as a string.
    pub fn schedule_function(
        &self,
        function: &Identifier,
        due_tick: u64,
        mode: ScheduleMode,
    ) -> usize {
        self.schedule(
            function.to_string(),
            format!("function {}", function),
            due_tick,
            mode,
        )
    }

    /// Removes every schedule whose key matches `key` and reports how many were
    /// dropped, mirroring `/schedule clear <id>`.
    pub fn clear(&self, key: &str) -> usize {
        let mut entries = self.entries.lock();
        let before = entries.len();
        entries.retain(|entry| entry.key != key);
        before - entries.len()
    }

    /// Removes every schedule, used when the world or the server shuts down.
    pub fn clear_all(&self) -> usize {
        let mut entries = self.entries.lock();
        let count = entries.len();
        entries.clear();
        count
    }

    /// The number of pending schedules.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.lock().len()
    }

    /// Whether nothing is scheduled.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The commands whose time has come at `tick`, in submission order.
    ///
    /// The entries are removed from the registry before being returned so a
    /// command that schedules another one cannot observe itself twice.
    pub fn due(&self, tick: u64) -> Vec<ScheduledCommand> {
        let mut entries = self.entries.lock();
        let mut due = Vec::new();
        let mut pending = Vec::with_capacity(entries.len());
        for entry in entries.drain(..) {
            if entry.due_tick <= tick {
                due.push(entry);
            } else {
                pending.push(entry);
            }
        }
        *entries = pending;
        due.sort_by_key(|entry| entry.due_tick);
        due
    }

    /// The keys of every pending schedule, sorted, for `/schedule` feedback and
    /// for the tests.
    #[must_use]
    pub fn keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self
            .entries
            .lock()
            .iter()
            .map(|entry| entry.key.clone())
            .collect();
        keys.sort();
        keys.dedup();
        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> ScheduleRegistry {
        ScheduleRegistry::new()
    }

    #[test]
    fn a_due_command_is_removed_and_returned_once() {
        let registry = registry();
        registry.schedule(
            "demo".to_owned(),
            "say hi".to_owned(),
            100,
            ScheduleMode::Replace,
        );

        assert!(registry.due(99).is_empty());
        assert_eq!(registry.len(), 1);

        let due = registry.due(100);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].command, "say hi");
        assert!(registry.is_empty());
        assert!(registry.due(100).is_empty());
    }

    #[test]
    fn replace_keeps_only_the_newest_schedule_of_a_name() {
        let registry = registry();
        assert_eq!(
            registry.schedule(
                "demo".to_owned(),
                "say first".to_owned(),
                10,
                ScheduleMode::Replace
            ),
            0
        );
        assert_eq!(
            registry.schedule(
                "demo".to_owned(),
                "say second".to_owned(),
                20,
                ScheduleMode::Replace
            ),
            1
        );

        let due = registry.due(100);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].command, "say second");
    }

    #[test]
    fn append_keeps_both_schedules() {
        let registry = registry();
        registry.schedule(
            "demo".to_owned(),
            "say first".to_owned(),
            10,
            ScheduleMode::Replace,
        );
        assert_eq!(
            registry.schedule(
                "demo".to_owned(),
                "say second".to_owned(),
                20,
                ScheduleMode::Append
            ),
            0
        );

        let due = registry.due(100);
        assert_eq!(
            due.iter().map(|entry| entry.command.as_str()).collect::<Vec<_>>(),
            vec!["say first", "say second"]
        );
    }

    #[test]
    fn due_entries_are_ordered_by_their_tick() {
        let registry = registry();
        registry.schedule("late".to_owned(), "say late".to_owned(), 30, ScheduleMode::Replace);
        registry.schedule(
            "early".to_owned(),
            "say early".to_owned(),
            20,
            ScheduleMode::Replace,
        );

        let due = registry.due(30);
        assert_eq!(
            due.iter().map(|entry| entry.key.as_str()).collect::<Vec<_>>(),
            vec!["early", "late"]
        );
    }

    #[test]
    fn clear_removes_only_the_named_schedule() {
        let registry = registry();
        registry.schedule("a".to_owned(), "say a".to_owned(), 10, ScheduleMode::Replace);
        registry.schedule("b".to_owned(), "say b".to_owned(), 10, ScheduleMode::Replace);

        assert_eq!(registry.clear("a"), 1);
        assert_eq!(registry.keys(), vec!["b".to_owned()]);
        assert_eq!(registry.clear("missing"), 0);
    }

    #[test]
    fn clear_all_drops_everything() {
        let registry = registry();
        for index in 0..3 {
            registry.schedule(
                format!("key{index}"),
                "say hi".to_owned(),
                10,
                ScheduleMode::Replace,
            );
        }
        assert_eq!(registry.clear_all(), 3);
        assert!(registry.is_empty());
    }

    #[test]
    fn a_function_schedule_is_named_by_its_identifier() {
        let registry = registry();
        let id = Identifier::vanilla_static("demo/hello");
        registry.schedule_function(&id, 5, ScheduleMode::Replace);

        assert_eq!(registry.keys(), vec!["minecraft:demo/hello".to_owned()]);
        let due = registry.due(5);
        assert_eq!(due[0].command, "function minecraft:demo/hello");
    }

    #[test]
    fn the_schedule_mode_parser_accepts_vanilla_names() {
        assert_eq!(ScheduleMode::parse("replace"), Some(ScheduleMode::Replace));
        assert_eq!(ScheduleMode::parse("append"), Some(ScheduleMode::Append));
        assert_eq!(ScheduleMode::parse("Replace"), None);
        assert_eq!(ScheduleMode::parse(""), None);
    }

    #[test]
    fn a_schedule_at_the_current_tick_runs_immediately() {
        let registry = registry();
        registry.schedule("now".to_owned(), "say now".to_owned(), 0, ScheduleMode::Replace);
        assert_eq!(registry.due(0).len(), 1);
    }
}
