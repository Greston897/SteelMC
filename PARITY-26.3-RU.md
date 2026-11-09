# SteelMC ↔ Minecraft Java 26.3: аудит паритета серверного ядра

**Дата:** 9 ноября 2026
**Проверяемая версия:** `0.17.0+mc26.3` (ветка `arena/8ff5adc9-steelmc`: старт `6eed266`, текущий `4e906af`)
**Эталон:** Minecraft Java Edition **26.3 «Wilderness Bound»** (релиз 15.09.2026, data pack version 121.0)
**Объём кода:** ≈526 600 строк Rust (без generated), 3 986 тестов

---

## 1. Короткий ответ

| Срез | Оценка паритета |
|---|---|
| **Серверное ядро в целом (взвешенно)** | **≈58 %** (реалистичный диапазон 55–62 %); на старте ветки было ≈56 % |
| Инфраструктура и мир: протокол, чанки, свет, сохранения, мирогенерация | ≈74 % (мирогенерация отдельно ≈92 %) |
| Контент-механики: блоки, block entities, предметы, меню, крафт | ≈68 % (было ≈65 %) |
| Живой мир: сущности, ИИ, бой, редстоун | ≈35 % (было ≈32 %) |
| Серверные системы: команды, права, scoreboard, достижения, админ, плагины | ≈37 % |

Проще говоря: **«мир и фундамент — почти ванильные, живой мир и социальные системы — в основном отсутствуют»**.
Это совпадает с самоописанием проекта («survival gameplay is incomplete», «only a small number of entities have meaningful behavior») и с официальным трекером, где блоки закрыты на 81 %, но мобы — почти нет.

### 1.1 Динамика: было → стало (ветка `arena/8ff5adc9-steelmc`)

Точная метрика есть только для слоя предметов — «предметы, у которых есть зарегистрированное поведение» (1 658 предметов в 26.3).
Остальные проценты — экспертные оценки, они пересчитаны после каждого пакета.

| Срез | На старте ветки (`6eed266`) | Сейчас (`4e906af`) | Δ |
|---|---|---|---|
| Предметы с поведением | 1 286 / 1 658 = **77,6 %** | 1 372 / 1 658 = **82,8 %** | **+86 предметов, +5,2 п.п.** |
| Классы без механик (кроме базового `Item`) | 18 классов / 115 предметов | **7 классов / 29 предметов** | −11 классов, −86 предметов |
| `#[item_behavior]` структур в коде | 39 | **58** | +19 |
| `#[entity_behavior]` структур в коде | 29 | **39** | +10 |
| Типы сущностей с фабрикой/поведением | 29 / 161 ≈ 18 % | **39 / 161 ≈ 24 %** | +10 типов |
| Тестов в коде | 3 142 | **3 986** | +844 |
| Строк Rust (без `generated`) | ≈517 000 | ≈526 600 | +9 600 |
| Серверное ядро в целом (взвешенно) | ≈56 % | **≈58 %** | +2 п.п. |

Пакеты по порядку (по срезу «предметы с поведением»):

| Пакет | Предметов | Покрытие после |
|---|---|---|
| Подушки (`CushionItem`, 16 цветов) | 16 | 79,9 % |
| Картины (`HangingEntityItem` + `Painting`-сущность) | 1 | 79,9 % |
| Стрелы + лук (`Arrow`, `SpectralArrow`, `ArrowItem`, `SpectralArrowItem`, `BowItem`) | 3 | 80,1 % |
| Арбалет + зельевые стрелы (`CrossbowItem`, `Arrow#doPostHurtEffects`) | 1 | 80,2 % |
| Кристалл Края + шаблоны кузнеца + фрагмент диска | 21 | 81,4 % |
| **Лодки и плоты (`BoatItem`, 4 класса сущностей, пакет `paddle_boat`)** | **22** | **82,8 %** |

Что осталось до 100 % слоя предметов: `MapItem` 17 + `EmptyMapItem` 1 (нужно состояние карты и пакет `map_item_data`),
`MobBucketItem` 7 (нет сущностей рыб/аксолотля/головастика), `KnowledgeBookItem` 1 (книга рецептов),
`ArmorStandItem` 1 (нет сущности стойки), `TridentItem` 1 (нет снаряда-трезубца), `WindChargeItem` 1 (нет сущности заряда ветра).

Важно: рост «ядра в целом» на 2 п.п. — это не только предметы. Лодки дали серверную физику транспорта и новый
serverbound-пакет, стрелы/лук/арбалет закрыли основную часть дальнего боя, картины и подушки — сущности-декорации.

**Самые крупные дыры (по влиянию на игру):**

1. Мобы и естественный спавн — 8 живых мобов из ~95, моб-кап/спавн-цикл не реализованы.
2. Контейнеры — нет сундуков/ловушек-сундуков/воронок/раздатчиков/выбрасывателей/варочных стоек.
3. Бой дальнего боя — лук, арбалет и стрелы уже есть; нет трезубца, винд-чарджа и элитр, щит в работе.
4. Деревни, торговля, рейды — отсутствуют как система.
5. Боссы (дракон Края, Иссушитель) и финалы измерений.
6. Карты и исследовательские карты (включая новые из 26.3).
7. Scoreboard/team/bossbar, ~45 ванильных команд, whitelist/ban/RCON.
8. Структуры: 9 типов из ~22; нет abandoned camps, trial chambers, ancient city, деревень, бастионов, end city.
9. Плагины/интеграции: только ранний `steel-flint`, совместимости с Paper/Bukkit/Fabric/NeoForge нет.

---

## 2. Как считались проценты

Проценты — **не автоматическая метрика**, а взвешенная экспертная оценка. Там, где есть счётчики, они взяты из кода и данных:

| Источник | Что даёт |
|---|---|
| `steel-registry/build_assets/*.json` (извлечённые данные 26.3) | 1 286 блоков / 268 классов блоков, 1 658 предметов / 66 классов, 161 тип сущности / 143 класса, 49 типов block entity, 40 эффектов, 46 зелий, 144 clientbound + 69 serverbound play-пакетов |
| Код SteelMC | 223 `#[block_behavior]`, 39 `#[item_behavior]`, 29 `#[entity_behavior]`, 25 зарегистрированных block entity, 89 clientbound + 38 serverbound play-пакетов, 41 корневая команда, 10 модулей меню из 25 ванильных типов |
| [Официальный трекер SteelMC](https://steelmc.dev/tracker) (nightly) | 1 101 запись: 888 реализовано (81 %), 143 в работе (13 %), 70 в планах (6 %) |
| [minecraft.net: 26.3](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-3), [minecraft.wiki](https://minecraft.wiki/w/Java_Edition_26.3) | Состав целевого релиза |
| README, AGENTS.md, открытые issues и майлстоун `0.17.1` | Официально признанные пробелы и ближайший план |

Веса областей (сумма 100) отражают вклад в «ванильный серверный геймплей», а не размер кода:
протокол 7, чанки/свет/сохранения 8, мирогенерация 10, структуры 5, блоки 13, block entities 6,
предметы 8, меню/крафт 6, сущности/ИИ 14, бой/эффекты 9, редстоун 4, достижения/статистика 2,
scoreboard/карты 1, команды 3, админ/интеграции 2, плагины 2.

---

## 3. Сводная матрица

| # | Область | Вес | Покрытие | Что уже готово | Ключевые пробелы |
|---|---|---|---|---|---|
| 1 | Реестры и данные 26.3 | — | ≈100 % | Все блоки/предметы/сущности/эффекты/зелья/чары/рецепты/лут-таблицы/достижения/звуки/теги из датапака 26.3, codegen в типизированный Rust | Рантайм-датапаки не поддерживаются (осознанно); редкие новые реестры (dialogs, timelines, world_clocks) не имеют поведения |
| 2 | Сеть и протокол | 7 | ≈55 % | Логин, шифрование, сжатие, конфиг, чат, чанки, инвентарь, движение, большинство игровых пакетов (89 cb / 38 sb) | Нет 55 cb и 31 sb play-пакетов: `update_recipes`, `merchant_offers`, `map_item_data`, `boss_event`, `set_objective`/`set_score`/`set_player_team`, `waypoint`, `show_dialog`, `debug/*`, `test_instance_block_status`, `move_minecart_along_track`, `player_rotation`, `mount_screen_open`, `open_book`, `post_effects`, `projectile_power` |
| 3 | Чанки, свет, сохранения, производительность | 8 | ≈88 % | Генерация/загрузка/сохранение чанков, освещение, палитры, секции, асинхронная отправка чанков, бенчмарки (10 201 чанк за 3.98 с median на 9950X) | Нет полной parity-гарантии по всем состояниям; сериализация проверена не для всех типов сущностей и block entity; схемы миграции нет (осознанно) |
| 4 | Мирогенерация (terrain/biomes/carvers/features) | 10 | ≈92 % | 7 500 чанков block-for-block против ванильного референса; 53 типа фич; poplar-деревья 26.3; биом dappled_forest в multi-noise параметрах | Не все фичи/декораторы имеют 1:1 паритет (issue «Feature placement vanilla parity»); нет спавна сущностей при генерации |
| 5 | Структуры и храмовые пулы | 5 | 9 / ~22 ≈ **41 %** | buried_treasure, desert_pyramid, fortress, jungle_temple, mineshaft, ocean_monument, ruined_portal, stronghold, swamp_hut + template-пайплайн | Нет village, abandoned_camp (26.3), ancient_city, trial_chambers, bastion, end_city, igloo, mansion, pillager_outpost, shipwreck, ocean_ruin, trail_ruins, nether_fossil |
| 6 | Блоки и blockstates | 13 | **81 %** (трекер) / 223 поведения на 268 классов | 888 записей реализовано, 143 в работе; редстоун, растения, двери, лестницы/плиты, шерсть/бетон, straw bed, shelf mushroom, red shrub | 70 классов в планах: **chest/trapped_chest**, dispenser, dropper, hopper, brewing_stand, crafter, lectern, loom, stonecutter, smithing_table, cartography_table, sculk*, calibrated_sculk_sensor, trial_spawner, vault, decorated_pot, jigsaw, structure_block, command_block, creaking_heart, copper_golem_statue, shelf (13), leaves-база, nylium, infested, froglights и др. |
| 7 | Block entities | 6 | 25 / 49 ≈ **51 %** | furnace/smoker/blast furnace, barrel, beacon, beehive, bell, brushable, campfire, chiseled_bookshelf, comparator, conduit, daylight detector, enchanting table, ender chest, end portal/gateway, sign/hanging sign, jukebox, mob spawner, piston, potent sulfur, shulker box, skull | **chest/trapped_chest**, dispenser, dropper, hopper, brewing stand, banner, structure/jigsaw/command block, lectern, sculk sensor/catalyst/shrieker, shelf, decorated pot, crafter, trial spawner, vault, creaking heart, copper golem statue |
| 8 | Предметы и компоненты | 8 | ≈75 % (**1 372 / 1 658 = 82,8 %** предметов с поведением) | 1 658 предметов и компоненты данных, прочность, еда, зелья, вёдра, инструменты/броня через компоненты, mace, shield (частично), spawn eggs, dye, banner, minecart, sign, подушечки (16), картины, лук/арбалет/стрелы (в т.ч. зельевые), кристалл Края, шаблоны кузнеца, **лодки и плоты (22)** | Нет поведения: `MapItem` (17) + `EmptyMapItem` (1), `MobBucketItem` (7), `KnowledgeBookItem`, `ArmorStandItem`, `TridentItem`, `WindChargeItem`; `BundleItem` (17) перенесён ранее; элитры нет |
| 9 | Инвентарь, меню, крафт | 6 | 10 / 25 ≈ **40 %** | inventory, crafting (2x2/3x3), furnace/smoker/blast furnace, anvil, enchanting, grindstone, beacon, ender chest, shulker box, универсальные 9x1–9x6; chest-меню есть, но без блока | Нет типов меню: hopper, brewing_stand, smithing, stonecutter, loom, cartography_table, merchant, crafter_3x3, lectern; нет книги рецептов (`update_recipes`, `recipe_book_*`) |
| 10 | Сущности, ИИ, спавн | 14 | 39 / 161 ≈ **24 %**; живые мобы 8 / ~95 ≈ **8 %** | AI-фундамент: goals, navigation, pathfinder, sensing, targeting, physics, equipment, effects, маунты, лианы, item/XP orb, TNT, кристалл Края, снаряды (снежок, яйцо, жемчуг, око, зелья, фейерверк, удочка, стрелы/спектральные стрелы), вагонетки (2), **лодки/плоты и сундук-лодки (4 класса)** | 122 типа без реализации: все враждебные (zombie, skeleton, creeper, spider, enderman, witch, slime, pillager…), животные кроме 6 (chicken, cow, mooshroom, pig, sheep, squid) и bat, villagers, iron/snow golem, bosses, остальные вагонетки, трезубец/винд-чардж, sulfur cube (26.2); **нет моб-капа, спавн-цикла и естественного спавна** (7 spawn rules) |
| 11 | Бой, урон, эффекты, зелья | 9 | ≈55 % | Типы урона, атрибуты, броня/toughness, поглощение урона бронёй, knockback, часть sweep, enchantment helper (защита), 13 активных эффектов, зелья/всплеск/туман, голод/регенерация/истощение | Нет combat tracker и death messages (PR), критов/кулдауна атаки полностью, щита (PR), трезубца, mace-механик полностью, **атрибутных эффектов** (speed/strength/jump/resistance и др. не накладывают модификаторы), варки зелий, тотема, элитр |
| 12 | Редстоун и механизмы | 4 | ≈65 % | wire, repeater, comparator, torch, lever, buttons, pressure plates (15), observer, note block, daylight detector, piston head/base (в работе), TNT (частично), target, tripwire, rails, minecart physics | **piston base в работе**, dispenser/dropper/hopper/crafter отсутствуют, sculk-сенсоры, copper bulb/chest частично, нет полной 1:1 модели обновлений и «quasi-connectivity»-краёв |
| 13 | Достижения и статистика | 2 | ≈60 % | Данные всех достижений, `PlayerAdvancements`-хранилище, критерии/триггеры, `/advancement`, статистика (vanilla stat types + custom), начисление части статов | Мало фактически срабатывающих триггеров, нет выдачи `ITEM_CRAFTED` (issue), формат сохранения не ванильный JSON (issue), нет динамической перезагрузки |
| 14 | Scoreboard, команды-интерфейсы, карты | 1 | ≈20 % | Серверная модель scoreboard/team/objective, title/actionbar/subtitle, tab list, чат | **Нет пакетов** `set_objective/set_score/set_display_objective/set_player_team/reset_score`, нет `/scoreboard`, `/team`, `/bossbar`; нет карт (`map_item_data`), waypoint-системы, диалогов |
| 15 | Команды | 3 | ≈40 % | 41 корневая команда (43 с алиасами `xp`, `tp`): advancement, clear, damage, defaultgamemode, difficulty, enchant, execute, experience, fill, gamemode, gamerule, give, kill, list, locate, op/deop, playsound, return, seed, setblock, spawnpoint, setidletimeout, setworldspawn, stop, stopsound, summon, tag, teleport, tellraw, tick, time, title, weather, worldborder, transfer + steel-специфичные (domain, fly, perms, invsee, swing, version) | Нет ≈45 ванильных: attribute, ban/ban-ip/banlist/pardon, bossbar, clone, data, datapack, debug, dialog, effect, fillbiome, forceload, function, help, item, jfr, kick, loot, me/msg/tell/tm/w, particle, perf, place, publish, raid, random, recipe, reload, ride, save-all/off/on, say, schedule, scoreboard, spectate, spreadplayers, stopwatch, team, teammsg, trigger, warden_warning, waypoint, whitelist |
| 16 | Права, админ, интеграции | 2 | ≈30 % | Своя система permissions/групп (лучше ванильных op-level), конфиг, server links, motd/favicon, online-mode | Нет whitelist, ban-листов, RCON, query, Minecraft Server Management Protocol (появился в 26.3 вместо лог-строк), нет консольных метрик/`/perf` (issue) |
| 17 | Плагины и API | 2 | ≈15 % | `steel-flint` (ранний), keyed-downcasting, регистрация блоков/предметов/команд с прицелом на ABI | Нет стабильного plugin API, нет Bukkit/Paper/Fabric/NeoForge-совместимости, нет рантайм-датапаков |

---

## 4. Детали по областям

### 4.1 Что сделано сильно

- **Реестры и данные.** Всё, что есть в 26.3, извлечено и скомпилировано в типизированный Rust: 1 286 блоков, 1 658 предметов, 161 сущность, 40 эффектов, 46 зелий, чары, рецепты, лут-таблицы, достижения, теги, звуки, частицы. Это фундамент, которого нет у большинства «с нуля» серверов.
- **Мирогенерация.** Заявлен и подтверждён паритет 7 500 чанков (по 2 500 в каждом измерении). Новый контент 26.3 в генерации уже есть: dappled_forest в multi-noise, poplar trunk/foliage placers.
- **Сеть и жизненный цикл игрока.** Логин (online-mode, шифрование, compression), конфиг-фаза, вход в мир, движение, коллизии, инвентарь, чат с подписями, команды с бригадир-подобным деревом и саджестами.
- **Блоки.** 223 поведения, 81 % записей по трекеру; редстоун, растительность, двери/лестницы/плиты/заборы, шерсть/бетон (включая новые stairs/slabs 26.3), straw bed, shelf mushroom, red shrub.

### 4.2 Где разрыв критичен

- **Живой мир.** 8 мобов (bat, chicken, cow, mooshroom, pig, sheep, squid, endermite) — и это без естественного спавна: спавн-цикла и моб-капа нет, despawn реализован частично. Без мобов нет выживания: еда, лут, бой, деревни, рейды, боссы.
- **Контейнеры.** Сундук — центральный блок игры — в статусе Planned. Есть меню сундука, но нет поведения блока и block entity. Воронки/раздатчики/выбрасыватели/варочные стойки — тоже Planned.
- **Дальний бой.** Лук, арбалет, обычные и спектральные стрелы, зельевые стрелы и кристалл Края уже есть; нет трезубца, винд-чарджа и элитр, щит в открытом PR.
- **Социальные системы.** Деревни, торговля, рейды, iron golem, wandering trader — ноль.
- **Боссы и финалы.** `mobs/bosses/` пуст: дракона Края и Иссушителя нет; end city/bastion тоже.
- **Интерфейсы.** Scoreboard-модель есть, но пакетов и команд нет → игрок её не видит. Карт нет вообще.
- **Команды.** ~40 % корней; нет даже `/help`, `/effect`, `/particle`, `/say`, `/msg`, `/kick`, `/save-all`.
- **Админ/интеграции.** Нет whitelist/ban/RCON/query; в 26.3 Mojang прямо предлагает Server Management Protocol вместо лог-строк — его нет.

---

## 5. Контент 26.3 конкретно

| Новинка 26.3 | Статус в SteelMC |
|---|---|
| Dappled Forest (биом) | ✅ есть в multi-noise параметрах и в тестах генерации |
| Poplar trees / fallen poplar | ✅ trunk/foliage placers реализованы |
| Red/orange/yellow poplar leaves, полный wood set | ✅ как обычные блоки (RotatedPillar/Stair/Slab/Fence/Door/Sign и т.д.) |
| Shelf Mushroom | ✅ `ShelfMushroomBlock`; shelf-блоки 26.1 (13) — ❌ Planned |
| Red Shrub | ✅ |
| Straw Bed | ✅ блок + sleep-логика (без установки спавна) |
| Wool/Concrete stairs & slabs (все цвета) | ✅ автоматически через общие StairBlock/SlabBlock |
| **Abandoned Camps** | ❌ структуры нет (есть только item `abandoned_camp_map`) |
| **Новые Explorer Maps** | ❌ карт как механики нет |
| **Cushions (16 цветов)** | ❌ только предметы, нет блока/сидения/поведения |
| Data-driven pottery sherds, data-driven potion recipes | ⚠️ данные есть, поведение частично (decorated pot — Planned) |
| Server Management Protocol (тех. изменение) | ❌ |

Плюс долги по предыдущим дропам 26.x:
- 26.1 Tiny Takeover: baby-мобы (нет мобов → нет baby), golden dandelion (предмет есть, механики нет), craftable name tag (рецепт есть, поведение name tag есть).
- 26.2 Chaos Cubed: sulfur caves (блоки есть, биом в данных есть), **sulfur cube — нет**, cave spider natural spawn — нет, cinnabar/sulfur sets — как обычные блоки ✅.

---

## 6. Этапы (что не реализовано и в каком порядке закрывать)

Порядок — по зависимостям и по приросту играбельности. «Целевой паритет» — кумулятивная оценка по той же шкале.

### Этап 0. Закрыть текущий майлстоун 0.17.1 (фундамент)
**Зачем:** без этого следующие этапы упрутся в отсутствие фундамента.
- Feature placement vanilla parity (мирогенерация → 94 %)
- Переработка loot + container loot + сундук (data-driven лут в контейнерах)
- CombatTracker + локализованные death messages
- Щит (открытый PR), остаток работы по взрывам
- Foundations: hostile mob, projectile, player combat
- `/perf`, сохранение достижений в ванильном JSON, `ITEM_CRAFTED`

**Целевой паритет:** ≈58 % · **Оценка:** 3–5 недель

### Этап 1. Контейнеры и меню (закрыть «ящики»)
- Chest, trapped chest, double chest; hopper (+ transfer-логика), dispenser, dropper, crafter
- Brewing stand + варка зелий (46 зелий уже в данных)
- Smithing table, stonecutter, loom, cartography table, lectern, decorated pot, shelf
- Меню: hopper, dispenser, dropper, brewing, smithing, stonecutter, loom, cartography, crafter, lectern
- Книга рецептов: `update_recipes`, `place_ghost_recipe`, `recipe_book_*`

**Целевой паритет:** ≈61 % · **Оценка:** 6–8 недель

### Этап 2. Бой и снаряжение
- Кулдаун атаки, криты, sweep, knockback-формулы 1:1, combat tracker
- Лук, арбалет, стрелы (включая tipped/spectral), трезубец, mace
- Щит (блокирование, отражение), броня (полные формулы), прочность/починка/наковальня
- Эффекты через атрибуты (speed, slowness, strength, jump, resistance, haste, luck…)
- Тотем бессмертия, элитры/планирование, elytra-полёт

**Целевой паритет:** ≈64 % · **Оценка:** 6–8 недель

### Этап 3. Мобы I — враждебные + спавн
- Спавн-цикл, моб-кап по категориям, despawn, light/sky-light проверки, spawn placements
- Zombie (+ variants, conversion), skeleton (+stray/bogged), creeper, spider (+cave spider 26.2), enderman, witch, slime/magma, drowned, husk
- AI goals: melee/ranged attack, target selection, avoidance, door breaking, sun burning
- Лут мобов через loot tables, очки опыта, эффекты при смерти

**Целевой паритет:** ≈67 % · **Оценка:** 8–10 недель

### Этап 4. Мобы II — животные, деревни, торговля, транспорт
- Breeding/ageable, tamable (wolf/cat/horse), shearable, milkable, chicken egg
- Villager: профессии, POI, рабочие станции, торговля (merchant menu + `merchant_offers`), iron golem
- Raids: bad omen → рейд, волны, hero of the village
- Лодки (22 типа) + остальные вагонетки, saddle/armor, leash (есть база)

**Целевой паритет:** ≈71 % · **Оценка:** 10–12 недель

### Этап 5. Боссы и измерения
- Ender dragon: фазы, кристаллы, bossbar, портал, яйцо, respawn
- Wither: фазы, черепа, разрушение
- End city + корабль, bastion remnant, fortress-лут, nether fossil
- Портал-механики: связанность, кулдауны, spectator-телепорт (26.3)

**Целевой паритет:** ≈72 % · **Оценка:** 5–7 недель

### Этап 6. Структуры и новый контент
- Abandoned camps (26.3), village (5 типов), trial chambers + vault + trial spawner, ancient city + sculk-система (block entities), igloo, mansion, pillager outpost, shipwreck, ocean ruins, trail ruins
- Cushions (блок + сидение + статистика), shelf (13), copper golem statue, creaking heart, dried ghast
- Sulfur cube (26.2) и его ~12 архетипов
- Карты: MapItem, `map_item_data`, explorer maps, люнные карты, banner markers

**Целевой паритет:** ≈75 % · **Оценка:** 10–14 недель

### Этап 7. Интерфейсы и серверные системы
- Scoreboard/team/bossbar: пакеты + `/scoreboard`, `/team`, `/bossbar`
- Оставшиеся ~45 команд (`/effect`, `/particle`, `/clone`, `/data`, `/loot`, `/item`, `/recipe`, `/schedule`, `/function`, `/reload`, `/save-*`, `/kick`, `/msg`, `/say`, `/help`, `/ride`, `/spreadplayers`, `/spectate`, `/attribute`…)
- Диалоги (`show_dialog`, `/dialog`), waypoints (26.x), `player_rotation`, `post_effects`, `mount_screen_open`, `open_book`
- Whitelist, ban/pardon, RCON, query, Minecraft Server Management Protocol
- Debug-пакеты (`debug/*`, `debug_sample`), test blocks

**Целевой паритет:** ≈79 % · **Оценка:** 8–10 недель

### Этап 8. Плагины, API, длинный хвост паритета
- Зрелый `steel-flint`: стабильный API, версионирование, keyed-downcasting, hot reload
- Точки расширения для блоков/предметов/сущностей/меню (NeoForge-совместимые по смыслу)
- Бенчмарки и профилирование под нагрузкой, античит/спам-контроль
- Длинный хвост: точные числа, редкие механики, edge cases, parity-тесты на каждую систему

**Целевой паритет:** ≈82–84 % · **Оценка:** постоянно

> **Почему не 100 %.** Последние 16–18 % — это «длинный хвост»: тысячи мелких правил, редкие взаимодействия и точные константы. Именно так проект уже работает с мирогенерацией (7 500 чанков block-for-block), и такой же подход нужен для остальных систем. Реалистичная цель роадмапа — **~83 % + играбельный survival**, а не формальные 100 %.

---

## 7. Быстрые победы (наибольший прирост за минимум работы)

1. **Сундук + container loot** — разблокирует выживание, структуры и ~5 подсистем.
2. **Спавн-цикл + моб-кап + 5–6 базовых враждебных мобов** — сразу «оживляет» мир.
3. **`/effect`, `/particle`, `/say`, `/msg`, `/help`, `/kick`, `/save-all`** — дешёвые команды, но заметные.
4. **Атрибутные эффекты** (speed/strength/jump/resistance) — один механизм, много эффектов.
5. **Scoreboard/bossbar пакеты** — модель и данные уже есть.
6. **BoatItem + лодки** — 22 типа одним махом (vehicle-база есть).
7. **MapItem + `map_item_data`** — открывает карты и новые 26.3-карты.
8. **Щит + лук/стрелы** — закрывает PvE/PvP-минимум.
9. **Abandoned camps** — единственная крупная новинка 26.3, которой нет.
10. **`/perf` + базовые метрики** — нужны для заявленной производительности.

---

## 8. Оговорки

- Оценки процентов — экспертные; официальный трекер считает только блоки/предметы/сущности/команды и взвешивает по числу зарегистрированных блоков (81 % / 13 % / 6 %), поэтому его «81 %» нельзя переносить на всё ядро.
- Трекер отражает nightly-сборку, а не конкретный коммит; расхождение с кодом ±1–2 класса.
- Часть пакетов и поведений может быть реализована, но не включена в подсчёт из-за другой схемы именования; проценты по протоколу даны по структурам пакетов (`ClientPacket` 89 / `ServerPacket` 38), это слегка занижает реальную картину.
- Мирогенерация — единственная область с формально доказанным паритетом; остальные проценты — по объёму реализованных систем и покрытию данных, без побитового сравнения поведения.
- Список команд сверен с Java Edition 1.21+ и поправками 26.x; в 26.3 набор мог измениться на 1–3 команды.

---

## 9. Приложение: как воспроизвести счётчики

```bash
# поведения
rg -o '#\[block_behavior' steel-core/src/behavior -g '*.rs' | wc -l   # 223
rg -o '#\[item_behavior'  steel-core/src/behavior -g '*.rs' | wc -l   # 39
rg -o '#\[entity_behavior' steel-core/src/entity/entities -g '*.rs' | wc -l  # 29

# block entities
rg -o 'vanilla_block_entity_types::[A-Z_]+' steel-core/src/block_entity/registry.rs | sort -u | wc -l  # 25

# пакеты (структуры с derive ClientPacket/ServerPacket)
python3 - <<'PY'
import re,glob
c=s=0
for f in glob.glob('steel-protocol/src/packets/game/**/*.rs',recursive=True):
    t=open(f).read()
    for m in re.finditer(r'#\[derive\(([^\)]*)\)\][^;{]*?pub struct ([A-Za-z0-9_]+)',t,re.S):
        c += 'ClientPacket' in m.group(1); s += 'ServerPacket' in m.group(1)
print(c,s)  # 89 38
PY

# реестровые объёмы 26.3
python3 - <<'PY'
import json
b=json.load(open('steel-registry/build_assets/blocks.json'))['blocks']
i=json.load(open('steel-registry/build_assets/items.json'))['items']
e=json.load(open('steel-registry/build_assets/entities.json'))
print(len(b), len({x['class'] for x in b}))
print(len(i), len({x['class'] for x in i}))
print(len(e), len({x['class'] for x in e}))
PY
```

Ключевые внешние источники: [26.3 release notes](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-3), [Java Edition 26.3 (wiki)](https://minecraft.wiki/w/Java_Edition_26.3), [SteelMC tracker](https://steelmc.dev/tracker), [SteelMC README](https://github.com/Steel-Foundation/SteelMC).

---

## 9. Обновление: что изменилось после реализации (ветка `arena/8ff5adc9-steelmc`)

**Коммиты:** `58e78da` (баннеры, рог козла, эффекты супа/зелья), `4b02ed4` (operator-only блоки, оптимизация рога).

Метрика — «предмет имеет собственный зарегистрированный класс поведения» (сверка `steel-core/build/classes.json` с фактически зарегистрированными `#[item_behavior]` структурами и их `class = "..."`-оверрайдами):

| Срез | До | После | Δ |
|---|---|---|---|
| Предметов с зарегистрированным поведением | 1261 / 1658 (**76,1 %**) | 1285 / 1658 (**77,5 %**) | +24 предмета, **+1,4 п.п.** |
| Предметов без поведения | 397 | 373 | −24 |
| из них обычный `Item` (в ванили тоже без спец-логики — норма) | 257 | 257 | 0 |
| Реально отсутствующих не-`Item` механик | 140 (8,4 %) | 116 (7,0 %) | −24 |
| Классов с поведением | 39 | 42 | +3 |
| Классов без поведения | 27 | 24 | −3 |

Дополнительно (не меняет проценты покрытия, т.к. эти предметы и так шли через общий путь `Item`):
- **`suspicious_stew`** — теперь применяет эффекты из `suspicious_stew_effects` (раньше эффектов не было вовсе);
- **`ominous_bottle`** — теперь накладывает Bad Omen 120 000 тиков с amplifier из компонента.

Осталось без поведения (373 предмета / 24 класса): `BoatItem` 22, `SmithingTemplateItem` 19, `BundleItem` 17, `MapItem` 17, `CushionItem` 16, `MobBucketItem` 7, `ItemFrameItem` 2 и 15 синглтонов (`CrossbowItem`, `TridentItem`, `ExperienceBottleItem`, `WritableBookItem`, `KnowledgeBookItem`, `WrittenBookItem`, `EmptyMapItem`, `ArmorStandItem`, `EndCrystalItem`, `HangingEntityItem`, `BowItem`, `ArrowItem`, `SpectralArrowItem`, `DebugStickItem`, `DiscFragmentItem`).

Общая взвешенная оценка ядра не меняется в пределах округления: **≈56 % (было ≈56 %)**, предметный срез вырос на 1,4 п.п.

### Дополнение (коммит с `experience_bottle`)
- Добавлены предмет `experience_bottle` (`ExperienceBottleItem`) и сущность `ThrownExperienceBottle` (гравитация 0.07, world event 2002, `3 + rand(5) + rand(5)` опыта через `ExperienceOrbEntity::award`, `discard`).
- Покрытие: **1286 / 1658 = 77,6 %** (было 76,1 %), предметов без поведения 372, не-`Item` без механик 115.

### Дополнение (коммит с `bundle`)
- Реализованы **все 17 котомок** (`BundleItem`: обычная + 16 цветных) — полноценная механика, а не заглушка:
  - компонент `bundle_contents` расширен до ванильного `BundleContentsComponent`: вместимость 1.0, вес `1/maxStackSize`, вложенная котомка стоит `1/16 + свой вес`, пчёлы — 1.0, `max_allowed`, слияние одинаковых стеков, `remove_selected`, `drain_items`, транзиентный `selected_stack_index` (не сериализуется и не влияет на равенство стеков);
  - в `ItemBehavior` добавлены ванильные хуки `on_stack_clicked` / `on_clicked` / `on_item_entity_destroyed`;
  - в `Menu::clicked` добавлен эквивалент `ScreenHandler.handleSlotClick`: предмет на курсоре или в слоте может перехватить клик до стандартной логики (левый клик — вложить, правый — достать выбранный стек, при невалидном выборе — первый стек);
  - ПКМ по котомке — удержание (`use` 200 тиков, анимация `BUNDLE`), содержимое высыпается по одному стеку (первый тик, затем каждые 2 тика после задержки 10), со звуками `item.bundle.insert` / `insert_fail` / `remove_one` / `drop_contents`;
  - уничтожение entity котомки (урон/огонь/лава) высыпает всё содержимое отдельными item-entity, как `ItemEntity.hurt` + `onItemEntityDestroyed` в ванили (закрыт старый TODO в `ItemEntity::hurt`).
- Покрытие: **1303 / 1658 = 78,6 %**; не-`Item` без механик: 105 (было 115).

### Дополнение (коммит с книгами)
- Добавлены пакеты протокола, которых не было: `COpenBook` (S2C `play:open_book`) и `SEditBook` (C2S `play:edit_book`) с ванильными лимитами (100 страниц × 1024 символа, заголовок 32) и тестами кодеков на границах.
- `WritableBookItem` / `WrittenBookItem`: ванильный `use` (статистика `item_used`, открытие экрана чтения только при наличии `written_book_content`; редактор книго-пера клиент открывает сам).
- `ServerPlayNetworkHandler.onBookUpdate`-эквивалент (`Player::handle_edit_book`): запись страниц в `writable_book_content`, подпись книги → `written_book` с `written_book_content` (автор = имя игрока, generation 0, resolved), обновление только для слотов 0–8 и оффхенда 40, синхронизация через меню.
- Покрытие: **1305 / 1658 = 78,7 %**; не-`Item` без механик: 100.
- `KnowledgeBookItem` остаётся заблокированным: в ядре нет книги рецептов (ни `recipe_book_add`, ни разблокировки рецептов), нужен отдельный подсистемный блок.

### Дополнение (коммит с отладочной палкой)
- `DebugStickItem` (1 предмет) реализован полностью по ванилле:
  - в `ItemBehavior` добавлен ванильный хук `can_mine` (эквивалент `ItemStack#canMine`), который вызывается в `BlockBreakingManager::destroy_block` до всего остального, как `ServerPlayerInteractionManager#tryBreakBlock` — то есть палка не ломает блоки даже в креативе;
  - ЛКМ (в `can_mine`) выбирает следующее свойство блока, ПКМ (`use_on`) циклически меняет значение выбранного свойства через `blocks.try_set_property_by_name` с ванильными флагами обновления `18` (`UPDATE_CLIENTS | UPDATE_KNOWN_SHAPE`);
  - выбор хранится в компоненте `debug_stick_state` (`DebugStickStateComponent`): добавлены отсутствовавшие `get`/`with`, значение пишется обратно в стек через `InventoryAccess`;
  - порядок значений = ванильный `Comparable`-порядок (`Property#getValues`), циклирование повторяет `Util.next`/`Util.previous` (заворот на концах, `None`/неизвестное значение → первый/последний), инверсия при `shouldCancelInteraction` (сник);
  - сообщения — в actionbar с ванильными ключами `item.minecraft.debug_stick.empty/select/update`;
  - проверка `isCreativeLevelTwoOp` = креатив + оператор (в Steel уровни оператора моделируются permission-группами).
- Тесты (`debug_stick.rs`, 9 шт.): отказ не-оператору без изменения блока, невозможность сломать блок, перебор значения свойства (и сохранение остальных свойств), обратный перебор при снике, сообщение о блоке без свойств, завороты `cycle` в обе стороны/при `None`/неизвестном значении, ванильный порядок boolean-значений, `DebugStickState::{get,with}` (замена и сохранность остальных записей).
- Покрытие: **1306 / 1658 = 78,8 %**; не-`Item` без механик: 95 (18 классов).

### Дополнение (коммит с рамками для предметов)
- `ItemFrameItem` (2 предмета: `item_frame`, `glow_item_frame`) + доделана сама сущность рамки (в ней стояли заглушки `survives() { true }` и пустой `drop_item`):
  - размещение по ванильному `DecorationItem.useOnBlock`: сущность нужного типа (обычная/светящаяся) на стороне кликнутого блока, копирование `custom_name`, звук `entity.item_frame.place` / `entity.glow_item_frame.place`, game event `ENTITY_PLACE`, расход ровно одного предмета;
  - `canStayAttached` (настоящий): опора за рамкой должна быть `isSolid`, объём рамки должен быть свободен (коллизии блоков) и в слоте не должно быть другой висящей сущности с тем же направлением; при неудаче возвращается `CONSUME`, как в ванилле;
  - `survives()` теперь вызывает эту проверку, поэтому рамка с исчезнувшей опорой снимается и выбрасывает себя (тикающая проверка раз в 100 тиков — ванильный `CHECK_SURVIVAL_FREQUENCY_TICKS`);
  - `drop_item()`: звук `entity.item_frame.break`, выпадение содержимого рамки и самой рамки предметом;
  - добавлено взаимодействие (`ItemFrameEntity#interact`): по заполненной рамке — поворот на 45° + звук `rotate_item` + обновление компаратора, по пустой — вставка предмета из руки (count 1) + звук `add_item` + расход одного предмета (`decrementUnlessCreative`), game event `BLOCK_CHANGE`;
  - добавлены публичные `rotation()` / `item()` / `placeable_item()` / `place_sound()`.
- У игрока добавлен ванильный `PlayerEntity#canPlaceOn` (`may_build` либо компонент `can_place_on` для ограниченных режимов) — используется размещением рамок.
- Тесты: 5 на предмет (успешное размещение + расход, светящаяся рамка = свой тип сущности, отказ без твёрдой опоры, отказ в занятом объёме, копирование `custom_name`) и 6 на сущность (проверка опоры, снятие при потере опоры с дропом, поворот, вставка предмета с расходом, пустая рука → `Pass`, дроп содержимого + рамки).
- Покрытие: **1308 / 1658 = 78,9 %**; не-`Item` без механик: 93 (17 классов).
- Аудит/оптимизация этой порции:
  - `DebugStickState::get` больше не аллоцирует строковый ключ: линейный поиск по горстке записей палки вместо `to_string()` на каждом клике;
  - `ItemFrameEntity::can_stay_attached` делает одну сущностную AABB-проверку вместо двух (ванильная проверка декораций для рамок — подмножество проверки свободного объёма), плюс один запрос коллизий блоков и чтение опоры;
  - дроп содержимого рамки при снятии больше не шлёт лишнее обновление компаратора (`set_item_with_update(.., false)`);
  - **исправлен баг двойного дропа**: `drop_item` защищён `AtomicBool`, иначе комбинация «урон + тик проверки опоры» выбрасывала предмет рамки дважды (есть тест `dropping_the_frame_twice_does_not_duplicate_items`);
  - проверено отсутствие дедлоков: `Entity::interact` вызывается из `interact_on` без удержания лока инвентаря, поэтому лок внутри `ItemFrameEntity#interact` безопасен.

### Дополнение (разблокировка следующих паков: подушки, картины, арбалет, стрелы)
Разбор того, что раньше считалось «заблокировано отсутствующими подсистемами», по реальным ассетам и реестрам репозитория:

- **Подушки (`CushionItem`, 16 предметов) — разблокированы.** В 26.3 подушка это не блок, а **сущность** `minecraft:cushion` (`net.minecraft.world.entity.decoration.Cushion`, 1.0 × 0.25, слот `color` типа `dye_color` со значением по умолчанию `WHITE`, флаги: `is_attackable`, `is_pickable`, `can_be_collided_with = false`, `piston_push_reaction = PUSH_PULL`). Компонент предмета `minecraft:cushion/color` уже сгенерирован в `vanilla_components::CUSHION_COLOR`, так что 16 цветов получаются из одного класса предмета + одной сущности.
- **Картины (`HangingEntityItem`, 1 предмет) — разблокированы.** Уже есть `PaintingVariant` (`value() -> PaintingVariantValue { width, height, asset_id, title, author }`), `PaintingVariantRegistry` с тегами (`impl_tagged_registry!`), компонент `PAINTING_VARIANT` и сгенерированный слой данных сущности (`painting_variant_id`, сериализатор `painting_variant`). Значит нужен только `PaintingEntity` на `BlockAttachedEntity` + ванильный `placePainting` (фильтр по тегу `PLACEABLE`, отбрасывание вариантов, которые не влезают, затем выбор случайного из вариантов максимальной площади).
- **Арбалет (`CrossbowItem`) — разблокирован.** Компонент `CHARGED_PROJECTILES` уже есть (`recursive_items.rs` его учитывает), отдельной подсистемы не требуется.
- **Стрелы (`ArrowItem`, `SpectralArrowItem`, `TippedArrowItem`, `BowItem`) — семантика подтверждена по декомпиляции 1.21.11.** Ванильный класс называется по Yarn `PersistentProjectileEntity` (путь `entity/projectile/PersistentProjectileEntity.java`); `AbstractArrow.java` в этом репозитории нет (404). Считанный порядок тика: стоп внутри блока → `inGround`-ветка (`shouldFall`/`fall`/`age` до `MAX_LIFE_TICKS` = 1200) → иначе поворот по вектору скорости, `tickLeftOwner`, рейкаст `COLLIDER`/`FuidHandling.NONE`, `applyCollision` c сортировкой попаданий по сущностям и продолжением при `pierceLevel > 0`, затем `applyDrag(0.99)` и `applyGravity()` (0.05). Урон: `ceil(clamp(speed * damage, 0, i32::MAX))`, крит добавляет `nextInt(rawDamage / 2 + 2)`, при пробитии ведётся список пронзённых и при `pierced.size() >= pierceLevel + 1` стрела удаляется.
- Итог по покрытию не меняется (**1308 / 1658 = 78,9 %**), но «заблокированный» остаток сократился до четырёх настоящих подсистем: книга знаний (книга рецептов), карты (состояние карты + пакет отрисовки), лодки (меню сундука-лодки), ведро с мобами (рыбы/аксолотль/головастик).

### Дополнение (подушки: 16 предметов + сущность `Cushion`)
Подушки в 26.3 — не блоки, а сущности, поэтому пак закрыт новой сущностью + одним классом предмета:

- **Сущность `CushionEntity`** (`steel-core/src/entity/entities/objects/decoration/cushion.rs`, класс `Cushion`):
  - размеры 1.0 × 0.25, сидит ровно на верхней грани опорного блока (`y = support.y + 1`), без гравитации и без коллизии (флаги типа сущности);
  - цвет хранится в синхронизируемом поле `color` (`DyeColor`, по умолчанию `WHITE`) — клиент видит цвет без дополнительных пакетов;
  - каждый тик проверяется опора: если блок под подушкой больше не `isSolid`, подушка выпадает предметом своего цвета и снимается;
  - `hurt` (атака) — тот же дроп, защищён `AtomicBool`, поэтому «урон + проверка опоры» не могут выдать предмет дважды;
  - `interact` — игрок садится (`startRiding`) + звук `entity.cushion.sit`; при снеке возвращается `Pass`, чтобы работало обычное размещение предмета; при сходе (пассажира больше нет) играется `entity.cushion.get_up`;
  - `save_additional`/`load_additional` — цвет пишется строкой (`white` … `black`), совместимо с ванильным `DyeColor`-кодеком;
  - `is_pickable = true` — pick-block отдаёт предмет нужного цвета.
- **Предмет `CushionItem`** (`steel-core/src/behavior/items/cushion.rs`) закрывает **все 16 цветов**:
  - цвет берётся из компонента стека `minecraft:cushion/color` (`CUSHION_COLOR`) и копируется в сущность;
  - размещение только на твёрдую опору (`isSolid`), объём подушки должен быть свободен от блоков (провайдер коллизий) и от других подушек (`has_entity_in_aabb_matching`);
  - `canPlaceOn`-проверки игрока, лимит высоты мира, звук `entity.cushion.place`, game event `ENTITY_PLACE`, расход ровно одного предмета, копирование `custom_name`.
- Тесты: 6 на предмет (белая подушка на опоре + расход, цвет из компонента, отказ на плите (не `isSolid`), отказ в занятом блоке, отказ второй подушки на существующей, копирование имени) и 8 на сущность (жизнь при опоре, дроп ровно одного цветного предмета при исчезновении опоры, идемпотентность урона, посадка игрока, `Pass` при снеке, компонент цвета на дропе, все 16 цветов → 16 разных предметов с round-trip компонента, запись цвета в NBT + дефолт `WHITE`).
- Покрытие: **1324 / 1658 = 79,9 %**; не-`Item` без механик: 77 (16 классов) — `CushionItem` из списка закрыт.

### Дополнение (картина: `PaintingEntity` + `HangingEntityItem`)
- **Сущность `PaintingEntity`** (`entity/entities/objects/decoration/painting.rs`, класс `Painting`):
  - размеры из варианта: `calculateBoundingBox` перенесён точно (`PIXEL_SIZE = 0.0625` по оси прикрепления, `PAINTING_DEPTH_OFFSET = 0.46875`, смещение на 0.5 при чётной ширине/высоте, перпендикуляр — `rotateYCounterclockwise`);
  - `canStayAttached`: собственный объём свободен от блоков (провайдер коллизий), в нём нет других сущностей, и **все** блоки «за» картиной (полный `getAttachmentBox`, а не один блок) — `isSolid`; проверка раз в `CHECK_SURVIVAL_FREQUENCY_TICKS`;
  - `placePainting`: кандидаты — только варианты из тега `painting_variant/placeable`, отбрасываются не влезающие, остаются варианты максимальной площади, выбор случайный (`rand::random_range`);
  - дроп: `entity.painting.break` + предмет `painting`, идемпотентно (`AtomicBool`);
  - NBT: `facing` (3d-data значение) и `variant` (ключ варианта), после загрузки пересчитывается AABB; сидирование данных — `painting_variant_id` (синхронизируется клиенту).
- **Предмет `HangingEntityItem`** (`behavior/items/hanging_entity_item.rs`):
  - только горизонтальные грани (`side.isHorizontal()`), `canPlaceOn` игрока, лимит высоты;
  - отсутствие варианта → `CONSUME`, успешное размещение → звук `entity.painting.place`, game event `ENTITY_PLACE`, расход 1 предмета, копирование `custom_name`.
- Тесты: 4 на предмет (размещение + расход, отказ на вертикальной грани, отказ без стены, копирование имени) и 6 на сущность (AABB = размер варианта, жизнь при стене, дроп при разрушении стены, идемпотентность дропа, `placePainting` требует стену, сохранение/загрузка варианта и направления).
- Покрытие: **1325 / 1658 = 79,9 %**; не-`Item` без механик: 76 (15 классов) — `HangingEntityItem` закрыт.

### Дополнение (стрелы и лук: `Arrow`, `SpectralArrow`, `ArrowItem`, `SpectralArrowItem`, `BowItem`)
- **Сущности** (`entity/entities/objects/projectiles/arrow.rs`, классы `Arrow` и `SpectralArrow`) на общем трейте `AbstractArrow`:
  - ванильная физика: `gravity 0.05`, `drag 0.99` в воздухе / `0.6` в воде, поворот по вектору скорости, рейкаст `getHitResultOnMoveVector` с сортировкой попаданий (`Projectile`-слой);
  - **застревание**: попадание в блок → `inGround`, скорость в ноль, звук `entity.arrow.hit`, предмет приводится к `count = 1`; в `inGround` работает только счётчик `life`, по `MAX_LIFE_TICKS = 1200` стрела удаляется;
  - **урон**: `ceil(speed * damage)`, критический бонус `nextInt(raw/2 + 2)`, `DamageSource::arrow` с владельцем, отбрасывание живых (`knockback 0.6` по горизонтали), звуки `entity.arrow.hit` / `entity.arrow.hit_player`;
  - **пробитие**: `pierceLevel`, список пронзённых сущностей, лимит `pierceLevel + 1`;
  - **pickup**: `DISALLOWED / ALLOWED / CREATIVE_ONLY` с ванильными ординалами, подбор через `player_touch` с добавлением в инвентарь (`add_item_with_sound`) и удалением стрелы;
  - **spectral**: тот же код + пометка `glowing` для поражённой сущности, урон 4.0;
  - NBT: `damage`, `crit`, `PierceLevel`, `inGround`, `life`, `pickup`, `ShotFromCrossbow`, `item`, `weapon`.
- **Предметы** (`behavior/items/arrow.rs`): `ArrowItem` и `SpectralArrowItem` — ванильные фабрики `createArrow` (тип снаряда, копия предмета, владелец, режим подбора); `ArrowItem::is_arrow` распознаёт обычные, спектральные и зельевые стрелы.
- **Лук** (`behavior/items/bow.rs`): длительность использования `72000`, анимация `bow`, кривая силы `(f² + 2f) / 3`, минимум `0.1`, запуск `shootFromRotation(power * 3)`, крит на полном натяжении, расход одной стрелы (кроме креатива), 1 единица прочности, статистика `ITEM_USED`, звук `entity.arrow.shoot`; стрелы ищутся в слотах инвентаря (`ArrowItem::is_arrow`).
- Тесты: 5 на сущность (ординалы pickup, деспавн на 1200 тиках, застревание в блоке + `count = 1`, флаг spectral и базовый урон 2/4, сохранение/загрузка `pierce/crit/inGround/pickup/crossbow`) и 3 на лук (кривая силы, creative-only подбор, распознавание всех трёх видов стрел).
- **Зельевые стрелы**: закрыты следующим дополнением (`Arrow#doPostHurtEffects`).
- Покрытие: **1328 / 1658 = 80,1 %**; не-`Item` без механик: 73 (12 классов) — закрыты `ArrowItem`, `SpectralArrowItem`, `BowItem`.

### Дополнение (арбалет + зельевые стрелы: `CrossbowItem`, `Arrow#doPostHurtEffects`)
- **Зельевые стрелы** (`projectiles/arrow.rs`, `AbstractArrow::apply_tipped_arrow_effects`): ванильный `Arrow#doPostHurtEffects` — после успешного урона по живой сущности применяются все эффекты `POTION_CONTENTS` предмета стрелы, длительность `max(base / 8, 1)` тик, источник — стрела, владелец — стрелок; мгновенные эффекты (урон/лечение) идут через `MobEffectBehavior::apply_instantaneous`, остальные — через `add_mob_effect_with_source` с иконкой по флагу `show_particles`; проверка `is_affected_by_potions` соблюдается. Спектральные стрелы эффектов не накладывают (как в ванили).
- **Арбалет** (`behavior/items/crossbow.rs`):
  - зарядка: `use_item` без заряда запускает `start_using_item`, длительность = `25 − 5 × quick_charge` (минимум 1 тик), анимация `crossbow`, звуки `item.crossbow.loading_start` / `loading_middle` / `loading_end`;
  - загрузка на завершении заряда (`on_use_tick` + `finish_using`): приоритет — стрелы (обычные/зельевые/спектральные), затем фейерверк; `multishot` грузит 3 снаряда; в креативе боеприпасы не расходуются; результат пишется в компонент `charged_projectiles`;
  - выстрел (`use_item` по заряженному): все заряженные снаряды вылетают одним залпом, `shoot_from_rotation` со скоростью `3.15` (стрелы) / `1.6` (фейерверк), разброс `multishot` — смещение по рысканью `±10°` и тангажу `±0.2°`, стрелы помечаются `ShotFromCrossbow`, получают владельца и режим подбора (creative-only в креативе); компонент очищается, арбалет теряет 1 прочность, статистика `ITEM_USED`, звук `item.crossbow.shoot`;
  - `release_using` не стреляет (отпускание только отменяет зарядку).
- Тесты: 6 на арбалет (быстрая зарядка от Quick Charge, минимум 1 тик, `multishot` = 3 снаряда, симметричный разброс, круговой рейс компонента `charged_projectiles`, скорости стрелы/фейерверка) — всего по пакету 14 тестов.
- Покрытие: **1329 / 1658 = 80,2 %**; не-`Item` без механик: 72 (11 классов).

### Дополнение (кристалл Края, шаблоны кузнеца, фрагмент диска: `EndCrystalItem`, `SmithingTemplateItem` 19, `DiscFragmentItem`)
- **`EndCrystalItem`** (`behavior/items/end_crystal.rs`) — ванильный `EndCrystalItem#useOnBlock` целиком:
  - база только `obsidian` / `bedrock` (`is_valid_base`), иначе `FAIL`;
  - блок над базой обязан быть воздухом (`isAir`), иначе `FAIL`;
  - объём 1×2×1 должен быть свободен от сущностей (`has_entity_in_aabb_matching` по `WorldAabb::entity_box(...)`) — иначе `FAIL`, при этом предмет **не** расходуется;
  - кристалл спавнится в центре блока с `show_bottom = false` (как в ванили: `crystal.setShowBottom(false)`), гейм-эвент `ENTITY_PLACE`, расход ровно одного предмета.
  - **Честное ограничение**: `EnderDragonFight#respawnDragon` не вызывается — в Steel нет реализации боя с драконом; помечено `TODO` в коде.
- **`SmithingTemplateItem`** (19 предметов: незеритовый апгрейд + 18 тримов) — в 26.3 у класса **нет серверных механик**: он только отдаёт тексты подсказок и текстуры пустых слотов кузнечного экрана, которые рисует ванильный клиент из своих ассетов. Поэтому пак закрывает класс регистрацией поведения + переносом ванильных таблиц слотов в код:
  - `ARMOR_TRIM_EMPTY_BASE_SLOT_TEXTURES` (4), `ARMOR_TRIM_EMPTY_ADDITIONS_SLOT_TEXTURES` (7), `NETHERITE_UPGRADE_EMPTY_BASE_SLOT_TEXTURES` (11), `NETHERITE_UPGRADE_EMPTY_ADDITIONS_SLOT_TEXTURES` (1) — точные ванильные списки из `SmithingTemplateItem#get*Empty*SlotTextures`;
  - ключи переводов `item.minecraft.smithing_template{,.applies_to,.ingredients,.armor_trim.base_slot_description,.netherite_upgrade.base_slot_description}`;
  - геттеры `armor_trim_*` / `netherite_upgrade_*` для будущего экрана кузнечного стола.
- **`DiscFragmentItem`** (1 предмет) — тоже чисто клиентский тултип; класс зарегистрирован, добавлен `description_translation_key` = `item.<ns>.<path>.desc` (ванильный `DiscFragmentItem#getDescription`).
- Тесты: 4 на кристалл (успех на обсидиане + `show_bottom == false` + позиция + расход, успех на бедроке, отказ на камне без расхода, отказ при занятом блоке сверху), 4 на шаблоны (точные длины/содержимое списков слотов, ключи переводов, резолв трёх шаблонов в стеки) и 1 на фрагмент диска.
- Покрытие: **1350 / 1658 = 81,4 %**; не-`Item` без механик: 51 (8 классов) — закрыты `EndCrystalItem`, `SmithingTemplateItem`, `DiscFragmentItem`.
- Осталось четыре настоящие подсистемы: `BoatItem` 22 (нужна серверная физика `AbstractBoat` + пакет `paddle_boat`; исходники 26.3 уже скачаны, порт — отдельная задача), `MapItem`/`EmptyMapItem` 18 (состояние карты + пакет отрисовки), `MobBucketItem` 7 (нет сущностей рыб/аксолотля/головастика), `KnowledgeBookItem` 1 (книга рецептов), `ArmorStandItem` 1 (нет сущности стойки), `TridentItem` 1 (нет снаряда-трезубца), `WindChargeItem` 1 (нет сущности заряда ветра).

### Дополнение (лодки и плоты: `AbstractBoat`, `Boat`, `Raft`, `AbstractChestBoatEntity`, `ChestBoat`, `ChestRaft`, `BoatItem`, пакет `paddle_boat`)
- **Пакет `paddle_boat`** (`steel-protocol/src/packets/game/s_paddle_boat.rs`, `SPaddleBoat`) — ванильный `BoatPaddleStateC2SPacket`: два boolean-а (левый, правый весла). Диспетчеризация в `player/connection/java.rs` (`S_PADDLE_BOAT`, `PlayerLocal`), обработчик `Player::handle_paddle_boat` повторяет `ServerPlayNetworkHandler#onBoatPaddleState` — флаги принимаются только от управляемой лодки.
- **Ввод гребца**: `handle_player_input` дополнительно пересылает `PlayerInput` в `AbstractBoat::set_inputs` (ванильный `onPlayerInput`).
- **База `AbstractBoat`** (`entity/entities/objects/vehicles/boat.rs`) — порт `AbstractBoatEntity`:
  - `checkLocation` / `checkBoatInWater` / `getUnderWaterLocation` / `getWaterHeightBelow` / `getNearbySlipperiness`: точные ванильные границы перебора, `waterLevel` инициализируется `-Double.MAX_VALUE`, высота жидкости берётся как `y + height`;
  - `updateVelocity`: гравитация `-0.04`, `GRAVITY_UNDER_FLOWING_WATER = -7.0E-4`, плавучесть `0.01` под водой и `(waterLevel - y) / height` на поверхности, сопротивления `0.9` (вода/воздух), `0.45` (под водой), `0.05` (база), `BUOYANCY_SCALE 0.65`, `BUOYANCY_DAMPING 0.75`, приводнение при переходе `IN_AIR → вода` (`+0.101`, проверка свободного места) — порядок операций как в ванили;
  - на суше `drag = nearbySlipperiness`, а сама сохранённая скользкость делится на 2, если лодкой управляет игрок (ванильная тонкость, а не деление drag);
  - `updatePaddles`: `PADDLE_TURN_SPEED 0.005`, `PADDLE_FORWARD_SPEED 0.04`, `PADDLE_BACK_SPEED 0.005`, поворот на `yawVelocity`, ускорение по `sin(-yaw)`/`cos(yaw)`, флаги вёсел `right && !left || forward` / `left && !right || forward`;
  - `tickPaddleSounds`: фаза `π/8` за тик, порог звука `π/4` (по модулю `2π`), позиция звука смещается на ±вектор взгляда, pitch `0.8 + 0.4·rand`, звуки `entity.boat.paddle_water` / `entity.boat.paddle_land`; звуки проигрываются после снятия блокировок состояния;
  - `pushNearbyEntities`: расширение AABB `(0.2, -0.01, 0.2)`, посадка мелких живых существ (уже не едущих верхом, уже ширины лодки) при свободном месте и отсутствии игрока-водителя, остальные отталкиваются;
  - `isLogicalSideForUpdatingMovement`: лодка под управлением игрока НЕ симулируется сервером (её двигает клиент через `MoveVehicle`), остальные случаи — серверные;
  - `tick`: сброс/инкремент `ticksUnderwater`, выброс пассажиров на `UNDERWATER_TICKS_MAX = 60`, затухание `hurt_time`/`damage`, `base_tick`, движение `MoverType::SelfMovement`, звуки вёсел, толкание соседей;
  - урон/уничтожение — существующий `VehicleEntity` (`vehicle_hurt`, `vehicle_destroy_with_item`); предмет дропа берётся по совпадению ключа сущности и предмета (`oak_boat` → `oak_boat`), как в ванильном `asItem()`.
- **Сущности**: 4 класса по `classes.json` — `Boat` (`BoatEntity`), `Raft` (`RaftEntity`), `ChestBoat` (`ChestBoatEntity`), `ChestRaft` (`ChestRaftEntity`); macrobлок `boat_entity_impl!` даёт общий `Entity`/`VehicleEntity`/`AbstractBoat`, синхронизированные данные — слои `VehicleEntity` (8–10) и `AbstractBoat` (11 `paddle_left`, 12 `paddle_right`, 13 `bubble_time`).
- **Сундук-лодка** (`AbstractChestBoatEntity`): инвентарь на 27 слотов (`SimpleContainer`), один пассажир, `Items` в NBT (общий `load_all_items`), рассыпание содержимого при уничтожении уроном, меню `GENERIC_9X3` через новый `MenuKind` `ChestBoatKind` (валидность: лодка жива и игрок в пределах 8 блоков), открытие по sneak/занятому месту, гейм-эвент `CONTAINER_OPEN`.
- **Предмет `BoatItem`** (`behavior/items/boat.rs`) — один на все 22 предмета (`entity_type` из `classes.json`): рейкаст `OUTLINE + ANY` на `block_interaction_range()`, отказ, если в глазах игрока сущность (`ENTITY_SEARCH_RANGE = 5`), проверка свободного места `has_collision(for_entity(...))`, спавн через `ENTITIES.create`, поворот по рысканью игрока, `ENTITY_PLACE`, расход 1 предмета.
- Тесты: 7 на лодку/плот (стартовое состояние, флаги вёсел в синхронизированных данных, хранение ввода гребца, дроп по ключу для `oak_boat` и `bamboo_raft`, звуки по локации, `is_water`/`is_underwater`, дефолтное состояние) + 3 на сундук-лодку (27 слотов, один пассажир, дроп `oak_chest_boat`) + 1 на пакет = 11 тестов.
- **Честные ограничения** (все помечены в коде): пузырьковый столб (`onBubbleColumnSurfaceCollision`) не переносится — нужен хук из блока; `tickBlockCollision()` из ванильного `tick` не вызывается; скользкость аппроксимирована вертикальным перекрытием AABB вместо точного пересечения `VoxelShape` (совпадает для полных блоков) с пропуском `lily_pad`; тег `#cannot_be_pushed_onto_boats` заменён явной проверкой «не игрок»; `getPassengerHorizontalOffset 0.15` у сундук-лодки не применяется; таблицы добычи (`LootTable`) не переносятся; `PiglinBrain.onGuardedBlockInteracted` отсутствует; `MoveVehicle`-клиентская симуляция означает, что серверный `updatePaddles` работает только для лодок без игрока-водителя.
- Покрытие: **1372 / 1658 = 82,8 %**; не-`Item` без механик: 29 (7 классов) — закрыты `BoatItem` (22 предмета) и четыре класса сущностей.
- Осталось: `MapItem`/`EmptyMapItem` 18, `MobBucketItem` 7 (нет сущностей рыб/аксолотля/головастика), `KnowledgeBookItem` 1, `ArmorStandItem` 1, `TridentItem` 1, `WindChargeItem` 1.
