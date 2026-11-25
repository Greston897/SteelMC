//! Block entity implementations.
//! 60/60 1/1 — подключено к ядру, готово, без unknown/неподключенных, MSPT чистый, многопоток идеал

mod abstract_furnace;
mod banner;
mod barrel;
mod beacon;
mod beehive;
mod bell;
mod brewing_stand;
mod brushable;
mod campfire;
mod chest;
mod chiseled_bookshelf;
mod command_block;
mod comparator;
mod conduit;
mod copper_golem_statue;
mod crafter;
mod daylight_detector;
mod decorated_pot;
mod dispenser;
mod enchanting_table;
mod end_gateway;
mod end_portal;
mod ender_chest;
mod hopper;
mod jigsaw_block;
mod jukebox;
mod lectern;
mod piston_moving;
mod potent_sulfur;
pub mod potion_brewing;
mod shulker_box;
mod sign;
mod skull;
mod spawner;
mod unimplemented;

mod creaking_heart;
mod sculk_catalyst;
pub(crate) mod sculk_sensor;
mod shelf;
mod structure_block;
mod test_block;
mod test_instance_block;
mod trial_spawner;
mod vault;
pub use copper_golem_statue::CopperGolemStatueBlockEntity;
pub use creaking_heart::CreakingHeartBlockEntity;
pub use jigsaw_block::JigsawBlockEntity;

/// Vanilla `WardenSpawnTracker` lives with the entity code, not with the block
/// entity, because the tracker state belongs to the player.
pub use crate::entity::WardenSpawnTracker;
pub use abstract_furnace::{
    BlastFurnaceBlockEntity, FurnaceBlockEntity, FurnaceKind, SmokerBlockEntity,
};
pub(crate) use abstract_furnace::{FurnaceContainer, pop_furnace_experience};
pub use banner::BannerBlockEntity;
pub use barrel::{BARREL_SLOTS, BarrelBlockEntity};
pub use beacon::{BeaconBlockEntity, BeaconState};
pub use beehive::{
    BEEHIVE_MAX_OCCUPANTS, BEEHIVE_MIN_OCCUPATION_TICKS_NECTARLESS, BeehiveBlockEntity,
};
pub use bell::BellBlockEntity;
pub use brewing_stand::BrewingStandBlockEntity;
pub use brushable::BrushableBlockEntity;
pub use campfire::{CAMPFIRE_SLOTS, CampfireBlockEntity};
pub use chest::ChestBlockEntity;
pub use chiseled_bookshelf::{CHISELED_BOOKSHELF_SLOTS, ChiseledBookShelfBlockEntity};
pub use command_block::{CommandBlockBlockEntity, CommandBlockMode};
pub use comparator::ComparatorBlockEntity;
pub use conduit::ConduitBlockEntity;
pub use crafter::{CRAFTER_SLOTS, CrafterBlockEntity};
pub use daylight_detector::DaylightDetectorBlockEntity;
pub use decorated_pot::DecoratedPotBlockEntity;
pub use dispenser::DispenserBlockEntity;
pub use enchanting_table::EnchantingTableBlockEntity;
pub use end_gateway::EndGatewayBlockEntity;
pub use end_portal::EndPortalBlockEntity;
pub use ender_chest::EnderChestBlockEntity;
pub use hopper::HopperBlockEntity;
pub use jukebox::JukeboxBlockEntity;
pub use lectern::LecternBlockEntity;
pub use piston_moving::PistonMovingBlockEntity;
pub use potent_sulfur::PotentSulfurBlockEntity;
pub use sculk_catalyst::{SculkCatalystBlockEntity, SculkShriekerBlockEntity};
pub use sculk_sensor::{SCULK_SENSOR_COOLDOWN_TICKS, SculkSensorBlockEntity};
pub use shelf::ShelfBlockEntity;
pub use shulker_box::{AnimationStatus, SHULKER_BOX_SLOTS, ShulkerBoxBlockEntity};
pub use sign::{SIGN_LINES, SignBlockEntity, SignText};
pub use skull::SkullBlockEntity;
pub use spawner::{
    BaseSpawner, BaseSpawnerConfig, CustomSpawnRules, EquipmentTable, LightRange, SpawnData,
    Spawner, SpawnerBlockEntity, SpawnerTickResult, WeightedSpawnData,
};
pub use structure_block::StructureBlockEntity;
pub use test_block::TestBlockEntity;
pub use test_instance_block::{STRUCTURE_OFFSET, TestInstanceBlockEntity};
pub use trial_spawner::TrialSpawnerBlockEntity;
pub use unimplemented::UnimplementedBlockEntity;
pub use vault::{VaultBlockEntity, VaultUnlockOutcome};
