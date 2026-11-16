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
                argument("targets", SteelArgumentType::entities()).then(
                    argument("effect", SteelArgumentType::identifier())
                        .executes(give_default)
                        .then(
                            argument("seconds", ArgumentType::integer(1, 1_000_000))
                                .executes(|context| give(context, false))
                                .then(
                                    argument("amplifier", ArgumentType::integer(0, 255))
                                        .executes(|context| give(context, false))
                                        .then(
                                            argument("hideParticles", ArgumentType::bool())
                                                .executes(|context| give(context, false)),
                                        ),
                                ),
                        )
                        .then(
                            literal("infinite").executes(give_infinite).then(
                                argument("amplifier", ArgumentType::integer(0, 255))
                                    .executes(give_infinite)
                                    .then(
                                        argument("hideParticles", ArgumentType::bool())
                                            .executes(give_infinite),
                                    ),
                            ),
                        ),
                ),
            ),
        )
        .then(
            literal("clear").executes(clear_self).then(
                argument("targets", SteelArgumentType::entities())
                    .executes(clear_all)
                    .then(argument("effect", SteelArgumentType::identifier()).executes(clear_one)),
            ),
        )
}

fn effect_of(
    context: &SteelCommandContext<CommandSource>,
) -> Result<steel_registry::mob_effect::MobEffectRef, CommandSyntaxError> {
    let key = context.identifier("effect")?;
    REGISTRY
        .mob_effects
        .by_key(key)
        .ok_or_else(|| CommandSyntaxError::dynamic(format!("Unknown effect '{key}'")))
}

fn give_default(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let effect = effect_of(context)?;
    apply(context, effect, 30 * 20, 0)
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
        // `add_mob_effect` reports how the active-effect map changed; vanilla's
        // `/effect give` counts any change (a fresh effect or an upgrade).
        if living
            .living_base()
            .add_mob_effect(MobEffectInstance::with_duration(
                effect, duration, amplifier,
            ))
            .changed()
        {
            applied += 1;
        }
    }
    if applied == 0 {
        return Err(CommandSyntaxError::dynamic("No entity accepted the effect"));
    }
    context.source().send_success(
        &TextComponent::plain(format!("Applied {} to {applied} entities", effect.key.path)),
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

fn clear_self(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let Some(entity) = context.source().entity() else {
        return Ok(0);
    };
    let Some(living) = entity.as_living_entity() else {
        return Ok(0);
    };
    let effects = living.living_base().active_mob_effects();
    let mut cleared = 0;
    for effect in effects {
        if living.living_base().remove_mob_effect(effect.effect()) {
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
