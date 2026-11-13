//! Client synchronisation for the command scoreboards.
//!
//! The scoreboard itself lives in [`crate::scoreboard`] and is deliberately
//! command-facing: it stores objectives, scores, teams and memberships without
//! knowing anything about the network. This module is the missing half — it
//! turns that state into the vanilla clientbound scoreboard packets.
//!
//! Vanilla keeps one `ServerScoreboard` per level and pushes every mutation to
//! the tracking players immediately (`ServerScoreboard#onObjectiveAdded` and
//! friends). Steel has the same shape: [`sync_full`] runs when a player joins a
//! level, and the `sync_*` helpers run after the corresponding command.

use std::sync::Arc;

use crate::player::Player;
use crate::scoreboard::{Scoreboard, ScoreHolder, TeamState, TeamVisibility};
use crate::server::Server;
use steel_protocol::packet_traits::ClientPacket;
use steel_protocol::packets::game::{
    CResetScore, CSetDisplayObjective, CSetObjective, CSetPlayerTeam, CSetScore, NO_COLOR,
    ObjectiveAction, RenderType, ScoreEntry, TeamAction, TeamParameters,
};
use text_components::TextComponent;

/// Vanilla `DisplaySlot` ordinals used by the client.
pub mod display_slot {
    /// `DisplaySlot.LIST` — the tab-list numbers.
    pub const LIST: i32 = 0;
    /// `DisplaySlot.SIDEBAR` — the right-hand sidebar.
    pub const SIDEBAR: i32 = 1;
}

/// Vanilla `ChatFormatting` ordinals for team colours.
///
/// The ordinal is the wire value, so the table has to stay in vanilla order.
fn team_color_id(color: Option<&str>) -> i32 {
    let Some(color) = color else {
        return NO_COLOR;
    };
    match color {
        "black" => 0,
        "dark_blue" => 1,
        "dark_green" => 2,
        "dark_aqua" => 3,
        "dark_red" => 4,
        "dark_purple" => 5,
        "gold" => 6,
        "gray" => 7,
        "dark_gray" => 8,
        "blue" => 9,
        "green" => 10,
        "aqua" => 11,
        "red" => 12,
        "light_purple" => 13,
        "yellow" => 14,
        "white" => 15,
        _ => NO_COLOR,
    }
}

/// Vanilla `PlayerTeam` visibility name, falling back to `always`.
const fn visibility_name(visibility: TeamVisibility) -> &'static str {
    visibility.name()
}

/// Vanilla `Team.collisionRule` — Steel tracks friendly fire only, so the rule
/// follows from it the way vanilla's default does.
const fn collision_rule(allow_friendly_fire: bool) -> &'static str {
    if allow_friendly_fire {
        "always"
    } else {
        "pushOtherTeams"
    }
}

/// Builds the wire parameters for one team.
fn team_parameters(state: &TeamState) -> TeamParameters {
    // Vanilla flags: bit 1 = friendly fire, bit 2 = see friendly invisibles.
    let mut flags = 0_u8;
    if state.allow_friendly_fire {
        flags |= 1;
    }
    flags |= 2;
    TeamParameters {
        display_name: TextComponent::plain(state.prefix.clone()),
        flags,
        name_tag_visibility: visibility_name(state.name_tag_visibility).to_owned(),
        collision_rule: collision_rule(state.allow_friendly_fire).to_owned(),
        color: team_color_id(state.color.as_deref()),
        prefix: TextComponent::plain(state.prefix.clone()),
        suffix: TextComponent::plain(state.suffix.clone()),
    }
}

/// Sends the whole scoreboard to one player.
///
/// Mirrors vanilla `ServerScoreboard#addPlayer` → `ClientboundSetObjectivePacket`,
/// `ClientboundSetScorePacket`, `ClientboundSetPlayerTeamPacket` and
/// `ClientboundSetDisplayObjectivePacket` for the active display slots.
pub fn sync_full(player: &Player, scoreboard: &Scoreboard, display_slots: &[(i32, String)]) {
    for (name, display, read_only) in scoreboard.objective_display_names() {
        player.send_packet(CSetObjective {
            name: name.clone(),
            action: ObjectiveAction::Add {
                display_name: TextComponent::plain(display),
                // Steel objectives are all plain counters; vanilla's `dummy`
                // criterion renders as integers too.
                render_type: if read_only {
                    RenderType::Hearts
                } else {
                    RenderType::Integer
                },
            },
        });
    }

    let scores = scoreboard.score_snapshot();
    if !scores.is_empty() {
        player.send_packet(CSetScore {
            entries: scores
                .into_iter()
                .map(|(owner, objective_name, value)| ScoreEntry {
                    owner,
                    objective_name,
                    value,
                })
                .collect(),
        });
    }

    for (name, state) in scoreboard.team_states() {
        player.send_packet(CSetPlayerTeam {
            name: name.clone(),
            action: TeamAction::Add {
                parameters: team_parameters(&state),
            },
        });
        let members: Vec<String> = scoreboard
            .team_member_snapshot()
            .into_iter()
            .filter(|(team, _)| *team == name)
            .map(|(_, holder)| holder)
            .collect();
        if !members.is_empty() {
            player.send_packet(CSetPlayerTeam {
                name,
                action: TeamAction::AddPlayers { players: members },
            });
        }
    }

    for (slot, objective) in display_slots {
        player.send_packet(CSetDisplayObjective {
            slot: *slot,
            objective_name: objective.clone(),
        });
    }
}

/// Broadcasts one objective creation to every online player on a domain.
pub fn sync_objective_added(server: &Arc<Server>, domain: &str, name: &str) {
    broadcast(server, domain, |_| {
        CSetObjective {
            name: name.to_owned(),
            action: ObjectiveAction::Add {
                display_name: TextComponent::plain(name),
                render_type: RenderType::Integer,
            },
        }
    });
}

/// Broadcasts an objective's new display name to every online player.
pub fn sync_objective_updated(
    server: &Arc<Server>,
    domain: &str,
    name: &str,
    display: &str,
    read_only: bool,
) {
    let display_name = TextComponent::plain(display);
    broadcast(server, domain, |_| CSetObjective {
        name: name.to_owned(),
        action: ObjectiveAction::Update {
            display_name: display_name.clone(),
            render_type: if read_only {
                RenderType::Hearts
            } else {
                RenderType::Integer
            },
        },
    });
}

/// Broadcasts one objective removal to every online player on a domain.
pub fn sync_objective_removed(server: &Arc<Server>, domain: &str, name: &str) {
    broadcast(server, domain, |_| CSetObjective {
        name: name.to_owned(),
        action: ObjectiveAction::Remove,
    });
}

/// Broadcasts one score write to every online player on a domain.
pub fn sync_score_changed(server: &Arc<Server>, domain: &str, holder: &str, objective: &str, value: i32) {
    broadcast(server, domain, |_| CSetScore {
        entries: vec![ScoreEntry {
            owner: holder.to_owned(),
            objective_name: objective.to_owned(),
            value,
        }],
    });
}

/// Broadcasts one score removal to every online player on a domain.
pub fn sync_score_removed(server: &Arc<Server>, domain: &str, holder: &str, objective: &str) {
    broadcast(server, domain, |_| CResetScore {
        owner: holder.to_owned(),
        objective_name: objective.to_owned(),
    });
}

/// Broadcasts one team creation to every online player on a domain.
pub fn sync_team_added(server: &Arc<Server>, domain: &str, name: &str, state: &TeamState) {
    let parameters = team_parameters(state);
    broadcast(server, domain, |_| CSetPlayerTeam {
        name: name.to_owned(),
        action: TeamAction::Add {
            parameters: parameters.clone(),
        },
    });
}

/// Broadcasts one team removal to every online player on a domain.
pub fn sync_team_removed(server: &Arc<Server>, domain: &str, name: &str) {
    broadcast(server, domain, |_| CSetPlayerTeam {
        name: name.to_owned(),
        action: TeamAction::Remove,
    });
}

/// Broadcasts a team parameter update to every online player on a domain.
pub fn sync_team_updated(server: &Arc<Server>, domain: &str, name: &str, state: &TeamState) {
    let parameters = team_parameters(state);
    broadcast(server, domain, |_| CSetPlayerTeam {
        name: name.to_owned(),
        action: TeamAction::Update {
            parameters: parameters.clone(),
        },
    });
}

/// Broadcasts a membership change to every online player on a domain.
///
/// `added` distinguishes "joined the team" from "left the team", which vanilla
/// splits into `mode = 3` and `mode = 4`.
pub fn sync_team_membership(
    server: &Arc<Server>,
    domain: &str,
    team: &str,
    holders: &[String],
    added: bool,
) {
    if holders.is_empty() {
        return;
    }
    let players = holders.to_vec();
    broadcast(server, domain, move |_| CSetPlayerTeam {
        name: team.to_owned(),
        action: if added {
            TeamAction::AddPlayers {
                players: players.clone(),
            }
        } else {
            TeamAction::RemovePlayers {
                players: players.clone(),
            }
        },
    });
}

/// Broadcasts the display-slot binding to every online player on a domain.
pub fn sync_display_slot(server: &Arc<Server>, domain: &str, slot: i32, objective: &str) {
    broadcast(server, domain, |_| CSetDisplayObjective {
        slot,
        objective_name: objective.to_owned(),
    });
}

/// Sends a packet built by `build` to every player whose world is `domain`.
fn broadcast<P, F>(server: &Arc<Server>, domain: &str, build: F)
where
    P: ClientPacket,
    F: Fn(&Player) -> P,
{
    server.online_players.iter_players(|uuid, player| {
        let _ = uuid;
        if player.get_world().domain() == domain {
            player.send_packet(build(player));
        }
        true
    });
}

/// Resolves the scoreboard name for a holder, matching vanilla's player-name
/// based scoring for online players.
#[must_use]
pub fn holder_name(player: &Player) -> ScoreHolder {
    ScoreHolder::new(player.scoreboard_name())
}

/// Sends the full scoreboard to a player that just joined a level.
pub fn sync_on_join(server: &Arc<Server>, player: &Player) {
    let domain = player.get_world().domain().to_owned();
    let Some(scoreboard) = server.scoreboards.get(&domain) else {
        return;
    };
    let display_slots = server.scoreboard_display_slots(&domain);
    sync_full(player, scoreboard, &display_slots);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoreboard::TeamVisibility;

    #[test]
    fn team_colour_ordinals_match_vanilla_chat_formatting() {
        // `ChatFormatting` ordinals are the wire values, so a shift here would
        // silently recolour every team.
        assert_eq!(team_color_id(Some("black")), 0);
        assert_eq!(team_color_id(Some("dark_blue")), 1);
        assert_eq!(team_color_id(Some("gold")), 6);
        assert_eq!(team_color_id(Some("white")), 15);
        assert_eq!(team_color_id(None), NO_COLOR);
        // Unknown names fall back to "no colour" instead of an arbitrary index.
        assert_eq!(team_color_id(Some("chartreuse")), NO_COLOR);
    }

    #[test]
    fn team_parameters_encode_friendly_fire_as_a_flag() {
        let mut state = TeamState::default();
        assert_eq!(team_parameters(&state).flags & 1, 0);
        state.allow_friendly_fire = true;
        assert_eq!(team_parameters(&state).flags & 1, 1);
        // Bit 2 is vanilla's "see friendly invisibles"; Steel always sets it.
        assert_eq!(team_parameters(&state).flags & 2, 2);
    }

    #[test]
    fn collision_rule_follows_friendly_fire() {
        assert_eq!(collision_rule(true), "always");
        assert_eq!(collision_rule(false), "pushOtherTeams");
    }

    #[test]
    fn display_slot_ordinals_match_vanilla() {
        assert_eq!(display_slot::LIST, 0);
        assert_eq!(display_slot::SIDEBAR, 1);
    }

    #[test]
    fn team_visibility_names_round_trip_to_the_wire() {
        // The client parses these names literally, so they must match
        // `PlayerTeam`'s serialized enum names.
        assert_eq!(visibility_name(TeamVisibility::Always), "always");
        assert_eq!(
            visibility_name(TeamVisibility::HideForOtherTeams),
            "hideForOtherTeams"
        );
        assert_eq!(
            visibility_name(TeamVisibility::HideForOwnTeam),
            "hideForOwnTeam"
        );
        assert_eq!(visibility_name(TeamVisibility::Never), "never");
    }
}
