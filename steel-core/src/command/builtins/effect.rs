//! Vanilla `/effect` command.

use steel_utils::Identifier;
use text_components::TextComponent;

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};
use crate::entity::MobEffectInstance;
use crate::entity::{Entity, LivingEntity};
use steel_registry::{REGISTRY, RegistryExt as _};

/// Vanilla `MobEffectInstance.INFINITE_DURATION`.
const INFINITE_DURATION: i32 = -1;

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("effect"), |_| command())
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("effect")
        .then(
            literal("give").then(
                argument("targets", SteelArgumentType::entities())
                    .then(
                        argument("effect", ArgumentType::word())
                            .then(
                                argument("seconds", ArgumentType::integer(1, 1_000_000))
                                    .then(
                                        argument(
                                            "amplifier",
                                            ArgumentType::integer(0, 255),
                                        )
                                        .executes(|context| give(context, false)),
                                    )
                                    .executes(|context| give(context, false)),
                            )
                            .executes(give_infinite),
                    ),
            ),
        )
        .then(
            literal("clear").then(
                argument("targets", SteelArgumentType::entities())
                    .then(
                        argument("effect", ArgumentType::word())
                            .executes(clear_one),
                    )
                    .executes(clear_all),
            ),
        )
}

fn effect_of(
    context: &SteelCommandContext<CommandSource>,
) -> Result<steel_registry::mob_effect::MobEffectRef, CommandSyntaxError> {
    let name = context.string("effect")?;
    let key = parse_effect_key(name)?;
    REGISTRY
        .mob_effects
        .by_key(&key)
        .ok_or_else(|| CommandSyntaxError::dynamic(format!("Unknown effect '{name}'")))
}

/// Accepts both `speed` and `minecraft:speed`, like vanilla's resource argument.
fn parse_effect_key(name: &str) -> Result<Identifier, CommandSyntaxError> {
    let (namespace, path) = match name.split_once(':') {
        Some((namespace, path)) => (namespace, path),
        None => (Identifier::VANILLA_NAMESPACE, name),
    };
    if !Identifier::validate(namespace, path) {
        return Err(CommandSyntaxError::dynamic(format!(
            "Invalid effect '{name}'"
        )));
    }
    Ok(Identifier::new(namespace.to_owned(), path.to_owned()))
}

fn targets(
    context: &SteelCommandContext<CommandSource>,
) -> Result<Vec<crate::entity::SharedEntity>, CommandSyntaxError> {
    context.entities("targets")
}

fn give(
    context: &SteelCommandContext<CommandSource>,
    _default: bool,
) -> Result<i32, CommandSyntaxError> {
    let effect = effect_of(context)?;
    let seconds = context.integer("seconds")?;
    let amplifier = context.integer("amplifier").unwrap_or(0);
    apply(context, effect, seconds.saturating_mul(20), amplifier)
}

fn give_infinite(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let effect = effect_of(context)?;
    apply(context, effect, INFINITE_DURATION, 0)
}

fn apply(
    context: &SteelCommandContext<CommandSource>,
    effect: steel_registry::mob_effect::MobEffectRef,
    duration: i32,
    amplifier: i32,
) -> Result<i32, CommandSyntaxError> {
    let entities = targets(context)?;
    let mut applied = 0;
    for entity in entities {
        let Some(living) = entity.as_living_entity() else {
            continue;
        };
        if living
            .living_base()
            .add_mob_effect(MobEffectInstance::with_duration(effect, duration, amplifier))
        {
            applied += 1;
        }
    }
    if applied == 0 {
        return Err(CommandSyntaxError::dynamic("No entity accepted the effect"));
    }
    context.source().send_success(
        &TextComponent::plain(format!(
            "Applied {} to {applied} entities",
            effect.key.path
        )),
        false,
    );
    Ok(applied)
}

fn clear_one(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let effect = effect_of(context)?;
    let entities = targets(context)?;
    let mut cleared = 0;
    for entity in entities {
        let Some(living) = entity.as_living_entity() else {
            continue;
        };
        if living.living_base().remove_mob_effect(effect) {
            cleared += 1;
        }
    }
    Ok(cleared)
}

fn clear_all(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let entities = targets(context)?;
    let mut cleared = 0;
    for entity in entities {
        let Some(living) = entity.as_living_entity() else {
            continue;
        };
        let effects = living.living_base().active_mob_effects();
        for effect in effects {
            if living.living_base().remove_mob_effect(effect.effect()) {
                cleared += 1;
            }
        }
    }
    Ok(cleared)
}
