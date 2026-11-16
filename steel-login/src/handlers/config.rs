//! Configuration state packet handlers.

use std::sync::Arc;

use steel_core::entity::next_entity_id;
use steel_core::player::PlayerConnection;
use steel_core::player::connection::JavaConnection;
use steel_core::player::{ClientInformation, Player, PlayerSession};
use steel_protocol::packets::common::CCustomPayload;
use steel_protocol::packets::common::{SClientInformation, SCustomPayload};
use steel_protocol::packets::config::CFinishConfiguration;
use steel_protocol::packets::game::{CUpdateEnabledFeatures, ResourcePackAction, SResourcePack};
use steel_protocol::packets::config::CSelectKnownPacks;
use steel_protocol::packets::config::SSelectKnownPacks;
use steel_protocol::packets::shared_implementation::KnownPack;
use steel_protocol::utils::ConnectionProtocol;
use steel_utils::{Identifier, translations};
use text_components::TextComponent;
use text_components::translation::TranslatedMessage;

use crate::tcp_client::{ConnectionAction, ConnectionUpdate, JavaTcpClient};

const BRAND_PAYLOAD: [u8; 5] = *b"Steel";

impl JavaTcpClient {
    /// Handles a custom payload packet during the configuration state.
    #[expect(clippy::unused_self, reason = "this is an api function")]
    pub fn handle_config_custom_payload(&self, packet: SCustomPayload) {
        log::debug!("Custom payload packet: {packet:?}");
    }

    /// Handles the client information packet during the configuration state.
    pub async fn handle_client_information(&self, packet: SClientInformation) {
        log::debug!("Client information packet: {packet:?}");

        let info = ClientInformation {
            language: packet.language,
            // `RuntimeConfig::clamp_view_distance` owns the floor and ceiling, so
            // the config- and play-state handlers stay in sync.
            view_distance: self
                .server
                .config
                .clamp_view_distance(packet.view_distance.cast_unsigned()),
            chat_visibility: packet.chat_visibility,
            chat_colors: packet.chat_colors,
            model_customization: packet.model_customization,
            main_hand: packet.main_hand,
            text_filtering_enabled: packet.text_filtering_enabled,
            allows_listing: packet.allows_listing,
            particle_status: packet.particle_status,
        };

        *self.client_information.lock().await = info;
    }

    /// Starts the configuration process by sending initial packets.
    pub async fn start_configuration(&self) {
        self.send_bare_packet_now(CCustomPayload::new(
            Identifier::vanilla_static("brand"),
            Box::new(BRAND_PAYLOAD),
        ))
        .await;

        // Send server links if enabled and configured
        if let Some(server_links) = self.server.config.server_links_packet() {
            self.send_bare_packet_now(server_links).await;
        }

        // Send custom report details if configured (vanilla ServerCommonPacketListenerImpl).
        if let Some(report_details) = self.server.custom_report_details_packet() {
            self.send_bare_packet_now(report_details).await;
        }

        // Vanilla `ServerConfigurationPacketListenerImpl.startConfiguration` sends
        // the enabled feature flags unconditionally, after the brand/server links and
        // before `select_known_packs`. The vanilla client gates feature-flagged
        // content on this set; omitting it leaves the connection's flag set empty.
        // Steel runs the vanilla feature set, i.e. just `minecraft:vanilla`.
        self.send_bare_packet_now(CUpdateEnabledFeatures {
            features: vec![Identifier::vanilla_static("vanilla")],
        })
        .await;

        self.send_bare_packet_now(CSelectKnownPacks::new(vec![KnownPack::new(
            "minecraft".to_string(),
            "core".to_string(),
            steel_utils::MC_VERSION.to_string(),
        )]))
        .await;

        // Vanilla `ServerConfigurationPacketListenerImpl` registers this as a
        // configuration task, so the pack is pushed before the client can
        // finish configuration. A declined required pack disconnects in
        // `handle_resource_pack_response`.
        if let Some(pack) = self.server.config.resource_pack_push_packet() {
            self.send_bare_packet_now(pack).await;
        }
    }

    /// Vanilla `ServerCommonPacketListenerImpl.handleResourcePackResponse`.
    ///
    /// Every action except `DECLINED` on a required pack is informational: the
    /// server keeps no per-pack state, so only the refusal matters.
    pub async fn handle_resource_pack_response(&self, packet: SResourcePack) {
        let declined = packet.decoded_action() == Some(ResourcePackAction::Declined);
        let required = self
            .server
            .config
            .resource_pack
            .as_ref()
            .is_some_and(|pack| pack.required);
        if declined && required {
            log::info!(
                "Disconnecting {} due to resource pack {} rejection",
                self.id,
                packet.id
            );
            // Vanilla key: `multiplayer.requiredTexturePrompt.disconnect`.
            self.kick(TextComponent::translated(TranslatedMessage {
                key: "multiplayer.requiredTexturePrompt.disconnect".into(),
                fallback: None,
                args: None,
            }))
            .await;
        }
    }

    /// Handles the select known packs packet during the configuration state.
    pub async fn handle_select_known_packs(&self, packet: SSelectKnownPacks) {
        let sequence_result = self.pre_play_state.lock().select_known_packs();
        if let Err(error) = sequence_result {
            self.reject_unexpected_packet(error).await;
            return;
        }
        log::debug!("Select known packs packet: {packet:?}");

        let registry_cache = Arc::clone(&self.server.registry_cache.registry_packets);
        for encoded_packet in registry_cache.iter() {
            self.send_packet_now(encoded_packet).await;
        }

        // Send the packet for tags
        self.send_packet_now(&self.server.registry_cache.tags_packet)
            .await;

        // Vanilla 26.3 `ServerCodeOfConductConfigurationTask`: when a code of
        // conduct is configured, send it before `CFinishConfiguration` and wait
        // for `SAcceptCodeOfConduct` to complete the configuration handshake.
        if let Some(code_of_conduct) = self.server.code_of_conduct_packet() {
            self.send_bare_packet_now(code_of_conduct).await;
            return;
        }

        // Finish configuration with CFinishConfigurationPacket
        self.send_bare_packet_now(CFinishConfiguration {}).await;
    }

    /// Completes the MC 26.3 code-of-conduct step (`ServerboundAcceptCodeOfConductPacket`)
    /// and sends `CFinishConfiguration`.
    pub async fn handle_accept_code_of_conduct(&self) {
        if self.server.code_of_conduct_packet().is_some() {
            self.send_bare_packet_now(CFinishConfiguration {}).await;
        }
    }

    /// Sends a Configuration-state custom dialog (`ClientboundShowDialogPacket`).
    pub async fn send_config_dialog(&self, dialog: simdnbt::owned::NbtCompound) {
        self.send_bare_packet_now(steel_protocol::packets::game::CConfigShowDialog { dialog })
            .await;
    }

    /// Sends a Configuration-state ping (`ClientboundPingPacket`).
    pub async fn send_config_ping(&self, id: i32) {
        self.send_bare_packet_now(steel_protocol::packets::game::CConfigPing { id })
            .await;
    }

    /// Resets the client's chat state during Configuration (`ClientboundResetChatPacket`).
    pub async fn send_reset_chat(&self) {
        self.send_bare_packet_now(steel_protocol::packets::game::CResetChat {})
            .await;
    }

    /// Sends a Login-state custom plugin query (`ClientboundCustomQueryPacket`).
    pub async fn send_login_custom_query(
        &self,
        transaction_id: i32,
        channel: Identifier,
        data: Vec<u8>,
    ) {
        self.send_bare_packet_now(steel_protocol::packets::login::CCustomQuery {
            transaction_id,
            channel,
            data,
        })
        .await;
    }

    /// Requests a stored cookie during Login (`ClientboundCookieRequestPacket`).
    pub async fn request_login_cookie(&self, key: Identifier) {
        self.send_bare_packet_now(steel_protocol::packets::login::CLoginCookieRequest { key })
            .await;
    }

    /// Finishes the configuration process and transitions to the play state.
    pub(crate) async fn finish_configuration(&self) -> ConnectionAction {
        let sequence_result = self.pre_play_state.lock().finish_configuration();
        let gameprofile = match sequence_result {
            Ok(gameprofile) => gameprofile,
            Err(error) => return self.reject_unexpected_packet(error).await,
        };
        // Admission can reject the join; the client already expects Play disconnect packets.
        self.protocol.store(ConnectionProtocol::Play);
        let Some(reservation) = self.server.try_reserve_player_join(gameprofile.id()) else {
            self.kick(TextComponent::translated(
                translations::MULTIPLAYER_DISCONNECT_DUPLICATE_LOGIN.msg(),
            ))
            .await;
            return ConnectionAction::none();
        };

        let client_info = self.client_information.lock().await.clone();

        let world = Arc::clone(self.server.overworld());
        let entity_id = next_entity_id();

        let session = Arc::new(PlayerSession::new(
            self.server.config.chat_spam_threshold_seconds,
            self.server.config.command_spam_threshold_seconds,
        ));
        let java_connection = JavaConnection::new(
            self.outgoing_queue.clone(),
            self.cancel_token.clone(),
            self.compression.load(),
            Arc::clone(&self.network_writer),
            self.id,
            Arc::clone(&session),
        );
        let connection = Arc::new(PlayerConnection::Java(java_connection));
        let player = Arc::new(Player::new(
            gameprofile,
            connection,
            Arc::clone(&session),
            world,
            Arc::downgrade(&self.server),
            Arc::clone(&self.server.config),
            entity_id,
            client_info,
        ));
        assert!(
            session.bind_initial_player(&player),
            "new client session was already bound to a player"
        );

        let connection = Arc::clone(&player.connection);
        if self
            .connection_updates
            .send(ConnectionUpdate::Upgrade(Arc::clone(&connection)))
            .is_err()
        {
            self.kick("Failed to update connection state".into()).await;
            return ConnectionAction::none();
        }

        tokio::select! {
            () = self.connection_updated.notified() => {}
            () = self.cancel_token.cancelled() => return ConnectionAction::none(),
        }
        reservation.queue_player_join(player);

        ConnectionAction::upgrade(connection)
    }
}
