//! Vehicle entity implementations.

mod abstract_minecart;
mod boat;
mod chest_boat;
mod chest_minecart;
mod minecart;
mod minecart_behavior;
mod vehicle_entity;

pub use abstract_minecart::{AbstractMinecart, AbstractMinecartBase};
pub(crate) use abstract_minecart::{CreateMinecartError, create_minecart};
pub use boat::{AbstractBoat, BoatEntity, BoatLocation, BoatState, RaftEntity, ride_boat_interact};
pub use chest_boat::{
    CHEST_BOAT_MAX_PASSENGERS, CHEST_BOAT_SLOTS, ChestBoatEntity, ChestRaftEntity,
};
pub use chest_minecart::ChestMinecartEntity;
pub use minecart::MinecartEntity;
pub use minecart_behavior::MinecartBehavior;
pub use vehicle_entity::VehicleEntity;

#[cfg(test)]
#[path = "tests/support.rs"]
mod test_support;
