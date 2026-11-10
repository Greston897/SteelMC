# SteelMC — полный прогресс по подсистемам (замер 10 ноября 2026)

Ветка `arena/8ff5adc9-steelmc`, HEAD `5f17d30`, CI зелёный
(`Test`, `Flint Tests`, `Lint`, `Lint Autofix`, `Build Report`).
Все числа ниже — машинный подсчёт по дереву репозитория и по извлечённым из
ванильного 26.3 данным (`steel-registry/build_assets/*.json`,
`steel-core/build/classes.json`), а не оценки «на глаз».

## 1. Сводка

| Подсистема | Готово | Всего | % |
|---|---:|---:|---:|
| Реестры (данные из vanilla 26.3) | 5 945 записей | 5 945 | **100 %** |
| Предметы (по классам поведения) | 1 400 | 1 658 | 84,4 % |
| Предметы (с учётом базового `Item`) | **1 657** | 1 658 | **99,94 %** |
| Блоки (с собственным поведением) | 1 031 | 1 286 | 80,2 % |
| Сущности (зарегистрированные фабрики) | **75** | 161 | **46,6 %** |
| Пакеты протокола (есть Rust-модуль) | 131 | 234 | 56,0 % |
| Команды | 43 | ~50 ванильных + свои | ~85 % |
| Тесты | 3 578 | — | — |
| Строк кода (`src` всех крейтов) | ~497 000 | — | — |

Реестр «5 945 / 100 %» — это блоки 1 286, предметы 1 658, сущности 161,
звуки 1 991, эффекты 40, частицы 128, зелья 46, level events 92, game events 61,
статистика 78, типы меню 25, decoration 40, атрибуты 40, профессии 15,
типы крестьян 7, флюиды 5, data components 122, типы звуков 131, подозрительные
супы 17, источники позиции 2; остальное — теги и datapack-контент.

## 2. Протокол

* Объявлено в протоколе 26.3: **234** пакета — 78 serverbound + 156 clientbound.
* Есть Rust-модуль (`ReadFrom`/`WriteTo` + `#[packet_id]`): **42 S + 89 C = 131**.
  * фазы `handshake`, `status`, `login`, `config` — покрыты полностью (это то, без
    чего клиент вообще не заходит в мир);
  * `play`: 32 из 69 serverbound и 74 из 144 clientbound.
* Реализованы «тяжёлые» вещи: чанки с блочным светом, entity tracker, инвентарные
  синхронизации, команды и автодополнение, чат и подписи, scoreboard, границы мира,
  боссбар (добавлен `/tpsbar`), таб-лист.
* Не хватает в основном нишевых clientbound-пакетов (отладочные, `test_instance`,
  resource pack push/pop, `player_combat_*`, `post_effects`, часть recipe book)
  и serverbound-пакетов меню/крафта (`place_recipe`, `select_trade`, теги запросов
  блоков/сущностей).

## 3. Предметы

| Класс | Записей | Статус |
|---|---:|---|
| `Item` | 257 | покрыты базовым `DefaultItemBehavior` (в ванили у них тоже нет своего поведения) |
| `KnowledgeBookItem` | 1 | **единственный отсутствующий** — нужен слой книги рецептов (`RecipeDisplay`) |
| 63 специализированных класса (`BlockItem`, `ShearsItem`, `BoneMealItem`, `FlintAndSteelItem`, `EndCrystalItem`, `LeadItem`, `NameTagItem`, `BucketItem`, `CrossbowItem`, `BundleItem`, `SmithingTemplateItem`, `HangingSignItem`, …) | 1 400 | реализованы в `behavior/items/*` |

Фактическая готовность: **1 657 / 1 658 (99,94 %)**.

## 4. Блоки

* 1 286 типов блоков, из них **1 031 (80,2 %)** имеют собственный класс поведения
  (216 структур `#[block_behavior]`).
* Оставшиеся 255 записей — это 44 класса, и большая часть из них не дыра:
  `Block` (182) — это и есть обычный блок с базовым поведением;
  реально не сделаны: `ShelfBlock` (13), `InfestedBlock` (6),
  `CopperGolemStatueBlock` + `WeatheringCopperGolemStatueBlock` (8),
  `CommandBlock` (3), `AirBlock` (3), `NyliumBlock` (2),
  `BonemealableFeaturePlacerBlock` (2), `LeavesBlock`, `SpongeBlock`/`WetSpongeBlock`.
* Работает: коллизии/шейпы (730 форм), редстоун — 24 модуля (диоды, кнопки, рычаги,
  наблюдатель, поршни с движущимся блоком, лампы, компаратор, детектор дня),
  контейнеры, двери/люки, вода и лава, рост растений, кровати, порталы, кемпинги,
  медовые блоки, колокола и т. д.
* Блок-энтити: 24 реализации (печь/костёр/варильня-семейство, сундуки, шалкер,
  пчелиный улей, колокол, маяк, доменная печь, проигрыватель, зачарование,
  поршневой движущийся блок, спавнер, brushable, potent sulfur, conduit и др.).

## 5. Сущности

**75 из 161 (46,6 %)** типов имеют зарегистрированную фабрику (57 классов):

* мобы: `Zombie`, `Husk`, `Drowned`, `ZombifiedPiglin`, `WitherSkeleton`,
  `Silverfish`, `CaveSpider`, `Spider`, `Piglin`, `PiglinBrute`, `Hoglin`, `Zoglin`,
  `Vindicator`, `Endermite`, `Bat`, `Cod`, `Salmon`, `Pufferfish`, `TropicalFish`,
  `Cow`, `Mooshroom`, `Pig`, `Sheep`, `Chicken`, `Squid`;
* объекты: стенд для брони, картина, подушка, рамки, блок/item-дисплеи, поводок,
  примленный ТНТ, кристалл Края, опыт-орб, падающий блок, стрелы и снаряды
  (трезубец, жемчуг, глаз, бутылочки, фейерверк, рыболовный крючок, заряд ветра,
  снежок, яйцо), лодки и лодки с сундуком, вагонетки и вагонетка с сундуком.
* Не сделано (86 классов): вся деревенская линия (`Villager`, `WanderingTrader`,
  `ZombieVillager`, `IronGolem`, `SnowGolem`, `Cat`, `Wolf`, `Horse`+`Donkey`+`Mule`+
  `Llama`+`TraderLlama`, `PolarBear`, `Panda`, `Fox`, `Ocelot`, `Parrot`, `Rabbit`,
  `Goat`, `Camel`, `Turtle`, `Armadillo`, `Sniffer`, `Allay`, `Bee`, `Dolphin`,
  `GlowSquid`, `Frog`, `Tadpole`, `Axolotl`, `Nautilus`, `ZombieNautilus`,
  `CopperGolem`, `CamelHusk`, `Creaking`, `Mannequin`, `Strider`, `SulfurCube`),
  дальнобойные (`Skeleton`, `Stray`, `Bogged`, `Parched`, `Blaze`, `Breeze`,
  `Ghast`, `Shulker`, `Guardian`, `ElderGuardian`, `Witch`, `Evoker`, `Illusioner`,
  `Pillager`, `Vex`, `Phantom`, `WitherSkull`, `SmallFireball`, `LargeFireball`,
  `DragonFireball`, `ShulkerBullet`, `LlamaSpit`, `BreezeWindCharge`,
  `EvokerFangs`, `OminousItemSpawner`, `LightningBolt`), боссы (`EnderDragon`,
  `WitherBoss`, `Warden`, `Ravager`, `IronGolem`), слаймы (`Slime`, `MagmaCube`,
  `Giant`, `Creeper`, `Enderman`), оставшиеся вагонетки (`TNT`, `Furnace`,
  `Hopper`, `Spawner`, `CommandBlock`), дисплеи (`TextDisplay`, `ItemDisplay`,
  `GlowItemFrame`).

## 6. Ядро (что реально работает)

* **Мир и чанки:** секции с палитрами, heightmap, блочный и небесный свет
  (light engine + chunk pyramid), тикеты и планировщик, статусы генерации,
  сохранение/загрузка региональных файлов, автосейв, сущности по чанкам.
* **Worldgen:** multi-noise биомы (`climate_sampler`, nether-сэмплер), плотности
  и шумы, поверхностные правила, 2 карвера (`cave`, `canyon`), **53 фичи**,
  **17 типов структур** + jigsaw-сборка, шаблоны (structure templates),
  океанические монументы, крепости, шахты, деревни через jigsaw.
* **ИИ и мобы:** 34 типа целей (паника, ближний бой, таргетинг, размножение,
  `TemptGoal`, `FollowParentGoal`, плавание, избегание воды, `RestrictSunGoal`,
  `FleeSunGoal`, `LeapAtTargetGoal`, …), наземный и водный pathfinding,
  инстинкты `Animal`/`AgeableMob`, солнечное горение по тегу, спавн по `SpawnRule`.
* **Физика:** AABB-коллизии, шаг вверх, вода/лава (движение и плавание),
  падающие блоки, вагонетки и лодки, взрывы, порталы (Nether/End).
* **Инвентарь и крафт:** 17 типов меню, инвентарь игрока с предиктами,
  matching рецептов сетки крафта, печи/коптильни/плавильни, шалкеры, бочки.
* **Прочее:** лут-таблицы (условия, функции, пулы), зачарования и наковальня,
  эффекты зелий и снарядов, scoreboard, worldborder, права/группы, 43 команды
  с Brigadier-парсером, чат с подписями и online-mode, шифрование и компрессия,
  профилирование тиков (`/tick`, таб-лист, `/tpsbar`).
* **Качество:** 3 578 тестов (`steel-core` 3 159, `steel-registry` 419,
  `steel-utils` 222, `steel-worldgen` 95, `steel-protocol` 75, `steel-login` 19,
  `steel` 31, `steel-flint` 3), CI-гейт по тестам, clippy (pedantic), rustfmt и typos.

## 7. ТОП-10 дыр по влиянию на «похожесть на ваниль»

1. **86 классов сущностей** — половина мобов; самый заметный дефицит.
2. **Нет дальнобойного ИИ** (`RangedAttackGoal`) — блокирует скелетов, блейзов,
   гастов, шалкеров, стражей, ведьм; отсюда же «мечи без стрел».
3. **Естественный спавн хост-мобов не подключён**: `SpawnRule` даёт
   `&dyn LevelReader`, а `Monster::check_monster_spawn_rules` требует `&World`.
4. **Книга рецептов / `KnowledgeBookItem`** — нет `RecipeDisplay`.
5. **Деревня и торговля**: нет крестьян, профессий, торговли, железного голема.
6. **Боссы**: `EnderDragon`, `WitherBoss`, `Warden` — ни одного.
7. **Лошади/езда/приручение** — нет лошадей, волков, кошек, попугаев.
8. **`Slime`/`MagmaCube`/`Creeper`/`Enderman`** — уникальные механики
   (прыжки, взрыв, телепорт) отсутствуют.
9. **255 блоков без спец. поведения**, из них реальные: `ShelfBlock` (13),
   `InfestedBlock` (6), статуи медного голема (8), `CommandBlock` (3),
   `NyliumBlock` (2).
10. **103 пакета протокола без модуля** — нишевые, но часть нужна для меню/крафта
    и resource pack.

## 8. План следующих батчей (по убыванию отдачи)

1. `RangedAttackGoal` + стрелы: открывает `Skeleton`, `Stray`, `Bogged`, `Parched`,
   `Pillager`, позже `Blaze`/`Ghast`.
2. Животные батчем (шаблон `Animal`/`AgeableMob` готов): `Rabbit`, `Fox`,
   `PolarBear`, `Panda`, `Goat`, `Camel`, `Turtle`, `Armadillo`, `Strider`,
   `Parrot`, `Ocelot`, `Cat`.
3. Водные мобы из ведра: `Axolotl`, `Tadpole`, `Dolphin`, `GlowSquid`, `Frog`,
   `Nautilus`, `SulfurCube`.
4. Транспорт: `MinecartTNT`, `MinecartFurnace`, `MinecartHopper`,
   `MinecartSpawner`, `MinecartCommandBlock`.
5. Дисплеи: `TextDisplay`, `ItemDisplay`, `GlowItemFrame`; `LightningBolt`.
6. Уникальные механики: `Creeper`, `Enderman`, `Slime`/`MagmaCube`.
7. Деревня: `Villager` + профессии + торговля + `IronGolem`/`SnowGolem`.
8. Боссы: `WitherBoss` → `EnderDragon` → `Warden`.
9. Книга рецептов (как только определимся с `RecipeDisplay`).
10. Оставшиеся блоки и пакеты.
