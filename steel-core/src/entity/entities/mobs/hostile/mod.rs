//! Hostile entity implementations.
//!
//! Those mobs are aggressive creatures that attack players on sight.

/// Shared helper functions for the slime family.
mod cube_mob;

/// Hostile breeze entity module.
pub mod breeze;
/// Copper golem entity module.
pub mod copper_golem;
/// Hostile creaking entity module.
pub mod creaking;
/// Hostile guardian entity module.
pub mod guardian;
/// Hostile phantom entity module.
pub mod phantom;
/// Hostile shulker entity module.
pub mod shulker;
/// Snow golem entity module.
pub mod snow_golem;
/// Hostile warden entity module.
pub mod warden;
/// Hostile elder guardian entity module.
pub mod elder_guardian;
/// Hostile blaze entity module.
pub mod blaze;
/// Hostile evoker entity module.
pub mod evoker;
/// Hostile ghast entity module.
pub mod ghast;
/// Hostile giant entity module.
pub mod giant;
/// Hostile illusioner entity module.
pub mod illusioner;
/// Hostile ravager entity module.
pub mod ravager;
/// Hostile vex entity module.
pub mod vex;
/// Hostile witch entity module.
pub mod witch;
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
/// Hostile skeleton entity module.
pub mod skeleton;
/// Hostile slime entity module.
pub mod slime;
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
pub use skeleton::SkeletonEntity;
pub use slime::SlimeEntity;
pub use spider::SpiderEntity;
pub use stray::StrayEntity;
pub use vindicator::VindicatorEntity;
pub use wither_skeleton::WitherSkeletonEntity;
pub use zoglin::ZoglinEntity;
pub use zombie::ZombieEntity;
pub use zombified_piglin::ZombifiedPiglinEntity;
