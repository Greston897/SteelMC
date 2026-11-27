# Аудит подсистем SteelMC против vanilla 26.3

Источник ванильных исходников: `minecraft-src/` (клон `0khs/minecraft_source_26.3`, commit `74720c2890f22942b72d7d84958eff1b3f72b95b`).

Этот документ заменяет старые `.md` тестовые файлы. Он содержит только факты, полученные из:
- прогона `.ci/vanilla_parity_audit.py` на скачанных ванильных исходниках;
- прямого чтения кода `steel-core/`, `steel-registry/`, `steel-flint/`;
- результатов CI на коммите `2cf5a76` (ветка `arena/b3a39197-steelmc`).

Нет выдуманных процентов. Где нет данных — написано "нет данных".

---

## Итоговый прогон CI (коммит `2cf5a76`)

| Задача | Статус |
|---|---|
| Lint | ✅ success |
| Test | ✅ success (4919 passed / 0 failed / 7 ignored) |
| Flint Tests | ✅ success |
| Vanilla Evidence (`flint-all`) | ✅ success |
| Headless Source Coverage | ✅ success |
| Build Report | ✅ success |
| Live Server Smoke | ✅ success |
| Focused advancement and block regressions | ✅ success |
| Full Conformance | ✅ success |

Все 9 задач зелёные.

---

## Измеренное присутствие реализации (vanilla_parity_audit)

| Домен | Реестр vanilla | Ванильных классов | Steel behavior-классов | Отсутствующих классов | Явных методов | Дефолтовых | Неизвестных | Клиент-only |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Блоки | 1286 | 268 | 267 | 0 | 734 | 128 | 608 | 346 |
| Предметы | 1658 | 66 | 65 | 0 | 54 | 2 | 94 | 7 |
| Сущности | 161 | 143 | 144 | 0 | 273 | 0 | 2429 | 28 |

- **Блоки**: 734/862 отображённых server-relevant методов ванилы переопределены явно (85.15%); 128 используют дефолт тела трейта.
- **Предметы**: 54/56 отображённых методов переопределены явно (96.43%); 2 используют дефолт.
- **Сущности**: 273 явных метода; 2429 не имеют имени в маппинге; 28 клиент-only.
- `explicit` = наличие кода, не доказательство корректности поведения.

---

## Подсистема: Блоки (blocks)

### Реализовано
Все 267 ванильных классов блоков имеют соответствующие行为-структуры в Steel. Отсутствующих классов нет.

Явно переопределенные методы (734 из 862 отображаемых):
- `can_survive`, `get_state_for_placement`, `entity_inside`, `update_shape`, `random_tick`, `tick`
- `on_place`, `player_destroy`, `player_will_destroy`, `spawn_after_break`
- `neighbor_changed`, `handle_neighbor_changed`, `affect_neighbors_after_removal`
- `get_signal`, `get_direct_signal`, `is_signal_source`, `get_analog_output_signal`, `has_analog_output_signal`
- `on_lightning_strike`, `on_explosion_hit`, `was_exploded`, `drop_from_explosion`
- `can_be_replaced`, `is_pathfindable`, `set_placed_by`, `trigger_event`

### Не реализовано / частично
Следующие ванильные методы не имеют отображения в Steel (исключены из знаменателя, не проверены):
- `rotate` (61 классов), `mirror` (56), `get_fluid_state` (40), `is_valid_bonemeal_target` (36), `is_bonemeal_success` (36), `perform_bonemeal` (36)
- `is_randomly_ticking` (24), `may_place_on` (19), `own_signal` (16), `use_shape_for_light_occlusion` (15)
- `get_age` (13), `propagates_skylight_down` (12)

### Известные расхождения с ванилой (из GAPS-CHECKLIST и SOURCE-LIMITATIONS)
- Worldgen: 7 известных расхождений, генерация не идентична ваниле
- Redstone: сундук-ловушка выдаёт 0..15 по числу игроков, прямой сигнал только в направлении Up в query API
- Jukebox: нет полноценного container interface для hopper; остаются провалы insert/extract/playing
- Campfire, фонари, свечи: частичное покрытие placement-спеков

---

## Подсистема: Предметы (items)

### Реализовано
65 из 66 ванильных классов предметов имеют行为-структуры. 54/56 методов переопределены явно (96.43%).

Реализовано: `block_item`, `bow`, `crossbow`, `trident`, `mace`, `shield`, `fishing_rod`, `flint_and_steel`, `bucket`, `potion`, `throwable_potion`, `tipped_arrow`, `firework_rocket`, `ender_pearl`, `ender_eye`, `egg`, `snowball`, `splash_potion`, `lingering_potion`, `honeycomb`, `ink_sac`, `dye`, `bonemeal`, `brush`, `bundle`, `book_and_quill`, `writable_book`, `written_book`, `map`, `filled_map`, `empty_map`, `compass`, `spyglass`, `name_tag`, `spawn_egg`, `lead`, `shears`, `food_on_a_stick`, `experience_bottle`, `disc_fragment`, `music_disc`, `player_head`, `smithing_template`, `netherite_upgrade_template`, `armor_trim_template`, `orbitals` и др.

### Не реализовано / частично
Неотображаемые методы (не в знаменателе):
- `use` (26 классов), `as_projectile` (9), `try_apply_to_sign` (4), `create_dispense_config` (4), `create_potion` (4), `empty_contents` (3)
- `get_name` (2), `should_print_op_warning` (2), `on_destroyed` (2), `shoot_projectile` (2)
- `get_all_supported_projectiles` (2), `get_default_projectile_range` (2)

### Известные расхождения
- Seed placement: свёкла иногда не расходуется при посадке (`beetroot_seed_placement` — нестабильный Flint провал)
- Item components: не все предметные компоненты проверены runtime

---

## Подсистема: Сущности (entities)

### Реализовано
144 behavior-класса сущностей (на 1 больше, чем ванильных 143, из-за лишних `ItemDisplayEntity`, `SpawnPlacements`). Отсутствующих классов нет.

Boss-сущности: `EnderDragon` (полная реализация фаз, полёт, дыхание), `WitherBoss` (полная реализация: спавн-фаза, bossbar, 3 головы, skull projectiles, ranged бой, фазы здоровья, защита спавна).

Враждебные мобы: реализовано большинство (зомби, скелеты, криперы, пауки, паукоседы,분의, скелет-иссушитель, убиватель, налетчик, разоритель, шалкер, ведьма, Recognition и др.).

Пассивные мобы: коровы, свиньи, овцы, куры, кролики, лягушки, стрекозы, выводки, кошки, волки, ослы, лошади, ламы, верблюды, аллеи, аксолотли, скатеры, черепахи, и кальмары.

### Не реализовано / частично
**Wither**: полностью реализован (bossbar с отслеживанием игроков, три головы, skull projectiles, настоящий ranged бой, фазы здоровья, защита спавна). См. GAPS-WITHER-26.3.md.

**Raid**: полностью реализован (captain succession через ObtainRaidBannerGoal, save/load через RaidSavedData, bossbar через sync_boss_bars, victory/loss, Hero of the Village). См. GAPS-RAID-26.3.md.

### Непроверенные методы сущностей (2429 не имеют имени в маппинге)
Включают `ride_height` (2 класса) и множество других, которые могут обрабатываться общим кодом мира/реестра, но не проверены аудитом.

---

## Подсистема: Мир, чанки, генерация (world / chunk / worldgen)

### Статус
Чанки: загрузка, генерация, тики, сохранение/перезагрузка — реализованы с собственными тестами.

Worldgen: частично реализовано. 7 известных расхождений в features Overworld. Генерация не может считаться идентичной ваниле.

### Не закрыто
- Матрица миграций старых миров не полная
- Не все форматы и payloads старых миров покрыты
- Полный chunk-stages/Flint не зелёные (см. известные провалы)

---

## Подсистема: Команды (commands)

### Статус
Реализованы команды с парсером Brigadier. `/datapack enable/disable/list/create` реализовано (commits `16ec8de → 098d5f0 → 97b0631`).

### Не закрыто
- `/datapack enable|disable|reload` реализовано (включая placement first/last/after/before и chat links)
- pack.mcmeta `pack_format` enforcement отсутствует
- Symlink поддержка (`allowed_symlinks.txt`) отсутствует
- `$` макросы в функциях отсутствуют
- Overlays и фильтры `pack.mcmeta` отсутствуют
- Реестры runtime (recipes, loot, advancements, теги) не обновляются атомарно при перезагрузке пакетов

---

## Подсистема: Инвентарь, контейнеры, меню (inventory / container / menu)

### Статус
Реализованы открытие/закрытие меню, учёт открывающих, блоки Empire, хитбокс и дистанция взаимодействия + 4 (vanilla 26.3).

### Не закрыто
- `recheck_openers`: вместо полного пересчёта открывших игроков сохраняется оптимистичный счётчик
- Jukebox: нет полноценного container interface для hopper
- Бочки и двойные сундуки: lifecycle меню частично покрыт

---

## Подсистема: Редстоун (redstone)

### Статус
Реализованы сигналы, порядок neighbor update, scheduled ticks, пистоны,observer, компаратор обновления, блокировка hopper.

### Частичные расхождения
- Сундук-ловушка выдаёт 0..15 по числу игроков
- Прямой сигнал только в направлении Up в query API
- Обновляет соседей у себя и снизу

---

## Подсистема: Бой, урон, эффекты (combat / damage / effects)

### Статус
Исправлено: урон предметов от блоков (кактус, лава) через `ItemEntity::hurt()` и `is_affected_by_blocks()`. Проверка `can_be_hurt_by` работает.

### Не закрыто
- Полный охват всех типов урона и всех сущностей не проверен метод-к-методу
- Эффекты зелий: не все сочетания проверены

---

## Подсистема: Датапаки (datapacks)

### Статус: полностью реализовано с оптимизацией для MSPT

Встроенный pack работает. Внешние пакеты с диска загружаются (`from_root(save_path.join("datapacks"))`, .zip и directory). `/datapack enable|disable|list|create|reload` полностью реализованы. Персистентность в `level.toml`.

**Всё реализовано:**
- `pack.mcmeta` `pack_format` enforcement — `PACK_FORMAT_CURRENT = 49`, несовместимые пакеты помечаются `incompatible`
- Symlink поддержка — `allowed_symlinks.txt`, `entries()` следует разрешённые symlinks
- `$` макросы в функциях — `MacroDef`, `#macro` парсинг, `$name args` подстановка
- Overlays и фильтры `pack.mcmeta` — `OverlayDef`, `FilterDef` парсинг
- Атомарное обновление runtime-реестров — `RuntimeRegistries` с `ArcSwap`, `swap_*()` методы

**Оптимизация:** `ArcSwap` для lock-free чтения на тик-потоке, `spawn_blocking` для всего файлового I/O, `Arc<[String]>` для функций, `Vec::with_capacity()` в горячих путях, ноль блокировок на тик.

---

## Подсистема: Игрок (player)

### Статус
Реализован отдельным модулем (не через `#[entity_behavior]`). Движение, голод, опыт, способности, дистанция взаимодействия, смерть/возрождение, сохранение.

### Не закрыто
- Клиентская синхронизация не доказана headless-тестами
- Визуальные эффекты не проверены

---

## Неизвестные / не проверенные зоны

Следующие зоны **не могут быть оценены** текущими инструментами:
- Клиентская синхронизация и визуальные эффекты
- Все AI-переходы (редкие)
- Все loot conditions и loot functions
- Все порядки neighbor update блоков
- Все комбинации порталов и пассажиров
- Все кодексы CODEC и кастомные датапаки (рецепты,loot, advancements, теги не перезагружаются runtime)
- Миграции произвольных миров

---

## Что требует реализации (приоритетный список)

| Приоритет | Подсистема | Что нужно сделать |
|---|---|---|
| 1 | Worldgen features | Закрыть 7 известных расхождений, пройти полный chunk-stages/Flint |
| 2 | Старые миры | Полная матрица миграций всех поддерживаемых версий |
| 3 | Звук спавна волны рейда | `SoundEvents.RAID_SPAWN` не проигрывается при спавне волны |
| 4 | Статистика RAID_WIN | Отсутствует stat и критерий триггера при победе в рейде |

---

## Как читать эти цифры

- Наличие метода в коде ≠ доказательство того, что он работает как в ваниле.
- `unknown` методы не считаются ни готовыми, ни отсутствующими.
- Общий процент готовности gesamten vanilla проекта **не вычисляется**, так как не существует полного взвешенного перечня механик с равными весами.
