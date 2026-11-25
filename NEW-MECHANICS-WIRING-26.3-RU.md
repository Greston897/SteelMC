# Новые механики: реализация, подключение и границы проверки

Это карта вызывающих путей, **не заявление о полном поведенческом паритете**. Существующий метод, аннотация фабрики, зелёный общий тест или отсутствие `todo!` не доказывают полноту механики. Общий инвентарь объектов и AI: [WIRING-INVENTORY-26.3-RU.md](WIRING-INVENTORY-26.3-RU.md), воспроизводится `.ci/wiring_inventory.py`.

## Найденные и исправленные неподключённые пути

| Механика | Файл реализации | Подключение |
|---|---|---|
| Преследование цели големом | `steel-core/src/entity/ai/goal/move_towards_target.rs` | `hostile/iron_golem.rs`: `new_with_base`, приоритет 2, скорость 0.9, дистанция 32 |
| Общение торговца с игроком | `steel-core/src/entity/ai/goal/interact.rs` | `passive/wandering_trader.rs`: `new_with_base`, приоритет 9, дистанция 3, вероятность 1 |
| Поиск жидкости дельфином | `steel-core/src/entity/ai/goal/try_find_liquid.rs` | `passive/dolphin.rs`: приоритет 0; `dolphin_tries_to_find` из `steel-registry/build_assets/vanilla_tags/fluid/dolphin_tries_to_find.json` |
| Внешние тела функций | `steel-core/src/server/datapacks.rs`: `from_root`, `reload`, `commands` | `Server::new_with_commands` → каталог `<save_path>/datapacks`; `/reload` → `reload_data_driven_state`; `/function` → `function_commands` |
| Теги load/tick | `steel-core/src/server/datapacks.rs`: `expand_tag`, `tick_functions` | `server/run_loop.rs`: `run_game_tick`, перед обработкой команд; load однократно после публикации, затем tick |
| Наследование контекста функции | `steel-core/src/command/builtins/function.rs`: `RunFunction` | Зарегистрированный `/function` → `parse_function_command` → `ExecutionControl::queue_function`; больше нет переподачи тела от имени Console |
| Локальный return функции | `steel-core/src/command/execution/queue.rs`: `queue_function` | Дочерний frame; общий лимит исполнения и очереди, изоляция return от продолжения вызывающей функции |

`hostile/…` и `passive/…` в таблице находятся внутри `steel-core/src/entity/entities/mobs/`.

Для торговца дополнительно убраны скопированные животные BreedGoal/FollowParentGoal; подключены AvoidEntityGoal для Zombie, Evoker, Vindicator, Vex, Pillager, Illusioner, Zoglin. Это **не** означает реализации его полного vanilla-набора целей: торговля, блуждание к точке и использование зелий требуют отдельной проверки/доработки.

## Рейды: проверенные по исходникам связи

| Механика | Реализация | Вызывающий путь |
|---|---|---|
| Тик рейда | `steel-core/src/world/raid.rs`: `RaidManager::tick` | `steel-core/src/server/run_loop.rs`: `world.raids.tick(world)` |
| Сохранение менеджера | `world/raid.rs`: `to_saved_data`, `from_saved_data` | `world/mod.rs`: загрузка/сохранение `saved_data_names::RAIDS` |
| Восстановление загруженного рейдера | `world/raid.rs`: `restore_raider` | `entity/mob/mod.rs`: `base_tick_mob`, до проверки NoAI; повторное восстановление не добавляет здоровье |
| Номер волны участника | `world/raid.rs`: `member_waves`, `member_save_data` | Создание волны/всадника, присоединение, восстановление, сохранение mob |
| Капитаны по волнам | `world/raid.rs`: `wave_leaders` | Spawn, restore, получение знамени, смерть/уход; старые сохранения мигрируются в `from_raids` |
| Получение знамени | `entity/ai/goal/obtain_raid_banner.rs`, `world/raid.rs`: `claim_captain_banner` | Цель приоритета 1 в шести конструкторах рейдеров; флаг MOVE, eligibility запрещает получение неспособным быть капитаном типам |
| Недостижимое знамя | `entity/ai/goal/obtain_raid_banner.rs`: `unreachable_until` | Неудачное построение достижимого пути; кеш на 600 world game ticks, не сохраняется |
| Зачёт убийцы | `world/raid.rs`: `raider_died` | `entity/living_entity.rs`: путь смерти |
| Удаление/смена мира без убийства | `world/raid.rs`: `raider_left_world` | `entity/callback.rs`: callback удаления; chunk unload не приравнен к убийству |
| Ломание двери vindicator | `entity/ai/goal/raid_break_door.rs` | `hostile/vindicator.rs`: цель приоритета 2; MOB_GRIEFING, сложность, членство рейда |
| Открывание двери vindicator | `entity/ai/goal/open_door.rs`: `for_raid` | `hostile/vindicator.rs`: цель приоритета 3; navigation can_open_doors |
| Празднование поражения деревни | `entity/ai/goal/raid_celebration.rs` | Конструкторы Pillager, Vindicator, Evoker, Witch, Ravager, Illusioner; `set_raid_celebrating` → synchronized entity data |

Пути без префикса в этой таблице начинаются с `steel-core/src/`.

**Исправленная связь:** получение знамени перенесено из прямого вызова после селекторов в отдельный ObtainRaidBannerGoal с MOVE и проверкой Path::can_reach. Карта капитанов по волнам сама по себе не доказывает все правила присоединения/очистки нескольких волн.

## Wither и черепы: связи и уточнённые отличия

| Механика | Реализация | Подключение |
|---|---|---|
| Фабрика Wither/черепа | `entity/entities/mobs/bosses/wither_boss.rs`, `entity/entities/objects/projectiles/wither_skull.rs` | Аннотации entity_behavior → `steel-core/build/entities.rs` → `entity/registry.rs::init_entities` |
| Центральная голова | `wither_boss.rs`: реализация RangedAttackMob | Зарегистрированный RangedAttackGoal |
| Боковые головы | `wither_boss.rs`: `tick_side_heads` | `server_ai_step` после mob AI; при потере существующей цели переназначение отложено до следующего срока |
| Точка прицеливания | `wither_boss.rs`: `fire_skull` callers | Y цели + половина eye height, не половина bounding box |
| Заряд появления | `wither_boss.rs`: `begin_spawn_phase`, `tick_spawn_phase` | Event finalize_spawn; создание из пирамиды черепов; чтение Invul не запускает новую зарядку |
| Регенерация | `wither_boss.rs`: spawn/combat ticks | `heal`, с учётом фактического max health |
| Сопротивление замедляющим блокам | `wither_boss.rs`: `Entity::make_stuck_in_block` | Виртуальный entity hook, как no-op vanilla Wither |
| Bossbar | `wither_boss.rs`, `entity/tracker/mod.rs` | Начало/конец tracking и обновления отслеживаемой сущности |
| Цикл черепа | `wither_skull.rs`: `Entity::tick` | Создаваемый WitherSkullEntity; больше не ThrowableProjectile tick |
| Удалённый владелец | `entity/reference.rs`, `entity/projectile/mod.rs`: `owner_was_removed` | Проверка черепа до обычного разрешения ссылки, очищающего cache |
| Отсутствующий чанк | `wither_skull.rs`: `Entity::tick` | Проверка доступного Full chunk перед перемещением |
| Отражение | `wither_skull.rs`: `Projectile::on_deflection` | Общий `hit_target_or_deflect_self`; сброс/уменьшение acceleration_power |
| Сохранение ускорения/blue variant | `wither_skull.rs`: save/load | Общая сериализация сущностей, acceleration_power и dangerous |
| Удар/взрыв/Wither II | `wither_skull.rs`: `Projectile::on_hit` | Общий raycast/hit dispatch → hurt/effect → explosion → discard |
| Post-attack enchantments | `wither_skull.rs`: `on_hit` | Успешный owned hit по выжившей цели → enchantment_helper |

Пути в этой таблице начинаются с `steel-core/src/`, кроме явно полного пути генератора.

## Проверка и незакрытый объём

В код добавлены проверки загрузки/повторной загрузки функций, сохранения предыдущего snapshot при ошибке, тегов load/tick, optional/required references и циклов тегов. Отдельная проверка command scheduler выполняет тело в дочернем frame, проверяет сохранение источника, return=42 и продолжение вызывающего frame. **Результат запуска следует брать из Actions для конкретного SHA, а не из наличия этих тестов.**

Не объявлено реализованным:

- Полный runtime datapack pipeline: ZIP, enable/disable и управление порядком, function macros, вызов произвольного function tag командой, загрузка рецептов/loot/advancements/worldgen и остальных динамических реестров.
- Полная транзакция `/reload` между функциями, permission groups и правилами всех миров.
- Полный набор AI торговца, дельфина и голема.
- Поведенческая дифференциальная проверка всех новых ветвей на официальном vanilla 26.3.
- Полный перечень **всех механик** Minecraft: 3105 входов генератора — объекты, а не количество механик; 37 общих целей — не весь AI.

Источники сверки: Java mirror `0khs/minecraft_source_26.3`, commit `74720c2890f22942b72d7d84958eff1b3f72b95b`; fluid tag `misode/mcmeta`, ref `26.3-data`, `data/minecraft/tags/fluid/dolphin_tries_to_find.json` (только `minecraft:water`, без flowing_water).

## Подтверждённые результаты для кода `b32beff`

Эта запись относится именно к указанному SHA. Последующее обновление этого Markdown или координат инвентаря не является новым прогоном кода.

| Проверка | Запуск Actions | Результат |
|---|---|---|
| Workspace tests | [44657024633](https://github.com/Greston897/SteelMC/actions/runs/44657024633) | SUCCESS: 4806 passed, 7 ignored; пропуски не засчитаны как PASS |
| Build Report | [44657024731](https://github.com/Greston897/SteelMC/actions/runs/44657024731) | SUCCESS |
| Flint Tests | [44657024657](https://github.com/Greston897/SteelMC/actions/runs/44657024657) | SUCCESS данного workflow, не автоматическое закрытие других Flint-наборов |
| Full Conformance | [44657024684](https://github.com/Greston897/SteelMC/actions/runs/44657024684) | SUCCESS существующего набора; название workflow не означает охват всех механик vanilla |
| Live Server Smoke | [44657024683](https://github.com/Greston897/SteelMC/actions/runs/44657024683) | SUCCESS всех трёх jobs: Steel, официальный vanilla, compare; настоящий запуск серверов, TCP/RCON, save/restart |
| Lint | [44657024670](https://github.com/Greston897/SteelMC/actions/runs/44657024670) | FAILURE; Spell Check завершился ошибкой. Не объявлено исправленным |
| Vanilla Evidence | [44657024628](https://github.com/Greston897/SteelMC/actions/runs/44657024628) | На момент фиксации QUEUED, результат не засчитан |

Live smoke не заменяет дифференциальные сценарии рейдов, AI и полного datapack pipeline. Эта работа не закрывает все пункты раздела «незакрытый объём» выше.
