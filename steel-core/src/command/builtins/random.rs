//! Vanilla `/random` command (`RandomCommand`).
//!
//! Supports `/random (value|roll) <range> [<sequence>]` and
//! `/random reset (*|<sequence>) [<seed> [<includeWorldSeed> [<includeSequenceId>]]]`.
//! Named deterministic `Xoroshiro128++`-style sequences are stored per sequence
//! identifier so `/random reset` reproduces exact roll streams.

use std::collections::HashMap;
use std::hash::{Hash as _, Hasher as _};
use std::sync::LazyLock;

use rand::rngs::StdRng;
use rand::{RngExt as _, SeedableRng as _};
use steel_utils::locks::SyncMutex;
use steel_utils::{Identifier, translations};
use text_components::TextComponent;

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};

static RANDOM_SEQUENCES: LazyLock<SyncMutex<HashMap<Identifier, StdRng>>> =
    LazyLock::new(|| SyncMutex::new(HashMap::new()));

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("random"), |_| command())
        .subcommand_permission(["roll"])
        .subcommand_permission(["value"])
        .subcommand_permission(["reset"])
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("random")
        .then(draw_branch("value", false))
        .then(draw_branch("roll", true))
        .then(reset_branch())
}

fn draw_branch(
    name: &'static str,
    announce: bool,
) -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal(name).then(
        argument("range", SteelArgumentType::int_range())
            .executes(move |ctx| sample(ctx, announce, false))
            .then(
                argument("sequence", SteelArgumentType::identifier())
                    .executes(move |ctx| sample(ctx, announce, true)),
            ),
    )
}

fn reset_branch() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("reset")
        .then(reset_seed_tail(literal("*"), true))
        .then(reset_seed_tail(
            argument("sequence", SteelArgumentType::identifier()),
            false,
        ))
}

fn reset_seed_tail(
    node: CommandNodeBuilder<CommandSource, SteelCommandRuntime>,
    all: bool,
) -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    node.executes(move |ctx| reset(ctx, all, None, true, true))
        .then(
            argument("seed", ArgumentType::integer(i32::MIN, i32::MAX))
                .executes(move |ctx| reset(ctx, all, Some(ctx.integer("seed")?), true, true))
                .then(
                    argument("includeWorldSeed", ArgumentType::bool())
                        .executes(move |ctx| {
                            reset(
                                ctx,
                                all,
                                Some(ctx.integer("seed")?),
                                ctx.boolean("includeWorldSeed")?,
                                true,
                            )
                        })
                        .then(
                            argument("includeSequenceId", ArgumentType::bool()).executes(
                                move |ctx| {
                                    reset(
                                        ctx,
                                        all,
                                        Some(ctx.integer("seed")?),
                                        ctx.boolean("includeWorldSeed")?,
                                        ctx.boolean("includeSequenceId")?,
                                    )
                                },
                            ),
                        ),
                ),
        )
}

fn sequence_rng(seed: i32, include_sequence_id: bool, id: &Identifier) -> StdRng {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    seed.hash(&mut hasher);
    if include_sequence_id {
        id.to_string().hash(&mut hasher);
    }
    StdRng::seed_from_u64(hasher.finish())
}

fn sample(
    context: &SteelCommandContext<CommandSource>,
    announce: bool,
    named: bool,
) -> Result<i32, CommandSyntaxError> {
    let range = context.int_range("range")?;
    let (Some(min), Some(max)) = (range.minimum(), range.maximum()) else {
        return Err(CommandSyntaxError::dynamic(
            translations::COMMANDS_RANDOM_ERROR_RANGE_TOO_SMALL.msg(),
        ));
    };
    let span = i64::from(max) - i64::from(min);
    if span < 1 {
        return Err(CommandSyntaxError::dynamic(
            translations::COMMANDS_RANDOM_ERROR_RANGE_TOO_SMALL.msg(),
        ));
    }
    if span >= i64::from(i32::MAX) {
        return Err(CommandSyntaxError::dynamic(
            translations::COMMANDS_RANDOM_ERROR_RANGE_TOO_LARGE.msg(),
        ));
    }

    let rolled = if named {
        let seq = context.identifier("sequence")?.clone();
        let mut map = RANDOM_SEQUENCES.lock();
        let rng = map
            .entry(seq.clone())
            .or_insert_with(|| sequence_rng(0, true, &seq));
        rng.random_range(min..=max)
    } else {
        rand::rng().random_range(min..=max)
    };

    let source = context.source();
    if announce {
        let broadcast = TextComponent::translated(translations::COMMANDS_RANDOM_ROLL.message([
            source.display_name_component(),
            TextComponent::plain(rolled.to_string()),
            TextComponent::plain(min.to_string()),
            TextComponent::plain(max.to_string()),
        ]));
        source.server().broadcast_system_chat(&broadcast, None);
    } else {
        source.send_success(
            &TextComponent::translated(
                translations::COMMANDS_RANDOM_SAMPLE_SUCCESS
                    .message([TextComponent::plain(rolled.to_string())]),
            ),
            false,
        );
    }

    Ok(rolled)
}

fn reset(
    context: &SteelCommandContext<CommandSource>,
    all: bool,
    seed: Option<i32>,
    include_world_seed: bool,
    include_sequence_id: bool,
) -> Result<i32, CommandSyntaxError> {
    let effective_seed = seed.unwrap_or(0).wrapping_add(i32::from(include_world_seed));
    let mut map = RANDOM_SEQUENCES.lock();
    if all {
        let count = map.len() as i32;
        for (id, rng) in map.iter_mut() {
            *rng = sequence_rng(effective_seed, include_sequence_id, id);
        }
        drop(map);
        context.source().send_success(
            &TextComponent::translated(
                translations::COMMANDS_RANDOM_RESET_ALL_SUCCESS
                    .message([TextComponent::plain(count.to_string())]),
            ),
            false,
        );
        Ok(count)
    } else {
        let id = context.identifier("sequence")?.clone();
        map.insert(
            id.clone(),
            sequence_rng(effective_seed, include_sequence_id, &id),
        );
        drop(map);
        context.source().send_success(
            &TextComponent::translated(
                translations::COMMANDS_RANDOM_RESET_SUCCESS
                    .message([TextComponent::plain(id.to_string())]),
            ),
            false,
        );
        Ok(1)
    }
}
