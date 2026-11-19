# MC 26.3 — воспроизводимая инвентаризация исходников

Код на момент замера: `e6b3959e61a61f3f410e6a8c7f9f2c6b6cda622d`. Генератор: `.ci/progress_inventory.py`.

Это **статический подсчёт**, не результаты запуска. Тесты = функции с `#[test]`/`#[tokio::test]`, включая условно компилируемые; макрогенерируемые, doc-тесты и внешние тестовые таргеты не учтены. Маркеры ищутся лексически (включая комментарии). LOC включает комментарии и пустые строки. Числа не являются процентом реализации.

## Крейты

| Крейт | .rs | LOC | Тестовых функций | todo!/unimplemented! | Пути |
|---|---:|---:|---:|---:|---|
| steel | 17 | 3973 | 36 | 0 | `steel/src`, `steel/build` |
| steel-login | 10 | 2464 | 22 | 0 | `steel-login/src`, `steel-login/build` |
| steel-macros | 6 | 1189 | 0 | 0 | `steel-macros/src`, `steel-macros/build` |
| steel-protocol | 232 | 13185 | 125 | 0 | `steel-protocol/src`, `steel-protocol/build` |
| steel-registry | 307 | 94739 | 438 | 0 | `steel-registry/src`, `steel-registry/build` |
| steel-worldgen | 76 | 29190 | 98 | 0 | `steel-worldgen/src`, `steel-worldgen/build` |
| steel-utils | 69 | 18947 | 255 | 0 | `steel-utils/src`, `steel-utils/build` |
| steel-core | 1382 | 467582 | 3690 | 0 | `steel-core/src`, `steel-core/build` |
| steel-crypto | 4 | 504 | 10 | 0 | `steel-crypto/src`, `steel-crypto/build` |
| steel-math | 10 | 1055 | 14 | 0 | `steel-math/src`, `steel-math/build` |
| steel-flint | 6 | 1468 | 12 | 0 | `steel-flint/src`, `steel-flint/build` |

## Зоны ядра (строки могут пересекаться; не суммировать)

| Зона | .rs | LOC | Тестовых функций | Маркеров | Пути |
|---|---:|---:|---:|---:|---|
| Реестры, теги, кодеки | 307 | 94739 | 438 | 0 | `steel-registry/src`, `steel-registry/build` |
| Синхронизация конфигурации | 12 | 3077 | 34 | 0 | `steel-core/src/server/registry_cache.rs`, `steel-core/src/player/connection`, `steel-protocol/src/packets/config` |
| Поведение блоков | 250 | 55103 | 353 | 0 | `steel-core/src/behavior/blocks`, `steel-core/src/behavior/block` |
| Поведение предметов | 66 | 12294 | 108 | 0 | `steel-core/src/behavior/items` |
| Block entities | 54 | 19428 | 139 | 0 | `steel-core/src/block_entity` |
| Сущности и AI | 359 | 128818 | 1225 | 0 | `steel-core/src/entity` |
| Игрок и соединение | 73 | 34641 | 331 | 0 | `steel-core/src/player` |
| Инвентарь и меню | 55 | 16311 | 127 | 0 | `steel-core/src/inventory` |
| Команды и права | 162 | 51341 | 419 | 0 | `steel-core/src/command`, `steel-core/src/permission` |
| Мир и тик | 92 | 46365 | 372 | 0 | `steel-core/src/world`, `steel-core/src/server` |
| Чанки, свет, сохранение | 80 | 39985 | 334 | 0 | `steel-core/src/chunk`, `steel-core/src/chunk_saver`, `steel-core/src/level_data` |
| Worldgen | 185 | 62911 | 192 | 0 | `steel-core/src/worldgen`, `steel-worldgen/src` |
| Loot, зачарования | 5 | 1968 | 19 | 0 | `steel-core/src/loot`, `steel-core/src/enchantment_helper.rs`, `steel-core/src/enchantment_helper` |
| Физика, жидкости, порталы | 22 | 7268 | 83 | 0 | `steel-core/src/physics`, `steel-core/src/fluid`, `steel-core/src/portal` |
| Протокол | 232 | 13185 | 125 | 0 | `steel-protocol/src` |

## Объекты и игровые зоны (без заявления о полноте поведения)

| Объекты / зона | .rs | LOC | Тестовых функций | Маркеров | Пути |
|---|---:|---:|---:|---:|---|
| Мирные мобы | 64 | 22836 | 179 | 0 | `steel-core/src/entity/entities/mobs/passive` |
| Нейтральные мобы | 2 | 397 | 3 | 0 | `steel-core/src/entity/entities/mobs/neutral` |
| Враждебные мобы | 44 | 14889 | 43 | 0 | `steel-core/src/entity/entities/mobs/hostile` |
| Боссы | 7 | 4680 | 31 | 0 | `steel-core/src/entity/entities/mobs/bosses` |
| Транспорт | 18 | 5770 | 45 | 0 | `steel-core/src/entity/entities/objects/vehicles` |
| Снаряды | 19 | 7101 | 60 | 0 | `steel-core/src/entity/entities/objects/projectiles` |
| Взрывчатка | 4 | 859 | 9 | 0 | `steel-core/src/entity/entities/objects/explosives` |
| Предметы в мире | 5 | 2451 | 16 | 0 | `steel-core/src/entity/entities/objects/items` |
| Биомы, измерения, пресеты | 7 | 2703 | 1 | 0 | `steel-registry/src/biome.rs`, `steel-registry/src/dimension_type.rs`, `steel-registry/src/world_preset.rs`, `steel-registry/build/biomes.rs`, `steel-registry/build/dimension_types.rs`, `steel-registry/build/dimensions.rs`, `steel-registry/build/world_presets.rs` |

## Пофайловый индекс

Указаны имена тестов из исходника, **не статус их прохождения**.

| Файл | LOC | Тесты | Маркеров |
|---|---:|---|---:|
| `steel/src/config/groups.rs` | 214 | — | 0 |
| `steel/src/config/logging.rs` | 111 | — | 0 |
| `steel/src/config/mod.rs` | 145 | — | 0 |
| `steel/src/config/server.rs` | 279 | — | 0 |
| `steel/src/config/tests.rs` | 538 | `packaged_configs_parse`, `packaged_config_matches_the_runtime_config_defaults`, `packaged_schema_declares_server_thread_settings`, `file_permission_group_store_round_trips_typed_config`, `server_config_defaults_backward_compatible_fields`, `configured_auth_server_flows_to_runtime_config`, `configured_profile_server_flows_to_runtime_config`, `configured_services_server_flows_to_runtime_config`, `validate_allows_zero_simulation_distance`, `configured_core_count_flows_to_the_runtime_config`, `a_configured_core_count_never_exceeds_the_container_limit`, `a_zero_core_setting_falls_back_to_detection`, `configured_thread_counts_parse_and_flow_to_runtime_config`, `validate_rejects_extended_view_distance_without_opt_in`, `validate_rejects_online_mode_without_encryption`, `validate_allows_offline_mode_without_encryption`, `validate_allows_extended_view_distance_with_opt_in`, `validate_rejects_view_distance_above_supported_ticket_range`, `validate_rejects_invalid_auth_server_url`, `validate_allows_http_auth_server_url`, `validate_rejects_unsupported_auth_server_scheme`, `validate_rejects_invalid_services_server_url`, `server_config_defaults_spam_thresholds_for_older_configs`, `log_config_defaults_for_older_configs` | 0 |
| `steel/src/lib.rs` | 213 | — | 0 |
| `steel/src/logger/file.rs` | 237 | `weekly_rotation_uses_elapsed_time_without_day_underflow`, `archive_filename_uses_collision_suffixes`, `startup_archives_existing_non_empty_latest` | 0 |
| `steel/src/logger/history.rs` | 83 | — | 0 |
| `steel/src/logger/input.rs` | 460 | — | 0 |
| `steel/src/logger/mod.rs` | 318 | `rendered_rows_include_wrapping_and_trailing_newlines`, `rendered_rows_ignore_ansi_and_count_tabs_conservatively`, `terminal_newlines_reset_the_output_column` | 0 |
| `steel/src/logger/output.rs` | 88 | — | 0 |
| `steel/src/logger/search.rs` | 181 | `searches_from_the_requested_entry_towards_older_ones`, `match_ranges_are_character_positions` | 0 |
| `steel/src/logger/selection.rs` | 46 | — | 0 |
| `steel/src/logger/state.rs` | 260 | — | 0 |
| `steel/src/logger/suggestions.rs` | 300 | `utf16_positions_map_to_character_positions`, `completion_replaces_its_range_instead_of_appending`, `unicode_suggestions_are_truncated_at_character_boundaries`, `log_rows_consume_reserved_completion_space` | 0 |
| `steel/src/main.rs` | 387 | — | 0 |
| `steel/src/shutdown.rs` | 113 | — | 0 |
| `steel-login/src/authentication.rs` | 241 | `test_positive_simple`, `test_negative_simple`, `test_leading_zero_preserves_positive_sign`, `test_leading_zero_with_high_bit`, `test_negative_twos_complement`, `test_negative_multi_byte`, `test_zero`, `test_empty`, `test_known_notchian_hashes`, `auth_url_defaults_to_mojang_session_server`, `auth_url_uses_configured_endpoint` | 0 |
| `steel-login/src/connection.rs` | 7 | — | 0 |
| `steel-login/src/handlers/config.rs` | 293 | — | 0 |
| `steel-login/src/handlers/login.rs` | 314 | `decrypt_shared_secret_accepts_valid_packet`, `decrypt_shared_secret_rejects_all_failures_alike` | 0 |
| `steel-login/src/handlers/mod.rs` | 5 | — | 0 |
| `steel-login/src/handlers/status.rs` | 57 | — | 0 |
| `steel-login/src/lib.rs` | 26 | — | 0 |
| `steel-login/src/pre_play_state.rs` | 436 | `online_flow_uses_only_the_authenticated_profile`, `offline_flow_uses_the_server_derived_profile`, `skipped_authentication_does_not_advance_login`, `skipped_known_pack_negotiation_does_not_finish_configuration`, `repeated_packets_do_not_advance_the_sequence` | 0 |
| `steel-login/src/tcp_client/tests.rs` | 76 | `login_deadline_matches_vanillas_post_increment_boundary`, `login_deadline_drops_in_flight_packet_processing`, `cancellation_wins_over_ready_packet_processing`, `configuration_handoff_disables_ready_login_deadline` | 0 |
| `steel-login/src/tcp_client.rs` | 1009 | — | 0 |
| `steel-macros/src/behavior.rs` | 40 | — | 0 |
| `steel-macros/src/lib.rs` | 90 | — | 0 |
| `steel-macros/src/packet.rs` | 91 | — | 0 |
| `steel-macros/src/read_from.rs` | 376 | — | 0 |
| `steel-macros/src/strategy.rs` | 117 | — | 0 |
| `steel-macros/src/write_to.rs` | 475 | — | 0 |
| `steel-protocol/src/lib.rs` | 24 | — | 0 |
| `steel-protocol/src/packet_reader.rs` | 352 | `payload_slice_starts_after_framing_and_packet_id`, `uncompressed_payload_slice_skips_compression_length`, `decompressed_payload_slice_starts_after_packet_id`, `rejects_zlib_stream_expanding_past_declared_length`, `rejects_zlib_stream_shorter_than_declared_length`, `accepts_zlib_stream_matching_declared_length` | 0 |
| `steel-protocol/src/packet_traits.rs` | 240 | — | 0 |
| `steel-protocol/src/packet_writer.rs` | 367 | `writes_length_prefixed_frame`, `writes_multi_byte_length_prefix`, `appends_packets_in_order`, `encryption_round_trips_the_frame`, `buffered_writes_share_one_flush`, `write_packet_keeps_flushing_for_urgent_sends` | 0 |
| `steel-protocol/src/packets/common/c_custom_payload.rs` | 22 | — | 0 |
| `steel-protocol/src/packets/common/c_disconnect.rs` | 20 | — | 0 |
| `steel-protocol/src/packets/common/c_keep_alive.rs` | 16 | — | 0 |
| `steel-protocol/src/packets/common/c_pong_response.rs` | 15 | — | 0 |
| `steel-protocol/src/packets/common/c_transfer.rs` | 29 | — | 0 |
| `steel-protocol/src/packets/common/c_update_tags.rs` | 20 | — | 0 |
| `steel-protocol/src/packets/common/mod.rs` | 22 | — | 0 |
| `steel-protocol/src/packets/common/s_client_information.rs` | 80 | `reads_vanilla_byte_fields_without_consuming_following_settings` | 0 |
| `steel-protocol/src/packets/common/s_custom_payload.rs` | 40 | — | 0 |
| `steel-protocol/src/packets/common/s_keep_alive.rs` | 6 | — | 0 |
| `steel-protocol/src/packets/common/s_ping_request.rs` | 6 | — | 0 |
| `steel-protocol/src/packets/config/c_finish_configuration.rs` | 6 | — | 0 |
| `steel-protocol/src/packets/config/c_registry_data.rs` | 32 | — | 0 |
| `steel-protocol/src/packets/config/c_select_known.rs` | 18 | — | 0 |
| `steel-protocol/src/packets/config/c_server_links.rs` | 92 | `built_in_link_encodes_either_left_then_url` | 0 |
| `steel-protocol/src/packets/config/mod.rs` | 19 | — | 0 |
| `steel-protocol/src/packets/config/s_accept_code_of_conduct.rs` | 25 | `reads_empty_unit_payload` | 0 |
| `steel-protocol/src/packets/config/s_finish_configuration.rs` | 4 | — | 0 |
| `steel-protocol/src/packets/config/s_select_known.rs` | 47 | `accepts_64_packs`, `rejects_65_packs` | 0 |
| `steel-protocol/src/packets/game/c_add_entity.rs` | 130 | `test_zero_velocity`, `test_tiny_velocity_is_zero`, `test_non_zero_velocity`, `test_negative_velocity_uses_absolute_scale`, `test_velocity_with_scale` | 0 |
| `steel-protocol/src/packets/game/c_add_transient_block.rs` | 46 | `encodes_block_pos_then_varint_state` | 0 |
| `steel-protocol/src/packets/game/c_animate.rs` | 48 | — | 0 |
| `steel-protocol/src/packets/game/c_award_stats.rs` | 26 | — | 0 |
| `steel-protocol/src/packets/game/c_block_changed_ack.rs` | 9 | — | 0 |
| `steel-protocol/src/packets/game/c_block_destruction.rs` | 12 | — | 0 |
| `steel-protocol/src/packets/game/c_block_entity_data.rs` | 23 | — | 0 |
| `steel-protocol/src/packets/game/c_block_event.rs` | 41 | — | 0 |
| `steel-protocol/src/packets/game/c_block_update.rs` | 10 | — | 0 |
| `steel-protocol/src/packets/game/c_boss_event.rs` | 263 | `remove_writes_uuid_then_action_id`, `update_progress_writes_big_endian_float`, `add_writes_colour_and_overlay_ordinals` | 0 |
| `steel-protocol/src/packets/game/c_bundle_delimiter.rs` | 27 | — | 0 |
| `steel-protocol/src/packets/game/c_change_difficulty.rs` | 15 | — | 0 |
| `steel-protocol/src/packets/game/c_chunks_biomes.rs` | 75 | `encodes_packed_chunk_pos_and_single_byte_array_prefix` | 0 |
| `steel-protocol/src/packets/game/c_clear_dialog.rs` | 9 | — | 0 |
| `steel-protocol/src/packets/game/c_clear_titles.rs` | 18 | — | 0 |
| `steel-protocol/src/packets/game/c_code_of_conduct.rs` | 32 | `encodes_plain_utf8_without_optional_flag` | 0 |
| `steel-protocol/src/packets/game/c_command_suggestions.rs` | 62 | — | 0 |
| `steel-protocol/src/packets/game/c_commands.rs` | 554 | `restricted_nodes_set_the_26_2_protocol_flag`, `command_node_flags_can_be_combined`, `command_argument_tail_uses_the_26_3_registry_ids`, `resource_selector_writes_its_registry_key` | 0 |
| `steel-protocol/src/packets/game/c_config_ping.rs` | 12 | — | 0 |
| `steel-protocol/src/packets/game/c_container_close.rs` | 9 | — | 0 |
| `steel-protocol/src/packets/game/c_container_set_content.rs` | 13 | — | 0 |
| `steel-protocol/src/packets/game/c_container_set_data.rs` | 11 | — | 0 |
| `steel-protocol/src/packets/game/c_container_set_slot.rs` | 13 | — | 0 |
| `steel-protocol/src/packets/game/c_cookie_request.rs` | 13 | — | 0 |
| `steel-protocol/src/packets/game/c_cooldown.rs` | 14 | — | 0 |
| `steel-protocol/src/packets/game/c_custom_chat_completions.rs` | 46 | `encodes_action_then_string_list` | 0 |
| `steel-protocol/src/packets/game/c_custom_report_details.rs` | 34 | — | 0 |
| `steel-protocol/src/packets/game/c_damage_event.rs` | 25 | — | 0 |
| `steel-protocol/src/packets/game/c_debug_block_value.rs` | 73 | `encodes_block_pos_subscription_id_and_optional_value` | 0 |
| `steel-protocol/src/packets/game/c_debug_chunk_value.rs` | 55 | `encodes_packed_chunk_pos_then_subscription_update` | 0 |
| `steel-protocol/src/packets/game/c_debug_entity_value.rs` | 48 | `encodes_varint_entity_id_then_update` | 0 |
| `steel-protocol/src/packets/game/c_debug_event.rs` | 46 | `encodes_subscription_id_and_direct_event_payload` | 0 |
| `steel-protocol/src/packets/game/c_debug_sample.rs` | 52 | `writes_long_array_before_sample_type` | 0 |
| `steel-protocol/src/packets/game/c_delete_chat.rs` | 67 | `encodes_cached_and_full_message_signatures` | 0 |
| `steel-protocol/src/packets/game/c_entity_event.rs` | 11 | — | 0 |
| `steel-protocol/src/packets/game/c_entity_position_sync.rs` | 130 | `linear_path_writes_type_then_position`, `stepped_path_writes_prefixed_steps` | 0 |
| `steel-protocol/src/packets/game/c_explode.rs` | 225 | `writes_fields_in_vanilla_wire_order`, `writes_direct_explosion_sound_holder`, `rejects_invalid_explosion_particle_weights` | 0 |
| `steel-protocol/src/packets/game/c_game_event.rs` | 28 | — | 0 |
| `steel-protocol/src/packets/game/c_game_rule_values.rs` | 56 | `encodes_namespaced_game_rule_keys` | 0 |
| `steel-protocol/src/packets/game/c_game_test_highlight_pos.rs` | 36 | `encodes_exact_sixteen_bytes` | 0 |
| `steel-protocol/src/packets/game/c_hurt_animation.rs` | 12 | — | 0 |
| `steel-protocol/src/packets/game/c_level_event.rs` | 52 | — | 0 |
| `steel-protocol/src/packets/game/c_level_particles.rs` | 101 | `writes_fields_in_vanilla_wire_order` | 0 |
| `steel-protocol/src/packets/game/c_login.rs` | 43 | — | 0 |
| `steel-protocol/src/packets/game/c_low_disk_space_warning.rs` | 9 | — | 0 |
| `steel-protocol/src/packets/game/c_map_item_data.rs` | 259 | `full_packet_encodes_vanilla_field_order_with_width_discriminator`, `empty_update_skips_both_optional_sections`, `decoration_name_is_optional_component`, `patch_without_decorations_writes_width_directly` | 0 |
| `steel-protocol/src/packets/game/c_merchant_offers.rs` | 176 | `encodes_merchant_offers_in_exact_vanilla_order` | 0 |
| `steel-protocol/src/packets/game/c_mount_screen_open.rs` | 40 | `encodes_entity_id_as_big_endian_i32` | 0 |
| `steel-protocol/src/packets/game/c_move_entity.rs` | 393 | `pos_packet_matches_vanilla_wire_format`, `pos_packet_writes_stepped_delta_with_packed_step_count`, `pos_rot_packet_matches_vanilla_wire_format`, `rot_packet_matches_vanilla_wire_format`, `test_encode_matches_java_rounding`, `test_calc_delta`, `test_angle_byte` | 0 |
| `steel-protocol/src/packets/game/c_move_minecart_along_track.rs` | 70 | — | 0 |
| `steel-protocol/src/packets/game/c_move_vehicle.rs` | 12 | — | 0 |
| `steel-protocol/src/packets/game/c_open_book.rs` | 46 | `open_book_writes_the_hand_as_a_varint` | 0 |
| `steel-protocol/src/packets/game/c_open_screen.rs` | 38 | — | 0 |
| `steel-protocol/src/packets/game/c_open_sign_editor.rs` | 39 | `open_sign_editor_writes_slot_as_varint` | 0 |
| `steel-protocol/src/packets/game/c_ping.rs` | 12 | — | 0 |
| `steel-protocol/src/packets/game/c_place_ghost_recipe.rs` | 33 | — | 0 |
| `steel-protocol/src/packets/game/c_player_abilities.rs` | 72 | — | 0 |
| `steel-protocol/src/packets/game/c_player_combat_end.rs` | 13 | — | 0 |
| `steel-protocol/src/packets/game/c_player_combat_enter.rs` | 9 | — | 0 |
| `steel-protocol/src/packets/game/c_player_combat_kill.rs` | 13 | — | 0 |
| `steel-protocol/src/packets/game/c_player_info_update.rs` | 417 | `initial_player_info_preserves_signed_texture_property` | 0 |
| `steel-protocol/src/packets/game/c_player_look_at.rs` | 92 | `position_target_encodes_without_entity_suffix`, `entity_target_encodes_id_and_anchor_after_presence_flag` | 0 |
| `steel-protocol/src/packets/game/c_player_position.rs` | 229 | `delta_matches_vanilla_relative_delta_set`, `absolute_with_velocity_preserves_delta_movement` | 0 |
| `steel-protocol/src/packets/game/c_player_rotation.rs` | 35 | — | 0 |
| `steel-protocol/src/packets/game/c_post_effects.rs` | 21 | — | 0 |
| `steel-protocol/src/packets/game/c_projectile_power.rs` | 15 | — | 0 |
| `steel-protocol/src/packets/game/c_recipe_book.rs` | 300 | `update_recipes_writes_set_count_then_stonecutter_count`, `recipe_book_add_ends_with_the_replace_flag`, `optional_group_is_encoded_plus_one`, `recipe_book_remove_writes_indices`, `settings_write_four_unprefixed_pairs` | 0 |
| `steel-protocol/src/packets/game/c_remove_entities.rs` | 23 | — | 0 |
| `steel-protocol/src/packets/game/c_remove_mob_effect.rs` | 44 | `remove_mob_effect_uses_raw_holder_registry_id` | 0 |
| `steel-protocol/src/packets/game/c_remove_player_info.rs` | 21 | — | 0 |
| `steel-protocol/src/packets/game/c_reset_chat.rs` | 9 | — | 0 |
| `steel-protocol/src/packets/game/c_reset_score.rs` | 46 | `encodes_nullable_objective_name` | 0 |
| `steel-protocol/src/packets/game/c_resource_pack_pop.rs` | 26 | — | 0 |
| `steel-protocol/src/packets/game/c_resource_pack_push.rs` | 83 | `pack_push_writes_vanilla_field_order` | 0 |
| `steel-protocol/src/packets/game/c_respawn.rs` | 20 | — | 0 |
| `steel-protocol/src/packets/game/c_rotate_head.rs` | 14 | — | 0 |
| `steel-protocol/src/packets/game/c_section_blocks_update.rs` | 79 | `writes_changes_with_section_relative_positions` | 0 |
| `steel-protocol/src/packets/game/c_select_advancements_tab.rs` | 16 | — | 0 |
| `steel-protocol/src/packets/game/c_server_data.rs` | 29 | — | 0 |
| `steel-protocol/src/packets/game/c_server_links.rs` | 3 | — | 0 |
| `steel-protocol/src/packets/game/c_set_action_bar_text.rs` | 19 | — | 0 |
| `steel-protocol/src/packets/game/c_set_camera.rs` | 9 | — | 0 |
| `steel-protocol/src/packets/game/c_set_cursor_item.rs` | 8 | — | 0 |
| `steel-protocol/src/packets/game/c_set_default_spawn_position.rs` | 17 | — | 0 |
| `steel-protocol/src/packets/game/c_set_display_objective.rs` | 15 | — | 0 |
| `steel-protocol/src/packets/game/c_set_entity_data.rs` | 47 | — | 0 |
| `steel-protocol/src/packets/game/c_set_entity_link.rs` | 39 | `entity_link_packet_uses_vanilla_raw_ints` | 0 |
| `steel-protocol/src/packets/game/c_set_entity_motion.rs` | 42 | — | 0 |
| `steel-protocol/src/packets/game/c_set_equipment.rs` | 116 | `equipment_packet_uses_vanilla_continue_bit`, `equipment_packet_rejects_empty_slot_updates` | 0 |
| `steel-protocol/src/packets/game/c_set_experience.rs` | 12 | — | 0 |
| `steel-protocol/src/packets/game/c_set_health.rs` | 14 | — | 0 |
| `steel-protocol/src/packets/game/c_set_held_slot.rs` | 9 | — | 0 |
| `steel-protocol/src/packets/game/c_set_objective.rs` | 86 | — | 0 |
| `steel-protocol/src/packets/game/c_set_passengers.rs` | 27 | — | 0 |
| `steel-protocol/src/packets/game/c_set_player_inventory.rs` | 33 | `player_inventory_slot_is_encoded_as_varint_before_item_stack` | 0 |
| `steel-protocol/src/packets/game/c_set_player_team.rs` | 172 | `add_team_encodes_26_3_parameters_order_and_player_list` | 0 |
| `steel-protocol/src/packets/game/c_set_score.rs` | 87 | `encodes_single_score_with_optional_trailers` | 0 |
| `steel-protocol/src/packets/game/c_set_simulation_distance.rs` | 13 | — | 0 |
| `steel-protocol/src/packets/game/c_set_subtitle_text.rs` | 19 | — | 0 |
| `steel-protocol/src/packets/game/c_set_time.rs` | 37 | — | 0 |
| `steel-protocol/src/packets/game/c_set_title_text.rs` | 19 | — | 0 |
| `steel-protocol/src/packets/game/c_set_titles_animation.rs` | 26 | — | 0 |
| `steel-protocol/src/packets/game/c_show_dialog.rs` | 117 | `registry_holder_writes_id_plus_one`, `inline_notice_writes_zero_holder_then_network_nbt_compound` | 0 |
| `steel-protocol/src/packets/game/c_sound.rs` | 203 | `registered_sound_packet_uses_holder_id` | 0 |
| `steel-protocol/src/packets/game/c_sound_entity.rs` | 25 | — | 0 |
| `steel-protocol/src/packets/game/c_start_configuration.rs` | 9 | — | 0 |
| `steel-protocol/src/packets/game/c_stop_sound.rs` | 41 | — | 0 |
| `steel-protocol/src/packets/game/c_store_cookie.rs` | 62 | `encodes_valid_cookie_and_rejects_over_five_kib` | 0 |
| `steel-protocol/src/packets/game/c_swing_animation.rs` | 59 | `writes_entity_hand_and_animation` | 0 |
| `steel-protocol/src/packets/game/c_tab_list.rs` | 64 | — | 0 |
| `steel-protocol/src/packets/game/c_tag_query.rs` | 19 | — | 0 |
| `steel-protocol/src/packets/game/c_take_item_entity.rs` | 47 | — | 0 |
| `steel-protocol/src/packets/game/c_teleport_entity.rs` | 73 | `teleport_entity_preserves_relative_flags_and_motion` | 0 |
| `steel-protocol/src/packets/game/c_test_instance_block_status.rs` | 38 | — | 0 |
| `steel-protocol/src/packets/game/c_ticking_state.rs` | 24 | — | 0 |
| `steel-protocol/src/packets/game/c_ticking_step.rs` | 20 | — | 0 |
| `steel-protocol/src/packets/game/c_update_advancement.rs` | 37 | — | 0 |
| `steel-protocol/src/packets/game/c_update_attributes.rs` | 56 | — | 0 |
| `steel-protocol/src/packets/game/c_update_enabled_features.rs` | 14 | — | 0 |
| `steel-protocol/src/packets/game/c_update_mob_effect.rs` | 94 | `update_mob_effect_uses_raw_holder_registry_id` | 0 |
| `steel-protocol/src/packets/game/c_waypoint.rs` | 320 | `track_vec3i_encodes_exact_vanilla_layout`, `untrack_encodes_empty_tracked_waypoint`, `update_chunk_and_azimuth_encode_exact_type_ordinals` | 0 |
| `steel-protocol/src/packets/game/chat/command.rs` | 121 | — | 0 |
| `steel-protocol/src/packets/game/chat/message.rs` | 43 | — | 0 |
| `steel-protocol/src/packets/game/chat/mod.rs` | 11 | — | 0 |
| `steel-protocol/src/packets/game/chat/player.rs` | 163 | — | 0 |
| `steel-protocol/src/packets/game/chat/session.rs` | 201 | `accepts_vanilla_maximum_field_lengths`, `rejects_fields_above_vanilla_limits`, `rejects_negative_and_extreme_field_lengths`, `rejects_truncated_field_bodies` | 0 |
| `steel-protocol/src/packets/game/chat/system.rs` | 19 | — | 0 |
| `steel-protocol/src/packets/game/chunk/batch.rs` | 21 | — | 0 |
| `steel-protocol/src/packets/game/chunk/data.rs` | 53 | — | 0 |
| `steel-protocol/src/packets/game/chunk/light.rs` | 68 | `writes_chunk_coordinates_as_varints_and_trims_empty_masks` | 0 |
| `steel-protocol/src/packets/game/chunk/mod.rs` | 12 | — | 0 |
| `steel-protocol/src/packets/game/chunk/tracking.rs` | 27 | — | 0 |
| `steel-protocol/src/packets/game/mod.rs` | 395 | — | 0 |
| `steel-protocol/src/packets/game/s_accept_teleportation.rs` | 50 | `accept_teleportation_reads_position_after_id` | 0 |
| `steel-protocol/src/packets/game/s_attack.rs` | 8 | — | 0 |
| `steel-protocol/src/packets/game/s_block_entity_tag_query.rs` | 14 | — | 0 |
| `steel-protocol/src/packets/game/s_bundle_item_selected.rs` | 14 | — | 0 |
| `steel-protocol/src/packets/game/s_change_difficulty.rs` | 12 | — | 0 |
| `steel-protocol/src/packets/game/s_change_game_mode.rs` | 7 | — | 0 |
| `steel-protocol/src/packets/game/s_client_command.rs` | 25 | — | 0 |
| `steel-protocol/src/packets/game/s_client_tick_end.rs` | 4 | — | 0 |
| `steel-protocol/src/packets/game/s_command_suggestion.rs` | 15 | — | 0 |
| `steel-protocol/src/packets/game/s_configuration_acknowledged.rs` | 7 | — | 0 |
| `steel-protocol/src/packets/game/s_container_button_click.rs` | 9 | — | 0 |
| `steel-protocol/src/packets/game/s_container_click.rs` | 185 | `unknown_click_type_falls_back_to_pickup`, `rejects_changed_slot_count_above_vanilla_limit` | 0 |
| `steel-protocol/src/packets/game/s_container_close.rs` | 7 | — | 0 |
| `steel-protocol/src/packets/game/s_container_slot_state_changed.rs` | 10 | — | 0 |
| `steel-protocol/src/packets/game/s_cookie_response.rs` | 3 | — | 0 |
| `steel-protocol/src/packets/game/s_custom_click_action.rs` | 104 | `decodes_absent_and_present_length_prefixed_optional_tag` | 0 |
| `steel-protocol/src/packets/game/s_debug_subscription_request.rs` | 32 | — | 0 |
| `steel-protocol/src/packets/game/s_edit_book.rs` | 144 | `reads_pages_without_a_title`, `reads_signing_title_and_pages`, `rejects_more_pages_than_vanilla_allows`, `rejects_pages_longer_than_vanilla_allows`, `rejects_titles_longer_than_vanilla_allows`, `accepts_the_vanilla_limits_at_the_boundary` | 0 |
| `steel-protocol/src/packets/game/s_entity_tag_query.rs` | 14 | — | 0 |
| `steel-protocol/src/packets/game/s_interact.rs` | 55 | `interact_packet_reads_vanilla_field_order` | 0 |
| `steel-protocol/src/packets/game/s_jigsaw_generate.rs` | 16 | — | 0 |
| `steel-protocol/src/packets/game/s_lock_difficulty.rs` | 10 | — | 0 |
| `steel-protocol/src/packets/game/s_move_player.rs` | 146 | — | 0 |
| `steel-protocol/src/packets/game/s_move_vehicle.rs` | 11 | — | 0 |
| `steel-protocol/src/packets/game/s_paddle_boat.rs` | 32 | `flags_decode_in_wire_order` | 0 |
| `steel-protocol/src/packets/game/s_pick_item_from_block.rs` | 9 | — | 0 |
| `steel-protocol/src/packets/game/s_pick_item_from_entity.rs` | 13 | — | 0 |
| `steel-protocol/src/packets/game/s_place_recipe.rs` | 16 | — | 0 |
| `steel-protocol/src/packets/game/s_player_abilities.rs` | 19 | — | 0 |
| `steel-protocol/src/packets/game/s_player_action.rs` | 28 | — | 0 |
| `steel-protocol/src/packets/game/s_player_command.rs` | 40 | — | 0 |
| `steel-protocol/src/packets/game/s_player_input.rs` | 60 | — | 0 |
| `steel-protocol/src/packets/game/s_player_load.rs` | 4 | — | 0 |
| `steel-protocol/src/packets/game/s_pong.rs` | 10 | — | 0 |
| `steel-protocol/src/packets/game/s_punch.rs` | 8 | — | 0 |
| `steel-protocol/src/packets/game/s_recipe_book_change_settings.rs` | 15 | — | 0 |
| `steel-protocol/src/packets/game/s_recipe_book_seen_recipe.rs` | 28 | `reads_varint_recipe_display_id` | 0 |
| `steel-protocol/src/packets/game/s_rename_item.rs` | 91 | `accepts_vanilla_utf16_boundary`, `rejects_more_than_vanilla_utf16_boundary`, `accepts_multibyte_name_above_the_old_byte_bound`, `rejects_encoded_length_above_vanilla_maximum`, `malformed_utf8_uses_replacement_characters`, `malformed_surrogate_sequence_uses_one_replacement_character` | 0 |
| `steel-protocol/src/packets/game/s_resource_pack.rs` | 64 | — | 0 |
| `steel-protocol/src/packets/game/s_seen_advancement.rs` | 9 | — | 0 |
| `steel-protocol/src/packets/game/s_select_trade.rs` | 11 | — | 0 |
| `steel-protocol/src/packets/game/s_set_beacon.rs` | 33 | — | 0 |
| `steel-protocol/src/packets/game/s_set_carried_item.rs` | 6 | — | 0 |
| `steel-protocol/src/packets/game/s_set_command_block.rs` | 18 | — | 0 |
| `steel-protocol/src/packets/game/s_set_command_minecart.rs` | 15 | — | 0 |
| `steel-protocol/src/packets/game/s_set_creative_mode_slot.rs` | 22 | — | 0 |
| `steel-protocol/src/packets/game/s_set_game_rule.rs` | 63 | `decodes_list_of_identifier_value_entries` | 0 |
| `steel-protocol/src/packets/game/s_set_jigsaw_block.rs` | 72 | `reads_string_joint_before_selection_and_placement_priorities` | 0 |
| `steel-protocol/src/packets/game/s_set_structure_block.rs` | 143 | — | 0 |
| `steel-protocol/src/packets/game/s_set_test_block.rs` | 16 | — | 0 |
| `steel-protocol/src/packets/game/s_sign_update.rs` | 60 | `sign_update_reads_lines_before_slot` | 0 |
| `steel-protocol/src/packets/game/s_spectator_action.rs` | 55 | `reads_absent_entity_id_from_zero_varint`, `reads_present_entity_id_as_optional_varint_minus_one` | 0 |
| `steel-protocol/src/packets/game/s_teleport_to_entity.rs` | 11 | — | 0 |
| `steel-protocol/src/packets/game/s_test_instance_block_action.rs` | 228 | — | 0 |
| `steel-protocol/src/packets/game/s_use_item.rs` | 15 | — | 0 |
| `steel-protocol/src/packets/game/s_use_item_on.rs` | 14 | — | 0 |
| `steel-protocol/src/packets/game/world_border.rs` | 58 | — | 0 |
| `steel-protocol/src/packets/handshake/mod.rs` | 36 | — | 0 |
| `steel-protocol/src/packets/login/c_custom_query.rs` | 68 | `writes_unprefixed_trailing_payload` | 0 |
| `steel-protocol/src/packets/login/c_hello.rs` | 31 | — | 0 |
| `steel-protocol/src/packets/login/c_login_compression.rs` | 16 | — | 0 |
| `steel-protocol/src/packets/login/c_login_cookie_request.rs` | 13 | — | 0 |
| `steel-protocol/src/packets/login/c_login_disconnect.rs` | 19 | — | 0 |
| `steel-protocol/src/packets/login/c_login_finished.rs` | 21 | — | 0 |
| `steel-protocol/src/packets/login/mod.rs` | 25 | — | 0 |
| `steel-protocol/src/packets/login/s_cookie_response.rs` | 74 | `enforces_five_kib_cookie_cap` | 0 |
| `steel-protocol/src/packets/login/s_custom_query_answer.rs` | 64 | `reads_declined_and_raw_trailing_answers` | 0 |
| `steel-protocol/src/packets/login/s_hello.rs` | 9 | — | 0 |
| `steel-protocol/src/packets/login/s_key.rs` | 16 | — | 0 |
| `steel-protocol/src/packets/login/s_login_acknowledged.rs` | 4 | — | 0 |
| `steel-protocol/src/packets/mod.rs` | 11 | — | 0 |
| `steel-protocol/src/packets/shared_implementation.rs` | 25 | — | 0 |
| `steel-protocol/src/packets/status/c_status_response.rs` | 71 | `secure_chat_enforcement_uses_vanilla_json_name` | 0 |
| `steel-protocol/src/packets/status/mod.rs` | 10 | — | 0 |
| `steel-protocol/src/packets/status/s_status_request.rs` | 4 | — | 0 |
| `steel-protocol/src/utils.rs` | 275 | — | 0 |
| `steel-registry/build/advancements/criterion.rs` | 31 | — | 0 |
| `steel-registry/build/advancements/display.rs` | 76 | — | 0 |
| `steel-registry/build/advancements/mod.rs` | 160 | — | 0 |
| `steel-registry/build/attributes.rs` | 74 | — | 0 |
| `steel-registry/build/banner_patterns.rs` | 100 | — | 0 |
| `steel-registry/build/biomes.rs` | 615 | — | 0 |
| `steel-registry/build/block_entity_types.rs` | 76 | — | 0 |
| `steel-registry/build/block_state_defaults.rs` | 144 | — | 0 |
| `steel-registry/build/block_transformers.rs` | 350 | — | 0 |
| `steel-registry/build/blocks.rs` | 1125 | — | 0 |
| `steel-registry/build/build.rs` | 374 | — | 0 |
| `steel-registry/build/carvers.rs` | 434 | — | 0 |
| `steel-registry/build/cat_sound_variants.rs` | 117 | — | 0 |
| `steel-registry/build/cat_variants.rs` | 144 | — | 0 |
| `steel-registry/build/chat_types.rs` | 144 | — | 0 |
| `steel-registry/build/chicken_sound_variants.rs` | 91 | — | 0 |
| `steel-registry/build/chicken_variants.rs` | 84 | — | 0 |
| `steel-registry/build/context_providers.rs` | 555 | — | 0 |
| `steel-registry/build/cow_sound_variants.rs` | 66 | — | 0 |
| `steel-registry/build/cow_variants.rs` | 83 | — | 0 |
| `steel-registry/build/custom_stats.rs` | 54 | — | 0 |
| `steel-registry/build/damage_types.rs` | 149 | — | 0 |
| `steel-registry/build/datapack.rs` | 104 | — | 0 |
| `steel-registry/build/decorated_pot_patterns.rs` | 83 | — | 0 |
| `steel-registry/build/dialogs.rs` | 163 | — | 0 |
| `steel-registry/build/dimension_types.rs` | 564 | — | 0 |
| `steel-registry/build/dimensions.rs` | 258 | — | 0 |
| `steel-registry/build/enchantment_providers.rs` | 163 | — | 0 |
| `steel-registry/build/enchantments/effects.rs` | 690 | — | 0 |
| `steel-registry/build/enchantments/generation.rs` | 205 | — | 0 |
| `steel-registry/build/enchantments/mod.rs` | 604 | — | 0 |
| `steel-registry/build/enchantments/nbt.rs` | 266 | — | 0 |
| `steel-registry/build/enchantments/parsing.rs` | 658 | — | 0 |
| `steel-registry/build/entities.rs` | 325 | — | 0 |
| `steel-registry/build/entity_data.rs` | 1443 | — | 0 |
| `steel-registry/build/features/common.rs` | 224 | — | 0 |
| `steel-registry/build/features/configured.rs` | 783 | — | 0 |
| `steel-registry/build/features/data.rs` | 1989 | — | 0 |
| `steel-registry/build/features/mod.rs` | 228 | — | 0 |
| `steel-registry/build/features/nbt.rs` | 659 | — | 0 |
| `steel-registry/build/features/placement.rs` | 288 | — | 0 |
| `steel-registry/build/features/providers.rs` | 370 | — | 0 |
| `steel-registry/build/features/structures.rs` | 561 | — | 0 |
| `steel-registry/build/fluids.rs` | 137 | — | 0 |
| `steel-registry/build/frog_variants.rs` | 67 | — | 0 |
| `steel-registry/build/functions.rs` | 157 | — | 0 |
| `steel-registry/build/game_events.rs` | 86 | — | 0 |
| `steel-registry/build/game_rules.rs` | 108 | — | 0 |
| `steel-registry/build/gametests.rs` | 324 | — | 0 |
| `steel-registry/build/generator_functions.rs` | 300 | — | 0 |
| `steel-registry/build/instruments.rs` | 101 | — | 0 |
| `steel-registry/build/items/attributes.rs` | 147 | — | 0 |
| `steel-registry/build/items/basic_components.rs` | 143 | — | 0 |
| `steel-registry/build/items/builder.rs` | 822 | — | 0 |
| `steel-registry/build/items/consumables.rs` | 258 | — | 0 |
| `steel-registry/build/items/kinetic.rs` | 101 | — | 0 |
| `steel-registry/build/items/metadata.rs` | 455 | — | 0 |
| `steel-registry/build/items/mod.rs` | 336 | — | 0 |
| `steel-registry/build/items/weapons.rs` | 190 | — | 0 |
| `steel-registry/build/jukebox_songs.rs` | 98 | — | 0 |
| `steel-registry/build/level_events.rs` | 52 | — | 0 |
| `steel-registry/build/loot_tables/conditions.rs` | 267 | — | 0 |
| `steel-registry/build/loot_tables/entries.rs` | 180 | — | 0 |
| `steel-registry/build/loot_tables/functions.rs` | 385 | — | 0 |
| `steel-registry/build/loot_tables/mod.rs` | 827 | — | 0 |
| `steel-registry/build/loot_tables/values.rs` | 469 | — | 0 |
| `steel-registry/build/map_decoration_types.rs` | 72 | — | 0 |
| `steel-registry/build/menu_types.rs` | 66 | — | 0 |
| `steel-registry/build/mob_effects.rs` | 216 | — | 0 |
| `steel-registry/build/packets.rs` | 80 | — | 0 |
| `steel-registry/build/painting_variants.rs` | 137 | — | 0 |
| `steel-registry/build/particle_types.rs` | 97 | — | 0 |
| `steel-registry/build/pig_sound_variants.rs` | 91 | — | 0 |
| `steel-registry/build/pig_variants.rs` | 82 | — | 0 |
| `steel-registry/build/poi_types.rs` | 168 | — | 0 |
| `steel-registry/build/position_source_types.rs` | 61 | — | 0 |
| `steel-registry/build/potions.rs` | 78 | — | 0 |
| `steel-registry/build/predicates.rs` | 58 | — | 0 |
| `steel-registry/build/recipes/cooking.rs` | 83 | — | 0 |
| `steel-registry/build/recipes/crafting.rs` | 318 | — | 0 |
| `steel-registry/build/recipes/mod.rs` | 129 | — | 0 |
| `steel-registry/build/recipes/serializer.rs` | 66 | — | 0 |
| `steel-registry/build/recipes/shared.rs` | 216 | — | 0 |
| `steel-registry/build/recipes/smithing.rs` | 69 | — | 0 |
| `steel-registry/build/recipes/stonecutting.rs` | 19 | — | 0 |
| `steel-registry/build/shared_structs.rs` | 127 | — | 0 |
| `steel-registry/build/sound_events.rs` | 67 | — | 0 |
| `steel-registry/build/sound_types.rs` | 76 | — | 0 |
| `steel-registry/build/structure/mod.rs` | 10 | — | 0 |
| `steel-registry/build/structure/processors.rs` | 384 | — | 0 |
| `steel-registry/build/structure/sets.rs` | 1329 | — | 0 |
| `steel-registry/build/structure/template_pools.rs` | 539 | — | 0 |
| `steel-registry/build/sulfur_cube_archetypes.rs` | 262 | — | 0 |
| `steel-registry/build/suspicious_stew_effects.rs` | 67 | — | 0 |
| `steel-registry/build/tags/block.rs` | 97 | — | 0 |
| `steel-registry/build/tags/common.rs` | 210 | — | 0 |
| `steel-registry/build/tags/fluid.rs` | 65 | — | 0 |
| `steel-registry/build/tags/item.rs` | 91 | — | 0 |
| `steel-registry/build/tags/mod.rs` | 115 | — | 0 |
| `steel-registry/build/timelines.rs` | 271 | — | 0 |
| `steel-registry/build/translations.rs` | 126 | — | 0 |
| `steel-registry/build/trial_spawners.rs` | 245 | — | 0 |
| `steel-registry/build/trim_materials.rs` | 72 | — | 0 |
| `steel-registry/build/trim_patterns.rs` | 80 | — | 0 |
| `steel-registry/build/villager_professions.rs` | 77 | — | 0 |
| `steel-registry/build/villager_trades.rs` | 613 | — | 0 |
| `steel-registry/build/villager_types.rs` | 54 | — | 0 |
| `steel-registry/build/wolf_sound_variants.rs` | 109 | — | 0 |
| `steel-registry/build/wolf_variants.rs` | 89 | — | 0 |
| `steel-registry/build/world_clocks.rs` | 70 | — | 0 |
| `steel-registry/build/world_presets.rs` | 177 | — | 0 |
| `steel-registry/build/zombie_nautilus_variants.rs` | 81 | — | 0 |
| `steel-registry/src/advancement/criterion.rs` | 115 | — | 0 |
| `steel-registry/src/advancement/display.rs` | 95 | — | 0 |
| `steel-registry/src/advancement/mod.rs` | 248 | — | 0 |
| `steel-registry/src/advancement/positioner.rs` | 406 | `single_root_no_children`, `root_with_linear_children`, `root_with_branching_children` | 0 |
| `steel-registry/src/advancement/registry.rs` | 296 | — | 0 |
| `steel-registry/src/attribute.rs` | 156 | — | 0 |
| `steel-registry/src/banner_pattern.rs` | 328 | `generated_patterns_follow_vanilla_registry_order`, `direct_codecs_and_hash_match_vanilla_shape`, `direct_codecs_reject_invalid_identifiers_and_long_network_strings` | 0 |
| `steel-registry/src/biome.rs` | 401 | — | 0 |
| `steel-registry/src/block_entity_type.rs` | 127 | `overlapping_type_memberships_preserve_structural_presence` | 0 |
| `steel-registry/src/block_transformer.rs` | 176 | — | 0 |
| `steel-registry/src/blocks/behavior.rs` | 338 | `xz_offset_matches_vanilla_clamping`, `xz_offset_uses_block_position_seed_without_y`, `xyz_offset_applies_vanilla_vertical_component` | 0 |
| `steel-registry/src/blocks/block_state_ext.rs` | 600 | `map_colors_match_vanilla_palette_ids`, `map_color_follows_per_state_overwrites`, `solid_render_uses_occlusion_shape_not_collision_shape`, `light_properties_match_generated_state_offsets`, `blocks_motion_matches_vanilla_base_predicate`, `suffocating_uses_extracted_vanilla_state_predicate`, `static_redstone_conductor_uses_extracted_vanilla_state_predicate`, `vanilla_air_variants_are_air`, `block_entity_presence_uses_extracted_type_validity`, `fence_post_supports_center_attachments_from_below`, `generated_shape_offset_flags_distinguish_visual_offset_from_server_shapes`, `with_properties_of_keeps_target_values_for_non_matching_properties`, `cached_random_tick_metadata_tracks_state_dependent_predicates`, `cached_fluid_state_preserves_source_flowing_and_falling_variants`, `cached_metadata_keeps_block_and_fluid_random_ticks_distinct` | 0 |
| `steel-registry/src/blocks/block_state_nbt.rs` | 91 | `propertyful_block_state_round_trips_vanilla_nbt_shape` | 0 |
| `steel-registry/src/blocks/mod.rs` | 1808 | `tag_modification_keeps_order_and_membership_in_sync`, `test_redstone_wire_properties`, `test_redstone_wire_state_count`, `property_iterator_preserves_mixed_radix_values_when_skipping`, `test_properties_default_state`, `test_state_id_from_properties_roundtrip`, `test_state_id_from_properties_partial`, `dynamically_named_property_update_preserves_other_state_values`, `test_state_id_from_block_defaulted_properties_keeps_missing_defaults`, `test_state_id_from_properties_empty`, `test_state_id_from_properties_invalid_block`, `test_state_id_from_properties_invalid_property`, `test_state_id_from_properties_invalid_value`, `same_named_direction_properties_translate_by_value_name`, `test_state_id_from_properties_rejects_properties_on_propertyless_block`, `test_stone_no_properties`, `test_all_redstone_power_levels`, `test_all_block_state_ids_match_minecraft` | 0 |
| `steel-registry/src/blocks/properties.rs` | 1439 | — | 0 |
| `steel-registry/src/blocks/shapes.rs` | 1496 | `boolean_op_matches_vanilla_truth_table`, `join_is_not_empty_detects_intersection`, `join_is_not_empty_rejects_disjoint_and`, `join_is_not_empty_detects_only_first_remainder`, `join_unoptimized_boxes_materializes_only_second_remainder`, `shape_full_block_accepts_tiled_boxes`, `shape_full_block_rejects_partial_boxes`, `offset_shape_full_block_rejects_shifted_full_block`, `offset_shape_full_block_rejects_z_gap_after_offset`, `zero_volume_boxes_are_empty`, `large_collision_shape_matches_vanilla_bounds_rule`, `face_full_accepts_union_covering_face`, `dynamic_face_support_uses_union_at_the_block_boundary`, `face_full_rejects_union_with_gap`, `offset_face_full_rejects_shifted_top_face`, `face_full_accepts_vanilla_azalea_top_shape`, `rigid_support_accepts_border_ring_covered_by_multiple_boxes`, `rigid_support_rejects_center_panel_without_border_ring`, `rigid_support_rejects_side_border_ring_without_full_face` | 0 |
| `steel-registry/src/carver.rs` | 225 | — | 0 |
| `steel-registry/src/cat_sound_variant.rs` | 97 | — | 0 |
| `steel-registry/src/cat_variant.rs` | 128 | — | 0 |
| `steel-registry/src/chat_type.rs` | 119 | — | 0 |
| `steel-registry/src/chicken_sound_variant.rs` | 95 | — | 0 |
| `steel-registry/src/chicken_variant.rs` | 113 | — | 0 |
| `steel-registry/src/consume_effect.rs` | 711 | — | 0 |
| `steel-registry/src/context_provider.rs` | 585 | — | 0 |
| `steel-registry/src/cow_sound_variant.rs` | 80 | — | 0 |
| `steel-registry/src/cow_variant.rs` | 116 | — | 0 |
| `steel-registry/src/damage_type.rs` | 119 | — | 0 |
| `steel-registry/src/data_component_predicate/attributes.rs` | 168 | — | 0 |
| `steel-registry/src/data_component_predicate/basic.rs` | 349 | — | 0 |
| `steel-registry/src/data_component_predicate/books.rs` | 239 | — | 0 |
| `steel-registry/src/data_component_predicate/collections.rs` | 211 | — | 0 |
| `steel-registry/src/data_component_predicate/core.rs` | 607 | — | 0 |
| `steel-registry/src/data_component_predicate/fireworks.rs` | 218 | — | 0 |
| `steel-registry/src/data_component_predicate/mod.rs` | 61 | — | 0 |
| `steel-registry/src/data_component_predicate/registry_predicates.rs` | 147 | — | 0 |
| `steel-registry/src/data_component_predicate/tests.rs` | 231 | `vanilla_predicate_types_follow_registry_order`, `every_builtin_predicate_payload_round_trips_persistence_and_network`, `adventure_and_lock_components_round_trip_both_codecs`, `exact_predicates_reject_component_values_that_cannot_persist`, `boolean_predicate_fields_hash_as_codec_booleans` | 0 |
| `steel-registry/src/data_component_predicate/vanilla.rs` | 96 | — | 0 |
| `steel-registry/src/data_components/component_data.rs` | 324 | `typed_values_downcast_by_deterministic_key`, `equality_requires_the_same_concrete_type` | 0 |
| `steel-registry/src/data_components/components/armor_trim.rs` | 199 | `registry_references_round_trip_both_codecs_and_hash_the_record`, `inline_material_and_pattern_round_trip_both_codecs`, `extracted_item_prototypes_do_not_define_a_default_trim` | 0 |
| `steel-registry/src/data_components/components/attribute_modifiers.rs` | 525 | `generated_diamond_sword_has_main_hand_attack_modifiers`, `generated_carved_pumpkin_has_hidden_head_modifier`, `unknown_attribute_modifier_slot_group_id_falls_back_to_any`, `unknown_attribute_modifier_display_id_falls_back_to_default`, `attribute_modifier_nbt_coerces_amount_and_rejects_invalid_optional_fields` | 0 |
| `steel-registry/src/data_components/components/banner_patterns.rs` | 282 | `layer_references_round_trip_both_codecs_and_hash_the_list`, `inline_patterns_round_trip_both_holder_codecs`, `empty_layers_use_vanilla_empty_list_shape`, `extracted_banners_and_shield_keep_empty_layers` | 0 |
| `steel-registry/src/data_components/components/bees.rs` | 258 | `bee_occupants_round_trip_both_codecs_and_hash_as_a_list`, `extracted_beehives_start_with_no_occupants` | 0 |
| `steel-registry/src/data_components/components/block_state.rs` | 190 | `property_maps_round_trip_both_codecs_and_hash_as_maps`, `empty_properties_use_an_empty_compound`, `extracted_block_items_keep_placement_properties` | 0 |
| `steel-registry/src/data_components/components/blocks_attacks.rs` | 722 | `shield_component_round_trips_both_codecs`, `persistence_constraints_reject_unsavable_values` | 0 |
| `steel-registry/src/data_components/components/books.rs` | 686 | `writable_pages_use_full_filterable_persistence_and_bounded_network`, `written_book_round_trips_text_pages_and_validates_generation`, `extracted_writable_book_starts_with_empty_pages` | 0 |
| `steel-registry/src/data_components/components/brewing_fuel.rs` | 126 | `round_trips_both_codecs` | 0 |
| `steel-registry/src/data_components/components/combat.rs` | 486 | `combat_components_coerce_numbers_but_reject_malformed_present_fields` | 0 |
| `steel-registry/src/data_components/components/compostable.rs` | 107 | `round_trips_both_codecs` | 0 |
| `steel-registry/src/data_components/components/consumable.rs` | 658 | `extracted_consumables_keep_typed_effects_and_round_trip`, `extracted_totem_death_protection_round_trips`, `network_animation_ids_use_vanilla_zero_fallback`, `persistent_float_ranges_match_vanilla_float_compare_bounds` | 0 |
| `steel-registry/src/data_components/components/container_loot.rs` | 186 | `loot_table_resource_keys_and_optional_seed_round_trip`, `omitted_seed_defaults_to_zero_without_registry_membership_validation` | 0 |
| `steel-registry/src/data_components/components/cooking_fuel.rs` | 126 | `round_trips_both_codecs` | 0 |
| `steel-registry/src/data_components/components/custom_data.rs` | 235 | `persistent_codec_accepts_compounds_and_flattened_snbt`, `network_codecs_differ_only_on_the_flattened_alternative`, `network_decode_enforces_vanilla_nbt_heap_quota`, `persistent_hash_uses_the_compound_codec_shape` | 0 |
| `steel-registry/src/data_components/components/custom_model_data.rs` | 390 | `persistent_codec_omits_default_lists`, `persistent_codec_accepts_numeric_collections_and_rgb_vectors`, `network_codec_round_trips_all_lists`, `network_codec_rejects_negative_counts_and_long_strings`, `equality_matches_java_float_rules_and_getters_are_safe`, `persistent_hash_uses_the_record_codec_shape` | 0 |
| `steel-registry/src/data_components/components/debug_stick_state.rs` | 231 | `block_property_map_validates_properties_and_uses_codec_derived_network`, `extracted_debug_stick_has_an_empty_state` | 0 |
| `steel-registry/src/data_components/components/enchantable.rs` | 146 | `persistent_codec_requires_a_positive_value`, `network_codec_uses_a_validated_varint`, `persistent_hash_uses_the_record_codec_shape` | 0 |
| `steel-registry/src/data_components/components/enchantments.rs` | 232 | `enchantment_nbt_requires_known_keys_and_vanilla_levels` | 0 |
| `steel-registry/src/data_components/components/equippable.rs` | 415 | `extracted_equippable_fields_gate_swapping_and_entity_types`, `equippable_network_round_trips_tag_and_direct_holder_sets`, `equippable_hash_includes_vanilla_codec_fields`, `equippable_nbt_defaults_only_missing_fields` | 0 |
| `steel-registry/src/data_components/components/fireworks.rs` | 609 | `explosion_codecs_match_shape_and_optional_fields`, `unknown_stream_shape_ids_fall_back_to_small_ball`, `fireworks_use_unsigned_byte_persistence_and_bounded_lists`, `extracted_firework_rocket_has_one_unit_of_flight` | 0 |
| `steel-registry/src/data_components/components/food.rs` | 224 | `food_codecs_use_required_values_and_optional_false`, `negative_nutrition_is_rejected_for_persistable_values`, `equality_uses_java_record_float_semantics`, `extracted_food_prototypes_keep_vanilla_values` | 0 |
| `steel-registry/src/data_components/components/instrument.rs` | 181 | `registry_reference_round_trips_both_codecs`, `valid_inline_instrument_round_trips_and_hashes_its_record`, `network_rejects_inline_values_that_cannot_persist`, `extracted_goat_horn_uses_ponder_instrument` | 0 |
| `steel-registry/src/data_components/components/item_colors.rs` | 300 | `dyed_color_accepts_ints_and_rgb_vectors`, `raw_int_network_codecs_round_trip`, `map_id_uses_varint_network_and_int_persistence`, `persistent_hashes_use_int_codec_shape` | 0 |
| `steel-registry/src/data_components/components/item_lore.rs` | 221 | `styled_lines_apply_lore_defaults_without_overriding_explicit_style`, `network_codec_round_trips_text_components`, `persistent_codec_round_trips_mixed_component_shapes`, `persistent_codec_collapses_plain_lines_to_strings` | 0 |
| `steel-registry/src/data_components/components/jukebox_playable.rs` | 135 | `registry_reference_round_trips_both_codecs`, `direct_holder_round_trips_stream_and_is_not_persistent` | 0 |
| `steel-registry/src/data_components/components/kinetic_weapon.rs` | 574 | `kinetic_weapon_round_trips_both_codecs_and_hashes_record_shape`, `negative_persistent_durations_are_rejected`, `equality_uses_java_record_float_semantics`, `extracted_netherite_spear_keeps_kinetic_thresholds_and_sounds` | 0 |
| `steel-registry/src/data_components/components/lodestone_tracker.rs` | 251 | `lodestone_target_round_trips_and_hashes_through_global_pos_codec`, `absent_fields_default_to_no_target_and_tracked` | 0 |
| `steel-registry/src/data_components/components/map_decorations.rs` | 343 | `decoration_maps_round_trip_codec_derived_network_and_hash`, `extracted_filled_map_has_empty_decorations`, `entry_equality_matches_java_record_float_semantics` | 0 |
| `steel-registry/src/data_components/components/map_post_processing.rs` | 84 | `network_ids_match_vanilla`, `out_of_bounds_network_ids_fall_back_to_lock` | 0 |
| `steel-registry/src/data_components/components/mob_visibility.rs` | 144 | `round_trips_both_codecs`, `persistent_codec_rejects_out_of_range_visibility` | 0 |
| `steel-registry/src/data_components/components/mod.rs` | 106 | — | 0 |
| `steel-registry/src/data_components/components/ominous_bottle_amplifier.rs` | 107 | `persistent_codec_enforces_vanilla_range`, `network_codec_accepts_any_varint_like_vanilla_constructor` | 0 |
| `steel-registry/src/data_components/components/painting_variant.rs` | 144 | `registry_reference_round_trips_both_codecs`, `direct_stream_holder_round_trips_but_is_not_persistent` | 0 |
| `steel-registry/src/data_components/components/pot_decorations.rs` | 222 | `named_sides_round_trip_both_codecs_and_hash`, `extracted_decorated_pot_defaults_to_brick_on_every_side` | 0 |
| `steel-registry/src/data_components/components/potion_contents.rs` | 586 | `full_and_alternative_potion_codecs_round_trip`, `all_effects_combines_base_potion_then_custom_effects`, `all_effects_is_empty_without_a_base_potion_or_custom_effects`, `has_effects_checks_base_potion_and_custom_effects_independently`, `get_color_prefers_custom_color_over_effect_blend`, `get_color_falls_back_to_default_without_visible_effects`, `get_color_blends_visible_effect_colors_by_amplifier_weight`, `extracted_potion_items_have_empty_contents` | 0 |
| `steel-registry/src/data_components/components/provides_pottery_pattern.rs` | 136 | `round_trips_both_codecs_and_hashes_by_key`, `extracted_sherd_item_provides_its_pattern` | 0 |
| `steel-registry/src/data_components/components/provides_trim_material.rs` | 204 | `registry_reference_round_trips_both_codecs`, `inline_material_round_trips_and_hashes_its_flattened_record`, `extracted_item_prototypes_reference_every_vanilla_trim_material` | 0 |
| `steel-registry/src/data_components/components/rarity.rs` | 137 | `network_ids_match_vanilla_and_fall_back_to_common` | 0 |
| `steel-registry/src/data_components/components/recipes.rs` | 188 | `recipe_keys_round_trip_persistent_and_derived_network_codecs`, `empty_and_abbreviated_recipe_key_lists_match_identifier_codec_rules`, `recipe_key_codec_does_not_require_a_registered_recipe`, `extracted_knowledge_book_keeps_its_empty_recipe_list` | 0 |
| `steel-registry/src/data_components/components/recursive_items.rs` | 1209 | `recursive_item_components_round_trip_both_codecs`, `sparse_container_persistence_and_dense_network_preserve_vanilla_shapes`, `recursive_collection_limits_are_enforced`, `bundle_weight_matches_nested_bundle_and_beehive_rules`, `bundle_weight_rejects_commons_fraction_denominator_overflow`, `extracted_item_prototypes_use_recursive_component_values`, `bundle_contents_enforces_the_vanilla_weight_limit`, `bundle_contents_merges_matching_stacks`, `bundle_contents_rejects_shulker_boxes`, `nested_bundles_cost_themselves_plus_one_sixteenth`, `remove_selected_pops_the_selected_stack_and_clears_the_selection`, `selected_index_is_transient_ui_state` | 0 |
| `steel-registry/src/data_components/components/registry_holder_sets.rs` | 294 | `damage_resistant_uses_the_vanilla_record_and_holder_set_codecs`, `repairable_accepts_tagged_and_direct_repair_items`, `provides_banner_patterns_uses_fixed_registry_holder_sets`, `extracted_item_prototypes_include_all_holder_set_components` | 0 |
| `steel-registry/src/data_components/components/rgb_color.rs` | 35 | — | 0 |
| `steel-registry/src/data_components/components/sign_text.rs` | 272 | `round_trips_both_codecs_without_filtered_messages`, `filtered_messages_equal_to_messages_are_not_persisted`, `round_trips_with_differing_filtered_messages`, `persistent_codec_defaults_color_and_glow_when_absent` | 0 |
| `steel-registry/src/data_components/components/suspicious_stew.rs` | 275 | `stew_effects_round_trip_and_malformed_duration_uses_lenient_default`, `extracted_suspicious_stew_has_an_empty_effect_list` | 0 |
| `steel-registry/src/data_components/components/swing_animation.rs` | 226 | `empty_compound_uses_vanilla_default`, `persistent_duration_must_be_positive`, `network_type_ids_fall_back_to_none` | 0 |
| `steel-registry/src/data_components/components/tool.rs` | 489 | `tool_network_round_trips_tag_and_direct_holder_sets`, `tool_nbt_uses_compact_holder_sets_and_numeric_coercion`, `malformed_present_tool_fields_fail_the_codec`, `tool_hash_matches_its_persistent_codec_shape` | 0 |
| `steel-registry/src/data_components/components/tooltip_display.rs` | 275 | `hidden_components_preserve_first_insertion_order`, `codecs_round_trip_registered_component_types` | 0 |
| `steel-registry/src/data_components/components/typed_entity_data.rs` | 283 | `entity_data_separates_type_from_payload_on_the_network`, `block_entity_data_uses_the_block_entity_registry`, `extracted_spawn_eggs_use_typed_empty_entity_data` | 0 |
| `steel-registry/src/data_components/components/use_cooldown.rs` | 161 | `nbt_accepts_positive_seconds_and_optional_group`, `nbt_rejects_non_positive_seconds`, `nbt_rejects_invalid_cooldown_group` | 0 |
| `steel-registry/src/data_components/components/use_effects.rs` | 206 | `empty_compound_uses_vanilla_defaults`, `persistent_codec_rejects_out_of_range_speed`, `equality_uses_java_record_float_semantics`, `network_booleans_treat_any_nonzero_byte_as_true` | 0 |
| `steel-registry/src/data_components/components/villager_food.rs` | 138 | `persistent_codec_round_trips_positive_nutrition`, `persistent_codec_rejects_non_positive_nutrition`, `persistent_codec_accepts_any_integral_nbt_tag_type`, `network_codec_round_trips_varint` | 0 |
| `steel-registry/src/data_components/mod.rs` | 61 | — | 0 |
| `steel-registry/src/data_components/registry/codecs.rs` | 273 | — | 0 |
| `steel-registry/src/data_components/registry/component_map.rs` | 169 | — | 0 |
| `steel-registry/src/data_components/registry/mod.rs` | 349 | — | 0 |
| `steel-registry/src/data_components/registry/patch.rs` | 172 | — | 0 |
| `steel-registry/src/data_components/registry/patch_network.rs` | 245 | — | 0 |
| `steel-registry/src/data_components/registry/patch_persistence.rs` | 259 | — | 0 |
| `steel-registry/src/data_components/registry/tests.rs` | 343 | `duplicate_component_registration_is_rejected_without_mutation`, `persistent_hash_rejects_values_rejected_by_the_persistent_codec`, `persistent_patch_nbt_omits_transient_components`, `persistent_patch_hash_uses_each_component_codec_hash`, `persistent_patch_decode_fails_on_invalid_entries`, `text_component_persistent_codec_collapses_plain_text`, `common_defaults_and_extracted_item_overrides_match_vanilla` | 0 |
| `steel-registry/src/data_components/vanilla_components/mod.rs` | 1043 | — | 0 |
| `steel-registry/src/data_components/vanilla_components/tests.rs` | 383 | `registry_matches_extracted_vanilla_catalog`, `vanilla_transient_components_are_marked_non_persistent`, `transient_component_network_codecs_match_vanilla`, `identifier_component_codecs_use_vanilla_namespace_rules`, `persistent_scalar_codecs_coerce_numeric_tags_and_enforce_ranges`, `unit_component_persistence_requires_a_compound`, `unit_component_network_codecs_match_vanilla`, `registry_validation_uses_concrete_downcast_keys` | 0 |
| `steel-registry/src/decorated_pot_pattern.rs` | 186 | `generated_patterns_are_registered`, `direct_codecs_and_hash_match_vanilla_shape` | 0 |
| `steel-registry/src/dialog.rs` | 1085 | — | 0 |
| `steel-registry/src/dimension_type.rs` | 468 | `teleportation_scale_matches_vanilla_coordinate_ratio` | 0 |
| `steel-registry/src/dye_color.rs` | 350 | `ids_names_and_network_fallback_match_vanilla`, `vanilla_color_metadata_matches_representative_values`, `persistent_codec_and_hash_use_serialized_names` | 0 |
| `steel-registry/src/enchantment/effect.rs` | 691 | `remove_binomial_matches_vanilla_small_input_loop`, `remove_binomial_matches_vanilla_gaussian_approximation` | 0 |
| `steel-registry/src/enchantment/mod.rs` | 573 | `sharpness_is_primary_for_swords_but_only_supported_for_axes`, `cost_windows_follow_the_vanilla_formula`, `binding_curse_has_prevent_armor_change_effect`, `enchantment_matching_slot_uses_slot_groups`, `existing_identical_or_exclusive_enchantments_are_incompatible`, `compatibility_checks_stored_enchantments_on_enchanted_books`, `enchantment_nbt_includes_raw_effect_payloads`, `movement_requirements_use_vanilla_double_bounds_in_nbt`, `unconditional_value_effects_modify_values`, `conditional_value_effects_are_not_applied_without_context`, `looting_equipment_drops_preserves_attacker_target`, `frost_walker_damage_immunity_preserves_damage_source_requirements`, `lunge_post_piercing_attack_preserves_entity_effects`, `thorns_post_attack_preserves_random_damage_and_item_damage_effects` | 0 |
| `steel-registry/src/enchantment_provider.rs` | 305 | — | 0 |
| `steel-registry/src/entity_data/mod.rs` | 599 | `zombie_villager_default_selects_profession_from_registry`, `empty_villager_profession_registry_preserves_fallback_without_rng_draw`, `projectile_item_stack_defaults_use_extracted_item`, `empty_item_stack_defaults_remain_empty`, `entity_effect_particle_color_options_encode_payload` | 0 |
| `steel-registry/src/entity_data/serializer.rs` | 95 | — | 0 |
| `steel-registry/src/entity_data/vanilla_serializers.rs` | 495 | `test_serializer_registration_order`, `test_serializers_write_correctly`, `holder_serializer_resolves_the_registered_variant_id`, `empty_resolvable_profile_matches_vanilla_static_empty_shape` | 0 |
| `steel-registry/src/entity_type.rs` | 551 | `attachment_points_clamp_index_and_rotate_like_vanilla`, `fallback_attachment_points_match_vanilla_defaults`, `attachment_average_uses_unrotated_scaled_points`, `vanilla_track_deltas_exclusions_match_entity_type_method`, `vanilla_class_hierarchy_flags_match_representative_entities` | 0 |
| `steel-registry/src/entity_variant.rs` | 794 | `ids_names_persistent_codecs_and_hashes_match_vanilla`, `network_out_of_bounds_strategies_match_vanilla`, `variant_specific_metadata_matches_vanilla` | 0 |
| `steel-registry/src/environment_attribute.rs` | 223 | — | 0 |
| `steel-registry/src/equipment.rs` | 367 | `equipment_slot_protocol_ids_match_vanilla`, `only_saddle_slot_does_not_increase_experience` | 0 |
| `steel-registry/src/feature/data.rs` | 1159 | — | 0 |
| `steel-registry/src/feature/mod.rs` | 261 | — | 0 |
| `steel-registry/src/fluid/mod.rs` | 342 | `from_block_level_uses_source_variant_for_level_zero`, `from_block_level_uses_flowing_variant_for_non_source_levels`, `from_block_level_clamps_all_falling_liquid_levels_to_full_amount`, `source_fluid_type_is_source_even_when_falling` | 0 |
| `steel-registry/src/frog_variant.rs` | 78 | — | 0 |
| `steel-registry/src/function.rs` | 180 | `a_function_resolves_by_full_identifier`, `a_bare_path_resolves_in_the_default_namespace`, `an_unknown_function_is_not_found`, `registering_the_same_id_twice_reports_the_replacement`, `freezing_rejects_later_registrations`, `ids_are_sorted` | 0 |
| `steel-registry/src/game_events.rs` | 52 | — | 0 |
| `steel-registry/src/game_profile.rs` | 500 | `properties_use_authlib_multimap_order_and_equality` | 0 |
| `steel-registry/src/game_rules.rs` | 569 | `typed_values_use_defaults_and_update_without_callsite_downcasts`, `typed_and_erased_sets_enforce_rule_bounds_and_value_type`, `erased_rule_codecs_round_trip_serialized_values`, `same_key_does_not_allow_forging_different_rule_metadata` | 0 |
| `steel-registry/src/gametest.rs` | 555 | — | 0 |
| `steel-registry/src/instrument.rs` | 304 | `nbt_uses_sound_event_registry_key` | 0 |
| `steel-registry/src/item_predicate.rs` | 1085 | `double_bounds_use_java_ordering`, `nbt_predicate_persistence_round_trips_heterogeneous_lists`, `block_predicates_use_typed_vanilla_property_order` | 0 |
| `steel-registry/src/item_stack.rs` | 2485 | `consume_shrinks_only_without_infinite_materials`, `consume_and_return_splits_before_shrinking`, `stored_book_enchantments_are_not_active_item_enchantments`, `written_book_raw_title_is_its_custom_and_hover_name`, `explicit_custom_name_takes_precedence_over_written_book_title`, `written_book_title_uses_java_blank_rules`, `item_damage_uses_generated_unbreaking_tool_requirements`, `match_tool_supports_generated_direct_item_sets`, `persistent_item_count_uses_vanilla_integer_codec`, `malformed_present_component_patch_rejects_the_item_stack`, `shrunk_to_empty_stack_exposes_no_components`, `component_patches_stay_sanitized_against_the_item_prototype`, `strict_validation_ignores_components_of_empty_stacks`, `default_count_is_always_present_in_persistent_encoding`, `untrusted_stack_rejects_direct_jukebox_holders`, `untrusted_stack_accepts_persistable_registry_holders`, `untrusted_stack_uses_persistent_count_range`, `untrusted_stack_validates_component_persistent_constraints`, `save_omits_invalid_component_value_but_keeps_present_patch_field`, `toggle_tooltips_updates_the_typed_display_component`, `set_custom_data_recursively_merges_and_removes_empty_values` | 0 |
| `steel-registry/src/item_stack_template.rs` | 627 | `item_only_alternative_decodes_and_primary_codec_omits_defaults`, `complete_templates_round_trip_both_codecs`, `hover_events_embed_the_typed_component_patch_codec_output`, `template_invariants_reject_empty_items_and_unpersistable_counts`, `stream_codec_accepts_nonzero_counts_outside_persistent_range`, `containing_component_hash_rejects_stream_only_nested_patch`, `item_only_air_alternative_returns_codec_failure`, `create_rejects_invalid_effective_stack_constraints`, `create_rejects_oversized_recursive_contents`, `create_rejects_excessive_bundle_weight_arithmetic` | 0 |
| `steel-registry/src/items/item.rs` | 63 | — | 0 |
| `steel-registry/src/items/mod.rs` | 284 | `new_item_uses_its_key_as_the_default_model`, `extracted_block_item_associations_match_vanilla`, `block_item_capability_includes_extracted_vanilla_subclasses` | 0 |
| `steel-registry/src/jukebox_song.rs` | 164 | — | 0 |
| `steel-registry/src/lib.rs` | 613 | — | 0 |
| `steel-registry/src/loot_table/conditions.rs` | 583 | — | 0 |
| `steel-registry/src/loot_table/context.rs` | 624 | — | 0 |
| `steel-registry/src/loot_table/entries.rs` | 622 | `a_stored_loot_seed_wins_over_every_other_source`, `a_random_sequence_derives_a_stable_world_seeded_stream`, `a_plain_table_defers_to_the_level_rng`, `distinct_sequences_from_one_world_seed_differ` | 0 |
| `steel-registry/src/loot_table/functions.rs` | 578 | — | 0 |
| `steel-registry/src/loot_table/mod.rs` | 23 | — | 0 |
| `steel-registry/src/loot_table/registry.rs` | 63 | — | 0 |
| `steel-registry/src/loot_table/tests.rs` | 584 | `test_oak_log_loot`, `set_instrument_selects_from_the_configured_holder_set`, `test_diamond_ore_loot_no_silk_touch`, `test_grass_block_loot_no_silk_touch`, `test_stone_loot_no_silk_touch`, `test_pig_loot_drops_raw_porkchop_when_not_on_fire`, `shearing_sheep_table_parses_the_flat_type_specific_sheep_key`, `chicken_lay_branches_carry_the_chicken_variant_component`, `chicken_lay_drops_the_egg_matching_the_chicken_variant`, `sheared_predicate_rejects_non_sheep_entities`, `test_pig_loot_smelt_condition_uses_entity_fire_flag`, `pig_loot_smelts_when_direct_attacker_holds_fire_aspect`, `pig_loot_stays_raw_when_direct_attacker_holds_plain_sword`, `pig_loot_count_uses_attacker_looting_level`, `pig_loot_ignores_looting_outside_its_enchantment_slots`, `enchanted_count_increase_limit_caps_the_stack_not_the_bonus`, `random_chance_with_enchanted_bonus_reads_attacker_not_tool`, `equipment_enchantment_level_ignores_empty_stacks`, `enchanted_count_increase_rounds_like_java_math_round`, `test_uniform_get_int_reaches_inclusive_max`, `test_explosion_decay_function`, `ominous_bottle_amplifier_function_clamps_to_persistent_range`, `test_survives_explosion_condition` | 0 |
| `steel-registry/src/macros.rs` | 215 | — | 0 |
| `steel-registry/src/map_color.rs` | 240 | `palette_matches_vanilla_ids`, `render_byte_packs_id_and_brightness`, `brightness_round_trips_and_scales`, `render_rgb_scales_each_channel`, `clear_renders_as_transparent` | 0 |
| `steel-registry/src/map_decoration_type.rs` | 98 | `extracted_types_follow_vanilla_registry_order_and_fields` | 0 |
| `steel-registry/src/menu_type.rs` | 48 | — | 0 |
| `steel-registry/src/mob_effect/instance.rs` | 535 | `map_duration_leaves_infinite_and_zero_sentinels_untouched`, `with_scaled_duration_leaves_infinite_and_zero_durations_untouched`, `with_scaled_duration_floors_and_clamps_finite_durations`, `effect_instances_round_trip_recursive_details_and_clamp_amplifier`, `effect_instance_equality_ignores_hidden_effects_like_vanilla` | 0 |
| `steel-registry/src/mob_effect/mod.rs` | 203 | `generated_effect_particles_match_vanilla_factories` | 0 |
| `steel-registry/src/painting_variant.rs` | 167 | — | 0 |
| `steel-registry/src/particle_type.rs` | 1264 | `every_vanilla_particle_payload_family_round_trips`, `particle_write_rejects_noncanonical_same_key_codec`, `particle_type_registry_rejects_duplicate_keys`, `block_particle_network_codec_rejects_invalid_state_ids`, `particle_payloads_round_trip_through_nbt`, `dust_scale_matches_vanilla_constructor_clamping` | 0 |
| `steel-registry/src/pig_sound_variant.rs` | 95 | — | 0 |
| `steel-registry/src/pig_variant.rs` | 153 | `select_spawn_variant_uses_highest_matching_biome_priority` | 0 |
| `steel-registry/src/poi/mod.rs` | 208 | `matching_states_respects_property_filter`, `matchers_reproduce_extracted_vanilla_states` | 0 |
| `steel-registry/src/position_source.rs` | 397 | `position_source_write_rejects_noncanonical_same_key_codec`, `position_source_type_registry_rejects_duplicate_keys` | 0 |
| `steel-registry/src/potion.rs` | 145 | `extracted_potions_follow_vanilla_ids_and_effects`, `vanilla_potion_tags_are_generated_from_builtin_data`, `tag_membership_and_iteration_follow_duplicate_key_replacement` | 0 |
| `steel-registry/src/recipe/crafting/matching.rs` | 551 | `asymmetric_shaped_recipe_matches_both_orientations`, `shaped_recipe_requires_empty_pattern_slots`, `firework_rocket_requires_one_to_three_fuel_items` | 0 |
| `steel-registry/src/recipe/crafting/mod.rs` | 438 | — | 0 |
| `steel-registry/src/recipe/display.rs` | 173 | `every_vanilla_recipe_produces_a_display`, `shaped_recipes_report_a_full_grid` | 0 |
| `steel-registry/src/recipe/ingredient.rs` | 136 | — | 0 |
| `steel-registry/src/recipe/mod.rs` | 37 | — | 0 |
| `steel-registry/src/recipe/properties.rs` | 212 | — | 0 |
| `steel-registry/src/recipe/registry.rs` | 678 | `generated_static_and_registry_lookup_share_the_same_recipe_data`, `generated_recipe_result_preserves_extracted_component_patch`, `typed_matching_finds_the_generated_iron_pickaxe_recipe`, `every_extracted_recipe_is_present_in_its_operational_type_bucket`, `plugin_type_keeps_custom_data_typed_through_matching_and_key_lookup`, `cached_check_tests_the_last_successful_recipe_before_scanning`, `shapeless_matching_backtracks_when_ingredients_overlap` | 0 |
| `steel-registry/src/recipe/single_item.rs` | 84 | — | 0 |
| `steel-registry/src/recipe/smithing.rs` | 107 | — | 0 |
| `steel-registry/src/recipe/types.rs` | 276 | — | 0 |
| `steel-registry/src/registry/holder.rs` | 185 | — | 0 |
| `steel-registry/src/registry/holder_set.rs` | 439 | `persistent_codec_matches_vanilla_holder_set_shapes`, `network_codec_round_trips_tag_direct_and_empty_sets`, `codecs_reject_unknown_registry_values`, `contains_resolves_tags_and_direct_entries`, `string_codec_rejects_malformed_identifiers` | 0 |
| `steel-registry/src/registry/mod.rs` | 1083 | `freeze_rejects_missing_biome_carver_reference`, `freeze_rejects_missing_biome_placed_feature_reference`, `vanilla_feature_registries_initialize_and_validate`, `vanilla_static_entity_data_registries_initialize_in_vanilla_order`, `vanilla_game_events_initialize_in_vanilla_order` | 0 |
| `steel-registry/src/registry/reference.rs` | 262 | `fixed_variant_references_share_vanilla_holder_codecs`, `fixed_variant_references_reject_unknown_network_ids` | 0 |
| `steel-registry/src/registry/tags.rs` | 56 | — | 0 |
| `steel-registry/src/resolvable_profile.rs` | 591 | `full_profile_round_trips_both_codecs_and_hashes_primary_shape`, `player_name_alternative_normalizes_to_dynamic_profile`, `profile_limits_prevent_values_that_cannot_be_reencoded` | 0 |
| `steel-registry/src/shared_structs.rs` | 278 | `pick_spawn_conditioned_entry_keeps_duplicate_highest_priority_matches` | 0 |
| `steel-registry/src/slot_display.rs` | 388 | `empty_and_any_fuel_displays_write_only_their_kind`, `item_display_writes_the_registry_id`, `composite_display_writes_its_length`, `shaped_display_writes_kind_width_height_and_count` | 0 |
| `steel-registry/src/sound_event.rs` | 344 | `direct_sound_range_uses_lenient_numeric_codec`, `direct_sound_equality_matches_java_float_rules`, `sound_holder_range_matches_variable_and_fixed_ranges` | 0 |
| `steel-registry/src/stat/custom.rs` | 60 | — | 0 |
| `steel-registry/src/stat/mod.rs` | 257 | `network_encode_and_decode_stat` | 0 |
| `steel-registry/src/stat/registry.rs` | 284 | — | 0 |
| `steel-registry/src/stat/vanilla_stat_types.rs` | 95 | `registry_matches_extracted_stat_types` | 0 |
| `steel-registry/src/steel_ticket_types.rs` | 34 | `chunk_request_is_registered_after_vanilla_types` | 0 |
| `steel-registry/src/structure/mod.rs` | 398 | `vanilla_structure_tags_are_registered`, `structure_generation_steps_use_vanilla_decoration_ordinals` | 0 |
| `steel-registry/src/structure/processor/data.rs` | 162 | — | 0 |
| `steel-registry/src/structure/processor/mod.rs` | 97 | — | 0 |
| `steel-registry/src/structure/set.rs` | 103 | — | 0 |
| `steel-registry/src/structure/template_pool.rs` | 264 | — | 0 |
| `steel-registry/src/sulfur_cube_archetype.rs` | 475 | — | 0 |
| `steel-registry/src/ticket_type.rs` | 135 | `rejects_duplicate_keys` | 0 |
| `steel-registry/src/timeline.rs` | 220 | `repeating_timeline_reports_current_tick_and_completed_periods`, `non_repeating_timeline_uses_absolute_ticks_and_zero_periods` | 0 |
| `steel-registry/src/trial_spawner.rs` | 216 | — | 0 |
| `steel-registry/src/trim_material.rs` | 255 | `generated_materials_follow_vanilla_registry_order_and_palette_ids`, `generated_definition_uses_the_flattened_persistent_shape` | 0 |
| `steel-registry/src/trim_pattern.rs` | 296 | `generated_patterns_follow_vanilla_registry_order`, `direct_codecs_always_encode_decal_and_default_it_when_absent`, `direct_persistent_codec_rejects_invalid_asset_identifiers` | 0 |
| `steel-registry/src/vanilla_criteria_triggers.rs` | 276 | `vanilla_trigger_list_matches_the_game` | 0 |
| `steel-registry/src/vanilla_ticket_types.rs` | 109 | `registry_matches_vanilla_order_timeouts_and_behavior` | 0 |
| `steel-registry/src/villager_profession.rs` | 45 | — | 0 |
| `steel-registry/src/villager_trade.rs` | 386 | — | 0 |
| `steel-registry/src/villager_type.rs` | 47 | — | 0 |
| `steel-registry/src/wolf_sound_variant.rs` | 91 | — | 0 |
| `steel-registry/src/wolf_variant.rs` | 101 | — | 0 |
| `steel-registry/src/world_clock.rs` | 57 | — | 0 |
| `steel-registry/src/world_preset.rs` | 220 | — | 0 |
| `steel-registry/src/zombie_nautilus_variant.rs` | 82 | — | 0 |
| `steel-worldgen/build/build.rs` | 67 | — | 0 |
| `steel-worldgen/build/density/functions.rs` | 1606 | — | 0 |
| `steel-worldgen/build/density/mod.rs` | 14 | — | 0 |
| `steel-worldgen/build/density/surface_rules.rs` | 630 | — | 0 |
| `steel-worldgen/build/density/transpiler/analyze.rs` | 303 | — | 0 |
| `steel-worldgen/build/density/transpiler/bounds.rs` | 258 | — | 0 |
| `steel-worldgen/build/density/transpiler/codegen_expr.rs` | 1415 | — | 0 |
| `steel-worldgen/build/density/transpiler/codegen_functions.rs` | 461 | — | 0 |
| `steel-worldgen/build/density/transpiler/codegen_structs.rs` | 573 | — | 0 |
| `steel-worldgen/build/density/transpiler/context.rs` | 104 | — | 0 |
| `steel-worldgen/build/density/transpiler/fingerprint.rs` | 216 | — | 0 |
| `steel-worldgen/build/density/transpiler/graph.rs` | 420 | — | 0 |
| `steel-worldgen/build/density/transpiler/material_ore.rs` | 80 | — | 0 |
| `steel-worldgen/build/density/transpiler/mod.rs` | 118 | — | 0 |
| `steel-worldgen/build/density/transpiler/naming.rs` | 52 | — | 0 |
| `steel-worldgen/build/density/transpiler/nonpositive.rs` | 148 | — | 0 |
| `steel-worldgen/build/density/types.rs` | 837 | `test_rarity_value_mapper_tunnels`, `test_rarity_value_mapper_caves` | 0 |
| `steel-worldgen/build/multi_noise.rs` | 187 | — | 0 |
| `steel-worldgen/build/noise_parameters.rs` | 150 | — | 0 |
| `steel-worldgen/src/biomes/biome_source.rs` | 547 | `fixed_source_search_draws_like_vanilla`, `end_possible_biomes_follow_vanilla_order` | 0 |
| `steel-worldgen/src/biomes/climate_sampler.rs` | 208 | — | 0 |
| `steel-worldgen/src/biomes/mod.rs` | 37 | `obfuscate_biome_seed_matches_guava_hash_long` | 0 |
| `steel-worldgen/src/biomes/nether_climate_sampler.rs` | 109 | — | 0 |
| `steel-worldgen/src/density/mod.rs` | 121 | — | 0 |
| `steel-worldgen/src/density/spline_eval.rs` | 155 | `find_interval_before_all`, `find_interval_exact_match`, `find_interval_between`, `find_interval_after_all`, `hermite_linear`, `evaluate_spline_extrapolate_before`, `evaluate_spline_extrapolate_after`, `evaluate_spline_constant_values` | 0 |
| `steel-worldgen/src/density/traits.rs` | 346 | — | 0 |
| `steel-worldgen/src/lib.rs` | 39 | — | 0 |
| `steel-worldgen/src/noise/aquifer.rs` | 1213 | `uniform_status_shortcut_matches_sampled_path` | 0 |
| `steel-worldgen/src/noise/beardifier.rs` | 295 | — | 0 |
| `steel-worldgen/src/noise/blended_noise.rs` | 331 | `vanilla_lerp_endpoint_and_main_coordinate_rounding`, `compute_column_matches_scalar_lanes`, `simd_endpoint_shortcuts_preserve_each_lane` | 0 |
| `steel-worldgen/src/noise/corner_column_store.rs` | 135 | `last_user_removes_the_column`, `concurrent_producers_count_as_uses` | 0 |
| `steel-worldgen/src/noise/end_islands.rs` | 91 | — | 0 |
| `steel-worldgen/src/noise/improved_noise.rs` | 1205 | `corner_gradient_batches_match_scalar_at_permutation_boundaries`, `test_noise_with_y_scale_4x_matches_scalar`, `smeared_noise_y_simd_matches_scalar`, `test_noise_y_simd_matches_scalar`, `test_zero_axis_helpers_match_full_noise`, `test_zero_axis_smeared_helpers_match_full_noise`, `test_noise_with_y_scale_simd8_matches_scalar`, `test_improved_noise_deterministic`, `scalar_noise_wraps_corner_coordinates_at_i32_max`, `test_improved_noise_range`, `test_improved_noise_spatial_variation`, `test_noise_with_derivative_produces_derivatives`, `test_noise_with_derivative_accumulates` | 0 |
| `steel-worldgen/src/noise/mod.rs` | 36 | — | 0 |
| `steel-worldgen/src/noise/noise_chunk.rs` | 812 | `air_column_rejects_rounding_overshoot`, `overworld_corner_reuse_matches`, `nether_corner_reuse_matches`, `end_corner_reuse_matches`, `overworld_air_skip_is_sound`, `nether_air_skip_is_sound`, `end_air_skip_is_sound` | 0 |
| `steel-worldgen/src/noise/normal_noise.rs` | 431 | `zero_axis_helpers_match_full_noise`, `value_y_simd_matches_scalar_f32_lanes` | 0 |
| `steel-worldgen/src/noise/ore_veinifier.rs` | 267 | `rejected_material_rules_do_not_sample_filler_gap` | 0 |
| `steel-worldgen/src/noise/perlin_noise.rs` | 576 | `test_perlin_noise_deterministic`, `test_get_value_matches_zero_y_params_path`, `test_get_value_simd_matches_scalar`, `test_perlin_noise_spatial_variation`, `test_create_from_random_different_seeds`, `test_zero_axis_helpers_match_full_noise` | 0 |
| `steel-worldgen/src/noise/perlin_simplex_noise.rs` | 123 | — | 0 |
| `steel-worldgen/src/noise/preliminary_surface_store.rs` | 156 | `packed_levels_round_trip_through_colliding_slots`, `levels_keep_sign_and_reject_out_of_range` | 0 |
| `steel-worldgen/src/noise/simplex_noise.rs` | 231 | `test_simplex_noise_deterministic`, `test_simplex_2d_spatial_variation`, `test_end_islands_noise_init` | 0 |
| `steel-worldgen/src/state_resolver.rs` | 69 | — | 0 |
| `steel-worldgen/src/structure/box_octree.rs` | 279 | `distant_boxes_do_not_collide`, `nearby_boxes_collide` | 0 |
| `steel-worldgen/src/structure/desert_pyramid.rs` | 142 | `desert_pyramid_piece_uses_full_procedural_payload` | 0 |
| `steel-worldgen/src/structure/end_city.rs` | 625 | `end_city_piece_uses_template_payload_and_overwrite_processor` | 0 |
| `steel-worldgen/src/structure/fortress.rs` | 769 | `fortress_constructor_rng_state_is_captured_in_piece_payloads` | 0 |
| `steel-worldgen/src/structure/generation.rs` | 508 | — | 0 |
| `steel-worldgen/src/structure/generator.rs` | 1426 | `vanilla_assets_cover_vanilla_structure_sets`, `ring_cache_keys_entries_by_placement_inputs`, `biome_source_cache_key_includes_seed`, `constructor_rejects_invalid_random_spread_spacing`, `exclusion_zone_checks_other_set_interactions`, `circular_exclusion_zones_fail_loudly`, `random_spread_candidates_follow_vanilla_shell_order`, `random_spread_candidates_use_locate_offset`, `locate_candidates_retain_only_structures_for_their_placement_scan`, `ring_candidates_are_ordered_by_vanilla_distance_probe` | 0 |
| `steel-worldgen/src/structure/igloo.rs` | 192 | `igloo_piece_uses_template_payload_with_height_adjustment` | 0 |
| `steel-worldgen/src/structure/jigsaw.rs` | 1523 | `free_space_large_matches_small_scan_after_octree_transition`, `start_jigsaw_name_can_anchor_feature_pool_element` | 0 |
| `steel-worldgen/src/structure/jungle_temple.rs` | 131 | `jungle_temple_piece_uses_full_procedural_payload` | 0 |
| `steel-worldgen/src/structure/mansion/grid.rs` | 451 | — | 0 |
| `steel-worldgen/src/structure/mansion/mod.rs` | 78 | — | 0 |
| `steel-worldgen/src/structure/mansion/placement.rs` | 405 | — | 0 |
| `steel-worldgen/src/structure/mansion/roof.rs` | 305 | — | 0 |
| `steel-worldgen/src/structure/mansion/rooms.rs` | 268 | — | 0 |
| `steel-worldgen/src/structure/mansion/template.rs` | 202 | — | 0 |
| `steel-worldgen/src/structure/mansion/tests.rs` | 57 | `recursive_corridor_branches_toward_selected_next_direction`, `mansion_piece_uses_template_payload_with_marker_handling` | 0 |
| `steel-worldgen/src/structure/mansion/walls.rs` | 96 | — | 0 |
| `steel-worldgen/src/structure/mineshaft.rs` | 863 | `mineshaft_matches_vanilla_seed_13579_chunk_0_0`, `mineshaft_generation_captures_piece_payload_state` | 0 |
| `steel-worldgen/src/structure/mod.rs` | 63 | — | 0 |
| `steel-worldgen/src/structure/nether_fossil.rs` | 234 | `nether_fossil_piece_uses_full_template_payload` | 0 |
| `steel-worldgen/src/structure/ocean_monument.rs` | 1007 | `monument_building_uses_full_procedural_payload`, `generated_child_order_captures_vanilla_fixed_children` | 0 |
| `steel-worldgen/src/structure/ocean_ruin.rs` | 393 | `ocean_ruin_piece_uses_template_payload_with_height_adjustment_and_processors` | 0 |
| `steel-worldgen/src/structure/piece.rs` | 289 | — | 0 |
| `steel-worldgen/src/structure/placement.rs` | 718 | `test_spread_type_linear`, `test_spread_type_triangular`, `test_village_placement_seed_0`, `test_negative_chunk_coords`, `test_frequency_reduction_skips_at_1`, `test_load_vanilla_structure_sets`, `test_concentric_rings_with_positions`, `test_generate_ring_positions_strongholds`, `test_generate_ring_positions_zero_count`, `test_java_round` | 0 |
| `steel-worldgen/src/structure/ruined_portal.rs` | 538 | `ruined_portal_piece_uses_template_payload_with_processors_and_postprocess` | 0 |
| `steel-worldgen/src/structure/shipwreck.rs` | 187 | `shipwreck_piece_uses_template_payload_with_height_adjustment` | 0 |
| `steel-worldgen/src/structure/single_piece.rs` | 70 | `buried_treasure_piece_uses_procedural_payload` | 0 |
| `steel-worldgen/src/structure/start.rs` | 249 | — | 0 |
| `steel-worldgen/src/structure/stronghold.rs` | 915 | `orient_box_swaps_offsets_for_east_west_like_vanilla`, `constructor_rng_state_is_captured_in_piece_payloads`, `library_and_filler_payloads_capture_non_random_state` | 0 |
| `steel-worldgen/src/structure/swamp_hut.rs` | 112 | `swamp_hut_piece_uses_full_procedural_payload` | 0 |
| `steel-worldgen/src/structure/utils.rs` | 42 | — | 0 |
| `steel-worldgen/src/surface.rs` | 249 | — | 0 |
| `steel-worldgen/src/surface_partial.rs` | 146 | `undecidable_branches_force_evaluation_only_when_they_could_place` | 0 |
| `steel-worldgen/src/utils.rs` | 486 | — | 0 |
| `steel-utils/build/build.rs` | 513 | — | 0 |
| `steel-utils/build/entity_events.rs` | 43 | — | 0 |
| `steel-utils/build/translations.rs` | 98 | — | 0 |
| `steel-utils/src/axis.rs` | 41 | — | 0 |
| `steel-utils/src/block_util.rs` | 193 | `max_rectangle_location_matches_vanilla_histogram_scan`, `largest_rectangle_around_scans_both_axes_from_center` | 0 |
| `steel-utils/src/climate/mod.rs` | 83 | `test_quantize_coord`, `test_unquantize_coord`, `test_quantize_roundtrip` | 0 |
| `steel-utils/src/climate/parameter_list.rs` | 543 | `test_parameter_list_find_value` | 0 |
| `steel-utils/src/climate/types.rs` | 293 | `test_target_point_from_floats`, `test_parameter_distance`, `test_parameter_point_fitness` | 0 |
| `steel-utils/src/codec/bit_set.rs` | 101 | `write_trims_empty_bit_set_to_zero_bytes`, `write_trims_to_last_non_zero_byte_little_endian`, `round_trips_through_read` | 0 |
| `steel-utils/src/codec/glam.rs` | 73 | — | 0 |
| `steel-utils/src/codec/lp_vec3.rs` | 138 | `zero_vector_is_single_zero_byte`, `packed_vector_matches_vanilla_quantization`, `continuation_scale_round_trips_large_vector` | 0 |
| `steel-utils/src/codec/mod.rs` | 15 | — | 0 |
| `steel-utils/src/codec/or.rs` | 103 | `test_or_write_left`, `test_or_write_right`, `test_or_helpers` | 0 |
| `steel-utils/src/codec/variable_integer.rs` | 349 | `test_varint_read_write_negative`, `test_varlong_read_write` | 0 |
| `steel-utils/src/color.rs` | 181 | `argb_average_matches_vanilla_per_channel_mean`, `rgb_preserves_ignored_upper_byte`, `argb_exposes_all_channels` | 0 |
| `steel-utils/src/direction.rs` | 460 | `horizontal_dirs_matches_vanilla_plane_horizontal_order`, `all_dirs_matches_vanilla_values_order`, `three_d_data_value_round_trips_vanilla_ids`, `from_yaw_matches_vanilla_from_y_rot_boundaries`, `from_3d_data_value_matches_java_abs_mod_wrap`, `from_yaw_preserves_vanilla_double_precision_for_large_angles`, `positive_for_axis_matches_vanilla_axis_direction_positive` | 0 |
| `steel-utils/src/downcast.rs` | 193 | `downcasts_matching_shared_reference`, `downcasts_matching_mutable_reference` | 0 |
| `steel-utils/src/front_vec.rs` | 162 | `front_space_reservation_and_write_safe`, `set_in_front_panics_if_no_space` | 0 |
| `steel-utils/src/geometry.rs` | 781 | `constructors_normalize_endpoints_like_vanilla`, `inflate_and_deflate_normalize_inverted_bounds`, `of_size_builds_vanilla_centered_box`, `block_local_aabb_translates_to_world_space`, `contains_uses_vanilla_exclusive_max_edge`, `world_aabb_distance_to_sqr_uses_nearest_surface_point`, `distance_to_sqr_aabb_uses_nearest_gap_between_boxes`, `closest_point_to_clamps_to_box_bounds`, `expand_towards_covers_start_and_end` | 0 |
| `steel-utils/src/hash.rs` | 627 | `test_int_hash`, `test_string_hash`, `test_bool_hash`, `test_empty_map_hash`, `unit_codec_hashes_as_an_empty_map`, `test_empty_list_hash`, `test_byte_array_hash`, `test_deterministic`, `nbt_compound_hash_is_independent_of_entry_order`, `nbt_list_hash_unwraps_vanillas_heterogeneous_list_marker`, `test_text_component_steel`, `test_text_component_simple_styled`, `test_text_component_rainbow` | 0 |
| `steel-utils/src/java.rs` | 188 | `matches_java_whitespace_exclusions`, `space_char_includes_unicode_space_separators`, `blank_combines_java_whitespace_and_space_char`, `float_string_matches_java_integral_values` | 0 |
| `steel-utils/src/lib.rs` | 94 | — | 0 |
| `steel-utils/src/locks.rs` | 37 | — | 0 |
| `steel-utils/src/logger/mod.rs` | 159 | — | 0 |
| `steel-utils/src/mth.rs` | 23 | `ceil_log2_matches_vanilla_worked_examples` | 0 |
| `steel-utils/src/nbt/codec.rs` | 234 | `owned_numeric_tags_use_codec_coercions`, `borrowed_numeric_tags_use_codec_coercions` | 0 |
| `steel-utils/src/nbt/json.rs` | 492 | `converts_primitives_and_nested_objects`, `wraps_and_unwraps_heterogeneous_lists_like_vanilla_nbt_ops`, `float_preferred_mode_produces_float_tags_and_rejects_null` | 0 |
| `steel-utils/src/nbt/mod.rs` | 476 | `compounds_and_lists_match_partially`, `empty_partial_list_only_matches_an_empty_list`, `partial_lists_match_heterogeneous_values`, `scalar_tags_require_the_same_nbt_type`, `exact_equality_matches_vanilla_map_and_float_rules`, `normalization_keeps_the_final_duplicate_key`, `compound_merge_recurses_only_when_both_values_are_compounds`, `heap_size_matches_vanilla_nbt_accounter_formulas` | 0 |
| `steel-utils/src/nbt/path.rs` | 1459 | `parses_path_argument_without_consuming_separator`, `counts_compound_child_matches`, `counts_list_index_and_wildcard_matches`, `negative_indices_select_from_end`, `predicate_nodes_use_partial_compound_matching`, `root_predicate_matches_root_compound`, `preserves_embedded_snbt_error_kind`, `quoted_key_errors_use_brigadier_cursors`, `index_errors_use_brigadier_integer_components`, `set_creates_missing_compound_parents`, `set_updates_all_list_elements`, `set_updates_indexed_array_elements`, `set_coerces_numeric_array_values_like_vanilla`, `empty_array_wildcard_reports_vanilla_change_count_for_rejected_value`, `set_regular_list_elements_can_change_type_like_vanilla`, `predicate_parent_creates_match_in_non_compound_list`, `set_predicate_parent_creates_matching_compound`, `set_reports_missing_intermediate_path` | 0 |
| `steel-utils/src/nbt/snbt/error.rs` | 331 | — | 0 |
| `steel-utils/src/nbt/snbt/mod.rs` | 13 | — | 0 |
| `steel-utils/src/nbt/snbt/number.rs` | 554 | — | 0 |
| `steel-utils/src/nbt/snbt/parser.rs` | 594 | — | 0 |
| `steel-utils/src/nbt/snbt/tests.rs` | 469 | `canonical_writer_matches_vanilla_snbt_shapes`, `canonical_writer_uses_semantic_heterogeneous_list_values`, `canonical_writer_preserves_wrapper_shaped_compound_values`, `canonical_writer_uses_java_floating_point_decimals`, `parses_compounds_lists_and_trailing_commas`, `parses_boolean_literals_case_insensitively`, `duplicate_compound_keys_keep_last_value`, `parses_integer_widths_and_unsigned_literals`, `hexadecimal_number_runs_are_greedy_before_suffixes`, `zero_with_a_byte_suffix_is_not_a_binary_prefix`, `negative_radix_literals_require_explicit_signed_suffixes`, `number_runs_allow_repeated_interior_underscores`, `parses_floating_point_literals`, `rejects_underscores_at_number_run_boundaries`, `parses_typed_arrays`, `parses_builtins`, `builtin_operation_lookup_uses_the_actual_argument_count`, `parses_string_escapes`, `argument_parser_does_not_consume_trailing_whitespace`, `full_parser_rejects_trailing_data`, `errors_preserve_semantic_kinds_and_translation_arguments`, `numeric_syntax_errors_preserve_specific_kinds_and_cursors`, `argument_parser_stops_after_a_complete_number`, `errors_match_vanilla_alternative_selection_and_cursors`, `nested_compounds_filter_past_the_depth_ceiling`, `nested_lists_filter_past_the_depth_ceiling`, `ordinary_nesting_still_parses` | 0 |
| `steel-utils/src/nbt/snbt/writer.rs` | 139 | — | 0 |
| `steel-utils/src/nbt/value_input.rs` | 259 | `integer_tags_narrow_by_keeping_low_bits`, `floating_tags_floor_into_integers`, `float_long_value_truncates_but_double_floors`, `floating_tags_saturate_before_narrowing`, `compound_reads_accept_any_numeric_tag`, `borrowed_compound_reads_accept_any_numeric_tag` | 0 |
| `steel-utils/src/random/gaussian.rs` | 30 | — | 0 |
| `steel-utils/src/random/legacy_random.rs` | 532 | `test_next_i32`, `test_next_i32_bounded`, `test_next_i32_between`, `test_next_i32_between_exclusive`, `test_next_f64`, `test_next_f32`, `test_next_i64`, `test_next_bool`, `test_next_gaussian`, `test_triangle`, `consume_count_matches_naive_loop`, `consume_count_negative_is_noop`, `test_fork`, `test_set_seed_matches_from_seed`, `test_set_large_feature_with_salt_trivial`, `test_set_large_feature_with_salt`, `test_set_large_feature_seed` | 0 |
| `steel-utils/src/random/mod.rs` | 121 | — | 0 |
| `steel-utils/src/random/name_hash.rs` | 215 | `const_md5_matches_crate`, `java_hash_matches_legacy`, `name_hash_is_const` | 0 |
| `steel-utils/src/random/weighted_random.rs` | 90 | `total_weight_sums_every_entry`, `weighted_item_walks_cumulative_weights`, `zero_total_weight_yields_nothing_without_consuming_the_random`, `random_item_is_deterministic_for_a_seed` | 0 |
| `steel-utils/src/random/worldgen_random.rs` | 170 | `set_decoration_seed_matches_vanilla_trace`, `feature_seed_matches_vanilla_first_ore_dirt_origin`, `feature_seed_preserves_pending_gaussian` | 0 |
| `steel-utils/src/random/xoroshiro.rs` | 455 | `test_mix_stafford_13`, `next_i32_matches_java`, `next_i32_bounded`, `next_i32_between_inclusive`, `next_i32_between_exclusive`, `next_f64_matches_java`, `next_f32_matches_java`, `next_i64_matches_java`, `next_bool_matches_java`, `next_gaussian_matches_java`, `triangle_matches_java`, `fork_creates_independent_rng`, `positional_random_splitter`, `zero_seed_produces_fallback_values` | 0 |
| `steel-utils/src/registry.rs` | 65 | — | 0 |
| `steel-utils/src/rotation.rs` | 239 | `rotate_direction`, `compose_rotations`, `vertical_unchanged`, `transform_pos_pivot_zero`, `bounding_box_none`, `bounding_box_cw90`, `bounding_box_cw180`, `rotate_size` | 0 |
| `steel-utils/src/saved_data.rs` | 406 | `saved_data_name_rejects_paths`, `missing_saved_data_loads_default`, `saved_data_round_trips_through_world_data_dir`, `ephemeral_saved_data_does_not_write`, `wincode_saved_data_round_trips_with_header`, `missing_and_ephemeral_wincode_data_return_none` | 0 |
| `steel-utils/src/serial/mod.rs` | 69 | — | 0 |
| `steel-utils/src/serial/prefixed_read.rs` | 379 | `truncated_string_leaves_cursor_after_prefix`, `length_failures_match_vanilla_cursor_consumption`, `accepts_maximum_encoded_length`, `malformed_sequences_match_java_replacement_grouping`, `rejects_impossible_maximum_without_reading_the_body` | 0 |
| `steel-utils/src/serial/prefixed_write.rs` | 99 | — | 0 |
| `steel-utils/src/serial/read.rs` | 142 | — | 0 |
| `steel-utils/src/serial/vec_as_map.rs` | 56 | — | 0 |
| `steel-utils/src/serial/write.rs` | 235 | — | 0 |
| `steel-utils/src/text/mod.rs` | 882 | `deprecated_vanilla_translations_are_applied`, `component_stream_codec_round_trips_an_unnamed_nbt_tag`, `component_codec_recursively_collapses_plain_components`, `component_stream_codec_writes_collapsed_string_tag` | 0 |
| `steel-utils/src/text/nbt.rs` | 334 | `renders_vanilla_suffixes_arrays_and_signed_bytes`, `quotes_and_escapes_strings_and_compound_keys`, `folds_nested_collections_at_vanillas_depth_limit`, `keeps_empty_collections_visible_at_vanillas_depth_limit`, `folds_arrays_after_vanillas_element_limit`, `renders_lists_with_vanillas_spacing` | 0 |
| `steel-utils/src/threading.rs` | 725 | `explicit_worker_count_is_capped_to_available_threads`, `zero_or_missing_worker_count_uses_auto_default`, `cgroup_v2_quota_becomes_whole_cores`, `cgroup_v2_quota_survives_odd_whitespace_and_trailing_newline`, `cgroup_v2_malformed_contents_are_ignored`, `sub_core_quotas_clamp_to_zero_before_the_caller_floors_them`, `a_six_core_container_gets_a_balanced_split`, `cpu_bound_pools_never_oversubscribe_the_core_budget`, `every_core_is_used_by_the_cpu_bound_pools`, `tiny_containers_never_spawn_idle_pools`, `a_zero_core_report_still_produces_a_usable_split`, `io_pools_keep_at_least_one_worker_on_every_size`, `nested_cgroup_v2_path_is_read_from_proc_self_cgroup`, `the_root_cgroup_is_its_own_path`, `a_v1_only_process_has_no_v2_path`, `a_mixed_hierarchy_picks_the_v2_line`, `a_cpuset_range_counts_its_cores`, `a_v1_cpuset_hierarchy_is_found_by_its_controller`, `a_host_without_a_cpuset_controller_has_no_v1_base`, `malformed_hierarchy_lines_do_not_hide_the_cpuset_controller`, `the_first_usable_cpu_set_wins_and_missing_files_are_skipped`, `a_cpuset_of_max_is_not_a_core_count`, `effective_cores_never_exceeds_the_container_allowance`, `effective_cores_treats_zero_as_auto_and_never_returns_zero`, `a_sub_core_quota_still_yields_one_worker`, `an_empty_or_malformed_cpuset_falls_back_to_the_os` | 0 |
| `steel-utils/src/types/codec_glue.rs` | 53 | — | 0 |
| `steel-utils/src/types/gameplay.rs` | 255 | — | 0 |
| `steel-utils/src/types/identifier.rs` | 226 | — | 0 |
| `steel-utils/src/types/mod.rs` | 22 | — | 0 |
| `steel-utils/src/types/packed_position.rs` | 394 | — | 0 |
| `steel-utils/src/types/position.rs` | 813 | `closer_to_center_than_uses_block_center`, `closer_than_compares_integer_offsets_strictly` | 0 |
| `steel-utils/src/types/tests.rs` | 200 | `identifier_parsing_matches_vanilla_default_namespace_rules`, `identifier_parsing_rejects_vanilla_invalid_names`, `test_block_pos_roundtrip`, `test_block_pos_specific_case`, `block_pos_within_manhattan_starts_in_vanilla_order`, `block_pos_spiral_around_radius_zero_returns_center`, `block_pos_spiral_around_matches_vanilla_order`, `block_pos_find_closest_match_uses_vanilla_order`, `packed_chunk_local_xz_masks_absolute_coordinates`, `packed_chunk_local_xz_rejects_invalid_local_coordinates`, `entity_positions_floor_before_chunk_and_section_conversion`, `packed_section_block_pos_masks_absolute_coordinates`, `packed_section_block_pos_rejects_invalid_raw_bits`, `packed_section_block_pos_rejects_invalid_local_coordinates`, `packed_section_block_pos_converts_to_absolute_block_pos`, `packed_position_newtypes_roundtrip` | 0 |
| `steel-utils/src/uuid_ext.rs` | 66 | `test_uuid_roundtrip`, `test_uuid_from_invalid_array` | 0 |
| `steel-utils/src/value_providers/float.rs` | 150 | — | 0 |
| `steel-utils/src/value_providers/height.rs` | 222 | — | 0 |
| `steel-utils/src/value_providers/int.rs` | 403 | — | 0 |
| `steel-utils/src/value_providers/mod.rs` | 14 | — | 0 |
| `steel-utils/src/value_providers/tests.rs` | 311 | `vertical_anchor_resolve`, `vertical_anchor_deserialize`, `height_provider_deserialize_shortcut`, `height_provider_uniform_from_carver_json`, `float_provider_bare_float`, `float_provider_uniform_from_carver_json`, `float_provider_trapezoid_from_carver_json`, `int_provider_clamped_normal_prefers_tagged_shape`, `provider_type_tags_require_extracted_registry_ids`, `provider_typed_payloads_deny_unknown_fields`, `int_provider_requires_typed_object_or_bare_constant`, `int_provider_symmetric_trapezoid_sample_matches_vanilla_shortcut`, `float_provider_uniform_sample_matches_vanilla`, `height_provider_uniform_sample_matches_vanilla` | 0 |
| `steel-utils/src/value_providers/vertical_anchor.rs` | 63 | — | 0 |
| `steel-core/build/blocks.rs` | 95 | — | 0 |
| `steel-core/build/build.rs` | 145 | — | 0 |
| `steel-core/build/candle_cakes.rs` | 33 | — | 0 |
| `steel-core/build/common.rs` | 479 | — | 0 |
| `steel-core/build/entities.rs` | 117 | — | 0 |
| `steel-core/build/flammable.rs` | 76 | — | 0 |
| `steel-core/build/items.rs` | 96 | — | 0 |
| `steel-core/build/skull_type.rs` | 49 | — | 0 |
| `steel-core/build/waxables.rs` | 54 | — | 0 |
| `steel-core/build/weathering.rs` | 99 | — | 0 |
| `steel-core/src/behavior/block/collision.rs` | 80 | — | 0 |
| `steel-core/src/behavior/block/context.rs` | 613 | `bounce_uses_fraction_of_requested_movement_actually_achieved`, `non_living_entities_bounce_lower` | 0 |
| `steel-core/src/behavior/block/mod.rs` | 1474 | — | 0 |
| `steel-core/src/behavior/block/registry.rs` | 95 | — | 0 |
| `steel-core/src/behavior/block/tests.rs` | 239 | `default_spawn_rule_requires_full_up_support_and_low_light_emission`, `clone_item_stack_uses_registered_block_item_association`, `drained_waterlogged_state_clears_waterlogged_property`, `drained_waterlogged_state_ignores_dry_or_non_waterloggable_blocks`, `waterlogged_barrier_pickup_requires_player_context`, `default_fluid_replacement_does_not_use_waterloggable_property_alone`, `default_behavior_preserves_unported_simple_waterlogged_blocks`, `default_behavior_schedules_shape_ticks_only_for_waterlogged_states`, `fall_on_facts_use_vanilla_width_squared_height_formula`, `world_aabb_bounds_contains_all_boxes` | 0 |
| `steel-core/src/behavior/block/waterlogging.rs` | 99 | — | 0 |
| `steel-core/src/behavior/blocks/building/amethyst_block.rs` | 61 | `projectile_chime_pitch_matches_vanilla_range` | 0 |
| `steel-core/src/behavior/blocks/building/amethyst_cluster.rs` | 82 | — | 0 |
| `steel-core/src/behavior/blocks/building/bar_block.rs` | 213 | — | 0 |
| `steel-core/src/behavior/blocks/building/barrier_block.rs` | 112 | `registered_barrier_rejects_no_user_liquid_placement`, `waterlogged_barrier_update_shape_schedules_water_tick` | 0 |
| `steel-core/src/behavior/blocks/building/bed_block.rs` | 401 | — | 0 |
| `steel-core/src/behavior/blocks/building/brushable_block.rs` | 263 | `suspicious_sand_and_gravel_fall_then_break_without_drops` | 0 |
| `steel-core/src/behavior/blocks/building/budding_amethyst.rs` | 175 | `growth_state_waterlogs_when_replacing_water_block`, `cluster_can_grow_in_falling_full_water`, `cluster_cannot_grow_in_partial_flowing_water` | 0 |
| `steel-core/src/behavior/blocks/building/campfire_block.rs` | 482 | `lit_campfire_damages_living_entities`, `unlit_campfire_does_not_damage_entities`, `campfire_does_not_damage_non_living_entities`, `burning_projectile_lights_only_dry_unlit_campfires`, `placement_state_sets_facing_and_signal_fire`, `update_shape_recomputes_signal_fire_from_below`, `water_placement_extinguishes_lit_campfire`, `is_lit_campfire_accepts_only_lit_campfires`, `douse_emits_block_change_without_touching_the_block` | 0 |
| `steel-core/src/behavior/blocks/building/cauldron_block.rs` | 216 | `registered_cauldron_behaviors_expose_vanilla_fill_levels` | 0 |
| `steel-core/src/behavior/blocks/building/composter_block.rs` | 344 | `first_layer_uses_context_provider_for_low_chance_compostable_items`, `cake_fills_seven_layers_then_waits_exactly_twenty_world_ticks`, `non_compostable_item_is_not_accepted_at_any_level`, `hopper_above_composts_one_item_and_hopper_below_extracts_once`, `full_hopper_cannot_delete_composter_produce`, `horizontal_hopper_does_not_compost_through_side`, `player_dispatch_consumes_in_survival_and_adventure_but_not_creative`, `hopper_skips_non_compostable_first_slot_without_consuming_it`, `registered_composter_outputs_its_full_state_level` | 0 |
| `steel-core/src/behavior/blocks/building/conduit_block.rs` | 95 | — | 0 |
| `steel-core/src/behavior/blocks/building/door_block.rs` | 584 | `lower_half_copies_transformed_upper_half_state`, `door_wooden_query_uses_can_open_by_hand_like_vanilla` | 0 |
| `steel-core/src/behavior/blocks/building/drop_experience_block.rs` | 110 | `generated_constant_and_uniform_providers_control_ore_experience`, `silk_touch_and_disabled_experience_suppress_ore_experience` | 0 |
| `steel-core/src/behavior/blocks/building/fence_block.rs` | 237 | `waterlogged_fence_update_shape_schedules_water_tick` | 0 |
| `steel-core/src/behavior/blocks/building/fence_gate_block.rs` | 260 | `redstone_power_opens_and_closes_fence_gate` | 0 |
| `steel-core/src/behavior/blocks/building/frosted_ice_block.rs` | 367 | `pick_block_returns_empty`, `custom_world_key_retains_end_dimension_semantics`, `fewer_neighbors_than_counts_adjacent_frosted_ice`, `slightly_melt_increments_age_before_melting`, `on_place_schedules_a_tick`, `neighbor_change_melts_isolated_frosted_ice`, `neighbor_change_keeps_well_supported_frosted_ice` | 0 |
| `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs` | 36 | — | 0 |
| `steel-core/src/behavior/blocks/building/hay_block.rs` | 68 | `hay_reduces_fall_damage_like_vanilla` | 0 |
| `steel-core/src/behavior/blocks/building/heavy_core.rs` | 60 | — | 0 |
| `steel-core/src/behavior/blocks/building/honey_block.rs` | 303 | `slide_requires_entity_to_be_airborne`, `slide_requires_entity_below_honey_top`, `slide_requires_fast_enough_downward_motion`, `slide_requires_entity_overlapping_honey_edge`, `slide_throttles_fast_fall_horizontal_velocity`, `slide_preserves_horizontal_velocity_when_fall_is_not_fast` | 0 |
| `steel-core/src/behavior/blocks/building/ice_block.rs` | 136 | `melts_into_water_by_default`, `prevents_ice_melting_with_silk_touch` | 0 |
| `steel-core/src/behavior/blocks/building/infested_block.rs` | 962 | `frogspawn_schedules_its_hatch_tick_on_placement`, `frogspawn_breaks_when_its_support_disappears`, `frogspawn_needs_support_below_and_no_fluid_above` | 0 |
| `steel-core/src/behavior/blocks/building/ladder_block.rs` | 183 | `support_neighbor_update_breaks_unsupported_ladder`, `waterlogged_ladder_contains_non_falling_source_water` | 0 |
| `steel-core/src/behavior/blocks/building/lava_cauldron_block.rs` | 150 | `filled_entity_inside_shape_includes_lava_column`, `registered_lava_cauldron_outputs_three` | 0 |
| `steel-core/src/behavior/blocks/building/magma_block.rs` | 89 | `magma_damages_non_careful_living_entities`, `magma_does_not_damage_careful_living_entities`, `magma_does_not_damage_non_living_entities` | 0 |
| `steel-core/src/behavior/blocks/building/mod.rs` | 95 | — | 0 |
| `steel-core/src/behavior/blocks/building/mud.rs` | 58 | `is_pathfindable_returns_false_for_all_types` | 0 |
| `steel-core/src/behavior/blocks/building/netherrack_block.rs` | 107 | — | 0 |
| `steel-core/src/behavior/blocks/building/potent_sulfur_block.rs` | 231 | `potent_sulfur_ticker_selection_matches_live_geyser_state` | 0 |
| `steel-core/src/behavior/blocks/building/powder_snow_block.rs` | 367 | `powder_snow_fall_sound_uses_vanilla_living_thresholds`, `falling_entities_collide_with_lower_powder_snow_shape`, `walkable_entities_use_default_powder_snow_collision_shape_when_above`, `non_walkable_or_descending_entities_have_no_powder_snow_collision`, `falling_blocks_use_default_powder_snow_collision_shape`, `placement_context_has_no_powder_snow_collision` | 0 |
| `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs` | 50 | — | 0 |
| `steel-core/src/behavior/blocks/building/scaffolding_block.rs` | 360 | `placement_context_has_no_scaffolding_collision`, `entity_above_scaffolding_collides_with_stable_shape`, `descending_entity_only_collides_with_unstable_bottom_shape`, `non_bottom_descending_scaffolding_has_empty_collision`, `shape_update_schedules_stability_and_water_ticks`, `distance_uses_sturdy_vertical_and_nearest_horizontal_support`, `newly_unsupported_scaffolding_breaks_and_drops_an_item`, `max_distance_waterlogged_scaffolding_falls_and_leaves_water` | 0 |
| `steel-core/src/behavior/blocks/building/simple_marker_blocks.rs` | 408 | — | 0 |
| `steel-core/src/behavior/blocks/building/slab_block.rs` | 203 | `double_slabs_cannot_be_waterlogged`, `single_slabs_accept_source_water_for_container_admission`, `single_slabs_reject_flowing_water_for_container_admission` | 0 |
| `steel-core/src/behavior/blocks/building/slime_block.rs` | 99 | `step_on_damps_horizontal_velocity_for_non_careful_entities`, `step_on_keeps_velocity_for_careful_entities`, `step_on_keeps_horizontal_velocity_when_vertical_speed_is_large` | 0 |
| `steel-core/src/behavior/blocks/building/snow_layer_block.rs` | 115 | — | 0 |
| `steel-core/src/behavior/blocks/building/soul_sand_block.rs` | 70 | `is_pathfindable_returns_false_for_all_types` | 0 |
| `steel-core/src/behavior/blocks/building/spawner_block.rs` | 94 | — | 0 |
| `steel-core/src/behavior/blocks/building/sponge_block.rs` | 150 | — | 0 |
| `steel-core/src/behavior/blocks/building/stair_block.rs` | 219 | `stair_update_shape_recomputes_shape_from_neighbors` | 0 |
| `steel-core/src/behavior/blocks/building/straw_bed_block.rs` | 172 | — | 0 |
| `steel-core/src/behavior/blocks/building/technical_blocks.rs` | 1240 | — | 0 |
| `steel-core/src/behavior/blocks/building/trapdoor_block.rs` | 380 | `closed_trapdoor_is_not_land_or_air_pathfindable`, `open_waterlogged_trapdoor_matches_vanilla_pathfinding`, `redundant_redstone_notification_does_not_schedule_water_tick` | 0 |
| `steel-core/src/behavior/blocks/building/wall_block.rs` | 361 | — | 0 |
| `steel-core/src/behavior/blocks/building/waterlogged_transparent_block.rs` | 117 | `waterlogged_transparent_block_returns_falling_source_water` | 0 |
| `steel-core/src/behavior/blocks/building/weathering_block.rs` | 184 | — | 0 |
| `steel-core/src/behavior/blocks/building/web_block.rs` | 50 | — | 0 |
| `steel-core/src/behavior/blocks/building/wet_sponge_block.rs` | 58 | — | 0 |
| `steel-core/src/behavior/blocks/colored/mod.rs` | 4 | — | 0 |
| `steel-core/src/behavior/blocks/colored/stained_glass_block.rs` | 44 | — | 0 |
| `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs` | 74 | — | 0 |
| `steel-core/src/behavior/blocks/container/anvil_block.rs` | 160 | `every_anvil_variant_starts_falling_without_support` | 0 |
| `steel-core/src/behavior/blocks/container/barrel_block.rs` | 146 | — | 0 |
| `steel-core/src/behavior/blocks/container/beacon_block.rs` | 113 | — | 0 |
| `steel-core/src/behavior/blocks/container/beehive_block.rs` | 338 | `shearing_full_hive_resets_honey_and_drops_three_honeycombs` | 0 |
| `steel-core/src/behavior/blocks/container/block_entity_blocks.rs` | 1263 | — | 0 |
| `steel-core/src/behavior/blocks/container/block_entity_blocks_tests.rs` | 105 | `powered_dispenser_grows_wheat_and_consumes_exactly_one_bone_meal`, `failed_bone_meal_dispense_keeps_inventory_intact`, `bottles_fill_from_source_flowing_water_and_waterlogged_blocks_without_draining`, `bottle_collects_honey_resets_hive_and_stores_stack_remainder`, `dispenser_flint_lights_candles_cakes_campfires_and_always_wears_on_blocked_targets` | 0 |
| `steel-core/src/behavior/blocks/container/chest_block.rs` | 492 | — | 0 |
| `steel-core/src/behavior/blocks/container/chiseled_bookshelf_block.rs` | 836 | `all_six_hit_regions_map_identically_for_every_horizontal_facing`, `hit_boundaries_use_vanilla_pixel_sections`, `placement_faces_the_player_and_uses_the_extracted_full_block_shape`, `interactions_update_inventory_occupied_state_and_comparator`, `enchanted_books_select_the_enchanted_insert_and_pickup_sounds`, `placement_applies_the_container_component_to_the_block_entity`, `destruction_drains_and_drops_every_stored_book` | 0 |
| `steel-core/src/behavior/blocks/container/copper_chest_block.rs` | 244 | — | 0 |
| `steel-core/src/behavior/blocks/container/crafting_table_block.rs` | 69 | — | 0 |
| `steel-core/src/behavior/blocks/container/enchanting_table_block.rs` | 205 | `bookshelves_count_only_with_a_clear_transmitter_block` | 0 |
| `steel-core/src/behavior/blocks/container/ender_chest_block.rs` | 114 | — | 0 |
| `steel-core/src/behavior/blocks/container/furnace_block.rs` | 212 | — | 0 |
| `steel-core/src/behavior/blocks/container/grindstone_block.rs` | 82 | — | 0 |
| `steel-core/src/behavior/blocks/container/mod.rs` | 77 | — | 0 |
| `steel-core/src/behavior/blocks/container/shulker_box_block.rs` | 295 | — | 0 |
| `steel-core/src/behavior/blocks/container/station_blocks.rs` | 186 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/banner_block.rs` | 151 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/cake_block.rs` | 172 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/candle_block.rs` | 523 | `waterlogged_candle_update_shape_schedules_water_tick`, `burning_projectile_lights_only_unlit_candles`, `water_placement_on_lit_candle_emits_block_change_event`, `is_lit_accepts_only_lit_candles_and_candle_cakes`, `extinguish_matches_vanilla_side_effects`, `candle_can_be_replaced_matching_stack`, `candle_placement_state_increments_and_preserves_properties`, `candle_can_place_when_clicking_block_below` | 0 |
| `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs` | 176 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/chain_block.rs` | 129 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/end_rod_block.rs` | 41 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs` | 369 | `placing_and_removing_a_plant_swaps_the_potted_state`, `update_shape_leaves_a_floating_pot_intact`, `clone_item_stack_returns_the_potted_plant` | 0 |
| `steel-core/src/behavior/blocks/decoration/jukebox_block.rs` | 228 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/jukebox_tests.rs` | 760 | `insertion_consumption_ejection_and_no_duplication_match_vanilla`, `stackable_component_backed_record_consumes_one_item`, `level_events_periodic_game_events_and_duration_completion_match_song_data`, `placement_data_and_nonzero_persistence_resume_without_restarting_audio`, `breaking_a_playing_jukebox_drops_one_record_and_runs_vanilla_cleanup` | 0 |
| `steel-core/src/behavior/blocks/decoration/lantern_block.rs` | 96 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/mod.rs` | 29 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/sign_block.rs` | 1098 | `all_sign_placements_preserve_source_and_flowing_water`, `standing_sign_only_schedules_water_when_support_survives`, `sign_variants_select_their_matching_vanilla_tickers`, `try_open_sign_editor_checks_waxed_and_may_build`, `try_apply_sign_item_dyes_sign_text` | 0 |
| `steel-core/src/behavior/blocks/decoration/skull/abstract_skull_block.rs` | 104 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/skull/mod.rs` | 337 | `roundtrip` | 0 |
| `steel-core/src/behavior/blocks/decoration/skull/piglin_wall_skull_block.rs` | 82 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/skull/player_head_block.rs` | 82 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/skull/player_wall_head_block.rs` | 82 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/skull/skull_block.rs` | 83 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/skull/wall_skull_block.rs` | 99 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/skull/wither_skull_block.rs` | 172 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/skull/wither_wall_skull_block.rs` | 94 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/torch_block.rs` | 142 | — | 0 |
| `steel-core/src/behavior/blocks/decoration/weathering_lantern_block.rs` | 64 | — | 0 |
| `steel-core/src/behavior/blocks/face_attached_horizontal_directional_block.rs` | 109 | — | 0 |
| `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs` | 129 | — | 0 |
| `steel-core/src/behavior/blocks/falling/dragon_egg_block.rs` | 128 | — | 0 |
| `steel-core/src/behavior/blocks/falling/falling_block.rs` | 116 | — | 0 |
| `steel-core/src/behavior/blocks/falling/mod.rs` | 14 | — | 0 |
| `steel-core/src/behavior/blocks/falling/sand_block.rs` | 68 | — | 0 |
| `steel-core/src/behavior/blocks/falling/tests.rs` | 71 | `falling_shape_update_schedules_the_vanilla_two_tick_delay`, `concrete_powder_shape_update_matches_vanilla_liquid_adjacency` | 0 |
| `steel-core/src/behavior/blocks/fluid/bubble_column_block.rs` | 430 | `bubble_column_update_shape_schedules_water_and_column_tick`, `bubble_column_update_column_uses_push_up_and_drag_down_blocks`, `precise_entity_with_open_block_above_uses_above_bubble_column_hook`, `precise_entity_with_fluid_above_stays_inside_bubble_column`, `imprecise_entity_does_not_apply_bubble_column_effect` | 0 |
| `steel-core/src/behavior/blocks/fluid/liquid_block.rs` | 404 | `update_shape_schedules_actual_flowing_fluid_variant`, `source_water_above_soul_sand_schedules_bubble_column_tick`, `flowing_water_does_not_schedule_bubble_column_tick`, `liquid_block_behavior_and_ext_identify_liquid_blocks`, `pickup_returns_the_fluid_bucket_and_sound_from_sources_only` | 0 |
| `steel-core/src/behavior/blocks/fluid/mod.rs` | 5 | — | 0 |
| `steel-core/src/behavior/blocks/mod.rs` | 27 | — | 0 |
| `steel-core/src/behavior/blocks/portal/end_portal_block.rs` | 367 | `registered_end_portal_blocks_reject_fluid_replacement`, `end_portal_entity_inside_shape_uses_outline_shape`, `end_portal_creates_end_portal_block_entity`, `end_portal_marks_non_player_inside_end_portal`, `end_gateway_creates_typed_block_entity` | 0 |
| `steel-core/src/behavior/blocks/portal/end_portal_frame_block.rs` | 91 | `end_portal_frame_analog_output_depends_on_eye` | 0 |
| `steel-core/src/behavior/blocks/portal/fire.rs` | 648 | `get_state_selects_soul_fire_on_soul_fire_base_block`, `get_state_selects_regular_fire_otherwise`, `update_shape_removes_unsupported_fire`, `horizontal_support_removal_extinguishes_fire`, `changing_floor_to_soul_sand_converts_existing_fire`, `update_shape_removes_unsupported_soul_fire` | 0 |
| `steel-core/src/behavior/blocks/portal/mod.rs` | 11 | — | 0 |
| `steel-core/src/behavior/blocks/portal/nether_portal_block.rs` | 134 | `nether_portal_marks_entity_inside_portal` | 0 |
| `steel-core/src/behavior/blocks/portal/respawn_anchor_block.rs` | 556 | `in_water_follows_vanilla_flow_rules`, `water_shields_blocks_from_anchor_explosion`, `standup_candidates_match_vanilla_order`, `charge_helpers_match_vanilla_bounds`, `charge_consume_after_respawn_matches_vanilla_condition`, `consumed_charge_state_decrements_exactly_once` | 0 |
| `steel-core/src/behavior/blocks/redstone/bell_block.rs` | 446 | `wall_attachment_uses_facing_as_support_direction`, `generated_registry_bell_behavior_creates_registered_typed_entity`, `placed_bell_is_stored_with_its_ticker` | 0 |
| `steel-core/src/behavior/blocks/redstone/button_block.rs` | 332 | `wooden_button_stays_pressed_while_arrow_intersects_its_shape` | 0 |
| `steel-core/src/behavior/blocks/redstone/copper_bulb_block.rs` | 239 | `bulb_toggles_lit_only_on_rising_edges` | 0 |
| `steel-core/src/behavior/blocks/redstone/daylight_detector_block.rs` | 244 | `daylight_detector_selects_vanilla_server_ticker_in_skylight_dimensions`, `java_round_matches_vanilla_half_toward_positive_infinity`, `signal_strength_matches_vanilla_sun_angle_adjustment`, `vanilla_trig_table_controls_overworld_rounding_boundary` | 0 |
| `steel-core/src/behavior/blocks/redstone/diode/base.rs` | 259 | — | 0 |
| `steel-core/src/behavior/blocks/redstone/diode/comparator.rs` | 509 | `output_calculation_matches_compare_and_subtract_modes`, `equality_powers_only_compare_mode`, `comparator_creates_typed_output_storage`, `item_frame_signal_uses_item_frame_capability` | 0 |
| `steel-core/src/behavior/blocks/redstone/diode/mod.rs` | 6 | — | 0 |
| `steel-core/src/behavior/blocks/redstone/diode/repeater.rs` | 347 | `input_reads_wire_power_in_front`, `side_lock_accepts_only_another_diode`, `powered_output_is_directional`, `support_requires_vanilla_rigid_face`, `neighbor_tick_priority_matches_vanilla_diode_ordering` | 0 |
| `steel-core/src/behavior/blocks/redstone/java_hash.rs` | 19 | — | 0 |
| `steel-core/src/behavior/blocks/redstone/lever_block.rs` | 250 | `wall_lever_survives_only_with_its_backing_face`, `powered_lever_strongly_powers_only_away_from_support`, `redstone_transition_side_effects_match_vanilla_for_lever` | 0 |
| `steel-core/src/behavior/blocks/redstone/lightning_rod_block.rs` | 168 | — | 0 |
| `steel-core/src/behavior/blocks/redstone/mod.rs` | 50 | — | 0 |
| `steel-core/src/behavior/blocks/redstone/note_block.rs` | 323 | `vertical_blocks_select_instruments_with_vanilla_priority`, `tuning_wraps_after_the_top_note`, `redstone_updates_powered_state_on_both_edges` | 0 |
| `steel-core/src/behavior/blocks/redstone/observer_block.rs` | 242 | `observed_face_update_schedules_one_two_tick_pulse`, `observer_output_is_powered_and_directional` | 0 |
| `steel-core/src/behavior/blocks/redstone/piston/base.rs` | 881 | `pushability_honors_bounds_reactions_and_block_entities`, `placement_uses_player_look_direction_not_clicked_face`, `moving_piston_delegates_loot_to_carried_state`, `normal_piston_extends_settles_and_retracts_without_pulling`, `retracting_piston_keeps_rear_face_attachments_supported`, `sticky_piston_pulls_settled_normal_block` | 0 |
| `steel-core/src/behavior/blocks/redstone/piston/head.rs` | 134 | — | 0 |
| `steel-core/src/behavior/blocks/redstone/piston/mod.rs` | 8 | — | 0 |
| `steel-core/src/behavior/blocks/redstone/piston/moving.rs` | 175 | `moving_piston_selects_only_the_piston_block_entity_ticker` | 0 |
| `steel-core/src/behavior/blocks/redstone/piston/structure_resolver.rs` | 316 | `resolver_accepts_twelve_blocks_and_rejects_thirteen`, `slime_branches_but_does_not_bind_honey` | 0 |
| `steel-core/src/behavior/blocks/redstone/powered_block.rs` | 44 | — | 0 |
| `steel-core/src/behavior/blocks/redstone/pressure_plate/base.rs` | 230 | `redstone_transition_side_effects_match_vanilla_for_pressure_plates` | 0 |
| `steel-core/src/behavior/blocks/redstone/pressure_plate/mod.rs` | 6 | — | 0 |
| `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs` | 271 | `pressure_plate_survives_on_rigid_or_center_support`, `powered_pressure_plate_strongly_powers_only_upward` | 0 |
| `steel-core/src/behavior/blocks/redstone/pressure_plate/weighted.rs` | 201 | `heavy_plate_uses_vanilla_float_ceiling_boundaries` | 0 |
| `steel-core/src/behavior/blocks/redstone/rail/base_rail_block.rs` | 199 | `rail_capability_requires_both_tag_and_behavior` | 0 |
| `steel-core/src/behavior/blocks/redstone/rail/detector_rail_block.rs` | 316 | `minecart_powers_detector_and_schedules_relative_tick` | 0 |
| `steel-core/src/behavior/blocks/redstone/rail/mod.rs` | 10 | — | 0 |
| `steel-core/src/behavior/blocks/redstone/rail/powered_rail_block.rs` | 352 | `powered_signal_reaches_exact_vanilla_depth_limit`, `powered_propagation_requires_exact_block_identity` | 0 |
| `steel-core/src/behavior/blocks/redstone/rail/rail_block.rs` | 180 | `powered_three_way_junction_uses_vanilla_curve_priority` | 0 |
| `steel-core/src/behavior/blocks/redstone/rail/rail_state.rs` | 436 | `four_way_curve_tie_uses_vanilla_sequential_overwrite_order`, `east_upper_neighbor_creates_slope_with_ordered_connections` | 0 |
| `steel-core/src/behavior/blocks/redstone/redstone_lamp_block.rs` | 70 | — | 0 |
| `steel-core/src/behavior/blocks/redstone/redstone_ore_block.rs` | 225 | `both_ore_variants_light_from_steps_and_extinguish_on_random_ticks`, `world_drop_resources_dispatches_redstone_ore_experience` | 0 |
| `steel-core/src/behavior/blocks/redstone/redstone_torch_block.rs` | 464 | `standing_torch_reads_power_from_its_support`, `wall_torch_omits_weak_signal_toward_its_support` | 0 |
| `steel-core/src/behavior/blocks/redstone/target_block.rs` | 181 | `hit_strength_uses_the_hit_face_and_floor_based_fraction` | 0 |
| `steel-core/src/behavior/blocks/redstone/tnt_block/tests.rs` | 80 | `adventure_owner_primes_only_when_the_igniting_stack_can_break_tnt`, `explosion_replaces_tnt_with_a_short_fuse_entity_owned_by_the_igniter` | 0 |
| `steel-core/src/behavior/blocks/redstone/tnt_block.rs` | 263 | — | 0 |
| `steel-core/src/behavior/blocks/redstone/tripwire/block.rs` | 325 | `entity_inside_powers_wire_and_attached_hooks` | 0 |
| `steel-core/src/behavior/blocks/redstone/tripwire/hook.rs` | 405 | `line_attachment_power_and_disarming_match_vanilla` | 0 |
| `steel-core/src/behavior/blocks/redstone/tripwire/mod.rs` | 5 | — | 0 |
| `steel-core/src/behavior/blocks/redstone/weathering_lightning_rod_block.rs` | 128 | — | 0 |
| `steel-core/src/behavior/blocks/redstone/wire/block.rs` | 647 | `isolated_cross_and_dot_preserve_their_vanilla_shapes`, `wire_climbs_sturdy_neighbor_only_when_above_is_connectable`, `powered_wire_signal_follows_recomputed_connections` | 0 |
| `steel-core/src/behavior/blocks/redstone/wire/evaluator.rs` | 541 | `seven_position_order_matches_target_jdk_hash_set_fixtures`, `incoming_wire_power_does_not_feed_back_through_signal_queries`, `target_strength_matches_vanilla`, `target_strength_uses_fewer_block_state_reads`, `live_world_wire_line_settles_after_source_toggle` | 0 |
| `steel-core/src/behavior/blocks/redstone/wire/mod.rs` | 4 | — | 0 |
| `steel-core/src/behavior/blocks/utils.rs` | 29 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/attached_stem_block.rs` | 239 | `attached_stem_keeps_its_fruit_and_reverts_to_mature_stem`, `attached_stem_breaks_when_its_support_disappears`, `registered_attached_mappings_are_not_swappable` | 0 |
| `steel-core/src/behavior/blocks/vegetation/azalea_block.rs` | 59 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/bamboo.rs` | 284 | `bamboo_growth_does_not_read_leaves_from_non_bamboo_support` | 0 |
| `steel-core/src/behavior/blocks/vegetation/bamboo_sapling.rs` | 130 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/base_coral_fan_block.rs` | 62 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/base_coral_plant_block.rs` | 97 | `waterlogged_base_coral_plant_update_shape_schedules_water_tick` | 0 |
| `steel-core/src/behavior/blocks/vegetation/base_coral_wall_fan_block.rs` | 78 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/beetroots.rs` | 84 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/big_dripleaf_block.rs` | 414 | `redstone_holds_big_dripleaf_upright` | 0 |
| `steel-core/src/behavior/blocks/vegetation/big_dripleaf_stem_block.rs` | 164 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/bonemealable.rs` | 156 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/bush_block.rs` | 84 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/cactus_block.rs` | 214 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/cactus_flower_block.rs` | 68 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/carpet_block.rs` | 91 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/carrot.rs` | 79 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/carved_pumpkin_block.rs` | 183 | `iron_pattern_requires_upper_arms_and_clear_corners_in_both_axes`, `copper_construction_preserves_wax_and_oxidation_at_both_extremes` | 0 |
| `steel-core/src/behavior/blocks/vegetation/cave_vines_block.rs` | 263 | `head_conversion_preserves_berries`, `grown_head_rolls_berries_independently`, `clone_item_is_glow_berries` | 0 |
| `steel-core/src/behavior/blocks/vegetation/cave_vines_plant_block.rs` | 190 | `body_conversion_preserves_berries`, `clone_item_is_glow_berries` | 0 |
| `steel-core/src/behavior/blocks/vegetation/chorus_flower_block.rs` | 428 | `grows_upward_from_chorus_support`, `mature_flower_dies_when_it_cannot_grow`, `creates_branches_when_upward_growth_is_obstructed`, `unsupported_flower_breaks_on_scheduled_tick`, `schedules_survival_tick_when_non_up_neighbor_changes` | 0 |
| `steel-core/src/behavior/blocks/vegetation/chorus_plant_block.rs` | 248 | `update_shape_tracks_vertical_connections`, `update_shape_schedules_tick_when_unsupported`, `is_never_pathfindable` | 0 |
| `steel-core/src/behavior/blocks/vegetation/cocoa_block.rs` | 157 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/coral_block.rs` | 105 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/coral_fan_block.rs` | 91 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/coral_plant_block.rs` | 169 | `dry_coral_plant_update_shape_schedules_die_tick`, `waterlogged_coral_plant_update_shape_schedules_water_not_die_tick` | 0 |
| `steel-core/src/behavior/blocks/vegetation/coral_wall_fan_block.rs` | 128 | `dead_wall_fan_state_preserves_facing` | 0 |
| `steel-core/src/behavior/blocks/vegetation/crop_block.rs` | 337 | `crop_survival_requires_vanilla_minimum_light`, `ravager_crop_breaking_requires_mob_griefing` | 0 |
| `steel-core/src/behavior/blocks/vegetation/dirt_path_block.rs` | 78 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/double_plant_block.rs` | 164 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/dry_vegetation_block.rs` | 47 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/eyeblossom_block.rs` | 119 | `eyeblossom_requires_vegetation_support` | 0 |
| `steel-core/src/behavior/blocks/vegetation/farmland_block.rs` | 224 | `fall_trampling_requires_random_below_fall_distance_minus_half`, `non_player_living_entities_need_mob_griefing_to_trample`, `small_or_non_living_entities_do_not_trample` | 0 |
| `steel-core/src/behavior/blocks/vegetation/firefly_bush_block.rs` | 86 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/flower_bed_block.rs` | 102 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/flower_block.rs` | 52 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/glow_lichen_block.rs` | 222 | `spreads_to_an_open_face_at_the_same_position_first`, `spreads_in_the_same_plane_when_the_source_has_no_open_face`, `wraps_around_a_supporting_block_when_same_plane_is_blocked`, `spreading_into_source_water_creates_waterlogged_lichen`, `bonemeal_requires_a_vacant_spread_direction` | 0 |
| `steel-core/src/behavior/blocks/vegetation/grass_block.rs` | 160 | `grass_block_updates_snowy_state` | 0 |
| `steel-core/src/behavior/blocks/vegetation/growing_plant_block.rs` | 21 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/growing_plant_body_block.rs` | 265 | `replacement_rejects_head_item_and_preserves_default_result` | 0 |
| `steel-core/src/behavior/blocks/vegetation/growing_plant_head_block.rs` | 317 | `connected_placement_uses_body_state` | 0 |
| `steel-core/src/behavior/blocks/vegetation/hanging_moss_block.rs` | 119 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/hanging_roots_block.rs` | 62 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/huge_mushroom_block.rs` | 145 | `placement_hides_only_faces_joined_to_the_same_mushroom_block`, `matching_neighbor_hides_the_joined_face`, `removing_a_neighbor_does_not_restore_a_hidden_face` | 0 |
| `steel-core/src/behavior/blocks/vegetation/kelp_block.rs` | 243 | `kelp_update_shape_schedules_water_tick`, `kelp_head_update_shape_schedules_break_tick_when_unsupported`, `kelp_head_converts_to_body_when_connected_above` | 0 |
| `steel-core/src/behavior/blocks/vegetation/kelp_plant_block.rs` | 216 | `kelp_plant_update_shape_schedules_water_tick`, `kelp_plant_update_shape_schedules_break_tick_when_unsupported`, `kelp_plant_converts_to_random_aged_head_when_open_above` | 0 |
| `steel-core/src/behavior/blocks/vegetation/leaf_litter_block.rs` | 59 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/leaves_block.rs` | 355 | `waterlogged_leaves_decay_into_water`, `distance_updates_from_decay_preventing_blocks`, `mangrove_leaves_register_bonemeal_behavior`, `mangrove_leaves_require_air_below_for_bonemeal` | 0 |
| `steel-core/src/behavior/blocks/vegetation/lily_pad_block.rs` | 77 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/mangrove_propagule_block.rs` | 471 | `new_hanging_propagule_starts_at_age_zero`, `placed_propagules_are_mature_and_preserve_source_water`, `hanging_growth_stops_at_age_four`, `bonemeal_targets_match_hanging_age_and_tree_height`, `planted_bonemeal_advances_stage_before_growing_tree`, `mature_planted_propagule_generates_mangrove_tree`, `unsupported_waterlogged_propagule_schedules_water_before_breaking` | 0 |
| `steel-core/src/behavior/blocks/vegetation/mangrove_roots_block.rs` | 157 | `placement_preserves_source_water`, `waterlogged_neighbor_update_schedules_water_tick`, `waterlogged_roots_contain_source_water` | 0 |
| `steel-core/src/behavior/blocks/vegetation/mod.rs` | 305 | `multiface_update_uses_supplied_neighbor_state_and_schedules_water_first` | 0 |
| `steel-core/src/behavior/blocks/vegetation/mossy_carpet_block.rs` | 279 | `updated_state_walls_up_against_the_supporting_block` | 0 |
| `steel-core/src/behavior/blocks/vegetation/multiface_block.rs` | 443 | `update_shape_uses_supplied_neighbor_state_and_schedules_water_first` | 0 |
| `steel-core/src/behavior/blocks/vegetation/mushroom_block.rs` | 272 | `mushroom_survival_uses_solid_render_support`, `mushroom_bonemeal_validation_checks_build_height` | 0 |
| `steel-core/src/behavior/blocks/vegetation/mycelium_block.rs` | 96 | `mycelium_block_updates_snowy_state` | 0 |
| `steel-core/src/behavior/blocks/vegetation/nether_fungus_block.rs` | 235 | `bonemeal_requires_matching_nylium_and_build_height`, `bonemeal_uses_vanilla_success_probability`, `bonemeal_grows_both_huge_fungus_variants` | 0 |
| `steel-core/src/behavior/blocks/vegetation/nether_roots_block.rs` | 56 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/nether_sprouts.rs` | 69 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/nether_vines.rs` | 13 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/nether_wart.rs` | 101 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/pitcher_crop.rs` | 277 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/pointed_dripstone_block.rs` | 1262 | `upward_tip_uses_stalagmite_fall_damage`, `non_tip_uses_default_fall_damage`, `downward_tip_uses_default_fall_damage`, `only_fast_tridents_break_speleothems`, `stalactite_drip_converts_mud_to_clay`, `stalactite_drip_fills_empty_cauldron_with_water`, `stalactite_drip_increments_layered_water_cauldron`, `stalactite_drip_does_not_overflow_full_water_cauldron`, `stalactite_drip_fills_empty_cauldron_with_lava`, `stalactite_drip_does_not_fill_lava_cauldron`, `stalactite_drip_skips_drip_when_random_value_exceeds_water_probability`, `stalactite_drip_skips_drip_when_random_value_exceeds_lava_probability` | 0 |
| `steel-core/src/behavior/blocks/vegetation/potato.rs` | 79 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/pumpkin_block.rs` | 218 | `carving_with_shears_drops_seeds_and_damages_shears` | 0 |
| `steel-core/src/behavior/blocks/vegetation/rooted_dirt_block.rs` | 108 | `rooted_dirt_bonemeal_rejects_bottom_build_height`, `rooted_dirt_bonemeal_accepts_in_bounds_air_below` | 0 |
| `steel-core/src/behavior/blocks/vegetation/sapling_block.rs` | 187 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/sculk_vein_block.rs` | 212 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/sea_pickle_block.rs` | 230 | `sea_pickle_checks_survival_before_scheduling_water` | 0 |
| `steel-core/src/behavior/blocks/vegetation/seagrass_block.rs` | 205 | `seagrass_update_shape_breaks_without_support`, `seagrass_update_shape_schedules_water_when_it_survives`, `seagrass_bonemeal_requires_water_block_above` | 0 |
| `steel-core/src/behavior/blocks/vegetation/segmentable_block.rs` | 159 | `placement_uses_horizontal_direction_and_preserves_existing_facing`, `replacement_matches_vanilla_segmentable_and_default_rules`, `only_flower_beds_are_bonemealable` | 0 |
| `steel-core/src/behavior/blocks/vegetation/shelf_mushroom_block.rs` | 152 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/short_dry_grass_block.rs` | 81 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/small_dripleaf_block.rs` | 187 | `small_dripleaf_schedules_water_before_double_plant_survival` | 0 |
| `steel-core/src/behavior/blocks/vegetation/snowy_block.rs` | 238 | `placement_resolves_snowy_from_above_block`, `upward_neighbor_update_recalculates_snowy`, `lateral_and_downward_neighbor_updates_keep_existing_state` | 0 |
| `steel-core/src/behavior/blocks/vegetation/spore_blossom_block.rs` | 56 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/spreading_grass_block.rs` | 81 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/stem_block.rs` | 511 | `survival_uses_extracted_support_without_a_light_requirement`, `random_growth_requires_light_nine_and_advances_one_age`, `mature_stems_place_their_own_fruit_and_attached_family`, `mature_growth_requires_air_and_fruit_support_and_does_not_scan`, `bonemeal_bounds_age_and_runs_the_mature_tick_with_the_same_rng`, `clone_items_match_each_stem_family`, `extracted_constructor_mappings_cover_both_families` | 0 |
| `steel-core/src/behavior/blocks/vegetation/sugar_cane.rs` | 198 | `sugar_cane_update_shape_schedules_break_tick_when_unsupported` | 0 |
| `steel-core/src/behavior/blocks/vegetation/sweet_berry_bush.rs` | 410 | `contact_damage_uses_old_position_for_server_authored_entities`, `contact_damage_uses_known_movement_for_client_authored_entities`, `contact_damage_is_age_gated`, `contact_damage_requires_threshold_movement`, `foxes_bees_and_non_living_entities_are_immune_to_sweet_berry_bush_effects` | 0 |
| `steel-core/src/behavior/blocks/vegetation/tall_dry_grass_block.rs` | 94 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/tall_flower_block.rs` | 90 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/tall_grass_block.rs` | 136 | `tall_grass_bonemeal_rejects_top_build_height` | 0 |
| `steel-core/src/behavior/blocks/vegetation/tall_seagrass_block.rs` | 214 | `tall_seagrass_lower_breaks_when_upper_half_is_missing`, `tall_seagrass_upper_breaks_when_lower_half_is_missing`, `tall_seagrass_lower_survives_in_falling_full_water` | 0 |
| `steel-core/src/behavior/blocks/vegetation/torchflower.rs` | 114 | `torchflower_age_two_becomes_flower_block` | 0 |
| `steel-core/src/behavior/blocks/vegetation/turtle_egg_block.rs` | 413 | `eggs_crack_twice_then_hatch_on_sand`, `eggs_do_not_advance_off_sand` | 0 |
| `steel-core/src/behavior/blocks/vegetation/twisting_vines_block.rs` | 155 | `bonemeal_target_requires_open_growth_position`, `connected_head_converts_to_body` | 0 |
| `steel-core/src/behavior/blocks/vegetation/twisting_vines_plant_block.rs` | 137 | `bonemeal_target_follows_connected_head` | 0 |
| `steel-core/src/behavior/blocks/vegetation/vegetation_block.rs` | 45 | — | 0 |
| `steel-core/src/behavior/blocks/vegetation/vine_block.rs` | 439 | `face_count_matches_vanilla_replacement_limit`, `shape_update_removes_faceless_vine` | 0 |
| `steel-core/src/behavior/blocks/vegetation/weeping_vines_block.rs` | 154 | `bonemeal_target_requires_open_growth_position`, `connected_head_converts_to_body` | 0 |
| `steel-core/src/behavior/blocks/vegetation/weeping_vines_plant_block.rs` | 136 | `bonemeal_target_follows_connected_head` | 0 |
| `steel-core/src/behavior/blocks/vegetation/wither_rose_block.rs` | 76 | — | 0 |
| `steel-core/src/behavior/consume_effect.rs` | 90 | — | 0 |
| `steel-core/src/behavior/context.rs` | 1060 | `item_use_side_effects_apply_to_all_success_variants`, `player_hand_source_reads_current_components_and_mutates_the_hand`, `replacement_dispatch_does_not_hold_the_inventory_lock`, `direct_source_mutates_the_callers_exact_stack`, `at_changes_geometry_and_retains_the_direct_source`, `directional_context_uses_vanilla_direction_order`, `singular_look_direction_is_not_reordered_around_clicked_face` | 0 |
| `steel-core/src/behavior/fluid.rs` | 74 | — | 0 |
| `steel-core/src/behavior/item.rs` | 896 | `potion_items_default_to_their_vanilla_base_potion`, `items_without_an_override_default_to_a_plain_stack`, `honey_bottle_stack_keeps_remaining_bottles_and_hands_off_the_remainder`, `instant_consumable_with_use_remainder_finishes_without_deadlocking`, `eating_food_applies_its_nutrition_and_saturation`, `consuming_an_item_awards_the_item_used_stat`, `eating_a_suspicious_stew_applies_its_component_effects`, `drinking_an_ominous_bottle_applies_bad_omen_with_its_amplifier` | 0 |
| `steel-core/src/behavior/item_utils.rs` | 54 | — | 0 |
| `steel-core/src/behavior/items/air.rs` | 21 | — | 0 |
| `steel-core/src/behavior/items/armor_stand.rs` | 312 | `placing_an_armor_stand_consumes_one_item`, `placement_snaps_the_stand_to_the_player_facing`, `placement_is_refused_under_a_block`, `placement_is_refused_into_a_solid_block`, `placement_copies_the_stack_custom_name` | 0 |
| `steel-core/src/behavior/items/arrow.rs` | 90 | — | 0 |
| `steel-core/src/behavior/items/banner_item.rs` | 166 | `banner_places_standing_block_on_top`, `banner_places_wall_block_on_the_side` | 0 |
| `steel-core/src/behavior/items/block_item.rs` | 272 | — | 0 |
| `steel-core/src/behavior/items/block_transformer/tests.rs` | 261 | `rooted_dirt_drops_loot_from_each_clicked_face_even_with_a_block_above`, `copper_door_transforms_both_halves_without_restoring_the_old_neighbor_shape`, `copper_chest_transforms_both_halves_and_preserves_independent_properties` | 0 |
| `steel-core/src/behavior/items/block_transformer.rs` | 205 | — | 0 |
| `steel-core/src/behavior/items/boat.rs` | 112 | — | 0 |
| `steel-core/src/behavior/items/bonemeal.rs` | 207 | — | 0 |
| `steel-core/src/behavior/items/books.rs` | 132 | `writable_book_use_succeeds_without_consuming_the_stack`, `written_book_use_succeeds_without_consuming_the_stack` | 0 |
| `steel-core/src/behavior/items/bottle.rs` | 197 | `fills_from_a_water_source_and_leaves_the_source`, `non_water_blocks_and_misses_do_not_fill` | 0 |
| `steel-core/src/behavior/items/bow.rs` | 227 | `the_power_curve_matches_vanilla_pull_progress`, `creative_players_get_creative_only_pickup`, `all_three_arrow_items_are_accepted` | 0 |
| `steel-core/src/behavior/items/brush.rs` | 117 | — | 0 |
| `steel-core/src/behavior/items/bucket.rs` | 450 | `filled_water_bucket_targets_non_waterlogged_liquid_container_in_place`, `adventure_buckets_require_can_place_on_and_fail_when_denied` | 0 |
| `steel-core/src/behavior/items/bundle.rs` | 550 | `left_click_with_a_cursor_bundle_absorbs_the_clicked_slot`, `right_click_with_a_cursor_bundle_refills_an_empty_slot`, `shulker_boxes_cannot_be_stuffed_into_a_bundle`, `dropping_a_bundled_stack_spawns_an_item_entity`, `destroying_a_bundle_item_entity_spills_every_stored_stack`, `clicking_a_bundle_in_a_slot_pulls_the_cursor_stack_into_it`, `right_clicking_a_bundle_in_a_slot_moves_a_stack_to_the_cursor` | 0 |
| `steel-core/src/behavior/items/compass.rs` | 247 | `compass_binds_to_lodestone_and_invalidates_when_removed`, `compass_invalidates_target_outside_world_bounds` | 0 |
| `steel-core/src/behavior/items/copper_chest_events.rs` | 93 | `connected_chest_pos_matches_vanilla_left_and_right_offsets`, `connected_chest_pos_ignores_single_chests` | 0 |
| `steel-core/src/behavior/items/crossbow.rs` | 470 | `charge_duration_shrinks_with_quick_charge`, `multishot_loads_three_projectiles`, `multishot_spread_is_symmetric`, `charged_projectiles_round_trip_through_the_component`, `firework_rockets_use_the_slower_shot_velocity`, `charge_duration_never_drops_below_one_tick` | 0 |
| `steel-core/src/behavior/items/cushion.rs` | 350 | `placing_a_white_cushion_spawns_it_on_the_support`, `the_entity_takes_the_colour_of_the_placed_item`, `a_cushion_is_not_placed_without_solid_support`, `a_cushion_is_not_placed_inside_a_block`, `a_second_cushion_is_refused_on_an_existing_one`, `placement_copies_the_stack_custom_name` | 0 |
| `steel-core/src/behavior/items/debug_stick.rs` | 521 | `debug_stick_refuses_non_operators_and_leaves_the_block_alone`, `debug_stick_never_breaks_blocks`, `debug_stick_cycle_wraps_in_both_directions`, `debug_stick_cycle_defaults_to_the_first_or_last_value`, `debug_stick_selects_a_property_then_cycles_its_value`, `debug_stick_cycles_backwards_while_crouching`, `debug_stick_reports_blocks_without_properties`, `debug_stick_orders_boolean_values_like_vanilla`, `debug_stick_state_remembers_and_replaces_selections` | 0 |
| `steel-core/src/behavior/items/default.rs` | 11 | — | 0 |
| `steel-core/src/behavior/items/disc_fragment.rs` | 63 | `the_description_key_is_the_item_key_plus_desc` | 0 |
| `steel-core/src/behavior/items/dye.rs` | 137 | `dye_sign_applicator_updates_color_and_respects_existing_color` | 0 |
| `steel-core/src/behavior/items/dynamic_name.rs` | 197 | `every_mc26_dynamic_item_name_override_uses_stack_components`, `air_name_ignores_stack_item_name_patch`, `dynamic_name_prefix_ignores_stack_item_name_patch` | 0 |
| `steel-core/src/behavior/items/egg.rs` | 46 | — | 0 |
| `steel-core/src/behavior/items/empty_map.rs` | 171 | `empty_map_becomes_a_filled_map_in_the_same_hand`, `stacked_empty_maps_keep_the_remainder_and_insert_the_new_map` | 0 |
| `steel-core/src/behavior/items/end_crystal.rs` | 242 | `placing_on_obsidian_spawns_a_crystal_without_a_base_plate`, `bedrock_is_also_a_valid_base`, `other_blocks_reject_the_crystal`, `a_blocked_block_above_rejects_the_crystal` | 0 |
| `steel-core/src/behavior/items/ender_eye.rs` | 387 | `end_portal_pattern_matches_player_built_inward_layout`, `end_portal_pattern_rejects_wrong_side_facing`, `end_portal_pattern_uses_vanilla_front_top_left_offset` | 0 |
| `steel-core/src/behavior/items/ender_pearl.rs` | 49 | — | 0 |
| `steel-core/src/behavior/items/experience_bottle.rs` | 117 | `using_an_experience_bottle_throws_it_and_consumes_one` | 0 |
| `steel-core/src/behavior/items/filled_map.rs` | 493 | `water_brightness_uses_vanilla_thresholds`, `land_brightness_uses_vanilla_thresholds`, `palette_picks_the_most_frequent_color`, `palette_ties_keep_first_seen_order`, `palette_ignores_never_seen_ids` | 0 |
| `steel-core/src/behavior/items/firework_rocket.rs` | 148 | `placement_position_offsets_outside_clicked_face` | 0 |
| `steel-core/src/behavior/items/fishing_rod.rs` | 271 | `retrieving_grounded_hook_does_not_relock_inventory`, `use_emits_vanilla_interact_vibrations`, `casting_fishing_rod_spawns_and_links_hook` | 0 |
| `steel-core/src/behavior/items/flint_and_steel.rs` | 235 | `can_light_rejects_waterlogged_campfires_and_candles`, `can_light_accepts_unlit_candle_cakes` | 0 |
| `steel-core/src/behavior/items/food_on_a_stick.rs` | 65 | — | 0 |
| `steel-core/src/behavior/items/game_master_block_item.rs` | 138 | `command_blocks_are_operator_only` | 0 |
| `steel-core/src/behavior/items/hanging_entity_item.rs` | 259 | `placing_a_painting_consumes_one_item`, `paintings_are_not_placed_on_vertical_faces`, `paintings_are_not_placed_without_a_wall`, `painting_placement_copies_the_stack_custom_name` | 0 |
| `steel-core/src/behavior/items/hanging_sign_item.rs` | 123 | — | 0 |
| `steel-core/src/behavior/items/honeycomb.rs` | 106 | — | 0 |
| `steel-core/src/behavior/items/ink_sac.rs` | 79 | — | 0 |
| `steel-core/src/behavior/items/instrument_item.rs` | 152 | `using_a_goat_horn_starts_its_instrument_cooldown` | 0 |
| `steel-core/src/behavior/items/item_frame.rs` | 327 | `placing_a_frame_spawns_it_on_the_clicked_face_and_consumes_one`, `glow_frames_use_the_glow_entity_type`, `frames_are_not_placed_without_solid_support`, `frames_are_not_placed_into_occupied_space`, `frames_copy_the_stack_custom_name` | 0 |
| `steel-core/src/behavior/items/knowledge_book.rs` | 120 | `knowledge_book_resolves_recipe_results`, `knowledge_book_component_keeps_its_keys` | 0 |
| `steel-core/src/behavior/items/lead.rs` | 80 | — | 0 |
| `steel-core/src/behavior/items/mace.rs` | 226 | — | 0 |
| `steel-core/src/behavior/items/minecart.rs` | 426 | `non_rails_fail_without_consumption_spawning_or_statistics`, `all_minecart_variants_place_consume_and_spawn_on_rails`, `placement_uses_the_clicked_rail_height_position_and_hand`, `placing_the_last_item_transfers_its_custom_name_before_consumption` | 0 |
| `steel-core/src/behavior/items/mob_bucket.rs` | 129 | — | 0 |
| `steel-core/src/behavior/items/mod.rs` | 130 | — | 0 |
| `steel-core/src/behavior/items/name_tag.rs` | 36 | — | 0 |
| `steel-core/src/behavior/items/place_on_water_block_item.rs` | 42 | — | 0 |
| `steel-core/src/behavior/items/player_head.rs` | 78 | — | 0 |
| `steel-core/src/behavior/items/potion.rs` | 141 | — | 0 |
| `steel-core/src/behavior/items/scaffolding_block_item.rs` | 381 | `routing_matches_normal_secondary_and_inside_direction_rules`, `horizontal_routing_allows_seventh_position_but_not_eighth`, `distance_seven_extension_places_then_waits_for_stability_tick`, `unsupported_direct_target_is_rejected_without_consuming_item`, `supported_source_water_placement_is_waterlogged` | 0 |
| `steel-core/src/behavior/items/shears.rs` | 324 | `use_on_crops_growing_head_and_emits_vanilla_effects`, `use_on_max_age_head_passes_without_effects` | 0 |
| `steel-core/src/behavior/items/shield.rs` | 32 | — | 0 |
| `steel-core/src/behavior/items/smithing_template.rs` | 188 | `armor_trims_expose_the_vanilla_slot_lists`, `the_netherite_upgrade_lists_equipment_and_tools`, `tooltip_keys_match_the_vanilla_translation_ids`, `every_template_item_resolves_to_this_class` | 0 |
| `steel-core/src/behavior/items/snowball.rs` | 46 | — | 0 |
| `steel-core/src/behavior/items/solid_bucket_item.rs` | 50 | — | 0 |
| `steel-core/src/behavior/items/spawn_egg.rs` | 250 | — | 0 |
| `steel-core/src/behavior/items/spyglass.rs` | 68 | — | 0 |
| `steel-core/src/behavior/items/standing_and_wall_block_item.rs` | 164 | — | 0 |
| `steel-core/src/behavior/items/throw_projectile.rs` | 77 | — | 0 |
| `steel-core/src/behavior/items/throwable_potion.rs` | 104 | — | 0 |
| `steel-core/src/behavior/items/tipped_arrow.rs` | 31 | — | 0 |
| `steel-core/src/behavior/items/trident.rs` | 268 | `a_throw_needs_ten_ticks_of_charge`, `thrown_tridents_use_the_vanilla_shoot_power`, `creative_throws_are_creative_only_pickups` | 0 |
| `steel-core/src/behavior/items/wind_charge.rs` | 87 | — | 0 |
| `steel-core/src/behavior/mob_effect.rs` | 79 | — | 0 |
| `steel-core/src/behavior/mod.rs` | 333 | — | 0 |
| `steel-core/src/block_entity/block_state_nbt.rs` | 6 | — | 0 |
| `steel-core/src/block_entity/components.rs` | 60 | — | 0 |
| `steel-core/src/block_entity/entities/abstract_furnace.rs` | 592 | `furnace_consumes_one_fuel_and_finishes_at_recipe_cooking_time`, `blast_furnace_uses_blasting_time_and_half_fuel_duration`, `furnace_sided_inventory_matches_vanilla_faces_and_bucket_rule` | 0 |
| `steel-core/src/block_entity/entities/banner.rs` | 132 | `banner_starts_without_layers` | 0 |
| `steel-core/src/block_entity/entities/barrel.rs` | 274 | `set_item_limits_stack_to_vanilla_container_maximum`, `pre_remove_preserves_slots_for_existing_menu_references` | 0 |
| `steel-core/src/block_entity/entities/beacon.rs` | 521 | — | 0 |
| `steel-core/src/block_entity/entities/beehive.rs` | 480 | `release_offsets_match_vanilla_faces`, `occupants_are_capped_at_three`, `nectarless_bees_wait_six_hundred_ticks` | 0 |
| `steel-core/src/block_entity/entities/bell.rs` | 287 | `ring_event_starts_shaking_in_the_supplied_direction`, `unrelated_block_event_is_rejected` | 0 |
| `steel-core/src/block_entity/entities/brewing_stand.rs` | 681 | `brewing_stand_has_five_slots`, `the_fuel_slot_accepts_a_stack_and_the_sides_reach_it`, `face_slots_match_vanilla`, `state_defaults_to_idle`, `brew_time_and_fuel_round_trip`, `the_ingredient_slot_starts_empty`, `one_brew_takes_vanilla_400_ticks`, `bottles_are_not_ingredients_and_nether_wart_is`, `brewing_requires_fuel_and_a_recipe`, `fuel_uses_come_from_the_brewing_fuel_component` | 0 |
| `steel-core/src/block_entity/entities/brushable.rs` | 529 | `save_loot_table_excludes_item_and_hit_direction`, `save_loot_table_omits_zero_seed`, `save_item_only_when_no_loot_table`, `load_loot_table_discards_item`, `load_item_when_no_loot_table`, `hit_direction_is_byte_on_update_tag_not_disk` | 0 |
| `steel-core/src/block_entity/entities/campfire.rs` | 283 | `campfire_round_trips_all_four_progress_tracks_with_items` | 0 |
| `steel-core/src/block_entity/entities/chest.rs` | 474 | `chest_holds_twenty_seven_slots`, `a_structure_chest_keeps_its_table_until_it_is_opened`, `a_chest_item_with_container_loot_carries_the_table`, `a_structure_chest_rolls_once_only`, `lid_opens_while_viewed_and_closes_after_the_latch`, `lid_angle_lerps_by_a_tenth_per_tick` | 0 |
| `steel-core/src/block_entity/entities/chiseled_bookshelf.rs` | 555 | `bookshelf_book_tag_and_slot_capacity_control_container_capability`, `insertion_and_removal_update_occupancy_and_last_slot_storage`, `extraction_requires_space_somewhere_in_destination`, `persistence_and_container_component_preserve_all_slots_and_last_slot`, `component_application_replaces_all_six_slots_without_changing_last_slot` | 0 |
| `steel-core/src/block_entity/entities/command_block.rs` | 649 | `a_new_block_is_empty_and_unpowered`, `setting_the_command_clears_the_success_count`, `disabling_output_tracking_drops_the_last_output`, `the_packet_ordinals_match_vanilla`, `the_mode_comes_from_the_block_identity` | 0 |
| `steel-core/src/block_entity/entities/comparator.rs` | 127 | `output_signal_round_trips_with_vanilla_nbt_key`, `output_signal_loads_from_any_numeric_tag`, `missing_output_signal_loads_vanilla_default` | 0 |
| `steel-core/src/block_entity/entities/conduit.rs` | 325 | — | 0 |
| `steel-core/src/block_entity/entities/copper_golem_statue.rs` | 127 | — | 0 |
| `steel-core/src/block_entity/entities/crafter.rs` | 608 | `crafter_has_nine_slots`, `disabled_slot_mask_roundtrips`, `craft_without_a_recipe_does_nothing`, `a_two_plank_column_matches_the_stick_recipe`, `disabled_slots_are_ignored_by_the_recipe_match`, `begin_crafting_sets_the_counter_once_and_requires_a_recipe`, `the_front_face_follows_the_orientation_property`, `the_trigger_counter_starts_at_zero` | 0 |
| `steel-core/src/block_entity/entities/creaking_heart.rs` | 291 | `heart_starts_unlinked` | 0 |
| `steel-core/src/block_entity/entities/daylight_detector.rs` | 120 | `detector_updates_only_on_vanilla_twenty_game_tick_cadence` | 0 |
| `steel-core/src/block_entity/entities/decorated_pot.rs` | 159 | `decorated_pot_starts_empty`, `set_sherd_roundtrips_through_side_accessor` | 0 |
| `steel-core/src/block_entity/entities/dispenser.rs` | 265 | `dispenser_has_nine_slots`, `selected_slot_defaults_to_negative_one` | 0 |
| `steel-core/src/block_entity/entities/enchanting_table.rs` | 170 | `applying_item_components_consumes_the_custom_name_and_stores_the_rest`, `applying_item_components_skips_placement_only_components`, `applying_an_unnamed_item_clears_a_previous_custom_name` | 0 |
| `steel-core/src/block_entity/entities/end_gateway.rs` | 281 | `end_gateway_saves_vanilla_nbt_keys`, `full_metadata_includes_type_and_position_after_additional_data`, `end_gateway_loads_vanilla_nbt_keys`, `end_gateway_rejects_exit_outside_spawnable_bounds` | 0 |
| `steel-core/src/block_entity/entities/end_portal.rs` | 45 | — | 0 |
| `steel-core/src/block_entity/entities/ender_chest.rs` | 46 | — | 0 |
| `steel-core/src/block_entity/entities/hopper.rs` | 815 | `shulker_rejects_nested_boxes_and_hopper_continues_to_next_acceptable_slot`, `hopper_has_five_slots`, `set_item_limits_stack_to_vanilla_container_maximum`, `transfer_cooldown_expires_on_the_eighth_tick_not_the_ninth`, `cooldown_starts_at_zero_and_round_trips`, `facing_is_read_from_the_block_state`, `add_into_slots_only_touches_slots_exposed_to_the_face`, `add_into_slots_respects_the_destination_max_stack_size`, `suck_column_matches_the_vanilla_shape_bounds`, `a_full_hopper_does_not_claim_an_item_entity`, `cooldown_is_persisted_in_nbt` | 0 |
| `steel-core/src/block_entity/entities/jigsaw_block.rs` | 231 | — | 0 |
| `steel-core/src/block_entity/entities/jukebox.rs` | 524 | `saved_song_resumes_before_but_not_at_its_vanilla_finish_tick`, `item_block_entity_data_merges_with_existing_record_before_loading`, `direct_song_uses_vanilla_unknown_registry_level_event_data`, `value_input_long_uses_each_numeric_tag_long_value` | 0 |
| `steel-core/src/block_entity/entities/lectern.rs` | 191 | `lectern_holds_books_and_tracks_page` | 0 |
| `steel-core/src/block_entity/entities/mod.rs` | 102 | — | 0 |
| `steel-core/src/block_entity/entities/piston_moving.rs` | 891 | `moving_state_and_progress_round_trip_with_vanilla_keys`, `collided_entity_filter_matches_vanilla_player_and_spectator_rules`, `movement_area_matches_vanilla_directional_sweep`, `piston_entity_move_can_reenter_moving_block_collision`, `final_tick_marks_a_detached_moving_entity_removed`, `stale_final_tick_cannot_remove_or_finish_a_replacement` | 0 |
| `steel-core/src/block_entity/entities/potent_sulfur.rs` | 353 | — | 0 |
| `steel-core/src/block_entity/entities/potion_brewing.rs` | 222 | `water_plus_nether_wart_makes_awkward`, `redstone_extends_swiftness`, `unknown_combination_has_no_recipe`, `every_recipe_targets_a_registered_potion`, `blaze_powder_acts_as_both_fuel_and_strength_ingredient`, `potions_are_brewable` | 0 |
| `steel-core/src/block_entity/entities/sculk_catalyst.rs` | 592 | `shrieker_tracks_warning_level`, `the_warning_level_saturates_at_the_vanilla_maximum`, `a_saved_cooldown_survives_a_round_trip` | 0 |
| `steel-core/src/block_entity/entities/sculk_sensor.rs` | 519 | `sensor_records_frequency`, `vibration_frequencies_match_vanilla_26_3_table`, `distance_attenuation_matches_vanilla_formula` | 0 |
| `steel-core/src/block_entity/entities/shelf.rs` | 157 | `shelf_has_three_slots` | 0 |
| `steel-core/src/block_entity/entities/shulker_box.rs` | 483 | — | 0 |
| `steel-core/src/block_entity/entities/sign.rs` | 544 | `plain_sign_lines_save_as_a_string_list`, `mixed_sign_lines_round_trip_through_the_component_codec`, `styled_plain_text_is_editable_but_translations_are_not`, `placed_sign_takes_text_and_wax_from_item_components`, `sign_tick_releases_state_before_player_lookup_and_editor_clear`, `set_color_updates_color_and_reports_change` | 0 |
| `steel-core/src/block_entity/entities/skull.rs` | 138 | — | 0 |
| `steel-core/src/block_entity/entities/spawner/base_spawner.rs` | 676 | — | 0 |
| `steel-core/src/block_entity/entities/spawner/mod.rs` | 169 | — | 0 |
| `steel-core/src/block_entity/entities/spawner/spawn_data.rs` | 268 | — | 0 |
| `steel-core/src/block_entity/entities/spawner/tests.rs` | 361 | `custom_spawner_light_ranges_decode_vanilla_interval_forms`, `custom_spawner_light_ranges_default_malformed_but_reject_out_of_bounds`, `unsupported_spawn_data_delays_and_reselects_without_inserting`, `weighted_selection_skips_zero_weights_and_preserves_order`, `nbt_round_trip_keeps_short_spawner_fields_and_spawn_data`, `update_tag_omits_spawn_potentials_for_client_sync`, `invalid_optional_spawn_data_fields_keep_the_entity`, `load_keeps_active_spawn_data_when_new_data_does_not_decode`, `set_entity_id_also_changes_the_shared_spawn_potential`, `set_entity_data_replaces_the_id_and_merges_the_rest`, `set_entity_id_normalizes_the_active_spawn_data` | 0 |
| `steel-core/src/block_entity/entities/structure_block.rs` | 474 | — | 0 |
| `steel-core/src/block_entity/entities/test_block.rs` | 242 | — | 0 |
| `steel-core/src/block_entity/entities/test_instance_block.rs` | 169 | — | 0 |
| `steel-core/src/block_entity/entities/trial_spawner.rs` | 486 | `trial_spawner_starts_unconfigured` | 0 |
| `steel-core/src/block_entity/entities/unimplemented.rs` | 137 | `full_metadata_replaces_stale_unimplemented_metadata`, `constructor_rejects_a_type_state_mismatch` | 0 |
| `steel-core/src/block_entity/entities/vault.rs` | 430 | `vault_starts_unconfigured`, `vault_matches_normal_and_ominous_keys` | 0 |
| `steel-core/src/block_entity/mod.rs` | 635 | — | 0 |
| `steel-core/src/block_entity/randomizable_container.rs` | 316 | `an_empty_container_has_no_table`, `a_container_keeps_the_seed_from_the_template`, `unpacking_clears_the_table_so_it_rolls_once`, `filling_stops_when_the_container_is_full`, `filling_never_overwrites_a_present_item`, `a_real_table_fills_the_container_deterministically`, `empty_loot_never_touches_the_slots` | 0 |
| `steel-core/src/block_entity/registry.rs` | 425 | — | 0 |
| `steel-core/src/block_entity/storage.rs` | 791 | `readding_the_same_entity_preserves_ownership_and_clears_the_marker`, `stale_removed_cleanup_cannot_delete_a_same_arc_revival`, `insert_if_absent_preserves_the_concurrent_owner`, `lifecycle_callbacks_are_reentrant_and_keep_transition_order`, `repeated_set_removed_calls_remain_observable`, `detached_dispatcher_preserves_same_arc_revival_order`, `cached_state_callback_is_staged_after_storage_commit` | 0 |
| `steel-core/src/block_state_provider/mod.rs` | 24 | — | 0 |
| `steel-core/src/block_state_provider/noise_cache.rs` | 104 | — | 0 |
| `steel-core/src/block_state_provider/predicates.rs` | 123 | — | 0 |
| `steel-core/src/block_state_provider/providers.rs` | 368 | — | 0 |
| `steel-core/src/block_state_provider/tests.rs` | 267 | `empty_random_block_providers_preserve_the_block_without_consuming_randomness`, `matching_nested_rule_without_a_result_continues_to_the_next_rule`, `rotated_provider_preserves_worldgen_rng_order_before_sampling_its_source`, `shipped_transformer_providers_never_consume_randomness`, `cached_threshold_noise_preserves_states_and_random_draws` | 0 |
| `steel-core/src/bootstrap.rs` | 31 | — | 0 |
| `steel-core/src/chunk/block_entity_listener.rs` | 105 | — | 0 |
| `steel-core/src/chunk/chunk_generation_task.rs` | 403 | — | 0 |
| `steel-core/src/chunk/chunk_holder.rs` | 1781 | `insert_chunk_publishes_the_authoritative_status`, `insert_chunk_rejects_full_status_for_proto_data`, `full_readiness_publication_waits_for_post_load_initialization`, `generated_full_status_is_accessible_when_readiness_is_published`, `late_lower_generation_completion_does_not_regress_published_status`, `status_waiter_observes_publication_after_subscribing`, `pending_status_waiters_wake_after_publication`, `full_registration_transfers_prepublication_block_and_fluid_ticks`, `client_deltas_require_confirmed_block_readiness`, `unpublished_status_claim_rolls_back_to_unloaded`, `unpublished_child_claim_rolls_back_to_published_parent`, `empty_claim_can_publish_a_higher_loaded_status`, `claimed_status_waiter_finishes_when_claim_is_abandoned`, `save_dependency_controls_ready_for_saving`, `revival_wins_over_an_in_flight_save_preparation`, `an_uncontested_save_preparation_keeps_its_snapshot`, `a_revived_holder_refuses_a_new_save_preparation` | 0 |
| `steel-core/src/chunk/chunk_map/generation_readiness.rs` | 628 | — | 0 |
| `steel-core/src/chunk/chunk_map/light_update_state.rs` | 201 | — | 0 |
| `steel-core/src/chunk/chunk_map/light_updates.rs` | 363 | — | 0 |
| `steel-core/src/chunk/chunk_map/mod.rs` | 1415 | — | 0 |
| `steel-core/src/chunk/chunk_map/persistence.rs` | 359 | — | 0 |
| `steel-core/src/chunk/chunk_map/player_tracking.rs` | 235 | `spectator_with_the_rule_off_holds_no_ticket`, `spectator_with_the_rule_on_loads_terrain`, `the_rule_never_affects_other_game_modes` | 0 |
| `steel-core/src/chunk/chunk_map/scheduled_ticks.rs` | 93 | — | 0 |
| `steel-core/src/chunk/chunk_map/tests/light_updates.rs` | 245 | `light_update_center_is_available_in_unloading_chunks`, `light_changed_marks_unloading_chunk_dirty`, `drained_light_updates_remain_unload_blocking_until_applied`, `light_update_unload_barrier_is_limited_to_cache_window`, `drained_light_update_window_remains_unload_blocking_until_applied`, `pending_light_updates_coalesce_changes_by_chunk_in_queue_order`, `pending_light_updates_prepend_blocked_drained_tasks`, `pending_light_updates_merge_requeued_task_with_existing_pending_task`, `pending_chunk_light_updates_sort_empty_section_changes_deterministically` | 0 |
| `steel-core/src/chunk/chunk_map/tests/mod.rs` | 293 | — | 0 |
| `steel-core/src/chunk/chunk_map/tests/persistence_unloads.rs` | 335 | `world_tick_spawns_dirty_unload_save_on_the_chunk_runtime`, `save_retry_marks_same_unloading_holder_dirty`, `revival_during_save_preparation_activates_the_holder_immediately`, `ticket_receipt_commits_while_the_holder_is_still_preparing_a_save`, `revival_during_save_preparation_keeps_the_generation_neighborhood_complete`, `final_full_chunk_unload_finalizes_chunk_owned_tick_queues`, `unloading_full_chunk_revival_keeps_chunk_owned_tick_queues`, `weak_revival_stays_dormant_until_the_same_holder_returns_to_full`, `gameplay_cache_scopes_observe_a_full_holder_revived_between_phases` | 0 |
| `steel-core/src/chunk/chunk_map/tests/player_tracking.rs` | 319 | `players_at_same_position_share_loading_and_simulation_sources`, `player_simulation_removal_applies_at_the_next_world_tick`, `light_changed_does_not_broadcast_unloading_full_chunk`, `broadcast_changed_chunks_does_not_defer_blocks_while_light_work_is_blocked`, `frozen_tick_broadcasts_block_changes_before_acknowledging_them`, `removing_player_invalidates_old_world_chunks_without_resetting_connection_pacing` | 0 |
| `steel-core/src/chunk/chunk_map/tests/request_cleanup.rs` | 107 | `dropping_pending_radius_request_preserves_pearl_simulation`, `panicking_radius_callback_releases_its_request`, `cancelling_handle_releases_once_and_preserves_another_lease` | 0 |
| `steel-core/src/chunk/chunk_map/tests/scheduled_ticks.rs` | 120 | `sparse_scheduler_collects_a_registered_chunk_owned_tick`, `block_callback_ticks_respect_the_block_fluid_phase_boundary`, `earlier_live_insertion_replaces_the_sparse_container_head`, `registered_full_chunks_use_active_order_for_equal_explicit_tick_heads` | 0 |
| `steel-core/src/chunk/chunk_map/tests/simulation_tickets.rs` | 217 | `restored_portal_ticket_initializes_both_levels_in_the_first_source_phase`, `unified_source_phase_updates_an_existing_holder_and_commits_its_receipt`, `simulation_changes_do_not_create_holders`, `removing_simulation_ticket_keeps_holder_with_load_only_ticket` | 0 |
| `steel-core/src/chunk/chunk_map/tests/tickets_generation_readiness.rs` | 527 | `ticket_changes_move_the_same_holder_only_at_boundary_commit`, `generation_priority_prefers_simulation_tickets`, `generation_priority_orders_simulation_by_simulation_level`, `generation_priority_orders_normal_by_load_level`, `cancelled_generation_task_keeps_cached_holders_pinned_for_in_flight_steps`, `cached_holder_rechecks_publication_and_generation_permission`, `full_publications_drive_block_and_entity_readiness_incrementally`, `first_block_readiness_anchors_pending_ticks_once`, `entity_tickability_requires_simulation_and_entity_readiness`, `full_load_activation_uses_packed_chunk_position_order` | 0 |
| `steel-core/src/chunk/chunk_map/tests/timed_simulation.rs` | 106 | `timed_simulation_expiration_follows_its_final_scheduled_tick` | 0 |
| `steel-core/src/chunk/chunk_pyramid.rs` | 447 | — | 0 |
| `steel-core/src/chunk/chunk_request.rs` | 377 | `dedupe_positions_preserves_first_occurrence_order`, `ready_chunk_still_waits_for_its_ticket_receipt_to_commit`, `a_cancelled_request_has_no_committed_tickets` | 0 |
| `steel-core/src/chunk/chunk_scheduler.rs` | 679 | `chunk_request_leases_share_one_ticket_and_keep_ordered_receipts`, `empty_batches_have_no_receipt_and_true_no_ops_are_barriers`, `unified_batch_and_receipt_commit_atomically`, `simulation_distance_reprojects_players_without_allocating_a_receipt` | 0 |
| `steel-core/src/chunk/chunk_status_tasks.rs` | 124 | — | 0 |
| `steel-core/src/chunk/chunk_ticket.rs` | 150 | `identity_is_type_pointer_and_level_not_timeout_state`, `timeout_aging_matches_java_long_behavior` | 0 |
| `steel-core/src/chunk/chunk_ticket_manager.rs` | 419 | `source_updates_are_deferred_and_idempotent`, `source_level_propagates_by_chebyshev_distance`, `overlapping_sources_keep_the_strongest_propagated_level`, `changing_a_source_level_updates_its_propagation`, `removing_a_source_reveals_overlapping_propagation`, `removing_the_last_source_reports_removed_levels`, `no_recalculation_occurs_without_source_changes`, `full_chunk_source_keeps_the_loading_moat`, `maximum_player_view_distance_fits_ticket_level`, `full_chunk_status_matches_vanilla_ticket_thresholds`, `ticket_level_for_status_allows_requested_status`, `non_full_ticket_level_maps_to_generation_status`, `ticket_level_for_status_creates_required_dependency_holders` | 0 |
| `steel-core/src/chunk/chunk_ticket_storage.rs` | 862 | `keep_dimension_active_follows_the_ticket_flag`, `duplicate_add_refreshes_timeout_without_adding_multiplicity`, `type_flags_control_projections_persistence_and_expiration`, `persistence_resolves_registered_type_and_skips_invalid_entries`, `persistence_defaults_ticks_and_duplicate_activation_refreshes_timeout`, `an_invalid_ticket_entry_is_skipped_without_dropping_the_valid_ones`, `a_document_that_is_not_a_ticket_list_fails_to_decode`, `timed_decrement_wraps_like_java_long` | 0 |
| `steel-core/src/chunk/chunk_tracker/mod.rs` | 347 | — | 0 |
| `steel-core/src/chunk/chunk_tracker/tests.rs` | 368 | `overlapping_sources_keep_the_strongest_propagated_level`, `repeated_updates_coalesce_to_the_original_level`, `load_only_level_cannot_alias_an_absent_simulation_source`, `weakening_a_source_removes_its_old_outer_levels`, `source_batch_order_does_not_change_the_result`, `incremental_source_updates_match_reference`, `deterministic_random_operations_match_reference`, `loading_updates_match_reference_across_generation_levels`, `moving_player_loading_coverage_preserves_overlaps_and_generation_moat` | 0 |
| `steel-core/src/chunk/data.rs` | 1623 | `transient_generation_state_survives_borrow_and_is_consumed_once`, `full_promotion_drops_leftover_transient_generation_state`, `postprocessing_offset_pack_unpack_matches_vanilla_layout`, `proto_scheduled_block_ticks_use_vanilla_zero_delay`, `full_promotion_retains_and_closes_proto_scheduled_tick_container`, `proto_chunk_preserves_distinct_air_states_in_empty_sections`, `pre_light_block_writes_defer_counts_until_light_initialization`, `proto_mutation_defers_lifecycle_and_promotion_revalidates_concrete_entities`, `proto_storage_preserves_removed_entries_until_full_promotion`, `pending_proto_entity_promotes_without_running_live_lifecycle`, `conditional_proto_marker_mutation_rejects_stale_worldgen_state`, `dummy_factory_outcomes_keep_proto_and_full_stage_semantics`, `stale_proto_factory_cannot_consume_a_replacement_marker` | 0 |
| `steel-core/src/chunk/full_chunk/game_event_tests.rs` | 371 | `active_block_entity_listener_uses_stored_selection_for_removal`, `full_activation_registers_listener_without_block_ticking_readiness`, `full_demotion_hides_listener_without_reordering_on_revival`, `retained_block_entity_state_does_not_reselect_listener` | 0 |
| `steel-core/src/chunk/full_chunk/mod.rs` | 1939 | — | 0 |
| `steel-core/src/chunk/full_chunk/tests.rs` | 648 | `promotion_installs_full_runtime_once`, `full_disk_construction_returns_initialized_runtime`, `lava_random_tick_classification_includes_block_and_fluid_hooks`, `block_random_positions_match_vanilla_lcg_layout`, `inactive_chunk_stages_lifecycle_callbacks_until_activation`, `extract_light_data_uses_chunk_owned_light_and_skylight_flag`, `empty_and_out_of_range_sections_return_air`, `draining_postprocessing_marks_full_chunk_dirty`, `conditional_block_set_rejects_a_stale_state`, `concurrent_consumers_cannot_both_claim_the_same_block_state`, `block_change_replaces_a_structurally_valid_unimplemented_entity_with_the_new_factory`, `breaking_an_unimplemented_entity_block_removes_its_unimplemented_entity`, `copper_chest_transformation_preserves_entity_identity`, `same_block_property_change_preserves_entity_data_and_updates_cached_state`, `shared_entity_type_does_not_imply_cross_block_preservation`, `insertion_rejects_an_entity_owned_by_another_chunk`, `insertion_below_world_does_not_alias_the_bottom_section`, `stale_no_entity_promotion_cannot_consume_a_replacement_marker`, `immediate_lookup_recovers_a_missing_implemented_entity` | 0 |
| `steel-core/src/chunk/full_chunk_readiness.rs` | 333 | `full_square_counts_track_block_and_entity_neighborhoods`, `invalid_extreme_coordinates_do_not_overflow` | 0 |
| `steel-core/src/chunk/gameplay_chunk_lookup_cache.rs` | 749 | `four_entry_cache_uses_most_recently_used_eviction`, `missing_holder_is_cached_within_scope`, `nested_scope_restores_outer_entries_and_releases_holders`, `dropping_scope_releases_entries_and_next_scope_starts_empty`, `foreign_owner_bypasses_active_cache`, `local_full_holder_cache_rechecks_full_permission`, `local_full_holder_cache_does_not_retain_unpublished_data`, `local_full_holder_cache_does_not_cross_scope_boundaries` | 0 |
| `steel-core/src/chunk/heightmap.rs` | 793 | `test_bits_per_value`, `test_get_index`, `heightmap_predicates_use_blocks_motion_and_fluid_state`, `initial_fill_update_tracks_only_matching_blocks`, `section_priming_preserves_heightmap_predicates_across_sections` | 0 |
| `steel-core/src/chunk/light/cache.rs` | 1339 | `cache_layout_matches_scalable_lux_chunk_indexing`, `cache_layout_classifies_inner_and_outer_cached_chunks`, `cache_layout_decodes_chunk_slots`, `cache_layout_iterates_full_setup_chunks_in_scalable_lux_order`, `cache_layout_adds_vertical_buffer_around_light_range`, `cache_layout_matches_scalable_lux_section_indexing`, `cache_layout_iterates_inner_chunk_light_section_slots`, `cache_layout_uses_scalable_lux_local_block_indices`, `cache_layout_maps_block_positions_to_cached_blocks`, `cache_layout_maps_cached_neighbors_across_section_edges`, `packed_light_block_pos_masks_to_scalable_lux_position_bits`, `cache_layout_encodes_scalable_lux_queue_position_window`, `cache_layout_maps_packed_positions_to_cached_blocks`, `slot_arrays_store_values_by_cached_slots`, `notification_cache_marks_one_block_light_neighborhood`, `notification_cache_rejects_partial_block_neighborhoods` | 0 |
| `steel-core/src/chunk/light/data_layer.rs` | 179 | — | 0 |
| `steel-core/src/chunk/light/mod.rs` | 456 | `data_layer_uses_vanilla_low_nibble_first_order`, `data_layer_preserves_homogeneous_non_zero_without_backing_bytes`, `light_section_range_matches_vanilla_padded_section_range`, `chunk_light_packet_omits_missing_and_internal_sections`, `chunk_light_packet_uses_empty_mask_for_visible_zero_sections`, `chunk_light_packet_expands_visible_homogeneous_non_zero_sections`, `chunk_light_packet_omits_sky_layer_when_dimension_has_no_skylight`, `changed_section_packet_preserves_ascending_light_section_order`, `chunk_light_data_reads_visible_block_and_sky_light`, `sections_collect_block_light_sources_in_scalable_lux_order`, `light_opacity_uses_vanilla_minimum_opacity`, `different_light_properties_match_vanilla_conditions`, `sky_light_sources_empty_chunk_extends_below_world`, `sky_light_sources_find_lowest_occluding_edge`, `sky_light_sources_update_adds_and_removes_occluding_edge`, `sky_light_sources_update_ignores_changes_below_current_source_edge` | 0 |
| `steel-core/src/chunk/light/packet.rs` | 138 | — | 0 |
| `steel-core/src/chunk/light/propagation.rs` | 1450 | `context_requires_block_layer`, `block_light_runner_publishes_visible_updates`, `block_light_changes_apply_empty_section_transitions`, `block_light_chunk_seeds_center_sources`, `block_light_chunk_pulls_neighbor_edge_levels`, `block_light_chunk_requires_center_chunk`, `block_light_changes_skip_missing_center_chunk`, `block_light_calculation_respects_occluding_faces` | 0 |
| `steel-core/src/chunk/light/queue.rs` | 1236 | `light_axis_direction_matches_scalable_lux_order`, `light_axis_direction_maps_to_steel_direction`, `light_axis_direction_opposites_flip_low_bit`, `light_direction_set_matches_scalable_lux_masks`, `light_direction_set_iterates_in_scalable_lux_order`, `light_queue_flags_match_scalable_lux_top_bits`, `packed_light_queue_entry_matches_scalable_lux_bit_layout`, `packed_light_queue_entry_reads_raw_scalable_lux_values`, `packed_light_propagation_queue_preserves_fifo_order`, `packed_light_propagation_queues_keep_increase_and_decrease_work_separate`, `light_queue_entry_decrease_entries_match_vanilla_bits`, `light_queue_entry_increase_entries_match_vanilla_bits`, `light_queue_entry_sky_source_entry_selects_horizontal_and_down_directions`, `light_queue_entry_masks_levels_like_vanilla`, `light_propagation_queue_preserves_fifo_order`, `light_propagation_queues_keep_increase_and_decrease_work_separate`, `oversized_lease_preserves_parked_pool_entry` | 0 |
| `steel-core/src/chunk/light/section_storage.rs` | 132 | — | 0 |
| `steel-core/src/chunk/light/sky_propagation/algorithms.rs` | 515 | — | 0 |
| `steel-core/src/chunk/light/sky_propagation/context.rs` | 196 | — | 0 |
| `steel-core/src/chunk/light/sky_propagation/mod.rs` | 283 | — | 0 |
| `steel-core/src/chunk/light/sky_propagation/queue_engine.rs` | 521 | — | 0 |
| `steel-core/src/chunk/light/sky_propagation/tests.rs` | 448 | `context_requires_sky_layer`, `sky_light_chunk_without_edge_checks_propagates_down_air_column`, `sky_light_chunk_without_edge_checks_keeps_sealed_roof_dark`, `sky_light_changes_add_and_remove_air_column_shadow`, `sky_light_chunk_edge_checks_pull_neighbor_under_ceiling`, `sky_light_chunk_requires_center_chunk`, `sky_light_changes_skip_missing_center_chunk` | 0 |
| `steel-core/src/chunk/light/sky_sources.rs` | 282 | — | 0 |
| `steel-core/src/chunk/light/storage.rs` | 473 | `sky_light_below_range_starts_search_at_min_light_section` | 0 |
| `steel-core/src/chunk/light/work_gate.rs` | 150 | `overlapping_windows_cannot_be_reserved_together`, `non_overlapping_windows_can_be_reserved_together`, `dropping_reservation_releases_window` | 0 |
| `steel-core/src/chunk/light/workset.rs` | 1464 | `workset_pins_cached_chunk_holder_until_dropped`, `workset_reports_missing_required_inner_chunk`, `chunk_read_cache_exposes_admitted_chunks`, `section_read_cache_uses_scalable_lux_local_indices`, `section_read_cache_reports_non_empty_sections`, `section_read_cache_reports_outer_chunk_emptiness_maps`, `workset_can_read_sections_without_writable_light_scope`, `light_edit_reads_writes_and_commits_sections`, `light_edit_drops_without_commit`, `light_edit_publishes_explicit_notifications`, `sky_edit_materializes_removed_missing_sections_transiently`, `light_edit_extrudes_lower_row_from_source_above` | 0 |
| `steel-core/src/chunk/mod.rs` | 33 | — | 0 |
| `steel-core/src/chunk/paletted_container/encode.rs` | 210 | `packs_every_width_like_simple_bit_storage` | 0 |
| `steel-core/src/chunk/paletted_container.rs` | 831 | `get_at_index_reads_homogeneous_palette_values`, `get_at_index_uses_y_z_x_linear_order_for_blocks`, `maybe_has_checks_palette_values_without_scanning_cells`, `copy_column_into_matches_get_for_homogeneous_container`, `copy_column_into_matches_get_for_heterogeneous_container`, `copy_column_into_matches_get_for_building_container`, `block_global_palette_uses_registry_derived_bit_width_and_long_count`, `biome_global_palette_uses_registry_derived_bit_width_and_long_count`, `written_block_sections_decode_to_their_values_for_every_strategy`, `written_biome_sections_decode_to_their_values_for_every_strategy`, `write_rejects_global_id_too_wide_for_the_global_palette`, `write_rejects_value_missing_from_palette_after_it_was_indexed_elsewhere` | 0 |
| `steel-core/src/chunk/player_chunk_view.rs` | 213 | `test_contains`, `test_difference`, `test_disjoint_difference` | 0 |
| `steel-core/src/chunk/player_ticket_tracker.rs` | 399 | `colocated_players_share_center_and_loading_tickets`, `overlapping_views_keep_loading_tickets_until_last_center_leaves`, `movement_is_atomic_and_stale_removal_cannot_remove_new_position`, `distance_changes_replace_simulation_tickets_and_adjust_only_view_ring` | 0 |
| `steel-core/src/chunk/section.rs` | 1400 | `stable_air_occupancy_tracks_canonical_air_and_non_air_transitions`, `arbitrary_palette_mutation_invalidates_stable_air_certificate_until_recount`, `write_guard_setter_keeps_stable_air_occupancy_tracked`, `raw_bulk_writes_invalidate_stable_air_certificate_until_recount`, `recount_uses_homogeneous_palette_frequency`, `recount_uses_heterogeneous_palette_frequencies`, `holder_keeps_random_tick_eligibility_in_sync`, `shared_random_tick_section_bits_follow_cross_word_updates`, `generation_recount_publishes_random_tick_section_bit`, `counter_aware_write_recounts_building_palette_before_delta` | 0 |
| `steel-core/src/chunk/simulation_ticket_manager.rs` | 8 | — | 0 |
| `steel-core/src/chunk/status.rs` | 109 | — | 0 |
| `steel-core/src/chunk_saver/bit_pack.rs` | 116 | `test_bits_for_palette_len`, `test_pack_unpack_roundtrip`, `test_pack_4096_entries` | 0 |
| `steel-core/src/chunk_saver/format.rs` | 1306 | `persistent_block_state_properties_round_trip`, `test_region_pos_from_chunk`, `test_local_chunk_pos`, `test_chunk_index`, `test_chunk_entry_roundtrip`, `test_chunk_entry_empty`, `test_sector_count`, `test_find_free_sectors_empty`, `test_find_free_sectors_gap` | 0 |
| `steel-core/src/chunk_saver/mod.rs` | 43 | — | 0 |
| `steel-core/src/chunk_saver/ram_only.rs` | 86 | — | 0 |
| `steel-core/src/chunk_saver/region_manager.rs` | 974 | `invalid_zstd_payload_is_removed_for_regeneration`, `semantically_invalid_complete_payload_is_removed_for_regeneration`, `incomplete_payload_read_is_an_error_and_keeps_slot`, `invalid_world_geometry_is_not_classified_as_chunk_corruption`, `invalid_chunk_status_byte_is_a_structural_error_and_is_preserved` | 0 |
| `steel-core/src/chunk_saver/registry.rs` | 253 | `domain_storage_accepts_durable_primary_or_entirely_ephemeral_worlds` | 0 |
| `steel-core/src/chunk_saver/storage/chunk.rs` | 766 | — | 0 |
| `steel-core/src/chunk_saver/storage/entities.rs` | 428 | — | 0 |
| `steel-core/src/chunk_saver/storage/light.rs` | 162 | — | 0 |
| `steel-core/src/chunk_saver/storage/mod.rs` | 812 | — | 0 |
| `steel-core/src/chunk_saver/storage/structures.rs` | 1088 | — | 0 |
| `steel-core/src/chunk_saver/storage/tests/chunks_sections.rs` | 339 | `chunk_save_rejects_full_status_for_proto_data`, `chunk_save_abandons_a_proto_status_snapshot_of_a_promoted_chunk`, `unknown_referenced_block_state_is_corruption_instead_of_air_recovery`, `unknown_referenced_biome_is_corruption_instead_of_plains_recovery`, `proto_heightmap_save_preserves_existing_maps_and_load_primes_missing_maps`, `terrain_heightmap_save_excludes_stale_worldgen_maps`, `ram_only_storage_restores_the_status_bundled_with_the_prepared_save`, `proto_postprocessing_roundtrips_through_persistent_chunk`, `full_chunk_postprocessing_roundtrips_through_persistent_chunk`, `heterogeneous_biome_section_roundtrips_through_persistent_chunk`, `chunk_save_refuses_a_section_palette_that_lost_a_block` | 0 |
| `steel-core/src/chunk_saver/storage/tests/entities.rs` | 718 | `persistent_entity_load_clamps_position_like_vanilla`, `persistent_entity_load_rejects_non_finite_rotation_like_vanilla`, `block_entity_stored_components_survive_chunk_save_load`, `owned_block_entity_load_preserves_stored_components`, `proto_block_entities_roundtrip_and_promote_to_full_chunk`, `persistent_block_entity_with_invalid_live_state_is_rejected_before_construction`, `persistent_block_entity_with_malformed_nbt_is_dropped`, `proto_entities_roundtrip_and_promote_to_full_chunk`, `unsupported_persistent_entity_is_dropped_after_load_and_resave`, `prepared_save_reports_handled_runtime_entity_ids`, `full_chunk_load_defers_entities_to_world_registration`, `runtime_entity_passengers_save_nested_and_load_flattened_for_registration`, `runtime_entity_passengers_skip_non_serializable_entities_like_vanilla`, `unimplemented_block_entities_preserve_nbt_through_proto_save_load` | 0 |
| `steel-core/src/chunk_saver/storage/tests/light.rs` | 120 | `persisted_light_is_ignored_before_light_status`, `loaded_sky_light_fills_missing_sections_below_loaded_data_with_zero`, `chunk_light_persistence_canonicalizes_visible_and_internal_sections`, `persistent_chunk_loads_chunk_owned_light_into_full_chunk` | 0 |
| `steel-core/src/chunk_saver/storage/tests/mod.rs` | 51 | — | 0 |
| `steel-core/src/chunk_saver/storage/tests/structures.rs` | 730 | `structure_persistence_filters_empty_starts_and_sorts_entries`, `structure_start_roundtrip_preserves_typed_jigsaw_state`, `structure_start_roundtrip_preserves_template_and_procedural_payloads`, `template_processor_list_roundtrips_ruined_portal_processors` | 0 |
| `steel-core/src/chunk_saver/storage/tests/ticks.rs` | 169 | `persisted_proto_ticks_deduplicate_while_full_ticks_retain_saved_entries`, `persisted_tick_priorities_clamp_to_vanilla_extremes`, `forced_prepare_preserves_dirty_set_after_save_decision`, `full_chunk_save_snapshots_chunk_owned_scheduled_ticks` | 0 |
| `steel-core/src/chunk_saver/storage/ticks.rs` | 92 | — | 0 |
| `steel-core/src/command/api.rs` | 959 | `command_reader_cursor_is_a_utf8_byte_offset` | 0 |
| `steel-core/src/command/brigadier/argument.rs` | 246 | — | 0 |
| `steel-core/src/command/brigadier/builder.rs` | 432 | — | 0 |
| `steel-core/src/command/brigadier/context.rs` | 572 | — | 0 |
| `steel-core/src/command/brigadier/context_chain.rs` | 166 | — | 0 |
| `steel-core/src/command/brigadier/dispatcher.rs` | 436 | — | 0 |
| `steel-core/src/command/brigadier/error.rs` | 372 | — | 0 |
| `steel-core/src/command/brigadier/mod.rs` | 90 | — | 0 |
| `steel-core/src/command/brigadier/node.rs` | 638 | — | 0 |
| `steel-core/src/command/brigadier/reader.rs` | 329 | — | 0 |
| `steel-core/src/command/brigadier/runtime.rs` | 29 | — | 0 |
| `steel-core/src/command/brigadier/string_range.rs` | 82 | — | 0 |
| `steel-core/src/command/brigadier/suggestion.rs` | 351 | — | 0 |
| `steel-core/src/command/brigadier/tests/argument.rs` | 193 | `parses_bounded_long_arguments`, `long_bounds_reset_the_error_cursor`, `parses_bounded_float_and_double_arguments`, `float_and_double_bounds_reset_the_error_cursor`, `parses_word_quotable_and_greedy_strings`, `quotable_strings_preserve_reader_escape_errors`, `empty_quoted_strings_are_valid_arguments`, `differently_configured_string_arguments_collide` | 0 |
| `steel-core/src/command/brigadier/tests/context_chain.rs` | 325 | `executable_context_uses_runtime_source_and_parsed_arguments`, `executable_context_exposes_all_primitive_arguments_and_metadata`, `identity_redirects_form_distinct_context_chain_stages`, `redirect_modifier_replaces_the_runtime_source`, `forked_execution_counts_successes_instead_of_command_results`, `forked_command_errors_are_reported_and_suppressed`, `fork_modifier_errors_are_suppressed_before_the_terminal_stage`, `non_forked_command_errors_are_reported_and_propagated`, `incomplete_input_does_not_build_an_executable_chain`, `parse_finalization_distinguishes_unknown_commands_and_arguments`, `parse_finalization_preserves_a_single_specific_parser_error` | 0 |
| `steel-core/src/command/brigadier/tests/mod.rs` | 416 | `reader_tracks_input_and_utf16_positions`, `reader_advances_utf8_bytes_and_tracks_utf16_positions`, `reader_skips_java_whitespace_only`, `reader_reads_unquoted_strings_with_brigadier_character_set`, `reader_reads_single_and_double_quoted_strings`, `reader_unescapes_only_the_active_quote_and_backslash`, `quoted_string_errors_match_brigadier_cursors`, `quoted_string_error_cursor_uses_utf16_units`, `reader_selects_quoted_or_unquoted_string`, `reader_reads_integers_and_rolls_back_invalid_values`, `reader_reads_longs_and_rolls_back_invalid_values`, `reader_reads_floating_point_numbers_and_rolls_back_invalid_values`, `reader_reads_booleans_and_rolls_back_invalid_values`, `reader_expects_symbols_without_consuming_mismatches`, `command_syntax_error_formats_brigadier_context`, `command_syntax_error_builds_translated_styled_feedback`, `command_syntax_error_context_does_not_split_unicode_scalars`, `dynamic_command_errors_have_no_parser_context`, `string_ranges_use_brigadier_utf16_positions` | 0 |
| `steel-core/src/command/brigadier/tests/parsing.rs` | 304 | `parses_literal_commands_and_tracks_ranges`, `literal_ranges_and_failure_cursors_use_utf16_units`, `requirements_hide_unavailable_nodes_from_parsing`, `execution_requirements_do_not_hide_descendants`, `incomplete_commands_stop_before_the_trailing_separator`, `unknown_subcommands_keep_the_last_successful_context`, `parses_boolean_and_bounded_integer_arguments`, `bounded_integer_errors_reset_to_the_argument_start`, `argument_parsing_requires_a_space_before_trailing_data`, `complete_potential_wins_over_an_incomplete_sibling`, `matching_literals_take_priority_over_argument_siblings`, `identity_redirects_create_a_child_parse_context` | 0 |
| `steel-core/src/command/brigadier/tests/registration.rs` | 336 | `registration_preserves_child_order_and_returns_stable_ids`, `compatible_literal_registration_merges_grandchildren`, `compatible_registration_preserves_or_replaces_command`, `duplicate_children_inside_one_builder_are_normalized`, `shared_requirement_identity_can_merge`, `incompatible_requirement_collision_is_atomic`, `incompatible_argument_collision_is_atomic`, `literal_and_argument_with_the_same_name_collide`, `incompatible_redirect_collision_is_atomic`, `redirect_targets_must_belong_to_the_dispatcher`, `redirected_nodes_cannot_have_children`, `symbolic_command_root_redirect_resolves_to_the_registered_root`, `command_roots_must_be_literals`, `requirements_remain_generic_source_predicates`, `command_callbacks_receive_the_generic_source_context` | 0 |
| `steel-core/src/command/brigadier/tests/runtime_model.rs` | 159 | `parsing_uses_the_runtime_argument_representation`, `parsing_preserves_opaque_runtime_payloads` | 0 |
| `steel-core/src/command/brigadier/tests/suggestion.rs` | 226 | `suggestion_builder_replaces_remaining_text_and_omits_noops`, `integer_suggestions_sort_numerically`, `suggestions_merge_different_ranges`, `suggestion_application_uses_utf16_ranges`, `suggestion_tooltips_are_preserved`, `suggestion_builder_can_restart_for_a_provider`, `completes_root_literals_case_insensitively`, `completes_subcommands_and_boolean_arguments`, `exact_suggestions_are_omitted`, `completion_follows_redirect_contexts`, `completion_filters_nodes_that_fail_requirements`, `completion_ranges_count_supplementary_characters_as_two_units` | 0 |
| `steel-core/src/command/builtins/advancement.rs` | 464 | — | 0 |
| `steel-core/src/command/builtins/attribute.rs` | 318 | — | 0 |
| `steel-core/src/command/builtins/ban.rs` | 309 | — | 0 |
| `steel-core/src/command/builtins/banip.rs` | 146 | — | 0 |
| `steel-core/src/command/builtins/bossbar.rs` | 421 | — | 0 |
| `steel-core/src/command/builtins/clear.rs` | 213 | `clear_graph_matches_vanilla_argument_shape` | 0 |
| `steel-core/src/command/builtins/clone.rs` | 196 | `adjacent_regions_do_not_intersect`, `overlapping_regions_intersect` | 0 |
| `steel-core/src/command/builtins/damage.rs` | 103 | — | 0 |
| `steel-core/src/command/builtins/data.rs` | 375 | — | 0 |
| `steel-core/src/command/builtins/datapack.rs` | 90 | — | 0 |
| `steel-core/src/command/builtins/defaultgamemode.rs` | 133 | `defaultgamemode_graph_matches_shape` | 0 |
| `steel-core/src/command/builtins/dialog.rs` | 89 | — | 0 |
| `steel-core/src/command/builtins/difficulty.rs` | 118 | `client_difficulty_permission_uses_the_command_root` | 0 |
| `steel-core/src/command/builtins/domain.rs` | 386 | `domain_graph_uses_the_loaded_world_argument` | 0 |
| `steel-core/src/command/builtins/effect.rs` | 182 | — | 0 |
| `steel-core/src/command/builtins/enchant.rs` | 360 | `enchant_graph_uses_all_entities_and_an_enchantment_resource`, `enchant_main_hand_applies_once_and_then_rejects_the_same_enchantment` | 0 |
| `steel-core/src/command/builtins/execute/condition.rs` | 828 | `block_region_volume_uses_inclusive_normalized_corners`, `data_match_count_returns_selected_tag_count`, `masked_regions_skip_only_vanilla_air`, `region_block_entities_compare_type_and_custom_data_only` | 0 |
| `steel-core/src/command/builtins/execute/mod.rs` | 90 | — | 0 |
| `steel-core/src/command/builtins/execute/source.rs` | 257 | — | 0 |
| `steel-core/src/command/builtins/execute/store.rs` | 448 | `score_store_creates_and_updates_each_holder`, `stored_value_distinguishes_numeric_results_from_success`, `data_types_match_java_numeric_narrowing`, `compound_path_mutation_creates_and_replaces_values`, `storage_mutation_reads_and_writes_the_current_compound` | 0 |
| `steel-core/src/command/builtins/experience.rs` | 404 | `experience_and_xp_roots_share_the_expected_graph` | 0 |
| `steel-core/src/command/builtins/fill.rs` | 539 | `fill_graph_exposes_all_vanilla_modes_and_typed_arguments`, `hollow_replaces_the_shell_and_clears_the_core`, `replace_filter_only_changes_matching_blocks`, `destroy_mode_counts_the_destroyed_block_and_drops_its_loot`, `fill_limit_and_unloaded_region_fail_before_mutation`, `fill_requires_block_ticking_readiness_for_its_neighbor_halo` | 0 |
| `steel-core/src/command/builtins/fillbiome.rs` | 174 | `volume_of_unit_region_is_one`, `volume_of_two_cube_is_eight` | 0 |
| `steel-core/src/command/builtins/fly.rs` | 279 | `fly_graph_uses_explicit_target_and_bounded_speed_branches`, `disabling_flight_clears_active_flying_state`, `fly_speed_uses_vanilla_default_as_the_multiplier_base` | 0 |
| `steel-core/src/command/builtins/forceload.rs` | 242 | `a_single_block_covers_exactly_its_chunk`, `a_rectangle_covers_every_chunk_between_the_corners`, `the_corner_order_does_not_matter` | 0 |
| `steel-core/src/command/builtins/function.rs` | 82 | — | 0 |
| `steel-core/src/command/builtins/gamemode.rs` | 352 | `scoped_game_mode_permissions_preserve_root_and_child_rules`, `client_switcher_requires_at_least_one_allowed_mode`, `gamemode_graph_matches_vanilla_shape` | 0 |
| `steel-core/src/command/builtins/gamerule.rs` | 184 | `vanilla_rules_have_short_and_qualified_literals`, `integer_rule_bounds_are_retained_in_the_graph` | 0 |
| `steel-core/src/command/builtins/give.rs` | 190 | `give_graph_uses_players_item_stack_and_positive_count` | 0 |
| `steel-core/src/command/builtins/help.rs` | 166 | `a_list_shorter_than_a_page_is_one_page`, `pages_grow_one_per_seven_commands`, `page_index_maps_to_the_right_slice` | 0 |
| `steel-core/src/command/builtins/invsee/tests.rs` | 723 | `base_and_modify_permissions_grant_the_expected_access`, `invsee_rejects_players_in_different_domains`, `readonly_target_slots_reject_pickup_and_creative_clone`, `modify_view_edits_armor_slots_within_equipment_rules`, `modify_view_synchronizes_target_armor_without_inventory_locks`, `self_invsee_synchronizes_own_armor_slot`, `modify_view_synchronizes_empty_offhand_after_removal`, `modify_view_synchronizes_target_hotbar_slot`, `modify_view_coalesces_to_latest_target_inventory_value`, `modify_view_drag_queues_each_changed_target_slot`, `overriding_menu_defers_main_inventory_sync_until_close`, `replacing_overriding_menu_keeps_main_inventory_sync_deferred`, `normal_menu_does_not_defer_main_inventory_sync`, `modify_view_moves_inventory_items_in_both_directions`, `modify_view_extracts_but_cannot_insert_crafting_items`, `self_invsee_quick_move_does_not_rearrange_the_aliased_inventory`, `open_menu_keeps_captured_access_and_tracks_target_lifecycle` | 0 |
| `steel-core/src/command/builtins/invsee.rs` | 293 | — | 0 |
| `steel-core/src/command/builtins/item.rs` | 168 | `hotbar_slots_map_to_the_hotbar`, `inventory_slots_shift_past_the_hotbar`, `equipment_slots_resolve`, `unknown_slot_is_rejected` | 0 |
| `steel-core/src/command/builtins/kick.rs` | 71 | — | 0 |
| `steel-core/src/command/builtins/kill.rs` | 108 | `kill_graph_supports_self_and_multiple_entity_targets` | 0 |
| `steel-core/src/command/builtins/list.rs` | 62 | — | 0 |
| `steel-core/src/command/builtins/locate.rs` | 610 | `locate_graph_exposes_only_the_supported_typed_structure_branch`, `locate_coordinates_component_matches_vanilla_interactivity`, `horizontal_distance_matches_vanillas_wrapping_int_and_float_math` | 0 |
| `steel-core/src/command/builtins/loot.rs` | 165 | `unknown_loot_tables_are_rejected`, `vanilla_loot_tables_are_registered` | 0 |
| `steel-core/src/command/builtins/mod.rs` | 744 | `first_builtin_slice_has_the_expected_graph_shape`, `startup_extensions_merge_after_builtins_with_namespaced_collision_fallbacks`, `invsee_discovers_only_steel_namespaced_permissions`, `execute_graph_uses_expected_redirects_and_argument_types` | 0 |
| `steel-core/src/command/builtins/msg.rs` | 78 | — | 0 |
| `steel-core/src/command/builtins/operator.rs` | 259 | `operator_group_updates_are_idempotent_and_preserve_other_groups`, `operator_targets_use_vanillas_game_profile_argument` | 0 |
| `steel-core/src/command/builtins/particle.rs` | 93 | — | 0 |
| `steel-core/src/command/builtins/perms/config.rs` | 338 | `permission_edits_replace_the_exact_context_only`, `permission_edits_replace_semantically_equal_context_orderings`, `required_default_and_inherited_groups_cannot_be_deleted` | 0 |
| `steel-core/src/command/builtins/perms/mod.rs` | 1492 | `perms_exposes_the_management_surface_without_old_aliases`, `perms_discovery_contains_static_admin_and_granular_command_permissions` | 0 |
| `steel-core/src/command/builtins/playsound.rs` | 257 | `playsound_graph_matches_vanilla_shape` | 0 |
| `steel-core/src/command/builtins/position.rs` | 33 | — | 0 |
| `steel-core/src/command/builtins/posteffect.rs` | 323 | `posteffect_graph_matches_vanilla_shapes` | 0 |
| `steel-core/src/command/builtins/random.rs` | 209 | — | 0 |
| `steel-core/src/command/builtins/recipe.rs` | 111 | `registration_builds` | 0 |
| `steel-core/src/command/builtins/reload.rs` | 146 | — | 0 |
| `steel-core/src/command/builtins/return_command.rs` | 162 | `return_graph_matches_vanillas_three_forms` | 0 |
| `steel-core/src/command/builtins/ride.rs` | 48 | — | 0 |
| `steel-core/src/command/builtins/rotate.rs` | 189 | `rotate_graph_matches_vanilla_rotation_and_facing_shapes` | 0 |
| `steel-core/src/command/builtins/save.rs` | 92 | — | 0 |
| `steel-core/src/command/builtins/say.rs` | 55 | — | 0 |
| `steel-core/src/command/builtins/schedule.rs` | 134 | — | 0 |
| `steel-core/src/command/builtins/scoreboard.rs` | 391 | — | 0 |
| `steel-core/src/command/builtins/seed.rs` | 39 | — | 0 |
| `steel-core/src/command/builtins/serverpack.rs` | 180 | — | 0 |
| `steel-core/src/command/builtins/setblock.rs` | 96 | — | 0 |
| `steel-core/src/command/builtins/setidletimeout.rs` | 50 | — | 0 |
| `steel-core/src/command/builtins/setworldspawn.rs` | 147 | `setworldspawn_graph_uses_deferred_coordinate_arguments` | 0 |
| `steel-core/src/command/builtins/spawnpoint.rs` | 200 | `spawnpoint_graph_matches_vanilla_argument_paths` | 0 |
| `steel-core/src/command/builtins/spectate.rs` | 118 | — | 0 |
| `steel-core/src/command/builtins/spreadplayers.rs` | 149 | `first_position_is_always_available`, `positions_stay_within_the_range`, `crowded_space_exhausts_the_attempt_budget` | 0 |
| `steel-core/src/command/builtins/stop.rs` | 28 | — | 0 |
| `steel-core/src/command/builtins/stopsound.rs` | 127 | — | 0 |
| `steel-core/src/command/builtins/stopwatch.rs` | 168 | — | 0 |
| `steel-core/src/command/builtins/summon.rs` | 212 | `summon_graph_uses_typed_entity_and_deferred_position_arguments` | 0 |
| `steel-core/src/command/builtins/swing.rs` | 122 | — | 0 |
| `steel-core/src/command/builtins/tag.rs` | 133 | — | 0 |
| `steel-core/src/command/builtins/team.rs` | 359 | — | 0 |
| `steel-core/src/command/builtins/teammsg.rs` | 87 | — | 0 |
| `steel-core/src/command/builtins/teleport.rs` | 573 | `teleport_graph_matches_vanilla_target_and_facing_shapes`, `different_worlds_strip_relative_position_but_preserve_direction_flags`, `direct_teleport_rejects_cross_domain_transitions`, `packet_values_rebase_source_relative_results_for_each_target` | 0 |
| `steel-core/src/command/builtins/tellraw.rs` | 104 | `tellraw_graph_uses_player_targets_and_component_messages` | 0 |
| `steel-core/src/command/builtins/tick.rs` | 410 | `tick_graph_matches_the_target_shape_and_permissions` | 0 |
| `steel-core/src/command/builtins/time.rs` | 445 | `time_graph_matches_the_26_2_clock_shape`, `wrap_time_matches_vanilla_modulus`, `time_marker_and_timeline_errors_use_vanillas_argument_order` | 0 |
| `steel-core/src/command/builtins/title.rs` | 315 | `title_graph_matches_vanilla_shape` | 0 |
| `steel-core/src/command/builtins/tpsbar.rs` | 70 | `tpsbar_is_registered_and_executable` | 0 |
| `steel-core/src/command/builtins/transfer.rs` | 104 | — | 0 |
| `steel-core/src/command/builtins/trigger.rs` | 169 | — | 0 |
| `steel-core/src/command/builtins/version.rs` | 112 | `version_is_a_bare_executable_literal` | 0 |
| `steel-core/src/command/builtins/waypoint.rs` | 175 | — | 0 |
| `steel-core/src/command/builtins/weather.rs` | 110 | `weather_kinds_map_to_vanilla_parameter_sets` | 0 |
| `steel-core/src/command/builtins/whitelist.rs` | 244 | — | 0 |
| `steel-core/src/command/builtins/worldborder.rs` | 364 | `formats_ticks_as_vanilla_seconds`, `formats_limits_like_vanilla_numeric_components`, `add_preserves_existing_lerp_only_when_time_is_explicit`, `size_subcommands_accept_optional_time` | 0 |
| `steel-core/src/command/execution/argument.rs` | 1737 | — | 0 |
| `steel-core/src/command/execution/argument_tests/coordinates.rs` | 149 | `block_position_retains_world_coordinates_until_execution`, `vec3_centers_absolute_integer_x_and_z_components`, `vec2_centers_absolute_integer_components_and_preserves_source_y`, `vec2_suggestions_stop_after_two_components`, `coordinate_arguments_parse_local_components_and_reject_mixed_types`, `block_position_requires_integers_only_for_absolute_components`, `rotation_argument_retains_yaw_then_pitch_expressions`, `coordinate_suggestions_include_vanilla_partial_prefixes` | 0 |
| `steel-core/src/command/execution/argument_tests/core_permissions.rs` | 321 | `keyed_argument_erasure_accepts_new_parser_and_value_types`, `keyed_parser_equality_includes_concrete_configuration`, `component_argument_parses_vanilla_snbt_forms`, `permission_arguments_parse_contexts_and_suggest_discovered_values`, `permission_group_argument_can_require_a_configured_group`, `owned_permission_arguments_scope_unset_suggestions_to_prior_arguments`, `component_argument_preserves_list_siblings_and_following_nodes`, `component_argument_reports_codec_errors_at_the_argument_start`, `component_argument_preserves_translatable_snbt_errors`, `component_argument_compiles_command_strings_during_parsing` | 0 |
| `steel-core/src/command/execution/argument_tests/general.rs` | 428 | `score_holder_argument_retains_deferred_names_uuids_selectors_and_wildcards`, `objective_and_integer_range_arguments_retain_vanilla_values`, `block_predicate_argument_parses_blocks_tags_properties_and_nbt`, `block_state_argument_parses_defaulted_properties_and_nbt_without_tags`, `block_arguments_use_vanilla_diagnostics`, `block_arguments_use_vanilla_stateful_suggestions`, `block_predicate_argument_validates_concrete_properties_but_defers_tag_properties`, `nbt_path_argument_retains_vanilla_path_nodes`, `swizzle_argument_retains_unique_axes_and_rejects_duplicates` | 0 |
| `steel-core/src/command/execution/argument_tests/item_predicate.rs` | 325 | `item_predicate_argument_matches_targets_boolean_terms_and_count_ranges`, `item_predicate_argument_decodes_exact_components_before_matching`, `item_predicate_argument_supports_damage_and_enchantment_predicates`, `item_predicate_argument_supports_partial_custom_data_matching`, `item_predicate_argument_decodes_every_registered_vanilla_partial_predicate`, `item_predicate_argument_matches_registered_firework_explosion_predicate`, `item_predicate_argument_matches_stream_only_nested_template_without_materializing_a_stack`, `item_predicate_argument_supports_attribute_modifier_collection_predicates`, `item_predicate_argument_rejects_noncanonical_holder_and_slot_values`, `item_predicate_argument_uses_map_codec_for_component_existence_predicates`, `item_predicate_argument_rejects_malformed_potion_predicate`, `item_predicate_argument_rejects_transient_components_as_component_tests`, `item_predicate_argument_suggests_items_tags_and_condition_types` | 0 |
| `steel-core/src/command/execution/argument_tests/item_stack.rs` | 473 | `item_stack_argument_parses_supported_components_and_registered_removals`, `item_stack_argument_parses_identifier_components`, `item_stack_argument_parses_custom_data_codecs`, `item_stack_argument_parses_custom_model_data_and_enchantability`, `item_stack_argument_parses_color_map_and_amplifier_components`, `item_stack_argument_parses_direct_entity_variant_components`, `item_stack_argument_parses_registry_holder_set_components`, `item_stack_argument_uses_vanilla_numeric_codec_coercions`, `item_stack_argument_parses_compound_component_values`, `item_stack_argument_rejects_unsupported_transient_and_invalid_components`, `item_stack_argument_rejects_invalid_recursive_contents`, `item_stack_argument_sanitizes_redundant_component_changes`, `item_arguments_propagate_translatable_snbt_errors`, `removing_max_stack_size_uses_vanillas_fallback_of_one`, `item_stack_argument_suggests_items_and_supported_component_operations` | 0 |
| `steel-core/src/command/execution/argument_tests/mod.rs` | 180 | — | 0 |
| `steel-core/src/command/execution/argument_tests/resources_world.rs` | 397 | `biome_or_tag_argument_resolves_registry_entries_and_tags`, `structure_or_tag_key_argument_defers_registry_resolution_until_execution`, `heightmap_argument_accepts_vanilla_live_world_names_and_suggests_them`, `domain_argument_resolves_and_suggests_only_configured_domains`, `world_argument_retains_relative_and_fully_qualified_names`, `storage_key_argument_parses_and_suggests_source_domain_keys`, `game_mode_argument_parses_only_vanilla_names`, `game_mode_argument_suggests_vanilla_names`, `entity_anchor_argument_parses_and_suggests_vanilla_names`, `summonable_entity_argument_resolves_only_registered_factories`, `summonable_entity_argument_suggests_only_registered_factories`, `enchantment_argument_resolves_and_suggests_registered_entries` | 0 |
| `steel-core/src/command/execution/argument_tests/selector_time.rs` | 191 | `entity_selector_argument_is_retained_for_deferred_resolution`, `entity_selector_argument_suggests_source_domain_players`, `time_argument_parses_vanilla_units_and_defaults_to_ticks`, `time_argument_uses_java_half_up_rounding`, `time_argument_rejects_invalid_units_and_values_below_its_minimum`, `time_argument_suggests_units_for_a_numeric_prefix`, `world_clock_argument_resolves_default_and_explicit_namespaces`, `world_clock_argument_rejects_unknown_resources`, `time_marker_argument_retains_default_namespace_identifier`, `time_marker_argument_suggests_only_visible_markers_for_selected_clock`, `timeline_suggestions_use_the_preceding_clock_argument` | 0 |
| `steel-core/src/command/execution/biome.rs` | 111 | — | 0 |
| `steel-core/src/command/execution/block/suggestions.rs` | 298 | — | 0 |
| `steel-core/src/command/execution/block.rs` | 443 | — | 0 |
| `steel-core/src/command/execution/coordinates.rs` | 544 | `world_coordinates_resolve_relative_components_at_execution`, `rotation_coordinates_keep_vanilla_pitch_yaw_component_order`, `local_coordinates_follow_source_rotation`, `coordinates_retain_axis_relative_metadata` | 0 |
| `steel-core/src/command/execution/item.rs` | 389 | — | 0 |
| `steel-core/src/command/execution/item_predicate.rs` | 1381 | — | 0 |
| `steel-core/src/command/execution/mod.rs` | 64 | — | 0 |
| `steel-core/src/command/execution/nbt.rs` | 30 | — | 0 |
| `steel-core/src/command/execution/permission.rs` | 410 | — | 0 |
| `steel-core/src/command/execution/profile.rs` | 148 | — | 0 |
| `steel-core/src/command/execution/queue.rs` | 877 | — | 0 |
| `steel-core/src/command/execution/runtime.rs` | 514 | — | 0 |
| `steel-core/src/command/execution/score.rs` | 260 | — | 0 |
| `steel-core/src/command/execution/selector/mod.rs` | 453 | — | 0 |
| `steel-core/src/command/execution/selector/model.rs` | 565 | — | 0 |
| `steel-core/src/command/execution/selector/parser.rs` | 980 | — | 0 |
| `steel-core/src/command/execution/selector/suggestions.rs` | 533 | — | 0 |
| `steel-core/src/command/execution/selector/tests.rs` | 334 | `selector_permissions_distinguish_basic_and_advanced_syntax`, `selector_argument_shapes_enforce_cardinality_and_player_only_rules`, `selector_parses_core_filters_and_nested_snbt`, `selector_preserves_translatable_snbt_errors`, `selector_reader_leaves_following_command_input_unconsumed`, `selector_reports_missing_predicate_and_advancement_foundations`, `selector_entity_filters_use_command_identity_and_nbt_snapshot`, `selector_score_and_team_filters_use_one_domain_scoreboard`, `selector_suggestions_include_supported_options_and_source_values` | 0 |
| `steel-core/src/command/execution/source.rs` | 1045 | `rotation_normalization_matches_command_source_stack`, `admin_broadcast_uses_current_execution_entity_name`, `authorization_context_captures_initial_world_scope`, `authorization_context_uses_published_snapshot_instead_of_stale_player_permissions`, `unrestricted_authorization_allows_console_permissions` | 0 |
| `steel-core/src/command/execution/structure.rs` | 88 | — | 0 |
| `steel-core/src/command/execution/suggestions.rs` | 124 | `suggest_list_matches_segment_starts_case_insensitively` | 0 |
| `steel-core/src/command/execution/swing_animation.rs` | 66 | — | 0 |
| `steel-core/src/command/execution/tests.rs` | 1457 | `queue_runs_standard_commands_with_the_runtime_source_callback`, `queue_drops_a_source_that_became_stale_before_execution`, `command_limit_stops_before_the_next_queued_action`, `forked_sources_execute_in_order`, `standard_modifiers_consume_one_sequence_cost`, `fork_limit_uses_vanillas_exclusive_boundary`, `modifier_failures_follow_fork_suppression_rules`, `terminal_failures_invoke_callbacks_but_only_non_forks_report_errors`, `custom_executor_returns_from_its_frame_and_discards_queued_work`, `custom_modifier_can_continue_with_return_propagation`, `queue_overflow_stops_work_queued_by_a_custom_executor`, `suspended_normal_executor_reports_its_delayed_result`, `suspended_normal_executor_cancels_when_its_effective_source_becomes_stale`, `suspended_normal_executor_is_retained_at_the_sequence_limit`, `suspended_normal_executor_reports_delayed_errors_like_a_standard_executor`, `suspended_normal_executor_reports_startup_errors_without_suspending`, `suspended_normal_executor_resumes_each_forked_source_and_suppresses_errors`, `suspended_normal_executor_preserves_forked_return_semantics`, `cancelling_suspended_normal_execution_cancels_its_work`, `suspended_execution_resumes_in_queue_order_with_the_original_frame`, `cancelling_execution_cancels_active_and_queued_suspensions`, `frame_return_cancels_discarded_suspension_work`, `queue_overflow_cancels_queued_suspension_work`, `pending_execution_queue_polls_once_per_tick_in_fifo_order`, `global_suspension_blocks_every_command_source_until_completion`, `pending_execution_queue_rejects_active_contexts`, `pending_execution_queue_cancels_retained_work`, `pending_execution_queue_cancels_work_whose_owner_is_stale` | 0 |
| `steel-core/src/command/execution/text.rs` | 608 | `selectors_use_resolved_separators_and_preserve_display_components`, `score_wildcard_resolves_separately_for_each_recipient`, `score_selectors_require_at_most_one_entity_and_fall_back_when_empty`, `nbt_resolution_selects_paths_and_applies_plain_rendering`, `interpreted_nbt_resolves_nested_scores_at_the_same_depth`, `interpreted_nbt_skips_components_with_invalid_compiled_strings` | 0 |
| `steel-core/src/command/execution/world.rs` | 75 | — | 0 |
| `steel-core/src/command/mod.rs` | 139 | `gamemaster_projection_enables_either_supported_client_capability` | 0 |
| `steel-core/src/command/protocol.rs` | 875 | `leading_slash_remains_in_the_packet_replacement_range`, `packet_projection_preserves_utf16_range_and_tooltip`, `packet_projection_applies_vanillas_suggestion_limit`, `command_tree_filters_both_requirement_kinds_but_marks_only_authorization`, `command_tree_hides_a_denied_executor_without_hiding_its_route`, `command_tree_remaps_redirects_after_filtering`, `command_tree_rejects_visible_redirects_to_filtered_targets`, `command_tree_projects_executable_primitive_arguments`, `permission_expression_nodes_project_as_terminal_greedy_strings`, `primitive_argument_projection_omits_default_bounds_and_maps_strings`, `steel_time_argument_projects_its_minimum`, `steel_position_arguments_project_vanilla_parsers`, `steel_entity_arguments_use_vanilla_client_suggestions`, `steel_scoreboard_arguments_project_vanilla_parsers`, `steel_biome_argument_projects_vanilla_resource_or_tag_parser`, `steel_structure_argument_projects_vanilla_resource_or_tag_key_parser`, `steel_block_predicate_argument_projects_vanillas_parser`, `steel_block_state_argument_projects_vanillas_parser`, `steel_nbt_path_argument_projects_vanillas_parser`, `steel_component_argument_projects_vanillas_parser`, `steel_storage_key_argument_projects_vanillas_identifier_parser`, `steel_world_argument_projects_vanillas_dimension_parser`, `steel_game_mode_argument_projects_vanillas_parser`, `steel_enchantment_argument_projects_vanillas_registry_parser`, `steel_item_stack_argument_asks_the_server_for_vanilla_parser_suggestions`, `steel_item_predicate_argument_asks_the_server_for_vanilla_parser_suggestions`, `steel_domain_argument_uses_server_resource_suggestions`, `summonable_entity_argument_projects_vanillas_resource_and_suggestions`, `steel_clock_arguments_project_vanilla_resource_parsers` | 0 |
| `steel-core/src/command/queue/mod.rs` | 6 | — | 0 |
| `steel-core/src/command/queue/pending.rs` | 175 | — | 0 |
| `steel-core/src/command/queue/requests.rs` | 348 | `executions_are_dequeued_in_submission_order`, `blocked_execution_sources_are_skipped_without_reordering_their_requests`, `full_execution_queue_rejects_without_dropping_pending_requests`, `suggestion_capacity_cannot_starve_execution_capacity`, `suggestions_from_one_sender_are_coalesced`, `busy_queues_are_drained_fairly`, `clear_discards_all_pending_requests` | 0 |
| `steel-core/src/command/registration/mod.rs` | 630 | — | 0 |
| `steel-core/src/command/registration/tests.rs` | 677 | `unique_commands_do_not_pollute_the_root_with_namespaced_variants`, `collisions_keep_the_first_root_and_expose_both_namespaced_commands`, `alias_collisions_use_the_same_owner_policy`, `symbolic_command_root_redirects_follow_each_registered_clone`, `duplicate_command_ids_are_rejected_without_replacing_the_first`, `aliases_cannot_duplicate_another_root_owned_by_the_same_command`, `namespaced_aliases_are_reserved_for_collision_fallbacks`, `command_root_must_match_its_stable_id_path`, `factories_receive_the_built_dispatcher_root_for_redirects`, `required_root_permissions_are_derived_from_stable_command_ids`, `dispatcher_build_retains_declared_and_derived_permission_keys`, `default_access_allows_unset_but_not_explicitly_denied_permissions`, `explicit_permission_expressions_replace_id_derivation`, `registration_composes_authorization_with_existing_context_requirements`, `derived_subcommand_permissions_only_allow_their_routes`, `derived_subcommand_permissions_follow_literals_through_arguments`, `default_access_still_honors_specific_subcommand_denies`, `missing_subcommand_permission_paths_fail_the_build`, `explicit_root_permissions_cannot_mix_with_derived_subcommands` | 0 |
| `steel-core/src/command/sender.rs` | 268 | — | 0 |
| `steel-core/src/command/storage.rs` | 345 | `missing_and_empty_values_match_vanilla_storage_semantics`, `persistent_storage_round_trips_binary_nbt`, `mutation_after_snapshot_remains_dirty`, `domains_keep_independent_command_storage`, `invalid_persisted_nbt_is_rejected` | 0 |
| `steel-core/src/config.rs` | 1620 | `a_container_quota_caps_the_configured_core_count`, `a_core_config_below_the_quota_is_kept`, `a_zero_core_config_detects_the_container_limit`, `the_effective_core_count_is_min_of_config_and_container`, `the_default_core_count_follows_detection`, `an_equally_sized_config_is_unchanged`, `a_report_of_zero_available_cores_still_yields_one`, `online_mode_requires_the_authenticated_encryption_flow`, `resolves_cascaded_world_defaults`, `resolves_selectable_player_storage_backends`, `resolves_explicit_portal_targets`, `rejects_missing_explicit_portal_target`, `rejects_self_explicit_portal_target`, `rejects_end_portal_target_on_end_dimension_world`, `rejects_end_portal_target_on_flat_end_dimension_world`, `rejects_end_portal_target_to_non_end_dimension_world`, `accepts_end_portal_target_to_flat_end_dimension_world`, `rejects_cross_domain_explicit_portal_target`, `rejects_reserved_global_domain`, `rejects_duplicate_world_names`, `rejects_missing_default_world`, `rejects_unknown_generator`, `flat_generator_config_is_optional`, `resolves_an_absolute_save_path`, `rejects_traversal_in_a_save_path`, `rejects_an_empty_save_path`, `world_storage_paths_stay_relative_to_the_save_root`, `view_distance_clamp_lifts_the_floor_on_both_sides`, `no_pack_means_no_packet`, `an_empty_url_is_treated_as_unconfigured`, `the_push_packet_carries_the_configured_fields`, `a_missing_hash_is_sent_empty` | 0 |
| `steel-core/src/enchantment_helper/item_in_use.rs` | 36 | — | 0 |
| `steel-core/src/enchantment_helper/selection.rs` | 187 | `third_offer_costs_at_least_twice_the_bookcases`, `available_results_pick_the_highest_level_whose_window_contains_the_value`, `books_accept_enchantments_that_are_not_primary_for_other_items` | 0 |
| `steel-core/src/enchantment_helper.rs` | 1653 | `unsupported_requirement_does_not_match_through_inversion`, `damage_enchantments_match_target_entity_tags`, `breach_modifies_armor_effectiveness`, `projectile_knockback_checks_direct_attacker_entity_tag`, `damage_source_properties_match_damage_type_tags`, `damage_protection_accumulates_across_equipment`, `damage_protection_ignores_enchantment_in_non_matching_slot`, `damage_protection_ignores_broken_equipment`, `damage_immunity_matches_equipment_slot_and_requirements`, `post_attack_ignite_applies_to_direct_melee_victim`, `projectile_spawned_effect_ignites_firework`, `post_attack_effects_match_enchantment_slot`, `post_attack_change_item_damage_calls_equipped_break_hook`, `lunge_post_piercing_requirements_and_effect_are_supported`, `post_attack_ignite_skips_indirect_damage_source`, `post_attack_mob_effect_matches_victim_predicate` | 0 |
| `steel-core/src/entity/ageable.rs` | 548 | `age_boundary_updates_synced_baby_flag`, `age_tick_grows_babies_and_reduces_positive_forced_age`, `age_lock_blocks_baby_growth`, `age_up_adds_forced_age_and_timer`, `age_up_applies_forced_age_when_reaching_adulthood`, `feeding_speed_up_seconds_matches_vanilla_integer_order` | 0 |
| `steel-core/src/entity/ai/control.rs` | 414 | `body_rotation_control_faces_body_forward_while_moving`, `body_rotation_control_turns_body_when_idle_head_moves`, `body_rotation_control_waits_then_turns_body_toward_stable_head`, `body_rotation_control_does_not_turn_idle_body_when_carrying_mob_passenger` | 0 |
| `steel-core/src/entity/ai/goal/avoid_entity.rs` | 217 | `avoid_entity_goal_uses_move_control`, `avoid_entity_default_selector_allows_non_player_living_entities`, `avoid_entity_goal_requires_world`, `avoid_entity_goal_sprints_when_close_to_avoided_entity`, `avoid_entity_goal_walks_when_far_from_avoided_entity` | 0 |
| `steel-core/src/entity/ai/goal/breath_air.rs` | 208 | `breath_air_goal_uses_move_and_look_controls`, `breath_air_goal_uses_vanilla_air_threshold`, `breath_air_goal_tick_applies_travel_input_to_velocity`, `air_search_bounds_match_vanilla_offsets`, `closed_box_scan_uses_vanilla_x_then_y_then_z_order` | 0 |
| `steel-core/src/entity/ai/goal/breed_goal.rs` | 153 | `breed_goal_uses_move_and_look_controls`, `breed_goal_requires_love_mode` | 0 |
| `steel-core/src/entity/ai/goal/climb_on_top_of_powder_snow.rs` | 130 | `climb_on_top_of_powder_snow_goal_uses_jump_control`, `climb_on_top_of_powder_snow_goal_requires_powder_snow_contact`, `climb_on_top_of_powder_snow_goal_uses_entity_type_tag_not_equipment_walkability`, `climb_on_top_of_powder_snow_goal_requires_world_after_tag_and_contact`, `climb_on_top_of_powder_snow_goal_ticks_jump_control` | 0 |
| `steel-core/src/entity/ai/goal/door_interact.rs` | 279 | `door_interact_goal_claims_no_controls_like_vanilla`, `door_interact_goal_requires_horizontal_collision`, `door_interact_path_scan_uses_current_node_and_one_ahead`, `door_interact_path_scan_uses_above_mob_as_fallback`, `door_interact_tick_marks_passed_after_crossing_door_plane`, `door_interact_can_use_requires_world_after_collision` | 0 |
| `steel-core/src/entity/ai/goal/eat_block_goal.rs` | 248 | `eat_block_goal_mirrors_fixed_animation_duration`, `eat_block_goal_turns_grass_into_dirt_and_runs_mob_ate`, `eat_block_goal_does_not_grief_grass_with_mob_griefing_disabled` | 0 |
| `steel-core/src/entity/ai/goal/flee_sun.rs` | 185 | `flee_sun_goal_uses_move_control`, `flee_sun_goal_requires_world`, `flee_sun_hide_pos_accepts_sheltered_negative_walk_target`, `flee_sun_hide_pos_rejects_sky_exposed_positions`, `flee_sun_hide_pos_rejects_non_negative_walk_targets` | 0 |
| `steel-core/src/entity/ai/goal/float_goal.rs` | 149 | `float_goal_sets_navigation_can_float`, `float_goal_uses_water_threshold_or_lava` | 0 |
| `steel-core/src/entity/ai/goal/follow_mob.rs` | 257 | `follow_mob_goal_uses_move_and_look_controls`, `follow_mob_goal_requires_world`, `follow_mob_goal_temporarily_removes_water_malus`, `follow_mob_goal_stops_when_no_navigation_is_running`, `follow_mob_goal_looks_at_following_mob` | 0 |
| `steel-core/src/entity/ai/goal/follow_parent.rs` | 119 | `follow_parent_goal_claims_no_controls_like_vanilla` | 0 |
| `steel-core/src/entity/ai/goal/hurt_by_target.rs` | 262 | `targets_attacker_and_alerts_unassigned_same_type_mobs` | 0 |
| `steel-core/src/entity/ai/goal/interact.rs` | 60 | `interact_goal_claims_look_and_move_controls_like_vanilla` | 0 |
| `steel-core/src/entity/ai/goal/leap_at_target.rs` | 181 | `leap_at_target_goal_uses_jump_and_move_controls`, `leap_at_target_goal_requires_target`, `leap_at_target_goal_uses_vanilla_distance_window`, `leap_at_target_goal_requires_ground`, `leap_at_target_goal_applies_vanilla_leap_velocity`, `leap_at_target_goal_continues_until_grounded` | 0 |
| `steel-core/src/entity/ai/goal/look_at_player.rs` | 247 | `look_at_player_goal_claims_only_look_control`, `look_at_player_goal_can_claim_custom_controls_for_vanilla_subclasses`, `look_at_player_goal_supports_selector_based_living_targets`, `look_at_player_goal_uses_vanilla_adjusted_look_time` | 0 |
| `steel-core/src/entity/ai/goal/melee_attack.rs` | 334 | `melee_attack_goal_uses_move_and_look_controls`, `melee_attack_goal_requires_world_to_start`, `melee_attack_goal_start_without_path_still_sets_aggressive`, `melee_attack_goal_stop_clears_aggression_and_navigation`, `melee_attack_goal_without_unseen_following_requires_active_navigation`, `melee_attack_goal_tick_recalculates_path_with_failed_move_penalty` | 0 |
| `steel-core/src/entity/ai/goal/merge_with_stone.rs` | 146 | `merge_constants_match_vanilla` | 0 |
| `steel-core/src/entity/ai/goal/mod.rs` | 75 | — | 0 |
| `steel-core/src/entity/ai/goal/move_to_block.rs` | 329 | `move_to_block_goal_uses_move_and_jump_controls`, `move_to_block_goal_requires_world_after_start_delay`, `move_to_block_goal_counts_down_next_start_tick_before_world_lookup`, `move_to_block_search_uses_vanilla_below_first_order`, `move_to_block_search_respects_vertical_search_start`, `move_to_block_goal_tracks_reached_target` | 0 |
| `steel-core/src/entity/ai/goal/move_towards_restriction.rs` | 81 | `move_towards_restriction_goal_uses_move_control`, `move_towards_restriction_goal_requires_outside_home` | 0 |
| `steel-core/src/entity/ai/goal/move_towards_target.rs` | 117 | `move_towards_target_goal_uses_move_control`, `move_towards_target_goal_requires_target`, `move_towards_target_goal_rejects_target_outside_range` | 0 |
| `steel-core/src/entity/ai/goal/nearest_attackable_target.rs` | 235 | `selects_nearest_living_entity_matching_selector`, `selector_can_reject_all_candidates` | 0 |
| `steel-core/src/entity/ai/goal/open_door.rs` | 106 | `open_door_goal_claims_no_controls_like_vanilla`, `open_door_goal_continue_requires_close_door_flag`, `open_door_goal_uses_vanilla_forget_time` | 0 |
| `steel-core/src/entity/ai/goal/panic_goal.rs` | 162 | `panic_goal_uses_move_control`, `panic_goal_uses_vanilla_panic_damage_tag` | 0 |
| `steel-core/src/entity/ai/goal/random_look_around.rs` | 170 | `random_look_around_sets_look_control_to_eye_height` | 0 |
| `steel-core/src/entity/ai/goal/random_pos.rs` | 348 | `random_direction_within_radians_matches_vanilla_seed`, `random_direction_within_radians_points_away_from_avoid_pos`, `random_direction_within_radians_rejects_points_outside_square` | 0 |
| `steel-core/src/entity/ai/goal/random_stroll.rs` | 104 | — | 0 |
| `steel-core/src/entity/ai/goal/random_swimming.rs` | 143 | `random_swimming_goal_uses_random_stroll_controls`, `random_swimmable_retry_accepts_first_pathfindable_pos`, `random_swimmable_retry_uses_next_candidate_until_pathfindable`, `random_swimmable_retry_returns_last_candidate_after_vanilla_retry_limit` | 0 |
| `steel-core/src/entity/ai/goal/ranged_attack.rs` | 164 | `ranged_attack_goal_uses_move_and_look_controls`, `attack_power_follows_the_vanilla_curve`, `zero_radius_never_divides_by_zero`, `see_time_threshold_matches_vanilla` | 0 |
| `steel-core/src/entity/ai/goal/restrict_open_door.rs` | 127 | `restrict_open_door_claims_no_controls_like_vanilla`, `restrict_open_door_requires_a_world`, `village_distance_matches_vanilla_radius` | 0 |
| `steel-core/src/entity/ai/goal/restrict_sun.rs` | 80 | `restrict_sun_goal_claims_no_controls_like_vanilla`, `restrict_sun_goal_requires_world`, `restrict_sun_goal_toggles_navigation_avoid_sun` | 0 |
| `steel-core/src/entity/ai/goal/selector.rs` | 692 | `lower_priority_goal_replaces_running_goal_for_same_control`, `non_interruptable_goal_blocks_replacement`, `disabled_control_stops_running_goal`, `tick_running_goals_respects_requires_update_every_tick`, `cleanup_stops_goal_that_can_no_longer_continue`, `running_panic_goal_is_visible_to_pathfinder_mob`, `remove_all_goals_stops_matching_running_goals_and_keeps_others`, `pathfinder_mob_remove_free_will_clears_all_goals` | 0 |
| `steel-core/src/entity/ai/goal/sleep_in_bed.rs` | 143 | — | 0 |
| `steel-core/src/entity/ai/goal/squid_flee_goal.rs` | 96 | — | 0 |
| `steel-core/src/entity/ai/goal/squid_random_movement_goal.rs` | 70 | — | 0 |
| `steel-core/src/entity/ai/goal/target_goal.rs` | 272 | `target_goal_base_continues_with_existing_mob_target`, `target_goal_base_restores_stored_target_while_continuing`, `target_goal_base_forgets_unseen_target_after_memory_ticks`, `target_goal_base_stop_clears_mob_and_stored_target`, `target_goal_base_can_attack_requires_world`, `target_goal_base_caches_unreachable_targets` | 0 |
| `steel-core/src/entity/ai/goal/tempt_goal.rs` | 212 | `tempt_goal_uses_move_and_look_controls`, `tempt_goal_should_follow_checks_both_hands` | 0 |
| `steel-core/src/entity/ai/goal/try_find_water.rs` | 161 | `try_find_water_goal_claims_no_controls_like_vanilla`, `try_find_water_goal_requires_on_ground`, `try_find_water_goal_requires_world_after_on_ground_check`, `water_search_bounds_match_vanilla_offsets`, `closed_box_scan_uses_vanilla_x_then_y_then_z_order` | 0 |
| `steel-core/src/entity/ai/goal/water_avoiding_random_stroll.rs` | 65 | — | 0 |
| `steel-core/src/entity/ai/mod.rs` | 15 | — | 0 |
| `steel-core/src/entity/ai/navigation.rs` | 1554 | `path_navigation_tracks_can_float_flag`, `path_navigation_tracks_can_open_doors_flag`, `path_navigation_tracks_can_walk_over_fences_flag`, `path_navigation_tracks_avoid_sun_flag`, `path_navigation_tracks_can_path_to_targets_below_surface_flag`, `avoid_sun_trims_path_before_first_sky_node`, `avoid_sun_keeps_path_when_mob_is_already_under_sky`, `move_to_trims_path_over_cauldrons`, `move_to_trims_avoid_sun_path`, `standing_in_water_still_retires_the_waypoint_underfoot`, `move_to_path_targets_next_node_after_current_node_is_reached`, `path_navigation_stops_after_final_node_is_reached`, `path_navigation_targets_next_node_when_mob_passed_current_node_directionally`, `path_navigation_does_not_cut_corner_through_walkable_door`, `path_navigation_without_update_does_not_advance_normally`, `path_navigation_without_update_advances_when_airborne_over_node`, `path_navigation_stops_when_stationary_past_stuck_interval`, `path_navigation_times_out_when_same_node_takes_too_long`, `path_recompute_request_delays_during_vanilla_cooldown`, `path_recompute_request_waits_until_path_can_update`, `stop_keeps_delayed_recompute_state`, `path_should_recompute_uses_vanilla_midpoint_window`, `move_to_same_path_keeps_current_progress`, `path_navigation_reuses_current_path_for_matching_target`, `create_path_reuses_current_path_for_matching_target`, `path_navigation_does_not_reuse_current_path_for_different_target`, `path_navigation_updates_current_speed_modifier`, `direct_target_stops_when_reached`, `create_path_finds_walkable_target_with_cached_navigation_state` | 0 |
| `steel-core/src/entity/ai/node.rs` | 531 | `node_hash_matches_vanilla_packing_shape`, `node_store_reuses_nodes_by_vanilla_hash`, `node_store_resets_search_state_without_losing_path_type_cost`, `node_heap_pops_lowest_f_cost_first`, `node_heap_change_cost_reorders_existing_node`, `target_tracks_best_node_hash` | 0 |
| `steel-core/src/entity/ai/path.rs` | 620 | `default_malus_matches_vanilla_path_types`, `malus_overrides_are_indexed_by_path_type`, `path_type_set_is_bit_indexed_by_vanilla_path_type_order`, `path_type_cache_uses_vanilla_direct_mapped_size`, `path_tracks_progress_and_target_distance`, `path_same_as_compares_vanilla_node_identity`, `path_type_cache_invalidates_matching_position`, `pathfinding_context_uses_cache_when_supplied` | 0 |
| `steel-core/src/entity/ai/pathfinder.rs` | 514 | `pathfinder_finds_direct_walkable_path`, `pathfinder_returns_closest_path_when_target_is_not_reached` | 0 |
| `steel-core/src/entity/ai/sensing.rs` | 93 | `sensing_caches_seen_targets_until_tick`, `sensing_caches_unseen_targets_until_tick` | 0 |
| `steel-core/src/entity/ai/targeting.rs` | 245 | `sight_check_applies_to_a_mob_that_does_not_pathfind` | 0 |
| `steel-core/src/entity/ai/walk/collision.rs` | 14 | — | 0 |
| `steel-core/src/entity/ai/walk/mod.rs` | 32 | — | 0 |
| `steel-core/src/entity/ai/walk/node_evaluator.rs` | 730 | — | 0 |
| `steel-core/src/entity/ai/walk/path_evaluator.rs` | 169 | — | 0 |
| `steel-core/src/entity/ai/walk/settings.rs` | 240 | — | 0 |
| `steel-core/src/entity/ai/walk/tests.rs` | 640 | `mob_path_settings_reads_can_open_doors_from_navigation`, `mob_path_settings_reads_can_walk_over_fences_from_navigation`, `path_type_from_state_matches_core_vanilla_special_cases`, `doors_use_vanilla_mob_interactable_door_tag`, `block_state_pathfindable_uses_behavior_overrides`, `walk_node_evaluator_applies_vanilla_door_adjustments_for_mobs`, `walk_node_evaluator_marks_rails_unpassable_when_mob_is_not_on_rails`, `large_walk_node_evaluator_caps_nearby_danger_cost_like_vanilla`, `walk_node_evaluator_floor_level_uses_collision_shape_below_node`, `get_start_uses_grounded_mob_block_position`, `get_start_floats_to_top_water_node_when_mob_can_float`, `get_start_scans_down_to_first_ground_when_airborne`, `get_start_uses_first_startable_bounding_box_corner`, `find_accepted_node_records_walkable_node_cost`, `find_accepted_node_falls_to_ground_when_within_max_fall_distance`, `find_accepted_node_blocks_falls_past_max_fall_distance`, `find_accepted_node_keeps_last_water_node_before_non_water`, `find_accepted_node_rejects_partial_collision_when_reach_is_blocked`, `get_neighbors_expands_all_cardinal_and_diagonal_nodes_on_flat_ground`, `get_neighbors_rejects_diagonals_through_walkable_doors`, `open_air_above_solid_ground_becomes_walkable`, `walkable_ground_adjacent_to_water_becomes_water_border` | 0 |
| `steel-core/src/entity/animal.rs` | 517 | `animal_spawn_rules_require_spawnable_block_tag`, `animal_spawn_rules_require_raw_brightness_above_eight`, `animal_spawn_rules_trial_spawner_ignores_light` | 0 |
| `steel-core/src/entity/attribute.rs` | 1024 | `attribute_supplier_is_cached_per_entity_type`, `all_generated_entity_default_attributes_resolve`, `player_gravity_is_initialized_from_default_attributes`, `vanilla_nbt_only_contains_permanent_modifiers`, `removing_all_modifiers_preserves_base_value`, `assigning_base_values_only_changes_attributes_present_in_both_maps`, `assigning_permanent_modifiers_adds_only_permanent_source_modifiers`, `assigning_permanent_modifiers_rejects_duplicate_ids` | 0 |
| `steel-core/src/entity/base/fire_freeze.rs` | 89 | — | 0 |
| `steel-core/src/entity/base/mod.rs` | 1810 | — | 0 |
| `steel-core/src/entity/base/movement.rs` | 342 | — | 0 |
| `steel-core/src/entity/base/persistence.rs` | 96 | — | 0 |
| `steel-core/src/entity/base/relationships.rs` | 108 | — | 0 |
| `steel-core/src/entity/base/tests.rs` | 1337 | `generation_distinguishes_reconstructed_entity_identity`, `piston_movement_is_limited_per_axis_per_tick`, `piston_movement_resets_each_game_tick`, `piston_movement_uses_first_non_zero_axis`, `piston_movement_keeps_sub_threshold_movement`, `collision_flags_clear_without_changing_ground_state`, `movement_emission_flags_match_vanilla_variants`, `movement_progress_accumulates_vanilla_step_and_fly_distance`, `base_tick_count_advances_like_vanilla_entity_tick_count`, `movement_flags_can_preserve_vertical_state_for_client_authored_horizontal_moves`, `lifecycle_state_tracks_removal`, `removal_unlocks_lifecycle_before_destroying_retained_sources`, `lifecycle_state_tracks_pending_world_change_tokens`, `killed_player_can_reserve_respawn_transition`, `try_set_position_rolls_back_when_commit_fails`, `set_position_local_panics_when_callback_requires_manager_commit`, `dimension_change_notifies_the_level_callback_of_new_bounds`, `base_state_caches_fluid_contact`, `fire_freeze_state_applies_inside_block_effects`, `fire_ignite_respects_remaining_fire_tick_cap`, `fire_ignite_respects_vanilla_cooldown_shape`, `base_tick_advances_powder_snow_and_fire_state`, `fire_freeze_state_round_trips_through_base_load`, `no_physics_is_stored_on_base_state`, `relationship_state_tracks_direct_vehicle_and_passengers`, `relationship_queries_follow_indirect_vehicle_chain`, `removal_cleans_up_relationship_state`, `base_fall_damage_propagates_to_passengers`, `physics_state_uses_current_base_bounding_box`, `old_position_is_explicit_movement_trace_state`, `set_velocity_ignores_non_finite_updates_like_vanilla`, `set_rotation_wraps_yaw_and_clamps_pitch_like_vanilla_snap`, `with_rotation_initializes_old_rotation_to_current_rotation`, `old_rotation_is_base_tick_snapshot_state`, `set_position_rejects_non_finite_values`, `set_old_position_rejects_non_finite_values`, `set_rotation_rejects_non_finite_values`, `known_speed_is_base_tick_position_delta`, `base_tick_state_decrements_boarding_cooldown`, `portal_cooldown_tick_decrements_portal_cooldown`, `base_tick_state_does_not_decrement_portal_cooldown`, `active_portal_process_reuses_same_portal_after_tick_is_consumed`, `active_portal_process_restarts_for_different_portal_kind`, `entity_tags_respect_vanilla_limit`, `movement_trace_falls_back_to_old_position_when_no_moves_were_recorded`, `movement_trace_replays_last_finalized_movements`, `movement_trace_appends_direct_position_change_after_recorded_moves`, `movement_trace_removes_latest_movement_recording`, `movement_trace_compacts_oldest_moves_at_vanilla_limit`, `fall_distance_is_stored_on_base_state`, `fall_distance_accumulation_uses_vanilla_float_cast`, `base_tick_lava_contact_dampens_fall_distance`, `base_tick_water_contact_resets_fall_distance`, `water_reset_runs_before_lava_fall_distance_damping`, `stuck_speed_multiplier_resets_fall_distance_and_applies_once`, `stuck_speed_multiplier_can_be_consumed_without_applying_for_pistons` | 0 |
| `steel-core/src/entity/block_attached_entity.rs` | 246 | — | 0 |
| `steel-core/src/entity/block_effects.rs` | 660 | `visited_positions_deduplicate_across_inline_capacity_boundary`, `entity_inside_shape_uses_swept_entity_center_against_inflated_shape`, `stationary_entity_inside_shape_uses_current_entity_box`, `stationary_trace_visits_target_aabb_blocks`, `horizontal_trace_includes_blocks_between_start_and_target`, `visitor_can_stop_trace`, `long_trace_visits_each_block_once_in_order` | 0 |
| `steel-core/src/entity/bucketable.rs` | 188 | — | 0 |
| `steel-core/src/entity/callback.rs` | 288 | — | 0 |
| `steel-core/src/entity/combat_rules.rs` | 65 | `magic_absorb_clamps_protection_to_vanilla_range` | 0 |
| `steel-core/src/entity/consume_effect/apply_effects.rs` | 30 | — | 0 |
| `steel-core/src/entity/consume_effect/clear_all_effects.rs` | 21 | — | 0 |
| `steel-core/src/entity/consume_effect/mod.rs` | 42 | — | 0 |
| `steel-core/src/entity/consume_effect/play_sound.rs` | 34 | — | 0 |
| `steel-core/src/entity/consume_effect/remove_effects.rs` | 25 | — | 0 |
| `steel-core/src/entity/consume_effect/teleport_randomly.rs` | 178 | `teleport_randomly_leaves_the_player_in_place_with_no_valid_landing`, `teleport_randomly_lands_on_solid_ground_within_diameter` | 0 |
| `steel-core/src/entity/conversion.rs` | 757 | `single_conversion_transfers_shared_state_and_discards_source`, `single_conversion_reboards_the_root_passenger`, `single_conversion_moves_equipment_before_the_callback`, `split_on_death_conversion_keeps_the_source_living`, `convert_to_fails_without_a_registered_factory`, `refused_conversion_keeps_the_source_passenger_mounted`, `refused_conversion_keeps_the_source_on_its_vehicle`, `refused_conversion_keeps_the_source_leash`, `convert_to_refuses_hostile_types_on_peaceful_difficulty`, `conversion_skips_equipment_when_not_requested`, `after_conversion_callback_receives_the_new_mob` | 0 |
| `steel-core/src/entity/damage/history/binding.rs` | 118 | — | 0 |
| `steel-core/src/entity/damage/history/source.rs` | 126 | — | 0 |
| `steel-core/src/entity/damage/history/tests.rs` | 133 | `age_expiry_unlocks_records_before_destroying_retained_entities`, `ownership_sweep_unlocks_records_before_destroying_retained_entities`, `victim_lifetime_expiry_unlocks_records_before_destroying_retained_entities` | 0 |
| `steel-core/src/entity/damage/history.rs` | 115 | — | 0 |
| `steel-core/src/entity/damage/mod.rs` | 11 | — | 0 |
| `steel-core/src/entity/damage/source.rs` | 173 | — | 0 |
| `steel-core/src/entity/damage/tests/mod.rs` | 282 | `removed_sources_keep_exact_entities_when_the_world_reuses_their_ids`, `directness_uses_allocation_identity_and_position_tracks_the_direct_entity`, `returned_self_damage_source_outlives_history_expiry_without_a_cycle`, `unmanaged_mutual_damage_does_not_retain_players_or_worlds`, `replacing_damage_history_releases_the_previous_source`, `returned_source_survives_history_clear_and_owner_teardown`, `projectile_callback_can_build_a_source_after_its_removal`, `successful_damage_uses_the_victims_clock_instead_of_the_supplied_world` | 0 |
| `steel-core/src/entity/damage/tests/retention.rs` | 252 | `removed_history_resolves_shooter_after_projectile_is_released_without_rebinding`, `active_history_retains_removed_attacker_but_not_its_removed_attacker`, `final_removal_cannot_be_undone_by_a_late_history_write`, `world_teardown_releases_self_history_on_the_frozen_ownership_sweep`, `chunk_recovery_preserves_history_until_final_unload`, `weak_history_preserves_live_position_and_explicit_position_semantics` | 0 |
| `steel-core/src/entity/dismount_helper.rs` | 223 | — | 0 |
| `steel-core/src/entity/entities/mobs/ambient/bat.rs` | 535 | `bat_starts_resting_and_round_trips_flags`, `bat_is_flapping_on_the_vanilla_tick_cadence` | 0 |
| `steel-core/src/entity/entities/mobs/ambient/mod.rs` | 7 | — | 0 |
| `steel-core/src/entity/entities/mobs/bosses/ender_dragon/flight_history.rs` | 135 | `a_fresh_history_reports_zeroed_samples`, `the_first_record_fills_the_whole_buffer`, `samples_are_read_back_by_delay_and_wrap_around` | 0 |
| `steel-core/src/entity/entities/mobs/bosses/ender_dragon/mod.rs` | 1444 | — | 0 |
| `steel-core/src/entity/entities/mobs/bosses/ender_dragon/part.rs` | 423 | `part_sizes_match_vanilla_sub_entities`, `only_the_head_and_neck_avoid_the_part_damage_penalty`, `part_boxes_use_the_vanilla_entity_box_shape`, `a_segment_reports_the_nearest_part_it_crosses`, `a_segment_starting_inside_a_part_hits_it_immediately`, `wing_sweep_boxes_grow_and_drop_like_vanilla` | 0 |
| `steel-core/src/entity/entities/mobs/bosses/ender_dragon/phase_ticks.rs` | 1124 | `the_podium_distance_is_the_largest_axis_delta`, `a_roll_from_the_step_float_stays_inside_the_bound`, `every_sitting_phase_reports_sitting_and_the_rest_do_not`, `the_head_look_vector_is_unit_length_for_every_phase` | 0 |
| `steel-core/src/entity/entities/mobs/bosses/ender_dragon/phases.rs` | 1140 | `phase_ids_are_vanilla_registry_order`, `an_unknown_phase_id_falls_back_to_the_holding_pattern`, `sitting_and_fly_speeds_match_the_vanilla_phase_classes`, `beginning_the_flaming_phase_counts_an_attack`, `beginning_a_phase_clears_the_path_and_target_but_keeps_the_direction`, `adjacency_rows_are_symmetric`, `adjacency_never_references_a_node_outside_the_arena`, `the_node_rings_match_vanilla_radii`, `a_manager_starts_in_the_hover_phase`, `switching_to_a_new_phase_resets_its_state_once`, `the_pathfinder_reaches_an_adjacent_node`, `a_path_starts_at_the_start_and_ends_at_the_goal`, `the_final_node_becomes_the_path_tail`, `an_already_reached_goal_is_a_single_node_path`, `the_open_set_pops_the_cheapest_node_first`, `the_open_set_keeps_insertion_order_on_equal_costs` | 0 |
| `steel-core/src/entity/entities/mobs/bosses/mod.rs` | 11 | — | 0 |
| `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs` | 403 | `spawn_phase_matches_vanilla_numbers`, `begin_spawn_phase_sets_the_invulnerability_timer` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/blaze.rs` | 254 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/bogged.rs` | 420 | `fresh_bogged_is_not_sheared_and_shearing_is_one_shot` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/breeze.rs` | 291 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs` | 300 | `cave_spider_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs` | 546 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/creaking.rs` | 369 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/creeper.rs` | 516 | `fuse_accumulates_and_deflates_without_going_negative`, `explosion_power_doubles_when_powered`, `swell_window_matches_vanilla_swell_goal`, `synced_flags_round_trip_through_the_entity_data_layer` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/cube_mob.rs` | 224 | `size_derived_values_match_vanilla_formulas`, `splits_only_halve_cubes_bigger_than_one`, `split_offsets_use_half_the_dying_width`, `random_spawn_sizes_stay_inside_the_natural_range`, `jump_delays_follow_the_goal_and_target_state`, `split_count_is_between_two_and_four` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/drowned.rs` | 294 | `drowned_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs` | 341 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/enderman.rs` | 467 | `gaze_test_matches_forward_and_away_vectors`, `enderman_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/endermite.rs` | 419 | `endermite_discards_after_max_lifetime_unless_persistence_required`, `endermite_nbt_round_trip` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/evoker.rs` | 254 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/ghast.rs` | 254 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/giant.rs` | 241 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/guardian.rs` | 304 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/hoglin.rs` | 298 | `hoglin_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/husk.rs` | 294 | `husk_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/illusioner.rs` | 254 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs` | 346 | `iron_golem_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs` | 584 | `size_drives_health_armor_damage_and_fire_immunity`, `size_is_clamped_to_the_vanilla_range`, `saved_size_is_zero_based_and_keeps_health` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/mod.rs` | 134 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/parched.rs` | 366 | `parched_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/phantom.rs` | 433 | `phantom_phase_cycle_matches_vanilla`, `phantom_flight_defaults_match_vanilla` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/piglin.rs` | 299 | `piglin_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs` | 296 | `piglin_brute_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/pillager.rs` | 435 | `pillager_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/ravager.rs` | 254 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/shulker.rs` | 297 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/silverfish.rs` | 301 | `silverfish_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/skeleton.rs` | 382 | `skeleton_starts_unconverted` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/slime.rs` | 648 | `size_drives_health_speed_damage_and_hitbox`, `size_is_clamped_to_the_vanilla_range`, `dying_slime_splits_into_smaller_slimes_in_the_world`, `saved_size_is_zero_based_and_keeps_health` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs` | 258 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/spider.rs` | 298 | `spider_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/stray.rs` | 365 | `stray_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/vex.rs` | 254 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/vindicator.rs` | 299 | `vindicator_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/warden.rs` | 375 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/witch.rs` | 254 | — | 0 |
| `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs` | 304 | `wither_skeleton_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/zoglin.rs` | 298 | `zoglin_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/zombie.rs` | 294 | `zombie_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs` | 466 | `zombie_villager_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs` | 309 | `zombified_piglin_uses_registry_attributes_and_round_trips_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/jockeys/mod.rs` | 1 | — | 0 |
| `steel-core/src/entity/entities/mobs/mod.rs` | 8 | — | 0 |
| `steel-core/src/entity/entities/mobs/neutral/mod.rs` | 6 | — | 0 |
| `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs` | 391 | `an_adult_stands_when_a_player_is_near_a_cub`, `a_cub_never_stands`, `polar_bears_eat_nothing` | 0 |
| `steel-core/src/entity/entities/mobs/passive/allay.rs` | 510 | `allay_flags_are_independent` | 0 |
| `steel-core/src/entity/entities/mobs/passive/armadillo.rs` | 413 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/axolotl.rs` | 516 | `new_axolotls_are_lucy_and_alive_on_land`, `variants_round_trip_and_clamp`, `play_dead_and_bucket_flags_are_independent`, `baby_axolotls_are_half_sized` | 0 |
| `steel-core/src/entity/entities/mobs/passive/bee.rs` | 301 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/camel.rs` | 535 | `camel_tame_and_saddle_bits_are_independent` | 0 |
| `steel-core/src/entity/entities/mobs/passive/camel_husk.rs` | 384 | `camel_husk_tame_and_saddle_bits_are_independent` | 0 |
| `steel-core/src/entity/entities/mobs/passive/cat/mod.rs` | 684 | `new_cats_start_untamed_with_a_red_collar`, `taming_sets_the_tame_flag_and_an_owner`, `sitting_toggles_independently_of_taming`, `collar_colour_round_trips_through_ids`, `lying_and_relax_flags_are_independent`, `raw_cod_and_salmon_are_cat_food`, `variants_and_sounds_round_trip_through_nbt` | 0 |
| `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs` | 689 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/chicken/tests/core.rs` | 229 | `chicken_initializes_vanilla_living_attributes_and_health`, `chicken_uses_vanilla_chicken_food_tag`, `chicken_sound_methods_follow_selected_sound_variant`, `chicken_sounds_switch_to_baby_set_for_babies`, `chicken_jockey_rewards_ten_experience`, `chicken_jockey_does_not_despawn_far_away`, `chicken_breeding_offspring_inherits_parent_variant`, `chicken_finalize_spawn_assigns_registered_variant_and_sound_variant`, `chicken_egg_timer_decrements_only_for_adult_non_jockeys`, `chicken_resets_egg_timer_within_vanilla_range_after_laying`, `chicken_slow_fall_reduces_downward_velocity` | 0 |
| `steel-core/src/entity/entities/mobs/passive/chicken/tests/mod.rs` | 18 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/chicken/tests/persistence.rs` | 82 | `chicken_saves_vanilla_state_and_variant_data`, `chicken_loads_vanilla_state_and_variant_data`, `chicken_load_defaults_jockey_and_keeps_random_egg_time_when_absent` | 0 |
| `steel-core/src/entity/entities/mobs/passive/cow/mod.rs` | 502 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/cow/tests/core.rs` | 238 | `cow_initializes_vanilla_living_attributes_and_health`, `cow_uses_vanilla_cow_food_tag`, `cow_sound_methods_follow_selected_sound_variant`, `cow_milks_bucket_into_milk_bucket_for_adults`, `cow_does_not_milk_when_baby`, `cow_breeding_offspring_inherits_parent_variant`, `try_as_dyn_exposes_cow_living_entity_behavior`, `try_as_dyn_exposes_cow_pathfinder_mob_behavior`, `try_as_dyn_exposes_cow_animal_behavior`, `cow_ambient_interval_and_source_match_vanilla_animal_defaults`, `cow_finalize_spawn_assigns_registered_variant_and_sound_variant` | 0 |
| `steel-core/src/entity/entities/mobs/passive/cow/tests/mod.rs` | 19 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/cow/tests/persistence.rs` | 58 | `cow_saves_vanilla_age_and_variant_data`, `cow_loads_vanilla_age_and_variant_data` | 0 |
| `steel-core/src/entity/entities/mobs/passive/dolphin.rs` | 492 | `new_dolphins_start_moist_and_unrewarded`, `got_fish_flag_round_trips`, `moisture_level_round_trips_through_nbt`, `raw_cod_is_dolphin_food` | 0 |
| `steel-core/src/entity/entities/mobs/passive/fish/cod.rs` | 286 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/fish/mod.rs` | 210 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs` | 792 | `new_nautilus_is_not_dashing`, `an_unsaddled_nautilus_refuses_a_riders_jump`, `a_fresh_nautilus_has_no_dash_cooldown`, `a_riders_jump_starts_the_vanilla_cooldown`, `the_dash_ends_five_ticks_before_the_cooldown_does`, `the_cooldown_must_run_out_before_another_dash`, `new_zombie_nautilus_is_not_dashing`, `nautilus_movement_speed_matches_vanilla` | 0 |
| `steel-core/src/entity/entities/mobs/passive/fish/pufferfish.rs` | 478 | `puff_states_are_vanilla_constants`, `puff_scales_match_vanilla`, `sting_damage_scales_with_puff_state` | 0 |
| `steel-core/src/entity/entities/mobs/passive/fish/salmon.rs` | 398 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/fish/tadpole.rs` | 380 | `tadpoles_spawn_as_babies`, `water_animal_constants_match_vanilla` | 0 |
| `steel-core/src/entity/entities/mobs/passive/fish/tests.rs` | 99 | `spawn_depth_window_matches_vanilla_water_animal`, `air_supply_cap_matches_vanilla_water_animal`, `flop_impulse_uses_fixed_upward_velocity`, `water_drag_matches_vanilla_fish_travel`, `water_drag_keeps_vanilla_constants`, `salmon_variant_pool_matches_vanilla_weights`, `salmon_variant_defaults_and_scales_match_vanilla`, `salmon_variant_round_trips_through_synced_id_and_name` | 0 |
| `steel-core/src/entity/entities/mobs/passive/fish/tropical_fish.rs` | 498 | `default_variant_is_kob_white_white`, `packing_round_trips_every_pattern_and_colour`, `large_patterns_keep_their_base_size_bit` | 0 |
| `steel-core/src/entity/entities/mobs/passive/fox/mod.rs` | 589 | `coat_variants_round_trip`, `flag_bits_are_set_and_cleared_independently`, `feeding_sweet_berries_makes_the_fox_trust_the_player`, `only_two_players_stay_trusted`, `sweet_berries_are_fox_food` | 0 |
| `steel-core/src/entity/entities/mobs/passive/frog.rs` | 418 | `frogs_default_to_temperate_and_have_no_tongue_target`, `variants_round_trip_through_keys`, `tongue_targets_round_trip`, `only_slime_balls_are_frog_food`, `frogs_are_small_but_do_not_shrink_when_babies` | 0 |
| `steel-core/src/entity/entities/mobs/passive/goat.rs` | 305 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs` | 324 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs` | 510 | `donkey_tame_and_saddle_bits_are_independent` | 0 |
| `steel-core/src/entity/entities/mobs/passive/horse/horse.rs` | 568 | `horse_tame_and_saddle_bits_are_independent`, `horse_ridden_speed_uses_movement_speed_attribute` | 0 |
| `steel-core/src/entity/entities/mobs/passive/horse/llama.rs` | 522 | `llama_tame_and_saddle_bits_are_independent` | 0 |
| `steel-core/src/entity/entities/mobs/passive/horse/mod.rs` | 17 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/horse/mule.rs` | 516 | `mule_tame_and_saddle_bits_are_independent` | 0 |
| `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs` | 374 | `skeleton_horse_tame_and_saddle_bits_are_independent` | 0 |
| `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs` | 405 | `trader_llama_tame_and_saddle_bits_are_independent` | 0 |
| `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs` | 362 | `zombie_horse_tame_and_saddle_bits_are_independent` | 0 |
| `steel-core/src/entity/entities/mobs/passive/mod.rs` | 73 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs` | 702 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/mooshroom/tests/mod.rs` | 264 | `mooshroom_initializes_vanilla_living_attributes_and_health`, `mooshroom_milks_bowl_into_mushroom_stew`, `mooshroom_milks_bucket_into_milk_bucket`, `brown_mooshroom_eats_flower_and_gives_suspicious_stew`, `mooshroom_thunder_hit_swaps_variant`, `mooshroom_shearing_converts_to_cow_and_drops_mushrooms`, `mooshroom_nbt_persistence_roundtrip` | 0 |
| `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs` | 383 | `trusting_starts_false_and_flips`, `raw_cod_is_ocelot_food` | 0 |
| `steel-core/src/entity/entities/mobs/passive/panda.rs` | 305 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs` | 505 | `coat_variants_round_trip_and_clamp`, `parrots_fly_unless_riding`, `imitation_sounds_exist_for_vanilla_mobs`, `seeds_tempt_parrots`, `imitation_delay_matches_the_vanilla_range` | 0 |
| `steel-core/src/entity/entities/mobs/passive/pig/mod.rs` | 560 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/pig/tests/ai_age.rs` | 283 | `pig_breeding_offspring_inherits_parent_variant`, `entity_check_despawn_discards_in_peaceful`, `pig_mob_ai_increments_no_action_time`, `pig_damage_resets_no_action_time`, `pig_keeps_vanilla_animal_far_away_persistence`, `pig_registers_vanilla_passive_goal_foundations`, `pig_path_target_feeds_move_control_forward_input`, `pig_age_updates_synchronized_baby_flag_on_boundary`, `pig_age_boundary_refreshes_dimensions`, `pig_growth_skips_position_fudging_until_after_first_tick`, `pig_scale_attribute_refreshes_dimensions` | 0 |
| `steel-core/src/entity/entities/mobs/passive/pig/tests/animal_lifecycle.rs` | 216 | `pig_uses_vanilla_animal_fire_path_malus`, `pig_uses_mob_passenger_as_controller_when_not_player_controlled`, `pig_uses_vanilla_pig_food_tag`, `pig_saves_vanilla_animal_love_data`, `pig_loads_vanilla_animal_love_data`, `pig_animal_love_ticks_only_for_adults`, `pig_damage_resets_vanilla_animal_love_time`, `pig_death_tick_removes_after_vanilla_death_duration`, `breeding_drops_a_single_experience_orb` | 0 |
| `steel-core/src/entity/entities/mobs/passive/pig/tests/core.rs` | 442 | `pig_initializes_vanilla_living_attributes_and_health`, `try_as_dyn_exposes_pig_living_entity_behavior`, `try_as_dyn_exposes_pig_pathfinder_mob_behavior`, `try_as_dyn_exposes_pig_mob_behavior`, `try_as_dyn_exposes_pig_animal_behavior`, `try_as_dyn_exposes_pig_item_steerable_behavior`, `pig_item_steerable_boost_updates_synced_total_once`, `pig_ridden_speed_uses_item_steering_boost_factor`, `pig_ridden_rotation_matches_controller_head_and_body_yaw`, `pig_can_mate_with_same_type_when_both_in_love`, `pig_uses_default_animal_love_mode`, `pig_saddle_slot_requires_alive_adult`, `pig_dispenser_can_equip_saddle_only_when_alive_adult_and_empty`, `pig_living_is_baby_uses_ageable_state`, `pig_saddled_state_reads_saddle_equipment`, `pig_saddle_equip_sound_uses_vanilla_sound`, `pig_hurt_and_death_sounds_use_current_sound_variant`, `pig_ambient_sound_uses_current_sound_variant`, `pig_uses_vanilla_animal_experience_reward`, `pig_baby_and_consumed_experience_follow_living_rules`, `mob_guaranteed_drop_marks_slot_preserved`, `mob_death_loot_without_world_keeps_preserved_equipment` | 0 |
| `steel-core/src/entity/entities/mobs/passive/pig/tests/mod.rs` | 65 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/pig/tests/persistence.rs` | 300 | `pig_saves_vanilla_mob_age_and_variant_data`, `pig_loads_vanilla_mob_age_and_variant_data`, `pig_saves_delayed_fence_knot_leash_as_vanilla_block_pos_int_array`, `pig_saves_live_fence_knot_leash_as_vanilla_block_pos_int_array`, `pig_loads_delayed_fence_knot_leash_from_vanilla_block_pos_int_array`, `pig_drop_leash_clears_live_leash_state`, `pig_remove_leash_clears_live_leash_state`, `pig_drop_all_leash_connections_clears_own_live_leash`, `pig_saves_and_loads_saddle_equipment` | 0 |
| `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs` | 446 | `variant_names_round_trip_through_the_entity`, `variant_ids_are_clamped_to_the_vanilla_range`, `carrots_are_rabbit_food` | 0 |
| `steel-core/src/entity/entities/mobs/passive/sheep/color_spawn_rules.rs` | 86 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs` | 532 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/sheep/tests/core.rs` | 452 | `sheep_initializes_vanilla_living_attributes_and_health`, `sheep_uses_vanilla_sheep_food_tag`, `sheep_wool_color_starts_white_and_unsheared`, `sheep_color_and_sheared_state_share_the_vanilla_wool_byte`, `sheep_sound_methods_follow_vanilla_sheep_sounds`, `sheep_shear_drops_wool_and_damages_shears`, `sheep_shear_interaction_is_consumed_when_not_ready`, `sheep_breeding_mixes_parent_colors_through_dye_recipes`, `sheep_breeding_falls_back_to_a_parent_color_without_a_mix_recipe`, `sheep_shearing_drop_spawns_one_item_entity_per_count_unit`, `dye_item_dyes_an_unsheared_sheep_and_consumes_the_dye`, `dye_item_passes_for_sheared_or_matching_color_sheep`, `sheep_ate_regrows_wool_and_speeds_up_baby_ageing`, `sheep_ate_keeps_adult_age_unchanged`, `sheep_shear_loot_resolves_the_matching_color_table` | 0 |
| `steel-core/src/entity/entities/mobs/passive/sheep/tests/mod.rs` | 19 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/sheep/tests/persistence.rs` | 36 | `sheep_saves_vanilla_wool_color_and_sheared_state`, `sheep_loads_vanilla_wool_color_and_sheared_state` | 0 |
| `steel-core/src/entity/entities/mobs/passive/sniffer.rs` | 305 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/squid/glow.rs` | 459 | `new_glow_squids_start_with_the_dark_counter`, `reaching_zero_dark_ticks_starts_the_glow`, `glow_squids_use_the_glow_squid_sounds` | 0 |
| `steel-core/src/entity/entities/mobs/passive/squid/mod.rs` | 511 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/squid/tests.rs` | 23 | `rotate_vector_trails_ink_behind_a_pitched_squid` | 0 |
| `steel-core/src/entity/entities/mobs/passive/strider.rs` | 305 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs` | 319 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/turtle.rs` | 301 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/villager.rs` | 372 | — | 0 |
| `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs` | 447 | `the_timer_holds_while_a_player_is_trading`, `an_untraded_trader_counts_down`, `a_command_summoned_trader_never_despawns` | 0 |
| `steel-core/src/entity/entities/mobs/passive/wolf.rs` | 404 | — | 0 |
| `steel-core/src/entity/entities/mod.rs` | 42 | — | 0 |
| `steel-core/src/entity/entities/objects/decoration/armor_stand.rs` | 662 | `default_poses_match_vanilla`, `small_stands_are_half_height`, `flags_can_be_combined_and_cleared`, `saving_writes_the_vanilla_keys`, `loading_restores_flags_and_poses`, `marker_stands_are_not_pickable_and_not_interactable` | 0 |
| `steel-core/src/entity/entities/objects/decoration/cushion.rs` | 464 | `cushion_stays_while_its_support_is_solid`, `removing_the_support_drops_the_coloured_item_once`, `hitting_a_cushion_drops_its_item_only_once`, `interacting_makes_the_player_sit_on_the_cushion`, `sneaking_does_not_sit_but_passes`, `the_item_stack_carries_the_cushion_colour`, `every_colour_has_its_own_item_and_round_trips`, `saving_writes_the_colour_name_and_defaults_stay_white` | 0 |
| `steel-core/src/entity/entities/objects/decoration/display.rs` | 486 | `billboard_ordinals_round_trip`, `item_displays_store_their_stack`, `text_display_flags_toggle_independently`, `text_display_defaults_match_vanilla` | 0 |
| `steel-core/src/entity/entities/objects/decoration/mannequin.rs` | 182 | — | 0 |
| `steel-core/src/entity/entities/objects/decoration/mod.rs` | 15 | — | 0 |
| `steel-core/src/entity/entities/objects/decoration/painting.rs` | 687 | `the_bounding_box_matches_the_variant_size`, `a_painting_only_stays_with_a_solid_wall`, `removing_the_wall_drops_the_painting_item`, `dropping_the_painting_twice_does_not_duplicate_it`, `place_painting_requires_a_wall`, `saving_and_loading_keeps_the_variant_and_facing` | 0 |
| `steel-core/src/entity/entities/objects/display_ui/block_display.rs` | 129 | — | 0 |
| `steel-core/src/entity/entities/objects/display_ui/glow_item_frame.rs` | 845 | `frame_survives_only_with_a_solid_support_behind_it`, `broken_support_removes_and_drops_the_frame`, `clicking_a_filled_frame_rotates_its_item`, `clicking_an_empty_frame_inserts_the_held_item_and_consumes_one`, `clicking_an_empty_frame_with_an_empty_hand_passes`, `dropping_the_frame_twice_does_not_duplicate_items`, `removing_a_filled_frame_drops_the_item_and_the_frame`, `load_wraps_facing_and_keeps_rotation_sign_like_java`, `load_defaults_missing_facing_to_down`, `item_frame_persists_structure_marker_state`, `item_frame_is_pickable_like_vanilla`, `analog_output_uses_item_presence_and_rotation` | 0 |
| `steel-core/src/entity/entities/objects/display_ui/item_frame.rs` | 872 | `frame_survives_only_with_a_solid_support_behind_it`, `broken_support_removes_and_drops_the_frame`, `clicking_a_filled_frame_rotates_its_item`, `clicking_an_empty_frame_inserts_the_held_item_and_consumes_one`, `clicking_an_empty_frame_with_an_empty_hand_passes`, `dropping_the_frame_twice_does_not_duplicate_items`, `removing_a_filled_frame_drops_the_item_and_the_frame`, `load_wraps_facing_and_keeps_rotation_sign_like_java`, `load_defaults_missing_facing_to_down`, `item_frame_persists_structure_marker_state`, `item_frame_is_pickable_like_vanilla`, `analog_output_uses_item_presence_and_rotation` | 0 |
| `steel-core/src/entity/entities/objects/display_ui/leash_fence_knot.rs` | 392 | `leash_knot_uses_vanilla_position_and_bounding_box`, `leash_knot_spawn_packet_uses_attached_block_pos`, `leash_knot_saves_no_type_specific_block_pos`, `leash_knot_survival_check_matches_vanilla_interval` | 0 |
| `steel-core/src/entity/entities/objects/display_ui/mod.rs` | 11 | — | 0 |
| `steel-core/src/entity/entities/objects/effects/area_effect_cloud/tests.rs` | 183 | `lingering_cloud_shrinks_away_before_its_duration_ends`, `affects_living_entities_once_per_reapplication_delay`, `extreme_saved_timings_wrap_like_vanilla_int_arithmetic`, `load_restores_age_and_resizes_to_saved_radius`, `load_accepts_any_numeric_tag` | 0 |
| `steel-core/src/entity/entities/objects/effects/area_effect_cloud.rs` | 495 | — | 0 |
| `steel-core/src/entity/entities/objects/effects/evoker_fangs.rs` | 163 | `evoker_fangs_use_fixed_dimensions`, `evoker_fangs_have_no_synced_payload_but_keep_base_data`, `vanilla_fangs_timings_match` | 0 |
| `steel-core/src/entity/entities/objects/effects/lightning_bolt.rs` | 268 | `new_lightning_bolts_are_not_visual_only`, `visual_only_flag_is_saved_into_nbt`, `vanilla_lightning_damage_matches` | 0 |
| `steel-core/src/entity/entities/objects/effects/mod.rs` | 9 | — | 0 |
| `steel-core/src/entity/entities/objects/explosives/end_crystal.rs` | 230 | `end_crystal_does_not_duplicate_shared_invulnerable_state`, `end_crystal_is_pickable_like_vanilla`, `end_crystal_blocks_building_like_vanilla` | 0 |
| `steel-core/src/entity/entities/objects/explosives/mod.rs` | 7 | — | 0 |
| `steel-core/src/entity/entities/objects/explosives/primed_tnt/tests.rs` | 237 | `priming_applies_vanilla_float_tau_launch_motion`, `cross_world_recreation_preserves_owner_and_explosion_attribution`, `flowing_water_pushes_primed_tnt_trajectory`, `colliding_tnt_requests_precise_position_sync_only_while_colliding`, `teleported_tnt_preserves_nether_portal_but_explodes_other_blocks`, `horizontally_aligned_explosion_does_not_push_primed_tnt_upward` | 0 |
| `steel-core/src/entity/entities/objects/explosives/primed_tnt.rs` | 385 | — | 0 |
| `steel-core/src/entity/entities/objects/items/experience_orb.rs` | 683 | `experience_value_buckets_match_vanilla`, `merge_id_uses_vanilla_grouping_and_value`, `experience_orb_merge_absorbs_existing_group`, `orb_damage_truncates_after_fractional_subtraction`, `orb_saves_and_loads_vanilla_state` | 0 |
| `steel-core/src/entity/entities/objects/items/falling_block/tests.rs` | 187 | `stacked_falling_blocks_settle_in_order_without_collapsing_into_one_state`, `concrete_powder_distinguishes_dry_side_water_and_fast_water_entry_landings`, `falling_block_persistence_and_spawn_data_preserve_the_carried_state` | 0 |
| `steel-core/src/entity/entities/objects/items/falling_block.rs` | 568 | — | 0 |
| `steel-core/src/entity/entities/objects/items/item_entity.rs` | 1004 | `item_entities_do_not_obstruct_block_placement`, `item_lava_hurt_sound_uses_vanilla_interval`, `item_with_stack_uses_vanilla_default_velocity`, `item_tick_refreshes_fluid_after_movement`, `fake_item_is_never_pickable_and_expires_on_its_next_tick`, `item_merge_preserves_vanilla_stack_and_timing`, `item_damage_truncates_after_fractional_subtraction`, `damage_resistant_item_ignores_matching_damage` | 0 |
| `steel-core/src/entity/entities/objects/items/mod.rs` | 9 | — | 0 |
| `steel-core/src/entity/entities/objects/mod.rs` | 10 | — | 0 |
| `steel-core/src/entity/entities/objects/projectiles/arrow.rs` | 1130 | `pickup_permissions_round_trip_through_their_vanilla_ordinals`, `stuck_arrows_despawn_after_the_vanilla_life_time`, `hitting_a_block_sticks_the_arrow`, `spectral_arrows_are_flagged_and_regular_arrows_are_not`, `pierce_level_survives_saving_and_loading` | 0 |
| `steel-core/src/entity/entities/objects/projectiles/breeze_wind_charge.rs` | 223 | `explosion_power_matches_vanilla`, `wind_charges_fly_without_gravity`, `breeze_charge_hitting_a_block_discards_it` | 0 |
| `steel-core/src/entity/entities/objects/projectiles/dragon_fireball.rs` | 210 | `dragon_fireball_radius_matches_vanilla`, `dragon_breath_cloud_matches_vanilla` | 0 |
| `steel-core/src/entity/entities/objects/projectiles/ender_pearl.rs` | 543 | `shoot_aligns_velocity_with_direction`, `owner_round_trips`, `default_item_is_ender_pearl`, `owner_death_vanish_predicate_respects_won_game`, `ticket_renewal_registers_with_the_resolved_owner`, `bubble_column_uses_clamped_entity_behavior` | 0 |
| `steel-core/src/entity/entities/objects/projectiles/experience_bottle.rs` | 276 | `experience_amount_matches_vanilla_range`, `impact_spawns_experience_orbs_and_discards_the_bottle` | 0 |
| `steel-core/src/entity/entities/objects/projectiles/eye_of_ender.rs` | 291 | — | 0 |
| `steel-core/src/entity/entities/objects/projectiles/firework_rocket.rs` | 789 | `launched_rocket_uses_fireworks_flight_duration_for_lifetime`, `firework_uses_vanilla_neutral_sound_source`, `hurt_marks_rocket_unless_base_invulnerable_and_always_returns_false`, `firework_metadata_carries_item_attachment_and_angle`, `attached_rocket_keeps_its_target_when_world_ids_are_reused`, `explosion_damages_replacement_with_attached_entity_id`, `firework_state_persists_with_vanilla_keys`, `firework_knockback_direction_points_from_rocket_to_target`, `firework_damage_source_has_no_raw_position`, `elytra_boost_matches_vanilla_vector_formula` | 0 |
| `steel-core/src/entity/entities/objects/projectiles/fishing_hook.rs` | 1266 | `spawn_data_identifies_the_owning_player`, `removal_only_clears_the_matching_active_hook`, `retrieving_discards_the_active_hook`, `shoot_from_player_respects_pitch_and_yaw_signs`, `grounded_hook_does_not_hook_owner_when_player_stands_on_it`, `submerged_hook_in_water_experiences_upward_buoyancy`, `open_water_calculation_identifies_open_lake_and_shallow_puddle` | 0 |
| `steel-core/src/entity/entities/objects/projectiles/large_fireball.rs` | 156 | `large_fireball_power_matches_vanilla` | 0 |
| `steel-core/src/entity/entities/objects/projectiles/lingering_potion.rs` | 203 | — | 0 |
| `steel-core/src/entity/entities/objects/projectiles/llama_spit.rs` | 155 | `llama_spit_damage_matches_vanilla`, `new_llama_spit_has_no_owner` | 0 |
| `steel-core/src/entity/entities/objects/projectiles/mod.rs` | 42 | — | 0 |
| `steel-core/src/entity/entities/objects/projectiles/shulker_bullet.rs` | 188 | `shulker_bullet_levitation_matches_vanilla`, `shulker_bullet_damage_matches_vanilla`, `new_shulker_bullet_has_no_owner` | 0 |
| `steel-core/src/entity/entities/objects/projectiles/small_fireball.rs` | 152 | `small_fireball_constants_match_vanilla` | 0 |
| `steel-core/src/entity/entities/objects/projectiles/snowball.rs` | 270 | `hurt_marks_snowball_unless_base_invulnerable_and_always_returns_false`, `impact_damage_is_three_for_blazes_and_zero_otherwise`, `on_hit_discards_the_snowball` | 0 |
| `steel-core/src/entity/entities/objects/projectiles/splash_potion.rs` | 302 | `on_hit_discards_the_splash_potion` | 0 |
| `steel-core/src/entity/entities/objects/projectiles/thrown_egg.rs` | 462 | `hurt_marks_egg_unless_base_invulnerable_and_always_returns_false`, `hatch_count_matches_vanilla_odds`, `hatchling_inherits_egg_variant_component`, `hatchling_without_variant_keeps_default`, `hatchling_spawns_baby_chicken_at_egg_position`, `hatchling_does_not_spawn_fully_enclosed_in_solid_block`, `hatchling_fits_when_egg_lands_on_top_of_solid_block` | 0 |
| `steel-core/src/entity/entities/objects/projectiles/wind_charge.rs` | 223 | `explosion_power_matches_vanilla`, `wind_charges_fly_without_gravity`, `hitting_a_block_discards_the_charge` | 0 |
| `steel-core/src/entity/entities/objects/projectiles/wither_skull.rs` | 220 | `wither_skull_damage_matches_vanilla`, `wither_skull_wither_effect_matches_vanilla`, `wither_skull_blast_power_matches_vanilla`, `new_wither_skull_has_no_owner` | 0 |
| `steel-core/src/entity/entities/objects/technical/interaction.rs` | 535 | `skip_attack_interaction_when_required`, `record_player_actions`, `update_dimensions_on_edit` | 0 |
| `steel-core/src/entity/entities/objects/technical/marker.rs` | 158 | `markers_cannot_get_hurt`, `markers_cannot_fall`, `markers_cannot_have_passenger` | 0 |
| `steel-core/src/entity/entities/objects/technical/mod.rs` | 9 | — | 0 |
| `steel-core/src/entity/entities/objects/technical/ominous_item_spawner.rs` | 182 | `new_ominous_item_spawners_start_empty`, `countdown_is_clamped_to_the_vanilla_window` | 0 |
| `steel-core/src/entity/entities/objects/vehicles/abstract_minecart.rs` | 638 | — | 0 |
| `steel-core/src/entity/entities/objects/vehicles/boat.rs` | 1242 | `a_new_boat_starts_in_the_air_without_paddling`, `paddle_flags_live_in_the_synchronized_data`, `rider_input_is_stored_on_the_boat`, `the_drop_item_is_derived_from_the_entity_id`, `paddle_sounds_follow_the_location`, `locations_report_their_water_state`, `the_default_state_is_in_air_with_unit_friction` | 0 |
| `steel-core/src/entity/entities/objects/vehicles/chest_boat.rs` | 324 | `a_chest_boat_has_a_full_size_inventory`, `a_chest_boat_seats_a_single_passenger`, `a_chest_boat_drops_the_matching_item` | 0 |
| `steel-core/src/entity/entities/objects/vehicles/chest_minecart.rs` | 317 | `chest_minecart_saves_structure_loot_table_state`, `chest_minecart_is_pickable_and_pushable_like_vanilla`, `chest_minecart_relative_portal_position_resets_forward_offset` | 0 |
| `steel-core/src/entity/entities/objects/vehicles/command_block_minecart.rs` | 361 | `new_command_block_minecarts_have_no_command`, `setting_the_command_mirrors_it_into_synced_data`, `command_block_minecart_round_trips_its_command` | 0 |
| `steel-core/src/entity/entities/objects/vehicles/furnace_minecart.rs` | 365 | `new_furnace_minecarts_are_unlit`, `one_fuel_item_adds_the_vanilla_fuel_budget`, `fuel_stops_accumulating_at_the_vanilla_ceiling` | 0 |
| `steel-core/src/entity/entities/objects/vehicles/hopper_minecart.rs` | 467 | `new_hoppers_start_enabled_and_empty`, `hoppers_accept_items_up_to_their_capacity`, `the_pickup_box_is_the_vanilla_quarter_block_inflation`, `transfer_cooldown_is_persisted_in_nbt`, `redstone_flips_the_enabled_flag` | 0 |
| `steel-core/src/entity/entities/objects/vehicles/minecart.rs` | 286 | — | 0 |
| `steel-core/src/entity/entities/objects/vehicles/minecart_behavior.rs` | 506 | — | 0 |
| `steel-core/src/entity/entities/objects/vehicles/mod.rs` | 34 | — | 0 |
| `steel-core/src/entity/entities/objects/vehicles/spawner_minecart.rs` | 212 | `new_spawner_minecarts_have_a_live_spawner`, `spawner_event_returns_whether_it_was_applied`, `saved_spawners_round_trip_their_delay` | 0 |
| `steel-core/src/entity/entities/objects/vehicles/tests/abstract_minecart.rs` | 226 | `natural_slowdown_handles_passengers_water_and_vertical_motion`, `derailing_clamps_before_ground_slowdown_and_applies_air_drag_after_moving`, `aligned_cart_collisions_exchange_horizontal_velocity`, `cart_push_distance_threshold_preserves_vanilla_float_precision`, `display_settings_and_rotation_round_trip_with_vanilla_has_ticked_semantics`, `loading_empty_nbt_clears_previous_settings_and_omits_default_display_fields` | 0 |
| `steel-core/src/entity/entities/objects/vehicles/tests/minecart.rs` | 47 | `powered_activation_starts_the_hurt_animation_without_destroying_the_cart`, `powered_activation_preserves_an_existing_hurt_animation` | 0 |
| `steel-core/src/entity/entities/objects/vehicles/tests/minecart_behavior.rs` | 172 | `known_movement_handles_nan_and_infinity_like_vanilla`, `unpowered_rail_braking_preserves_the_stop_threshold`, `powered_rail_accelerates_after_slowdown_and_caps_only_displacement`, `rail_projection_handles_all_slopes_at_negative_coordinates`, `rotation_flip_boundaries_are_asymmetric` | 0 |
| `steel-core/src/entity/entities/objects/vehicles/tests/support.rs` | 51 | — | 0 |
| `steel-core/src/entity/entities/objects/vehicles/tests/vehicle_entity.rs` | 94 | `damage_accumulates_and_destruction_requires_more_than_forty`, `destruction_respects_entity_drops_and_preserves_the_custom_name` | 0 |
| `steel-core/src/entity/entities/objects/vehicles/tnt_minecart.rs` | 311 | `new_tnt_minecarts_are_defused`, `priming_uses_the_vanilla_fuse_length`, `explosion_power_grows_with_speed_and_is_capped` | 0 |
| `steel-core/src/entity/entities/objects/vehicles/vehicle_entity.rs` | 117 | — | 0 |
| `steel-core/src/entity/entity/mod.rs` | 4323 | — | 0 |
| `steel-core/src/entity/entity/tests.rs` | 63 | `look_at_rotation_matches_vanilla_axes`, `forced_rotation_offsets_relative_arguments`, `landing_particles_use_vanilla_fall_power_threshold_and_cap`, `landing_particles_shift_to_the_landed_on_block_edge` | 0 |
| `steel-core/src/entity/entity_selector.rs` | 11 | — | 0 |
| `steel-core/src/entity/fluid_contact.rs` | 537 | `scan_reports_fluid_height_above_entity_feet`, `scan_uses_effective_fluid_height`, `scan_ignores_fluid_below_interaction_box`, `scan_marks_eye_inside_matching_fluid_column`, `scan_accumulates_player_fluid_current_as_average_flow`, `scan_accumulates_non_player_fluid_current_as_normalized_flow`, `shallow_current_is_scaled_by_fluid_height` | 0 |
| `steel-core/src/entity/identity.rs` | 23 | — | 0 |
| `steel-core/src/entity/inside_block_effects.rs` | 271 | `collector_flushes_effects_in_vanilla_type_order_per_step`, `collector_runs_before_and_after_callbacks_around_default_effect`, `collector_stops_after_effect_makes_entity_not_alive` | 0 |
| `steel-core/src/entity/item_based_steering.rs` | 170 | `boost_starts_once_and_returns_vanilla_total_range`, `tick_boost_uses_vanilla_post_increment_expiry` | 0 |
| `steel-core/src/entity/item_frame.rs` | 15 | — | 0 |
| `steel-core/src/entity/leash.rs` | 545 | — | 0 |
| `steel-core/src/entity/living_base/mod.rs` | 1962 | — | 0 |
| `steel-core/src/entity/living_base/tests.rs` | 674 | `ends_within_treats_infinite_duration_as_never_ending`, `living_constructor_initializes_health_from_max_health`, `absorption_amount_clamps_to_attribute_range`, `fall_damage_starts_above_safe_fall_distance`, `last_hurt_by_player_memory_ticks_down_then_clears_reference`, `fall_damage_applies_block_and_attribute_multipliers`, `post_impulse_grace_counts_down_by_tick`, `post_impulse_grace_keeps_larger_existing_window`, `current_impulse_context_tracks_fall_damage_impact_position`, `current_impulse_context_resets_after_grace_window`, `fall_flying_is_living_entity_state`, `fall_flying_ticks_are_living_entity_state`, `living_rotation_is_base_tick_snapshot_state`, `living_swing_uses_vanilla_restart_gate`, `living_swing_tick_keeps_full_progress_until_after_duration`, `living_swing_restart_preserves_progress_only_for_same_description`, `living_swing_nonpositive_duration_never_divides_by_zero`, `equipment_is_living_entity_state`, `sprinting_is_living_entity_state_and_speed_modifier`, `active_mob_effect_presence_is_living_entity_state`, `active_mob_effect_amplifier_is_living_entity_state`, `mob_effect_attribute_modifiers_use_extracted_vanilla_data`, `mob_effect_duration_tick_removes_expired_effect`, `stronger_shorter_effect_downgrades_to_hidden_effect`, `sleeping_uses_living_entity_sleeping_position`, `last_climbable_pos_is_living_entity_state`, `discard_friction_is_living_entity_state`, `living_travel_input_is_shared_living_state`, `jumping_and_jump_delay_are_shared_living_state`, `poison_and_regeneration_tick_schedules_match_vanilla_interval_formulas`, `refreshing_an_active_effect_is_not_reported_as_added` | 0 |
| `steel-core/src/entity/living_entity.rs` | 3871 | `shield_component_is_present_on_shield_items`, `non_shield_items_do_not_block` | 0 |
| `steel-core/src/entity/living_reference.rs` | 31 | — | 0 |
| `steel-core/src/entity/manager/benchmark_support.rs` | 81 | — | 0 |
| `steel-core/src/entity/manager/entries.rs` | 85 | — | 0 |
| `steel-core/src/entity/manager/mod.rs` | 2473 | — | 0 |
| `steel-core/src/entity/manager/spatial_query.rs` | 181 | — | 0 |
| `steel-core/src/entity/manager/tests/lifecycle.rs` | 117 | `add_live_entity_rejects_manager_owned_unloaded_chunk`, `add_live_entity_accepts_external_unloaded_chunk`, `add_live_entity_rejects_duplicate_uuid_without_registering_second_entity`, `add_live_entity_tree_rejects_duplicate_uuid_without_partial_registration`, `duplicate_entity_id_is_a_loud_invariant_failure` | 0 |
| `steel-core/src/entity/manager/tests/mod.rs` | 289 | — | 0 |
| `steel-core/src/entity/manager/tests/movement_and_visibility.rs` | 597 | `committed_move_updates_spatial_index_within_the_same_section`, `committed_move_updates_chunk_index_for_loaded_destination`, `validate_move_rejects_manager_owned_unloaded_destination`, `commit_move_rejects_destination_unloaded_after_validation`, `chunk_recovery_restores_same_entity_arc_before_final_unload`, `chunk_recovery_refreshes_bounds_changed_while_inactive`, `live_or_unloading_membership_excludes_removed_live_entities`, `live_or_unloading_membership_includes_unload_retained_entities`, `chunk_unload_retains_manager_owned_passenger_tree`, `passenger_chunk_unload_hides_passenger_without_unloading_vehicle_tree`, `loaded_entity_tree_can_restore_passenger_in_hidden_chunk`, `attached_passenger_can_move_while_its_own_chunk_is_hidden`, `passenger_move_from_hidden_to_loaded_chunk_becomes_accessible`, `passenger_move_from_loaded_to_hidden_chunk_becomes_inaccessible`, `hidden_chunk_passenger_is_not_ticked_by_loaded_vehicle`, `non_passenger_tick_snapshots_old_position_and_rotation_before_tick`, `passenger_tick_snapshots_old_position_and_rotation_before_ride_tick` | 0 |
| `steel-core/src/entity/manager/tests/persistence.rs` | 315 | `final_chunk_unload_marks_stale_arc_removed_and_allows_same_identity_to_reload`, `saveable_entities_include_manager_owned_live_unloading_and_pending_entities`, `save_pending_acknowledgement_clears_only_persisted_entities`, `add_live_entity_rejects_duplicate_uuid_in_save_pending_entities`, `add_live_entity_panics_on_duplicate_id_in_save_pending_entities`, `add_live_entity_rejects_duplicate_uuid_in_unloading_entities`, `add_live_entity_panics_on_duplicate_id_in_unloading_entities`, `chunk_recovery_does_not_restore_removed_retained_entities`, `chunk_recovery_keeps_saveable_removed_retained_entities_pending`, `saveable_entities_outside_saved_chunks_reports_only_manager_owned_entities` | 0 |
| `steel-core/src/entity/manager/tests/queries_and_order.rs` | 677 | `aabb_matching_query_filters_accessible_entities`, `hard_collision_broad_phase_matches_full_scan_with_source_dependent_targets`, `conservative_custom_source_scans_non_hard_targets`, `hard_collision_index_tracks_move_bounds_unload_restore_and_remove`, `visibility_transitions_separate_tracking_and_ticking`, `has_aabb_matching_query_respects_bounds_accessibility_and_predicate`, `aabb_matching_bounding_box_query_returns_only_matching_intersections`, `nearest_aabb_matching_query_returns_closest_match`, `accessible_entities_keep_tracking_start_order`, `indexed_ordered_entity_ids_compaction_preserves_order_across_reinsertion`, `aabb_queries_use_vanilla_section_order_then_section_insertion_order`, `spatial_cell_reentry_preserves_section_insertion_order`, `spatial_query_candidates_skip_distant_entities_in_the_same_section`, `bounding_box_change_updates_spatial_index_without_position_change`, `delayed_bounding_box_callback_cannot_restore_stale_bounds` | 0 |
| `steel-core/src/entity/manager/tests/spatial_queries.rs` | 184 | `collision_snapshot_keeps_candidates_moved_into_bounds_by_an_earlier_callback`, `overlapping_cell_updates_keep_both_indexes_ordered`, `borrowed_query_fallback_preserves_order_and_slot_reuse` | 0 |
| `steel-core/src/entity/manager/tests/ticking.rs` | 410 | `tick_entities_skips_external_entities`, `tick_entities_ticks_external_always_ticking_entities_without_dirtying_chunks`, `chunk_unload_retention_preserves_external_always_ticking_passenger`, `tick_entities_uses_start_of_tick_snapshot_for_added_entities`, `tick_entities_checks_despawn_for_ticking_entities`, `tick_entities_skips_despawn_for_tracked_non_ticking_entities`, `tick_entities_skips_pending_world_change_entities`, `tick_entities_skips_passengers_of_pending_world_change_vehicles_before_despawn`, `tick_entities_skips_frozen_entities_and_despawn_checks`, `tick_entities_ticks_player_passenger_vehicle_while_frozen`, `passenger_detached_during_ride_tick_ticks_again_at_later_root_slot`, `stable_passenger_tree_ticks_once_in_vehicle_depth_first_order` | 0 |
| `steel-core/src/entity/mob/mod.rs` | 2166 | — | 0 |
| `steel-core/src/entity/mob/pathfinder.rs` | 463 | — | 0 |
| `steel-core/src/entity/mob/tests.rs` | 1693 | `spawner_obstruction_rejects_overlapping_players`, `equipment_drop_attempt_gate_matches_vanilla_conditions`, `mob_base_uses_vanilla_fire_path_malus_defaults`, `pathfinder_mob_reads_below_surface_capability_from_navigation`, `mob_server_ai_step_increments_no_action_time`, `mob_control_flags_enable_goals_without_controller_or_boat`, `mob_control_flags_disable_goals_for_mob_controller`, `mob_control_flags_disable_jump_when_riding_boat`, `mob_attack_damage_source_uses_item_damage_type_component`, `mob_target_stores_living_target_weakly`, `mob_target_can_be_cleared`, `mob_target_expires_with_target_entity`, `mob_target_rejects_non_living_entities`, `mob_target_rejects_targets_it_cannot_attack`, `mob_target_filters_invalid_target_without_clearing_stored_target`, `mob_target_clears_previous_target_when_new_target_is_invalid`, `melee_attack_range_uses_vanilla_default_reach`, `melee_attack_range_uses_vehicle_expanded_attack_box`, `target_reach_uses_vanilla_horizontal_endpoint_distance`, `mob_can_attack_excludes_ghasts`, `mob_base_tick_increments_ambient_sound_time_when_roll_fails`, `mob_base_tick_resets_ambient_sound_time_after_vanilla_roll`, `mob_hurt_sound_resets_ambient_sound_time`, `mob_do_hurt_target_applies_attack_damage_and_records_target`, `mob_do_hurt_target_applies_vanilla_extra_knockback`, `mob_look_control_rotates_head_yaw_without_turning_body_yaw`, `mob_look_control_returns_head_yaw_toward_body_when_idle`, `mob_body_rotation_control_uses_tick_position_delta`, `mob_tick_leash_applies_default_elastic_pull`, `mob_leash_snap_drops_leash_and_disables_move_control`, `pathfinder_leash_attach_pins_home_restriction_to_holder`, `mob_on_leash_removed_clears_home_restriction`, `pathfinder_close_range_leash_navigates_toward_holder`, `pathfinder_close_range_leash_skips_following_while_panicking`, `mob_that_cannot_be_leashed_refuses_a_lead`, `mob_despawn_resets_no_action_time_near_player`, `mob_despawn_discards_far_removable_mob`, `mob_persistence_resets_no_action_time_and_blocks_removal`, `mob_home_restriction_uses_vanilla_radius`, `looting_collects_nearby_item_into_main_hand`, `looting_runs_through_ai_step_even_with_no_ai`, `looting_skips_when_mob_cannot_pick_up_loot`, `pick_up_item_leaves_an_occupied_main_hand_untouched`, `equip_routes_armor_to_its_armor_slot`, `equip_replaces_worse_armor_and_drops_the_old_piece`, `equip_carries_worse_armor_in_a_free_hand`, `equip_rejects_worse_armor_when_the_hands_are_full`, `equip_respects_prevent_armor_change_on_worn_gear`, `pick_up_item_takes_one_from_a_stack_and_leaves_the_rest`, `equip_keeps_a_preferred_weapon_over_a_stronger_one`, `equip_takes_a_preferred_weapon_over_a_held_non_preferred_one`, `ground_path_target_air_rewrites_to_surface_above_ground`, `ground_path_target_solid_rewrites_to_first_open_block_above`, `daylight_burning_ignites_unprotected_mob_in_burn_in_daylight_tag`, `daylight_burning_damages_a_protecting_helmet_without_igniting`, `daylight_burning_is_gated_off_at_night_and_in_the_nether`, `mob_ai_step_gates_daylight_burning_on_burn_in_daylight_tag` | 0 |
| `steel-core/src/entity/mob_effect/absorption.rs` | 29 | — | 0 |
| `steel-core/src/entity/mob_effect/bad_omen.rs` | 58 | — | 0 |
| `steel-core/src/entity/mob_effect/heal_or_harm.rs` | 84 | — | 0 |
| `steel-core/src/entity/mob_effect/hunger.rs` | 24 | — | 0 |
| `steel-core/src/entity/mob_effect/infested.rs` | 48 | — | 0 |
| `steel-core/src/entity/mob_effect/mod.rs` | 108 | — | 0 |
| `steel-core/src/entity/mob_effect/oozing.rs` | 45 | — | 0 |
| `steel-core/src/entity/mob_effect/poison.rs` | 32 | — | 0 |
| `steel-core/src/entity/mob_effect/raid_omen.rs` | 37 | — | 0 |
| `steel-core/src/entity/mob_effect/regeneration.rs` | 24 | — | 0 |
| `steel-core/src/entity/mob_effect/saturation.rs` | 25 | — | 0 |
| `steel-core/src/entity/mob_effect/weaving.rs` | 38 | — | 0 |
| `steel-core/src/entity/mob_effect/wind_charged.rs` | 40 | — | 0 |
| `steel-core/src/entity/mob_effect/wither.rs` | 29 | — | 0 |
| `steel-core/src/entity/mod.rs` | 1496 | — | 0 |
| `steel-core/src/entity/monster/tests.rs` | 780 | `monster_constructor_defaults_match_vanilla`, `monster_is_dark_enough_to_spawn_rejects_bright_overworld`, `monster_is_dark_enough_to_spawn_accepts_dark_nether`, `monster_spawn_rules_accept_only_sturdy_non_glowing_surfaces`, `monster_spawn_rules_consult_block_specific_spawn_surfaces`, `monster_spawn_rules_combine_light_and_surface_checks`, `monster_any_light_spawn_rules_skip_light_but_keep_surface_checks`, `monster_surface_spawn_rules_require_sky_visibility_or_spawner`, `monster_spawn_rules_refuse_hostile_types_on_peaceful_difficulty`, `monster_walk_target_value_prefers_darkness`, `monster_ai_step_grows_no_action_time_by_vanilla_light_rate`, `monster_check_spawn_obstruction_rejects_liquids_without_checking_blocks`, `monster_spawn_obstruction_checks_entities_and_excludes_spectators_and_shared_vehicles`, `enemy_monsters_cannot_be_leashed`, `monster_downcasts_from_shared_entity` | 0 |
| `steel-core/src/entity/monster.rs` | 216 | — | 0 |
| `steel-core/src/entity/mount.rs` | 223 | `resizing_keeps_the_saddle`, `a_fresh_mount_inventory_has_no_saddle` | 0 |
| `steel-core/src/entity/movement_sync.rs` | 1326 | `movement_sync_records_delta_when_packed_delta_fits`, `movement_sync_records_full_when_forced`, `movement_sync_records_full_when_delta_overflows`, `rotation_sync_records_body_rotation_only_when_packed_angle_changes`, `rotation_sync_records_head_rotation_only_when_packed_angle_changes`, `velocity_sync_records_packet_when_delta_exceeds_threshold`, `velocity_sync_skips_sub_threshold_non_zero_delta`, `velocity_sync_records_packet_when_entity_becomes_stationary`, `movement_sync_state_tracks_position_and_rotation_together`, `movement_sync_update_emits_position_rotation_before_head_rotation`, `movement_sync_update_full_position_marks_body_and_head_rotation_sent`, `server_entity_sync_sends_track_delta_velocity_before_position`, `server_entity_sync_skips_velocity_for_non_track_delta_without_needs_sync`, `server_entity_sync_processes_and_clears_explicit_needs_sync`, `server_entity_sync_processes_dirty_data_gate_between_intervals`, `server_entity_sync_forces_full_position_when_on_ground_changes`, `server_entity_sync_rotates_passenger_without_position_packet`, `sync_decision_builds_position_delta_packet`, `sync_decision_builds_position_rotation_delta_packet`, `sync_decision_builds_full_position_sync_packet` | 0 |
| `steel-core/src/entity/natural_spawner.rs` | 418 | `spawn_interval_matches_vanilla`, `spawn_ring_matches_vanilla`, `monster_cap_matches_vanilla_math`, `peaceful_world_spawns_nothing`, `category_names_are_the_biome_spawner_keys` | 0 |
| `steel-core/src/entity/patrol_spawner.rs` | 300 | `the_delay_stays_inside_the_vanilla_window`, `the_effective_difficulty_matches_the_vanilla_ids` | 0 |
| `steel-core/src/entity/phantom_spawner.rs` | 232 | `the_delay_stays_inside_the_vanilla_window`, `the_group_size_follows_the_difficulty`, `insomnia_threshold_is_three_in_game_days` | 0 |
| `steel-core/src/entity/potion_contents.rs` | 154 | `splash_rebuild_takes_its_icon_flag_from_visibility`, `instant_health_amplifier_at_shift_width_does_not_panic_and_wraps_like_vanilla` | 0 |
| `steel-core/src/entity/projectile/mod.rs` | 1293 | `compute_margin_ramps_from_zero_to_cap`, `compute_margin_matches_vanilla_float_widening`, `clip_segment_hits_box_in_path`, `clip_segment_returns_start_when_inside`, `may_break_requires_impact_projectile_tag`, `move_vector_synthesizes_world_border_block_hit`, `firework_deflects_without_exploding_on_deflecting_entity`, `base_block_hit_dispatches_vanilla_block_callbacks`, `owner_collision_range_checks_root_vehicle_passengers`, `owner_collision_range_ignores_non_pickable_root_vehicle_passengers` | 0 |
| `steel-core/src/entity/projectile/owner_tests.rs` | 215 | `cached_dead_owner_takes_precedence_until_cache_expires`, `owner_lookup_isolated_by_domain_for_uuid_and_cache`, `owner_cache_rejects_mismatched_uuid_and_deflection_updates_owner`, `removed_owner_is_not_resolved_from_world_index`, `uuid_owner_is_attributed_on_projectile_entity_hit`, `serialized_owner_resolves_by_uuid_and_loading_clears_old_cache` | 0 |
| `steel-core/src/entity/projectile/throwable.rs` | 243 | `bubble_column_affects_throwable_projectile_before_its_first_movement`, `first_tick_bubble_column_does_not_clamp_projectile_velocity` | 0 |
| `steel-core/src/entity/projectile/throwable_item.rs` | 52 | — | 0 |
| `steel-core/src/entity/projectile/thrown_potion.rs` | 213 | — | 0 |
| `steel-core/src/entity/pushing/impulse.rs` | 33 | — | 0 |
| `steel-core/src/entity/pushing/mod.rs` | 151 | — | 0 |
| `steel-core/src/entity/pushing/snapshot.rs` | 60 | — | 0 |
| `steel-core/src/entity/pushing/test_fixtures.rs` | 329 | — | 0 |
| `steel-core/src/entity/pushing/tests.rs` | 286 | `mixed_species_pushes_match_sequential_contacts_and_custom_callbacks`, `pushes_match_sequential_contacts_at_the_cramming_boundary`, `bat_keeps_its_empty_push_entities_override`, `default_impulses_preserve_bits_and_custom_velocity_accessors`, `separation_rounding_matches_java_reference` | 0 |
| `steel-core/src/entity/ranged_attack.rs` | 156 | `flat_shot_keeps_horizontal_direction_and_adds_the_arc`, `vertical_shot_keeps_the_drop`, `diagonal_distance_uses_the_horizontal_plane`, `inaccuracy_matches_the_vanilla_difficulty_table`, `torso_aim_factor_matches_vanilla_get_y` | 0 |
| `steel-core/src/entity/reference.rs` | 89 | — | 0 |
| `steel-core/src/entity/registry.rs` | 451 | `registered_living_load_restores_current_head_and_body_yaw`, `create_and_load_skips_unregistered_entity_type`, `create_forwards_entity_type_to_factory`, `strict_load_needs_only_an_nbt_factory` | 0 |
| `steel-core/src/entity/selector.rs` | 34 | — | 0 |
| `steel-core/src/entity/spawn.rs` | 832 | `recursive_spawner_load_rejects_unknown_and_unimplemented_entities`, `recursive_spawner_load_skips_failed_passengers_and_keeps_valid_siblings`, `recursive_spawner_load_rejects_hostile_entities_in_peaceful`, `ageable_group_data_increments_before_later_baby_rolls_can_apply`, `ageable_group_data_can_disable_baby_spawns`, `item_entity_data_overrides_entity_state_without_replacing_defaults`, `item_pig_variant_overrides_spawn_variant`, `item_cow_components_override_spawn_state`, `item_chicken_components_override_spawn_state`, `item_sheep_color_overrides_spawn_state` | 0 |
| `steel-core/src/entity/spawn_placements.rs` | 52 | — | 0 |
| `steel-core/src/entity/spawn_util.rs` | 116 | — | 0 |
| `steel-core/src/entity/storage.rs` | 256 | `saveable_entities_keep_unloaded_to_chunk_removals`, `add_rejects_duplicate_entity_ids`, `close_drains_staged_entities_and_returns_late_adds`, `concurrent_add_and_close_leave_entity_with_exactly_one_owner` | 0 |
| `steel-core/src/entity/synced_data.rs` | 415 | `synced_data_reads_no_gravity_from_generated_base_layer`, `synced_data_reads_shift_key_down_from_generated_base_layer`, `synced_data_reads_swimming_from_generated_base_layer`, `synced_data_reads_invisible_from_generated_base_layer`, `synced_data_writes_individual_shared_flags_without_stomping`, `synced_data_writes_fire_and_freeze_base_layer`, `synced_data_writes_shared_save_base_layer`, `living_synced_data_writes_sleeping_pos_layer` | 0 |
| `steel-core/src/entity/tests/damage.rs` | 383 | `generic_living_hurt_applies_health_damage`, `generic_living_hurt_ignores_fire_damage_with_fire_resistance`, `generic_living_hurt_processes_default_death_once`, `generic_living_hurt_applies_armor_and_absorption`, `generic_living_hurt_applies_resistance`, `damage_reductions_use_the_retained_attacker_after_removal`, `generic_living_hurt_applies_damage_protection_enchantments`, `generic_living_default_does_not_damage_armor_equipment`, `generic_living_hurt_applies_source_position_knockback`, `try_as_dyn_exposes_living_entity_behavior`, `head_yaw_uses_living_head_rotation_only`, `living_equipment_attribute_modifiers_refresh_for_slot`, `generic_living_hurt_respects_no_knockback_damage_tag`, `generic_living_hurt_scales_knockback_by_resistance`, `full_damage_starts_the_vanilla_hurt_animation_timer`, `hurt_time_counts_down_in_the_living_base_tick`, `partial_damage_within_the_cooldown_leaves_hurt_time_alone`, `death_side_effects_run_only_once`, `lethal_hurts_consume_main_hand_totem_then_off_hand_totem` | 0 |
| `steel-core/src/entity/tests/death_loot.rs` | 119 | `melee_kill_with_fire_aspect_drops_cooked_porkchop`, `arrow_kill_smelts_by_direct_attacker_and_loots_by_attacker` | 0 |
| `steel-core/src/entity/tests/equipment_and_freezing.rs` | 357 | `can_glide_using_matches_vanilla_component_gate`, `living_armor_cover_counts_non_empty_humanoid_armor_slots`, `living_visibility_percent_uses_discrete_and_invisible_scaling`, `living_visibility_percent_uses_matching_mob_head_disguise`, `living_freeze_immunity_uses_armor_equipment`, `living_freeze_immunity_uses_body_armor_equipment`, `living_freeze_immunity_ignores_non_armor_equipment`, `living_freezing_decays_when_not_in_powder_snow`, `living_freezing_keeps_ticks_while_in_powder_snow`, `living_freezing_adds_powder_snow_speed_modifier`, `living_freezing_removes_stale_powder_snow_speed_modifier`, `living_freezing_damages_fully_frozen_entities_on_frequency`, `default_ai_step_ticks_freezing_after_travel`, `entity_cramming_damage_threshold_matches_vanilla_push_entities`, `freezing_damage_hurts_extra_tagged_entity_types`, `living_powder_snow_walkability_uses_feet_equipment`, `living_powder_snow_walkability_ignores_non_feet_equipment`, `default_can_glide_uses_living_equipment`, `try_to_start_fall_flying_uses_vanilla_glider_gate`, `gliding_non_player_living_entity_forces_velocity_sync`, `try_to_start_fall_flying_rejects_levitation`, `update_fall_flying_damages_glider_every_second_event_interval`, `update_fall_flying_stops_when_glider_gate_fails` | 0 |
| `steel-core/src/entity/tests/fall_and_fluids.rs` | 354 | `fall_damage_sound_selects_vanilla_small_and_big_sounds`, `living_fall_damage_uses_shared_damage_path_from_entity_dispatch`, `living_fall_damage_caps_distance_from_current_impulse`, `living_fall_damage_resets_current_impulse_when_landing_above_impact`, `stop_fall_flying_toggles_shared_state_back_to_false`, `lava_contact_is_ignored_until_after_first_tick`, `fluid_falling_adjustment_matches_vanilla_special_falling_case`, `fluid_falling_adjustment_is_skipped_while_sprinting`, `water_float_while_ridden_uses_vanilla_entity_type_tag_and_threshold`, `water_float_while_ridden_ignores_non_vehicle_tagged_entity`, `inside_bubble_column_pushes_up_and_resets_fall_distance`, `inside_bubble_column_drags_down_and_resets_fall_distance`, `above_bubble_column_uses_vanilla_stronger_velocity_limits`, `above_bubble_column_drag_down_uses_vanilla_stronger_velocity_limit`, `flying_players_ignore_bubble_column_entity_hooks`, `dolphins_grace_water_travel_hook_uses_active_mob_effect_state`, `living_air_supply_decrements_while_eye_in_water`, `living_air_supply_drowning_damage_resets_air`, `water_breathing_refills_air_underwater`, `breath_of_the_nautilus_prevents_drowning_without_refilling_air`, `entity_type_can_breathe_underwater_refills_air`, `living_air_supply_refills_out_of_water`, `living_base_tick_damages_entities_in_wall`, `living_environmental_damage_applies_in_wall_before_drowning`, `living_base_tick_skips_in_wall_damage_while_sleeping` | 0 |
| `steel-core/src/entity/tests/living_state.rs` | 231 | `default_entity_tick_dispatches_living_tick`, `living_base_tick_decrements_last_hurt_by_player_memory`, `kill_credit_keeps_a_player_in_another_world_until_memory_expires`, `living_base_tick_advances_swing_before_living_state`, `modified_swing_duration_uses_vanilla_dig_effects`, `attack_animation_uses_held_item_component`, `living_combat_memory_stores_and_expires_last_hurt_by_mob`, `living_combat_memory_clears_dead_last_hurt_mob`, `living_death_loot_table_uses_default_and_custom_mob_tables`, `closest_open_space_direction_matches_vanilla_order_on_ties`, `closest_open_space_direction_skips_full_collision_neighbors` | 0 |
| `steel-core/src/entity/tests/mod.rs` | 1272 | `non_player_command_identity_uses_uuid_and_resolved_name`, `entity_display_name_preserves_the_custom_name_component`, `command_data_compare_nbt_contains_base_and_custom_data`, `spawn_data_clamps_position_and_refreshes_old_transform`, `command_data_compare_nbt_contains_implemented_living_data`, `kill_uses_vanilla_living_and_non_living_paths`, `water_entry_emits_splash_only_on_transition`, `experience_orb_water_entry_emits_no_splash`, `entity_downcast_uses_concrete_type_key_not_registry_type`, `wither_rose_effect_ticks_vanilla_wither_damage`, `wither_effect_only_damages_on_its_vanilla_interval`, `wither_rose_respects_difficulty_invulnerability_and_effect_immunity`, `default_mob_effect_eligibility_uses_vanilla_entity_type_tags`, `heal_or_harm_behavior_inverts_for_undead_mobs`, `adding_absorption_effect_grants_absorption_hearts`, `living_relative_portal_position_resets_forward_offset` | 0 |
| `steel-core/src/entity/tests/movement.rs` | 275 | `resolved_movement_application_matches_vanilla_threshold`, `move_without_physics_returns_none_when_position_commit_rejects`, `move_preserves_fluid_contact_until_the_entity_fluid_update`, `living_move_refreshes_fluid_before_fall_state_is_applied`, `fall_damage_reset_clip_target_matches_vanilla_thresholds`, `input_vector_ignores_tiny_input_like_vanilla`, `input_vector_normalizes_large_input_and_rotates_by_yaw`, `look_angle_matches_vanilla_view_vector_axes`, `fall_flying_movement_applies_vanilla_gravity_lift_and_drag`, `fall_flying_movement_converts_upward_pitch_to_lift`, `fall_flying_collision_damage_matches_vanilla_threshold`, `in_wall_eye_box_requires_suffocating_state_and_shape_overlap`, `fall_flying_free_fall_interval_matches_vanilla_cadence`, `jump_boost_power_uses_active_effect_amplifier`, `levitation_travel_uses_active_effect_amplifier`, `slow_falling_caps_effective_gravity_only_while_falling`, `fall_distance_accumulation_clamps_like_vanilla` | 0 |
| `steel-core/src/entity/tests/portals.rs` | 237 | `default_tick_runs_vanilla_entity_base_tick`, `can_use_portal_requires_alive_entity`, `static_vanilla_portal_overrides_reject_special_entities`, `projectile_owner_uuid_reports_projectile_owner_identity`, `can_use_portal_respects_passenger_gate`, `indirect_passengers_match_vanilla_preorder`, `passenger_transition_rotation_matches_vanilla_relative_flags`, `passenger_transition_position_preserves_vehicle_offset`, `dimension_transition_persistence_keeps_non_chunk_serializable_entities`, `remove_after_changing_dimensions_clears_old_mob_leash_and_equipment`, `can_use_portal_rejects_sleeping_living_entities`, `dimension_changing_delay_uses_vanilla_class_overrides`, `set_as_inside_portal_starts_portal_process_when_not_on_cooldown`, `set_as_inside_portal_resets_cooldown_without_starting_process` | 0 |
| `steel-core/src/entity/tests/riding_and_leashes.rs` | 403 | `living_ride_tick_resets_fall_distance`, `default_below_world_hook_discards_entity`, `base_entity_has_no_controlling_passenger`, `start_riding_entities_links_passenger_and_vehicle`, `failed_save_restore_drops_leash_but_keeps_home_restriction`, `mob_start_riding_drops_attached_leash`, `transfer_leashables_to_holder_moves_valid_mobs`, `transfer_leashables_to_holder_skips_mobs_outside_snap_distance`, `set_leashed_to_notifies_replaced_holder`, `tick_leash_notifies_live_holder`, `tick_leash_snaps_live_holder_past_snap_distance`, `start_riding_entities_respects_boarding_cooldown`, `start_riding_entities_rejects_vehicle_cycles`, `start_riding_entities_inserts_player_passenger_first`, `controlled_vehicle_uses_player_known_movement_and_speed`, `controlled_vehicle_returns_direct_controlled_vehicle_not_root_vehicle`, `controlled_vehicle_known_movement_falls_back_without_active_player_controller`, `push_entity_separates_pushable_entities_like_vanilla` | 0 |
| `steel-core/src/entity/tests/travel.rs` | 223 | `jump_from_ground_uses_jump_strength_and_marks_velocity_sync`, `sprint_jump_from_ground_adds_vanilla_horizontal_impulse`, `living_jump_in_water_uses_fluid_jump_impulse_without_cooldown`, `living_jump_without_input_resets_jump_delay_like_vanilla`, `living_ai_step_zeroes_tiny_player_velocity_like_vanilla`, `living_ai_step_keeps_player_horizontal_velocity_above_combined_threshold`, `default_ai_step_resets_idle_jump_delay_and_dampens_input_before_travel`, `default_ai_step_resets_fall_distance_for_slow_falling_and_levitation`, `default_ai_step_jumps_from_ground_and_sets_vanilla_cooldown`, `living_travel_fluid_predicate_matches_vanilla_hooks`, `open_trapdoor_matches_ladder_facing_for_climbable`, `closed_trapdoor_is_not_usable_as_ladder`, `trapdoor_ladder_facing_must_match`, `vertical_collision_state_update_matches_vanilla_authority_gate`, `push_impulse_updates_velocity_and_marks_sync` | 0 |
| `steel-core/src/entity/ticking.rs` | 76 | — | 0 |
| `steel-core/src/entity/tracker/mod.rs` | 954 | — | 0 |
| `steel-core/src/entity/tracker/tests/mod.rs` | 201 | — | 0 |
| `steel-core/src/entity/tracker/tests/packet_sync.rs` | 233 | `send_changes_broadcasts_dirty_attributes_once`, `send_changes_broadcasts_dirty_equipment_once`, `send_changes_broadcasts_equipment_before_attributes`, `send_changes_syncs_hurt_marked_player_motion_to_self`, `send_changes_broadcasts_hurt_marked_non_player_motion` | 0 |
| `steel-core/src/entity/tracker/tests/passenger_leash.rs` | 315 | `spawn_pairing_omits_untracked_passenger_for_vehicle`, `spawn_pairing_includes_tracked_passenger_packet_for_vehicle`, `spawn_pairing_for_passenger_omits_untracked_vehicle_packet`, `spawn_pairing_for_passenger_includes_tracked_vehicle_passenger_packet`, `spawn_pairing_includes_live_mob_leash_link_packet`, `send_changes_broadcasts_leash_link_changes_once`, `send_changes_broadcasts_passenger_changes_once`, `send_changes_removes_untracked_passenger_from_vehicle_packet` | 0 |
| `steel-core/src/entity/tracker/tests/spawn_despawn.rs` | 63 | `spawn_pairing_includes_syncable_attributes`, `spawn_pairing_includes_non_empty_equipment`, `spawn_pairing_uses_entity_spawn_packet_position` | 0 |
| `steel-core/src/entity/tracker/tests/visibility_range.rs` | 85 | `client_tracking_range_is_converted_to_blocks`, `zero_client_tracking_range_disables_tracking`, `tracking_distance_uses_horizontal_circle`, `tracking_distance_is_capped_by_player_view_distance`, `vehicle_effective_tracking_range_uses_widest_passenger_range` | 0 |
| `steel-core/src/entity/villager_trades.rs` | 399 | `every_profession_offers_at_least_one_trade`, `a_jobless_villager_trades_nothing`, `a_farmer_gets_its_trades_from_the_registry_id`, `an_unknown_profession_trades_nothing`, `fresh_offers_are_in_stock`, `a_trade_locks_once_it_has_been_used_up`, `a_locked_trade_reopens_after_a_restock`, `demand_raises_the_price_but_never_below_one`, `the_second_payment_is_optional` | 0 |
| `steel-core/src/entity/warden_spawn_tracker.rs` | 260 | `a_fresh_tracker_starts_at_zero`, `the_warning_level_saturates_at_four`, `the_cooldown_refuses_a_second_warning`, `a_level_decays_after_twelve_thousand_quiet_ticks`, `the_cooldown_counts_down` | 0 |
| `steel-core/src/fluid/collision.rs` | 180 | `waterlogged_source_blocks_block_same_fluid_flowing_variant_spread`, `dry_waterloggable_blocks_reject_flowing_water`, `kelp_is_liquid_container_without_waterlogged_property` | 0 |
| `steel-core/src/fluid/conversion.rs` | 422 | `hole_check_rejects_dry_waterloggable_for_flowing_water_fallback`, `hole_check_treats_source_and_flowing_variants_as_same_fluid_below`, `spread_keeps_the_closest_slope_even_when_its_target_rejects_replacement` | 0 |
| `steel-core/src/fluid/flowing_fluid.rs` | 224 | — | 0 |
| `steel-core/src/fluid/fluid_behavior.rs` | 146 | — | 0 |
| `steel-core/src/fluid/fluids/empty.rs` | 80 | — | 0 |
| `steel-core/src/fluid/fluids/lava.rs` | 262 | — | 0 |
| `steel-core/src/fluid/fluids/mod.rs` | 8 | — | 0 |
| `steel-core/src/fluid/fluids/water.rs` | 124 | — | 0 |
| `steel-core/src/fluid/mod.rs` | 34 | — | 0 |
| `steel-core/src/fluid/spread_context.rs` | 95 | — | 0 |
| `steel-core/src/fluid/state.rs` | 406 | `vanilla_fluid_states_resolve_to_their_liquid_block_at_every_level`, `height_treats_source_and_flowing_variants_as_same_fluid_above`, `empty_fluid_height_is_zero`, `flow_points_toward_lower_same_fluid_neighbor`, `falling_flow_pulls_down_when_horizontal_neighbor_has_solid_face` | 0 |
| `steel-core/src/inventory/click.rs` | 484 | `pickup_slot_and_outside`, `out_of_range_slots_rejected`, `swap_targets`, `throw_and_pickup_all_buttons`, `quickcraft_encoding`, `programmatic_click_validation_covers_every_public_index` | 0 |
| `steel-core/src/inventory/container/crafting.rs` | 102 | — | 0 |
| `steel-core/src/inventory/container/helper.rs` | 89 | `missing_slot_defaults_to_zero`, `numeric_slot_narrows_to_a_byte`, `negative_slot_is_unsigned_and_out_of_range`, `non_numeric_slot_rejects_the_entry` | 0 |
| `steel-core/src/inventory/container/mod.rs` | 620 | `test_with_indices_disjoint`, `test_with_indices_single`, `test_with_indices_empty`, `with_indices_uses_physical_item_storage`, `clear_or_count_matching_items_counts_without_mutating`, `clear_or_count_matching_items_applies_cap_in_slot_order`, `clear_or_count_matching_items_removes_every_match_for_negative_limit`, `comparator_signal_uses_vanilla_discrete_non_empty_floor`, `test_with_indices_duplicate_panics`, `test_with_indices_out_of_bounds_panics` | 0 |
| `steel-core/src/inventory/container/result.rs` | 75 | — | 0 |
| `steel-core/src/inventory/container/simple.rs` | 55 | — | 0 |
| `steel-core/src/inventory/ender_chest.rs` | 84 | — | 0 |
| `steel-core/src/inventory/equipment.rs` | 95 | — | 0 |
| `steel-core/src/inventory/fuel_values.rs` | 179 | `vanilla_fuels_respect_specific_values_and_non_flammable_wood_removal` | 0 |
| `steel-core/src/inventory/lock.rs` | 634 | `erased_container_ref_preserves_id_and_typed_access`, `block_entity_container_capability_is_independently_lockable`, `non_container_block_entity_ref_is_rejected`, `unlocked_callback_releases_and_reacquires_every_container`, `barrel_change_reenters_analog_read_without_holding_container_lock` | 0 |
| `steel-core/src/inventory/menu/behavior.rs` | 1173 | `quick_move_source_persists_cloned_remainder_with_vanilla_callbacks`, `safe_insert_uses_set_by_player_before_the_menu_notification`, `quick_move_skips_aliases_of_the_source_slot` | 0 |
| `steel-core/src/inventory/menu/builder.rs` | 1288 | `build_rejects_a_slot_count_that_disagrees_with_the_menu_type`, `builds_with_an_erased_menu_kind`, `direct_section_rejects_a_range_past_container_capacity`, `result_slot_rejects_a_container_without_slot_zero`, `result_slot_claims_slot_zero_against_normal_sections`, `result_slot_registers_handler_dependencies`, `drain_rejects_a_result_slot`, `build_rejects_a_result_alias_through_player_inventory`, `build_allows_non_fake_player_inventory_aliases`, `route_rejects_overlapping_source_sections`, `section_at_preserves_the_given_index_order`, `section_at_rejects_indices_claimed_by_another_section`, `section_at_rejects_a_repeated_index`, `display_kind_rejects_placement`, `custom_kind_lowers_through_the_factory`, `custom_kind_rejects_a_mismatched_physical_backing` | 0 |
| `steel-core/src/inventory/menu/grid.rs` | 1522 | `full_width_placement_merges_into_one_section`, `narrow_placement_yields_one_section_per_row`, `place_slots_lands_in_row_major_order`, `at_indices_maps_cells_in_the_given_order`, `at_indices_rejects_a_count_that_disagrees_with_the_rect`, `kind_lowers_cells_through_a_custom_factory`, `place_slots_panics_on_count_mismatch`, `sibling_grids_stack_vertically`, `cols_carve_side_by_side`, `rows_and_offset_carve_one_container`, `restricted_placement_covers_like_place`, `placements_mask_paint_in_any_order`, `result_slot_lands_on_its_cell`, `result_placement_rejects_a_container_that_differs_from_the_handler`, `result_placement_rejects_a_container_without_slot_zero`, `result_placement_rejects_a_normal_alias`, `raw_grid_slots_cannot_alias_result_backing_storage`, `overlapping_placements_panic`, `out_of_bounds_placement_panics`, `uncovered_cells_panic`, `subgrid_must_cover_itself_despite_parent_paint`, `mixing_carve_axes_panics`, `grid_after_partial_row_panics`, `range_flavors_and_axis_orders_agree`, `unbounded_ends_resolve_against_the_subgrid`, `empty_range_panics_at_construction`, `from_range_starting_past_the_edge_panics`, `single_panics_on_multi_row_narrow_region` | 0 |
| `steel-core/src/inventory/menu/kind.rs` | 184 | — | 0 |
| `steel-core/src/inventory/menu/kinds/anvil_menu/tests.rs` | 335 | `validity_requires_anvil_tag_and_interaction_range`, `sacrifice_enchantments_conflict_with_earlier_merges`, `rename_only_result_preserves_unused_second_input`, `rename_only_result_restores_default_repair_cost_component`, `rename_validation_counts_filtered_java_utf16_units`, `rename_uses_java_blank_rules`, `unchanged_extended_potion_name_does_not_create_rename_result`, `full_inputs_do_not_fallback_to_the_hotbar`, `client_level_cost_uses_protocol_short_wrapping`, `partial_result_overflow_is_discarded` | 0 |
| `steel-core/src/inventory/menu/kinds/anvil_menu.rs` | 418 | — | 0 |
| `steel-core/src/inventory/menu/kinds/basic_menu.rs` | 14 | — | 0 |
| `steel-core/src/inventory/menu/kinds/beacon_menu.rs` | 512 | `beacon_menu_rejects_other_items_and_consumes_iron_on_effect_selection`, `shift_click_moves_payment_item_when_payment_slot_is_occupied`, `closing_the_menu_drops_the_unspent_payment` | 0 |
| `steel-core/src/inventory/menu/kinds/chest_boat_menu.rs` | 64 | — | 0 |
| `steel-core/src/inventory/menu/kinds/chest_menu.rs` | 208 | `chest_uses_exactly_the_rows_requested_from_oversized_container` | 0 |
| `steel-core/src/inventory/menu/kinds/container_menus.rs` | 225 | `slot_counts_match_the_vanilla_menu_types`, `the_dispenser_grid_holds_nine_slots`, `the_hopper_menu_holds_five_container_slots`, `the_lectern_menu_has_no_player_inventory` | 0 |
| `steel-core/src/inventory/menu/kinds/crafter_menu.rs` | 224 | — | 0 |
| `steel-core/src/inventory/menu/kinds/crafting_menu/tests.rs` | 93 | `partial_result_overflow_uses_the_default_drop_policy` | 0 |
| `steel-core/src/inventory/menu/kinds/crafting_menu.rs` | 176 | — | 0 |
| `steel-core/src/inventory/menu/kinds/enchantment_menu/tests.rs` | 279 | `full_bookshelf_ring_rolls_three_offers_with_clues`, `offers_are_identical_for_the_same_seed_and_cleared_for_unenchantable_items`, `button_is_rejected_without_lapis_or_levels`, `offers_only_reroll_when_the_table_slots_change`, `enchanting_spends_levels_and_lapis_and_rerolls_the_seed`, `creative_players_skip_lapis_but_still_spend_levels`, `books_become_enchanted_books_with_stored_enchantments` | 0 |
| `steel-core/src/inventory/menu/kinds/enchantment_menu.rs` | 418 | — | 0 |
| `steel-core/src/inventory/menu/kinds/ender_chest_menu.rs` | 50 | — | 0 |
| `steel-core/src/inventory/menu/kinds/furnace_menu.rs` | 339 | `manual_input_accepts_items_without_a_matching_recipe` | 0 |
| `steel-core/src/inventory/menu/kinds/grindstone_menu.rs` | 331 | — | 0 |
| `steel-core/src/inventory/menu/kinds/inventory_menu/tests.rs` | 85 | `partial_result_overflow_has_no_thrower` | 0 |
| `steel-core/src/inventory/menu/kinds/inventory_menu.rs` | 400 | — | 0 |
| `steel-core/src/inventory/menu/kinds/merchant_menu.rs` | 366 | `the_merchant_menu_has_three_trade_slots_above_the_inventory`, `an_empty_trade_list_still_builds_a_menu`, `a_payment_must_cover_the_whole_cost`, `an_offer_without_a_second_cost_ignores_the_second_slot` | 0 |
| `steel-core/src/inventory/menu/kinds/mod.rs` | 40 | — | 0 |
| `steel-core/src/inventory/menu/kinds/mount_menu.rs` | 232 | `mount_inventory_has_a_saddle_slot_an_armor_slot_and_three_rows`, `mount_menu_covers_the_container_once`, `a_mount_menu_has_no_registry_menu_type` | 0 |
| `steel-core/src/inventory/menu/kinds/shulker_box_menu.rs` | 53 | — | 0 |
| `steel-core/src/inventory/menu/kinds/station_menu.rs` | 1067 | `smithing_netherite_upgrade_preserves_custom_components`, `smithing_armor_trim_applies_once_and_rejects_identical_retrim`, `loom_applies_pattern_and_enforces_six_layer_limit`, `stonecutter_produces_stone_recipes` | 0 |
| `steel-core/src/inventory/menu/layout.rs` | 124 | — | 0 |
| `steel-core/src/inventory/menu/mod.rs` | 704 | — | 0 |
| `steel-core/src/inventory/menu/tests.rs` | 368 | `swap_locks_player_inventory_when_menu_has_no_inventory_slots`, `draining_a_block_entity_slot_marks_its_chunk_dirty`, `one_slot_creative_clone_drag_is_a_vanilla_noop`, `partial_swap_overflow_marks_displaced_item_as_thrown`, `partial_swap_overflow_is_discarded_in_creative`, `crafter_menu_toggles_the_block_entity_mask_through_data_slot_zero`, `crafter_menu_refuses_to_disable_an_occupied_slot` | 0 |
| `steel-core/src/inventory/mod.rs` | 16 | — | 0 |
| `steel-core/src/inventory/openers_counter.rs` | 169 | `counter_tracks_viewers` | 0 |
| `steel-core/src/inventory/prelude.rs` | 17 | — | 0 |
| `steel-core/src/inventory/recipe_manager.rs` | 459 | `external_crafting_interpreter_assembles_a_generated_dye_recipe` | 0 |
| `steel-core/src/inventory/slots/anvil_slots.rs` | 215 | `anvil_damage_does_not_notify_neighbors` | 0 |
| `steel-core/src/inventory/slots/armor_slot.rs` | 86 | — | 0 |
| `steel-core/src/inventory/slots/crafting_slots.rs` | 238 | — | 0 |
| `steel-core/src/inventory/slots/enchantment_slots.rs` | 56 | — | 0 |
| `steel-core/src/inventory/slots/furnace_slots.rs` | 200 | — | 0 |
| `steel-core/src/inventory/slots/grindstone_slots.rs` | 127 | — | 0 |
| `steel-core/src/inventory/slots/mod.rs` | 25 | — | 0 |
| `steel-core/src/inventory/slots/normal_slot.rs` | 110 | — | 0 |
| `steel-core/src/inventory/slots/restricted_slot.rs` | 216 | `max_stack_size_delegates_to_the_container_and_item` | 0 |
| `steel-core/src/inventory/slots/result_handler.rs` | 61 | — | 0 |
| `steel-core/src/inventory/slots/result_slot.rs` | 246 | `constructor_uses_the_handlers_result_container`, `a_plain_result_slot_does_not_count_crafts`, `a_crafting_result_slot_counts_the_taken_amount` | 0 |
| `steel-core/src/inventory/slots/slot.rs` | 361 | `custom_slot_safe_insert_override_survives_erasure` | 0 |
| `steel-core/src/java_block_pos_set/mod.rs` | 257 | — | 0 |
| `steel-core/src/java_block_pos_set/removal.rs` | 164 | — | 0 |
| `steel-core/src/java_block_pos_set/tests.rs` | 223 | `linked_order_matches_openjdk_25_after_each_insertion_and_resize`, `removing_tree_heads_preserves_order_and_lookup_for_both_policies`, `removed_entries_do_not_count_towards_the_resize_threshold`, `resize_untreeifies_small_unsplit_bins_after_iterator_removals` | 0 |
| `steel-core/src/java_block_pos_set/tree.rs` | 274 | — | 0 |
| `steel-core/src/java_block_pos_set/worldgen_tests.rs` | 145 | `duplicate_positions_are_not_inserted_twice`, `iteration_matches_java_hash_set_after_resize`, `collision_chain_triggers_java_hash_map_resize`, `reinserted_position_returns_to_its_bucket_tail`, `pop_uses_java_bucket_order`, `tree_bin_insertion_and_iterator_removal_match_java_hash_set`, `tree_bin_resize_matches_java_hash_set` | 0 |
| `steel-core/src/level_data/game_time.rs` | 40 | — | 0 |
| `steel-core/src/level_data/game_time_tests.rs` | 183 | `game_time_primary_only_survives_both_shutdown_save_orders`, `game_time_load_validates_only_the_authoritative_time_and_keeps_other_errors`, `game_time_wraps_and_persists_the_signed_value`, `damage_history_expires_at_forty_one_ticks_across_signed_clock_wrap` | 0 |
| `steel-core/src/level_data/mod.rs` | 963 | `load_seed_prefers_saved_level_toml`, `load_seed_returns_default_when_level_toml_is_missing`, `adopts_missing_generation_settings`, `rejects_mismatched_generation_settings`, `respawn_data_wraps_yaw_and_clamps_pitch`, `respawn_data_round_trips_through_toml`, `level_data_uses_legacy_spawn_as_respawn_default`, `level_data_round_trips_world_clock_state`, `level_data_round_trips_typed_game_rules_with_untagged_toml_values` | 0 |
| `steel-core/src/lib.rs` | 44 | — | 0 |
| `steel-core/src/loot/mod.rs` | 3 | — | 0 |
| `steel-core/src/loot/resolvers.rs` | 89 | — | 0 |
| `steel-core/src/permission/context.rs` | 587 | — | 0 |
| `steel-core/src/permission/expression.rs` | 79 | — | 0 |
| `steel-core/src/permission/groups.rs` | 463 | — | 0 |
| `steel-core/src/permission/key.rs` | 184 | — | 0 |
| `steel-core/src/permission/manager.rs` | 296 | — | 0 |
| `steel-core/src/permission/metadata.rs` | 616 | — | 0 |
| `steel-core/src/permission/mod.rs` | 41 | — | 0 |
| `steel-core/src/permission/rule_expression.rs` | 351 | — | 0 |
| `steel-core/src/permission/set.rs` | 602 | — | 0 |
| `steel-core/src/permission/subject.rs` | 120 | — | 0 |
| `steel-core/src/permission/tests/groups.rs` | 301 | `groups_inherit_contextual_metadata_and_subject_values_break_ties`, `rule_expressions_parse_plain_and_contextual_keys`, `rule_expressions_reject_invalid_context_selectors`, `contextual_group_rules_evaluate_in_the_parsed_context`, `default_config_contains_editable_operator_group`, `group_configuration_rejects_invalid_names_and_missing_operator_group`, `groups_inherit_parent_permissions_once`, `inherited_rules_keep_their_defining_group_priority`, `subject_rules_override_equal_group_rules`, `group_inheritance_rejects_missing_parents_and_cycles`, `default_groups_apply_and_unknown_assignments_are_ignored`, `groups_config_round_trips_through_toml` | 0 |
| `steel-core/src/permission/tests/manager.rs` | 235 | `manager_persists_before_publishing_replacement`, `reload_from_store_swaps_in_the_file_contents`, `reload_from_store_keeps_state_when_the_file_is_invalid`, `manager_without_a_store_cannot_reload`, `manager_keeps_state_when_store_fails`, `manager_keeps_state_when_validation_fails`, `manager_updates_the_latest_config_under_one_lock`, `fallible_update_skips_unchanged_and_rejected_writes` | 0 |
| `steel-core/src/permission/tests/mod.rs` | 442 | `metadata_keys_require_explicit_valid_namespaces`, `metadata_expressions_share_permission_context_syntax`, `metadata_resolution_uses_context_source_priority_and_order`, `keys_validate_segments_and_wildcard_position`, `typed_segments_build_child_keys`, `wildcards_match_only_their_descendants`, `chained_contexts_are_canonical_and_idempotent`, `chained_contexts_reject_conflicting_values`, `contexts_reject_non_roundtrippable_values_and_mismatched_world_domains`, `active_context_rejects_conflicting_custom_values`, `unset_permissions_default_to_deny`, `key_specificity_precedes_context_specificity`, `context_specificity_breaks_equal_key_ties`, `chained_context_is_more_specific_than_one_constraint`, `rule_context_builds_an_equivalent_active_context_for_admin_checks`, `scoped_child_rules_override_parent_fallbacks`, `expression_all_and_any_preserve_unset_state`, `deny_wins_a_complete_tie`, `source_and_group_priority_break_only_equal_specificity_ties`, `set_and_unset_replace_only_exact_rules` | 0 |
| `steel-core/src/physics/collision.rs` | 1491 | `test_intersects_aabb`, `closest_free_position_returns_preferred_center_without_collisions`, `closest_free_position_excludes_expanded_collisions`, `closest_free_position_returns_none_when_fully_blocked`, `block_collision_helper_reports_intersecting_collision_shape`, `live_block_collisions_use_bounded_region_reads`, `prefetched_candidates_use_full_cursor_only_for_special_sections`, `prefetched_callbacks_match_full_cursor_across_chunk_section_and_missing_data`, `collision_helper_reports_intersecting_entity_shape`, `new_shape_collision_helper_ignores_collision_already_touching_old_box`, `pre_move_collisions_exclude_world_border_collisions`, `supporting_block_tie_breaker_matches_vanilla_ordering`, `supporting_block_distance_uses_block_center`, `block_collision_search_bounds_match_vanilla_epsilon_range`, `collision_cursor_type_matches_vanilla_boundary_count`, `collision_cursor_callbacks_match_vanilla_x_y_z_order`, `static_looking_plugin_block_keeps_full_cursor_callback_order`, `collision_shape_resolution_filter_matches_vanilla_cursor_rules` | 0 |
| `steel-core/src/physics/entity_move.rs` | 937 | `test_move_entity_free_fall`, `test_move_entity_land_on_ground`, `test_move_entity_no_collision_in_air`, `test_item_on_ground_with_accumulated_velocity`, `test_item_slightly_above_ground`, `test_crouching_backs_off_from_edge_incrementally`, `test_sneak_edge_treats_entity_collision_as_support`, `test_not_crouching_can_move_off_edge`, `test_entity_collision_clips_horizontal_movement`, `test_world_border_collision_clips_horizontal_movement`, `test_step_up_uses_obstacle_candidate_height`, `test_step_up_rejects_obstacle_above_max_step`, `horizontal_collision_uses_vanilla_mth_equal_tolerance` | 0 |
| `steel-core/src/physics/mod.rs` | 66 | — | 0 |
| `steel-core/src/physics/movement_validation.rs` | 291 | `movement_error_delta_matches_vanilla_y_branch`, `movement_validation_accepts_no_physics_even_with_new_collision`, `movement_validation_rejects_new_collision_for_physical_entity`, `client_movement_tick_reset_updates_good_positions_and_packet_base`, `client_movement_position_sync_reset_clears_packet_counts_and_known_movement`, `client_movement_floating_tracker_counts_only_while_floating`, `client_movement_floating_tracker_resets_when_tick_conditions_do_not_count`, `client_movement_clear_floating_resets_flag_and_counter`, `client_movement_reset_flying_ticks_preserves_current_floating_status` | 0 |
| `steel-core/src/physics/physics_state.rs` | 175 | — | 0 |
| `steel-core/src/physics/shapes.rs` | 444 | `test_collide_no_obstacle`, `test_collide_with_obstacle`, `test_collide_no_overlap_on_other_axes`, `collide_ignores_cross_axis_overlap_below_vanilla_epsilon`, `collide_keeps_cross_axis_overlap_above_vanilla_epsilon`, `test_join_is_not_empty_intersecting`, `test_join_is_not_empty_non_intersecting`, `test_translate_shape`, `merged_offset_face_occludes_respects_shape_offset` | 0 |
| `steel-core/src/player/abilities.rs` | 192 | — | 0 |
| `steel-core/src/player/advancement/mod.rs` | 504 | `persisted_progress_skips_advancements_without_progress`, `apply_persisted_progress_restores_progress_and_resets_client_state`, `apply_persisted_progress_ignores_unknown_advancements` | 0 |
| `steel-core/src/player/advancement/predicate.rs` | 520 | `an_empty_instance_matches_everything`, `item_predicate_accepts_the_listed_item`, `item_predicate_rejects_a_missing_item`, `location_predicate_checks_each_axis`, `player_sneaking_flag_is_honoured`, `unknown_predicates_do_not_block_the_criterion`, `unparseable_instances_fail_open` | 0 |
| `steel-core/src/player/advancement/progress.rs` | 226 | — | 0 |
| `steel-core/src/player/advancement/rewards.rs` | 52 | — | 0 |
| `steel-core/src/player/advancement/saving.rs` | 46 | — | 0 |
| `steel-core/src/player/advancement/visibility_evaluator.rs` | 97 | — | 0 |
| `steel-core/src/player/chat/message_chain.rs` | 386 | `test_link_advance`, `test_link_advance_at_max`, `test_link_descendant`, `test_chain_validation`, `test_chain_out_of_order` | 0 |
| `steel-core/src/player/chat/message_validator.rs` | 242 | `checksum_is_a_crc8_over_the_signature_bytes`, `checksum_is_order_sensitive_and_covers_every_byte`, `empty_acknowledgements_checksum_to_zero` | 0 |
| `steel-core/src/player/chat/mod.rs` | 632 | `operators_are_exempt_from_both_spam_disconnects`, `non_operators_still_trigger_both_spam_disconnects`, `unchanged_profile_key_does_not_reset_the_session`, `expiry_downgrade_precedes_service_key_availability`, `missing_service_keys_ignore_new_session`, `service_signature_result_controls_session_acceptance` | 0 |
| `steel-core/src/player/chat/profile_key.rs` | 463 | `profile_key_signature_binds_uuid_expiry_and_public_key`, `profile_key_signature_rejects_tampered_signature`, `signed_payload_preserves_negative_expiry_millis` | 0 |
| `steel-core/src/player/chat/signature_cache.rs` | 272 | — | 0 |
| `steel-core/src/player/chunk_sender/tests.rs` | 594 | `batch_resolution_keeps_every_still_pending_chunk_in_batch_order`, `batch_resolution_walks_past_prepared_chunks_that_failed_to_encode`, `batch_resolution_drops_chunks_no_longer_pending`, `batch_resolution_drops_chunks_demoted_after_encoding`, `parallel_chunk_encoding_preserves_batch_order_and_cache_entries`, `readiness_demotion_invalidates_prepared_chunk_encoding`, `encoding_cache_requires_holder_identity_and_exact_readiness_generation`, `chunk_batch_ack_updates_pacing_at_the_next_prepare_boundary`, `ack_between_prepare_and_commit_cannot_overwrite_the_current_batch`, `feedback_queue_rejects_unsolicited_and_duplicate_acks`, `feedback_drains_in_arrival_order_and_handles_nan`, `first_ack_expands_the_send_window_and_full_window_resumes_after_feedback`, `fractional_feedback_accumulates_credit_until_a_whole_chunk_can_be_sent`, `feedback_clamps_nonfinite_and_out_of_range_rates_before_preparing`, `filtered_commit_charges_only_sent_chunks_and_keeps_feedback_until_next_prepare`, `invalidated_epoch_discards_batch_without_consuming_quota_or_accepted_feedback`, `chunk_distance_squared_handles_far_chunk_coordinates`, `chunk_distance_squared_handles_valid_world_extremes`, `chunk_distance_squared_saturates_for_invalid_i32_extremes` | 0 |
| `steel-core/src/player/chunk_sender.rs` | 559 | — | 0 |
| `steel-core/src/player/connection/java.rs` | 1782 | `pre_join_custom_payload_uses_serverbound_play_packet_id`, `queued_domain_switch_records_only_perform_respawn_at_connection_gate`, `custom_payload_defaults_to_global_exclusive_scheduling`, `pre_join_allows_initial_play_acknowledgements`, `scheduled_domain_handshake_classification_is_narrow`, `client_tick_end_is_scheduled_for_the_inter_tick_phase`, `packet_execution_classification_separates_local_and_serialized_work`, `inventory_execution_reflects_complete_transaction_boundaries`, `player_command_execution_is_action_sensitive`, `player_action_execution_is_action_sensitive`, `chat_message_and_ack_share_the_serialized_commit_lane`, `cross_player_and_unimplemented_handlers_remain_global_barriers`, `audited_handlers_use_the_narrowest_safe_execution_class`, `keep_alive_remains_on_the_immediate_connection_path`, `chunk_batch_ack_uses_the_immediate_connection_path`, `sender_writes_packets_through_a_boxed_transport` | 0 |
| `steel-core/src/player/connection/mod.rs` | 273 | — | 0 |
| `steel-core/src/player/connection/packet_budget.rs` | 416 | `a_fresh_bucket_spends_its_capacity`, `a_bucket_refills_over_time`, `a_bucket_saturates_at_its_capacity`, `a_packet_flood_is_cut_off_after_one_second_of_allowance`, `exceeding_the_packet_rate_closes_the_connection`, `an_occasional_drop_does_not_kick`, `a_payload_flood_is_cut_off`, `keep_alives_are_never_refused_for_bytes`, `a_configured_rate_limit_tightens_the_allowance`, `a_disabled_rate_limit_falls_back_to_the_ceiling`, `a_huge_rate_limit_cannot_exceed_the_ceiling`, `violations_close_the_connection_after_the_tolerated_count` | 0 |
| `steel-core/src/player/container_counter.rs` | 39 | `starts_at_one`, `wraps_after_one_hundred` | 0 |
| `steel-core/src/player/entity_state.rs` | 450 | `player_pose_dimensions_match_vanilla_avatar`, `player_pose_dimensions_preserve_vanilla_vehicle_attachment`, `swimming_state_continues_while_sprinting_in_water`, `swimming_state_stops_when_current_swimmer_stops_sprinting`, `swimming_state_starts_when_sprinting_underwater_in_water_block`, `swimming_state_does_not_start_from_body_water_only`, `swimming_state_stops_while_passenger`, `player_pose_selection_keeps_pose_when_swimming_cannot_fit`, `player_pose_selection_allows_spectator_desired_pose`, `player_pose_selection_allows_passenger_desired_pose`, `player_pose_selection_falls_back_to_crouching_when_desired_pose_is_blocked`, `player_pose_selection_falls_back_to_swimming_when_crouching_is_blocked` | 0 |
| `steel-core/src/player/experience.rs` | 341 | — | 0 |
| `steel-core/src/player/food_data.rs` | 493 | `player_tick_processes_food_exhaustion_once`, `exhaustion_drains_saturation_then_food`, `add_food_applies_absolute_saturation_without_recomputing_it`, `fast_regen`, `slow_regen`, `starvation_by_difficulty`, `no_regen_when_dead`, `peaceful_never_drains_food` | 0 |
| `steel-core/src/player/game_mode/adventure_mode.rs` | 184 | `may_use_item_at_tests_the_block_behind_the_placement_face`, `block_in_world_predicates_ignore_component_matchers` | 0 |
| `steel-core/src/player/game_mode/block_breaking.rs` | 701 | — | 0 |
| `steel-core/src/player/game_mode/block_interaction.rs` | 479 | `use_item_on_rejects_non_finite_hit_locations` | 0 |
| `steel-core/src/player/game_mode/entity_interaction.rs` | 919 | — | 0 |
| `steel-core/src/player/game_mode/item_interaction/use_on_tests.rs` | 385 | `extra_remainder_merges_into_the_other_hand_without_losing_inventory_changes`, `extra_remainder_drops_when_the_inventory_is_full`, `broken_tool_uses_the_original_statistics_remainder_and_cooldown`, `sign_applicators_count_once_without_item_use_component_side_effects`, `sneaking_bypasses_sign_applicators_in_both_hands`, `glow_ink_rejections_do_not_consume_or_change_the_sign`, `honeycomb_can_wax_blank_signs_without_consuming_in_creative` | 0 |
| `steel-core/src/player/game_mode/item_interaction.rs` | 452 | `use_item_discards_non_finite_rotation_components`, `adventure_mode_requires_can_place_on_before_item_use`, `use_item_refuses_normal_food_at_full_hunger`, `use_item_allows_always_edible_food_at_full_hunger` | 0 |
| `steel-core/src/player/game_mode/mod.rs` | 115 | — | 0 |
| `steel-core/src/player/game_mode/player_methods.rs` | 357 | — | 0 |
| `steel-core/src/player/game_mode/raycast.rs` | 100 | — | 0 |
| `steel-core/src/player/game_mode/swing.rs` | 40 | — | 0 |
| `steel-core/src/player/game_mode/tests.rs` | 67 | `invalid_attack_targets_include_xp_orbs`, `changing_current_records_previous_mode`, `setting_same_mode_keeps_previous_mode`, `persistent_restore_sets_current_and_previous`, `initial_state_has_no_previous_mode` | 0 |
| `steel-core/src/player/health_sync.rs` | 60 | — | 0 |
| `steel-core/src/player/item_cooldowns.rs` | 176 | `cooldown_blocks_until_duration_ticks_pass`, `explicit_group_is_shared_between_items` | 0 |
| `steel-core/src/player/lifecycle/mod.rs` | 554 | `domain_switch_requires_matching_phase_and_token`, `client_loaded_flag_is_explicit`, `client_load_timeout_eventually_marks_loaded`, `joined_world_flag_is_explicit` | 0 |
| `steel-core/src/player/lifecycle/respawn.rs` | 903 | `personal_respawn_loads_only_chunks_touched_by_standup_search` | 0 |
| `steel-core/src/player/lifecycle/respawn_restore.rs` | 481 | `replacement_retains_ender_chest_for_death_and_end_respawns`, `replacement_is_fresh_detached_entity_in_same_session`, `respawn_preserves_enchantment_seed_independently_of_experience_transfer`, `restore_all_copies_vanilla_restore_state_without_mutating_source`, `death_restore_keeps_fresh_state_unless_inventory_transfer_is_requested` | 0 |
| `steel-core/src/player/lifecycle/spawn_sync.rs` | 454 | — | 0 |
| `steel-core/src/player/lifecycle/world_transition.rs` | 117 | — | 0 |
| `steel-core/src/player/mod.rs` | 3692 | — | 0 |
| `steel-core/src/player/movement/input.rs` | 170 | `movement_input_matches_vanilla_intent_axes`, `input_flags_round_trip` | 0 |
| `steel-core/src/player/movement/mod.rs` | 1317 | `test_clamp_horizontal`, `test_clamp_vertical`, `test_wrap_degrees`, `sprinting_charges_food_exhaustion_once_per_move`, `floating_validation_exempts_the_riptide_spin_attack`, `floating_validation_exempts_levitation_like_vanilla` | 0 |
| `steel-core/src/player/movement/state.rs` | 386 | `movement_state_starts_with_zero_known_client_movement`, `client_tick_end_clears_known_movement_without_accepted_movement`, `client_tick_end_keeps_known_movement_after_accepted_movement`, `movement_state_tracks_last_client_input`, `tick_reset_updates_both_good_positions_and_packet_count`, `position_sync_reset_clears_packet_counts_and_known_movement`, `floating_tracker_counts_only_while_client_is_floating`, `floating_tracker_resets_when_tick_conditions_do_not_count`, `vehicle_tick_reset_tracks_active_vehicle_id`, `vehicle_last_good_update_is_guarded_by_active_vehicle_id`, `vehicle_floating_update_is_guarded_by_active_vehicle_id`, `vehicle_state_clears_when_no_controlled_vehicle_is_active`, `reset_flying_ticks_applies_to_vehicle_counter` | 0 |
| `steel-core/src/player/movement/teleport.rs` | 58 | — | 0 |
| `steel-core/src/player/permissions.rs` | 169 | — | 0 |
| `steel-core/src/player/player_data.rs` | 596 | — | 0 |
| `steel-core/src/player/player_data_storage/file_storage.rs` | 506 | — | 0 |
| `steel-core/src/player/player_data_storage/known_players.rs` | 76 | — | 0 |
| `steel-core/src/player/player_data_storage/memory_storage.rs` | 82 | — | 0 |
| `steel-core/src/player/player_data_storage/mod.rs` | 846 | — | 0 |
| `steel-core/src/player/player_data_storage/permissions.rs` | 428 | `subject_file_round_trip_preserves_groups_and_contextual_rules`, `subject_file_rejects_invalid_group_names`, `empty_subject_state_removes_the_file_entry`, `subject_file_rejects_duplicate_uuid_spellings` | 0 |
| `steel-core/src/player/player_data_storage/stats.rs` | 94 | — | 0 |
| `steel-core/src/player/player_data_storage/tests.rs` | 1046 | `atomic_path_replacement_retains_the_last_committed_generation`, `interrupted_permission_publication_recovers_before_the_next_update`, `corrupt_live_permission_file_does_not_fall_back_to_its_backup`, `interrupted_first_known_player_publication_discards_its_temporary_file`, `corrupt_known_player_cache_loads_as_empty`, `incompatible_known_player_cache_version_loads_as_empty`, `interrupted_first_permission_publication_does_not_apply_uncommitted_access`, `known_player_cache_round_trips_and_rejects_stale_writes`, `known_player_cache_persists_vanillas_mru_limit`, `a_version_10_player_file_still_loads`, `a_version_11_player_file_still_loads`, `an_unknown_player_storage_version_is_rejected`, `a_newer_payload_data_version_is_rejected`, `player_file_roundtrip_preserves_domain_world_data`, `player_file_roundtrip_preserves_absent_previous_game_mode`, `global_file_roundtrip_preserves_last_active_domain`, `permission_subject_snapshot_removes_noncanonical_uuid_key`, `player_file_roundtrip_preserves_root_vehicle`, `player_file_roundtrip_preserves_respawn_config`, `stale_player_payload_version_is_rejected`, `player_stats_file_roundtrips`, `duplicate_stat_fails_conversion`, `in_memory_player_data_round_trips_per_domain`, `disk_player_data_advancements_round_trip`, `in_memory_known_player_cache_rejects_stale_writes`, `in_memory_permission_subjects_round_trip` | 0 |
| `steel-core/src/player/player_inventory/container.rs` | 249 | — | 0 |
| `steel-core/src/player/player_inventory/core.rs` | 316 | — | 0 |
| `steel-core/src/player/player_inventory/equipment.rs` | 370 | — | 0 |
| `steel-core/src/player/player_inventory/mod.rs` | 182 | — | 0 |
| `steel-core/src/player/player_inventory/player_handlers.rs` | 1407 | — | 0 |
| `steel-core/src/player/player_inventory/tests.rs` | 2148 | `death_drops_destroy_vanishing_items`, `vanilla_inventory_nbt_contains_main_slots_only`, `replacement_copy_preserves_all_logical_slots_and_selection`, `replacement_copy_does_not_drain_or_share_storage_with_source`, `add_marks_changed_when_stack_fills_existing_slot`, `add_to_selected_existing_slot_marks_inventory_changed`, `add_to_empty_selected_slot_marks_inventory_changed`, `add_merges_into_existing_offhand_stack_before_main_inventory`, `contains_stack_compares_components`, `filled_result_replaces_single_survival_hand_stack`, `filled_result_adds_result_for_stacked_survival_hand_stack`, `filled_result_creative_limited_keeps_matching_held_stack`, `filled_result_creative_limited_adds_missing_result_without_consuming_hand`, `filled_result_empty_result_still_consumes_survival_hand_stack`, `filled_result_creative_unlimited_returns_unadded_result_to_drop`, `clear_content_counts_equipment_items`, `container_traversal_matches_visible_slot_indices`, `equipment_trait_aliases_vanilla_container_indices`, `mutable_container_slice_exposes_all_logical_slots`, `main_inventory_search_does_not_use_equipment_slots`, `equipment_main_hand_follows_selected_slot`, `non_empty_equipment_items_uses_selected_item_as_main_hand`, `packet_selected_slot_rejects_invalid_values_without_wrapping`, `shrink_item_in_hand_marks_inventory_changed`, `split_item_in_hand_marks_inventory_changed`, `mutating_only_held_item_components_marks_inventory_changed`, `hurt_item_in_hand_marks_inventory_changed`, `hurt_and_convert_item_in_hand_damages_without_breaking`, `hurt_and_convert_item_in_hand_replaces_broken_item`, `swap_hands_swaps_selected_and_offhand`, `equippable_single_item_moves_to_empty_armor_slot`, `equippable_swap_respects_prevent_armor_change_effect`, `repair_with_xp_repairs_damaged_mending_item`, `repair_with_xp_returns_leftover_when_item_is_fully_repaired`, `equippable_stack_moves_one_item_and_returns_old_equipment_to_inventory`, `disconnected_menu_removal_drops_transient_items`, `drained_items_return_without_player_inventory_slots`, `menu_item_return_policy_preserves_world_changes_only`, `menu_tick_hook_can_close_the_current_menu`, `menu_click_hook_can_close_the_current_menu`, `dead_player_container_click_only_resynchronizes`, `malformed_quickcraft_encoding_resets_active_drag`, `malformed_non_quickcraft_click_resets_active_drag`, `closing_menu_while_dead_does_not_return_items_to_inventory`, `programmatic_out_of_range_menu_click_is_ignored`, `menu_open_hook_can_close_the_new_menu`, `menu_open_hook_can_replace_the_new_menu`, `menu_hook_defers_a_factory_that_reuses_its_locked_container`, `open_menu_rejects_a_factory_with_the_wrong_container_id`, `menu_removed_hook_can_open_a_replacement`, `terminal_menu_removal_returns_carried_item_and_rejects_replacement`, `terminal_menu_removal_skips_queued_factory_and_drains_base_menu`, `pending_terminal_removal_preserves_drop_disposition`, `menu_open_stops_when_predecessor_removal_turns_terminal`, `prepared_menu_is_cleaned_when_replacement_removal_turns_terminal`, `deferred_factory_is_not_run_when_earlier_close_turns_terminal`, `terminal_removal_stays_active_while_pending_menu_cleanup_runs`, `opening_a_menu_closes_a_replacement_created_during_removal`, `creative_crafting_grid_updates_the_result_slot`, `container_button_clicks_reach_the_open_menu_only_for_its_container_id`, `resolve_beacon_effect_separates_absent_from_unknown_ids`, `editing_a_writable_book_stores_the_pages`, `signing_a_writable_book_creates_a_written_book`, `book_updates_outside_the_hotbar_and_offhand_are_ignored`, `book_updates_on_a_non_book_slot_are_ignored` | 0 |
| `steel-core/src/player/post_effects.rs` | 161 | `a_fresh_list_is_pushed_once_even_when_empty`, `only_changes_make_the_list_dirty`, `clearing_an_empty_list_reports_no_change`, `replace_keeps_order_and_drops_duplicates` | 0 |
| `steel-core/src/player/profile/known_players.rs` | 251 | `record_updates_an_existing_uuid_name`, `record_reassigns_a_duplicate_name_to_the_latest_uuid`, `record_renews_an_unchanged_identity`, `name_resolution_expires_and_removes_stale_entries` | 0 |
| `steel-core/src/player/profile/lookup.rs` | 163 | `profile_lookup_url_uses_mojangs_default_endpoint`, `profile_lookup_url_uses_the_configured_endpoint` | 0 |
| `steel-core/src/player/profile/mod.rs` | 52 | `validates_vanilla_player_names`, `offline_uuid_matches_vanilla_name_uuid` | 0 |
| `steel-core/src/player/recipe_book.rs` | 540 | `a_new_book_is_empty_and_closed`, `adding_a_known_recipe_allocates_a_display_index`, `unknown_recipes_are_rejected`, `removal_keeps_the_display_numbering_stable`, `join_packets_carry_every_unlocked_display`, `add_all_registered_unlocks_the_whole_registry`, `update_recipes_lists_every_stonecutter_recipe` | 0 |
| `steel-core/src/player/recipe_placement.rs` | 937 | `clear_grid_merges_equal_stacks_into_one_free_slot`, `clear_grid_fails_when_the_inventory_is_full`, `clear_grid_accepts_a_partially_filled_matching_slot` | 0 |
| `steel-core/src/player/session.rs` | 215 | `replacement_requires_session_ownership_and_exact_current_player`, `closed_session_cannot_be_bound_again` | 0 |
| `steel-core/src/player/sleep.rs` | 350 | `sleep_admission_rejects_a_player_outside_vanilla_range`, `sleep_admission_checks_both_bed_halves_for_obstruction` | 0 |
| `steel-core/src/player/sleep_state.rs` | 62 | `sleep_counter_matches_vanilla_wake_animation_window` | 0 |
| `steel-core/src/player/spam_throttler.rs` | 59 | `threshold_at_or_below_zero_disables_throttling`, `increment_reaches_threshold_and_tick_decays` | 0 |
| `steel-core/src/player/stats_counter.rs` | 413 | `stat_counter_query_dirty_and_modifications`, `overflow_cap`, `no_underflow_cap`, `reset_stats` | 0 |
| `steel-core/src/player/tests.rs` | 1988 | `client_information_initializes_player_cosmetic_metadata`, `play_client_information_dirties_changed_cosmetic_metadata_once`, `last_client_move_intent_matches_vanilla_directional_inputs`, `advancing_domain_residence_invalidates_stale_restore_owners`, `permission_state_replacement_is_versioned_and_keeps_both_rule_sets`, `respawn_request_is_allowed_after_dead_reconnect`, `ai_step_copies_player_yaw_to_head_yaw`, `respawn_request_is_ignored_while_alive`, `respawn_request_uses_health_not_death_processed_guard`, `end_credits_respawn_keeps_vanilla_attribute_data_only`, `death_keeps_menu_items_until_entity_removal`, `death_removes_tracked_entities_from_dead_players_client`, `death_respawn_cleanup_drops_menu_items_before_fresh_player_construction`, `end_credits_removes_all_menus_before_detaching`, `admitted_world_change_prevents_end_credits_detach`, `duplicate_exact_player_admission_cleans_existing_membership`, `disabled_damage_game_rule_matches_vanilla_player_damage_gates`, `disabled_damage_game_rule_ignores_unrelated_damage`, `hurt_uses_explicit_world_difficulty`, `conditional_damage_does_not_scale_without_a_living_non_player_cause`, `player_damage_applies_armor_and_absorption`, `player_absorption_amount_clamps_to_attribute_range`, `player_damage_hurts_armor_equipment`, `player_attack_applies_thorns_damage_and_breaks_enchanted_armor`, `equipping_player_target_uses_inventory_equipment_storage`, `living_tick_detects_raw_inventory_equipment_mutation`, `fresh_death_respawn_redetects_unchanged_kept_equipment`, `fresh_death_respawn_discards_stale_pending_equipment_change`, `equipment_detection_tracks_selected_main_hand`, `equipment_detection_suppresses_exact_hand_swap_packet`, `equipment_detection_coalesces_before_tracker_drain`, `clear_matching_items_uses_inventory_crafting_then_carried_order`, `point_level_up_sound_uses_first_crossed_five_level_boundary`, `point_grants_update_entity_score_with_java_wrapping`, `persistent_player_data_restores_independent_experience_fields_and_score`, `enchanting_spends_levels_and_rerolls_the_seed`, `only_a_zero_saved_enchantment_seed_is_rerolled`, `saved_enchantment_seed_round_trips_through_player_data`, `persistent_player_data_restores_equipment_inventory_slots`, `effect_visibility_refresh_preserves_spectator_invisibility`, `block_action_restriction_precedes_redstone_ore_attack`, `block_action_restriction_is_rechecked_before_block_destruction`, `creative_drop_throttle_survives_respawn_replacement`, `extra_items_created_on_use_fill_inventory_when_there_is_room`, `extra_items_created_on_use_are_dropped_when_inventory_is_full`, `drinking_honey_bottle_from_full_inventory_drops_the_remainder_through_the_tick_loop`, `throttle_player_dropping_items_from_creative_menu`, `monster_prevents_rest_blocks_sleeping_near_a_bed`, `sword_does_not_destroy_blocks_in_creative`, `spawn_sends_view_and_simulation_distances_in_vanilla_order` | 0 |
| `steel-core/src/player/tick_state.rs` | 149 | `tick_count_advances_with_wrapping_semantics`, `attack_strength_ticker_advances_and_resets`, `xp_pickup_delay_decrements_during_tick`, `level_up_sound_uses_vanillas_strict_hundred_tick_cooldown`, `block_ack_keeps_highest_sequence_until_taken` | 0 |
| `steel-core/src/player/title.rs` | 40 | — | 0 |
| `steel-core/src/poi/mod.rs` | 12 | — | 0 |
| `steel-core/src/poi/poi_instance.rs` | 63 | — | 0 |
| `steel-core/src/poi/poi_set.rs` | 207 | `test_pack_unpack`, `test_add_remove_poi`, `test_get_by_type_and_occupation` | 0 |
| `steel-core/src/poi/poi_storage.rs` | 501 | `horizontal_square_query_matches_y_unbounded_vanilla_search` | 0 |
| `steel-core/src/portal/end_gateway.rs` | 333 | `zero_gateway_position_has_zero_search_direction`, `exit_search_candidates_cover_vanilla_probe_range`, `block_square_chunks_cover_radius_across_chunk_edges` | 0 |
| `steel-core/src/portal/end_portal.rs` | 165 | `end_platform_origin_is_below_vanilla_end_spawn`, `end_entry_player_position_is_one_block_below_spawn_center`, `end_platform_prewarm_covers_negative_z_edge` | 0 |
| `steel-core/src/portal/mod.rs` | 671 | `portal_processor_reaches_transition_after_vanilla_threshold`, `portal_processor_does_not_increment_when_teleport_is_disallowed`, `portal_processor_decays_when_entity_leaves_portal`, `portal_processor_updates_entry_position_only_after_tick_is_consumed`, `nether_portal_transition_rule_matches_player_invulnerability`, `portal_transition_time_is_clamped_non_negative`, `relative_portal_transition_rotates_velocity_by_yaw_delta`, `relative_position_transition_resolves_only_flagged_axes`, `pitch_relative_transition_uses_absolute_yaw_and_relative_pitch`, `resolved_rotation_clamps_pitch_like_vanilla`, `absolute_transition_replaces_velocity_and_rotation`, `post_transition_composition_preserves_vanilla_order` | 0 |
| `steel-core/src/portal/nether_portal.rs` | 237 | `search_radius_matches_vanilla_portal_forcer_targets`, `prewarm_radius_covers_poi_search_and_exit_rectangle_edges` | 0 |
| `steel-core/src/portal/portal_shape.rs` | 477 | `relative_portal_position_matches_vanilla_axis_math`, `relative_portal_position_clamps_right_and_up_offsets`, `relative_portal_position_uses_vanilla_fallbacks_when_entity_fills_rectangle` | 0 |
| `steel-core/src/scoreboard.rs` | 1700 | `display_slots_bind_and_clear_objectives`, `display_name_defaults_to_the_objective_name`, `snapshots_cover_every_stored_entry`, `score_value_and_lock_state_are_independent`, `stopwatches_advance_one_tick_at_a_time`, `read_only_objective_rejects_score_writes`, `team_assignment_replaces_prior_membership`, `persisted_scoreboard_round_trips_and_becomes_clean_after_save`, `mutation_after_snapshot_remains_dirty_when_snapshot_is_marked_saved`, `a_trigger_objective_starts_locked_until_enabled`, `a_trigger_write_relocks_the_score`, `a_trigger_write_needs_an_existing_unlocked_entry`, `criteria_are_classified_like_vanilla`, `a_dummy_objective_is_not_a_trigger`, `a_statistic_objective_cannot_be_written_by_commands`, `domains_keep_independent_scoreboards` | 0 |
| `steel-core/src/server/bans.rs` | 358 | `a_banned_player_is_rejected`, `a_pardon_lets_the_player_back_in`, `a_temporary_ban_expires_on_its_own`, `a_second_ban_replaces_the_first`, `address_bans_are_separate_from_player_bans`, `an_empty_reason_uses_the_vanilla_default`, `listings_are_sorted_by_name` | 0 |
| `steel-core/src/server/boss_bars.rs` | 212 | `insert_get_and_remove_round_trip`, `update_reports_unknown_ids`, `all_lists_bars_sorted_by_id` | 0 |
| `steel-core/src/server/broadcasting.rs` | 474 | — | 0 |
| `steel-core/src/server/jobs/domain_switch.rs` | 541 | — | 0 |
| `steel-core/src/server/jobs/mod.rs` | 392 | `pending_job_is_polled_until_finished`, `jobs_spawned_during_poll_wait_until_next_tick`, `poll_now_or_spawn_drops_finished_job`, `poll_now_or_spawn_queues_pending_job_after_first_poll` | 0 |
| `steel-core/src/server/jobs/teleport.rs` | 1057 | — | 0 |
| `steel-core/src/server/known_players.rs` | 264 | — | 0 |
| `steel-core/src/server/mod.rs` | 1346 | — | 0 |
| `steel-core/src/server/packet_processor.rs` | 2105 | `per_player_packet_limit_rejects_before_assigning_a_sequence`, `scheduled_respawn_is_retained_if_domain_switch_queues_before_worker_gate`, `queued_packets_resolve_the_player_bound_when_they_start`, `per_player_byte_limit_is_independent_of_packet_count`, `production_limit_allows_a_watchdog_window_of_mounted_client_traffic`, `independent_session_limits_do_not_accumulate_globally`, `discarding_stale_session_keeps_replacement_session_work`, `paused_fifo_is_re_admitted_after_unrelated_global_work`, `pausing_active_player_session_defers_its_tail_until_exact_resume`, `stale_pause_token_cannot_resume_a_later_pause`, `paused_packets_do_not_hold_tick_drain_open`, `active_packet_remains_charged_until_completion`, `discarding_lane_removes_hidden_barriers_and_keeps_active_work_charged`, `discarding_idle_lane_removes_ready_exclusive_barrier`, `queued_packets_start_in_submission_order_when_opened`, `packet_lane_only_allows_one_active_handler`, `serialized_packet_overlaps_player_local_work_but_not_another_serialized_packet`, `serialized_packet_hidden_in_an_active_lane_preserves_serialized_order`, `blocking_worker_bypasses_active_serialized_work_for_player_local_work`, `exclusive_packet_waits_for_active_work_and_blocks_later_packets`, `exclusive_packet_waits_for_serialized_work_and_blocks_player_local_work`, `exclusive_packet_hidden_in_an_active_lane_still_blocks_later_lanes`, `closed_phase_retains_new_packets_for_the_next_open_phase`, `tick_drain_only_processes_packets_submitted_before_its_cutoff`, `tick_drain_orders_hidden_serialized_and_exclusive_work_before_its_cutoff`, `tick_drain_waits_for_active_packet_work`, `overload_progress_waits_for_one_active_packet_to_finish`, `stopped_queue_discards_pending_and_future_work`, `stopping_with_active_work_keeps_completion_accounting_valid`, `stopping_with_active_serialized_and_local_work_clears_all_queue_state`, `blocking_worker_only_starts_work_during_the_open_phase` | 0 |
| `steel-core/src/server/permissions.rs` | 224 | — | 0 |
| `steel-core/src/server/player_admission.rs` | 770 | — | 0 |
| `steel-core/src/server/player_lifecycle.rs` | 641 | — | 0 |
| `steel-core/src/server/pregen.rs` | 658 | `pregen_size_accepts_zero_as_disabled`, `pregen_size_accepts_odd_side_lengths`, `pregen_size_rejects_even_side_lengths`, `pregen_size_rejects_negative_side_lengths`, `pregen_window_order_keeps_consecutive_dependency_areas_local`, `pregen_window_size_requires_a_positive_integer`, `pregen_window_size_must_fit_unload_backpressure_budget` | 0 |
| `steel-core/src/server/rcon.rs` | 297 | `packet_round_trips_through_the_wire_format`, `empty_body_keeps_the_protocol_nuls` | 0 |
| `steel-core/src/server/registry_cache.rs` | 363 | `every_synced_tag_matches_pinned_vanilla_membership`, `synchronized_registries_match_vanilla_order_and_known_pack_elision` | 0 |
| `steel-core/src/server/run_loop.rs` | 880 | `chunk_send_commit_rechecks_live_world_membership` | 0 |
| `steel-core/src/server/schedule.rs` | 348 | `a_due_command_is_removed_and_returned_once`, `replace_keeps_only_the_newest_schedule_of_a_name`, `append_keeps_both_schedules`, `due_entries_are_ordered_by_their_tick`, `clear_removes_only_the_named_schedule`, `clear_all_drops_everything`, `a_function_schedule_is_named_by_its_identifier`, `the_schedule_mode_parser_accepts_vanilla_names`, `a_schedule_at_the_current_tick_runs_immediately` | 0 |
| `steel-core/src/server/scoreboard_sync.rs` | 352 | `team_colour_ordinals_match_vanilla_team_color`, `team_parameters_encode_friendly_fire_as_option_bit`, `collision_rule_ordinals_follow_friendly_fire`, `display_slot_ordinals_match_vanilla`, `team_visibility_ordinals_match_vanilla_stream_codec` | 0 |
| `steel-core/src/server/service_keys.rs` | 248 | `parses_player_certificate_keys`, `empty_player_certificate_keys_disable_validation`, `malformed_player_certificate_key_rejects_snapshot`, `malformed_profile_property_key_rejects_snapshot`, `initial_failures_use_authlib_backoff_cap` | 0 |
| `steel-core/src/server/tests/connection_lifecycle.rs` | 325 | `blocked_disconnect_write_does_not_delay_player_removal`, `duplicate_login_evicts_relocating_player_and_waits_for_disconnect_admission_release`, `duplicate_login_wait_matches_vanillas_deadline_ordering` | 0 |
| `steel-core/src/server/tests/damage_history.rs` | 104 | `disconnect_releases_self_damage_history_while_frozen`, `disconnect_releases_mutual_damage_history_while_frozen`, `disconnect_outside_world_membership_releases_history_while_frozen` | 0 |
| `steel-core/src/server/tests/game_time.rs` | 446 | `game_time_domains_freeze_steps_sprint_and_transfer_damage`, `game_time_full_partial_periodic_packets_keep_world_clocks_independent`, `game_time_startup_and_chunk_reload_use_the_configured_primary`, `game_time_rejects_ephemeral_primary_before_touching_derived_save` | 0 |
| `steel-core/src/server/tests/player_limit.rs` | 234 | `max_players_counts_admitted_players_not_pending_preparation`, `max_players_rechecks_group_bypass_after_preparation`, `max_players_rejected_prepared_join_disconnects_and_releases_uuid`, `max_players_bypass_requires_explicit_boolean_metadata` | 0 |
| `steel-core/src/server/tests/projectile_owners.rs` | 124 | `uuid_pearl_impact_overworld_to_nether`, `uuid_pearl_impact_nether_to_overworld`, `same_dimension_uuid_pearl_impact`, `cached_owner_pearl_impact_between_dimensions` | 0 |
| `steel-core/src/server/tests.rs` | 4751 | `saved_location_planning_honors_explicit_world_selection`, `domain_switch_job_progresses_across_chunk_scheduling_boundaries`, `domain_restore_jobs_wait_while_their_owner_has_no_live_membership`, `domain_restore_jobs_follow_same_session_player_replacement`, `domain_restore_jobs_do_not_follow_a_different_player_session`, `domain_detach_snapshots_pending_restores_before_stale_jobs_finish`, `domain_detach_invalidates_an_encoded_source_chunk_batch`, `detached_domain_switch_owns_disconnect_snapshot_exclusively`, `failed_target_admission_preserves_only_valid_target_restores`, `rejected_queued_domain_switch_retries_deferred_respawn_request`, `portal_job_validity_rechecks_vanilla_portal_eligibility`, `first_domain_visit_resets_domain_scoped_player_data`, `first_advancement_flush_resets_client_without_progress`, `command_world_scope_survives_entity_transforms`, `execute_as_entity_transform_uses_receiver_with_initiator_permissions`, `command_gameplay_availability_tracks_exact_domain_residence`, `player_world_selection_uses_one_token_owned_route`, `same_domain_world_selection_waits_for_safe_spawn_and_full_chunk_square`, `same_domain_world_selection_applies_a_world_entry`, `cross_domain_world_selection_overrides_the_destination_or_only_the_rotation`, `awkward_saved_data_does_not_break_an_exact_or_rotation_entry`, `domain_command_accepts_its_forms_and_keeps_the_players_own_rotation`, `domain_and_teleport_check_the_location_and_rotation_like_vanilla`, `disconnect_before_the_vehicle_restore_keeps_the_vehicle`, `initial_player_info_precedes_entity_spawn_for_existing_players`, `client_information_broadcasts_hat_updates_only_when_the_hat_bit_changes`, `initial_admission_installs_restores_before_scheduling_jobs`, `player_disconnect_detaches_before_async_persistence`, `online_respawn_replacement_rejects_admission_and_stale_owners`, `simultaneous_disconnects_batch_tab_list_removal`, `online_player_snapshot_includes_player_detached_for_end_credits`, `death_respawn_replaces_the_live_player_incarnation`, `end_credits_respawn_replaces_the_detached_player_incarnation`, `positive_thread_count_is_capped_to_available_threads`, `zero_thread_count_keeps_pool_default`, `tab_list_distinguishes_recent_and_five_second_tick_times`, `uncached_uuid_target_is_preserved_in_online_mode`, `uncached_uuid_target_is_preserved_in_offline_mode`, `uncached_uuid_profile_uses_a_canonical_display_label`, `known_player_changes_are_coalesced_while_a_save_is_running`, `known_player_change_cannot_be_lost_when_a_worker_becomes_idle`, `known_player_change_during_a_failed_save_gets_a_follow_up`, `known_player_cache_closes_only_after_the_worker_is_idle`, `permission_updates_reject_only_new_unknown_group_assignments`, `command_source_and_operator_checks_use_published_subject_state`, `renamed_join_message_only_reaches_existing_players`, `effective_permissions_reflect_published_group_revocation`, `nether_portal_entry_obeys_allow_entering_nether_gamerule`, `can_teleport_passenger_gate_only_applies_to_end_return`, `ender_pearl_end_return_requires_owner_seen_credits_when_owner_is_player`, `damage_command_records_by_entity_as_the_responsible_player`, `sync_tab_list_sends_single_batched_update_to_joiner`, `title_command_delivers_vanilla_packets_to_recorded_connections`, `setblock_command_places_blocks_and_keep_mode_skips_occupied_positions`, `a_second_server_bootstraps_in_the_same_process`, `offline_startup_still_requests_the_services_keys`, `save_and_shutdown_writes_player_and_world_data`, `save_and_shutdown_disconnects_players_and_claims_their_removal` | 0 |
| `steel-core/src/server/tick_overload.rs` | 161 | `on_time_loop_keeps_its_schedule`, `small_backlog_is_still_replayed`, `large_backlog_is_skipped_and_resyncs_the_clock`, `a_second_report_waits_for_the_report_gap`, `a_sprint_restarts_the_report_gap`, `a_backlog_is_dropped_again_once_the_report_gap_has_passed`, `a_zero_length_tick_is_ignored` | 0 |
| `steel-core/src/server/tick_rate_manager.rs` | 362 | — | 0 |
| `steel-core/src/server/tps_bar.rs` | 209 | `progress_maps_tps_onto_the_bar`, `colours_follow_the_tab_list_thresholds`, `bar_text_reports_tps_and_all_mspt_views` | 0 |
| `steel-core/src/server/whitelist.rs` | 243 | `a_disabled_whitelist_lets_everyone_in`, `an_enabled_empty_whitelist_rejects_everyone`, `an_enabled_whitelist_admits_listed_players_only`, `removing_a_player_takes_effect_immediately`, `adding_the_same_player_twice_reports_the_duplicate`, `a_rename_updates_the_listed_name`, `entries_are_sorted_by_name`, `toggling_reports_the_previous_state` | 0 |
| `steel-core/src/server/world_changes.rs` | 632 | — | 0 |
| `steel-core/src/server/world_tick_workers.rs` | 195 | `persistent_workers_tick_every_world_across_boundaries`, `shared_time_is_published_while_primary_worker_is_delayed` | 0 |
| `steel-core/src/server/worlds.rs` | 325 | `domain_uuid_lookup_excludes_replaced_worlds_without_retaining_worlds`, `nether_portal_target_names_follow_vanilla_level_keys`, `end_entry_portal_target_name_is_only_for_non_end_sources` | 0 |
| `steel-core/src/vanilla_26_3_mega_parity.rs` | 2744 | `vanilla_26_3_grand_unified_full_server_parity_and_no_lag_suite` | 0 |
| `steel-core/src/world/block_entity_ticker.rs` | 462 | `additions_during_phase_wait_and_follow_between_phase_additions`, `rebind_before_turn_uses_new_owner_in_the_original_slot`, `remove_then_add_during_phase_creates_a_pending_tail`, `frozen_phase_merges_pending_and_prunes_unbound_without_callbacks` | 0 |
| `steel-core/src/world/block_event.rs` | 287 | `ordered_queue_suppresses_only_exact_duplicates`, `server_queue_defers_then_dispatches_the_current_block_event` | 0 |
| `steel-core/src/world/block_region.rs` | 542 | `stable_air_region_requires_full_data_and_tracks_live_block_writes`, `region_reuses_section_reads_across_chunk_and_section_boundaries`, `oversized_region_uses_streaming_reads` | 0 |
| `steel-core/src/world/block_updates/mod.rs` | 744 | — | 0 |
| `steel-core/src/world/block_updates/neighbor_updater.rs` | 364 | `nested_updates_interrupt_multi_update_and_keep_sibling_fifo_order`, `chained_update_limit_counts_enqueued_tasks_and_reports_only_first_skip`, `zero_limit_still_starts_and_resets_a_root_run_without_executing_it` | 0 |
| `steel-core/src/world/border.rs` | 813 | `default_border_matches_vanilla_initial_settings`, `border_bounds_are_clamped_to_absolute_max_size`, `block_position_bounds_use_vanilla_inclusive_min_exclusive_max`, `clamp_to_bounds_matches_vanilla_blockpos_containing`, `moving_border_ticks_to_target_size`, `close_to_border_matches_vanilla_distance_gate`, `collision_shape_represents_outside_of_border`, `outside_damage_is_ignored_inside_safe_zone`, `outside_damage_uses_vanilla_minimum_damage`, `outside_damage_uses_vanilla_floor_after_scaling`, `outside_damage_is_disabled_when_damage_per_block_is_not_positive` | 0 |
| `steel-core/src/world/broadcasts.rs` | 344 | — | 0 |
| `steel-core/src/world/clock.rs` | 460 | `initializes_every_registered_clock`, `separate_worlds_keep_independent_clock_state`, `rate_accumulates_partial_ticks_like_vanilla`, `pause_and_advance_time_gate_network_rate_and_ticks`, `repeating_time_marker_moves_strictly_forward`, `manager_round_trips_through_toml` | 0 |
| `steel-core/src/world/domain_entity_directory.rs` | 50 | — | 0 |
| `steel-core/src/world/dragon_fight.rs` | 919 | `saved_data_round_trips_every_field`, `every_respawn_stage_name_round_trips`, `gateway_ring_is_a_seeded_permutation`, `a_live_dragon_owns_the_bar_and_blocks_respawn`, `the_first_kill_drops_the_egg_and_the_second_does_not`, `killing_a_crystal_on_the_exit_portal_column_arms_the_respawn_once` | 0 |
| `steel-core/src/world/entity_management.rs` | 826 | — | 0 |
| `steel-core/src/world/environment.rs` | 597 | `overworld_sky_light_uses_generated_day_timeline`, `overworld_sky_light_interpolates_sunset_from_generated_keyframes`, `overworld_sun_angle_uses_vanilla_cubic_bezier_easing`, `sky_light_level_applies_vanilla_weather_alpha_layers`, `fixed_nether_sky_light_uses_dimension_attribute`, `sky_darkening_matches_vanilla_integer_cast`, `monsters_burn_follows_the_generated_daylight_window`, `monsters_burn_snaps_at_keyframes_instead_of_interpolating`, `monsters_burn_is_off_in_dimensions_without_the_day_timeline` | 0 |
| `steel-core/src/world/events.rs` | 153 | — | 0 |
| `steel-core/src/world/explosion/mod.rs` | 553 | — | 0 |
| `steel-core/src/world/explosion/server/block_effects.rs` | 168 | `fire_creation_draws_before_testing_air_and_support`, `combined_explosion_drops_never_exceed_vanilla_stack_limit` | 0 |
| `steel-core/src/world/explosion/server/block_rays/cache.rs` | 482 | — | 0 |
| `steel-core/src/world/explosion/server/block_rays/tests/gameplay.rs` | 230 | `radius_four_tnt_matches_vanilla_order_and_fuse_bounds` | 0 |
| `steel-core/src/world/explosion/server/block_rays/tests.rs` | 632 | `deterministic_empty_world_rays_match_the_java_hash_set_fixture`, `precomputed_ray_steps_match_java_bit_digest`, `maximum_radius_air_rays_match_the_java_membership_fixture`, `unusual_radii_retain_vanilla_ray_sampling_and_bounds_behavior`, `immutable_rays_match_sequential_lane_in_a_complete_region`, `cached_ray_decisions_keep_uncached_order_at_already_affected_positions`, `bounded_floor_matches_rust_floor_cast_across_i32_domain`, `incomplete_bounded_region_falls_back_before_calculator_hooks`, `bounded_immutable_reader_covers_maximum_power_rays`, `java_block_pos_set_resizes_on_a_ninth_collision_before_capacity_sixty_four` | 0 |
| `steel-core/src/world/explosion/server/block_rays.rs` | 328 | — | 0 |
| `steel-core/src/world/explosion/server/entity_effects.rs` | 131 | — | 0 |
| `steel-core/src/world/explosion/server/exposure.rs` | 223 | — | 0 |
| `steel-core/src/world/explosion/server/tests.rs` | 673 | `player_knockback_map_excludes_creative_flying_and_spectator_players`, `source_and_custom_calculator_hooks_run_on_the_sequential_lane`, `cached_exposure_raycast_matches_clear_partial_and_blocked_paths`, `stable_air_certificate_skips_rays_and_observes_later_block_writes`, `resting_tnt_certificate_includes_vanilla_adjusted_source_block`, `cached_exposure_matches_across_chunk_and_section_boundaries`, `exposure_cache_does_not_retain_air_from_a_missing_chunk`, `exposure_clipping_uses_the_entity_collision_context`, `exposure_cache_is_cleared_before_block_mutating_entity_callbacks`, `moving_piston_exposure_uses_live_block_entities`, `non_destructive_explosion_ignores_blocklike_entities_without_mob_griefing`, `mob_explosion_does_not_push_vehicles_when_mob_griefing_is_disabled`, `submerged_source_explosion_does_not_push_block_attached_entities`, `destructive_explosion_removes_stone_and_spawns_its_loot` | 0 |
| `steel-core/src/world/explosion/server.rs` | 176 | — | 0 |
| `steel-core/src/world/game_event/context.rs` | 39 | — | 0 |
| `steel-core/src/world/game_event/listener.rs` | 1016 | `collect_in_range_filters_by_listener_radius`, `unregister_removes_empty_section_bucket`, `duplicate_registrations_are_removed_one_at_a_time`, `repeated_deferred_unregister_decrements_physical_count_once`, `canceling_pending_addition_restores_physical_count`, `dropping_retained_registry_releases_physical_count`, `collect_in_range_records_exact_distance_for_delivery_sorting`, `visit_in_range_skips_listener_unregistered_before_turn`, `visit_in_range_defers_same_section_registration_until_next_visit`, `visit_in_range_allows_listener_to_unregister_itself`, `deferred_listener_destruction_runs_without_storage_lock`, `world_dispatch_uses_vanilla_chunk_order_not_registration_order`, `equal_distance_delivery_keeps_vanilla_traversal_order` | 0 |
| `steel-core/src/world/game_event/mod.rs` | 8 | — | 0 |
| `steel-core/src/world/level_effects.rs` | 838 | `sound_range_fallback_matches_vanilla_boundary_and_relocation` | 0 |
| `steel-core/src/world/level_reader.rs` | 358 | `pathfinding_cost_uses_vanilla_curved_light_value`, `pathfinding_cost_lerps_toward_full_light_with_ambient_light`, `max_local_raw_brightness_matches_vanilla_horizontal_limit`, `light_level_dependent_magic_value_applies_current_sky_darkening`, `can_see_sky_uses_vanilla_sky_light_threshold` | 0 |
| `steel-core/src/world/map_state.rs` | 1487 | `center_snaps_to_the_scale_grid`, `zoom_out_keeps_center_and_clears_pixels`, `copy_locks_the_map_and_keeps_an_independent_grid`, `put_color_only_reports_real_changes`, `delta_patch_covers_the_dirty_rectangle`, `decoration_sync_is_throttled_to_every_fifth_request`, `off_map_players_use_the_fallback_markers`, `unlimited_tracking_keeps_distant_markers_off_limits`, `on_map_players_use_the_vanilla_rotation_offset`, `nether_markers_rotate_from_hashed_game_time`, `decoration_count_only_tracks_counted_types`, `replace_decorations_renumbers_keys`, `persisted_state_round_trips`, `unknown_decoration_types_are_skipped_on_load`, `ids_are_allocated_monotonically`, `player_markers_are_only_drawn_in_the_maps_own_dimension`, `map_states_report_color_and_decoration_changes_as_dirty` | 0 |
| `steel-core/src/world/mod.rs` | 1932 | — | 0 |
| `steel-core/src/world/player_index/area.rs` | 289 | `test_add_and_get`, `test_remove`, `test_view_change`, `test_multiple_players` | 0 |
| `steel-core/src/world/player_index/map/tests.rs` | 128 | `replacement_updates_both_indexes_without_reordering`, `stale_expected_player_cannot_replace_current_player`, `replacement_rejects_changed_index_keys`, `concurrent_replacements_with_the_same_expected_player_have_one_winner` | 0 |
| `steel-core/src/world/player_index/map.rs` | 257 | — | 0 |
| `steel-core/src/world/player_index/mod.rs` | 5 | — | 0 |
| `steel-core/src/world/player_spawn_finder/candidates.rs` | 67 | — | 0 |
| `steel-core/src/world/player_spawn_finder/chunk_load.rs` | 172 | `scan_resumes_without_skipping_or_repeating_candidates` | 0 |
| `steel-core/src/world/player_spawn_finder/fallback.rs` | 152 | — | 0 |
| `steel-core/src/world/player_spawn_finder/mod.rs` | 433 | — | 0 |
| `steel-core/src/world/player_spawn_finder/tests.rs` | 435 | `candidates_are_consumed_in_vanilla_order_across_chunks`, `prefetched_chunks_are_read_directly_once_their_tickets_are_committed`, `prefetch_follows_the_radius_the_search_uses`, `ground_removed_before_the_search_reaches_a_spot_is_noticed`, `ground_placed_before_the_search_reaches_a_spot_is_noticed`, `a_zero_budget_evaluates_one_candidate_per_poll`, `a_poll_budget_that_ends_mid_chunk_does_not_skip_candidates`, `safe_spawn_in_a_ceiling_dimension_avoids_fluid_and_the_roof`, `safe_spawn_does_not_drop_through_an_ocean_floor_into_a_cave`, `a_surface_that_cannot_be_stood_on_ends_the_search_with_the_height_fixup`, `failed_search_in_a_lava_cavern_falls_back_to_a_ledge_only_when_asked` | 0 |
| `steel-core/src/world/portals.rs` | 592 | — | 0 |
| `steel-core/src/world/properties.rs` | 311 | — | 0 |
| `steel-core/src/world/raid.rs` | 287 | — | 0 |
| `steel-core/src/world/raycast.rs` | 1705 | `voxel_shape_clip_matches_java_full_cube_at_exact_opposite_boundaries`, `aabb_clip_matches_java_strict_side_epsilon_in_every_axis`, `aabb_clip_matches_java_direction_threshold_in_every_axis`, `aabb_clip_preserves_java_axis_order_for_equal_corner_hits`, `scalar_axis_envelope_matches_full_vanilla_dda_visits`, `collision_path_matches_vanilla_axis_tie_breaking`, `collision_path_preserves_vanilla_lerp_rounding_at_integer_boundaries`, `extensible_collision_query_invalidates_every_exposure_cache` | 0 |
| `steel-core/src/world/redstone.rs` | 99 | `burnout_requires_eight_toggles_at_the_same_position`, `toggle_window_expires_only_after_sixty_game_ticks` | 0 |
| `steel-core/src/world/signal_getter.rs` | 364 | `powered_button_directly_powers_its_support_block`, `non_conductor_does_not_relay_direct_signal`, `control_input_special_cases_redstone_block_and_wire` | 0 |
| `steel-core/src/world/sleep.rs` | 134 | — | 0 |
| `steel-core/src/world/sleep_status.rs` | 72 | `sleepers_needed_matches_vanilla_percentage_rule` | 0 |
| `steel-core/src/world/spawn.rs` | 214 | — | 0 |
| `steel-core/src/world/tests.rs` | 808 | `respawn_world_handoff_requires_the_exact_old_player`, `sound_range_uses_event_range_and_strict_vanilla_boundary`, `face_directed_item_drops_respect_block_drops_game_rule`, `generic_shape_update_does_not_schedule_non_source_fluid`, `closest_portal_candidate_filters_then_tiebreaks_by_y`, `nether_portal_frame_offsets_match_vanilla_create_portal_axes`, `nether_portal_creation_scan_origin_matches_vanilla_column_shift`, `entity_breaker_is_available_to_chorus_flower_loot`, `wall_torch_variants_use_their_standing_torch_loot_tables`, `nearest_player_range_uses_vanilla_strict_boundary`, `nearest_player_negative_range_is_unbounded`, `level_event_recipient_range_uses_block_corner_and_strict_boundary`, `level_event_range_is_independent_of_chunk_tracking_view`, `particle_recipient_range_uses_block_center_and_strict_boundary`, `spawnable_bounds_match_vanilla_teleport_command_bounds`, `absolute_world_bounds_use_the_vanilla_limit_instead_of_chunk_storage_bounds`, `block_state_outside_world_bounds_is_void_air`, `set_block_matches_vanilla_update_limit_and_client_publication_gates`, `light_packet_tracking_border_matches_vanilla_pending_chunk_rule`, `navigating_mob_tracker_tracks_only_pathfinder_mobs`, `clip_shape_hits_closest_block_face`, `clip_shape_reports_inside_start_like_vanilla_voxel_shape`, `clip_local_aabb_supports_runtime_fluid_heights`, `can_see_sky_requires_full_sky_light_like_vanilla_26_2`, `precipitation_sky_exposure_requires_light_and_heightmap_gates`, `fluid_clip_height_treats_source_and_flowing_variants_as_same_fluid_above` | 0 |
| `steel-core/src/world/tick_scheduler/list.rs` | 341 | — | 0 |
| `steel-core/src/world/tick_scheduler/mod.rs` | 415 | — | 0 |
| `steel-core/src/world/tick_scheduler/run_batch.rs` | 63 | — | 0 |
| `steel-core/src/world/tick_scheduler/scheduler.rs` | 743 | — | 0 |
| `steel-core/src/world/tick_scheduler/tests.rs` | 840 | `schedule_deduplicates_by_position_and_type`, `proto_pending_scheduling_is_linearized_with_full_promotion`, `chunk_snapshot_does_not_wait_for_world_scheduler_metadata`, `pending_ticks_are_unindexed_until_idempotent_unpack`, `pending_and_live_ticks_share_dedup_before_unpack`, `absolute_time_makes_ineligible_deadlines_overdue`, `global_cap_retains_ready_overflow`, `block_and_fluid_collection_use_the_same_absolute_time`, `selection_respects_each_chunks_deadline_head`, `exact_intra_tick_ties_keep_draining_the_current_chunk`, `exact_loaded_head_ties_follow_the_active_scc_order`, `ineligible_live_head_stays_indexed_until_reentry`, `failed_active_reconciliation_preserves_the_published_index`, `only_popped_containers_report_a_persistence_change`, `persistence_uses_absolute_time_and_rebuilds_loaded_order`, `proto_saved_ticks_deduplicate_in_first_occurrence_order`, `removing_pending_ticks_releases_their_deduplication_keys`, `unpack_preserves_proto_tick_insertion_order`, `execution_snapshot_contains_only_ticks_that_have_not_started`, `can_reschedule_after_ready_tick_is_removed`, `priority_ordering_matches_vanilla_discriminants` | 0 |
| `steel-core/src/world/tick_scheduler/world.rs` | 267 | — | 0 |
| `steel-core/src/world/wandering_trader_spawner.rs` | 337 | `a_failed_roll_advances_the_chance_by_a_quarter`, `a_successful_roll_attempts_and_resets_the_delay`, `the_delay_only_expires_after_the_full_day` | 0 |
| `steel-core/src/world/waypoint/tests.rs` | 130 | `new_manager_tracks_nothing`, `forgetting_an_observer_drops_its_tracking_state`, `distance_tiers_select_vec3i_chunk_and_azimuth_with_floor_chunk_division`, `update_suppression_handles_block_azimuth_and_icon_changes`, `icon_color_and_style_overrides_clean_up_default_entries`, `tracked_state_preserves_ordering` | 0 |
| `steel-core/src/world/waypoint.rs` | 354 | — | 0 |
| `steel-core/src/world/weather.rs` | 512 | — | 0 |
| `steel-core/src/world/world_entities.rs` | 363 | — | 0 |
| `steel-core/src/worldgen/carver/canyon.rs` | 180 | — | 0 |
| `steel-core/src/worldgen/carver/cave.rs` | 262 | — | 0 |
| `steel-core/src/worldgen/carver/mask.rs` | 313 | `set_and_get_roundtrip`, `reused_mask_is_clear_across_height_ranges`, `set_if_unset_reports_first_visit`, `indexing_matches_vanilla_layout`, `x_and_z_are_masked_to_chunk_local`, `worldgen_context_excludes_protected_edges`, `visit_splits_ranges_at_clear_bits_and_columns` | 0 |
| `steel-core/src/worldgen/carver/mod.rs` | 583 | `steep_material_condition_matches_vanilla_asymmetry` | 0 |
| `steel-core/src/worldgen/chunk_stage_hashes.rs` | 1662 | `chunk_stage_hashes` | 0 |
| `steel-core/src/worldgen/feature/configured.rs` | 1330 | `template_feature_position_uses_template_depth_for_north_offset` | 0 |
| `steel-core/src/worldgen/feature/features/bamboo.rs` | 115 | — | 0 |
| `steel-core/src/worldgen/feature/features/basalt_columns.rs` | 177 | — | 0 |
| `steel-core/src/worldgen/feature/features/basalt_pillar.rs` | 104 | — | 0 |
| `steel-core/src/worldgen/feature/features/block_blob.rs` | 61 | — | 0 |
| `steel-core/src/worldgen/feature/features/block_column.rs` | 86 | — | 0 |
| `steel-core/src/worldgen/feature/features/block_entities.rs` | 122 | — | 0 |
| `steel-core/src/worldgen/feature/features/block_pile.rs` | 72 | — | 0 |
| `steel-core/src/worldgen/feature/features/blue_ice.rs` | 87 | — | 0 |
| `steel-core/src/worldgen/feature/features/bonus_chest.rs` | 65 | — | 0 |
| `steel-core/src/worldgen/feature/features/chorus_plant.rs` | 129 | — | 0 |
| `steel-core/src/worldgen/feature/features/coral.rs` | 242 | — | 0 |
| `steel-core/src/worldgen/feature/features/delta.rs` | 86 | — | 0 |
| `steel-core/src/worldgen/feature/features/desert_well.rs` | 163 | — | 0 |
| `steel-core/src/worldgen/feature/features/disk.rs` | 91 | — | 0 |
| `steel-core/src/worldgen/feature/features/dripstone/cluster.rs` | 416 | — | 0 |
| `steel-core/src/worldgen/feature/features/dripstone/large.rs` | 321 | — | 0 |
| `steel-core/src/worldgen/feature/features/dripstone/mod.rs` | 3 | — | 0 |
| `steel-core/src/worldgen/feature/features/dripstone/pointed.rs` | 191 | — | 0 |
| `steel-core/src/worldgen/feature/features/end_gateway.rs` | 104 | — | 0 |
| `steel-core/src/worldgen/feature/features/end_island.rs` | 33 | — | 0 |
| `steel-core/src/worldgen/feature/features/end_platform.rs` | 32 | — | 0 |
| `steel-core/src/worldgen/feature/features/end_podium.rs` | 147 | — | 0 |
| `steel-core/src/worldgen/feature/features/end_spike.rs` | 296 | — | 0 |
| `steel-core/src/worldgen/feature/features/fossil.rs` | 197 | — | 0 |
| `steel-core/src/worldgen/feature/features/freeze_top_layer.rs` | 49 | — | 0 |
| `steel-core/src/worldgen/feature/features/geode.rs` | 360 | — | 0 |
| `steel-core/src/worldgen/feature/features/glowstone_blob.rs` | 57 | — | 0 |
| `steel-core/src/worldgen/feature/features/huge_fungus.rs` | 321 | — | 0 |
| `steel-core/src/worldgen/feature/features/huge_mushroom.rs` | 341 | `huge_red_mushroom_validation_radius_matches_vanilla_sentinel_call`, `huge_brown_mushroom_validation_radius_ignores_tree_height_like_vanilla` | 0 |
| `steel-core/src/worldgen/feature/features/iceberg.rs` | 504 | — | 0 |
| `steel-core/src/worldgen/feature/features/kelp.rs` | 59 | — | 0 |
| `steel-core/src/worldgen/feature/features/lake.rs` | 227 | — | 0 |
| `steel-core/src/worldgen/feature/features/mod.rs` | 58 | — | 0 |
| `steel-core/src/worldgen/feature/features/monster_room.rs` | 191 | — | 0 |
| `steel-core/src/worldgen/feature/features/multiface_growth.rs` | 477 | — | 0 |
| `steel-core/src/worldgen/feature/features/nether_forest_vegetation.rs` | 52 | — | 0 |
| `steel-core/src/worldgen/feature/features/ore.rs` | 758 | `ore_tested_position_index_matches_vanilla_layout`, `ore_tested_position_index_keeps_vanilla_inclusive_edge_layout`, `ore_tested_positions_deduplicate_and_grow` | 0 |
| `steel-core/src/worldgen/feature/features/projected_random_patchy_square.rs` | 52 | — | 0 |
| `steel-core/src/worldgen/feature/features/random_neighbor_spread.rs` | 64 | — | 0 |
| `steel-core/src/worldgen/feature/features/replace_blobs.rs` | 64 | — | 0 |
| `steel-core/src/worldgen/feature/features/sculk_patch.rs` | 1050 | — | 0 |
| `steel-core/src/worldgen/feature/features/sea_pickle.rs` | 36 | — | 0 |
| `steel-core/src/worldgen/feature/features/seagrass.rs` | 50 | — | 0 |
| `steel-core/src/worldgen/feature/features/simple_block.rs` | 177 | — | 0 |
| `steel-core/src/worldgen/feature/features/single_block_pillar.rs` | 47 | — | 0 |
| `steel-core/src/worldgen/feature/features/speleothem.rs` | 565 | — | 0 |
| `steel-core/src/worldgen/feature/features/spike.rs` | 169 | — | 0 |
| `steel-core/src/worldgen/feature/features/spring.rs` | 86 | — | 0 |
| `steel-core/src/worldgen/feature/features/stepped_column_cluster.rs` | 169 | — | 0 |
| `steel-core/src/worldgen/feature/features/tree/decorators.rs` | 960 | — | 0 |
| `steel-core/src/worldgen/feature/features/tree/fallen.rs` | 185 | — | 0 |
| `steel-core/src/worldgen/feature/features/tree/foliage.rs` | 1371 | `acacia_top_layer_keeps_cross_and_inner_corners`, `acacia_lower_layers_skip_only_outer_corners`, `conifer_layers_skip_only_nonzero_outer_corners`, `mega_pine_uses_circular_row_cutoff_with_vanilla_seven_limit`, `fancy_foliage_uses_shifted_circular_cutoff`, `dark_oak_double_trunk_skips_outer_corner_extensions_on_center_layer`, `dark_oak_side_crowns_skip_lower_outer_corners`, `dark_oak_upper_layer_uses_diagonal_cutoff` | 0 |
| `steel-core/src/worldgen/feature/features/tree/leaves.rs` | 253 | — | 0 |
| `steel-core/src/worldgen/feature/features/tree/mod.rs` | 597 | `tree_write_preflight_rejects_unwritable_chunks`, `tree_write_preflight_overlays_accepted_writes` | 0 |
| `steel-core/src/worldgen/feature/features/tree/root_system.rs` | 234 | — | 0 |
| `steel-core/src/worldgen/feature/features/tree/roots.rs` | 245 | — | 0 |
| `steel-core/src/worldgen/feature/features/tree/trunk.rs` | 1319 | `fancy_tree_shape_rejects_lower_third`, `fancy_tree_shape_uses_half_circle_crown`, `fancy_log_axis_prefers_x_on_horizontal_tie` | 0 |
| `steel-core/src/worldgen/feature/features/twisting_vines.rs` | 101 | — | 0 |
| `steel-core/src/worldgen/feature/features/underwater_magma.rs` | 98 | — | 0 |
| `steel-core/src/worldgen/feature/features/vegetation_patch.rs` | 257 | — | 0 |
| `steel-core/src/worldgen/feature/features/vines.rs` | 44 | — | 0 |
| `steel-core/src/worldgen/feature/features/void_start_platform.rs` | 85 | `void_start_platform_checkerboard_distance_matches_vanilla` | 0 |
| `steel-core/src/worldgen/feature/features/weeping_vines.rs` | 128 | — | 0 |
| `steel-core/src/worldgen/feature/instrumentation.rs` | 403 | — | 0 |
| `steel-core/src/worldgen/feature/mod.rs` | 29 | — | 0 |
| `steel-core/src/worldgen/feature/placed.rs` | 577 | — | 0 |
| `steel-core/src/worldgen/feature/placement.rs` | 140 | — | 0 |
| `steel-core/src/worldgen/feature/predicates.rs` | 41 | — | 0 |
| `steel-core/src/worldgen/feature/prelude.rs` | 58 | — | 0 |
| `steel-core/src/worldgen/feature/runner.rs` | 469 | — | 0 |
| `steel-core/src/worldgen/feature/sorter.rs` | 260 | — | 0 |
| `steel-core/src/worldgen/feature/state.rs` | 113 | — | 0 |
| `steel-core/src/worldgen/feature/tests.rs` | 383 | `feature_direction_order_matches_java_direction_values`, `feature_horizontal_direction_order_matches_java_plane_horizontal`, `within_manhattan_iteration_starts_in_vanilla_order`, `vanilla_feature_sorter_builds_for_all_builtin_biome_sources`, `structures_for_decoration_step_use_registry_order_inside_vanilla_step`, `structure_start_resolution_uses_vanilla_reference_order_and_filters_invalid_starts`, `structure_reference_set_iterates_like_vanilla_long_open_hash_set`, `structure_step_seed_uses_vanilla_feature_seed_shape`, `structure_piece_clip_box_is_center_chunk_build_height_box`, `structure_start_reference_pos_uses_first_piece_center_and_min_y`, `block_column_truncation_matches_vanilla_tip_priority`, `spring_source_fluid_state_creates_vanilla_legacy_source_block`, `scattered_ore_offset_rounding_matches_java_math_round`, `blue_ice_horizontal_spread_radius_uses_java_integer_division` | 0 |
| `steel-core/src/worldgen/feature/weather.rs` | 148 | — | 0 |
| `steel-core/src/worldgen/generator/context.rs` | 132 | — | 0 |
| `steel-core/src/worldgen/generator/empty.rs` | 65 | — | 0 |
| `steel-core/src/worldgen/generator/flat.rs` | 375 | — | 0 |
| `steel-core/src/worldgen/generator/generation_chunk.rs` | 360 | `capability_rejects_the_wrong_published_input`, `capability_rejects_an_advanced_published_input` | 0 |
| `steel-core/src/worldgen/generator/mod.rs` | 130 | — | 0 |
| `steel-core/src/worldgen/generator/registry.rs` | 696 | `default_flat_config_matches_vanilla_superflat`, `single_biome_config_defaults_to_plains_and_rejects_unknown_biomes`, `accepts_flat_decoration_options` | 0 |
| `steel-core/src/worldgen/generator/surface_biomes.rs` | 370 | — | 0 |
| `steel-core/src/worldgen/generator/vanilla.rs` | 1395 | `fixed_biome_overworld_keeps_climate_spawn_origin`, `retained_and_rebuilt_aquifers_produce_identical_carver_output` | 0 |
| `steel-core/src/worldgen/mod.rs` | 45 | — | 0 |
| `steel-core/src/worldgen/region.rs` | 1876 | `full_imposter_maps_worldgen_height_queries_to_final_heightmaps`, `chessboard_distance_matches_chunk_dependency_radius`, `biome_quart_y_indices_clamp_to_vertical_biome_range`, `bulk_section_index_matches_world_height_bounds`, `bulk_section_local_coord_uses_vanilla_section_mask`, `bulk_section_reads_use_empty_proto_sections_for_full_imposters`, `bulk_write_defers_counts_only_for_pre_light_proto_chunks`, `cached_worldgen_view_observes_holder_status_advancement` | 0 |
| `steel-core/src/worldgen/stages/biomes.rs` | 21 | — | 0 |
| `steel-core/src/worldgen/stages/empty.rs` | 33 | — | 0 |
| `steel-core/src/worldgen/stages/features.rs` | 30 | — | 0 |
| `steel-core/src/worldgen/stages/full.rs` | 15 | — | 0 |
| `steel-core/src/worldgen/stages/leaf_distance.rs` | 494 | `resolves_center_leaves_from_sources_across_chunk_boundaries`, `stale_leaf_distances_without_a_source_converge_to_seven`, `six_block_halo_preserves_the_distance_seven_boundary`, `scaffolding_distance_is_not_leaf_distance` | 0 |
| `steel-core/src/worldgen/stages/light.rs` | 470 | `light_stage_generates_center_sky_and_block_light`, `light_stage_skips_sky_when_dimension_has_no_skylight`, `generated_light_resolves_leaf_distances_and_preserves_other_ticks`, `loaded_light_stage_preserves_persisted_interior_block_light`, `loaded_light_stage_keeps_pending_leaf_ticks`, `loaded_light_stage_validates_persisted_sky_light` | 0 |
| `steel-core/src/worldgen/stages/mod.rs` | 11 | — | 0 |
| `steel-core/src/worldgen/stages/spawn.rs` | 295 | `a_zero_weight_list_picks_nothing`, `weights_are_respected` | 0 |
| `steel-core/src/worldgen/stages/structures.rs` | 98 | — | 0 |
| `steel-core/src/worldgen/stages/terrain.rs` | 126 | — | 0 |
| `steel-core/src/worldgen/structure/mod.rs` | 37 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/buried_treasure.rs` | 115 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/desert_pyramid.rs` | 734 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/fortress.rs` | 812 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/jungle_temple.rs` | 370 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/mineshaft.rs` | 1138 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/mod.rs` | 461 | `registered_non_mob_is_skipped`, `supported_structure_mob_preserves_placement_and_persistence`, `unsupported_structure_mob_is_skipped` | 0 |
| `steel-core/src/worldgen/structure/piece_placer/ocean_monument/helpers.rs` | 235 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/ocean_monument/mod.rs` | 119 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/ocean_monument/room_core.rs` | 102 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/ocean_monument/room_double.rs` | 385 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/ocean_monument/room_simple.rs` | 219 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/ocean_monument/room_special.rs` | 117 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/ocean_monument/shell.rs` | 620 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/pool_element.rs` | 169 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/ruined_portal.rs` | 318 | `ruined_portal_netherrack_replacement_matches_vanilla_air_checks` | 0 |
| `steel-core/src/worldgen/structure/piece_placer/scattered_feature.rs` | 343 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/stronghold.rs` | 920 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/swamp_hut.rs` | 125 | — | 0 |
| `steel-core/src/worldgen/structure/piece_placer/template_piece.rs` | 872 | `center_gated_expanded_clip_requires_template_center_inside_center_chunk`, `marker_water_check_uses_fluid_state_for_seagrass` | 0 |
| `steel-core/src/worldgen/structure/piece_placer/template_processors.rs` | 179 | — | 0 |
| `steel-core/src/worldgen/surface/mod.rs` | 677 | — | 0 |
| `steel-core/src/worldgen/template/loading.rs` | 418 | — | 0 |
| `steel-core/src/worldgen/template/mod.rs` | 131 | — | 0 |
| `steel-core/src/worldgen/template/placement.rs` | 736 | — | 0 |
| `steel-core/src/worldgen/template/processors.rs` | 820 | — | 0 |
| `steel-core/src/worldgen/template/state_transforms.rs` | 405 | — | 0 |
| `steel-core/src/worldgen/template/tests.rs` | 595 | `palette_blocks_skips_all_out_of_bounds_for_current_chunk_processors`, `palette_blocks_keeps_out_of_bounds_for_capped_processors`, `zero_position_with_transform_matches_vanilla_rotation_offsets`, `bounding_box_with_transform_matches_vanilla_mirror_rotation_pivot`, `block_pos_seed_matches_vanilla_mth_get_seed`, `village_template_loads_entity_payloads`, `template_block_entity_resolution_uses_final_block_state`, `entity_position_and_rotation_transform_match_vanilla_offsets`, `hanging_entity_facing_applies_rotation_before_mirror`, `mirrored_door_transform_toggles_hinge`, `chiseled_bookshelf_rotation_and_mirror_preserve_occupied_slots`, `jigsaw_replacement_uses_final_state_and_removes_nbt`, `jigsaw_replacement_accepts_trailing_text_like_vanilla_parser`, `jigsaw_replacement_drops_structure_void_final_state`, `structure_block_ignore_modes_match_vanilla_single_variants`, `block_age_processor_preserves_slab_properties`, `lava_submerged_processor_keeps_non_full_blocks_as_lava`, `blackstone_replace_processor_preserves_stair_orientation`, `data_markers_read_shipwreck_structure_blocks`, `data_markers_read_igloo_chest_structure_block` | 0 |
| `steel-crypto/src/key_store.rs` | 75 | `decrypt_reports_no_reason_for_failure`, `public_key_der_round_trips` | 0 |
| `steel-crypto/src/lib.rs` | 40 | — | 0 |
| `steel-crypto/src/rsa_utils.rs` | 101 | `test_generate_key_pair`, `test_public_key_round_trip`, `test_pem_round_trip` | 0 |
| `steel-crypto/src/signature.rs` | 288 | `test_sign_and_verify`, `test_invalid_signature`, `profile_key_validator_uses_sha1_service_signatures`, `profile_key_validator_requires_service_keys`, `test_no_validation` | 0 |
| `steel-math/src/angle.rs` | 67 | `wrap_degrees_matches_vanilla_range`, `rotation_segment_wraps_and_rounds_to_nearest_segment` | 0 |
| `steel-math/src/float_consts.rs` | 12 | — | 0 |
| `steel-math/src/lib.rs` | 19 | — | 0 |
| `steel-math/src/noise_math/coordinate.rs` | 177 | `test_floor`, `test_wrap`, `test_wrap_4x_matches_vanilla` | 0 |
| `steel-math/src/noise_math/gradient.rs` | 121 | — | 0 |
| `steel-math/src/noise_math/interpolation.rs` | 316 | `test_lerp`, `test_smoothstep` | 0 |
| `steel-math/src/noise_math/mod.rs` | 36 | — | 0 |
| `steel-math/src/rounding.rs` | 42 | `round_f32_matches_java_math_round_on_halves` | 0 |
| `steel-math/src/trig.rs` | 167 | `sin_at_zero_is_zero`, `sin_matches_table_entries`, `cos_is_sin_shifted` | 0 |
| `steel-math/src/vector.rs` | 98 | `rotating_by_zero_returns_the_input`, `quarter_turns_match_vanilla_axis_mapping`, `x_rot_opposes_glam_rotate_x_while_y_rot_agrees` | 0 |
| `steel-flint/src/adapter.rs` | 415 | `shards_cover_each_fixture_once_without_overlap`, `test_run_flint_selected`, `test_run_all_flint_benchmarks` | 0 |
| `steel-flint/src/convert.rs` | 103 | `block_round_trip_with_and_without_prefix` | 0 |
| `steel-flint/src/lib.rs` | 48 | — | 0 |
| `steel-flint/src/player.rs` | 266 | `stack_conversion_reports_requested_plain_components` | 0 |
| `steel-flint/src/test_connection.rs` | 41 | — | 0 |
| `steel-flint/src/world.rs` | 595 | `daylight_detector_registered_ticker_corrects_stale_power_at_noon`, `setup_applies_numeric_and_registry_marker_times_and_weather_or_reports_errors`, `runner_applies_world_setup_before_the_first_assertion_and_surfaces_invalid_setup`, `daytime_wraps_at_midnight_and_normalizes_negative_clock_values`, `test_world_is_released_after_adapter_drop`, `test_player_is_removed_from_world_on_adapter_drop`, `test_player_is_registered_in_world_for_real_item_interactions` | 0 |
