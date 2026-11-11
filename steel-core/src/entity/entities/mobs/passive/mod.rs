//! Passive entity implementations.
/// Those mobs are passive creatures that run away when attacked by a player.
mod chicken;
mod cow;
mod fish;
mod mooshroom;
mod pig;
mod rabbit;
mod sheep;
mod squid;

pub use chicken::ChickenEntity;
pub use cow::CowEntity;
pub use fish::cod::CodEntity;
pub use fish::pufferfish::PufferfishEntity;
pub use fish::salmon::SalmonEntity;
pub use fish::tropical_fish::TropicalFishEntity;
pub use mooshroom::MushroomCowEntity;
pub use pig::PigEntity;
pub use rabbit::RabbitEntity;
pub use sheep::SheepEntity;
pub use squid::SquidEntity;
