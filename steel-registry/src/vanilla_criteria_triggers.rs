//! Ids of the vanilla advancement criteria triggers.
//!
//! Vanilla registers one `CriterionTrigger` per id in `CriteriaTriggers`; the
//! server fires them by id and every advancement criterion listening for that id
//! gains progress. Steel only needs the ids here — the matching itself is done by
//! `PlayerAdvancement::trigger`.
//!
//! Names match the `trigger` field of the vanilla advancement JSON exactly.

use steel_utils::Identifier;

/// Returns the trigger id for a vanilla criterion trigger registered under
/// `minecraft:<path>`.
const fn trigger(path: &'static str) -> Identifier {
    Identifier::vanilla_static(path)
}

/// `minecraft:allay_drop_item_on_block`
pub const ALLAY_DROP_ITEM_ON_BLOCK: Identifier = trigger("allay_drop_item_on_block");
/// `minecraft:any_block_use`
pub const ANY_BLOCK_USE: Identifier = trigger("any_block_use");
/// `minecraft:avoid_vibration`
pub const AVOID_VIBRATION: Identifier = trigger("avoid_vibration");
/// `minecraft:bee_nest_destroyed`
pub const BEE_NEST_DESTROYED: Identifier = trigger("bee_nest_destroyed");
/// `minecraft:bred_animals`
pub const BRED_ANIMALS: Identifier = trigger("bred_animals");
/// `minecraft:brewed_potion`
pub const BREWED_POTION: Identifier = trigger("brewed_potion");
/// `minecraft:changed_dimension`
pub const CHANGED_DIMENSION: Identifier = trigger("changed_dimension");
/// `minecraft:channeled_lightning`
pub const CHANNELED_LIGHTNING: Identifier = trigger("channeled_lightning");
/// `minecraft:construct_beacon`
pub const CONSTRUCT_BEACON: Identifier = trigger("construct_beacon");
/// `minecraft:consume_item`
pub const CONSUME_ITEM: Identifier = trigger("consume_item");
/// `minecraft:crafter_recipe_crafted`
pub const CRAFTER_RECIPE_CRAFTED: Identifier = trigger("crafter_recipe_crafted");
/// `minecraft:crafting_table_used`
pub const CRAFTING_TABLE_USED: Identifier = trigger("crafting_table_used");
/// `minecraft:cured_zombie_villager`
pub const CURED_ZOMBIE_VILLAGER: Identifier = trigger("cured_zombie_villager");
/// `minecraft:default_block_use`
pub const DEFAULT_BLOCK_USE: Identifier = trigger("default_block_use");
/// `minecraft:destroyed_block`
pub const DESTROYED_BLOCK: Identifier = trigger("destroyed_block");
/// `minecraft:effects_changed`
pub const EFFECTS_CHANGED: Identifier = trigger("effects_changed");
/// `minecraft:enchanted_item`
pub const ENCHANTED_ITEM: Identifier = trigger("enchanted_item");
/// `minecraft:enter_block`
pub const ENTER_BLOCK: Identifier = trigger("enter_block");
/// `minecraft:entity_hurt_player`
pub const ENTITY_HURT_PLAYER: Identifier = trigger("entity_hurt_player");
/// `minecraft:entity_killed_player`
pub const ENTITY_KILLED_PLAYER: Identifier = trigger("entity_killed_player");
/// `minecraft:fall_after_explosion`
pub const FALL_AFTER_EXPLOSION: Identifier = trigger("fall_after_explosion");
/// `minecraft:fall_from_height`
pub const FALL_FROM_HEIGHT: Identifier = trigger("fall_from_height");
/// `minecraft:filled_bucket`
pub const FILLED_BUCKET: Identifier = trigger("filled_bucket");
/// `minecraft:fishing_rod_hooked`
pub const FISHING_ROD_HOOKED: Identifier = trigger("fishing_rod_hooked");
/// `minecraft:hero_of_the_village`
pub const HERO_OF_THE_VILLAGE: Identifier = trigger("hero_of_the_village");
/// `minecraft:honey_block_slide`
pub const HONEY_BLOCK_SLIDE: Identifier = trigger("honey_block_slide");
/// `minecraft:inventory_changed`
pub const INVENTORY_CHANGED: Identifier = trigger("inventory_changed");
/// `minecraft:item_delivered_to_player`
pub const ITEM_DELIVERED_TO_PLAYER: Identifier = trigger("item_delivered_to_player");
/// `minecraft:item_durability_changed`
pub const ITEM_DURABILITY_CHANGED: Identifier = trigger("item_durability_changed");
/// `minecraft:item_used_on_block`
pub const ITEM_USED_ON_BLOCK: Identifier = trigger("item_used_on_block");
/// `minecraft:kill_mob_near_sculk_catalyst`
pub const KILL_MOB_NEAR_SCULK_CATALYST: Identifier = trigger("kill_mob_near_sculk_catalyst");
/// `minecraft:killed_by_crossbow`
pub const KILLED_BY_CROSSBOW: Identifier = trigger("killed_by_crossbow");
/// `minecraft:levitation`
pub const LEVITATION: Identifier = trigger("levitation");
/// `minecraft:lightning_strike`
pub const LIGHTNING_STRIKE: Identifier = trigger("lightning_strike");
/// `minecraft:location`
pub const LOCATION: Identifier = trigger("location");
/// `minecraft:nether_travel`
pub const NETHER_TRAVEL: Identifier = trigger("nether_travel");
/// `minecraft:placed_block`
pub const PLACED_BLOCK: Identifier = trigger("placed_block");
/// `minecraft:player_generates_container_loot`
pub const PLAYER_GENERATES_CONTAINER_LOOT: Identifier = trigger("player_generates_container_loot");
/// `minecraft:player_hurt_entity`
pub const PLAYER_HURT_ENTITY: Identifier = trigger("player_hurt_entity");
/// `minecraft:player_interacted_with_entity`
pub const PLAYER_INTERACTED_WITH_ENTITY: Identifier = trigger("player_interacted_with_entity");
/// `minecraft:player_killed_entity`
pub const PLAYER_KILLED_ENTITY: Identifier = trigger("player_killed_entity");
/// `minecraft:player_sheared_equipment`
pub const PLAYER_SHEARED_EQUIPMENT: Identifier = trigger("player_sheared_equipment");
/// `minecraft:recipe_crafted`
pub const RECIPE_CRAFTED: Identifier = trigger("recipe_crafted");
/// `minecraft:recipe_unlocked`
pub const RECIPE_UNLOCKED: Identifier = trigger("recipe_unlocked");
/// `minecraft:ride_entity_in_lava`
pub const RIDE_ENTITY_IN_LAVA: Identifier = trigger("ride_entity_in_lava");
/// `minecraft:shot_crossbow`
pub const SHOT_CROSSBOW: Identifier = trigger("shot_crossbow");
/// `minecraft:slept_in_bed`
pub const SLEPT_IN_BED: Identifier = trigger("slept_in_bed");
/// `minecraft:slide_down_block`
pub const SLIDE_DOWN_BLOCK: Identifier = trigger("slide_down_block");
/// `minecraft:started_riding`
pub const STARTED_RIDING: Identifier = trigger("started_riding");
/// `minecraft:summoned_entity`
pub const SUMMONED_ENTITY: Identifier = trigger("summoned_entity");
/// `minecraft:tame_animal`
pub const TAME_ANIMAL: Identifier = trigger("tame_animal");
/// `minecraft:target_hit`
pub const TARGET_HIT: Identifier = trigger("target_hit");
/// `minecraft:thrown_item_picked_up_by_entity`
pub const THROWN_ITEM_PICKED_UP_BY_ENTITY: Identifier = trigger("thrown_item_picked_up_by_entity");
/// `minecraft:thrown_item_picked_up_by_player`
pub const THROWN_ITEM_PICKED_UP_BY_PLAYER: Identifier =
    trigger("thrown_item_picked_up_by_player");
/// `minecraft:tick`
pub const TICK: Identifier = trigger("tick");
/// `minecraft:trade`
pub const TRADE: Identifier = trigger("trade");
/// `minecraft:travel_through_dimension`
pub const TRAVEL_THROUGH_DIMENSION: Identifier = trigger("travel_through_dimension");
/// `minecraft:used_ender_eye`
pub const USED_ENDER_EYE: Identifier = trigger("used_ender_eye");
/// `minecraft:used_totem`
pub const USED_TOTEM: Identifier = trigger("used_totem");
/// `minecraft:using_item`
pub const USING_ITEM: Identifier = trigger("using_item");
/// `minecraft:villager_trade`
pub const VILLAGER_TRADE: Identifier = trigger("villager_trade");
/// `minecraft:voluntary_exile`
pub const VOLUNTARY_EXILE: Identifier = trigger("voluntary_exile");

/// Every vanilla trigger id, used by tests that cross-check the list above.
pub static ALL: &[&Identifier] = &[
    &ALLAY_DROP_ITEM_ON_BLOCK,
    &ANY_BLOCK_USE,
    &AVOID_VIBRATION,
    &BEE_NEST_DESTROYED,
    &BRED_ANIMALS,
    &BREWED_POTION,
    &CHANGED_DIMENSION,
    &CHANNELED_LIGHTNING,
    &CONSTRUCT_BEACON,
    &CONSUME_ITEM,
    &CRAFTER_RECIPE_CRAFTED,
    &CRAFTING_TABLE_USED,
    &CURED_ZOMBIE_VILLAGER,
    &DEFAULT_BLOCK_USE,
    &DESTROYED_BLOCK,
    &EFFECTS_CHANGED,
    &ENCHANTED_ITEM,
    &ENTER_BLOCK,
    &ENTITY_HURT_PLAYER,
    &ENTITY_KILLED_PLAYER,
    &FALL_AFTER_EXPLOSION,
    &FALL_FROM_HEIGHT,
    &FILLED_BUCKET,
    &FISHING_ROD_HOOKED,
    &HERO_OF_THE_VILLAGE,
    &HONEY_BLOCK_SLIDE,
    &INVENTORY_CHANGED,
    &ITEM_DELIVERED_TO_PLAYER,
    &ITEM_DURABILITY_CHANGED,
    &ITEM_USED_ON_BLOCK,
    &KILL_MOB_NEAR_SCULK_CATALYST,
    &KILLED_BY_CROSSBOW,
    &LEVITATION,
    &LIGHTNING_STRIKE,
    &LOCATION,
    &NETHER_TRAVEL,
    &PLACED_BLOCK,
    &PLAYER_GENERATES_CONTAINER_LOOT,
    &PLAYER_HURT_ENTITY,
    &PLAYER_INTERACTED_WITH_ENTITY,
    &PLAYER_KILLED_ENTITY,
    &PLAYER_SHEARED_EQUIPMENT,
    &RECIPE_CRAFTED,
    &RECIPE_UNLOCKED,
    &RIDE_ENTITY_IN_LAVA,
    &SHOT_CROSSBOW,
    &SLEPT_IN_BED,
    &SLIDE_DOWN_BLOCK,
    &STARTED_RIDING,
    &SUMMONED_ENTITY,
    &TAME_ANIMAL,
    &TARGET_HIT,
    &THROWN_ITEM_PICKED_UP_BY_ENTITY,
    &THROWN_ITEM_PICKED_UP_BY_PLAYER,
    &TICK,
    &TRADE,
    &TRAVEL_THROUGH_DIMENSION,
    &USED_ENDER_EYE,
    &USED_TOTEM,
    &USING_ITEM,
    &VILLAGER_TRADE,
    &VOLUNTARY_EXILE,
];
