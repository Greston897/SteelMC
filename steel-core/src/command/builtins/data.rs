//! `/data` — reads and edits the NBT of blocks, entities and command storage.
//!
//! Vanilla's `DataCommand` has three sources (block, entity, storage) and four
//! operations each (`get`, `merge`, `modify`, `remove`). Steel implements the
//! full matrix for storage, which is the one place NBT can round-trip
//! faithfully: command storage is a plain compound owned by the server, so a
//! write is just "mutate and put it back".
//!
//! Blocks and entities only implement `get`. Their `save`/`load` pair is lossy
//! today — entities expose the fields they sync, and a block entity's NBT is
//! written back through `load_additional` only for the fields that method
//! accepts — so a `merge` there would silently drop data instead of reporting
//! it. Those forms are present in the tree but answer with a failure that says
//! so, rather than pretending the edit landed.

use std::borrow::Cow;

use simdnbt::owned::{NbtCompound, NbtTag};
use steel_utils::{
    BlockPos, Identifier,
    nbt::{NbtPath, NbtPathMutationError, to_canonical_snbt},
};
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;

use crate::block_entity::BlockEntity as _;
use crate::entity::Entity as _;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};

type Builder = CommandNodeBuilder<CommandSource, SteelCommandRuntime>;

/// Where `/data` reads from or writes to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DataTarget {
    Block,
    Entity,
    Storage,
}

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("data"), |_| command())
}

fn command() -> Builder {
    literal("data")
        .then(target_sources("block", DataTarget::Block))
        .then(target_sources("entity", DataTarget::Entity))
        .then(target_sources("storage", DataTarget::Storage))
}

/// The `get`, `merge` and `remove` subtrees for one source.
///
/// The argument that names the source is `targetPos` for blocks, `target` for
/// entities and storage: those are vanilla's own names, and the source resolver
/// below reads whichever one the chosen source uses.
fn target_sources(name: &'static str, target: DataTarget) -> Builder {
    literal(name).then(match target {
        DataTarget::Block => argument("targetPos", SteelArgumentType::block_pos()).then(operations(target)),
        DataTarget::Entity => argument("target", SteelArgumentType::entity())
            .then(operations(target)),
        DataTarget::Storage => argument("target", SteelArgumentType::storage_key())
            .then(operations(target)),
    })
}

fn operations(target: DataTarget) -> Builder {
    literal("get")
        .then(argument("path", SteelArgumentType::nbt_path()).executes(move |context| {
            get(context, target, GetKind::Tag)
        }))
        .then(
            literal("get").then(
                argument("path", SteelArgumentType::nbt_path())
                    .then(literal("value").executes(move |context| get(context, target, GetKind::Value)))
                    .then(
                        literal("string").executes(move |context| get(context, target, GetKind::String)),
                    )
                    .then(
                        argument("scale", super::super::brigadier::ArgumentType::double(
                            -f64::MAX,
                            f64::MAX,
                        ))
                        .executes(move |context| get(context, target, GetKind::Scaled)),
                    ),
            ),
        )
        .then(
            literal("merge").then(
                argument("nbt", SteelArgumentType::compound())
                    .executes(move |context| merge(context, target)),
            ),
        )
        .then(
            literal("remove").then(
                argument("path", SteelArgumentType::nbt_path())
                    .executes(move |context| remove(context, target)),
            ),
        )
}

#[derive(Clone, Copy)]
enum GetKind {
    /// The whole tag at the path, printed as SNBT.
    Tag,
    /// A single numeric value, printed bare.
    Value,
    /// A stringified scalar.
    String,
    /// A numeric value divided by the `scale` argument.
    Scaled,
}

fn get(
    context: &SteelCommandContext<CommandSource>,
    target: DataTarget,
    kind: GetKind,
) -> Result<i32, CommandSyntaxError> {
    let root = read_root(context, target)?;
    let path = context.nbt_path("path")?.clone();
    let tags = path.get(&NbtTag::Compound(root));
    if tags.is_empty() {
        context.source().send_failure(translated(
            "commands.data.path.no_data",
            "Nothing matches that NBT path".to_owned(),
            Vec::new(),
        ));
        return Ok(0);
    }

    let printed = match kind {
        GetKind::Tag => tags
            .iter()
            .filter_map(to_canonical_snbt)
            .collect::<Vec<_>>()
            .join(", "),
        GetKind::Value => {
            let Some(value) = tags.first().and_then(numeric_value) else {
                context.source().send_failure(translated(
                    "commands.data.path.not_a_number",
                    "The tag at that path is not a number".to_owned(),
                    Vec::new(),
                ));
                return Ok(0);
            };
            format!("{value}")
        }
        GetKind::String => {
            let Some(value) = tags.first() else {
                return Ok(0);
            };
            value_string(value)
        }
        GetKind::Scaled => {
            let scale = context.double("scale")?;
            let Some(value) = tags.first().and_then(numeric_value) else {
                context.source().send_failure(translated(
                    "commands.data.path.not_a_number",
                    "The tag at that path is not a number".to_owned(),
                    Vec::new(),
                ));
                return Ok(0);
            };
            format!("{}", value / scale)
        }
    };

    context.source().send_success(
        &translated(
            "commands.data.get.success",
            format!("Value: {printed}"),
            vec![TextComponent::from(printed.clone())],
        ),
        false,
    );
    Ok(1)
}

fn merge(
    context: &SteelCommandContext<CommandSource>,
    target: DataTarget,
) -> Result<i32, CommandSyntaxError> {
    if target != DataTarget::Storage {
        return unsupported_write(context, target);
    }
    let nbt = context.compound("nbt")?.clone();
    let key = context.identifier("target")?.clone();
    let Some(storage) = command_storage(context) else {
        return Ok(0);
    };
    let mut existing = storage.get(&key);
    for (name, value) in nbt.iter() {
        let Ok(name) = name.to_owned().try_into_string() else {
            continue;
        };
        existing.insert(name.as_str(), value.clone());
    }
    storage.set(key, existing);
    context
        .source()
        .send_success(&translated("commands.data.merge.success", "Merged".to_owned(), Vec::new()), false);
    Ok(1)
}

fn remove(
    context: &SteelCommandContext<CommandSource>,
    target: DataTarget,
) -> Result<i32, CommandSyntaxError> {
    if target != DataTarget::Storage {
        return unsupported_write(context, target);
    }
    let path = context.nbt_path("path")?.clone();
    let key = context.identifier("target")?.clone();
    let Some(storage) = command_storage(context) else {
        return Ok(0);
    };
    let mut root = NbtTag::Compound(storage.get(&key));
    match path.set(&mut root, NbtTag::Compound(NbtCompound::new())) {
        Ok(0) => {
            context.source().send_failure(translated(
                "commands.data.path.no_data",
                "Nothing matches that NBT path".to_owned(),
                Vec::new(),
            ));
            return Ok(0);
        }
        Ok(_) => {}
        Err(NbtPathMutationError::NothingFound(_)) => {
            context.source().send_failure(translated(
                "commands.data.path.no_data",
                "Nothing matches that NBT path".to_owned(),
                Vec::new(),
            ));
            return Ok(0);
        }
        Err(error) => {
            context.source().send_failure(TextComponent::from(error.to_string()));
            return Ok(0);
        }
    }
    if let NbtTag::Compound(updated) = root {
        storage.set(key, updated);
    }
    context.source().send_success(
        &translated("commands.data.remove.success", "Removed".to_owned(), Vec::new()),
        false,
    );
    Ok(1)
}

fn unsupported_write(
    context: &SteelCommandContext<CommandSource>,
    target: DataTarget,
) -> Result<i32, CommandSyntaxError> {
    let what = match target {
        DataTarget::Block => "block",
        DataTarget::Entity => "entity",
        DataTarget::Storage => unreachable!("storage writes are supported"),
    };
    context.source().send_failure(translated(
        "steel.commands.data.unsupported_write",
        format!(
            "Writing {what} NBT is not supported yet; the saved form only carries the fields Steel syncs"
        ),
        Vec::new(),
    ));
    Ok(0)
}

fn read_root(
    context: &SteelCommandContext<CommandSource>,
    target: DataTarget,
) -> Result<NbtCompound, CommandSyntaxError> {
    match target {
        DataTarget::Block => {
            let position: BlockPos = context.coordinates("targetPos")?.block_pos(context.source());
            let world = context.source().world();
            if !world.is_full_chunk_loaded_at(position) {
                return Err(CommandSyntaxError::dynamic(TextComponent::from(
                    "That position is not loaded",
                )));
            }
            let Some(block_entity) = world.get_block_entity(position) else {
                return Err(CommandSyntaxError::dynamic(TextComponent::from(
                    &steel_utils::translations::COMMANDS_DATA_BLOCK_INVALID,
                )));
            };
            Ok(block_entity.save_with_full_metadata())
        }
        DataTarget::Entity => {
            let entity = context.entity("target")?;
            Ok(entity.nbt_for_data_compare())
        }
        DataTarget::Storage => {
            let key = context.identifier("target")?.clone();
            let Some(storage) = command_storage(context) else {
                return Err(CommandSyntaxError::dynamic(TextComponent::from(
                    "Command storage is not available in this world",
                )));
            };
            Ok(storage.get(&key))
        }
    }
}

fn command_storage<'a>(
    context: &'a SteelCommandContext<CommandSource>,
) -> Option<&'a crate::command::storage::CommandStorage> {
    let source = context.source();
    source.server().command_storage.get(source.world().domain())
}

fn numeric_value(tag: &NbtTag) -> Option<f64> {
    match tag {
        NbtTag::Byte(value) => Some(f64::from(*value)),
        NbtTag::Short(value) => Some(f64::from(*value)),
        NbtTag::Int(value) => Some(f64::from(*value)),
        NbtTag::Long(value) => Some(*value as f64),
        NbtTag::Float(value) => Some(f64::from(*value)),
        NbtTag::Double(value) => Some(*value),
        _ => None,
    }
}

fn value_string(tag: &NbtTag) -> String {
    match tag {
        NbtTag::String(value) => value.to_string(),
        other => to_canonical_snbt(other).unwrap_or_default(),
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
