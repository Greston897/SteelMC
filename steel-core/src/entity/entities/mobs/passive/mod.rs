//! Passive entity implementations.
/// Those mobs are passive creatures that run away when attacked by a player.
mod cat;
mod chicken;
mod cow;
mod fish;
mod fox;
mod mooshroom;
mod ocelot;
mod parrot;
mod pig;
mod rabbit;
mod sheep;
mod squid;

pub use cat::{CatEntity, flags as cat_flags};
pub use chicken::ChickenEntity;
pub use cow::CowEntity;
pub use fish::cod::CodEntity;
pub use fish::pufferfish::PufferfishEntity;
pub use fish::salmon::SalmonEntity;
pub use fish::tadpole::TadpoleEntity;
pub use fish::tropical_fish::TropicalFishEntity;
pub use fox::{FoxEntity, FoxVariant, flags as fox_flags};
pub use mooshroom::MushroomCowEntity;
pub use ocelot::OcelotEntity;
pub use parrot::ParrotEntity;
pub use pig::PigEntity;
pub use rabbit::RabbitEntity;
pub use sheep::SheepEntity;
pub use squid::{GlowSquidEntity, SquidEntity};
