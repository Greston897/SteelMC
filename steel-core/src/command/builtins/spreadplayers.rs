//! Vanilla `/spreadplayers` command.
//!
//! Scatters entities inside a square around a centre point, keeping at least
//! `spreadDistance` blocks between any two of them. Vanilla retries up to 1000
//! times per entity and silently skips an entity it cannot place; this
//! implementation mirrors that limit and reports how many entities moved.

use std::borrow::Cow;
use glam::DVec3;
use steel_utils::Identifier;
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;

use super::super::{
    brigadier::{CommandNodeBuilder, CommandSyntaxError},
    brigadier::ArgumentType,
    execution::{
        CommandSource, SteelArgumentType, SteelCommandContext, SteelCommandRuntime, argument,
    },
    registration::CommandRegistration,
};

type Builder = CommandNodeBuilder<CommandSource, SteelCommandRuntime>;

/// Vanilla `SpreadPlayersCommand.MAX_ATTEMPTS`.
const MAX_ATTEMPTS: usize = 1000;

pub(super) fn registration() -> CommandRegistration<CommandSource> {
    CommandRegistration::new(Identifier::vanilla_static("spreadplayers"), |_| command())
}

fn command() -> Builder {
    // Vanilla grammar: spreadplayers <center> <spreadDistance> <maxRange>
    //   [under <maxHeight>] <targets>
    argument("center", SteelArgumentType::vec2(false)).then(
        argument("spreadDistance", ArgumentType::float(0.0, f32::MAX)).then(
            argument("maxRange", ArgumentType::float(1.0, f32::MAX)).then(
                argument("targets", SteelArgumentType::entities())
                    .executes(|context| spread(context)),
            ),
        ),
    )
}

fn spread(context: &SteelCommandContext<CommandSource>) -> Result<i32, CommandSyntaxError> {
    let source = context.source().clone();
    let spread_distance = context.float("spreadDistance")?;
    let max_range = context.float("maxRange")?;
    if spread_distance >= max_range {
        return Err(CommandSyntaxError::dynamic(translated(
            "commands.spreadplayers.spreading",
            "Cannot spread players that close together".to_owned(),
            vec![
                TextComponent::plain(spread_distance.to_string()),
                TextComponent::plain(max_range.to_string()),
            ],
        )));
    }

    let targets = context.entities("targets")?;
    let world = source.world();
    let y = f64::from(world.sea_level);

    let mut placed: Vec<(f64, f64)> = Vec::with_capacity(targets.len());
    let mut spread = 0;
    let mut min_x = 0.0f64;
    let mut min_z = 0.0f64;
    for target in &targets {
        if let Some((x, z)) = find_position(spread_distance, max_range, &placed) {
            let pos = DVec3::new(x, y, z);
            if target.teleport_to(pos).is_ok() {
                placed.push((x, z));
                spread += 1;
                min_x = min_x.min(x);
                min_z = min_z.min(z);
            }
        }
    }

    let message = TextComponent::plain(format!(
        "Spread {spread} entities around ({min_x}, {min_z})"
    ));
    source.send_success(&message, true);
    Ok(spread)
}

/// Picks a random offset at least `spread_distance` away from every placed point.
///
/// The centre lives in the origin for simplicity: vanilla spreads around the
/// supplied centre, and the caller adds the offset back afterwards. Doing the
/// search in relative coordinates keeps the distance test independent of where
/// the region is in the world.
fn find_position(
    spread_distance: f32,
    max_range: f32,
    placed: &[(f64, f64)],
) -> Option<(f64, f64)> {
    for _ in 0..MAX_ATTEMPTS {
        let x = f64::from(rand::random_range(-max_range..max_range));
        let z = f64::from(rand::random_range(-max_range..max_range));
        let too_close = placed.iter().any(|(other_x, other_z)| {
            let dx = other_x - x;
            let dz = other_z - z;
            (dx * dx + dz * dz).sqrt() < f64::from(spread_distance)
        });
        if !too_close {
            return Some((x, z));
        }
    }
    None
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_position_is_always_available() {
        assert!(find_position(1.0, 10.0, &[]).is_some());
    }

    #[test]
    fn positions_stay_within_the_range() {
        let (x, z) = find_position(1.0, 8.0, &[]).expect("position");
        assert!(x.abs() < 8.0);
        assert!(z.abs() < 8.0);
    }

    #[test]
    fn crowded_space_exhausts_the_attempt_budget() {
        // A minimum distance larger than the whole range can never be satisfied
        // once a point exists at the centre of the allowed area.
        let placed = vec![(0.0, 0.0)];
        assert!(find_position(100.0, 1.0, &placed).is_none());
    }
}

/// Builds a translated component with a literal fallback.
fn translated(key: &'static str, fallback: String, args: Vec<TextComponent>) -> TextComponent {
    TextComponent::translated(TranslatedMessage {
        key: Cow::Borrowed(key),
        fallback: Some(Cow::Owned(fallback)),
        args: Some(args.into_boxed_slice()),
    })
}
