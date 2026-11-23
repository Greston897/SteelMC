# Измерение проверенных сценариев Minecraft 26.3

**Общий процент готовности всего vanilla-ядра: неизвестен.** Полного независимого перечня поведения и исчерпывающих проверок нет.

Рабочий HEAD: `ba05896e9be39d886c73777df1e653b65de772e0`; незакоммиченные изменения: `True`. Изменённый checkout не равен проверенному CI коммиту.

## Парные проверки серверов

### Автоматический live smoke

38 выбранных проверок запуска, RCON, мира и рестарта; без реальных клиентов; это не перечень всех возможностей vanilla.
Всего: **38**; совпали проверки: **38**; Steel не прошёл: **0**; эталон не прошёл: **0**; неизвестно: **0**.
Доля совпавших проверок в этом перечне: **100.0%**; не подтверждено для текущего рабочего состояния.
Доказательство: [d8810e66bc078e2d78df043ab802a64cac878d77](https://github.com/Greston897/SteelMC/actions/runs/45082985134); проверка метаданных CI: `metadata_verified`. Метаданные CI не подтверждают подлинность произвольно предоставленного файла результатов.

| Сценарий | Результат |
|---|---|
| `boot_real_server` | matched_assertions |
| `tcp_status_and_ping_without_players` | matched_assertions |
| `rcon_rejects_wrong_password` | matched_assertions |
| `forceload_loads_chunk_without_players` | matched_assertions |
| `setblock_and_execute_read_world` | matched_assertions |
| `clone_copies_live_block` | matched_assertions |
| `item_replace_and_container_predicate` | matched_assertions |
| `redstone_neighbor_update_lights_lamp` | matched_assertions |
| `scheduled_tick_unlights_lamp` | matched_assertions |
| `hopper_moves_items_on_server_ticks_without_players` | matched_assertions |
| `container_wildcard_matches` | matched_assertions |
| `container_negative_predicate` | matched_assertions |
| `falling_sand_lands_on_solid_block` | matched_assertions |
| `piston_extends_and_pushes` | matched_assertions |
| `piston_head_created` | matched_assertions |
| `piston_retracts` | matched_assertions |
| `normal_piston_does_not_pull` | matched_assertions |
| `furnace_smelts_on_real_ticks` | matched_assertions |
| `powered_hopper_is_disabled` | matched_assertions |
| `powered_hopper_does_not_transfer_over_ticks` | matched_assertions |
| `locked_hopper_retains_source` | matched_assertions |
| `unpowered_hopper_resumes_transfer` | matched_assertions |
| `comparator_reads_filled_container` | matched_assertions |
| `inventory_write_updates_comparator` | matched_assertions |
| `comparator_relights_after_item_write` | matched_assertions |
| `summoned_entity_is_queryable_without_players` | matched_assertions |
| `live_damage_changes_pig_health_to_six` | matched_assertions |
| `instant_health_restores_live_entity` | matched_assertions |
| `dead_entity_is_not_selectable` | matched_assertions |
| `graceful_stop_saves_world` | matched_assertions |
| `restart_same_disk_world` | matched_assertions |
| `scoreboard_survives_restart` | matched_assertions |
| `chunk_blocks_survive_restart` | matched_assertions |
| `container_items_survive_restart` | matched_assertions |
| `transferred_items_survive_restart` | matched_assertions |
| `furnace_output_survives_restart` | matched_assertions |
| `falling_block_result_survives_restart` | matched_assertions |
| `second_clean_shutdown` | matched_assertions |

### Ручной перечень

Ручные сценарии, не полный и не независимый перечень всех vanilla-механик.
Всего: **107**; совпали проверки: **0**; Steel не прошёл: **0**; эталон не прошёл: **0**; неизвестно: **107**.
Доля совпавших проверок в этом перечне: **0.0%**; не подтверждено для текущего рабочего состояния.

| Подсистема | Совпало / всего | Неизвестно |
|---|---:|---:|
| AI | 0 / 24 | 24 |
| Worldgen | 0 / 3 | 3 |
| Блоки | 0 / 14 | 14 |
| Бой | 0 / 6 | 6 |
| Боссы | 0 / 2 | 2 |
| Данные | 0 / 4 | 4 |
| Инвентарь | 0 / 4 | 4 |
| Инфраструктура | 0 / 5 | 5 |
| Команды | 0 / 4 | 4 |
| Мир | 0 / 4 | 4 |
| Предметы | 0 / 8 | 8 |
| Протокол | 0 / 7 | 7 |
| Редстоун | 0 / 6 | 6 |
| Рейды | 0 / 8 | 8 |
| Физика | 0 / 4 | 4 |
| Чанки | 0 / 4 | 4 |

| ID | Подсистема / сценарий | Результат |
|---|---|---|
| `SYS01` | Инфраструктура: Чистый запуск | unknown |
| `SYS02` | Инфраструктура: Ошибочный конфиг | unknown |
| `SYS03` | Инфраструктура: Права оператора | unknown |
| `SYS04` | Инфраструктура: RCON | unknown |
| `SYS05` | Инфраструктура: Остановка во время записи | unknown |
| `NET01` | Протокол: Вход и отказ сессии | unknown |
| `NET02` | Протокол: Configuration и реестры | unknown |
| `NET03` | Протокол: Resource pack | unknown |
| `NET04` | Протокол: Чат и подписи | unknown |
| `NET05` | Протокол: Tracking сущностей | unknown |
| `NET06` | Протокол: Предсказание блоков | unknown |
| `NET07` | Протокол: Интерфейсы | unknown |
| `WRL01` | Мир: День, часы и сон | unknown |
| `WRL02` | Мир: Дождь, снег, замерзание | unknown |
| `WRL03` | Мир: Границы мира | unknown |
| `WRL04` | Мир: Scheduled ticks | unknown |
| `CHK01` | Чанки: Tickets и simulation distance | unknown |
| `CHK02` | Чанки: Свет через границу | unknown |
| `CHK03` | Чанки: Сохранение block entities | unknown |
| `CHK04` | Чанки: Импорт старого мира | unknown |
| `GEN01` | Worldgen: Рельеф и биомы | unknown |
| `GEN02` | Worldgen: Структуры | unknown |
| `GEN03` | Worldgen: Features и декорация | unknown |
| `PHY01` | Физика: Перемещение и столкновения | unknown |
| `PHY02` | Физика: Вода, плавание и пузырьки | unknown |
| `PHY03` | Физика: Elytra и riptide | unknown |
| `PHY04` | Физика: Порталы и respawn | unknown |
| `INV01` | Инвентарь: Клики и перенос | unknown |
| `INV02` | Инвентарь: Крафт и книга рецептов | unknown |
| `INV03` | Инвентарь: Смерть и keepInventory | unknown |
| `INV04` | Инвентарь: Bundle и вложенные компоненты | unknown |
| `COM01` | Бой: Ближний бой | unknown |
| `COM02` | Бой: Щит и топор | unknown |
| `COM03` | Бой: Броня, эффекты и тотем | unknown |
| `COM04` | Бой: Снаряды и атрибуция | unknown |
| `COM05` | Бой: Зелья и облако | unknown |
| `COM06` | Бой: Голод и регенерация | unknown |
| `RED01` | Редстоун: Пыль и соседние обновления | unknown |
| `RED02` | Редстоун: Повторитель и компаратор | unknown |
| `RED03` | Редстоун: Поршни | unknown |
| `RED04` | Редстоун: Observer и квазисвязность | unknown |
| `RED05` | Редстоун: Рельсы и транспорт | unknown |
| `RED06` | Редстоун: Скалк и вибрации | unknown |
| `BLK01` | Блоки: Печь, blast furnace, smoker | unknown |
| `BLK02` | Блоки: Воронка и dropper | unknown |
| `BLK03` | Блоки: Dispenser | unknown |
| `BLK04` | Блоки: Crafter | unknown |
| `BLK05` | Блоки: Brewing stand | unknown |
| `BLK06` | Блоки: Anvil и enchanting | unknown |
| `BLK07` | Блоки: Beacon и conduit | unknown |
| `BLK08` | Блоки: Книги, таблички, lectern | unknown |
| `BLK09` | Блоки: Jukebox и note block | unknown |
| `BLK10` | Блоки: Улей и пчёлы | unknown |
| `BLK11` | Блоки: Растения и bonemeal | unknown |
| `BLK12` | Блоки: Waterlogging и соседние состояния | unknown |
| `BLK13` | Блоки: Огонь, лёд и снег | unknown |
| `BLK14` | Блоки: Trial spawner и vault | unknown |
| `MOB01` | AI: Выбор цели и поиск пути | unknown |
| `MOB02` | AI: Размножение и приручение | unknown |
| `MOB03` | AI: Житель и торговля | unknown |
| `MOB04` | AI: Зомбификация и лечение жителя | unknown |
| `MOB05` | AI: Враждебные стрелки | unknown |
| `MOB06` | AI: Взрывающиеся мобы | unknown |
| `MOB07` | AI: Piglin и бартер | unknown |
| `MOB08` | AI: Enderman и shulker | unknown |
| `MOB09` | AI: Warden и creaking | unknown |
| `MOB10` | AI: Водные мобы | unknown |
| `MOB11` | AI: Лошади и транспортные животные | unknown |
| `MOB12` | AI: Подбор и обработка предметов | unknown |
| `RAID01` | Рейды: Omen и центр рейда | unknown |
| `RAID02` | Рейды: Полные волны | unknown |
| `RAID03` | Рейды: Спавн и horn | unknown |
| `RAID04` | Рейды: Сторонние участники и уход | unknown |
| `RAID05` | Рейды: Смена капитана | unknown |
| `RAID06` | Рейды: Сохранение участников | unknown |
| `RAID07` | Рейды: Победа, поражение и герой | unknown |
| `RAID08` | Рейды: Специальные goals | unknown |
| `BOSS01` | Боссы: Wither: создание и бой | unknown |
| `BOSS02` | Боссы: Ender dragon | unknown |
| `CMD01` | Команды: Селекторы и execute | unknown |
| `CMD02` | Команды: Data и item | unknown |
| `CMD03` | Команды: Scoreboard и teams | unknown |
| `CMD04` | Команды: Functions и schedule | unknown |
| `DATA01` | Данные: Runtime datapacks | unknown |
| `DATA02` | Данные: Loot и зачарования | unknown |
| `DATA03` | Данные: Advancements и статистика | unknown |
| `DATA04` | Данные: Карты и explorer maps | unknown |
| `ITEM01` | Предметы: Рыбалка | unknown |
| `ITEM02` | Предметы: Археология и кисть | unknown |
| `ITEM03` | Предметы: Поводок и name tag | unknown |
| `ITEM04` | Предметы: Shears, axe, shovel и hoe | unknown |
| `ITEM05` | Предметы: Mace и wind charge | unknown |
| `ITEM06` | Предметы: Ender pearl и eye of ender | unknown |
| `ITEM07` | Предметы: Пиротехника | unknown |
| `ITEM08` | Предметы: Вёдра и бутылки | unknown |
| `MOB13` | AI: Кошки, оцелоты и волки | unknown |
| `MOB14` | AI: Овцы, коровы, mooshroom и chickens | unknown |
| `MOB15` | AI: Goat, armadillo и sniffer | unknown |
| `MOB16` | AI: Frog, slime и magma cube | unknown |
| `MOB17` | AI: Panda, parrot и rabbit | unknown |
| `MOB18` | AI: Turtle и яйца | unknown |
| `MOB19` | AI: Fish schooling и pufferfish | unknown |
| `MOB20` | AI: Llama и trader llama | unknown |
| `MOB21` | AI: Sulfur cube | unknown |
| `MOB22` | AI: Camel husk и undead horses | unknown |
| `MOB23` | AI: Blaze, breeze и snow/iron golems | unknown |
| `MOB24` | AI: Vex, evoker и illusioner | unknown |

## Парный datapack-прогон

Сценариев: **5**; совпали условия: **2**; не прошли на Steel: **1**; заблокированы на Steel: **2**; невалидный/заблокированный эталон: **0**.
[ba05896e9be39d886c73777df1e653b65de772e0](https://github.com/Greston897/SteelMC/actions/runs/45090490433): общий результат CI `failure`, метаданные CI `metadata_verified`, чистый checkout этого SHA: `False`.
Каталожные функции через RCON, без подключённых клиентов; не полный datapack.

| Сценарий | Результат |
|---|---|
| `startup_directory_function` | matched_assertions |
| `reload_modified_function` | matched_assertions |
| `disable_stops_function` | steel_failed |
| `enable_restores_function` | blocked_steel |
| `disabled_selection_survives_restart` | blocked_steel |

## Инвентаризация advancement triggers

Vanilla зарегистрировала **58** типов; в проверенном статическом gate Steel присутствуют **6**, остальные **52** в этом gate не разрешены.
Это инвентаризация исходников, не проверка работы, предикатов или процента паритета; `impossible` специально не является обычным событием. Другая проводка не исключена.
Разрешённые gate: `slept_in_bed`, `changed_dimension`, `placed_block`, `consume_item`, `nether_travel`, `item_used_on_block`.
SHA-256 vanilla-источника: `df23e839c3d458cb89a6e8a7c2cc4ad35357f898c2d59928145b8ad7c8d2bf82`; проверен на этом запуске: `True`. Без файла используется сохранённая инвентаризация, а не повторное чтение vanilla.

## Ограничения

- Совпавшие assertions не означают совпадения всех выходов или всех веток механики.
- Эти перечни неполны и пересекаются: их нельзя суммировать или выводить из них общий процент.
- Неизвестное не означает ни работающую, ни сломанную функциональность.
- Нет тестирования официальных игровых клиентов; live smoke использует RCON без игроков.
- JSON-отчёт содержит результаты по каждому ID; подтверждения привязаны к SHA и запуску.
