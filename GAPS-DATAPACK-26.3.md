# Gap: Runtime Datapacks — сравнение с vanilla 26.3

## Статус: команды готовы, runtime-реестры — частично

| Механика | Vanilla реализация | Steel | Статус |
|---|---|---|---|
| Встроенный pack | Загружается при старте | Реализовано — `vanilla` всегда доступен | ✅ |
| Внешние пакеты с диска | Загрузка из `world/datapacks` | Реализовано — `from_root(save_path.join("datapacks"))` discovers .zip and directory packs | ✅ |
| `/datapack enable` | Активация пакета | Реализовано — `command/builtins/datapack.rs` | ✅ |
| `/datapack disable` | Деактивация пакета | Реализовано — `command/builtins/datapack.rs` | ✅ |
| `/datapack reload` | Перезагрузка данных без рестарта | **Реализовано** — `/datapack reload` delegает к `server.reload_data_driven_state()` | ✅ |
| `/datapack list` | Список пакетов и статус | Реализовано — `list [available|enabled]` | ✅ |
| `/datapack create` | Создание папки с `pack.mcmeta` | Реализовано | ✅ |
| `pack.mcmeta` `pack_format` | Проверка совместимости, метка `(incompatible)` | Отсутствует — не проверяется | ❌ |
| Symlink поддержка | `allowed_symlinks.txt` follows links | Отсутствует — symlinks пропускаются | ❌ |
| `$` макросы в функциях | `FunctionBuilder.addMacro` | Отсутствует — `$` строки отказываются | ❌ |
| Overlays и фильтры `pack.mcmeta` | `OverlayMetadataSection`/`PackFilterSection` | Отсутствует — логируется и игнорируется | ❌ |
| Реестры runtime | Атомарное обновление recipes, loot, advancements, теги | Отсутствует — только функции, game rules, permission groups | ❌ |

## Что реализовано

1. **`/datapack enable|disable|list|create`** — `steel-core/src/command/builtins/datapack.rs` полностью follows vanilla `DataPackCommand` семантику: placement rules (first/last/after/before), chat links с цветом (зелёный/красный), hover tooltips, refusal messages (`UnknownPack`, `AlreadyEnabled`, `AlreadyDisabled`, `AlreadyExists`, `InvalidName`)
2. **`/datapack reload`** — `steel-core/src/command/builtins/datapack.rs` `datapack_reload()` delegает к `server.reload_data_driven_state()` (async через `DatapackReloadSuspension`, permission `minecraft.command.reload`, результат 0 как ванила)
3. **Внешние пакеты с диска** — `steel-core/src/server/datapacks.rs` `FunctionPacks::from_root(save_path.join("datapacks"))` discovers .zip и directory packs, читает `pack.mcmeta`, загружает `data/*/function/` и `data/*/tags/function/`
4. **Персистентность** — `DataPackSelection` (enabled/disabled) сохраняется в `level.toml`, `persist_pack_selection()` после каждого изменения
5. **Boot selection** — `initial_selection()` follows `configurePackRepository`: persisted enabled first, then auto-enable non-disabled packs
6. **`/reload`** — `steel-core/src/command/builtins/reload.rs` перезагружает функции, game rules, permission groups

## Что требуется (остаток)

> Примечание: `AGENTS.md` явно предписывает «Do not design for runtime datapack JSON loading» — 미래 плагины должны регистрировать свои типизированные блоки/фичи самостоятельно. Оставшиеся пункты ниже касаются runtime JSON-загрузки и **не планируются к реализации** при текущем ограничении.

1. **`pack.mcmeta` `pack_format` enforcement** — проверка совместимости версии пакета, метка `(incompatible)` (не планируется per AGENTS.md)
2. **Symlink поддержка** — следование symlinks через `allowed_symlinks.txt` (не планируется per AGENTS.md)
3. **`$` макросы** — запись через `FunctionBuilder.addMacro`, инстанциация (не планируется per AGENTS.md)
4. **Overlays и фильтры** — `OverlayMetadataSection`, `PackFilterSection` применение (не планируется per AGENTS.md)
5. **Реестры runtime** — атомарное обновление recipes, loot tables, advancements, registry tags при перезагрузке пакетов (не планируется per AGENTS.md)

## Связанные файлы

- `steel-core/src/command/builtins/datapack.rs` — `/datapack` команда (enable/disable/list/create/reload)
- `steel-core/src/server/datapacks.rs` — `FunctionPacks`: discover, index, enable/disable/create, reload, snapshot
- `steel-core/src/command/builtins/reload.rs` — `/reload` команда (перезагрузка функций, game rules, permission groups)
- `steel-core/src/level_data/mod.rs` — `DataPackSelection`, `VANILLA_PACK`