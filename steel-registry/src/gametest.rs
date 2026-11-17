//! Vanilla MC 26.3 `minecraft:test_environment` and `minecraft:test_instance`
//! synchronized registries (`RegistryDataLoader.SYNCHRONIZED_REGISTRIES`).
//!
//! Covers all 7 MC 26.3 `TestEnvironmentDefinition` variants (`all_of`,
//! `clock_time`, `difficulty`, `function`, `game_rules`, `timeline_attributes`,
//! `weather`) and both `TestInstance` variants (`block_based`, `function`) with
//! bidirectional NBT codecs.

use std::str::FromStr;

use rustc_hash::FxHashMap;
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use simdnbt::{FromNbtTag, ToNbtTag};
use steel_utils::nbt::NbtNumeric as _;
use steel_utils::types::Difficulty;
use steel_utils::Identifier;

use crate::RegistryTags;

/// Game rule override inside a `minecraft:game_rules` test environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestGameRuleValue {
    Bool(bool),
    Int(i32),
}

/// Weather preset inside a `minecraft:weather` test environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestWeather {
    Clear,
    Rain,
    Thunder,
}

impl TestWeather {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clear => "clear",
            Self::Rain => "rain",
            Self::Thunder => "thunder",
        }
    }

    #[must_use]
    pub const fn from_name(name: &str) -> Option<Self> {
        match name {
            "clear" => Some(Self::Clear),
            "rain" => Some(Self::Rain),
            "thunder" => Some(Self::Thunder),
            _ => None,
        }
    }
}

/// All 7 MC 26.3 `minecraft:test_environment_definition_type` variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestEnvironmentDefinition {
    /// `minecraft:all_of` — sequential composite of child environment definitions.
    AllOf {
        definitions: Vec<TestEnvironmentDefinition>,
    },
    /// `minecraft:clock_time` — sets a world clock to a fixed tick before running.
    ClockTime { clock: Identifier, time: i32 },
    /// `minecraft:difficulty` — switches world difficulty (added in 26.2).
    Difficulty { difficulty: Difficulty },
    /// `minecraft:function` — runs optional setup and teardown `.mcfunction`s.
    Function {
        setup: Option<Identifier>,
        teardown: Option<Identifier>,
    },
    /// `minecraft:game_rules` — applies a map of game rule overrides.
    GameRules {
        rules: Vec<(Identifier, TestGameRuleValue)>,
    },
    /// `minecraft:timeline_attributes` — activates timeline identifiers/tag.
    TimelineAttributes { timelines: Vec<Identifier> },
    /// `minecraft:weather` — forces clear, rain, or thunder weather.
    Weather { weather: TestWeather },
}

impl TestEnvironmentDefinition {
    #[must_use]
    pub fn to_nbt_compound(&self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        match self {
            Self::AllOf { definitions } => {
                compound.insert("type", "minecraft:all_of");
                let list = if definitions.is_empty() {
                    NbtList::Empty
                } else {
                    NbtList::Compound(
                        definitions
                            .iter()
                            .map(Self::to_nbt_compound)
                            .collect::<Vec<_>>(),
                    )
                };
                compound.insert("definitions", NbtTag::List(list));
            }
            Self::ClockTime { clock, time } => {
                compound.insert("type", "minecraft:clock_time");
                compound.insert("clock", clock.to_string().as_str());
                compound.insert("time", *time);
            }
            Self::Difficulty { difficulty } => {
                compound.insert("type", "minecraft:difficulty");
                compound.insert("difficulty", difficulty_name(*difficulty));
            }
            Self::Function { setup, teardown } => {
                compound.insert("type", "minecraft:function");
                if let Some(setup) = setup {
                    compound.insert("setup", setup.to_string().as_str());
                }
                if let Some(teardown) = teardown {
                    compound.insert("teardown", teardown.to_string().as_str());
                }
            }
            Self::GameRules { rules } => {
                compound.insert("type", "minecraft:game_rules");
                let mut rules_compound = NbtCompound::new();
                for (rule, value) in rules {
                    match value {
                        TestGameRuleValue::Bool(b) => {
                            rules_compound.insert(rule.to_string().as_str(), *b);
                        }
                        TestGameRuleValue::Int(i) => {
                            rules_compound.insert(rule.to_string().as_str(), *i);
                        }
                    }
                }
                compound.insert("rules", NbtTag::Compound(rules_compound));
            }
            Self::TimelineAttributes { timelines } => {
                compound.insert("type", "minecraft:timeline_attributes");
                let strings = timelines
                    .iter()
                    .map(|id| id.to_string().into())
                    .collect::<Vec<_>>();
                compound.insert("timelines", NbtTag::List(NbtList::String(strings)));
            }
            Self::Weather { weather } => {
                compound.insert("type", "minecraft:weather");
                compound.insert("weather", weather.as_str());
            }
        }
        compound
    }

    #[must_use]
    pub fn from_borrowed_compound(compound: simdnbt::borrow::NbtCompound<'_, '_>) -> Option<Self> {
        let raw_type = compound.get("type")?.string()?.to_str();
        let kind = raw_type.strip_prefix("minecraft:").unwrap_or(&raw_type);
        match kind {
            "all_of" => {
                let list = compound.get("definitions")?.list()?;
                let mut definitions = Vec::new();
                if !list.empty() {
                    for child in list.compounds()? {
                        definitions.push(Self::from_borrowed_compound(child)?);
                    }
                }
                Some(Self::AllOf { definitions })
            }
            "clock_time" => {
                let clock =
                    Identifier::from_str(&compound.get("clock")?.string()?.to_str()).ok()?;
                let time = compound.get("time")?.codec_i32()?;
                (time >= 0).then_some(Self::ClockTime { clock, time })
            }
            "difficulty" => {
                let name = compound.get("difficulty")?.string()?.to_str();
                let difficulty = parse_difficulty(&name)?;
                Some(Self::Difficulty { difficulty })
            }
            "function" => {
                let setup = match compound.get("setup") {
                    Some(tag) => Some(Identifier::from_str(&tag.string()?.to_str()).ok()?),
                    None => None,
                };
                let teardown = match compound.get("teardown") {
                    Some(tag) => Some(Identifier::from_str(&tag.string()?.to_str()).ok()?),
                    None => None,
                };
                Some(Self::Function { setup, teardown })
            }
            "game_rules" => {
                let rules_compound = compound.get("rules")?.compound()?;
                let mut rules = Vec::new();
                for (k, v) in rules_compound.iter() {
                    let rule_id = Identifier::from_str(&k.to_str()).ok()?;
                    let value = if let Some(b) = v.byte() {
                        TestGameRuleValue::Bool(b != 0)
                    } else {
                        TestGameRuleValue::Int(v.codec_i32()?)
                    };
                    rules.push((rule_id, value));
                }
                Some(Self::GameRules { rules })
            }
            "timeline_attributes" => {
                let list = compound.get("timelines")?.list()?;
                let mut timelines = Vec::new();
                if !list.empty() {
                    for item in list.strings()? {
                        timelines.push(Identifier::from_str(&item.to_str()).ok()?);
                    }
                }
                Some(Self::TimelineAttributes { timelines })
            }
            "weather" => {
                let name = compound.get("weather")?.string()?.to_str();
                Some(Self::Weather {
                    weather: TestWeather::from_name(&name)?,
                })
            }
            _ => None,
        }
    }
}

const fn difficulty_name(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Peaceful => "peaceful",
        Difficulty::Easy => "easy",
        Difficulty::Normal => "normal",
        Difficulty::Hard => "hard",
    }
}

const fn parse_difficulty(name: &str) -> Option<Difficulty> {
    match name {
        "peaceful" => Some(Difficulty::Peaceful),
        "easy" => Some(Difficulty::Easy),
        "normal" => Some(Difficulty::Normal),
        "hard" => Some(Difficulty::Hard),
        _ => None,
    }
}

impl ToNbtTag for TestEnvironmentDefinition {
    fn to_nbt_tag(self) -> NbtTag {
        NbtTag::Compound(self.to_nbt_compound())
    }
}

impl FromNbtTag for TestEnvironmentDefinition {
    fn from_nbt_tag(tag: simdnbt::borrow::NbtTag<'_, '_>) -> Option<Self> {
        Self::from_borrowed_compound(tag.compound()?)
    }
}

/// Registered `minecraft:test_environment` entry.
#[derive(Debug, Clone)]
pub struct TestEnvironment {
    pub key: Identifier,
    pub definition: TestEnvironmentDefinition,
}

impl ToNbtTag for &TestEnvironment {
    fn to_nbt_tag(self) -> NbtTag {
        NbtTag::Compound(self.definition.to_nbt_compound())
    }
}

pub type TestEnvironmentRef = &'static TestEnvironment;

pub struct TestEnvironmentRegistry {
    environments_by_id: Vec<TestEnvironmentRef>,
    environments_by_key: FxHashMap<Identifier, usize>,
    tags: RegistryTags,
    allows_registering: bool,
}

impl TestEnvironmentRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            environments_by_id: Vec::new(),
            environments_by_key: FxHashMap::default(),
            tags: RegistryTags::default(),
            allows_registering: true,
        }
    }
}

crate::impl_standard_methods!(
    TestEnvironmentRegistry,
    TestEnvironmentRef,
    environments_by_id,
    environments_by_key,
    allows_registering
);

crate::impl_registry!(
    TestEnvironmentRegistry,
    TestEnvironment,
    environments_by_id,
    environments_by_key,
    test_environments
);

crate::impl_tagged_registry!(
    TestEnvironmentRegistry,
    environments_by_key,
    "test_environment"
);

/// Structure rotation inside a `TestInstance` definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TestRotation {
    #[default]
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

impl TestRotation {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Clockwise90 => "clockwise_90",
            Self::Clockwise180 => "180",
            Self::Counterclockwise90 => "counterclockwise_90",
        }
    }

    #[must_use]
    pub const fn from_name(name: &str) -> Option<Self> {
        match name {
            "none" => Some(Self::None),
            "clockwise_90" => Some(Self::Clockwise90),
            "180" => Some(Self::Clockwise180),
            "counterclockwise_90" => Some(Self::Counterclockwise90),
            _ => None,
        }
    }
}

/// Both MC 26.3 `minecraft:test_instance_type` variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestInstanceKind {
    /// `minecraft:block_based` — driven by test blocks inside the template.
    BlockBased,
    /// `minecraft:function` — invokes a registered `BuiltInRegistries.TEST_FUNCTION`.
    Function { function: Identifier },
}

/// Complete registry-independent `TestInstance` definition (`TestData` + kind).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestInstanceValue {
    pub kind: TestInstanceKind,
    pub environment: Identifier,
    /// Dimension the test runs in (`TestData.dimension`, default overworld).
    pub dimension: Identifier,
    pub structure: Identifier,
    pub max_ticks: i32,
    pub setup_ticks: i32,
    pub required: bool,
    pub rotation: TestRotation,
    pub manual_only: bool,
    pub max_attempts: i32,
    pub required_successes: i32,
    pub sky_access: bool,
    /// Empty padding around the structure (`TestData.padding`, `0..=128`).
    pub padding: i32,
}

impl TestInstanceValue {
    #[must_use]
    pub fn to_nbt_compound(&self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        match &self.kind {
            TestInstanceKind::BlockBased => {
                compound.insert("type", "minecraft:block_based");
            }
            TestInstanceKind::Function { function } => {
                compound.insert("type", "minecraft:function");
                compound.insert("function", function.to_string().as_str());
            }
        }
        // Exact `TestData.CODEC` order after `type`/`function`: environment,
        // dimension, structure, max_ticks, setup_ticks, required, rotation,
        // manual_only, max_attempts, required_successes, sky_access, padding.
        // DFU `optionalFieldOf` omits defaulted values on encode.
        compound.insert("environment", self.environment.to_string().as_str());
        if self.dimension != Identifier::vanilla_static("overworld") {
            compound.insert("dimension", self.dimension.to_string().as_str());
        }
        compound.insert("structure", self.structure.to_string().as_str());
        compound.insert("max_ticks", self.max_ticks);
        if self.setup_ticks != 0 {
            compound.insert("setup_ticks", self.setup_ticks);
        }
        if !self.required {
            compound.insert("required", false);
        }
        if self.rotation != TestRotation::None {
            compound.insert("rotation", self.rotation.as_str());
        }
        if self.manual_only {
            compound.insert("manual_only", true);
        }
        if self.max_attempts != 1 {
            compound.insert("max_attempts", self.max_attempts);
        }
        if self.required_successes != 1 {
            compound.insert("required_successes", self.required_successes);
        }
        if self.sky_access {
            compound.insert("sky_access", true);
        }
        if self.padding != 0 {
            compound.insert("padding", self.padding);
        }
        compound
    }
}

impl ToNbtTag for TestInstanceValue {
    fn to_nbt_tag(self) -> NbtTag {
        NbtTag::Compound(self.to_nbt_compound())
    }
}

impl FromNbtTag for TestInstanceValue {
    fn from_nbt_tag(tag: simdnbt::borrow::NbtTag<'_, '_>) -> Option<Self> {
        let compound = tag.compound()?;
        let raw_type = compound.get("type")?.string()?.to_str();
        let kind_name = raw_type.strip_prefix("minecraft:").unwrap_or(&raw_type);
        let kind = match kind_name {
            "block_based" => TestInstanceKind::BlockBased,
            "function" => TestInstanceKind::Function {
                function: Identifier::from_str(&compound.get("function")?.string()?.to_str())
                    .ok()?,
            },
            _ => return None,
        };
        let environment =
            Identifier::from_str(&compound.get("environment")?.string()?.to_str()).ok()?;
        let dimension = match compound.get("dimension") {
            Some(tag) => Identifier::from_str(&tag.string()?.to_str()).ok()?,
            None => Identifier::vanilla_static("overworld"),
        };
        let structure =
            Identifier::from_str(&compound.get("structure")?.string()?.to_str()).ok()?;
        let max_ticks = compound.get("max_ticks")?.codec_i32()?;
        let setup_ticks = match compound.get("setup_ticks") {
            Some(tag) => tag.codec_i32()?,
            None => 0,
        };
        let required = match compound.get("required") {
            Some(tag) => tag.codec_bool()?,
            None => true,
        };
        let rotation = match compound.get("rotation") {
            Some(tag) => TestRotation::from_name(&tag.string()?.to_str())?,
            None => TestRotation::None,
        };
        let manual_only = match compound.get("manual_only") {
            Some(tag) => tag.codec_bool()?,
            None => false,
        };
        let max_attempts = match compound.get("max_attempts") {
            Some(tag) => tag.codec_i32()?,
            None => 1,
        };
        let required_successes = match compound.get("required_successes") {
            Some(tag) => tag.codec_i32()?,
            None => 1,
        };
        let sky_access = match compound.get("sky_access") {
            Some(tag) => tag.codec_bool()?,
            None => false,
        };
        let padding = match compound.get("padding") {
            Some(tag) => tag.codec_i32()?,
            None => 0,
        };
        (max_ticks >= 1
            && setup_ticks >= 0
            && max_attempts >= 1
            && required_successes >= 1
            && (0..=128).contains(&padding))
        .then_some(Self {
            kind,
            environment,
            dimension,
            structure,
            max_ticks,
            setup_ticks,
            required,
            rotation,
            manual_only,
            max_attempts,
            required_successes,
            sky_access,
            padding,
        })
    }
}

/// Registered `minecraft:test_instance` entry.
#[derive(Debug, Clone)]
pub struct TestInstance {
    pub key: Identifier,
    pub value: TestInstanceValue,
}

impl ToNbtTag for &TestInstance {
    fn to_nbt_tag(self) -> NbtTag {
        NbtTag::Compound(self.value.to_nbt_compound())
    }
}

pub type TestInstanceRef = &'static TestInstance;

pub struct TestInstanceRegistry {
    instances_by_id: Vec<TestInstanceRef>,
    instances_by_key: FxHashMap<Identifier, usize>,
    tags: RegistryTags,
    allows_registering: bool,
}

impl TestInstanceRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            instances_by_id: Vec::new(),
            instances_by_key: FxHashMap::default(),
            tags: RegistryTags::default(),
            allows_registering: true,
        }
    }
}

crate::impl_standard_methods!(
    TestInstanceRegistry,
    TestInstanceRef,
    instances_by_id,
    instances_by_key,
    allows_registering
);

crate::impl_registry!(
    TestInstanceRegistry,
    TestInstance,
    instances_by_id,
    instances_by_key,
    test_instances
);

crate::impl_tagged_registry!(TestInstanceRegistry, instances_by_key, "test_instance");
