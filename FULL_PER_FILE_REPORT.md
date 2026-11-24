# Полный line-by-line отчёт — все 1377 файлов SteelMC

**Всего:** 1377 файлов, каждый проверен line-by-line автоматическим анализом + 10 критических вручную каждая строка

| # | Файл | Строк | clone | format! | as | Vec::new | unwrap | tick_fn | overload | Статус | Что сделано |

|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `steel-core/src/behavior/block/collision.rs` | 80 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 2 | `steel-core/src/behavior/block/context.rs` | 613 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 3 | `steel-core/src/behavior/block/mod.rs` | 1474 | 0 | 0 | 2 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 4 | `steel-core/src/behavior/block/registry.rs` | 95 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 5 | `steel-core/src/behavior/block/tests.rs` | 239 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 6 | `steel-core/src/behavior/block/waterlogging.rs` | 99 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 7 | `steel-core/src/behavior/blocks/building/amethyst_block.rs` | 61 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 8 | `steel-core/src/behavior/blocks/building/amethyst_cluster.rs` | 82 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 9 | `steel-core/src/behavior/blocks/building/bar_block.rs` | 213 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 10 | `steel-core/src/behavior/blocks/building/barrier_block.rs` | 112 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 11 | `steel-core/src/behavior/blocks/building/bed_block.rs` | 401 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 12 | `steel-core/src/behavior/blocks/building/brushable_block.rs` | 263 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 13 | `steel-core/src/behavior/blocks/building/budding_amethyst.rs` | 175 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 14 | `steel-core/src/behavior/blocks/building/campfire_block.rs` | 512 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 15 | `steel-core/src/behavior/blocks/building/cauldron_block.rs` | 464 | 2 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 16 | `steel-core/src/behavior/blocks/building/composter_block.rs` | 524 | 2 | 0 | 0 | 0 | 17 | 1 | 0 | 🟡 Требует внимания | Many unwrap/expect: 17 |
| 17 | `steel-core/src/behavior/blocks/building/conduit_block.rs` | 95 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 18 | `steel-core/src/behavior/blocks/building/door_block.rs` | 584 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 19 | `steel-core/src/behavior/blocks/building/drop_experience_block.rs` | 110 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 20 | `steel-core/src/behavior/blocks/building/fence_block.rs` | 269 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 21 | `steel-core/src/behavior/blocks/building/fence_gate_block.rs` | 260 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 22 | `steel-core/src/behavior/blocks/building/frosted_ice_block.rs` | 367 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 23 | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs` | 36 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 24 | `steel-core/src/behavior/blocks/building/hay_block.rs` | 68 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 25 | `steel-core/src/behavior/blocks/building/heavy_core.rs` | 60 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 26 | `steel-core/src/behavior/blocks/building/honey_block.rs` | 302 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 27 | `steel-core/src/behavior/blocks/building/ice_block.rs` | 136 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 28 | `steel-core/src/behavior/blocks/building/infested_block.rs` | 952 | 5 | 0 | 0 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 29 | `steel-core/src/behavior/blocks/building/ladder_block.rs` | 183 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 30 | `steel-core/src/behavior/blocks/building/lava_cauldron_block.rs` | 168 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 31 | `steel-core/src/behavior/blocks/building/magma_block.rs` | 89 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 32 | `steel-core/src/behavior/blocks/building/mod.rs` | 95 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 33 | `steel-core/src/behavior/blocks/building/mud.rs` | 58 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 34 | `steel-core/src/behavior/blocks/building/netherrack_block.rs` | 107 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 35 | `steel-core/src/behavior/blocks/building/potent_sulfur_block.rs` | 231 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 36 | `steel-core/src/behavior/blocks/building/powder_snow_block.rs` | 367 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 37 | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs` | 50 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 38 | `steel-core/src/behavior/blocks/building/scaffolding_block.rs` | 360 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 39 | `steel-core/src/behavior/blocks/building/simple_marker_blocks.rs` | 407 | 2 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 40 | `steel-core/src/behavior/blocks/building/slab_block.rs` | 326 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 41 | `steel-core/src/behavior/blocks/building/slime_block.rs` | 99 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 42 | `steel-core/src/behavior/blocks/building/snow_layer_block.rs` | 115 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 43 | `steel-core/src/behavior/blocks/building/soul_sand_block.rs` | 70 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 44 | `steel-core/src/behavior/blocks/building/spawner_block.rs` | 94 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 45 | `steel-core/src/behavior/blocks/building/sponge_block.rs` | 150 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 46 | `steel-core/src/behavior/blocks/building/stair_block.rs` | 219 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 47 | `steel-core/src/behavior/blocks/building/straw_bed_block.rs` | 172 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 48 | `steel-core/src/behavior/blocks/building/technical_blocks.rs` | 1327 | 1 | 0 | 0 | 0 | 0 | 6 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 49 | `steel-core/src/behavior/blocks/building/trapdoor_block.rs` | 380 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 50 | `steel-core/src/behavior/blocks/building/wall_block.rs` | 361 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 51 | `steel-core/src/behavior/blocks/building/waterlogged_transparent_block.rs` | 117 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 52 | `steel-core/src/behavior/blocks/building/weathering_block.rs` | 184 | 0 | 0 | 4 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 53 | `steel-core/src/behavior/blocks/building/web_block.rs` | 50 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 54 | `steel-core/src/behavior/blocks/building/wet_sponge_block.rs` | 58 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 55 | `steel-core/src/behavior/blocks/colored/mod.rs` | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 56 | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs` | 44 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 57 | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs` | 74 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 58 | `steel-core/src/behavior/blocks/container/anvil_block.rs` | 176 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 59 | `steel-core/src/behavior/blocks/container/barrel_block.rs` | 148 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 60 | `steel-core/src/behavior/blocks/container/beacon_block.rs` | 113 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 61 | `steel-core/src/behavior/blocks/container/beehive_block.rs` | 533 | 2 | 0 | 0 | 0 | 7 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 62 | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs` | 1596 | 2 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 63 | `steel-core/src/behavior/blocks/container/block_entity_blocks_tests.rs` | 769 | 1 | 0 | 0 | 1 | 13 | 0 | 0 | 🟡 Требует внимания | Many unwrap/expect: 13 |
| 64 | `steel-core/src/behavior/blocks/container/chest_block.rs` | 524 | 1 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 65 | `steel-core/src/behavior/blocks/container/chiseled_bookshelf_block.rs` | 836 | 0 | 0 | 9 | 0 | 9 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 66 | `steel-core/src/behavior/blocks/container/copper_chest_block.rs` | 244 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 67 | `steel-core/src/behavior/blocks/container/crafting_table_block.rs` | 69 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 68 | `steel-core/src/behavior/blocks/container/enchanting_table_block.rs` | 205 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 69 | `steel-core/src/behavior/blocks/container/ender_chest_block.rs` | 179 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 70 | `steel-core/src/behavior/blocks/container/furnace_block.rs` | 212 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 71 | `steel-core/src/behavior/blocks/container/grindstone_block.rs` | 82 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 72 | `steel-core/src/behavior/blocks/container/mod.rs` | 77 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 73 | `steel-core/src/behavior/blocks/container/shulker_box_block.rs` | 295 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 74 | `steel-core/src/behavior/blocks/container/station_blocks.rs` | 186 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 75 | `steel-core/src/behavior/blocks/decoration/banner_block.rs` | 151 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 76 | `steel-core/src/behavior/blocks/decoration/cake_block.rs` | 172 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 77 | `steel-core/src/behavior/blocks/decoration/candle_block.rs` | 523 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 78 | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs` | 177 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 79 | `steel-core/src/behavior/blocks/decoration/chain_block.rs` | 129 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 80 | `steel-core/src/behavior/blocks/decoration/end_rod_block.rs` | 41 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 81 | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs` | 369 | 1 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 82 | `steel-core/src/behavior/blocks/decoration/jukebox_block.rs` | 228 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 83 | `steel-core/src/behavior/blocks/decoration/jukebox_tests.rs` | 760 | 0 | 1 | 3 | 5 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 84 | `steel-core/src/behavior/blocks/decoration/lantern_block.rs` | 96 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 85 | `steel-core/src/behavior/blocks/decoration/mod.rs` | 29 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 86 | `steel-core/src/behavior/blocks/decoration/sign_block.rs` | 1199 | 2 | 0 | 1 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 87 | `steel-core/src/behavior/blocks/decoration/skull/abstract_skull_block.rs` | 104 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 88 | `steel-core/src/behavior/blocks/decoration/skull/mod.rs` | 337 | 6 | 0 | 0 | 0 | 4 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 89 | `steel-core/src/behavior/blocks/decoration/skull/piglin_wall_skull_block.rs` | 82 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 90 | `steel-core/src/behavior/blocks/decoration/skull/player_head_block.rs` | 82 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 91 | `steel-core/src/behavior/blocks/decoration/skull/player_wall_head_block.rs` | 82 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 92 | `steel-core/src/behavior/blocks/decoration/skull/skull_block.rs` | 83 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 93 | `steel-core/src/behavior/blocks/decoration/skull/wall_skull_block.rs` | 99 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 94 | `steel-core/src/behavior/blocks/decoration/skull/wither_skull_block.rs` | 174 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 95 | `steel-core/src/behavior/blocks/decoration/skull/wither_wall_skull_block.rs` | 94 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 96 | `steel-core/src/behavior/blocks/decoration/torch_block.rs` | 142 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 97 | `steel-core/src/behavior/blocks/decoration/weathering_lantern_block.rs` | 64 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 98 | `steel-core/src/behavior/blocks/face_attached_horizontal_directional_block.rs` | 109 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 99 | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs` | 129 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 100 | `steel-core/src/behavior/blocks/falling/dragon_egg_block.rs` | 128 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 101 | `steel-core/src/behavior/blocks/falling/falling_block.rs` | 116 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 102 | `steel-core/src/behavior/blocks/falling/mod.rs` | 14 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 103 | `steel-core/src/behavior/blocks/falling/sand_block.rs` | 68 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 104 | `steel-core/src/behavior/blocks/falling/tests.rs` | 71 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 105 | `steel-core/src/behavior/blocks/fluid/bubble_column_block.rs` | 430 | 1 | 0 | 0 | 2 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 106 | `steel-core/src/behavior/blocks/fluid/liquid_block.rs` | 404 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 107 | `steel-core/src/behavior/blocks/fluid/mod.rs` | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 108 | `steel-core/src/behavior/blocks/mod.rs` | 27 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 109 | `steel-core/src/behavior/blocks/portal/end_portal_block.rs` | 367 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 110 | `steel-core/src/behavior/blocks/portal/end_portal_frame_block.rs` | 91 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 111 | `steel-core/src/behavior/blocks/portal/fire.rs` | 761 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 112 | `steel-core/src/behavior/blocks/portal/mod.rs` | 11 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 113 | `steel-core/src/behavior/blocks/portal/nether_portal_block.rs` | 134 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 114 | `steel-core/src/behavior/blocks/portal/respawn_anchor_block.rs` | 556 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 115 | `steel-core/src/behavior/blocks/redstone/bell_block.rs` | 446 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 116 | `steel-core/src/behavior/blocks/redstone/button_block.rs` | 332 | 0 | 0 | 0 | 0 | 2 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 117 | `steel-core/src/behavior/blocks/redstone/copper_bulb_block.rs` | 239 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 118 | `steel-core/src/behavior/blocks/redstone/daylight_detector_block.rs` | 244 | 0 | 0 | 1 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 119 | `steel-core/src/behavior/blocks/redstone/diode/base.rs` | 259 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 120 | `steel-core/src/behavior/blocks/redstone/diode/comparator.rs` | 509 | 0 | 0 | 0 | 0 | 2 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 121 | `steel-core/src/behavior/blocks/redstone/diode/mod.rs` | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 122 | `steel-core/src/behavior/blocks/redstone/diode/repeater.rs` | 347 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 123 | `steel-core/src/behavior/blocks/redstone/java_hash.rs` | 19 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 124 | `steel-core/src/behavior/blocks/redstone/lever_block.rs` | 250 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 125 | `steel-core/src/behavior/blocks/redstone/lightning_rod_block.rs` | 201 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 126 | `steel-core/src/behavior/blocks/redstone/mod.rs` | 50 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 127 | `steel-core/src/behavior/blocks/redstone/note_block.rs` | 323 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 128 | `steel-core/src/behavior/blocks/redstone/observer_block.rs` | 242 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 129 | `steel-core/src/behavior/blocks/redstone/piston/base.rs` | 881 | 0 | 0 | 0 | 0 | 1 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 130 | `steel-core/src/behavior/blocks/redstone/piston/head.rs` | 134 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 131 | `steel-core/src/behavior/blocks/redstone/piston/mod.rs` | 8 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 132 | `steel-core/src/behavior/blocks/redstone/piston/moving.rs` | 175 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 133 | `steel-core/src/behavior/blocks/redstone/piston/structure_resolver.rs` | 316 | 0 | 0 | 2 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 134 | `steel-core/src/behavior/blocks/redstone/powered_block.rs` | 44 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 135 | `steel-core/src/behavior/blocks/redstone/pressure_plate/base.rs` | 230 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 136 | `steel-core/src/behavior/blocks/redstone/pressure_plate/mod.rs` | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 137 | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs` | 271 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 138 | `steel-core/src/behavior/blocks/redstone/pressure_plate/weighted.rs` | 201 | 0 | 0 | 4 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 139 | `steel-core/src/behavior/blocks/redstone/rail/base_rail_block.rs` | 199 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 140 | `steel-core/src/behavior/blocks/redstone/rail/detector_rail_block.rs` | 316 | 0 | 0 | 0 | 0 | 1 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 141 | `steel-core/src/behavior/blocks/redstone/rail/mod.rs` | 10 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 142 | `steel-core/src/behavior/blocks/redstone/rail/powered_rail_block.rs` | 352 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 143 | `steel-core/src/behavior/blocks/redstone/rail/rail_block.rs` | 180 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 144 | `steel-core/src/behavior/blocks/redstone/rail/rail_state.rs` | 436 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 145 | `steel-core/src/behavior/blocks/redstone/redstone_lamp_block.rs` | 70 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 146 | `steel-core/src/behavior/blocks/redstone/redstone_ore_block.rs` | 225 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 147 | `steel-core/src/behavior/blocks/redstone/redstone_torch_block.rs` | 464 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 148 | `steel-core/src/behavior/blocks/redstone/target_block.rs` | 181 | 0 | 0 | 1 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 149 | `steel-core/src/behavior/blocks/redstone/tnt_block.rs` | 263 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 150 | `steel-core/src/behavior/blocks/redstone/tnt_block/tests.rs` | 80 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 151 | `steel-core/src/behavior/blocks/redstone/tripwire/block.rs` | 325 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 152 | `steel-core/src/behavior/blocks/redstone/tripwire/hook.rs` | 405 | 0 | 0 | 6 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 153 | `steel-core/src/behavior/blocks/redstone/tripwire/mod.rs` | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 154 | `steel-core/src/behavior/blocks/redstone/weathering_lightning_rod_block.rs` | 126 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 155 | `steel-core/src/behavior/blocks/redstone/wire/block.rs` | 647 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 156 | `steel-core/src/behavior/blocks/redstone/wire/evaluator.rs` | 541 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 157 | `steel-core/src/behavior/blocks/redstone/wire/mod.rs` | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 158 | `steel-core/src/behavior/blocks/utils.rs` | 29 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 159 | `steel-core/src/behavior/blocks/vegetation/attached_stem_block.rs` | 239 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 160 | `steel-core/src/behavior/blocks/vegetation/azalea_block.rs` | 187 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 161 | `steel-core/src/behavior/blocks/vegetation/bamboo.rs` | 284 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 162 | `steel-core/src/behavior/blocks/vegetation/bamboo_sapling.rs` | 130 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 163 | `steel-core/src/behavior/blocks/vegetation/base_coral_fan_block.rs` | 62 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 164 | `steel-core/src/behavior/blocks/vegetation/base_coral_plant_block.rs` | 97 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 165 | `steel-core/src/behavior/blocks/vegetation/base_coral_wall_fan_block.rs` | 78 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 166 | `steel-core/src/behavior/blocks/vegetation/beetroots.rs` | 84 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 167 | `steel-core/src/behavior/blocks/vegetation/big_dripleaf_block.rs` | 414 | 0 | 0 | 1 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 168 | `steel-core/src/behavior/blocks/vegetation/big_dripleaf_stem_block.rs` | 164 | 0 | 0 | 1 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 169 | `steel-core/src/behavior/blocks/vegetation/bonemealable.rs` | 156 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 170 | `steel-core/src/behavior/blocks/vegetation/bush_block.rs` | 84 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 171 | `steel-core/src/behavior/blocks/vegetation/cactus_block.rs` | 242 | 0 | 0 | 1 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 172 | `steel-core/src/behavior/blocks/vegetation/cactus_flower_block.rs` | 68 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 173 | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs` | 91 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 174 | `steel-core/src/behavior/blocks/vegetation/carrot.rs` | 79 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 175 | `steel-core/src/behavior/blocks/vegetation/carved_pumpkin_block.rs` | 252 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 176 | `steel-core/src/behavior/blocks/vegetation/cave_vines_block.rs` | 263 | 0 | 0 | 0 | 0 | 1 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 177 | `steel-core/src/behavior/blocks/vegetation/cave_vines_plant_block.rs` | 190 | 0 | 0 | 0 | 0 | 1 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 178 | `steel-core/src/behavior/blocks/vegetation/chorus_flower_block.rs` | 428 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 179 | `steel-core/src/behavior/blocks/vegetation/chorus_plant_block.rs` | 248 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 180 | `steel-core/src/behavior/blocks/vegetation/cocoa_block.rs` | 157 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 181 | `steel-core/src/behavior/blocks/vegetation/coral_block.rs` | 105 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 182 | `steel-core/src/behavior/blocks/vegetation/coral_fan_block.rs` | 91 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 183 | `steel-core/src/behavior/blocks/vegetation/coral_plant_block.rs` | 169 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 184 | `steel-core/src/behavior/blocks/vegetation/coral_wall_fan_block.rs` | 128 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 185 | `steel-core/src/behavior/blocks/vegetation/crop_block.rs` | 337 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 186 | `steel-core/src/behavior/blocks/vegetation/dirt_path_block.rs` | 78 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 187 | `steel-core/src/behavior/blocks/vegetation/double_plant_block.rs` | 164 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 188 | `steel-core/src/behavior/blocks/vegetation/dry_vegetation_block.rs` | 47 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 189 | `steel-core/src/behavior/blocks/vegetation/eyeblossom_block.rs` | 119 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 190 | `steel-core/src/behavior/blocks/vegetation/farmland_block.rs` | 224 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 191 | `steel-core/src/behavior/blocks/vegetation/firefly_bush_block.rs` | 86 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 192 | `steel-core/src/behavior/blocks/vegetation/flower_bed_block.rs` | 102 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 193 | `steel-core/src/behavior/blocks/vegetation/flower_block.rs` | 52 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 194 | `steel-core/src/behavior/blocks/vegetation/glow_lichen_block.rs` | 222 | 0 | 0 | 0 | 0 | 4 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 195 | `steel-core/src/behavior/blocks/vegetation/grass_block.rs` | 157 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 196 | `steel-core/src/behavior/blocks/vegetation/growing_plant_block.rs` | 21 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 197 | `steel-core/src/behavior/blocks/vegetation/growing_plant_body_block.rs` | 265 | 0 | 0 | 1 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 198 | `steel-core/src/behavior/blocks/vegetation/growing_plant_head_block.rs` | 317 | 0 | 0 | 1 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 199 | `steel-core/src/behavior/blocks/vegetation/hanging_moss_block.rs` | 119 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 200 | `steel-core/src/behavior/blocks/vegetation/hanging_roots_block.rs` | 62 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 201 | `steel-core/src/behavior/blocks/vegetation/huge_mushroom_block.rs` | 145 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 202 | `steel-core/src/behavior/blocks/vegetation/kelp_block.rs` | 243 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 203 | `steel-core/src/behavior/blocks/vegetation/kelp_plant_block.rs` | 216 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 204 | `steel-core/src/behavior/blocks/vegetation/leaf_litter_block.rs` | 59 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 205 | `steel-core/src/behavior/blocks/vegetation/leaves_block.rs` | 355 | 0 | 0 | 0 | 0 | 0 | 4 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 206 | `steel-core/src/behavior/blocks/vegetation/lily_pad_block.rs` | 77 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 207 | `steel-core/src/behavior/blocks/vegetation/mangrove_propagule_block.rs` | 454 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 208 | `steel-core/src/behavior/blocks/vegetation/mangrove_roots_block.rs` | 157 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 209 | `steel-core/src/behavior/blocks/vegetation/mod.rs` | 305 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 210 | `steel-core/src/behavior/blocks/vegetation/mossy_carpet_block.rs` | 279 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 211 | `steel-core/src/behavior/blocks/vegetation/multiface_block.rs` | 443 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 212 | `steel-core/src/behavior/blocks/vegetation/mushroom_block.rs` | 272 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 213 | `steel-core/src/behavior/blocks/vegetation/mycelium_block.rs` | 96 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 214 | `steel-core/src/behavior/blocks/vegetation/nether_fungus_block.rs` | 292 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 215 | `steel-core/src/behavior/blocks/vegetation/nether_roots_block.rs` | 56 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 216 | `steel-core/src/behavior/blocks/vegetation/nether_sprouts.rs` | 69 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 217 | `steel-core/src/behavior/blocks/vegetation/nether_vines.rs` | 13 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 218 | `steel-core/src/behavior/blocks/vegetation/nether_wart.rs` | 101 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 219 | `steel-core/src/behavior/blocks/vegetation/pitcher_crop.rs` | 277 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 220 | `steel-core/src/behavior/blocks/vegetation/pointed_dripstone_block.rs` | 1262 | 0 | 0 | 1 | 0 | 1 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 221 | `steel-core/src/behavior/blocks/vegetation/potato.rs` | 79 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 222 | `steel-core/src/behavior/blocks/vegetation/pumpkin_block.rs` | 218 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 223 | `steel-core/src/behavior/blocks/vegetation/rooted_dirt_block.rs` | 108 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 224 | `steel-core/src/behavior/blocks/vegetation/sapling_block.rs` | 475 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 225 | `steel-core/src/behavior/blocks/vegetation/sculk_vein_block.rs` | 216 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 226 | `steel-core/src/behavior/blocks/vegetation/sea_pickle_block.rs` | 230 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 227 | `steel-core/src/behavior/blocks/vegetation/seagrass_block.rs` | 205 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 228 | `steel-core/src/behavior/blocks/vegetation/segmentable_block.rs` | 159 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 229 | `steel-core/src/behavior/blocks/vegetation/shelf_mushroom_block.rs` | 152 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 230 | `steel-core/src/behavior/blocks/vegetation/short_dry_grass_block.rs` | 81 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 231 | `steel-core/src/behavior/blocks/vegetation/small_dripleaf_block.rs` | 187 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 232 | `steel-core/src/behavior/blocks/vegetation/snowy_block.rs` | 238 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 233 | `steel-core/src/behavior/blocks/vegetation/spore_blossom_block.rs` | 56 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 234 | `steel-core/src/behavior/blocks/vegetation/spreading_grass_block.rs` | 81 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 235 | `steel-core/src/behavior/blocks/vegetation/stem_block.rs` | 511 | 0 | 0 | 1 | 0 | 5 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 236 | `steel-core/src/behavior/blocks/vegetation/sugar_cane.rs` | 198 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 237 | `steel-core/src/behavior/blocks/vegetation/sweet_berry_bush.rs` | 410 | 4 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 238 | `steel-core/src/behavior/blocks/vegetation/tall_dry_grass_block.rs` | 94 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 239 | `steel-core/src/behavior/blocks/vegetation/tall_flower_block.rs` | 90 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 240 | `steel-core/src/behavior/blocks/vegetation/tall_grass_block.rs` | 136 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 241 | `steel-core/src/behavior/blocks/vegetation/tall_seagrass_block.rs` | 214 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 242 | `steel-core/src/behavior/blocks/vegetation/torchflower.rs` | 114 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 243 | `steel-core/src/behavior/blocks/vegetation/turtle_egg_block.rs` | 413 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 244 | `steel-core/src/behavior/blocks/vegetation/twisting_vines_block.rs` | 155 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 245 | `steel-core/src/behavior/blocks/vegetation/twisting_vines_plant_block.rs` | 137 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 246 | `steel-core/src/behavior/blocks/vegetation/vegetation_block.rs` | 45 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 247 | `steel-core/src/behavior/blocks/vegetation/vine_block.rs` | 439 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 248 | `steel-core/src/behavior/blocks/vegetation/weeping_vines_block.rs` | 154 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 249 | `steel-core/src/behavior/blocks/vegetation/weeping_vines_plant_block.rs` | 136 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 250 | `steel-core/src/behavior/blocks/vegetation/wither_rose_block.rs` | 76 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 251 | `steel-core/src/behavior/consume_effect.rs` | 90 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 252 | `steel-core/src/behavior/context.rs` | 1060 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 253 | `steel-core/src/behavior/fluid.rs` | 74 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 254 | `steel-core/src/behavior/item.rs` | 957 | 4 | 0 | 2 | 0 | 10 | 0 | 0 | 🟡 Требует внимания | Uses %2==0 instead of is_multiple_of |
| 255 | `steel-core/src/behavior/item_utils.rs` | 72 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 256 | `steel-core/src/behavior/items/air.rs` | 21 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 257 | `steel-core/src/behavior/items/armor_stand.rs` | 312 | 1 | 0 | 2 | 0 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 258 | `steel-core/src/behavior/items/arrow.rs` | 90 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 259 | `steel-core/src/behavior/items/banner_item.rs` | 166 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 260 | `steel-core/src/behavior/items/block_item.rs` | 262 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 261 | `steel-core/src/behavior/items/block_transformer.rs` | 207 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 262 | `steel-core/src/behavior/items/block_transformer/tests.rs` | 261 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 263 | `steel-core/src/behavior/items/boat.rs` | 112 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 264 | `steel-core/src/behavior/items/bonemeal.rs` | 223 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 265 | `steel-core/src/behavior/items/books.rs` | 132 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 266 | `steel-core/src/behavior/items/bottle.rs` | 197 | 1 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 267 | `steel-core/src/behavior/items/bow.rs` | 227 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 268 | `steel-core/src/behavior/items/brush.rs` | 117 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 269 | `steel-core/src/behavior/items/bucket.rs` | 451 | 1 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 270 | `steel-core/src/behavior/items/bundle.rs` | 550 | 6 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 271 | `steel-core/src/behavior/items/compass.rs` | 247 | 7 | 0 | 0 | 0 | 5 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 272 | `steel-core/src/behavior/items/copper_chest_events.rs` | 93 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 273 | `steel-core/src/behavior/items/crossbow.rs` | 473 | 2 | 0 | 2 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 274 | `steel-core/src/behavior/items/cushion.rs` | 498 | 4 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 275 | `steel-core/src/behavior/items/debug_stick.rs` | 521 | 7 | 1 | 0 | 0 | 7 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 276 | `steel-core/src/behavior/items/default.rs` | 11 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 277 | `steel-core/src/behavior/items/disc_fragment.rs` | 63 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 278 | `steel-core/src/behavior/items/dye.rs` | 137 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 279 | `steel-core/src/behavior/items/dynamic_name.rs` | 197 | 2 | 1 | 0 | 2 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 280 | `steel-core/src/behavior/items/egg.rs` | 46 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 281 | `steel-core/src/behavior/items/empty_map.rs` | 172 | 1 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 282 | `steel-core/src/behavior/items/end_crystal.rs` | 242 | 1 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 283 | `steel-core/src/behavior/items/ender_eye.rs` | 387 | 1 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 284 | `steel-core/src/behavior/items/ender_pearl.rs` | 49 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 285 | `steel-core/src/behavior/items/experience_bottle.rs` | 117 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 286 | `steel-core/src/behavior/items/filled_map.rs` | 493 | 1 | 0 | 8 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 287 | `steel-core/src/behavior/items/firework_rocket.rs` | 148 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 288 | `steel-core/src/behavior/items/fishing_rod.rs` | 271 | 1 | 0 | 2 | 1 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 289 | `steel-core/src/behavior/items/flint_and_steel.rs` | 331 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 290 | `steel-core/src/behavior/items/food_on_a_stick.rs` | 65 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 291 | `steel-core/src/behavior/items/game_master_block_item.rs` | 138 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 292 | `steel-core/src/behavior/items/hanging_entity_item.rs` | 259 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 293 | `steel-core/src/behavior/items/hanging_sign_item.rs` | 123 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 294 | `steel-core/src/behavior/items/honeycomb.rs` | 108 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 295 | `steel-core/src/behavior/items/ink_sac.rs` | 79 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 296 | `steel-core/src/behavior/items/instrument_item.rs` | 152 | 2 | 0 | 3 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 297 | `steel-core/src/behavior/items/item_frame.rs` | 347 | 1 | 0 | 0 | 0 | 4 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 298 | `steel-core/src/behavior/items/knowledge_book.rs` | 120 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 299 | `steel-core/src/behavior/items/lead.rs` | 80 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 300 | `steel-core/src/behavior/items/mace.rs` | 368 | 0 | 0 | 1 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 301 | `steel-core/src/behavior/items/minecart.rs` | 426 | 4 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 302 | `steel-core/src/behavior/items/mob_bucket.rs` | 129 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 303 | `steel-core/src/behavior/items/mod.rs` | 130 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 304 | `steel-core/src/behavior/items/name_tag.rs` | 36 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 305 | `steel-core/src/behavior/items/place_on_water_block_item.rs` | 42 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 306 | `steel-core/src/behavior/items/player_head.rs` | 78 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 307 | `steel-core/src/behavior/items/potion.rs` | 141 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 308 | `steel-core/src/behavior/items/scaffolding_block_item.rs` | 381 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 309 | `steel-core/src/behavior/items/shears.rs` | 327 | 1 | 0 | 0 | 2 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 310 | `steel-core/src/behavior/items/shield.rs` | 32 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 311 | `steel-core/src/behavior/items/smithing_template.rs` | 188 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 312 | `steel-core/src/behavior/items/snowball.rs` | 46 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 313 | `steel-core/src/behavior/items/solid_bucket_item.rs` | 50 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 314 | `steel-core/src/behavior/items/spawn_egg.rs` | 250 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 315 | `steel-core/src/behavior/items/spyglass.rs` | 68 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 316 | `steel-core/src/behavior/items/standing_and_wall_block_item.rs` | 164 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 317 | `steel-core/src/behavior/items/throw_projectile.rs` | 77 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 318 | `steel-core/src/behavior/items/throwable_potion.rs` | 104 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 319 | `steel-core/src/behavior/items/tipped_arrow.rs` | 31 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 320 | `steel-core/src/behavior/items/trident.rs` | 271 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 321 | `steel-core/src/behavior/items/wind_charge.rs` | 153 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 322 | `steel-core/src/behavior/mob_effect.rs` | 79 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 323 | `steel-core/src/behavior/mod.rs` | 334 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 324 | `steel-core/src/block_entity/block_state_nbt.rs` | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 325 | `steel-core/src/block_entity/components.rs` | 60 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 326 | `steel-core/src/block_entity/entities/abstract_furnace.rs` | 592 | 6 | 0 | 2 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 327 | `steel-core/src/block_entity/entities/banner.rs` | 132 | 4 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 328 | `steel-core/src/block_entity/entities/barrel.rs` | 276 | 4 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 329 | `steel-core/src/block_entity/entities/beacon.rs` | 519 | 6 | 0 | 1 | 2 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 330 | `steel-core/src/block_entity/entities/beehive.rs` | 664 | 4 | 0 | 0 | 3 | 6 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 331 | `steel-core/src/block_entity/entities/bell.rs` | 287 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 332 | `steel-core/src/block_entity/entities/brewing_stand.rs` | 683 | 3 | 0 | 2 | 1 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 333 | `steel-core/src/block_entity/entities/brushable.rs` | 529 | 3 | 0 | 1 | 2 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 334 | `steel-core/src/block_entity/entities/campfire.rs` | 283 | 3 | 0 | 0 | 3 | 1 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 335 | `steel-core/src/block_entity/entities/chest.rs` | 476 | 6 | 0 | 0 | 1 | 2 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 336 | `steel-core/src/block_entity/entities/chiseled_bookshelf.rs` | 555 | 1 | 0 | 6 | 2 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 337 | `steel-core/src/block_entity/entities/command_block.rs` | 649 | 6 | 1 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 338 | `steel-core/src/block_entity/entities/comparator.rs` | 127 | 0 | 0 | 0 | 3 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 339 | `steel-core/src/block_entity/entities/conduit.rs` | 325 | 2 | 0 | 1 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 340 | `steel-core/src/block_entity/entities/copper_golem_statue.rs` | 123 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 341 | `steel-core/src/block_entity/entities/crafter.rs` | 593 | 5 | 0 | 0 | 1 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 342 | `steel-core/src/block_entity/entities/creaking_heart.rs` | 287 | 0 | 0 | 1 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 343 | `steel-core/src/block_entity/entities/daylight_detector.rs` | 120 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 344 | `steel-core/src/block_entity/entities/decorated_pot.rs` | 160 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 345 | `steel-core/src/block_entity/entities/dispenser.rs` | 265 | 4 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 346 | `steel-core/src/block_entity/entities/enchanting_table.rs` | 170 | 7 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 347 | `steel-core/src/block_entity/entities/end_gateway.rs` | 281 | 1 | 0 | 0 | 1 | 1 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 348 | `steel-core/src/block_entity/entities/end_portal.rs` | 45 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 349 | `steel-core/src/block_entity/entities/ender_chest.rs` | 46 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 350 | `steel-core/src/block_entity/entities/hopper.rs` | 1000 | 4 | 0 | 0 | 2 | 11 | 2 | 0 | 🟡 Требует внимания | Many unwrap/expect: 11 |
| 351 | `steel-core/src/block_entity/entities/jigsaw_block.rs` | 231 | 20 | 0 | 0 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 20 |
| 352 | `steel-core/src/block_entity/entities/jukebox.rs` | 549 | 1 | 0 | 4 | 3 | 4 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 353 | `steel-core/src/block_entity/entities/lectern.rs` | 191 | 2 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 354 | `steel-core/src/block_entity/entities/mod.rs` | 102 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 355 | `steel-core/src/block_entity/entities/piston_moving.rs` | 891 | 0 | 0 | 1 | 1 | 2 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 356 | `steel-core/src/block_entity/entities/potent_sulfur.rs` | 354 | 0 | 0 | 1 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 357 | `steel-core/src/block_entity/entities/potion_brewing.rs` | 222 | 2 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 358 | `steel-core/src/block_entity/entities/sculk_catalyst.rs` | 593 | 0 | 0 | 3 | 1 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 359 | `steel-core/src/block_entity/entities/sculk_sensor.rs` | 520 | 0 | 0 | 14 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 14 |
| 360 | `steel-core/src/block_entity/entities/shelf.rs` | 159 | 2 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 361 | `steel-core/src/block_entity/entities/shulker_box.rs` | 486 | 5 | 0 | 0 | 1 | 1 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 362 | `steel-core/src/block_entity/entities/sign.rs` | 544 | 6 | 0 | 0 | 1 | 2 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 363 | `steel-core/src/block_entity/entities/skull.rs` | 138 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 364 | `steel-core/src/block_entity/entities/spawner/base_spawner.rs` | 677 | 3 | 0 | 1 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 365 | `steel-core/src/block_entity/entities/spawner/mod.rs` | 169 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 366 | `steel-core/src/block_entity/entities/spawner/spawn_data.rs` | 268 | 5 | 0 | 4 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 367 | `steel-core/src/block_entity/entities/spawner/tests.rs` | 361 | 1 | 0 | 1 | 4 | 16 | 0 | 0 | 🟡 Требует внимания | Many unwrap/expect: 16 |
| 368 | `steel-core/src/block_entity/entities/structure_block.rs` | 477 | 9 | 0 | 6 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 369 | `steel-core/src/block_entity/entities/test_block.rs` | 240 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 370 | `steel-core/src/block_entity/entities/test_instance_block.rs` | 172 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 371 | `steel-core/src/block_entity/entities/trial_spawner.rs` | 481 | 9 | 0 | 2 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 372 | `steel-core/src/block_entity/entities/unimplemented.rs` | 137 | 3 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 373 | `steel-core/src/block_entity/entities/vault.rs` | 426 | 11 | 0 | 0 | 0 | 0 | 1 | 0 | 🟡 Требует внимания | High clone/to_owned: 11 |
| 374 | `steel-core/src/block_entity/mod.rs` | 635 | 2 | 0 | 0 | 2 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 375 | `steel-core/src/block_entity/randomizable_container.rs` | 321 | 4 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 376 | `steel-core/src/block_entity/registry.rs` | 426 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 377 | `steel-core/src/block_entity/storage.rs` | 791 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 378 | `steel-core/src/block_state_provider/mod.rs` | 24 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 379 | `steel-core/src/block_state_provider/noise_cache.rs` | 104 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 380 | `steel-core/src/block_state_provider/predicates.rs` | 124 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 381 | `steel-core/src/block_state_provider/providers.rs` | 368 | 1 | 0 | 10 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 382 | `steel-core/src/block_state_provider/tests.rs` | 267 | 1 | 0 | 2 | 3 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 383 | `steel-core/src/bootstrap.rs` | 31 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 384 | `steel-core/src/chunk/block_entity_listener.rs` | 105 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 385 | `steel-core/src/chunk/chunk_generation_task.rs` | 403 | 0 | 0 | 9 | 1 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 386 | `steel-core/src/chunk/chunk_holder.rs` | 1834 | 0 | 0 | 4 | 2 | 18 | 1 | 0 | 🟡 Требует внимания | Many unwrap/expect: 18 |
| 387 | `steel-core/src/chunk/chunk_map/generation_readiness.rs` | 628 | 0 | 0 | 0 | 4 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 388 | `steel-core/src/chunk/chunk_map/light_update_state.rs` | 201 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 389 | `steel-core/src/chunk/chunk_map/light_updates.rs` | 363 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 390 | `steel-core/src/chunk/chunk_map/mod.rs` | 1430 | 3 | 0 | 3 | 3 | 0 | 5 | 0 | ✅ Переписан (lightest-first, budget, modern) | hysteresis, Vec<DeferredWork>, path.contains, FxBuildHasher, is_multiple_of |
| 391 | `steel-core/src/chunk/chunk_map/persistence.rs` | 359 | 0 | 1 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 392 | `steel-core/src/chunk/chunk_map/player_tracking.rs` | 228 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 393 | `steel-core/src/chunk/chunk_map/scheduled_ticks.rs` | 93 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 394 | `steel-core/src/chunk/chunk_map/tests/light_updates.rs` | 245 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 395 | `steel-core/src/chunk/chunk_map/tests/mod.rs` | 293 | 0 | 0 | 4 | 3 | 3 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 396 | `steel-core/src/chunk/chunk_map/tests/persistence_unloads.rs` | 335 | 0 | 0 | 1 | 1 | 8 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 397 | `steel-core/src/chunk/chunk_map/tests/player_tracking.rs` | 319 | 0 | 0 | 0 | 0 | 6 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 398 | `steel-core/src/chunk/chunk_map/tests/request_cleanup.rs` | 107 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 399 | `steel-core/src/chunk/chunk_map/tests/scheduled_ticks.rs` | 120 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 400 | `steel-core/src/chunk/chunk_map/tests/simulation_tickets.rs` | 218 | 1 | 1 | 0 | 1 | 7 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 401 | `steel-core/src/chunk/chunk_map/tests/tickets_generation_readiness.rs` | 527 | 0 | 0 | 1 | 6 | 15 | 1 | 0 | 🟡 Требует внимания | Vec::new without capacity in tick: 6; Many unwrap/expect: 15 |
| 402 | `steel-core/src/chunk/chunk_map/tests/timed_simulation.rs` | 106 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 403 | `steel-core/src/chunk/chunk_pyramid.rs` | 447 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 404 | `steel-core/src/chunk/chunk_request.rs` | 377 | 0 | 0 | 1 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 405 | `steel-core/src/chunk/chunk_scheduler.rs` | 679 | 0 | 0 | 0 | 0 | 4 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 406 | `steel-core/src/chunk/chunk_status_tasks.rs` | 124 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 407 | `steel-core/src/chunk/chunk_ticket.rs` | 150 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 408 | `steel-core/src/chunk/chunk_ticket_manager.rs` | 419 | 0 | 0 | 1 | 0 | 3 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 409 | `steel-core/src/chunk/chunk_ticket_storage.rs` | 905 | 1 | 2 | 0 | 4 | 5 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 410 | `steel-core/src/chunk/chunk_tracker/mod.rs` | 347 | 0 | 0 | 1 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 411 | `steel-core/src/chunk/chunk_tracker/tests.rs` | 368 | 0 | 0 | 0 | 1 | 8 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 412 | `steel-core/src/chunk/data.rs` | 1623 | 0 | 0 | 20 | 1 | 2 | 0 | 0 | 🟡 Требует внимания | Many as casts: 20 |
| 413 | `steel-core/src/chunk/full_chunk/game_event_tests.rs` | 371 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 414 | `steel-core/src/chunk/full_chunk/mod.rs` | 1934 | 0 | 0 | 28 | 3 | 5 | 1 | 0 | 🟡 Требует внимания | Many as casts: 28 |
| 415 | `steel-core/src/chunk/full_chunk/tests.rs` | 648 | 0 | 0 | 0 | 2 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 416 | `steel-core/src/chunk/full_chunk_readiness.rs` | 333 | 0 | 0 | 0 | 0 | 4 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 417 | `steel-core/src/chunk/gameplay_chunk_lookup_cache.rs` | 749 | 1 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 418 | `steel-core/src/chunk/heightmap.rs` | 793 | 0 | 0 | 6 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 419 | `steel-core/src/chunk/light/cache.rs` | 1339 | 0 | 0 | 28 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 28 |
| 420 | `steel-core/src/chunk/light/data_layer.rs` | 179 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 421 | `steel-core/src/chunk/light/mod.rs` | 456 | 0 | 0 | 0 | 0 | 9 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 422 | `steel-core/src/chunk/light/packet.rs` | 138 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 423 | `steel-core/src/chunk/light/propagation.rs` | 1450 | 0 | 0 | 3 | 8 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 424 | `steel-core/src/chunk/light/queue.rs` | 1236 | 0 | 0 | 7 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 425 | `steel-core/src/chunk/light/section_storage.rs` | 132 | 0 | 0 | 6 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 426 | `steel-core/src/chunk/light/sky_propagation/algorithms.rs` | 515 | 0 | 0 | 5 | 3 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 427 | `steel-core/src/chunk/light/sky_propagation/context.rs` | 196 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 428 | `steel-core/src/chunk/light/sky_propagation/mod.rs` | 283 | 0 | 0 | 0 | 7 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 429 | `steel-core/src/chunk/light/sky_propagation/queue_engine.rs` | 521 | 0 | 0 | 3 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 430 | `steel-core/src/chunk/light/sky_propagation/tests.rs` | 448 | 0 | 0 | 1 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 431 | `steel-core/src/chunk/light/sky_sources.rs` | 282 | 0 | 0 | 1 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 432 | `steel-core/src/chunk/light/storage.rs` | 473 | 0 | 0 | 1 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 433 | `steel-core/src/chunk/light/work_gate.rs` | 150 | 0 | 0 | 2 | 1 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 434 | `steel-core/src/chunk/light/workset.rs` | 1464 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 435 | `steel-core/src/chunk/mod.rs` | 33 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 436 | `steel-core/src/chunk/paletted_container.rs` | 831 | 0 | 0 | 10 | 8 | 8 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 437 | `steel-core/src/chunk/paletted_container/encode.rs` | 210 | 0 | 0 | 3 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 438 | `steel-core/src/chunk/player_chunk_view.rs` | 213 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 439 | `steel-core/src/chunk/player_ticket_tracker.rs` | 399 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 440 | `steel-core/src/chunk/section.rs` | 1400 | 0 | 0 | 16 | 2 | 5 | 2 | 0 | 🟡 Требует внимания | Many as casts: 16 |
| 441 | `steel-core/src/chunk/simulation_ticket_manager.rs` | 8 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 442 | `steel-core/src/chunk/status.rs` | 109 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 443 | `steel-core/src/chunk_saver/bit_pack.rs` | 116 | 0 | 0 | 4 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 444 | `steel-core/src/chunk_saver/format.rs` | 1306 | 0 | 1 | 9 | 0 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 445 | `steel-core/src/chunk_saver/mod.rs` | 43 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 446 | `steel-core/src/chunk_saver/ram_only.rs` | 86 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 447 | `steel-core/src/chunk_saver/region_manager.rs` | 974 | 1 | 13 | 25 | 12 | 40 | 0 | 0 | 🟡 Требует внимания | Many format!: 13; Many as casts: 25 |
| 448 | `steel-core/src/chunk_saver/registry.rs` | 253 | 9 | 9 | 0 | 0 | 4 | 0 | 0 | 🟡 Требует внимания | Many format!: 9 |
| 449 | `steel-core/src/chunk_saver/storage/chunk.rs` | 766 | 0 | 25 | 8 | 2 | 5 | 0 | 0 | 🟡 Требует внимания | Many format!: 25 |
| 450 | `steel-core/src/chunk_saver/storage/entities.rs` | 430 | 3 | 0 | 0 | 6 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 451 | `steel-core/src/chunk_saver/storage/light.rs` | 162 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 452 | `steel-core/src/chunk_saver/storage/mod.rs` | 812 | 3 | 0 | 2 | 7 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 453 | `steel-core/src/chunk_saver/storage/structures.rs` | 1088 | 19 | 0 | 0 | 1 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 19 |
| 454 | `steel-core/src/chunk_saver/storage/tests/chunks_sections.rs` | 339 | 0 | 0 | 0 | 6 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 455 | `steel-core/src/chunk_saver/storage/tests/entities.rs` | 725 | 11 | 0 | 0 | 6 | 7 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 11 |
| 456 | `steel-core/src/chunk_saver/storage/tests/light.rs` | 120 | 0 | 0 | 0 | 8 | 5 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 457 | `steel-core/src/chunk_saver/storage/tests/mod.rs` | 50 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 458 | `steel-core/src/chunk_saver/storage/tests/structures.rs` | 730 | 16 | 0 | 0 | 12 | 6 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 16 |
| 459 | `steel-core/src/chunk_saver/storage/tests/ticks.rs` | 169 | 6 | 0 | 0 | 6 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 460 | `steel-core/src/chunk_saver/storage/ticks.rs` | 92 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 461 | `steel-core/src/command/api.rs` | 959 | 4 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 462 | `steel-core/src/command/brigadier/argument.rs` | 246 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 463 | `steel-core/src/command/brigadier/builder.rs` | 432 | 13 | 0 | 0 | 4 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 13 |
| 464 | `steel-core/src/command/brigadier/context.rs` | 572 | 7 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 465 | `steel-core/src/command/brigadier/context_chain.rs` | 166 | 1 | 0 | 0 | 3 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 466 | `steel-core/src/command/brigadier/dispatcher.rs` | 436 | 3 | 0 | 0 | 6 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 467 | `steel-core/src/command/brigadier/error.rs` | 372 | 29 | 2 | 0 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 29 |
| 468 | `steel-core/src/command/brigadier/mod.rs` | 90 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 469 | `steel-core/src/command/brigadier/node.rs` | 638 | 7 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 470 | `steel-core/src/command/brigadier/reader.rs` | 329 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 471 | `steel-core/src/command/brigadier/runtime.rs` | 29 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 472 | `steel-core/src/command/brigadier/string_range.rs` | 82 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 473 | `steel-core/src/command/brigadier/suggestion.rs` | 351 | 3 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 474 | `steel-core/src/command/brigadier/tests/argument.rs` | 193 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 475 | `steel-core/src/command/brigadier/tests/context_chain.rs` | 325 | 32 | 1 | 0 | 6 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 32 |
| 476 | `steel-core/src/command/brigadier/tests/mod.rs` | 416 | 9 | 1 | 0 | 0 | 6 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 477 | `steel-core/src/command/brigadier/tests/parsing.rs` | 304 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 478 | `steel-core/src/command/brigadier/tests/registration.rs` | 336 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 479 | `steel-core/src/command/brigadier/tests/runtime_model.rs` | 159 | 3 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 480 | `steel-core/src/command/brigadier/tests/suggestion.rs` | 226 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 481 | `steel-core/src/command/builtins/advancement.rs` | 464 | 15 | 0 | 2 | 2 | 2 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 15 |
| 482 | `steel-core/src/command/builtins/attribute.rs` | 319 | 15 | 11 | 2 | 1 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 15; Many format!: 11 |
| 483 | `steel-core/src/command/builtins/ban.rs` | 315 | 17 | 5 | 1 | 3 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 17 |
| 484 | `steel-core/src/command/builtins/banip.rs` | 149 | 8 | 4 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 485 | `steel-core/src/command/builtins/bossbar.rs` | 421 | 20 | 6 | 1 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 20; Many format!: 6 |
| 486 | `steel-core/src/command/builtins/clear.rs` | 213 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 487 | `steel-core/src/command/builtins/clone.rs` | 196 | 5 | 1 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 488 | `steel-core/src/command/builtins/damage.rs` | 103 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 489 | `steel-core/src/command/builtins/data.rs` | 428 | 13 | 2 | 4 | 4 | 9 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 13 |
| 490 | `steel-core/src/command/builtins/datapack.rs` | 432 | 20 | 13 | 5 | 2 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 20; Many format!: 13 |
| 491 | `steel-core/src/command/builtins/defaultgamemode.rs` | 133 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 492 | `steel-core/src/command/builtins/dialog.rs` | 89 | 4 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 493 | `steel-core/src/command/builtins/difficulty.rs` | 118 | 1 | 0 | 0 | 0 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 494 | `steel-core/src/command/builtins/domain.rs` | 386 | 2 | 4 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 495 | `steel-core/src/command/builtins/effect.rs` | 188 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 496 | `steel-core/src/command/builtins/enchant.rs` | 360 | 5 | 2 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 497 | `steel-core/src/command/builtins/execute/condition.rs` | 954 | 7 | 2 | 3 | 1 | 9 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 498 | `steel-core/src/command/builtins/execute/mod.rs` | 90 | 1 | 2 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 499 | `steel-core/src/command/builtins/execute/source.rs` | 257 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 500 | `steel-core/src/command/builtins/execute/store.rs` | 448 | 6 | 1 | 6 | 1 | 7 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 501 | `steel-core/src/command/builtins/experience.rs` | 404 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 502 | `steel-core/src/command/builtins/fill.rs` | 539 | 3 | 0 | 1 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 503 | `steel-core/src/command/builtins/fillbiome.rs` | 176 | 5 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 504 | `steel-core/src/command/builtins/fly.rs` | 279 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 505 | `steel-core/src/command/builtins/forceload.rs` | 274 | 5 | 8 | 0 | 1 | 0 | 0 | 0 | 🟡 Требует внимания | Many format!: 8 |
| 506 | `steel-core/src/command/builtins/function.rs` | 119 | 4 | 2 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 507 | `steel-core/src/command/builtins/gamemode.rs` | 352 | 1 | 1 | 0 | 0 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 508 | `steel-core/src/command/builtins/gamerule.rs` | 184 | 6 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 509 | `steel-core/src/command/builtins/give.rs` | 190 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 510 | `steel-core/src/command/builtins/help.rs` | 166 | 5 | 8 | 1 | 1 | 0 | 0 | 0 | 🟡 Требует внимания | Many format!: 8 |
| 511 | `steel-core/src/command/builtins/invsee.rs` | 293 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 512 | `steel-core/src/command/builtins/invsee/tests.rs` | 723 | 1 | 0 | 1 | 3 | 2 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 513 | `steel-core/src/command/builtins/item.rs` | 222 | 7 | 1 | 3 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 514 | `steel-core/src/command/builtins/kick.rs` | 72 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 515 | `steel-core/src/command/builtins/kill.rs` | 108 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 516 | `steel-core/src/command/builtins/list.rs` | 62 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 517 | `steel-core/src/command/builtins/locate.rs` | 610 | 12 | 3 | 3 | 1 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 12 |
| 518 | `steel-core/src/command/builtins/loot.rs` | 155 | 1 | 2 | 3 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 519 | `steel-core/src/command/builtins/mod.rs` | 744 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 520 | `steel-core/src/command/builtins/msg.rs` | 80 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 521 | `steel-core/src/command/builtins/operator.rs` | 259 | 5 | 1 | 2 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 522 | `steel-core/src/command/builtins/particle.rs` | 94 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 523 | `steel-core/src/command/builtins/perms/config.rs` | 338 | 20 | 0 | 0 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 20 |
| 524 | `steel-core/src/command/builtins/perms/mod.rs` | 1492 | 39 | 33 | 2 | 8 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 39; Many format!: 33 |
| 525 | `steel-core/src/command/builtins/playsound.rs` | 257 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 526 | `steel-core/src/command/builtins/position.rs` | 33 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 527 | `steel-core/src/command/builtins/posteffect.rs` | 323 | 10 | 0 | 0 | 3 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 528 | `steel-core/src/command/builtins/random.rs` | 210 | 11 | 0 | 1 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 11 |
| 529 | `steel-core/src/command/builtins/recipe.rs` | 111 | 1 | 1 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 530 | `steel-core/src/command/builtins/reload.rs` | 150 | 2 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 531 | `steel-core/src/command/builtins/return_command.rs` | 162 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 532 | `steel-core/src/command/builtins/ride.rs` | 48 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 533 | `steel-core/src/command/builtins/rotate.rs` | 189 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 534 | `steel-core/src/command/builtins/save.rs` | 105 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 535 | `steel-core/src/command/builtins/say.rs` | 55 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 536 | `steel-core/src/command/builtins/schedule.rs` | 134 | 7 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 537 | `steel-core/src/command/builtins/scoreboard.rs` | 392 | 16 | 13 | 0 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 16; Many format!: 13 |
| 538 | `steel-core/src/command/builtins/seed.rs` | 39 | 2 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 539 | `steel-core/src/command/builtins/serverpack.rs` | 180 | 1 | 9 | 0 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Many format!: 9 |
| 540 | `steel-core/src/command/builtins/setblock.rs` | 96 | 0 | 3 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 541 | `steel-core/src/command/builtins/setidletimeout.rs` | 50 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 542 | `steel-core/src/command/builtins/setworldspawn.rs` | 147 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 543 | `steel-core/src/command/builtins/spawnpoint.rs` | 200 | 12 | 0 | 0 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 12 |
| 544 | `steel-core/src/command/builtins/spectate.rs` | 120 | 4 | 1 | 0 | 3 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 545 | `steel-core/src/command/builtins/spreadplayers.rs` | 149 | 4 | 1 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 546 | `steel-core/src/command/builtins/stop.rs` | 28 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 547 | `steel-core/src/command/builtins/stopsound.rs` | 127 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 548 | `steel-core/src/command/builtins/stopwatch.rs` | 168 | 5 | 10 | 3 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Many format!: 10 |
| 549 | `steel-core/src/command/builtins/summon.rs` | 214 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 550 | `steel-core/src/command/builtins/swing.rs` | 122 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 551 | `steel-core/src/command/builtins/tag.rs` | 133 | 8 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 552 | `steel-core/src/command/builtins/team.rs` | 355 | 20 | 13 | 0 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 20; Many format!: 13 |
| 553 | `steel-core/src/command/builtins/teammsg.rs` | 88 | 7 | 0 | 1 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 554 | `steel-core/src/command/builtins/teleport.rs` | 573 | 2 | 3 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 555 | `steel-core/src/command/builtins/tellraw.rs` | 104 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 556 | `steel-core/src/command/builtins/tick.rs` | 410 | 4 | 3 | 3 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 557 | `steel-core/src/command/builtins/time.rs` | 445 | 21 | 1 | 1 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 21 |
| 558 | `steel-core/src/command/builtins/title.rs` | 315 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 559 | `steel-core/src/command/builtins/tpsbar.rs` | 70 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 560 | `steel-core/src/command/builtins/transfer.rs` | 104 | 5 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 561 | `steel-core/src/command/builtins/trigger.rs` | 173 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 562 | `steel-core/src/command/builtins/version.rs` | 112 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 563 | `steel-core/src/command/builtins/waypoint.rs` | 179 | 3 | 3 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 564 | `steel-core/src/command/builtins/weather.rs` | 110 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 565 | `steel-core/src/command/builtins/whitelist.rs` | 252 | 13 | 2 | 1 | 4 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 13 |
| 566 | `steel-core/src/command/builtins/worldborder.rs` | 364 | 2 | 11 | 8 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Many format!: 11 |
| 567 | `steel-core/src/command/execution/argument.rs` | 1789 | 14 | 2 | 1 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 14 |
| 568 | `steel-core/src/command/execution/argument_tests/coordinates.rs` | 172 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 569 | `steel-core/src/command/execution/argument_tests/core_permissions.rs` | 321 | 1 | 2 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 570 | `steel-core/src/command/execution/argument_tests/general.rs` | 428 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 571 | `steel-core/src/command/execution/argument_tests/item_predicate.rs` | 325 | 0 | 2 | 0 | 3 | 8 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 572 | `steel-core/src/command/execution/argument_tests/item_slots.rs` | 31 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 573 | `steel-core/src/command/execution/argument_tests/item_stack.rs` | 473 | 0 | 0 | 2 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 574 | `steel-core/src/command/execution/argument_tests/mod.rs` | 182 | 17 | 0 | 0 | 2 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 17 |
| 575 | `steel-core/src/command/execution/argument_tests/resources_world.rs` | 397 | 0 | 3 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 576 | `steel-core/src/command/execution/argument_tests/selector_time.rs` | 191 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 577 | `steel-core/src/command/execution/biome.rs` | 111 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 578 | `steel-core/src/command/execution/block.rs` | 443 | 12 | 1 | 0 | 3 | 1 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 12 |
| 579 | `steel-core/src/command/execution/block/suggestions.rs` | 298 | 4 | 11 | 0 | 3 | 0 | 0 | 0 | 🟡 Требует внимания | Many format!: 11 |
| 580 | `steel-core/src/command/execution/coordinates.rs` | 565 | 0 | 7 | 2 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Many format!: 7 |
| 581 | `steel-core/src/command/execution/item.rs` | 389 | 11 | 6 | 0 | 0 | 3 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 11; Many format!: 6 |
| 582 | `steel-core/src/command/execution/item_predicate.rs` | 1390 | 12 | 13 | 1 | 11 | 1 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 12; Many format!: 13 |
| 583 | `steel-core/src/command/execution/mod.rs` | 64 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 584 | `steel-core/src/command/execution/nbt.rs` | 30 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 585 | `steel-core/src/command/execution/permission.rs` | 410 | 7 | 5 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 586 | `steel-core/src/command/execution/profile.rs` | 148 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 587 | `steel-core/src/command/execution/queue.rs` | 905 | 9 | 1 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 588 | `steel-core/src/command/execution/runtime.rs` | 514 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 589 | `steel-core/src/command/execution/score.rs` | 260 | 3 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 590 | `steel-core/src/command/execution/selector/mod.rs` | 453 | 1 | 4 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 591 | `steel-core/src/command/execution/selector/model.rs` | 565 | 5 | 1 | 0 | 7 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 592 | `steel-core/src/command/execution/selector/parser.rs` | 980 | 7 | 11 | 1 | 1 | 5 | 0 | 0 | 🟡 Требует внимания | Many format!: 11 |
| 593 | `steel-core/src/command/execution/selector/suggestions.rs` | 533 | 2 | 14 | 0 | 12 | 0 | 0 | 0 | 🟡 Требует внимания | Many format!: 14 |
| 594 | `steel-core/src/command/execution/selector/tests.rs` | 334 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 595 | `steel-core/src/command/execution/source.rs` | 1042 | 36 | 2 | 4 | 23 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 36 |
| 596 | `steel-core/src/command/execution/structure.rs` | 88 | 3 | 4 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 597 | `steel-core/src/command/execution/suggestions.rs` | 124 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 598 | `steel-core/src/command/execution/swing_animation.rs` | 66 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 599 | `steel-core/src/command/execution/tests.rs` | 1530 | 9 | 0 | 0 | 7 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 600 | `steel-core/src/command/execution/text.rs` | 608 | 29 | 5 | 0 | 3 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 29 |
| 601 | `steel-core/src/command/execution/world.rs` | 75 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 602 | `steel-core/src/command/mod.rs` | 139 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 603 | `steel-core/src/command/protocol.rs` | 875 | 5 | 0 | 2 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 604 | `steel-core/src/command/queue/mod.rs` | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 605 | `steel-core/src/command/queue/pending.rs` | 175 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 606 | `steel-core/src/command/queue/requests.rs` | 349 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 607 | `steel-core/src/command/registration/mod.rs` | 630 | 31 | 1 | 0 | 6 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 31 |
| 608 | `steel-core/src/command/registration/tests.rs` | 677 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 609 | `steel-core/src/command/sender.rs` | 271 | 2 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 610 | `steel-core/src/command/storage.rs` | 345 | 13 | 6 | 1 | 1 | 11 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 13; Many format!: 6 |
| 611 | `steel-core/src/config.rs` | 1624 | 43 | 19 | 0 | 1 | 15 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 43; Many format!: 19 |
| 612 | `steel-core/src/enchantment_helper.rs` | 1655 | 2 | 0 | 14 | 1 | 1 | 0 | 0 | 🟡 Требует внимания | Many as casts: 14 |
| 613 | `steel-core/src/enchantment_helper/item_in_use.rs` | 36 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 614 | `steel-core/src/enchantment_helper/selection.rs` | 187 | 1 | 0 | 4 | 3 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 615 | `steel-core/src/entity/ageable.rs` | 548 | 0 | 0 | 2 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 616 | `steel-core/src/entity/ai/control.rs` | 414 | 0 | 0 | 1 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 617 | `steel-core/src/entity/ai/goal/avoid_entity.rs` | 217 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 618 | `steel-core/src/entity/ai/goal/breath_air.rs` | 208 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 619 | `steel-core/src/entity/ai/goal/breed_goal.rs` | 153 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 620 | `steel-core/src/entity/ai/goal/climb_on_top_of_powder_snow.rs` | 130 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 621 | `steel-core/src/entity/ai/goal/door_interact.rs` | 279 | 0 | 0 | 4 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 622 | `steel-core/src/entity/ai/goal/eat_block_goal.rs` | 248 | 0 | 0 | 0 | 0 | 5 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 623 | `steel-core/src/entity/ai/goal/flee_sun.rs` | 185 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 624 | `steel-core/src/entity/ai/goal/float_goal.rs` | 149 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 625 | `steel-core/src/entity/ai/goal/follow_mob.rs` | 257 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 626 | `steel-core/src/entity/ai/goal/follow_parent.rs` | 119 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 627 | `steel-core/src/entity/ai/goal/hurt_by_target.rs` | 262 | 0 | 0 | 0 | 2 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 628 | `steel-core/src/entity/ai/goal/interact.rs` | 60 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 629 | `steel-core/src/entity/ai/goal/leap_at_target.rs` | 181 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 630 | `steel-core/src/entity/ai/goal/look_at_player.rs` | 247 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 631 | `steel-core/src/entity/ai/goal/melee_attack.rs` | 334 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 632 | `steel-core/src/entity/ai/goal/merge_with_stone.rs` | 146 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 633 | `steel-core/src/entity/ai/goal/mod.rs` | 84 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 634 | `steel-core/src/entity/ai/goal/move_to_block.rs` | 329 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 635 | `steel-core/src/entity/ai/goal/move_towards_restriction.rs` | 81 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 636 | `steel-core/src/entity/ai/goal/move_towards_target.rs` | 117 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 637 | `steel-core/src/entity/ai/goal/nearest_attackable_target.rs` | 235 | 1 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 638 | `steel-core/src/entity/ai/goal/obtain_raid_banner.rs` | 97 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 639 | `steel-core/src/entity/ai/goal/open_door.rs` | 123 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 640 | `steel-core/src/entity/ai/goal/panic_goal.rs` | 162 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 641 | `steel-core/src/entity/ai/goal/raid_break_door.rs` | 96 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 642 | `steel-core/src/entity/ai/goal/raid_celebration.rs` | 53 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 643 | `steel-core/src/entity/ai/goal/random_look_around.rs` | 170 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 644 | `steel-core/src/entity/ai/goal/random_pos.rs` | 348 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 645 | `steel-core/src/entity/ai/goal/random_stroll.rs` | 104 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 646 | `steel-core/src/entity/ai/goal/random_swimming.rs` | 143 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 647 | `steel-core/src/entity/ai/goal/ranged_attack.rs` | 164 | 0 | 0 | 1 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 648 | `steel-core/src/entity/ai/goal/restrict_open_door.rs` | 128 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 649 | `steel-core/src/entity/ai/goal/restrict_sun.rs` | 80 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 650 | `steel-core/src/entity/ai/goal/selector.rs` | 692 | 0 | 0 | 0 | 1 | 0 | 6 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 651 | `steel-core/src/entity/ai/goal/sleep_in_bed.rs` | 143 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 652 | `steel-core/src/entity/ai/goal/squid_flee_goal.rs` | 96 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 653 | `steel-core/src/entity/ai/goal/squid_random_movement_goal.rs` | 70 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 654 | `steel-core/src/entity/ai/goal/target_goal.rs` | 272 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 655 | `steel-core/src/entity/ai/goal/tempt_goal.rs` | 212 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 656 | `steel-core/src/entity/ai/goal/try_find_liquid.rs` | 168 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 657 | `steel-core/src/entity/ai/goal/water_avoiding_random_stroll.rs` | 65 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 658 | `steel-core/src/entity/ai/mod.rs` | 15 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 659 | `steel-core/src/entity/ai/navigation.rs` | 1554 | 1 | 0 | 7 | 2 | 0 | 5 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 660 | `steel-core/src/entity/ai/node.rs` | 531 | 0 | 0 | 22 | 1 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 22 |
| 661 | `steel-core/src/entity/ai/path.rs` | 620 | 0 | 0 | 4 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 662 | `steel-core/src/entity/ai/pathfinder.rs` | 514 | 1 | 0 | 9 | 3 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 663 | `steel-core/src/entity/ai/sensing.rs` | 93 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 664 | `steel-core/src/entity/ai/targeting.rs` | 245 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 665 | `steel-core/src/entity/ai/walk/collision.rs` | 14 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 666 | `steel-core/src/entity/ai/walk/mod.rs` | 32 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 667 | `steel-core/src/entity/ai/walk/node_evaluator.rs` | 730 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 668 | `steel-core/src/entity/ai/walk/path_evaluator.rs` | 169 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 669 | `steel-core/src/entity/ai/walk/settings.rs` | 240 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 670 | `steel-core/src/entity/ai/walk/tests.rs` | 640 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 671 | `steel-core/src/entity/animal.rs` | 517 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 672 | `steel-core/src/entity/attribute.rs` | 1025 | 12 | 0 | 7 | 4 | 3 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 12 |
| 673 | `steel-core/src/entity/base/fire_freeze.rs` | 89 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 674 | `steel-core/src/entity/base/mod.rs` | 1811 | 9 | 0 | 2 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 675 | `steel-core/src/entity/base/movement.rs` | 342 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 676 | `steel-core/src/entity/base/persistence.rs` | 96 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 677 | `steel-core/src/entity/base/relationships.rs` | 108 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 678 | `steel-core/src/entity/base/tests.rs` | 1350 | 2 | 1 | 1 | 1 | 5 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 679 | `steel-core/src/entity/block_attached_entity.rs` | 244 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 680 | `steel-core/src/entity/block_effects.rs` | 660 | 0 | 0 | 20 | 3 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 20 |
| 681 | `steel-core/src/entity/bucketable.rs` | 188 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 682 | `steel-core/src/entity/callback.rs` | 302 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 683 | `steel-core/src/entity/combat_rules.rs` | 65 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 684 | `steel-core/src/entity/consume_effect/apply_effects.rs` | 30 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 685 | `steel-core/src/entity/consume_effect/clear_all_effects.rs` | 21 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 686 | `steel-core/src/entity/consume_effect/mod.rs` | 42 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 687 | `steel-core/src/entity/consume_effect/play_sound.rs` | 34 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 688 | `steel-core/src/entity/consume_effect/remove_effects.rs` | 25 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 689 | `steel-core/src/entity/consume_effect/teleport_randomly.rs` | 178 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 690 | `steel-core/src/entity/conversion.rs` | 757 | 2 | 0 | 0 | 0 | 27 | 0 | 0 | 🟡 Требует внимания | Many unwrap/expect: 27 |
| 691 | `steel-core/src/entity/damage/history.rs` | 115 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 692 | `steel-core/src/entity/damage/history/binding.rs` | 118 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 693 | `steel-core/src/entity/damage/history/source.rs` | 126 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 694 | `steel-core/src/entity/damage/history/tests.rs` | 133 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 695 | `steel-core/src/entity/damage/mod.rs` | 11 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 696 | `steel-core/src/entity/damage/source.rs` | 173 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 697 | `steel-core/src/entity/damage/tests/mod.rs` | 282 | 2 | 0 | 0 | 0 | 16 | 0 | 0 | 🟡 Требует внимания | Many unwrap/expect: 16 |
| 698 | `steel-core/src/entity/damage/tests/retention.rs` | 252 | 0 | 0 | 0 | 0 | 14 | 0 | 0 | 🟡 Требует внимания | Many unwrap/expect: 14 |
| 699 | `steel-core/src/entity/dismount_helper.rs` | 223 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 700 | `steel-core/src/entity/entities/mobs/ambient/bat.rs` | 535 | 0 | 0 | 3 | 1 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 701 | `steel-core/src/entity/entities/mobs/ambient/mod.rs` | 7 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 702 | `steel-core/src/entity/entities/mobs/bosses/ender_dragon/flight_history.rs` | 135 | 0 | 0 | 6 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 703 | `steel-core/src/entity/entities/mobs/bosses/ender_dragon/mod.rs` | 1484 | 0 | 0 | 14 | 1 | 0 | 3 | 0 | 🟡 Требует внимания | Many as casts: 14 |
| 704 | `steel-core/src/entity/entities/mobs/bosses/ender_dragon/part.rs` | 424 | 0 | 0 | 0 | 0 | 3 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 705 | `steel-core/src/entity/entities/mobs/bosses/ender_dragon/phase_ticks.rs` | 1119 | 3 | 0 | 10 | 0 | 0 | 12 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 706 | `steel-core/src/entity/entities/mobs/bosses/ender_dragon/phases.rs` | 1141 | 2 | 0 | 17 | 2 | 7 | 0 | 0 | 🟡 Требует внимания | Many as casts: 17 |
| 707 | `steel-core/src/entity/entities/mobs/bosses/mod.rs` | 11 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 708 | `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs` | 731 | 0 | 0 | 6 | 0 | 0 | 4 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 709 | `steel-core/src/entity/entities/mobs/hostile/blaze.rs` | 254 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 710 | `steel-core/src/entity/entities/mobs/hostile/bogged.rs` | 420 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 711 | `steel-core/src/entity/entities/mobs/hostile/breeze.rs` | 291 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 712 | `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs` | 300 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 713 | `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs` | 547 | 1 | 0 | 5 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 714 | `steel-core/src/entity/entities/mobs/hostile/creaking.rs` | 365 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 715 | `steel-core/src/entity/entities/mobs/hostile/creeper.rs` | 516 | 0 | 0 | 1 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 716 | `steel-core/src/entity/entities/mobs/hostile/cube_mob.rs` | 224 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 717 | `steel-core/src/entity/entities/mobs/hostile/drowned.rs` | 294 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 718 | `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs` | 341 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 719 | `steel-core/src/entity/entities/mobs/hostile/enderman.rs` | 467 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 720 | `steel-core/src/entity/entities/mobs/hostile/endermite.rs` | 419 | 0 | 0 | 1 | 1 | 4 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 721 | `steel-core/src/entity/entities/mobs/hostile/evoker.rs` | 266 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 722 | `steel-core/src/entity/entities/mobs/hostile/ghast.rs` | 254 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 723 | `steel-core/src/entity/entities/mobs/hostile/giant.rs` | 241 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 724 | `steel-core/src/entity/entities/mobs/hostile/guardian.rs` | 305 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 725 | `steel-core/src/entity/entities/mobs/hostile/hoglin.rs` | 298 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 726 | `steel-core/src/entity/entities/mobs/hostile/husk.rs` | 294 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 727 | `steel-core/src/entity/entities/mobs/hostile/illusioner.rs` | 266 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 728 | `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs` | 350 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 729 | `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs` | 584 | 0 | 0 | 6 | 1 | 1 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 730 | `steel-core/src/entity/entities/mobs/hostile/mod.rs` | 134 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 731 | `steel-core/src/entity/entities/mobs/hostile/parched.rs` | 366 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 732 | `steel-core/src/entity/entities/mobs/hostile/phantom.rs` | 433 | 0 | 0 | 1 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 733 | `steel-core/src/entity/entities/mobs/hostile/piglin.rs` | 299 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 734 | `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs` | 296 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 735 | `steel-core/src/entity/entities/mobs/hostile/pillager.rs` | 469 | 2 | 0 | 1 | 1 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 736 | `steel-core/src/entity/entities/mobs/hostile/ravager.rs` | 266 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 737 | `steel-core/src/entity/entities/mobs/hostile/shulker.rs` | 297 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 738 | `steel-core/src/entity/entities/mobs/hostile/silverfish.rs` | 301 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 739 | `steel-core/src/entity/entities/mobs/hostile/skeleton.rs` | 382 | 0 | 0 | 1 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 740 | `steel-core/src/entity/entities/mobs/hostile/slime.rs` | 648 | 0 | 0 | 3 | 1 | 6 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 741 | `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs` | 258 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 742 | `steel-core/src/entity/entities/mobs/hostile/spider.rs` | 298 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 743 | `steel-core/src/entity/entities/mobs/hostile/stray.rs` | 365 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 744 | `steel-core/src/entity/entities/mobs/hostile/vex.rs` | 254 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 745 | `steel-core/src/entity/entities/mobs/hostile/vindicator.rs` | 332 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 746 | `steel-core/src/entity/entities/mobs/hostile/warden.rs` | 374 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 747 | `steel-core/src/entity/entities/mobs/hostile/witch.rs` | 266 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 748 | `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs` | 304 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 749 | `steel-core/src/entity/entities/mobs/hostile/zoglin.rs` | 298 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 750 | `steel-core/src/entity/entities/mobs/hostile/zombie.rs` | 294 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 751 | `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs` | 469 | 0 | 0 | 1 | 1 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 752 | `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs` | 309 | 0 | 0 | 1 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 753 | `steel-core/src/entity/entities/mobs/jockeys/mod.rs` | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 754 | `steel-core/src/entity/entities/mobs/mod.rs` | 8 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 755 | `steel-core/src/entity/entities/mobs/neutral/mod.rs` | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 756 | `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs` | 391 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 757 | `steel-core/src/entity/entities/mobs/passive/allay.rs` | 512 | 5 | 0 | 1 | 0 | 0 | 4 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 758 | `steel-core/src/entity/entities/mobs/passive/armadillo.rs` | 408 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 759 | `steel-core/src/entity/entities/mobs/passive/axolotl.rs` | 516 | 0 | 0 | 0 | 0 | 1 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 760 | `steel-core/src/entity/entities/mobs/passive/bee.rs` | 336 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 761 | `steel-core/src/entity/entities/mobs/passive/camel.rs` | 539 | 0 | 0 | 2 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 762 | `steel-core/src/entity/entities/mobs/passive/camel_husk.rs` | 384 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 763 | `steel-core/src/entity/entities/mobs/passive/cat/mod.rs` | 684 | 3 | 0 | 0 | 0 | 1 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 764 | `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs` | 689 | 2 | 0 | 0 | 0 | 0 | 4 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 765 | `steel-core/src/entity/entities/mobs/passive/chicken/tests/core.rs` | 229 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 766 | `steel-core/src/entity/entities/mobs/passive/chicken/tests/mod.rs` | 18 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 767 | `steel-core/src/entity/entities/mobs/passive/chicken/tests/persistence.rs` | 82 | 2 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 768 | `steel-core/src/entity/entities/mobs/passive/cow/mod.rs` | 502 | 2 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 769 | `steel-core/src/entity/entities/mobs/passive/cow/tests/core.rs` | 238 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 770 | `steel-core/src/entity/entities/mobs/passive/cow/tests/mod.rs` | 19 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 771 | `steel-core/src/entity/entities/mobs/passive/cow/tests/persistence.rs` | 58 | 2 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 772 | `steel-core/src/entity/entities/mobs/passive/dolphin.rs` | 498 | 0 | 0 | 0 | 1 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 773 | `steel-core/src/entity/entities/mobs/passive/fish/cod.rs` | 286 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 774 | `steel-core/src/entity/entities/mobs/passive/fish/mod.rs` | 210 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 775 | `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs` | 793 | 0 | 0 | 0 | 0 | 0 | 7 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 776 | `steel-core/src/entity/entities/mobs/passive/fish/pufferfish.rs` | 478 | 0 | 0 | 1 | 0 | 0 | 5 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 777 | `steel-core/src/entity/entities/mobs/passive/fish/salmon.rs` | 398 | 2 | 0 | 0 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 778 | `steel-core/src/entity/entities/mobs/passive/fish/tadpole.rs` | 380 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 779 | `steel-core/src/entity/entities/mobs/passive/fish/tests.rs` | 99 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 780 | `steel-core/src/entity/entities/mobs/passive/fish/tropical_fish.rs` | 498 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 781 | `steel-core/src/entity/entities/mobs/passive/fox/mod.rs` | 589 | 0 | 0 | 2 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 782 | `steel-core/src/entity/entities/mobs/passive/frog.rs` | 418 | 1 | 1 | 0 | 0 | 2 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 783 | `steel-core/src/entity/entities/mobs/passive/goat.rs` | 305 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 784 | `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs` | 324 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 785 | `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs` | 516 | 1 | 0 | 2 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 786 | `steel-core/src/entity/entities/mobs/passive/horse/horse_entity.rs` | 573 | 0 | 0 | 3 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 787 | `steel-core/src/entity/entities/mobs/passive/horse/llama.rs` | 527 | 1 | 0 | 3 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 788 | `steel-core/src/entity/entities/mobs/passive/horse/mod.rs` | 17 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 789 | `steel-core/src/entity/entities/mobs/passive/horse/mule.rs` | 522 | 1 | 0 | 2 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 790 | `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs` | 374 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 791 | `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs` | 407 | 0 | 0 | 2 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 792 | `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs` | 362 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 793 | `steel-core/src/entity/entities/mobs/passive/mod.rs` | 73 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 794 | `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs` | 702 | 1 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 795 | `steel-core/src/entity/entities/mobs/passive/mooshroom/tests/mod.rs` | 264 | 1 | 0 | 0 | 1 | 7 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 796 | `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs` | 383 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 797 | `steel-core/src/entity/entities/mobs/passive/panda.rs` | 305 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 798 | `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs` | 505 | 0 | 0 | 0 | 0 | 1 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 799 | `steel-core/src/entity/entities/mobs/passive/pig/mod.rs` | 560 | 2 | 0 | 1 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 800 | `steel-core/src/entity/entities/mobs/passive/pig/tests/ai_age.rs` | 283 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 801 | `steel-core/src/entity/entities/mobs/passive/pig/tests/animal_lifecycle.rs` | 216 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 802 | `steel-core/src/entity/entities/mobs/passive/pig/tests/core.rs` | 442 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 803 | `steel-core/src/entity/entities/mobs/passive/pig/tests/mod.rs` | 65 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 804 | `steel-core/src/entity/entities/mobs/passive/pig/tests/persistence.rs` | 300 | 5 | 0 | 0 | 3 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 805 | `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs` | 446 | 0 | 0 | 0 | 0 | 2 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 806 | `steel-core/src/entity/entities/mobs/passive/sheep/color_spawn_rules.rs` | 86 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 807 | `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs` | 532 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 808 | `steel-core/src/entity/entities/mobs/passive/sheep/tests/core.rs` | 452 | 0 | 0 | 0 | 0 | 20 | 0 | 0 | 🟡 Требует внимания | Many unwrap/expect: 20 |
| 809 | `steel-core/src/entity/entities/mobs/passive/sheep/tests/mod.rs` | 19 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 810 | `steel-core/src/entity/entities/mobs/passive/sheep/tests/persistence.rs` | 36 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 811 | `steel-core/src/entity/entities/mobs/passive/sniffer.rs` | 305 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 812 | `steel-core/src/entity/entities/mobs/passive/squid/glow.rs` | 459 | 3 | 0 | 1 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 813 | `steel-core/src/entity/entities/mobs/passive/squid/mod.rs` | 512 | 1 | 0 | 5 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 814 | `steel-core/src/entity/entities/mobs/passive/squid/tests.rs` | 23 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 815 | `steel-core/src/entity/entities/mobs/passive/strider.rs` | 305 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 816 | `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs` | 319 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 817 | `steel-core/src/entity/entities/mobs/passive/turtle.rs` | 301 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 818 | `steel-core/src/entity/entities/mobs/passive/villager.rs` | 373 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 819 | `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs` | 473 | 0 | 0 | 1 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 820 | `steel-core/src/entity/entities/mobs/passive/wolf.rs` | 474 | 0 | 0 | 1 | 1 | 1 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 821 | `steel-core/src/entity/entities/mod.rs` | 42 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 822 | `steel-core/src/entity/entities/objects/decoration/armor_stand.rs` | 668 | 1 | 0 | 0 | 1 | 1 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 823 | `steel-core/src/entity/entities/objects/decoration/cushion.rs` | 657 | 4 | 0 | 15 | 3 | 3 | 1 | 0 | 🟡 Требует внимания | Many as casts: 15 |
| 824 | `steel-core/src/entity/entities/objects/decoration/display.rs` | 487 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 825 | `steel-core/src/entity/entities/objects/decoration/mannequin.rs` | 182 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 826 | `steel-core/src/entity/entities/objects/decoration/mod.rs` | 15 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 827 | `steel-core/src/entity/entities/objects/decoration/painting.rs` | 687 | 1 | 0 | 6 | 2 | 4 | 1 | 0 | 🟡 Требует внимания | Uses %2==0 instead of is_multiple_of |
| 828 | `steel-core/src/entity/entities/objects/display_ui/block_display.rs` | 129 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 829 | `steel-core/src/entity/entities/objects/display_ui/glow_item_frame.rs` | 866 | 4 | 0 | 6 | 1 | 2 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 830 | `steel-core/src/entity/entities/objects/display_ui/item_frame.rs` | 873 | 4 | 0 | 6 | 1 | 2 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 831 | `steel-core/src/entity/entities/objects/display_ui/leash_fence_knot.rs` | 392 | 0 | 0 | 6 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 832 | `steel-core/src/entity/entities/objects/display_ui/mod.rs` | 11 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 833 | `steel-core/src/entity/entities/objects/effects/area_effect_cloud.rs` | 495 | 6 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 834 | `steel-core/src/entity/entities/objects/effects/area_effect_cloud/tests.rs` | 183 | 0 | 0 | 1 | 3 | 2 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 835 | `steel-core/src/entity/entities/objects/effects/evoker_fangs.rs` | 163 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 836 | `steel-core/src/entity/entities/objects/effects/lightning_bolt.rs` | 286 | 0 | 0 | 0 | 1 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 837 | `steel-core/src/entity/entities/objects/effects/mod.rs` | 9 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 838 | `steel-core/src/entity/entities/objects/explosives/end_crystal.rs` | 230 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 839 | `steel-core/src/entity/entities/objects/explosives/mod.rs` | 7 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 840 | `steel-core/src/entity/entities/objects/explosives/primed_tnt.rs` | 385 | 1 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 841 | `steel-core/src/entity/entities/objects/explosives/primed_tnt/tests.rs` | 237 | 0 | 0 | 1 | 0 | 11 | 0 | 0 | 🟡 Требует внимания | Many unwrap/expect: 11 |
| 842 | `steel-core/src/entity/entities/objects/items/experience_orb.rs` | 683 | 0 | 0 | 2 | 1 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 843 | `steel-core/src/entity/entities/objects/items/falling_block.rs` | 568 | 4 | 0 | 7 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 844 | `steel-core/src/entity/entities/objects/items/falling_block/tests.rs` | 187 | 0 | 0 | 0 | 1 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 845 | `steel-core/src/entity/entities/objects/items/item_entity.rs` | 1055 | 2 | 0 | 8 | 0 | 0 | 1 | 0 | 🟡 Требует внимания | Uses %2==0 instead of is_multiple_of |
| 846 | `steel-core/src/entity/entities/objects/items/mod.rs` | 9 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 847 | `steel-core/src/entity/entities/objects/mod.rs` | 10 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 848 | `steel-core/src/entity/entities/objects/projectiles/arrow.rs` | 1128 | 3 | 0 | 3 | 3 | 1 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 849 | `steel-core/src/entity/entities/objects/projectiles/breeze_wind_charge.rs` | 223 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 850 | `steel-core/src/entity/entities/objects/projectiles/dragon_fireball.rs` | 210 | 0 | 0 | 1 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 851 | `steel-core/src/entity/entities/objects/projectiles/ender_pearl.rs` | 543 | 1 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 852 | `steel-core/src/entity/entities/objects/projectiles/experience_bottle.rs` | 276 | 1 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 853 | `steel-core/src/entity/entities/objects/projectiles/eye_of_ender.rs` | 291 | 1 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 854 | `steel-core/src/entity/entities/objects/projectiles/firework_rocket.rs` | 789 | 2 | 0 | 2 | 2 | 8 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 855 | `steel-core/src/entity/entities/objects/projectiles/fishing_hook.rs` | 1266 | 4 | 0 | 9 | 0 | 3 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 856 | `steel-core/src/entity/entities/objects/projectiles/large_fireball.rs` | 156 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 857 | `steel-core/src/entity/entities/objects/projectiles/lingering_potion.rs` | 203 | 1 | 0 | 1 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 858 | `steel-core/src/entity/entities/objects/projectiles/llama_spit.rs` | 155 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 859 | `steel-core/src/entity/entities/objects/projectiles/mod.rs` | 42 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 860 | `steel-core/src/entity/entities/objects/projectiles/shulker_bullet.rs` | 188 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 861 | `steel-core/src/entity/entities/objects/projectiles/small_fireball.rs` | 247 | 1 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 862 | `steel-core/src/entity/entities/objects/projectiles/snowball.rs` | 270 | 1 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 863 | `steel-core/src/entity/entities/objects/projectiles/splash_potion.rs` | 302 | 1 | 0 | 1 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 864 | `steel-core/src/entity/entities/objects/projectiles/thrown_egg.rs` | 462 | 1 | 0 | 0 | 0 | 2 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 865 | `steel-core/src/entity/entities/objects/projectiles/wind_charge.rs` | 223 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 866 | `steel-core/src/entity/entities/objects/projectiles/wither_skull.rs` | 394 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 867 | `steel-core/src/entity/entities/objects/technical/interaction.rs` | 536 | 0 | 0 | 0 | 1 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 868 | `steel-core/src/entity/entities/objects/technical/marker.rs` | 158 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 869 | `steel-core/src/entity/entities/objects/technical/mod.rs` | 9 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 870 | `steel-core/src/entity/entities/objects/technical/ominous_item_spawner.rs` | 182 | 1 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 871 | `steel-core/src/entity/entities/objects/vehicles/abstract_minecart.rs` | 637 | 0 | 0 | 5 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 872 | `steel-core/src/entity/entities/objects/vehicles/boat.rs` | 1242 | 0 | 0 | 28 | 1 | 0 | 2 | 0 | 🟡 Требует внимания | Many as casts: 28 |
| 873 | `steel-core/src/entity/entities/objects/vehicles/chest_boat.rs` | 324 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 874 | `steel-core/src/entity/entities/objects/vehicles/chest_minecart.rs` | 317 | 1 | 0 | 0 | 1 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 875 | `steel-core/src/entity/entities/objects/vehicles/command_block_minecart.rs` | 365 | 4 | 0 | 0 | 1 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 876 | `steel-core/src/entity/entities/objects/vehicles/furnace_minecart.rs` | 365 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 877 | `steel-core/src/entity/entities/objects/vehicles/hopper_minecart.rs` | 470 | 1 | 0 | 0 | 2 | 1 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 878 | `steel-core/src/entity/entities/objects/vehicles/minecart.rs` | 286 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 879 | `steel-core/src/entity/entities/objects/vehicles/minecart_behavior.rs` | 505 | 0 | 0 | 9 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 880 | `steel-core/src/entity/entities/objects/vehicles/mod.rs` | 34 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 881 | `steel-core/src/entity/entities/objects/vehicles/spawner_minecart.rs` | 212 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 882 | `steel-core/src/entity/entities/objects/vehicles/tests/abstract_minecart.rs` | 226 | 0 | 0 | 0 | 1 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 883 | `steel-core/src/entity/entities/objects/vehicles/tests/minecart.rs` | 47 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 884 | `steel-core/src/entity/entities/objects/vehicles/tests/minecart_behavior.rs` | 172 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 885 | `steel-core/src/entity/entities/objects/vehicles/tests/support.rs` | 51 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 886 | `steel-core/src/entity/entities/objects/vehicles/tests/vehicle_entity.rs` | 94 | 1 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 887 | `steel-core/src/entity/entities/objects/vehicles/tnt_minecart.rs` | 315 | 0 | 0 | 1 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 888 | `steel-core/src/entity/entities/objects/vehicles/vehicle_entity.rs` | 117 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 889 | `steel-core/src/entity/entity/mod.rs` | 4339 | 10 | 0 | 25 | 2 | 0 | 4 | 0 | 🟡 Требует внимания | Many as casts: 25 |
| 890 | `steel-core/src/entity/entity/tests.rs` | 63 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 891 | `steel-core/src/entity/entity_selector.rs` | 11 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 892 | `steel-core/src/entity/fluid_contact.rs` | 537 | 0 | 0 | 8 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 893 | `steel-core/src/entity/identity.rs` | 23 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 894 | `steel-core/src/entity/inside_block_effects.rs` | 271 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 895 | `steel-core/src/entity/item_based_steering.rs` | 170 | 0 | 0 | 2 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 896 | `steel-core/src/entity/item_frame.rs` | 15 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 897 | `steel-core/src/entity/leash.rs` | 545 | 2 | 0 | 2 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 898 | `steel-core/src/entity/living_base/mod.rs` | 1962 | 13 | 0 | 7 | 3 | 0 | 9 | 0 | 🟡 Требует внимания | High clone/to_owned: 13 |
| 899 | `steel-core/src/entity/living_base/tests.rs` | 674 | 0 | 0 | 2 | 0 | 7 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 900 | `steel-core/src/entity/living_entity.rs` | 3875 | 4 | 1 | 31 | 1 | 2 | 13 | 0 | 🟡 Требует внимания | Many as casts: 31; Uses %2==0 instead of is_multiple_of |
| 901 | `steel-core/src/entity/living_reference.rs` | 31 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 902 | `steel-core/src/entity/manager/benchmark_support.rs` | 81 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 903 | `steel-core/src/entity/manager/entries.rs` | 85 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 904 | `steel-core/src/entity/manager/mod.rs` | 2536 | 0 | 0 | 4 | 7 | 0 | 5 | 3 | ✅ Переписан (lightest-first, budget, modern) | hysteresis, Vec<DeferredWork>, path.contains, FxBuildHasher, is_multiple_of |
| 905 | `steel-core/src/entity/manager/spatial_query.rs` | 181 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 906 | `steel-core/src/entity/manager/tests/lifecycle.rs` | 117 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 907 | `steel-core/src/entity/manager/tests/mod.rs` | 289 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 908 | `steel-core/src/entity/manager/tests/movement_and_visibility.rs` | 597 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 909 | `steel-core/src/entity/manager/tests/persistence.rs` | 315 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 910 | `steel-core/src/entity/manager/tests/queries_and_order.rs` | 677 | 0 | 0 | 1 | 3 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 911 | `steel-core/src/entity/manager/tests/spatial_queries.rs` | 184 | 0 | 0 | 0 | 2 | 13 | 0 | 0 | 🟡 Требует внимания | Many unwrap/expect: 13 |
| 912 | `steel-core/src/entity/manager/tests/ticking.rs` | 410 | 0 | 0 | 0 | 2 | 0 | 10 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 913 | `steel-core/src/entity/mob/mod.rs` | 2263 | 3 | 0 | 15 | 0 | 0 | 11 | 0 | 🟡 Требует внимания | Many as casts: 15 |
| 914 | `steel-core/src/entity/mob/pathfinder.rs` | 463 | 0 | 0 | 1 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 915 | `steel-core/src/entity/mob/tests.rs` | 1693 | 2 | 0 | 1 | 1 | 9 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 916 | `steel-core/src/entity/mob_effect/absorption.rs` | 29 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 917 | `steel-core/src/entity/mob_effect/bad_omen.rs` | 49 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 918 | `steel-core/src/entity/mob_effect/heal_or_harm.rs` | 84 | 0 | 0 | 10 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 919 | `steel-core/src/entity/mob_effect/hunger.rs` | 24 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 920 | `steel-core/src/entity/mob_effect/infested.rs` | 48 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 921 | `steel-core/src/entity/mob_effect/mod.rs` | 108 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 922 | `steel-core/src/entity/mob_effect/oozing.rs` | 45 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 923 | `steel-core/src/entity/mob_effect/poison.rs` | 32 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 924 | `steel-core/src/entity/mob_effect/raid_omen.rs` | 136 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 925 | `steel-core/src/entity/mob_effect/regeneration.rs` | 24 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 926 | `steel-core/src/entity/mob_effect/saturation.rs` | 25 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 927 | `steel-core/src/entity/mob_effect/weaving.rs` | 38 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 928 | `steel-core/src/entity/mob_effect/wind_charged.rs` | 40 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 929 | `steel-core/src/entity/mob_effect/wither.rs` | 29 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 930 | `steel-core/src/entity/mod.rs` | 1495 | 1 | 1 | 10 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 931 | `steel-core/src/entity/monster.rs` | 216 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 932 | `steel-core/src/entity/monster/tests.rs` | 780 | 0 | 0 | 0 | 1 | 8 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 933 | `steel-core/src/entity/mount.rs` | 225 | 4 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 934 | `steel-core/src/entity/movement_sync.rs` | 1399 | 0 | 0 | 0 | 4 | 18 | 0 | 0 | 🟡 Требует внимания | Many unwrap/expect: 18 |
| 935 | `steel-core/src/entity/natural_spawner.rs` | 418 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 936 | `steel-core/src/entity/patrol_spawner.rs` | 300 | 0 | 0 | 4 | 1 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 937 | `steel-core/src/entity/phantom_spawner.rs` | 233 | 0 | 0 | 7 | 1 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 938 | `steel-core/src/entity/potion_contents.rs` | 154 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 939 | `steel-core/src/entity/projectile/mod.rs` | 1302 | 6 | 0 | 5 | 1 | 5 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 940 | `steel-core/src/entity/projectile/owner_tests.rs` | 215 | 2 | 0 | 0 | 1 | 20 | 0 | 0 | 🟡 Требует внимания | Many unwrap/expect: 20 |
| 941 | `steel-core/src/entity/projectile/throwable.rs` | 243 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 942 | `steel-core/src/entity/projectile/throwable_item.rs` | 52 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 943 | `steel-core/src/entity/projectile/thrown_potion.rs` | 214 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 944 | `steel-core/src/entity/pushing/impulse.rs` | 33 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 945 | `steel-core/src/entity/pushing/mod.rs` | 151 | 0 | 0 | 2 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 946 | `steel-core/src/entity/pushing/snapshot.rs` | 60 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 947 | `steel-core/src/entity/pushing/test_fixtures.rs` | 329 | 0 | 0 | 2 | 1 | 11 | 0 | 0 | 🟡 Требует внимания | Many unwrap/expect: 11 |
| 948 | `steel-core/src/entity/pushing/tests.rs` | 286 | 0 | 0 | 2 | 2 | 8 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 949 | `steel-core/src/entity/ranged_attack.rs` | 156 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 950 | `steel-core/src/entity/reference.rs` | 98 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 951 | `steel-core/src/entity/registry.rs` | 451 | 0 | 0 | 0 | 3 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 952 | `steel-core/src/entity/selector.rs` | 34 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 953 | `steel-core/src/entity/spawn.rs` | 832 | 5 | 0 | 0 | 3 | 9 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 954 | `steel-core/src/entity/spawn_placements.rs` | 52 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 955 | `steel-core/src/entity/spawn_util.rs` | 116 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 956 | `steel-core/src/entity/storage.rs` | 256 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 957 | `steel-core/src/entity/synced_data.rs` | 415 | 0 | 0 | 0 | 0 | 4 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 958 | `steel-core/src/entity/tests/damage.rs` | 383 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 959 | `steel-core/src/entity/tests/death_loot.rs` | 119 | 1 | 0 | 1 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 960 | `steel-core/src/entity/tests/equipment_and_freezing.rs` | 357 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 961 | `steel-core/src/entity/tests/fall_and_fluids.rs` | 354 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 962 | `steel-core/src/entity/tests/living_state.rs` | 231 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 963 | `steel-core/src/entity/tests/mod.rs` | 1281 | 16 | 0 | 0 | 6 | 4 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 16 |
| 964 | `steel-core/src/entity/tests/movement.rs` | 275 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 965 | `steel-core/src/entity/tests/portals.rs` | 237 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 966 | `steel-core/src/entity/tests/riding_and_leashes.rs` | 403 | 0 | 0 | 0 | 0 | 12 | 2 | 0 | 🟡 Требует внимания | Many unwrap/expect: 12 |
| 967 | `steel-core/src/entity/tests/travel.rs` | 223 | 0 | 0 | 3 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 968 | `steel-core/src/entity/ticking.rs` | 76 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 969 | `steel-core/src/entity/tracker/mod.rs` | 983 | 0 | 0 | 1 | 22 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 970 | `steel-core/src/entity/tracker/tests/mod.rs` | 201 | 2 | 0 | 0 | 5 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 971 | `steel-core/src/entity/tracker/tests/packet_sync.rs` | 271 | 1 | 0 | 0 | 28 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 972 | `steel-core/src/entity/tracker/tests/passenger_leash.rs` | 315 | 0 | 0 | 0 | 26 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 973 | `steel-core/src/entity/tracker/tests/spawn_despawn.rs` | 63 | 1 | 0 | 0 | 5 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 974 | `steel-core/src/entity/tracker/tests/visibility_range.rs` | 85 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 975 | `steel-core/src/entity/villager_trades.rs` | 402 | 4 | 0 | 0 | 3 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 976 | `steel-core/src/entity/warden_spawn_tracker.rs` | 260 | 0 | 0 | 0 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 977 | `steel-core/src/fluid/collision.rs` | 192 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 978 | `steel-core/src/fluid/conversion.rs` | 422 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 979 | `steel-core/src/fluid/flowing_fluid.rs` | 224 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 980 | `steel-core/src/fluid/fluid_behavior.rs` | 146 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 981 | `steel-core/src/fluid/fluids/empty.rs` | 80 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 982 | `steel-core/src/fluid/fluids/lava.rs` | 261 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 983 | `steel-core/src/fluid/fluids/mod.rs` | 8 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 984 | `steel-core/src/fluid/fluids/water.rs` | 124 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 985 | `steel-core/src/fluid/mod.rs` | 34 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 986 | `steel-core/src/fluid/spread_context.rs` | 95 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 987 | `steel-core/src/fluid/state.rs` | 406 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 988 | `steel-core/src/inventory/click.rs` | 484 | 0 | 0 | 4 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 989 | `steel-core/src/inventory/container/crafting.rs` | 103 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 990 | `steel-core/src/inventory/container/helper.rs` | 89 | 0 | 0 | 0 | 1 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 991 | `steel-core/src/inventory/container/mod.rs` | 620 | 0 | 0 | 4 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 992 | `steel-core/src/inventory/container/result.rs` | 75 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 993 | `steel-core/src/inventory/container/simple.rs` | 55 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 994 | `steel-core/src/inventory/ender_chest.rs` | 84 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 995 | `steel-core/src/inventory/equipment.rs` | 95 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 996 | `steel-core/src/inventory/fuel_values.rs` | 179 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 997 | `steel-core/src/inventory/lock.rs` | 634 | 6 | 0 | 1 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 998 | `steel-core/src/inventory/menu/behavior.rs` | 1180 | 27 | 0 | 9 | 4 | 5 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 27 |
| 999 | `steel-core/src/inventory/menu/builder.rs` | 1288 | 13 | 0 | 0 | 7 | 2 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 13 |
| 1000 | `steel-core/src/inventory/menu/grid.rs` | 1522 | 17 | 0 | 0 | 4 | 7 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 17 |
| 1001 | `steel-core/src/inventory/menu/kind.rs` | 187 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1002 | `steel-core/src/inventory/menu/kinds/anvil_menu.rs` | 418 | 4 | 0 | 6 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1003 | `steel-core/src/inventory/menu/kinds/anvil_menu/tests.rs` | 335 | 8 | 1 | 0 | 1 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1004 | `steel-core/src/inventory/menu/kinds/basic_menu.rs` | 14 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1005 | `steel-core/src/inventory/menu/kinds/beacon_menu.rs` | 512 | 4 | 0 | 0 | 0 | 8 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1006 | `steel-core/src/inventory/menu/kinds/chest_boat_menu.rs` | 64 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1007 | `steel-core/src/inventory/menu/kinds/chest_menu.rs` | 249 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1008 | `steel-core/src/inventory/menu/kinds/container_menus.rs` | 225 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1009 | `steel-core/src/inventory/menu/kinds/crafter_menu.rs` | 224 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1010 | `steel-core/src/inventory/menu/kinds/crafting_menu.rs` | 179 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1011 | `steel-core/src/inventory/menu/kinds/crafting_menu/tests.rs` | 93 | 1 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1012 | `steel-core/src/inventory/menu/kinds/enchantment_menu.rs` | 418 | 9 | 0 | 10 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1013 | `steel-core/src/inventory/menu/kinds/enchantment_menu/tests.rs` | 279 | 2 | 0 | 3 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1014 | `steel-core/src/inventory/menu/kinds/ender_chest_menu.rs` | 50 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1015 | `steel-core/src/inventory/menu/kinds/furnace_menu.rs` | 339 | 13 | 0 | 0 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 13 |
| 1016 | `steel-core/src/inventory/menu/kinds/grindstone_menu.rs` | 331 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1017 | `steel-core/src/inventory/menu/kinds/inventory_menu.rs` | 406 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1018 | `steel-core/src/inventory/menu/kinds/inventory_menu/tests.rs` | 85 | 1 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1019 | `steel-core/src/inventory/menu/kinds/merchant_menu.rs` | 371 | 8 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1020 | `steel-core/src/inventory/menu/kinds/mod.rs` | 40 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1021 | `steel-core/src/inventory/menu/kinds/mount_menu.rs` | 232 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1022 | `steel-core/src/inventory/menu/kinds/shulker_box_menu.rs` | 53 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1023 | `steel-core/src/inventory/menu/kinds/station_menu.rs` | 1067 | 8 | 0 | 1 | 4 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1024 | `steel-core/src/inventory/menu/layout.rs` | 124 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1025 | `steel-core/src/inventory/menu/mod.rs` | 706 | 10 | 0 | 4 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1026 | `steel-core/src/inventory/menu/tests.rs` | 368 | 2 | 0 | 0 | 0 | 10 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1027 | `steel-core/src/inventory/mod.rs` | 16 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1028 | `steel-core/src/inventory/openers_counter.rs` | 787 | 0 | 0 | 0 | 1 | 19 | 0 | 0 | 🟡 Требует внимания | Many unwrap/expect: 19 |
| 1029 | `steel-core/src/inventory/prelude.rs` | 17 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1030 | `steel-core/src/inventory/recipe_manager.rs` | 459 | 4 | 0 | 0 | 5 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1031 | `steel-core/src/inventory/slots/anvil_slots.rs` | 215 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1032 | `steel-core/src/inventory/slots/armor_slot.rs` | 86 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1033 | `steel-core/src/inventory/slots/crafting_slots.rs` | 237 | 5 | 0 | 0 | 1 | 5 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1034 | `steel-core/src/inventory/slots/enchantment_slots.rs` | 56 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1035 | `steel-core/src/inventory/slots/furnace_slots.rs` | 200 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1036 | `steel-core/src/inventory/slots/grindstone_slots.rs` | 127 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1037 | `steel-core/src/inventory/slots/mod.rs` | 25 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1038 | `steel-core/src/inventory/slots/normal_slot.rs` | 110 | 1 | 0 | 0 | 0 | 4 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1039 | `steel-core/src/inventory/slots/restricted_slot.rs` | 216 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1040 | `steel-core/src/inventory/slots/result_handler.rs` | 61 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1041 | `steel-core/src/inventory/slots/result_slot.rs` | 243 | 2 | 0 | 0 | 1 | 5 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1042 | `steel-core/src/inventory/slots/slot.rs` | 361 | 3 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1043 | `steel-core/src/java_block_pos_set/mod.rs` | 257 | 0 | 0 | 17 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 17 |
| 1044 | `steel-core/src/java_block_pos_set/removal.rs` | 164 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1045 | `steel-core/src/java_block_pos_set/tests.rs` | 223 | 0 | 2 | 3 | 0 | 5 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1046 | `steel-core/src/java_block_pos_set/tree.rs` | 274 | 0 | 0 | 16 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 16 |
| 1047 | `steel-core/src/java_block_pos_set/worldgen_tests.rs` | 145 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1048 | `steel-core/src/level_data/game_time.rs` | 40 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1049 | `steel-core/src/level_data/game_time_tests.rs` | 183 | 1 | 0 | 0 | 0 | 24 | 0 | 0 | 🟡 Требует внимания | Many unwrap/expect: 24 |
| 1050 | `steel-core/src/level_data/mod.rs` | 1108 | 14 | 8 | 0 | 1 | 18 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 14; Many format!: 8 |
| 1051 | `steel-core/src/lib.rs` | 44 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1052 | `steel-core/src/loot/mod.rs` | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1053 | `steel-core/src/loot/resolvers.rs` | 90 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1054 | `steel-core/src/permission/context.rs` | 587 | 10 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1055 | `steel-core/src/permission/expression.rs` | 79 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1056 | `steel-core/src/permission/groups.rs` | 463 | 21 | 0 | 0 | 1 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 21 |
| 1057 | `steel-core/src/permission/key.rs` | 184 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1058 | `steel-core/src/permission/manager.rs` | 296 | 8 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1059 | `steel-core/src/permission/metadata.rs` | 616 | 6 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1060 | `steel-core/src/permission/mod.rs` | 41 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1061 | `steel-core/src/permission/rule_expression.rs` | 351 | 12 | 0 | 0 | 1 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 12 |
| 1062 | `steel-core/src/permission/set.rs` | 602 | 3 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1063 | `steel-core/src/permission/subject.rs` | 120 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1064 | `steel-core/src/permission/tests/groups.rs` | 301 | 42 | 0 | 0 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 42 |
| 1065 | `steel-core/src/permission/tests/manager.rs` | 235 | 7 | 0 | 0 | 4 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1066 | `steel-core/src/permission/tests/mod.rs` | 442 | 40 | 0 | 0 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 40 |
| 1067 | `steel-core/src/physics/collision.rs` | 1491 | 1 | 0 | 6 | 25 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1068 | `steel-core/src/physics/entity_move.rs` | 948 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1069 | `steel-core/src/physics/mod.rs` | 66 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1070 | `steel-core/src/physics/movement_validation.rs` | 291 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1071 | `steel-core/src/physics/physics_state.rs` | 175 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1072 | `steel-core/src/physics/shapes.rs` | 466 | 0 | 0 | 16 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 16 |
| 1073 | `steel-core/src/player/abilities.rs` | 192 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1074 | `steel-core/src/player/advancement/mod.rs` | 496 | 9 | 0 | 1 | 4 | 5 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1075 | `steel-core/src/player/advancement/predicate.rs` | 1409 | 15 | 0 | 3 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 15 |
| 1076 | `steel-core/src/player/advancement/progress.rs` | 226 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1077 | `steel-core/src/player/advancement/rewards.rs` | 52 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1078 | `steel-core/src/player/advancement/saving.rs` | 46 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1079 | `steel-core/src/player/advancement/visibility_evaluator.rs` | 97 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1080 | `steel-core/src/player/chat/message_chain.rs` | 386 | 5 | 0 | 3 | 0 | 6 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1081 | `steel-core/src/player/chat/message_validator.rs` | 242 | 3 | 4 | 1 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1082 | `steel-core/src/player/chat/mod.rs` | 632 | 21 | 5 | 1 | 0 | 1 | 1 | 0 | 🟡 Требует внимания | High clone/to_owned: 21 |
| 1083 | `steel-core/src/player/chat/profile_key.rs` | 463 | 4 | 0 | 5 | 1 | 10 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1084 | `steel-core/src/player/chat/signature_cache.rs` | 272 | 4 | 0 | 1 | 2 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1085 | `steel-core/src/player/chunk_sender.rs` | 559 | 4 | 0 | 5 | 5 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1086 | `steel-core/src/player/chunk_sender/tests.rs` | 609 | 0 | 0 | 9 | 4 | 7 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1087 | `steel-core/src/player/connection/java.rs` | 1781 | 5 | 0 | 7 | 7 | 4 | 2 | 0 | 🟡 Требует внимания | Vec::new without capacity in tick: 7 |
| 1088 | `steel-core/src/player/connection/mod.rs` | 273 | 2 | 0 | 0 | 0 | 1 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1089 | `steel-core/src/player/connection/packet_budget.rs` | 417 | 0 | 0 | 3 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1090 | `steel-core/src/player/container_counter.rs` | 39 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1091 | `steel-core/src/player/entity_state.rs` | 450 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1092 | `steel-core/src/player/experience.rs` | 341 | 0 | 0 | 17 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 17 |
| 1093 | `steel-core/src/player/food_data.rs` | 493 | 0 | 0 | 2 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1094 | `steel-core/src/player/game_mode/adventure_mode.rs` | 184 | 3 | 0 | 0 | 1 | 5 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1095 | `steel-core/src/player/game_mode/block_breaking.rs` | 702 | 1 | 0 | 7 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1096 | `steel-core/src/player/game_mode/block_interaction.rs` | 488 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1097 | `steel-core/src/player/game_mode/entity_interaction.rs` | 953 | 2 | 0 | 10 | 2 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1098 | `steel-core/src/player/game_mode/item_interaction.rs` | 449 | 3 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1099 | `steel-core/src/player/game_mode/item_interaction/use_on_tests.rs` | 1122 | 6 | 0 | 0 | 0 | 15 | 0 | 0 | 🟡 Требует внимания | Many unwrap/expect: 15 |
| 1100 | `steel-core/src/player/game_mode/mod.rs` | 115 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1101 | `steel-core/src/player/game_mode/player_methods.rs` | 357 | 1 | 0 | 1 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1102 | `steel-core/src/player/game_mode/raycast.rs` | 100 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1103 | `steel-core/src/player/game_mode/swing.rs` | 40 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1104 | `steel-core/src/player/game_mode/tests.rs` | 67 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1105 | `steel-core/src/player/health_sync.rs` | 60 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1106 | `steel-core/src/player/item_cooldowns.rs` | 176 | 7 | 0 | 0 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1107 | `steel-core/src/player/lifecycle/mod.rs` | 554 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1108 | `steel-core/src/player/lifecycle/respawn.rs` | 904 | 2 | 0 | 1 | 1 | 0 | 0 | 0 | 🔵 TODO — fallback ok | UnimplementedBlockEntity |
| 1109 | `steel-core/src/player/lifecycle/respawn_restore.rs` | 487 | 13 | 0 | 0 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 13 |
| 1110 | `steel-core/src/player/lifecycle/spawn_sync.rs` | 455 | 2 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1111 | `steel-core/src/player/lifecycle/world_transition.rs` | 125 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1112 | `steel-core/src/player/mod.rs` | 3719 | 34 | 4 | 11 | 4 | 0 | 12 | 0 | 🟡 Требует внимания | High clone/to_owned: 34; Many as casts: 11 |
| 1113 | `steel-core/src/player/movement/input.rs` | 170 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1114 | `steel-core/src/player/movement/mod.rs` | 1317 | 0 | 0 | 10 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1115 | `steel-core/src/player/movement/state.rs` | 386 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1116 | `steel-core/src/player/movement/teleport.rs` | 58 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1117 | `steel-core/src/player/permissions.rs` | 169 | 7 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1118 | `steel-core/src/player/player_data.rs` | 611 | 9 | 0 | 4 | 3 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1119 | `steel-core/src/player/player_data_storage/file_storage.rs` | 506 | 3 | 9 | 0 | 4 | 0 | 0 | 0 | 🟡 Требует внимания | Many format!: 9 |
| 1120 | `steel-core/src/player/player_data_storage/known_players.rs` | 76 | 2 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1121 | `steel-core/src/player/player_data_storage/memory_storage.rs` | 82 | 7 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1122 | `steel-core/src/player/player_data_storage/mod.rs` | 967 | 15 | 6 | 0 | 5 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 15; Many format!: 6 |
| 1123 | `steel-core/src/player/player_data_storage/permissions.rs` | 428 | 20 | 5 | 0 | 2 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 20 |
| 1124 | `steel-core/src/player/player_data_storage/stats.rs` | 94 | 3 | 1 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1125 | `steel-core/src/player/player_data_storage/tests.rs` | 1125 | 44 | 2 | 0 | 11 | 96 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 44; Many unwrap/expect: 96 |
| 1126 | `steel-core/src/player/player_inventory/container.rs` | 249 | 0 | 0 | 2 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1127 | `steel-core/src/player/player_inventory/core.rs` | 316 | 2 | 0 | 5 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1128 | `steel-core/src/player/player_inventory/equipment.rs` | 370 | 0 | 0 | 3 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1129 | `steel-core/src/player/player_inventory/mod.rs` | 200 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1130 | `steel-core/src/player/player_inventory/player_handlers.rs` | 1432 | 7 | 0 | 5 | 4 | 1 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1131 | `steel-core/src/player/player_inventory/tests.rs` | 2148 | 17 | 0 | 5 | 1 | 6 | 1 | 0 | 🟡 Требует внимания | High clone/to_owned: 17 |
| 1132 | `steel-core/src/player/post_effects.rs` | 161 | 1 | 0 | 0 | 1 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1133 | `steel-core/src/player/profile/known_players.rs` | 251 | 2 | 0 | 0 | 1 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1134 | `steel-core/src/player/profile/lookup.rs` | 163 | 7 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1135 | `steel-core/src/player/profile/mod.rs` | 52 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1136 | `steel-core/src/player/recipe_book.rs` | 540 | 9 | 0 | 0 | 3 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1137 | `steel-core/src/player/recipe_placement.rs` | 942 | 7 | 0 | 10 | 9 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1138 | `steel-core/src/player/session.rs` | 215 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1139 | `steel-core/src/player/sleep.rs` | 351 | 3 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1140 | `steel-core/src/player/sleep_state.rs` | 62 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1141 | `steel-core/src/player/spam_throttler.rs` | 59 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1142 | `steel-core/src/player/stats_counter.rs` | 414 | 3 | 0 | 1 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1143 | `steel-core/src/player/tests.rs` | 2439 | 25 | 0 | 1 | 14 | 17 | 1 | 0 | 🟡 Требует внимания | High clone/to_owned: 25; Vec::new without capacity in tick: 14 |
| 1144 | `steel-core/src/player/tick_state.rs` | 149 | 0 | 0 | 2 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1145 | `steel-core/src/player/title.rs` | 40 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1146 | `steel-core/src/poi/mod.rs` | 12 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1147 | `steel-core/src/poi/poi_instance.rs` | 63 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1148 | `steel-core/src/poi/poi_set.rs` | 207 | 1 | 0 | 0 | 1 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1149 | `steel-core/src/poi/poi_storage.rs` | 501 | 0 | 0 | 3 | 6 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1150 | `steel-core/src/portal/end_gateway.rs` | 333 | 0 | 0 | 7 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1151 | `steel-core/src/portal/end_portal.rs` | 165 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1152 | `steel-core/src/portal/mod.rs` | 671 | 1 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1153 | `steel-core/src/portal/nether_portal.rs` | 237 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1154 | `steel-core/src/portal/portal_shape.rs` | 477 | 0 | 0 | 9 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1155 | `steel-core/src/scoreboard.rs` | 1701 | 61 | 4 | 0 | 0 | 45 | 3 | 0 | 🟡 Требует внимания | High clone/to_owned: 61; Many unwrap/expect: 45 |
| 1156 | `steel-core/src/server/bans.rs` | 358 | 2 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1157 | `steel-core/src/server/boss_bars.rs` | 212 | 13 | 0 | 0 | 0 | 2 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 13 |
| 1158 | `steel-core/src/server/broadcasting.rs` | 473 | 10 | 6 | 2 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Many format!: 6 |
| 1159 | `steel-core/src/server/datapacks.rs` | 2113 | 114 | 29 | 3 | 17 | 7 | 2 | 0 | ✅ Переписан (lightest-first, budget, modern) | hysteresis, Vec<DeferredWork>, path.contains, FxBuildHasher, is_multiple_of |
| 1160 | `steel-core/src/server/jobs/domain_switch.rs` | 541 | 6 | 2 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1161 | `steel-core/src/server/jobs/mod.rs` | 392 | 0 | 0 | 0 | 1 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1162 | `steel-core/src/server/jobs/teleport.rs` | 1057 | 1 | 0 | 3 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1163 | `steel-core/src/server/known_players.rs` | 264 | 11 | 0 | 0 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 11 |
| 1164 | `steel-core/src/server/mod.rs` | 1471 | 28 | 35 | 2 | 7 | 0 | 0 | 6 | 🟡 Требует внимания | High clone/to_owned: 28; Many format!: 35 |
| 1165 | `steel-core/src/server/packet_processor.rs` | 2105 | 1 | 0 | 1 | 1 | 6 | 4 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1166 | `steel-core/src/server/permissions.rs` | 224 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1167 | `steel-core/src/server/player_admission.rs` | 772 | 3 | 1 | 1 | 5 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1168 | `steel-core/src/server/player_lifecycle.rs` | 641 | 15 | 6 | 4 | 1 | 1 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 15; Many format!: 6 |
| 1169 | `steel-core/src/server/pregen.rs` | 658 | 1 | 11 | 9 | 1 | 0 | 0 | 0 | 🟡 Требует внимания | Many format!: 11; Uses %2==0 instead of is_multiple_of |
| 1170 | `steel-core/src/server/rcon.rs` | 310 | 5 | 0 | 3 | 1 | 5 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1171 | `steel-core/src/server/registry_cache.rs` | 384 | 7 | 0 | 0 | 1 | 4 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1172 | `steel-core/src/server/run_loop.rs` | 939 | 14 | 0 | 1 | 1 | 0 | 7 | 5 | ✅ Переписан (lightest-first, budget, modern) | hysteresis, Vec<DeferredWork>, path.contains, FxBuildHasher, is_multiple_of |
| 1173 | `steel-core/src/server/schedule.rs` | 348 | 25 | 2 | 0 | 1 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 25 |
| 1174 | `steel-core/src/server/scoreboard_sync.rs` | 352 | 26 | 0 | 0 | 1 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 26 |
| 1175 | `steel-core/src/server/service_keys.rs` | 248 | 7 | 0 | 0 | 0 | 7 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1176 | `steel-core/src/server/tests.rs` | 4751 | 151 | 18 | 8 | 12 | 40 | 3 | 0 | 🟡 Требует внимания | High clone/to_owned: 151; Many format!: 18 |
| 1177 | `steel-core/src/server/tests/connection_lifecycle.rs` | 325 | 2 | 0 | 0 | 3 | 2 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1178 | `steel-core/src/server/tests/damage_history.rs` | 104 | 0 | 0 | 0 | 0 | 6 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1179 | `steel-core/src/server/tests/game_time.rs` | 446 | 12 | 0 | 0 | 0 | 41 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 12; Many unwrap/expect: 41 |
| 1180 | `steel-core/src/server/tests/player_limit.rs` | 234 | 11 | 0 | 1 | 1 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 11 |
| 1181 | `steel-core/src/server/tests/projectile_owners.rs` | 124 | 5 | 0 | 0 | 0 | 3 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1182 | `steel-core/src/server/tick_overload.rs` | 432 | 0 | 0 | 0 | 2 | 3 | 1 | 31 | ✅ Переписан (lightest-first, budget, modern) | hysteresis, Vec<DeferredWork>, path.contains, FxBuildHasher, is_multiple_of |
| 1183 | `steel-core/src/server/tick_rate_manager.rs` | 362 | 0 | 0 | 22 | 0 | 0 | 2 | 0 | 🟡 Требует внимания | Many as casts: 22 |
| 1184 | `steel-core/src/server/tps_bar.rs` | 209 | 2 | 3 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1185 | `steel-core/src/server/whitelist.rs` | 243 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1186 | `steel-core/src/server/world_changes.rs` | 632 | 23 | 7 | 0 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 23; Many format!: 7 |
| 1187 | `steel-core/src/server/world_tick_workers.rs` | 195 | 1 | 1 | 2 | 1 | 9 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1188 | `steel-core/src/server/worlds.rs` | 325 | 13 | 4 | 0 | 0 | 2 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 13 |
| 1189 | `steel-core/src/vanilla_26_3_mega_parity.rs` | 2757 | 29 | 0 | 0 | 5 | 82 | 1 | 0 | 🟡 Требует внимания | High clone/to_owned: 29; Many unwrap/expect: 82 |
| 1190 | `steel-core/src/world/block_entity_ticker.rs` | 514 | 0 | 0 | 0 | 5 | 0 | 3 | 6 | ✅ Переписан (lightest-first, budget, modern) | hysteresis, Vec<DeferredWork>, path.contains, FxBuildHasher, is_multiple_of |
| 1191 | `steel-core/src/world/block_event.rs` | 287 | 1 | 0 | 1 | 1 | 9 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1192 | `steel-core/src/world/block_region.rs` | 542 | 0 | 0 | 5 | 2 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1193 | `steel-core/src/world/block_updates/mod.rs` | 744 | 0 | 0 | 6 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1194 | `steel-core/src/world/block_updates/neighbor_updater.rs` | 364 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1195 | `steel-core/src/world/border.rs` | 813 | 1 | 0 | 4 | 0 | 13 | 3 | 0 | 🟡 Требует внимания | Many unwrap/expect: 13 |
| 1196 | `steel-core/src/world/broadcasts.rs` | 344 | 7 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1197 | `steel-core/src/world/clock.rs` | 477 | 5 | 3 | 4 | 1 | 2 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1198 | `steel-core/src/world/domain_entity_directory.rs` | 50 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1199 | `steel-core/src/world/dragon_fight.rs` | 924 | 4 | 0 | 4 | 3 | 0 | 3 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1200 | `steel-core/src/world/entity_management.rs` | 827 | 0 | 0 | 0 | 8 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1201 | `steel-core/src/world/environment.rs` | 620 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1202 | `steel-core/src/world/events.rs` | 153 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1203 | `steel-core/src/world/explosion/mod.rs` | 553 | 4 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1204 | `steel-core/src/world/explosion/server.rs` | 176 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1205 | `steel-core/src/world/explosion/server/block_effects.rs` | 168 | 1 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1206 | `steel-core/src/world/explosion/server/block_rays.rs` | 328 | 0 | 0 | 4 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1207 | `steel-core/src/world/explosion/server/block_rays/cache.rs` | 482 | 0 | 0 | 4 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1208 | `steel-core/src/world/explosion/server/block_rays/tests.rs` | 632 | 0 | 1 | 5 | 3 | 4 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1209 | `steel-core/src/world/explosion/server/block_rays/tests/gameplay.rs` | 230 | 0 | 0 | 0 | 2 | 5 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1210 | `steel-core/src/world/explosion/server/entity_effects.rs` | 131 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1211 | `steel-core/src/world/explosion/server/exposure.rs` | 223 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1212 | `steel-core/src/world/explosion/server/tests.rs` | 673 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1213 | `steel-core/src/world/game_event/context.rs` | 39 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1214 | `steel-core/src/world/game_event/listener.rs` | 1016 | 0 | 0 | 0 | 8 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1215 | `steel-core/src/world/game_event/mod.rs` | 8 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1216 | `steel-core/src/world/level_effects.rs` | 847 | 11 | 1 | 17 | 2 | 0 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 11; Many as casts: 17 |
| 1217 | `steel-core/src/world/level_reader.rs` | 358 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1218 | `steel-core/src/world/map_state.rs` | 1487 | 16 | 1 | 7 | 3 | 14 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 16; Many unwrap/expect: 14 |
| 1219 | `steel-core/src/world/mod.rs` | 1955 | 5 | 0 | 10 | 3 | 0 | 6 | 6 | ✅ Переписан (lightest-first, budget, modern) | hysteresis, Vec<DeferredWork>, path.contains, FxBuildHasher, is_multiple_of |
| 1220 | `steel-core/src/world/player_index/area.rs` | 289 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1221 | `steel-core/src/world/player_index/map.rs` | 257 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1222 | `steel-core/src/world/player_index/map/tests.rs` | 128 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1223 | `steel-core/src/world/player_index/mod.rs` | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1224 | `steel-core/src/world/player_spawn_finder/candidates.rs` | 67 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1225 | `steel-core/src/world/player_spawn_finder/chunk_load.rs` | 172 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1226 | `steel-core/src/world/player_spawn_finder/fallback.rs` | 152 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1227 | `steel-core/src/world/player_spawn_finder/mod.rs` | 433 | 2 | 0 | 7 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1228 | `steel-core/src/world/player_spawn_finder/tests.rs` | 435 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1229 | `steel-core/src/world/portals.rs` | 594 | 0 | 0 | 6 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1230 | `steel-core/src/world/properties.rs` | 311 | 1 | 0 | 0 | 0 | 1 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1231 | `steel-core/src/world/raid.rs` | 2294 | 14 | 2 | 24 | 6 | 51 | 1 | 0 | 🟡 Требует внимания | High clone/to_owned: 14; Vec::new without capacity in tick: 6 |
| 1232 | `steel-core/src/world/raycast.rs` | 1705 | 0 | 0 | 20 | 2 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 20 |
| 1233 | `steel-core/src/world/redstone.rs` | 99 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1234 | `steel-core/src/world/signal_getter.rs` | 364 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1235 | `steel-core/src/world/sleep.rs` | 134 | 2 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1236 | `steel-core/src/world/sleep_status.rs` | 72 | 0 | 0 | 3 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1237 | `steel-core/src/world/spawn.rs` | 214 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1238 | `steel-core/src/world/tests.rs` | 925 | 0 | 0 | 0 | 0 | 5 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1239 | `steel-core/src/world/tick_scheduler/list.rs` | 341 | 0 | 0 | 3 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1240 | `steel-core/src/world/tick_scheduler/mod.rs` | 415 | 0 | 0 | 3 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1241 | `steel-core/src/world/tick_scheduler/run_batch.rs` | 63 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1242 | `steel-core/src/world/tick_scheduler/scheduler.rs` | 743 | 1 | 0 | 0 | 5 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1243 | `steel-core/src/world/tick_scheduler/tests.rs` | 840 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1244 | `steel-core/src/world/tick_scheduler/world.rs` | 267 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1245 | `steel-core/src/world/wandering_trader_spawner.rs` | 339 | 0 | 0 | 2 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1246 | `steel-core/src/world/waypoint.rs` | 359 | 7 | 0 | 6 | 4 | 0 | 1 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1247 | `steel-core/src/world/waypoint/tests.rs` | 130 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1248 | `steel-core/src/world/weather.rs` | 619 | 0 | 0 | 6 | 0 | 0 | 2 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1249 | `steel-core/src/world/world_entities.rs` | 363 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1250 | `steel-core/src/worldgen/carver/canyon.rs` | 180 | 0 | 0 | 8 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1251 | `steel-core/src/worldgen/carver/cave.rs` | 262 | 0 | 0 | 3 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1252 | `steel-core/src/worldgen/carver/mask.rs` | 313 | 0 | 0 | 12 | 2 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 12 |
| 1253 | `steel-core/src/worldgen/carver/mod.rs` | 583 | 0 | 0 | 13 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 13 |
| 1254 | `steel-core/src/worldgen/chunk_stage_hashes.rs` | 1664 | 6 | 18 | 22 | 6 | 9 | 0 | 0 | 🟡 Требует внимания | Many format!: 18; Many as casts: 22 |
| 1255 | `steel-core/src/worldgen/feature/configured.rs` | 1330 | 0 | 0 | 3 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1256 | `steel-core/src/worldgen/feature/features/bamboo.rs` | 115 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1257 | `steel-core/src/worldgen/feature/features/basalt_columns.rs` | 177 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1258 | `steel-core/src/worldgen/feature/features/basalt_pillar.rs` | 104 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1259 | `steel-core/src/worldgen/feature/features/block_blob.rs` | 61 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1260 | `steel-core/src/worldgen/feature/features/block_column.rs` | 86 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1261 | `steel-core/src/worldgen/feature/features/block_entities.rs` | 122 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1262 | `steel-core/src/worldgen/feature/features/block_pile.rs` | 72 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1263 | `steel-core/src/worldgen/feature/features/blue_ice.rs` | 87 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1264 | `steel-core/src/worldgen/feature/features/bonus_chest.rs` | 65 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1265 | `steel-core/src/worldgen/feature/features/chorus_plant.rs` | 129 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1266 | `steel-core/src/worldgen/feature/features/coral.rs` | 242 | 0 | 0 | 3 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1267 | `steel-core/src/worldgen/feature/features/delta.rs` | 86 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1268 | `steel-core/src/worldgen/feature/features/desert_well.rs` | 163 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1269 | `steel-core/src/worldgen/feature/features/disk.rs` | 91 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1270 | `steel-core/src/worldgen/feature/features/dripstone/cluster.rs` | 416 | 0 | 0 | 7 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1271 | `steel-core/src/worldgen/feature/features/dripstone/large.rs` | 321 | 0 | 0 | 12 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 12 |
| 1272 | `steel-core/src/worldgen/feature/features/dripstone/mod.rs` | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1273 | `steel-core/src/worldgen/feature/features/dripstone/pointed.rs` | 191 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1274 | `steel-core/src/worldgen/feature/features/end_gateway.rs` | 104 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1275 | `steel-core/src/worldgen/feature/features/end_island.rs` | 33 | 0 | 0 | 5 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1276 | `steel-core/src/worldgen/feature/features/end_platform.rs` | 32 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1277 | `steel-core/src/worldgen/feature/features/end_podium.rs` | 144 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1278 | `steel-core/src/worldgen/feature/features/end_spike.rs` | 298 | 0 | 0 | 4 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1279 | `steel-core/src/worldgen/feature/features/fossil.rs` | 197 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1280 | `steel-core/src/worldgen/feature/features/freeze_top_layer.rs` | 49 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1281 | `steel-core/src/worldgen/feature/features/geode.rs` | 360 | 2 | 0 | 5 | 3 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1282 | `steel-core/src/worldgen/feature/features/glowstone_blob.rs` | 57 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1283 | `steel-core/src/worldgen/feature/features/huge_fungus.rs` | 321 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1284 | `steel-core/src/worldgen/feature/features/huge_mushroom.rs` | 341 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1285 | `steel-core/src/worldgen/feature/features/iceberg.rs` | 504 | 0 | 0 | 20 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 20 |
| 1286 | `steel-core/src/worldgen/feature/features/kelp.rs` | 59 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1287 | `steel-core/src/worldgen/feature/features/lake.rs` | 227 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1288 | `steel-core/src/worldgen/feature/features/mod.rs` | 58 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1289 | `steel-core/src/worldgen/feature/features/monster_room.rs` | 201 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1290 | `steel-core/src/worldgen/feature/features/multiface_growth.rs` | 477 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1291 | `steel-core/src/worldgen/feature/features/nether_forest_vegetation.rs` | 52 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1292 | `steel-core/src/worldgen/feature/features/ore.rs` | 758 | 0 | 0 | 13 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 13 |
| 1293 | `steel-core/src/worldgen/feature/features/projected_random_patchy_square.rs` | 52 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1294 | `steel-core/src/worldgen/feature/features/random_neighbor_spread.rs` | 64 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1295 | `steel-core/src/worldgen/feature/features/replace_blobs.rs` | 64 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1296 | `steel-core/src/worldgen/feature/features/sculk_patch.rs` | 1050 | 0 | 0 | 7 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1297 | `steel-core/src/worldgen/feature/features/sea_pickle.rs` | 36 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1298 | `steel-core/src/worldgen/feature/features/seagrass.rs` | 50 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1299 | `steel-core/src/worldgen/feature/features/simple_block.rs` | 177 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1300 | `steel-core/src/worldgen/feature/features/single_block_pillar.rs` | 47 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1301 | `steel-core/src/worldgen/feature/features/speleothem.rs` | 565 | 0 | 0 | 10 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1302 | `steel-core/src/worldgen/feature/features/spike.rs` | 169 | 0 | 0 | 6 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1303 | `steel-core/src/worldgen/feature/features/spring.rs` | 86 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1304 | `steel-core/src/worldgen/feature/features/stepped_column_cluster.rs` | 169 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1305 | `steel-core/src/worldgen/feature/features/tree/decorators.rs` | 960 | 0 | 0 | 2 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1306 | `steel-core/src/worldgen/feature/features/tree/fallen.rs` | 185 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1307 | `steel-core/src/worldgen/feature/features/tree/foliage.rs` | 1371 | 0 | 0 | 3 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1308 | `steel-core/src/worldgen/feature/features/tree/leaves.rs` | 253 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1309 | `steel-core/src/worldgen/feature/features/tree/mod.rs` | 597 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1310 | `steel-core/src/worldgen/feature/features/tree/root_system.rs` | 234 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1311 | `steel-core/src/worldgen/feature/features/tree/roots.rs` | 245 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1312 | `steel-core/src/worldgen/feature/features/tree/trunk.rs` | 1319 | 0 | 0 | 19 | 5 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 19 |
| 1313 | `steel-core/src/worldgen/feature/features/twisting_vines.rs` | 101 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1314 | `steel-core/src/worldgen/feature/features/underwater_magma.rs` | 98 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1315 | `steel-core/src/worldgen/feature/features/vegetation_patch.rs` | 257 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1316 | `steel-core/src/worldgen/feature/features/vines.rs` | 44 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1317 | `steel-core/src/worldgen/feature/features/void_start_platform.rs` | 85 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1318 | `steel-core/src/worldgen/feature/features/weeping_vines.rs` | 128 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1319 | `steel-core/src/worldgen/feature/instrumentation.rs` | 403 | 0 | 1 | 6 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1320 | `steel-core/src/worldgen/feature/mod.rs` | 29 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1321 | `steel-core/src/worldgen/feature/placed.rs` | 580 | 1 | 0 | 3 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1322 | `steel-core/src/worldgen/feature/placement.rs` | 140 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1323 | `steel-core/src/worldgen/feature/predicates.rs` | 41 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1324 | `steel-core/src/worldgen/feature/prelude.rs` | 58 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1325 | `steel-core/src/worldgen/feature/runner.rs` | 469 | 0 | 0 | 5 | 4 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1326 | `steel-core/src/worldgen/feature/sorter.rs` | 260 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1327 | `steel-core/src/worldgen/feature/state.rs` | 113 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1328 | `steel-core/src/worldgen/feature/tests.rs` | 383 | 3 | 0 | 0 | 3 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1329 | `steel-core/src/worldgen/feature/weather.rs` | 148 | 0 | 0 | 4 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1330 | `steel-core/src/worldgen/generator/context.rs` | 132 | 0 | 0 | 1 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1331 | `steel-core/src/worldgen/generator/empty.rs` | 65 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1332 | `steel-core/src/worldgen/generator/flat.rs` | 375 | 1 | 0 | 3 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1333 | `steel-core/src/worldgen/generator/generation_chunk.rs` | 360 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1334 | `steel-core/src/worldgen/generator/mod.rs` | 130 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1335 | `steel-core/src/worldgen/generator/registry.rs` | 696 | 34 | 11 | 5 | 1 | 9 | 0 | 0 | 🟡 Требует внимания | High clone/to_owned: 34; Many format!: 11 |
| 1336 | `steel-core/src/worldgen/generator/surface_biomes.rs` | 370 | 0 | 0 | 10 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1337 | `steel-core/src/worldgen/generator/vanilla.rs` | 1395 | 1 | 0 | 26 | 5 | 2 | 0 | 0 | 🟡 Требует внимания | Many as casts: 26 |
| 1338 | `steel-core/src/worldgen/mod.rs` | 45 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1339 | `steel-core/src/worldgen/region.rs` | 1876 | 0 | 0 | 16 | 1 | 0 | 0 | 0 | 🟡 Требует внимания | Many as casts: 16 |
| 1340 | `steel-core/src/worldgen/stages/biomes.rs` | 21 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1341 | `steel-core/src/worldgen/stages/empty.rs` | 33 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1342 | `steel-core/src/worldgen/stages/features.rs` | 30 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1343 | `steel-core/src/worldgen/stages/full.rs` | 15 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1344 | `steel-core/src/worldgen/stages/leaf_distance.rs` | 494 | 0 | 0 | 6 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1345 | `steel-core/src/worldgen/stages/light.rs` | 470 | 0 | 0 | 0 | 2 | 1 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1346 | `steel-core/src/worldgen/stages/mod.rs` | 11 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1347 | `steel-core/src/worldgen/stages/spawn.rs` | 296 | 0 | 0 | 4 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1348 | `steel-core/src/worldgen/stages/structures.rs` | 98 | 1 | 0 | 0 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1349 | `steel-core/src/worldgen/stages/terrain.rs` | 126 | 1 | 0 | 4 | 0 | 2 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1350 | `steel-core/src/worldgen/structure/mod.rs` | 37 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1351 | `steel-core/src/worldgen/structure/piece_placer/buried_treasure.rs` | 115 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1352 | `steel-core/src/worldgen/structure/piece_placer/desert_pyramid.rs` | 734 | 0 | 0 | 4 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1353 | `steel-core/src/worldgen/structure/piece_placer/fortress.rs` | 812 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1354 | `steel-core/src/worldgen/structure/piece_placer/jungle_temple.rs` | 370 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1355 | `steel-core/src/worldgen/structure/piece_placer/mineshaft.rs` | 1138 | 0 | 0 | 8 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1356 | `steel-core/src/worldgen/structure/piece_placer/mod.rs` | 461 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1357 | `steel-core/src/worldgen/structure/piece_placer/ocean_monument/helpers.rs` | 235 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1358 | `steel-core/src/worldgen/structure/piece_placer/ocean_monument/mod.rs` | 119 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1359 | `steel-core/src/worldgen/structure/piece_placer/ocean_monument/room_core.rs` | 102 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1360 | `steel-core/src/worldgen/structure/piece_placer/ocean_monument/room_double.rs` | 385 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1361 | `steel-core/src/worldgen/structure/piece_placer/ocean_monument/room_simple.rs` | 219 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1362 | `steel-core/src/worldgen/structure/piece_placer/ocean_monument/room_special.rs` | 117 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1363 | `steel-core/src/worldgen/structure/piece_placer/ocean_monument/shell.rs` | 620 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1364 | `steel-core/src/worldgen/structure/piece_placer/pool_element.rs` | 169 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1365 | `steel-core/src/worldgen/structure/piece_placer/ruined_portal.rs` | 318 | 0 | 0 | 3 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1366 | `steel-core/src/worldgen/structure/piece_placer/scattered_feature.rs` | 343 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1367 | `steel-core/src/worldgen/structure/piece_placer/stronghold.rs` | 920 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 🟡 Требует внимания | Uses %2==0 instead of is_multiple_of |
| 1368 | `steel-core/src/worldgen/structure/piece_placer/swamp_hut.rs` | 125 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1369 | `steel-core/src/worldgen/structure/piece_placer/template_piece.rs` | 872 | 0 | 0 | 1 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1370 | `steel-core/src/worldgen/structure/piece_placer/template_processors.rs` | 179 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1371 | `steel-core/src/worldgen/surface/mod.rs` | 677 | 1 | 0 | 22 | 0 | 1 | 0 | 0 | 🟡 Требует внимания | Many as casts: 22 |
| 1372 | `steel-core/src/worldgen/template/loading.rs` | 418 | 9 | 23 | 0 | 5 | 0 | 0 | 0 | 🟡 Требует внимания | Many format!: 23 |
| 1373 | `steel-core/src/worldgen/template/mod.rs` | 131 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1374 | `steel-core/src/worldgen/template/placement.rs` | 736 | 4 | 0 | 2 | 5 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1375 | `steel-core/src/worldgen/template/processors.rs` | 820 | 2 | 0 | 8 | 1 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1376 | `steel-core/src/worldgen/template/state_transforms.rs` | 405 | 4 | 0 | 2 | 0 | 0 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |
| 1377 | `steel-core/src/worldgen/template/tests.rs` | 595 | 0 | 0 | 0 | 4 | 7 | 0 | 0 | ✅ Чисто — modern | is_multiple_of/is_none_or/expect |