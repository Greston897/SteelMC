//! Минимальный плагин — аналог `JavaPlugin` из Bukkit, но в Rust.
//! Демонстрирует: жизненный цикл, события, команды, права, ViaVersion хук.

use std::sync::Arc;
use steel_api::permission::{Permission, PermissionDefault};
use steel_api::plugin::{Plugin, PluginContext, PluginResult};
use steel_api::prelude::*;
use steel_api::{EventBus, EventPriority, ProtocolVersion, Scheduler, ViaPipeline};

#[derive(Default)]
pub struct HelloWorldPlugin;

impl Plugin for HelloWorldPlugin {
    fn name(&self) -> &str {
        "hello_world"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn on_load(&self, ctx: &PluginContext) -> PluginResult<()> {
        ctx.logger()
            .info("HelloWorld on_load — регистрирую команду /hello");
        ctx.commands()
            .register(steel_api::command::literal("hello").executes(|src, _| {
                src.send_success("Привет от Steel API! 👋");
                Ok(1)
            }))
            .map_err(|e| {
                steel_api::plugin::PluginError::InitFailed("hello_world".into(), e.to_string())
            })?;

        ctx.permissions().register(
            Permission::new("hello_world.use", PermissionDefault::True)
                .with_description("разрешает /hello"),
        );
        Ok(())
    }

    fn on_enable(&self, ctx: &PluginContext) -> PluginResult<()> {
        ctx.logger()
            .info("HelloWorld enabled! Подписываюсь на события");
        let bus = ctx.events().clone();
        bus.subscribe::<PlayerJoinEvent, _>(
            EventPriority::Normal,
            "hello_world".to_string(),
            move |e| {
                log::info!("[HelloWorld] {} зашёл", e.player_name);
            },
        );
        let bus2 = ctx.events().clone();
        bus2.subscribe::<EntityDamageEvent, _>(
            EventPriority::High,
            "hello_world".to_string(),
            |e| {
                if e.cause == "fall" && e.entity_id % 2 == 0 {
                    e.set_cancelled(true);
                }
            },
        );
        let sched = ctx.scheduler().clone();
        sched.run_task_timer(
            "hello_world".to_string(),
            std::time::Duration::from_secs(60),
            || {
                log::info!("[HelloWorld] 60 секунд прошло");
            },
        );
        Ok(())
    }

    fn on_disable(&self, ctx: &PluginContext) {
        ctx.logger().info("HelloWorld выключается — чищу задачи");
        ctx.scheduler().cancel_all_for("hello_world");
    }
}

steel_api::export_plugin!(HelloWorldPlugin);

fn main() {
    let bus = Arc::new(EventBus::new());
    let _sched = Arc::new(Scheduler::new());
    let via = ViaPipeline::new();
    let (id, data) = via
        .translate_inbound(
            ProtocolVersion::V1_21_8,
            ProtocolVersion::V26_3,
            42,
            vec![1, 2, 3],
        )
        .expect("via");
    println!("Via 1.21.8 -> 26.3: id {id} data {data:?}");
    bus.subscribe_simple::<PlayerJoinEvent, _>(EventPriority::Normal, |e| {
        println!("event: {} joined", e.player_name);
    });
    let mut ev = PlayerJoinEvent {
        player_id: uuid::Uuid::nil(),
        player_name: "Notch".into(),
        join_message: Some("Notch joined".into()),
    };
    bus.fire(&mut ev);
    println!("hello_world example OK — API работает!");
}
