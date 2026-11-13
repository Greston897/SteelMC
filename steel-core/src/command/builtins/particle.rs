//! Vanilla `/particle` command.

use std::sync::Arc;

use glam::DVec3;
use simdnbt::owned::NbtCompound;
use steel_registry::particle_type::ParticleData;
use steel_registry::{REGISTRY, RegistryExt as _};
use steel_utils::Identifier;

use super::super::{
    brigadier::{ArgumentType, CommandNodeBuilder, CommandSyntaxError},
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
        literal,
    },
    registration::CommandRegistration,
};

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("particle"), |_| command())
}

/// Argument trees share the trailing `<pos> <delta> <speed> <count> [viewers]
/// [force]` shape, with and without the vanilla `[payload]` argument.
fn position_tree() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    argument("pos", SteelArgumentType::vec3(false))
        .then(
            argument("delta", SteelArgumentType::vec3(false))
                .then(
                    argument("speed", ArgumentType::float(0.0, f32::MAX))
                        .then(
                            argument("count", ArgumentType::integer(0, i32::MAX))
                                .executes(|context| emit(context, None)),
                        )
                        .executes(|context| emit(context, None)),
                )
                .executes(|context| emit(context, None)),
        )
        .executes(|context| emit(context, None))
}

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("particle").then(
        argument("name", ArgumentType::word())
            // Vanilla `particle <name> <pos> ...` for types without options, and
            // `particle <name> <payload> <pos> ...` for the configurable ones.
            .then(position_tree())
            .then(
                argument("payload", SteelArgumentType::compound())
                    .then(position_tree()),
            ),
    )
}

/// Resolves the particle, decoding an optional vanilla `<payload>` compound.
///
/// Vanilla requires the payload for option-carrying types (`dust`, `block`,
/// `item`, `entity_effect`, ...) and rejects it for the plain ones.
fn particle_for(name: &str, payload: Option<&NbtCompound>) -> Option<ParticleData> {
    let key = if name == "dust" {
        Identifier::vanilla("dust".to_owned())
    } else if name.contains(':') {
        let (namespace, path) = name
            .split_once(':')
            .expect("the branch guarantees a namespace separator");
        Identifier::new(namespace.to_owned(), path.to_owned())
    } else {
        Identifier::vanilla(name.to_owned())
    };
    let particle_type = REGISTRY.particle_types.by_key(&key)?;
    let Some(payload) = payload else {
        return Some(ParticleData::simple(particle_type));
    };
    ParticleData::from_nbt_compound(payload)
}

fn emit(
    context: &SteelCommandContext<CommandSource>,
    _viewer: Option<&Arc<crate::world::World>>,
) -> Result<i32, CommandSyntaxError> {
    let name = context.string("name")?;
    let payload = context.compound("payload").ok();
    let Some(particle) = particle_for(name, payload) else {
        return Err(CommandSyntaxError::dynamic(format!(
            "Unknown particle '{name}'"
        )));
    };

    let world = context.source().world().clone();
    let pos = vec3_argument(context, "pos")?;
    let delta = vec3_argument(context, "delta").unwrap_or(DVec3::ZERO);
    let speed = f64::from(context.float("speed").unwrap_or(0.0));
    let count = context.integer("count").unwrap_or(1);

    Ok(world.send_particles(particle, pos, count, delta, speed))
}

fn vec3_argument(
    context: &SteelCommandContext<CommandSource>,
    name: &str,
) -> Result<DVec3, CommandSyntaxError> {
    context
        .coordinates(name)
        .map(|coordinates| coordinates.position(context.source()))
}
