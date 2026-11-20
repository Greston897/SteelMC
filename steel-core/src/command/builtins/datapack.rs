//! `/datapack` lists baked vanilla resources and loaded directory function packs.
//! Enable/disable and other runtime data registries remain explicitly unsupported.

use std::borrow::Cow;
use std::iter::once;

use steel_utils::Identifier;
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{CommandSource, SteelCommandContext, SteelCommandRuntime, literal},
    registration::CommandRegistration,
};

/// The always-enabled vanilla data compiled into the binary.
const BUILTIN_PACK: &str = "vanilla";

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("datapack"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("datapack")
        .then(
            literal("list")
                .executes(|context| list(context, Listing::Enabled))
                .then(literal("enabled").executes(|context| list(context, Listing::Enabled)))
                .then(literal("available").executes(|context| list(context, Listing::Available))),
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
    let packs = context.source().server().function_packs.pack_names();
    let names = match listing {
        Listing::Enabled => once(BUILTIN_PACK.to_owned())
            .chain(packs)
            .collect::<Vec<_>>(),
        Listing::Available => Vec::new(),
    };
    let count = names.len();

    let header = translated(
        match listing {
            Listing::Enabled => "commands.datapack.list.enabled.success",
            Listing::Available => "commands.datapack.list.available.success",
        },
        format!(
            "{count} data pack(s): {}. Directory packs currently load functions and function tags only",
            names.join(", ")
        ),
        vec![
            TextComponent::from(BUILTIN_PACK),
            TextComponent::from(count.to_string()),
        ],
    );
    context.source().send_success(&header, false);
    Ok(count as i32)
}

fn unsupported(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    context.source().send_failure(translated(
        "commands.datapack.unknown",
        "Directory-pack functions load at startup and /reload; enable/disable ordering is not implemented"
            .to_owned(),
        Vec::new(),
    ));
    Ok(0)
}
