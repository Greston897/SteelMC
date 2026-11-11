//! Steel-specific `/tpsbar` command.
//!
//! Toggles a server-wide boss bar that reports TPS and three MSPT views
//! (recent, five second average, p95). The bar itself is broadcast from the
//! server tick loop on the tab list cadence, so the command only flips the flag
//! and answers with the new state.

use steel_utils::Identifier;
use text_components::TextComponent;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{CommandSource, SteelCommandContext, SteelCommandRuntime, literal},
    registration::CommandRegistration,
};

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("tpsbar"), |_| command()).default_access()
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("tpsbar").executes(toggle_tps_bar)
}

/// Toggles the boss bar and reports the new state.
#[expect(
    clippy::unnecessary_wraps,
    reason = "Command executors use a shared fallible callback signature."
)]
fn toggle_tps_bar(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let enabled = context.source().server().toggle_tps_bar();
    let message = if enabled {
        TextComponent::plain("TPS bar enabled")
    } else {
        TextComponent::plain("TPS bar disabled")
    };
    context.source().send_success(&message, false);
    Ok(i32::from(enabled))
}

#[cfg(test)]
mod tests {
    use super::super::create_dispatcher;

    /// `/tpsbar` is part of the built-in graph and executable without arguments.
    #[test]
    fn tpsbar_is_registered_and_executable() {
        steel_registry::init_vanilla_registry();
        let Ok(dispatcher) = create_dispatcher() else {
            panic!("built-in commands should register");
        };
        let Some(roots) = dispatcher.children(dispatcher.root()) else {
            panic!("dispatcher root should exist");
        };
        let Some(tpsbar) = roots.iter().copied().find(|root| {
            dispatcher
                .node(*root)
                .is_some_and(|node| node.name() == "tpsbar")
        }) else {
            panic!("tpsbar root should exist");
        };
        let Some(node) = dispatcher.node(tpsbar) else {
            panic!("tpsbar node should exist");
        };
        assert!(
            node.is_executable(),
            "tpsbar should be executable without arguments"
        );
    }
}
