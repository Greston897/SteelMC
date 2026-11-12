//! Vanilla `/recipe` command.
//!
//! Grants or revokes recipe unlocks for players. Vanilla exposes `give`/`take`
//! with `*` for every recipe and a specific recipe key; Steel's
//! `ServerRecipeBook` already stores exactly that set and already knows how to
//! add every registered recipe, so the command only translates the two forms.

use std::sync::Arc;

use steel_utils::Identifier;
use text_components::TextComponent;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};
use crate::player::Player;

type Builder = CommandNodeBuilder<CommandSource, SteelCommandRuntime>;

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("recipe"), |_| command())
}

fn command() -> Builder {
    literal("recipe").then(
        literal("give")
            .then(targets_then(true))
            .then(literal("take").then(targets_then(false))),
    )
}

/// `give|take <targets> <recipe>`, where `<recipe>` is `*` or a recipe key.
fn targets_then(give: bool) -> Builder {
    argument("targets", SteelArgumentType::players())
        .then(
            literal("*").executes(move |context| apply(context, give, None)),
        )
        .then(
            argument("recipe", SteelArgumentType::identifier())
                .executes(move |context| apply(context, give, Some(context.identifier("recipe")?))),
        )
}

fn apply(
    context: &SteelCommandContext<CommandSource>,
    give: bool,
    key: Option<&Identifier>,
) -> Result<i32, CommandSyntaxError> {
    let targets = context.players("targets")?;

    let mut changed = 0;
    for target in &targets {
        changed += if give {
            give_recipes(target, key)
        } else {
            take_recipes(target, key)
        };
    }

    let action = if give { "Unlocked" } else { "Locked" };
    let message = TextComponent::plain(format!("{action} {changed} recipe(s)"));
    context.source().send_success(&message, true);
    Ok(changed)
}

fn give_recipes(target: &Arc<Player>, key: Option<&Identifier>) -> i32 {
    let mut book = target.recipe_book.lock();
    match key {
        Some(key) => i32::from(book.add(key).is_some()),
        None => i32::try_from(book.add_all_registered()).unwrap_or(i32::MAX),
    }
}

fn take_recipes(target: &Arc<Player>, key: Option<&Identifier>) -> i32 {
    let mut book = target.recipe_book.lock();
    match key {
        Some(key) => i32::from(book.remove(key).is_some()),
        None => {
            let count = i32::try_from(book.len()).unwrap_or(i32::MAX);
            book.clear();
            count
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration_builds() {
        let _ = registration();
    }
}
