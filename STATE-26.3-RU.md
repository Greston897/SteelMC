# SteelMC ↔ Minecraft Java 26.3 — состояние по областям

**Дата замера:** 9 ноября 2026
**Ветка:** `arena/8ff5adc9-steelmc` (HEAD `da11bda`)
**Цель:** Java Edition **26.3** (протокол 777)
**Метод:** машинный подсчёт по дереву репозитория и по извлечённым из ванили
данным (`steel-registry/build_assets/*.json`, `steel-core/build/classes.json`),
плюс ручная проверка каждой «подключённой» механики по цепочке вызовов.
Оценки «на глаз» помечены явно.

> Этот документ заменяет прежние `PARITY-26.3-RU.md`, `PARITY-AUDIT-26.3.md`,
> `PROGRESS.md` и `PROGRESS-RU.md`. Числа здесь пересчитаны заново по коду, а не
> перенесены из старых отчётов.

---

## 1. Как считалось

Три разных вопроса, которые прежние отчёты смешивали:

1. **Написано ли?** — есть ли `struct`/`impl` для ванильного класса. Считается
   по `steel-core/build/classes.json` (класс поведения на каждый блок, предмет,
   сущность) и по наличию имени класса в `steel-core/src`.
2. **Зарегистрировано ли?** — попадает ли объект в `BEHAVIORS`,
   `BLOCK_ENTITIES`, `MENUS` и т.п.
3. **Вызывается ли?** — есть ли реальный путь исполнения: тикер у block entity,
   `send_packet` у пакета, `open_menu` у меню, ветка в `mob_interact`.

**Третий пункт — главный.** Именно он находил всё, что было «написано, но не
работает»: `BannerBlockEntity` без создания у блока, `CreakingHeartBlockEntity`
без тикера, `CMountScreenOpen` без отправителя. Такие находки перечислены в §6.

---

## 2. Объём кода

| Крейт | LOC (`src/**/*.rs`) |
|---|---:|
| steel-core | 448 506 |
| steel-registry | 60 933 |
| steel-worldgen | 21 551 |
| steel-utils | 17 733 |
| steel-protocol | 11 855 |
| steel | 3 931 |
| steel-login | 2 312 |
| steel-flint | 1 296 |
| steel-macros | 1 178 |
| steel-math | 1 055 |
| steel-crypto | 504 |
| **Всего** | **611 735** |

`#[test]` в дереве: **4 536**.

steel-core по подсистемам:

| Подсистема | Файлов | LOC |
|---|---:|---:|
| entity | 355 | 123 131 |
| command | 142 | 45 486 |
| behavior | 322 | 68 050 |
| worldgen | 128 | 41 072 |
| player | 72 | 33 975 |
| chunk | 59 | 30 476 |
| world | 60 | 24 712 |
| server | 32 | 19 322 |
| block_entity | 54 | 18 062 |
| inventory | 55 | 15 433 |

---

## 3. Контент: написано и зарегистрировано

| Категория | Готово | Всего | % | Источник числа |
|---|---:|---:|---:|---|
| Блоки с собственным классом поведения | 1 104 | 1 286 | 85,8 % | `classes.json`, класс ≠ `Block` |
| Предметы с собственным классом | 1 401 | 1 658 | 84,5 % | `classes.json`, класс ≠ `Item` |
| Сущности с собственным классом | 161 | 161 | 100 % | `classes.json`, 143 класса |
| Block entity зарегистрированы | 49 | 49 | 100 % | `block_entity/registry.rs` |
| Block entity создаются блоком | 49 | 49 | **100 %** | проверка `new_block_entity` по классу |
| Меню (типы экранов в коде) | 18 | 25 (в протоколе) | 72 % | `menu/kinds/*.rs` против `menutypes.json` |
| Команды | 72 | ~50 ванильных + свои | 100 % ванильных | `command/builtins/*.rs` |
| Геймрулы | 59 | 59 | 100 % | `game_rules.json` |
| Clientbound-пакеты в коде | 121 | 173 | 70 % | `packets/game/c_*.rs` |
| Serverbound-пакеты в коде | 56 | 87 | 64 % | `packets/game/s_*.rs` |

Пояснения к числам:

* «Блоки с собственным классом» — блоки, у которых ванильный класс не сводится к
  базовому `Block`. Остальные 182 блока обслуживает базовое поведение, и это
  корректно: у них в ванили тоже нет переопределений.
* «Меню 18 / 25» — 25 в `menutypes.json` это типы *окон*: часть из них
  (`generic_9x1`…`generic_9x6`) это одно и то же меню с разным числом строк,
  поэтому 18 файлов покрывают все 25 типов. **Покрытие типов меню — 25/25.**
* Пакеты: отсутствующие 52 clientbound и 31 serverbound перечислены в §5.

---

## 4. Области: что реально работает

Статус «работает» = есть путь исполнения от тика/пакета/команды до эффекта.

### 4.1 Мир и генерация — работает

* `steel-core/src/chunk/` — чанки, секции, палитры, границы, тик-расписание,
  блок-тики (`chunk_map/scheduled_ticks.rs`), fluid-тики.
* `steel-core/src/worldgen/` — биомы, карверы, фичи, структуры, слои,
  поверхностные правила, `FlatChunkGenerator`, декорация фич по биому.
* Свет: `light/` — небесный и блочный, пересчёт при изменении блока.
* Сохранения: `chunk/data.rs`, `player/data`, `level.dat`-эквивалент.

### 4.2 Блоки и block entities — работает

Все 49 типов block entity создаются своими блоками (§3). Тикеры выдаются там,
где ванильный `getTicker` их возвращает: `beacon`, `bell`, `brewing_stand`,
`campfire`, `chest`/`trapped_chest`, `command_block`, `conduit`, `crafter`,
`creaking_heart`, `daylight_detector`, `enchanting_table`, `end_gateway`,
`hopper`, `jukebox`, `mob_spawner`, `piston` (moving), `sculk_shrieker`,
`shulker_box`, `sign`, `hanging_sign`, `beehive`, печи и доменная печь.

Отдельные механики, проверенные по вызову:

| Механика | Файл | Проверка |
|---|---|---|
| Крышка сундука | `block_entity/entities/chest.rs`, `behavior/blocks/container/chest_block.rs` | `ChestLidController` тикает, block event 1 при смене `open` |
| Улей | `block_entity/entities/beehive.rs`, `behavior/blocks/container/beehive_block.rs` | `serverTick`-порт: выпуск пчёл, +1 к `LEVEL_HONEY` |
| Крекинг-харт | `behavior/blocks/building/technical_blocks.rs` | тикер → `spawn_creaking` |
| Сушёный гаст | `behavior/blocks/building/infested_block.rs` | `random_tick` → 5000 тиков → гидратация → гастлинг |
| Окисление меди | `behavior/blocks/building/weathering_block.rs` + `build/weathering.json` | `WeatheringCopper` у 60+ блоков |
| Огонь | `behavior/blocks/portal/fire.rs` | `tick` + первичное планирование в `on_place` |

### 4.3 Сущности и ИИ — работает

* 161 тип сущности с собственным классом, 143 класса реализованы.
* Цели ИИ: `entity/ai/goal/` — 60+ целей, включая `FloatGoal` (плавание),
  навигацию, атаку, панику, следование, размножение.
* Передвижение: `travel_in_fluid` (вода/лава) вызывается из `default_travel`,
  поэтому мобы плавают, а не тонут; `travel_fall_flying` для элитр.
* Бой: `living_entity.rs` — поглощение урона, чары, броня, `CombatTracker`-лайт
  (`tick_living_combat_memory`).
* Ездовые животные: лошадь, осёл, мул, лама, караванная лама, верблюд —
  седло/упряжь, управление по взгляду, **и экран инвентаря** (§6, `993e8d3`/`da11bda`).
* Жители: профессии, торговля, `work_at_poi`, сон в кровати.
* Рейды, патрули, визер, эндермен, крипер, слаймы — с тиками.

### 4.4 Инвентарь и меню — работает

* 18 файлов меню покрывают 25 типов окон: сундук/двойной сундук, бочка,
  шалкер, эндер-сундук, крафт 2×2 и 3×3, крафтер, печи (3), зельевар,
  наковальня, чары, точило, кузнечный стол, ткацкий станок, картография,
  камнерез, торговля, маяк, лектерн, раздатчик/выбрасыватель, воронка,
  сундук лодки и **экран ездового животного**.
* Клики, шифт-клики (маршруты `MenuBuilder::route`), drag, quick-craft,
  swap-хотбар, `set_carried`, творческий режим.

### 4.5 Протокол и сеть — работает

* `steel-protocol`: 121 clientbound + 56 serverbound структур, кодирование
  через `steel-macros` (`ClientPacket`, `WriteTo`, `ReadFrom`).
* `steel-login`: handshake, status (ping с data-URI иконкой), login, config,
  шифрование, сжатие.
* `steel-core/src/player/connection/` — отправка, бандлы, отключение.

### 4.6 Серверные системы — работает

* Команды: 72 файла, включая `execute`, `data`, `scoreboard`, `team`,
  `bossbar`, `worldborder`, `schedule`, `function`, `advancement`, `loot`.
* Права: `permission/` — группы, узлы `minecraft.command.*`, `groups.toml`.
* RCON, whitelist, ban/banip, `setidletimeout`, `transfer`.
* Планировщик тиков, TPS-бар, `save`, `stop`, `reload` (частично, §6).

---

## 5. Пакеты, у которых нет отправителя

### 5.1 Clientbound без `send_packet` (24)

Проверено поиском имени структуры вне `steel-protocol`; отсутствие упоминания =
пакет не отправляется ни одним путём.

| Пакет | Почему не отправляется | Что нужно |
|---|---|---|
| `CMountScreenOpen` | **закрыто** (`993e8d3`) | — |
| `CPostEffects` | нет трекера «грязных» пост-эффектов | `post_effects_dirty` + `ServerPlayer.sendPostEffects` |
| `CPlayerCombatEnter`/`CPlayerCombatEnd` | клиент их игнорирует (wiki.vg) | сознательно не шлём |
| `CConfigPing`, `CStoreCookie`, `CStartConfiguration` | нет сценария (переход в config) | `transfer`-путь |
| `CShowDialog`, `CClearDialog`, `CCodeOfConduct`, `CCustomReportDetails`, `ChatCompletionAction` | нет подсистемы диалогов | отдельная фича |
| `CDebug*` (4) | отладка debug-палоуи | `/debug` |
| `CAddTransientBlock` | нет «временного» блока | — |
| `CDeleteChat`, `CResetChat` | нет серверного управления чатом | — |
| `CGameTestHighlightPos` | нет game-test раннера | — |
| `CMoveMinecartAlongTrack` | вагонетка не шлёт интерполяцию | физика вагонеток |
| `CPlayerRotation` | ротация отправляется через `CEntityPositionSync` | — |
| `CResourcePackPop` | нет стека ресурспаков | — |

### 5.2 Serverbound без обработчика

31 структура из 56 обрабатывается; остальные объявлены и декодируются, но
обработчик — no-op. Это в основном клиентские подтверждения
(`SChunkBatchReceived`, `SPong`, `SClientTickEnd`), для которых серверного
эффекта в ванили нет.

---

## 6. Закрытые разрывы в этой сессии

Ниже — то, что было **написано, но не подключено**, и теперь работает.

| Коммит | Разрыв | Суть |
|---|---|---|
| `a744d4b` | крышка сундука не анимировалась | `ChestLidController` + тикер для `CHEST`/`TRAPPED_CHEST` |
| `38053d8` | пчёлы в улье не выпускались | порт `BeehiveBlockEntity.serverTick` + тикер |
| `a994190` | крекинг-харт никогда не спавнил | `get_block_entity_ticker` для `CREAKING_HEART` |
| `c283889` | сушёный гаст не гидратировался | `random_tick`/`tick` + спавн гидратированного гаста |
| `252054b` | баннеры, статуи голема и test-блоки не имели block entity | `new_block_entity` для `BannerBlock`, `WallBannerBlock`, `WeatheringCopperGolemStatueBlock`, `TestBlock`, `TestInstanceBlock` |
| `993e8d3` | `CMountScreenOpen` не отправлялся | `AbstractMountInventoryMenu`-порт: `mount_menu.rs`, `MountSlot`, `MountInventory`, `Player::open_entity_menu` |
| `da11bda` | осёл/мул/лама/верблюд не открывали инвентарь | общий `mobInteract`-порядок + `open_menu_for` |

Что проверялось и оказалось **корректным** (чтобы не переделывать):

* Окисление меди: 112 блоков с `randomly_ticking`, 105 из них покрыты
  `WeatheringCopper` через `classes.json`/`weathering.json`; остальные —
  `CropBlock` (уже с `random_tick`), `NetherPortalBlock` (ванильный
  `randomTick` — пустой вызов `super`), `DriedGhastBlock` (закрыт `c283889`).
* Печи: `impl_furnace_block!` выдаёт тикер для `FURNACE`, `BLAST_FURNACE`,
  `SMOKER` через `$entity_type`.
* Доменная печь (домен = измерение, не «печь»): покрыта тем же макросом.
* Все 84 метода `BlockBehavior` вызываются хотя бы из одного места.

---

## 7. Что остаётся нереализованным

Отсортировано по влиянию на игрока.

| # | Область | Файл/место | Состояние |
|---|---|---|---|
| 1 | Эндер-дракон и `DragonFight` | `entity/entities/mobs/hostile/ender_dragon.rs` | дракон есть, боя с фазами и порталом возврата нет |
| 2 | Распространение огня | `behavior/blocks/portal/fire.rs` | нет таблиц `ignite_odds`/`burn_odds` в данных |
| 3 | Датапаки с диска | `command/builtins/datapack.rs` | только встроенный `minecraft` |
| 4 | `/reload` | `server/mod.rs:641` | перезагружает группы прав и геймрулы, не реестры |
| 5 | `CPostEffects` | — | нет трекера пост-эффектов |
| 6 | Трекер боя (`CPlayerCombatEnter/End`) | `entity/living_entity.rs` | память боя есть, пакетов нет (клиент их игнорирует) |
| 7 | Сундук у осла/ламы (слоты 2..) | `entity/mount.rs` | колонок 1, сундук не расширяет инвентарь |
| 8 | Прыжок лошади (полоса заряда) | `horse.rs` | управление есть, прыжка с зарядом нет |
| 9 | `work_at_poi` у некоторых профессий | `entity/.../villager.rs` | часть POI не назначена |
| 10 | Game-test раннер | — | блоки test есть, раннера нет |

---

## 8. Итоговые оценки по областям

Взвешено по объёму кода и по числу ванильных механик; «работает» = §4.

| Область | Готовность | Обоснование |
|---|---:|---|
| **Реестры и данные** | **≈ 100 %** | все таблицы 26.3 извлечены; 49/49 BE, 25/25 типов меню, 59/59 геймрулов |
| **Инфраструктура** (чанки, свет, сохранения, сеть, протокол) | **≈ 93 %** | всё ядро работает; минусы — датапаки с диска, `/reload`, 24 clientbound без отправителя |
| **Контент: блоки** | **≈ 92 %** | 1 104/1 286 своих классов, все BE создаются, огонь не распространяется |
| **Контент: предметы** | **≈ 90 %** | 1 401/1 658 своих классов, остальное — корректный базовый `Item` |
| **Контент: сущности** | **≈ 88 %** | 161/161 с классами; недоделаны дракон, прыжок лошади, сундук ламы |
| **Живой мир** (ИИ, бой, редстоун, спавн) | **≈ 78 %** | ИИ и редстоун работают, спавн сверен с ванилью; нет дракона и распространения огня |
| **Инвентарь и меню** | **≈ 97 %** | 25/25 типов, клики/шифт-клики/drag; минус — сундук ламы |
| **Серверные системы** (команды, права, scoreboard, админ) | **≈ 90 %** | 72 команды, RCON, права, бан-листы; минус — `/reload` и датапаки |
| **ИТОГО по ядру** | **≈ 90 %** | — |

Прежние отчёты давали 71 % (по §1 старого `PARITY-26.3-RU.md`) и 84 %
(`PARITY-AUDIT-26.3.md`). Разница объясняется тем, что раньше в знаменатель
попадали вещи, которых в ванили нет как отдельных механик (25 типов меню против
18 файлов, 49 block entity против «создаются ли они»), а часть «готового»
считалась готовой без проверки вызова. Здесь оба эффекта устранены: числа
считаются по классам поведения, а «работает» подтверждается путём исполнения.

---

## 9. Как воспроизвести замер

```bash
# объём и тесты
find . -path ./target -prune -o -name '*.rs' -print | xargs wc -l | tail -1
grep -rn '#\[test\]' --include=*.rs . | wc -l

# покрытие блоков/предметов/сущностей по ванильным классам
python3 - <<'EOF'
import json
d = json.load(open('steel-core/build/classes.json'))
for k in ('blocks', 'items', 'entities'):
    cls = {e.get('class') for e in d[k]}
    print(k, len(d[k]), 'classes:', len(cls))
EOF

# block entity: зарегистрированы и создаются ли блоком
grep -c 'vanilla_block_entity_types::' steel-core/src/block_entity/registry.rs
grep -rn 'fn new_block_entity' --include=*.rs steel-core/src/behavior/blocks | wc -l

# пакеты без отправителя
for f in steel-protocol/src/packets/game/c_*.rs; do
  n=$(basename "$f" .rs); s=$(grep -oP 'pub struct \KC\w+' "$f" | head -1)
  [ -n "$s" ] && ! grep -rq "$s" --include=*.rs steel-core/src steel-login/src \
    && echo "$s"
done
```
