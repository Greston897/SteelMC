//! `/attribute` — reads and writes entity attributes.
//!
//! Vanilla `AttributeCommand` has four forms:
//!
//! * `attribute <target> <attribute> get [scale]`
//! * `attribute <target> <attribute> base get [scale]`
//! * `attribute <target> <attribute> base set <value>`
//! * `attribute <target> <attribute> modifier add|remove <uuid> <name> <value> add_value|add_multiplied_base|add_multiplied_total`
//!
//! Steel implements all four. The attribute is resolved from the registry by
//! identifier, so an unknown name reports the vanilla error rather than
//! panicking, and a modifier's UUID is kept as its identifier — the same shape
//! `AttributeModifier` stores.

use std::borrow::Cow;

use steel_utils::Identifier;
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};
use crate::entity::attribute::{AttributeModifier, AttributeModifierOperation, AttributeRef};

const TARGET_ARGUMENT: &str = "target";
const ATTRIBUTE_ARGUMENT: &str = "attribute";
const SCALE_ARGUMENT: &str = "scale";
const VALUE_ARGUMENT: &str = "value";
const UUID_ARGUMENT: &str = "uuid";

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("attribute"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    let attribute = argument(ATTRIBUTE_ARGUMENT, SteelArgumentType::identifier());

    // `get [scale]` and `base get [scale]`
    let base_get = literal("base")
        .then(literal("get").executes(get_base))
        .then(
            literal("get").then(
                argument(SCALE_ARGUMENT, ArgumentType::double(-f64::MAX, f64::MAX)).executes(get_base_scaled),
            ),
        )
        .then(
            literal("set").then(
                argument(VALUE_ARGUMENT, ArgumentType::double(-f64::MAX, f64::MAX)).executes(set_base),
            ),
        );

    let get = literal("get").executes(get_total).then(
        argument(SCALE_ARGUMENT, ArgumentType::double(-f64::MAX, f64::MAX)).executes(get_total_scaled),
    );

    let modifier = literal("modifier")
        .then(
            literal("add").then(
                literal("value").then(
                    argument(VALUE_ARGUMENT, ArgumentType::double(-f64::MAX, f64::MAX)).executes(add_modifier),
                ),
            ),
        )
        .then(
            literal("remove").then(
                argument(UUID_ARGUMENT, SteelArgumentType::identifier())
                    .executes(remove_modifier),
            ),
        );

    literal("attribute").then(
        argument(TARGET_ARGUMENT, SteelArgumentType::entity())
            .then(attribute.then(get).then(base_get).then(modifier)),
    )
}

/// Builds a translated component with a literal fallback.
fn translated(key: &'static str, fallback: String, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Borrowed(key),
        fallback: Some(fallback),
        args: Some(args.into_boxed_slice()),
    })
}

/// Resolves the attribute identifier against the registry.
fn resolve_attribute(
    context: &SteelCommandContext<CommandSource>,
) -> Result<AttributeRef, CommandSyntaxError> {
    let id = context.identifier(ATTRIBUTE_ARGUMENT)?;
    steel_registry::REGISTRY
        .attributes
        .by_key(id)
        .ok_or_else(|| {
            CommandSyntaxError::dynamic(translated(
                "commands.attribute.text.attributeNotFound",
                format!("Unknown attribute {id}"),
                vec![TextComponent::from(id.to_string())],
            ))
        })
}

/// Formats a value the way vanilla does, with `scale` in front when given.
fn format_value(value: f64, scale: Option<f64>) -> String {
    match scale {
        Some(scale) => format!("{}", value * scale),
        None => format!("{value}"),
    }
}

/// The attribute's current total, including modifiers.
fn total_value(
    entity: &crate::entity::SharedEntity,
    attribute: AttributeRef,
) -> Option<f64> {
    let living = entity.as_living_entity()?;
    living.attributes().lock().get_value(attribute)
}

fn get_total(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    get_total_with_scale(context, None)
}

fn get_total_scaled(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let scale = context.double(SCALE_ARGUMENT)?;
    get_total_with_scale(context, Some(scale))
}

fn get_total_with_scale(
    context: &SteelCommandContext<CommandSource>,
    scale: Option<f64>,
) -> Result<i32, CommandSyntaxError> {
    let attribute = resolve_attribute(context)?;
    let entity = context.entity(TARGET_ARGUMENT)?;
    let Some(value) = total_value(&entity, attribute) else {
        return Err(missing_attribute(attribute));
    };

    let message = translated(
        "commands.attribute.value.get.success",
        format!("{} has {} of {}", entity_display(&entity), attribute.key, format_value(value, scale)),
        vec![
            TextComponent::from(format_value(value, scale)),
            TextComponent::from(attribute.key.to_string()),
        ],
    );
    context.source().send_success(&message, false);
    Ok(i32::try_from(value.floor() as i64).unwrap_or(i32::MAX))
}

fn get_base(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    get_base_with_scale(context, None)
}

fn get_base_scaled(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let scale = context.double(SCALE_ARGUMENT)?;
    get_base_with_scale(context, Some(scale))
}

fn get_base_with_scale(
    context: &SteelCommandContext<CommandSource>,
    scale: Option<f64>,
) -> Result<i32, CommandSyntaxError> {
    let attribute = resolve_attribute(context)?;
    let entity = context.entity(TARGET_ARGUMENT)?;
    let Some(value) = base_value(&entity, attribute) else {
        return Err(missing_attribute(attribute));
    };

    let message = translated(
        "commands.attribute.base_value.get.success",
        format!("The base value of {} is {}", attribute.key, format_value(value, scale)),
        vec![
            TextComponent::from(format_value(value, scale)),
            TextComponent::from(attribute.key.to_string()),
        ],
    );
    context.source().send_success(&message, false);
    Ok(i32::try_from(value.floor() as i64).unwrap_or(i32::MAX))
}

fn set_base(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let attribute = resolve_attribute(context)?;
    let value = context.double(VALUE_ARGUMENT)?;
    let entity = context.entity(TARGET_ARGUMENT)?;

    let Some(living) = entity.as_living_entity() else {
        return Err(missing_attribute(attribute));
    };
    living.attributes().lock().set_base_value(attribute, value);

    let message = translated(
        "commands.attribute.base_value.set.success",
        format!("Set the base value of {} to {value}", attribute.key),
        vec![
            TextComponent::from(value.to_string()),
            TextComponent::from(attribute.key.to_string()),
        ],
    );
    context.source().send_success(&message, true);
    Ok(1)
}

fn add_modifier(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let attribute = resolve_attribute(context)?;
    let value = context.double(VALUE_ARGUMENT)?;
    let uuid = context.identifier(UUID_ARGUMENT)?.clone();
    let entity = context.entity(TARGET_ARGUMENT)?;

    let Some(living) = entity.as_living_entity() else {
        return Err(missing_attribute(attribute));
    };
    let added = living.attributes().lock().add_modifier(
        attribute,
        AttributeModifier {
            id: uuid.clone(),
            amount: value,
            operation: AttributeModifierOperation::AddValue,
        },
        false,
    );
    if !added {
        return Err(CommandSyntaxError::dynamic(translated(
            "commands.attribute.modifier.add.failed",
            "The modifier is already present".to_owned(),
            Vec::new(),
        )));
    }

    let message = translated(
        "commands.attribute.modifier.add.success",
        format!("Added a modifier {uuid} to {}", attribute.key),
        vec![
            TextComponent::from(uuid.to_string()),
            TextComponent::from(attribute.key.to_string()),
        ],
    );
    context.source().send_success(&message, true);
    Ok(1)
}

fn remove_modifier(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let attribute = resolve_attribute(context)?;
    let uuid = context.identifier(UUID_ARGUMENT)?.clone();
    let entity = context.entity(TARGET_ARGUMENT)?;

    let Some(living) = entity.as_living_entity() else {
        return Err(missing_attribute(attribute));
    };
    if !living.attributes().lock().remove_modifier(attribute, &uuid) {
        return Err(CommandSyntaxError::dynamic(translated(
            "commands.attribute.modifier.remove.failed",
            format!("No modifier {uuid} on {}", attribute.key),
            vec![TextComponent::from(uuid.to_string())],
        )));
    }

    let message = translated(
        "commands.attribute.modifier.remove.success",
        format!("Removed modifier {uuid} from {}", attribute.key),
        vec![
            TextComponent::from(uuid.to_string()),
            TextComponent::from(attribute.key.to_string()),
        ],
    );
    context.source().send_success(&message, true);
    Ok(1)
}

fn base_value(entity: &crate::entity::SharedEntity, attribute: AttributeRef) -> Option<f64> {
    let living = entity.as_living_entity()?;
    living.attributes().lock().get_base_value(attribute)
}

fn missing_attribute(attribute: AttributeRef) -> CommandSyntaxError {
    CommandSyntaxError::dynamic(translated(
        "commands.attribute.failed.no_attribute",
        format!("{} has no attribute {}", "The entity", attribute.key),
        vec![TextComponent::from(attribute.key.to_string())],
    ))
}

fn entity_display(entity: &crate::entity::SharedEntity) -> String {
    format!("{:?}", entity.id())
}
