# Оценка ядра по исходникам, подключению и Actions

Срез игрового кода: `e6b3959e61a61f3f410e6a8c7f9f2c6b6cda622d`.
Проверка на живом пользовательском сервере не включена.

## Как читать проценты

**82,5% — укрупнённая инженерная оценка по 60 пунктам, а не измеренный
процент всех vanilla-механик.** 1 балл означает наличие основного пути;
0,5 — частичность, известные ограничения или неустановленную полноту;
0 — не засчитано. Сами оценки ручные; наличие файла и теста не является
доказательством правильности всех ветвей. Полного пометодного аудита нет.

В текущем чеклисте исключены требования внешнего plugin/scripting API:
это не обязательные функции vanilla. Вместо них учитываются отсутствующие
runtime datapack reload и `/datapack enable/disable` (оба 0). Сумма не изменилась.
Подсистемы пересекаются; веса равные, не по LOC и не по трудозатратам.
Все 60 критериев с путями: `IMPLEMENTATION-26.3-RU.md`.
Актуальный пофайловый индекс: `PROGRESS-26.3-INVENTORY.md`.

## Проверенные результаты одного SHA

- Test: [44417341904](https://github.com/Greston897/SteelMC/actions/runs/44417341904) — success.
- Full Conformance: [44417342076](https://github.com/Greston897/SteelMC/actions/runs/44417342076) — success.
- Его per-crate summary: **4737 passed, 0 failed, 7 ignored**;
  отдельный mega-suite повторно запускает один тест, не включён в эту сумму.
- Vanilla Evidence: [44417341916](https://github.com/Greston897/SteelMC/actions/runs/44417341916) — failure.
- Полный Flint: **952/1012 passed, 60 failed (94,07%) в каждом из двух entrypoints**.
  Собраны все 32 shard. Это доля конкретных specs, не процент ядра.
- Registry-sync/tags: 2/2 PASS, включая порядок registry/known-pack elision и
  независимое сравнение membership тегов. Это не покрывает все CODEC и custom datapacks.
- FEATURES Overworld: семь известных расхождений; нельзя считать генерацию идентичной.
- Features diagnostic [44417342121](https://github.com/Greston897/SteelMC/actions/runs/44417342121) — failure.
  Java capture произвёл данные семи чанков, но завершающая проверка/Steel block diff
  не завершились успешно. Нельзя представлять его как подтверждённый reference.
- Зелёные Lint/Build Report не равны чистым diagnostics: опубликованы Clippy errors
  (в том числе 30 в build script steel-registry) и rustfmt differences. Часть шагов
  диагностическая либо использует pipeline, скрывающий код ошибки.

## Конкретно отсутствует или неполно

1. `steel-core/src/command/builtins/datapack.rs`: enable/disable направлены в
   `unsupported`; загрузка внешних пакетов с диска прямо объявлена неподдерживаемой.
2. `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs`: документировано
   отсутствие boss bar и трёхголовых skull volleys. Есть spawn-фаза, но это не полный Wither.
3. `steel-core/src/inventory/openers_counter.rs::recheck_openers`: вместо полного
   пересчёта открывших игроков сохраняется оптимистичный счётчик.
4. `steel-core/src/block_entity/entities/jukebox.rs`: нет полноценного container
   interface для hopper; полный Flint подтверждает оставшиеся insert/extract/playing failures.
5. Не закрыты перечисленные ниже placement, fluids, redstone, dispenser, cauldron,
   sculk и entity specs. Падение spec означает наблюдаемое расхождение тестового
   исполнения, но само по себе не локализует ошибку в конкретной функции.
6. Полнота всех CODEC, datapack stacking/reload/provenance, миграций произвольных
   миров, навигации/видового AI и сложных progression chains не установлена.
   Это зоны неполной проверки, их нельзя автоматически называть отсутствующим кодом.

## Оставшиеся 60 Flint specs

- `Campfire Shovel Extinguishes`
- `Fence Connections`
- `Fence Jack O Lantern Connection`
- `Firework Rocket Dispenser Launch Sideways`
- `Firework Rocket Dispenser Launch Up`
- `Nether Brick Fence Connections`
- `Nether Brick Fence Exceptions`
- `Wolf Collar Dye Other Owner Ignored`
- `Wolf Collar Dye Untamed Wolf Ignored`
- `advance_time`
- `amethyst_block_resonance`
- `bamboo_stacking_on_sapling`
- `bell_double_wall_losing_supports`
- `bell_double_wall_placement`
- `bell_single_wall_becomes_double_wall`
- `bell_single_wall_loses_support`
- `bell_single_wall_placement`
- `brush_dispenser_without_armadillo`
- `cactus_fluid_interaction`
- `copper_wall_torch_four_side_placement`
- `coral_wall_fan_placement_in_air`
- `coral_wall_fan_placement_in_flowing_water`
- `coral_wall_fan_placement_in_water`
- `crimson_fungus_support_loss`
- `crimson_stem_survives_fire`
- `dead_coral_wall_fan_placed_in_water_is_waterlogged`
- `ender_chest_waterlogged_spreads_water`
- `flint_and_steel_dispenser_blocked_target_still_takes_damage`
- `flint_and_steel_side_of_flammable_block_places_fire`
- `flowering_azalea_bonemeal_build_height_limit`
- `flowering_azalea_bonemeal_consumed`
- `flowering_azalea_bonemeal_grows_tree`
- `flowering_azalea_bonemeal_rejected_under_water`
- `glass_bottle_dispenser_beehive_collect_honey`
- `glass_bottle_dispenser_fills_flowing_water`
- `glass_bottle_dispenser_fills_water_source`
- `glass_bottle_dispenser_fills_waterlogged_slab`
- `grindstone_placement_follows_clicked_face`
- `grindstone_wall_placement_facing`
- `hay_block_east_face_placement`
- `hay_block_north_face_placement`
- `jack_o_lantern_dispenser_drops_item`
- `jukebox_hopper_extracts_idle_disc`
- `jukebox_hopper_inserts_disc`
- `jukebox_playing_locks_hopper_below`
- `oak_hanging_sign_chaining`
- `oak_pressure_plate_marker_stand_ignored`
- `oak_wall_hanging_sign_never_breaks`
- `pale_oak_wood_creaking_heart_axis_mismatch`
- `pale_oak_wood_creaking_heart_dormant`
- `red_shulker_box_washed_in_cauldron`
- `shovel_extinguishes_campfire_with_block_above`
- `shulker_box_dispenser_placement`
- `slab_merging`
- `slab_waterlogging_placement`
- `stained_glass_side_face_placement`
- `stone_does_not_resonate`
- `water_cauldron_glass_bottle`
- `water_cauldron_wash_shulker_box`
- `waxed_lightning_rod_struck_by_lightning`
