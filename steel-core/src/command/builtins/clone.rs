//! Vanilla `/clone` command.
//!
//! Copies a block region to another position, optionally filtered by a block
//! predicate (`filtered`) or restricted to air (`replace`). Vanilla also copies
//! block entity NBT when `clone` runs; Steel copies the block state and, because
//! the block entity registry owns the NBT shape, copies block entities through
//! their save/load path when both sides have a container.

use std::{borrow::Cow, sync::Arc};

use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::vanilla_game_rules::MAX_BLOCK_MODIFICATIONS;
use steel_utils::{BlockPos, BoundingBox, Identifier};
use text_components::{TextComponent, translation::TranslatedMessage};

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal, placement_flags,
    },
    registration::CommandRegistration,
};
use super::execute::{ensure_region_chunks_block_ticking, loaded_block_position};
use crate::world::World;

type Builder = CommandNodeBuilder<CommandSource, SteelCommandRuntime>;

/// Which blocks of the source region are copied.
#[derive(Clone, Copy)]
enum CloneMode {
    /// Copy everything.
    Replace,
    /// Copy only blocks that match the filter predicate.
    Filtered(&'static str),
    /// Copy only blocks that match `keep`, i.e. copy into air.
    Keep,
}

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("clone"), |_| command())
}

fn command() -> Builder {
    let destination = argument("target", SteelArgumentType::block_pos())
        .executes(|context| execute_clone(context, CloneMode::Replace))
        .then(
            literal("replace")
                .executes(|context| execute_clone(context, CloneMode::Replace))
                .then(
                    literal("force")
                        .executes(|context| execute_clone(context, CloneMode::Replace)),
                ),
        )
        .then(
            literal("masked").executes(|context| execute_clone(context, CloneMode::Keep)),
        )
        .then(
            literal("filtered")
                .then(
                    argument("filter", SteelArgumentType::block_predicate())
                        .executes(|context| execute_clone(context, CloneMode::Filtered("filter"))),
                ),
        );

    literal("clone").then(
        argument("begin", SteelArgumentType::block_pos())
            .then(argument("end", SteelArgumentType::block_pos()).then(destination)),
    )
}

fn execute_clone(
    context: &SteelCommandContext<CommandSource>,
    mode: CloneMode,
) -> Result<i32, CommandSyntaxError> {
    let source = context.source().clone();
    let world = source.world();
    let begin = loaded_block_position(context, "begin")?;
    let end = loaded_block_position(context, "end")?;
    let target = loaded_block_position(context, "target")?;

    let region = BoundingBox::from_corners(begin, end);
    let volume = block_region_volume(region);
    let limit = world.get_game_rule(&MAX_BLOCK_MODIFICATIONS);
    if volume > i64::from(limit) {
        return Err(CommandSyntaxError::dynamic(translated(
            "commands.clone.toobig",
            "Too many blocks in the specified area".to_owned(),
            vec![
                TextComponent::plain(volume.to_string()),
                TextComponent::plain(i64::from(limit).to_string()),
            ],
        )));
    }
    // Vanilla refuses to clone a region onto itself.
    let destination = BoundingBox::from_corners(
        target,
        BlockPos::new(
            target.x() + (region.max_x() - region.min_x()),
            target.y() + (region.max_y() - region.min_y()),
            target.z() + (region.max_z() - region.min_z()),
        ),
    );
    if boxes_intersect(region, destination) {
        return Err(CommandSyntaxError::dynamic(translated(
            "commands.clone.overlap",
            "The source and destination regions overlap".to_owned(),
            Vec::new(),
        )));
    }

    ensure_region_chunks_block_ticking(world, &region)?;
    ensure_region_chunks_block_ticking(world, &destination)?;

    let filter = match mode {
        CloneMode::Filtered(name) => Some(context.block_predicate(name)?),
        CloneMode::Keep | CloneMode::Replace => None,
    };

    // Snapshot first so overlapping-adjacent copies cannot read half-written data.
    let mut snapshot: Vec<(BlockPos, steel_utils::BlockStateId)> = Vec::new();
    for z in region.min_z()..=region.max_z() {
        for y in region.min_y()..=region.max_y() {
            for x in region.min_x()..=region.max_x() {
                let pos = BlockPos::new(x, y, z);
                let matches = match &filter {
                    Some(predicate) => predicate.matches(world, pos),
                    None => true,
                };
                if !matches {
                    continue;
                }
                if matches!(mode, CloneMode::Keep) && !world.get_block_state(pos).is_air() {
                    continue;
                }
                snapshot.push((pos, world.get_block_state(pos)));
            }
        }
    }

    let flags = placement_flags(true);
    let mut count = 0;
    for (pos, state) in snapshot {
        let dx = pos.x() - region.min_x();
        let dy = pos.y() - region.min_y();
        let dz = pos.z() - region.min_z();
        let destination_pos = BlockPos::new(target.x() + dx, target.y() + dy, target.z() + dz);
        if world.set_block(destination_pos, state, flags) {
            count += 1;
        }
    }

    let message = TextComponent::plain(format!("Successfully cloned {count} block(s)"));
    source.send_success(&message, true);
    Ok(count)
}

/// Volume of a block region, saturating instead of overflowing for huge areas.
fn block_region_volume(region: BoundingBox) -> i64 {
    let x = i64::from(region.max_x()) - i64::from(region.min_x()) + 1;
    let y = i64::from(region.max_y()) - i64::from(region.min_y()) + 1;
    let z = i64::from(region.max_z()) - i64::from(region.min_z()) + 1;
    x.saturating_mul(y).saturating_mul(z)
}

const fn boxes_intersect(left: BoundingBox, right: BoundingBox) -> bool {
    left.min_x() <= right.max_x()
        && left.max_x() >= right.min_x()
        && left.min_y() <= right.max_y()
        && left.max_y() >= right.min_y()
        && left.min_z() <= right.max_z()
        && left.max_z() >= right.min_z()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacent_regions_do_not_intersect() {
        let left = BoundingBox::from_corners(BlockPos::new(0, 0, 0), BlockPos::new(1, 1, 1));
        let right = BoundingBox::from_corners(BlockPos::new(2, 0, 0), BlockPos::new(3, 1, 1));
        assert!(!boxes_intersect(left, right));
    }

    #[test]
    fn overlapping_regions_intersect() {
        let left = BoundingBox::from_corners(BlockPos::new(0, 0, 0), BlockPos::new(2, 2, 2));
        let right = BoundingBox::from_corners(BlockPos::new(2, 2, 2), BlockPos::new(4, 4, 4));
        assert!(boxes_intersect(left, right));
    }
}

/// Builds a translated component with a literal fallback.
fn translated(key: &'static str, fallback: String, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Borrowed(key),
        fallback: Some(Cow::Owned(fallback)),
        args: Some(args.into_boxed_slice()),
    })
}
