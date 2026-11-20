//! Vanilla MC 26.3 `minecraft:trial_spawner` registry (`28/28`).
//!
//! Mirrors `net.minecraft.world.level.block.entity.trialspawner.TrialSpawnerConfig`.

use rustc_hash::FxHashMap;
use simdnbt::ToNbtTag;
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use steel_utils::Identifier;

/// Spawn equipment table (`SpawnData.equipment`: loot table + drop chances).
#[derive(Debug, Clone, PartialEq)]
pub struct TrialSpawnEquipment {
    pub loot_table: Identifier,
    pub slot_drop_chances: f32,
}

/// Extra primitive tag inside `SpawnData.entity` beyond the `id`
/// (vanilla trial data uses `IsBaby` and `Size`).
#[derive(Debug, Clone, PartialEq)]
pub enum TrialEntityTagValue {
    Byte(bool),
    Int(i32),
    Float(f32),
    Text(String),
}

/// Weighted mob spawn entry inside `TrialSpawnerConfig.spawn_potentials`.
#[derive(Debug, Clone, PartialEq)]
pub struct TrialSpawnPotential {
    pub entity_id: Identifier,
    pub weight: i32,
    pub equipment: Option<TrialSpawnEquipment>,
    /// Extra `entity` compound entries (`IsBaby`, `Size`, ...).
    pub extra_entity_tags: Vec<(String, TrialEntityTagValue)>,
}

impl TrialSpawnPotential {
    /// Whether the spawned entity is a baby (`IsBaby: 1b`).
    #[must_use]
    pub fn is_baby(&self) -> bool {
        self.extra_entity_tags
            .iter()
            .any(|(key, value)| key == "IsBaby" && matches!(value, TrialEntityTagValue::Byte(true)))
    }

    /// Slime/magma-cube `Size`, when the spawn data sets one.
    #[must_use]
    pub fn size(&self) -> Option<i32> {
        self.extra_entity_tags.iter().find_map(|(key, value)| {
            if key == "Size"
                && let TrialEntityTagValue::Int(size) = value
            {
                return Some(*size);
            }
            None
        })
    }
}

/// Weighted loot table entry inside `TrialSpawnerConfig.loot_tables_to_eject`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrialWeightedLootTable {
    pub loot_table: Identifier,
    pub weight: i32,
}

/// Registered `minecraft:trial_spawner` configuration.
#[derive(Debug, Clone)]
pub struct TrialSpawnerConfig {
    pub key: Identifier,
    pub spawn_range: i32,
    pub total_mobs: f32,
    pub simultaneous_mobs: f32,
    pub total_mobs_added_per_player: f32,
    pub simultaneous_mobs_added_per_player: f32,
    pub ticks_between_spawn: i32,
    pub spawn_potentials: Vec<TrialSpawnPotential>,
    pub loot_tables_to_eject: Vec<TrialWeightedLootTable>,
    pub items_to_drop_when_ominous: Identifier,
}

impl TrialSpawnerConfig {
    /// Vanilla `TrialSpawnerConfig.calculateTargetTotalMobs(additionalPlayers)`.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        reason = "matches vanilla Mth.floor(totalMobs + totalMobsAddedPerPlayer * additionalPlayers)"
    )]
    pub fn calculate_target_total_mobs(&self, additional_players: usize) -> i32 {
        (self.total_mobs + self.total_mobs_added_per_player * (additional_players as f32)).floor()
            as i32
    }

    /// Vanilla `TrialSpawnerConfig.calculateTargetSimultaneousMobs(additionalPlayers)`.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        reason = "matches vanilla Mth.floor(simultaneousMobs + simultaneousMobsAddedPerPlayer * additionalPlayers)"
    )]
    pub fn calculate_target_simultaneous_mobs(&self, additional_players: usize) -> i32 {
        (self.simultaneous_mobs
            + self.simultaneous_mobs_added_per_player * (additional_players as f32))
            .floor() as i32
    }
}

impl ToNbtTag for &TrialSpawnerConfig {
    fn to_nbt_tag(self) -> NbtTag {
        let mut compound = NbtCompound::new();
        compound.insert("spawn_range", self.spawn_range);
        compound.insert("total_mobs", self.total_mobs);
        compound.insert("simultaneous_mobs", self.simultaneous_mobs);
        compound.insert(
            "total_mobs_added_per_player",
            self.total_mobs_added_per_player,
        );
        compound.insert(
            "simultaneous_mobs_added_per_player",
            self.simultaneous_mobs_added_per_player,
        );
        compound.insert("ticks_between_spawn", self.ticks_between_spawn);
        let potentials: Vec<NbtCompound> = self
            .spawn_potentials
            .iter()
            .map(|p| {
                let mut entity = NbtCompound::new();
                entity.insert("id", p.entity_id.to_string().as_str());
                for (key, value) in &p.extra_entity_tags {
                    match value {
                        TrialEntityTagValue::Byte(flag) => {
                            entity.insert(key.as_str(), *flag);
                        }
                        TrialEntityTagValue::Int(v) => {
                            entity.insert(key.as_str(), *v);
                        }
                        TrialEntityTagValue::Float(v) => {
                            entity.insert(key.as_str(), *v);
                        }
                        TrialEntityTagValue::Text(v) => {
                            entity.insert(key.as_str(), v.as_str());
                        }
                    }
                }
                let mut data = NbtCompound::new();
                data.insert("entity", NbtTag::Compound(entity));
                if let Some(eq) = &p.equipment {
                    let mut equipment = NbtCompound::new();
                    equipment.insert("loot_table", eq.loot_table.to_string().as_str());
                    equipment.insert("slot_drop_chances", eq.slot_drop_chances);
                    data.insert("equipment", NbtTag::Compound(equipment));
                }
                let mut entry = NbtCompound::new();
                entry.insert("data", NbtTag::Compound(data));
                entry.insert("weight", p.weight);
                entry
            })
            .collect();
        compound.insert(
            "spawn_potentials",
            NbtTag::List(NbtList::Compound(potentials)),
        );
        let loot: Vec<NbtCompound> = self
            .loot_tables_to_eject
            .iter()
            .map(|l| {
                let mut e = NbtCompound::new();
                e.insert("data", l.loot_table.to_string().as_str());
                e.insert("weight", l.weight);
                e
            })
            .collect();
        compound.insert(
            "loot_tables_to_eject",
            NbtTag::List(NbtList::Compound(loot)),
        );
        compound.insert(
            "items_to_drop_when_ominous",
            self.items_to_drop_when_ominous.to_string().as_str(),
        );
        NbtTag::Compound(compound)
    }
}

pub type TrialSpawnerConfigRef = &'static TrialSpawnerConfig;

pub struct TrialSpawnerConfigRegistry {
    configs_by_id: Vec<TrialSpawnerConfigRef>,
    configs_by_key: FxHashMap<Identifier, usize>,
    allows_registering: bool,
}

impl TrialSpawnerConfigRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            configs_by_id: Vec::new(),
            configs_by_key: FxHashMap::default(),
            allows_registering: true,
        }
    }
}

crate::impl_standard_methods!(
    TrialSpawnerConfigRegistry,
    TrialSpawnerConfigRef,
    configs_by_id,
    configs_by_key,
    allows_registering
);

crate::impl_registry!(
    TrialSpawnerConfigRegistry,
    TrialSpawnerConfig,
    configs_by_id,
    configs_by_key,
    trial_spawners
);
