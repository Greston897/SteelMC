# Steel Plugin System — Полный Bukkit/Paper повтор 1:1, но на Rust и лучше

> **Один файл — как плагин попадает в ядро, как компилируется, как парсится, как не течёт.**
> Версия API `0.17.0+mc26.3` | Protocol 777 | Rust `nightly-2026-10-14`

---

## 1. Философия: почему Rust, а не Java

| Bukkit Java | Steel Rust |
|---|---|
| `ClassLoader` на каждый плагин, `URLClassLoader` утекает, `static Bukkit` глобалка, NPE | `Arc<dyn Plugin>` + `Weak` в колбэках, `SyncRwLock`, `Drop` guard, компилятор закрывает жизнь |
| `@EventHandler` рефлексия — падает тихо | `ctx.events().subscribe::<T>(…)` — типобезопасно, `TypeId` + `Send+Sync` |
| `BukkitRunnable` утекает если не `cancel` | `TaskId` + `Drop` + `Scheduler::cancel_all_for(plugin)` на `disable` автоматически |
| `HashMap` + `synchronized` | `FxHashMap` + `SyncRwLock`/`SyncMutex` везде, `Mspt` без блокировок |
| ViaVersion — внешний jar | `ViaPipeline` встроен, `Arc<ViaPipeline>` zero-copy |

**Гарантия утечек:** любой вшивый плагин не может утечь, потому что:
- Нет `Box::leak`, `mem::forget`, `static mut`.
- Все хендлы — RAII: `ListenerHandle::Drop` → `EventBus::unsubscribe`, `TaskHandle::Drop` → `Scheduler::cancel`.
- `PluginHandle::Drop` снимает **всё**: команды, регионы, рецепты, placeholder, packet listeners, hooks, `Messenger`.
- `LibraryHandle` (для `cdylib`) `Drop` делает `dlclose` только когда `Arc` count==0.
- Клиппи `pedantic+ nursery -Dwarnings` не пропускает `forget`.

---

## 2. Как плагин заходит в ядро — 3 типа

### 2.1 Static (рекомендуется, для прод)
```toml
# plugins/my_plugin/Cargo.toml
[package] name="my_plugin" version="1.0.0"
[lib] crate-type=["lib"]
[dependencies] steel-api={path="../../steel-api"}
```
```rust
use steel_api::prelude::*;
#[derive(Default)] pub struct MyPlugin;
impl Plugin for MyPlugin {
  fn name(&self)->&str{"my_plugin"} fn version(&self)->&str{"1.0.0"}
  fn on_enable(&self,ctx:&PluginContext)->PluginResult<()>{
    ctx.events().subscribe::<PlayerJoinEvent,_>(EventPriority::Normal,"my_plugin",|e| log::info!("{}",e.player_name));
    Ok(())
  }
}
steel_api::export_plugin!(MyPlugin); // compile-time check
```
Сервер собирается `cargo build --release -p steel` — плагин линкуется статически. **Плюсы:** LTO, нет `dlopen`, самый быстрый, safest.

### 2.2 Dynamic cdylib (как jar, hot-load)
```toml
[lib] crate-type=["cdylib"]  # → libmy_plugin.so / .dll / .dylib
```
```rust
#[no_mangle] pub extern "C" fn _steel_create()->*mut dyn Plugin { Box::into_raw(Box::new(MyPlugin)) }
#[no_mangle] pub extern "C" fn _steel_descriptor()->PluginDescriptor { MyPlugin.descriptor() }
```
Кладёшь `libmy_plugin.so` + `steel-plugin.toml` (или `plugin.yml`) в `plugins/` → ядро `libloading::Library::new(path)` → `symbol::<CreateFn>` → `Arc::from_raw` → `register`.  
**Безопасность:** `Arc` + `Weak` в шине, версия `api-version` проверяется, `hash` проверяется, `dlclose` только на `Drop` когда задачи отлетели.

### 2.3 WASM (sandbox, не доверенные)
```toml
[lib] crate-type=["cdylib"] target="wasm32-wasi"
```
`wasmtime` грузит `my_plugin.wasm` с WIT-интерфейсом `steel:plugin/host`. Полная изоляция, лимит памяти.

**Итог:** все 3 типа парсят один дескриптор и приходят к одному `Arc<dyn Plugin>`.

---

## 3. Дескриптор — повтор `plugin.yml` точь-в-точь

Поддерживаются **оба файла** (приоритет `steel-plugin.toml`):

`steel-plugin.toml`:
```toml
[plugin]
name="my_plugin"              # [a-z0-9._-] 1..64
version="1.2.3"
main="my_plugin::MyPlugin"    # для cdylib — символ
api-version="26.3"
description="Лучший плагин"
author="Notch"                # первый
authors=["Dinnerbone"]        # остальные
contributors=["jeb_"]
website="https://example.com"
prefix="MyPl"                 # лог-префикс, иначе name
load="POSTWORLD"              # STARTUP | POSTWORLD
depend=["Vault"]              # hard
softdepend=["PlaceholderAPI"] # soft
loadbefore=["BrokenPlugin"]
provides=["OldName"]
libraries=["com.squareup.okhttp3:okhttp:4.9.0"]
folia-supported=true

[commands.heal]
description="Heal player"
aliases=["h"]
permission="my.heal"
usage="/heal [player]"

[permissions."my.*"]
description="All"
default="op"
children={ "my.heal"=true }
```

Эквивалент `plugin.yml`:
```yaml
name: my_plugin
version: 1.2.3
main: my_plugin::MyPlugin
api-version: 26.3
depend: [Vault]
softdepend: [PlaceholderAPI]
loadbefore: [BrokenPlugin]
libraries: [com.squareup.okhttp3:okhttp:4.9.0]
commands:
  heal:
    description: Heal player
    aliases: [h]
    permission: my.heal
permissions:
  my.*:
    description: All
    default: op
    children: { my.heal: true }
```

Парсинг: `PluginSpec::from_toml` / `from_yaml` → `validate()` → `sort_plugins` (Kahn по `depend`/`softdepend`/`loadbefore`) → `PluginManager::register`.

---

## 4. Жизненный цикл — как `JavaPlugin`

```rust
trait Plugin: Send+Sync+'static {
  fn name(&self)->&str;
  fn version(&self)->&str;
  fn on_load(&self, ctx:&PluginContext)->PluginResult<()> { Ok(()) }   // до миров, регистрируй блоки/рецепты/worldgen
  fn on_enable(&self, ctx:&PluginContext)->PluginResult<()> { Ok(()) } // после миров, команды/события/таски
  fn on_disable(&self, ctx:&PluginContext) {}                           // cancel, save
  fn descriptor(&self)->PluginDescriptor { … } // автоген из Cargo.toml
}
```

`PluginManager::register` → `on_load` → `PluginState::Loaded`  
`enable_all` → `on_enable` → `Enabled` (с топосортом зависимостей)  
`disable` → `on_disable` + `scheduler.cancel_all_for(name)` + `event_bus.unsubscribe_all(name)` → `Disabled`

---

## 5. Конфиги — как `FileConfiguration` но `serde`

```rust
#[derive(Deserialize, Serialize, Clone, Default)]
struct MyCfg { heal_cooldown: u64, prefix: String }

let mut file = ctx.configs().yaml_file("config", MyCfg::default());
file.load()?;               // читает plugins/my_plugin/config.yml, создаёт с дефолтом если нет
file.get().heal_cooldown;   // &MyCfg
file.get_mut().prefix = "Hi".into();
file.save()?;               // pretty json/yaml
file.reload()?;             // перечит
```

`ConfigManager::json_file` / `yaml_file` / `file(name, default)` — 1:1 с Bukkit `getConfig()`, `saveDefaultConfig()`, `reloadConfig()`.  
`ConfigFile<T>` где `T: DeserializeOwned+Serialize+Clone+Default` — типобезопасно, `serde`.

---

## 6. Сообщения — MiniMessage + Adventure + i18n

```rust
// messages_en.yml (рядом с config.yml)
prefix: "<gray>[<green>My<gray>]"
heal: "<green>Healed {player}!"

// в коде
let msg = ctx.configs().file("messages_en.yml", HashMap::<String,String>::default());
let pref = msg.get().get("prefix").unwrap();
let c = MiniMessage::deserialize_with_placeholders(&pref, &ph);
player.send_message(&c.to_legacy());
```

`Component`, `MiniMessage`, `Title`, `BossBar`, `Tab` — 1:1 с Paper Adventure.

---

## 7. Команды — Brigadier, override любой ванили

```rust
ctx.override_command(literal("give") // заменя день Essentials
  .then(argument_player("t").then(argument_string("item").executes(|src,ctx| { Ok(1) })))
);
ctx.commands().register(literal("heal").requires(|s| s.has_permission("my.heal")).executes(|src,_| { src.send_success("healed"); Ok(1) }));
```

`CommandRegistryHandle::register` / `register_override` (возвращает старый `CommandNode`), алиасы, `requires`, `suggests`, все 22 `ArgType`.

---

## 8. События — 80+ шт, покрыто всё

`PlayerJoinEvent`, `PlayerQuitEvent`, `BlockBreakEvent` (Cancellable), `EntityDamageEvent`, `InventoryClickEvent`, `ServerLoadEvent` и т.д.  
```rust
let h = ctx.events().subscribe::<PlayerJoinEvent,_>(EventPriority::Normal, "my", |e| e.join_message = None);
ctx.events().unsubscribe(h); // или Drop guard
```

---

## 9. Мир / Мульти-мир — как Multiverse

```rust
let wm = ctx.world_manager();
wm.register(WorldCreator::new("lobby").seed(42).env(WorldEnv::Normal).gamerule("keepInventory","true"));
ctx.server().create_world("lobby", Some(42));
ctx.server().unload_world("lobby", true);
WorldBorder::new(0.0,0.0,100.0).is_inside(x,z);
```

`WorldCreator` (Normal/Nether/TheEnd/Custom, Flat/LargeBiomes, seed, generator `my:gen`, biome `my:biome`, gamerules, border, spawn, hardcore).  
`WorldManager::create(server, creator)` — 1:1 с `Bukkit.createWorld`.

---

## 10. Ресурс-пак — ваниль точь-в-точь

```rust
let pack = ResourcePack::new("https://cdn/pack.zip").with_hash("a".repeat(40)).with_prompt("Need pack!").required(true);
ctx.resourcepacks().add_global(pack.clone()); // всем новеньким
player.set_resource_pack(pack); // конкретному
// клиент шлёт ResourcePackStatus → ctx.resourcepacks().on_status(player, id, status) → PlayerResourcePackStatusEvent
```

`ResourcePackManager::validate_hash`, `PlayerResourcePackStatusEvent` как в Paper.

---

## 11. Plugin Messaging (BungeeCord)

```rust
ctx.messenger().register_incoming("my","bungeecord:main")?;
ctx.messenger().add_listener("my","bungeecord:main", Arc::new(MyListener));
ctx.messenger().dispatch_incoming(Some(player), "my:channel", b"data");
Messenger::bungee_connect("lobby"); // хелперы Connect/Forward
```

---

## 12. Баны / Вайтлист — как `BanList`

```rust
ctx.bans().ban_list(BanType::Name).add("Notch", Some("grief"), Some("Admin"), Some(Duration::from_secs(86400)));
ctx.bans().is_whitelisted("Alex");
ctx.bans().set_whitelisted("Alex", true);
```

---

## 13. Services (Vault/LuckPerms) — `ServicesManager`

```rust
ctx.services().register::<dyn EconomyProvider>("Vault", Arc::new(MyEco), ServicePriority::Normal);
let eco = ctx.services().get::<dyn EconomyProvider>().unwrap();
```

---

## 14. Остальные мини-механики (покрыто)

- `Chunk`/`Clipboard`/`fill_area` — WorldEdit из коробки
- `Region`/`RegionManager`/`Flag` — WorldGuard
- `EconomyHandle`/`PlaceholderManager`/`TagRegistry`/`RecipeManager`/`PacketBus`/`WorldgenRegistry`
- `BlockBehaviorHook`/`ItemBehaviorHook`/`FluidBehaviorHook`/`RedstoneHook`/`ChunkGeneratorHook`/`BiomeProviderHook`/`FeatureHook`
- `Attribute`/`Effect`/`Enchantment`/`Potion`/`Scoreboard`/`BossBar`/`Tab`/`Inventory`/`ItemStack`
- `Nbt`/`Metadata`/`PersistentDataContainer` — типобезопасно

Если ваниль это делает — ты можешь это поменять. Если Paper это умеет — Steel умеет или лучше.

---

## 15. Чек-лист плагинов

| Плагин | Порт на Steel |
|---|---|
| Essentials (`/heal`, `/give` override, homes) | `override_command` + `World#spawn` + `EconomyHandle` |
| WorldEdit | `Clipboard` + `Chunk::set_blocks_bulk` |
| WorldGuard/GriefPrevention | `RegionManager` + `Flag` |
| Vault | `ServicesManager` + `EconomyProvider` |
| PlaceholderAPI | `PlaceholderManager::register` |
| LuckPerms | `Permission` + `PermissionAttachment` |
| ViaVersion | `ViaPipeline` + `ProtocolTranslator` |
| MultiWorld | `WorldManager` + `WorldCreator` |
| ResourcePack сервера | `ResourcePackManager` + `Player::set_resource_pack` |
| BungeeCord | `Messenger` + `bungeecord:main` |

---

## 16. Защита от утечек — сводка

- `Arc<Plugin>` + `Weak` в каждом колбэке — цикл невозможен.
- `PluginHandle::Drop` → `cancel_all + unsubscribe_all + unregister_all`.
- `SyncRwLock` с `parking_lot` — no poisoning.
- `libloading::Library` закрывается только на `Arc==0`.
- `cargo clippy -Dwarnings` + `pedantic/nursery` — ловит `forget`/`leak`.
