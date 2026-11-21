//! Test player implementation for `SteelMC`.
//!
//! Wraps a real `steel_core::player::Player` with a `FlintConnection` so player
//! interactions (like `use_item_on`) run without a network.

use std::any::Any;
use std::sync::Arc;

use flint_core::test_spec::{GameMode, PlayerSlot};
use flint_core::{FlintPlayer, Item};
use glam::DVec3;
use rustc_hash::FxHashMap;
use steel_core::entity::{Entity, LivingEntity};
use steel_core::inventory::container::Container;
use steel_core::player::game_mode;
use steel_core::player::player_inventory::PlayerInventory;
use steel_core::player::{
    ClientInformation, GameProfile, Player, PlayerConnection, PlayerSession, offline_uuid,
};
use steel_core::server::Server;
use steel_core::world::{ClipBlockShape, ClipFluid, World};
use steel_registry::item_stack::ItemStack;
use steel_registry::{REGISTRY, RegistryExt};
use steel_utils::Identifier;
use steel_utils::types::{GameType, InteractionHand};

use crate::convert::clip_to_block_hit;
use crate::test_connection::FlintConnection;

const TEST_PLAYER_NAME: &str = "flintmc_testbot";

/// Test player implementation that wraps a real `Player`.
pub struct SteelTestPlayer {
    player: Arc<Player>,
}

impl SteelTestPlayer {
    /// Creates a new test player in the given world.
    ///
    /// # Panics
    /// Panics if the test identity is invalid or already registered in this world.
    pub fn new(world: Arc<World>, server: &Arc<Server>) -> Self {
        // Same identity as the FlintCLI bot, so specs asserting an `Owner` UUID
        // stay portable between runners.
        let game_profile = GameProfile::new(
            offline_uuid(TEST_PLAYER_NAME),
            TEST_PLAYER_NAME.to_owned(),
            vec![],
        )
        .expect("test player name is valid");
        let player_connection = Arc::new(PlayerConnection::Other(Box::new(
            FlintConnection::default(),
        )));
        let player = Player::new(
            game_profile,
            player_connection,
            Arc::new(PlayerSession::new(10, 10)),
            Arc::clone(&world),
            Arc::downgrade(server),
            server.config.clone(),
            -1, // Negative entity ID for test players
            ClientInformation::default(),
        );
        // The real Flint bot joins away from the test fixture. Starting at the
        // origin can put it inside a block before the first scripted teleport,
        // changing its pose or generating unintended vibrations.
        player
            .try_set_position(DVec3::new(0.5, 100.0, 0.5))
            .expect("test player spawn position");
        // Mark as loaded so interactions work
        player.set_client_loaded(true);
        let player = Arc::new(player);
        assert!(
            world.add_test_player(Arc::clone(&player)),
            "Flint player must be registered through the normal join path"
        );
        Self { player }
    }
}

impl Drop for SteelTestPlayer {
    fn drop(&mut self) {
        let world = self.player.world.load_full();
        world.remove_test_player(&self.player);
    }
}

/// Converts a Flint [`PlayerSlot`] to a Steel inventory slot index.
///
/// Flint uses semantic slot names (e.g., `Hotbar1`, `OffHand`, `Helmet`),
/// while Steel uses numeric indices. This function maps between the two:
/// - Hotbar slots 1-9 → indices 0-8
/// - `OffHand` → `PlayerInventory::SLOT_OFFHAND`
/// - Armor slots → indices 36-39 (boots to helmet)
const fn player_slot_to_index(slot: PlayerSlot) -> usize {
    match slot {
        PlayerSlot::Hotbar1 => 0,
        PlayerSlot::Hotbar2 => 1,
        PlayerSlot::Hotbar3 => 2,
        PlayerSlot::Hotbar4 => 3,
        PlayerSlot::Hotbar5 => 4,
        PlayerSlot::Hotbar6 => 5,
        PlayerSlot::Hotbar7 => 6,
        PlayerSlot::Hotbar8 => 7,
        PlayerSlot::Hotbar9 => 8,
        PlayerSlot::OffHand => PlayerInventory::SLOT_OFFHAND,
        PlayerSlot::Boots => 36,
        PlayerSlot::Leggings => 37,
        PlayerSlot::Chestplate => 38,
        PlayerSlot::Helmet => 39,
    }
}

/// Converts a Flint [`Item`] to a Steel [`ItemStack`].
///
/// Handles the `minecraft:` namespace prefix (strips it if present) and
/// looks up the item in the registry. Returns an empty stack if the item
/// is not found.
fn flint_item_to_stack(item: &Item) -> ItemStack {
    let item_id = item.id.strip_prefix("minecraft:").unwrap_or(&item.id);
    let Some(item_ref) = REGISTRY
        .items
        .by_key(&Identifier::vanilla(item_id.to_string()))
    else {
        tracing::warn!("Unknown item: {} - returning empty stack", item.id);
        return ItemStack::empty();
    };
    ItemStack::with_count(item_ref, i32::from(item.count))
}

/// Converts a Steel [`ItemStack`] to a Flint [`Item`].
///
/// Returns `None` for empty stacks. Adds the `minecraft:` namespace prefix
/// to the item ID for consistency with Flint's expected format.
fn stack_to_flint_item(stack: &ItemStack, requested_data: Vec<String>) -> Option<Item> {
    if stack.is_empty() {
        return None;
    }

    let id = format!("minecraft:{}", stack.item.key.path);
    let mut map: FxHashMap<String, String> = FxHashMap::default();
    for key in requested_data {
        let Some(data) = stack.get_effective_value_raw(&Identifier::vanilla(key.clone())) else {
            continue;
        };
        // Flint only compares strings, so leaf values use their plain textual
        // form and structured components fall back to their `Debug` rendering.
        // Both are stable for a given Steel build, which is what a test
        // assertion needs; nothing here is part of the wire protocol.
        let value = if data.downcast_ref::<()>().is_some() {
            String::new()
        } else if let Some(b) = data.downcast_ref::<bool>() {
            b.to_string()
        } else if let Some(i) = data.downcast_ref::<i32>() {
            i.to_string()
        } else if let Some(f) = data.downcast_ref::<f32>() {
            f.to_string()
        } else if let Some(id) = data.downcast_ref::<Identifier>() {
            id.to_string()
        } else {
            format!("{data:?}")
        };
        map.insert(key, value);
    }
    Some(Item {
        id,
        count: stack.count.try_into().unwrap_or(1),
        data: map,
    })
}

impl FlintPlayer for SteelTestPlayer {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn set_slot(&mut self, slot: PlayerSlot, item: Option<&Item>) -> Result<(), anyhow::Error> {
        let index = player_slot_to_index(slot);
        let stack = item.map_or_else(ItemStack::empty, flint_item_to_stack);

        let mut inv = self.player.inventory.lock();
        inv.set_item(index, stack);
        Ok(())
    }

    fn get_slot(
        &mut self,
        slot: PlayerSlot,
        requested_data: Vec<String>,
    ) -> Result<Option<Item>, anyhow::Error> {
        let index = player_slot_to_index(slot);

        let inv = self.player.inventory.lock();
        let stack = inv.get_item(index);
        Ok(stack_to_flint_item(stack, requested_data))
    }

    fn select_hotbar(&mut self, slot: u8) -> Result<(), anyhow::Error> {
        if (1..=9).contains(&slot) {
            // Flint uses 1-9, Steel uses 0-8
            self.player.inventory.lock().set_selected_slot(slot - 1);
        }
        Ok(())
    }

    fn selected_hotbar(&self) -> u8 {
        // Steel uses 0-8, Flint uses 1-9
        self.player.inventory.lock().get_selected_slot() + 1
    }

    fn teleport(&mut self, pos: [f64; 3], rot: Option<[f32; 2]>) -> Result<(), anyhow::Error> {
        self.player
            .try_set_position(DVec3::new(pos[0], pos[1], pos[2]))?;
        self.player.set_rotation((rot.unwrap_or([0.0, 0.0])).into());
        self.player.update_pose();
        Ok(())
    }

    fn interact(&mut self) -> Result<(), anyhow::Error> {
        let world = self.player.get_world();
        let (start, end) = self.player.get_ray_endpoints();
        let clip = world.clip(start, end, ClipBlockShape::Outline, ClipFluid::None);

        let hand = InteractionHand::MainHand;
        let animation = self.player.interact_animation(hand);
        let result = if clip.is_miss() {
            game_mode::use_item(&self.player, &world, hand)
        } else {
            game_mode::use_item_on(&self.player, &world, hand, &clip_to_block_hit(clip))
        };

        if result.should_swing() {
            self.player.swing(hand, animation, true);
        }
        self.player.broadcast_inventory_changes();

        tracing::debug!("interact({start:?} -> {end:?}) -> {result:?}");
        Ok(())
    }

    fn set_game_mode(&mut self, mode: GameMode) -> Result<(), anyhow::Error> {
        self.player.set_game_mode(match mode {
            GameMode::Survival => GameType::Survival,
            GameMode::Creative => GameType::Creative,
            GameMode::Adventure => GameType::Adventure,
            GameMode::Spectator => GameType::Spectator,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::SteelTestWorld;
    use flint_core::test_spec::{GameMode as FlintGameMode, PlayerSlot as FlintPlayerSlot};
    use flint_core::{Block as FlintBlock, FlintWorld as _};
    use steel_registry::data_components::vanilla_components::DAMAGE;
    use steel_registry::vanilla_items::STONE;

    /// One Flint player-interaction placement scenario.
    struct PlacementScenario {
        /// Human readable scenario name used in the failure report.
        name: &'static str,
        /// Fixture blocks placed before the interaction.
        setup: Vec<([i32; 3], FlintBlock)>,
        /// Player position used for the teleport action.
        player_pos: [f64; 3],
        /// Player rotation (yaw, pitch) used for the teleport action.
        player_rot: [f32; 2],
        /// Item held in the first hotbar slot.
        item: &'static str,
        /// Position where the block is expected to appear.
        target: [i32; 3],
        /// Expected block identifier at the target position.
        expected: &'static str,
    }

    /// The Flint placement scenarios that currently fail.
    fn placement_scenarios() -> Vec<PlacementScenario> {
        vec![
            PlacementScenario {
                name: "coral fan from inside support",
                setup: vec![([8, 1, 4], FlintBlock::new("minecraft:stone"))],
                player_pos: [8.5, 0.0, 4.5],
                player_rot: [-90.0, 0.0],
                item: "minecraft:brain_coral_fan",
                target: [7, 1, 4],
                expected: "minecraft:brain_coral_wall_fan",
            },
            PlacementScenario {
                name: "hay east",
                setup: vec![([0, 1, 0], FlintBlock::new("minecraft:stone"))],
                player_pos: [3.5, 0.0, 0.5],
                player_rot: [90.0, 0.0],
                item: "minecraft:hay_block",
                target: [1, 1, 0],
                expected: "minecraft:hay_block",
            },
            PlacementScenario {
                name: "hay north",
                setup: vec![([0, 1, 0], FlintBlock::new("minecraft:stone"))],
                player_pos: [0.5, 0.0, -2.5],
                player_rot: [0.0, 0.0],
                item: "minecraft:hay_block",
                target: [0, 1, -1],
                expected: "minecraft:hay_block",
            },
            PlacementScenario {
                name: "stained glass side",
                setup: vec![([0, 1, 0], FlintBlock::new("minecraft:red_stained_glass"))],
                player_pos: [0.5, 0.0, -3.5],
                player_rot: [0.0, 0.0],
                item: "minecraft:red_stained_glass",
                target: [0, 1, -1],
                expected: "minecraft:red_stained_glass",
            },
            PlacementScenario {
                name: "grindstone wall",
                setup: vec![([0, 1, 0], FlintBlock::new("minecraft:stone"))],
                player_pos: [0.5, 0.0, -3.5],
                player_rot: [0.0, 0.0],
                item: "minecraft:grindstone",
                target: [0, 1, -1],
                expected: "minecraft:grindstone",
            },
            PlacementScenario {
                name: "grindstone clicked face",
                setup: vec![([0, 1, 0], FlintBlock::new("minecraft:stone"))],
                player_pos: [0.5, 2.5, -2.5],
                player_rot: [0.0, 36.0],
                item: "minecraft:grindstone",
                target: [0, 2, 0],
                expected: "minecraft:grindstone",
            },
            PlacementScenario {
                name: "hanging sign chain",
                setup: vec![
                    ([0, 5, 0], FlintBlock::new("minecraft:stone")),
                    (
                        [0, 4, 0],
                        FlintBlock::with_properties(
                            "minecraft:oak_hanging_sign",
                            [
                                ("rotation".to_owned(), "8".to_owned()),
                                ("attached".to_owned(), "false".to_owned()),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    ),
                ],
                player_pos: [0.5, 1.0, 0.5],
                player_rot: [0.0, -90.0],
                item: "minecraft:oak_hanging_sign",
                target: [0, 3, 0],
                expected: "minecraft:oak_hanging_sign",
            },
        ]
    }

    /// Runs one scenario and reports the ray, the clip result, the placed
    /// block, and the held stack.
    fn run_placement_scenario(scenario: &PlacementScenario) -> (String, String) {
        let mut world = SteelTestWorld::new();
        for (pos, block) in &scenario.setup {
            world.set_block(*pos, block).expect("fixture placement");
        }

        let mut player = world.create_player();
        player
            .set_slot(
                FlintPlayerSlot::Hotbar1,
                Some(&flint_core::Item::new(scenario.item)),
            )
            .expect("hotbar slot");
        player.select_hotbar(1).expect("hotbar selection");
        player
            .set_game_mode(FlintGameMode::Survival)
            .expect("game mode");

        world.do_tick().expect("world tick");
        player
            .teleport(scenario.player_pos, Some(scenario.player_rot))
            .expect("teleport");
        player.interact().expect("interact");

        let placed = world.get_block(scenario.target, &[]).expect("block read");
        let slot = player
            .get_slot(FlintPlayerSlot::Hotbar1, Vec::new())
            .expect("slot read");
        let concrete = player
            .as_any_mut()
            .downcast_mut::<SteelTestPlayer>()
            .expect("steel player");
        let (start, end) = concrete.player.get_ray_endpoints();
        let clip =
            concrete
                .player
                .get_world()
                .clip(start, end, ClipBlockShape::Outline, ClipFluid::None);
        let position = concrete.player.position();
        let rotation = concrete.player.rotation();

        let report = format!(
            "{}: position={position:?} rotation={rotation:?} ray={start:?}->{end:?} \
             clip={clip:?} placed={placed:?} slot={slot:?}",
            scenario.name,
        );
        (report, placed.id)
    }

    /// Reproduces the failing Flint player-interaction placement scenarios
    /// through the real world, player, and interaction path used by the Flint
    /// adapter.
    #[test]
    fn flint_player_placement_scenarios() {
        crate::init();

        let failures: Vec<String> = placement_scenarios()
            .iter()
            .filter_map(|scenario| {
                let (report, placed_id) = run_placement_scenario(scenario);
                let expected = scenario.expected;
                (placed_id != expected).then(|| format!("{report} (expected {expected})"))
            })
            .collect();

        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn stack_conversion_reports_requested_plain_components() {
        crate::init();
        let mut stack = ItemStack::new(&STONE);
        stack.set(DAMAGE, 7);

        let item = stack_to_flint_item(&stack, vec!["damage".to_owned()])
            .expect("non-empty stack converts");
        assert_eq!(item.id, "minecraft:stone");
        assert_eq!(item.data.get("damage").map(String::as_str), Some("7"));

        // Components that were not asked for stay out of the map.
        let unrequested =
            stack_to_flint_item(&stack, Vec::new()).expect("non-empty stack converts");
        assert!(unrequested.data.is_empty());
    }
}
