use super::{
    Arc, CEntityEvent, CSystemChat, CTabList, CTickingState, CTickingStep, Color, CommandSender,
    CommandSource, DisplayResolutor, Entity, Modifier, Player, Server, SprintReport,
    TabListTickStats, TextComponent, Uuid, client_permission_event, command_tree_packet,
    translations,
};
use std::borrow::Cow;

use crate::scoreboard::Scoreboard;
use steel_protocol::packets::game::{CDisguisedChat, ChatTypeBound};
use steel_registry::RegistryEntry as _;
use steel_registry::chat_type::{ChatStyle, ChatTypeDecoration, ChatTypeRef};
use text_components::translation::TranslatedMessage;

impl Server {
    /// Logs and broadcasts a system chat message to online players.
    pub fn broadcast_system_chat(&self, message: &TextComponent, excluded_player: Option<Uuid>) {
        log::info!("{}", message.to_plain(&DisplayResolutor));
        self.online_players.iter_players(|uuid, player| {
            if Some(*uuid) != excluded_player {
                player.send_packet(CSystemChat::new(message, false, player));
            }
            true
        });
    }

    /// Vanilla `PlayerList.broadcastChatMessage` for a decorated chat type.
    ///
    /// `/say`, `/me` and friends reach the client as `disguised_chat`, not as a
    /// system message: the packet carries the chat type so the *client* applies
    /// the decoration (translation key, parameters, style) and so the client can
    /// honour its own chat-visibility setting. Rendering the decoration on the
    /// server and pushing `system_chat` bypasses both, which is why `/say` used
    /// to survive "commands only" chat visibility.
    ///
    /// Vanilla also logs the decorated line, so the server log keeps showing the
    /// readable form.
    pub fn broadcast_decorated_chat(
        &self,
        chat_type: ChatTypeRef,
        content: &TextComponent,
        sender_name: TextComponent,
        target_name: Option<TextComponent>,
    ) {
        log::info!(
            "{}",
            decorated_chat_component(chat_type, &sender_name, target_name.as_ref(), content)
                .to_plain(&DisplayResolutor)
        );

        let packet = CDisguisedChat {
            message: content.clone(),
            chat_type: ChatTypeBound {
                registry_id: chat_type.id() as i32,
                sender_name,
                target_name,
            },
        };
        self.online_players.iter_players(|_, player| {
            player.send_packet(packet.clone());
            true
        });
    }

    /// Sends one decorated chat line to an explicit set of players.
    ///
    /// Vanilla `PlayerList.broadcastChatMessage(message, filter, recipients)`
    /// counterpart: used by `/msg`, where the incoming and outgoing variants of
    /// the same message go to disjoint audiences.
    pub fn broadcast_decorated_chat_to(
        &self,
        chat_type: ChatTypeRef,
        content: &TextComponent,
        sender_name: TextComponent,
        target_name: Option<TextComponent>,
        recipients: &[Arc<Player>],
    ) {
        let packet = CDisguisedChat {
            message: content.clone(),
            chat_type: ChatTypeBound {
                registry_id: chat_type.id() as i32,
                sender_name,
                target_name,
            },
        };
        for player in recipients {
            player.send_packet(packet.clone());
        }
    }

    /// Logs and broadcasts a system chat message, honouring a team's
    /// `death_message_visibility` when the message belongs to a team.
    ///
    /// Players on the emitting team see it according to that team's visibility,
    /// matching vanilla `PlayerTeam.sendMessage`. When `team` is `None` the call
    /// degrades to a normal broadcast.
    pub fn broadcast_team_message(
        &self,
        message: &TextComponent,
        team: Option<&str>,
        excluded_player: Option<Uuid>,
    ) {
        log::info!("{}", message.to_plain(&DisplayResolutor));

        let Some(team) = team else {
            self.broadcast_system_chat(message, excluded_player);
            return;
        };

        self.online_players.iter_players(|uuid, player| {
            if Some(*uuid) == excluded_player {
                return true;
            }
            let world = player.get_world();
            let can_see = self
                .scoreboards
                .get(world.domain())
                .is_none_or(|scoreboard| {
                    scoreboard.can_see_team_message(
                        &crate::scoreboard::ScoreHolder::new(player.scoreboard_name()),
                        team,
                    )
                });
            if can_see {
                player.send_packet(CSystemChat::new(message, false, player));
            }
            true
        });
    }

    /// Returns the objective bound to each display slot of a domain's scoreboard.
    ///
    /// Used when a player joins so the sidebar and tab-list numbers appear
    /// without waiting for the next mutation.
    #[must_use]
    pub fn scoreboard_display_slots(&self, domain: &str) -> Vec<(i32, String)> {
        self.scoreboards
            .get(domain)
            .map_or_default(Scoreboard::display_slots)
    }

    /// Builds the tab list header/footer with recent and five-second tick statistics.
    pub(super) fn tab_list_components(
        tick_stats: TabListTickStats,
    ) -> (TextComponent, TextComponent) {
        // Color TPS based on value
        let tps_color = if tick_stats.tps >= 19.5 {
            Color::Green
        } else if tick_stats.tps >= 15.0 {
            Color::Yellow
        } else {
            Color::Red
        };

        let mspt_color = |mspt: f32| {
            if mspt <= 50.0 {
                Color::Aqua
            } else {
                Color::Red
            }
        };

        let header = TextComponent::plain("\n").add_children(vec![
            TextComponent::plain("Steel Dev Build").color(Color::Yellow),
            TextComponent::plain("\n"),
        ]);
        let footer = TextComponent::plain("\n").add_children(vec![
            TextComponent::plain("TPS: ").color(Color::Gray),
            TextComponent::plain(format!("{:.1}", tick_stats.tps)).color(tps_color),
            TextComponent::plain(" | ").color(Color::DarkGray),
            TextComponent::plain("MSPT: ").color(Color::Gray),
            TextComponent::plain(format!("{:.2}", tick_stats.recent_mspt))
                .color(mspt_color(tick_stats.recent_mspt)),
            TextComponent::plain(" recent | ").color(Color::Gray),
            TextComponent::plain(format!("{:.2}", tick_stats.average_mspt))
                .color(mspt_color(tick_stats.average_mspt)),
            TextComponent::plain(" avg (5s) | ").color(Color::Gray),
            TextComponent::plain(format!("{:.2}", tick_stats.p95_mspt))
                .color(mspt_color(tick_stats.p95_mspt)),
            TextComponent::plain(" p95").color(Color::Gray),
            TextComponent::plain("\n"),
        ]);

        (header, footer)
    }

    /// Broadcasts the tab list header/footer with current TPS and MSPT statistics.
    pub(super) fn broadcast_tab_list(&self, tick_stats: TabListTickStats) {
        let (header, footer) = Self::tab_list_components(tick_stats);

        self.broadcast_to_online_with(|player| CTabList::new(&header, &footer, player));
    }

    /// Broadcasts a sprint completion report to all players.
    pub(crate) fn broadcast_sprint_report(&self, report: &SprintReport) {
        let message: TextComponent = translations::COMMANDS_TICK_SPRINT_REPORT
            .message([
                TextComponent::from(format!("{}", report.ticks_per_second)),
                TextComponent::from(format!("{:.2}", report.ms_per_tick)),
            ])
            .into();

        self.broadcast_system_chat(&message, None);
    }

    pub(super) fn broadcast_player_join_message(
        &self,
        player: &Player,
        previous_name: Option<&str>,
    ) {
        let display_name = player.display_name();
        // Fallback to the current name when the cache has no prior entry.
        let old_name = previous_name.unwrap_or(player.gameprofile.name());
        let message: TextComponent = if player.gameprofile.name().eq_ignore_ascii_case(old_name) {
            translations::MULTIPLAYER_PLAYER_JOINED
                .message([display_name])
                .into()
        } else {
            translations::MULTIPLAYER_PLAYER_JOINED_RENAMED
                .message([display_name, TextComponent::plain(old_name.to_owned())])
                .into()
        };
        let message = message.color(Color::Yellow);
        self.broadcast_system_chat(&message, Some(player.gameprofile.id()));
    }

    pub(super) fn broadcast_player_leave_message(&self, player: &Player) {
        let message: TextComponent = translations::MULTIPLAYER_PLAYER_LEFT
            .message([player.display_name()])
            .into();
        let message = message.color(Color::Yellow);
        self.broadcast_system_chat(&message, None);
    }

    /// Broadcasts the current tick rate and frozen state to all clients.
    /// This should be called whenever the tick rate or frozen state changes.
    pub fn broadcast_ticking_state(&self) {
        let tick_manager = self.tick_rate_manager.read();
        let packet = CTickingState::new(tick_manager.tick_rate(), tick_manager.is_frozen());
        drop(tick_manager);

        self.broadcast_to_online(packet);
    }

    /// Broadcasts the current step tick count to all clients.
    /// This should be called whenever the step tick count changes.
    pub fn broadcast_ticking_step(&self) {
        let tick_manager = self.tick_rate_manager.read();
        let packet = CTickingStep::new(tick_manager.frozen_ticks_to_run());
        drop(tick_manager);

        self.broadcast_to_online(packet);
    }

    /// Sends the current ticking state and step packets to a joining player.
    /// This should be called when a player joins the server.
    pub fn send_ticking_state_to_player(&self, player: &Player) {
        let tick_manager = self.tick_rate_manager.read();
        let state_packet = CTickingState::new(tick_manager.tick_rate(), tick_manager.is_frozen());
        let step_packet = CTickingStep::new(tick_manager.frozen_ticks_to_run());
        drop(tick_manager);

        player.send_packet(state_packet);
        player.send_packet(step_packet);
    }

    /// Resends client state that is not fully covered by `CRespawn`.
    pub fn resend_player_context(self: &Arc<Self>, player: &Arc<Player>) {
        player.send_difficulty();
        player.send_inventory_to_remote();

        self.resend_player_permission_context(player);

        self.send_ticking_state_to_player(player);
    }

    /// Resends the command tree and vanilla client permission-level projection.
    pub fn resend_player_permission_context(self: &Arc<Self>, player: &Arc<Player>) {
        let world = player.get_world();
        player.send_packet(CEntityEvent {
            entity_id: player.id(),
            event: client_permission_event(player, &world),
        });

        let Some(server) = player.try_server() else {
            return;
        };
        if !Arc::ptr_eq(&server, self) {
            tracing::error!(
                player = %player.gameprofile.name(),
                "cannot project commands from a different server"
            );
            return;
        }
        let Some(shared_player) = self.online_players.get_by_uuid(&player.gameprofile.id()) else {
            tracing::error!(
                player = %player.gameprofile.name(),
                "cannot project commands for a player outside the online player map"
            );
            return;
        };
        if !Arc::ptr_eq(&shared_player, player) {
            tracing::error!(
                player = %player.gameprofile.name(),
                "cannot project commands for a stale player handle"
            );
            return;
        }
        let source = CommandSource::new(CommandSender::Player(shared_player), server);
        let commands = {
            let dispatcher = self.command_dispatcher.read();
            command_tree_packet(&dispatcher, &source)
        };
        match commands {
            Ok(commands) => player.send_packet(commands),
            Err(error) => tracing::error!(
                player = %player.gameprofile.name(),
                %error,
                "failed to project the player's command tree"
            ),
        }
    }

    /// Configures or clears the MC 26.3 server code of conduct text shown to
    /// joining players during Configuration (`ClientboundCodeOfConductPacket`).
    pub fn set_code_of_conduct(&self, text: Option<String>) {
        *self.code_of_conduct.lock() = text.filter(|s| !s.is_empty());
    }

    /// Builds a `CCodeOfConduct` packet if a non-empty code of conduct is active.
    #[must_use]
    pub fn code_of_conduct_packet(&self) -> Option<steel_protocol::packets::game::CCodeOfConduct> {
        self.code_of_conduct.lock().clone().map(|code_of_conduct| {
            steel_protocol::packets::game::CCodeOfConduct { code_of_conduct }
        })
    }

    /// Replaces the custom report details attached to client crash/abuse reports
    /// (`ClientboundCustomReportDetailsPacket`) and pushes the update to online players.
    pub fn set_custom_report_details(
        &self,
        details: Vec<steel_protocol::packets::game::ReportDetail>,
    ) {
        self.custom_report_details.lock().clone_from(&details);
        self.broadcast_to_online(steel_protocol::packets::game::CCustomReportDetails { details });
    }

    /// Builds a `CCustomReportDetails` packet if custom report details are registered.
    #[must_use]
    pub fn custom_report_details_packet(
        &self,
    ) -> Option<steel_protocol::packets::game::CCustomReportDetails> {
        let details = self.custom_report_details.lock().clone();
        (!details.is_empty())
            .then_some(steel_protocol::packets::game::CCustomReportDetails { details })
    }

    /// Vanilla `PlayerList.deleteMessage`: instructs all connected clients to
    /// hide/delete a previously seen signed chat message (`ClientboundDeleteChatPacket`).
    pub fn delete_signed_chat(
        &self,
        message_signature: steel_protocol::packets::game::PackedMessageSignature,
    ) {
        self.broadcast_to_online(steel_protocol::packets::game::CDeleteChat { message_signature });
    }

    /// Pushes custom chat tab-completion suggestions to all online players
    /// (`ClientboundCustomChatCompletionsPacket`).
    pub fn broadcast_custom_chat_completions(
        &self,
        action: steel_protocol::packets::game::CustomChatCompletionAction,
        entries: Vec<String>,
    ) {
        self.broadcast_to_online(steel_protocol::packets::game::CCustomChatCompletions {
            action,
            entries,
        });
    }
}

/// Splits a vanilla `ChatTypeDecoration` key into an `en_us` format string.
///
/// Vanilla builds the decorated line from the translation key and its
/// parameters; the server copy is only used for the log line, because the
/// *client* renders `disguised_chat` itself.
fn decoration_args(
    dec: &ChatTypeDecoration,
    sender: &TextComponent,
    target: Option<&TextComponent>,
    content: &TextComponent,
) -> Vec<TextComponent> {
    dec.parameters
        .iter()
        .map(|name| match *name {
            "sender" => sender.clone(),
            "target" => target.cloned().unwrap_or_default(),
            "content" => content.clone(),
            other => TextComponent::plain(other.to_string()),
        })
        .collect()
}

/// Vanilla `ChatTypeDecoration.decorate`: resolves the decoration against the
/// bound sender/target/content and applies the decoration's style.
///
/// Only the server-side log line and non-player command feedback need this;
/// players receive the raw `disguised_chat` and let their own client decorate.
pub fn decorated_chat_component(
    chat_type: ChatTypeRef,
    sender: &TextComponent,
    target: Option<&TextComponent>,
    content: &TextComponent,
) -> TextComponent {
    let dec = &chat_type.chat;

    let component = TextComponent::translated(TranslatedMessage {
        key: Cow::Borrowed(dec.translation_key),
        fallback: None,
        args: Some(decoration_args(dec, sender, target, content).into_boxed_slice()),
    });

    apply_chat_style(component, dec.style.as_ref())
}

/// Applies a vanilla `ChatStyle` to a component, leaving unset flags alone.
fn apply_chat_style(mut component: TextComponent, style: Option<&ChatStyle>) -> TextComponent {
    let Some(style) = style else {
        return component;
    };
    if let Some(color) = style.color
        && let Some(color) = parse_chat_color(color)
    {
        component = component.color(color);
    }
    if let Some(bold) = style.bold {
        component = component.bold(bold);
    }
    if let Some(italic) = style.italic {
        component = component.italic(italic);
    }
    if let Some(underlined) = style.underlined {
        component = component.underlined(underlined);
    }
    if let Some(strikethrough) = style.strikethrough {
        component = component.strikethrough(strikethrough);
    }
    if let Some(obfuscated) = style.obfuscated {
        component = component.obfuscated(obfuscated);
    }
    component
}

/// Maps a data-pack colour name (`red`, `#ff0000`) to [`Color`].
fn parse_chat_color(name: &str) -> Option<Color> {
    Some(match name {
        "aqua" => Color::Aqua,
        "black" => Color::Black,
        "blue" => Color::Blue,
        "dark_aqua" => Color::DarkAqua,
        "dark_blue" => Color::DarkBlue,
        "dark_gray" => Color::DarkGray,
        "dark_green" => Color::DarkGreen,
        "dark_purple" => Color::DarkPurple,
        "dark_red" => Color::DarkRed,
        "gold" => Color::Gold,
        "gray" => Color::Gray,
        "green" => Color::Green,
        "light_purple" => Color::LightPurple,
        "red" => Color::Red,
        "white" => Color::White,
        "yellow" => Color::Yellow,
        _ => Color::from_hex(name)?,
    })
}
