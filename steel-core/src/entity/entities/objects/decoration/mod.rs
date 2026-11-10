//! Decoration entity implementations.

mod armor_stand;
mod cushion;
mod painting;

pub use armor_stand::ArmorStandEntity;
pub use cushion::{CushionEntity, cushion_item_for_color};
pub use painting::PaintingEntity;
