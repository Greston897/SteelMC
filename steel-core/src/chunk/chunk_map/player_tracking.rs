use super::{CSetChunkCenter, ChunkMap, ChunkPos, Entity, Player, PlayerChunkView};
use crate::chunk::chunk_scheduler::PlayerTicketOperation;
use steel_registry::vanilla_game_rules;
use steel_utils::types::GameType;

impl ChunkMap {
    /// Vanilla `ChunkMap#skipPlayer`: a spectator holds no distance-manager
    /// ticket when the `spectatorsGenerateChunks` game rule is off.
    ///
    /// A skipped player still runs the chunk-tracking path, so it keeps
    /// receiving the chunks other players load; it just never loads or
    /// generates terrain around itself.
    fn skip_player(&self, player: &Player) -> bool {
        skip_player_for(
            player.game_mode(),
            self.world_gen_context
                .world()
                .get_game_rule(&vanilla_game_rules::SPECTATORS_GENERATE_CHUNKS),
        )
    }

    /// Applies the ticket move for one tracking update.
    ///
    /// Vanilla keeps a `skippedPlayer` flag per player: a skipped player never
    /// adds a ticket, and its removal is applied during the *next*
    /// `updatePlayerStatus` (or `remove_player`) rather than at the moment the
    /// game mode changes. The flag is stored as the tracked player state, so
    /// leaving spectator mode re-adds the ticket on the following update.
    fn apply_ticket_ops(
        &self,
        skip_player: bool,
        operations: impl IntoIterator<Item = PlayerTicketOperation>,
    ) {
        if !skip_player {
            let _ = self.scheduling.queue_player_ticket_operations(operations);
            return;
        }

        // Skipped: never add, and retire a ticket left over from before the
        // player became a spectator. Vanilla `ChunkMap.move` removes it here.
        let mut cleanup = Vec::new();
        for operation in operations {
            if let PlayerTicketOperation::Remove { pos, player_id } = operation {
                cleanup.push(PlayerTicketOperation::Remove { pos, player_id });
            }
        }
        if !cleanup.is_empty() {
            let _ = self.scheduling.queue_player_ticket_operations(cleanup);
        }
    }

    /// Updates the player's status in the chunk map.
    pub fn update_player_status(&self, player: &Player) {
        let current_chunk_pos = ChunkPos::from_entity_pos(player.position());
        *player.last_chunk_pos.lock() = current_chunk_pos;
        let view_distance = player.view_distance();

        let new_view = PlayerChunkView::new(current_chunk_pos, view_distance);
        let world = self.world_gen_context.world();
        let player_id = player.gameprofile.id();
        let skip_player = self.skip_player(player);
        let mut last_view_guard = player.last_tracking_view.lock();

        let mut retired_center = None;
        if player.skip_player.get() != skip_player {
            // Vanilla flags a player `skippedPlayer` and forces the next
            // `updatePlayerStatus` to run the full path, which retires the
            // distance-manager ticket when the rule turns the player off and
            // re-adds it when the player stops being skipped. A spectator that
            // never moves would otherwise keep its old ticket forever.
            player.skip_player.set(skip_player);
            retired_center = last_view_guard.as_ref().map(|last_view| last_view.center);
            *last_view_guard = None;
        }

        if last_view_guard.as_ref() != Some(&new_view) {
            if let Some(last_view) = last_view_guard.as_ref() {
                if last_view.center != new_view.center {
                    self.apply_ticket_ops(
                        skip_player,
                        [
                            PlayerTicketOperation::Remove {
                                pos: last_view.center,
                                player_id,
                            },
                            PlayerTicketOperation::Add {
                                pos: new_view.center,
                                player_id,
                            },
                        ],
                    );

                    player.send_packet(CSetChunkCenter {
                        x: new_view.center.0.x,
                        y: new_view.center.0.y,
                    });
                }

                // Track chunks for PlayerAreaMap update
                let mut added_chunks = Vec::new();
                let mut removed_chunks = Vec::new();

                // We lock here to ensure we have unique access for the duration of the diff
                let mut chunk_sender = player.chunk_sender().lock();
                let connection = &*player.connection;
                PlayerChunkView::difference(
                    last_view,
                    &new_view,
                    |pos, ctx: &mut (&mut _, &mut Vec<_>, &mut Vec<_>)| {
                        ctx.0.mark_chunk_pending_to_send(pos);
                        ctx.1.push(pos);
                    },
                    |pos, ctx: &mut (&mut _, &mut Vec<_>, &mut Vec<_>)| {
                        ctx.0.drop_chunk(connection, pos);
                        ctx.2.push(pos);
                    },
                    &mut (&mut chunk_sender, &mut added_chunks, &mut removed_chunks),
                );
                drop(chunk_sender);

                // Update the player area map with the diff
                world.player_area_map.on_player_view_change(
                    player.id(),
                    &added_chunks,
                    &removed_chunks,
                );
            } else {
                self.apply_ticket_ops(
                    skip_player,
                    retired_center
                        .map(|pos| PlayerTicketOperation::Remove { pos, player_id })
                        .into_iter()
                        .chain([PlayerTicketOperation::Add {
                            pos: new_view.center,
                            player_id,
                        }]),
                );

                // Send initial chunk cache center to client
                player.send_packet(CSetChunkCenter {
                    x: new_view.center.0.x,
                    y: new_view.center.0.y,
                });

                let mut chunk_sender = player.chunk_sender().lock();
                new_view.for_each(|pos| {
                    chunk_sender.mark_chunk_pending_to_send(pos);
                });
                drop(chunk_sender);

                // First time - add all chunks in view to player area map
                world.player_area_map.on_player_join(player, &new_view);
            }

            *last_view_guard = Some(new_view);
        }
        drop(last_view_guard);

        // Entity visibility also depends on exact player position, not only
        // chunk-view changes. Vanilla refreshes tracked entities for accepted
        // movement within the same chunk as well.
        let sent_chunks = player.chunk_sender().lock().sent_chunks_snapshot();
        world
            .entity_tracker()
            .update_player(player, &new_view, |chunk| sent_chunks.contains(&chunk));
    }

    /// Removes a player from the chunk map.
    pub fn remove_player(&self, player: &Player) {
        // Keep the view guard through ticket submission so a concurrent tracking
        // update cannot re-add the player before this removal is ordered.
        // Keep the same view -> sender lock order as `update_player_status`.
        let mut last_view = player.last_tracking_view.lock();
        let removed_view = last_view.take();
        // A player that leaves the world starts untracked, so the next join
        // re-runs the full ticket path even if it was a skipped spectator.
        player.skip_player.set(false);
        {
            let mut chunk_sender = player.chunk_sender().lock();
            let mut chunk_send_epoch = player.chunk_send_epoch.lock();
            *chunk_send_epoch = chunk_send_epoch.wrapping_add(1);
            chunk_sender.clear_world_chunks();
            *player.last_chunk_pos.lock() = ChunkPos::new(i32::MAX, i32::MAX);
        }

        if let Some(removed_view) = removed_view {
            let player_id = player.gameprofile.id();
            self.scheduling
                .queue_player_ticket_operation(PlayerTicketOperation::Remove {
                    pos: removed_view.center,
                    player_id,
                });
        }
    }
}

/// Vanilla `ChunkMap#skipPlayer` as a pure predicate.
///
/// Only spectators are affected: every other game mode keeps its ticket
/// regardless of the rule.
fn skip_player_for(game_mode: GameType, spectators_generate_chunks: bool) -> bool {
    game_mode == GameType::Spectator && !spectators_generate_chunks
}

#[cfg(test)]
mod spectator_ticket_tests {
    use super::skip_player_for;
    use steel_utils::types::GameType;

    #[test]
    fn spectator_with_the_rule_off_holds_no_ticket() {
        assert!(skip_player_for(GameType::Spectator, false));
    }

    #[test]
    fn spectator_with_the_rule_on_loads_terrain() {
        assert!(!skip_player_for(GameType::Spectator, true));
    }

    #[test]
    fn the_rule_never_affects_other_game_modes() {
        for game_mode in [
            GameType::Survival,
            GameType::Creative,
            GameType::Adventure,
        ] {
            assert!(!skip_player_for(game_mode, false), "{game_mode:?}");
            assert!(!skip_player_for(game_mode, true), "{game_mode:?}");
        }
    }
}
