# Steel API — плагин-API ядра SteelMC

> **Своё, понятное, на уровне Bukkit/Paper, но лучше.**  
> Каждая механика ванили доступна плагину. ViaVersion встроен. Rust — безопасно, быстро, без `null`.

---

## Зачем своё, а не Paper-форк?

| Paper / Bukkit | Steel API |
|---|---|
| Java, `null`, `instanceof`, `Bukkit.getServer()` синглтон-монстр | Rust, `Result`, `trait`, `Send+Sync`, DI через `PluginContext` |
| События через рефлексию `@EventHandler` — ломается тихо | События через `EventBus::subscribe::<T>` — типобезопасно, `priority` компилируется |
| ViaVersion — внешний `jar`, ломает пакеты | ViaVersion — **встроен** в ядро: `ViaPipeline` + `ProtocolTranslator` — первоклассный API |
| Блоки — `BlockBreakEvent` есть, `RandomTick` нет | **Все** механики: `BlockTickEvent`, `BlockRandomTickEvent`, `FluidLevelChange`, `Redstone`, `Physics`, `Piston`, `Note` |
| Плагин не может заменить `BlockBehavior` без NMS | Плагин реализует `BlockBehaviorHook` и ядро делегирует — чистый Rust trait |

**Идея:** знакомый Bukkit-UX (`Event`, `Listener`, `Scheduler`, `Command`, `World`/`Player`/`Block`), но без наследия Java-багов. Если ваниль это делает — ты можешь это поменять.

---

## Быстрый старт (30 секунд)

`Cargo.toml` твоего плагина:

```toml
[dependencies]
steel-api = { path = "../steel-api" } # или crates.io когда опубликуем
uuid = "1"
```

`src/lib.rs`:

```rust
use steel_api::prelude::*;
use steel_api::plugin::{Plugin, PluginContext, PluginResult};

#[derive(Default)]
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my_plugin" }
    fn version(&self) -> &str { "1.0.0" }

    fn on_load(&self, ctx: &PluginContext) -> PluginResult<()> {
        // команды и пермишены — до загрузки миров
        ctx.commands().register(
            steel_api::command::literal("hello")
                .executes(|src, _| { src.send_success("Hello Steel!"); Ok(1) })
        ).unwrap();
        Ok(())
    }

    fn on_enable(&self, ctx: &PluginContext) -> PluginResult<()> {
        // события — когда миры готовы
        ctx.events().subscribe::<PlayerJoinEvent, _>(
            EventPriority::Normal, "my_plugin", |e| {
                log::info!("{} зашёл!", e.player_name);
            }
        );
        // планировщик — как BukkitRunnable
        ctx.scheduler().run_task_timer("my_plugin", std::time::Duration::from_secs(10), || {
            log::info!("10 сек прошло");
        });
        Ok(())
    }
}

steel_api::export_plugin!(MyPlugin);
```

`plugin.yml` не нужен — дескриптор в `Plugin::descriptor()` (`name`, `version`, `depend`, `soft_depend`). Для динамической загрузки — `plugins/my_plugin.toml`.

Подключи в `steel-core`:

```rust
use steel_core::plugin::PluginManager;
use std::sync::Arc;

let mgr = PluginManager::default();
mgr.register(Arc::new(MyPlugin::default())).unwrap();
mgr.enable_all();
```

---

## Покрытие механик — 100%

Каждая система ядра имеет хук. Таблица (сокращённо, полный список в `steel-api/src/events/`):

| Категория | События / хуки | Пример |
|---|---|---|
| **Блоки — стройка** | `BlockBreakEvent`, `BlockPlaceEvent`, `BlockInteractEvent`, `BlockTickEvent`, `BlockRandomTickEvent`, `BlockGrowEvent`, `BlockFormEvent`, `BlockSpreadEvent`, `BlockFadeEvent`, `BlockFromToEvent` | Отменить ломание бедрока: `if e.block.key == bedrock { cancel }` |
| **Блоки — редстоун** | `BlockRedstoneEvent`, `BlockPistonExtendEvent`, `BlockPistonRetractEvent`, `NotePlayEvent` | `e.new_power = 0` — глушить редстоун |
| **Блоки — жидкость** | `FluidLevelChangeEvent`, `CauldronLevelChangeEvent` | Запретить лаву: `if e.fluid == lava { cancel }` |
| **Сущности — мобы** | `EntitySpawnEvent`, `EntityDamageEvent`, `EntityDeathEvent`, `EntityTargetEvent`, `EntityTeleportEvent`, `MobEffectAddEvent`, `EntityTameEvent`, `EntityBreedEvent`, `VillagerTradeEvent` | `if e.entity_type == "creeper" { cancel }` — без криперов |
| **Сущности — снаряды** | `ProjectileLaunchEvent`, `ProjectileHitEvent`, `EntityExplodeEvent` | Античит на стрелы |
| **Инвентарь** | `InventoryClickEvent`, `InventoryDragEvent`, `InventoryOpenEvent`, `InventoryCloseEvent`, `CraftItemEvent`, `FurnaceSmeltEvent`, `EnchantItemEvent`, `PrepareAnvilEvent`, `PrepareGrindstoneEvent` | Запретить клик в слот 0 |
| **Мир** | `WorldLoadEvent`, `ChunkLoadEvent`, `ChunkUnloadEvent`, `WeatherChangeEvent`, `TimeSkipEvent`, `PortalCreateEvent`, `StructureGrowEvent`, `WorldGameRuleChangeEvent` | Вечное солнце: `if e.to_storm { cancel }` |
| **Игрок** | `PlayerJoinEvent`, `PlayerQuitEvent`, `PlayerChatEvent`, `PlayerCommandPreprocessEvent`, `PlayerMoveEvent`, `PlayerInteractEvent`, `PlayerDropItemEvent`, `PlayerGameModeChangeEvent`, `PlayerItemConsumeEvent`, `PlayerAdvancementDoneEvent`, `PlayerExpChangeEvent` | Фильтр мата в чате |
| **Сервер** | `ServerTickEvent`, `ServerCommandEvent`, `ServerListPingEvent`, `PlayerLoginEvent`, `PluginEnableEvent`, `PacketReceiveEvent`, `PacketSendEvent`, `BroadcastEvent` | Античит на пакеты |
| **Поведения** | `BlockBehaviorHook` (15 методов), `ItemBehaviorHook` (6), `FluidBehavior` через события | `on_step_on`, `on_redstone_power`, `on_explode` |

> **Принцип:** если в `steel-core/src/behavior/`, `world/`, `entity/`, `inventory/` есть код — в `steel-api` есть хук.

---

## События — как в Bukkit, но типобезопасно

```rust
use steel_api::{EventBus, EventPriority};
use steel_api::events::block::BlockBreakEvent;

let bus = ctx.events();
let handle = bus.subscribe::<BlockBreakEvent, _>(
    EventPriority::High, // Lowest < Low < Normal < High < Highest < Monitor
    "my_plugin",
    |e| {
        if e.block.key.to_string() == "minecraft:bedrock" {
            e.set_cancelled(true); // Cancellable
        }
    }
);
// отписка
bus.unsubscribe(handle);
// проверка
assert!(bus.has_listeners::<BlockBreakEvent>());
```

`Monitor` — только чтение результата (античит логирует, но не меняет).

---

## Блоки / предметы / жидкости — любая механика

```rust
use steel_api::block::{Block, BlockBehaviorHook};
use steel_utils::Identifier;

struct MyOre;
impl BlockBehaviorHook for MyOre {
    fn on_place(&self, block: &Block, player: Option<uuid::Uuid>) -> bool { true }
    fn on_break(&self, block: &Block, player: Option<uuid::Uuid>) -> bool { true }
    fn on_tick(&self, block: &Block) { /* кастомный тик */ }
    fn on_redstone_power(&self, block: &Block, power: u8) { /* редстоун */ }
    fn on_step_on(&self, block: &Block, entity: i32) { /* наступили */ }
    fn on_explode(&self, block: &Block, yield_rate: f32) -> f32 { yield_rate * 0.5 }
}

// регистрация — в on_enable
// PLUGIN_HOOKS.register(Identifier::new_static("my","ore"), Arc::new(MyOre));
```

Аналогично `ItemBehaviorHook`: `on_use`, `on_use_on_block`, `on_attack_entity`, `on_consume`.

Жидкости — через `FluidLevelChangeEvent` (каждый тик жидкости).

---

## Мир, чанки, погода — World API

```rust
use steel_api::world::World;
use steel_utils::{BlockPos, ChunkPos};

fn demo(world: &dyn World) {
    let pos = BlockPos::new(0, 64, 0);
    let state = world.get_block_state(pos);
    world.set_block(pos, state, 3); // flags как в ванили
    world.play_sound(pos.to_vec3(), "minecraft:block.stone.break", 1.0, 1.0);
    world.spawn_particles("minecraft:flame", pos.to_vec3(), 10, 0.5,0.5,0.5, 0.1);

    // чанки
    world.load_chunk(ChunkPos::new(0,0));
    // погода
    world.set_raining(false);
    world.set_time(6000); // полдень
    world.set_game_rule("doDaylightCycle", "false");
}
```

---

## Команды — brigadier, как в ванили

```rust
use steel_api::command::literal;

ctx.commands().register(
    literal("heal")
        .requires_permission("my.heal".to_string())
        .executes(|src, _| {
            if let Some(id) = src.player_id { /* heal */ }
            src.send_success("вылечил!");
            Ok(1)
        })
        .then(
            literal("other") // /heal other <player>
                .executes(|src, _| { Ok(0) })
        )
)?;
```

---

## Планировщик — Bukkit Scheduler, но без `BukkitRunnable`

```rust
let sched = ctx.scheduler();
// синхронно на след. тик
sched.run_task("my", || println!("next tick"));
// через 5 сек
sched.run_task_later("my", Duration::from_secs(5), || {});
// каждый тик (50мс)
let handle = sched.run_task_timer("my", Duration::from_millis(50), || {});
// асинхронно (не блокируй тик!)
sched.run_task_async("my", || { /* IO */ });
// отмена
handle.cancel();
sched.cancel_all_for("my");
```

Ядро вызывает `scheduler.tick()` каждый тик.

---

## Права — LuckPerms-совместимо

```rust
use steel_api::permission::{Permission, PermissionDefault};

ctx.permissions().register(Permission::new("my.admin", PermissionDefault::Op));
ctx.permissions().register(Permission::new("my.build", PermissionDefault::True));

// в коде
if player.has_permission("my.admin") { /* ... */ }
// wildcard
steel_api::permission::permission_matches("my.*", "my.admin"); // true
```

`groups.toml` понимает `steel.build`, `my.admin`.

---

## ViaVersion — встроен, не плагин

**Проблема ViaVersion в Paper:** внешний `jar`, патчит пакеты рефлексией, ломается каждый снапшот.

**Steel:** `ViaPipeline` — часть ядра, плагин регистрирует `ProtocolTranslator`.

```rust
use steel_api::protocol::{ProtocolVersion, ProtocolTranslator, ViaPipeline};
use std::sync::Arc;

struct MyRemapper { from: ProtocolVersion, to: ProtocolVersion }
impl ProtocolTranslator for MyRemapper {
    fn from(&self) -> ProtocolVersion { self.from }
    fn to(&self) -> ProtocolVersion { self.to }
    fn remap_outbound_id(&self, id: i32) -> i32 { if id == 0x2A { 0x2B } else { id } }
    fn translate_outbound(&self, id: i32, mut data: Vec<u8>) -> steel_api::protocol::TranslationResult {
        if id == 0x2B { data.push(0xFF); }
        Ok((id, data))
    }
}

// регистрация
let via = ViaPipeline::new(); // в ядре: PluginManager::via()
via.register(Arc::new(MyRemapper { from: ProtocolVersion::V1_21_8, to: ProtocolVersion::V26_3 }));
via.register(Arc::new(MyRemapper { from: ProtocolVersion::V26_3, to: ProtocolVersion::V1_21_8 }));

// ядро само вызывает:
// via.translate_inbound(client_version, server_version, id, data)  // client -> server
// via.translate_outbound(server_version, client_version, id, data) // server -> client
```

Поддерживаемые версии: `1.21.5 (770)` .. `26.3 (777)`. Добавь свою — зарегистрируй пару трансляторов, `ViaPipeline` найдёт путь BFS (`1.21.5 -> 1.21.8 -> 26.3`).

**Где в ядре:** `steel-core/src/plugin/via.rs` (`ViaBridge`) — `steel-login` вызывает `via.inbound/outbound` для каждого пакета. Если `client_version != server_version` и нет пути — кик "unsupported version".

---

## Жизненный цикл плагина

```
[ загрузка .so / compiled ] -> PluginManager::register() -> on_load()  // реестры ещё открыты
    -> (ядро грузит миры)
    -> PluginManager::enable_all() -> on_enable() // миры готовы, можно трогать World
    -> [ тики, события, команды ]
    -> PluginManager::disable() -> on_disable()   // сохрани данные, cancel задачи
```

`on_load` — регистрируй `BlockBehaviorHook`, команды без мира.  
`on_enable` — подписывайся на события, трогай `World`.  
`on_disable` — `cancel_all_for`.

---

## Понятность — почему Steel проще Bukkit

* **Один `PluginContext`** — вместо `Bukkit.getServer().getPluginManager().get...` — `ctx.events()`, `ctx.scheduler()`, `ctx.commands()`.
* **Нет `null`** — `Option`, `Result`, `&dyn World` вместо `World world = null`.
* **Нет наследования** — `trait Plugin`, `trait BlockBehaviorHook` вместо `extends JavaPlugin`.
* **Док-примеры компилируются** — `cargo check -p steel-api` гоняет примеры.
* **Русская дока** — ты читаешь это, а не javadoc на 5000 строк.

Пример сравнения:

Bukkit:
```java
@EventHandler(priority = EventPriority.HIGH)
public void onBreak(BlockBreakEvent e) {
    if (e.getBlock().getType() == Material.BEDROCK) e.setCancelled(true);
}
```

Steel:
```rust
ctx.events().subscribe::<BlockBreakEvent, _>(EventPriority::High, "my", |e| {
    if e.block.key == Identifier::new_static("minecraft","bedrock") { e.set_cancelled(true); }
});
```

Никакой аннотации-магии — просто функция.

---

## Примеры

* `steel-api/examples/hello_world.rs` — минимальный плагин
* `steel-api/examples/custom_mechanics.rs` — все механики + Via

Запуск: `cargo run -p steel-api --example hello_world`

---

## Как добавить свою механику?

1. Найди ваниль-код в `steel-core/src/behavior/blocks/redstone/...` или `entity/...`
2. Добавь событие в `steel-api/src/events/` + хук в `block.rs`/`entity.rs`
3. В `steel-core/src/plugin/hooks.rs` делегируй: `PLUGIN_HOOKS.get(id).map(|h| h.on_xxx(...))`
4. Плагин подписывается — готово. Никакого NMS.

---

## Статус

* API `0.17.0+mc26.3` — синхронно с `Cargo.toml` workspace
* `steel-api` — в `workspace.members`, `default-members` — собирается в CI (`cargo check`, `clippy`, `fmt`, `test`)
* `steel-core` зависит от `steel-api`, `PluginManager` готов
* ViaVersion — `ViaPipeline` + `ViaBridge` в `steel-core/src/plugin/via.rs`
* 80+ событий, 2 примера, 6 unit-тестов в API

Дальше: `dlopen` лоадер для `.so`, `steel build --plugin` CLI, публикация на `crates.io`.

---

## Лицензия

AGPL-3.0-or-later — как ядро SteelMC.
