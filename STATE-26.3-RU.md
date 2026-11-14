# SteelMC ↔ Minecraft Java 26.3 — состояние по областям

**Дата замера:** 9 ноября 2026
**Ветка:** `arena/8ff5adc9-steelmc` (HEAD `64c3863`)
**Цель:** Java Edition **26.3** (протокол 777)
**Метод:** машинный подсчёт по дереву репозитория и по извлечённым из ванили
данным (`steel-registry/build_assets/*.json`, `steel-core/build/classes.json`),
плюс проверка каждой «подключённой» механики по цепочке вызовов и сверка с
исходниками ванили 26.3 (`Renekovski/26.2-mcp`, пути `src/net/minecraft/...`).

> Документ заменяет прежние `PARITY-26.3-RU.md`, `PARITY-AUDIT-26.3.md`,
> `PROGRESS.md`, `PROGRESS-RU.md` и предыдущую версию этого файла. Все числа
> пересчитаны по коду на `64c3863`; утверждения, которые не подтверждаются
> вызовом, из отчёта убраны и либо реализованы, либо явно помечены как пробел.

---

## 1. Как считалось

Три разных вопроса, которые легко смешать:

1. **Написано ли?** — есть ли `struct`/`impl` для ванильного класса. Считается по
   `steel-core/build/classes.json` (класс поведения на каждый блок, предмет,
   сущность) и по наличию имени класса в `steel-core/src`.
2. **Зарегистрировано ли?** — попадает ли объект в `BEHAVIORS`,
   `BLOCK_ENTITIES`, `MENUS` и т.п.
3. **Вызывается ли?** — есть ли реальный путь исполнения: тикер у block entity,
   `send_packet` у пакета, `open_menu` у меню, ветка в `mob_interact`.

**Третий пункт — главный.** Именно он находил всё, что было «написано, но не
работает». В этой сессии так нашлись и закрыты: `/reload`, который ничего не
перечитывал; `DragonFight`, который не был связан с драконом; выдуманный
`reset_combat`, которого нет в ванили; неверный формат `CMoveMinecartAlongTrack`;
`FrogspawnBlock` без `mayPlaceOn` по тегам и без `entityInside`.

---

## 2. Объём кода

| Крейт | LOC (`src/**/*.rs`) |
|---|---:|
| steel-core | 456 038 |
| steel-registry | 86 852 |
| steel-worldgen | 29 413 |
| steel-utils | 18 969 |
| steel-protocol | 11 871 |
| steel | 3 931 |
| steel-login | 2 312 |
| steel-flint | 1 332 |
| steel-macros | 1 178 |
| steel-math | 1 055 |
| steel-crypto | 504 |
| **Всего** | **613 455** |

`#[test]` в дереве: **4 542** (было 4 536 до этой сессии).

steel-core: **1 358** файлов. По подсистемам:

| Подсистема | Файлов | LOC |
|---|---:|---:|
| entity | 355 | 123 858 |
| command | 142 | 45 557 |
| behavior | 322 | 68 566 |
| worldgen | 128 | 41 072 |
| player | 72 | 34 019 |
| chunk | 59 | 30 476 |
| world | 60 | 25 028 |
| server | 32 | 19 354 |
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
| Block entity создаются блоком | 49 | 49 | 100 % | `fn new_block_entity` (50 реализаций) |
| Меню (типы экранов) | 25 | 25 | 100 % | `menu/kinds/*.rs` против `menutypes.json` (18 файлов) |
| Команды | 72 | ~50 ванильных + свои | 100 % ванильных | `command/builtins/*.rs` |
| Геймрулы | 59 | 59 | 100 % | `game_rules.json` |
| Clientbound-пакеты в коде | 121 | 173 | 70 % | `packets/game/c_*.rs` |
| Serverbound-пакеты в коде | 56 | 87 | 64 % | `packets/game/s_*.rs` |

Пояснения:

* «Блоки с собственным классом» — блоки, у которых ванильный класс не сводится к
  базовому `Block` (268 разных классов на 1 286 блоков). Остальные 182 блока
  обслуживает базовое поведение, и это корректно: у них в ванили тоже нет
  переопределений.
* «Меню 25/25» — 25 типов окон покрыты 18 файлами: `generic_9x1`…`generic_9x6`
  это одно меню с разным числом строк.
* Пакеты: 121 clientbound-структура против 173 в протоколе 26.3; 21 из
  существующих не имеет отправителя (§5.1).

---

## 4. Области: что реально работает

Статус «работает» = есть путь исполнения от тика/пакета/команды до эффекта.

### 4.1 Мир и генерация — работает

* `chunk/` — чанки, секции, палитры, границы, тик-расписание, блок-тики
  (`chunk_map/scheduled_ticks.rs`), fluid-тики.
* `worldgen/` — биомы, карверы, фичи, структуры, слои, поверхностные правила,
  `FlatChunkGenerator`, декорация фич по биому.
* Свет: `light/` — небесный и блочный, пересчёт при изменении блока.
* Сохранения: `chunk/data.rs`, `player/data`, `level.toml` (`LevelDataManager`).
* Рейды: `world/raid.rs` (`RaidManager`), тик из `server/run_loop.rs`, подъём
  через `BadOmenMobEffect`.

### 4.2 Блоки и block entities — работает

Все 49 типов block entity создаются своими блоками. Тикер
(`get_block_entity_ticker`) переопределён у 26 `BlockBehavior` из 250:
`beacon`, `beehive`, `bell`, `brewing_stand`, `campfire`, `chest`,
`command_block`, `conduit`, `crafter`, `creaking_heart`, `daylight_detector`,
`end_gateway`, `end_portal`, `hopper`, `jigsaw`, `jukebox`, `lectern`,
`moving_piston`, `potent_sulfur`, `shulker_box`, `spawner`, `structure`,
`hanging_sign`×2, `sign`×2 — плюс печи через макрос `impl_furnace_block!`
(`FURNACE`, `BLAST_FURNACE`, `SMOKER`).

Отдельные механики, проверенные по вызову:

| Механика | Файл | Проверка |
|---|---|---|
| Крышка сундука | `block_entity/entities/chest.rs`, `container/chest_block.rs` | `ChestLidController` тикает, block event 1 при смене `open` |
| Улей | `block_entity/entities/beehive.rs`, `container/beehive_block.rs` | `serverTick`-порт: выпуск пчёл, +1 к `LEVEL_HONEY` |
| Крекинг-харт | `behavior/blocks/building/technical_blocks.rs` | тикер → `spawn_creaking` |
| Сушёный гаст | `behavior/blocks/building/infested_block.rs` | `random_tick` → 5000 тиков → гидратация → гастлинг |
| Икра лягушки | `behavior/blocks/building/infested_block.rs` | `mayPlaceOn` по тегам, `entityInside`, `tick` → 2–5 головастиков |
| Окисление меди | `weathering_block.rs` + `build/weathering.json` | `WeatheringCopper` у 60+ блоков |
| Огонь | `behavior/blocks/portal/fire.rs` | `tick` + планирование в `on_place`, `ignite_odds`/`burn_odds` из `flammable.json` (175 блоков) |
| Лодка на лилии | `vegetation/lily_pad_block.rs` | `mayPlaceOn` по `SUPPORTS_LILY_PAD`, `entityInside` ломает |

### 4.3 Сущности и ИИ — работает

* 161 тип сущности с собственным классом, 143 класса реализованы.
* Цели ИИ: `entity/ai/goal/` — 60+ целей: `FloatGoal`, навигация, атака,
  паника, следование, размножение.
* Передвижение: `default_travel` → `should_travel_in_fluid` → `travel_in_fluid`
  (`living_entity.rs`), поэтому мобы плавают, а не тонут; `travel_fall_flying`
  для элитр.
* Бой: `CombatTracker` портирован целиком — вход в бой при
  `!inCombat && isAlive() && source.entity instanceof LivingEntity`, выход по
  таймауту 300/100 тиков и в смерти (`recheckStatus` в `LivingEntity.die` и
  `ServerPlayer.die`), длительность уходит в `CPlayerCombatEnd`.
* Эндер-дракон: `EnderDragonFight`-состояние (живой дракон, первый килл, арм
  респавна, прогресс босс-бара) в `world/dragon_fight.rs`; дракон регистрируется
  сам и кормит босс-бар, `setDragonKilled` открывает портал выхода, кладёт яйцо и
  снимает бар, кристаллы бьют лучом в дракона.
* Ездовые животные: лошадь, осёл, мул, лама, караванная лама, верблюд —
  седло/упряжь, управление по взгляду, прыжок с зарядом (включая `dash` у
  верблюда), экран инвентаря.
* Жители: профессии, торговля, `work_at_poi`, сон в кровати.
* Рейды, патрули, визер, эндермен, крипер, слаймы — с тиками.

### 4.4 Инвентарь и меню — работает

* 18 файлов меню покрывают 25 типов окон: сундук/двойной сундук, бочка,
  шалкер, эндер-сундук, крафт 2×2 и 3×3, крафтер, печи (3), зельевар,
  наковальня, чары, точило, кузнечный стол, ткацкий станок, картография,
  камнерез, торговля, маяк, лектерн, раздатчик/выбрасыватель, воронка,
  сундук лодки и экран ездового животного.
* Клики, шифт-клики (маршруты `MenuBuilder::route`), drag, quick-craft,
  swap-хотбар, `set_carried`, творческий режим.

### 4.5 Протокол и сеть — работает

* `steel-protocol`: 121 clientbound + 56 serverbound структур, кодирование через
  `steel-macros` (`ClientPacket`, `WriteTo`, `ReadFrom`).
* `steel-login`: handshake, status (ping с data-URI иконкой), login, config,
  шифрование, сжатие.
* `player/connection/` — отправка, бандлы, отключение, `ScheduledPlayPacket`.

### 4.6 Серверные системы — работает

* Команды: 72 файла, включая `execute`, `data`, `scoreboard`, `team`,
  `bossbar`, `worldborder`, `schedule`, `function`, `advancement`, `loot`.
  `/reload` перечитывает `groups.toml` и геймрулы `level.toml` (async, через
  `ReloadSuspension`).
* Права: `permission/` — группы, узлы `minecraft.command.*`, `groups.toml`,
  hot-reload через `PermissionGroupManager::reload_from_store`.
* RCON, whitelist, ban/banip, `setidletimeout`, `transfer`.
* Планировщик тиков, TPS-бар, `save`, `stop`.

---

## 5. Пакеты, у которых нет отправителя

### 5.1 Clientbound без `send_packet` (21 из 121)

Проверено поиском имени структуры вне `steel-protocol`.

| Пакет | Почему не отправляется | Что нужно |
|---|---|---|
| `CMountScreenOpen` | закрыто (`993e8d3`) | — |
| `CPlayerCombatEnter`/`CPlayerCombatEnd` | отправляются из `Player` (проверено вызовом) | — |
| `CMoveMinecartAlongTrack` | ваниль шлёт его **только** при `FeatureFlags.MINECART_IMPROVEMENTS` → `NewMinecartBehavior`; Steel реализует старое поведение вагонеток, флагов фич нет | реализация `NewMinecartBehavior` вместе с флагами фич |
| `CPostEffects` | нет хранилища пост-эффектов на игроке | ordered set + NBT `post_effects` (Paper делает так) |
| `CStartConfiguration` | нет обратного перехода play → config | путь `switchToConfig` |
| `CConfigPing`, `CStoreCookie` | нет сценария transfer/куки | `transfer`-путь |
| `CShowDialog`, `CClearDialog`, `CCodeOfConduct`, `CCustomReportDetails`, `ChatCompletionAction` | нет подсистемы диалогов | отдельная фича |
| `CDebugBlockValue`, `CDebugChunkValue`, `CDebugEntityValue`, `CDebugEvent` | нет debug-палоуи | `/debug` |
| `CAddTransientBlock` | нет «временного» блока-превью | фича |
| `CDeleteChat`, `CResetChat` | нет серверного управления чатом | фича |
| `CGameTestHighlightPos` | нет game-test раннера | раннер |
| `CPlayerRotation` | тело игрока синкается через `CEntityPositionSync` | — |
| `CResourcePackPop` | нет стека ресурспаков | `resource-pack` в `server.toml` |

### 5.2 Serverbound

56 структур; все, кроме `HashedPatchMap` и `TestInstanceBlockData`, разобраны в
`player/connection/java.rs`. Эти два — клиентские подтверждения без серверного
эффекта.

---

## 6. Закрыто в этой сессии

| Коммит | Разрыв | Суть |
|---|---|---|
| `84747fc` | `DragonFight` не был связан с драконом | `updateDragon`/`setDragonKilled`/`tryRespawn` + босс-бар в `BossBarRegistry` + портал выхода и яйцо |
| `84747fc` | икра лягушки | точный `mayPlaceOn` по `SUPPORTS_FROGSPAWN`, `entityInside`, `updateShape` по всем соседям |
| `84747fc` | теги `SUPPORTS_FROGSPAWN`/`SUPPORTS_LILY_PAD` отсутствовали в реестре | `steel-registry/build_assets/vanilla_tags/` + мерж в билдерах тегов |
| `84747fc` | выдуманный `reset_combat` | удалён; вместо него настоящий `recheckStatus()` в `die` |
| `631b64f` | `CMoveMinecartAlongTrack` писал `f32` вместо rotation-byte и терял `weight` | формат приведён к ванильному кодецу |
| `64c3863` | `/reload` ничего не перечитывал | `load_groups` + `reload_from_store` + `reload_game_rules`, async-саспеншн |

Ранее в ветке: `e74ed3e` (сундук ламы/осла), `993e8d3`/`da11bda`
(`CMountScreenOpen` и экран ездового), `a5e95fa` (прыжок ездовых),
`ebf4efc` (`CombatTracker`), `9b06a06` (икра лягушки: hatch/on_place/tick).

Что проверялось и оказалось **корректным** (чтобы не переделывать):

* `ChorusPlantBlock`, `KelpPlantBlock`, `FrostedIceBlock`, `LeafLitterBlock` —
  совпадают с ванильными константами и логикой.
* Окисление меди: 112 блоков с `randomly_ticking`, покрыты `WeatheringCopper`.
* `ScheduledPlayPacketKind` — 63 варианта, все обработаны в арме.
* Все no-op методы `BlockBehavior` (190 пустых тел) — ванильные хуки, у каждого
  есть вызов.
* `CMoveMinecartAlongTrack` без отправителя — корректное состояние (§5.1).

---

## 7. Что остаётся нереализованным

Отсортировано по влиянию на игрока.

| # | Область | Файл/место | Состояние |
|---|---|---|---|
| 1 | Фазы полёта дракона (10 `Phase`, дыхание, заход на портал) | `entity/entities/mobs/bosses/ender_dragon.rs` | у дракона обычный набор целей (`FloatGoal`, `MeleeAttackGoal`, `WaterAvoidingRandomStrollGoal`, `LookAt*`); `phase` синкается, но не продвигается |
| 2 | Респавн-анимация дракона (20 тиков лучей) и кольцо гейтвеев | `world/dragon_fight.rs` | арм и сброс кристаллов есть, анимации и 4×4 геометрии гейтвея нет |
| 3 | Пост-эффекты (`CPostEffects`) | — | нет хранилища на игроке |
| 4 | Датапаки с диска | `command/builtins/datapack.rs` | только встроенный `minecraft` |
| 5 | Реестры в `/reload` | `server/mod.rs` | блоки/предметы/биомы запекаются на старте, нужен рестарт |
| 6 | Ресурспаки (`CResourcePackPush`/`Pop`) | `steel-login`, `steel/src/config/server.rs` | нет поля URL и стека |
| 7 | Game-test раннер | — | блоки test есть, раннера нет |
| 8 | Диалоги (`CShowDialog`) | — | подсистемы нет |
| 9 | `NewMinecartBehavior` + флаги фич | `entity/.../vehicles/minecart_behavior.rs` | реализовано старое поведение |
| 10 | Обратный переход play → config | `player/connection/java.rs` | `CStartConfiguration` не шлётся |
| 11 | Функции `/function` | `command/builtins/function.rs` | каждая строка исполняется как `Console`, контекст отправителя теряется |
| 12 | `work_at_poi` у части профессий | `entity/.../villager.rs` | часть POI не назначена |

---

## 8. Итоговые оценки по областям

Взвешено по объёму кода и по числу ванильных механик; «работает» = §4.
Оценки округлены и помечены как оценки.

| Область | Готовность | Обоснование |
|---|---:|---|
| **Реестры и данные** | **≈ 100 %** | все таблицы 26.3 извлечены; 49/49 BE, 25/25 типов меню, 59/59 геймрулов, недостающие теги добавлены |
| **Инфраструктура** (чанки, свет, сохранения, сеть, протокол) | **≈ 94 %** | всё ядро работает; минусы — датапаки с диска, реестры в `/reload`, 21 clientbound без отправителя |
| **Контент: блоки** | **≈ 93 %** | 1 104/1 286 своих классов, все BE создаются, икра/лилия/огонь доведены до ванили |
| **Контент: предметы** | **≈ 90 %** | 1 401/1 658 своих классов, остальное — корректный базовый `Item` |
| **Контент: сущности** | **≈ 90 %** | 161/161 с классами, дракон связан с боем; недоделаны фазы полёта и респавн-анимация |
| **Живой мир** (ИИ, бой, редстоун, спавн) | **≈ 82 %** | ИИ, бой (полный `CombatTracker`), рейды и спавн работают; нет фаз дракона |
| **Инвентарь и меню** | **≈ 97 %** | 25/25 типов, клики/шифт-клики/drag |
| **Серверные системы** (команды, права, scoreboard, админ) | **≈ 92 %** | 72 команды, RCON, права, бан-листы, рабочий `/reload`; минус — датапаки с диска |
| **ИТОГО по ядру** | **≈ 91 %** | — |

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
    print(k, len(d[k]), 'classes:', len({e.get('class') for e in d[k]}))
EOF

# block entity: зарегистрированы и создаются ли блоком
grep -c 'vanilla_block_entity_types::' steel-core/src/block_entity/registry.rs
grep -rn 'fn new_block_entity' --include=*.rs steel-core/src/behavior/blocks | wc -l

# пакеты без отправителя
python3 - <<'EOF'
import os, re, subprocess
for f in sorted(os.listdir('steel-protocol/src/packets/game')):
    if not (f.startswith('c_') and f.endswith('.rs')):
        continue
    s = open('steel-protocol/src/packets/game/' + f, encoding='utf8').read()
    m = re.search(r'pub struct (C\w+)', s)
    if not m:
        continue
    name = m.group(1)
    if subprocess.run(['grep', '-rq', name, '--include=*.rs', 'steel-core/src',
                       'steel-login/src', 'steel/src']).returncode != 0:
        print(name)
EOF
```
