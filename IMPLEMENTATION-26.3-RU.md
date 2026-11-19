# Реализация всего ядра — укрупнённая инженерная оценка

Это **оценка по коду, не процент vanilla-паритета и не доля прошедших тестов**. Охвачены все 15 основных подсистем, но только 60 явно перечисленных возможностей. Не выполнен аудит каждого метода и каждой механики Minecraft.

Шкала: **1** — основной исполняемый путь реализован; **0,5** — частично, известны ограничения либо полнота не установлена; **0** — возможность не засчитана. Вес каждого пункта одинаковый. Это условная инженерная шкала: 0,5 не означает, что измерена ровно половина исходников. Изменение списка или весов меняет итог. Не читать показатель как «осталось ровно столько-то работы».

| Подсистема | Баллы / 4 | Оценка по чеклисту |
|---|---:|---:|
| Инфраструктура | 3 / 4 | 75.0% |
| Мир и тики | 3.5 / 4 | 87.5% |
| Чанки и сохранение | 3.5 / 4 | 87.5% |
| Синхронизация и протокол | 3.5 / 4 | 87.5% |
| Реестры, теги, кодеки | 3 / 4 | 75.0% |
| Генерация мира | 3.5 / 4 | 87.5% |
| Блоки | 3 / 4 | 75.0% |
| Предметы | 3.5 / 4 | 87.5% |
| Block entities | 3 / 4 | 75.0% |
| Сущности и AI | 3 / 4 | 75.0% |
| Игрок | 3.5 / 4 | 87.5% |
| Инвентарь и меню | 3.5 / 4 | 87.5% |
| Физика, жидкости, снаряды | 3.5 / 4 | 87.5% |
| Команды и права | 3 / 4 | 75.0% |
| Loot, зачарования, прогрессия | 3.5 / 4 | 87.5% |
| **Итого** | **49.5 / 60** | **82.5%** |

**Граница достоверности:** наличие основного пути не доказывает отсутствие багов или всех исключений. Это воспроизводимый расчёт из ручных оценок `.ci/implementation_assessment.json`, а не автоматически измеренная готовность всего Minecraft. Точное общее значение по всем механикам неизвестно.

## Основания каждого балла

### Инфраструктура

- **1** — Запуск и конфигурация: `steel-core/src/bootstrap.rs`.
- **1** — Планировщик, домены и workers: `steel-core/src/server/schedule.rs`.
- **1** — RCON и администрирование: `steel-core/src/server/rcon.rs`.
- **0** — Полный runtime reload внешних vanilla datapacks: `steel-core/src/command/builtins/datapack.rs`.

### Мир и тики

- **1** — Игровой цикл и погода: `steel-core/src/world/weather.rs`.
- **1** — Часы и временные шкалы: `steel-core/src/world/clock.rs`.
- **1** — Симуляция и scheduled ticks: `steel-core/src/world/tick_scheduler`.
- **0.5** — Все пограничные случаи переходов между мирами: `steel-core/src/server/world_changes.rs`.

### Чанки и сохранение

- **1** — Загрузка, tickets и статусы: `steel-core/src/chunk/chunk_map`.
- **1** — Region-хранилище и NBT: `steel-core/src/chunk_saver`.
- **1** — Свет и распространение изменений: `steel-core/src/chunk/light`.
- **0.5** — Миграция произвольных старых миров: `steel-core/src/chunk_saver/format.rs`.

### Синхронизация и протокол

- **1** — Configuration и registry cache: `steel-core/src/server/registry_cache.rs`.
- **1** — Игрок и соединение: `steel-core/src/player/connection`.
- **1** — Отслеживание и обновления сущностей: `steel-core/src/world/world_entities.rs`.
- **0.5** — Полная перезагрузка datapack-состояния онлайн: `steel-core/src/server/registry_cache.rs`.

### Реестры, теги, кодеки

- **1** — Статические и динамические записи: `steel-registry/src`.
- **1** — Теги и ID: `steel-core/src/server/registry_cache.rs`.
- **0.5** — Полностью типизированные кодеки всех данных: `steel-utils/src/nbt/json.rs`.
- **0.5** — Стек datapack/reload/provenance: `steel-registry/src`.

### Генерация мира

- **1** — Биомы и noise: `steel-worldgen/src`.
- **1** — Terrain и carvers: `steel-core/src/worldgen/stages`.
- **1** — Structure starts, references и размещение: `steel-core/src/worldgen/structure`.
- **0.5** — Features без известных расхождений: `steel-core/src/worldgen/feature`.

### Блоки

- **1** — Размещение и состояния: `steel-core/src/behavior/block`.
- **1** — Формы и соседние обновления: `steel-core/src/world/block_updates`.
- **0.5** — Все механизмы redstone: `steel-core/src/behavior/blocks/redstone`.
- **0.5** — Все специальные взаимодействия: `steel-core/src/behavior/blocks`.

### Предметы

- **1** — Использование и размещение: `steel-core/src/behavior/items`.
- **1** — Компоненты и ItemStack: `steel-registry/src/item_stack.rs`.
- **1** — Базовые боевые действия: `steel-core/src/player/game_mode`.
- **0.5** — Все предметные исключения и взаимодействия: `steel-core/src/behavior/items`.

### Block entities

- **1** — Фабрики типов: `steel-core/src/block_entity/registry.rs`.
- **1** — Загрузка и сохранение NBT: `steel-core/src/block_entity`.
- **0.5** — Все runtime-механизмы: `steel-core/src/block_entity/entities`.
- **0.5** — Полнота всех специальных контейнеров: `steel-core/src/behavior/blocks/container`.

### Сущности и AI

- **1** — Создание, загрузка и жизненный цикл: `steel-core/src/entity/spawn.rs`.
- **1** — Синхронизированные данные и атрибуты: `steel-core/src/entity/attribute.rs`.
- **0.5** — Навигация и цели во всех ситуациях: `steel-core/src/entity/ai`.
- **0.5** — Полное видовое поведение всех мобов: `steel-core/src/entity/entities/mobs`.

### Игрок

- **1** — Вход и жизненный цикл: `steel-core/src/server/player_lifecycle.rs`.
- **1** — Движение и состояния: `steel-core/src/player/movement`.
- **1** — Игровые действия и взаимодействие: `steel-core/src/player/game_mode`.
- **0.5** — Полнота всех advancement-предикатов: `steel-core/src/player/advancement`.

### Инвентарь и меню

- **1** — Контейнеры и слоты: `steel-core/src/inventory/container`.
- **1** — Клики и переносы: `steel-core/src/inventory/click.rs`.
- **1** — Рецепты и крафт: `steel-core/src/inventory/recipe_manager.rs`.
- **0.5** — Все транзакционные edge cases: `steel-core/src/inventory/menu`.

### Физика, жидкости, снаряды

- **1** — Коллизии и движение: `steel-core/src/physics`.
- **1** — Распространение жидкостей: `steel-core/src/fluid`.
- **1** — Снаряды и столкновения: `steel-core/src/entity/entities/objects/projectiles`.
- **0.5** — Все межсистемные edge cases: `steel-core/src/physics`.

### Команды и права

- **1** — Парсер и дерево команд: `steel-core/src/command/brigadier`.
- **1** — Permissions и command sources: `steel-core/src/permission`.
- **1** — Встроенные команды: `steel-core/src/command/builtins`.
- **0** — Vanilla /datapack enable и disable: `steel-core/src/command/builtins/datapack.rs`.

### Loot, зачарования, прогрессия

- **1** — Вычисление loot tables: `steel-core/src/loot`.
- **1** — Зачарования и эффекты: `steel-core/src/enchantment_helper`.
- **1** — Разблокировка рецептов: `steel-core/src/inventory/recipe_manager.rs`.
- **0.5** — Все сложные цепочки прогрессии и торговли: `steel-core/src/entity/entities/mobs`.

## Открытые ошибки отдельно от оценки

- Overworld FEATURES: известное расхождение хешей не исправлено.
- Полный Flint остаётся красным; исправление отдельных механик не закрывает весь набор.
- Для новых исправлений окончательные расширенные результаты нужно сверять с Actions.
- Оценка не даёт гарантии TPS, производительности, сетевой совместимости или отсутствия потери данных.

