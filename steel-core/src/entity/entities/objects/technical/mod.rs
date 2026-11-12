//! Technical entity implementations.

mod interaction;
mod marker;
mod ominous_item_spawner;

pub use interaction::{InteractionEntity, InteractionEntityDataView, PlayerAction};
pub use marker::MarkerEntity;
pub use ominous_item_spawner::OminousItemSpawnerEntity;
