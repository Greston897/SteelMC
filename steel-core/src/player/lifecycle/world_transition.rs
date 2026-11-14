use super::*;

impl Player {
    fn apply_post_teleport_transition(&self, post_transition: &TeleportPostTransition) {
        for action in post_transition.actions() {
            match *action {
                TeleportPostAction::PlayPortalSound => {
                    self.send_packet(CLevelEvent::new(
                        level_events::SOUND_PORTAL_TRAVEL,
                        BlockPos::ZERO,
                        0,
                        false,
                    ));
                }
                TeleportPostAction::PlacePortalTicket(target) => {
                    let ticket_position = match target {
                        PortalTicketTarget::Destination => BlockPos::from(self.position()),
                        PortalTicketTarget::Block(pos) => pos,
                    };
                    self.get_world().place_portal_ticket(ticket_position);
                }
            }
        }
    }

    /// Applies an ordinary player transition that has already passed server world-change checks.
    /// Cross-domain player state is restored only by the domain-switch workflow.
    pub(crate) fn change_world_within_domain(
        self: &Arc<Self>,
        teleport_transition: &TeleportTransition,
    ) -> bool {
        let current_world = self.get_world();
        let new_world = Arc::clone(&teleport_transition.target_world);
        if current_world.domain() != new_world.domain() {
            tracing::error!(
                entity_id = self.id(),
                source_domain = current_world.domain(),
                target_domain = new_world.domain(),
                "Refusing player world change outside the domain-switch workflow"
            );
            return false;
        }

        let current_position = self.position();
        let current_rotation = self.rotation();
        let current_velocity = self.velocity();
        let position = teleport_transition.resolved_position(current_position);
        let rotation = teleport_transition.resolved_rotation(current_rotation);
        let velocity =
            teleport_transition.resolved_velocity(current_velocity, current_rotation, rotation);
        self.set_portal_cooldown(teleport_transition.portal_cooldown);
        if !teleport_transition.as_passenger {
            self.stop_riding();
        }
        if Arc::ptr_eq(&current_world, &new_world) {
            if let Err(error) = self.teleport_with_velocity_packet(
                position,
                velocity,
                rotation,
                teleport_transition.position,
                teleport_transition.velocity,
                teleport_transition.rotation,
                teleport_transition.relatives,
            ) {
                panic!(
                    "failed to commit same-world portal teleport for player {}: {error}",
                    self.id()
                );
            }
            self.reset_flying_ticks();
        } else {
            // Vanilla `ServerPlayer.changeDimension` fires `changed_dimension` with the
            // source and target dimension keys, and `nether_travel` when one side of
            // the trip is the Nether (used by `nether/fast_travel` and the
            // `nether/return_to_sender` distances).
            let source_dimension = current_world.key.clone();
            let target_dimension = new_world.key.clone();
            self.trigger_advancement_with_context(
                &steel_registry::vanilla_criteria_triggers::CHANGED_DIMENSION,
                &crate::player::advancement::predicate::TriggerContext::dimension(
                    &source_dimension,
                    &target_dimension,
                ),
            );
            if is_nether_dimension(&source_dimension) || is_nether_dimension(&target_dimension) {
                self.trigger_advancement_with_context(
                    &steel_registry::vanilla_criteria_triggers::NETHER_TRAVEL,
                    &crate::player::advancement::predicate::TriggerContext::nether_travel(
                        position.distance(current_position),
                    ),
                );
            }
            self.reset(new_world, ResetReason::WorldChange);
            if !self.spawn_with_velocity_packet(
                position,
                rotation,
                velocity,
                ResetReason::WorldChange,
                teleport_transition.position,
                teleport_transition.rotation,
                teleport_transition.velocity,
                teleport_transition.relatives,
            ) {
                return false;
            }
            // Vanilla: PlayerList.sendAllPlayerInfo -> inventoryMenu.sendAllDataToRemote
            self.send_inventory_to_remote();
        }
        self.apply_post_teleport_transition(&teleport_transition.post_transition);
        true
    }
}

/// Whether a dimension key names the Nether, as vanilla's `Level.NETHER` check does.
fn is_nether_dimension(dimension: &Identifier) -> bool {
    dimension.namespace == "minecraft" && dimension.path.as_ref() == "the_nether"
}
