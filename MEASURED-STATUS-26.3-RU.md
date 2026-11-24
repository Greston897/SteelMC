# Замеренный статус 26.3 — только реальные запуски и пересчёт по файлам

Этот документ не содержит оценок «на глаз». Каждое число получено либо
завершённым запуском CI на указанном SHA, либо пересчётом исходников
инструментом `.ci/vanilla_parity_audit.py`. Неизвестное помечено как
неизвестное.

## Реальные запуски на `29ac73c` (ветка `arena/b3a39197-steelmc`)

| Workflow | Run | Результат |
|---|---|---|
| Test | [45096687184](https://github.com/Greston897/SteelMC/actions/runs/45096687184) | success — **4919 passed / 0 failed / 7 ignored** (полный `cargo test --workspace`) |
| Lint | [45096687196](https://github.com/Greston897/SteelMC/actions/runs/45096687196) | success |
| Live Server Smoke | [45096687203](https://github.com/Greston897/SteelMC/actions/runs/45096687203) | success — steel, vanilla 26.3, compare |
| Flint Tests | [45096687225](https://github.com/Greston897/SteelMC/actions/runs/45096687225) | success |
| Build Report | [45096687207](https://github.com/Greston897/SteelMC/actions/runs/45096687207) | success |
| Vanilla Evidence | [45096687201](https://github.com/Greston897/SteelMC/actions/runs/45096687201) | **failure** — только job `flint-all`; biomes, structures, chunk-stages, registry-sync-tags, api-doctests, live-server — success |

- Прежние документы заявляли `4802 passed`. Реальный пересчёт на этом SHA даёт
  **4919 passed**. Старое число не актуально и не должно использоваться.
- `7 ignored` не исполнены. Это не успех и не провал.
- Запуск идёт на одной ветке без людей и без настоящих клиентов; клиентский
  слой этим не доказан.

## Flint: что именно не прошло

В зафиксированном прогоне `flint-all` (1012 тестов, 32 шарда) — **1011
прошли, 1 упал**:

- `beetroot_seed_placement` (шард 20):
  `t9: expected minecraft:air, got minecraft:beetroot_seeds @ Hotbar1`.

Тест нестабильный: в другом запуске тот же шард 20 прошёл. Это единственное
воспроизведённое расхождение Flint на этом SHA, а не «весь Flint красный».
Причина — семя свёклы иногда не расходуется при посадке; код предмета —
`BlockItem` (`steel-core/src/behavior/items/...`), поведение блока —
`steel-core/src/behavior/blocks/vegetation/beetroots.rs`.

## Замеренный аудит реализации

Инструмент: `.ci/vanilla_parity_audit.py` (самопроверки —
`.ci/test_vanilla_parity_audit.py`, 12 тестов). Отчёт: `VANILLA-PARITY-MEASURED-26.3.md`.

| Домен | Реестр vanilla | Поведенческие классы | Отсутствующие классы | Explicit-методы | Defaulted | Unknown |
|---|---:|---:|---:|---:|---:|---:|
| Блоки | 1286 | 267 | 0 | 734/862 (85.15%) | 128 | 608 |
| Предметы | 1658 | 65 | 0 | 54/56 (96.43%) | 2 | 94 |
| Сущности | 161 | 144 | 0 | см. JSON | — | 2 |

- Реестры совпадают с `steel-registry/build_assets` полностью.
- `Default Block`/`Default Item` обрабатываются дефолтным поведением; `Player`
  реализован отдельным модулем, не через `#[entity_behavior]`.
- `Explicit` — vanilla-метод `@Override` имеет переопределённый одноимённый
  метод в Rust-структуре. `Defaulted` — структура не переопределяет метод,
  используется дефолт трейта. `Unknown` — сопоставление не найдено; эти методы
  не входят в знаменатель и не считаются ни готовыми, ни отсутствующими.
- Процент — это **наличие реализации, а не корректность**. Зелёный тест и
  существование метода не доказывают совпадение с vanilla.

## Что этим НЕ доказано

- Корректность поведения в целом; только наличие кода и прохождение тестов.
- Клиентская синхронизация, визуальные эффекты, все AI-переходы, все loot
  conditions/functions, все порядки neighbor update.
- Datapack runtime-реестры (recipes, loot, advancements, теги) обновляются
  атомарно через `RuntimeRegistries`/`ArcSwap`; pack.mcmeta pack_format
  (`PACK_FORMAT_CURRENT = 49`), symlinks (`allowed_symlinks.txt`), `$`
  макросы, overlays/фильтры реализованы. Команды `/datapack
  enable|disable|list|create|reload` реализованы.
- Общий процент готовности Minecraft: полного согласованного перечня механик
  с равными весами не существует, поэтому он не вычисляется.

## Известные пробелы из документов (в этом проходе не перепроверялись)

- `/datapack enable|disable|list|create|reload` реализованы; внешние пакеты
  с диска загружаются. pack.mcmeta pack_format, symlinks, $ макросы,
  overlays/фильтры, runtime-реестры — все реализованы с оптимизацией для MSPT.
- Wither: полностью реализован (bossbar, три головы, ranged-атаки, фазы здоровья, защита спавна).
- Raid: полностью реализован (captain succession, save/load участников, bossbar, victory/loss, Hero of the Village).
- Worldgen/Flint: расхождение  above; полный `flint-all` красный из-за него.
- Старые миры: матрица миграций не полная.
