//! Скорборд-API — `Scoreboard`, `Objective`, `Team`, `Criteria` как в Bukkit/Paper.
//! Полный аналог `org.bukkit.scoreboard` + Paper-расширения (number format, display).
//! Плагин может создавать скорборд на игрока или глобальный.

use rustc_hash::FxHashMap;
use steel_utils::locks::SyncRwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplaySlot {
    List,
    Sidebar,
    BelowName,
    TeamBlack, TeamDarkBlue, TeamDarkGreen, TeamDarkAqua, TeamDarkRed, TeamDarkPurple,
    TeamGold, TeamGray, TeamDarkGray, TeamBlue, TeamGreen, TeamAqua, TeamRed, TeamLightPurple, TeamYellow, TeamWhite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Criteria {
    Dummy,
    Trigger,
    DeathCount,
    PlayerKillCount,
    TotalKillCount,
    Health,
    Xp,
    Level,
    Food,
    Air,
    Armor,
    Custom(String),
}

impl Criteria {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Dummy => "dummy", Self::Trigger => "trigger", Self::DeathCount => "deathCount",
            Self::PlayerKillCount => "playerKillCount", Self::TotalKillCount => "totalKillCount",
            Self::Health => "health", Self::Xp => "xp", Self::Level => "level",
            Self::Food => "food", Self::Air => "air", Self::Armor => "armor",
            Self::Custom(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberFormat {
    Blank,
    Styled { color: &'static str },
    Fixed { content: &'static str },
}

#[derive(Debug, Clone)]
pub struct Objective {
    pub name: String,
    pub criteria: Criteria,
    pub display_name: String,
    pub display_slot: Option<DisplaySlot>,
    pub number_format: Option<NumberFormat>,
    pub scores: FxHashMap<String, i32>,
}

impl Objective {
    pub fn new(name: impl Into<String>, criteria: Criteria, display: impl Into<String>) -> Self {
        Self { name: name.into(), criteria, display_name: display.into(), display_slot: None, number_format: None, scores: FxHashMap::default() }
    }
    pub fn set_display_slot(&mut self, slot: DisplaySlot) { self.display_slot = Some(slot); }
    pub fn set_score(&mut self, entry: impl Into<String>, score: i32) { self.scores.insert(entry.into(), score); }
    pub fn get_score(&self, entry: &str) -> Option<i32> { self.scores.get(entry).copied() }
    pub fn reset_score(&mut self, entry: &str) -> bool { self.scores.remove(entry).is_some() }
}

#[derive(Debug, Clone)]
pub struct Team {
    pub name: String,
    pub display_name: String,
    pub prefix: String,
    pub suffix: String,
    pub color: Option<String>,
    pub entries: Vec<String>,
    pub allow_friendly_fire: bool,
    pub can_see_friendly_invisibles: bool,
    pub nametag_visibility: String,
    pub collision_rule: String,
}

impl Team {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), display_name: String::new(), prefix: String::new(), suffix: String::new(), color: None, entries: Vec::new(), allow_friendly_fire: true, can_see_friendly_invisibles: true, nametag_visibility: "always".into(), collision_rule: "always".into() }
    }
    pub fn add_entry(&mut self, e: impl Into<String>) { self.entries.push(e.into()); }
    pub fn remove_entry(&mut self, e: &str) -> bool { let before = self.entries.len(); self.entries.retain(|x| x != e); before != self.entries.len() }
}

pub struct Scoreboard {
    objectives: SyncRwLock<FxHashMap<String, Objective>>,
    teams: SyncRwLock<FxHashMap<String, Team>>,
}

impl Default for Scoreboard { fn default() -> Self { Self::new() } }

impl Scoreboard {
    pub fn new() -> Self { Self { objectives: SyncRwLock::new(FxHashMap::default()), teams: SyncRwLock::new(FxHashMap::default()) } }

    pub fn register_objective(&self, obj: Objective) -> bool {
        let mut w = self.objectives.write();
        if w.contains_key(&obj.name) { return false; }
        w.insert(obj.name.clone(), obj); true
    }
    pub fn get_objective(&self, name: &str) -> Option<Objective> { self.objectives.read().get(name).cloned() }
    pub fn remove_objective(&self, name: &str) -> bool { self.objectives.write().remove(name).is_some() }
    pub fn objectives(&self) -> Vec<Objective> { self.objectives.read().values().cloned().collect() }

    pub fn register_team(&self, team: Team) -> bool {
        let mut w = self.teams.write();
        if w.contains_key(&team.name) { return false; }
        w.insert(team.name.clone(), team); true
    }
    pub fn get_team(&self, name: &str) -> Option<Team> { self.teams.read().get(name).cloned() }
    pub fn remove_team(&self, name: &str) -> bool { self.teams.write().remove(name).is_some() }
    pub fn teams(&self) -> Vec<Team> { self.teams.read().values().cloned().collect() }

    pub fn clear(&self) { self.objectives.write().clear(); self.teams.write().clear(); }
}
