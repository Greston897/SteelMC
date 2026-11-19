# testwithoutplayers — обязательный автоматический контур

Объект `Player`, тестовый сетевой connection или несколько серверных сущностей
**не требуют живых игроков**. Логику таких сценариев нельзя откладывать в ручное
тестирование. `testwithplayers.md` выделяет только недоказанную текущим стендом
клиентскую и сетевую end-to-end часть.

## Полный код, не выборка механик

Обязательный запуск: `cargo test --workspace --verbose` с `RUSTFLAGS=-Dwarnings`.
Workflow `.github/workflows/test.yml` сохраняет исходный лог и
`automated-mechanics-tests.json`: **каждое фактически исполненное имя теста**, его
крейт/модуль, passed/failed/ignored. Нулевой вывод, compile failure или skipped
не считаются успешным выполнением механик. Поимённый отчёт дополняет exit code
cargo, а не подменяет его. Повторы тестов нельзя складывать как разные механики.

Физические зоны, которым нужна автоматическая проверка:

| Зона | Проверяемые стороны, в том числе отрицательные условия |
|---|---|
| Инфраструктура | bootstrap, конфиги, tick loop, shutdown, tasks, permissions, codecs, error paths |
| Протокол/синхронизация | packet codecs, ranges/limits, metadata/equipment/container updates, chunk tracking |
| Реестры/данные | registry/tag IDs, component roundtrip/defaults, recipes, loot, enchantments, migrations |
| Мир/чанки | tickets, load/generation stages, light, entity ticking, unload/save/reload, dimensions |
| Генерация | biomes/structures/features/decoration, независимые vanilla hashes, границы чанков |
| Блоки/предметы | placement, state transitions, survival/support, waterlogging, consume/durability/remainders |
| Физика | collision, falling blocks, fluid spread, explosions, projectiles, entity pushing |
| Редстоун | signals, neighbor order, scheduled ticks, pistons, observers, comparator updates, hopper locks |
| Block entities | inventory IO, furnace/brewing/campfire timers, loot unpacking, dirty callbacks, persistence |
| Игрок | movement rules, hunger/XP, abilities, interaction reach, death/respawn, persistence |
| Бой | cooldown, crit, sweep, reach, armor/toughness, effects, shields, knockback, projectiles, PvP rules |
| AI/мобы | targeting/navigation, schedules/goals, spawn/despawn, equipment, tame/breed, bosses/raids |
| Меню | recipes, slot mappings, shift/drag/cursor, validation, open/close/recount, concurrent mutations |
| Команды | parser/executor/wire tree, selectors, permissions, world bounds, data/item/score mutations |
| Сохранение | clean stop, restart, empty/nonempty state, block entities, entities, inventories, scoreboard |

Таблица — **обязательная область проверки**, не утверждение о покрытии каждого
элемента. Поимённые существующие тесты/пути и физическая статистика находятся
в `PROGRESS-26.3-INVENTORY.md`. Отсутствующий тест — пробел, а не PASS.

## Настоящие процессы Steel и vanilla 26.3

`.github/workflows/live-server-smoke.yml` запускает один сценарный файл
`.ci/live_server_smoke.py` на двух двигателях и затем сравнивает результаты.
Официальный jar имеет проверенные по manifest Mojang version/size/SHA-1.
Необходимы полные одинаковые непустые списки проверок и отсутствие failures;
любое расхождение или усечённый прогон блокирует `compare`.

Включены TCP/RCON, загрузка мира без клиентов, блоки/clone, item predicates,
лампы/scheduled ticks, песок, поршень, печь, воронка и её блокировка,
компаратор при изменении предметов, summon/damage/instant-health/death и
повторный запуск того же дискового мира. Статус каждого нового набора берётся
из конкретного завершённого Actions run; добавленный сценарий сам по себе не PASS.

Отрицательная проверка переноса заблокированной воронкой наблюдается на
протяжении серверных тиков: мгновенный успех до первого тика недостаточен.
Условия обязаны выполнить независимый `scoreboard players get` с точным ответом,
а не просто вернуть число в ошибке или эхо команды.

## Независимое соответствие

`.github/workflows/vanilla-evidence.yml` дополнительно запускает worldgen hashes,
registry/tags, doctests и **полный** закреплённый Flint-набор. Зелёный стандартный
Rust-набор и реальные smoke-сценарии не заменяют этот контур. Известные failures
нельзя превращать в ignored, удалять из spec или закрывать заменой goldens без
независимой проверки. Прежние ошибки/неполнота перечислены в
`READINESS-26.3-FACTS.md`; свежий результат нельзя выводить из старого SHA.

## Новые боевые регрессии

`steel-core/src/player/tests.rs`:

- `combat_critical_requires_full_cooldown_and_does_not_multiply_enchantment_bonus`:
  слабый/полный удар, отдельный бонус sharpness, слепота;
- `combat_sweep_requires_grounded_charged_sword_and_uses_primary_target_box`:
  несколько настоящих серверных Player-entities, меч/топор, земля/воздух,
  заряженный/незаряженный удар, вторичная цель за пределами старого радиуса.

Они исполняются автоматически полным workspace-набором, не требуют клиентов
или людей. Это ещё не исчерпывающая проверка всего боя.

## Рейды: реальное исполнение без людей

Проверенный полный workspace-run:
[Test 44608097288 / код 0da2e9b](https://github.com/Greston897/SteelMC/actions/runs/44608097288).

- `world/raid.rs`: 6 законченных столкновений Easy/Normal/Hard с обычным и
  бонусным количеством волн — 33 реально созданных группы; наездники поздних волн;
  блокирование прогресса живыми/не загруженными участниками; настоящий death hook.
- Там же: victory delay, hero attribution и исключение наблюдателя, duration/amplifier
  эффекта, loss, timeout, сериализация, occupied village POI и высотные границы,
  оружие капитана, enchantment providers и геометрия horn.
- `entity/mob_effect/raid_omen.rs`: 600 тиков перехода, сохранённый origin после
  перемещения, отсутствие origin и запрет измерения.
- `entity/entities/mobs/hostile/pillager.rs`: Quick Charge экипированного оружия
  меняет серверное время заряжания — 25/20/15 тиков.
- `player/player_data_storage/tests.rs`: roundtrip origin и чтение старой v12
  без выдумывания origin; существующие v10/v11 migrations остаются в общем наборе.

38 live-smoke условий **не являются** live-проверкой рейда: рейды проверены
указанными headless world/entity-тестами. Полнота AI и настоящий клиентский UI
не выводятся из успешности этих регрессий.
