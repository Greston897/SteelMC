//! `/forceload` — keeps chunks loaded with or without a nearby player.
//!
//! Vanilla `ForceLoadCommand` works on chunks, not blocks: `add` and `remove`
//! take two corner blocks and cover every chunk between them, `query` reports
//! the forced chunks in the source's dimension, and a bare command reports the
//! total. The chunks stay loaded because the `FORCED` ticket has no timeout and
//! is persisted with the world, which is exactly what Steel's ticket storage
//! does for this type.
//!
//! Arguments are vanilla two-coordinate column positions. Requests are bounded
//! to 256 chunks before allocation and to the vanilla world coordinate limit.

use std::borrow::Cow;

use steel_utils::{BlockPos, ChunkPos, Identifier};
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};

const FROM_ARGUMENT: &str = "from";
const TO_ARGUMENT: &str = "to";

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("forceload"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    let from = argument(FROM_ARGUMENT, SteelArgumentType::column_pos());
    let to = argument(TO_ARGUMENT, SteelArgumentType::column_pos());

    // `forceload query` reports the source dimension's forced chunks.
    let query = literal("query")
        .executes(query_all)
        .then(from.clone().executes(query_one));

    let add = literal("add").then(
        from.clone()
            .executes(|context| change(context, Change::Add))
            .then(to.clone().executes(|context| change(context, Change::Add))),
    );

    let remove = literal("remove").then(
        from.executes(|context| change(context, Change::Remove))
            .then(to.executes(|context| change(context, Change::Remove))),
    );

    literal("forceload").then(query).then(add).then(remove)
}

#[derive(Clone, Copy)]
enum Change {
    Add,
    Remove,
}

/// Builds a translated component with a literal fallback.
fn translated(key: &'static str, fallback: String, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Borrowed(key),
        fallback: Some(Cow::Owned(fallback)),
        args: Some(args.into_boxed_slice()),
    })
}

/// Every chunk between two corner blocks, in chunk coordinates.
///
/// Vanilla iterates the block rectangle and converts each block to its chunk;
/// deduplicating through a set is cheaper here and produces the same set. The
/// order is by chunk x then z so the feedback is deterministic.
fn chunks_between(from: BlockPos, to: BlockPos) -> Vec<ChunkPos> {
    let from_chunk = ChunkPos::from_block_pos(from);
    let to_chunk = ChunkPos::from_block_pos(to);
    let (min_x, max_x) = (
        from_chunk.x().min(to_chunk.x()),
        from_chunk.x().max(to_chunk.x()),
    );
    let (min_z, max_z) = (
        from_chunk.z().min(to_chunk.z()),
        from_chunk.z().max(to_chunk.z()),
    );

    let mut chunks = Vec::new();
    for x in min_x..=max_x {
        for z in min_z..=max_z {
            chunks.push(ChunkPos::new(x, z));
        }
    }
    chunks
}

fn validate_region(from: BlockPos, to: BlockPos) -> Result<(), CommandSyntaxError> {
    for coordinate in [from.x(), from.z(), to.x(), to.z()] {
        if !(-30_000_000..30_000_000).contains(&coordinate) {
            return Err(CommandSyntaxError::dynamic("Position outside world bounds"));
        }
    }
    let a = ChunkPos::from_block_pos(from);
    let b = ChunkPos::from_block_pos(to);
    let count = (i64::from(a.x()) - i64::from(b.x())).abs() + 1;
    let count = count * ((i64::from(a.z()) - i64::from(b.z())).abs() + 1);
    if count > 256 {
        return Err(CommandSyntaxError::dynamic(
            "Cannot force load more than 256 chunks",
        ));
    }
    Ok(())
}

fn change(
    context: &SteelCommandContext<CommandSource>,
    change: Change,
) -> Result<i32, CommandSyntaxError> {
    let from = context
        .coordinates(FROM_ARGUMENT)?
        .block_pos(context.source());
    // A single position is a one-block rectangle, which is how vanilla handles
    // `forceload add <pos>`.
    let to = if let Ok(coordinates) = context.coordinates(TO_ARGUMENT) {
        coordinates.block_pos(context.source())
    } else {
        from
    };

    validate_region(from, to)?;
    let chunks = chunks_between(from, to);
    let world = context.source().world();
    let chunk_map = &world.chunk_map;

    let mut changed = 0;
    match change {
        Change::Add => {
            for chunk in &chunks {
                chunk_map.force_load(*chunk);
                changed += 1;
            }
        }
        Change::Remove => {
            for chunk in &chunks {
                if chunk_map.remove_force_load(*chunk) {
                    changed += 1;
                }
            }
        }
    }

    let key = match change {
        Change::Add => "commands.forceload.added.multiple",
        Change::Remove => "commands.forceload.removed.multiple",
    };
    let fallback = match change {
        Change::Add => format!("Marked {changed} chunk(s) to be force loaded"),
        Change::Remove => format!("Marked {changed} chunk(s) as no longer force loaded"),
    };
    let message = translated(
        key,
        fallback,
        vec![
            TextComponent::from(changed.to_string()),
            TextComponent::from(format!("{from:?}")),
            TextComponent::from(format!("{to:?}")),
        ],
    );
    context.source().send_success(&message, true);
    Ok(changed)
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "command executors return the command result type"
)]
fn query_all(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let world = context.source().world();
    let blocked = world.chunk_map.forced_chunk_count();
    let message = translated(
        "commands.forceload.query",
        format!("{blocked} chunk(s) are force loaded"),
        vec![TextComponent::from(blocked.to_string())],
    );
    context.source().send_success(&message, false);
    Ok(i32::try_from(blocked).unwrap_or(i32::MAX))
}

fn query_one(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let from = context
        .coordinates(FROM_ARGUMENT)?
        .block_pos(context.source());
    let chunk = ChunkPos::from_block_pos(from);
    let world = context.source().world();
    let forced = world.chunk_map.is_force_loaded(chunk);

    let key = if forced {
        "commands.forceload.query.success"
    } else {
        "commands.forceload.query.failure"
    };
    let fallback = if forced {
        format!("Chunk at {chunk:?} is force loaded")
    } else {
        format!("Chunk at {chunk:?} is not force loaded")
    };
    let message = translated(
        key,
        fallback,
        vec![TextComponent::from(format!("{chunk:?}"))],
    );
    if forced {
        context.source().send_success(&message, false);
        Ok(1)
    } else {
        context.source().send_failure(message);
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_block_covers_exactly_its_chunk() {
        let pos = BlockPos::new(17, 64, -3);
        let chunks = chunks_between(pos, pos);
        assert_eq!(chunks, vec![ChunkPos::new(1, -1)]);
    }

    #[test]
    fn a_rectangle_covers_every_chunk_between_the_corners() {
        // Blocks 0..15 are chunk 0 and 16..31 chunk 1, so a 0..17 block
        // rectangle spans two chunks on each axis.
        let chunks = chunks_between(BlockPos::new(0, 64, 0), BlockPos::new(17, 64, 17));
        assert_eq!(
            chunks,
            vec![
                ChunkPos::new(0, 0),
                ChunkPos::new(0, 1),
                ChunkPos::new(1, 0),
                ChunkPos::new(1, 1),
            ]
        );
    }

    #[test]
    fn the_corner_order_does_not_matter() {
        let forward = chunks_between(BlockPos::new(0, 0, 0), BlockPos::new(40, 0, 40));
        let backward = chunks_between(BlockPos::new(40, 0, 40), BlockPos::new(0, 0, 0));
        let mut forward_sorted = forward;
        let mut backward_sorted = backward;
        forward_sorted.sort_by_key(|chunk| (chunk.x(), chunk.z()));
        backward_sorted.sort_by_key(|chunk| (chunk.x(), chunk.z()));
        assert_eq!(forward_sorted, backward_sorted);
    }
}

#[cfg(test)]
mod bounds_tests {
    use super::*;
    #[test]
    fn validates_limits_before_allocating_chunk_rectangle() {
        let origin = BlockPos::new(0, 64, 0);
        assert!(validate_region(origin, BlockPos::new(255, 64, 255)).is_ok());
        assert!(validate_region(origin, BlockPos::new(256, 64, 255)).is_err());
        assert!(validate_region(BlockPos::new(-30_000_001, 0, 0), origin).is_err());
        assert!(validate_region(origin, BlockPos::new(30_000_000, 0, 0)).is_err());
        assert!(validate_region(origin, BlockPos::new(-16, 0, -16)).is_ok());
    }
}
