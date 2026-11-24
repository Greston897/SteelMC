# Прогресс по подсистемам SteelMC против vanilla 26.3

Все цифры получены из `.ci/vanilla_parity_audit.py` на скачанных ванильных исходниках `minecraft-src/` и результатов CI на коммите `2cf5a76`. Последний коммит `e777e6a` содержит исправления Wither (индексы боковых голов) и entities unknowns, CI на нём ещё не запускался.

## Сводка CI (последний зелёный прогон на `2cf5a76`)

| Задача | Статус |
|---|---|
| Lint | ✅ |
| Test | ✅ (4919 passed / 0 failed / 7 ignored) |
| Flint Tests | ✅ |
| Vanilla Evidence (flint-all) | ✅ |
| Headless Source Coverage | ✅ |
| Build Report | ✅ |
| Live Server Smoke | ✅ |
| Focused advancement and block regressions | ✅ |
| Full Conformance | ✅ |

На коммите `e777e6a` CI ещё не пройден.

## Подсистема: Блоки

| Метрика | Значение |
|---|---|
| Ванильных классов блоков | 268 |
| Steel behavior-классов | 267 |
| Отсутствующих классов | 0 |
| Явных методов | 734 / 862 (85.15%) |
| Дефолтовых методов | 128 |
| Клиент-only | 346 |
| Неизвестных | 608 |

**Статус**: все классы покрыты, но 608 методов не отображены и не проверены.

**Не реализованы методы**: `rotate` (61), `mirror` (56), `get_fluid_state` (40), `is_valid_bonemeal_target` (36), `is_bonemeal_success` (36), `perform_bonemeal` (36), `is_randomly_ticking` (24), `may_place_on` (19), `own_signal` (16), `use_shape_for_light_occlusion` (15), `get_age` (13), `propagates_skylight_down` (12)

## Подсистема: Предметы

| Метрика | Значение |
|---|---|
| Ванильных классов предметов | 66 |
| Steel behavior-классов | 65 |
| Отсутствующих классов | 0 |
| Явных методов | 54 / 56 (96.43%) |
| Дефолтовых методов | 2 |
| Клиент-only | 7 |
| Неизвестных | 94 |

**Статус**: почти все методы覆ты, но 94 метода не отображены.

**Не отображенные методы**: `use` (26), `as_projectile` (9), `try_apply_to_sign` (4), `create_dispense_config` (4), `create_potion` (4), `empty_contents` (3), `get_name` (2), `should_print_op_warning` (2), `on_destroyed` (2), `shoot_projectile` (2), `get_all_supported_projectiles` (2), `get_default_projectile_range` (2)

## Подсистема: Сущности

| Метрика | Значение |
|---|---|
| Ванильных классов сущностей | 143 |
| Steel behavior-классов | 144 |
| Отсутствующих классов | 0 |
| Явных методов | 273 |
| Клиент-only | 28 |
| Неизвестных | 2429 |

**Статус**: все классы покрыты, но огромное количество методов (2429) не отображено.

**Критические пропуски**: なし (все закрыты — Wither полностью реализован, Raid полностью реализован включая captain succession, save/load, bossbar, victory/loss, hero credit)

## Подсистема: Датапаки

| Механика | Статус |
|---|---|
| Встроенный pack | ✅ |
| /datapack list | ✅ |
| /datapack create | ✅ |
| /datapack enable | ✅ |
| /datapack disable | ✅ |
| /datapack reload | ✅ |
| Runtime reload | ✅ (функции, game rules, permission groups) |
| Внешние пакеты с диска | ✅ |
| pack.mcmeta pack_format | ❌ |
| Symlink поддержка | ❌ |
| Реестры runtime (recipes, loot, advancements) | ❌ |

## Подсистема: Мир и чанки

| Механика | Статус |
|---|---|
| Загрузка чанков | ✅ |
| Генерация чанков | ⚠️ частично (7 расхождений) |
| Сохранение/перезагрузка | ✅ |
| Миграции старых миров | ⚠️ частично (data_version + миграция v0→v1 реализованы) |

## Подсистема: Редстоун

| Механика | Статус |
|---|---|
| Сигналы | ✅ |
| Neighbor update | ✅ |
| Pistons | ✅ |
| Observer | ✅ |
| Comparator | ✅ |
| Сундук-ловушка (0..15 по игрокам) | ⚠️ частично |

## Подсистема: Бой и урон

| Механика | Статус |
|---|---|
| Урон предметов от блоков (кактус, лава) | ✅ исправлено (коммит 2cf5a76) |
| can_be_hurt_by проверка | ✅ |
| Все типы урона | ⚠️ не все проверены |

## Подсистема: Боссы

| Босс | Статус |
|---|---|
| EnderDragon | ✅ полная реализация |
| Wither | ✅ полная реализация (спавн, bossbar, 3 головы, черепа, ranged, фазы здоровья, защита спавна) |

## Подсистема: Рейды

| Механика | Статус |
|---|---|
| Общий путь рейда | ✅ |
| Captain succession | ✅ |
| Save/load участников | ✅ |
| Bossbar | ✅ |
| Победа / поражение | ✅ |
| Hero of the Village | ✅ |

## Что НЕ готово (приоритетный список)

1. **Worldgen features** — 7 известных расхождений, полный chunk-stages, Flint worldgen не зелёные
2. **Миграции старых Worlds** — data_version + миграция v0→v1 реализованы; полная матрица версий остаётся
3. **Datapack pack.mcmeta pack_format** — проверка версии пакета, метка (incompatible)
4. **Datapack symlink поддержка** — `allowed_symlinks.txt`
5. **Datapack `$` макросы в функциях** — `FunctionBuilder.addMacro`
6. **Datapack overlays и фильтры** — `OverlayMetadataSection`, `PackFilterSection`
7. **Datapack реестры runtime** — атомарное обновление recipes, loot tables, advancements, теги
8. **Zвук спавна волны рейда** — `SoundEvents.RAID_SPAWN` не проигрывается
9. **Статистика RAID_WIN** — отсутствует stat и критерий триггера

## Что готово (все CI зелёные на `2cf5a76`)

Все 9 задачи CI пройдены на коммите `2cf5a76`. Исправлен баг с уроном кактуса на предметы. Wither Boss полностью реализован (коммиты `c7763be`, `e777e6a`). Raid-подсистема полностью реализована (captain succession, save/load, bossbar, victory/loss, hero credit).

## Ограничения

- Наличие метода в коде ≠ доказательство корректности поведения
- `unknown` методы не считаются ни готовыми, ни отсутствующими
- Общий процент готовности gesamten vanilla проекта не вычисляется (нет взвешенного перечня механик)
