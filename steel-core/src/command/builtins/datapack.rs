//! `/datapack` — reports the data packs the server has loaded.
//!
//! Vanilla `DatapackCommand` lists packs, enables, disables and reorders them.
//! Steel has one pack: the built-in `minecraft` datapack, compiled into the
//! binary at build time (recipes, loot tables, advancements, functions and the
//! rest of the registry data). There is no folder loading yet, so `enable`,
//! `disable` and `list available` have nothing to operate on; they report that
//! plainly instead of pretending to work.
//!
//! `list` and `list enabled` are real: they report the pack and how many
//! functions and recipes came out of it, which is the useful answer to "did the
//! datapack load at all?".

use std::borrow::Cow;

use steel_utils::Identifier;
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{CommandSource, SteelCommandContext, SteelCommandRuntime, literal},
    registration::CommandRegistration,
};

/// The only data pack the server has: the vanilla one compiled into the binary.
const BUILTIN_PACK: &str = "vanilla";

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("datapack"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("datapack")
        .then(literal("list").executes(|context| list(context, Listing::Enabled)))
        .then(
            literal("list").then(
                literal("enabled").executes(|context| list(context, Listing::Enabled)),
            ),
        )
        .then(
            literal("list").then(
                literal("available").executes(|context| list(context, Listing::Available)),
            ),
        )
        .then(literal("enable").executes(unsupported))
        .then(literal("disable").executes(unsupported))
}

#[derive(Clone, Copy)]
enum Listing {
    Enabled,
    Available,
}

/// Builds a translated component with a literal fallback.
fn translated(key: &'static str, fallback: String, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Borrowed(key),
        fallback: Some(Cow::Owned(fallback)),
        args: Some(args.into_boxed_slice()),
    })
}

fn list(
    context: &SteelCommandContext<CommandSource>,
    listing: Listing,
) -> Result<i32, CommandSyntaxError> {
    // Both lists answer the same here: the built-in pack is the only one, and it
    // is always enabled.
    let _ = listing;
    let functions = steel_registry::functions::FUNCTIONS.len();
    let recipes = steel_registry::REGISTRY.recipes.len();

    let header = translated(
        "commands.datapack.list.enabled.success",
        format!(
            "There is 1 data pack: {BUILTIN_PACK} ({functions} functions, {recipes} recipes)"
        ),
        vec![
            TextComponent::from(BUILTIN_PACK),
            TextComponent::from("1".to_owned()),
        ],
    );
    context.source().send_success(&header, false);
    Ok(1)
}

fn unsupported(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    context.source().send_failure(translated(
        "commands.datapack.unknown",
        "Steel loads one built-in data pack; loading packs from disk is not supported yet"
            .to_owned(),
        Vec::new(),
    ));
    Ok(0)
}
