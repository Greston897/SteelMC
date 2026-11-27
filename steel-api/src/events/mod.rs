//! Все ваниль-события — по категориям. Покрыто ~80 событий = 100% механик.

pub mod block;
pub mod entity;
pub mod inventory;
pub mod player;
pub mod server;
pub mod world;

pub use block::*;
pub use entity::*;
pub use inventory::*;
pub use player::*;
pub use server::*;
pub use world::*;
