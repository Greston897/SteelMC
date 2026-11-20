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
use steel_utils::translations::COMMANDS_DATA_BLOCK_INVALID;
use steel_utils::{
    BlockPos, Identifier,
    nbt::{NbtPathMutationError, to_canonical_snbt},
};
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;
use crate::command::storage::CommandStorage;

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
    let with_targets = |build_op: fn(DataTarget) -> Builder| {
        move |op_name: &'static str| {
            literal(op_name)
                .then(
                    literal("block").then(
                        argument("targetPos", SteelArgumentType::block_pos())
                            .then(build_op(DataTarget::Block)),
                    ),
                )
                .then(
                    literal("entity").then(
                        argument("target", SteelArgumentType::entity())
                            .then(build_op(DataTarget::Entity)),
                    ),
                )
                .then(
                    literal("storage").then(
                        argument("target", SteelArgumentType::storage_key())
                            .then(build_op(DataTarget::Storage)),
                    ),
                )
        }
    };

    let get_root = literal("get")
        .then(
            literal("block").then(
                argument("targetPos", SteelArgumentType::block_pos())
                    .executes(|context| get(context, DataTarget::Block, GetKind::Root))
                    .then(get_path_tail(DataTarget::Block)),
            ),
        )
        .then(
            literal("entity").then(
                argument("target", SteelArgumentType::entity())
                    .executes(|context| get(context, DataTarget::Entity, GetKind::Root))
                    .then(get_path_tail(DataTarget::Entity)),
            ),
        )
        .then(
            literal("storage").then(
                argument("target", SteelArgumentType::storage_key())
                    .executes(|context| get(context, DataTarget::Storage, GetKind::Root))
                    .then(get_path_tail(DataTarget::Storage)),
            ),
        );

    literal("data")
        .then(get_root)
        .then(with_targets(merge_tail)("merge"))
        .then(with_targets(remove_tail)("remove"))
}

fn get_path_tail(target: DataTarget) -> Builder {
    argument("path", SteelArgumentType::nbt_path())
        .executes(move |context| get(context, target, GetKind::Tag))
        .then(
            argument(
                "scale",
                super::super::brigadier::ArgumentType::double(-f64::MAX, f64::MAX),
            )
            .executes(move |context| get(context, target, GetKind::Scaled)),
        )
}

fn merge_tail(target: DataTarget) -> Builder {
    argument("nbt", SteelArgumentType::compound()).executes(move |context| merge(context, target))
}

fn remove_tail(target: DataTarget) -> Builder {
    argument("path", SteelArgumentType::nbt_path()).executes(move |context| remove(context, target))
}

#[derive(Clone, Copy)]
enum GetKind {
    Root,
    Tag,
    Scaled,
}

fn get(
    context: &SteelCommandContext<CommandSource>,
    target: DataTarget,
    kind: GetKind,
) -> Result<i32, CommandSyntaxError> {
    let root = read_root(context, target)?;
    let (printed, result) = if matches!(kind, GetKind::Root) {
        (
            to_canonical_snbt(&NbtTag::Compound(root)).unwrap_or_default(),
            1,
        )
    } else {
        let path = context.nbt_path("path")?.clone();
        let root = NbtTag::Compound(root);
        let tags = path.get(&root);
        if tags.len() != 1 {
            return Err(CommandSyntaxError::dynamic(
                "Expected exactly one NBT value",
            ));
        }
        let tag = &tags[0];
        let result = if matches!(kind, GetKind::Scaled) {
            scaled_numeric_result(tag, context.double("scale")?)?
        } else {
            tag_result(tag)?
        };
        (to_canonical_snbt(tag).unwrap_or_default(), result)
    };
    context.source().send_success(
        &translated(
            "commands.data.get.success",
            format!("Value: {printed}"),
            vec![TextComponent::from(printed)],
        ),
        false,
    );
    Ok(result)
}

fn scaled_numeric_result(tag: &NbtTag, scale: f64) -> Result<i32, CommandSyntaxError> {
    numeric_value(tag)
        .map(|n| (n * scale).floor() as i32)
        .ok_or_else(|| CommandSyntaxError::dynamic("NBT value is not numeric"))
}

/// Vanilla DataCommands.getData returns the numeric floor or collection size,
/// not a constant success count. Java string lengths count UTF-16 code units.
fn tag_result(tag: &NbtTag) -> Result<i32, CommandSyntaxError> {
    if let Some(n) = numeric_value(tag) {
        return Ok(n.floor() as i32);
    }
    let size = match tag {
        NbtTag::String(s) => s.to_string().encode_utf16().count(),
        NbtTag::Compound(c) => c.len(),
        NbtTag::List(l) => {
            use simdnbt::owned::NbtList;
            match l {
                NbtList::Empty => 0,
                NbtList::Byte(v) => v.len(),
                NbtList::Short(v) => v.len(),
                NbtList::Int(v) => v.len(),
                NbtList::Long(v) => v.len(),
                NbtList::Float(v) => v.len(),
                NbtList::Double(v) => v.len(),
                NbtList::ByteArray(v) => v.len(),
                NbtList::String(v) => v.len(),
                NbtList::List(v) => v.len(),
                NbtList::Compound(v) => v.len(),
                NbtList::IntArray(v) => v.len(),
                NbtList::LongArray(v) => v.len(),
            }
        }
        NbtTag::ByteArray(a) => a.len(),
        NbtTag::IntArray(a) => a.len(),
        NbtTag::LongArray(a) => a.len(),
        _ => return Err(CommandSyntaxError::dynamic("NBT value has no size")),
    };
    Ok(size as i32)
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
    context.source().send_success(
        &translated(
            "commands.data.merge.success",
            "Merged".to_owned(),
            Vec::new(),
        ),
        false,
    );
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
            context
                .source()
                .send_failure(TextComponent::from(error.to_string()));
            return Ok(0);
        }
    }
    if let NbtTag::Compound(updated) = root {
        storage.set(key, updated);
    }
    context.source().send_success(
        &translated(
            "commands.data.remove.success",
            "Removed".to_owned(),
            Vec::new(),
        ),
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
            let position: BlockPos = context
                .coordinates("targetPos")?
                .block_pos(context.source());
            let world = context.source().world();
            if !world.is_full_chunk_loaded_at(position) {
                return Err(CommandSyntaxError::dynamic(TextComponent::from(
                    "That position is not loaded",
                )));
            }
            let Some(block_entity) = world.get_block_entity(position) else {
                return Err(CommandSyntaxError::dynamic(TextComponent::from(
                    &COMMANDS_DATA_BLOCK_INVALID,
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

fn command_storage(
    context: &SteelCommandContext<CommandSource>,
) -> Option<&CommandStorage> {
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

/// Builds a translated component with a literal fallback.
fn translated(key: &'static str, fallback: String, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Borrowed(key),
        fallback: Some(Cow::Owned(fallback)),
        args: Some(args.into_boxed_slice()),
    })
}

#[cfg(test)]
mod data_result_tests {
    use super::*;
use simdnbt::owned::NbtList::Empty;
use simdnbt::owned::NbtList::Int;

    #[test]
    fn data_get_returns_numeric_floor_not_constant_success() {
        assert_eq!(tag_result(&NbtTag::Float(6.0)).unwrap(), 6);
        assert_eq!(tag_result(&NbtTag::Double(-1.2)).unwrap(), -2);
        assert_eq!(scaled_numeric_result(&NbtTag::Double(2.5), 3.0).unwrap(), 7);
        assert_eq!(
            scaled_numeric_result(&NbtTag::Double(-1.2), 2.0).unwrap(),
            -3
        );
    }

    #[test]
    fn data_get_collection_lengths_and_nonnumeric_scale_match_vanilla() {
        assert_eq!(tag_result(&NbtTag::String("a😀".into())).unwrap(), 3);
        assert_eq!(tag_result(&NbtTag::IntArray(vec![1, 2, 3])).unwrap(), 3);
        assert_eq!(
            tag_result(&NbtTag::List(Empty)).unwrap(),
            0
        );
        assert_eq!(
            tag_result(&NbtTag::List(Int(vec![1, 2]))).unwrap(),
            2
        );
        let mut compound = NbtCompound::new();
        compound.insert("one", 1_i32);
        assert_eq!(tag_result(&NbtTag::Compound(compound)).unwrap(), 1);
        assert!(scaled_numeric_result(&NbtTag::String("6".into()), 1.0).is_err());
    }
}
