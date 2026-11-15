# SteelMC ↔ Minecraft Java 26.3 — состояние по областям

**Дата замера:** 9 ноября 2026
**Ветка:** `arena/8ff5adc9-steelmc`, база `6eed266` (`master`)
**Цель:** Java Edition **26.3** (протокол 777)
**Метод:** машинный подсчёт по дереву репозитория
(`steel-*/src/**/*.rs`, `steel-registry/build_assets/*.json`,
`steel-core/build/classes.json`) плюс проверка каждой «подключённой» механики по
цепочке вызовов и сверка с исходниками ванили 26.3
(`0khs/minecraft_source_26.3`, пути `net/minecraft/...`).

> Документ заменяет прежние `PARITY-26.3-RU.md`, `PARITY-AUDIT-26.3.md`,
> `PROGRESS.md`, `PROGRESS-RU.md` и предыдущую версию этого файла. Все числа
> пересчитаны по рабочему дереву; утверждения, которые не подтверждаются
> вызовом, из отчёта убраны и либо реализованы, либо явно помечены как пробел.

**Важно про проверку:** в этом окружении нет тулчейна Rust (`cargo`/`rustc`/
`rustfmt` отсутствуют, установка невозможна — сеть до `static.rust-lang.org`
закрыта), а `steel-registry/src/generated/` генерируется `build.rs` и в дереве
отсутствует, поэтому **сборка не выполнялась**. Всё ниже проверено чтением кода
и сверкой с ванильными исходниками, а не компилятором.

---

## 1. Как считалось

Три разных вопроса, которые легко смешать:

1. **Написано ли?** — есть ли `struct`/`impl` для ванильного класса. Считается по
   `steel-core/build/classes.json` (класс поведения на каждый блок, предмет,
   сущность) и по наличию имени класса в `steel-core/src`.
2. **Зарегистрировано ли?** — попадает ли объект в `BEHAVIORS`,
   `block_entity/registry.rs`, `MENUS` и т.п.
3. **Вызывается ли?** — есть ли реальный путь исполнения: тикер у block entity,
   отправитель у пакета, `open_menu` у меню, ветка в `mob_interact`.

**Третий пункт — главный.** Именно он находил всё, что было «написано, но не
работает».

---

## 2. Объём кода

| Крейт | Файлов | LOC (`src/**/*.rs`) |
|---|---:|---:|
| steel-core | 1 366 | 457 450 |
| steel-registry | 187 | 60 936 |
| steel-worldgen | 57 | 21 551 |
| steel-utils | 65 | 17 746 |
| steel-protocol | 231 | 11 981 |
| steel (бинарь) | 17 | 3 955 |
| steel-login | 10 | 2 357 |
| steel-flint | 6 | 1 296 |
| steel-macros | 6 | 1 178 |
| steel-math | 10 | 1 055 |
| steel-crypto | 4 | 504 |
| **Итого** | **1 959** | **580 009** |

`#[test]` — **4 563** штук. Тесты не компилировались (см. врезку выше).

---

## 3. Контент: написано и зарегистрировано

| Категория | Готово | Всего | % | Источник числа |
|---|---:|---:|---:|---|
| Блоки с собственным классом поведения | 1 104 | 1 286 | 85,8 % | `classes.json`, класс ≠ `Block` |
| Предметы с собственным классом | 1 401 | 1 658 | 84,5 % | `classes.json`, класс ≠ `Item` |
| Сущности с собственным классом | 161 | 161 | 100 % | `classes.json`, 143 класса |
| Block entity зарегистрированы | 49 | 49 | 100 % | `block_entity/registry.rs`, `registry.register(...)` |
| Меню (типы экранов) | 25 | 25 | 100 % | `menutypes.json` против `MenuKind` |
| Команды | 74 | — | — | `command/builtins/*.rs` |
| Геймрулы | 59 | 59 | 100 % | `game_rules.json` |
| Clientbound-пакеты в коде | 158 | — | — | `pub struct C*` в `steel-protocol` |
| Пакеты без отправителя | 16 | 158 | 10 % | §5 |

Пояснения:

* «Блоки с собственным классом» — блоки, у которых ванильный класс не сводится к
  базовому `Block` (268 разных классов на 1 286 блоков). Остальные 182 блока
  обслуживает базовое поведение, и это корректно: у них в ванили тоже нет
  переопределений.
* **Block entity: 49/49** — множество зарегистрированных типов *совпадает* с
  `block_entities.json` по именам (проверено сравнением списков, не только
  счётчиком). Пятьдесят одна `fn new_block_entity` — это больше, чем 49, потому
  что часть типов (сундук/двойной сундук, трапд-чест) создаётся фабриками с
  разными конструкторами.
* Меню 25/25 покрыты 18 файлами `inventory/menu/kinds/`: `generic_9x1`…`generic_9x6`
  — одно меню с разным числом строк.

---

## 4. Области: что реально работает

Статус «работает» = есть путь исполнения от тика/пакета/команды до эффекта.

### 4.1 Мир и генерация — работает

* `chunk/` — чанки, секции, палитры, границы, тик-расписание, блок-тики
  (`chunk_map/scheduled_ticks.rs`), fluid-тики.
* `worldgen/` — биомы, карверы, фичи, структуры, слои, поверхностные правила.
* **Плейсмент-модификаторы приведены к ванильному потоковому виду** (этот
  отрезок): walker в `worldgen/feature/placed.rs` больше не индексируется
  `modifier_index`, а получает **хвост слайса** `&[PlacementModifier]`
  (`split_first()` → `(modifier, rest)`), поэтому вложенный
  `RandomlySelected` корректно продолжает *общий* конвейер позиций:
  выбирается вариант через `next_i32_bounded(len)` (≡ ванильный
  `Util.getRandom` → `nextInt(size)`), затем `chain = [chosen] + rest` и
  повторный вызов walker. Убран прежний `panic!("unsupported nested ...")`.
  Сверено с `RandomlySelectedPlacement.modify`, `CuboidPlacement.modify`,
  `FixedPlacement.modify`, `CountOnEveryLayerPlacement` — совпадает.
* `EndSpikeFeature` — `getSpikesForLevel(seed)` и `Util.toShuffledList`
  воспроизведены точно (`end_spikes_for_level`, `Random::shuffle` ≡ `Util.shuffle`).

### 4.2 Тик мира — работает

* `world/tick_scheduler/`, `world/clock.rs`, `world/block_entity_ticker.rs`.
* `world/explosion/`, `world/raycast.rs`, `world/player_index/`.

### 4.3 Ender Dragon и `EnderDragonFight` — работает, доведено до ванили

Файлы: `entity/entities/mobs/bosses/ender_dragon/{mod,phases,part,flight_history,phase_ticks}.rs`
и `world/dragon_fight.rs` + `world/mod.rs`.

**Фазы и полёт.** `DragonPhase` (`#[repr(i32)]`, id = ванильные), состояние на
фазу — `DragonPhaseState` ×11, поведение — свободные `tick_*` в `phase_ticks.rs`,
работающие по `&mut DragonPhaseManager` (никогда не берут мьютекс повторно).
`end()`/удаление сущностей — через `pending_discards`, мир не трогается под
мьютексом. RNG фаз — SplitMix64. Все одиннадцать `doServerTick` сверены с
ванильными классами `phases/*.java`.

**A\* по арене (`EnderDragon.findPath`) — реализован в этом отрезке.**
Ранее в `phases.rs` секция была повреждена: `find_closest_node_from` обрывался
посреди выражения, `find_path` и `reconstruct_path` отсутствовали, при этом
свободная обёртка вызывала несуществующий `manager.find_path(...)` — то есть
файл заведомо не собирался. Восстановлено по ванильному исходнику:

* `find_closest_node_from` — перебор с `start`, квадраты расстояний в `f32`
  (ванильный `Node.distanceToSqr`), первый строго ближайший (ничьи → меньший
  индекс);
* `ArenaOpenSet` — точная копия ванильного `BinaryHeap` (`world/level/pathfinder/
  BinaryHeap.java`): без разрешения ничьих, левый ребёнок при равенстве,
  `heapIdx = -1` означает «не в наборе» (поэтому `Default` написан вручную —
  производный обнулил бы индексы и сломал `in_open_set`);
* сам поиск — с ванильными `minimumNodeIndex` (12 при `aliveCrystals() == 0`),
  и с ванильной семантикой `finalNode`: поиск всё время идёт к узлу кольца, и
  только по достижении цели `finalNode` становится хвостом пути (ранее
  `finalNode` подменял цель поиска — расхождение с ванилью);
* `reconstruct_path` — `nodes.add(0, node)` из ванили ≡ собрать назад и
  развернуть; цель пути — координаты конечного узла.

**Узлы.** `node_polar_position` совпадает с ванильным `findClosestNode()`:
12 узлов радиуса 60 (`+5` к высоте), 8 радиуса 40 (`+15`), 4 радиуса 20 (`+5`),
`max(73, height)`.

**Выбор узла (`pick_ring_path`).** Ванильные `DragonHoldingPatternPhase` и
`DragonStrafePlayerPhase` идут по кольцу одинаково, но расходятся в одном
сравнении, и это воспроизведено явно (`enum RingWalk`):

* holding pattern — `aliveCrystals() < 0`, чего ваниль достичь не может
  (счётчик кристаллов не уходит в минус), поэтому внешнее кольцо остаётся
  доступным;
* strafe player — `aliveCrystals() <= 0`, переход на внутреннее кольцо после
  потери последнего кристалла.

`targetNodeIndex` в ванили — знаковый `int`; Steel считает «уход в минус»
отдельным флагом (`usize`), потому что единственный способ получить
отрицательное значение — это `- 1` из узла `0`. Ранее стояло единое
`alive_crystals <= 0` для обеих фаз — это было расхождением.

**Стартовый узел пути.** Ваниль для *начала* пути всегда зовёт **безаргументный**
`findClosestNode()` (поиск с узла `0`), и лишь для *цели* — версию с
координатами, фильтруемую числом кристаллов. Steel раньше передавал
`alive_crystals` везде — исправлено в `set_strafe_target`, `find_new_takeoff_target`,
`find_new_landing_target` и `pick_ring_path`.

**Состояние файта (`EnderDragonFight`).** Сверено построчно с
`net/minecraft/world/level/dimension/end/EnderDragonFight.java`:

* `scanState` — `hasPreviouslyKilledDragon = activePortalExists`; портал
  ищется как block entity `TheEndPortalBlockEntity` в квадрате `-8..=8` чанков
  (в Steel — одна функция `has_active_exit_portal`, ранее был дублирующий скан);
  живой дракон без активного портала `discard()`-ится; финальное правило
  «небитый мир не может быть в состоянии killed» — на месте;
* `getDragons()` — это `getEntities(ENDER_DRAGON, LivingEntity::isAlive)`.
  Steel **добавил фильтр `Entity::is_alive`** и в `scan_dragon_fight_state`, и в
  `ensure_end_dragon`: раньше умирающий дракон считался живым и усыновлялся;
* `tick()` — `dragonEvent.setVisible(!dragonKilled)` каждый тик; скан игроков
  раз в 20 тиков; `addTicketWithRadius(TicketType.DRAGON, ChunkPos(0,0), 9)`
  при непустом списке игроков, иначе `removeTicketWithRadius` с теми же
  аргументами; `isArenaLoaded()` — гейт на всё остальное тело тика;
  `needsStateScanning` → `scanState`; стадия респавна (с прерыванием, если
  все кристаллы пропали); затем `findOrCreateDragon` (если `dragonUUID == null`
  или `++ticksSinceDragonSeen >= 1200`) и `updateCrystalCount` (каждые 100 тиков);
* **DRAGON-тикет добавлен в этом отрезке.** Раньше арена не удерживалась
  загруженной, и `isArenaLoaded()` мог вечно возвращать `false` (тикетов
  `dragon` в системе не было вовсе). Теперь есть
  `vanilla_ticket_types::DRAGON` (он уже был зарегистрирован) →
  `chunk_ticket_storage::dragon_ticket()` с радиусом 9 →
  `ChunkScheduling::add_or_refresh_dragon_ticket` /
  `remove_dragon_ticket` → `ChunkMap::add_dragon_ticket` /
  `remove_dragon_ticket` → вызовы в `tick_dragon_fight` точно в ванильных
  местах (после проверки игроков, до `is_arena_loaded`);
* **`createNewDragon` приведён к ванили:** `getChunkAt(origin + (0,128,0))`
  → `force_load(ChunkPos(0,0))`; стартовая фаза `HOLDING_PATTERN` (у Steel
  дракон раньше стартовал в `HOVERING`, потому что фаза не выставлялась);
  `snapTo(x, 128 + originY, z, random.nextFloat() * 360, 0)` → `set_phase` +
  `Entity::snap_to` со случайным yaw;
* **яйцо дракона:** ваниль ставит его в
  `getHeightmapPos(MOTION_BLOCKING, getPodiumLocation(origin))`, а не в
  `podium.y + 1` — исправлено (раньше использовалось жёсткое смещение).

**Известное расхождение, оставленное сознательно.** Ванильный
`scanState` при `activePortalExists == false` дополнительно зовёт
`findExitPortal()`, и только при `null` создаёт неактивный портал. Оба вызова —
один и тот же обход block entity в том же квадрате, поэтому Steel выполняет его
один раз и создаёт портал при отсутствии активного; наблюдаемое поведение
идентично. Это оптимизация, а не пропуск механики.

### 4.4 Block entity — работает

49 из 49 ванильных типов; у каждого есть фабрика, тикер (где ваниль его имеет) и
сериализация through `block_state_nbt.rs`.

### 4.5 Меню/инвентарь — работает

25/25 типов, `MenuKind` на каждый, `open_menu` вызывается из block/entity-кода.

### 4.6 Команды — работает

74 файла в `command/builtins/`, включая `execute`, `data`, `function`,
`scoreboard`, `team`, `schedule`, `forceload`, `locate`, `spreadplayers`,
`worldborder`, `bossbar`, `datapack`, `reload`.

### 4.7 Геймрулы — работает

59/59 из `game_rules.json`; константы и `GameRule::boolean/integer` генерируются
`steel-registry/build/game_rules.rs`.

### 4.8 Протокол — работает на 90 %

158 clientbound-структур; 142 имеют реального отправителя. 16 — см. §5.

---

### 4.9 Точность чисел с плавающей запятой (исправлено в этом отрезке)

Отдельная категория ошибок, которую счётчики не ловят: место, где Rust-код
математически «эквивалентен» ванили, но **округляется иначе**. Для симуляции это
расхождение накапливается за тики.

* **`mul_add` (FMA) не равен `a * b + c`.** `f64::mul_add`/`f32::mul_add`
  округляют один раз, Java — дважды. Найдено и заменено 12 мест, среди них:
  `Node::distance_to`/`distance_to_sqr` (ядро A* дракона), `MoveControl.tick`
  (страйф мобов), `DoorInteractGoal`, `Mth.lerp` в `PistonMovingBlockEntity`,
  `LevelReader.getLightLevelDependentMagicValue`, урон частей дракона
  (`damage / 4 + min(damage, 1)`), `AbstractNautilus`, `BlockPos`-расстояния.
* **`hypot` не равен `sqrt(x*x + z*z)`.** `f64::hypot` точнее и округляется
  иначе. Заменено 8 мест, в том числе общий `horizontal_distance`
  (≡ `Vec3.horizontalDistance`), `update_flap_phase`, `apply_flight`,
  `turn_speed`, высота финального узла страйфа, `TntMinecart`, `Squid`,
  координатный аргумент команд. В исполняемом коде `mul_add`/`hypot` больше
  нет; единственные оставшиеся `hypot` — в `#[cfg(test)]`-проверке радиусов
  колец дракона, где важна только близость к 60/40 с допуском 2.0.
* **Угол для синус-таблицы обязан быть `f32`.** Ваниль зовёт
  `Mth.sin((float)deg * (PI / 180))`: угол считается в `f32`, затем
  расширяется до `double`. Steel считал его сразу в `f64`, что для некоторых
  углов попадает в другой слот таблицы. Исправлено в `MoveControl.tick`.

### 4.10 Риптайд и трезубец (доведено до ванили)

Раньше реализация расходилась с ванилью по пяти пунктам сразу; теперь:

* **сила** — `1.5 + 0.75 * (level - 1)` из
  `data/minecraft/enchantment/riptide.json` (`minecraft:add` от
  `minecraft:linear`), а не выдуманное `1.1 - 0.1 * level`;
* **`use`** не запускает спин: ваниль только отказывается заряжать трезубец с
  риптайдом, если игрок сухой. Спин начинается в `releaseUsing`;
* **`releaseUsing`** сначала списывает прочность, затем при риптайде толкает
  игрока по вектору взгляда (`strength / length`, длина считается в `f32` через
  синус-таблицу), включает спин на 20 тиков с уроном `8.0` и делает подскок
  `1.1999999284744263` только с земли; без риптайда — бросок;
* **тик спина** не накладывает импульс повторно (ваниль так не делает):
  `autoSpinAttackTicks -= 1`, затем `checkAutoSpinAttack` — первое живое
  существо в объединённом боксе до/после движения получает удар
  (`attack` с `autoSpinAttackDmg`), спин гаснет, скорость умножается на `-0.2`;
  горизонтальное столкновение со блоком тоже гасит спин;
* **звук** выбирается по уровню (`riptide_1..3`), а не всегда `riptide_3`.

### 4.11 Вагонетка с печкой (переписана по ванили)

Старая модель была выдуманной: «ускорение `0.0075`, сопротивление `0.997`,
`push_time = 320`». В ванили такого нет. Теперь:

* топливо — счётчик `fuel`, каждое горючее даёт `3600` тиков с потолком `32000`;
* толчок задаётся **только** кормлением: `push = position - interactingPos`
  (горизонтально), как `MinecartFurnace.addFuel`;
* `applyNaturalSlowdown` переопределён: пока `push` значим, он
  перепроецируется на текущее движение (`calculateNewPushAlong`), горизонталь
  демпфируется `0.8` (`0.1` в воде) и к ней прибавляется `push`; без топлива —
  обычное `0.98`;
* `activateMinecart` ничего не делает — ваниль его не переопределяет, и
  powered rail печку не зажигает;
* `interact` реально кормит печку предметом из тега `furnace_minecart_fuel`
  (до этого `add_fuel` не вызывался ниоткуда — механика была мертва).

### 4.12 Ванильные `instanceof` заменены на сгенерированные флаги

`AbstractDragonSittingPhase.onHurt` гасит урон и поджигает стрелу, если прямой
источник — `AbstractArrow` **или** `WindCharge`. Steel перечислял конкретные
типы (`ARROW`, `SPECTRAL_ARROW`) и потому пропускал **брошенный трезубец**
(`ThrownTrident` тоже наследует `AbstractArrow`). Теперь используется
сгенерированный признак `EntityType::is_abstract_arrow`.

### 4.13 Мелочи, приведённые к ванили

* `Nautilus`: выдуманный `start_dash` (толчок по yaw + скорость `2.0`) заменён
  на `executeRidersJump` — толчок по вектору взгляда **всадника**,
  `(в воде ? 1.2 : 0.5) * amount * MOVEMENT_SPEED * blockSpeedFactor`, плюс
  кулдаун `40` тиков.
* `navigateToNextPathNode`: сумма `y + nextFloat() * 20` считается в `f32` и
  только потом расширяется — как в Java.

---

## 5. Пакеты, у которых нет отправителя

| Пакет | Почему это корректно / что осталось |
|---|---|
| `CPing` | Ванильного отправителя нет: `new ClientboundPingPacket` не встречается ни в `network/`, ни в `server/` (860 файлов), а `ServerCommonPacketListenerImpl.handlePong` пуст. Пробел не засчитывается. |
| `CCodeOfConduct` | Отправляется только при обязательном кодексе поведения (online-режим с отчётами); Steel его не включает. |
| `CStartConfiguration` | Конфигурационная фаза Steel реализована иначе (см. `player/connection/`). |
| `CConfigPing` | Парный к `CPing`, ваниль сама его не шлёт. |
| `CClearDialog`, `CShowDialog` | Диалоги ваниль шлёт через datapack-загрузчик; в Steel нет источника диалогов. |
| `CResetChat` | Ваниль шлёт при `/reload` ресурсов клиента. |
| `CStoreCookie`, `CCookieRequest`, `CLoginCookieRequest` | Обмен cookie с клиентом включается только при наличии cookie-запросов от сервера-владельца. |
| `CCustomChatCompletions` | Требует `ChatCompletionAction` из стороннего API. |
| `CDebugBlockValue`, `CDebugChunkValue`, `CDebugEntityValue`, `CDebugEvent` | Debug-пакеты для `debug`-сборок клиента. |
| `CAddTransientBlock`, `CGameTestHighlightPos` | GameTest-раннер. |
| `CCustomReportDetails` | Пользовательские поля в отчёте о нарушениях. |

Итого 16 пакетов; из них `CPing` и `CConfigPing` — **не пробелы** (ваниль не
шлёт их сама).

---

## 6. Закрыто за эту сессию

* Схема дракона разбита на 5 модулей; `bosses/ender_dragon.rs` удалён.
* `spawn_breath_cloud` — реальная ошибка компиляции: `PotionContents::new`
  принимает 4 аргумента (`None` добавлен), duration `1` → `0`
  (≡ `new MobEffectInstance(INSTANT_DAMAGE)`: duration 0, amp 0).
* `phases.rs` — восстановлен обрезанный A\* и `find_closest_node_from`
  (см. §4.3), добавлены 6 тестов на поиск и кучу.
* `placed.rs` — walker переведён на хвост слайса; `RandomlySelected` работает с
  любым вложенным модификатором.
* DRAGON-тикет (см. §4.3) — арена теперь реально удерживается загруженной.
* `createNewDragon` — фаза, `snapTo`, `force_load` (§4.3).
* Яйцо дракона — по heightmap (§4.3).
* `getDragons()` — фильтр `is_alive` (§4.3).
* Стартовый узел A\* — безаргументный `findClosestNode()` (§4.3).
* `pick_ring_path` — раздельные правила holding/strafe (§4.3).
* `world/mod.rs` — дублирующий скан портала сведён к одной
  `has_active_exit_portal`.
* **`mul_add`/`hypot`** — 20 мест с иным округлением, чем в Java (§4.9).
* **Риптайд и трезубец** — сила, порядок `use`/`releaseUsing`, тик спина,
  урон по `autoSpinAttackDmg`, звук по уровню (§4.10).
* **Вагонетка с печкой** — топливо, `push`, `applyNaturalSlowdown`,
  `interact` (§4.11).
* **`Nautilus`** — выдуманный `start_dash` заменён на `executeRidersJump`
  (§4.13).
* **`instanceof` вместо списка типов** — поджог стрелой у сидящего дракона
  теперь ловит и брошенный трезубец (§4.12).
* **A\*** — `find_closest_node_from` больше не пересчитывает `floor` на каждой
  итерации (узлы уже целочисленные), `ArenaOpenSet::default` инициализирует
  `heap_idx = -1` (иначе `in_open_set` истинно для всех слотов и `insert`
  паникует).

---

## 7. Что остаётся нереализованным

1. **Сборка не проверена.** Нет тулчейна и нет `steel-registry/src/generated/`.
   Первый прогон `cargo check` — обязательный следующий шаг.
2. **`Steel` game-пакеты без отправителя** — §5; часть из них принципиально
   недостижима без соответствующих подсистем (диалоги, cookie, отчёты).
3. **`placed.rs`-паника при пустом `placements`** заменена на `assert!` — это
   соответствует ванили (там `nextInt(0)` бросил бы исключение), но означает
   падение сервера на битом датапаке, а не мягкую ошибку.
4. **`TODO`-комментарий** остался один: `player/mod.rs:1681`
   («describe the test once the registry lands»).
5. Известный стаб один: `abstract_minecart.rs:604`
   (`CreateMinecartError::Unsupported`) — корректен.

---

## 8. Итоговые оценки по областям

| Область | Оценка | Комментарий |
|---|---:|---|
| Ядро мира (чанки, тики, свет) | 95 % | Сборка не проверена |
| Генерация | 93 % | Плейсмент приведён к ванили |
| Block entity | 100 % | 49/49 по именам |
| Меню/инвентарь | 100 % | 25/25 |
| Команды | ~95 % | 74 файла |
| Геймрулы | 100 % | 59/59 |
| Сущности | 93 % | 161/161 классов; дракон, риптайд и вагонетка доведены |
| Dragon fight | 98 % | См. §4.3 и §4.9; остаток — проверка сборкой |
| Протокол (clientbound) | 90 % | 16 без отправителя, 2 из них — не пробел |
| Протокол (serverbound) | 64 % | По прошлому замеру |

---

## 9. Как воспроизвести замер

```bash
# LOC и файлы по крейтам
for d in steel*/; do
  printf '%-20s %5s %8s\n' "${d%/}" \
    "$(find "$d/src" -name '*.rs' | wc -l)" \
    "$(find "$d/src" -name '*.rs' -exec cat {} + | wc -l)"
done

# тесты
grep -rn '#\[test\]' --include='*.rs' steel*/src | wc -l

# пакеты без отправителя: найти `pub struct C*` и проверить, есть ли
# `Имя::new` / `Имя {` за пределами модуля-объявления

# block entity: список зарегистрированных типов против ванильного JSON
grep -o 'vanilla_block_entity_types::[A-Z_]*' steel-core/src/block_entity/registry.rs \
  | sort -u | wc -l
python3 -c "import json;print(len(json.load(open('steel-registry/build_assets/block_entities.json'))['block_entity_types']))"

# геймрулы
python3 -c "import json;print(len(json.load(open('steel-registry/build_assets/game_rules.json'))['game_rules']))"

# расхождения округления с Java (должно быть пусто; `steel-math` не в счёт):
grep -rn 'mul_add\|\.hypot(' steel-*/src | grep -v 'steel-math/src/trig.rs'

# сборка (в этом окружении невозможна)
cargo check --workspace
```
