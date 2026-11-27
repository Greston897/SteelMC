//! Продвинутый пример — переопределение КАЖДОЙ механики через API.

#![allow(missing_docs, reason = "example")]
#![allow(clippy::pedantic, reason = "example is illustrative")]
#![allow(clippy::nursery, reason = "example")]
#![allow(clippy::unwrap_used, reason = "example unwraps for brevity")]
#![allow(clippy::print_stdout, reason = "example prints")]
#![allow(clippy::too_many_lines, reason = "example registers many mechanics")]
#![allow(clippy::redundant_clone, reason = "example")]
#![allow(clippy::absolute_paths, reason = "example uses absolute paths for clarity")]
#![allow(clippy::unnecessary_literal_bound, reason = "example plugin name is static")]

use std::sync::Arc;
use steel_api::block::{Block, BlockBehaviorHook};
use steel_api::item::ItemBehaviorHook;
use steel_api::permission::PermissionDefault;
use steel_api::plugin::{Plugin, PluginContext, PluginResult};
use steel_api::prelude::*;
use steel_api::protocol::{ProtocolTranslator, ProtocolVersion, ViaPipeline};
use steel_utils::{BlockPos, Identifier};

struct GlowingOreHook;
impl BlockBehaviorHook for GlowingOreHook {
    fn on_step_on(&self, block: &Block, entity: i32) {
        log::info!("entity {entity} на {:?}", block.pos());
    }
    fn on_break(&self, block: &Block, player: Option<uuid::Uuid>) -> bool {
        log::info!("ломают {:?} игрок {:?}", block.pos(), player);
        true
    }
    fn is_redstone_conductor(&self, _block: &Block) -> Option<bool> {
        Some(false)
    }
}

struct TotemHook;
impl ItemBehaviorHook for TotemHook {
    fn on_use(&self, _stack: &mut ItemStack, player: uuid::Uuid) -> bool {
        log::info!("player {player} использовал тотем");
        true
    }
}

struct MyPacketRemapper {
    from: ProtocolVersion,
    to: ProtocolVersion,
}
impl ProtocolTranslator for MyPacketRemapper {
    fn from(&self) -> ProtocolVersion {
        self.from
    }
    fn to(&self) -> ProtocolVersion {
        self.to
    }
    fn name(&self) -> &'static str {
        "my_remapper"
    }
    fn remap_outbound_id(&self, id: i32) -> i32 {
        if id == 0x2A { 0x2B } else { id }
    }
    fn translate_outbound(
        &self,
        id: i32,
        mut data: Vec<u8>,
    ) -> steel_api::protocol::TranslationResult {
        if id == 0x2B {
            data.push(0xFF);
        }
        Ok((id, data))
    }
}

#[derive(Default)]
pub struct AllMechanicsPlugin;

impl Plugin for AllMechanicsPlugin {
    fn name(&self) -> &str {
        "all_mechanics"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn on_load(&self, ctx: &PluginContext) -> PluginResult<()> {
        ctx.logger()
            .info("AllMechanics on_load — регистрирую команды");
        for cmd in [
            "heal", "feed", "gm", "world", "chunk", "redstone", "summon", "via",
        ] {
            let name = cmd.to_string();
            let n2 = name.clone();
            ctx.commands()
                .register(steel_api::command::literal(cmd).executes(move |src, _| {
                    src.send_success(format!("команда /{n2} выполнена"));
                    Ok(0)
                }))
                .ok();
        }
        Ok(())
    }

    fn on_enable(&self, ctx: &PluginContext) -> PluginResult<()> {
        ctx.events()
            .subscribe::<BlockBreakEvent, _>(EventPriority::Normal, "all_mechanics", |e| {
                if e.block.key == Identifier::new_static("minecraft", "bedrock") {
                    e.set_cancelled(true);
                }
            });
        ctx.events()
            .subscribe::<BlockPlaceEvent, _>(EventPriority::High, "all_mechanics", |e| {
                log::info!("place {:?}", e.pos);
            });
        ctx.events().subscribe::<BlockRedstoneEvent, _>(
            EventPriority::Monitor,
            "all_mechanics",
            |e| {
                log::info!("redstone {:?} {}->{}", e.pos, e.old_power, e.new_power);
            },
        );
        ctx.events().subscribe::<FluidLevelChangeEvent, _>(
            EventPriority::Normal,
            "all_mechanics",
            |e| {
                if e.fluid == Identifier::new_static("minecraft", "lava") {
                    e.set_cancelled(true);
                }
            },
        );
        ctx.events().subscribe::<EntitySpawnEvent, _>(
            EventPriority::Normal,
            "all_mechanics",
            |e| {
                if e.entity_type == "minecraft:creeper" {
                    e.set_cancelled(true);
                }
            },
        );
        ctx.events()
            .subscribe::<EntityDamageEvent, _>(EventPriority::High, "all_mechanics", |e| {
                if e.cause == "void" {
                    e.set_cancelled(true);
                }
            });
        ctx.events().subscribe::<InventoryClickEvent, _>(
            EventPriority::Normal,
            "all_mechanics",
            |e| {
                if e.slot == 0 {
                    e.set_cancelled(true);
                }
            },
        );
        ctx.events()
            .subscribe::<ChunkLoadEvent, _>(EventPriority::Normal, "all_mechanics", |e| {
                log::info!("chunk loaded {:?}", e.pos);
            });
        ctx.events().subscribe::<WeatherChangeEvent, _>(
            EventPriority::Normal,
            "all_mechanics",
            |e| {
                if e.to_storm {
                    e.set_cancelled(true);
                }
            },
        );
        ctx.events()
            .subscribe::<PlayerChatEvent, _>(EventPriority::Normal, "all_mechanics", |e| {
                if e.message.contains("badword") {
                    e.set_cancelled(true);
                }
            });
        ctx.events().subscribe::<ServerTickEvent, _>(
            EventPriority::Monitor,
            "all_mechanics",
            |e| {
                if e.tick % 1200 == 0 {
                    log::info!("tick {}", e.tick);
                }
            },
        );
        ctx.events()
            .subscribe::<PacketReceiveEvent, _>(EventPriority::Low, "all_mechanics", |e| {
                if e.packet_id == 999 {
                    e.set_cancelled(true);
                }
            });
        ctx.scheduler().run_task_timer(
            "all_mechanics",
            std::time::Duration::from_millis(50),
            || {},
        );
        for perm in [
            "all_mechanics.admin",
            "all_mechanics.build",
            "all_mechanics.via.bypass",
        ] {
            ctx.permissions()
                .register(steel_api::permission::Permission::new(
                    perm,
                    PermissionDefault::Op,
                ));
        }
        let via = ViaPipeline::new();
        via.register(Arc::new(MyPacketRemapper {
            from: ProtocolVersion::V1_21_8,
            to: ProtocolVersion::V26_3,
        }));
        via.register(Arc::new(MyPacketRemapper {
            from: ProtocolVersion::V26_3,
            to: ProtocolVersion::V1_21_8,
        }));
        let (id, data) = via
            .translate_outbound(
                ProtocolVersion::V26_3,
                ProtocolVersion::V1_21_8,
                0x2A,
                vec![1, 2],
            )
            .expect("translation should succeed in example");
        assert_eq!(id, 0x2B);
        assert_eq!(data, vec![1, 2, 0xFF]);
        ctx.logger()
            .info("AllMechanics enabled — все механики перехвачены!");
        Ok(())
    }
}

steel_api::export_plugin!(AllMechanicsPlugin);

fn main() {
    println!("custom_mechanics example — все хуки скомпилировались!");
    let _ = GlowingOreHook;
    let _ = TotemHook;
    let _ = BlockPos::new(0, 0, 0);
}
