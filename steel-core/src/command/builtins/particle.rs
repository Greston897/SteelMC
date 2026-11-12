//! Vanilla `/particle` command.

use std::sync::Arc;

use glam::DVec3;
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

fn command() -> CommandNodeBuilder<CommandSource, SteelCommandRuntime> {
    literal("particle").then(
        argument("name", ArgumentType::word()).then(
            argument("pos", SteelArgumentType::vec3(false))
                .then(
                    argument("delta", SteelArgumentType::vec3(false))
                        .then(
                            argument("speed", SteelArgumentType::float(0.0, f32::MAX))
                                .then(
                                    argument("count", ArgumentType::integer(0, i32::MAX))
                                        .executes(|context| emit(context, None)),
                                )
                                .executes(|context| emit(context, None)),
                        )
                        .executes(|context| emit(context, None)),
                )
                .executes(|context| emit(context, None)),
        ),
    )
}

fn particle_for(name: &str) -> Option<ParticleData> {
    let key = if name == "dust" {
        Identifier::vanilla("dust".to_owned())
    } else if name.contains(':') {
        Identifier::new(
            name.split_once(':').map_or("minecraft", |(namespace, _)| namespace),
            name.split_once(':').map_or(name, |(_, path)| path),
        )
    } else {
        Identifier::vanilla(name.to_owned())
    };
    let particle_type = REGISTRY.particle_types.by_key(&key)?;
    Some(ParticleData::simple(particle_type))
}

fn emit(
    context: &SteelCommandContext<CommandSource>,
    _viewer: Option<&Arc<crate::world::World>>,
) -> Result<i32, CommandSyntaxError> {
    let name = context.string("name")?;
    let Some(particle) = particle_for(name) else {
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
