//! Projectile entity implementations.

mod arrow;
mod ender_pearl;
mod experience_bottle;
mod eye_of_ender;
mod firework_rocket;
mod fishing_hook;
mod lingering_potion;
mod llama_spit;
mod snowball;
mod shulker_bullet;
mod splash_potion;
mod thrown_egg;
mod wind_charge;
mod wither_skull;

pub use arrow::{
    AbstractArrow, ArrowEntity, ArrowHandle, ArrowPickup, SpectralArrowEntity, TRIDENT_DAMAGE,
    ThrownTridentEntity,
};
pub use ender_pearl::EnderPearlEntity;
pub use experience_bottle::ExperienceBottleEntity;
pub use eye_of_ender::EyeOfEnderEntity;
pub use firework_rocket::FireworkRocketEntity;
pub use fishing_hook::{FishingHookEntity, FishingHookState};
pub use lingering_potion::LingeringPotionEntity;
pub use llama_spit::LlamaSpitEntity;
pub use snowball::SnowballEntity;
pub use shulker_bullet::ShulkerBulletEntity;
pub use splash_potion::SplashPotionEntity;
pub use thrown_egg::ThrownEggEntity;
pub use wind_charge::{EXPLOSION_POWER, WindChargeEntity};
pub use wither_skull::WitherSkullEntity;
