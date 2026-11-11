//! Hostile entity implementations.
//!
//! Those mobs are aggressive creatures that attack players on sight.

/// Shared helper functions for the slime family.
mod cube_mob;

/// Hostile bogged entity module.
pub mod bogged;
/// Hostile cave spider entity module.
pub mod cave_spider;
/// Hostile creeper entity module.
pub mod creeper;
/// Hostile drowned entity module.
pub mod drowned;
/// Hostile enderman entity module.
pub mod enderman;
/// Hostile endermite entity module.
pub mod endermite;
/// Hostile hoglin entity module.
pub mod hoglin;
/// Hostile husk entity module.
pub mod husk;
/// Hostile magma cube entity module.
pub mod magma_cube;
/// Hostile parched entity module.
pub mod parched;
/// Hostile piglin entity module.
pub mod piglin;
/// Hostile piglin brute entity module.
pub mod piglin_brute;
/// Hostile pillager entity module.
pub mod pillager;
/// Hostile silverfish entity module.
pub mod silverfish;
/// Hostile slime entity module.
pub mod slime;
/// Hostile skeleton entity module.
pub mod skeleton;
/// Hostile spider entity module.
pub mod spider;
/// Hostile stray entity module.
pub mod stray;
/// Hostile vindicator entity module.
pub mod vindicator;
/// Hostile wither skeleton entity module.
pub mod wither_skeleton;
/// Hostile zoglin entity module.
pub mod zoglin;
/// Hostile zombie entity module.
pub mod zombie;
/// Hostile zombified piglin entity module.
pub mod zombified_piglin;

pub use bogged::BoggedEntity;
pub use cave_spider::CaveSpiderEntity;
pub use creeper::CreeperEntity;
pub use drowned::DrownedEntity;
pub use enderman::EndermanEntity;
pub use endermite::EndermiteEntity;
pub use hoglin::HoglinEntity;
pub use husk::HuskEntity;
pub use magma_cube::MagmaCubeEntity;
pub use parched::ParchedEntity;
pub use piglin::PiglinEntity;
pub use piglin_brute::PiglinBruteEntity;
pub use pillager::PillagerEntity;
pub use silverfish::SilverfishEntity;
pub use slime::SlimeEntity;
pub use skeleton::SkeletonEntity;
pub use spider::SpiderEntity;
pub use stray::StrayEntity;
pub use vindicator::VindicatorEntity;
pub use wither_skeleton::WitherSkeletonEntity;
pub use zoglin::ZoglinEntity;
pub use zombie::ZombieEntity;
pub use zombified_piglin::ZombifiedPiglinEntity;
