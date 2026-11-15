//! Vanilla `/posteffect` command.
//!
//! `PostEffectCommand` manipulates the per-player `post_effects` list that the
//! client applies as a full-screen shader pipeline. Every form requires
//! `LEVEL_GAMEMASTERS`, which Steel models through the command's own permission.

use std::sync::Arc;

use steel_utils::Identifier;
use text_components::{TextComponent, translation::TranslatedMessage};

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};
use crate::entity::Entity as _;
use crate::player::Player;

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("posteffect"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("posteffect")
        .then(
            literal("add").then(
                argument("targets", SteelArgumentType::players()).then(
                    argument("posteffect", SteelArgumentType::identifier())
                        .executes(add_post_effect),
                ),
            ),
        )
        .then(
            literal("clear").then(
                argument("targets", SteelArgumentType::players()).executes(clear_post_effects),
            ),
        )
        .then(
            literal("list").then(
                argument("target", SteelArgumentType::player()).executes(list_post_effects),
            ),
        )
        .then(
            literal("remove").then(
                argument("targets", SteelArgumentType::players()).then(
                    argument("posteffect", SteelArgumentType::identifier())
                        .executes(remove_post_effect),
                ),
            ),
        )
}

/// Resolves the `posteffect` argument, keeping vanilla's registry-key form.
fn post_effect_id(
    context: &SteelCommandContext<CommandSource>,
) -> Result<Identifier, CommandSyntaxError> {
    Ok(context.identifier("posteffect")?.clone())
}

fn targets(
    context: &SteelCommandContext<CommandSource>,
) -> Result<Vec<Arc<Player>>, CommandSyntaxError> {
    context.players("targets")
}

/// Vanilla `CommandResponseTracker`: counts the players a form changed and
/// keeps the only one of them, which selects the singular message.
#[derive(Default)]
struct Tracker {
    changed: i32,
    only: Option<Arc<Player>>,
}

impl Tracker {
    fn track(&mut self, player: &Arc<Player>, changed: bool) {
        if !changed {
            return;
        }
        self.changed += 1;
        self.only = if self.changed == 1 {
            Some(Arc::clone(player))
        } else {
            None
        };
    }
}

fn add_post_effect(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let effect = post_effect_id(context)?;
    let players = targets(context)?;

    let mut tracker = Tracker::default();
    for player in &players {
        let added = player.add_post_effect(effect.clone());
        tracker.track(player, added);
    }

    if tracker.changed == 0 {
        return Err(CommandSyntaxError::dynamic(translated(
            "commands.posteffect.add.failed",
            "Player already has the specified post effect",
            Vec::new(),
        )));
    }

    let message = if let Some(player) = &tracker.only {
        translated(
            "commands.posteffect.add.success.single",
            "Added post effect %s to %s",
            vec![
                TextComponent::plain(effect.to_string()),
                player.display_name(),
            ],
        )
    } else {
        translated(
            "commands.posteffect.add.success.multiple",
            "Added post effect %s to %s players",
            vec![
                TextComponent::plain(effect.to_string()),
                TextComponent::plain(tracker.changed.to_string()),
            ],
        )
    };
    context.source().send_success(&message, true);
    Ok(tracker.changed)
}

fn remove_post_effect(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let effect = post_effect_id(context)?;
    let players = targets(context)?;

    let mut tracker = Tracker::default();
    for player in &players {
        let removed = player.remove_post_effect(&effect);
        tracker.track(player, removed);
    }

    if tracker.changed == 0 {
        return Err(CommandSyntaxError::dynamic(translated(
            "commands.posteffect.remove.failed",
            "Player does not have the specified post effect",
            Vec::new(),
        )));
    }

    let message = if let Some(player) = &tracker.only {
        translated(
            "commands.posteffect.remove.success.single",
            "Removed post effect %s from %s",
            vec![
                TextComponent::plain(effect.to_string()),
                player.display_name(),
            ],
        )
    } else {
        translated(
            "commands.posteffect.remove.success.multiple",
            "Removed post effect %s from %s players",
            vec![
                TextComponent::plain(effect.to_string()),
                TextComponent::plain(tracker.changed.to_string()),
            ],
        )
    };
    context.source().send_success(&message, true);
    Ok(tracker.changed)
}

fn clear_post_effects(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let players = targets(context)?;

    let mut tracker = Tracker::default();
    for player in &players {
        let cleared = player.clear_post_effects();
        tracker.track(player, cleared);
    }

    if tracker.changed == 0 {
        return Err(CommandSyntaxError::dynamic(translated(
            "commands.posteffect.clear.failed",
            "Player does not have any post effects to remove",
            Vec::new(),
        )));
    }

    let message = if let Some(player) = &tracker.only {
        translated(
            "commands.posteffect.clear.success.single",
            "Removed all post effects from %s",
            vec![player.display_name()],
        )
    } else {
        translated(
            "commands.posteffect.clear.success.multiple",
            "Removed all post effects from %s players",
            vec![TextComponent::plain(tracker.changed.to_string())],
        )
    };
    context.source().send_success(&message, true);
    Ok(tracker.changed)
}

fn list_post_effects(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let target = context.player("target")?;
    let effects = target.post_effects();
    let message = if effects.is_empty() {
        translated(
            "commands.posteffect.list.empty",
            "Player %s does not have any post effects",
            vec![target.display_name()],
        )
    } else {
        let names = effects
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        translated(
            "commands.posteffect.list.success",
            "Player %s has %s post effect(s): %s",
            vec![
                target.display_name(),
                TextComponent::plain(effects.len().to_string()),
                TextComponent::plain(names),
            ],
        )
    };
    context.source().send_success(&message, false);
    Ok(i32::try_from(effects.len()).unwrap_or(i32::MAX))
}

fn translated(key: &'static str, fallback: &'static str, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: key.into(),
        fallback: Some(fallback.into()),
        args: if args.is_empty() {
            None
        } else {
            Some(args.into_boxed_slice())
        },
    })
}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;

    use super::super::create_dispatcher;
    use crate::command::{
        brigadier::{CommandDispatcher, NodeId},
        execution::{CommandSource, SteelArgumentType, SteelCommandRuntime},
    };

    type Dispatcher = CommandDispatcher<CommandSource, SteelCommandRuntime>;

    fn child(dispatcher: &Dispatcher, parent: NodeId, name: &str) -> NodeId {
        let Some(children) = dispatcher.children(parent) else {
            panic!("parent node should exist");
        };
        let Some(child) = children.iter().copied().find(|child| {
            dispatcher
                .node(*child)
                .is_some_and(|node| node.name() == name)
        }) else {
            panic!("child {name} should exist");
        };
        child
    }

    #[test]
    fn posteffect_graph_matches_vanilla_shapes() {
        init_vanilla_registry();
        let Ok(dispatcher) = create_dispatcher() else {
            panic!("built-in commands should register");
        };

        let root = child(&dispatcher, dispatcher.root(), "posteffect");
        for (branch, terminal) in [("add", "posteffect"), ("remove", "posteffect")] {
            let branch_node = child(&dispatcher, root, branch);
            let targets = child(&dispatcher, branch_node, "targets");
            assert!(matches!(
                dispatcher.node(targets),
                Some(node) if node.argument_type() == Some(&SteelArgumentType::players())
            ));
            let effect = child(&dispatcher, targets, terminal);
            assert!(matches!(
                dispatcher.node(effect),
                Some(node)
                    if node.is_executable()
                        && node.argument_type() == Some(&SteelArgumentType::identifier())
            ));
        }

        let clear = child(&dispatcher, root, "clear");
        let targets = child(&dispatcher, clear, "targets");
        assert!(matches!(
            dispatcher.node(targets),
            Some(node) if node.is_executable()
        ));

        let list = child(&dispatcher, root, "list");
        let target = child(&dispatcher, list, "target");
        assert!(matches!(
            dispatcher.node(target),
            Some(node)
                if node.is_executable()
                    && node.argument_type() == Some(&SteelArgumentType::player())
        ));
    }
}
