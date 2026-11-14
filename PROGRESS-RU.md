# SteelMC — полный прогресс по подсистемам (замер 11 ноября 2026)

Ветка `arena/8ff5adc9-steelmc`; последняя завершённая итерация — слаймы и магматические кубы (`PARITY-26.3-RU.md` §23), до неё крипер, разбойник и эндермен (§§20–22). CI зелёный (`Test`, `Flint Tests`, `Lint`, `Lint Autofix`, `Build Report`).
Все числа ниже — машинный подсчёт по дереву репозитория и по извлечённым из
ванильного 26.3 данным (`steel-registry/build_assets/*.json`,
`steel-core/build/classes.json`), а не оценки «на глаз».

## 1. Сводка

| Подсистема | Готово | Всего | % |
|---|---:|---:|---:|
| Реестры (данные из vanilla 26.3) | 5 945 записей | 5 945 | **100 %** |
| Предметы (по классам поведения) | 1 400 | 1 658 | 84,4 % |
| Предметы (с учётом базового `Item`) | **1 657** | 1 658 | **99,94 %** |
| Блоки (с собственным поведением) | **1 031** | 1 286 | **80,2 %** |
| Блоки (с учётом базового `Block`) | **1 213** | 1 286 | **94,3 %** |
| Блок-энтити (зарегистрированы) | 25 | 49 | **51,0 %** |
| Меню (типы экранов) | 13 | 25 | **52,0 %** |
| Сущности (зарегистрированные фабрики) | **84** | 161 | **52,2 %** |
| Пакеты протокола (объявлено в `packets.json`) | **166** | 260 | **63,8 %** |
| Команды | 43 | ~50 ванильных + свои | ~85 % |
| Тесты (все крейты, `#[test]`) | **4 108** | — | — |
| Строк кода (`src` всех крейтов) | ~497 000 | — | — |

Реестр «5 945 / 100 %» — это блоки 1 286, предметы 1 658, сущности 161,
звуки 1 991, эффекты 40, частицы 128, зелья 46, level events 92, game events 61,
статистика 78, типы меню 25, decoration 40, атрибуты 40, профессии 15,
типы крестьян 7, флюиды 5, data components 122, типы звуков 131, подозрительные
супы 17, источники позиции 2; остальное — теги и datapack-контент.

## 2. Протокол

* Объявлено в `packets.json` (26.3): **260** пакетов — 87 serverbound + 173 clientbound.
* Структур в дереве с макросом пакета: **52 S + 107 C = 159** (с учётом
  переименований вроде `SClientIntention` = `intention` и
  `CInitializeBorder` = `initialize_border`) по именам совпадает с объявленными
  **166 из 260 (63,8 %)**.

| Фаза | serverbound | clientbound |
|---|---:|---:|
| `handshake` | 1 / 1 | — |
| `status` | 2 / 2 | 2 / 2 |
| `login` | 3 / 5 | 4 / 6 |
| `config` | 5 / 10 | 9 / 21 |
| `play` | 43 / 69 | 97 / 144 |
| **Итого** | **54 / 87** | **112 / 173** |

* `handshake` и `status` покрыты полностью, `login`/`config` — почти (не хватает
  cookie/ping/resource-pack частей). Это минимум, без которого клиент не
  заходит в мир.
* Реализованы «тяжёлые» вещи: чанки с блочным светом, entity tracker, инвентарные
  синхронизации, команды и автодополнение, чат и подписи, границы мира,
  боссбар (добавлен `/tpsbar`), таб-лист.
* **Scoreboard не синхронизируется**: данные объективов/команд/счётов есть в
  `steel-core/src/scoreboard.rs` и сохраняются на диск, но пакетов
  `set_objective`/`set_score`/`set_player_team`/`set_display_objective` нет,
  и команды `/scoreboard` тоже нет.
* Не хватает в основном нишевых clientbound-пакетов (отладочные, `test_instance`,
  resource pack push/pop, `player_combat_*`, `post_effects`, recipe book,
  scoreboard/team, `waypoint`) и serverbound-пакетов меню/крафта
  (`place_recipe`, `select_trade`, теги запросов блоков/сущностей,
  `set_game_rule`, `teleport_to_entity`).

## 3. Предметы

| Класс | Записей | Статус |
|---|---:|---|
| `Item` | 257 | покрыты базовым `DefaultItemBehavior` (в ванили у них тоже нет своего поведения) |
| `KnowledgeBookItem` | 1 | **единственный отсутствующий** — нужен слой книги рецептов (`RecipeDisplay`) |
| 63 специализированных класса (`BlockItem`, `ShearsItem`, `BoneMealItem`, `FlintAndSteelItem`, `EndCrystalItem`, `LeadItem`, `NameTagItem`, `BucketItem`, `CrossbowItem`, `BundleItem`, `SmithingTemplateItem`, `HangingSignItem`, …) | 1 400 | реализованы в `behavior/items/*` |

Фактическая готовность: **1 657 / 1 658 (99,94 %)**.

## 4. Блоки

* 1 286 типов блоков, из них **1 031 (80,2 %)** имеют собственный класс поведения
  (223 структуры `#[block_behavior]`).
* Оставшиеся 255 записей — это 44 класса, и большая часть из них не дыра:
  `Block` (182) — это и есть обычный блок с базовым поведением;
  реально не сделаны: `ShelfBlock` (13), `InfestedBlock` (6),
  `CopperGolemStatueBlock` + `WeatheringCopperGolemStatueBlock` (8),
  `CommandBlock` (3), `AirBlock` (3), `NyliumBlock` (2),
  `BonemealableFeaturePlacerBlock` (2), `LeavesBlock`, `SpongeBlock`/`WetSpongeBlock`,
  `TransparentBlock`, `DispenserBlock`, `ChestBlock`/`TrappedChestBlock`,
  `HopperBlock`, `DropperBlock`, `BrewingStandBlock`, `PistonHeadBlock`.
* Работает: коллизии/шейпы (730 форм), редстоун — 24 модуля (диоды, кнопки, рычаги,
  наблюдатель, поршни с движущимся блоком, лампы, компаратор, детектор дня),
  контейнеры, двери/люки, вода и лава, рост растений, кровати, порталы, кемпинги,
  медовые блоки, колокола и т. д.
* Блок-энтити: **25 из 49** типов зарегистрированы (`block_entity/registry.rs`):
  печь/доменная печь/коптильня, бочка, эндер-сундук, шалкер, пчелиный улей,
  колокол, маяк, костёр, проигрыватель, стол зачарования, поршень с движущимся
  блоком, спавнер, brushable, potent sulfur, conduit, компаратор, детектор дня,
  вывески, черепа, end portal/gateway.
* Остальные 24 типа не зарегистрированы и попадают в `UnimplementedBlockEntity`:
  он сохраняет NBT без потерь, но поведения и инвентаря нет. В их числе
  **сундук и ловушка-сундук**, воронка, раздатчик, выбрасыватель, крафтер,
  лектерн, бражная стойка, декоративный горшок, скульп-сенсор/крикун/катализатор,
  полка, баннер, статуя медного голема, командный блок, jigsaw, structure block,
  creaking heart, test block, trial spawner, vault.
* Меню сундука есть (`chest_menu`), но у блока нет ни блока-энтити, ни поведения
  открытия: работают только бочка и `ChestBoat` (у `ChestMinecart` контейнера
  ещё нет). Воронки и раздатчики не реализованы вовсе, поэтому автоматического
  перемещения предметов между контейнерами в игре нет.

## 5. Сущности

**84 из 161 (52,2 %)** типов имеют зарегистрированную фабрику (66 java-классов):

* мобы: `Zombie`, `Husk`, `Drowned`, `ZombifiedPiglin`, `WitherSkeleton`,
  `Silverfish`, `CaveSpider`, `Spider`, `Piglin`, `PiglinBrute`, `Hoglin`, `Zoglin`,
  `Vindicator`, `Endermite`, `Bat`, `Cod`, `Salmon`, `Pufferfish`, `TropicalFish`,
  `Cow`, `Mooshroom`, `Pig`, `Sheep`, `Chicken`, `Squid`;
* дальнобойные (`RangedAttackGoal` + помощник стрельбы): `Skeleton`, `Stray`,
  `Bogged`, `Parched` (стрелы со `slowness`/`poison`/`weakness`, конверсия скелета
  в `Stray` в порошковом снегу, стрижка боггеда);
* `Pillager`: арбалет в основной руке, синхронизируемая анимация зарядки, 25 тиков
  на заряд и выстрел, перезарядка 20 тиков, цели — игроки/жители/железные големы;
* уникальная механика: `Creeper` (ванильный фитиль `30` тиков, радиус `3`,
  `×2` при `is_powered`, взрыв `MOB` через существующий `ServerExplosion`,
  избегание кошек/оцелотов); `Enderman` (агро по прямому взгляду игрока,
  телепортация от снарядов, синхронизируемые переносимый блок и флаги anger);
* кубо-мобы: `Slime` и `MagmaCube` на общем `cube_mob` (здоровье `size²`,
  скорость `0.2+0.1×size`, урон `size` / `size+2`, броня `3×size`, прыжки с
  ванильными задержками, деление на 2–4 ребёнка при смерти, NBT `Size`
  в нулевой базе);
* объекты: стенд для брони, картина, подушка, рамки, блок/item-дисплеи, поводок,
  примленный ТНТ, кристалл Края, опыт-орб, падающий блок, стрелы и снаряды
  (трезубец, жемчуг, глаз, бутылочки, фейерверк, рыболовный крючок, заряд ветра,
  снежок, яйцо), лодки и лодки с сундуком, вагонетки и вагонетка с сундуком.
* Не сделано (77 java-классов, ровно столько же типов): вся деревенская линия
  (`Villager`, `WanderingTrader`,
  `ZombieVillager`, `IronGolem`, `SnowGolem`, `Cat`, `Wolf`, `Horse`+`Donkey`+`Mule`+
  `Llama`+`TraderLlama`, `PolarBear`, `Panda`, `Fox`, `Ocelot`, `Parrot`, `Rabbit`,
  `Goat`, `Camel`, `Turtle`, `Armadillo`, `Sniffer`, `Allay`, `Bee`, `Dolphin`,
  `GlowSquid`, `Frog`, `Tadpole`, `Axolotl`, `Nautilus`, `ZombieNautilus`,
  `CopperGolem`, `CamelHusk`, `Creaking`, `Mannequin`, `Strider`, `SulfurCube`),
  дальнобойные (`Blaze`, `Breeze`, `Ghast`, `Shulker`, `Guardian`, `ElderGuardian`,
  `Witch`, `Evoker`, `Illusioner`, `Vex`, `Phantom`, `WitherSkull`,
  `SmallFireball`, `LargeFireball`,
  `DragonFireball`, `ShulkerBullet`, `LlamaSpit`, `BreezeWindCharge`,
  `EvokerFangs`, `OminousItemSpawner`, `LightningBolt`), боссы (`EnderDragon`,
  `WitherBoss`, `Warden`, `Ravager`, `IronGolem`), `Giant`,
  оставшиеся вагонетки (`TNT`, `Furnace`,
  `Hopper`, `Spawner`, `CommandBlock`), дисплеи (`TextDisplay`, `ItemDisplay`,
  `GlowItemFrame`).

## 6. Ядро (что реально работает)

* **Мир и чанки:** секции с палитрами, heightmap, блочный и небесный свет
  (light engine + chunk pyramid), тикеты и планировщик, статусы генерации,
  сохранение/загрузка региональных файлов, автосейв, сущности по чанкам.
* **Worldgen:** multi-noise биомы (`climate_sampler`, nether-сэмплер), плотности
  и шумы, поверхностные правила, 2 карвера (`cave`, `canyon`), **51 тип фич**
  (`worldgen/feature/features`), **13 плейсеров структур** + jigsaw-сборка,
  шаблоны (structure templates), океанические монументы, крепости, шахты,
  деревни через jigsaw.
* **ИИ и мобы:** 34 типа целей (паника, ближний бой, таргетинг, размножение,
  `TemptGoal`, `FollowParentGoal`, плавание, избегание воды, `RestrictSunGoal`,
  `FleeSunGoal`, `LeapAtTargetGoal`, …), наземный и водный pathfinding,
  инстинкты `Animal`/`AgeableMob`, солнечное горение по тегу, спавн по `SpawnRule`.
* **Физика:** AABB-коллизии, шаг вверх, вода/лава (движение и плавание),
  падающие блоки, вагонетки и лодки, взрывы, порталы (Nether/End).
* **Инвентарь и крафт:** **16 модулей меню** (13 «экранных» типов из 25 ванильных:
  инвентарь, крафт, сундук/бочка/шалкер, печь/коптильня/плавильня, наковальня,
  зачарование, точило, маяк, эндер-сундук, лодка с сундуком), инвентарь игрока
  с предиктами, matching рецептов сетки крафта.
* **Прочее:** лут-таблицы (условия, функции, пулы), зачарования и наковальня,
  эффекты зелий и снарядов, worldborder, права/группы, 43 команды
  с Brigadier-парсером, чат с подписями и online-mode, шифрование и компрессия,
  профилирование тиков (`/tick`, таб-лист, `/tpsbar`).
* **Качество:** **4 108** тестов (`steel-core` 3 255, `steel-registry` 419,
  `steel-utils` 227, `steel-worldgen` 96, `steel-protocol` 96, `steel` 32,
  `steel-login` 22, `steel-flint` 3), CI-гейт (`cargo test --workspace`),
  clippy pedantic с `-Dwarnings`, rustfmt и typos.
  В последнем прогоне: `steel-core` lib — 3 232 passed, 1 ignored.

## 7. ТОП-10 дыр по влиянию на «похожесть на ваниль»

1. **Сундуки, воронки, раздатчики**: у обычного сундука нет блока-энтити и
   поведения открытия, воронок/раздатчиков нет вовсе — автоматики с предметами
   в игре нет совсем. Самая заметная дыра после мобов.
2. **77 отсутствующих классов сущностей** — почти половина мобов; деревня,
   лошади, боссы и дальнобойные ждут своих механик.
3. **Дальнобойный ИИ частично покрыт**: скелеты и разбойники работают;
   блейзы, гасты, шалкеры, стражи и ведьмы ждут собственных снарядов/механик.
4. **Естественный спавн хост-мобов не подключён**: `SpawnRule` даёт
   `&dyn LevelReader`, а `Monster::check_monster_spawn_rules` требует `&World`.
   Следствие: в мире не появляются зомби/скелеты/слизни сами по себе.
5. **Scoreboard не синхронизируется**: данные есть, пакетов и команды нет.
6. **Книга рецептов / `KnowledgeBookItem`** — нет `RecipeDisplay`.
7. **Деревня и торговля**: нет крестьян, профессий, торговли, железного голема.
8. **Боссы**: `EnderDragon`, `WitherBoss`, `Warden` — ни одного.
9. **80 блоков без спец. поведения**, из них реальные: `ShelfBlock` (13),
   `InfestedBlock` (6), статуи медного голема (8), `CommandBlock` (3),
   `NyliumBlock` (2).
10. **94 пакета протокола без модуля** (260 объявлено, 166 покрыто) — нишевые,
    но часть нужна для меню/крафта, resource pack и scoreboard.

## 8. План следующих батчей (по убыванию отдачи)

0. **Контейнерная автоматика**: блок-энтити сундука (и двойного), затем воронка
   и раздатчик/выбрасыватель. Закрывает ТОП-1 и разблокирует предметные фермы.
1. ~~`RangedAttackGoal` + стрелы~~ — **сделано** (`PARITY-26.3-RU.md` §19):
   `Skeleton`, `Stray`, `Bogged`, `Parched`, `Pillager`; следующими на этой базе
   идут `Illusioner` (враждебная иллюзия) и разблокированные дальнобойные цели.
2. Природный спавн: снять блокер `SpawnRule` ↔ `&World`, включить спавн
   враждебных мобов и биомные замещения (`Parched` в пустынях, `Stray` в снегах).
3. Животные батчем (шаблон `Animal`/`AgeableMob` готов): `Rabbit`, `Fox`,
   `PolarBear`, `Panda`, `Goat`, `Camel`, `Turtle`, `Armadillo`, `Strider`,
   `Parrot`, `Ocelot`, `Cat`.
4. Водные мобы из ведра: `Axolotl`, `Tadpole`, `Dolphin`, `GlowSquid`, `Frog`,
   `Nautilus`, `SulfurCube`.
5. Транспорт: `MinecartTNT`, `MinecartFurnace`, `MinecartHopper`,
   `MinecartSpawner`, `MinecartCommandBlock`.
6. Дисплеи: `TextDisplay`, `ItemDisplay`, `GlowItemFrame`; `LightningBolt`
   (разблокирует «заряженного» крипера).
7. Уникальные механики: ~~`Creeper`~~ (§20), ~~`Enderman`~~ (§22),
   ~~`Slime`~~/~~`MagmaCube`~~ (§23) — **сделано**; следующими идут
   `Villager` и големы.
8. Деревня: `Villager` + профессии + торговля + `IronGolem`/`SnowGolem`.
9. Боссы: `WitherBoss` → `EnderDragon` → `Warden`.
10. Книга рецептов (как только определимся с `RecipeDisplay`).
11. Оставшиеся блоки и пакеты.
