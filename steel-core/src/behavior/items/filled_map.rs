//! Filled map item behaviour.
//!
//! Port of vanilla `FilledMapItem`: `createMap` allocates a map id, the
//! per-tick inventory hook refreshes markers and repaints the color grid around
//! the holder, and `updateColors` walks one sixteenth of the map columns per
//! tick exactly like vanilla so a map costs a bounded amount of work.
//!
//! Not ported yet (see `PARITY-26.3-RU.md`): crafting copies/zoom
//! (`MapPostProcessingComponent`), banner markers and item-frame markers.

use steel_macros::item_behavior;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::data_components::components::MapId;
use steel_registry::data_components::vanilla_components::MAP_ID;
use steel_registry::equipment::EquipmentSlotType;
use steel_registry::item_stack::ItemStack;
use steel_registry::map_color::{Brightness, WATER_BLUE, by_id};
use steel_registry::vanilla_blocks;
use steel_utils::{BlockPos, ChunkPos};

use crate::behavior::{InventoryTickContext, ItemBehavior};
use crate::chunk::full_chunk::FullChunkRef;
use crate::chunk::heightmap::HeightmapType;
use crate::entity::Entity as _;
use crate::player::Player;
use crate::world::World;
use crate::world::map_state::{MAP_SIZE, MapState};

/// Vanilla `FilledMapItem.MAP_WIDTH`.
pub const MAP_WIDTH: i32 = MAP_SIZE as i32;

/// Vanilla `FilledMapItem` color-update constants.
const NETHER_HASH_MULTIPLIER_A: i32 = 231_871;
const NETHER_HASH_MULTIPLIER_B: i32 = 31_287_121;
const NETHER_HASH_ADDEND: i32 = 11;
const NETHER_HASH_BIT_SHIFT: u32 = 20;
const NETHER_DIRT_WEIGHT: u16 = 10;
const NETHER_STONE_WEIGHT: u16 = 100;
const NETHER_BASE_HEIGHT: f64 = 100.0;

const WATER_BRIGHTNESS_HIGH_THRESHOLD: f64 = 0.5;
const WATER_BRIGHTNESS_LOW_THRESHOLD: f64 = 0.9;
const LAND_BRIGHTNESS_HIGH_THRESHOLD: f64 = 0.6;
const LAND_BRIGHTNESS_LOW_THRESHOLD: f64 = -0.6;
const LAND_SLOPE_FACTOR: f64 = 4.0;
const LAND_SLOPE_CHECKER_OFFSET: f64 = 0.4;
const WATER_DEPTH_FACTOR: f64 = 0.1;
const CHECKER_FACTOR: f64 = 0.2;

/// Vanilla `FilledMapItem`.
#[item_behavior(class = "MapItem")]
pub struct FilledMapItem;

impl FilledMapItem {
    /// Vanilla `FilledMapItem.createMap`.
    ///
    /// Allocates a fresh map id in `world` and returns a `filled_map` stack
    /// carrying the `map_id` component.
    #[must_use]
    pub fn create_map(
        world: &World,
        x: i32,
        z: i32,
        scale: u8,
        show_icons: bool,
        unlimited_tracking: bool,
    ) -> ItemStack {
        let (state, _) = MapState::of(
            f64::from(x),
            f64::from(z),
            scale,
            show_icons,
            unlimited_tracking,
            world.key.clone(),
        );
        let id = world.map_states.lock().insert(state);
        let mut stack = ItemStack::new(&steel_registry::vanilla_items::FILLED_MAP);
        stack.set(MAP_ID, MapId::new(id));
        stack
    }

    /// Vanilla `FilledMapItem.getMapState(ItemStack, World)` returning the id.
    #[must_use]
    pub fn map_id(stack: &ItemStack) -> Option<i32> {
        stack.get(MAP_ID).map(|id| id.id())
    }
}

impl ItemBehavior for FilledMapItem {
    fn inventory_tick(&self, context: &mut InventoryTickContext<'_>) {
        let Some(map_id) = context
            .with_item(|stack| FilledMapItem::map_id(stack))
            .flatten()
        else {
            return;
        };
        let Some(player) = context.owner.as_player() else {
            return;
        };
        let held_in_hand = context
            .slot
            .is_some_and(|slot| slot.slot_type() == EquipmentSlotType::Hand);

        let world = context.world;
        let mut maps = world.map_states.lock();
        let Some(state) = maps.get_mut(map_id) else {
            return;
        };
        state.update(world, player);
        if !state.locked() && held_in_hand {
            update_colors(world, player, state);
        }
    }
}

/// Vanilla `FilledMapItem.updateColors`.
///
/// Repaints the pixel column selected by the player's update tick; only changed
/// pixels are written, and each write widens the player's pending delta patch.
pub fn update_colors(world: &World, player: &Player, state: &mut MapState) {
    if world.key != *state.dimension() {
        return;
    }

    let block_scale = 1_i32 << state.scale();
    let center_x = state.center_x();
    let center_z = state.center_z();
    let position = player.position();
    let pixel_offset_x = (position.x - f64::from(center_x)).floor() as i32;
    let pixel_offset_z = (position.z - f64::from(center_z)).floor() as i32;
    let player_pixel_x = pixel_offset_x / block_scale + MAP_WIDTH / 2;
    let player_pixel_z = pixel_offset_z / block_scale + MAP_WIDTH / 2;
    let mut render_radius = MAP_WIDTH / block_scale;
    let has_ceiling = world.dimension_type.has_ceiling;
    if has_ceiling {
        render_radius /= 2;
    }

    let update_tick = state.advance_update_tick(player.uuid()) as i32;
    let edge_limit = (render_radius - 2) * (render_radius - 2);
    let radius_squared = render_radius * render_radius;
    let mut needs_update = false;

    for pixel_x in (player_pixel_x - render_radius + 1)..(player_pixel_x + render_radius) {
        if (pixel_x & 15) != (update_tick & 15) && !needs_update {
            continue;
        }
        needs_update = false;
        let mut previous_average_height = 0.0_f64;

        for pixel_z in (player_pixel_z - render_radius - 1)..(player_pixel_z + render_radius) {
            if pixel_x < 0 || pixel_z < -1 || pixel_x >= MAP_WIDTH || pixel_z >= MAP_WIDTH {
                continue;
            }

            let delta_x = pixel_x - player_pixel_x;
            let delta_z = pixel_z - player_pixel_z;
            let distance_squared = delta_x * delta_x + delta_z * delta_z;
            let is_edge_pixel = distance_squared > edge_limit;
            let world_x = (center_x / block_scale + pixel_x - MAP_WIDTH / 2) * block_scale;
            let world_z = (center_z / block_scale + pixel_z - MAP_WIDTH / 2) * block_scale;

            let Some(sample) = sample_pixel(world, world_x, world_z, block_scale, has_ceiling)
            else {
                continue;
            };

            let brightness = if by_id(sample.dominant) == WATER_BLUE {
                let water_factor = f64::from(sample.water_depth) * WATER_DEPTH_FACTOR
                    + f64::from((pixel_x + pixel_z) & 1) * CHECKER_FACTOR;
                water_brightness(water_factor)
            } else {
                let slope = (sample.average_height - previous_average_height) * LAND_SLOPE_FACTOR
                    / f64::from(block_scale + 4)
                    + (f64::from((pixel_x + pixel_z) & 1) - 0.5) * LAND_SLOPE_CHECKER_OFFSET;
                land_brightness(slope)
            };

            previous_average_height = sample.average_height;

            if pixel_z >= 0
                && distance_squared < radius_squared
                && (!is_edge_pixel || ((pixel_x + pixel_z) & 1) != 0)
            {
                let color = by_id(sample.dominant).render_byte(brightness);
                needs_update |= state.put_color(pixel_x as usize, pixel_z as usize, color);
            }
        }
    }
}

/// Vanilla water brightness rule: shallow water is bright, deep water is dark.
fn water_brightness(water_factor: f64) -> Brightness {
    if water_factor < WATER_BRIGHTNESS_HIGH_THRESHOLD {
        Brightness::High
    } else if water_factor > WATER_BRIGHTNESS_LOW_THRESHOLD {
        Brightness::Low
    } else {
        Brightness::Normal
    }
}

/// Vanilla land brightness rule: slopes facing up are bright, facing down dark.
fn land_brightness(slope: f64) -> Brightness {
    if slope > LAND_BRIGHTNESS_HIGH_THRESHOLD {
        Brightness::High
    } else if slope < LAND_BRIGHTNESS_LOW_THRESHOLD {
        Brightness::Low
    } else {
        Brightness::Normal
    }
}

/// One sampled map pixel.
struct PixelSample {
    dominant: u8,
    water_depth: i32,
    average_height: f64,
}

/// Samples the surface color of one map pixel.
///
/// Matches vanilla's inner loop: `block_scale x block_scale` column samples, the
/// most frequent palette id wins (ties keep first-seen order), water depth is
/// averaged and the block's own map color is used for the fluid case because
/// Steel's `FluidState` carries no block state.
fn sample_pixel(
    world: &World,
    world_x: i32,
    world_z: i32,
    block_scale: i32,
    has_ceiling: bool,
) -> Option<PixelSample> {
    let chunk_pos = ChunkPos::new(world_x >> 4, world_z >> 4);
    world
        .chunk_map
        .with_full_chunk(chunk_pos, |chunk| {
            sample_pixel_in_chunk(chunk, world_x, world_z, block_scale, has_ceiling)
        })
}

fn sample_pixel_in_chunk(
    chunk: FullChunkRef<'_>,
    world_x: i32,
    world_z: i32,
    block_scale: i32,
    has_ceiling: bool,
) -> PixelSample {
    // Palette counters instead of vanilla's `LinkedHashMultiset` allocations,
    // which keeps the hot path allocation-free.
    let mut palette = PaletteCounter::default();
    let bottom_y = chunk.min_y();
    let mut water_depth_sum = 0_i32;
    let mut average_height = 0.0_f64;

    if has_ceiling {
        let mut hash = world_x.wrapping_add(world_z.wrapping_mul(NETHER_HASH_MULTIPLIER_A));
        hash = hash
            .wrapping_mul(hash)
            .wrapping_mul(NETHER_HASH_MULTIPLIER_B)
            .wrapping_add(hash.wrapping_mul(NETHER_HASH_ADDEND));
        if (hash >> NETHER_HASH_BIT_SHIFT) & 1 == 0 {
            palette.add(
                vanilla_blocks::DIRT.default_state().map_color(),
                NETHER_DIRT_WEIGHT,
            );
        } else {
            palette.add(
                vanilla_blocks::STONE.default_state().map_color(),
                NETHER_STONE_WEIGHT,
            );
        }
        average_height = NETHER_BASE_HEIGHT;
    } else {
        let divisor = block_scale * block_scale;
        for sub_x in 0..block_scale {
            for sub_z in 0..block_scale {
                let block_x = world_x + sub_x;
                let block_z = world_z + sub_z;
                let local_x = (block_x & 15) as usize;
                let local_z = (block_z & 15) as usize;
                let surface = chunk.get_height(HeightmapType::WorldSurface, local_x, local_z);

                let mut surface_y = surface;
                let block_state = if surface <= bottom_y {
                    vanilla_blocks::BEDROCK.default_state()
                } else {
                    loop {
                        surface_y -= 1;
                        let candidate =
                            chunk.get_block_state(BlockPos::new(block_x, surface_y, block_z));
                        if candidate.map_color() != 0 || surface_y <= bottom_y {
                            break candidate;
                        }
                    }
                };

                if surface_y > bottom_y && !block_state.get_fluid_state().is_empty() {
                    let mut fluid_y = surface_y - 1;
                    loop {
                        let check_y = fluid_y;
                        fluid_y -= 1;
                        let fluid_block =
                            chunk.get_block_state(BlockPos::new(block_x, check_y, block_z));
                        water_depth_sum += 1;
                        if fluid_y <= bottom_y || fluid_block.get_fluid_state().is_empty() {
                            break;
                        }
                    }
                }

                average_height += f64::from(surface_y) / f64::from(divisor);
                palette.add(block_state.map_color(), 1);
            }
        }
    }

    let divisor = block_scale * block_scale;
    let water_depth = if divisor == 0 {
        0
    } else {
        water_depth_sum / divisor
    };

    PixelSample {
        dominant: palette.dominant(),
        water_depth,
        average_height,
    }
}

/// Sends the pending map delta packets for every map the player carries.
///
/// Vanilla does this in `ServerPlayerEntity.playerTick` for every non-empty
/// inventory slot, independently of the per-item color update.
pub fn send_map_packets(player: &Player) {
    let world = player.get_world();
    let uuid = player.uuid();
    let map_ids = {
        let inventory = player.inventory.lock();
        let mut ids = Vec::new();
        for stack in inventory.get_items() {
            if let Some(id) = FilledMapItem::map_id(stack)
                && !ids.contains(&id)
            {
                ids.push(id);
            }
        }
        ids
    };
    if map_ids.is_empty() {
        return;
    }

    let mut packets = Vec::new();
    {
        let mut maps = world.map_states.lock();
        for id in map_ids {
            if let Some(state) = maps.get_mut(id)
                && let Some(packet) = state.take_packet(uuid)
            {
                packets.push(packet);
            }
        }
    }
    for packet in packets {
        player.send_packet(packet);
    }
}

/// Allocation-free stand-in for vanilla's `LinkedHashMultiset<MapColor>`.
///
/// Ties keep insertion order, matching `Multisets.copyHighestCountFirst` over a
/// `LinkedHashMultiset`.
#[derive(Debug)]
struct PaletteCounter {
    counts: [u16; 64],
    first_seen: [u8; 64],
    first_seen_len: usize,
}

impl Default for PaletteCounter {
    fn default() -> Self {
        Self {
            counts: [0; 64],
            first_seen: [0; 64],
            first_seen_len: 0,
        }
    }
}

impl PaletteCounter {
    fn add(&mut self, id: u8, weight: u16) {
        let index = usize::from(id & 63);
        if self.counts[index] == 0 && self.first_seen_len < self.first_seen.len() {
            self.first_seen[self.first_seen_len] = index as u8;
            self.first_seen_len += 1;
        }
        self.counts[index] = self.counts[index].saturating_add(weight);
    }

    fn dominant(&self) -> u8 {
        let mut dominant = 0_u8;
        let mut best = 0_u16;
        for index in self.first_seen.iter().take(self.first_seen_len) {
            let count = self.counts[usize::from(*index)];
            if count > best {
                best = count;
                dominant = *index;
            }
        }
        dominant
    }
}

#[cfg(test)]
mod tests {
    use super::{Brightness, PaletteCounter, land_brightness, water_brightness};

    #[test]
    fn water_brightness_uses_vanilla_thresholds() {
        // Vanilla: `< 0.5` is HIGH, `> 0.9` is LOW, otherwise NORMAL.
        assert_eq!(water_brightness(0.0), Brightness::High);
        assert_eq!(water_brightness(0.49), Brightness::High);
        assert_eq!(water_brightness(0.5), Brightness::Normal);
        assert_eq!(water_brightness(0.9), Brightness::Normal);
        assert_eq!(water_brightness(0.91), Brightness::Low);
    }

    #[test]
    fn land_brightness_uses_vanilla_thresholds() {
        // Vanilla: `> 0.6` is HIGH, `< -0.6` is LOW, otherwise NORMAL.
        assert_eq!(land_brightness(0.61), Brightness::High);
        assert_eq!(land_brightness(0.6), Brightness::Normal);
        assert_eq!(land_brightness(0.0), Brightness::Normal);
        assert_eq!(land_brightness(-0.6), Brightness::Normal);
        assert_eq!(land_brightness(-0.61), Brightness::Low);
    }

    #[test]
    fn palette_picks_the_most_frequent_color() {
        let mut palette = PaletteCounter::default();
        palette.add(12, 1);
        palette.add(11, 3);
        palette.add(12, 1);
        assert_eq!(palette.dominant(), 11);
    }

    #[test]
    fn palette_ties_keep_first_seen_order() {
        // Vanilla `Multisets.copyHighestCountFirst` keeps insertion order on
        // equal counts with a `LinkedHashMultiset`.
        let mut palette = PaletteCounter::default();
        palette.add(5, 1);
        palette.add(9, 1);
        assert_eq!(palette.dominant(), 5);

        let mut reversed = PaletteCounter::default();
        reversed.add(9, 1);
        reversed.add(5, 1);
        assert_eq!(reversed.dominant(), 9);
    }

    #[test]
    fn palette_ignores_never_seen_ids() {
        let palette = PaletteCounter::default();
        assert_eq!(palette.dominant(), 0, "CLEAR wins when nothing was sampled");
    }
}
