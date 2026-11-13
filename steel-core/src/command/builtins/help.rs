//! `/help` — lists the commands the caller can run.
//!
//! Vanilla's `HelpCommand` prints a page of every command the source is allowed
//! to use, seven per page, with `/help [command]` showing one command's usage.
//! Steel's dispatcher already answers permission checks for suggestions, so this
//! command reuses the same list and paginates it the same way.
//!
//! The usage line for a single command is the command name followed by
//! `<...>` for its arguments, which is what vanilla prints when it has no
//! smarter usage string for the command.

use std::borrow::Cow;

use steel_utils::Identifier;
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelCommandContext, SteelCommandRuntime, argument, literal,
    },
    registration::CommandRegistration,
};

const COMMAND_ARGUMENT: &str = "command";
const PAGE_ARGUMENT: &str = "page";

/// Vanilla `HelpCommand.HELP_PER_PAGE`.
const HELP_PER_PAGE: usize = 7;

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("help"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("help")
        .executes(|context| list_page(context, 1))
        .then(argument(PAGE_ARGUMENT, ArgumentType::integer(1, 100)).executes(|context| {
            let page = context.integer(PAGE_ARGUMENT)?;
            list_page(context, page)
        }))
        .then(argument(COMMAND_ARGUMENT, ArgumentType::greedy_string()).executes(describe_command))
}

/// Builds a translated component with a literal fallback.
fn translated(key: &'static str, fallback: String, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Borrowed(key),
        fallback: Some(Cow::Owned(fallback)),
        args: Some(args.into_boxed_slice()),
    })
}

/// Prints one page of the command list.
fn list_page(
    context: &SteelCommandContext<CommandSource>,
    page: i32,
) -> Result<i32, CommandSyntaxError> {
    let server = context.source().server();
    let commands = server.command_names();
    if commands.is_empty() {
        context
            .source()
            .send_failure(translated(
                "commands.help.failed",
                "No commands are available".to_owned(),
                Vec::new(),
            ));
        return Ok(0);
    }

    let page_count = commands.len().div_ceil(HELP_PER_PAGE).max(1);
    let Ok(page_index) = usize::try_from(page) else {
        return Err(unknown_page(page));
    };
    if page_index == 0 || page_index > page_count {
        return Err(unknown_page(page));
    }

    let header = translated(
        "commands.help.header",
        format!("--- Help page {page_index}/{page_count} ---"),
        vec![
            TextComponent::from(page_index.to_string()),
            TextComponent::from(page_count.to_string()),
        ],
    );
    context.source().send_success(&header, false);

    let start = (page_index - 1) * HELP_PER_PAGE;
    for name in commands.iter().skip(start).take(HELP_PER_PAGE) {
        // Vanilla renders each entry as a clickable `/name`; the click event
        // needs the client's chat component handling, which Steel's success
        // feedback already goes through.
        let entry = translated(
            "commands.help.entry",
            format!("/{name}"),
            vec![TextComponent::from(format!("/{name}"))],
        );
        context.source().send_success(&entry, false);
    }

    Ok(commands.len() as i32)
}

/// Prints the usage line of one named command.
fn describe_command(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let name = context.string(COMMAND_ARGUMENT)?.trim().to_owned();
    let server = context.source().server();
    if !server.command_names().iter().any(|known| known == &name) {
        context.source().send_failure(translated(
            "commands.help.failed",
            format!("Unknown command: {name}"),
            vec![TextComponent::from(name)],
        ));
        return Ok(0);
    }

    let message = translated(
        "commands.help.usage",
        format!("Usage: /{name}"),
        vec![TextComponent::from(format!("/{name}"))],
    );
    context.source().send_success(&message, false);
    Ok(1)
}

fn unknown_page(page: i32) -> CommandSyntaxError {
    CommandSyntaxError::dynamic(translated(
        "commands.help.failed",
        format!("Unknown help page {page}"),
        vec![TextComponent::from(page.to_string())],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_list_shorter_than_a_page_is_one_page() {
        for count in 0..=HELP_PER_PAGE {
            let pages = count.div_ceil(HELP_PER_PAGE).max(1);
            assert_eq!(pages, 1, "{count} commands should fit on one page");
        }
    }

    #[test]
    fn pages_grow_one_per_seven_commands() {
        assert_eq!(HELP_PER_PAGE, 7);
        assert_eq!(8_usize.div_ceil(HELP_PER_PAGE), 2);
        assert_eq!(14_usize.div_ceil(HELP_PER_PAGE), 2);
        assert_eq!(15_usize.div_ceil(HELP_PER_PAGE), 3);
    }

    #[test]
    fn page_index_maps_to_the_right_slice() {
        let commands: Vec<String> = (0..20).map(|index| format!("cmd{index}")).collect();
        let page = 2_usize;
        let start = (page - 1) * HELP_PER_PAGE;
        let slice: Vec<_> = commands.iter().skip(start).take(HELP_PER_PAGE).collect();
        assert_eq!(slice.len(), HELP_PER_PAGE);
        assert_eq!(slice[0], "cmd7");
    }
}
