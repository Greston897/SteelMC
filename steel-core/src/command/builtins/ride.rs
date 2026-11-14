//! Vanilla `/ride` command.

use steel_utils::Identifier;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};
use crate::entity::Entity as _;

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("ride"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("ride").then(
        argument("target", SteelArgumentType::entity())
            .then(literal("mount").then(
                argument("vehicle", SteelArgumentType::entity()).executes(mount),
            ))
            .then(literal("dismount").executes(dismount)),
    )
}

fn mount(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let target = context.entity("target")?;
    let vehicle = context.entity("vehicle")?;
    if target.start_riding(&vehicle) {
        Ok(1)
    } else {
        Err(CommandSyntaxError::dynamic(
            "The entity could not start riding the vehicle",
        ))
    }
}

fn dismount(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let target = context.entity("target")?;
    if !target.is_passenger() {
        return Err(CommandSyntaxError::dynamic("The entity is not riding"));
    }
    target.stop_riding();
    Ok(1)
}
