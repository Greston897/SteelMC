# steel-api — Steel Plugin API

Bukkit-уровень мощности, свой Rust-дизайн. Каждая механика плагуема, ViaVersion встроен.

См. `../STEEL-API.md` — полная дока на русском (архитектура, все хуки, примеры).

```rust
use steel_api::prelude::*;
use steel_api::plugin::{Plugin, PluginContext, PluginResult};

#[derive(Default)]
pub struct MyPlugin;
impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my" }
    fn version(&self) -> &str { "1.0.0" }
    fn on_enable(&self, ctx: &PluginContext) -> PluginResult<()> {
        ctx.events().subscribe::<PlayerJoinEvent, _>(
            EventPriority::Normal, "my", |e| println!("{}", e.player_name)
        );
        Ok(())
    }
}
```

`cargo run -p steel-api --example hello_world`
