//! Hostile entity implementations.
//!
//! Those mobs are aggressive creatures that attack players on sight.

/// Hostile cave spider entity module.
pub mod cave_spider;
/// Hostile drowned entity module.
pub mod drowned;
/// Hostile endermite entity module.
pub mod endermite;
/// Hostile husk entity module.
pub mod husk;
/// Hostile silverfish entity module.
pub mod silverfish;
/// Hostile vindicator entity module.
pub mod vindicator;
/// Hostile wither skeleton entity module.
pub mod wither_skeleton;
/// Hostile zombie entity module.
pub mod zombie;
/// Hostile zombified piglin entity module.
pub mod zombified_piglin;

pub use cave_spider::CaveSpiderEntity;
pub use drowned::DrownedEntity;
pub use endermite::EndermiteEntity;
pub use husk::HuskEntity;
pub use silverfish::SilverfishEntity;
pub use vindicator::VindicatorEntity;
pub use wither_skeleton::WitherSkeletonEntity;
pub use zombie::ZombieEntity;
pub use zombified_piglin::ZombifiedPiglinEntity;
