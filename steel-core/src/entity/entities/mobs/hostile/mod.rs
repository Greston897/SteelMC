//! Hostile entity implementations.
//!
//! Those mobs are aggressive creatures that attack players on sight.

/// Shared helper functions for the slime family.
mod cube_mob;

/// Hostile blaze entity module.
pub mod blaze;
/// Hostile bogged entity module.
pub mod bogged;
/// Hostile breeze entity module.
pub mod breeze;
/// Hostile cave spider entity module.
pub mod cave_spider;
/// Copper golem entity module.
pub mod copper_golem;
/// Hostile creaking entity module.
pub mod creaking;
/// Hostile creeper entity module.
pub mod creeper;
/// Hostile drowned entity module.
pub mod drowned;
/// Hostile elder guardian entity module.
pub mod elder_guardian;
/// Hostile enderman entity module.
pub mod enderman;
/// Hostile endermite entity module.
pub mod endermite;
/// Hostile evoker entity module.
pub mod evoker;
/// Hostile ghast entity module.
pub mod ghast;
/// Hostile giant entity module.
pub mod giant;
/// Hostile guardian entity module.
pub mod guardian;
/// Hostile hoglin entity module.
pub mod hoglin;
/// Hostile husk entity module.
pub mod husk;
/// Hostile illusioner entity module.
pub mod illusioner;
/// Iron golem entity module.
pub mod iron_golem;
/// Hostile magma cube entity module.
pub mod magma_cube;
/// Hostile parched entity module.
pub mod parched;
/// Hostile phantom entity module.
pub mod phantom;
/// Hostile piglin entity module.
pub mod piglin;
/// Hostile piglin brute entity module.
pub mod piglin_brute;
/// Hostile pillager entity module.
pub mod pillager;
/// Hostile ravager entity module.
pub mod ravager;
/// Hostile shulker entity module.
pub mod shulker;
/// Hostile silverfish entity module.
pub mod silverfish;
/// Hostile skeleton entity module.
pub mod skeleton;
/// Hostile slime entity module.
pub mod slime;
/// Snow golem entity module.
pub mod snow_golem;
/// Hostile spider entity module.
pub mod spider;
/// Hostile stray entity module.
pub mod stray;
/// Hostile vex entity module.
pub mod vex;
/// Hostile vindicator entity module.
pub mod vindicator;
/// Hostile warden entity module.
pub mod warden;
/// Hostile witch entity module.
pub mod witch;
/// Hostile wither skeleton entity module.
pub mod wither_skeleton;
/// Hostile zoglin entity module.
pub mod zoglin;
/// Hostile zombie entity module.
pub mod zombie;
/// Hostile zombie villager entity module.
pub mod zombie_villager;
/// Hostile zombified piglin entity module.
pub mod zombified_piglin;

pub use blaze::BlazeEntity;
pub use bogged::BoggedEntity;
pub use breeze::BreezeEntity;
pub use cave_spider::CaveSpiderEntity;
pub use copper_golem::CopperGolemEntity;
pub use creaking::CreakingEntity;
pub use creeper::CreeperEntity;
pub use drowned::DrownedEntity;
pub use elder_guardian::ElderGuardianEntity;
pub use enderman::EndermanEntity;
pub use endermite::EndermiteEntity;
pub use evoker::EvokerEntity;
pub use ghast::GhastEntity;
pub use giant::GiantEntity;
pub use guardian::GuardianEntity;
pub use hoglin::HoglinEntity;
pub use husk::HuskEntity;
pub use illusioner::IllusionerEntity;
pub use iron_golem::IronGolemEntity;
pub use magma_cube::MagmaCubeEntity;
pub use parched::ParchedEntity;
pub use phantom::PhantomEntity;
pub use piglin::PiglinEntity;
pub use piglin_brute::PiglinBruteEntity;
pub use pillager::PillagerEntity;
pub use ravager::RavagerEntity;
pub use shulker::ShulkerEntity;
pub use silverfish::SilverfishEntity;
pub use skeleton::SkeletonEntity;
pub use slime::SlimeEntity;
pub use snow_golem::SnowGolemEntity;
pub use spider::SpiderEntity;
pub use stray::StrayEntity;
pub use vex::VexEntity;
pub use vindicator::VindicatorEntity;
pub use warden::WardenEntity;
pub use witch::WitchEntity;
pub use wither_skeleton::WitherSkeletonEntity;
pub use zoglin::ZoglinEntity;
pub use zombie::ZombieEntity;
pub use zombie_villager::ZombieVillagerEntity;
pub use zombified_piglin::ZombifiedPiglinEntity;
