# Разбор файлов вне LLVM: проверенные границы ответственности

Источник отсутствия в LLVM: `5a52baf`, Headless 44664434286. Ниже прочитан весь текст каждого перечисленного файла и проверены указанные ссылки. Это **структурный разбор**, не полное соответствие механик vanilla. Ни один файл не удалён и не исключён из общего аудита.

| Файл `steel-core/src/…` | Что содержит и куда подключён | Что ещё проверять |
|---|---|---|
| `behavior/items/default.rs` | DefaultItemBehavior с пустым impl ItemBehavior; `behavior/item.rs` создаёт его в таблице обработчиков для предметов без специализации | Все унаследованные методы ItemBehavior, компоненты предметов и конкретные use-пути. Пустой impl не означает отсутствие поведения |
| `block_entity/block_state_nbt.rs` | Только reexport load/save из steel-registry; вызывается piston_moving, falling_block, primed_tnt, abstract_minecart | Общий codec, сохранение/загрузка каждого вызывающего типа; подключать дублирующий codec сюда не нужно |
| `chunk/simulation_ticket_manager.rs` | Alias ChunkTracker с пределом BLOCK_TICKING_CHUNK; реальный экземпляр хранит chunk_scheduler; тесты chunk_tracker/tests используют alias | Граф распространения, переходы уровней, отмена загрузки, ticking boundary в ChunkTracker/scheduler. Отсутствие строк alias в LLVM не является отсутствием simulation tracker |
| `command/brigadier/runtime.rs` | Trait associated types, типы callback, marker BrigadierRuntime; реализации runtime-алгоритма здесь нет | Graph/parser/executor и конкретный server runtime; типовые декларации сами по себе не доказывают корректность команд |
| `entity/item_frame.rs` | Trait ItemFrame с direction и analog_output, reexport из entity/mod.rs | Конкретные item_frame/glow_item_frame и comparator callers, вращение/пустой предмет/сохранение |
| `inventory/menu/kinds/basic_menu.rs` | BasicKind, уникальный DowncastType key, пустой impl MenuKind | Унаследованные MenuKind callbacks, создание меню, обработка кликов и sync; не мёртвый файл лишь из-за пустого impl |
| `permission/mod.rs` | Декларации дочерних модулей, reexports и cfg(test) | Реальные реализации context/groups/manager/subject; permission-модель Steel не имеет взаимно-однозначной пары классу vanilla |
| `poi/mod.rs` | Декларации poi_instance/poi_set/poi_storage и reexports | POI updates при set_block/load, поиск, occupation, сохранение; аудит самого mod.rs этого не покрывает |
| `world/player_index/mod.rs` | Декларации area/map и reexports PlayerAreaMap/PlayerMap | Обновление spatial index на move/teleport/disconnect, consumers видимости и chunk tracking |
| `loot/mod.rs` | Декларация модуля resolvers | Алгоритмы resolvers, loot context и реальные вызовы из мира/сущностей |

Остальные файлы вне LLVM не получают автоматически такую же классификацию. Особенно generated packet codecs и vendor/runtime требуют проверки исполняемых методов, а не решения по имени файла.
