//! Vanilla `/fillbiome` command.
//!
//! Fills the biome of every block position in a block region. Vanilla writes a
//! 4x4x4 biome cell per block position, so a fill that touches part of a cell
//! still covers the whole cell; Steel mirrors that by writing the section's biome
//! palette entry for the containing cell.

use steel_registry::vanilla_game_rules::MAX_BLOCK_MODIFICATIONS;
use steel_utils::{BlockPos, BoundingBox, Identifier};
use text_components::TextComponent;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};
use super::execute::{ensure_region_chunks_block_ticking, loaded_block_position};
use crate::chunk::ChunkPos;

type Builder = CommandNodeBuilder<CommandSource, SteelCommandRuntime>;

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("fillbiome"), |_| command())
}

fn command() -> Builder {
    let biome = argument("biome", SteelArgumentType::biome())
        .executes(|context| execute_fillbiome(context));

    literal("fillbiome").then(
        argument("from", SteelArgumentType::block_pos())
            .then(argument("to", SteelArgumentType::block_pos()).then(biome)),
    )
}

fn execute_fillbiome(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let source = context.source().clone();
    let world = source.world();
    let from = loaded_block_position(context, "from")?;
    let to = loaded_block_position(context, "to")?;
    let biome_id = context.biome_or_tag("biome")?.to_biome_id()?;

    let region = BoundingBox::from_corners(from, to);
    let area = block_region_volume(region);
    let limit = world.get_game_rule(&MAX_BLOCK_MODIFICATIONS);
    if area > i64::from(limit) {
        return Err(CommandSyntaxError::dynamic(
            "commands.fillbiome.toobig",
            "Too many blocks in the specified area",
            area,
            i64::from(limit),
        ));
    }
    ensure_region_chunks_block_ticking(&source, &region)?;

    let mut changed = 0i32;
    for z in region.min_z()..=region.max_z() {
        for y in region.min_y()..=region.max_y() {
            for x in region.min_x()..=region.max_x() {
                if set_biome_at(world, BlockPos::new(x, y, z), biome_id) {
                    changed += 1;
                }
            }
        }
    }

    let message = TextComponent::plain(format!(
        "Applied the biome to {changed} block(s) at ({}, {}, {}) to ({}, {}, {})",
        from.x(),
        from.y(),
        from.z(),
        to.x(),
        to.y(),
        to.z(),
    ));
    source.send_success(&message, true);
    Ok(changed)
}

/// Volume of a block region, saturating instead of overflowing for huge areas.
fn block_region_volume(region: BoundingBox) -> i64 {
    let x = i64::from(region.max_x()) - i64::from(region.min_x()) + 1;
    let y = i64::from(region.max_y()) - i64::from(region.min_y()) + 1;
    let z = i64::from(region.max_z()) - i64::from(region.min_z()) + 1;
    x.saturating_mul(y).saturating_mul(z)
}

/// Writes the biome id into the 4x4x4 cell containing `pos`.
fn set_biome_at(world: &crate::world::World, pos: BlockPos, biome_id: u16) -> bool {
    let chunk_pos = ChunkPos::new(pos.x() >> 4, pos.z() >> 4);
    world
        .chunk_map
        .with_full_chunk(chunk_pos, |chunk| chunk.set_biome_cell(pos, biome_id))
        == Some(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_of_unit_region_is_one() {
        let region = BoundingBox::from_corners(BlockPos::new(0, 0, 0), BlockPos::new(0, 0, 0));
        assert_eq!(block_region_volume(region), 1);
    }

    #[test]
    fn volume_of_two_cube_is_eight() {
        let region = BoundingBox::from_corners(BlockPos::new(0, 0, 0), BlockPos::new(1, 1, 1));
        assert_eq!(block_region_volume(region), 8);
    }
}
