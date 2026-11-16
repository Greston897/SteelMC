//! Boss mob implementations.

pub mod ender_dragon;
mod wither_boss;

pub use ender_dragon::EnderDragonEntity;
pub use ender_dragon::part::{
    DRAGON_PART_COUNT, DRAGON_PART_HEAD, DRAGON_PART_NECK, DRAGON_PARTS, DragonPart, DragonParts,
};
pub use ender_dragon::phases::{ALL_DRAGON_PHASES, DragonPhase, DragonPhaseManager};
pub use wither_boss::WitherBossEntity;
