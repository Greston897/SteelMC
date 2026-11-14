//! Domain-scoped command scoreboard state.
//!
//! This module owns the scoreboard data needed by selectors and command
//! execution: objective identity and mutability, score values and locks, and
//! team membership. Display slots and client presentation are outside this
//! command-system scope.

use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    io,
    sync::atomic::{AtomicU64, Ordering},
};

use serde::{Deserialize, Serialize};
use steel_utils::locks::{AsyncMutex, SyncRwLock};
use thiserror::Error;

use crate::{server::worlds::WorldMap, world::World};
use steel_utils::saved_data::names as saved_data_names;

/// Score holder name stored by the vanilla scoreboard.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScoreHolder {
    name: String,
}

impl ScoreHolder {
    /// Creates a score holder from its scoreboard name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Returns the scoreboard name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Objective identity resolved from one domain scoreboard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoreboardObjective {
    name: String,
    read_only: bool,
    criterion: Option<String>,
}

impl ScoreboardObjective {
    /// Returns the objective name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether commands may change scores for this objective.
    #[must_use]
    pub const fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Returns the criterion this objective tracks, if any.
    ///
    /// Vanilla stores it on `Objective#getCriteria`; Steel keeps the identifier
    /// string so statistic criteria and the `trigger` criterion stay data.
    #[must_use]
    pub fn criterion(&self) -> Option<&str> {
        self.criterion.as_deref()
    }

    /// Whether this objective uses vanilla's `trigger` criterion.
    #[must_use]
    pub fn is_trigger(&self) -> bool {
        self.criterion() == Some(TRIGGER_CRITERION)
    }
}

/// Vanilla criterion id for objectives writable through `/trigger`.
pub const TRIGGER_CRITERION: &str = "trigger";

/// Vanilla `ObjectiveCriteria` names that have no data source in Steel.
///
/// Vanilla registers these as constant criteria; the ones that reflect live
/// player state (`health`, `food`, `air`, `armor`, `xp`, `level`) are read by
/// clients from the player entity, and the kill/death counters are bumped by
/// the vanilla scoreboard owners Steel does not run. They are legal to create
/// and read-only for commands, exactly like in vanilla.
const NON_STATISTIC_CRITERIA: [&str; 9] = [
    "dummy",
    "trigger",
    "deathCount",
    "playerKillCount",
    "totalKillCount",
    "health",
    "food",
    "air",
    "armor",
];

/// Vanilla team colours usable in `teamkill.<colour>` / `killedByTeam.<colour>`.
const TEAM_COLORS: [&str; 16] = [
    "black", "dark_blue", "dark_green", "dark_aqua", "dark_red", "dark_purple", "gold", "gray",
    "dark_gray", "blue", "green", "aqua", "red", "light_purple", "yellow", "white",
];

/// Classifies a criterion the way vanilla's `ObjectiveCriteria` does.
///
/// Returns `None` for a criterion vanilla would reject, `Some(true)` for a
/// criterion commands may not write, and `Some(false)` for `dummy` and
/// `trigger`, which are the only writable ones in vanilla.
///
/// Vanilla keys its criterion registry by the exact name `/scoreboard` was
/// given, so `health` is legal and `minecraft:health` is not: only the
/// statistic criteria carry a namespace.
#[must_use]
pub fn criterion_is_read_only(criterion: &str) -> Option<bool> {
    // `xp` and `level` are the pre-1.20.5 spellings of the same criteria.
    if criterion == "xp" || criterion == "level" {
        return Some(true);
    }
    if let Some(rest) = criterion
        .strip_prefix("teamkill.")
        .or_else(|| criterion.strip_prefix("killedByTeam."))
    {
        return TEAM_COLORS.contains(&rest).then_some(true);
    }
    if NON_STATISTIC_CRITERIA.contains(&criterion) {
        return Some(criterion != "dummy" && criterion != TRIGGER_CRITERION);
    }
    if steel_registry::stat::is_statistic_criterion(criterion) {
        return Some(true);
    }
    None
}

/// Team identity resolved from one domain scoreboard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoreboardTeam {
    name: String,
}

impl ScoreboardTeam {
    /// Returns the team name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Stored score fields used by command execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScoreboardScore {
    value: i32,
    #[serde(default = "default_score_locked")]
    locked: bool,
}

impl ScoreboardScore {
    const fn new(value: i32) -> Self {
        Self {
            value,
            locked: true,
        }
    }

    /// Returns the integer score value.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.value
    }

    /// Returns whether `/trigger`-style writes are locked.
    #[must_use]
    pub const fn is_locked(self) -> bool {
        self.locked
    }
}

impl Default for ScoreboardScore {
    fn default() -> Self {
        Self::new(0)
    }
}

const fn default_score_locked() -> bool {
    true
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ObjectiveState {
    read_only: bool,
    /// Name shown to clients; `None` means "same as the objective name".
    ///
    /// Vanilla `Objective#getDisplayName`, which defaults to the name.
    #[serde(default)]
    display_name: Option<String>,
    /// Statistic this objective tracks, if it was created with a stat criterion.
    ///
    /// Vanilla allows any criterion id here; Steel only models the statistic
    /// criteria, which are the ones with a data source on the server.
    #[serde(default)]
    criterion: Option<String>,
}

/// Vanilla `Team.Visibility`: who may see a message routed through the team.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamVisibility {
    /// Every player sees the message.
    #[default]
    Always,
    /// Only players on other teams see the message.
    HideForOwnTeam,
    /// Only players on the same team see the message.
    HideForOtherTeams,
    /// Nobody sees the message.
    Never,
}

impl TeamVisibility {
    /// Returns whether `viewer_team` (the viewer's team name, if any) may see a
    /// message emitted on `message_team`.
    #[must_use]
    pub fn visible_to(&self, message_team: &str, viewer_team: Option<&str>) -> bool {
        let same_team = viewer_team == Some(message_team);
        match self {
            Self::Always => true,
            Self::HideForOwnTeam => !same_team,
            Self::HideForOtherTeams => same_team,
            Self::Never => false,
        }
    }

    /// Vanilla command name (`always`, `hideForOtherTeams`, `hideForOwnTeam`, `never`).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::HideForOwnTeam => "hideForOwnTeam",
            Self::HideForOtherTeams => "hideForOtherTeams",
            Self::Never => "never",
        }
    }
}

/// Persisted state of one scoreboard team.
///
/// Mirrors the fields vanilla's `PlayerTeam` writes to `scoreboard.dat` and to the
/// client packet, so `/team modify` can round-trip every option.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamState {
    /// Who may see messages routed through this team.
    #[serde(default)]
    pub visibility: TeamVisibility,
    /// Who may see nametags of members.
    #[serde(default)]
    pub name_tag_visibility: TeamVisibility,
    /// Whether members may push each other.
    #[serde(default)]
    pub allow_friendly_fire: bool,
    /// Whether death messages are shown for members.
    #[serde(default)]
    pub death_message_visibility: TeamVisibility,
    /// Colour for member names, if any.
    #[serde(default)]
    pub color: Option<String>,
    /// Prefix shown before member names.
    #[serde(default)]
    pub prefix: String,
    /// Suffix shown after member names.
    #[serde(default)]
    pub suffix: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct PersistentScoreboard {
    objectives: BTreeMap<String, ObjectiveState>,
    scores: BTreeMap<String, BTreeMap<String, ScoreboardScore>>,
    teams: BTreeMap<String, TeamState>,
    holder_teams: BTreeMap<String, String>,
    /// Named server tick counters driven by `/stopwatch`.
    ///
    /// Vanilla keeps these on the scoreboard so they persist with the level and
    /// are visible to `/execute if stopwatch`.
    stopwatches: BTreeMap<String, StopwatchState>,
    /// Objective bound to each vanilla `DisplaySlot` ordinal.
    ///
    /// Vanilla `Scoreboard#displayObjectives`; slots without an objective are
    /// absent rather than mapped to an empty string.
    #[serde(default)]
    display_slots: BTreeMap<i32, String>,
}

/// One named stopwatch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StopwatchState {
    ticks: i64,
}

impl StopwatchState {
    /// Ticks this stopwatch has accumulated.
    #[must_use]
    pub const fn ticks(self) -> i64 {
        self.ticks
    }
}

/// Invalid scoreboard operation or persisted state.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ScoreboardError {
    /// Objective names may not be empty.
    #[error("objective name cannot be empty")]
    EmptyObjectiveName,
    /// Team names may not be empty.
    #[error("team name cannot be empty")]
    EmptyTeamName,
    /// Stopwatch names may not be empty.
    #[error("stopwatch name cannot be empty")]
    EmptyStopwatchName,
    /// Score holder names may not be empty.
    #[error("score holder name cannot be empty")]
    EmptyScoreHolderName,
    /// An objective already exists.
    #[error("objective '{0}' already exists")]
    DuplicateObjective(String),
    /// A team already exists.
    #[error("team '{0}' already exists")]
    DuplicateTeam(String),
    /// The requested objective does not exist.
    #[error("objective '{0}' does not exist")]
    MissingObjective(String),
    /// The requested team does not exist.
    #[error("team '{0}' does not exist")]
    MissingTeam(String),
    /// The objective cannot be written by commands.
    #[error("objective '{0}' is read-only")]
    ReadOnlyObjective(String),
    /// The objective does not use the `trigger` criterion.
    #[error("objective '{0}' does not use the trigger criterion")]
    NotATriggerObjective(String),
    /// The trigger score has not been enabled for this holder.
    #[error("trigger '{0}' is not enabled")]
    TriggerLocked(String),
}

struct ScoreboardSaveSnapshot {
    revision: u64,
    state: PersistentScoreboard,
}

/// Command-facing scoreboard for one Steel domain.
pub struct Scoreboard {
    state: SyncRwLock<PersistentScoreboard>,
    revision: AtomicU64,
    saved_revision: AtomicU64,
}

impl Scoreboard {
    /// Creates an empty, clean scoreboard.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: SyncRwLock::new(PersistentScoreboard::default()),
            revision: AtomicU64::new(0),
            saved_revision: AtomicU64::new(0),
        }
    }

    fn from_persistent(state: PersistentScoreboard) -> Result<Self, ScoreboardError> {
        validate_persistent_scoreboard(&state)?;
        Ok(Self {
            state: SyncRwLock::new(state),
            revision: AtomicU64::new(0),
            saved_revision: AtomicU64::new(0),
        })
    }

    /// Adds a writable objective.
    ///
    /// # Errors
    ///
    /// Returns an error if the objective name is empty or already exists.
    pub fn add_objective(
        &self,
        name: impl Into<String>,
    ) -> Result<ScoreboardObjective, ScoreboardError> {
        self.add_objective_with_read_only(name, false)
    }

    /// Creates a named stopwatch (vanilla `/stopwatch create`).
    ///
    /// Returns `false` when a stopwatch with that name already exists, matching
    /// vanilla's "already exists" failure without clobbering its counter.
    pub fn create_stopwatch(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        let mut state = self.state.write();
        if state.stopwatches.contains_key(name) {
            return false;
        }
        state
            .stopwatches
            .insert(name.to_owned(), StopwatchState::default());
        self.mark_dirty();
        true
    }

    /// Removes a named stopwatch, returning whether it existed.
    pub fn remove_stopwatch(&self, name: &str) -> bool {
        let mut state = self.state.write();
        let removed = state.stopwatches.remove(name).is_some();
        if removed {
            self.mark_dirty();
        }
        removed
    }

    /// Reads a stopwatch's accumulated ticks.
    #[must_use]
    pub fn stopwatch_ticks(&self, name: &str) -> Option<i64> {
        self.state
            .read()
            .stopwatches
            .get(name)
            .map(|stopwatch| stopwatch.ticks())
    }

    /// Names of every stopwatch, sorted.
    #[must_use]
    pub fn stopwatch_names(&self) -> Vec<String> {
        self.state.read().stopwatches.keys().cloned().collect()
    }

    /// Advances every stopwatch by one tick.
    ///
    /// Called once per game tick from the world tick phase, exactly where
    /// vanilla advances `ServerScoreboard` stopwatches.
    pub fn tick_stopwatches(&self) {
        let mut state = self.state.write();
        if state.stopwatches.is_empty() {
            return;
        }
        for stopwatch in state.stopwatches.values_mut() {
            stopwatch.ticks = stopwatch.ticks.saturating_add(1);
        }
    }

    /// Adds an objective with explicit command mutability.
    ///
    /// # Errors
    ///
    /// Returns an error if the objective name is empty or already exists.
    pub fn add_objective_with_read_only(
        &self,
        name: impl Into<String>,
        read_only: bool,
    ) -> Result<ScoreboardObjective, ScoreboardError> {
        let name = name.into();
        ensure_objective_name(&name)?;
        let mut state = self.state.write();
        if state.objectives.contains_key(&name) {
            return Err(ScoreboardError::DuplicateObjective(name));
        }
        state.objectives.insert(
            name.clone(),
            ObjectiveState {
                read_only,
                display_name: None,
                criterion: None,
            },
        );
        self.mark_dirty();
        Ok(ScoreboardObjective {
            name,
            read_only,
            criterion: None,
        })
    }

    /// Adds an objective that tracks `criterion`.
    ///
    /// Mirrors `/scoreboard objectives add <name> <criterion>`. Vanilla keys the
    /// objective's behaviour off the criterion, so a statistic criterion
    /// (`minecraft.mined:minecraft.stone`) is filled by its statistic and refuses
    /// command writes, while `dummy` and `trigger` stay command-writable.
    ///
    /// # Errors
    ///
    /// Returns an error if the objective name is empty or already exists.
    pub fn add_objective_with_criterion(
        &self,
        name: impl Into<String>,
        criterion: impl Into<String>,
    ) -> Result<ScoreboardObjective, ScoreboardError> {
        let name = name.into();
        let criterion = criterion.into();
        ensure_objective_name(&name)?;
        let mut state = self.state.write();
        if state.objectives.contains_key(&name) {
            return Err(ScoreboardError::DuplicateObjective(name));
        }
        let read_only = criterion_is_read_only(&criterion).unwrap_or(true);
        state.objectives.insert(
            name.clone(),
            ObjectiveState {
                read_only,
                display_name: None,
                criterion: Some(criterion),
            },
        );
        self.mark_dirty();
        Ok(ScoreboardObjective {
            name,
            read_only,
            criterion: Some(criterion),
        })
    }

    /// Objective names that track `criterion`, in name order.
    #[must_use]
    pub fn objectives_with_criterion(&self, criterion: &str) -> Vec<String> {
        self.state
            .read()
            .objectives
            .iter()
            .filter(|(_, objective)| objective.criterion.as_deref() == Some(criterion))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Returns an objective by name.
    #[must_use]
    pub fn objective(&self, name: &str) -> Option<ScoreboardObjective> {
        self.state
            .read()
            .objectives
            .get(name)
            .map(|objective| ScoreboardObjective {
                name: name.to_owned(),
                read_only: objective.read_only,
                criterion: objective.criterion.clone(),
            })
    }

    /// Returns objective names in stable order.
    #[must_use]
    pub fn objective_names(&self) -> Vec<String> {
        self.state.read().objectives.keys().cloned().collect()
    }

    /// Adds a team.
    ///
    /// # Errors
    ///
    /// Returns an error if the team name is empty or already exists.
    pub fn add_team(&self, name: impl Into<String>) -> Result<ScoreboardTeam, ScoreboardError> {
        let name = name.into();
        ensure_team_name(&name)?;
        let mut state = self.state.write();
        if state
            .teams
            .insert(name.clone(), TeamState::default())
            .is_some()
        {
            return Err(ScoreboardError::DuplicateTeam(name));
        }
        self.mark_dirty();
        Ok(ScoreboardTeam { name })
    }

    /// Returns a team by name.
    #[must_use]
    pub fn team(&self, name: &str) -> Option<ScoreboardTeam> {
        self.state
            .read()
            .teams
            .contains_key(name)
            .then(|| ScoreboardTeam {
                name: name.to_owned(),
            })
    }

    /// Returns the persisted state of a team.
    #[must_use]
    pub fn team_state(&self, name: &str) -> Option<TeamState> {
        self.state.read().teams.get(name).cloned()
    }

    /// Mutates a team's state in place and returns the result.
    ///
    /// Returns `None` when the team does not exist.
    pub fn update_team<F>(&self, name: &str, f: F) -> Option<TeamState>
    where
        F: FnOnce(&mut TeamState),
    {
        let mut state = self.state.write();
        let team = state.teams.get_mut(name)?;
        f(team);
        let result = team.clone();
        drop(state);
        self.mark_dirty();
        Some(result)
    }

    /// Removes a team and every membership pointing at it.
    ///
    /// Returns whether the team existed.
    pub fn remove_team(&self, name: &str) -> bool {
        let mut state = self.state.write();
        if state.teams.remove(name).is_none() {
            return false;
        }
        state.holder_teams.retain(|_, team| team != name);
        drop(state);
        self.mark_dirty();
        true
    }

    /// Removes a holder from its team.
    ///
    /// Returns whether the holder was on a team.
    pub fn remove_holder_from_team(&self, holder: &ScoreHolder) -> bool {
        let mut state = self.state.write();
        if state.holder_teams.remove(holder.name()).is_none() {
            return false;
        }
        drop(state);
        self.mark_dirty();
        true
    }

    /// Returns the team the holder belongs to, if any.
    #[must_use]
    pub fn team_of(&self, holder: &ScoreHolder) -> Option<String> {
        self.state
            .read()
            .holder_teams
            .get(holder.name())
            .cloned()
    }

    /// Returns every holder currently on `team_name`, in stable order.
    #[must_use]
    pub fn team_members(&self, team_name: &str) -> Vec<String> {
        self.state
            .read()
            .holder_teams
            .iter()
            .filter(|(_, team)| team.as_str() == team_name)
            .map(|(holder, _)| holder.clone())
            .collect()
    }

    /// Returns whether `viewer` may see a message emitted on `team_name`.
    ///
    /// `viewer` resolves through their own team membership, so callers only pass
    /// the score holder.
    #[must_use]
    pub fn can_see_team_message(&self, viewer: &ScoreHolder, team_name: &str) -> bool {
        let state = self.state.read();
        let Some(team) = state.teams.get(team_name) else {
            return true;
        };
        let viewer_team = state.holder_teams.get(viewer.name()).map(String::as_str);
        team.visibility.visible_to(team_name, viewer_team)
    }

    /// Returns team names in stable order.
    #[must_use]
    pub fn team_names(&self) -> Vec<String> {
        self.state.read().teams.keys().cloned().collect()
    }

    /// Returns the current team name for a score holder.
    #[must_use]
    pub fn holder_team_name(&self, holder: &ScoreHolder) -> Option<String> {
        self.state.read().holder_teams.get(holder.name()).cloned()
    }

    /// Adds a holder to a team, replacing any prior membership.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty holder or a team that no longer exists.
    pub fn add_holder_to_team(
        &self,
        holder: &ScoreHolder,
        team: &ScoreboardTeam,
    ) -> Result<(), ScoreboardError> {
        ensure_holder_name(holder.name())?;
        let mut state = self.state.write();
        if !state.teams.contains_key(team.name()) {
            return Err(ScoreboardError::MissingTeam(team.name().to_owned()));
        }
        if state
            .holder_teams
            .insert(holder.name().to_owned(), team.name().to_owned())
            .as_deref()
            == Some(team.name())
        {
            return Ok(());
        }
        self.mark_dirty();
        Ok(())
    }

    /// Returns tracked score holders in stable order.
    #[must_use]
    pub fn tracked_holders(&self) -> Vec<ScoreHolder> {
        self.state
            .read()
            .scores
            .keys()
            .map(|name| ScoreHolder::new(name.to_owned()))
            .collect()
    }

    /// Returns the complete score entry for a holder and objective.
    #[must_use]
    pub fn score_entry(
        &self,
        holder: &ScoreHolder,
        objective: &ScoreboardObjective,
    ) -> Option<ScoreboardScore> {
        self.state
            .read()
            .scores
            .get(holder.name())
            .and_then(|scores| scores.get(objective.name()).copied())
    }

    /// Returns the integer score for a holder and objective.
    #[must_use]
    pub fn score(&self, holder: &ScoreHolder, objective: &ScoreboardObjective) -> Option<i32> {
        self.score_entry(holder, objective)
            .map(ScoreboardScore::value)
    }

    /// Sets a holder's score, preserving its lock state when already present.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty holder, a missing objective, or a read-only objective.
    pub fn set_score(
        &self,
        holder: &ScoreHolder,
        objective: &ScoreboardObjective,
        value: i32,
    ) -> Result<(), ScoreboardError> {
        ensure_holder_name(holder.name())?;
        let mut state = self.state.write();
        ensure_writable_objective(&state, objective)?;
        let scores = state.scores.entry(holder.name().to_owned()).or_default();
        match scores.entry(objective.name().to_owned()) {
            Entry::Vacant(entry) => {
                entry.insert(ScoreboardScore::new(value));
            }
            Entry::Occupied(mut entry) => {
                if entry.get().value == value {
                    return Ok(());
                }
                entry.get_mut().value = value;
            }
        }
        self.mark_dirty();
        Ok(())
    }

    /// Whether `holder` may currently write this objective through `/trigger`.
    ///
    /// Vanilla `Scoreboard#hasPlayerScore` + `Score.isLocked`: a missing score
    /// counts as locked, so `/trigger` fails until `/scoreboard players enable`.
    #[must_use]
    pub fn is_trigger_enabled(
        &self,
        holder: &ScoreHolder,
        objective: &ScoreboardObjective,
    ) -> bool {
        self.state
            .read()
            .scores
            .get(holder.name())
            .and_then(|scores| scores.get(objective.name()))
            .is_some_and(|score| !score.locked)
    }

    /// Vanilla `Scoreboard#enablePlayerTrigger`: unlocks a trigger score.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty holder, a missing objective, or an
    /// objective that does not use the `trigger` criterion.
    pub fn enable_trigger(
        &self,
        holder: &ScoreHolder,
        objective: &ScoreboardObjective,
    ) -> Result<(), ScoreboardError> {
        if !objective.is_trigger() {
            return Err(ScoreboardError::NotATriggerObjective(
                objective.name().to_owned(),
            ));
        }
        self.set_score_locked(holder, objective, false)
    }

    /// Vanilla `/trigger` write: sets the value once, honouring the lock.
    ///
    /// Vanilla `Scoreboard#setPlayerScore` is not used here; `/trigger` goes
    /// through `Score.set` on the unlocked entry and relocks it afterwards.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty holder, a missing objective, a non-trigger
    /// objective, or a still-locked score.
    pub fn trigger_score(
        &self,
        holder: &ScoreHolder,
        objective: &ScoreboardObjective,
        value: i32,
    ) -> Result<(), ScoreboardError> {
        ensure_holder_name(holder.name())?;
        if !objective.is_trigger() {
            return Err(ScoreboardError::NotATriggerObjective(
                objective.name().to_owned(),
            ));
        }
        let mut state = self.state.write();
        ensure_objective_exists(&state, objective)?;
        let scores = state.scores.entry(holder.name().to_owned()).or_default();
        let Some(entry) = scores.get_mut(objective.name()) else {
            return Err(ScoreboardError::TriggerLocked(objective.name().to_owned()));
        };
        if entry.locked {
            return Err(ScoreboardError::TriggerLocked(objective.name().to_owned()));
        }
        entry.value = value;
        // Vanilla relocks the score after every successful `/trigger` value, so
        // a second write needs another `/scoreboard players enable`.
        entry.locked = true;
        drop(state);
        self.mark_dirty();
        Ok(())
    }

    /// Writes a score derived from a data source that the objective itself owns.
    ///
    /// Vanilla `ServerStatsCounter.setValue` writes statistic-backed objectives
    /// directly through `Scoreboard#set`, which ignores the command-facing
    /// read-only flag. Command handlers keep using [`Self::set_score`].
    ///
    /// # Errors
    ///
    /// Returns an error for an empty holder or a missing objective.
    pub fn set_objective_source_score(
        &self,
        holder: &ScoreHolder,
        objective: &ScoreboardObjective,
        value: i32,
    ) -> Result<(), ScoreboardError> {
        ensure_holder_name(holder.name())?;
        let mut state = self.state.write();
        ensure_objective_exists(&state, objective)?;
        let scores = state.scores.entry(holder.name().to_owned()).or_default();
        match scores.entry(objective.name().to_owned()) {
            Entry::Vacant(entry) => {
                entry.insert(ScoreboardScore::new(value));
            }
            Entry::Occupied(mut entry) => {
                if entry.get().value == value {
                    return Ok(());
                }
                entry.get_mut().value = value;
            }
        }
        drop(state);
        self.mark_dirty();
        Ok(())
    }

    /// Changes a score's trigger lock state.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty holder or a missing objective.
    pub fn set_score_locked(
        &self,
        holder: &ScoreHolder,
        objective: &ScoreboardObjective,
        locked: bool,
    ) -> Result<(), ScoreboardError> {
        ensure_holder_name(holder.name())?;
        let mut state = self.state.write();
        ensure_objective_exists(&state, objective)?;
        let scores = state.scores.entry(holder.name().to_owned()).or_default();
        match scores.entry(objective.name().to_owned()) {
            Entry::Vacant(entry) => {
                entry.insert(ScoreboardScore { value: 0, locked });
            }
            Entry::Occupied(mut entry) => {
                if entry.get().locked == locked {
                    return Ok(());
                }
                entry.get_mut().locked = locked;
            }
        }
        self.mark_dirty();
        Ok(())
    }

    /// Removes an objective and every score stored against it.
    ///
    /// # Errors
    ///
    /// Returns an error when the objective is missing or read-only.
    pub fn remove_objective(&self, name: &str) -> Result<(), ScoreboardError> {
        let mut state = self.state.write();
        let Some(_) = state.objectives.get(name) else {
            return Err(ScoreboardError::MissingObjective(name.to_owned()));
        };
        if state
            .objectives
            .get(name)
            .is_some_and(|objective| objective.read_only)
        {
            return Err(ScoreboardError::ReadOnlyObjective(name.to_owned()));
        }
        state.objectives.remove(name);
        for scores in state.scores.values_mut() {
            scores.remove(name);
        }
        self.mark_dirty();
        Ok(())
    }

    /// Removes one holder's score for an objective.
    ///
    /// # Errors
    ///
    /// Returns an error when the objective is missing or read-only.
    pub fn remove_score(
        &self,
        holder: &ScoreHolder,
        objective: &ScoreboardObjective,
    ) -> Result<(), ScoreboardError> {
        ensure_holder_name(holder.name())?;
        let mut state = self.state.write();
        ensure_writable_objective(&state, objective)?;
        if let Some(scores) = state.scores.get_mut(holder.name()) {
            scores.remove(objective.name());
        }
        self.mark_dirty();
        Ok(())
    }

    /// Returns objective names that have a score for the holder.
    #[must_use]
    pub fn holder_objectives(&self, holder: &ScoreHolder) -> BTreeSet<String> {
        self.state
            .read()
            .scores
            .get(holder.name())
            .map_or_else(BTreeSet::new, |scores| scores.keys().cloned().collect())
    }

    /// Binds an objective to a display slot, or clears it when `objective` is empty.
    ///
    /// Mirrors vanilla `Scoreboard#setDisplayObjective`.
    pub fn set_display_slot(&self, slot: i32, objective: &str) {
        let mut state = self.state.write();
        if objective.is_empty() {
            state.display_slots.remove(&slot);
        } else {
            state.display_slots.insert(slot, objective.to_owned());
        }
        drop(state);
        self.mark_dirty();
    }

    /// Returns every bound display slot as `(slot, objective)` in stable order.
    #[must_use]
    pub fn display_slots(&self) -> Vec<(i32, String)> {
        self.state
            .read()
            .display_slots
            .iter()
            .map(|(slot, objective)| (*slot, objective.clone()))
            .collect()
    }

    /// Sets the client-visible name of an objective.
    ///
    /// # Errors
    ///
    /// Returns an error when the objective does not exist.
    pub fn set_display_name(&self, name: &str, display: &str) -> Result<(), ScoreboardError> {
        let mut state = self.state.write();
        let Some(objective) = state.objectives.get_mut(name) else {
            return Err(ScoreboardError::MissingObjective(name.to_owned()));
        };
        objective.display_name = Some(display.to_owned());
        drop(state);
        self.mark_dirty();
        Ok(())
    }

    /// Returns the client-visible name of an objective, defaulting to its name.
    #[must_use]
    pub fn display_name(&self, name: &str) -> Option<String> {
        let state = self.state.read();
        let objective = state.objectives.get(name)?;
        Some(
            objective
                .display_name
                .clone()
                .unwrap_or_else(|| name.to_owned()),
        )
    }

    /// Returns whether an objective exists and is read-only.
    #[must_use]
    pub fn is_read_only(&self, name: &str) -> bool {
        self.state
            .read()
            .objectives
            .get(name)
            .is_some_and(|objective| objective.read_only)
    }

    /// Objective names in stable order, with their read-only flag.
    #[must_use]
    pub fn objective_states(&self) -> Vec<(String, bool)> {
        self.state
            .read()
            .objectives
            .iter()
            .map(|(name, state)| (name.clone(), state.read_only))
            .collect()
    }

    /// Objective names in stable order with the client-visible display name.
    #[must_use]
    pub fn objective_display_names(&self) -> Vec<(String, String, bool)> {
        self.state
            .read()
            .objectives
            .iter()
            .map(|(name, state)| {
                (
                    name.clone(),
                    state.display_name.clone().unwrap_or_else(|| name.clone()),
                    state.read_only,
                )
            })
            .collect()
    }

    /// Every stored score as `(holder, objective, value)` in stable order.
    ///
    /// Used for the initial client synchronisation: vanilla sends the whole
    /// scoreboard when a player joins a level.
    #[must_use]
    pub fn score_snapshot(&self) -> Vec<(String, String, i32)> {
        self.state
            .read()
            .scores
            .iter()
            .flat_map(|(holder, scores)| {
                scores
                    .iter()
                    .map(|(objective, score)| (holder.clone(), objective.clone(), score.value))
            })
            .collect()
    }

    /// Team names with their full state, in stable order.
    #[must_use]
    pub fn team_states(&self) -> Vec<(String, TeamState)> {
        self.state
            .read()
            .teams
            .iter()
            .map(|(name, state)| (name.clone(), state.clone()))
            .collect()
    }

    /// Every team membership as `(team, holder)` in stable order.
    #[must_use]
    pub fn team_member_snapshot(&self) -> Vec<(String, String)> {
        self.state
            .read()
            .holder_teams
            .iter()
            .map(|(holder, team)| (team.clone(), holder.clone()))
            .collect()
    }

    fn mark_dirty(&self) {
        self.revision.fetch_add(1, Ordering::Release);
    }

    fn pending_save(&self) -> Option<ScoreboardSaveSnapshot> {
        let state = self.state.read();
        let revision = self.revision.load(Ordering::Acquire);
        if revision == self.saved_revision.load(Ordering::Acquire) {
            return None;
        }
        Some(ScoreboardSaveSnapshot {
            revision,
            state: state.clone(),
        })
    }

    fn mark_saved(&self, revision: u64) {
        self.saved_revision.fetch_max(revision, Ordering::Release);
    }
}

impl Default for Scoreboard {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_persistent_scoreboard(state: &PersistentScoreboard) -> Result<(), ScoreboardError> {
    for name in state.objectives.keys() {
        ensure_objective_name(name)?;
    }
    for name in state.teams.keys() {
        ensure_team_name(name)?;
    }
    for name in state.stopwatches.keys() {
        if name.is_empty() {
            return Err(ScoreboardError::EmptyStopwatchName);
        }
    }
    for (holder, scores) in &state.scores {
        ensure_holder_name(holder)?;
        for objective in scores.keys() {
            if !state.objectives.contains_key(objective) {
                return Err(ScoreboardError::MissingObjective(objective.clone()));
            }
        }
    }
    for (holder, team) in &state.holder_teams {
        ensure_holder_name(holder)?;
        if !state.teams.contains_key(team) {
            return Err(ScoreboardError::MissingTeam(team.clone()));
        }
    }
    Ok(())
}

const fn ensure_objective_name(name: &str) -> Result<(), ScoreboardError> {
    if name.is_empty() {
        Err(ScoreboardError::EmptyObjectiveName)
    } else {
        Ok(())
    }
}

const fn ensure_team_name(name: &str) -> Result<(), ScoreboardError> {
    if name.is_empty() {
        Err(ScoreboardError::EmptyTeamName)
    } else {
        Ok(())
    }
}

const fn ensure_holder_name(name: &str) -> Result<(), ScoreboardError> {
    if name.is_empty() {
        Err(ScoreboardError::EmptyScoreHolderName)
    } else {
        Ok(())
    }
}

fn ensure_objective_exists(
    state: &PersistentScoreboard,
    objective: &ScoreboardObjective,
) -> Result<ObjectiveState, ScoreboardError> {
    state
        .objectives
        .get(objective.name())
        .cloned()
        .ok_or_else(|| ScoreboardError::MissingObjective(objective.name().to_owned()))
}

fn ensure_writable_objective(
    state: &PersistentScoreboard,
    objective: &ScoreboardObjective,
) -> Result<(), ScoreboardError> {
    if ensure_objective_exists(state, objective)?.read_only {
        Err(ScoreboardError::ReadOnlyObjective(
            objective.name().to_owned(),
        ))
    } else {
        Ok(())
    }
}

/// Loaded command scoreboards keyed by Steel domain.
pub struct DomainScoreboards {
    scoreboards: BTreeMap<String, Scoreboard>,
    save_lock: AsyncMutex<()>,
}

impl DomainScoreboards {
    /// Loads one scoreboard through each domain's default world saved-data boundary.
    pub async fn load(worlds: &WorldMap) -> io::Result<Self> {
        let mut domains = worlds.domain_names().collect::<Vec<_>>();
        domains.sort_unstable();
        let mut scoreboards = BTreeMap::new();
        for domain in domains {
            let world = domain_default_world(worlds, domain)?;
            let persistent: PersistentScoreboard = world
                .saved_data
                .load_or_default(saved_data_names::SCOREBOARD)
                .await
                .map_err(|error| scoreboard_io_error(domain, error))?;
            let scoreboard = Scoreboard::from_persistent(persistent).map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid scoreboard for domain '{domain}': {error}"),
                )
            })?;
            scoreboards.insert(domain.to_owned(), scoreboard);
        }
        Ok(Self {
            scoreboards,
            save_lock: AsyncMutex::new(()),
        })
    }

    /// Returns the scoreboard for a domain.
    #[must_use]
    /// Advances the stopwatches of every domain scoreboard by one tick.
    pub fn tick_stopwatches(&self) {
        for scoreboard in self.scoreboards.values() {
            scoreboard.tick_stopwatches();
        }
    }

    /// Domains that own a scoreboard, in sorted order.
    pub fn domain_names(&self) -> impl Iterator<Item = &str> {
        self.scoreboards.keys().map(String::as_str)
    }

    pub fn get(&self, domain: &str) -> Option<&Scoreboard> {
        self.scoreboards.get(domain)
    }

    /// Saves every dirty domain scoreboard and returns the number written.
    pub async fn save(&self, worlds: &WorldMap) -> io::Result<usize> {
        let _save_guard = self.save_lock.lock().await;
        let mut saved = 0;
        for (domain, scoreboard) in &self.scoreboards {
            let Some(snapshot) = scoreboard.pending_save() else {
                continue;
            };
            let world = domain_default_world(worlds, domain)?;
            world
                .saved_data
                .save(saved_data_names::SCOREBOARD, &snapshot.state)
                .await
                .map_err(|error| scoreboard_io_error(domain, error))?;
            scoreboard.mark_saved(snapshot.revision);
            saved += 1;
        }
        Ok(saved)
    }
}

fn domain_default_world<'a>(worlds: &'a WorldMap, domain: &str) -> io::Result<&'a World> {
    worlds
        .default_world(domain)
        .map(AsRef::as_ref)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("domain '{domain}' has no loaded default world"),
            )
        })
}

fn scoreboard_io_error(domain: &str, error: io::Error) -> io::Error {
    io::Error::new(
        error.kind(),
        format!("scoreboard I/O failed for domain '{domain}': {error}"),
    )
}

#[cfg(test)]
mod tests {
    use std::{
        env::temp_dir,
        time::{SystemTime, UNIX_EPOCH},
    };
    use tokio::fs;

    use steel_utils::saved_data::{SavedDataManager, names as saved_data_names};

    use super::{
        AsyncMutex, DomainScoreboards, PersistentScoreboard, ScoreHolder, Scoreboard,
        ScoreboardError, TRIGGER_CRITERION, criterion_is_read_only,
    };

    #[test]
    fn display_slots_bind_and_clear_objectives() {
        let scoreboard = Scoreboard::new();
        scoreboard
            .add_objective("kills")
            .expect("objective should be added");
        assert!(scoreboard.display_slots().is_empty());

        scoreboard.set_display_slot(1, "kills");
        assert_eq!(scoreboard.display_slots(), vec![(1, "kills".to_owned())]);

        // An empty objective name clears the slot, matching vanilla
        // `Scoreboard#setDisplayObjective(slot, null)`.
        scoreboard.set_display_slot(1, "");
        assert!(scoreboard.display_slots().is_empty());
    }

    #[test]
    fn display_name_defaults_to_the_objective_name() {
        let scoreboard = Scoreboard::new();
        scoreboard
            .add_objective("kills")
            .expect("objective should be added");
        assert_eq!(scoreboard.display_name("kills").as_deref(), Some("kills"));

        scoreboard
            .set_display_name("kills", "Player Kills")
            .expect("display name should be settable");
        assert_eq!(
            scoreboard.display_name("kills").as_deref(),
            Some("Player Kills")
        );
        // Renaming does not affect the stored objective name.
        assert!(scoreboard.objective("kills").is_some());
    }

    #[test]
    fn snapshots_cover_every_stored_entry() {
        let scoreboard = Scoreboard::new();
        let kills = scoreboard
            .add_objective("kills")
            .expect("objective should be added");
        let deaths = scoreboard
            .add_objective("deaths")
            .expect("objective should be added");
        let holder = ScoreHolder::new("Steve");
        scoreboard
            .set_score(&holder, &kills, 3)
            .expect("score should be writable");
        scoreboard
            .set_score(&holder, &deaths, 1)
            .expect("score should be writable");
        scoreboard
            .add_team("red")
            .expect("team should be added");
        scoreboard
            .add_holder_to_team(&holder, &scoreboard.team("red").expect("team exists"))
            .expect("holder should join the team");

        // The join synchronisation sends everything the client needs to render
        // the scoreboard, so the snapshots must not drop entries.
        assert_eq!(scoreboard.objective_states().len(), 2);
        let mut scores = scoreboard.score_snapshot();
        scores.sort();
        assert_eq!(
            scores,
            vec![
                ("Steve".to_owned(), "deaths".to_owned(), 1),
                ("Steve".to_owned(), "kills".to_owned(), 3),
            ]
        );
        assert_eq!(scoreboard.team_states().len(), 1);
        assert_eq!(
            scoreboard.team_member_snapshot(),
            vec![("red".to_owned(), "Steve".to_owned())]
        );
    }

    #[test]
    fn score_value_and_lock_state_are_independent() {
        let scoreboard = Scoreboard::new();
        let objective = scoreboard
            .add_objective("kills")
            .expect("objective should be added");
        let holder = ScoreHolder::new("Steve");

        scoreboard
            .set_score(&holder, &objective, 7)
            .expect("score should be writable");
        scoreboard
            .set_score_locked(&holder, &objective, false)
            .expect("score lock should change");
        scoreboard
            .set_score(&holder, &objective, 9)
            .expect("score should remain writable");

        let entry = scoreboard
            .score_entry(&holder, &objective)
            .expect("score should exist");
        assert_eq!(entry.value(), 9);
        assert!(!entry.is_locked());
    }

    #[test]
    fn stopwatches_advance_one_tick_at_a_time() {
        let scoreboard = Scoreboard::new();
        assert!(scoreboard.create_stopwatch("raid"));
        // Duplicate creation keeps the existing counter instead of resetting it.
        assert!(!scoreboard.create_stopwatch("raid"));
        assert!(!scoreboard.create_stopwatch(""));

        scoreboard.tick_stopwatches();
        scoreboard.tick_stopwatches();
        assert_eq!(scoreboard.stopwatch_ticks("raid"), Some(2));
        assert_eq!(scoreboard.stopwatch_names(), vec!["raid".to_owned()]);

        assert!(scoreboard.remove_stopwatch("raid"));
        assert!(!scoreboard.remove_stopwatch("raid"));
        assert_eq!(scoreboard.stopwatch_ticks("raid"), None);
        // Ticking with no stopwatches is a no-op, not a panic.
        scoreboard.tick_stopwatches();
    }

    #[test]
    fn read_only_objective_rejects_score_writes() {
        let scoreboard = Scoreboard::new();
        let objective = scoreboard
            .add_objective_with_read_only("health", true)
            .expect("objective should be added");

        assert_eq!(
            scoreboard.set_score(&ScoreHolder::new("Steve"), &objective, 20),
            Err(ScoreboardError::ReadOnlyObjective("health".to_owned()))
        );
    }

    #[test]
    fn team_assignment_replaces_prior_membership() {
        let scoreboard = Scoreboard::new();
        let red = scoreboard
            .add_team("red")
            .expect("red team should be added");
        let blue = scoreboard
            .add_team("blue")
            .expect("blue team should be added");
        let holder = ScoreHolder::new("Steve");

        scoreboard
            .add_holder_to_team(&holder, &red)
            .expect("holder should join red");
        scoreboard
            .add_holder_to_team(&holder, &blue)
            .expect("holder should move to blue");

        assert_eq!(
            scoreboard.holder_team_name(&holder).as_deref(),
            Some("blue")
        );
    }

    #[tokio::test]
    async fn persisted_scoreboard_round_trips_and_becomes_clean_after_save() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_nanos();
        let path = temp_dir().join(format!("steel-scoreboard-{unique}"));
        let manager = SavedDataManager::new(Some(&path));
        let scoreboard = Scoreboard::new();
        let objective = scoreboard
            .add_objective("kills")
            .expect("objective should be added");
        let holder = ScoreHolder::new("Steve");
        scoreboard
            .set_score(&holder, &objective, 5)
            .expect("score should be writable");

        let snapshot = scoreboard
            .pending_save()
            .expect("scoreboard should be dirty");
        manager
            .save(saved_data_names::SCOREBOARD, &snapshot.state)
            .await
            .expect("scoreboard should save");
        scoreboard.mark_saved(snapshot.revision);
        assert!(scoreboard.pending_save().is_none());

        let persistent: PersistentScoreboard = manager
            .load_or_default(saved_data_names::SCOREBOARD)
            .await
            .expect("scoreboard should load");
        let restored = Scoreboard::from_persistent(persistent).expect("scoreboard should validate");
        let restored_objective = restored
            .objective("kills")
            .expect("objective should persist");
        assert_eq!(restored.score(&holder, &restored_objective), Some(5));

        fs::remove_dir_all(path)
            .await
            .expect("temporary scoreboard directory should be removed");
    }

    #[test]
    fn mutation_after_snapshot_remains_dirty_when_snapshot_is_marked_saved() {
        let scoreboard = Scoreboard::new();
        scoreboard
            .add_objective("kills")
            .expect("objective should be added");
        let snapshot = scoreboard
            .pending_save()
            .expect("scoreboard should be dirty");

        scoreboard
            .add_objective("deaths")
            .expect("second objective should be added");
        scoreboard.mark_saved(snapshot.revision);

        let pending = scoreboard
            .pending_save()
            .expect("newer mutation should remain dirty");
        assert!(pending.revision > snapshot.revision);
        assert!(pending.state.objectives.contains_key("deaths"));
    }

    #[test]
    #[test]
    fn a_trigger_objective_starts_locked_until_enabled() {
        let scoreboard = Scoreboard::new();
        let objective = scoreboard
            .add_objective_with_criterion("menu", TRIGGER_CRITERION)
            .expect("trigger objective should be added");
        assert!(objective.is_trigger());
        assert!(
            !objective.is_read_only(),
            "trigger objectives are writable, unlike statistic objectives"
        );
        let holder = ScoreHolder::new("steve");
        assert!(
            !scoreboard.is_trigger_enabled(&holder, &objective),
            "vanilla scores start locked"
        );
        scoreboard
            .enable_trigger(&holder, &objective)
            .expect("enable should unlock the score");
        assert!(scoreboard.is_trigger_enabled(&holder, &objective));
    }

    #[test]
    fn a_trigger_write_relocks_the_score() {
        // Vanilla behaviour players rely on: one `/trigger` per `enable`.
        let scoreboard = Scoreboard::new();
        let objective = scoreboard
            .add_objective_with_criterion("menu", TRIGGER_CRITERION)
            .expect("trigger objective should be added");
        let holder = ScoreHolder::new("steve");
        scoreboard
            .enable_trigger(&holder, &objective)
            .expect("enable should unlock the score");
        scoreboard
            .trigger_score(&holder, &objective, 5)
            .expect("first trigger write should succeed");
        assert_eq!(scoreboard.score(&holder, &objective), Some(5));
        assert!(
            scoreboard
                .trigger_score(&holder, &objective, 9)
                .is_err(),
            "a second write without another enable must fail"
        );
        assert_eq!(scoreboard.score(&holder, &objective), Some(5));
    }

    #[test]
    fn a_trigger_write_needs_an_existing_unlocked_entry() {
        let scoreboard = Scoreboard::new();
        let objective = scoreboard
            .add_objective_with_criterion("menu", TRIGGER_CRITERION)
            .expect("trigger objective should be added");
        let holder = ScoreHolder::new("steve");
        assert!(
            scoreboard.trigger_score(&holder, &objective, 1).is_err(),
            "vanilla `Scoreboard.setPlayerScore` fails for a missing score"
        );
    }

    #[test]
    #[test]
    fn criteria_are_classified_like_vanilla() {
        // Writable criteria: `dummy` and `trigger` only.
        assert_eq!(criterion_is_read_only("dummy"), Some(false));
        assert_eq!(criterion_is_read_only("trigger"), Some(false));
        // Read-only vanilla criteria.
        for criterion in [
            "health", "food", "air", "armor", "xp", "level", "deathCount",
            "playerKillCount", "totalKillCount", "teamkill.red", "killedByTeam.blue",
            "minecraft.mined:minecraft.stone", "minecraft.custom:minecraft.walk_one_cm",
        ] {
            assert_eq!(
                criterion_is_read_only(criterion),
                Some(true),
                "{criterion} should be a read-only vanilla criterion"
            );
        }
        // Namespaced spellings of the constant criteria do not exist in vanilla.
        assert_eq!(criterion_is_read_only("minecraft:health"), None);
        assert_eq!(criterion_is_read_only("teamkill.chartreuse"), None);
        assert_eq!(criterion_is_read_only("nonsense"), None);
    }

    fn a_dummy_objective_is_not_a_trigger() {
        let scoreboard = Scoreboard::new();
        let objective = scoreboard
            .add_objective_with_criterion("kills", "dummy")
            .expect("dummy objective should be added");
        assert!(!objective.is_trigger());
        let holder = ScoreHolder::new("steve");
        assert!(scoreboard.enable_trigger(&holder, &objective).is_err());
    }

    #[test]
    fn a_statistic_objective_cannot_be_written_by_commands() {
        let scoreboard = Scoreboard::new();
        let objective = scoreboard
            .add_objective_with_criterion("mined", "minecraft.mined:minecraft.stone")
            .expect("statistic objective should be added");
        assert!(
            objective.is_read_only(),
            "vanilla renders statistic objectives read-only for commands"
        );
        let holder = ScoreHolder::new("steve");
        assert!(scoreboard.set_score(&holder, &objective, 1).is_err());
        // The statistic itself still writes it, exactly like vanilla's
        // `ServerStatsCounter.setValue`.
        scoreboard
            .set_objective_source_score(&holder, &objective, 7)
            .expect("the statistic source owns this objective");
        assert_eq!(scoreboard.score(&holder, &objective), Some(7));
    }

    fn domains_keep_independent_scoreboards() {
        let scoreboards = DomainScoreboards {
            scoreboards: [
                ("alpha".to_owned(), Scoreboard::new()),
                ("beta".to_owned(), Scoreboard::new()),
            ]
            .into_iter()
            .collect(),
            save_lock: AsyncMutex::new(()),
        };
        scoreboards
            .get("alpha")
            .expect("alpha scoreboard should exist")
            .add_objective("kills")
            .expect("alpha objective should be added");

        assert!(
            scoreboards
                .get("beta")
                .expect("beta scoreboard should exist")
                .objective("kills")
                .is_none()
        );
    }
}
