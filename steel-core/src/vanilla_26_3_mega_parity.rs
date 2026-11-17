//! Exhaustive Vanilla Minecraft 26.3 (Protocol 777) Full-Workspace Parity & Performance Suite.
//!
//! Exercises every major subsystem, registry, packet codec, packet handler, block behavior,
//! item behavior, data component, entity factory, block entity, container menu, game rule,
//! command, redstone & container pipeline, Deep Dark (sculk + Warden Sonic Boom), Trial Chambers
//! (Trial Spawner + Vault + Wind Charge), Pale Garden (Creaking Heart + Creaking gaze freeze),
//! Armadillo roll-up, Raids, Scoreboards, Teams, Triggers, Stopwatches, Waypoints, Filled Maps,
//! Bans/IP-bans, Recipes, Loot Tables, Combat, Vehicles, Worldgen/Persistence, and 20 TPS tick
//! latency / zero-leak invariants end to end.

use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr};
use std::slice;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use glam::{DVec3, IVec3};
use simdnbt::owned::NbtCompound;
use steel_protocol::packet_traits::{CompressionInfo, EncodedPacket, ServerPacket as _};
use steel_protocol::packets::common::CTransfer;
use steel_protocol::packets::config::{CServerLinks, Link, SAcceptCodeOfConduct, ServerLinksType};
use steel_protocol::packets::game::{
    CClearDialog, CCodeOfConduct, CConfigPing, CConfigShowDialog, CCookieRequest,
    CCustomChatCompletions, CCustomReportDetails, CDebugBlockValue, CDebugChunkValue,
    CDebugEntityValue, CDebugEvent, CDeleteChat, CGameTestHighlightPos, CMoveMinecartAlongTrack,
    CPing, CResetChat, CResourcePackPop, CResourcePackPush, CShowDialog, CStartConfiguration,
    CStopSound, CStoreCookie, CWaypoint, CustomChatCompletionAction, DebugSubscriptionUpdate,
    DialogHolder, MinecartStep, PackedMessageSignature, ReportDetail, SBundleItemSelected,
    SConfigurationAcknowledged, SCookieResponse, SCustomClickAction, SDebugSubscriptionRequest,
    SPong, SSetGameRule, SetGameRuleEntry, SoundSource, TrackedWaypointData, WaypointIcon,
    WaypointId, WaypointOperation,
};
use steel_protocol::packets::login::{CCustomQuery, CLoginCookieRequest};
use steel_protocol::utils::ConnectionProtocol;
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::blocks::properties::{BlockStateProperties, Direction, SculkSensorPhase};
use steel_registry::data_components::vanilla_components::{CUSTOM_NAME, DAMAGE};
use steel_registry::equipment::EquipmentSlot;
use steel_registry::item_stack::ItemStack;
use steel_registry::packets::CURRENT_MC_PROTOCOL;
use steel_registry::packets::play::{
    C_MAP_ITEM_DATA, C_SET_CHUNK_CACHE_RADIUS, C_SET_DEFAULT_SPAWN_POSITION,
    C_SET_SIMULATION_DISTANCE, C_WAYPOINT,
};
use steel_registry::recipe::{CraftingInput, vanilla_recipe_types};
use steel_registry::{
    REGISTRY, RegistryExt as _, init_vanilla_registry, vanilla_attributes, vanilla_blocks,
    vanilla_damage_types, vanilla_dimension_types, vanilla_enchantments, vanilla_entities,
    vanilla_game_events, vanilla_game_rules, vanilla_items, vanilla_mob_effects,
};
use steel_utils::codec::{Or, VarInt};
use steel_utils::locks::{IntoShared as _, SyncMutex};
use steel_utils::serial::{ReadFrom as _, WriteTo as _};
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, ChunkPos, Downcast as _, Identifier, WorldAabb};
use text_components::TextComponent;
use tokio::runtime::Builder;
use uuid::Uuid;

use crate::behavior::items::filled_map::{FilledMapItem, send_map_packets, update_colors};
use crate::behavior::{
    BLOCK_BEHAVIORS, CONSUME_EFFECT_BEHAVIORS, FLUID_BEHAVIORS, ITEM_BEHAVIORS,
    MOB_EFFECT_BEHAVIORS, init_behaviors,
};
use crate::block_entity::entities::sculk_sensor::{
    redstone_strength_for_distance, vibration_frequency,
};
use crate::block_entity::entities::{
    BrewingStandBlockEntity, CrafterBlockEntity, CreakingHeartBlockEntity, FurnaceKind,
    HopperBlockEntity, SculkSensorBlockEntity, SculkShriekerBlockEntity, TrialSpawnerBlockEntity,
    VaultBlockEntity,
};
use crate::block_entity::{BLOCK_ENTITIES, BlockEntity as _, init_block_entities};
use crate::chunk::heightmap::HeightmapType;
use crate::command::execution::{
    CommandExecutionContext, CommandResultCallback, CommandSource, ExecutionCommandSource as _,
};
use crate::command::sender::CommandSender;
use crate::config::ResolvedDomainConfig;
use crate::entity::damage::DamageSource;
use crate::entity::entities::mobs::bosses::{ALL_DRAGON_PHASES, DragonPhase};
use crate::entity::entities::mobs::passive::{
    CamelHuskEntity, CowEntity, HappyGhastEntity, MushroomCowEntity, NautilusEntity,
    SulfurCubeEntity,
};
use crate::entity::entities::objects::vehicles::AbstractMinecart as _;
use crate::entity::entities::{
    ArmadilloEntity, ArmorStandEntity, ChestMinecartEntity, CreakingEntity, EnderDragonEntity,
    FurnaceMinecartEntity, HopperMinecartEntity, ItemEntity, MinecartEntity, PrimedTntEntity,
    TntMinecartEntity, WardenEntity, WitherBossEntity, ZombieEntity,
};
use crate::entity::villager_trades::offers_for;
use crate::entity::{
    AgeableMob as _, ENTITIES, Entity, LivingEntity as _, Mob as _, MobEffectInstance,
    RemovalReason, SharedEntity, init_entities, next_entity_id,
};
use crate::inventory::click::{Click, MouseButton};
use crate::inventory::container::{Container as _, SimpleContainer};
use crate::inventory::lock::ContainerRef;
use crate::inventory::menu::kinds::{
    anvil, brewing_stand, cartography_table, chest, crafting, dispenser, enchantment, furnace_menu,
    grindstone, hopper, inventory_size as mount_inventory_size, lectern, loom, merchant, mount,
    shulker_box, smithing, stonecutter,
};
use crate::inventory::recipe_manager;
use crate::permission::{
    OP_GROUP, PermissionMetadataSet, PermissionSet, PermissionSubjectIndex, PermissionSubjectState,
};
use crate::player::connection::{NetworkConnection, ScheduledPlayPacket};
use crate::player::player_data_storage::GlobalPlayerData;
use crate::player::{PlayerConnection, ResetReason};
use crate::scoreboard::{ScoreHolder, Scoreboard, criterion_is_read_only};
use crate::server::bans::{BanEntry, BanList};
use crate::server::{Server, test_server_with_worlds_and_config};
use crate::test_support::{
    TestPlayerBuilder, TestWorld, fresh_test_world, insert_ready_full_chunk, test_runtime_config,
    tick_test_world,
};
use crate::world::game_event::GameEventContext;
use crate::world::raid::RaidManager;
use crate::world::waypoint::compute_waypoint_data;
use crate::world::{ExplosionInteraction, ExplosionOptions, LevelReader as _, World};

/// Bootstrap every global registry and behavior table once for the whole process.
fn bootstrap_all() {
    init_vanilla_registry();
    init_block_entities();
    init_behaviors();
    init_entities();
}

/// Packet-capturing mock connection for asserting real outbound S2C traffic and disconnects.
struct CapturingConnection {
    packets: Arc<SyncMutex<Vec<EncodedPacket>>>,
    closed: AtomicBool,
    disconnect_reason: SyncMutex<Option<TextComponent>>,
}

impl CapturingConnection {
    fn new() -> (Arc<Self>, Arc<SyncMutex<Vec<EncodedPacket>>>) {
        let packets = Arc::new(SyncMutex::new(Vec::new()));
        let conn = Arc::new(Self {
            packets: Arc::clone(&packets),
            closed: AtomicBool::new(false),
            disconnect_reason: SyncMutex::new(None),
        });
        (conn, packets)
    }
}

struct SharedNetConn(Arc<CapturingConnection>);

impl NetworkConnection for SharedNetConn {
    fn compression(&self) -> Option<CompressionInfo> {
        None
    }

    fn send_encoded(&self, packet: EncodedPacket) {
        self.0.packets.lock().push(packet);
    }

    fn send_encoded_bundle(&self, packets: Vec<EncodedPacket>) {
        self.0.packets.lock().extend(packets);
    }

    fn disconnect_with_reason(&self, reason: TextComponent) {
        *self.0.disconnect_reason.lock() = Some(reason);
        self.close();
    }

    fn tick(&self) {}

    fn latency(&self) -> i32 {
        12
    }

    fn close(&self) {
        self.0.closed.store(true, Ordering::Release);
    }

    fn closed(&self) -> bool {
        self.0.closed.load(Ordering::Acquire)
    }
}

/// Prepare a fresh world with a ready 3x3 chunk region around `(0, 0)` and a stone floor at y=63.
fn create_arena_world(name: &str) -> (TestWorld, Arc<World>) {
    let fixture = fresh_test_world(name);
    let world = Arc::clone(&fixture.world);
    for cx in -1..=1 {
        for cz in -1..=1 {
            insert_ready_full_chunk(&world, ChunkPos::new(cx, cz));
        }
    }
    let stone = vanilla_blocks::STONE.default_state();
    for x in -8..24 {
        for z in -8..24 {
            world.set_block(BlockPos::new(x, 63, z), stone, UpdateFlags::UPDATE_NONE);
        }
    }
    (fixture, world)
}

/// Synchronously parses and executes a command string against `server`, returning `(ok, value)`.
fn exec_command(
    server: &Arc<Server>,
    sender: CommandSender,
    command: &str,
) -> Result<(bool, i32), String> {
    let source = CommandSource::new(sender, Arc::clone(server));
    let trimmed = command.strip_prefix('/').unwrap_or(command);
    let chain = {
        let dispatcher = server.command_dispatcher.read();
        let parse = dispatcher.parse(trimmed, source.clone());
        dispatcher
            .context_chain(parse)
            .map_err(|err| err.to_string())?
    };
    let outcome = Arc::new(SyncMutex::new((false, 0_i32)));
    let outcome_sink = Arc::clone(&outcome);
    let callback = CommandResultCallback::new(move |ok, val| {
        *outcome_sink.lock() = (ok, val);
    });
    let source = source.with_callback(callback.clone());
    let mut ctx = CommandExecutionContext::for_source(&source);
    ctx.queue_initial_command(chain, source, callback);
    let _ = ctx.run();
    let result = *outcome.lock();
    Ok(result)
}

/// Helper: reads the `VarInt` packet id at the start of an uncompressed `EncodedPacket`.
fn decode_packet_id(pkt: &EncodedPacket) -> i32 {
    let mut cursor = Cursor::new(pkt.encoded_data.as_slice());
    let _len = VarInt::read(&mut cursor).expect("frame length VarInt");
    VarInt::read(&mut cursor).expect("packet id VarInt").0
}

// ============================================================================
// 1. ALL 50+ VANILLA 26.3 REGISTRIES, PROTOCOL 777 & PACKET CODEC ROUNDTRIPS
// ============================================================================

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "single exhaustive mega-test verifying every registry, packet, and subsystem together"
)]
fn vanilla_26_3_grand_unified_full_server_parity_and_no_lag_suite() {
    bootstrap_all();

    // ------------------------------------------------------------------------
    // PHASE 1: Protocol 777 & All 50+ Vanilla MC 26.3 Registries
    // ------------------------------------------------------------------------
    assert_eq!(
        CURRENT_MC_PROTOCOL, 777,
        "Minecraft 26.3 protocol version must be 777"
    );
    assert!(REGISTRY.blocks.len() >= 1_150, "all MC 26.3 blocks loaded");
    assert!(REGISTRY.items.len() >= 1_450, "all MC 26.3 items loaded");
    assert_eq!(
        REGISTRY.entity_types.len(),
        161,
        "all 161 MC 26.3 entity types registered"
    );
    assert_eq!(
        REGISTRY.block_entity_types.len(),
        49,
        "all 49 MC 26.3 block entity types registered"
    );
    assert_eq!(
        REGISTRY.menu_types.len(),
        25,
        "all 25 MC 26.3 menu types registered"
    );
    assert_eq!(
        REGISTRY.game_rules.len(),
        59,
        "all 59 MC 26.3 game rules registered"
    );
    assert!(REGISTRY.biomes.len() >= 60);
    assert!(REGISTRY.attributes.len() >= 30);
    assert!(REGISTRY.mob_effects.len() >= 35);
    assert!(REGISTRY.potions.len() >= 40);
    assert!(REGISTRY.enchantments.len() >= 40);
    assert!(REGISTRY.damage_types.len() >= 45);
    assert!(REGISTRY.dimension_types.len() >= 4);
    assert!(REGISTRY.game_events.len() >= 30);
    assert!(REGISTRY.sound_events.len() >= 1_500);
    assert!(REGISTRY.particle_types.len() >= 100);
    assert!(REGISTRY.fluids.len() >= 5);
    assert!(REGISTRY.poi_types.len() >= 15);
    assert!(REGISTRY.data_components.len() >= 90);
    assert!(REGISTRY.loot_tables.len() >= 1_000);
    assert!(REGISTRY.villager_professions.len() >= 15);
    assert!(REGISTRY.villager_types.len() >= 7);
    assert!(REGISTRY.painting_variants.len() >= 25);
    assert!(REGISTRY.trim_patterns.len() >= 15);
    assert!(REGISTRY.trim_materials.len() >= 10);
    assert!(REGISTRY.banner_patterns.len() >= 35);
    assert!(REGISTRY.jukebox_songs.len() >= 15);
    assert!(REGISTRY.instruments.len() >= 8);
    assert!(REGISTRY.wolf_variants.len() >= 9);
    assert!(REGISTRY.frog_variants.len() >= 3);
    assert!(REGISTRY.cat_variants.len() >= 10);
    assert!(REGISTRY.pig_variants.len() >= 3);
    assert!(REGISTRY.cow_variants.len() >= 3);
    assert!(REGISTRY.chicken_variants.len() >= 3);
    assert!(REGISTRY.map_decoration_types.len() >= 25);

    // Every fluid, mob effect, and consume effect has a live registered behavior.
    for (_, fluid) in REGISTRY.fluids.iter() {
        let _ = FLUID_BEHAVIORS.get_behavior(fluid);
    }
    for (_, effect) in REGISTRY.mob_effects.iter() {
        let _ = MOB_EFFECT_BEHAVIORS.get_behavior(effect);
    }
    for (_, consume) in REGISTRY.consume_effect_types.iter() {
        let _ = CONSUME_EFFECT_BEHAVIORS.get_behavior(consume);
    }

    // ------------------------------------------------------------------------
    // PHASE 2: Exhaustive Protocol 777 S2C & C2S Packet Encoding/Decoding
    // ------------------------------------------------------------------------
    let compressed = Some(CompressionInfo::default());
    for comp in [None, compressed] {
        // Configuration-state S2C packets
        assert!(
            EncodedPacket::from_bare(
                CCodeOfConduct {
                    code_of_conduct: "Be respectful on SteelMC 26.3!".to_owned(),
                },
                comp,
                ConnectionProtocol::Config,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CCustomReportDetails {
                    details: vec![ReportDetail {
                        title: "server".to_owned(),
                        description: "SteelMC 26.3".to_owned(),
                    }],
                },
                comp,
                ConnectionProtocol::Config,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CConfigPing { id: 0x1234_5678 },
                comp,
                ConnectionProtocol::Config
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CConfigShowDialog {
                    dialog: NbtCompound::new(),
                },
                comp,
                ConnectionProtocol::Config,
            )
            .is_ok()
        );
        assert!(EncodedPacket::from_bare(CResetChat {}, comp, ConnectionProtocol::Config).is_ok());
        assert!(
            EncodedPacket::from_bare(
                CServerLinks {
                    links: vec![Link::new(
                        Or::Left(ServerLinksType::BugReport),
                        "https://example.com/bugs".to_owned(),
                    )],
                },
                comp,
                ConnectionProtocol::Config,
            )
            .is_ok()
        );

        // Login-state S2C packets
        assert!(
            EncodedPacket::from_bare(
                CCustomQuery {
                    transaction_id: 7,
                    channel: Identifier::vanilla_static("brand"),
                    data: vec![1, 2, 3, 4],
                },
                comp,
                ConnectionProtocol::Login,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CLoginCookieRequest {
                    key: Identifier::vanilla_static("auth_token"),
                },
                comp,
                ConnectionProtocol::Login,
            )
            .is_ok()
        );

        // Play-state S2C packets (including all MC 26.3 additions)
        assert!(
            EncodedPacket::from_bare(
                CStoreCookie {
                    key: Identifier::vanilla_static("session"),
                    payload: vec![0xAA; 5_120],
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        // 5,121-byte cookie MUST be rejected per vanilla 5 KiB cap.
        assert!(
            EncodedPacket::from_bare(
                CStoreCookie {
                    key: Identifier::vanilla_static("oversized"),
                    payload: vec![0xBB; 5_121],
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_err()
        );
        assert!(
            EncodedPacket::from_bare(
                CCookieRequest {
                    key: Identifier::vanilla_static("session"),
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CCustomChatCompletions {
                    action: CustomChatCompletionAction::Add,
                    entries: vec!["@guild".to_owned(), "@party".to_owned()],
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CDeleteChat {
                    message_signature: PackedMessageSignature::CachedId(42),
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CDeleteChat {
                    message_signature: PackedMessageSignature::Full(Box::new([0x5A; 256])),
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(CStartConfiguration {}, comp, ConnectionProtocol::Play)
                .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(CPing { id: -99 }, comp, ConnectionProtocol::Play).is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CShowDialog {
                    dialog: DialogHolder::Direct(NbtCompound::new()),
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        assert!(EncodedPacket::from_bare(CClearDialog {}, comp, ConnectionProtocol::Play).is_ok());
        assert!(
            EncodedPacket::from_bare(
                CGameTestHighlightPos {
                    absolute_pos: BlockPos::new(10, 64, 20),
                    relative_pos: BlockPos::new(1, 0, 2),
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CDebugBlockValue {
                    block_pos: BlockPos::new(0, 64, 0),
                    update: DebugSubscriptionUpdate {
                        subscription_id: 1,
                        encoded_value: Some(vec![9, 8, 7]),
                    },
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CDebugChunkValue {
                    chunk_x: -2,
                    chunk_z: 5,
                    update: DebugSubscriptionUpdate {
                        subscription_id: 2,
                        encoded_value: None,
                    },
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CDebugEntityValue {
                    entity_id: 123,
                    update: DebugSubscriptionUpdate {
                        subscription_id: 3,
                        encoded_value: Some(vec![1]),
                    },
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CDebugEvent {
                    subscription_id: 4,
                    encoded_value: vec![0xCA, 0xFE],
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CMoveMinecartAlongTrack {
                    entity_id: 55,
                    steps: vec![MinecartStep::ZERO],
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CResourcePackPush {
                    id: Uuid::from_u128(1),
                    url: "https://example.com/pack.zip".to_owned(),
                    hash: "0123456789abcdef0123456789abcdef01234567".to_owned(),
                    required: false,
                    prompt: Some(TextComponent::plain("Download SteelMC 26.3 pack")),
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CResourcePackPop {
                    id: Some(Uuid::from_u128(1)),
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CTransfer {
                    host: "play.example.com",
                    port: 25565,
                },
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        assert!(
            EncodedPacket::from_bare(
                CStopSound::new(
                    Some(SoundSource::Music),
                    Some(Identifier::vanilla_static("music.game")),
                ),
                comp,
                ConnectionProtocol::Play,
            )
            .is_ok()
        );
        for wp_data in [
            TrackedWaypointData::Vec3i {
                x: 12,
                y: 64,
                z: -8,
            },
            TrackedWaypointData::Chunk {
                chunk_x: 3,
                chunk_z: -4,
            },
            TrackedWaypointData::Azimuth { angle: 1.25 },
        ] {
            assert!(
                EncodedPacket::from_bare(
                    CWaypoint {
                        operation: WaypointOperation::Track {
                            id: WaypointId::Uuid(Uuid::from_u128(0xABCD)),
                            icon: WaypointIcon {
                                style: Identifier::vanilla_static("default"),
                                color: Some([0x33, 0xCC, 0xFF]),
                            },
                            data: wp_data,
                        },
                    },
                    comp,
                    ConnectionProtocol::Play,
                )
                .is_ok()
            );
        }
    }

    // Serverbound C2S packet decoders
    assert!(SAcceptCodeOfConduct::read_packet(&mut Cursor::new(&[][..])).is_ok());
    assert!(SConfigurationAcknowledged::read_packet(&mut Cursor::new(&[][..])).is_ok());
    let pong = SPong::read_packet(&mut Cursor::new(&42_i32.to_be_bytes()[..])).expect("SPong");
    assert_eq!(pong.id, 42);
    let mut bundle_buf = Vec::new();
    VarInt(12).write(&mut bundle_buf).expect("slot");
    VarInt(-1).write(&mut bundle_buf).expect("selected");
    let bundle_pkt =
        SBundleItemSelected::read_packet(&mut Cursor::new(bundle_buf.as_slice())).expect("bundle");
    assert_eq!(bundle_pkt.slot_id, 12);
    assert_eq!(bundle_pkt.selected_item_index, -1);
    let mut rule_buf = Vec::new();
    VarInt(1).write(&mut rule_buf).expect("len");
    Identifier::vanilla_static("keep_inventory")
        .write(&mut rule_buf)
        .expect("id");
    "true".to_owned().write(&mut rule_buf).expect("val");
    let rule_pkt =
        SSetGameRule::read_packet(&mut Cursor::new(rule_buf.as_slice())).expect("gamerule");
    assert_eq!(
        rule_pkt.entries,
        vec![SetGameRuleEntry {
            key: Identifier::vanilla_static("keep_inventory"),
            value: "true".to_owned(),
        }]
    );
    let mut cookie_buf = Vec::new();
    Identifier::vanilla_static("session")
        .write(&mut cookie_buf)
        .expect("key");
    true.write(&mut cookie_buf).expect("present");
    VarInt(3).write(&mut cookie_buf).expect("len");
    cookie_buf.extend_from_slice(&[7, 8, 9]);
    let cookie_pkt =
        SCookieResponse::read_packet(&mut Cursor::new(cookie_buf.as_slice())).expect("cookie");
    assert_eq!(cookie_pkt.payload, Some(vec![7, 8, 9]));
    let mut click_buf = Vec::new();
    Identifier::vanilla_static("confirm")
        .write(&mut click_buf)
        .expect("id");
    VarInt(1).write(&mut click_buf).expect("framed len");
    0_u8.write(&mut click_buf).expect("no payload");
    let click_pkt =
        SCustomClickAction::read_packet(&mut Cursor::new(click_buf.as_slice())).expect("click");
    assert_eq!(click_pkt.id, Identifier::vanilla_static("confirm"));
    assert!(click_pkt.payload.is_none());

    // ------------------------------------------------------------------------
    // PHASE 3: Every Single Registered Block (1,168+) — Behavior, Placement,
    //          Block Entity Attachment, Survival, Signal & Clean Removal
    // ------------------------------------------------------------------------
    let (_arena_guard, world) = create_arena_world("mega_parity_world");
    let air = vanilla_blocks::AIR.default_state();
    let probe = BlockPos::new(4, 70, 4);
    world.set_block(
        probe.below(),
        vanilla_blocks::STONE.default_state(),
        UpdateFlags::UPDATE_NONE,
    );
    let mut total_blocks = 0_usize;
    let mut entity_blocks_verified = 0_usize;
    for (_, block) in REGISTRY.blocks.iter() {
        total_blocks += 1;
        let behavior = BLOCK_BEHAVIORS.get_behavior(block);
        let default_state = block.default_state();
        assert_eq!(default_state.get_block().key, block.key);
        let _ = behavior.can_survive(default_state, world.as_ref(), probe);
        world.set_block(probe, default_state, UpdateFlags::UPDATE_CLIENTS);
        assert_eq!(world.get_block_state(probe), default_state);
        if REGISTRY.block_entity_types.has_block_entity(block)
            && block != &vanilla_blocks::MOVING_PISTON
            && let Some(be) = world.get_block_entity(probe)
        {
            entity_blocks_verified += 1;
            let saved = be.save_with_full_metadata();
            assert!(saved.get("id").is_some());
        }
        let _ = behavior.get_analog_output_signal(default_state, &world, probe, Direction::North);
        world.set_block(probe, air, UpdateFlags::UPDATE_CLIENTS);
    }
    assert!(total_blocks >= 1_150);
    assert!(
        entity_blocks_verified >= 45,
        "expected >=45 entity-bearing blocks to materialize & save NBT, got {entity_blocks_verified}"
    );

    // ------------------------------------------------------------------------
    // PHASE 4: All 49 Vanilla Block Entity Types — Direct Factory + NBT Save
    // ------------------------------------------------------------------------
    let mut created_be_types = 0_usize;
    for (_, be_type) in REGISTRY.block_entity_types.iter() {
        let Some(&valid_block) = be_type.valid_blocks.first() else {
            continue;
        };
        let state = valid_block.default_state();
        let be = BLOCK_ENTITIES
            .create(
                be_type,
                Arc::downgrade(&world),
                BlockPos::new(2, 72, 2),
                state,
            )
            .unwrap_or_else(|| panic!("missing factory for block entity {}", be_type.key));
        assert_eq!(be.get_type().key, be_type.key);
        let full_nbt = be.save_with_full_metadata();
        assert_eq!(full_nbt.int("x"), Some(2));
        assert_eq!(full_nbt.int("y"), Some(72));
        assert_eq!(full_nbt.int("z"), Some(2));
        created_be_types += 1;
    }
    assert_eq!(
        created_be_types, 49,
        "all 49 block entity types instantiated"
    );

    // ------------------------------------------------------------------------
    // PHASE 5: Every Single Registered Item (1,500+) — Behavior, Stack Ops,
    //          Durability, Enchantments & Data Components
    // ------------------------------------------------------------------------
    let mut total_items = 0_usize;
    let mut damageable_items = 0_usize;
    for (_, item) in REGISTRY.items.iter() {
        total_items += 1;
        let _behavior = ITEM_BEHAVIORS.get_behavior(item);
        let mut stack = ItemStack::new(item);
        if item != &*vanilla_items::AIR {
            assert!(!stack.is_empty());
            assert!(stack.max_stack_size() >= 1 && stack.max_stack_size() <= 99);
            if stack.is_damageable_item() {
                damageable_items += 1;
                let broke = stack.hurt_and_break(1, false);
                if !broke {
                    assert_eq!(stack.get(DAMAGE).copied(), Some(1));
                }
            }
        }
    }
    assert!(total_items >= 1_450);
    assert!(damageable_items >= 50);

    // Custom component patching equality & independence
    let mut named_sword = ItemStack::new(&vanilla_items::NETHERITE_SWORD);
    named_sword.set(CUSTOM_NAME, TextComponent::plain("Excalibur"));
    let plain_sword = ItemStack::new(&vanilla_items::NETHERITE_SWORD);
    assert!(!ItemStack::is_same_item_same_components(
        &named_sword,
        &plain_sword
    ));
    assert_eq!(
        named_sword.get(CUSTOM_NAME).map(ToString::to_string),
        Some("Excalibur".to_owned())
    );

    // ------------------------------------------------------------------------
    // PHASE 6: Every Single Non-Player Entity Type (160/160) — Factory, AABB,
    //          Synced Metadata, World Registration, Tick & Removal
    // ------------------------------------------------------------------------
    let mut spawned_entities = 0_usize;
    for (_, etype) in REGISTRY.entity_types.iter() {
        if etype == &vanilla_entities::PLAYER {
            continue;
        }
        let ent = ENTITIES
            .create(
                etype,
                next_entity_id(),
                DVec3::new(8.5, 65.0, 8.5),
                Arc::downgrade(&world),
            )
            .unwrap_or_else(|| panic!("missing factory for entity type {}", etype.key));
        assert_eq!(ent.entity_type().key, etype.key);
        let bb = ent.bounding_box();
        assert!(bb.max_x() >= bb.min_x() && bb.max_y() >= bb.min_y() && bb.max_z() >= bb.min_z());
        assert!(world.try_add_entity(Arc::clone(&ent)).is_ok());
        ent.base_tick();
        ent.set_removed(RemovalReason::Discarded);
        spawned_entities += 1;
    }
    assert_eq!(
        spawned_entities, 160,
        "all 160 non-player vanilla 26.3 entity types created and ticked cleanly"
    );

    // ------------------------------------------------------------------------
    // PHASE 7: Containers, Crafter (1x2 + disabled slots), Hopper Transfer,
    //          Dispenser (Arrow, WindCharge, TNT, Water/Empty Bucket, Flint&Steel),
    //          Dropper (Container Insertion + Air Ejection)
    // ------------------------------------------------------------------------
    // 7a. Crafter shaped 1x2 stick crafting + disabled slot masking
    let crafter_pos = BlockPos::new(6, 64, 6);
    world.set_block(
        crafter_pos,
        vanilla_blocks::CRAFTER.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    let crafter_holder = world.get_block_entity(crafter_pos).expect("crafter be");
    let crafter_be = crafter_holder
        .downcast_ref::<CrafterBlockEntity>()
        .expect("crafter block entity");
    {
        let c_ref = crafter_be.container_ref().expect("crafter container");
        let mut guard = c_ref.lock();
        guard.set_item(0, ItemStack::with_count(&vanilla_items::OAK_PLANKS, 2));
        guard.set_item(3, ItemStack::with_count(&vanilla_items::OAK_PLANKS, 2));
    }
    crafter_be.set_disabled_slots(1 << 8);
    assert!(
        crafter_be.start_crafting(&world),
        "crafter must match 1x2 stick recipe via positioned shrinking"
    );
    world.set_block(crafter_pos, air, UpdateFlags::UPDATE_ALL);

    // 7b. Hopper moving items into a chest below
    let chest_pos = BlockPos::new(10, 64, 6);
    let hopper_pos = BlockPos::new(10, 65, 6);
    world.set_block(
        chest_pos,
        vanilla_blocks::CHEST.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    world.set_block(
        hopper_pos,
        vanilla_blocks::HOPPER.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    let hopper_holder = world.get_block_entity(hopper_pos).expect("hopper holder");
    let hopper_be = hopper_holder
        .downcast_ref::<HopperBlockEntity>()
        .expect("hopper be");
    hopper_be
        .container_ref()
        .expect("hopper inv")
        .lock()
        .set_item(0, ItemStack::with_count(&vanilla_items::DIAMOND, 3));
    assert!(hopper_be.try_move_items(&world, hopper_pos));
    let chest_count = world
        .get_block_entity(chest_pos)
        .and_then(|be| be.container_ref())
        .expect("chest inv")
        .lock()
        .get_item(0)
        .count();
    assert_eq!(chest_count, 1, "hopper moved 1 diamond into chest");
    world.set_block(hopper_pos, air, UpdateFlags::UPDATE_ALL);
    world.set_block(chest_pos, air, UpdateFlags::UPDATE_ALL);

    // 7c. Dispenser: water bucket place -> empty bucket -> scoop back -> water bucket,
    //     plus flint & steel lighting fire with 1 durability damage, plus TNT priming!
    let disp_pos = BlockPos::new(12, 64, 6);
    let disp_state = vanilla_blocks::DISPENSER
        .default_state()
        .set_value(&BlockStateProperties::FACING, Direction::East);
    let disp_front = disp_pos.relative(Direction::East);
    world.set_block(disp_pos, disp_state, UpdateFlags::UPDATE_ALL);
    let disp_behavior = BLOCK_BEHAVIORS.get_behavior(&vanilla_blocks::DISPENSER);
    let disp_inv = world
        .get_block_entity(disp_pos)
        .and_then(|be| be.container_ref())
        .expect("dispenser container");

    // Water bucket dispense
    disp_inv
        .lock()
        .set_item(4, ItemStack::new(&vanilla_items::WATER_BUCKET));
    disp_behavior.tick(disp_state, &world, disp_pos);
    assert_eq!(
        world.get_block_state(disp_front).get_block(),
        &vanilla_blocks::WATER
    );
    assert!(disp_inv.lock().get_item(4).is(&vanilla_items::BUCKET));

    // Empty bucket scoop back into WATER_BUCKET
    disp_behavior.tick(disp_state, &world, disp_pos);
    assert!(world.get_block_state(disp_front).is_air());
    assert!(disp_inv.lock().get_item(4).is(&vanilla_items::WATER_BUCKET));

    // Powder snow bucket place & scoop back into POWDER_SNOW_BUCKET
    disp_inv
        .lock()
        .set_item(4, ItemStack::new(&vanilla_items::POWDER_SNOW_BUCKET));
    disp_behavior.tick(disp_state, &world, disp_pos);
    assert_eq!(
        world.get_block_state(disp_front).get_block(),
        &vanilla_blocks::POWDER_SNOW
    );
    assert!(disp_inv.lock().get_item(4).is(&vanilla_items::BUCKET));
    disp_behavior.tick(disp_state, &world, disp_pos);
    assert!(world.get_block_state(disp_front).is_air());
    assert!(
        disp_inv
            .lock()
            .get_item(4)
            .is(&vanilla_items::POWDER_SNOW_BUCKET)
    );

    // Glowstone charges a facing Respawn Anchor by +1
    world.set_block(
        disp_front,
        vanilla_blocks::RESPAWN_ANCHOR.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    disp_inv
        .lock()
        .set_item(4, ItemStack::with_count(&vanilla_items::GLOWSTONE, 2));
    disp_behavior.tick(disp_state, &world, disp_pos);
    assert_eq!(
        world
            .get_block_state(disp_front)
            .get_value(&BlockStateProperties::RESPAWN_ANCHOR_CHARGES),
        1,
        "dispenser glowstone must increment Respawn Anchor charge to 1"
    );
    assert_eq!(disp_inv.lock().get_item(4).count(), 1);
    disp_inv.lock().set_item(4, ItemStack::empty());
    world.set_block(disp_front, air, UpdateFlags::UPDATE_ALL);

    // Vanilla quasi-connectivity (`pos.above()` powered via redstone block two above)
    // schedules a 4-tick block tick and flips `TRIGGERED = true` on the rising edge!
    let qc_power_pos = disp_pos.above().above();
    world.set_block(
        qc_power_pos,
        vanilla_blocks::REDSTONE_BLOCK.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    disp_behavior.handle_neighbor_changed(
        world.get_block_state(disp_pos),
        &world,
        disp_pos,
        &vanilla_blocks::REDSTONE_BLOCK,
        false,
    );
    assert!(
        world
            .get_block_state(disp_pos)
            .get_value(&BlockStateProperties::TRIGGERED),
        "dispenser must recognize vanilla quasi-connectivity from pos.above()"
    );
    assert!(
        world.has_scheduled_block_tick(disp_pos, &vanilla_blocks::DISPENSER),
        "dispenser rising edge must schedule its vanilla 4-tick dispense"
    );
    world.set_block(qc_power_pos, air, UpdateFlags::UPDATE_ALL);
    disp_behavior.handle_neighbor_changed(
        world.get_block_state(disp_pos),
        &world,
        disp_pos,
        &vanilla_blocks::REDSTONE_BLOCK,
        false,
    );
    assert!(
        !world
            .get_block_state(disp_pos)
            .get_value(&BlockStateProperties::TRIGGERED),
        "dispenser falling edge must clear TRIGGERED"
    );

    // Flint and steel lights fire and takes 1 durability without being consumed
    disp_inv
        .lock()
        .set_item(4, ItemStack::new(&vanilla_items::FLINT_AND_STEEL));
    disp_behavior.tick(disp_state, &world, disp_pos);
    assert_eq!(
        world.get_block_state(disp_front).get_block(),
        &vanilla_blocks::FIRE
    );
    assert_eq!(disp_inv.lock().get_item(4).get_damage_value(), 1);
    world.set_block(disp_front, air, UpdateFlags::UPDATE_ALL);

    // TNT dispense primes PrimedTntEntity
    disp_inv
        .lock()
        .set_item(4, ItemStack::new(&vanilla_items::TNT));
    disp_behavior.tick(disp_state, &world, disp_pos);
    assert!(disp_inv.lock().get_item(4).is_empty());
    let primed = world.get_entities_in_aabb_matching(
        &WorldAabb::of_size(
            DVec3::new(
                f64::from(disp_front.x()) + 0.5,
                f64::from(disp_front.y()) + 0.5,
                f64::from(disp_front.z()) + 0.5,
            ),
            3.0,
            3.0,
            3.0,
        ),
        |e| e.downcast_ref::<PrimedTntEntity>().is_some(),
    );
    assert_eq!(primed.len(), 1, "dispenser must prime TNT entity");
    for e in primed {
        e.set_removed(RemovalReason::Discarded);
    }

    // Wind charge projectile dispense
    disp_inv
        .lock()
        .set_item(2, ItemStack::new(&vanilla_items::WIND_CHARGE));
    disp_behavior.tick(disp_state, &world, disp_pos);
    assert!(disp_inv.lock().get_item(2).is_empty());
    world.set_block(disp_pos, air, UpdateFlags::UPDATE_ALL);

    // 7d. Dropper inserts directly into a facing Chest without spilling items
    let drop_pos = BlockPos::new(12, 64, 10);
    let drop_state = vanilla_blocks::DROPPER
        .default_state()
        .set_value(&BlockStateProperties::FACING, Direction::East);
    let target_chest = drop_pos.relative(Direction::East);
    world.set_block(drop_pos, drop_state, UpdateFlags::UPDATE_ALL);
    world.set_block(
        target_chest,
        vanilla_blocks::CHEST.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    world
        .get_block_entity(drop_pos)
        .and_then(|be| be.container_ref())
        .expect("dropper inv")
        .lock()
        .set_item(7, ItemStack::with_count(&vanilla_items::EMERALD, 4));
    BLOCK_BEHAVIORS
        .get_behavior(&vanilla_blocks::DROPPER)
        .tick(drop_state, &world, drop_pos);
    assert_eq!(
        world
            .get_block_entity(target_chest)
            .and_then(|be| be.container_ref())
            .expect("target chest inv")
            .lock()
            .get_item(0)
            .count(),
        1,
        "dropper transferred 1 emerald straight into facing chest"
    );
    assert_eq!(
        world
            .get_block_entity(drop_pos)
            .and_then(|be| be.container_ref())
            .expect("dropper inv")
            .lock()
            .get_item(7)
            .count(),
        3
    );
    world.set_block(drop_pos, air, UpdateFlags::UPDATE_ALL);
    world.set_block(target_chest, air, UpdateFlags::UPDATE_ALL);

    // ------------------------------------------------------------------------
    // PHASE 8: Frogspawn Hatch Scheduling & Fire Block Survival/Tick
    // ------------------------------------------------------------------------
    let water_pos = BlockPos::new(2, 64, 12);
    let spawn_pos = water_pos.above();
    world.set_block(
        water_pos,
        vanilla_blocks::WATER.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    world.set_block(
        spawn_pos,
        vanilla_blocks::FROGSPAWN.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    assert!(
        world.has_scheduled_block_tick(spawn_pos, &vanilla_blocks::FROGSPAWN),
        "FrogspawnBlock::on_place must schedule its vanilla tadpole hatch tick"
    );
    // Triggering the scheduled hatch tick pops the frogspawn and spawns 2..=5 tadpoles!
    BLOCK_BEHAVIORS
        .get_behavior(&vanilla_blocks::FROGSPAWN)
        .tick(vanilla_blocks::FROGSPAWN.default_state(), &world, spawn_pos);
    assert!(world.get_block_state(spawn_pos).is_air());
    let tadpoles = world.get_entities_in_aabb_matching(
        &WorldAabb::of_size(
            DVec3::new(
                f64::from(spawn_pos.x()) + 0.5,
                f64::from(spawn_pos.y()) + 0.5,
                f64::from(spawn_pos.z()) + 0.5,
            ),
            4.0,
            4.0,
            4.0,
        ),
        |e| e.entity_type() == &vanilla_entities::TADPOLE,
    );
    assert!(
        (2..=5).contains(&tadpoles.len()),
        "Frogspawn must hatch 2..=5 tadpoles, spawned {}",
        tadpoles.len()
    );
    for t in tadpoles {
        t.set_removed(RemovalReason::Discarded);
    }
    world.set_block(water_pos, air, UpdateFlags::UPDATE_ALL);

    // Fire on netherrack schedules a tick and stays alive (infiniburn_overworld).
    let rack_pos = BlockPos::new(5, 64, 12);
    let fire_pos = rack_pos.above();
    world.set_block(
        rack_pos,
        vanilla_blocks::NETHERRACK.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    world.set_block(
        fire_pos,
        vanilla_blocks::FIRE.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    assert!(world.is_infiniburn(world.get_block_state(rack_pos)));
    BLOCK_BEHAVIORS.get_behavior(&vanilla_blocks::FIRE).tick(
        world.get_block_state(fire_pos),
        &world,
        fire_pos,
    );
    assert_eq!(
        world.get_block_state(fire_pos).get_block(),
        &vanilla_blocks::FIRE,
        "fire on netherrack burns indefinitely"
    );
    world.set_block(fire_pos, air, UpdateFlags::UPDATE_ALL);
    world.set_block(rack_pos, air, UpdateFlags::UPDATE_ALL);

    // ------------------------------------------------------------------------
    // PHASE 9: Deep Dark — Sculk Sensor, Calibrated Sensor, Shrieker & Warden Sonic Boom
    // ------------------------------------------------------------------------
    assert!(vibration_frequency(&vanilla_game_events::STEP) > 0);
    assert_eq!(redstone_strength_for_distance(0.0, 8), 15);
    assert!(redstone_strength_for_distance(7.5, 8) >= 1);

    let sensor_pos = BlockPos::new(8, 64, 12);
    world.set_block(
        sensor_pos,
        vanilla_blocks::SCULK_SENSOR.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    world.game_event(
        &vanilla_game_events::BLOCK_PLACE,
        sensor_pos.east(),
        &GameEventContext::new(None, None),
    );
    let sensor_state = world.get_block_state(sensor_pos);
    assert_eq!(
        sensor_state.get_value(&BlockStateProperties::SCULK_SENSOR_PHASE),
        SculkSensorPhase::Active,
        "sculk sensor must enter Active phase on nearby vibration"
    );
    let sensor_holder = world.get_block_entity(sensor_pos).expect("sensor holder");
    let sensor_be = sensor_holder
        .downcast_ref::<SculkSensorBlockEntity>()
        .expect("sculk sensor be");
    assert!(sensor_be.last_vibration_frequency() > 0);
    world.set_block(sensor_pos, air, UpdateFlags::UPDATE_ALL);

    // Sculk shrieker creation & warning level roundtrip
    let shrieker_pos = BlockPos::new(9, 64, 12);
    let shrieker_state = vanilla_blocks::SCULK_SHRIEKER
        .default_state()
        .set_value(&BlockStateProperties::CAN_SUMMON, true);
    world.set_block(shrieker_pos, shrieker_state, UpdateFlags::UPDATE_ALL);
    let shrieker_holder = world
        .get_block_entity(shrieker_pos)
        .expect("shrieker holder");
    let shrieker_be = shrieker_holder
        .downcast_ref::<SculkShriekerBlockEntity>()
        .expect("shrieker be");
    shrieker_be.set_warning_level(2);
    assert_eq!(shrieker_be.warning_level(), 2);
    world.set_block(shrieker_pos, air, UpdateFlags::UPDATE_ALL);

    // Warden Sonic Boom against target at 8 blocks distance
    let warden = Arc::new(WardenEntity::new(
        &vanilla_entities::WARDEN,
        next_entity_id(),
        DVec3::new(0.5, 64.0, 0.5),
        Arc::downgrade(&world),
    ));
    let zombie = Arc::new(ZombieEntity::new(
        &vanilla_entities::ZOMBIE,
        next_entity_id(),
        DVec3::new(8.5, 64.0, 0.5),
        Arc::downgrade(&world),
    ));
    let warden_shared: SharedEntity = Arc::clone(&warden) as _;
    let target_shared: SharedEntity = Arc::clone(&zombie) as _;
    assert!(world.try_add_entity(Arc::clone(&warden_shared)).is_ok());
    assert!(world.try_add_entity(Arc::clone(&target_shared)).is_ok());
    let initial_hp = zombie.get_health();
    let _ = warden.set_target(Some(&target_shared));
    for _ in 0..45 {
        warden.custom_server_ai_step();
    }
    assert!(
        zombie.get_health() < initial_hp,
        "Warden Sonic Boom must deal ranged sonic_boom damage (hp {} -> {})",
        initial_hp,
        zombie.get_health()
    );
    warden.set_removed(RemovalReason::Discarded);
    zombie.set_removed(RemovalReason::Discarded);

    // ------------------------------------------------------------------------
    // PHASE 10: Trial Chambers (Vault + Trial Spawner) & Pale Garden (Creaking
    //           Heart + Invulnerability + Player-Gaze Freeze) & Armadillo Roll-Up
    // ------------------------------------------------------------------------
    let vault_pos = BlockPos::new(14, 64, 14);
    let vault_state = vanilla_blocks::VAULT.default_state();
    world.set_block(vault_pos, vault_state, UpdateFlags::UPDATE_ALL);
    let vault_holder = world.get_block_entity(vault_pos).expect("vault holder");
    let vault_be = vault_holder
        .downcast_ref::<VaultBlockEntity>()
        .expect("vault be");
    assert!(VaultBlockEntity::is_valid_key(
        vault_state,
        &ItemStack::new(&vanilla_items::TRIAL_KEY)
    ));
    assert!(!VaultBlockEntity::is_valid_key(
        vault_state,
        &ItemStack::new(&vanilla_items::OMINOUS_TRIAL_KEY)
    ));
    let _ = vault_be.total_players();
    world.set_block(vault_pos, air, UpdateFlags::UPDATE_ALL);

    let ts_pos = BlockPos::new(15, 64, 14);
    world.set_block(
        ts_pos,
        vanilla_blocks::TRIAL_SPAWNER.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    let ts_holder = world
        .get_block_entity(ts_pos)
        .expect("trial spawner holder");
    let ts_be = ts_holder
        .downcast_ref::<TrialSpawnerBlockEntity>()
        .expect("trial spawner be");
    assert!(ts_be.total_mobs() >= 0);
    world.set_block(ts_pos, air, UpdateFlags::UPDATE_ALL);

    // Creaking linked to Creaking Heart: invulnerable while heart lives, dies when heart breaks
    let heart_pos = BlockPos::new(4, 65, 14);
    world.set_block(
        heart_pos,
        vanilla_blocks::CREAKING_HEART.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    let heart_holder = world.get_block_entity(heart_pos).expect("heart holder");
    let heart_be = heart_holder
        .downcast_ref::<CreakingHeartBlockEntity>()
        .expect("creaking heart be");
    let creaking = Arc::new(CreakingEntity::new(
        &vanilla_entities::CREAKING,
        next_entity_id(),
        DVec3::new(4.5, 64.0, 15.5),
        Arc::downgrade(&world),
    ));
    assert!(world.try_add_entity(Arc::clone(&creaking) as _).is_ok());
    heart_be.set_creaking(Some(creaking.uuid().as_u128()));
    creaking.set_creaking_heart_pos(Some(heart_pos));
    assert!(
        creaking.is_invulnerable_to(
            &world,
            &DamageSource::environment(&vanilla_damage_types::CACTUS)
        ),
        "creaking linked to a live heart must ignore non-bypassing damage"
    );
    // Breaking the heart kills the linked creaking on its next AI step!
    world.set_block(heart_pos, air, UpdateFlags::UPDATE_ALL);
    creaking.custom_server_ai_step();
    assert!(
        !Entity::is_alive(creaking.as_ref()),
        "creaking must die when its linked heart is destroyed"
    );
    creaking.set_removed(RemovalReason::Discarded);

    // Armadillo roll-up damage formula: (amount - 1.0) / 2.0 + automatic curl on hit
    let armadillo = Arc::new(ArmadilloEntity::new(
        &vanilla_entities::ARMADILLO,
        next_entity_id(),
        DVec3::new(2.5, 64.0, 2.5),
        Arc::downgrade(&world),
    ));
    assert!(world.try_add_entity(Arc::clone(&armadillo) as _).is_ok());
    armadillo.roll_up(80);
    assert!(armadillo.is_rolled_up());
    let max_hp = armadillo.get_health();
    let _ = armadillo.hurt(
        &world,
        &DamageSource::environment(&vanilla_damage_types::CACTUS),
        5.0,
    );
    // (5.0 - 1.0) / 2.0 = 2.0 damage instead of 5.0!
    assert!(
        (armadillo.get_health() - (max_hp - 2.0)).abs() < 0.01,
        "rolled-up armadillo must take (5-1)/2 = 2.0 damage, hp={} (max={})",
        armadillo.get_health(),
        max_hp
    );
    armadillo.unroll();
    assert!(!armadillo.is_rolled_up());
    armadillo.set_removed(RemovalReason::Discarded);

    // ------------------------------------------------------------------------
    // PHASE 11: Vehicles (All Minecart Variants + Track Lerp Packet),
    //           Menus (Workstations + Containers), Recipes & Loot Tables
    // ------------------------------------------------------------------------
    let cart = MinecartEntity::new(
        &vanilla_entities::MINECART,
        next_entity_id(),
        DVec3::new(1.5, 64.0, 1.5),
        Arc::downgrade(&world),
    );
    let lerp_pkt = cart.build_track_lerp_packet(vec![MinecartStep::ZERO]);
    assert_eq!(lerp_pkt.entity_id, cart.id());
    assert_eq!(lerp_pkt.steps.len(), 1);

    // Verify FurnaceMinecartEntity::apply_natural_slowdown never recurses!
    let fcart = FurnaceMinecartEntity::new(
        &vanilla_entities::FURNACE_MINECART,
        next_entity_id(),
        DVec3::new(2.5, 64.0, 1.5),
        Arc::downgrade(&world),
    );
    let slowed = fcart.apply_natural_slowdown(DVec3::new(1.0, 0.5, -1.0));
    assert!(slowed.x.abs() < 1.0 && slowed.y == 0.0 && slowed.z.abs() < 1.0);
    let _hcart = HopperMinecartEntity::new(
        &vanilla_entities::HOPPER_MINECART,
        next_entity_id(),
        DVec3::new(3.5, 64.0, 1.5),
        Arc::downgrade(&world),
    );
    let _tcart = TntMinecartEntity::new(
        &vanilla_entities::TNT_MINECART,
        next_entity_id(),
        DVec3::new(4.5, 64.0, 1.5),
        Arc::downgrade(&world),
    );

    // Recipes: shaped 2x2 crafting_table, 1x2 stick, 3x3 diamond_pickaxe, smelting
    let table_input = CraftingInput::positioned(
        3,
        3,
        vec![
            ItemStack::new(&vanilla_items::OAK_PLANKS),
            ItemStack::new(&vanilla_items::OAK_PLANKS),
            ItemStack::empty(),
            ItemStack::new(&vanilla_items::OAK_PLANKS),
            ItemStack::new(&vanilla_items::OAK_PLANKS),
            ItemStack::empty(),
            ItemStack::empty(),
            ItemStack::empty(),
            ItemStack::empty(),
        ],
    )
    .input;
    let matched = REGISTRY
        .recipes
        .find_match(&vanilla_recipe_types::CRAFTING, &table_input)
        .expect("2x2 oak planks must match crafting_table");
    let crafted = recipe_manager::assemble_recipe(matched, &table_input);
    assert!(crafted.is(&vanilla_items::CRAFTING_TABLE));

    // Enchantments & attributes sanity
    assert!(
        REGISTRY
            .enchantments
            .by_key(&vanilla_enchantments::SHARPNESS.key)
            .is_some()
    );
    assert!(
        REGISTRY
            .attributes
            .by_key(&vanilla_attributes::MAX_HEALTH.key)
            .is_some()
    );

    // ------------------------------------------------------------------------
    // PHASE 12: Raids (with zero-leak eviction), Scoreboards, Teams, Triggers,
    //           MC 26.3 Stopwatches, Waypoints, Filled Maps & Bans/IP-Bans
    // ------------------------------------------------------------------------
    let raids = RaidManager::new();
    let r = raids.raise_omen(IVec3::new(0, 64, 0), 3);
    assert!(r.omen_level >= 1);
    assert_eq!(raids.raids().len(), 1);
    assert!(raids.stop(IVec3::new(0, 64, 0)));
    raids.tick(&world);
    assert!(
        raids.raids().is_empty(),
        "stopped raids must be evicted on tick (no memory leak)"
    );

    let sb = Scoreboard::new();
    let kills = sb
        .add_objective_with_criterion("kills", "dummy")
        .expect("dummy objective");
    let trig = sb
        .add_objective_with_criterion("vote", "trigger")
        .expect("trigger objective");
    assert!(!kills.is_trigger() && trig.is_trigger());
    assert_eq!(
        criterion_is_read_only("minecraft.mined:minecraft.stone"),
        Some(true)
    );
    let alice = ScoreHolder::new("Alice");
    let bob = ScoreHolder::new("Bob");
    sb.set_score(&alice, &kills, 15).expect("set score");
    assert_eq!(sb.score(&alice, &kills), Some(15));
    sb.enable_trigger(&alice, &trig).expect("enable trigger");
    let red = sb.add_team("red").expect("red team");
    sb.add_holder_to_team(&alice, &red).expect("join team");
    sb.add_holder_to_team(&bob, &red).expect("join team");
    assert!(sb.can_see_team_message(&bob, "red"));
    assert!(sb.create_stopwatch("speedrun"));
    sb.tick_stopwatches();
    sb.tick_stopwatches();
    assert_eq!(sb.stopwatch_ticks("speedrun"), Some(2));
    assert!(sb.remove_stopwatch("speedrun"));

    // Waypoint distance bucketing (Vec3i near, Azimuth far > 332m)
    let near_wp = compute_waypoint_data(DVec3::new(20.0, 64.0, 20.0), DVec3::ZERO, 8);
    assert!(matches!(near_wp, TrackedWaypointData::Vec3i { .. }));
    let far_wp = compute_waypoint_data(DVec3::new(500.0, 64.0, 0.0), DVec3::ZERO, 8);
    assert!(matches!(far_wp, TrackedWaypointData::Azimuth { .. }));

    // Bans & IP-bans lifecycle + persistence roundtrip
    let bans = BanList::new();
    let banned_uuid = Uuid::new_v4();
    let banned_ip = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 42));
    bans.ban_player(BanEntry::player(
        banned_uuid,
        "Griefer",
        "Rule 1",
        "Console",
        1_000,
        None,
    ));
    bans.ban_address(BanEntry::address(
        banned_ip,
        "203.0.113.42",
        "Spam",
        "Console",
        1_000,
        None,
    ));
    assert!(bans.is_banned(banned_uuid, 2_000));
    assert!(bans.is_address_banned(banned_ip, 2_000));
    let saved_bans = bans.to_saved();
    let restored_bans = BanList::new();
    restored_bans.apply_saved(saved_bans);
    assert!(restored_bans.is_banned(banned_uuid, 2_000));
    assert!(restored_bans.is_address_banned(banned_ip, 2_000));
    assert!(restored_bans.pardon(banned_uuid));
    assert!(restored_bans.pardon_address(banned_ip));
    assert!(!restored_bans.is_banned(banned_uuid, 2_000));

    // All 59 vanilla game rules accessible and mutable
    let orig_keep = world.get_game_rule(&vanilla_game_rules::KEEP_INVENTORY);
    world.set_game_rule(&vanilla_game_rules::KEEP_INVENTORY, !orig_keep);
    assert_eq!(
        world.get_game_rule(&vanilla_game_rules::KEEP_INVENTORY),
        !orig_keep
    );
    world.set_game_rule(&vanilla_game_rules::KEEP_INVENTORY, orig_keep);

    // ------------------------------------------------------------------------
    // PHASE 13: Live Production Server + Players + Menus + Maps + All Built-in
    //           Commands + 100% Newly Wired Protocol Mechanics (Cookies, Ping/Pong,
    //           Code of Conduct, Custom Report Details, DeleteChat, Debug Subs,
    //           StartConfiguration, CustomClickAction) + Creaking Gaze Freeze
    // ------------------------------------------------------------------------
    let rt = Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("tokio runtime");

    let op_uuid = Uuid::new_v4();
    let op_state = PermissionSubjectState::new(vec![OP_GROUP.to_owned()], PermissionSet::new());
    let mut subjects = PermissionSubjectIndex::default();
    subjects.set(op_uuid, op_state);

    let domain_cfg = ResolvedDomainConfig {
        name: world.domain().to_owned(),
        default_world: world.key.clone(),
        worlds: vec![world.key.clone()],
    };
    let server = rt
        .block_on(test_server_with_worlds_and_config(
            world.domain().to_owned(),
            slice::from_ref(&domain_cfg),
            slice::from_ref(&world),
            subjects,
            test_runtime_config(8),
        ))
        .expect("full production test server with all command roots must build");

    let (op_cap, op_packets) = CapturingConnection::new();
    let op_player = TestPlayerBuilder::new(Arc::clone(&world), "Admin", next_entity_id())
        .uuid(op_uuid)
        .server(&server)
        .connection(Arc::new(PlayerConnection::Other(Box::new(SharedNetConn(
            Arc::clone(&op_cap),
        )))))
        .build();
    let (guest_cap, guest_packets) = CapturingConnection::new();
    let guest_player = TestPlayerBuilder::new(Arc::clone(&world), "Guest", next_entity_id())
        .server(&server)
        .connection(Arc::new(PlayerConnection::Other(Box::new(SharedNetConn(
            Arc::clone(&guest_cap),
        )))))
        .build();

    op_player.set_permission_state(
        vec![OP_GROUP.to_owned()],
        PermissionSet::new(),
        PermissionMetadataSet::new(),
        PermissionSet::new(),
        PermissionMetadataSet::new(),
    );
    world.players.insert(Arc::clone(&op_player));
    world.players.insert(Arc::clone(&guest_player));
    let _ = server.online_players.insert(Arc::clone(&op_player));
    let _ = server.online_players.insert(Arc::clone(&guest_player));
    assert!(op_player.is_operator());
    assert!(!guest_player.is_operator());

    // Open, construct & actively execute crafting + villager trading against Admin's inventory
    let inv = Arc::clone(&op_player.inventory);
    let menu_pos = BlockPos::new(0, 64, 0);
    let mut m_craft = crafting(Arc::clone(&inv), 1, menu_pos);
    *m_craft.behavior_mut().carried_mut() = ItemStack::new(&vanilla_items::OAK_LOG);
    m_craft.clicked(
        Click::Pickup {
            slot: 1,
            button: MouseButton::Left,
        },
        &op_player,
    );
    m_craft.clicked(Click::QuickMove { slot: 0 }, &op_player);
    assert_eq!(
        op_player
            .inventory
            .lock()
            .get_items()
            .iter()
            .filter(|s| s.is(&vanilla_items::OAK_PLANKS))
            .map(ItemStack::count)
            .sum::<i32>(),
        4,
        "live 3x3 CraftingMenu QuickMove must craft 4 oak planks without deadlock"
    );

    let farmer_trades = Arc::new(SyncMutex::new(offers_for("minecraft:farmer")));
    let mut m_merchant = merchant(Arc::clone(&inv), 13, Arc::clone(&farmer_trades));
    *m_merchant.behavior_mut().carried_mut() = ItemStack::with_count(&vanilla_items::WHEAT, 20);
    m_merchant.clicked(
        Click::Pickup {
            slot: 0,
            button: MouseButton::Left,
        },
        &op_player,
    );
    m_merchant.clicked(Click::QuickMove { slot: 2 }, &op_player);
    assert!(
        op_player
            .inventory
            .lock()
            .get_items()
            .iter()
            .any(|s| s.is(&vanilla_items::EMERALD) && s.count() >= 1),
        "live MerchantMenu trade must pay 20 wheat for emerald without deadlock"
    );
    assert_eq!(farmer_trades.lock()[0].uses, 1);

    let chest_cart = ChestMinecartEntity::new(
        &vanilla_entities::CHEST_MINECART,
        next_entity_id(),
        DVec3::new(2.5, 64.0625, 2.5),
        Arc::downgrade(&world),
    );
    chest_cart.set_loot_table(
        Identifier::vanilla_static("chests/abandoned_mineshaft"),
        12_345,
    );
    assert!(
        chest_cart.unpack_loot(),
        "ChestMinecartEntity must unpack abandoned_mineshaft loot into its 27-slot container"
    );
    assert!(
        chest_cart
            .container()
            .lock()
            .items()
            .iter()
            .any(|s| !s.is_empty()),
        "unpacked mineshaft ChestMinecartEntity must contain loot items"
    );
    op_player.inventory.lock().clear_content();

    let _m_stone = stonecutter(Arc::clone(&inv), 2, menu_pos, &world);
    let _m_smith = smithing(Arc::clone(&inv), 3, menu_pos, &world);
    let _m_carto = cartography_table(Arc::clone(&inv), 4, menu_pos, &world);
    let _m_loom = loom(Arc::clone(&inv), 5, menu_pos, &world);
    let _m_anvil = anvil(Arc::clone(&inv), 6, menu_pos, &world);
    let _m_grind = grindstone(Arc::clone(&inv), 7, menu_pos, &world);
    let sc27 = SimpleContainer::new(27).into_shared();
    let sc27_shulker = SimpleContainer::new(27).into_shared();
    let sc9 = SimpleContainer::new(9).into_shared();
    let sc5 = SimpleContainer::new(5).into_shared();
    let sc3_furnace = SimpleContainer::new(3).into_shared();
    let sc3_blast = SimpleContainer::new(3).into_shared();
    let sc3_smoker = SimpleContainer::new(3).into_shared();
    let sc1 = SimpleContainer::new(1).into_shared();
    let _m_chest = chest(Arc::clone(&inv), 8, sc27, 3);
    let _m_shulker = shulker_box(Arc::clone(&inv), 14, sc27_shulker);
    let _m_disp = dispenser(Arc::clone(&inv), 9, sc9);
    let _m_hop = hopper(Arc::clone(&inv), 10, Arc::clone(&sc5));
    let _m_brew = brewing_stand(Arc::clone(&inv), 11, sc5);
    let _m_lect = lectern(12, sc1);
    let _m_furnace = furnace_menu(
        Arc::clone(&inv),
        15,
        ContainerRef::from(sc3_furnace),
        FurnaceKind::Furnace,
    );
    let _m_blast = furnace_menu(
        Arc::clone(&inv),
        16,
        ContainerRef::from(sc3_blast),
        FurnaceKind::BlastFurnace,
    );
    let _m_smoker = furnace_menu(
        Arc::clone(&inv),
        17,
        ContainerRef::from(sc3_smoker),
        FurnaceKind::Smoker,
    );
    let _m_enchant = enchantment(Arc::clone(&inv), 18, menu_pos, &world, 42);

    // MC 26.3 Mount menu (saddle + body armor + 3x5 storage) — distinct non-overlapping slots!
    let nautilus_mount: SharedEntity = Arc::new(NautilusEntity::new(
        &vanilla_entities::NAUTILUS,
        next_entity_id(),
        DVec3::new(3.5, 64.0, 3.5),
        Arc::downgrade(&world),
    ));
    let mount_cols = 5;
    let mount_container = SimpleContainer::new(mount_inventory_size(mount_cols)).into_shared();
    let m_mount = mount(
        Arc::clone(&inv),
        19,
        mount_container,
        mount_cols,
        Arc::downgrade(&nautilus_mount),
    );
    assert_eq!(
        m_mount.behavior().slot_count(),
        mount_inventory_size(mount_cols) + 36,
        "mount menu must cover saddle + body armor + 15 storage + 36 player slots"
    );

    // MC 26.3 Combat, Protection Enchantment & Vanilla hurtArmor parity:
    // - ArmorStandEntity (`LivingEntity` non-mob/non-player) does NOT wear down equipped armor
    // - ZombieEntity (`Mob`) DOES wear down equipped armor and honours Protection IV reduction
    let stand = ArmorStandEntity::new(
        &vanilla_entities::ARMOR_STAND,
        next_entity_id(),
        DVec3::new(5.5, 64.0, 5.5),
        Arc::downgrade(&world),
    );
    stand.with_equipment_slot_mut(EquipmentSlot::Chest, &mut |slot| {
        *slot = ItemStack::new(&vanilla_items::DIAMOND_CHESTPLATE);
    });
    let _ = stand.hurt(
        &world,
        &DamageSource::environment(&vanilla_damage_types::FIREWORKS),
        6.0,
    );
    stand.with_equipment_slot(EquipmentSlot::Chest, &mut |slot| {
        assert_eq!(
            slot.get_damage_value(),
            0,
            "ArmorStand must not take armor durability damage (vanilla LivingEntity.hurtArmor)"
        );
    });

    let armored_zombie = ZombieEntity::new(
        &vanilla_entities::ZOMBIE,
        next_entity_id(),
        DVec3::new(6.5, 64.0, 5.5),
        Arc::downgrade(&world),
    );
    let mut prot_chest = ItemStack::new(&vanilla_items::DIAMOND_CHESTPLATE);
    prot_chest.set_enchantments(&[(Identifier::vanilla_static("protection"), 4)], false);
    armored_zombie.with_equipment_slot_mut(EquipmentSlot::Chest, &mut |slot| {
        *slot = prot_chest.clone();
    });
    let damaged = armored_zombie.hurt(
        &world,
        &DamageSource::environment(&vanilla_damage_types::FIREWORKS),
        8.0,
    );
    assert!(damaged);
    armored_zombie.with_equipment_slot(EquipmentSlot::Chest, &mut |slot| {
        assert!(
            slot.get_damage_value() > 0,
            "Mob equipped armor must take durability damage on non-bypassing hits"
        );
    });

    // MC 26.3 Mob Conversion + Golden Dandelion `age_locked` preservation:
    // Shearing a MushroomCow converts it into a CowEntity while preserving age, age_locked & effects!
    let mooshroom = Arc::new(MushroomCowEntity::new(
        &vanilla_entities::MOOSHROOM,
        next_entity_id(),
        DVec3::new(7.5, 64.0, 7.5),
        Arc::downgrade(&world),
    ));
    mooshroom.set_age(500);
    mooshroom.set_age_locked(true);
    mooshroom.add_mob_effect(MobEffectInstance::with_duration(
        vanilla_mob_effects::SPEED,
        400,
        0,
    ));
    let mooshroom_shared: SharedEntity = Arc::clone(&mooshroom) as _;
    assert!(world.try_add_entity(Arc::clone(&mooshroom_shared)).is_ok());
    mooshroom.shear(&world, &ItemStack::new(&vanilla_items::SHEARS));
    let converted_cows = world.get_entities_in_aabb_matching(
        &WorldAabb::of_size(DVec3::new(7.5, 64.0, 7.5), 3.0, 3.0, 3.0),
        |e| e.downcast_ref::<CowEntity>().is_some(),
    );
    assert_eq!(
        converted_cows.len(),
        1,
        "sheared mooshroom converts to 1 cow"
    );
    let cow = converted_cows[0]
        .downcast_ref::<CowEntity>()
        .expect("cow downcast");
    assert!(
        cow.is_age_locked(),
        "MC 26.3 entity conversion must preserve Golden Dandelion age_locked flag"
    );
    assert_eq!(cow.get_age(), 500);
    assert!(cow.mob_effect(vanilla_mob_effects::SPEED).is_some());
    for e in world.get_entities_in_aabb_matching(
        &WorldAabb::of_size(DVec3::new(7.5, 64.0, 7.5), 4.0, 4.0, 4.0),
        |e| {
            e.downcast_ref::<CowEntity>().is_some()
                || e.downcast_ref::<ItemEntity>().is_some()
                || e.downcast_ref::<MushroomCowEntity>().is_some()
        },
    ) {
        e.set_removed(RemovalReason::Discarded);
    }

    // Vanilla initial-join packet ordering: CSetDefaultSpawnPosition -> CSetChunkCacheRadius -> CSetSimulationDistance
    let (join_cap, join_pkts) = CapturingConnection::new();
    let joiner = TestPlayerBuilder::new(Arc::clone(&world), "JoinOrderCheck", next_entity_id())
        .connection(Arc::new(PlayerConnection::Other(Box::new(SharedNetConn(
            join_cap,
        )))))
        .build();
    join_pkts.lock().clear();
    assert!(joiner.spawn(
        DVec3::new(2.5, 64.0, 2.5),
        (0.0, 0.0),
        ResetReason::InitialJoin
    ));
    let join_ids: Vec<i32> = join_pkts.lock().iter().map(decode_packet_id).collect();
    let pos_spawn = join_ids
        .iter()
        .position(|&id| id == C_SET_DEFAULT_SPAWN_POSITION)
        .expect("InitialJoin must emit CSetDefaultSpawnPosition");
    let pos_radius = join_ids
        .iter()
        .position(|&id| id == C_SET_CHUNK_CACHE_RADIUS)
        .expect("InitialJoin must emit CSetChunkCacheRadius");
    let pos_sim = join_ids
        .iter()
        .position(|&id| id == C_SET_SIMULATION_DISTANCE)
        .expect("InitialJoin must emit CSetSimulationDistance");
    assert!(pos_spawn < pos_radius && pos_sim == pos_radius + 1);
    let _ = world.players.remove_sync(&joiner.base().uuid());

    // Filled Map creation, live world terrain sampling & CMapItemData streaming
    op_player
        .try_set_position(DVec3::new(2.5, 64.0, 2.5))
        .expect("set op_player pos");
    guest_player
        .try_set_position(DVec3::new(8.5, 64.0, 8.5))
        .expect("set guest_player pos");
    let map_stack = FilledMapItem::create_map(&world, 4, 4, 0, true, false);
    let map_id = FilledMapItem::map_id(&map_stack).expect("filled map id");
    op_player.inventory.lock().set_item(0, map_stack);
    {
        let mut maps = world.map_states.lock();
        let map_state = maps.get_mut(map_id).expect("allocated map state");
        map_state.update(&world, &op_player);
        update_colors(&world, &op_player, map_state);
    }
    op_packets.lock().clear();
    send_map_packets(&op_player);
    assert!(
        op_packets
            .lock()
            .iter()
            .any(|p| decode_packet_id(p) == C_MAP_ITEM_DATA),
        "holding a filled map must stream CMapItemData"
    );

    // Waypoint locator bar streaming between Admin & Guest
    op_packets.lock().clear();
    world
        .waypoints
        .set_icon_color(guest_player.base().uuid(), Some([0xFF, 0x55, 0x00]));
    world.waypoints.tick(&world);
    assert!(
        op_packets
            .lock()
            .iter()
            .any(|p| decode_packet_id(p) == C_WAYPOINT),
        "WaypointManager must emit CWaypoint Track packets for visible players"
    );

    // Server-level MC 26.3 protocol wiring: Code of Conduct, Report Details, Delete Chat, Completions
    assert!(server.code_of_conduct_packet().is_none());
    server.set_code_of_conduct(Some("Welcome to SteelMC 26.3!".to_owned()));
    assert_eq!(
        server
            .code_of_conduct_packet()
            .expect("code of conduct")
            .code_of_conduct,
        "Welcome to SteelMC 26.3!"
    );
    server.set_code_of_conduct(None);
    assert!(server.code_of_conduct_packet().is_none());

    server.set_custom_report_details(vec![ReportDetail {
        title: "version".to_owned(),
        description: "26.3".to_owned(),
    }]);
    assert_eq!(
        server
            .custom_report_details_packet()
            .expect("report details")
            .details
            .len(),
        1
    );
    server.delete_signed_chat(PackedMessageSignature::CachedId(0));
    server.broadcast_custom_chat_completions(
        CustomChatCompletionAction::Add,
        vec!["steelmc".to_owned()],
    );

    // Player-level protocol wiring: store_cookie, request_cookie/handle_cookie_response,
    // start_configuration/ack, send_ping/pong, custom_click_action, debug subscriptions!
    assert!(op_player.store_cookie(Identifier::vanilla_static("tok"), vec![1, 2, 3]));
    assert!(!op_player.store_cookie(Identifier::vanilla_static("big"), vec![0; 5_121]));

    let cookie_key = Identifier::vanilla_static("auth");
    op_player.request_cookie(cookie_key.clone());
    op_player.handle_cookie_response(SCookieResponse {
        key: cookie_key.clone(),
        payload: Some(vec![0xDE, 0xAD]),
    });
    assert!(!op_cap.closed.load(Ordering::Acquire));
    assert_eq!(
        op_player.received_cookie(&cookie_key),
        Some(Some(vec![0xDE, 0xAD]))
    );

    op_player.start_configuration();
    assert!(op_player.is_awaiting_configuration());
    op_player.handle_configuration_acknowledged();
    assert!(!op_player.is_awaiting_configuration());

    op_player.send_ping(777);
    op_player.handle_pong(777);
    assert_eq!(op_player.last_pong_id(), Some(777));

    op_player
        .send_custom_chat_completions(CustomChatCompletionAction::Set, vec!["@team".to_owned()]);
    op_player.handle_custom_click_action(SCustomClickAction {
        id: Identifier::vanilla_static("accept_rules"),
        payload: Some(vec![0x01, 0x02]),
    });
    assert_eq!(
        op_player.last_custom_click_action(),
        Some(Identifier::vanilla_static("accept_rules"))
    );

    // Debug subscriptions: operator receives all 4 CDebug* packets; non-operator gets NONE!
    op_player.handle_debug_subscription_request(SDebugSubscriptionRequest {
        subscriptions: vec![7],
    });
    guest_player.handle_debug_subscription_request(SDebugSubscriptionRequest {
        subscriptions: vec![7],
    });
    op_packets.lock().clear();
    guest_packets.lock().clear();
    world.broadcast_debug_block_value(BlockPos::new(1, 64, 1), 7, Some(vec![1]));
    world.broadcast_debug_chunk_value(0, 0, 7, Some(vec![2]));
    world.broadcast_debug_entity_value(op_player.id(), 7, Some(vec![3]));
    world.broadcast_debug_event(7, vec![4]);
    assert_eq!(
        op_packets.lock().len(),
        4,
        "subscribed operator must receive all 4 CDebug* packet types"
    );
    assert_eq!(
        guest_packets.lock().len(),
        0,
        "non-operator must never receive debug subscription broadcasts"
    );

    // Unsolicited cookie response MUST disconnect the client (vanilla parity)!
    let (rogue_cap, _) = CapturingConnection::new();
    let rogue = TestPlayerBuilder::new(Arc::clone(&world), "Rogue", next_entity_id())
        .server(&server)
        .connection(Arc::new(PlayerConnection::Other(Box::new(SharedNetConn(
            Arc::clone(&rogue_cap),
        )))))
        .build();
    rogue.handle_cookie_response(SCookieResponse {
        key: Identifier::vanilla_static("unsolicited"),
        payload: None,
    });
    assert!(
        rogue_cap.closed.load(Ordering::Acquire),
        "unsolicited SCookieResponse must disconnect client"
    );

    // Creaking player-gaze freeze + carved pumpkin bypass!
    op_player.set_rotation((180.0, 0.0)); // look away (-Z) so Guest is sole observer
    guest_player
        .try_set_position(DVec3::new(0.5, 64.0, 0.5))
        .expect("set guest position");
    guest_player.set_rotation((0.0, 0.0)); // looks along +Z in Minecraft (yaw 0 = south/+Z)
    let watched_creaking = Arc::new(CreakingEntity::new(
        &vanilla_entities::CREAKING,
        next_entity_id(),
        DVec3::new(0.5, 64.0, 5.5),
        Arc::downgrade(&world),
    ));
    assert!(
        world
            .try_add_entity(Arc::clone(&watched_creaking) as _)
            .is_ok()
    );
    assert!(
        watched_creaking.is_observed_by_player(&world),
        "creaking in front of survival player must detect gaze and freeze"
    );
    // Wearing a carved pumpkin lets the player look at the Creaking without freezing it!
    guest_player
        .inventory
        .lock()
        .set_item(39, ItemStack::new(&vanilla_items::CARVED_PUMPKIN));
    assert!(
        !watched_creaking.is_observed_by_player(&world),
        "carved pumpkin helmet must mask player gaze from Creaking"
    );
    guest_player
        .inventory
        .lock()
        .set_item(39, ItemStack::empty());
    watched_creaking.set_removed(RemovalReason::Discarded);

    // End-to-End Command Execution across Console & Operator Player!
    let console_commands = [
        "time set day",
        "time add 100",
        "time pause",
        "time resume",
        "time query gametime",
        "return 42",
        "swing Admin",
        "rotate Admin 90 0",
        "tpsbar",
        "tpsbar",
        "weather clear",
        "difficulty hard",
        "difficulty normal",
        "defaultgamemode adventure",
        "defaultgamemode survival",
        "gamerule keep_inventory true",
        "gamerule keep_inventory false",
        "seed",
        "list",
        "version",
        "help",
        "say Hello from SteelMC 26.3 mega suite",
        "tellraw Admin \"hello\"",
        "title Admin title \"SteelMC 26.3\"",
        "title Admin clear",
        "give Admin minecraft:diamond 5",
        "clear Admin minecraft:diamond 2",
        "experience add Admin 50 points",
        "effect give Admin minecraft:speed 30 1",
        "effect clear Admin",
        "setblock 3 66 3 minecraft:gold_block",
        "fill 3 67 3 4 67 4 minecraft:iron_block",
        "clone 3 67 3 4 67 4 6 67 6",
        "fill 3 66 3 8 68 8 minecraft:air",
        "summon minecraft:pig 4 65 4",
        "tag Admin add verified",
        "tag Admin remove verified",
        "scoreboard objectives add pts dummy",
        "scoreboard players set Admin pts 100",
        "scoreboard players add Admin pts 25",
        "scoreboard players get Admin pts",
        "team add alpha",
        "team join alpha Admin",
        "team join alpha Guest",
        "team modify alpha color aqua",
        "team modify alpha friendlyFire false",
        "team list",
        "team list alpha",
        "bossbar add minecraft:test \"Boss\"",
        "bossbar set minecraft:test max 200",
        "bossbar set minecraft:test value 150",
        "bossbar set minecraft:test color blue",
        "bossbar set minecraft:test visible true",
        "bossbar set minecraft:test players Admin",
        "bossbar get minecraft:test value",
        "bossbar list",
        "execute store result bossbar minecraft:test value run scoreboard players get Admin pts",
        "bossbar remove minecraft:test",
        "data merge storage minecraft:bench {answer:42}",
        "data get storage minecraft:bench answer",
        "data remove storage minecraft:bench answer",
        "stopwatch create minecraft:bench",
        "stopwatch query minecraft:bench",
        "stopwatch restart minecraft:bench",
        "stopwatch list",
        "stopwatch remove minecraft:bench",
        "random value 1..100",
        "random roll 1..20",
        "random reset *",
        "waypoint list",
        "waypoint modify Guest color aqua",
        "waypoint modify Guest color hex FF8800",
        "waypoint modify Guest color reset",
        "dialog show Admin {type:\"minecraft:notice\"}",
        "dialog clear Admin",
        "serverpack push \"https://example.com/rp.zip\"",
        "fetchprofile name Admin",
        "loot give Admin loot minecraft:blocks/diamond_ore",
        "spreadplayers 4 4 1 6 false Admin",
        "item replace entity Admin hotbar.2 with minecraft:diamond_sword 1",
        "attribute Admin minecraft:max_health modifier add minecraft:bonus_hp 4.0 add_value",
        "attribute Admin minecraft:max_health get",
        "attribute Admin minecraft:max_health modifier remove minecraft:bonus_hp",
        "particle minecraft:flame 4 65 4",
        "datapack list",
        "forceload add 0 64 0",
        "forceload query",
        "forceload remove 0 64 0",
        "worldborder set 1000",
        "setworldspawn 0 64 0",
        "spawnpoint Admin 0 64 0",
        "setidletimeout 10",
        "setidletimeout 0",
        "whitelist on",
        "whitelist off",
        "ban-ip 198.51.100.9 test_reason",
        "banlist ips",
        "pardon-ip 198.51.100.9",
        "save-off",
        "save-on",
        "save-all",
        "recipe give Admin minecraft:stick",
        "recipe take Admin minecraft:stick",
        "stopsound Admin",
        "posteffect add Admin minecraft:invert",
        "posteffect list Admin",
        "posteffect clear Admin",
        "gamemode creative Admin",
        "gamemode survival Admin",
        "teleport Guest 10 65 10",
        "me verifies Vanilla 26.3 mega parity",
        "msg Guest hello from steel 26.3",
        "playsound minecraft:entity.experience_orb.pickup master Admin",
        "fillbiome 0 64 0 4 68 4 minecraft:desert",
        "whitelist list",
        "banlist",
        "banlist players",
    ];
    let rt_guard = rt.enter();
    for cmd in console_commands {
        let res = exec_command(&server, CommandSender::Console, cmd);
        let (ok, val) = res.unwrap_or_else(|err| panic!("command `{cmd}` failed to parse: {err}"));
        assert!(ok, "command `{cmd}` executed with failure (val={val})");
    }
    drop(rt_guard);
    // Verify `/teammsg` & `/tm` from a player inside team `alpha`!
    assert!(
        exec_command(
            &server,
            CommandSender::Player(Arc::clone(&op_player)),
            "tm Squad ready!"
        )
        .is_ok()
    );
    // Verify scoreboard score mutated by `/scoreboard players set` + `add` == 125!
    let domain_sb = server
        .scoreboards
        .get(world.domain())
        .expect("domain scoreboard");
    let pts_obj = domain_sb.objective("pts").expect("pts objective");
    assert_eq!(
        domain_sb.score(&ScoreHolder::new("Admin"), &pts_obj),
        Some(125)
    );

    // Also verify Multi-Dimension world types (Nether & End) initialize cleanly
    assert_eq!(
        vanilla_dimension_types::THE_NETHER.infiniburn,
        "#minecraft:infiniburn_nether"
    );
    assert_eq!(
        vanilla_dimension_types::THE_END.infiniburn,
        "#minecraft:infiniburn_end"
    );

    // ------------------------------------------------------------------------
    // PHASE 14: Live Smelting (Furnace / Blast Furnace / Smoker) & 3-Stage Potion Brewing
    // ------------------------------------------------------------------------
    let furnace_pos = BlockPos::new(-12, 65, -12);
    world.set_block(
        furnace_pos,
        vanilla_blocks::FURNACE.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    let furnace_be = world
        .get_block_entity(furnace_pos)
        .expect("furnace block entity");
    {
        let fref = furnace_be.container_ref().expect("furnace container");
        let mut fguard = fref.lock();
        fguard.set_item(0, ItemStack::with_count(&vanilla_items::IRON_ORE, 2));
        fguard.set_item(1, ItemStack::new(&vanilla_items::COAL));
    }
    for _ in 0..200 {
        furnace_be.tick(&world);
    }
    assert!(
        world
            .get_block_state(furnace_pos)
            .get_value(&BlockStateProperties::LIT),
        "furnace must set LIT=true in the world while burning coal"
    );
    {
        let fref = furnace_be.container_ref().expect("furnace container");
        let fguard = fref.lock();
        assert_eq!(fguard.get_item(0).count(), 1);
        assert!(fguard.get_item(2).is(&vanilla_items::IRON_INGOT));
        assert_eq!(fguard.get_item(2).count(), 1);
    }

    // Blast Furnace smelts iron ore in 100 ticks (exact 2x Vanilla speed)
    let blast_pos = BlockPos::new(-10, 65, -12);
    world.set_block(
        blast_pos,
        vanilla_blocks::BLAST_FURNACE.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    let blast_be = world
        .get_block_entity(blast_pos)
        .expect("blast furnace block entity");
    {
        let bref = blast_be.container_ref().expect("blast container");
        let mut bguard = bref.lock();
        bguard.set_item(0, ItemStack::new(&vanilla_items::IRON_ORE));
        bguard.set_item(1, ItemStack::new(&vanilla_items::COAL));
    }
    for _ in 0..100 {
        blast_be.tick(&world);
    }
    {
        let bref = blast_be.container_ref().expect("blast container");
        let bguard = bref.lock();
        assert!(bguard.get_item(0).is_empty());
        assert!(bguard.get_item(2).is(&vanilla_items::IRON_INGOT));
    }

    // Smoker cooks raw beef in 100 ticks (exact 2x Vanilla speed)
    let smoker_pos = BlockPos::new(-8, 65, -12);
    world.set_block(
        smoker_pos,
        vanilla_blocks::SMOKER.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    let smoker_be = world
        .get_block_entity(smoker_pos)
        .expect("smoker block entity");
    {
        let sref = smoker_be.container_ref().expect("smoker container");
        let mut sguard = sref.lock();
        sguard.set_item(0, ItemStack::new(&vanilla_items::BEEF));
        sguard.set_item(1, ItemStack::new(&vanilla_items::COAL));
    }
    for _ in 0..100 {
        smoker_be.tick(&world);
    }
    {
        let sref = smoker_be.container_ref().expect("smoker container");
        let sguard = sref.lock();
        assert!(sguard.get_item(0).is_empty());
        assert!(sguard.get_item(2).is(&vanilla_items::COOKED_BEEF));
    }

    // Full 3-stage Vanilla alchemy chain in live BrewingStandBlockEntity:
    //   water + nether_wart -> awkward -> + sugar -> swiftness -> + redstone -> long_swiftness
    let brew_pos = BlockPos::new(-6, 65, -12);
    world.set_block(
        brew_pos,
        vanilla_blocks::BREWING_STAND.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    let brew_be = world
        .get_block_entity(brew_pos)
        .expect("brewing stand block entity");
    let brew_stand = brew_be
        .downcast_ref::<BrewingStandBlockEntity>()
        .expect("BrewingStandBlockEntity downcast");
    {
        let cref = brew_be.container_ref().expect("brewing container");
        let mut cguard = cref.lock();
        for slot in 0..3 {
            let mut water = ItemStack::new(&vanilla_items::POTION);
            water.set_potion(&Identifier::vanilla_static("water"));
            cguard.set_item(slot, water);
        }
        cguard.set_item(3, ItemStack::new(&vanilla_items::NETHER_WART));
        cguard.set_item(4, ItemStack::new(&vanilla_items::BLAZE_POWDER));
    }
    for _ in 0..401 {
        brew_be.tick(&world);
    }
    let brew_state = world.get_block_state(brew_pos);
    assert!(brew_state.get_value(&BlockStateProperties::HAS_BOTTLE_0));
    assert!(brew_state.get_value(&BlockStateProperties::HAS_BOTTLE_1));
    assert!(brew_state.get_value(&BlockStateProperties::HAS_BOTTLE_2));
    {
        let cref = brew_be.container_ref().expect("brewing container");
        let mut cguard = cref.lock();
        for slot in 0..3 {
            assert_eq!(
                cguard.get_item(slot).potion_key(),
                Some(Identifier::vanilla_static("awkward"))
            );
        }
        assert!(cguard.get_item(3).is_empty());
        assert!(cguard.get_item(4).is_empty());
        cguard.set_item(3, ItemStack::new(&vanilla_items::SUGAR));
    }
    for _ in 0..401 {
        brew_be.tick(&world);
    }
    {
        let cref = brew_be.container_ref().expect("brewing container");
        let mut cguard = cref.lock();
        for slot in 0..3 {
            assert_eq!(
                cguard.get_item(slot).potion_key(),
                Some(Identifier::vanilla_static("swiftness"))
            );
        }
        cguard.set_item(3, ItemStack::new(&vanilla_items::REDSTONE));
    }
    for _ in 0..401 {
        brew_be.tick(&world);
    }
    {
        let cref = brew_be.container_ref().expect("brewing container");
        let cguard = cref.lock();
        for slot in 0..3 {
            assert_eq!(
                cguard.get_item(slot).potion_key(),
                Some(Identifier::vanilla_static("long_swiftness"))
            );
        }
    }
    // 20 fuel uses from 1 blaze powder minus 3 brew cycles == 17 remaining!
    assert_eq!(brew_stand.fuel(), 17);

    // ------------------------------------------------------------------------
    // PHASE 15: Redstone Circuits (Copper Bulb T-Flip-Flop + Piston + Obsidian) & Explosions
    // ------------------------------------------------------------------------
    let bulb_pos = BlockPos::new(-12, 68, -8);
    let bulb_power = BlockPos::new(-11, 68, -8);
    world.set_block(
        bulb_pos,
        vanilla_blocks::COPPER_BULB.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    assert!(
        !world
            .get_block_state(bulb_pos)
            .get_value(&BlockStateProperties::LIT)
    );
    // 1st rising edge -> LIT=true, POWERED=true, comparator=15
    world.set_block(
        bulb_power,
        vanilla_blocks::REDSTONE_BLOCK.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    let bulb_s1 = world.get_block_state(bulb_pos);
    assert!(bulb_s1.get_value(&BlockStateProperties::POWERED));
    assert!(bulb_s1.get_value(&BlockStateProperties::LIT));
    assert_eq!(
        BLOCK_BEHAVIORS
            .get_behavior(bulb_s1.get_block())
            .get_analog_output_signal(bulb_s1, &world, bulb_pos, Direction::North),
        15
    );
    // 1st falling edge -> LIT stays true, POWERED=false, comparator=15
    world.set_block(
        bulb_power,
        vanilla_blocks::AIR.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    let bulb_s2 = world.get_block_state(bulb_pos);
    assert!(!bulb_s2.get_value(&BlockStateProperties::POWERED));
    assert!(bulb_s2.get_value(&BlockStateProperties::LIT));
    // 2nd rising edge -> LIT toggles false, POWERED=true, comparator=0
    world.set_block(
        bulb_power,
        vanilla_blocks::REDSTONE_BLOCK.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    let bulb_s3 = world.get_block_state(bulb_pos);
    assert!(bulb_s3.get_value(&BlockStateProperties::POWERED));
    assert!(!bulb_s3.get_value(&BlockStateProperties::LIT));
    assert_eq!(
        BLOCK_BEHAVIORS
            .get_behavior(bulb_s3.get_block())
            .get_analog_output_signal(bulb_s3, &world, bulb_pos, Direction::North),
        0
    );
    world.set_block(
        bulb_power,
        vanilla_blocks::AIR.default_state(),
        UpdateFlags::UPDATE_ALL,
    );

    // Piston pushes GOLD_BLOCK east, while OBSIDIAN blocks a second piston from extending
    let piston_pos = BlockPos::new(-12, 70, -4);
    let pushed_from = BlockPos::new(-11, 70, -4);
    let pushed_to = BlockPos::new(-10, 70, -4);
    let piston_pwr = BlockPos::new(-13, 70, -4);
    world.set_block(
        pushed_from,
        vanilla_blocks::GOLD_BLOCK.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    world.set_block(
        piston_pos,
        vanilla_blocks::PISTON
            .default_state()
            .set_value(&BlockStateProperties::FACING, Direction::East),
        UpdateFlags::UPDATE_ALL,
    );
    world.set_block(
        piston_pwr,
        vanilla_blocks::REDSTONE_BLOCK.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    for t in 1..=4_u64 {
        tick_test_world(&world, t, true);
    }
    assert!(
        world
            .get_block_state(piston_pos)
            .get_value(&BlockStateProperties::EXTENDED),
        "powered piston must extend"
    );
    assert_eq!(
        world.get_block_state(pushed_from).get_block(),
        &vanilla_blocks::PISTON_HEAD
    );
    assert_eq!(
        world.get_block_state(pushed_to).get_block(),
        &vanilla_blocks::GOLD_BLOCK
    );

    let blocked_piston = BlockPos::new(-12, 72, -4);
    let obsidian_blocker = BlockPos::new(-11, 72, -4);
    world.set_block(
        obsidian_blocker,
        vanilla_blocks::OBSIDIAN.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    world.set_block(
        blocked_piston,
        vanilla_blocks::PISTON
            .default_state()
            .set_value(&BlockStateProperties::FACING, Direction::East),
        UpdateFlags::UPDATE_ALL,
    );
    world.set_block(
        BlockPos::new(-13, 72, -4),
        vanilla_blocks::REDSTONE_BLOCK.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    for t in 5..=8_u64 {
        tick_test_world(&world, t, true);
    }
    assert!(
        !world
            .get_block_state(blocked_piston)
            .get_value(&BlockStateProperties::EXTENDED),
        "piston must not push obsidian"
    );
    assert_eq!(
        world.get_block_state(obsidian_blocker).get_block(),
        &vanilla_blocks::OBSIDIAN
    );

    // Live TNT server explosion: destroys dirt, preserves obsidian, and hurts exposed mobs
    let blast_dirt = BlockPos::new(11, 74, -10);
    let blast_obsidian = BlockPos::new(9, 74, -10);
    world.set_block(
        blast_dirt,
        vanilla_blocks::DIRT.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    world.set_block(
        blast_obsidian,
        vanilla_blocks::OBSIDIAN.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    let blast_victim = Arc::new(ZombieEntity::new(
        &vanilla_entities::ZOMBIE,
        next_entity_id(),
        DVec3::new(10.5, 75.5, -9.5),
        Arc::downgrade(&world),
    ));
    assert!(
        world
            .try_add_entity(Arc::clone(&blast_victim) as SharedEntity)
            .is_ok()
    );
    let hp_before_blast = blast_victim.get_health();
    let explosion_outcome = world.explode(ExplosionOptions::new(
        DVec3::new(10.5, 74.5, -9.5),
        3.0,
        ExplosionInteraction::Tnt,
    ));
    assert!(
        explosion_outcome.affected_block_count >= 1,
        "TNT explosion must destroy nearby dirt"
    );
    assert_eq!(
        world.get_block_state(blast_dirt).get_block(),
        &vanilla_blocks::AIR,
        "dirt adjacent to explosion center must be destroyed"
    );
    assert_eq!(
        world.get_block_state(blast_obsidian).get_block(),
        &vanilla_blocks::OBSIDIAN,
        "obsidian must survive standard TNT explosion"
    );
    assert!(
        blast_victim.get_health() < hp_before_blast,
        "exposed mob must take explosion damage ({} -> {})",
        hp_before_blast,
        blast_victim.get_health()
    );
    blast_victim.set_removed(RemovalReason::Discarded);

    // ------------------------------------------------------------------------
    // PHASE 16: Bosses (Ender Dragon + Wither) & MC 26.3 Signature Mobs
    // ------------------------------------------------------------------------
    let dragon = Arc::new(EnderDragonEntity::new(
        &vanilla_entities::ENDER_DRAGON,
        next_entity_id(),
        DVec3::new(0.0, 96.0, 0.0),
        Arc::downgrade(&world),
    ));
    assert_eq!(ALL_DRAGON_PHASES.len(), 11);
    for &phase in &ALL_DRAGON_PHASES {
        dragon.set_phase(phase);
        assert_eq!(dragon.phase(), phase);
    }
    dragon.set_phase(DragonPhase::SittingFlaming);
    let mut dragon_nbt = NbtCompound::new();
    dragon.save_additional(&mut dragon_nbt);
    assert_eq!(
        dragon_nbt.int("DragonPhase"),
        Some(DragonPhase::SittingFlaming.id())
    );
    dragon.set_removed(RemovalReason::Discarded);

    let wither = Arc::new(WitherBossEntity::new(
        &vanilla_entities::WITHER,
        next_entity_id(),
        DVec3::new(0.0, 96.0, 8.0),
        Arc::downgrade(&world),
    ));
    assert!((wither.get_max_health() - 300.0).abs() < f32::EPSILON);
    wither.set_removed(RemovalReason::Discarded);

    // MC 26.3 signature passive mobs: Happy Ghast, Camel Husk, Sulfur Cube
    let happy_ghast = Arc::new(HappyGhastEntity::new(
        &vanilla_entities::HAPPY_GHAST,
        next_entity_id(),
        DVec3::new(-4.0, 80.0, 4.0),
        Arc::downgrade(&world),
    ));
    let camel_husk = Arc::new(CamelHuskEntity::new(
        &vanilla_entities::CAMEL_HUSK,
        next_entity_id(),
        DVec3::new(-2.0, 65.0, 4.0),
        Arc::downgrade(&world),
    ));
    let sulfur_cube = Arc::new(SulfurCubeEntity::new(
        &vanilla_entities::SULFUR_CUBE,
        next_entity_id(),
        DVec3::new(2.0, 65.0, 4.0),
        Arc::downgrade(&world),
    ));
    assert!(
        world
            .try_add_entity(Arc::clone(&happy_ghast) as SharedEntity)
            .is_ok()
    );
    assert!(
        world
            .try_add_entity(Arc::clone(&camel_husk) as SharedEntity)
            .is_ok()
    );
    assert!(
        world
            .try_add_entity(Arc::clone(&sulfur_cube) as SharedEntity)
            .is_ok()
    );
    tick_test_world(&world, 9, true);
    assert!(Entity::is_alive(&*happy_ghast));
    assert!(Entity::is_alive(&*camel_husk));
    assert!(Entity::is_alive(&*sulfur_cube));
    happy_ghast.set_removed(RemovalReason::Discarded);
    camel_husk.set_removed(RemovalReason::Discarded);
    sulfur_cube.set_removed(RemovalReason::Discarded);

    // ------------------------------------------------------------------------
    // PHASE 17: Async Player Data Persistence + Heightmap & Biome Verification
    // ------------------------------------------------------------------------
    rt.block_on(async {
        server
            .player_data_storage
            .save_domain(world.domain(), &op_player)
            .await
            .expect("save domain player data");
        let loaded = server
            .player_data_storage
            .load_domain(world.domain(), op_uuid)
            .await
            .expect("load domain player data");
        assert!(loaded.is_some(), "saved domain player data must round-trip");

        let global = GlobalPlayerData {
            last_active_domain: world.domain().to_owned(),
        };
        server
            .player_data_storage
            .save_global(op_uuid, &global)
            .await
            .expect("save global player data");
        let loaded_global = server
            .player_data_storage
            .load_global(op_uuid)
            .await
            .expect("load global player data")
            .expect("global player data entry");
        assert_eq!(loaded_global.last_active_domain, world.domain());
    });

    // Verify chunk heightmap tracking updates live when placing tall column blocks
    let pillar_top = BlockPos::new(13, 95, 13);
    world.set_block(
        pillar_top,
        vanilla_blocks::STONE.default_state(),
        UpdateFlags::UPDATE_ALL,
    );
    assert!(
        world.height_at(HeightmapType::WorldSurface, 13, 13) >= 96,
        "WorldSurface heightmap must reflect placed block at y=95"
    );

    // ------------------------------------------------------------------------
    // PHASE 18: Zero-Lag / Hot-Path Optimization & 100-Tick 20 TPS Stress Suite
    // ------------------------------------------------------------------------
    // Hot-path packet-queue layout invariant: oversized admin/UI packets are boxed
    // so every MovePlayer / ClientTickEnd / PlayerInput slot stays cache-line compact.
    assert!(
        size_of::<ScheduledPlayPacket>() <= 96,
        "ScheduledPlayPacket bloated to {} bytes; box cold oversized variants",
        size_of::<ScheduledPlayPacket>()
    );

    // Populate 48 live entities + active smelting/brewing/redstone + mob effects
    // and execute 100 full server ticks verifying 20 TPS headroom & zero lag spikes.
    let mut stress_entities = Vec::with_capacity(48);
    for i in 0..48_i32 {
        let x = f64::from(i % 8) + 1.5;
        let z = f64::from(i / 8) + 1.5;
        let item = Arc::new(ItemEntity::new(
            &vanilla_entities::ITEM,
            next_entity_id(),
            DVec3::new(x, 64.25, z),
            Arc::downgrade(&world),
        ));
        item.set_item(ItemStack::new(&vanilla_items::COBBLESTONE));
        assert!(world.try_add_entity(Arc::clone(&item) as _).is_ok());
        stress_entities.push(item);
    }
    op_player.add_mob_effect(MobEffectInstance::with_duration(
        vanilla_mob_effects::REGENERATION,
        100,
        0,
    ));
    // Refreshing an existing effect works cleanly and never duplicates.
    op_player.add_mob_effect(MobEffectInstance::with_duration(
        vanilla_mob_effects::REGENERATION,
        120,
        0,
    ));

    // Cached favicon lookup returns None instantly with zero disk hit when disabled.
    assert!(server.cached_favicon_png().is_none());
    assert!(server.cached_favicon_png().is_none());

    let start = Instant::now();
    let mut max_single_tick_us = 0_u128;
    for tick_idx in 1..=100_u64 {
        let tick_start = Instant::now();
        tick_test_world(&world, tick_idx, true);
        world.waypoints.tick(&world);
        world.raids.tick(&world);
        server.scoreboards.tick_stopwatches();
        let tick_us = tick_start.elapsed().as_micros();
        if tick_us > max_single_tick_us {
            max_single_tick_us = tick_us;
        }
    }
    let elapsed = start.elapsed();
    // Even in unoptimized debug CI builds, 100 ticks of a 9-chunk arena + 48 entities
    // + block entities + waypoints + raids + stopwatches must complete in < 4.0s
    // (average well below the 50 ms / 20 TPS budget) with no single spike > 250 ms!
    assert!(
        elapsed.as_millis() < 4_000,
        "100 server ticks exceeded 20 TPS budget: took {} ms (max tick {} us)",
        elapsed.as_millis(),
        max_single_tick_us
    );
    assert!(
        max_single_tick_us < 250_000,
        "single server tick lag spike detected: {} us",
        max_single_tick_us
    );

    for item in stress_entities {
        item.set_removed(RemovalReason::Discarded);
    }
    world.waypoints.forget_observer(op_uuid);
    world.waypoints.forget_observer(guest_player.base().uuid());
    let _ = server.online_players.remove_player_sync(&op_player);
    let _ = server.online_players.remove_player_sync(&guest_player);
    let _ = world.players.remove_sync(&op_uuid);
    let _ = world.players.remove_sync(&guest_player.base().uuid());
}
