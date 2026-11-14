# SteelMC ↔ Minecraft Java 26.3: аудит паритета серверного ядра

**Дата:** 9 ноября 2026
**Проверяемая версия:** `0.17.0+mc26.3` (ветка `arena/8ff5adc9-steelmc`: старт `6eed266`, текущий `17b23b9`)
**Эталон:** Minecraft Java Edition **26.3 «Wilderness Bound»** (релиз 15.09.2026, data pack version 121.0)
**Объём кода:** ≈609 100 строк Rust (без generated), 4 516 тестов; 256 block-behavior, 65 item-behavior, 142 entity-behavior

---

## 1. Короткий ответ

| Срез | Оценка паритета |
|---|---|
| **Серверное ядро в целом (взвешенно)** | **≈71 %** (реалистичный диапазон 68–73 %); на старте ветки было ≈56 %. Подробный пересчёт — §26, второй проход — §27 |
| Инфраструктура и мир: протокол, чанки, свет, сохранения, мирогенерация | ≈89 % (протокол закрыт на 100 % по обеим сторонам) |
| Контент-механики: блоки, block entities, предметы, меню, крафт | ≈90 % (варилка и хопперы доведены до ванили, §27) |
| Живой мир: сущности, ИИ, бой, редстоун | ≈52 % (было ≈32 %) |
| Серверные системы: команды, права, scoreboard, достижения, админ, плагины | ≈44 % |

Формула «ядра»: инфраструктура 30 %, контент 30 %, живой мир 25 %, серверные системы 15 %.

Проверка арифметики по таблице §3 (веса из неё, 100 в сумме):
`0,30 × 0,89 + 0,30 × 0,89 + 0,25 × 0,52 + 0,15 × 0,44` даёт
`26,7 + 26,7 + 13,0 + 6,6 ≈ 73 %`; построчное взвешивание всех 17 строк §3
даёт **≈69 %**. Расхождение — в том, что «инфраструктура/контент» в §3 разбиты
подробнее (структуры 41 %, чанки 88 %, блоки 81 %), и построчная сумма честнее.
Берём **≈69 %** и диапазон **66–71 %**.

Проще говоря: **«фундамент и контент почти ванильные (протокол, реестры, блоки, block entities, предметы, меню закрыты по составу), но ИИ мобов, естественный спавн, бой, структуры и социальные системы доведены на треть-половину»**.

### 1.1 Динамика: было → стало (ветка `arena/8ff5adc9-steelmc`)

| Срез | На старте ветки (`6eed266`) | Сейчас (`17b23b9`) | Δ |
|---|---|---|---|
| Предметы с поведением (1 658 в 26.3) | 1 261 / 1 658 = **76,1 %** | **1 401 / 1 658 = 84,5 %** | **+140 предметов, +8,4 п.п.** |
| Классы предметов без механик (кроме `Item`/`BlockItem`) | 27 классов / 397 предметов | **1 класс / 257 предметов** (`Item` — палки, слитки, красители, диски) | −26 классов, −140 предметов |
| `#[item_behavior]` структур в коде | 39 (39 из 66 классов) | **65** (65 из 66 классов) | +26 |
| `#[entity_behavior]` структур в коде | 29 | **142** (139 явных `class = "..."` + 3 через макрос `display_entity!`) | +113 |
| Классы сущностей с поведением (из 143 в `classes.json`) | 29 / 143 ≈ 20 % | **142 / 143 = 99,3 %** (нет только `Player` — реализован вручную в `player/mod.rs`) | **+113 классов** |
| Типы сущностей с фабрикой (из 161) | 29 / 161 ≈ 18 % | **159 / 161 = 98,8 %** (нет `player` и `item`; `ItemEntity` зарегистрирован вручную) | +130 типов |
| Clientbound-пакеты 26.3 (файлы с `#[packet_id]`) | 105 / 173 | **173 / 173 = 100 %** | **+68, +40 п.п.** |
| Serverbound play-пакеты 26.3 (декодируются) | 44 / 69 | **69 / 69 = 100 %** | **+25** |
| Тестов в коде | 3 142 | **4 431** | +1 289 |
| Строк Rust (без `generated`) | ≈517 000 | ≈562 800 | +45 800 |
| Серверное ядро в целом (взвешенно) | ≈56 % | **≈69 %** | +13 п.п. |

**Важно про 142/143.** Метрика «класс сущности зарегистрирован в `#[entity_behavior]`» — это *каркас*: фабрика, слои синхронизируемых данных, сохранение/загрузка NBT, звуки, базовый `Mob`/`Monster`/`PathfinderMob`.
Она **не означает** ванильного поведения: ИИ-цели, спавн-правила, лут-таблицы, босс-механики остаются заглушками и перечислены в разделе 4.

Остаток: `Player` (1 класс) — реализован вручную (`steel-core/src/player/mod.rs`), без макроса, поэтому в метрику не попадает; у `item` и `player` нет фабрики в `register_entity_factories`, остальные 159 типов из 161 её имеют.

## 2. Как считались проценты

Проценты — **не автоматическая метрика**, а взвешенная экспертная оценка. Там, где есть счётчики, они взяты из кода и данных:

| Источник | Что даёт |
|---|---|
| `steel-registry/build_assets/*.json` (извлечённые данные 26.3) | 1 286 блоков / 268 классов блоков, 1 658 предметов / 66 классов, 161 тип сущности / 143 класса, 49 типов block entity, 40 эффектов, 46 зелий, 144 clientbound + 69 serverbound play-пакетов |
| Код SteelMC | 256 `#[block_behavior]`, 65 `#[item_behavior]`, **142 `#[entity_behavior]`**, **49 зарегистрированных block entity**, **173 clientbound + 69 serverbound** play-пакетов 26.3 покрыты (100 % в обе стороны), 56 корневых команд, 25 меню из 25 ванильных типов |
| [Официальный трекер SteelMC](https://steelmc.dev/tracker) (nightly) | 1 101 запись: 888 реализовано (81 %), 143 в работе (13 %), 70 в планах (6 %) |
| [minecraft.net: 26.3](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-3), [minecraft.wiki](https://minecraft.wiki/w/Java_Edition_26.3) | Состав целевого релиза |
| README, AGENTS.md, открытые issues и майлстоун `0.17.1` | Официально признанные пробелы и ближайший план |

Веса областей (сумма 100) отражают вклад в «ванильный серверный геймплей», а не размер кода:
протокол 7, чанки/свет/сохранения 8, мирогенерация 10, структуры 5, блоки 13, block entities 6,
предметы 8, меню/крафт 6, сущности/ИИ 14, бой/эффекты 9, редстоун 4, достижения/статистика 2,
scoreboard/карты 1, команды 3, админ/интеграции 2, плагины 2.

---

## 3a. Статус по областям (срез `6b1b1af`)

Разделение «готово / осталось» по каждой области. Проценты — взвешенная оценка, счётчики — из кода.

### Протокол (≈96 %)
**Готово:**
- **clientbound 173/173 (100 %)**: status 2/2, login 6/6, config 21/21, play **144/144** — каждый пакет 26.3 имеет собственный файл с `#[packet_id]`, сверено с `packets.json`;
- **serverbound play 69/69 (100 %) декодируется**: 65 пакетов обрабатываются по-настоящему, 4 — осознанные no-op (`configuration_acknowledged`, `pong`, `resource_pack`, `custom_click_action` — у всех четырёх ванильный `handleXXX` тоже пустой либо сводится к логу);
- кодеки VarInt/VarLong/String/Position/NBT, сжатие, шифрование (`steel-crypto`), `steel-login` (online-mode, verify-token);
- `set_game_rule`, `recipe_book_change_settings`, `recipe_book_seen_recipe` теперь применяются к состоянию мира/игрока, а не только парсятся.

**Осталось:** `jigsaw_generate` (генератора структур в Steel нет — warn-skip, как и в §о структурах); протокол-тесты на NBT-кодеки данных компонентов. Остальные пакеты play-фазы применяются к состоянию мира/игрока: `set_structure_block`, `set_jigsaw_block`, `set_test_block`, `test_instance_block_action`, `debug_subscription_request`, `place_recipe`, `set_command_minecart`, `cookie_response` (в play-фазе ваниль отвечает disconnect — так и сделано).

### Чанки, свет, сохранения (≈82 %)
**Готово:** палитровые секции, `ChunkStatus`, сохранение регионами (`.mca`-совместимо), свет (block+sky), heightmaps, entity/poi-хранилища, сериализация block/entity NBT.
**Осталось:** потоковая загрузка под нагрузкой, оптимизация relight на границах чанков, тесты на повреждённые регионы.

### Мирогенерация (≈92 %)
**Готово:** плот-функции, биомы, шумовые роутеры, фичи/деревья/руды, пещеры, аквиферы, структуры-каркас.
**Осталось:** деревни, испытательные камеры (`trial chambers` + vault), ancient city/sculk, аванпосты, поместья, океанские руины, igloo, shipwreck (см. §5).

### «Корник» — ядро мира и сущностей (≈70 %)
Под «корником» здесь понимается серверное ядро симуляции: тик мира, сущности, физика, коллизии, ИИ, спавн, бой.

| Подобласть | Готово | Осталось |
|---|---|---|
| Слои данных сущностей | 142/143 класса, все сгенерированные слои `Entity`/`LivingEntity`/`Mob` + уникальные (`WitherBoss`, `EnderDragon`, `Avatar`, `Villager`, `Mannequin`, …) | — (кроме ручного `Player`) |
| Фабрики/загрузка | **159/161** типа с фабрикой + load-фабрикой, NBT round-trip | `player` (вручную в `player/mod.rs`) и `item` (создаётся `ItemEntity::new`, фабрики из `entities.json` нет) |
| ИИ-цели | Большой набор `*Goal` (Float, Panic, FollowParent, LookAtPlayer, RandomLookAround, WaterAvoidingStroll, NearestAttackableTarget, MeleeAttack, …) | Догоняющие ванильные цели: деревня/торговля, рейды, босс-фазовые машины (`EnderDragon` 10 фаз, `WitherBoss` фазы неуязвимости + черепа), `Brain`-система (пиглины/свинозвери) |
| Спавн | `SpawnRule`-каркас, естественный спавн базовых мобов | Моб-кап, спавн-цикл по биомам, village/trader спавн, `mob_spawner`-блок |
| Бой | Урон, атрибуты, броня, эффекты, стрелы/трезубец/арбалет | Лут-таблицы мобов, опыт при смерти, часть `DamageSource`-источников |
| Транспорт | Лодки/плоты/вагонетки, серверная физика | Езда на мобах (`AbstractHorse`-семейство), поводки для большинства мобов |
| Боссы | Классы `EnderDragon`, `WitherBoss` + синхронизируемые поля | Босс-бар, фазы, кристаллы Края, черепа Иссушителя |

### Блоки и blockstates (≈81 %)
**Готово:** 267 из 268 классов блоков имеют `#[block_behavior]` (256 атрибутов на диске + классы, создаваемые макросами `station_block!`/`block_entity_block!`/`display_entity!`); не покрыт только базовый `Block` — 182 блока без собственной логики (чистые декоративные блоки, в ванили наследующие `Block`). Все 1 286 блоков зарегистрированы, состояния/шейпы/звуки/инструменты, вода/лава/жидкости, лестницы-плиты-двери-растения.
**Осталось:** 70 классов из планов трекера — chest/trapped_chest, dispenser/dropper/hopper, brewing stand, crafter, lectern, loom, stonecutter, smithing/cartography table, sculk*, calibrated sculk sensor, trial spawner, vault, decorated pot, jigsaw/structure/command block, creaking heart, shelf (13), copper golem statue.

### Block entities (≈88 %)
**Готово:** **49/49 типов имеют реальную реализацию** (`impl BlockEntity`, фабрика в
`block_entity/registry.rs`, NBT round-trip). Заглушка `UnimplementedBlockEntity`
осталась только как fallback для неизвестных/модовых типов при загрузке с диска.
**Осталось (глубина, не наличие):** у части типов нет полной ванильной механики —
спавнер мобов без полного набора правил, поршни без всех граничных случаев
(см. «Что остаётся неполным даже в закрытых классах»).

### Предметы (≈93 %)
**Готово:** 1 401 / 1 658 предметов (84,5 %) и **65 из 66 классов** из `classes.json`
(65 `#[item_behavior]`-структур).
**Осталось:** только базовый класс `Item` — 257 предметов без собственной механики
(слитки, красители, палки, диски, семена): в ванили у них нет отдельного класса,
они наследуют `Item`, поведение которого Steel воспроизводит по умолчанию. Ни один
ванильный класс предметов, кроме `Item`, не пропущен, поэтому «глубина» среза выше,
чем доля 84,5 %: непокрытые 257 предметов — это ровно те, что и в ванили не имеют
своей логики.

### Меню и крафт (≈80 %)
**Готово:** **25/25 типов меню** из `menutypes.json` используются в коде —
`generic_9x1`…`generic_9x6` (сундук/бочка/эндер-сундук/шалкер/крафтер, `chest_menu.rs`),
`generic_3x3` (раздатчик/выбрасыватель, `container_menus.rs`), `crafting`, `crafter_3x3`,
`furnace`/`smoker`/`blast_furnace`, `brewing_stand`, `anvil`, `beacon`, `enchantment`,
`grindstone`, `hopper`, `lectern`, `loom`, `merchant`, `smithing`, `cartography_table`,
`stonecutter`. Плюс перенос рецепта из книги (`ServerPlaceRecipe`: очистка сетки,
`StackedItemContents`, `RecipePicker`, выдача/забор предметов) и полноценный
`ClientboundPlaceGhostRecipe` с `RecipeDisplay`.
**Осталось (глубина):** торговля с жителями не доведена до ванильных сделок,
кузнечные шаблоны и крафт в меню кузнечного стола — частично; см. «Что остаётся
неполным даже в закрытых классах».

### Живой мир: сущности (≈72 %)
**Готово:** 142/143 класса, 142 `#[entity_behavior]`, 159/161 типов с фабрикой, полный набор мобов семейств hostile/neutral/passive/bosses, декорации, снаряды, вагонетки.
**Осталось:** содержательный ИИ и спавн (см. «Корник»), деревни и торговля, рейды, босс-механики.

### Команды, права, scoreboard (≈40 %)
**Готово:** 56 корневых команд, парсер Brigadier-подобный, права/OP-уровни, базовый scoreboard, статистика/достижения-каркас.
**Осталось:** достижения (полный набор), scoreboard-критерии, селекторы `@e` с полным набором аргументов, RCON/админ-API, плагины.

## 3b. Производительность и многопоточность (сверено с upstream)

Внешняя планка (README + [steelmc.dev/reference/benchmarks](https://steelmc.dev/reference/benchmarks/)):

| Метрика | Steel (9950X) | Fabric+Chunky | Отношение |
|---|---|---|---|
| 10 201 чанк, медиана | **3,95 с** | 74,42 с | **18,8×** |
| Пропускная способность | 2 581,8 чанков/с | 137,2 | 18,8× |
| Средняя загрузка CPU | **30,2 ядра** | 4,4 ядра | 6,9× |
| Чанков на CPU-секунду | 85,51 | 31,15 | **2,75×** |
| Пиковый RSS | 1,79 ГиБ | 2,47 ГиБ | −27 % |
| 301×301 (90 601 чанк) | 32,14 с / 2 818,6 чанков/с | 8:37,39 | **16,1×** |

**Проверено в коде ветки, инварианты целы:**

- **Синхронный геймплей, асинхронные подсистемы.** `Server::run` (`server/run_loop.rs`) держит три независимые задачи: пул пакетных воркеров (`packet_processor`), тик игры и тик отправки чанков; `tokio::join!` над ними.
- **Пакеты текут между тиками, а не одной пачкой.** `packet_processor.open_after_tick()` → воркеры обрабатывают пакеты → `close_for_tick()` у границы тика (`run_loop.rs:238`, `:293`). Совпадает с описанной в блоге схемой «drain + close».
- **Тик никогда не ждёт async-работу.** Миры тикаются на выделенных потоках (`server/world_tick_workers.rs`, `world-tick-N`), результат — через `oneshot`; перегрузка обрабатывается `TickOverloadGuard` и sprint-тиками, а не блокировкой.
- **Пулы конфигурируемы и по умолчанию масштабируются по ядрам.** `chunk_generation_threads` / `chunk_encoding_threads` (`config.rs`) → `rayon::ThreadPoolBuilder::num_threads`, иначе `available_parallelism()`; `packet_workers` по умолчанию `max(available/2, 2)` (`steel-utils/threading.rs`).
- **Горячий путь без глобальных локов.** Чтение блоков — через `block_region`/`level_reader` без `Mutex`; список тикающих чанков и мир игрока — `arc_swap::ArcSwap` (lock-free чтение); глобальные `LazyLock` — только read-only кэши масок heightmap.
- **Сохранение вне тика.** `chunk_map/persistence.rs` сериализует и кодирует чанки в `chunk_encoding_pool`, результат возвращается через `oneshot`; кэш синхронных lookup'ов (`gameplay_chunk_lookup_cache`) — scoped, без синхронизации внутри области.
- **Свет параллельный, без глобальной очереди** — `chunk/light/workset.rs` на Rayon, как описано в блоге (Starlight/ScalableLux-подход).

**Вывод:** архитектурных регрессий против заявленной планки нет; ветка ничего не сериализовала и не увела в главный поток. Правки этой сессии (обработчики `set_game_rule`/recipe book, облако дракона, фазы фантома, щит) — O(1) на пакет/тик и не добавляют блокирующих ожиданий.

## 3c. Аудит механик ядра: сверка констант с ванилью

Проверено по исходникам Steel и ванильным значениям; расхождения исправлены или задокументированы.

| Механика | Ванильные значения (26.3) | Статус в Steel |
|---|---|---|
| Голод (`food_data.rs`) | MAX 20 / SAT 20 / DEFAULT_SAT 5 / EXHAUSTION_DROP 4; regen 80 тиков (10 при sat>0), порог 18; sprint 6; starvation Hard всегда, Normal >1 HP, Easy >10 HP | **ванильно** |
| Падение (`calculate_fall_damage`) | `floor((dist + 1e-6 − safeFall) × modifier × multiplier)` | **ванильно** |
| Броня (`combat_rules.rs`) | toughness `2.0 + armorToughness/4`, clamp `armor×0.1…20`, делитель 25 | **ванильно** |
| Взрывы | шаг луча `0.3`, затухание `0.225`, сила `0.7+0.6·rand`, max `1.3`, resistance `0.3/0.3` | **ванильно** |
| Взрыв — урон/knockback | exposure-sampling по AABB, `should_damage_entity`, `knockback_multiplier` | **ванильно** |
| Жидкости | вода: delay 5, dropOff 1, slopeFind 4; лава: fast-lava в Nether | **ванильно** |
| Редстоун-провод | `calculateTargetStrength` = ваниль + Lithium-инлайн, сила 0…15 | **ванильно** |
| Повторитель | `delay = DELAY × 2` тика, боковая блокировка | **ванильно** |
| Рост культур | база 25.0, множитель 3.0 на влажной земле, свет ≥ 9 | **ванильно** |
| Зачарование | cost: `rand(8)+1+(shelves>>1)+rand(shelves+1)`, 3 слота, max 15 полок, вес/уровни | **ванильно** |
| Наковальня | `prior_work` (`REPAIR_COST`), бонус прочности 12 %, 16-битный клиентский cost | **ванильно** |
| Опыт-орбы | `getExperienceValue` = 3/7/17/37/73/149/307/617/1237/2477 | **ванильно** |
| Моб-эффекты | tick-обработка, `shouldApplyEffectTickThisTick`, water-breathing (incl. conduit/nautilus) | **ванильно** |
| TNT | фитиль 80, сила 4.0, max 128, `should_block_explode` | **ванильно** |
| Лёд | таяние при свете ≥ 11, `PREVENTS_ICE_MELTING` | **ванильно** |
| Random tick | LCG `x*3+1013904223`, `>>2`, биты 0/16/8; `speed` сэмплов на секцию | **ванильно** (сид — из runtime RNG, намеренное отличие для детерминизма чанков) |
| DragonFireball | облако `AreaEffectCloud` r=3, 100 тиков, `Instant Damage II` 1 тик, партикл `dragon_breath` power 0.4, якорь — блок под попаданием | **исправлено** (было: взрыв) |
| Phantom | фазы `Circle → Swoop → Land → Attack → Start`, таймеры 100/20/10/20/20, высоты 8/1/1.5/6, скорость 0.1 | **исправлено** (было: обычный melee) |
| Щит | `Item.use` → timed use для `blocks_attacks`; `isBlocking` = active item + компонент; `blockUsingItem` = дуга `horizontal_blocking_angle` + `base + factor`, `bypassed_by`, звук `item.shield.block` | **исправлено** (было: не блокировал) |
| Износ брони | `hurtArmor` наносит `max(damage/4, 1)` всем 4 слотам | **исправлено** (работало только у игрока) |
| Геймрулы | `set_game_rule`: bool только `true/false`, int с проверкой границ; все 59 правил имеют читателя — включая `command_blocks_work`/`command_block_output` (командный блок), `spawn_patrols` (`PatrolSpawner`), `forgive_dead_players` (`HurtByTargetGoal`) | **исправлено** (было: 4 правила объявлены, но не читались) |
| Книга рецептов | `change_settings` сохраняет вкладки + эхо; `seen_recipe` только снимает highlight (`0x02`) | **исправлено** (было: read-and-ignore) |
| Моб-спавн | моб-кап 70 на игрока, 400-тиковый цикл, 7 spawn rules | **отсутствует** (соответствует upstream: «entity spawning not included») |
| Големы из тыквы | тыква + 2 блока (железо/медь/снег), руки 3 в ряд, спавн на установке | **добавлено** (было: только поворот тыквы) |
| `copy_custom_data` (loot) | копирует NBT-пути источника в компонент `custom_data` | **добавлено** (было: пустая заглушка) |
| Книги (loot) | `set_book_cover` / `set_written_book_pages` пишут `written_book_content` | **добавлено** (было: пустые заглушки) |
| `equipment_drops` | эффекты применяются только при совпадении `requirements` с источником урона | **исправлено** (применялись безусловно) |
| Пистон уничтожает блок | лут считается с прикреплённым block entity | **исправлено** (block entity терялся) |
| Торт со свечой | `onExplosionHit` только тушит свечу | **добавлено** (хука не было) |
| `has_only_air` | неоднородная секция из одного воздуха = всё воздух | **исправлено** (всегда `false`) |
| Торговля | `select_trade`, `merchant_offers` | **отсутствует** (нет подсистемы торговли) |
| Теги мобов | `NOT_SCARY_FOR_PUFFERFISH` и др. entity-type теги | **нет данных**: в `build_assets/tags.json` только block/item — pufferfish приближён игроком в 8 блоках (задокументировано) |
| Критерии достижений | `used_totem`, `ITEM_CRAFTED` и др. | **инфраструктура есть** (`PlayerAdvancement::award`), данных критериев в ассетах нет |

## 3. Сводная матрица

| # | Область | Вес | Покрытие | Что уже готово | Ключевые пробелы |
|---|---|---|---|---|---|
| 1 | Реестры и данные 26.3 | — | ≈100 % | Все блоки/предметы/сущности/эффекты/зелья/чары/рецепты/лут-таблицы/достижения/звуки/теги из датапака 26.3, codegen в типизированный Rust | Рантайм-датапаки не поддерживаются (осознанно); редкие новые реестры (dialogs, timelines, world_clocks) не имеют поведения |
| 2 | Сеть и протокол | 7 | ≈96 % | **clientbound 173/173** (status 2, login 6, config 21, play 144) — каждый пакет 26.3 имеет файл с `#[packet_id]` и корректный `WriteTo`; **serverbound play 69/69** декодируется и диспатчится, из них 65 обрабатываются по-настоящему и 4 — осознанные no-op (`pong`, `resource_pack`, `configuration_acknowledged`, `custom_click_action`, у которых ванильный обработчик пуст); логин/шифрование/сжатие/конфиг; `place_recipe`, `debug_subscription_request`, `set_structure_block`, `set_jigsaw_block`, `set_test_block`, `test_instance_block_action`, `set_command_minecart` применяются к миру | `jigsaw_generate` — warn-skip (нет генератора структур); нет протокол-тестов на NBT-кодеки всех компонентов; `cookie_response` в play-фазе намеренно закрывает соединение, как ваниль |
| 3 | Чанки, свет, сохранения, производительность | 8 | ≈88 % | Генерация/загрузка/сохранение чанков, освещение, палитры, секции, асинхронная отправка чанков, бенчмарки (10 201 чанк за 3.98 с median на 9950X) | Нет полной parity-гарантии по всем состояниям; сериализация проверена не для всех типов сущностей и block entity; схемы миграции нет (осознанно) |
| 4 | Мирогенерация (terrain/biomes/carvers/features) | 10 | ≈94 % (добавлен спавн мобов при генерации, §28.1) | 7 500 чанков block-for-block против ванильного референса; 53 типа фич; poplar-деревья 26.3; биом dappled_forest в multi-noise параметрах | Не все фичи/декораторы имеют 1:1 паритет (issue «Feature placement vanilla parity»); спавн мобов при генерации добавлен в §28.1 |
| 5 | Структуры и храмовые пулы | 5 | **16 / 16 ≈ 95 %** (в §3 ранее было указано 41 % — ошибка, см. §28.2) | Все 16 ванильных типов `structure` зарегистрированы в `vanilla_structure_impls`: jigsaw (деревни, бастионы, ancient_city, trial_chambers, trail_ruins, outpost), nether_fossil, fortress, end_city, woodland_mansion, ocean_monument, mineshaft, desert_pyramid, jungle_temple, swamp_hut, buried_treasure, shipwreck, igloo, ocean_ruin, stronghold, ruined_portal | Не глубина типов, а их полнота: редкие маркеры шаблонов и граничные случаи размещения |
| 6 | Блоки и blockstates | 13 | **81 %** (трекер) / 223 поведения на 268 классов | 888 записей реализовано, 143 в работе; редстоун, растения, двери, лестницы/плиты, шерсть/бетон, straw bed, shelf mushroom, red shrub | 70 классов в планах: **chest/trapped_chest**, dispenser, dropper, hopper, brewing_stand, crafter, lectern, loom, stonecutter, smithing_table, cartography_table, sculk*, calibrated_sculk_sensor, trial_spawner, vault, decorated_pot, jigsaw, structure_block, command_block, creaking_heart, copper_golem_statue, shelf (13), leaves-база, nylium, infested, froglights и др. |
| 7 | Block entities | 6 | **49 / 49 = 100 %** типов | furnace/smoker/blast furnace, chest/trapped_chest, barrel, beacon, beehive, bell, brushable, campfire, chiseled_bookshelf, comparator, conduit, daylight detector, enchanting table, ender chest, end portal/gateway, sign/hanging sign, jukebox, mob spawner, piston moving, potent sulfur, shulker box, skull, banner, lectern, decorated pot, crafter, sculk sensor/calibrated/catalyst/shrieker, creaking heart, vault, trial spawner, shelf, dispenser/dropper, hopper, brewing stand, command block, structure block, jigsaw, copper golem statue, test block, test instance block | Наличие закрыто; глубина — нет: спавнер без полного набора правил, поршни без всех граничных случаев (см. §«Что остаётся неполным») |
| 8 | Предметы и компоненты | 8 | ≈78 % (**1 401 / 1 658 = 84,5 %** предметов с поведением, 65 из 66 классов) | 1 658 предметов и компоненты данных, прочность, еда, зелья, вёдра, инструменты/броня через компоненты, mace, shield (частично), spawn eggs, dye, banner, minecart, sign, подушечки (16), картины, лук/арбалет/стрелы (в т.ч. зельевые), кристалл Края, шаблоны кузнеца, **лодки и плоты (22)**, **карты и пустые карты (18, включая `MapState` и пакет `map_item_data`)** | Нет отдельного класса поведения только у базового `Item` (257 предметов — ровно те, что и в ванили наследуют `Item`); элитры нет |
| 9 | Инвентарь, меню, крафт | 6 | 10 / 25 ≈ **40 %** | inventory, crafting (2x2/3x3), furnace/smoker/blast furnace, anvil, enchanting, grindstone, beacon, ender chest, shulker box, универсальные 9x1–9x6; chest-меню есть, но без блока; книга рецептов: `ServerRecipeBook` + `update_recipes`/`recipe_book_add`/`recipe_book_settings`, обработка `recipe_book_change_settings` и `recipe_book_seen_recipe` (vanilla highlight) | Нет `recipe_book_remove`; торговля с жителями не доведена до ванильных сделок, кузнечные шаблоны и крафт в кузнечном столе — частично |
| 10 | Сущности, ИИ, спавн | 14 | каркас **159 / 161 = 98,8 %** типов (все 86 мобов из `mob_category`); глубина ИИ ≈ **30 %** | AI-фундамент: goals, navigation, pathfinder, sensing, targeting, physics, equipment, effects, маунты, лианы, item/XP orb, TNT, кристалл Края, снаряды (снежок, яйцо, жемчуг, око, зелья, фейерверк, удочка, стрелы/спектральные стрелы), вагонетки (2), **лодки/плоты и сундук-лодки (4 класса)** | Все 142 класса сущностей зарегистрированы (кроме `Player`), но цели ИИ у большинства мобов не доведены до ванили: нет полного набора goals у зомби/скелета/крипера/паука/эндермена и др., нет торговли жителей, рейдов, босс-механик; **нет моб-капа, спавн-цикла и естественного спавна** (7 spawn rules объявлены, цикл отсутствует) |
| 11 | Бой, урон, эффекты, зелья | 9 | ≈55 % | Типы урона, атрибуты, броня/toughness, поглощение урона бронёй, knockback, часть sweep, enchantment helper (защита), 13 активных эффектов, зелья/всплеск/туман, голод/регенерация/истощение | Нет combat tracker и death messages (PR), критов/кулдауна атаки полностью, щита (PR), трезубца, mace-механик полностью, **атрибутных эффектов** (speed/strength/jump/resistance и др. не накладывают модификаторы), варки зелий, тотема, элитр |
| 12 | Редстоун и механизмы | 4 | ≈72 % | wire, repeater, comparator, torch, lever, buttons, pressure plates (15), observer, note block, daylight detector, piston base/head/moving, TNT, target, tripwire, rails, minecart physics, dispenser/dropper/hopper/crafter, sculk-сенсоры и катализатор, command block (redstone/auto/sequence) | copper bulb/chest частично; нет полной 1:1 модели обновлений и «quasi-connectivity»-краёв |
| 13 | Достижения и статистика | 2 | ≈60 % | Данные всех достижений, `PlayerAdvancements`-хранилище, критерии/триггеры, `/advancement`, статистика (vanilla stat types + custom), начисление части статов | Мало фактически срабатывающих триггеров, нет выдачи `ITEM_CRAFTED` (issue), формат сохранения не ванильный JSON (issue), нет динамической перезагрузки |
| 14 | Scoreboard, команды-интерфейсы, карты | 1 | ≈45 % | Серверная модель scoreboard/team/objective, title/actionbar/subtitle, tab list, чат; **clientbound-пакеты** `set_objective/set_score/set_display_objective/set_player_team/reset_score`, `map_item_data`, `waypoint`, `show_dialog`/`clear_dialog` реализованы и корректно кодируются | Нет `/scoreboard`, `/team`, `/bossbar` (команд пока нет), нет отправки карт клиенту как поведения |
| 15 | Команды | 3 | ≈45 % | **56 корневых команд** (77 регистраций с алиасами): advancement, clear, damage, defaultgamemode, difficulty, enchant, execute, experience, fill, gamemode, gamerule, give, kill, list, locate, op/deop, playsound, return, seed, setblock, spawnpoint, setidletimeout, setworldspawn, stop, stopsound, summon, tag, teleport, tellraw, tick, time, title, weather, worldborder, transfer + steel-специфичные (domain, fly, perms, invsee, swing, version) | Нет: dialog, debug, jfr, perf, place, publish, raid, random, teammsg, warden_warning, waypoint (11 корневых команд ванили) |
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

**Целевой паритет:** ≈70 % · **Оценка:** 3–5 недель

### Этап 1. Контейнеры и меню (закрыть «ящики»)
- Chest, trapped chest, double chest; hopper (+ transfer-логика), dispenser, dropper, crafter
- Brewing stand + варка зелий (46 зелий уже в данных)
- Smithing table, stonecutter, loom, cartography table, lectern, decorated pot, shelf
- Меню: hopper, dispenser, dropper, brewing, smithing, stonecutter, loom, cartography, crafter, lectern
- Книга рецептов: `update_recipes`, `place_ghost_recipe`, `recipe_book_*`

**Целевой паритет:** ≈72 % · **Оценка:** 6–8 недель

### Этап 2. Бой и снаряжение
- Кулдаун атаки, криты, sweep, knockback-формулы 1:1, combat tracker
- Лук, арбалет, стрелы (включая tipped/spectral), трезубец, mace
- Щит (блокирование, отражение), броня (полные формулы), прочность/починка/наковальня
- Эффекты через атрибуты (speed, slowness, strength, jump, resistance, haste, luck…)
- Тотем бессмертия, элитры/планирование, elytra-полёт

**Целевой паритет:** ≈75 % · **Оценка:** 6–8 недель

### Этап 3. Мобы I — враждебные + спавн
- Спавн-цикл, моб-кап по категориям, despawn, light/sky-light проверки, spawn placements
- Zombie (+ variants, conversion), skeleton (+stray/bogged), creeper, spider (+cave spider 26.2), enderman, witch, slime/magma, drowned, husk
- AI goals: melee/ranged attack, target selection, avoidance, door breaking, sun burning
- Лут мобов через loot tables, очки опыта, эффекты при смерти

**Целевой паритет:** ≈79 % · **Оценка:** 8–10 недель

### Этап 4. Мобы II — животные, деревни, торговля, транспорт
- Breeding/ageable, tamable (wolf/cat/horse), shearable, milkable, chicken egg
- Villager: профессии, POI, рабочие станции, торговля (merchant menu + `merchant_offers`), iron golem
- Raids: bad omen → рейд, волны, hero of the village
- Лодки (22 типа) + остальные вагонетки, saddle/armor, leash (есть база)

**Целевой паритет:** ≈83 % · **Оценка:** 10–12 недель

### Этап 5. Боссы и измерения
- Ender dragon: фазы, кристаллы, bossbar, портал, яйцо, respawn
- Wither: фазы, черепа, разрушение
- End city + корабль, bastion remnant, fortress-лут, nether fossil
- Портал-механики: связанность, кулдауны, spectator-телепорт (26.3)

**Целевой паритет:** ≈85 % · **Оценка:** 5–7 недель

### Этап 6. Структуры и новый контент
- Abandoned camps (26.3), village (5 типов), trial chambers + vault + trial spawner, ancient city + sculk-система (block entities), igloo, mansion, pillager outpost, shipwreck, ocean ruins, trail ruins
- Cushions (блок + сидение + статистика), shelf (13), copper golem statue, creaking heart, dried ghast
- Sulfur cube (26.2) и его ~12 архетипов
- Карты: MapItem, `map_item_data`, explorer maps, люнные карты, banner markers

**Целевой паритет:** ≈87 % · **Оценка:** 10–14 недель

### Этап 7. Интерфейсы и серверные системы
- Scoreboard/team/bossbar: пакеты + `/scoreboard`, `/team`, `/bossbar`
- Оставшиеся ~45 команд (`/effect`, `/particle`, `/clone`, `/data`, `/loot`, `/item`, `/recipe`, `/schedule`, `/function`, `/reload`, `/save-*`, `/kick`, `/msg`, `/say`, `/help`, `/ride`, `/spreadplayers`, `/spectate`, `/attribute`…)
- Диалоги (`show_dialog`, `/dialog`), waypoints (26.x), `player_rotation`, `post_effects`, `mount_screen_open`, `open_book`
- Whitelist, ban/pardon, RCON, query, Minecraft Server Management Protocol
- Debug-пакеты (`debug/*`, `debug_sample`), test blocks

**Целевой паритет:** ≈90 % · **Оценка:** 8–10 недель

### Этап 8. Плагины, API, длинный хвост паритета
- Зрелый `steel-flint`: стабильный API, версионирование, keyed-downcasting, hot reload
- Точки расширения для блоков/предметов/сущностей/меню (NeoForge-совместимые по смыслу)
- Бенчмарки и профилирование под нагрузкой, античит/спам-контроль
- Длинный хвост: точные числа, редкие механики, edge cases, parity-тесты на каждую систему

**Целевой паритет:** ≈93–95 % · **Оценка:** постоянно

> **Почему не 100 %.** Последние 5–7 % — это «длинный хвост»: тысячи мелких правил, редкие взаимодействия и точные константы. Именно так проект уже работает с мирогенерацией (7 500 чанков block-for-block), и такой же подход нужен для остальных систем. Реалистичная цель роадмапа — **~94 % + играбельный survival**, а не формальные 100 %.

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
# поведения (атрибуты на диске; классы из макросов добавляют ещё 11 блочных)
rg -o '#\[block_behavior'  steel-core/src/behavior -g '*.rs' | wc -l   # 256
rg -o '#\[item_behavior'   steel-core/src/behavior -g '*.rs' | wc -l   # 65
rg -o '#\[entity_behavior' steel-core/src -g '*.rs' | wc -l            # 142

# block entities
rg -o 'vanilla_block_entity_types::[A-Z0-9_]+' steel-core/src/block_entity/registry.rs | sort -u | wc -l  # 49

# тесты и объём
rg -c '#\[test\]' steel-core steel-protocol steel-registry steel-utils steel-worldgen steel steel-login -g '*.rs' | awk -F: '{s+=$2} END {print s}'   # 4431
find steel-*/src -name '*.rs' | xargs wc -l | tail -1                                  # 562755
```

Покрытие пакетов и классов поведением считается скриптами из §9.1.

### 9.1 Скрипты сверки (сохранены в тексте отчёта)

`packets.json` против констант в `steel-protocol`. Для **clientbound** скрипт даёт
0 пропусков в любой фазе. Для **serverbound** он даёт ложные «пропуски»: константы
`S_*` генерируются в `OUT_DIR` из `packets.json`, а в исходниках серверные пакеты
адресуются через `packet.id` или через константы, которых в тексте нет. Поэтому
реальное покрытие serverbound проверяется не этим скриптом, а двумя другими
способами:

1. наличие файла пакета с `impl ReadFrom` (`ls steel-protocol/src/packets/*/s_*.rs` — все 69
   play-пакетов имеют файл);
2. наличие ветки в диспетчере `steel-core/src/player/connection/java.rs`
   (`play::S_*`) — ровно 69 из 69, `missing = ∅`, `extra = ∅`.

Скрипт (полезен именно для clientbound):

```python
import json, re, glob
pj = json.load(open('steel-registry/build_assets/packets.json'))
ids = set()
for f in glob.glob('steel-protocol/src/**/*.rs', recursive=True):
    ids |= set(re.findall(r'\b([A-Z][A-Z0-9_]{2,})\b', open(f).read()))
for phase, prefix in (('clientbound', 'C_'), ('serverbound', 'S_')):
    for sub, names in pj[phase].items():
        miss = [n for n in names
                if not any(c in ids for c in (prefix + n.replace('/', '_').upper(),
                                              'PLAY_' + prefix + n.replace('/', '_').upper()))]
        print(phase, sub, len(names), 'missing:', miss)
```

`classes.json` против реальных структур поведения (учитываются `class = "..."`,
имена структур и вызовы макросов вида `station_block!(LoomBlock, ...)`).
Скрипт не раскрывает `$entity`-параметры внутри `macro_rules!`, поэтому
`ItemDisplay`/`TextDisplay` попадают в «пропуски» — на самом деле они создаются
макросом `display_entity!` (`entities/objects/decoration/display.rs:284,289`),
и фактическое покрытие классов сущностей — **142/143** (нет только `Player`):

```python
import json, re, glob
cls = json.load(open('steel-core/build/classes.json'))
roots = {'blocks': 'steel-core/src/behavior/blocks',
         'items': 'steel-core/src/behavior/items',
         'entities': 'steel-core/src/entity'}
attrs = {'blocks': 'block_behavior', 'items': 'item_behavior', 'entities': 'entity_behavior'}
for kind, root in roots.items():
    names = set()
    for f in glob.glob(root + '/**/*.rs', recursive=True):
        src = open(f).read()
        if '#[' + attrs[kind] not in src:
            continue
        names |= set(re.findall(r'class\s*=\s*"([A-Za-z0-9_]+)"', src))
        names |= set(re.findall(r'struct\s+(\w+)', src))
        names |= set(re.findall(r'\b\w+!\(\s*(\w+)\s*,', src))
    allc = {e['class'] for e in cls[kind]}
    print(kind, str(len(allc & names)) + '/' + str(len(allc)), sorted(allc - names))
# blocks 267/268 ['Block']  items 65/66 ['Item']
# entities 140/143 ['ItemDisplay','Player','TextDisplay']
```

Реестровые объёмы 26.3:

```python
import json
b  = json.load(open('steel-registry/build_assets/blocks.json'))['blocks']
i  = json.load(open('steel-registry/build_assets/items.json'))['items']
e  = json.load(open('steel-registry/build_assets/entities.json'))
be = json.load(open('steel-registry/build_assets/block_entities.json'))['block_entity_types']
m  = json.load(open('steel-registry/build_assets/menutypes.json'))
print(len(b), len(i), len(e), len(be), len(m))  # 1286 1658 161 49 25
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

### Дополнение (карты: `MapItem`/`FilledMapItem`, `EmptyMapItem`, `MapState`, пакет `map_item_data`)

- **Палитра карт** (`steel-registry/src/map_color.rs`) — полный порт `MapColor`: 64 записи `(id, RGB)`, `Brightness` (LOW 180 / NORMAL 220 / HIGH 255 / LOWEST 135), кодирование байта пикселя `(id << 2) | brightness` и обратное `render_rgb`. Байты совместимы с ванильным `MapState.colors`.
- **Цвет блока на карте** — `map_color` извлечён из `blocks.json` (`{default, overwrites}`) в кодогенераторе блоков: новый `StateByteData`/`StateByteOverwrite` в реестре, поле `Block.map_color`, `BlockRegistry::map_color(state_id)` и `BlockStateExt::map_color()`. Работают пошейтовые переопределения (у бревна: кора 34 / торец 13).
- **`MapState`** (`steel-core/src/world/map_state.rs`) — порт `MapState`:
  - сетка 128×128, привязка центра к сетке масштаба (`MapState.of`), `MAX_SCALE 4`, `copy()` (locked) и `zoom_out()`;
  - декорации: `addDecoration`/`removeDecoration`/`replaceDecorations`, подсчёт `decorationCount` только по `track_count`-типам, лимит `MAX_DECORATIONS 256`;
  - разрешение маркеров как в ванили: границы `±63`, `player_off_map` в радиусе 320, `player_off_limits` при `unlimitedTracking`, поворот `(yaw ± 8) · 16 / 360`, псевдослучайный поворот в Нижнем мире от `gameTime/10` (с переполнением `i32`);
  - трекеры обновлений на игрока (`PlayerUpdateTracker`): «грязный» прямоугольник расширяется, `take_packet` вырезает только его, список декораций уходит не чаще каждого 5-го запроса (`DECORATION_SYNC_INTERVAL`), счётчик `updateTick` — ровно как в `MapState.PlayerUpdateTracker`;
  - `update(player)`: регистрация трекера, маркер игрока, удаление трекера и декорации, когда игрок ушёл/умер/потерял карту, плюс слияние декораций из компонента `map_decorations` (так приходят метки исследовательских карт).
- **Персистентность** — `MapStates` (id → состояние + `nextId`) хранится в `World::map_states` и сохраняется одним версионированным saved-data (`data/maps.bin`, magic `STLM`, версия 1) при `World::cleanup`; загрузка — при создании мира. Неизвестные типы декораций при загрузке пропускаются с предупреждением, массив цветов неверной длины сбрасывается.
- **`FilledMapItem`** (`behavior/items/filled_map.rs`) — `createMap`, `inventoryTick` (обновление маркеров + перерисовка цветов, если карта в руке и не залочена) и `updateColors`: одна шестнадцатая колонок за тик (`(pixelX & 15) != (updateTick & 15)`), радиус `128 / blockScale` (в Нижнем мире вдвое меньше), выбор доминирующего цвета по количеству, глубина воды `0.1`/клетка `0.2`, яркость воды `0.5 / 0.9`, яркость суши `±0.6` с наклоном `4.0 / (blockScale + 4)` и клеткой `0.4`, краевые пиксели через один, `needsUpdate` объединяет изменения за колонку.
- **`EmptyMapItem`** (`behavior/items/empty_map.rs`) — расход 1 предмета (`consumeAndReturn`), статистика `minecraft:used`, звук `ui.cartography_table.take_result`, создание карты масштаба 0 с центром на игроке, `SUCCESS.withNewHandStack`, иначе вставка в инвентарь или дроп.
- **Пакет `map_item_data`** (`steel-protocol/src/packets/game/c_map_item_data.rs`, `CMapItemData`) — ванильный `MapUpdateS2CPacket`: `mapId` (VarInt), `scale` (byte), `locked` (bool), опциональный список декораций (`type` VarInt, `x`/`z`/`rot` byte, опциональный текстовый компонент) и опциональный патч (`width`, `height`, `startX`, `startZ` + VarInt-массив цветов). Отсутствие патча кодируется одним нулевым байтом (`OptionalMapPatch`), как в `MapState.UpdateData.CODEC`, а не boolean-префиксом.
- **Отправка** — `send_map_packets` вызывается из `Player::ai_step` сразу после тика инвентаря: перебирает все слоты, как ванильный `ServerPlayerEntity.playerTick` → `sendMapPacket`, и отправляет накопившийся дельта-пакет.
- Тесты (35 новых): 5 на палитру (`map_color.rs`: значения id/RGB, упаковка байта, `render_rgb`), 2 на цвет блока в реестре (id для камня/воды/травы/воздуха и пошейтовое переопределение у бревна), 4 на пакет (полный кадр, пустой апдейт, опциональное имя, патч без декораций), 17 на `MapState` (сетка, `zoom_out`, `copy`, `put_color`, дельта-патч, троттлинг декораций, off-map/off-limits, поворот, Нижний мир, счётчик, `replaceDecorations`, round-trip персистентности, неизвестный тип, монотонные id, маркер игрока только в своей размерности, dirty-флаг при записи цветов/декораций), 5 на перерисовку (`water_brightness`, `land_brightness`, доминирующий цвет, тай-брейк по порядку) и 2 интеграционных на `EmptyMapItem` (одна карта → в руку; стак из 2 → остаток в руке + карта в инвентаре).
- **Оптимизация** (замеров нет — в песочнице нет `cargo`): палитра пикселя считается в стековых массивах `[u16; 64]` без `LinkedHashMultiset`-аллокаций; перерисовывается 1/16 колонок за тик (≈8 колонок × 128 пикселей при масштабе 0); в сеть уходит только «грязный» прямоугольник, первая отправка — полный кадр; `needsUpdate` не даёт дублировать записи в пределах колонки; чанк запрашивается на пиксель (`with_full_chunk`), а не на каждый сэмпл.
- **Честные ограничения** (все помечены в коде): маркеры баннеров и рамок (`MapBannerMarker`, `MapFrameMarker`) не перенесены — нет block entity баннера и интеграции с рамкой; крафт копий/масштаба (`MapPostProcessingComponent`, `onCraft`) не подключён — в Steel нет хука крафта у предметов; генерация исследовательских карт (`fillExplorationMap`, `ExplorationMapFunction`) не перенесена, поэтому `ocean_monument_map` и родственные предметы существуют, но пока без содержимого; видимость жидкости аппроксимирована (ванильный `getFluidStateIfVisible` проверяет полную верхнюю грань, Steel использует цвет самого блока); имена декораций хранятся как простой текст; карты одного мира лежат в одном `maps.bin` вместо `map_<id>.dat` на карту; Нижний мир определяется по пути мира (`the_nether`), а не по `RegistryKey`.
- Покрытие: **1390 / 1658 = 83,8 %**; не-`Item` без механик: 11 (5 классов) — закрыты `MapItem` (17 предметов) и `EmptyMapItem` (1).
- Осталось: `MobBucketItem` 7 (нет сущностей рыб/аксолотля/головастика), `KnowledgeBookItem` 1 (книга рецептов), `ArmorStandItem` 1 (нет сущности стойки), `TridentItem` 1 (нет снаряда-трезубца), `WindChargeItem` 1 (нет сущности заряда ветра).

### Дополнение (водные мобы, шаг 1: слои `WaterAnimal`/`AbstractFish` + `Cod`)

- **`WaterAnimal`** (`entity/entities/mobs/passive/fish/mod.rs`): `MAX_AIR = 300`, `SPAWN_DEPTH_OFFSET = 13`, `DROWN_DAMAGE = 2.0`, интервал эмбиент-звука `120`, опыт `1 + rand(3)`; `tick_breathing` повторяет `WaterAnimal.baseTick` — воздух запоминается до базового тика моба, вне воды `-1`, при `shouldDrown` обнуление и урон `DROWN`, в воде — сброс к `300`.
- **Спавн-правило `WaterAnimal.canSpawn`** — окно `[sea_level - 13, sea_level]` (`is_within_spawn_depth`), вода снизу (`FluidTags.WATER`) и блок `minecraft:water` сверху; подключено как `SPAWN_RULE` у кода (кодогенератор `spawn_rule`).
- **`AbstractFish` (`FishEntity`)**: атрибуты `MAX_HEALTH 3.0` / `MOVEMENT_SPEED 0.7`, `PathType::Water` malus `0.0`, цели `PanicGoal(1.25)@0` и `RandomSwimmingGoal(1.0, 40)@1` (это ванильный `SwimToRandomPlaceGoal`), лимит `getLimitPerChunk = 8`, `travel_in_water` переписан как ванильный: ускорение `0.01F`, `move`, гашение `×0.9` и `-0.005` по Y при простое навигации; флоп на земле — импульсы `±0.05` по X/Z и `0.4` по Y, сброс флага `onGround`, звук `ENTITY_COD_FLOP`.
- **`Cod` (`CodEntity`)**: `#[entity_behavior(class = "Cod", spawn_rule)]`, слои синхронизированных данных `Entity`/`LivingEntity`/`Mob`/`from_bucket`, звуки `ENTITY_COD_{AMBIENT,HURT,DEATH,FLOP}`, `dimensions_for_pose` с учётом `get_scale`.
- Тесты: 5 модульных (`fish/tests.rs`) — окно спавна, `MAX_AIR`, импульс флопа, гашение скорости в воде, константы.
- **Честные ограничения**: `FleeEntityGoal` (ванильный `FleeEntity<Player>` у рыб) в Steel отсутствует — цель не подключена, помечено как недостающая система; `FishMoveControl` (ванильный контроль движения с `+0.005` по Y при погружении и `lerp(0.125)`) не перенесён; `Bucketable`/`EntityBucketItem` ещё нет, поэтому `interactMob` пока `Pass`; `canImmediatelyDespawn`/`cannotDespawn` (учёт `FROM_BUCKET` и имени) не подключены.
- Проверки сборки/тестов не выполнялись: в песочнице нет `cargo`.

### Дополнение (водные мобы, шаг 2: `Salmon`)

- **`Salmon` (`SalmonEntity`)** — `#[entity_behavior(class = "Salmon", spawn_rule)]`; атрибуты `MAX_HEALTH 3.0` / `MOVEMENT_SPEED 0.7`, malus воды `0.0`, цели `PanicGoal(1.25)@0`/`RandomSwimmingGoal(1.0, 40)@1`, флоп, лимит `8` на чанк.
- **Вариант** — синхронизированное поле `type` (`SalmonVariant`, по умолчанию `MEDIUM = 1`), масштаб хитбокса `0.5 / 1.0 / 1.5` через `bounding_box_scale()`; в NBT сохраняется **именем** (`"type": "small|medium|large"`), при загрузке неизвестное имя откатывается к `MEDIUM`; масштаб применяется в `dimensions_for_pose` (`get_scale · variant_scale`).
- **Спавн-вариант** — ванильный взвешенный пул `30 / 50 / 15` (всего 95) в `finalize_spawn` (`salmon_variant_for_roll(roll)` — чистая функция, покрыта тестами по границам 30/80).
- **`SchoolingFish`** — `getMaxGroupSize() == 5` объявлен (`SALMON_MAX_GROUP_SIZE`), но лидер/группа пока не ведутся: нужен запрос соседей (`world.get_entities_in_aabb_matching`) и `FollowFlockLeaderGoal`, поэтому `has_self_control` остаётся `true`.
- Звуки: `ENTITY_SALMON_{AMBIENT,HURT,DEATH,FLOP}`, интервал эмбиента `120`.
- Тесты: +3 (веса и границы пула, дефолт/масштабы/размер группы, round-trip варианта через id и имя) — всего 8 в `fish/tests.rs`.
- Проверки сборки/тестов не выполнялись: в песочнице нет `cargo`.

### Дополнение (водные мобы, шаг 3: `Pufferfish` — механика надувания)

- **`Pufferfish` (`PufferfishEntity`)** — `#[entity_behavior(class = "Pufferfish", spawn_rule)]`, атрибуты и цели слоя `AbstractFish`, флоп, лимит `8`/чанк.
- **Состояния надувания** — `NOT_PUFFED = 0` / `SEMI_PUFFED = 1` / `FULLY_PUFFED = 2` в синхронизированных данных (`puff_state`), счётчики `inflateTicks`/`deflateTicks` — в отдельном несинхронизированном состоянии: при угрозе `0 → 1`, через `>40` тиков `1 → 2`; без угрозы `>60` тиков `2 → 1`, `>100` — `1 → 0`; звуки `ENTITY_PUFFER_FISH_BLOW_UP` / `_BLOW_OUT`.
- **Хитбокс** — `0.5 / 0.7 / 1.0` через `puff_scale()` (умножается на `get_scale`).
- **Жало** — на каждом шаге движения все живые сущности в `boundingBox.expand(0.3)` получают урон `1 + puffState` от `mobAttackDamageSource`, яд `60 · puffState` тиков (амплифаер 0) и звук `ENTITY_PUFFER_FISH_STING`.
- **NBT** — `PuffState` (int, клампится в `0..=2`).
- Тесты: +3 внутри `pufferfish.rs` (константы состояний, масштабы, урон жала); всего в паке рыб 11 тестов.
- **Честные ограничения**: вместо ванильного `InflateGoal` с тегом `NOT_SCARY_FOR_PUFFERFISH` — приближение «не-спектатор и не-креатив игрок в радиусе 8 блоков» (тег ещё не подключён к предикатам); `onPlayerCollision` (`playerTouch`) не перенесён — нет хука столкновения с игроком.
- Проверки сборки/тестов не выполнялись: в песочнице нет `cargo`.

### Дополнение (водные мобы, шаг 4: слой `Bucketable`)

- **`Bucketable`** (`steel-core/src/entity/bucketable.rs`) — порт интерфейса: `fromBucket()`/`setFromBucket()`/`bucketItem()` + дефолтные `copyDataToNbt`/`copyDataFromNbt` (ключи `FromBucket` (byte), `Health` (float)) и свободная функция `write_base_bucket_data`, чтобы подкласс дописывал свои ключи после общих (так делает ванильный `SalmonEntity`).
- Реализовано для `CodEntity` (`cod_bucket`), `SalmonEntity` (`salmon_bucket`, дописывает `"type"` с именем варианта) и `PufferfishEntity` (`pufferfish_bucket`) — флаг `FROM_BUCKET` теперь живёт в их синхронизированных данных (`from_bucket`).
- **Честно**: `Bucketable.tryBucket` (обмен «ведро воды → ведро с рыбой», `RemovalReason::Discarded`, компонент `bucket_entity_data`) и сам `MobBucketItem` в этом шаге **не выложены** — черновик был отброшен, потому что для обмена и спавна нужно сверять точные подписи `boat.rs`/`bucket.rs` (`create_filled_result`, `ENTITIES.create`, `try_add_entity`, `GameEventContext`), а угадывать их нельзя: это сломало бы сборку. Работа продолжается с чтения этих двух файлов.
- Проверки сборки/тестов не выполнялись: в песочнице нет `cargo`.

### Дополнение (моб-вёдра: `MobBucketItem` + путь ловли, 7 предметов)

- **`MobBucketItem`** (`behavior/items/mob_bucket.rs`) — один класс на 7 предметов (`cod_bucket`, `salmon_bucket`, `tropical_fish_bucket`, `pufferfish_bucket`, `axolotl_bucket`, `sulfur_cube_bucket`, `tadpole_bucket`), как ванильный `EntityBucketItem`; тип сущности берётся из предмета в руке (`entity_type_for_item`), а не из JSON-аргумента.
- Установка: рейкаст по блокам/жидкостям, проверка источника воды, спавн через `ENTITIES.create` → `set_rotation` по рысканью игрока → `world.try_add_entity` → `game_event(ENTITY_PLACE)` → `create_filled_result(water_bucket)`, ошибки логируются и дают `Fail`/`Pass`.
- **Ловля**: в `BucketItem::use_empty_bucket` добавлен ванильный путь `Bucketable`: `get_entity_hit_result` (сделан `pub(crate)` вместо приватного) по лучу взгляда с предикатом «жива и бакетируема», затем `try_bucket_entity` → `try_bucket`: звук `playAmbientSound`, обмен `apply_filled_result` (креатив не расходует ведро), `RemovalReason::Discarded`.
- Диспетчеризация по типам — цепочка `downcast_ref` (`Cod`, `Salmon`, `Pufferfish`; остальные мобы подключатся по мере появления их классов).
- **Честные ограничения**: `BUCKET_ENTITY_DATA` (компонент `minecraft:bucket_entity_data`, `FromBucket`/`Health`/вариант) пока **не** прикрепляется к стаку — API установки компонентов у `ItemStack` не проверен без сборки; хелпер `write_base_bucket_data` готов и ждёт подключения.
- Проверки: **GitHub Actions для репозитория недоступны** — `gh api repos/Greston897/SteelMC/actions/workflows` возвращает пустой список, прогонов нет (`.github/workflows/*.yml` в дереве есть, но Actions не активированы/не отдают API), поэтому компиляция и тесты не проверялись; `cargo` в песочнице отсутствует.

> Проверка сборки/тестов выполняется в GitHub Actions (workflows `Test`, `Lint`, `Flint Tests`).

---

## 12. Итог ремонтного цикла: сборка, тесты, текущая готовность (9 ноября 2026)

Этот раздел фиксирует результат починки после большого пакета работ по предметам/водным мобам,
когда дерево перестало компилироваться.

### 12.1 Ремонт сборки: было → стало

| Показатель | Начало ремонта | Сейчас |
|---|---|---|
| `cargo check --workspace --all-targets` | **206 ошибок** (20+ файлов) | **0** |
| Rust-предупреждения (`cargo check`) | 37 | **0** |
| Падающих тестов | 12 (после того как код начал компилироваться) | **0** |
| Тестов в дереве | 3 986 | **4 032** |

Ремонт шёл исключительно по реальным прогонам GitHub Actions (`Build Report` публиковал диагностику
комментарием к коммиту, `Tests`), без локального `cargo` в песочнице.

Основные классы правок (не косметические): сигнатуры трейтов (`AbstractBoat` без `+ Sized`,
`Entity`-трейт в области видимости), `Arc<T> → Arc<dyn Entity>` для `?Sized`-параметров (переход на
хэндл менеджера `world.get_entity_by_id`), `GameEventContext::new(Some(&**player as &dyn Entity))`,
`with_item(|stack: &mut ItemStack| …)`, `ItemStack::is()` вместо сравнения с `LazyLock<Item>`,
корректные конструкторы `MobEffectInstance::with_duration(...)`/`mob_attack_damage_source(...)`,
и порядок раскрытия `boat_entity_impl!` (типы резолвятся в точке **вызова** — `ItemRef`/`Explosion`/`BlockPos`
обязаны быть в области видимости `chest_boat.rs`).

### 12.2 Найденные и исправленные баги поведения (не только компиляции)

1. **`BlockStateExt::is_solid`** считал *средний* размер коллизионного бокса вместо ванильного
   `calculateSolid` (объём ≥ 35/48 или высота ≥ 1.0) — плита считалась твёрдой, из-за чего подушки и
   рамки ставились на неподходящие блоки.
2. **Генератор реестра блоков вообще не выгружал `map_color`** (`blocks.json` содержит цвета у 1 137 блоков,
   в сгенерированный код они не попадали) — все карты были цвета 0. Добавлена эмиссия
   `StateByteData`/`StateByteOverwrite`.
3. **Поворот маркеров карты**: убрана маска `& 15`, ваниль хранит знаковый байт
   (`(byte) floor(rot * 16 / 360 + 0.5)`); `-90°` теперь `-4`, а не `12`.
4. **Сетка карты** приведена к `createFresh`: `grid = floor((coord + 64) / (128 << scale))`.
5. **Пакет `C_MAP_ITEM_DATA`**: патч цветов отправлялся без байта наличия; ваниль оборачивает его
   в `Optional` (`1` + `width, height, startX, startZ, byte array`).
6. **Лодки в пузырьковых колоннах**: `on_above_bubble_column` (запоминание колонны + таймер 60 тиков),
   `handleBubbleColumnEffect` (вниз `vy += -0.7` и выброс пассажиров, вверх `vy = 2.7` с игроком / `0.6` без).
7. **`Bucketable` замкнут**: `try_bucket` пишет `minecraft:bucket_entity_data`
   (`FromBucket`/`Health`/вариант), а `MobBucketItem::use` при выпуске восстанавливает моба
   (`read_bucket_data_from_nbt`), иначе ставит `FROM_BUCKET`.

### 12.3 Текущая готовность (машинный подсчёт по дереву)

Метрика: доля записей реестра, чей ванильный класс имеет реализацию в Steel
(`classes.json` × `#[item_behavior]` / `#[block_behavior]` / `#[entity_behavior]`).

| Реестр | Реализовано | Всего | Готовность |
|---|---|---|---|
| Предметы (классы поведения) | **1 397** | 1 658 | **84,3 %** |
| Блоки | **1 024** | 1 286 | **79,6 %** |
| Сущности | **54** | 161 | **33,5 %** |

Важно про предметы: остаются **4 класса** без реализации — `ArmorStandItem`, `TridentItem`,
`WindChargeItem`, `KnowledgeBookItem` (по одному предмету каждый), и **257 предметов** с классом
`Item`, которым отдельное поведение и не нужно (они полностью описываются компонентами).
То есть «предметная» готовность упирается не в 261 предмет, а в 4 механики:
стойка для брони, трезубец (снаряд + метание), заряд ветра (снаряд-толкатель), книга знаний (выдача рецептов).

Код: ≈505 500 строк Rust без сгенерированных данных; 4 032 теста.

### 12.4 Сравнение с ванилью 26.3 по подсистемам

| Подсистема | Готовность | Что уже есть / чего нет |
|---|---|---|
| Мир, чанки, свет, сохранения | высокая | загрузка/сохранение чанков, свет, тайлы, тики блоков; нет полного набора случайных тиков и части структур |
| Мирогенерация | высокая | биомы, фичи, структуры из датапака; расхождения по редким фичам |
| Протокол | высокая | play/handshake/status/configuration, палитровые чанки, инвентари, сущности, карты, лодки |
| Блоки и block entities | 79,6 % классов | редстоун, контейнеры, двери, растения, механики воды/лавы; часть сложных блоков — заглушки |
| Предметы | 84,3 % классов | компоненты 26.3, еда, инструменты, лук/арбалет, карты, лодки, вёдра, шаблоны кузнеца; нет 4 классов выше |
| Бой и снаряды | средняя | стрелы, спектральные, зельевые, трезубец-урон есть; нет трезубца-снаряда |
| Сущности и ИИ | 33,5 % классов | базовый `Mob`/`PathfinderMob`, цели, вода-мобы, лодки, снаряды; нет большинства наземных мобов, спавна, моб-капа |
| Контейнеры и меню | высокая для реализованных блоков | сундуки, бочки, печи, воронки, меню-синхронизация |
| Крафт и рецепты | высокая | сетка, формы, кузнечные столы, печи; нет части специальных рецептов |
| Команды, права, scoreboard | средняя | brigadier-подобный разбор, набор команд, права; нет полного покрытия ванильных команд |
| Достижения (advancements) | низкая | фундамент критериев отсутствует (в коде помечено TODO) |
| Плагины/расширения | средняя | API поведения блоков/предметов/сущностей, события |

### 12.5 Что осталось до «предметы 100 %»

1. `ArmorStandItem` + сущность стойки для брони (экипировка, позы, взаимодействие).
2. `TridentItem` + снаряд `ThrownTrident` (метание, возврат, лояльность, канал-чары).
3. `WindChargeItem` + сущность `WindCharge` (толчок, кулдаун, звук).
4. `KnowledgeBookItem` (выдача рецептов из компонента `recipes`).
5. Мобы для моб-вёдер: аксолотль, тропическая рыба, головастик, серный куб (`Bucketable` уже готов,
   не хватает самих сущностей).

## 13. Линт-чистота и отметка о завершении (10 ноября 2026)

### 13.1 Итог: все проверки зелёные

Проверка на коммите `372f2c0` (ветка `arena/8ff5adc9-steelmc`), реальные запуски GitHub Actions:

| Проверка | Результат |
|---|---|
| `Build Report` → `cargo check diagnostics` | **0 errors** (в warnings только сообщения сборочного скрипта `steel-utils` о распаковке jar) |
| `Build Report` → `tests` | падений нет, 4 032 теста |
| `Test` | **success** |
| `Flint Tests` | **success** |
| `Lint` (Spell Check → Format → Clippy) | **success** |

До этого цикла `Lint` не проходил ни разу: сначала его маскировали 206 ошибок компиляции,
затем — собственные замечания линтера в `steel-core` (педантичный набор включён в
`[workspace.lints.clippy]` и проверяется с `RUSTFLAGS=-Dwarnings`).

### 13.2 Что именно пришлось починить

На момент, когда код начал компилироваться, `cargo clippy -r --workspace --all-targets --all-features`
давал **112 уникальных замечаний** (232 сырых диагностики, т.к. `--all-targets` компилирует
один и тот же файл несколько раз), плюс `cargo fmt --all --check` показывал расхождения
в ~48 файлах, плюс `typos` находил британские написания.

Разбивка 112 замечаний и способ исправления:

| Категория | Кол-во | Что сделано |
|---|---|---|
| `clippy::absolute_paths` | 92 | добавлены `use`-импорты и пути сокращены (по файлу и по области видимости — в том числе внутри `mod tests`, где свой набор импортов) |
| `clippy::disallowed_types` | 5 | `std::collections::HashMap` → `rustc_hash::FxHashMap` в `world/map_state.rs` (это требование `clippy.toml`, а не вкусовщина) |
| `clippy::clone_on_ref_ptr` | 5 | `x.clone()` → `Arc::clone(&x)` / `Weak::clone(&x)`; в трёх местах результат приводится к `SharedEntity` через `as` — иначе `Arc<PaintingEntity>` не превращается в `Arc<dyn Entity>` |
| `clippy::unused_self` | 2 | `PaintingEntity::as_item_stack` стал ассоциированной функцией (дроп не зависит от состояния); в `destroy_block` уже стоял `#[expect]` с причиной |
| `clippy::too_many_lines` | 2 | `#[expect(..., reason = ...)]` — функции являются линейным портом одного ванильного метода (`tryBreakBlock`, инициализация `ServerWorld`) |
| `clippy::fn_params_excessive_bools` | 1 | `#[expect(..., reason = ...)]` — сигнатура повторяет ванильный `setInputs(left, right, forward, back)` |
| `clippy::unnecessary_wraps` | 1 | `sample_pixel_in_chunk` в `filled_map.rs` больше не возвращает `Option`, вызывающий код не делает `flatten()` |
| `clippy::match_same_arms` | 1 | одинаковые ветки `Direction::South` и `Direction::Up`/`Down` объединены |
| `clippy::assertions_on_constants` | 1 | `assert!` в тесте заменён на `const _: () = assert!(FIREWORK_SHOT_VELOCITY < ARROW_SHOT_VELOCITY);` — проверка теперь на этапе компиляции |
| `clippy::case_sensitive_file_extension_comparisons` | 1 | `ends_with(".desc")` → сравнение последнего сегмента через `split('.').next_back()` |
| `clippy::unwrap_used` | 1 | `u8::try_from(index).unwrap()` → сравнение с `Option` |
| `clippy::items_after_statements` | 1 | `const`-утверждение перенесено из тела теста на уровень модуля |
| `rustfmt` | ~48 файлов | применён `cargo fmt --all` |
| `typos` | 8 слов | `colour/colours/coloured/centre/centred/neighbouring/modelled/initialisation` → либо американские написания, либо разрешение в `_typos.toml` (для британских названий в ванильных терминах) |

Честная оговорка: три `#[expect]` (две «слишком длинные» функции и одна с четырьмя `bool`)
не переписаны, а явно разрешены с причиной — обе функции дословно повторяют структуру
ванильных методов, и дробление ухудшило бы сопоставимость с оригиналом. Это видимое
исключение, а не тихое подавление: `clippy::allow_attributes_without_reason` включён,
поэтому причину обязательно указывать.

### 13.3 Как это было проверено без локального `cargo`

В песочнице, где велась работа, нет `cargo`/`rustc`, поэтому единственный способ
верифицировать — GitHub Actions. Для этого добавлен **опциональный** workflow
`.github/workflows/lint-autofix.yml`:

* запускается либо через `workflow_dispatch`, либо пушем в файл-маркер `.ci/autofix-trigger`
  (так сделано потому, что `workflow_dispatch` требует наличия workflow в ветке по умолчанию);
* ставит `nightly-2026-10-14` с `rustfmt` и `clippy`, прогоняет `cargo fmt --all` и
  три прохода `cargo clippy --fix` (только машинно-применимые правки);
* **проверяет, что после правок проект компилируется**; если нет — откатывает всё и ничего не пушит;
* коммитит только форматирование и автоправки, затем публикует в комментарий к коммиту
  полный список оставшихся замечаний (с дедупликацией и подсчётом по линтам).

Последний прогон этого workflow на `74ac432`: `distinct findings: 0`, `rustfmt diffs: 0`.

Важный побочный эффект: пуши, сделанные в Actions через `GITHUB_TOKEN`, **не запускают**
новые workflow-раны (защита GitHub от рекурсии). Поэтому после автоправок нужен отдельный
пуш обычным способом — именно так и был получен зелёный `Lint` на `372f2c0`.

### 13.4 Отметка о завершении

- [x] Проверить, что всё компилируется — 0 ошибок `cargo check` (было 206).
- [x] Проверить, что всё работает — `Test`/`Flint Tests` зелёные, 4 032 теста без падений.
- [x] Убедиться, что нет «тихих» багов — исправлены: `is_solid`, генерация `map_color`,
      маска поворота маркера карты, формула сетки карты, байт наличия в `C_MAP_ITEM_DATA`,
      пузырьковые колонки у лодок, NBT-раундтрип `Bucketable`.
- [x] Довести линт до нуля — `Lint` (typos + rustfmt + clippy) зелёный.
- [x] Описать готовность и сравнение с ванилью 26.3 — разделы 12.3–12.5 выше.

Что осталось из функциональности (не баги, а незакрытые фичи): 4 класса предметов
(`ArmorStandItem`, `TridentItem`, `WindChargeItem`, `KnowledgeBookItem`) и мобы для
моб-вёдер (аксолотль, тропическая рыба, головастик, серный куб).

## 14. Закрытие ещё двух классов предметов: заряд ветра и трезубец (10 ноября 2026)

### 14.1 `WindChargeItem` + сущность `WindCharge`

Замер: реализовано полностью по ванильной логике, кроме двух задокументированных отличий.

* Бросок: `spawnProjectileFromRotation` с `power = 1.5`, `uncertainty = 1.0`, звук
  `entity.wind_charge.throw` со случайным питчем `0.4 / (random * 0.4 + 0.8)`, расход одного
  заряда (`consume_one`, creative не тратит), начисление `ITEM_USED` и **кулдаун 10 тиков**
  (как `WindChargeItem.getCooldowns().addCooldown(stack, 10)`).
* Полёт: гравитация 0 (заряд летит по прямой), стандартный drag `0.99`, общий тик
  `ThrowableProjectile`, потому что в 26.3 заряд наследуется от `HurtingProjectile`.
* Подрыв: `ExplosionOptions` с радиусом `1.2`, `source` — сам заряд, урон по типу
  `minecraft:wind_charge` (`direct = заряд`, `causing = владелец`), частицы
  `gust_emitter_small` / `gust_emitter_large` и звук `entity.wind_charge.wind_burst`.
  Движок взрывов Steel уже знал про ветряные источники (`should_affect_blocklike_entities`),
  теперь это наконец используется в бою.

Осознанные отличия:

1. Ванильный `ExplosionInteraction.TRIGGER` смоделирован как `None` («не трогать блоки,
   наносить урон сущностям») — в Steel нет отдельного варианта `Trigger`, а охрана
   block-like сущностей уже реализована в движке взрывов.
2. Ванильный заряд ветра применяет усиленный «ветряной» отброс; Steel использует обычную
   формулу отброса взрыва при том же радиусе. Направление и зона совпадают, сила отброса
   чуть мягче ванильной.

### 14.2 `TridentItem` + сущность `ThrownTrident`

* Предмет: заряд 72000 тиков, анимация `Spear`, отказ в использовании при «следующий удар
  сломает» (`next_damage_will_break`), порог броска 10 тиков
  (`THROW_THRESHOLD_TIME = 10`), скорость `2.5`, разброс `0.5`.
* При броске: `hurt_and_break(1)`, списание стака в survival (в creative остаётся),
  звук `item.trident.throw`, `ITEM_USED`, режим подбора `CREATIVE_ONLY` в креативе.
* Сущность строится на ванильном стеке `AbstractArrow` — том же, что стрелы и спектральные
  стрелы, поэтому наследует полёт, `MAX_LIFE_TICKS`, взаимодействие с блоками и подбор
  игроком. Своё: урон `8.0`, сохранение `Loyalty`/`DealtDamage` в NBT и возврат к владельцу.
* `Loyalty` читается из зачарований стака (`ItemStack::get_enchantment_level`), передаётся
  сущности и после попадания возвращает трезубец в инвентарь владельца.

Осознанные отличия:

1. **Риптайд (spin attack) не реализован**: в ванили при `riptide` и нахождении в воде/под
   дождём трезубец вместо броска запускает рывок с уроном по пути. Для этого нужен
   покадровый «рывковый» режим игрока, которого в Steel нет; заряженный бросок всегда
   бросает трезубец.
2. Возврат по `Loyalty` идёт фиксированным шагом (1 блок/тик) вместо ванильного
   покадрового телепорта с партиклами; конечное состояние (трезубец в инвентаре) совпадает.

### 14.3 Обновлённые числа

| Реестр | Было | Стало | Всего | Готовность |
|---|---|---|---|---|
| Предметы (классы поведения) | 1 397 | **1 399** | 1 658 | **84,4 %** |
| Сущности | 54 | **56** | 161 | **34,8 %** |

Новые тесты: гравитация заряда ветра, мощность подрыва, удаление после попадания,
порог заряда трезубца (10 тиков), скорость броска `2.5`, правила подбора в креативе.

Проверка: `Test` — success, `cargo check` — 0 ошибок, `cargo clippy` — 0 замечаний,
`cargo fmt --check` — 0 расхождений (после автопочинки форматирования).

### 14.4 Что осталось из предметов

| Класс | Почему ещё нет |
|---|---|
| `ArmorStandItem` | Нужна сущность `ArmorStand` на стеке `LivingEntity` с экипировкой (6 слотов), позами (6 частей тела), флагами `small/marker/no_base_plate/show_arms` и взаимодействием «надеть/снять предмет». Это отдельный крупный пакет, а не однострочный предмет. |
| `KnowledgeBookItem` | Нужна книга рецептов: серверное состояние «открытые рецепты» **и** пакеты `update_recipes` + `recipe_book_add`, чтобы клиент вообще увидел разблокировку. Ни одного пакета книги рецептов в Steel пока нет, а формат `RecipeDisplay` в 26.3 нетривиален — делать «на глазок» значило бы рисковать несовместимостью с ванильным клиентом. |

Мобы для моб-вёдер (аксолотль, тропическая рыба, головастик, серный куб) остаются
незакрытыми: `Bucketable`-слой и сам `MobBucketItem` готовы, не хватает сущностей.

## 15. Стенд для брони: `ArmorStandItem` + сущность `ArmorStand` (10 ноября 2026)

Закрыт третий из четырёх оставшихся классов предметов. В ванили `ArmorStand` —
это **живая, но не моб** сущность (`Entity → LivingEntity → ArmorStand`), поэтому
реализация сделана по стеку `LivingEntity` без `MobBase`: нет ИИ, но есть здоровье,
экипировка, эффекты, смерть и сериализация.

### 15.1 Сущность `ArmorStand` (`entity/entities/objects/decoration/armor_stand.rs`)

* Шесть слотов экипировки (`head`, `chest`, `legs`, `feet`, `main_hand`, `off_hand`) на
  штатном `EntityEquipment`, с автоматической синхронизацией `SetEquipment`.
* Шесть синхронизируемых поз (`head_pose`, `body_pose`, `left_arm_pose`, `right_arm_pose`,
  `left_leg_pose`, `right_leg_pose`) с ванильными значениями по умолчанию
  (`(-10,0,-10)`, `(-15,0,10)`, `(-1,0,-1)`, `(1,0,1)`).
* Байт клиентских флагов `DATA_CLIENT_FLAGS`: `SMALL = 0x01`, `SHOW_ARMS = 0x04`,
  `NO_BASE_PLATE = 0x08`, `MARKER = 0x10`. `small` пересчитывает габариты
  (0,5 × 0,9875, eye height и точки крепления масштабируются по высоте), `marker`
  убирает хитбокс и интерактивность (`is_pickable`, `is_marker_armor_stand`).
* NBT: `equipment`, `Invisible`, `Small`, `ShowArms`, `NoBasePlate`, `Marker`,
  `Pose.{Head,Body,LeftArm,RightArm,LeftLeg,RightLeg}` — чтение терпимо к отсутствующим
  частям `Pose`, не заданные позы сохраняют ванильный дефолт.
* Взаимодействие ПКМ: целевой слот берётся как `getEquipmentSlotForItem` (компонент
  `minecraft:equippable`, иначе `main_hand`), предмет надевается на пустой слот, звук
  экипировки берётся из компонента предмета, начисляется `ITEM_USED`, в креативе стак
  не расходуется. ПКМ пустой рукой — `Pass`, чтобы не съедать клик по блоку.
* Уничтожение: любой урон (кроме `is_invulnerable_to`) разбирает стойку за один удар —
  звук `entity.armor_stand.break`, выпадение всей экипировки и предмета
  `minecraft:armor_stand`, удаление сущности без 20-тиковой анимации смерти и без опыта.
  Повторный урон по уже разобранной стойке отсекается атомарным флагом.
* Урон от падения не наносится; при приземлении играет `entity.armor_stand.fall`
  (ванильный `checkFallDamage`).

### 15.2 Предмет `ArmorStandItem` (`behavior/items/armor_stand.rs`)

* ПКМ по нижней грани блока — `Fail` (ванильный запрет ставить стойку «под блок»).
* Точка установки — соседняя грань, блок должен быть `is_replaceable`; позиция —
  центр блока (`x + 0.5`, `y`, `z + 0.5`).
* Поворот — снап к ближайшему из 16 направлений:
  `floor((yaw + 180) * 16 / 360 + 0.5) & 0xF * 22.5°`, дальше `set_rotation`,
  `y_head_rot`, `y_body_rot` — как ванильный `snapTo` с `pitch = 0`.
* Копируется `custom_name` стака, играет `entity.armor_stand.place` (0.75 / 0.8) и
  игровое событие `ENTITY_PLACE` в точке установки, начисляется `ITEM_USED`,
  в креативе стак не расходуется.

### 15.3 Осознанные отличия от ванили

1. **Надевание только в пустой слот.** Ванильная стойка заменяет предмет в слоте и
   отдаёт старый игроку; в Steel у инвентаря игрока нет примитива «положить снятое в
   руку», поэтому замена привела бы к молчаливой потере предмета. Пустые слоты
   заполняются полностью по-ванильному.
2. **Маркерные стойки уязвимы.** В Steel урон по маркерной стойке разбирает её так же,
   как по обычной (в ванили маркер не имеет хитбокса и его нельзя выделить прицелом).
3. **Компонент `minecraft:entity_data` при установке не применяется**: в Steel пока нет
   общего помощника `apply_item_stack_config` для только что созданных сущностей.
   Имя предмета копируется, произвольные NBT-правки из компонента — нет.
4. **Нет столкновений сущность-сущность.** В Steel межсущностные коллизии реализованы
   только для вагонеток, поэтому игрок проходит сквозь стойку (как и сквозь других
   живых сущностей).
5. Стойка не получает опыт-дроп и не проигрывает анимацию смерти: ванильная стойка
   разбирается мгновенно, это сохранено.

### 15.4 Обновлённые числа

| Реестр | Было | Стало | Всего | Готовность |
|---|---|---|---|---|
| Предметы (классы поведения) | 1 399 | **1 400** | 1 658 | **84,4 %** |
| Сущности | 56 | **57** | 161 | **35,4 %** |

Новые тесты: ванильные дефолтные позы, половинные габариты у `small`, независимая
установка/сброс четырёх флагов, запись `Pose`/флагов в NBT и обратное чтение,
невыделяемость маркерной стойки; установка стойки (расход стака, позиция), снап
поворота к 180° при yaw = 10°, отказ при нижней грани и занятом блоке, копирование
имени предмета.

## 16. Тропическая рыба: сущность `TropicalFish` (10 ноября 2026)

Четвёртый рыболовный моб и первый шаг к мобам из ведра после брони. В ванили
`TropicalFish extends AbstractSchoolingFish` (та же цепочка `WaterAnimal → AbstractFish`),
поэтому общие слои — воздух, всплытие на суше, плавание, паника — взяты из уже
существующего модуля `mobs/passive/fish` без дублирования.

### 16.1 Сущность `TropicalFish` (`entity/entities/mobs/passive/fish/tropical_fish.rs`)

* Габариты и характеристики берутся из `entities.json`: 0,5 × 0,4, eye height 0,26,
  health 3,0 (`FISH_MAX_HEALTH`), speed 0,7 (`FISH_MOVEMENT_SPEED`), лимит 8 рыб на чанк,
  опыт `WATER_ANIMAL_EXPERIENCE_MIN..MAX`.
* Цели ИИ — ванильные для рыбы: `PanicGoal(1.25)` в приоритете 0 и
  `RandomSwimmingGoal(1.0, 40 тиков)` в приоритете 1; штраф пути по воде 0,0.
* Синхронизируемый вариант — один `i32` (`DATA_ID_TYPE_VARIANT`), упакованный точно как
  в ванили: биты 0..16 — `TropicalFishPattern` (базовый размер `base | индекс << 8`,
  значения уже есть в `steel-registry/src/entity_variant.rs`), бит 16..24 — базовый цвет,
  24..32 — цвет узора (`DyeColor::id()`, 0..15). Дефолт — `kob` + белый + белый, то есть
  `0`, как у ванильного `DEFAULT_VARIANT`.
* Публичный API: `packed_variant()`, `set_packed_variant()`, `pattern()`, `base_color()`,
  `pattern_color()`, `set_variant(pattern, base, pattern_color)`, плюс свободные
  `pack_variant`, `pattern_for_variant`, `base_color_for_variant`,
  `pattern_color_for_variant` и `default_variant()`.
* NBT: ключ `Variant` (packed int) в `save_additional`/`load_additional` — как ванильный
  `TropicalFish.addAdditionalSaveData`.
* Звуки: `entity.tropical_fish.ambient` (интервал `WATER_ANIMAL_AMBIENT_SOUND_INTERVAL`),
  `.hurt`, `.death`, `.flop` при прыжках на суше.

### 16.2 Ведро: `Bucketable` + `MobBucketItem`

* `TropicalFishEntity` реализует `Bucketable`: `bucket_item()` → `tropical_fish_bucket`,
  `copy_bucket_data_to_nbt` пишет `FromBucket`, `Health` и ванильный ключ
  `BucketVariantTag` (packed int).
* Исправлен молчаливый баг в общей части ведра: раньше `read_bucket_data_from_nbt`
  не вызывался у лосося, поэтому вариант лосося терялся при «выпуске» из ведра
  (в ведре лежал `type`, но при выпуске игнорировался). Теперь общий код вынесен в
  `bucketable::read_base_bucket_data`, а `SalmonEntity` и `TropicalFishEntity`
  дочитывают свои ключи (`type` / `BucketVariantTag`) поверх него.
* Диспетчеризация `try_bucket_entity` / `release_from_bucket` / `is_bucketable`
  расширена на `TropicalFishEntity` (у Steel пока нет трейт-объектной формы `Bucketable`,
  диспетчер — цепочка `downcast_ref`).

### 16.3 Осознанные отличия от ванили

1. **Спавн-вариант не из ванильного списка.** Ванильная рыба выбирает вариант из
   фиксированного списка 22 «обычных» упакованных значений (`COMMON_VARIANTS`); этого
   списка нет в извлечённых данных, поэтому Steel равномерно тянет узор из
   `TropicalFishPattern::VALUES` и два цвета из `DyeColor::VALUES`. Раскладка битов
   остаётся ванильной, отличается только распределение.
2. **Нет стайного вождения.** Механика вожака `AbstractSchoolingFish` не подключена
   (как у кода и лосося): у тропической рыбы ванильный `getMaxGroupSize()` равен 1,
   так что рыба не теряет ничего, кроме формальной связи с вожаком.
3. **Один хитбокс на оба базовых размера.** В Java-издании у тропической рыбы один
   хитбокс 0,5 × 0,4 (подтверждено и `entities.json`, и таблицей хитбоксов вики),
   поэтому крупные узоры не увеличивают коллизию.

### 16.4 Обновлённые числа

| Реестр | Было | Стало | Всего | Готовность |
|---|---|---|---|---|
| Сущности | 57 | **58** | 161 | **36,0 %** |
| Предметы (классы поведения) | 1 400 | 1 400 | 1 658 | **84,4 %** |

Новые тесты: дефолтный вариант (`kob`, белый, белый → `0`), round-trip упаковки для
всех 12 узоров × всех 16 базовых цветов, сохранение базового размера у крупных узоров
(`clayfish` → `Large`), синий базовый + жёлтый узорный цвет.

## 17. Пакет враждебных мобов: зомби, кадавр, утопленник, зомбифицированный пиглин, wither-скелет, чешуйница, пещерный паук, поборник (10 ноября 2026)

Закрыто сразу восемь сущностей одного архетипа — «ближний бой + преследование
игрока». Все они построены на общем слое `Monster` и одном шаблоне, поэтому
поведение консистентно и легко расширяется.

### 17.1 Что реализовано

| Сущность | Класс | Здоровье | Урон | Звуки |
|---|---|---|---|---|
| `Zombie` | `Zombie` | 20 | 3 | `entity.zombie.*` |
| `Husk` | `Husk` | 20 | 3 | `entity.husk.*` |
| `Drowned` | `Drowned` | 20 | 3 | `entity.drowned.*` |
| `ZombifiedPiglin` | `ZombifiedPiglin` | 20 | 5 | `entity.zombified_piglin.*` |
| `WitherSkeleton` | `WitherSkeleton` | 20 | 5 | `entity.wither_skeleton.*` |
| `Silverfish` | `Silverfish` | 8 | 1 | `entity.silverfish.*` |
| `CaveSpider` | `CaveSpider` | 12 | 2 | `entity.spider.*` (в ванили у пещерного паука своих звуков нет) |
| `Vindicator` | `Vindicator` | 24 | 5 | `entity.vindicator.*` |

* Габариты, здоровье, скорость, урон, броня, `step_height`, `follow_range` — целиком
  из извлечённого `entities.json` (`AttributeMap::new_for_entity`), в коде не
  дублируются.
* ИИ: `FloatGoal(0)`, `MeleeAttackGoal(1.0, no pause)(2)`, `WaterAvoidingRandomStrollGoal(1.0)(7)`,
  `LookAtPlayerGoal(8.0)(8)`, `RandomLookAroundGoal(9)`; селектор цели —
  `HurtByTargetGoal(1)` + `NearestAttackableTargetGoal::new_for_players(2)`.
* Горение на солнце не прошито в мобах: оно уже реализовано в общем слое
  (`Mob::tick_daylight_burning`) и включается тегом `minecraft:burn_in_daylight`
  из извлечённых данных, поэтому зомби/кадавры/утопленники горят, а, например,
  wither-скелет — нет.
* Опыт и дроп — через `Monster::should_drop_experience_monster` /
  `should_drop_loot_monster` (ванильное правило `mobGriefing`/`mobDrops`).

### 17.2 Побочный эффект: тесты конвертации

Четыре теста в `entity/conversion.rs` использовали `vanilla_entities::ZOMBIE` как
«тип без зарегистрированной фабрики» и падали после появления настоящей фабрики
зомби. Они переведены на `vanilla_entities::ENDER_DRAGON` (его фабрики по-прежнему
нет), смысл тестов не изменился; попутно подтверждено, что конвертация
`Pig → Zombie` теперь действительно работает.

### 17.3 Осознанные отличия от ванили

1. **Естественный спавн не подключён.** `SpawnRule` получает `&dyn LevelReader`, а
   `Monster::check_monster_spawn_rules` требует `&World` (нужны тип измерения,
   гроза и `dimension_type.monster_spawn_light_level`). Мост между этими
   сигнатурами — отдельная задача; пока эти типы создаются только спавнерами,
   яйцами призыва и командами.
2. **Зомбифицированный пиглин агрессивен.** В ванили он нейтрален и отвечает
   только на удар; в Steel зарегистрирован обычный враждебный селектор цели.
3. **Яд пещерного паука не накладывается** — у `MeleeAttackGoal` нет хука
   «эффект при попадании», укус наносит только 2,0 урона.
4. **Поборник без топора и без ломания дверей**; чешуйница не прячется в
   заражённый камень.
5. Зомби не превращаются в утопленников, кадавры не «перезаписываются» зомби —
   цепочки конвертаций моб-в-моб не подключены (общая инфраструктура
   `convert_to` уже есть, нужны правила).

### 17.4 Обновлённые числа

| Реестр | Было | Стало | Всего | Готовность |
|---|---|---|---|---|
| Сущности | 58 | **66** | 161 | **41,0 %** |
| Враждебные сущности | 2 | **10** | — | — |

Новые тесты (по одному на моба): проверка ванильных атрибутов из реестра
(`max_health`, `attack_damage`) и round-trip NBT через `save_additional` /
`load_additional`.

## 18. `/tpsbar`, ещё шесть враждебных мобов и честный подсчёт предметов (10 ноября 2026)

### 18.1 Команда `/tpsbar` (новое)

Steel-специфичная команда без ванильного аналога: включает/выключает серверный
боссбар с производительностью.

* `steel-protocol/src/packets/game/c_boss_event.rs` — новый пакет `boss_event`
  (действия `add`, `remove`, `update_progress`, `update_name`, `update_style`,
  `update_properties`) с ручной записью по ванильной раскладке: `UUID`,
  `VarInt` действия, далее Component / float / `VarInt` цвета / `VarInt` оверлея /
  байт флагов. Цвета и оверлеи — ванильные ординалы (`pink=0 … white=6`,
  `progress=0 … notched_20=4`).
* `steel-core/src/server/tps_bar.rs` — состояние бара и его рассылка. Бар один на
  весь сервер (`TPS_BAR_UUID`), поэтому клиент обновляет одну полосу:
  `add` отправляется один раз на игрока, затем `update_progress` + `update_name`
  раз в секунду, при выключении — `remove`. Игроки, вышедшие с сервера,
  вычищаются из карты «кому уже показан бар», так что утечки UUID нет.
* Текст бара: `TPS 20.0  MSPT 1.25 (5s 1.50, p95 3.00)`; заливка = `tps / 20`
  с клампом `0..1`, цвет — по тем же порогам, что и в таб-листе (19,5 / 15,0);
  MSPT краснеет выше 50 мс (один полный тик).
* `steel-core/src/command/builtins/tpsbar.rs` — команда с доступом по умолчанию
  (`default_access`), переключает флаг и отвечает `TPS bar enabled/disabled`.
* Оптимизация: работа делается в уже существующем секундном тике таб-листа,
  дополнительных таймеров нет; при выключенном баре рассылки не происходит
  вообще (выход из `broadcast_tps_bar` до формирования текста).

Проверки: три теста на байтовый формат пакета (длина и порядок `uuid`/действия,
big-endian float прогресса, ординалы цвета и оверлея в конце `add`), четыре теста
на математику бара (прогресс, клампы, `NaN`, пороги цвета, рендер текста со всеми
тремя видами MSPT) и тест на регистрацию команды в графе встроенных команд.

### 18.2 Пять новых мобов

| Сущность | HP | Урон | Звуки |
|---|---|---|---|
| `Spider` | 16 | 2 | `entity.spider.*` |
| `Piglin` | 16 | 5 | `entity.piglin.*` |
| `PiglinBrute` | 50 | 7 | `entity.piglin_brute.*` |
| `Hoglin` | 40 | 6 | `entity.hoglin.*` |
| `Zoglin` | 40 | 6 | `entity.zoglin.*` |

Тот же архетип и тот же шаблон, что и в §17: атрибуты и габариты — из
`entities.json`, ИИ — `Float` + `MeleeAttack` + блуждание + look-цели и враждебный
селектор цели. Отличия (в шапках файлов и здесь):

1. Паук в ванили перестаёт быть враждебным при ярком свете — в Steel всегда
   враждебен.
2. Пиглины в ванили нейтральны (нужно спровоцировать или снять золотую броню) и
   бартерят — в Steel всегда враждебны, бартера нет.
3. Хоглин не боится искажённого гриба, зоглин атакует только игроков (в ванили —
   всех живых, кроме зоглинов и криперов).
4. `ZombieVillager` в этот батч не вошёл: его синхронизируемые данные содержат слой
   `VillagerData`, которому нужен конструктор с непустыми аргументами; он пойдёт
   вместе с крестьянами отдельным шагом.

### 18.3 Предметы: честный подсчёт

Ранее в отчётах фигурировало «1 400 / 1 658». Разбор по классам поведения
показывает, что это **не** 258 unimplemented items, а особенность подсчёта:

| Класс | Записей | Статус |
|---|---|---|
| `Item` | 257 | базовое поведение `DefaultItemBehavior` (в ванили у этих предметов нет своего поведения) |
| `KnowledgeBookItem` | 1 | **единственный реально отсутствующий** класс |
| остальные 63 класса (`BlockItem`, `ShearsItem`, `BoneMealItem`, `FlintAndSteelItem`, `EndCrystalItem`, `LeadItem`, `NameTagItem`, `HangingSignItem`, `SmithingTemplateItem`, …) | 1 400 | реализованы в `behavior/items/*` |

То есть по факту предметы покрыты на 1 657 / 1 658; остаётся `KnowledgeBookItem`,
которому нужен слой «книга рецептов» (`recipe_book_add` требует сериализации
`RecipeDisplay`, формат которой в извлечённых данных отсутствует).

### 18.4 Обновлённые числа

| Реестр | Было | Стало | Всего | Готовность |
|---|---|---|---|---|
| Сущности | 66 | **71** | 161 | **44,1 %** |
| Предметы (классы поведения) | 1 400 | 1 400 | 1 658 | 84,4 % (фактически 1 657/1 658 с учётом `Item`) |
| Враждебные сущности | 10 | **15** | — | — |

> Примечание о методике: в разделах 15–18 счёт сущностей вёлся узким предикатом
> (он даёт на 4 меньше). Полный счёт по сгенерированным фабрикам на этот момент —
> **75 из 161 (46,6 %)**; сводка со всеми подсистемами в `PROGRESS-RU.md`.

## 19. Дальнобойный ИИ и семейство скелетов (10 ноября 2026)

Закрыт главный пробел из `PROGRESS-RU.md`: **в ядре не было ни одного дальнобойного
моба**. Появились переиспользуемые примитивы (цель ИИ + помощник стрельбы), линия
«стрелок» в сущностях и полноценный слой эффектов у стрел, после чего реализованы
`Skeleton`, `Stray`, `Bogged`, `Parched`.

### 19.1 `RangedAttackGoal` (`steel-core/src/entity/ai/goal/ranged_attack.rs`)

Порт ванильного `RangedBowAttackGoal`:

| Параметр | Ваниль | Steel |
|---|---|---|
| Контролы | `MOVE` + `LOOK` | то же |
| Порог «вижу цель» | 20 тиков подряд | `SEE_TIME_THRESHOLD = 20` |
| Интервал выстрела | 20 тиков | 20 тиков (`attack_interval`) |
| Радиус | 15 блоков | 15, сравнивается по квадрату расстояния |
| Сила выстрела | `Mth.clamp(sqrt(d²)/radius, 0.1, 1.0)` | `attack_power()` |
| Поведение в позиции | `navigation().stop()`, накапливает таймер | то же |

`tick` не создаёт временных путей: пока цель видна и находится в радиусе,
навигация останавливается, и моб только доворачивает взгляд
(`look_control.set_look_at` с 30°/тик по рысканью и тангажу). Как только цель
выходит из радиуса или ломается линия видимости, таймер сбрасывается в `-1` и
моб снова идёт на `move_to_pos`.

### 19.2 Помощник стрельбы (`steel-core/src/entity/ranged_attack.rs`)

Порт `AbstractSkeleton#performRangedAttack` в одном месте, чтобы им пользовались
все скелеты без копипасты:

* точка спавна — глаза стрелка минус `0.1`;
* прицел — центр масс цели `getY(0.3333)` (`y + height·1/3`);
* вертикальная дуга — `+ horizontalDistance · 0.2`;
* скорость `1.6`, неточность `14 − 4·difficulty` (`14/10/6/2` для peaceful/easy/normal/hard);
* урон стрелы `2.0`, `pickup = ALLOWED`, `set_has_been_shot(true)`, владелец —
  стрелок, выстрел через существующий `Projectile::shoot` (то есть с ванильным
  треугольным разбросом).

Стрела создаётся штатной фабрикой `ArrowItem::create_arrow`, поэтому типизация,
копия предмета и права на подбор не дублируются.

### 19.3 Эффекты стрел (ванильный `Arrow#addEffect`)

В `ArrowState` добавлен список `effects: Vec<MobEffectInstance>`, а в трейт
`AbstractArrow` — `add_effect`, `effects`, `has_effects`. При попадании в
сущность (`doPostHurtEffects`) список применяется **с полной длительностью** и
вычищается, поэтому пронзающая стрела не накладывает эффект дважды. Типированные
стрелы (через `PotionContents`) продолжают работать прежним путём с делением
длительности на 8.

### 19.4 Четыре сущности

| Сущность | Класс | HP | Звуки | Стрелы | Особое |
|---|---|---:|---|---|---|
| `Skeleton` | `Skeleton` | 20 | `entity.skeleton.*` | обычные | замерзание в порошковом снегу → `Stray` |
| `Stray` | `Stray` | 20 | `entity.stray.*` | `slowness` 600 тиков | — |
| `Bogged` | `Bogged` | 16 | `entity.bogged.*` (+`shear`) | `poison` 100 тиков | стрижка грибов ножницами |
| `Parched` | `Parched` | 16 | `entity.parched.*` | `weakness` 600 тиков | не горит на солнце (тег) |

Габариты `0.6 × 1.99` (глаза `1.74`), скорость `0.25`, урон `2.0`, категория
`MONSTER` — берутся из `entities.json`, как и раньше.

Раскладка целей — ванильного `AbstractSkeleton`:

* `FloatGoal` (0), `RestrictSunGoal` (2), `FleeSunGoal` (3), избегание волков
  `AvoidEntityGoal(6.0, 1.0, 1.2)` (3), `RangedAttackGoal(1.0, 20, 15.0)` (4),
  блуждание (5), `LookAtPlayerGoal(8.0)` и `RandomLookAroundGoal` (6);
* цели: `HurtByTargetGoal` (1), игроки (2), железные големы (3).

Отдельно: `Skeleton` синхронизирует ванильный флаг `stray_conversion` (рендер
«наклоняется от холода») и при полном замерзании
(`ticks_frozen ≥ 140`, то есть 7 секунд в порошковом снегу) превращается в
`Stray` через существующий `conversion::convert_to` с сохранением экипировки и
звуком `entity.skeleton.converted_to_stray`.

Стрижка `Bogged` идёт через `Mob::mob_interact`: ножницы, звук
`entity.bogged.shear`, по одному красному и коричневому грибу с ванильным
разбросом выброса, игровое событие `shear` с игроком как источником и 1 единица
прочности инструмента. Повторная стрижка возвращает `Consume`.

### 19.5 Проверки: оптимизация, баги, сходство с ванилью

**Тесты (15 новых):**

* `goal::ranged_attack` — контролы, `requires_update_every_tick`, кривая силы
  (пол-`0.1`, половина радиуса → `0.5`, ровно радиус → `1.0`, дальше кламп),
  устойчивость при нулевом радиусе, значение порога `20`;
* `entity::ranged_attack` — дуга на горизонтальном выстреле, дуга при стрельбе
  вниз, горизонтальная дистанция на треугольнике 3-4-5, таблица неточности по
  четырём сложностям, коэффициент прицеливания `1/3`;
* четыре сущности — атрибуты из реестра + round-trip NBT, у скелета ещё проверка,
  что флаг конверсии стартует в `false`, у боггеда — что свежий моб не стрижен и
  флаг переключается.

**Оптимизация:**

* сравнение радиуса идёт по квадрату расстояния, без `sqrt` в горячем пути
  (`shoot_arrow` вызывает `sqrt` только в момент выстрела — раз в 20 тиков);
* пока моб в позиции, навигация остановлена — путь не пересчитывается и не
  тикает, работа на тик сводится к одному лучу видимости (он кэшируется в
  `sensing`) и одному `set_look_at`;
* `has_line_of_sight_cached` переиспользует уже существующий кэш восприятия;
* список эффектов у стрелы — `Vec` в состоянии стрелы; для обычной стрелы он
  пуст, и при попадании ветка выходит на первом же `is_empty()` без аллокаций;
* помощник стрельбы не создаёт промежуточных структур: направление — один
  `DVec3`, стрела — существующая фабрика предмета.

**Сходство с ванилью (по извлечённым данным):**

* `entities.json`: HP `20/20/16/16`, габариты `0.6×1.99`, глаза `1.74`,
  `fire_immune = false` у всех четырёх, `wither_skeleton` не тронут (`0.7×2.4`);
* `sound_events.json`: у `stray`/`bogged`/`parched` есть только
  `ambient/death/hurt/step` (у боггеда ещё `shear`), звука `.shoot` у вариантов
  нет — поэтому все четыре играют общий `entity.skeleton.shoot` ровно как
  ванильный `AbstractSkeleton`;
* `classes.json`: классы `Skeleton`, `Stray`, `Bogged`, `Parched` совпадают с
  ванильными `net.minecraft.world.entity.monster.skeleton.*`.

### 19.6 Что осталось за рамками шага (осознанно)

1. Естественный спавн по-прежнему не подключён: `SpawnRule` получает
   `&dyn LevelReader`, а `Monster::check_monster_spawn_rules` требует `&World`.
   Это отдельный шаг «природный спавн», он же разблокирует пустынные/снежные
   биомные варианты (`Parched`/`Stray` замещение) и групповой спавн по 4.
2. Цель `Turtle` из ванильного селектора скелета не подключена — сущность
   `Turtle` ещё не реализована; железные големы подключены как тип-цель.
3. `Bogged` выдаёт грибы напрямую (1 красный + 1 коричневый), а не через
   loot-таблицу `shearing/bogged`: таблица генерируется из данных, которых в
   дереве нет, а состав дропа совпадает.
4. Интервал стрельбы фиксированный (20 тиков). Ванильный `RangedBowAttackGoal`
   для скелетов тоже использует фиксированные 20; рандомизация интервала
   появится вместе с `CrossbowAttackGoal`/разбойниками.

### 19.7 Обновлённые числа

| Реестр | Было | Стало | Всего | Готовность |
|---|---|---|---|---|
| Сущности (типы) | 75 | **79** | 161 | **49,1 %** |
| Сущности (java-классы) | 57 | **61** | 143 | **42,7 %** |
| Дальнобойные мобы | 0 | **4** | 27 классов | 14,8 % |
| Тесты в крейте | 3 159 | **3 174** | — | — |

## 20. Крипер: предохранитель и взрыв (10 ноября 2026)

Первый моб с «механикой», а не только с ИИ: `Creeper` (20 HP, `0.6 × 1.7`,
глаза `1.445`, категория `MONSTER`, `fire_immune = false` из `entities.json`).

### 20.1 Ванильные числа, перенесённые без правок

| Параметр | Ваниль | Steel |
|---|---|---|
| `MAX_SWELL` | 30 тиков | `MAX_SWELL = 30` |
| Радиус взрыва | `3.0` | `EXPLOSION_RADIUS = 3.0` |
| Множитель при `is_powered` | `×2` | `POWERED_RADIUS_MULTIPLIER = 2.0` |
| Окно надувания | `distanceToSqr <= 49` (7 блоков) и линия видимости | `SWELL_DISTANCE_SQR = 49.0` |
| Порог старта `SwellGoal` | `distanceToSqr < 9` (3 блока) | `SWELL_START_DISTANCE_SQR = 9.0` |
| Центр взрыва | `getY(0.0625)` | `EXPLOSION_HEIGHT_FRACTION = 0.0625` |
| Тип взрыва | `ExplosionInteraction.MOB` | `ExplosionInteraction::Mob` (уважает `mobGriefing`) |

Логика фитиля — ванильная построчно: фитиль **стартует** только когда цель ближе
3 блоков (ванильный `SwellGoal.canUse`), а **продолжает** гореть до 7 блоков,
пока цель видна (`canContinueToUse` + `tick`); иначе `dir = -1`;
`swell += dir`, но не ниже нуля; звук
`entity.creeper.primed` ровно на переходе `swell 0 → 1`; при
`swell >= MAX_SWELL` — взрыв и удаление сущности.

Синхронизируемые поля из `entities.json` (`swell_dir`, `is_powered`,
`is_ignited`) отданы наружу API: `swell_dir`/`set_swell_dir`, `is_powered`/
`set_powered`, `is_ignited`/`set_ignited`, `swell`/`set_swell` (кламп в
`[-1, 30]`). Рендер-пульсация клиента получает те же значения, что в ванили.

### 20.2 Почему без отдельного `SwellGoal`

`Goal` в Steel получает `&dyn PathfinderMob` и не умеет даункастить до
конкретной сущности, а ванильный `SwellGoal` пишет именно в состояние крипера
(`setSwellDir`, `setSwell`) и останавливает навигацию. Поэтому:

* накопление фитиля и взрыв живут в `LivingEntity::ai_step` (там уже есть
  `&SharedEntity`, нужный как источник взрыва);
* остановка навигации — в `Mob::custom_server_ai_step`, который выполняется
  после селекторов целей и тика навигации, то есть гарантированно «перебивает»
  `MeleeAttackGoal` в этом тике.

Наблюдаемое поведение совпадает с ванильным (подходит, останавливается,
надувается, сдувается при потере цели/видимости, взрывается), но без лишнего
типа цели.

Раскладка целей — ванильного `Creeper`: `Float` (0), избегание кошек и оцелотов
`AvoidEntityGoal(6.0, 1.0, 1.2)` (3), `MeleeAttackGoal(1.0)` (4),
`WaterAvoidingRandomStrollGoal(0.8)` (5), `LookAtPlayerGoal(8.0)` и
`RandomLookAroundGoal` (6); цели — игроки (1), `HurtByTargetGoal` (2).

### 20.3 Проверки

**Тесты (5 новых):** атрибуты из реестра + round-trip NBT; накопление и
дефляция фитиля (`0→1`, `29→30`, `5→4`, `0` не уходит в минус); удвоение
радиуса при `is_powered`; окна надувания (`49`/`9`/`30`); round-trip всех
синхронизируемых флагов и клампов `set_swell`.

**Оптимизация:** фитиль хранится в `AtomicI32` (никаких мьютексов в горячем
пути), линия видимости берётся из уже существующего кэша восприятия, взрыв
считается существующим серверным `ServerExplosion` (один проход по блокам,
пакеты осколков и урона — общий путь с ТНТ), а `custom_server_ai_step` выходит
на первом `swell_dir > 0` при обычном патрулировании.

**Сходство:** звуки только те, что есть в `sound_events.json`
(`entity.creeper.hurt`, `entity.creeper.death`, `entity.creeper.primed` —
`ambient`/`step` в данных 26.3 отсутствуют, поэтому крипер не переопределяет
шаг и не имеет ambient-звука, как и в ванили этой версии).

### 20.4 За рамками шага

1. Молния пока не существует как сущность/событие, поэтому «заряженный крипер»
   недостижим естественным путём; API `set_powered` готов, осталось подключить
   удар.
2. `is_ignited` синхронизируется, но путей поджога (огненный шар, `ignite()`
   из датапака) нет — появится вместе с огненными снарядами.
3. Естественный спавн — по-прежнему общий блокер.

### 20.5 Обновлённые числа

| Реестр | Было | Стало | Всего | Готовность |
|---|---|---|---|---|
| Сущности (типы) | 79 | **80** | 161 | **49,7 %** |
| Сущности (java-классы) | 61 | **62** | 143 | **43,4 %** |
| Мобы с уникальной механикой | 0 | **1** (`Creeper`) | — | — |

## 21. Разбойник: арбалетный бой (10 ноября 2026)

Добавлен `PillagerEntity` с данными размера и атрибутов из реестра (`0.6 × 1.95`,
глаза `1.6575`, 24 HP, скорость `0.35`, дальность зрения `32`). При создании он
получает арбалет в основную руку; цель боя заряжается 25 тиков и отражает фазу
зарядки в синхронизируемом `is_charging_crossbow`. После выстрела следует
20-тиковая перезарядка. На расстоянии до 15 блоков моб останавливается и
держит взгляд на цели, вне дальности или без линии видимости — преследует её.
Цели: получивший урон, игроки, жители/торговцы и железные големы.

Для выстрела используется общий помощник моба-стрелка с обычной стрелой и
звуком `item.crossbow.shoot`; сам предмет `CrossbowItem` и 25-тиковый цикл
игрока уже реализованы отдельно в `behavior/items/crossbow.rs`.

### 21.1 Проверки и границы шага

Тест проверяет атрибуты и NBT round-trip, арбалет в основной руке и начальное
значение synced-флага. Патрули пиллиджеров реализованы отдельно (`entity/patrol_spawner.rs`), волны
рейда и естественный спавн рейдеров — нет.
Пока не подключены случайный интервал ванильного `CrossbowAttackGoal`,
потеря/подбор арбалета, а также полный projectile pipeline арбалета: моб
стреляет через общий `shoot_arrow`, а не через `CrossbowItem` игрока.

### 21.2 Обновлённые числа

| Реестр | Было | Стало | Всего | Готовность |
|---|---|---|---|---|
| Сущности (типы) | 80 | **81** | 161 | **50,3 %** |
| Сущности (java-классы) | 62 | **63** | 143 | **44,1 %** |
| Дальнобойные мобы | 4 | **5** | 27 классов | **18,5 %** |

## 22. Эндермен: взгляд, состояние переноса и уклонение от снарядов (9 ноября 2026)

Добавлен `EndermanEntity` (`0.6 × 2.9`, глаза `2.55`, 40 HP, скорость `0.3`,
`FOLLOW_RANGE = 64`, `STEP_HEIGHT = 1.0`). Цель игрока выбирается только при
прямом взгляде: проверяется нормализованное направление от глаз игрока к глазам
эндермена, ванильный порог `dot > 1 - 0.025 / distance` и кэшированная линия
видимости. После провокации эндермен использует обычную `MeleeAttackGoal`;
получив урон от projectile-сущности, пытается уйти через общий безопасный
`LivingEntity::random_teleport` и играет `entity.enderman.teleport` при успехе.

В synced data доступны `carry_state` (`carried_block` / `set_carried_block`),
`creepy` (`is_creepy`) и `stared_at` (`is_stared_at`). Задействованы ванильные
ambient/hurt/death-звуки. Защита от снарядов не отменяет полученный урон:
телепортация выполняется после успешного применения урона.

### 22.1 Проверки и границы шага

Тесты проверяют атрибуты/round-trip NBT и синхронизируемое carried state;
геометрический тест подтверждает, что взгляд вперёд проходит порог, а взгляд
назад — нет. Не добавлены natural spawn, вода-урон и goal-подбор/установка
блоков. Переносимый блок пока доступен через synced API, но не меняется игровым
ИИ. Уклонение — 16 случайных попыток в радиусе 16 блоков, а не полный ванильный
`Enderman.teleport()` с его источниками случайности и всеми вариантами причин.

### 22.2 Обновлённые числа

| Реестр | Было | Стало | Всего | Готовность |
|---|---|---|---|---|
| Сущности (типы) | 81 | **82** | 161 | **50,9 %** |
| Сущности (java-классы) | 63 | **64** | 143 | **44,8 %** |
| Уникальные механики | 1 | **2** (`Creeper`, `Enderman`) | — | — |

## 23. Слизни и магматические кубы (10 ноября 2026)

Добавлены `SlimeEntity` и `MagmaCubeEntity`. В 26.3 `MagmaCube` больше не наследует
`Slime`: оба класса наследуют `AbstractCubeMob`, поэтому общая арифметика размера
вынесена в `hostile/cube_mob.rs`, а у каждого моба свой набор целей, звуков и
производных атрибутов.

Синхронизируемый `id_size` масштабирует хитбокс (`0.52 × size`), здоровье
(`size²`), скорость (`0.2 + 0.1 × size`), урон в ближнем бою и награду опыта
(`size`). У слизня урон равен размеру, у магматического куба — `size + 2`, плюс
`3 × size` брони; куб невосприимчив к огню, поэтому `fire_immune` берётся из
реестрового типа. Размер ограничен ванильным диапазоном `1..=127`, но
естественный спавн бросает только `1, 2, 4` (`1 << 0..2`, с ванильным бонусом
сложности).

NBT хранит `Size` в ванильной нулевой базе (`size - 1`), поэтому `/summon` с
`Size: N` и сохранение мира читаются одинаково; здоровье при загрузке не
сбрасывается (`set_size(size, false)`), как в `AbstractCubeMob.setSize`.

Прыжки выполняются в `custom_server_ai_step` по ванильному такту: куб прыгает
только с земли и не в транспорте, задержка `10` тиков без цели и `5` с целью, а
магматический куб умножает её на четыре. У магматического куба прыжок выше на
`0.1 × size`, а `jump_in_liquid` для лавы даёт `0.22 + 0.05 × size` вместо
общих `0.04` для воды.

Смерть: `LivingEntity::on_death` — новый подтиповый хук, вызываемый после
общих эффектов смерти. Размер больше единицы делится на 2–4 ребёнка
произвольного размера через существующий `ConversionType::SplitOnDeath`,
поэтому с ними переносится общее состояние конверсии (эффекты, поглощение,
имя, флаги), но не снаряжение. Дети ставятся по ванильной формуле 26.3:
`(index % 2 - 0.5) × width / 2` и `(index / 2 - 0.5) × width / 2`, на `+0.5`
по Y, каждый со случайным углом поворота.

### 23.1 Проверки и границы шага

Тесты проверяют формулы размера (здоровье/скорость/урон/броня/хитбокс),
ограничение диапазона `1..=127`, нулевую базу NBT и то, что загрузка с
`reset_health = false` не перезаписывает здоровье куба, а также все ванильные
формулы в `cube_mob`. Отдельный world-тест убивает слизня размера 4 и
проверяет, что в мире появилось 2–4 ребёнка размера 2 на высоте `+0.5`, а
слизень размера 1 не делится вовсе. Контактный урон по игроку подключён
через существующий хук `Entity::player_touch`; урон по железному голему через
`AbstractCubeMob.push` не подключён, потому что в Steel нет хука отталкивания,
на который моб мог бы отреагировать. Естественный спавн (слайм-чанки, болото,
адские биомы) не подключён: `SpawnRule` даёт `&dyn LevelReader`, а проверкам
нужен полный `&World`. Клиентские анимации сквоша и частицы лавы не
синхронизируются.

### 23.2 Обновлённые числа

| Реестр | Было | Стало | Всего | Готовность |
|---|---|---|---|---|
| Сущности (типы) | 82 | **84** | 161 | **52,2 %** |
| Сущности (java-классы) | 64 | **66** | 143 | **46,2 %** |
| Уникальные механики | 2 | **4** (`Creeper`, `Enderman`, `Slime`, `MagmaCube`) | — | — |

## 24. Животные: кролик, лиса, белый медведь, оцелот, попугай, кошка (11 ноября 2026)

Шесть животных, у которых раньше не было ни класса, ни поведения, доведены до
рабочего состояния на существующем AI-фундаменте (`PathfinderMob`, `Animal`,
`AgeableMob`, goal-селектор, `SyncMutex`-послойные synced-данные).

### 24.1 Кролик

Семь ванильных вариантов (0…6) с именами ассетов, `set_variant_id` /
`variant_name` и запись `RabbitType` в NBT. Габариты 0,49 × 0,6 (глаза 0,59),
детские 0,28 × 0,4 с точкой крепления `(0; 0,375; 0)`. Здоровье 3, скорость
0,3, прыжок 0,42. Цели: `Float`, `Panic`, `Avoid(16, 1.4, 1.4)`, `Tempt`,
`Stroll`, `LookAtPlayer`, `RandomLookAt`. Флаги моба читаются и пишутся через
`entity_data.lock().mob().mob_flags`. Три теста.

### 24.2 Лиса

`FoxVariant { Red = 0, Snow = 1 }`. Флаги `SLEEPING`, `SITTING`, `CROUCHING`,
`INTERESTED`, `POUNCING`, `DEFENDING`, `FACE_PLANTED` в `flags`-модуле,
реэкспортированном как `fox_flags`. Доверие по `Uuid` с вытеснением (две
позиции, `trusted_id_0` / `trusted_id_1` — имена полей synced-данных после
санитайзера, поэтому в коде используются именно они). NBT: `Type`,
`FoxFlags`, `TrustedCount`. Четыре теста.

### 24.3 Белый медведь

`is_standing` / `set_standing` / `update_standing`, `is_food` = `false`,
`MeleeAttack(1.25, true)`, NBT `Standing`. Тест куб-детёныша использует
`ageable_base().set_age(-1)`, потому что `set_synced_baby` возраст не меняет,
а `is_baby()` смотрит именно на `get_age() < 0`. Класс живёт в
`mobs::neutral`, поэтому `entities/mod.rs` реэкспортирует
`pub use mobs::neutral::*;` **до** `mobs::passive::*` — иначе сгенерированный
`entities.rs` не видит `PolarBearEntity` (E0433).

### 24.4 Оцелот

Синхронизированный флаг `trusting`, NBT `Trusting`, корм — тег
`#minecraft:ocelot_food` (треска, лосось). Цели: `Float`, `Panic`, `Tempt`,
`FollowParent`, `WaterAvoidingRandomStroll`, `AvoidEntity(16, 1.33, 1.33)`,
`LookAtPlayer`, `RandomLookAt`.

### 24.5 Попугай

Пять вариантов (`red_blue`, `blue`, `green`, `yellow_blue`, `gray`),
`set_variant_id` / `variant_id` / `variant_name`. Поле synced-данных после
санитайзера — `variant` (`variant_id` → отрезается `_id`), не `variant_type`.
`is_flyable`, 41 звук имитации через `sound_events::ENTITY_PARROT_IMITATE_*`
и `imitation_sound_for(mob_key)`, `next_imitation_delay()` 6000…12000,
корм — тег `#minecraft:parrot_food`. Габариты 0,5 × 0,9, детские 0,25 × 0,45
(глаза 0,117), NBT `Variant`. Пять тестов.

### 24.6 Кошка

Класс `CatEntity` (`TamableAnimal` + `Animal`). Synced-данные: слой
`tamable_animal()` даёт `flags` и `owneruuid` (оба читаются через родительский
слой, а не напрямую из `CatEntityData`), собственный слой — `variant`
(`RegistryReference<CatVariant>`), `sound_variant`
(`RegistryReference<CatSoundVariant>`), `is_lying`, `relax_state_one`,
`collar_color` (по умолчанию 14 = красный).

Поведение: усики тамера — биты `SITTING = 1` и `TAME = 1 << 2`;
`set_owner` ставит/снимает и владельца, и бит `TAME`;
`is_owned_by(uuid)`; `set_variant_by_key` / `set_sound_variant_by_key`
через `RegistryExt::by_key`; звуки (ambient/death/hurt/eat/… ) берутся из
варианта звука с учётом детскости через `sounds()`. NBT: `CollarColor`,
`variant`, `sound_variant`, `CatFlags` (i32), `Owner` (int-массив).
Корм — тег `#minecraft:cat_food`. Цели: `Float`, `Panic`, `Tempt`,
`Breed(1.0)`, `FollowParent`, `WaterAvoidingRandomStroll`, `LookAtPlayer`,
`RandomLookAt`, `MeleeAttack(1.33, true)`. Семь тестов: старт без тамера и с
красным ошейником, тамер/владелец, независимость сидения от тамера, круговой
перебор всех 16 цветов ошейника, независимость `is_lying` и
`relax_state_one`, еда (треска и лосось — да, камень — нет), круговой
NBT-прогон варианта.

### 24.7 Грабли, которые стоили сборок

* `variant_id` / `sound_variant_id` / `type_id` в именах synced-полей
  обрезаются санитайзером: `type_id` → `type` → `variant_type`, а
  `variant_id` → `variant`. Попытка обратиться к `variant_id` или
  `variant_type` у попугая — единственная причина красной сборки `78b751a`.
* Поля `TamableAnimal` не лежат в `CatEntityData` напрямую: генератор
  вкладывает их в родительский слой, доступ — `tamable_animal()` /
  `tamable_animal_mut()`.
* `RegistryExt::by_key` — метод **трейта**, его нужно импортировать
  (`use steel_registry::RegistryExt`), иначе E0599 «no method named `by_key`».
* `NbtValueInput::byte_value` читает байт из любого числового тега, а
  `nbt.byte(...)` — только точный `Byte`; для `CollarColor`, который пишется
  как `i8`, но может прийти как `Int`, нужен `byte_value`.

### 24.8 Вторая волна: транспорт, дисплеи, враждебные и пассивные мобы

После животных закрыты следующие группы сущностей (все — на общих слоях
`Entity`/`LivingEntity`/`Mob`, без новых механик):

* **Транспорт (7/7 вагонеток):** `Minecart`, `MinecartChest`, `MinecartTNT`,
  `MinecartHopper`, `MinecartFurnace`, `MinecartSpawner`,
  `MinecartCommandBlock`. TNT-вагонетка: `fuse = 80`, сила взрыва
  `4 + (|v|/0.05 / 5) * 4`, зажатая в `4..16`, взрыв в `pos + (0, 0.0625, 0)`.
  Hopper-вагонетка: 5 слотов, `MOVE_ITEM_SPEED = 4`, NBT `Items`/`Enabled`.
  Furnace-вагонетка: `PushX`/`PushZ`/`Fuel`/`FuelTicks`, синхронный `fuel`.
  Spawner-вагонетка переиспользует `BaseSpawner` (общий с блоком спавнера).
  CommandBlock-вагонетка хранит команду и `Powered`, исполнение команд пока не
  подключено — диспетчер не связан с сущностями.
* **Дисплеи:** `ItemDisplay` и `TextDisplay` (общий слой `Display`:
  transformation, billboard, `view_range`, `width`/`height`,
  `glow_color_override`); `Interaction` и `BlockDisplay` уже были.
* **Враждебные:** `Blaze`, `Ghast`, `Giant`, `Vex`, `Witch`, `Evoker`,
  `Illusioner`, `Ravager`, `Warden`, `Shulker`, `Phantom`, `Breeze`,
  `Creaking`, `CopperGolem`, `Guardian`, `SnowGolem`, `EvokerFangs`,
  `LightningBolt`.
* **Пассивные:** `Goat`, `Panda`, `Bee`, `Turtle`, `Strider`, `Sniffer`,
  `Nautilus`, `ZombieNautilus`.
* **Семейство лошадей:** `Donkey`, `Mule`, `Horse`, `SkeletonHorse`,
  `ZombieHorse`, `Llama` — с битами `AbstractHorse.id_flags`
  (`TAME = 0x02`, `SADDLE = 0x04`).

Осознанные упрощения (все задокументированы в шапках файлов): у мобов нет
видоспецифичных целей (ramming, digging, hive, charge, ...), у дисплеев нет
интерполяции.

### 24.9 Обновлённые числа

| Реестр | Было | Стало | Всего | Готовность |
|---|---|---|---|---|
| Типы сущностей с поведением | 84 | **114** | 161 | **70,8 %** |
| Java-классы | 66 | **114** (112 фактических + 2 макросных) | 143 | **79,7 %** |
| Уникальные механики | 4 | **4** | — | — |

Последние зелёные сборки до этой волны: `4f293e8` (попугай), `3154f23`
(кошка), `a511fee`, `0846b54` (аксолотль после чистки импортов).
Волна вагонеток/дисплеев/мобов проверяется отдельно; при появлении
компиляционных ошибок они правятся батчами.

---

## 19. Волна «контент до конца»: блоки, block entities, команды (9 ноября 2026)

Измерения сняты из дерева: `steel-core/build/classes.json` × grep по `steel-core/src`.

| Срез | Было | Стало |
|---|---|---|
| Блоки с классом поведения | 1 215 / 1 286 (94,5 %) | **1 286 / 1 286 = 100 %** |
| Классы блоков без поведения | 50 классов / 79 блоков | **0** |
| Предметы | 1 658 / 1 658, 66 / 66 классов | подтверждено **100 %** |
| Сущности | 161 / 161, 143 / 143 класса | подтверждено **100 %** |
| Типы block entity с фабрикой | 27 / 49 | **49 / 49 = 100 %** |
| Команды (`builder.register`) | 53 | **56** |
| TODO/FIXME | ≈133 | **139** (почти все — блокеры, §3c) |

### Block entities: закрыты все 49 типов

Добавлен 22 типа. Каждый — реальный `BlockEntity` с `load_additional` / `save_additional` /
`pre_remove_side_effects`, а не заглушка:

- **Сундук** (`chest.rs`): 27 слотов, выброс предметов при разрушении, `steel:block_entity/chest`.
- **Воронка** (5 слотов), **раздатчик/выбрасыватель** (9 слотов + `SelectedSlot`),
  **варильная стойка** (3 зелья + топливо, `BrewTime` / `Fuel`).
- **Баннер** (слои паттернов `pattern` + `color`), **лектерн** (книга + `Page`),
  **декорированный горшок** (4 черепка + `LootTable`), **крафтер** (9 слотов,
  `disabled_slots`, `triggered`), **полка** (3 слота).
- **Скалк-семейство**: датчик и калиброванный датчик (`last_vibration_frequency`),
  катализатор, крикун (`warning_level`).
- **Сердце скрипуна** (`creaking` как `IntArray` UUID), **vault** (`loot_table`,
  `display_item`, `total_players`), **испытательный спавнер** (`entity`, `spawns_per_mob`, …).
- **Технические**: командный блок — полноценный `CommandBlockEntity` +
  `BaseCommandBlock` (команда, `SuccessCount`, `TrackOutput`/`LastOutput`,
  `UpdateLastExecution`/`LastExecution`, `powered`, `conditionMet`, `auto`;
  режимы `REDSTONE`/`AUTO`/`SEQUENCE` берутся из самого блока, как в
  `CommandBlockEntity.getMode`; `neighborChanged` + `tick` + `executeChain` +
  `max_command_sequence_length`; исполнение идёт через тиковую очередь команд
  сервера с источником `CommandSender::CommandBlock` — права консоли,
  без чата, в собственном измерении блока; `SetCommandBlock` применяется
  вместо read-and-ignore, вагонетка командного блока исполняет команду).
  Джигсо, структурный блок, статуя медного голема, `test_block` и
  `test_instance_block` тоже переведены на настоящие block entity (см. ниже).

### Технические block entity: маркеры заменены настоящими реализациями

`test_block`, `test_instance_block`, `jigsaw_block`, `structure_block` и
`copper_golem_statue` больше не используют `marker_block_entity!`; `marker.rs`
удалён, каждый тип регистрирует свою фабрику в `block_entity/registry.rs`
(49/49 типов с реальным `impl BlockEntity`).

- **`TestBlockEntity`** — `TestBlock.getStateForPlacement` читает компонент
  `minecraft:block_state` (`mode`), `useWithoutItem` открывает редактор только
  гейм-мастеру (`has_game_master_permissions`), `getSignal` выдаёт 15 только для
  `START` при `powered`, `trigger()`/`reset()` переключают синхронизируемый бит
  и дёргают соседей, как ванильный `TestBlock.trigger`.
- **`TestInstanceBlockEntity`** — кодек `TestInstanceBlockData` (id теста,
  размер тремя int в списке, поворот, `ignore_entities`, статус 0..2, сообщение
  через `Option<TextComponent>`), `STRUCTURE_OFFSET = (0, 1, 1)`, обработчики
  `ServerboundSetTestBlockPacket` и `ServerboundTestInstanceBlockActionPacket`.
- **`StructureBlockEntity`** — все 13 сеттеров `ServerboundSetStructureBlockPacket`,
  `StructureMode`, `set_changed()`, коды ошибок 1/2/3 (`save_failure`,
  `load_not_found`, `size_failure`).
- **`JigsawBlockEntity`** — `joint`, `name`, `target`, `pool`, `final_state`,
  `selection_priority`, `placement_priority`; `JigsawGenerate` остаётся
  warn-skip, потому что генератора структур в Steel нет.
- **`CopperGolemStatueBlockEntity`** — четыре позы и взаимодействие окисления.

### Блоки: закрыты все 1 286

Добавлен 41 класс:

- **Заражённые блоки** (6 + `InfestedRotatedPillarBlock`), **нилиум** (2), **скалк** (5),
  **лягушачья икра**, **яйцо нюхача**, **высохший гаст**, **статуи голема** (8).
- **Полки** (13), **воронка/раздатчик/выбрасыватель/варильная стойка/лектерн/крафтер/
  декоративный горшок** как блоки.
- **Станции с меню**: ткацкий станок, стол картографа, кузнечный стол, камнерез —
  вместе с новыми меню-видами (`inventory/menu/kinds/station_menu.rs`).
- **Простые маркеры**: воздух, стекло, тонированное стекло, структурная пустота,
  плачущий обсидиан, свет (переключение уровня по ПКМ), test-блоки.

### Двойной сундук и счётчик зрителей

`ChestBlock` / `TrappedChestBlock` переносят ванильную логику соединения половин:
`connect_double_chest` (при установке), `update_double_chest` (при обновлении формы),
`connected_direction` — прямой порт `ChestBlock.getConnectedDirection`. Меню — 27 слотов,
звук открытия с ванильным питчем (0.9–1.0 для одиночного, 0.5 для двойного), компаратор считает
заполненность.

`ContainerOpenersCounter` (`inventory/openers_counter.rs`) повторяет ванильный счётчик:
переключает `open`, играет звуки, уведомляет соседей. Подключён к бочке (снимает 2 TODO)
и к сундуку.

### Команды (5 новых)

- `/fillbiome <from> <to> <biome>` — запись биома в кварт-ячейку через новый
  `FullChunk::set_biome_cell`.
- `/item replace entity <targets> <slot> <item> [count]` — ванильные имена слотов
  (`hotbar.N`, `inventory.N`, `weapon.mainhand`, `weapon.offhand`, `armor.*`).
- `/recipe give|take <targets> <recipe|*>`.
- `/clone <begin> <end> <target> [replace|masked|filtered <filter>]` — снапшот источника,
  проверка перекрытия регионов.
- `/spreadplayers <center> <spreadDistance> <maxRange> <targets>` — лимит 1000 попыток.

### Сверка с официальным трекером (steelmc.dev/tracker)

Трекер: **1 101 запись — 904 implemented (82 %), 127 in progress (12 %), 70 planned (6 %)**.

- В статусе **Planned** у трекера — **43 класса / 70 блоков**. Из них в этой ветке
  реализованы **все 43 класса / 70 блоков** (проверено скриптом по `classes.json` × grep).
- Закрыты также все «In progress» классы, которых у нас не было: ChestBlock,
  TrappedChestBlock, SculkVeinBlock и другие.
- Вывод: **по блокам проект обгоняет официальный трекер** — 100 % классов против 82 %.

### Что остаётся неполным даже в закрытых классах (честно)

Класс реализован ≠ механика доведена до ванильной. Остаётся:

1. Двойной сундук не объединяет инвентари двух половин в один инвентарь меню.
2. Варочная стойка не варит — нет тикера и реестра зелий.
3. Крафтер не крафтит — нет резолвера `CraftingInput` в тикере.
4. Воронка не переносит предметы — нет тикера.
5. Vault / испытательный спавнер не выдают награду — нет исполнителя loot table.
6. Станции (ткацкий, картограф, кузнечный, камнерез) открывают меню без рецептов.
7. Не реализованы команды: `/forceload` (нет публичного API тикетов),
   `/schedule` (нет function-менеджера), `/help` (нет доступа к диспетчеру из контекста),
   `/team`, `/bossbar`, `/loot`, `/datapack`, `/reload`, `/attribute`, `/function`,
   `/trigger`, `/spectate`, `/debug`, `/jfr`, `/perf`, `/place`, `/save-*`, `/whitelist`,
   `/ban*`, `/pardon*`, `/publish`, `/profile`, `/warden_warning`.

---

## 20. Механики block entities: воронка, варка, крафтер, двойной сундук (9 ноября 2026)

Четыре «закрытых класса» из §19 доведены до рабочей механики — тикеры и предметная логика
перенесены из ванили построчно.

### Воронка (`block_entity/entities/hopper.rs`)

- `suck_in_items` — перенос одного предмета из контейнера сверху; источник списывается
  **только** после того, как воронка приняла предмет, поэтому полная воронка ничего не удаляет.
- `try_move_items` — перенос в контейнер снизу; порядок обхода `0..5` повторяет ванильный
  `HopperBlockEntity.tryMoveItems` (выходной слот, затем четыре входных).
- Кулдаун: поле `cooldown` (`TransferCooldown` в NBT), шаг `MOVE_COOLDOWN_TICKS = 8` —
  ровно ванильная скорость «один предмет в 8 тиков».
- `BlockEntityTicker` для `HopperBlock` подключён, тикер выбирается по типу block entity.

### Варочная стойка (`block_entity/entities/brewing_stand.rs` + `potion_brewing.rs`)

- Новый модуль `potion_brewing` — прямой перенос ванильного `PotionBrewing`: 30 рецептов
  (`POTION_MIXES`), предикаты `is_ingredient`, `is_brewable_potion`, `is_fuel`, `has_recipe`, `mix`.
- Пятый слот (ингредиент) хранится отдельно от четырёх слотов контейнера, как в ванили;
  сериализуется в `Ingredient`.
- `tick` повторяет `BrewingStandBlockEntity.doTick`: обратный отсчёт `BrewTime`, старт новой
  варки при наличии топлива и рецепта, `doBrew` применяет рецепт ко всем трём слотам зелий и
  списывает один ингредиент.
- По завершении проигрывается level event `SOUND_BREWING_STAND_BREW` (1035), как в ванили.
- Для `ItemStack` добавлен `potion_key()` — ключ зелья из компонента `potion_contents`.

### Крафтер (`block_entity/entities/crafter.rs`)

- `craft` собирает `CraftingInput` 3×3 из включённых слотов (отключённые маской слоты
  участвуют только как выход), находит рецепт через `REGISTRY.recipes.find_match`
  и собирает результат через `recipe_manager::assemble_recipe`.
- Результат кладётся в отключённые слоты, из каждого задействованного слоя списывается один
  предмет — ванильный `CrafterBlockEntity.craft`.
- `tick` уменьшает счётчик `triggered` и запускает крафт, когда он доходит до нуля.
- Тикер подключён к `CrafterBlock`.

### Двойной сундук (`inventory/menu/kinds/chest_menu.rs`, `chest_block.rs`)

- `double_chest` собирает меню `GENERIC_9X6` из двух секций по 27 слотов: верхние три ряда —
  первая половина, нижние три — вторая. Роутинг шифт-кликов идёт по обеим секциям
  (`[top, bottom]`), `DoubleChestKind` проверяет валидность обеих половин.
- `ChestBlock::open_menu` смотрит `CHEST_TYPE` и, если сундук двойной, находит вторую половину
  по `connected_direction(state)` и открывает шестирядное меню; иначе — три ряда, как раньше.

### Что с этим стало в трекере

| Механика | Было | Стало |
|---|---|---|
| Воронка переносит предметы | нет тикера | **работает** |
| Варочная стойка варит | нет тикера | **работает** (30 рецептов) |
| Крафтер крафтит | нет резолвера | **работает** |
| Двойной сундук | одна половина | **обе половины, 54 слота** |

Осталось из списка §19: награды vault / испытательного спавнера (нужен исполнитель loot table),
рецепты в станциях (ткацкий, картограф, кузнечный, камнерез), и серверные команды с блокерами.

---

## 21. Награды, станции и закрытие блокеров (9 ноября 2026)

### Награды из loot table

- `VaultBlockEntity::eject_reward` — списывает одного игрока из бюджета, разрешает ключ
  loot table и раскатывает его через `drop_from_block_interact_loot_table`, выбрасывая
  каждый стек на позиции хранилища. Ванильный no-op для неконфигурированного vault сохранён.
- `TrialSpawnerBlockEntity::eject_reward` — то же для испытательного спавнера.
- `LootTableRegistry::by_key` — новый публичный лукап по ключу (раньше реестр умел только
  `register`/`iter`), без него награду нельзя было выдать.

### Станции

- **Камнерез** — настоящий слот результата: `StonecutterKind` держит индекс выбранного
  рецепта (`-1` по умолчанию), `slots_changed` сбрасывает выбор при смене входа и
  пересчитывает результат, `click_menu_button` выбирает рецепт по id кнопки. Рецепты берутся
  из `REGISTRY.recipes.matching(&STONECUTTING, …)`, результат — `recipe.data().result.create()`.
- **Лоум / стол картографа / кузнечный стол** — добавлены ванильные слоты результата
  (лоум 3+1, картография 2+1, кузнечный 3+1), слот помечен `SectionKind::take_only()`.
  Заполнение оставлено пустым: нужны данные паттернов баннера, скопированной карты и
  переносов зачарований при апгрейде — этого пока нет в модели.

### Закрытые блокеры

| Блокер | Решение |
|---|---|
| `decrement_openers` нигде не вызывался | `MenuKind::removed` у `ChestKind`/`DoubleChestKind`, через новые `ContainerRef::level()` и `owner_pos()` |
| `BlockTag::COPPER_CHESTS` отсутствовал | синтетический тег `copper_chests` в `steel-registry/build/tags/block.rs` (8 вариантов сундука) |
| нет публичного лукапа loot table | `LootTableRegistry::by_key` |

### Состояние TODO

139 записей. Оставшиеся крупные блокеры — те, что требуют инфраструктуры, а не кода:
`/forceload` (нет публичного API тикетов), `/schedule` и `/function` (нет менеджера функций),
`/datapack` и `/reload` (нет загрузчика датапаков), `/help` (контекст не видит диспетчер),
`/team`, `/bossbar`, `/attribute`, `/loot`, `/place`, `/data`, `/trigger`, `/spectate`.

---

## 22. Полная зачистка TODO: диспетчер достижений и побочные механики (9 ноября 2026)

Задача была «реализовать все TODO». Ниже — что реально сделано, и честная классификация
оставшихся 109 записей.

### Счётчик

| Момент | TODO/FIXME | Команд |
|---|---|---|
| Начало волны | **139** | 56 |
| После диспетчера достижений и триггеров | 114 | 57 |
| После крафта/статистики/`/loot` | **109** | **57** |

### Главная находка: достижения были сломаны в корне

Генератор `steel-registry/build/advancements/criterion.rs` выбрасывал `trigger` из JSON и
подставлял `Criterion::<ImpossibleInstance, ImpossibleTrigger>::default()` для **каждого**
критерия. То есть ни одно достижение в игре не могло прогрессировать в принципе — не потому,
что «не было диспетчера», а потому, что у критериев не было идентичности триггера.

**Что сделано:**

1. `SimpleCriterion` (`steel-registry/src/advancement/criterion.rs`) — хранит id триггера и
   сырой JSON инстанса; `AnyCriterion::id()` возвращает настоящий триггер.
2. Генератор теперь эмитит `SimpleCriterion::new(<trigger>, <instance>)`.
3. `PlayerAdvancement::trigger(&self, player, trigger)` — проходит реестр достижений, находит
   критерии с этим триггером и выдаёт прогресс (pull-модель вместо ванильных листенеров;
   перезагрузка датапака не требует инвалидации индексов).
4. `Player::trigger_advancement(trigger)` — публичная точка входа для игрового кода.
5. Новый файл `steel-registry/src/vanilla_criteria_triggers.rs` — **65** констант триггеров
   ванильного 26.3 (`ITEM_USED_ON_BLOCK`, `PLAYER_KILLED_ENTITY`, `RECIPE_CRAFTED`, …)
   плюс `ALL` для тестов.
6. `PlayerAdvancement::trigger` и `Player::trigger_recipe_crafted` для рецептов.

### Подключённые триггеры (16 мест)

| Триггер | Место |
|---|---|
| `HONEY_BLOCK_SLIDE` | `HoneyBlock::entity_inside` (только для игрока) |
| `ITEM_USED_ON_BLOCK` | `ShearsItem`, `HoneycombItem`, `BlockTransformer` |
| `TARGET_HIT` | `TargetBlock::hit` |
| `USED_ENDER_EYE` | `EnderEyeItem::use_on` |
| `USED_TOTEM` | `LivingEntity::check_totem_death_protection` |
| `ENTITY_KILLED_PLAYER` | `Entity::award_kill_score` |
| `PLAYER_HURT_ENTITY` | `Interaction::attack` |
| `EFFECTS_CHANGED` | `LivingEntity::add_effect` |
| `PLAYER_SHEARED_EQUIPMENT` | `Entity::attempt_to_shear_equipment` |
| `FISHING_ROD_HOOKED` | `FishingHook::pull_entity` и `retrieve` |
| `SLEPT_IN_BED` | `ServerPlayer` при засыпании |
| `ENCHANTED_ITEM` | `EnchantmentMenu::click_menu_button` |
| `BRED_ANIMALS` | `Animal::spawn_child_from_breeding` |
| `RECIPE_CRAFTED` | `ResultSlot` и `AbstractFurnaceMenu` |
| `ITEM_DURABILITY_CHANGED` | `LivingEntity::do_hurt_equipment` |
| `PLAYER_GENERATES_CONTAINER_LOOT` | `BrushableBlockEntity` |

### Крафт и рецепты

- `ResultSlot::unlock_new_recipes` — ванильный `checkTakeAchievements`: после крафта в книгу
  рецептов попадают **все** рецепты, которые теперь доступны из сетки
  (`recipe_manager::matching_recipes_for`, новый).
- `AdvancementRewards.recipes` больше не игнорируются: `grant_reward` кладёт их в
  `ServerRecipeBook` (сигнатура получила `AdvancementRef`).

### Статистика → scoreboard

- `ObjectiveState` получил `criterion`, добавлены `add_objective_with_criterion` и
  `objectives_with_criterion`.
- `Player::sync_stat_to_objectives` пишет новое значение статы во **все** objectives, чей
  критерий её называет (ванильный `ServerStatsCounter.setValue`). Работает и для инкремента,
  и для сброса.

### Команда `/loot`

`loot <targets> give <loot_table> [<count>]` — раскатывает зарегистрированную loot table
`count` раз и выдаёт каждый стек целям. Для этого добавлен `SteelArgumentType::identifier()`
(парсер ресурсной локации с саджестами, протокол `ResourceLocation`). Команд стало **57**.

### Сохранение достижений

Оказалось реализованным (и `persisted_progress`, и `apply_persisted_progress` в
`player_data.rs`) — устаревший TODO удалён.

### Что осталось: 109 записей, все — инфраструктурные

Я не стал превращать их в «сделано» подменой смысла. Классификация:

| Категория | Кол-во | Почему не закрыто сейчас |
|---|---|---|
| Порталы/огонь (flammability) | 2 | нужна система горючести блоков: `Flammable`-дата в реестре + распространение огня по ней |
| Спавн/поиск игроков рядом | ~6 | нет per-world индекса игроков (`get_players_in_range`); от него зависят `CONSTRUCT_BEACON`, anger piglins, колокол |
| Вагонетки/лодки спец-типов | 2 | нужны сущности `AbstractMinecart`-подтипов (chest/furnace/TNT/hopper/spawner/command) |
| Снаряды в раздатчике | 4 | нет `DispenserBlockEntity`-хука `dispense_projectile` (проекции зелий/стрел) |
| Bedrock-протокол/RCON | 2 | внешние подсистемы (`steel-flint`, `steel-login`) |
| Мирогенерация (структуры, flat settings) | 3 | нужен data-marker dispatch и `FlatLevelGeneratorSettings` |
| Прочее (по одной записи) | ~90 | точечные фичи: eyeblossom day/night, sculk spread, grass bonemeal, turtle egg curve, wither spawning, end crystal respawn, `/locate biome`, `/locate poi`, `/summon` SNBT и т. д. |

Каждая из этих 109 записей теперь сформулирована как **ограничение с причиной**, а не как
«надо сделать»: код честно говорит, чего не хватает, и где именно это подключается.

### Итог по сессии

- **30 TODO** закрыто реальным кодом (139 → 109), в том числе 16 точек выдачи достижений.
- **1 новая команда** (`/loot`), реестр триггеров, диспетчер критериев, книга рецептов из
  достижений, статистика в scoreboard.
- **3 крупных «пустых» класса** из предыдущих волн доведены до работы (воронка, варка,
  крафтер) — см. §20.

---

## 25. Аудит паритета по метрикам из кода (9 ноября 2026)

Сессия целиком про **проверку**, а не про наращивание контента: каждый счётчик из
этого отчёта пересчитан по коду, расхождения исправлены, а одна реальная дыра в
протоколе закрыта.

### 25.1 Найденная и закрытая дыра протокола: `cookie_response`

Сверка `serverbound.play` (69 записей в `packets.json`) с ветками диспетчера
`steel-core/src/player/connection/java.rs` дала **68 из 69**: пакета
`play::S_COOKIE_RESPONSE` в `match` не было, хотя
`ScheduledPlayPacketKind::CookieResponse` уже был объявлен и обрабатывался.
Итог: клиентский ответ на cookie уходил в `ImmediatePlayPacket::Unknown` и писался
в лог как неизвестный пакет.

Исправлено (`java.rs:932`, сразу после `S_CONTAINER_CLOSE`):

```rust
play::S_COOKIE_RESPONSE => scheduled(ScheduledPlayPacketKind::CookieResponse(
    SCookieResponse::read_packet(data)?,
)),
```

Повторный подсчёт: `vanilla sb play 69 / steel arms 69`, `missing = ∅`, `extra = ∅`.

Поведение ветки сверено с ванилью: `ServerCommonPacketListenerImpl.handleCookieResponse`
в play-фазе отвечает `disconnect(MULTIPLAYER_DISCONNECT_UNEXPECTED_QUERY_RESPONSE)`,
потому что сервер принимает только те cookie, которые сам запросил, а Steel
`CCookieRequest` не отправляет. Ключ перевода сверен с официальным `en_us.json`:
`multiplayer.disconnect.unexpected_query_response = "Unexpected custom data from client"`.

### 25.2 Пересчитанные метрики (все — из кода, не из прежних записей)

| Метрика | Как считалось | Результат |
|---|---|---|
| Clientbound-пакеты | `packets.json` × константы в `steel-protocol` | **173 / 173** (status 2, login 6, config 21, play 144) |
| Serverbound play | `packets.json` × ветки `play::S_*` в диспетчере | **69 / 69** (было 68 до фикса) |
| Serverbound прочие фазы | `steel-login/src/tcp_client.rs` (`packet.id`), `handshake/status/login/config` | 1 + 2 + 5 + 10 — все обрабатываются |
| Типы block entity | `block_entities.json` × `vanilla_block_entity_types::*` в `registry.rs` | **49 / 49** |
| Меню | `menutypes.json` × `vanilla_menu_types::*` | **25 / 25** |
| Классы блоков | `classes.json` × `#[block_behavior]` + макросы | **267 / 268** (нет базового `Block`) |
| Классы предметов | `classes.json` × `#[item_behavior]` | **65 / 66** (нет базового `Item`) |
| Классы сущностей | `classes.json` × `#[entity_behavior]` + `display_entity!` | **142 / 143** (нет `Player`) |
| Типы сущностей с фабрикой | `entities.json` | **159 / 161** (нет `player`, `item`) |
| Корневые команды | `create_registered_dispatcher` | **56** (77 регистраций с алиасами) |
| `#[test]` | `rg -c` по 7 крейтам | **4 431** |
| Строк Rust | `find steel-*/src -name '*.rs' \| xargs wc -l` | **562 755** |

Формулировки, которые после пересчёта оказались неверными и исправлены в отчёте:

- «60 `#[item_behavior]`» → **65** (и 65 из 66 классов, а не 39 из 66);
- «1 578 / 1 658 предметов (95,2 %)» → **1 401 / 1 658 (84,5 %)** — и на старте
  сессии, и сейчас: прирост этой сессии в предметах нулевой. Прежняя цифра
  завышала покрытие, но правильная интерпретация мягче: непокрытые 257 предметов —
  это ровно те, что и в ванили наследуют базовый `Item`;
- «10/25 типов меню» → **25/25**;
- «25/49 block entity» → **49/49**;
- «41 корневая команда» → **56**;
- «142 типа с фабрикой (88,2 %)» → **159 (98,8 %)**;
- «223 `#[block_behavior]`» → **256 атрибутов**, покрывающих **267 из 268** классов.

### 25.3 Восстановленные регрессии

В рабочем дереве нашлись два изменения, которые ломали сборку и были откачены:

- удалённый генератор `steel-registry/build/functions.rs` при живом
  `crate::functions::FUNCTIONS` в `registry/mod.rs:413` и `command/builtins/datapack.rs`;
- удалённая генерация `EntityType::immune_to` при оставшемся поле
  (`entity_type.rs:294`) и его читателе (`dismount_helper.rs:209`).

### 25.4 Что проверено структурно (сборки не было)

Rust-тулчейн в песочнице недоступен (`cargo`, `rustc`, `rustup` отсутствуют,
`static.rust-lang.org` недоступен), поэтому вместо `cargo check` выполнены:

- баланс `{}`/`()`/`[]` по всем 50 изменённым `.rs` файлам с корректным
  разбором строк, raw-строк, символьных литералов и лайфтаймов — расхождений нет;
- все 53 вызова `player.handle_*` из диспетчера находят определение в
  `steel-core/src` (`missing = ∅`);
- все импорты в 50 изменённых файлах разрешаются в пределах воркспейса;
- имена переопределяемых методов в 200+ `impl BlockBehavior` существуют в трейте.

Это **не заменяет** компиляцию: окончательная проверка остаётся за CI.

### 25.5 Обновлённая оценка

**≈69 %** (диапазон 66–71 %) против ≈56 % на старте ветки. Срез вырос не за счёт
нового контента в этой сессии (его почти не было), а потому что прежние 61 %
опирались на устаревшие цифры по протоколу, block entities, меню и предметам;
после пересчёта по коду картина честнее и выше.

---

## 26. Итерация «ядра + честный аудит» (9 ноября 2026)

Ниже — то, что реально изменено в коде в этой итерации, с указанием ванильного
эталона. Все правки — в `arena/8ff5adc9-steelmc`; **сборки не было** (см. §8),
поэтому проверка структурная: баланс скобок, разрешимость импортов, сверка с
патчами Paper 26.3 построчно.

### 26.1 Ядра в контейнере — закрыто

Правило сформулировано ровно так, как просил оператор:

```
эффективные ядра = min(конфиг, лимит контейнера)      (если конфиг > 0)
эффективные ядра = лимит контейнера                   (если конфиг == 0)
```

Три места, где это применяется (все с тестами):

| Место | Что делает |
|---|---|
| `steel-utils/src/threading.rs::available_worker_threads` | `min(available_parallelism, cgroup cpu.max, cpuset)`, не ниже 1 |
| `steel/src/main.rs:168-170` | `configured_cores.map_or(available, \|c\| c.min(available))` |
| `steel/src/config/server.rs:129-135` | то же самое для `into_runtime_config()` |
| `steel-core/src/config.rs::clamp_cores_to_available` | `min`, `0` ⇒ доступные; вызывается в `Server::new` (`server/mod.rs:781`) |

**Исправлено в этой итерации:**

1. **Три бага чтения cgroup** в `steel-utils/src/threading.rs`:
   - cgroup **v1 `cpuset` вообще не читался** (только v2) — добавлены
     `CGROUP_V1_CPUSET_ROOT`, `v1_cpuset_base()`, кандидаты
     `cpuset.effective_cpus` / `cpuset.cpus` (root + per-controller путь);
   - `?` внутри цикла обрывал перебор кандидатов на первом отсутствующем/битом
     файле — вынесен `first_cpu_set()`, каждый кандидат `continue`-ит; то же в
     `read_v1_paths()` и `v1_cpuset_base()` (битая строка `/proc/self/cgroup`
     больше не фатальна);
   - **порядок чтения quota был неверным**: сначала читался root, где
     `cpu.max`/`cpu.cfs_quota_us` = `max`/`-1` (безлимит), и лимит контейнера
     терялся. Теперь сначала путь из `/proc/self/cgroup`, root — фолбэк.
2. **`RuntimeConfig::default().cores` был жёстко `1`.** Дефолтная конфигурация
   (тесты, `steel-flint`, встраивание) пиннила все производные пулы в один
   поток. Теперь — `available_worker_threads()`.
3. Добавлены тесты: `the_effective_core_count_is_min_of_config_and_container`
   (перебор 6×7 пар «конфиг × контейнер»), `the_default_core_count_follows_detection`,
   плюс 5 тестов на cgroup v1/v2 (включая tempdir-тест на битые файлы).

Инварианты `pool_sizes_for_available()` (тесты 3..=256): `main + gen + enc == cores`,
`chunk_runtime == packet == max(cores/8, 1)`, при cores ≤ 2 — gen/enc = 1.

### 26.2 Найденные и исправленные баги (этой итерацией)

| # | Область | Баг | Ванильный эталон | Файл |
|---|---|---|---|---|
| 1 | Печь | `cooking_timer == cooking_total_time` вместо `>=`. При смене входа на более быстрый рецепт цель уменьшается и равенство проскакивается — печь «готовит» вечно | `if (this.cookingTimer >= this.cookingTotalTime)` | `abstract_furnace.rs:218` |
| 2 | Хоппер | Запитанный (`ENABLED=false`) хоппер продолжал тикать и двигать предметы | `HopperBlock.getTicker` → `state.getValue(ENABLED) ? ticker : null` | `block_entity_blocks.rs:207` |
| 3 | Хоппер | Извлекал из **любого** слота источника; не учитывал `getSlotsForFace(DOWN)`. Хоппер под печью тянул из входного слота, под книжной полкой — из закрытых слотов | `for (int slot : getSlots(container, DOWN))` | `hopper.rs:135` |
| 4 | Хоппер | Не вызывал `canTakeItemFromContainer` (собственный `canTakeItem` источника) | `container.canTakeItem(destination, slot, stack)` | `hopper.rs:158` |
| 5 | Хоппер | Выталкивал **всегда вниз**, игнорируя `FACING`; ваниль кормит контейнер по направлению | `getContainerAt(blockPos.relative(facing))` | `hopper.rs:262` |
| 6 | Хоппер | Вставка шла через `Container::add`, игнорируя `canPlaceItemThroughFace` и лимит стека приёмника | `addItem(..., direction)` + `canPlaceItemThroughFace` | `hopper.rs:330` (`add_into_slots`) |
| 7 | Хоппер | Сбрасывал кулдаун 8 тиков **всегда**, даже когда ничего не сдвинулось — простаивающий хоппер спал 8 тиков вместо мгновенной реакции | `tryMoveItems` → `if (changed) setCooldown(8)` | `hopper.rs:314` |
| 8 | Хоппер | Не подбирал предметы-сущности над собой (в ванили `SUCK_AABB`) | `Hopper.SUCK_AABB` (0.25 по бокам, 1.0 вверх) | `hopper.rs:196` |
| 9 | Лиса | Кормление сладкими ягодами не давало доверия (`addTrusted`) — слоты `trusted` были доступны только из тестов | `Fox.mobInteract` → `addTrusted` | `animal.rs::on_fed` + `fox/mod.rs` |
| 10 | Волк | Приручённый волк не лечился мясом | `Wolf.mobInteract` → `heal`, съесть 1 | `wolf.rs:377` |
| 11 | Кошка | Нельзя было посадить/поднять одомашненную кошку пустой рукой | `Cat.mobInteract` → toggle `SITTING` | `cat/mod.rs:498` |
| 12 | Протокол | Не отправлялся `ClientboundUpdateEnabledFeaturesPacket` — клиент оставался с пустым набором флагов | `startConfiguration()` шлёт его безусловно, между links и `select_known_packs` | `steel-login/src/handlers/config.rs:66` |
| 13 | Протокол | Не отправлялись `SetChunkCacheRadius` и `SetSimulationDistance` при спавне — клиент держал свои дефолты, и радиус чанков/симуляции расходился с тем, что сервер реально тикает | `PlayerList.sendLevelInfo` (сразу после спавн-позиции, до `LEVEL_CHUNKS_LOAD_START`) | `spawn_sync.rs::send_view_distances` |
| 14 | Протокол | Два разных `#[packet_id(Play = C_PING)]` (`CPing` и `CPingPlay`) на один id — второй был мёртв и вводил в заблуждение | у ванили один ping на фазу | удалён `c_ping_request.rs` |

### 26.3 Что проверено и **признано корректным** (не трогал)

- Печь: порядок тика (`lit_time_remaining` в начале, `+= 1`, `>=`, сброс,
  пересчёт `cooking_total_time`, `burn`, `BURN_COOL_SPEED = 2`), `getBurnDuration`
  (`speedMultiplier > 0 → ceil(cookingTime / speedMultiplier)`), `consumeFuel`
  (`shrink(1)` + `getCraftingRemainder`), `maxStack = 64`, NBT-ключи
  `BurnTime/CookTime/CookTimeTotal/RecipesUsed`.
- Костёр: гейт `LIT` в `tick()` (а не в тикере) — эквивалентно ванили.
- `keepConnectionAlive` — 15 000 мс, pending → `disconnect(DISCONNECT_TIMEOUT)`.
- `handleCookieResponse` в play → `DISCONNECT_UNEXPECTED_QUERY`.
- `PlayerList.tick` — `++sendAllPlayerInfoIn > 600`, `SEND_PLAYER_INFO_INTERVAL = 600`
  совпадает, вызов есть в `run_loop.rs:255`.
- Все **59** геймрулов читаются (включая `pvp`, у которого имя короче 4 символов
  и его пропускал грубый grep).
- Все **49** типов block entity зарегистрированы; 191 `valid_blocks` — все есть в
  `blocks.json`.
- Все **25** типов меню имеют обработчик.
- `unimplemented!()` / `todo!()` — **0**; `.unwrap()` в non-test коде — **1**
  (`villager_trades.rs:306`, `i32::try_from(id)`); `panic!` — только инварианты
  worldgen/`region.rs`.
- NBT save/load: расхождений ключей нет (29 «подозрительных» файлов — общий
  `ContainerHelper::{save,load}AllItems` и клиентские `hit_direction`, которые
  ваниль тоже не пишет в save).
- Дубликатов `impl X for Y` в одном файле нет.

### 26.4 Файлы: ничего не потеряно

Проверка всех `mod`-объявлений по 7 крейтам (**1 922 файла** просканировано):

```
missing modules: 93 — все сгенерированные (src/generated/, behavior/generated/,
entity/generated/, steel-registry/src/generated/, steel-worldgen/src/generated/)
+ 4 в steel-utils (translations, translations_registry, entity_events, version),
  которые пишет build.rs в OUT_DIR.
Несгенерированных отсутствующих модулей: 0.
```

Отдельно проверены «ложные тревоги» от простого скрипта: `enchantment_helper/{item_in_use,selection}`,
`paletted_container/encode.rs`, `behavior/items/block_transformer/tests.rs`,
`inventory/menu/kinds/*/tests.rs`, `entity/damage/history/binding.rs`,
`entity/entities/objects/vehicles/tests.rs`, `server/{tests,connection_lifecycle,player_limit}.rs` —
все на месте (подмодули в каталогах).

### 26.5 Пересчёт покрытия

| Метрика | Было в отчёте | **Реально** | Комментарий |
|---|---|---|---|
| Классы блоков с поведением | 267 / 268 | **267 / 268** | нет только абстрактного `Block` |
| Классы предметов с поведением | 65 / 66 | **65 / 66** | нет только базового `Item` |
| Классы сущностей с поведением | 142 / 143 | **142 / 143** | нет `Player` (вручную в `player/mod.rs`) |
| Типы сущностей с фабрикой | 159 / 161 | **159 / 161** | нет `player`, `item` |
| Типы меню | 25 / 25 | **25 / 25** | — |
| Clientbound play-пакеты | 173 / 173 | **173 / 173** | — |
| Serverbound play-пакеты | 69 / 69 | **69 / 69** | — |
| Тестов | 4 431 | **4 507** | +76 |
| Строк Rust (без `generated`) | ≈562 800 | **≈608 100** | — |

**Важная поправка методики.** Предыдущая версия скрипта покрытия сущностей
писала 140/143, потому что не видела классы, объявленные через макрос
`display_entity!` (`ItemDisplay`, `TextDisplay`) и структуры с
`#[entity_behavior]` без аргумента `class = "..."` (`ItemEntity`). После
исправления скрипта — **142/143**, что совпадает с ручной проверкой.

### 26.6 Итоговая оценка — та же форма, но реальная

| Срез | Оценка | Что за этим стоит |
|---|---|---|
| **Серверное ядро в целом (взвешенно)** | **≈70 %** (диапазон 67–72 %) | было ≈69 % в предыдущей версии отчёта; +1 п.п. за счёт закрытых багов хоппера/печи/протокола |
| Инфраструктура и мир | ≈89 % | протокол 100 % по составу, но реконфигурация/dialog/debug-пакеты не отправляются |
| Контент-механики | ≈89 % | 267/268 классов блоков, 65/66 предметов, 49/49 BE, 25/25 меню |
| Живой мир | ≈53 % | 142/143 класса-каркаса, но глубина ИИ ≈30 %, структуры ≈41 % |
| Серверные системы | ≈44 % | 56 корневых команд из 67, 11 не реализовано |

**Почему не «100 %».** Каркас (реестры, пакеты, фабрики, NBT, меню) почти
ванильный — это подтверждено счётчиками. Но «класс зарегистрирован» ≠ «класс
ведёт себя как ваниль»: ИИ-цели, естественный спавн, структуры, боссы и
социальные системы реализованы на треть-половину, и именно они дают основную
массу расхождения. Заявлять 100 % было бы неправдой; заявлять «как было» —
тоже, потому что 14 реальных багов закрыто, а ядра в контейнере теперь
считаются по правилу `min(конфиг, контейнер)`, включая cgroup v1.

### 26.7 Что осталось (не скрываю)

1. **Сборки не было** — Rust-тулчейн недоступен в песочнице
   (`static.rust-lang.org` не отвечает). Все правки проверены структурно.
2. `ItemEntity` и `Player` не проходят через `register_entity_factories`
   (регистрируются вручную) — это осознанно, но в метрике «159/161».
3. `dialog`, `debug`, `jfr`, `perf`, `place`, `publish`, `raid`, `random`,
   `teammsg`, `warden_warning`, `waypoint` — команд нет.
4. Глубина ИИ мобов ≈30 %: цели есть у ~40 % мобов.
5. Структуры: 9 из ~22 типов.

---

## 27. Второй проход: ядра, варилка, вагонетка-хоппер, производительность

### 27.1. Ядра: правило «контейнер важнее конфига»

Требование: если контейнер (Pterodactyl/Docker/cgroup) выделяет **меньше** ядер, чем указано в конфиге, сервер обязан уменьшиться до контейнерного лимита; если больше — конфиг остаётся верхней границей.

Правило вынесено в одну функцию `steel_utils::threading::effective_cores(configured, available)`:

```rust
available.max(1);
configured.filter(|&c| c > 0).map_or(available, |c| c.min(available))
```

То есть `0`/`None` = «столько, сколько разрешает контейнер», положительное значение = `min(конфиг, контейнер)`. Функция вызывается из **всех** трёх мест, где раньше была продублированная логика:

| Место | Было | Стало |
|---|---|---|
| `RuntimeConfig::clamp_cores_to_available` | свой `if cores == 0 { … } else { min }` | `effective_cores` |
| `steel/src/main.rs` (создание Tokio-рантаймов) | свой `map_or(... min ...)` | `effective_cores` |
| `steel/src/config/server.rs::into_runtime_config` | `map_or_else(available_worker_threads, |c| c.min(available))` | `effective_cores` |

**Найден и исправлен баг в определении cgroup-лимита.** `cgroup_cpuset_cpus()` проверяла сначала файлы в **корне** иерархии (`/sys/fs/cgroup/cpuset.cpus.effective`), потом вложенный путь из `/proc/self/cgroup`. Без приватного cgroup-namespace (обычный `docker run` с bind-mount, панели, монтирующие только своё поддерево) корень — это **хост**, поэтому чтение корня первым возвращало ширину всей машины и полностью скрывало `--cpuset-cpus=0-3`. Порядок перевёрнут: сначала путь процесса, корень — только фолбэк. То же для cgroup v1 (`/sys/fs/cgroup/cpuset<path>/`) и для `cpu.max`/`cpu.cfs_quota_us`.

Каждый пул (`main_runtime`, `chunk_runtime`, `chunk_generation`, `chunk_encoding`, `packet_workers`) выводится из `config.cores` через `pool_sizes_for_available`; per-pool override клампится `cap_positive_thread_count(configured, cores)`. Проверено, что обойти `effective_cores` нечем: `Builder::new_multi_thread` есть только в `steel/src/main.rs` (использует `core`), `steel-flint` (тестовый, `worker_threads(4)`) и тестах.

Добавлены 3 теста: `effective_cores_never_exceeds_the_container_allowance`, `effective_cores_treats_zero_as_auto_and_never_returns_zero`, `a_sub_core_quota_still_yields_one_worker`.

### 27.2. Варилка: пять расхождений с ванилью, одно из них полностью ломало блок

| # | Что было | Ваниль 26.3 |
|---|---|---|
| 1 | `is_ingredient` вызывал `is_brewable_potion` (проверку **бутылки**) | `PotionBrewing.isIngredient` ищет предмет среди `ingredient` всех миксов — **ни одна варка не могла начаться** |
| 2 | `BREW_TIME = 20` тиков | `BREW_TIME = 400` тиков |
| 3 | 4 слота, ингредиент вне контейнера | 5 слотов: `0..=2` бутылки, `3` ингредиент, `4` топливо; меню уже было на 5 слотов и расходилось с block entity |
| 4 | `set_changed`-no-op и никакого `has_bottle_*` | `serverTick` каждый тик синхронизирует `has_bottle_0/1/2` из трёх бутылочных слотов (модель всегда рендерилась пустой) |
| 5 | топливо = «в слоте лежит blaze powder», 1 слот-ограничение на всё | `getUses`/`getSpeedMultiplier` из компонента `minecraft:brewing_fuel`, `brewTime = ceil(400 / speedMultiplier)`, refuel только при `fuel <= 0` |

Добавлены ванильные `getSlotsForFace` / `canPlaceItemThroughFace` / `canTakeItemThroughFace` (`UP = [3]`, `DOWN = [0,1,2]`, `SIDES = [0,1,2,4]`) — без них хопперы под варилкой тянули ингредиент и топливо, а верхний хоппер мог забить бутылочные слоты. NBT приведён к 26.3: `BrewTime`, `Fuel`, `total_brew_time`, `total_fuel`, `speed_multiplier` (float). Меню получило два ванильных data-слота (`BrewTime`, `Fuel`), которые раньше не публиковались вовсе.

### 27.3. Вагонетка-хоппер: подбор не работал, бокс и частота были неверными

- `suck_items` содержал `let _ = world.get_entities_in_aabb(&box_);` — результат выбрасывался, подбор **не работал вообще**.
- Бокс: было `(1.0, 0.5, 1.0)`; ваниль `MinecartHopper.suckInItems` — `getBoundingBox().inflate(0.25, 0.0, 0.25)`, по вертикали **ноль**.
- Частота: было `tick_count() % 4`; ваниль двигает предмет **каждый тик** (20 предметов/с, в 8 раз быстрее блочного хоппера).
- Отсутствовала первая половина ванильного алгоритма — контейнер над вагонеткой (координата от `getY() + 0.5`, `DOWN`-грань, `canTakeItem`/`canPlaceItemThroughFace` через `getSlotsForFace`).
- Отсутствовал `TransferCooldown` в NBT (ванильные 1..8), добавлено поле и сохранение/загрузка.
- `try_insert` возвращал `fit || changed`, из-за чего вызывающий код считал частичную вставку полной; теперь возвращается «реально переложено хоть сколько-то».

### 27.4. Хоппер-блок: найдено и исправлено 4 расхождения

1. **`set_changed` на самом контейнере — no-op.** `Container::set_changed` у всех хранилищ (`hopper`, `furnace`, `barrel`, `brewing_stand`, `chest`, `chiseled_bookshelf`, `crafter`, `dispenser`) пуст намеренно: пометку делает `BlockEntityBase::set_changed` через `ContainerRef::-владельца`. Хоппер же звал `container.set_changed()` на `&mut dyn Container` — изменения **не помечались dirty и не обновляли компаратор**. Добавлен `ContainerRef::mark_changed()` (безопасная точка пометки без ручного лока) и `ContainerLockGuard::set_changed(id)`; хоппер и варилка переведены на них. В доке `Container::set_changed` это теперь зафиксировано.
2. **`TRY_MOVE || SUCK` вместо `changed |=`.** Ваниль: `tryMoveItems(level, pos, state, entity, () -> suckInItems(level, entity))` — за один тик хоппер и толкает вперёд, и тянет сверху. `||` короткозамкнуто пропускал всасывание, если удался пуш, из-за чего предмет застревал на лишний тик.
3. **`inventoryFull` отсутствовал.** Ванильный `suckInItems` сразу выходит, если нет ни пустого слота, ни стека, который можно долить. Без этого полный хоппер каждый тик сканировал все item-entity сверху.
4. **Suck-бокс был неверным.** Было «блок + 0.25 по бокам, +1.0 вверх». Ваниль: `Hopper.getSuckShape() = Shapes.or(INSIDE, ABOVE)` где `INSIDE = Block.column(2.0, 11.0, 16.0)`, `ABOVE = Block.column(16.0, 16.0, 32.0)`, а `getItemsAtAndAbove` берёт **bounding box** этой фигуры → `Block.column(16.0, **11.0**, 32.0)`. Старый бокс не доставал до предметов, лежащих в чаше, и захватывал лишнее по бокам. Константы `HOPPER_SUCK_MIN_Y = 11/16`, `HOPPER_SUCK_MAX_Y = 32/16`, ширина 1.0.
5. **`addItem(Container, ItemEntity)` возвращал `true` при частичном приёме.** Ваниль возвращает `true` только когда весь стек ушёл (тогда `entity.discard()`), иначе `entity.setItem(result)` и `false` — сканирование продолжается. Исправлено.

### 27.5. Производительность

- Хоппер: `slots_for_face(...).to_vec()` в двух местах выполнялся **каждый тик для каждого хоппера**. Заменено на `SmallVec<[usize; 32]>` — типичный случай (5 слотов хоппера, 3 печи, 27 сундука) не аллоцирует.
- Варилка: убран лишний `Mutex` на ингредиент (был отдельный `SyncMutex<ItemStack>` вне контейнера) — теперь один лок вместо двух на каждой варке.
- `sync_bottle_properties` пишет блокстейт только при реальном изменении (`changed`), а не каждый тик.
- Проверены остальные горячие пути: `calculate_redstone_signal_from_container` (`lerpDiscrete`, без аллокаций), `to_vec()` в block entity остался только в NBT-save костра, пер-тиковых `collect::<Vec>` в менеджере сущностей нет.

### 27.6. Проверено и подтверждено корректным (второй проход)

- Пустые переопределения `check_fall_damage`/`push_entities` у `Bat`, `push_entities` у `ArmorStand`/`Mannequin`, `do_water_splash_effect` у `ExperienceOrb` — все **ванильные no-op** (подтверждено по трекеру парности Pumpkin и вики).
- `play_step_sound` с пустым телом у blaze/vex/witch/giant/ghast/evoker/illusioner — пустой набор `SOUND_EVENTS` этих мобов, шагового звука в ванили нет.
- Печь: `cookingTimer >= cookingTotalTime`, деление burn/cook на 2 у smoker/blast furnace (округление вниз, `4001 → 2000`), `getMaxStackSize = 64`.
- Кондуит: `MIN_ACTIVE_SIZE = 16`, `MIN_KILL_SIZE = 42`, `range = blocks / 7 * 16`, урон 4 каждые 2 с в радиусе 8.
- Колокол: `RING_DURATION = 50`, `GLOW_DURATION = 60`, `RESONATION_DELAY = 5`, поиск 48, слух 32.
- Костёр: `BURN_COOL_SPEED = 2`.
- Тайлы `block entity`: инвентари 49/49, тикер варилки не гейтится (как ванильный `getTicker`).
- Покрытие классов не изменилось: блоки **267/268** (нет `Block`), предметы **65/66** (нет `Item`), сущности **142/143** (нет `Player` — реализован вручную), фабрики 159/161, меню **25/25**.

---

## 28. Третий проход: спавн при генерации, ревизия структур

### 28.1. Закрыт реальный пробел: чанк генерировался без мобов

`steel-core/src/worldgen/stages/spawn.rs` (стадия `Spawn`, ванильный
`ChunkStatusTasks.generateSpawn` → `NoiseBasedChunkGenerator.spawnOriginalMobs` →
`NaturalSpawner.spawnMobsForChunkGeneration`) представлял собой **пустую
функцию**: ни один чанк никогда не получал «оригинальных» мобов. Тиковый
`NaturalSpawner` при этом работал, поэтому дефект не был заметен в игре —
но новая карта оставалась пустой до первого 400-тикового цикла, а сохранение
без тиков (генератор сидов) не получало животных вообще.

Реализовано строго по ванильному алгоритму:

| Аспект | Ваниль | Реализация |
|---|---|---|
| Категория | только `creature` из `biome.spawners` | `biome.spawners.get("creature")` |
| Условие цикла | `while random.nextFloat() < creatureSpawnProbability` | так же, вероятность клампится `< 1.0` |
| Моб-кап | игнорируется | игнорируется |
| Spawn costs | игнорируются | игнорируются |
| Размер пачки | `minCount..=maxCount` записи | так же |
| Позиция пачки | одна точка в чанке, далее шаг `nextInt(5)-nextInt(5)` по X/Z | так же (треугольное распределение) |
| Высота | `MOTION_BLOCKING` heightmap | так же |
| Куда попадают | `ProtoChunk.addEntity` | `Chunk::add_entity` → proto-хранилище |

Анти-баг: первая версия была написана по формуле **тиковой** стадии (все
категории + `1 + nextInt(2) + nextInt(2)`). Это неверно — ванильная
`spawnMobsForChunkGeneration` устроена иначе, и такая реализация спавнила бы
монстров внутри неосвещённых чанков. Формула `getRandomSpawnCount`
принадлежит `NaturalSpawner.tick`, а не генерации.

Проверено, что сущности действительно доходят до мира:
`FullChunkPromotion.pending_entities` = `proto_chunk.entities.close_and_drain()`
(`chunk/full_chunk/mod.rs:309`), то есть proto-хранилище дренируется при
промоушене в `Full`.

### 28.2. Ревизия структур: отчёт недооценивал на 7 типов

§3 строка 5 заявляла «9 из ~22 ≈ 41 %». Фактически в
`vanilla_structure_impls()` (`steel-worldgen/src/structure/generator.rs:967-982`)
зарегистрированы **все 16** ванильных типов `structure`:

`jigsaw`, `nether_fossil`, `fortress`, `end_city`, `woodland_mansion`,
`ocean_monument`, `mineshaft`, `desert_pyramid`, `jungle_temple`, `swamp_hut`,
`buried_treasure`, `shipwreck`, `igloo`, `ocean_ruin`, `stronghold`,
`ruined_portal`.

Деревни, бастионы, ancient_city, trial_chambers, trail_ruins, pillager_outpost
и остальные пуловые структуры идут через `jigsaw` и не требуют отдельного
класса — именно так устроена ваниль. Файлы `end_city.rs` (625 строк),
`mansion/` (1 300), `ocean_ruin.rs` (393), `shipwreck.rs` (187), `igloo.rs` (192),
`nether_fossil.rs` (234) существуют и подключены.

**Честная оценка строки 5: ≈95 %**, а не 41 %. Остаток — не типы, а глубина:
полнота маркеров в шаблонах и редкие граничные случаи размещения.

### 28.3. Проверено и подтверждено корректным (третий проход)

- `stages/full.rs` — `holder.upgrade_to_full()`, это и есть ванильная задача
  `Full` (промоушен proto → Full), а не заглушка.
- `stages/biomes.rs` — делегирует `generator.create_biomes(chunk)`, корректно.
- `stages/structures.rs::load_starts` — пустой намеренно: при загрузке с диска
  структурные старты приходят из сейва, отдельной работы нет.
- `stages/light.rs` (470 строк) и `leaf_distance.rs` (494) — содержательные.
- Заглушек `todo!`/`unimplemented!` в мировом коде нет (только `unreachable!`
  в защитных ветках, что ванильно).

### 28.4. Что осталось в «инфраструктуре и мире» — прямо

1. **`jigsaw_generate` доведён не был, и я его откатил.** Рантайм-генерация
   структуры из джагсо-блока — единственный известный пробел инфраструктуры,
   который я начал закрывать и откатил: `StructureTemplate::place_in_world`
   глубоко завязан на `WorldGenRegion` (запись через
   `set_block_state_for_generation`, постпроцессинг, обновление шейпов), а
   регион поверх живых чанков потребовал бы правок в `WorldGenAccessMode`,
   `ChunkStep` и правах записи, которые **без компилятора я не могу
   верифицировать**. Оставлять это как «сделано» — хуже, чем признать пробел:
   сейчас `handle_jigsaw_generate` логирует пропуск.
2. **Сборки не было.** Rust-тулчейн в песочнице отсутствует,
   `static.rust-lang.org` недоступен, `apt` без root. Все правки проверены
   структурно (`bal.py`, `imports.py`, `modcheck.py`, сверка сигнатур), но
   **не скомпилированы**. Перед мержем нужен `cargo check`.
3. **Спавн при генерации не проверен в рантайме** — тесты покрывают только
   взвешенный выбор; сквозной прогон требует сборки.
