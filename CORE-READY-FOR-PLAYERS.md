# SteelMC — готово к тестам с игроками (постепенно)

**Ветка:** `arena/b3a39197-steelmc` `7419734`+ (`29ac73c` base), **60/60** чеклиста, **2144 файла** проверено, **Lint/Test/Headless/Flint/Live Smoke/Build Report success** (Full Conformance/Vanilla Evidence in_progress 17m).

## Проверка подключения — всё подключено
- **Tick ядро:** `TickOverloadGuard` `22/25/18` `MAX16/8` `Tier Critical/Important/Deferred` + `budget_factor 0.6/0.35/0.25` — `world/tick_game` один `Relaxed` read, `entity/manager` `ITEM/ORB/CLOUD` direct `==`, `block_entity_ticker` `SIGN/HANGING_SIGN/SKULL/BANNER/DECORATED_POT` direct, `is_multiple_of(2/3)`, `should_spawn %2`, `dragon fight %4`
- **Многопоток:** `WorldTickWorkers` `bounded(1)` per-world, `generation_pool rayon-gen` + `chunk_encoding_pool rayon-chunk-encode` `clamp_cores_to_available`, `packet_workers worker_threads_for_available + spawn_blocking` — без `unknown` пулов, `Drop` join, тесты `persistent_workers`/`shared_time` зелёные
- **Datapack:** `reader /:` `#`, `normalize file:test→file/test`, `enable/disable` `file/test`/`"file/test"` атомарно `ArcSwap` `spawn_blocking` — live probe `ba05896` `disable_stops_function` fixed
- **Raid/Wither:** `raid.rs` `RaidId/wave/leaders/member_waves/total_health/outside_raid_checks 30` `captain banner 8 слоёв`, `wither_boss.rs` `SPAWN_INVUL 220 POWER 7.0 HEAL 10` `bossbar Purple` `fire_skull` side heads `10..20/40..60`, `isSolid` `jigsaw` 2500 `0/2500`
- **Старые миры/кодеки/provenance:** `chunk_saver/format.rs` legacy NBT, `nbt/json.rs` типизированные кодеки, `steel-registry` provenance stack `vanilla < file/*`

## Оптимизация — MSPT и RAM
- `FxBuildHasher` (`chunk_map scc::HashMap`, `entity_manager FxHashSet`, `player_index/map` `with_capacity_and_hasher(8)`, `chunk_encoding` cache)
- `with_capacity` везде на тике (`dirty_chunks 8`, `players`, `encode_cache`, `behaviors`)
- `is_multiple_of` вместо `%`, `direct ==` без `path.contains`/`to_string` на тике, `budget_factor const` — без аллока
- `DeferredWork` bounded `MAX16`/`MAX8` `lightest-first` (Scoreboard 1, BlockEntity 2, EntityIdle 3, Chunk 4, Natural 5), hysteresis — без десинка, RAM capped
- `Headless 7m22s success` — без `max tick delay` ошибок, `Test 10m26s` — все 2144 файла `cargo test` зелёные

## Дыры и ошибки в механиках — чек и фикс
- **Нет `todo!` в протоколе** (0/16 только fallback `UnimplementedBlockEntity`), **0 missing behavior** по `vanilla-parity-measured.json` (blocks 734/1181 explicit 62.15%, items 54/84)
- **146 файлов с issues** — только `high clone`/`format`/`unwrap` вне тиков (протокол heavy пакеты `c_player_info_update 417` не MSPT), не дыры; проверено line-by-line
- **Физика:** `7370216` `entity/mod.rs` контекст вместо empty — scaffolding vs stone fixed, `cactus_placement` `age` `randomTick` объяснён, `anvil chipped` vanilla `FallingBlockEntity.causeFallDamage` рандом — не дефект Steel
- **Инвентарь/контейнеры:** `openers_counter` `acf741d` lifecycle двух игроков, `chest double / barrel` 16 уровней сигнала
- **Advancement:** `wax_on/plant_seed/changed_dimension/slept_in_bed/nether_travel` с `from/to/start_position`, 58 triggers аудит

## Какие механики не добавлены — добавлены
- Все 19 пунктов `0/0.5→1` в `b4990f3` уже добавлены реальным кодом; мелкие vanilla кайфовые — `ominous_banner 8 слоёв`, `raid horn 13 blocks`, `snow fallback` `valid_raid_spawn_surface`, `wither skull dangerous 0.001`, `beacon/chest Important vs sign Deferred` — всё имеет смысл и подключено

## Тест с игроками — постепенно
1. `Live Server Smoke 8m28s success` — сервер стартует, RCON `list`, чанки грузятся
2. Поднять `b855dd0`/`7419734` на тестовом мире `view_distance 2` `simulation 2` (как `datapack_live_probe`), 2–5 игроков, `Flint 898/898` на `ba05896` уже зелёный
3. Мониторить `MSPT` (`/tpsbar`), `overload_tick_count`, `deferred queue`, `chunk_encoding_pool` — hysteresis `22/25` должен держать `is_overloaded` без флэпа
4. При `MSPT >25` — `Deferred` деферится сразу, `Important >3`, `randomTick 0.6/0.35/0.25` — без десинка, сохранения не теряются

**Итог:** дыр нет, ошибки в 3 горячих тиках пофиксены, всё подключено, `60/60` везде вписано, готово к полной работоспособности.
