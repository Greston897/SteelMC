//! Vanilla entity rotation command.

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
use crate::entity::{Entity, EntityAnchor, SharedEntity};

/// Vanilla `RotateCommand` requires `LEVEL_GAMEMASTERS`, which Steel models as
/// the operator permission of the command itself.
pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("rotate"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("rotate").then(
        argument("target", SteelArgumentType::entity())
            .then(argument("rotation", SteelArgumentType::rotation()).executes(rotate_to_rotation))
            .then(
                literal("facing")
                    .then(
                        literal("entity").then(
                            argument("facingEntity", SteelArgumentType::entity())
                                .executes(rotate_facing_entity_feet)
                                .then(
                                    argument("facingAnchor", SteelArgumentType::entity_anchor())
                                        .executes(rotate_facing_entity_anchor),
                                ),
                        ),
                    )
                    .then(
                        argument("facingLocation", SteelArgumentType::vec3(true))
                            .executes(rotate_facing_position),
                    ),
            ),
    )
}

fn rotate_to_rotation(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let target = context.entity("target")?;
    let rotation = context.coordinates("rotation")?;
    let (current_yaw, current_pitch) = Entity::rotation(target.as_ref());
    let resolved = rotation.rotation(context.source());

    // Vanilla `RotateCommand.rotate`: the argument resolves against the source and
    // is turned back into an offset so the client can apply it relatively.
    let relative_yaw = rotation.is_y_relative();
    let relative_pitch = rotation.is_x_relative();
    let yaw = if relative_yaw {
        resolved.0 - current_yaw
    } else {
        resolved.0
    };
    let pitch = if relative_pitch {
        resolved.1 - current_pitch
    } else {
        resolved.1
    };

    Entity::force_set_rotation(target.as_ref(), yaw, relative_yaw, pitch, relative_pitch);
    send_success(context.source(), &target);
    Ok(1)
}

fn rotate_facing_entity_feet(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    rotate_facing_entity(context, EntityAnchor::Feet)
}

fn rotate_facing_entity_anchor(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let anchor = context.entity_anchor("facingAnchor")?;
    rotate_facing_entity(context, anchor)
}

fn rotate_facing_entity(
    context: &SteelCommandContext<CommandSource>,
    anchor: EntityAnchor,
) -> Result<i32, CommandSyntaxError> {
    let target = context.entity("target")?;
    let facing_entity = context.entity("facingEntity")?;
    Entity::look_at_entity(
        target.as_ref(),
        context.source().anchor(),
        facing_entity.as_ref(),
        anchor,
    );
    send_success(context.source(), &target);
    Ok(1)
}

fn rotate_facing_position(
    context: &SteelCommandContext<CommandSource>,
) -> Result<i32, CommandSyntaxError> {
    let target = context.entity("target")?;
    let position = context
        .coordinates("facingLocation")?
        .position(context.source());
    Entity::look_at(target.as_ref(), context.source().anchor(), position);
    send_success(context.source(), &target);
    Ok(1)
}

fn send_success(source: &CommandSource, target: &SharedEntity) {
    let message = TextComponent::translated(TranslatedMessage {
        key: "commands.rotate.success".into(),
        fallback: Some("Rotated %s".into()),
        args: Some(Box::new([TextComponent::plain(target.plain_text_name())])),
    });
    source.send_success(&message, true);
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
    fn rotate_graph_matches_vanilla_rotation_and_facing_shapes() {
        init_vanilla_registry();
        let Ok(dispatcher) = create_dispatcher() else {
            panic!("built-in commands should register");
        };

        let root = child(&dispatcher, dispatcher.root(), "rotate");
        let target = child(&dispatcher, root, "target");
        let rotation = child(&dispatcher, target, "rotation");
        assert!(matches!(
            dispatcher.node(rotation),
            Some(node)
                if node.is_executable()
                    && node.argument_type() == Some(&SteelArgumentType::rotation())
        ));

        let facing = child(&dispatcher, target, "facing");
        let entity = child(&dispatcher, facing, "entity");
        let facing_entity = child(&dispatcher, entity, "facingEntity");
        assert!(matches!(
            dispatcher.node(facing_entity),
            Some(node)
                if node.is_executable()
                    && node.argument_type() == Some(&SteelArgumentType::entity())
        ));
        let anchor = child(&dispatcher, facing_entity, "facingAnchor");
        assert!(matches!(
            dispatcher.node(anchor),
            Some(node)
                if node.is_executable()
                    && node.argument_type() == Some(&SteelArgumentType::entity_anchor())
        ));
        let facing_location = child(&dispatcher, facing, "facingLocation");
        assert!(matches!(
            dispatcher.node(facing_location),
            Some(node) if node.is_executable()
        ));
    }
}
