use super::*;
use crate::player::advancement::predicate::TriggerContext;
use steel_registry::vanilla_criteria_triggers::CHANGED_DIMENSION;
use steel_registry::vanilla_criteria_triggers::NETHER_TRAVEL;

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
            self.reset(Arc::clone(&new_world), ResetReason::WorldChange);
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
            // Advancement rewards must observe the destination, and failed
            // admission must neither save an entry point nor grant a criterion.
            let source_dimension = &current_world.key;
            let target_dimension = &new_world.key;
            let overworld = Identifier::vanilla_static("overworld");
            let nether = Identifier::vanilla_static("the_nether");
            let entry_position = {
                let mut entered = self.entered_nether_position.lock();
                if source_dimension == &overworld && target_dimension == &nether {
                    *entered = Some(current_position);
                }
                let route = if source_dimension == &nether && target_dimension == &overworld {
                    *entered
                } else {
                    None
                };
                if target_dimension != &nether {
                    *entered = None;
                }
                route
            };
            self.trigger_advancement_with_context(
                &CHANGED_DIMENSION,
                &TriggerContext::dimension(source_dimension, target_dimension),
            );
            if let Some(entry_position) = entry_position {
                self.trigger_advancement_with_context(
                    &NETHER_TRAVEL,
                    &TriggerContext::nether_route(entry_position, position),
                );
            }
            // Vanilla: PlayerList.sendAllPlayerInfo -> inventoryMenu.sendAllDataToRemote
            self.send_inventory_to_remote();
        }
        self.apply_post_teleport_transition(&teleport_transition.post_transition);
        true
    }
}
