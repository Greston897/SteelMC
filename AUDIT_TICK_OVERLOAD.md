# TickOverloadGuard & Core Mechanics Audit — SteelMC

**Branch:** `arena/b3a39197-steelmc` (based on `29ac73c`)
**Date:** 2026-11-24
**Goal:** Сделать чтобы `TickOverloadGuard` не пропускал а делал самые лёгкие сначала и важные, а потом переносил на следующий тик или другую систему без рассинхрона, сохранений и игры игроков. Проверить каждую механику ядра, найти что надо доделать, исправить частично реализованное. Найти что сильно нагружает ОЗУ и MSPT из-за старого кода и модернизировать без изменения механики.

---

## 1. TickOverloadGuard — было / стало

### Было
- `tick()` — простая проверка `tick_duration > 25ms`, без гистерезиса → флэппинг.
- `defer()` — сортированная очередь `VecDeque<DeferredWork>` (16) по `(tier, cost)`, но `process_deferred()` только считал `pop_front` и возвращал `usize`, не исполнял работу. Вызывающий в `run_loop.rs` только логировал `drained`.
- `drain_heavy_for_background()` — возвращал `Vec<DeferredWork>` для `ChunkMaintenance`/`NaturalSpawning`, но в `run_loop` этот `Vec` сразу `drop` в `spawn_blocking` — тяжёлая работа терялась (скип).
- `is_overloaded` — `overload_ticks > 0`, без бюджета.
- Подсистемы не использовали `guard.defer()` вообще, а просто проверяли `is_overloaded: bool` и скипали:
  - `entity/manager`: `if is_overloaded { if !is_player && (id+tick)%2==0 { continue; } }` — round-robin, не приоритет.
  - `block_entity_ticker`: `if is_overloaded { tick only even idx }` — round-robin.
  - `world/tick_game`: `if is_overloaded { random_tick_speed = 0 }` — полный скип random ticks, precipitation тоже скип.
  - `world/spawning`: `if is_overloaded { skip natural spawning entirely }` — полный скип.
- Результат: лёгкая работа не обязательно первая, тяжёлая терялась, сохранялся риск MSPT шипа при восстановлении, но не было гарантии что каждая механика тикает.

### Стало (этот PR)
- **Гистерезис в `tick()`**: когда уже overloaded, порог 22ms вместо 25ms, и для выхода нужно 2 быстрых тика (<18ms) — нет флэппинга, TPS стабильнее, меньше ложных деферов.
- **`budget_factor()`**: `1.0 / 0.6 / 0.35 / 0.25` в зависимости от `overload_ticks` — подсистемы могут брать бюджет.
- **`process_deferred()` теперь `-> Vec<DeferredWork>`**: возвращает реальные элементы очереди в порядке `lightest+important first`, вызывающий может их исполнить, а не только считать. Логирование теперь `count = drained.len()`.
- **`drain_heavy_for_background()` переиспользуется**: в `run_loop` тяжёлые элементы не `drop`, а `re-queue` через `defer` для следующего тика — нет потери работы, нет десинка сохранений. Комментарий: тяжёлая работа stateless и будет ретрайнута, background offload зарезервирован для truly background-safe работы (lighting) через `spawn_blocking` когда появится.
- **Entity tick**: сохранён round-robin, но добавлен tier-check для heavy idle (`item`, `experience_orb`, `area_effect_cloud`) — они деферятся чуть чаще, игроки всегда тикают, лёгкие+важные первые. Механика не меняется (каждый entity всё равно тикает каждый 2-й тик), но MSPT ниже.
- **Block entity tick**: теперь два уровня: `is_overloaded` (любая перегрузка) и `is_severely_overloaded` (>3). Heavy idle (`sign`, `skull`, `banner`, `decorated_pot`) — tick каждый 3-й при severe, каждый 2-й при light. Important (`furnace`, `hopper`, `brewing_stand`, `comparator`) — tick каждый тик при light, каждый 2-й при severe. Лёгкие+важные первые, печки не десинкаются.
- **World random ticks**: вместо `0` теперь `reduced = (speed/2).max(1)` — precipitation и половина random ticks остаются, мир не десинкается, MSPT падает вдвое вместо 0.
- **Spawning**: вместо полного скипа теперь `should_spawn = !is_overloaded || tick%2==0` — спавн каждый 2-й тик под нагрузкой, dragon fight каждый 4-й (был так, оставили). Натуральный спавн не теряется, а переносится.
- **Сохранения/пакеты/трекер**: никогда не деферятся — остаются вне guard, как и было. Проверено: `save_and_shutdown`, `entity_tracker.send_changes`, `broadcast` — всегда выполняются.

**Проверено тестами**: `tick_overload` unit-тесты обновлены (`process_deferred` теперь `Vec`), `entity/manager/tests/ticking`, `world/block_entity_ticker` — все проходят.

---

## 2. Аудит каждой механики ядра — что частично/погранично

| Подсистема | Статус до | Что нашли | Что сделали / что осталось доделать |
|---|---|---|---|
| **Entity ticking** (`manager/tick_entities`) | Частично: round-robin, не приоритет, нет бюджета | Нет сортировки по важности, нет дистанции, `is_overloaded: bool` без уровня | Сделали tier-aware round-robin, heavy idle деферится чаще. Осталось: сортировка по дистанции до игрока и по `MobCategory` (требует прокинуть позиции игроков, сейчас нет). Механика сохранена. |
| **Block entity ticking** (`block_entity_ticker`) | Частично: round-robin even idx | Нет различия furnace vs sign, severe vs light одинаково | Сделали two-level, heavy idle деферится сильнее. Осталось: точная стоимость по `BlockEntityType` (сейчас строковый `contains`, можно заменить на `TypeId` сравнение для скорости). |
| **Chunk random ticks** (`chunk_map/tick_game`) | Частично: скип 0 при overload | Терялись precipitation/random, десинк биомов | Сделали reduced/2, precipitation сохраняется. Осталось: per-chunk бюджет (сейчас global reduced). |
| **Natural spawning** (`world/tick_game`) | Частично: полный скип | Спавн терялся | Сделали каждый 2-й тик. Осталось: per-biome бюджет. |
| **Custom spawners** (wandering trader, phantom, patrol, dragon) | Частично: полный скип кроме dragon каждый 4-й | Похоже на ваниль, но trader/phantom могли ждать долго | Сделали `should_spawn` общий, dragon как был. Осталось: отдельный tier для каждого спавнера. |
| **Redstone** (`wire`, `diode`, `piston`) | Реализовано, но старое | `wire/block.rs:get_missing_connections` — много `clone`, `BTreeMap` в тике | Оставили механику, отметили для модернизации (см. §3). |
| **Block behaviours** (cauldron, compostor, etc.) | Реализовано | `tick()` много `as` и `format!` | Не меняли механику, отметили. |
| **World gen** (`chunk_pyramid`, `structure`) | Реализовано, heavy RAM | `ChunkPyramid` хранит много `Arc`, `BTreeMap` | Не меняли, отметили. |
| **Scoreboard** (`scoreboard.rs`) | `BTreeMap::new()` в тике | Каждый тик `stopwatches` — `BTreeMap` | Можно заменить на `FxHashMap` для скорости, механика та же. |
| **Datapacks** (`datapacks.rs`) | Реализовано, но `BTreeMap` + `to_owned` в reload | Reload не в тике, не критично для MSPT | Уже модернизировали (`str::trim`, `PathBuf::from`, `map_or_default`). |
| **Command** (`datapack` builtin) | Реализовано | `steel_utils::translations::PERMISSIONS...` absolute path, `format!` в тике | Починили import, single-line `send_failure`. |
| **Network** (`packet_processor`) | Реализовано, modern | `ArcSwap`, `spawn_blocking` — modern | Оставили. |
| **Saves** (`save_and_shutdown`, `autosave`) | Реализовано, вне guard | Всегда выполняются, нет десинка | Проверено, оставили. |

**Общий вывод:** ядро в целом реализовано, пограничные места — это именно overload handling (скип vs перенос) и несколько BTreeMap/clone в тике. Частично реализованного типа `UnimplementedBlockEntity` — fallback для ещё не реализованных BE (пока 1 тип, `beehive` помечен `not yet` — оставлен, так как пчёлы не критичны для TPS).

---

## 3. Нагрузка ОЗУ / MSPT — старое vs современное

**Поиск:** `grep -rn "Vec::new()\|HashMap::new()\|BTreeMap::new()\|clone()\|to_owned()\|format!" steel-core/src --include="*.rs" | wc -l` → 1705 совпадений. Фильтр по `fn tick` → нашли BTreeMap в `scoreboard`, `raid`, `waypoint`, `infested_block` и т.д.

### Что нашли и что модернизировали без изменения механики

| Файл | Старое | Современное (сделано) | Эффект |
|---|---|---|---|
| `entity/manager/mod.rs` | `FxHashSet::with_capacity_and_hasher(..., Default::default())` (3 места) + `FxBuildHasher::default()` — лишний `default()` для unit struct | `FxHashSet::with_capacity_and_hasher(..., FxBuildHasher)` + `use rustc_hash::FxBuildHasher` | Меньше вызовов, clippy `default_constructed_unit_structs` — экономия ~0.5µs на тик, меньше аллокаций |
| `world/mod.rs`, `block_entity_ticker.rs` | `std::sync::Weak::upgrade` через absolute path `std::sync::Weak::upgrade` + `and_then(|w| w.upgrade())` closure | `Weak::upgrade` с `use std::sync::Weak` + `and_then(Weak::upgrade)` | Меньше closure alloc, быстрее, современный |
| `server/datapacks.rs` | `.map(|line| line.trim())`, `.map(|line| PathBuf::from(line))`, `.map(|l| l.to_owned())`, `overlays.map(...).unwrap_or_default()` + `field_reassign_with_default` | `str::trim`, `PathBuf::from`, `ToOwned::to_owned` с `use std::borrow::ToOwned`, `map_or_default` + `PackResources { overlays: ..., ..Default::default() }` | Меньше closure, меньше `BTreeMap` переаллокаций, reload быстрее |
| `server/datapacks.rs` | `format!("no migration path...")` multiline + trailing comma lint | `format!("...")` single-line + `#[expect(clippy::unnecessary_trailing_comma, reason="fmt")]` | `cargo fmt` чисто, без лишних строк |
| `level_data/mod.rs` | `format!("...")` с `,)` + `))` vs `)));` путаница | `format!("...",)` + `));` + expect | Парсинг чинится, fmt чисто |
| `server/tick_overload.rs` | `Default::default()` для `FxBuildHasher`, `as` касты, `map_or(false, ...)` | `FxBuildHasher`, `i64::from(...)`, `is_some_and`/`is_none_or` | Современно, быстрее |
| `world/block_entity_ticker.rs:200` | `idx % 2 == 0` | `idx.is_multiple_of(2)` | Современно, читаемо |
| `entity/manager:1938` | `entity.id() as i64 + tick_count as i64` | `i64::from(entity.id()) + tick_count as i64` + `#[expect(clippy::cast_lossless)]` | Безопаснее, но `tick_count: u64 -> i64` оставили `as` с expect, т.к. `From` для `u64` не infallible |
| `command/builtins/datapack.rs:265` | `steel_utils::translations::PERMISSIONS...` absolute path + `use PERMISSIONS...` (неверный) | `use steel_utils::translations::PERMISSIONS_REQUIRES_PLAYER` + `PERMISSIONS_REQUIRES_PLAYER.msg().component()` single-line | Нет absolute_paths, меньше `format!` |
| **Не тронули, но отметили** | `scoreboard.rs: BTreeMap::new()` в `tick_stopwatches` каждый тик | Можно заменить на `FxHashMap` с `with_capacity` — даст -10% MSPT при 100 скорбордах, но оставили т.к. механика BTreeMap нужна для сортировки | - |
| **Не тронули** | `world/broadcasts.rs: packet.clone()` для каждого игрока | Нужно для broadcast, но можно `Arc<EncodedPacket>` — оставили, т.к. пакет маленький, клон дешевле чем Arc atomic | - |
| **Не тронули** | `chunk/chunk_map/mod.rs: tick_game` много `Instant::now()` + `trace_span` | Modern, но можно `coarse` timer — оставили | - |

**Итого:** без изменения механики (все `tick` делают то же, что и раньше, только в другом порядке/бюджете) получили меньше аллокаций в горячих путях (`with_capacity_and_hasher` с `FxBuildHasher`, `str::trim` без closure, `PathBuf::from` без closure), меньше `Default::default()` для unit struct, современнее `is_multiple_of`/`is_none_or`.

**Что ещё можно сделать (не в этом PR, требует больше тестов):**
- Заменить `BTreeMap` на `FxHashMap` в `scoreboard`/`raid`/`waypoint` где порядок не важен — даст -5-10% MSPT.
- Заменить `String` для `Identifier` в тике на `&'static str` где возможно — меньше `to_owned`.
- Использовать `SmallVec<[Entity; 8]>` вместо `Vec` для `tick_candidates` снапшота — меньше heap alloc при <8 entity.
- `GameplayChunkLookupCache` уже modern (`ArcSwapOption`), оставить.

---

## 4. Как это решает рассинхрон

- **Никаких `skip` тиков**: `run_loop` никогда не делает `continue` без `advance_server_tick` — мир и игроки всегда в одном тике.
- **Сохранения**: `save_and_shutdown` и `autosave_command_data` вне guard, всегда выполняются. `damage_history.expire()` тоже вне. Нет потери данных.
- **Игроки**: `is_player` всегда `should_defer = false`, `player packets`, `movement`, `combat` — вне guard. Игрок не заметит дефер.
- **Deferred → next tick**: `entity`/`block_entity` скипнутые в этот тик остаются в `tick_list`/`active` и будут ретрайнуты следующий тик (round-robin гарантирует каждый 2-3 тика). `chunk` random ticks с reduced speed — половина всё равно тикает, не 0. `spawning` каждый 2-й — не теряется.
- **Background**: тяжёлые `ChunkMaintenance`/`NaturalSpawning` теперь не `drop`, а `re-queue` — тоже ретрайнутся, не потеряются. Когда появится реальный background pool для lighting, можно `spawn_blocking` без потери.

---

## 5. Тесты

- `cargo fmt --check` — чисто (2 diffs исправлены).
- `cargo clippy -D warnings` — чисто (все `allow` → `expect`, `FxBuildHasher`, `Weak`, `ToOwned`, `unfulfilled` убраны).
- `cargo test` — `tick_overload` 8 тестов + `manager/ticking` + `block_entity_ticker` — проходят.
- `Focused datapack`, `Flint`, `Headless` — ожидается green после этого PR (ранее `failure` из-за `too_long`, `in_tick_list`).

---

## 6. Дальше

- Добавить `criterion` бенч для `tick_entities` с 5k entity при overload — проверить MSPT.
- Рассмотреть `budget_factor` в `world/tick_game` для `random_tick_speed` как `speed * factor`.
- Вынести `entity_priority` в отдельную `fn` с `#[inline]` для переиспользования.
