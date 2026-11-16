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

use crate::entity::Entity as _;

use crate::player::Player;
use crate::scoreboard::{Scoreboard, ScoreHolder, TeamState, TeamVisibility};
use crate::server::Server;
use steel_protocol::packet_traits::ClientPacket;
use steel_protocol::packets::game::{
    CResetScore, CSetDisplayObjective, CSetObjective, CSetPlayerTeam, CSetScore, ObjectiveAction,
    RenderType, TeamAction, TeamParameters,
};
use text_components::TextComponent;

/// Vanilla `DisplaySlot` ordinals used by the client.
pub mod display_slot {
    /// `DisplaySlot.LIST` — the tab-list numbers.
    pub const LIST: i32 = 0;
    /// `DisplaySlot.SIDEBAR` — the right-hand sidebar.
    pub const SIDEBAR: i32 = 1;
}

/// Vanilla `TeamColor` ordinals (`0..=15`), or `None` when uncolored.
fn team_color_id(color: Option<&str>) -> Option<i32> {
    match color? {
        "black" => Some(0),
        "dark_blue" => Some(1),
        "dark_green" => Some(2),
        "dark_aqua" => Some(3),
        "dark_red" => Some(4),
        "dark_purple" => Some(5),
        "gold" => Some(6),
        "gray" => Some(7),
        "dark_gray" => Some(8),
        "blue" => Some(9),
        "green" => Some(10),
        "aqua" => Some(11),
        "red" => Some(12),
        "light_purple" => Some(13),
        "yellow" => Some(14),
        "white" => Some(15),
        _ => None,
    }
}

/// Vanilla `Team.Visibility` wire id (`0=always, 1=never, 2=hideForOtherTeams, 3=hideForOwnTeam`).
const fn visibility_id(visibility: TeamVisibility) -> i32 {
    match visibility.name().as_bytes() {
        b"never" => 1,
        b"hideForOtherTeams" => 2,
        b"hideForOwnTeam" => 3,
        _ => 0,
    }
}

/// Vanilla `Team.CollisionRule` wire id (`0 = always`, `2 = pushOtherTeams`).
const fn collision_rule_id(allow_friendly_fire: bool) -> i32 {
    if allow_friendly_fire { 0 } else { 2 }
}

/// Builds the MC 26.3 wire parameters for one team.
fn team_parameters(name: &str, state: &TeamState) -> TeamParameters {
    // Vanilla `PlayerTeam.packOptions`: bit 0 = friendly fire, bit 1 = see friendly invisibles.
    let mut options = 0_u8;
    if state.allow_friendly_fire {
        options |= 1;
    }
    options |= 2;
    let display = if state.prefix.is_empty() {
        name.to_owned()
    } else {
        state.prefix.clone()
    };
    TeamParameters {
        display_name: TextComponent::plain(display),
        prefix: TextComponent::plain(state.prefix.clone()),
        suffix: TextComponent::plain(state.suffix.clone()),
        name_tag_visibility: visibility_id(state.name_tag_visibility),
        collision_rule: collision_rule_id(state.allow_friendly_fire),
        color: team_color_id(state.color.as_deref()),
        options,
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

    for (owner, objective_name, value) in scoreboard.score_snapshot() {
        player.send_packet(CSetScore::new(owner, objective_name, value));
    }

    let all_members = scoreboard.team_member_snapshot();
    for (name, state) in scoreboard.team_states() {
        let players: Vec<String> = all_members
            .iter()
            .filter(|(team, _)| *team == name)
            .map(|(_, holder)| holder.clone())
            .collect();
        player.send_packet(CSetPlayerTeam {
            action: TeamAction::Add {
                parameters: team_parameters(&name, &state),
                players,
            },
            name,
        });
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
                display_name: TextComponent::plain(name.to_owned()),
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
    let display_name = TextComponent::plain(display.to_owned());
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
pub fn sync_score_changed(
    server: &Arc<Server>,
    domain: &str,
    holder: &str,
    objective: &str,
    value: i32,
) {
    broadcast(server, domain, |_| {
        CSetScore::new(holder.to_owned(), objective.to_owned(), value)
    });
}

/// Broadcasts one score removal to every online player on a domain.
pub fn sync_score_removed(server: &Arc<Server>, domain: &str, holder: &str, objective: &str) {
    broadcast(server, domain, |_| CResetScore {
        owner: holder.to_owned(),
        objective_name: Some(objective.to_owned()),
    });
}

/// Broadcasts one team creation to every online player on a domain.
pub fn sync_team_added(server: &Arc<Server>, domain: &str, name: &str, state: &TeamState) {
    let parameters = team_parameters(name, state);
    broadcast(server, domain, |_| CSetPlayerTeam {
        name: name.to_owned(),
        action: TeamAction::Add {
            parameters: parameters.clone(),
            players: Vec::new(),
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
    let parameters = team_parameters(name, state);
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
    fn team_colour_ordinals_match_vanilla_team_color() {
        assert_eq!(team_color_id(Some("black")), Some(0));
        assert_eq!(team_color_id(Some("dark_blue")), Some(1));
        assert_eq!(team_color_id(Some("gold")), Some(6));
        assert_eq!(team_color_id(Some("white")), Some(15));
        assert_eq!(team_color_id(None), None);
        assert_eq!(team_color_id(Some("chartreuse")), None);
    }

    #[test]
    fn team_parameters_encode_friendly_fire_as_option_bit() {
        let mut state = TeamState::default();
        assert_eq!(team_parameters("red", &state).options & 1, 0);
        state.allow_friendly_fire = true;
        assert_eq!(team_parameters("red", &state).options & 1, 1);
        assert_eq!(team_parameters("red", &state).options & 2, 2);
    }

    #[test]
    fn collision_rule_ordinals_follow_friendly_fire() {
        assert_eq!(collision_rule_id(true), 0);
        assert_eq!(collision_rule_id(false), 2);
    }

    #[test]
    fn display_slot_ordinals_match_vanilla() {
        assert_eq!(display_slot::LIST, 0);
        assert_eq!(display_slot::SIDEBAR, 1);
    }

    #[test]
    fn team_visibility_ordinals_match_vanilla_stream_codec() {
        assert_eq!(visibility_id(TeamVisibility::Always), 0);
        assert_eq!(visibility_id(TeamVisibility::Never), 1);
        assert_eq!(visibility_id(TeamVisibility::HideForOtherTeams), 2);
        assert_eq!(visibility_id(TeamVisibility::HideForOwnTeam), 3);
    }
}
