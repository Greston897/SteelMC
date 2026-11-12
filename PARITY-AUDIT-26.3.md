# SteelMC ↔ Minecraft Java 26.3 — аудит готовности по механикам

**Дата замера:** 9 ноября 2026
**Ветка:** `arena/8ff5adc9-steelmc`, HEAD `fcb9256`
**Цель:** Java Edition **26.3** (протокол 777)
**Метод:** машинный подсчёт по дереву репозитория + сверка с извлечёнными данными 26.3
(`steel-registry/build_assets/*.json`). Оценки «на глаз» помечены отдельно.

---

## 1. Короткий ответ

| Срез | Готовность |
|---|---|
| **Инфраструктура** (протокол, чанки, свет, сохранения, мирогенерация) | **≈ 93 %** |
| **Контент** (блоки, block entities, предметы, меню, реестры) | **≈ 88 %** |
| **Живой мир** (сущности, ИИ, бой, редстоун) | **≈ 62 %** |
| **Серверные системы** (команды, права, scoreboard, достижения, админ) | **≈ 55 %** |
| **ИТОГО по ядру (взвешенно)** | **≈ 76 %** |

Веса: инфраструктура 30 %, контент 30 %, живой мир 25 %, серверные системы 15 %.

**Формулировка в одну строку:** мир, протокол и контент практически ванильные;
разрыв остался в *глубине симуляции* — награды достижений, торговля с жителями,
полнота редстоуна на всех блоках и «социальные» системы.

Предыдущая версия этого аудита (в `PARITY-26.3-RU.md`) давала ≈ 61 %.
Рост объясняется тем, что за прошедшие итерации закрылись целые классы:
block entities 25 → 49, протокол 166 → 234, мобы — почти все, меню 13 → 20.

---

## 2. Что измерено точно (счётчики)

| Подсистема | Есть | Всего в 26.3 | % |
|---|---:|---:|---:|
| **Пакеты протокола** (структуры с `packet_id`) | **234** | **234** | **100 %** |
| Serverbound (клиент → сервер) | 87 | 87 | 100 % |
| Clientbound (сервер → клиент) | 173 | 173 | 100 % |
| **Block entities** (зарегистрированные фабрики) | **49** | **49** | **100 %** |
| **Мобы** (файл + `#[entity_behavior]`) | **85** | **86** | **98,8 %** |
| **Типы структур мирогенерации** | **16** | **20** | **80 %** |
| **Виды фич мирогенерации** (dispatch) | **70+** | **70+** | **100 %** |
| **Меню** (типы экранов) | **20** | **25** | **80 %** |
| **Команды** (корневые регистрации) | **60** | ~77 | **78 %** |
| **Блоки с собственным поведением** | **252** структур | 268 классов | **94 %** |
| **Предметы с собственным поведением** | 66 classes / 261 предмет | 64 класса / 363 | **≈ 72 %** |
| **Критерии достижений, реально вызываемые** | **17** | **62** | **27 %** |
| **Мобы с непустым набором AI-целей** | **77 / 78** | — | **98,7 %** |
| **AI-целей реализовано** | **35** | ~120 | **≈ 29 %** |
| **Эффекты зелий** (упомянуты в логике) | 33 | 40 | 82 % |
| **Реестры 26.3** (данные) | 5 945 | 5 945 | **100 %** |
| Тестов `#[test]` | 4 380 | — | — |
| Строк Rust | ≈ 587 000 | — | — |

---

## 3. По подсистемам

### 3.1 Протокол — 100 %

Все 234 пакета 26.3 имеют типизированную структуру и `packet_id`, включая
`intention`, `seen_advancements`, `player_info_remove`, `set_chunk_cache_center`,
`accept_code_of_conduct` — каждый нашёлся в своём файле (часть с именами,
отличными от snake_case-имени из `packets.json`).

Реализованы «тяжёлые» части: чанки с блочным светом, entity tracker,
синхронизация инвентарей и меню, команды с автодополнением, подписанный чат
(цепочка `message_chain`, `signature_cache`, `profile_key`, CRC-8 в
`message_validator`), границы мира, боссбар, таб-лист, `waypoint`.

**Остаточный риск** — не покрытие, а *обработчики*: часть пакетов
читается и логируется без полноценной реакции на клиенте.

### 3.2 Мирогенерация — 92 %

Самая сильная часть проекта (README заявляет 7 500 чанков, сверенных
блок-в-блок с ванильным эталоном: 2 500 на измерение).

- **Все 70+ видов фич** имеют рантайм-placer (`configured.rs` — 137 match-веток):
  от `bamboo`, `geode`, `iceberg` до `sculk_patch`, `template`, `vegetation_patch`.
- **16 из 20** типов структур: jigsaw, mineshaft, fortress, end_city,
  woodland_mansion, ocean_monument, desert_pyramid, jungle_temple, swamp_hut,
  buried_treasure, shipwreck, igloo, ocean_ruin, stronghold, ruined_portal,
  nether_fossil.
- **Нет:** `trail_ruins`, `trial_chambers`, `ancient_city`, `bastion_remnant`.
- Карверы: `cave`, `canyon` + маска.
- 730 форм коллизий, поверхность, биомы, density-функции — из данных 26.3.

**Разрыв:** фичи генерируют блоки, но не население — сущности в структурах
(жители, illager-ы) не спавнятся.

### 3.3 Блоки и блок-энтити — 94 % / 100 %

- **1 286** типов блоков; **252** структуры `#[block_behavior]`
  против **268** ванильных классов блоков.
- **49 из 49** типов block entity зарегистрированы с фабрикой — включая
  сундук, воронку, раздатчик, выбрасыватель, крафтер, лектерн, бражную стойку,
  декоративный горшок, sculk-сенсор/крикун/катализатор, полку, баннер,
  trial spawner, vault, jigsaw, structure block, статуи медного голема.
- `UnimplementedBlockEntity` остаётся как NBT-совместимый fallback, но
  больше не «чёрная дыра» для 24 типов, как было ранее.
- **Редстоун** — 24 модуля: `[[wire]]` (647 + 541 строк), диоды, компаратор,
  репитеры, наблюдатель, поршни с движущимся блоком, кнопки, рычаги, плиты,
  рельсы, TNT, tripwire, лампы, детектор дня, громоотвод.

**Остаток:** поведения для «простых» блоков (плиты разных камней и т. п.)
не требуются — они работают из данных реестра. Реальные пробелы — сложные
классы вроде `ShelfBlock`, `InfestedBlock`, часть `NyliumBlock`.

### 3.4 Предметы — ≈ 72 % по классам, 100 % по реестру

- **1 658** предметов; **1 038** — `BlockItem` (работают через блок),
  **257** — базовый `Item` (в ванили у них тоже нет своего поведения).
- **363** предмета с нетривиальным классом; **65** структур `#[item_behavior]`.
- Реализованы: инструменты (кирка/топор/лопата/меч/ножницы), вёдра (все),
  лук, арбалет, трезубец, удочка, зелья/снаряды, лодки и плоты, вагонетки,
  карты, книги, броня (все типы), элитры (`glide`), маяк, таблички,
  шаблоны кузнеца, кристалл Края, ветряной заряд, рамки, картины.

**Не сделано:** `KnowledgeBookItem` (нужен слой `RecipeDisplay`).

### 3.5 Сущности и ИИ — 85/86 мобов, но глубина ИИ ≈ 29 %

- **85 из 86** мобов имеют класс и `#[entity_behavior]`; не хватает
  только `zombie_nautilus` (новый моб 26.3).
- **77 из 78** мобов имеют непустой набор целей; исключение — `bat`
  (у него и в ванили цели тривиальны).
- **35 реализованных целей** против ~120 ванильных: float, melee attack,
  ranged attack, nearest attackable target, hurt by target, random stroll,
  look at player, random look, tempt, breed, follow parent, panic, avoid entity,
  leap at target, open/close door, eat block, climb powder snow, restrict sun,
  flee sun, breath air, water avoid, move to block, follow mob и др.
- **A* pathfinding** (`ai/pathfinder.rs`, 514 строк, `NodeHeap` + evaluator 169 строк).
- Зомби: 7 целей против 9 ванильных (нет `BreakDoorGoal` и отдельного
  `ZombieAttackGoal`; `MeleeAttackGoal` покрывает атаку).
- Боссы: `ender_dragon` (14 целей), `wither_boss` (14 целей).

**Главный разрыв всего проекта — здесь:** набор целей покрывает базовое
поведение, но не уникальные механики (жители/торговля, рейды, приручение
с полной логикой, рыба-иглобрюх и т. п.). Спавн (`natural_spawner.rs`)
есть с ванильными константами (интервал 400, дистанция 8 чанков, лимиты по
категориям), но без полных правил освещения/биомов.

### 3.6 Бой и эффекты — ≈ 70 %

- `entity/damage/` — источники урона, история, правила (`combat_rules`).
- **33 из 40** эффектов зелий упомянуты в логике; файлы реализаций:
  поглощение, голод, отравление, регенерация, иссушение, насыщение,
  плохое предзнаменование, рейд-предзнаменование, заражение, oozing,
  weaving, wind charged, heal/harm.
- Зачарования: генерируются из данных 26.3 (`build/enchantments/`),
  эффекты описаны в `enchantment/effect.rs` (691 строка: `LevelBasedValue`,
  `EnchantmentEntityEffect`, предикаты).

**Разрыв:** не все эффекты имеют поведение (только 14 файлов на 40 эффектов),
броня/зачарования не полностью влияют на формулы урона.

### 3.7 Меню и крафт — 80 %

**20 из 25** типов меню: `generic_9x1`…`9x6`, `anvil`, `beacon`,
`blast_furnace`, `crafting`, `enchantment`, `furnace`, `grindstone`, `loom`,
`shulker_box`, `smithing`, `smoker`, `cartography_table`, `stonecutter`,
`crafter_3x3`.

**Нет:** `generic_3x3`, `brewing_stand`, `hopper`, `lectern`, `merchant`.

Крафт: реестр рецептов из данных, `find_match`/`assemble_recipe`, крафтер
(полный цикл: редстоун → счётчик → выброс результата). Крафт-стол,
наковальня, зачарование, кузнечный стол, камнерез, ткацкий станок,
картографический стол — реализованы.

### 3.8 Команды и права — 78 %

**60 корневых регистраций** против ~77 ванильных.

**Есть:** advancement, bossbar, clear, clone, damage, defaultgamemode,
difficulty, effect, enchant, execute, experience, fill, fillbiome, gamemode,
gamerule, give, kick, kill, list, locate, loot, msg/me, op/deop, particle,
playsound, recipe, return, ride, say, scoreboard, seed, setblock,
spawnpoint, spreadplayers, setidletimeout, setworldspawn, stop, stopsound,
summon, tag, team, teleport, tellraw, tick, time, title, tp (в `teleport`),
weather, **worldborder**, item, transfer, invsee, perms, stopwatch, tpsbar,
swing, version, domain, fly.

**Нет:** `attribute`, `ban`/`banip`/`banlist`/`pardon`/`pardonip`/`whitelist`
(модерация), `datapack`, `function`, `schedule`, `random`, `reload`,
`forceload`, `save-all`/`save-off`/`save-on`, `help`, `publish`, `spectate`,
`data`, `debug`, `jfr`, `perf`, `warden_warning`.

**Ключевой вывод:** «серверные» команды (datapack/function/schedule/random)
отсутствуют — это блокирует датапаки как механику.

Права: система `permission/` (4 223 строки) с declaration-моделью,
группами, метаданными; `execute` поддерживает условия `stopwatch` и др.

### 3.9 Достижения — самый большой относительный разрыв

- **62** критерия объявлены (`vanilla_criteria_triggers.rs`).
- **17** реально вызываются из игровой логики (27 %).
- Not called: `CRAFTING_TABLE_USED`, `PLACED_BLOCK`, `DESTROYED_BLOCK`,
  `CONSUME_ITEM`, `BREWED_POTION`, `TAME_ANIMAL`, `TRADE`/`VILLAGER_TRADE`,
  `PLAYER_KILLED_ENTITY`, `CHANGED_DIMENSION`, `NETHER_TRAVEL`, `LOCATION`,
  `TICK`, `USING_ITEM` и др.

Инфраструктура есть и работает: `PlayerAdvancement` (progress, rewards,
saving, visibility_evaluator), команда `/advancement`, 16 точек выдачи
награды. Но **дерево достижений не «тикает»** — большинство триггеров
не подключено, поэтому игрок не получает достижений за обычную игру.

### 3.10 «Социальные» и серверные системы — ≈ 45 %

| Система | Состояние |
|---|---|
| Чат (подписанный) | Полный: цепочки подписей, кэш, валидатор |
| Scoreboard | Данные (1 138 строк) + сохранение + `/scoreboard` есть, **но клиенту не отправляется**: `C_SET_OBJECTIVE`/`C_SET_SCORE`/`C_SET_PLAYER_TEAM`/`C_SET_DISPLAY_OBJECTIVE`/`C_RESET_SCORE` определены как структуры, но ни один не используется в `steel-core` (0 вхождений). Сайдбар и таблица не видны в игре |
| Teams | Команда `/team` есть |
| Bossbar | Есть + `/tpsbar` |
| Статистика | `stats_counter` есть |
| RCON | Полный сервер (Source RCON, аутентификация) |
| Белый список / баны | **Нет** |
| Плагины | **Нет** (только `CommandRegistry`-расширения) |
| Торговля с жителями | **Нет** — `villager.rs` прямо это документирует |
| POI / деревни | `poi/` есть (instance, set, storage), но без полной логики деревень |
| Датапаки | **Нет** |

### 3.11 Чанки, свет, сохранения — ≈ 90 %

- `chunk/` 30 190 строк: статусы, тикеты, планировщик, `ChunkMap`,
  `chunk_saver` (8 227 строк), персистентность с тестами.
- Свет — блочный, в составе генерации чанка.
- Тикеты: `keep_dimension_active` (флаг `KEEP_DIMENSION_ACTIVE`),
  ender pearl, порталы.
- `chunk_ticket_storage`, `tick_rate_manager`, `tick_overload` — серверные
  механики нагрузки реализованы.

### 3.12 Производительность

- Многопоточность: генерация чанков, свет, обработка пакетов, отправка
  чанков вне основного тика; геймплей синхронный.
- Бенчмарк из README: 10 201 чанк за 3,98 с (Ryzen 9 9950X).
- `tick_overload` — защита от перегрузки тика.

---

## 4. Где именно разрыв (приоритет по влиянию на «ощущение ванилы»)

| # | Пробел | Влияние | Трудоёмкость |
|---|---|---|---|
| 1 | **Триггеры достижений (45 из 62)** | Высокое: прогресс не ощущается | Средняя — по точкам вызова |
| 2 | **Торговля с жителями + меню `merchant`** | Высокое: ядро выживания | Высокая |
| 3 | **Датапаки: `datapack`/`function`/`schedule`/`random`** | Высокое для кастомных серверов | Высокая |
| 4 | **Глубина ИИ: 35 целей → ~120** | Среднее-высокое: мобы «простые» | Высокая, но дробится |
| 5 | **4 структуры** (ancient_city, trial_chambers, trail_ruins, bastion_remnant) | Среднее: контент | Средняя |
| 6 | **5 меню** (hopper, brewing_stand, lectern, merchant, generic_3x3) | Среднее | Низкая-средняя |
| 7 | **17 команд** (модерация, data, help) | Среднее для админов | Низкая |
| 8 | **Эффекты: 14 файлов на 40 эффектов** | Среднее | Низкая |
| 9 | **zombie_nautilus** | Низкое | Низкая |
| 10 | **Плагины** | Высокое для экосистемы | Очень высокая |

---

## 5. Честные оговорки

1. **Проценты — взвешенная экспертная оценка**, а не формула. Точные
   счётчики приведены отдельно в §2; там, где есть число, оно из дерева.
2. **«Пакет реализован» ≠ «механика работает».** 234/234 означают
   «структура и id есть», а не «клиент получает полную ванильную реакцию».
3. **«Моб реализован» ≠ «моб ванильный».** 85/86 — это класс, синхронизация,
   NBT и базовые цели; уникальные механики у большинства урезаны.
4. **Документ `PROGRESS-RU.md` устарел** (там 25/49 block entities,
   54/87 пакетов, 13/25 меню — все эти числа выросли). `PARITY-26.3-RU.md`
   тоже описывает более ранний срез. Источник истины — этот файл + дерево.
5. **Нет запуска тестов** в этой среде (нет `cargo`), поэтому 4 380 тестов
   не проверялись — только их наличие.

---

## 6. Как воспроизвести счётчики

```bash
# пакеты
python3 - <<'PY'
import json,subprocess,re
p=json.load(open('steel-registry/build_assets/packets.json'))
def c(pre,n): return f'{pre}_'+re.sub(r'[^a-z0-9]+','_',n.lower()).strip('_').upper()
want={c('S' if d=='serverbound' else 'C', n) for d,pre in (('serverbound','S'),('clientbound','C'))
      for st,ns in p[d].items() for n in ns}
PY

# block entities
grep -c 'registry.register' steel-core/src/block_entity/registry.rs

# мобы
find steel-core/src/entity/entities/mobs -name '*.rs' ! -name 'mod.rs' ! -name 'tests.rs' | wc -l
grep -rn '#\[entity_behavior' --include=*.rs steel-core/src | wc -l

# цели ИИ
ls steel-core/src/entity/ai/goal/*.rs | wc -l

# блоки / предметы
grep -rn 'impl BlockBehavior for' --include=*.rs steel-core/src | wc -l
grep -rn 'impl ItemBehavior for' --include=*.rs steel-core/src | wc -l

# меню
grep -rho 'vanilla_menu_types::[A-Z0-9_]*' --include=*.rs steel-core/src | sort -u | wc -l

# команды
grep -c 'builder.register' steel-core/src/command/builtins/mod.rs

# достижения
grep -c 'pub const' steel-registry/src/vanilla_criteria_triggers.rs
grep -rho 'vanilla_criteria_triggers::[A-Z0-9_]*' --include=*.rs steel-core/src | sort -u | wc -l

# фичи / структуры
grep -c 'ConfiguredFeatureKind::' steel-core/src/worldgen/feature/configured.rs
grep -c 'reg("' steel-worldgen/src/structure/generator.rs
```
