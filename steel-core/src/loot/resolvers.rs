//! World-backed implementations of the loot-table resolver hooks.
//!
//! The scoreboard and command storage both live on `Server`, which sits above
//! `steel-registry`, so the registry exposes traits and the game layer installs
//! these adapters when it builds a loot context.

use std::sync::Arc;

use simdnbt::owned::NbtTag;
use steel_registry::loot_table::{ScoreResolver, StorageResolver};
use steel_utils::Identifier;

use crate::scoreboard::ScoreHolder;
use crate::server::Server;

/// Resolves `EntityScores`/`Score` values from the domain scoreboard.
pub struct WorldScoreResolver<'a> {
    server: &'a Arc<Server>,
    domain: &'a str,
}

impl<'a> WorldScoreResolver<'a> {
    /// Creates a resolver bound to one world domain.
    #[must_use]
    pub const fn new(server: &'a Arc<Server>, domain: &'a str) -> Self {
        Self { server, domain }
    }
}

impl ScoreResolver for WorldScoreResolver<'_> {
    fn score(&self, holder: &str, objective: &str) -> Option<i32> {
        let scoreboard = self.server.scoreboards.get(self.domain)?;
        let objective = scoreboard.objective(objective)?;
        scoreboard.score(&ScoreHolder::new(holder), &objective)
    }
}

/// Resolves `minecraft:storage` number providers from `/data` command storage.
pub struct WorldStorageResolver<'a> {
    server: &'a Arc<Server>,
    domain: &'a str,
}

impl<'a> WorldStorageResolver<'a> {
    /// Creates a resolver bound to one world domain.
    #[must_use]
    pub const fn new(server: &'a Arc<Server>, domain: &'a str) -> Self {
        Self { server, domain }
    }
}

impl StorageResolver for WorldStorageResolver<'_> {
    fn storage_number(&self, storage: &Identifier, path: &str) -> Option<f64> {
        let contents = self.server.command_storage.get(self.domain)?.get(storage);
        // Vanilla evaluates the provider path as a full NBT path, so list
        // indices and filters behave like `/data get`.
        let parsed = steel_utils::nbt::parse_nbt_path(path).ok()?;
        let tag = NbtTag::Compound(contents.clone());
        // Vanilla only accepts a single numeric result; a path matching several
        // tags or none yields zero.
        match parsed.get(&tag).as_slice() {
            [value] => numeric_value(value),
            _ => None,
        }
    }
}

/// Vanilla `NbtTag#getAsDouble`-style widening: every numeric tag counts, and
/// anything else makes the provider yield zero.
fn numeric_value(tag: &NbtTag) -> Option<f64> {
    Some(match tag {
        NbtTag::Double(value) => *value,
        NbtTag::Float(value) => f64::from(*value),
        NbtTag::Int(value) => f64::from(*value),
        NbtTag::Short(value) => f64::from(*value),
        NbtTag::Byte(value) => f64::from(*value),
        NbtTag::Long(value) => {
            // Vanilla widens longs through f64, losing precision above 2^53.
            #[expect(
                clippy::cast_precision_loss,
                reason = "vanilla widens long NBT values to double for storage providers"
            )]
            {
                *value as f64
            }
        }
        _ => return None,
    })
}
