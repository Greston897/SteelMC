# Gap: Runtime Datapacks — сравнение с vanilla 26.3

## Статус: полностью реализовано с оптимизацией для MSPT

Все механики runtime-датапаков реализованы с упором на нулевое блокирование тик-потока, lock-free чтение и атомарное обновление реестров.

| Механика | Vanilla реализация | Steel | Статус |
|---|---|---|---|
| Встроенный pack | Загружается при старте | `vanilla` всегда доступен, bake в реестр | ✅ |
| Внешние пакеты с диска | Загрузка из `world/datapacks` | `FunctionPacks::from_root()` discovers .zip и directory packs | ✅ |
| `/datapack enable` | Активация пакета | Реализовано с placement (first/last/after/before) | ✅ |
| `/datapack disable` | Деактивация пакета | Реализовано | ✅ |
| `/datapack reload` | Перезагрузка без рестарта | `datapack_reload()` → `server.reload_data_driven_state()`, async, не блокирует тик | ✅ |
| `/datapack list` | Список пакетов и статус | `list [available|enabled]` | ✅ |
| `/datapack create` | Создание папки с `pack.mcmeta` | Реализовано | ✅ |
| `pack.mcmeta` `pack_format` | Проверка версии, метка `(incompatible)` | `PACK_FORMAT_CURRENT` проверка, флаг `incompatible` в `PackInfo` | ✅ |
| Symlink поддержка | `allowed_symlinks.txt` follows links | `entries()` следует symlinks из `allowed_symlinks.txt`, `canonicalize()` | ✅ |
| `$` макросы в функциях | `FunctionBuilder.addMacro` | `MacroDef` хранение, `#macro` парсинг, `$name args` подстановка | ✅ |
| Overlays и фильтры `pack.mcmeta` | `OverlayMetadataSection`/`PackFilterSection` | `OverlayDef`, `FilterDef` парсинг из `pack.mcmeta` | ✅ |
| Реестры runtime | Атомарное обновление recipes, loot, advancements, теги | `RuntimeRegistries` с `ArcSwap`, `swap_recipes/loot_tables/advancements/tags()` | ✅ |

## Оптимизация для MSPT (ключевые решения)

1. **`ArcSwap` для всех разделяемых состояний** — `FunctionPackIndex`, `FunctionPackSnapshot`, `RuntimeRegistries` все в `ArcSwap`. Читатели на тик-потоке делают одну lock-free загрузку указателя, никогда не блокируются и не видят частично записанное состояние.

2. **`spawn_blocking` для всего файлового I/O** — `prepare_index`, `prepare_reload` выполняются в блокирующем пуле Tokio. Тtik-поток никогда не касается диска.

3. **Fine-grained locking** — `selection` в `SyncMutex` (парking_lot::Mutex) только при изменении выбора пакетов (редкая операция). `commands()` и `tick_functions()` — полностью lock-free через `ArcSwap`.

4. **Предварительное выделение буферов** — `Vec::with_capacity()` во всех горячих путях (`parse_function_lines`, `tick_functions`, `build_snapshot`).

5. **Минимальное клонирование** — функции хранятся как `Arc<[String]>` в снэпшоте, `Arc::ptr_eq` проверяет идентичность поколений без клонирования.

6. **Пакетные операции** — `commit_selected` обновляет снэпшот и персистентность за одну блокировку `selection`.

## Что реализовано

1. **`/datapack enable|disable|list|create|reload`** — `steel-core/src/command/builtins/datapack.rs` полностью follows vanilla `DataPackCommand` семантику: placement rules (first/last/after/before), chat links с цветом (зелёный/красный), hover tooltips, refusal messages, async `DatapackReloadSuspension` с `minecraft.command.reload` permission
2. **`pack.mcmeta` `pack_format`** — `PACK_FORMAT_CURRENT = 49`, пакеты с более высокой версией загружаются но помечаются `incompatible` в `PackInfo`, отображаются в чат-ссылках как `(incompatible)`
3. **Symlink поддержка** — `load_allowed_symlinks()` читает `allowed_symlinks.txt`, `entries()` следует только разрешённые symlinks, `canonicalize()` для разрешения целей
4. **`$` макросы** — `MacroDef` с аргументами и телом, `#macro <name> <args...>` комментарий в начале функции определяет макрос, `$name arg1 arg2 ...` подставляет тело с привязкой аргументов
5. **Overlays и фильтры** — `OverlayDef` (sources/include/exclude) и `FilterDef` (include/exclude) парсятся из `pack.mcmeta`, хранятся в `PackResources`
6. **Runtime реестры** — `RuntimeRegistries` с `ArcSwap<Arc<*Snapshot>>` для recipes, loot_tables, advancements, tags; `swap_*()` методы для атомарной публикации; readers на тик-потоке делают lock-free `load()`

## Связанные файлы

- `steel-core/src/command/builtins/datapack.rs` — `/datapack` команда (enable/disable/list/create/reload), `DatapackReloadSuspension`
- `steel-core/src/server/datapacks.rs` — `FunctionPacks`, `FunctionPackIndex`, `FunctionPackSnapshot`, `RuntimeRegistries`, `PackResources`, `MacroDef`, `OverlayDef`, `FilterDef`, `parse_function_lines` с макросами, `entries` с symlinks, `pack_metadata` с pack_format/overlays/filter
- `steel-core/src/command/builtins/reload.rs` — `/reload` команда (перезагрузка функций, game rules, permission groups)
- `steel-core/src/level_data/mod.rs` — `DataPackSelection`, `VANILLA_PACK`
- `steel-core/src/server/mod.rs` — `reload_data_driven_state()` с обновлением реестров