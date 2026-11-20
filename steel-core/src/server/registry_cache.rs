use simdnbt::ToNbtTag;
use std::sync::Arc;

use steel_protocol::packet_traits::{ClientPacket, EncodedPacket};
use steel_protocol::packets::common::TagCollection;
use steel_protocol::{
    packets::{
        common::CUpdateTags,
        config::{CRegistryData, RegistryEntry},
    },
    utils::ConnectionProtocol,
};

use steel_registry::{
    BANNER_PATTERN_REGISTRY, BIOMES_REGISTRY, BLOCK_STATE_PROVIDER_REGISTRY,
    BLOCK_TRANSFORMER_REGISTRY, BLOCKS_REGISTRY, CAT_SOUND_VARIANT_REGISTRY, CAT_VARIANT_REGISTRY,
    CHAT_TYPE_REGISTRY, CHICKEN_SOUND_VARIANT_REGISTRY, CHICKEN_VARIANT_REGISTRY,
    COW_SOUND_VARIANT_REGISTRY, COW_VARIANT_REGISTRY, DAMAGE_TYPE_REGISTRY,
    DECORATED_POT_PATTERN_REGISTRY, DIALOG_REGISTRY, DIMENSION_TYPE_REGISTRY, ENCHANTMENT_REGISTRY,
    ENTITY_TYPE_REGISTRY, FLUID_REGISTRY, FROG_VARIANT_REGISTRY, GAME_EVENT_REGISTRY,
    INSTRUMENT_REGISTRY, ITEMS_REGISTRY, JUKEBOX_SONG_REGISTRY, PAINTING_VARIANT_REGISTRY,
    PIG_SOUND_VARIANT_REGISTRY, PIG_VARIANT_REGISTRY, POI_TYPE_REGISTRY, POTION_REGISTRY, REGISTRY,
    Registry, RegistryEntry as _, SULFUR_CUBE_ARCHETYPE_REGISTRY, TEST_ENVIRONMENT_REGISTRY,
    TEST_INSTANCE_REGISTRY, TIMELINE_REGISTRY, TRIM_MATERIAL_REGISTRY, TRIM_PATTERN_REGISTRY,
    TaggedRegistryExt, WOLF_SOUND_VARIANT_REGISTRY, WOLF_VARIANT_REGISTRY, WORLD_CLOCK_REGISTRY,
    ZOMBIE_NAUTILUS_VARIANT_REGISTRY,
};
use steel_utils::Identifier;
use steel_utils::codec::VarInt;

use steel_protocol::packet_traits::CompressionInfo;

/// Caches compressed registry packets to avoid re-compressing them for every player.
pub struct RegistryCache {
    /// The cached registry data packets, with full entry NBT.
    pub registry_packets: Arc<[EncodedPacket]>,
    /// The cached registry data packets with entry contents elided (id-only
    /// entries), for clients that negotiated our known packs.
    pub elided_registry_packets: Arc<[EncodedPacket]>,
    /// The cached tags packet.
    pub tags_packet: Arc<EncodedPacket>,
}

impl RegistryCache {
    /// Creates a new `RegistryCache` from the given registry.
    #[must_use]
    pub fn new(compression: Option<CompressionInfo>) -> Self {
        let registry_packets = Self::build_registry_packets(&REGISTRY, true);
        let tags_by_registry_packet = Self::build_tags_packet(&REGISTRY);

        let (registry_packets, tags_packet) =
            build_compressed_packets(registry_packets, tags_by_registry_packet, compression);

        // Vanilla `RegistrySynchronization.packRegistry` skips an entry's
        // contents when its known pack is in the negotiated client set. Steel
        // serves a frozen all-vanilla registry, so every entry belongs to
        // `minecraft:core` and a negotiated client gets the id-only form of
        // every packet. Tags are never elided.
        let elided_packets = Self::build_registry_packets(&REGISTRY, false);
        let elided_registry_packets = compress_registry_packets(elided_packets, compression);

        Self {
            registry_packets,
            elided_registry_packets,
            tags_packet: Arc::new(tags_packet),
        }
    }

    fn build_registry_packets(registry: &Registry, include_data: bool) -> Vec<CRegistryData> {
        let mut packets = Vec::with_capacity(32);

        macro_rules! add_registry {
            ($reg_key:expr, $field:ident) => {
                packets.push(CRegistryData::new(
                    $reg_key,
                    registry
                        .$field
                        .iter()
                        .map(|(_, entry)| {
                            let data = include_data.then(|| entry.to_nbt_tag());
                            RegistryEntry::new(entry.key.clone(), data)
                        })
                        .collect(),
                ));
            };
        }

        // Every entry is sent with its full NBT form. Vanilla entries get it from
        // their generated `ToNbtTag` implementation; because `add_registry!` is
        // generic over that trait, an entry registered by a plugin (non-vanilla)
        // is encoded the same way as long as it implements `ToNbtTag`. Registering
        // a type that cannot produce NBT is therefore a compile error rather than
        // a silently empty payload on the wire.

        // Exact vanilla MC 26.3 `RegistryDataLoader.SYNCHRONIZED_REGISTRIES`
        // order (32/32). Vanilla clients resolve registry IDs positionally in
        // some paths, so any deviation desyncs IDs.
        add_registry!(BIOMES_REGISTRY, biomes);
        add_registry!(CHAT_TYPE_REGISTRY, chat_types);
        add_registry!(TRIM_PATTERN_REGISTRY, trim_patterns);
        add_registry!(TRIM_MATERIAL_REGISTRY, trim_materials);
        add_registry!(WOLF_VARIANT_REGISTRY, wolf_variants);
        add_registry!(WOLF_SOUND_VARIANT_REGISTRY, wolf_sound_variants);
        add_registry!(PIG_VARIANT_REGISTRY, pig_variants);
        add_registry!(PIG_SOUND_VARIANT_REGISTRY, pig_sound_variants);
        add_registry!(FROG_VARIANT_REGISTRY, frog_variants);
        add_registry!(CAT_VARIANT_REGISTRY, cat_variants);
        add_registry!(CAT_SOUND_VARIANT_REGISTRY, cat_sound_variants);
        add_registry!(COW_SOUND_VARIANT_REGISTRY, cow_sound_variants);
        add_registry!(COW_VARIANT_REGISTRY, cow_variants);
        add_registry!(CHICKEN_SOUND_VARIANT_REGISTRY, chicken_sound_variants);
        add_registry!(CHICKEN_VARIANT_REGISTRY, chicken_variants);
        add_registry!(ZOMBIE_NAUTILUS_VARIANT_REGISTRY, zombie_nautilus_variants);
        add_registry!(PAINTING_VARIANT_REGISTRY, painting_variants);
        add_registry!(SULFUR_CUBE_ARCHETYPE_REGISTRY, sulfur_cube_archetypes);
        add_registry!(DIMENSION_TYPE_REGISTRY, dimension_types);
        add_registry!(DAMAGE_TYPE_REGISTRY, damage_types);
        add_registry!(BANNER_PATTERN_REGISTRY, banner_patterns);
        add_registry!(ENCHANTMENT_REGISTRY, enchantments);
        add_registry!(JUKEBOX_SONG_REGISTRY, jukebox_songs);
        add_registry!(INSTRUMENT_REGISTRY, instruments);
        add_registry!(TEST_ENVIRONMENT_REGISTRY, test_environments);
        add_registry!(TEST_INSTANCE_REGISTRY, test_instances);
        add_registry!(DIALOG_REGISTRY, dialogs);
        add_registry!(WORLD_CLOCK_REGISTRY, world_clocks);
        add_registry!(TIMELINE_REGISTRY, timelines);
        add_registry!(DECORATED_POT_PATTERN_REGISTRY, decorated_pot_patterns);
        add_registry!(BLOCK_TRANSFORMER_REGISTRY, block_transformers);
        add_registry!(BLOCK_STATE_PROVIDER_REGISTRY, block_state_providers);

        packets
    }

    fn build_tags_packet(registry: &Registry) -> CUpdateTags {
        // All 15 vanilla MC 26.3 synced tag registries (vanilla
        // `TagNetworkSerialization.serializeTagsToNetwork` over
        // `networkSafeRegistries`, skipping empty payloads: every static
        // registry plus the networkable dynamic ones). The remaining tag
        // folders (`worldgen/structure`,
        // `villager_trade`, `worldgen/feature`,
        // `worldgen/flat_level_generator_preset`, `worldgen/world_preset`) are
        // server-only and never sent.
        let mut tags_by_registry: TagCollection = Vec::with_capacity(15);
        macro_rules! add_tags {
            ($reg_key:expr, $field:ident) => {
                let mut tags: Vec<(Identifier, Vec<VarInt>)> =
                    Vec::with_capacity(registry.$field.tag_keys().count());
                for tag_key in registry.$field.tag_keys() {
                    let mut ids = Vec::with_capacity(registry.$field.iter_tag(tag_key).count());
                    for entry in registry.$field.iter_tag(tag_key) {
                        ids.push(VarInt::from(entry.id()));
                    }
                    tags.push((tag_key.clone(), ids));
                }
                tags_by_registry.push(($reg_key, tags));
            };
        }

        add_tags!(BLOCKS_REGISTRY, blocks);
        add_tags!(ITEMS_REGISTRY, items);
        add_tags!(FLUID_REGISTRY, fluids);
        add_tags!(ENTITY_TYPE_REGISTRY, entity_types);
        add_tags!(GAME_EVENT_REGISTRY, game_events);
        add_tags!(BIOMES_REGISTRY, biomes);
        add_tags!(DAMAGE_TYPE_REGISTRY, damage_types);
        add_tags!(ENCHANTMENT_REGISTRY, enchantments);
        add_tags!(INSTRUMENT_REGISTRY, instruments);
        add_tags!(PAINTING_VARIANT_REGISTRY, painting_variants);
        add_tags!(BANNER_PATTERN_REGISTRY, banner_patterns);
        add_tags!(DIALOG_REGISTRY, dialogs);
        add_tags!(TIMELINE_REGISTRY, timelines);
        add_tags!(POI_TYPE_REGISTRY, poi_types);
        add_tags!(POTION_REGISTRY, potions);

        // Build and return a CUpdateTagsPacket based on the registry data
        CUpdateTags::new(tags_by_registry)
    }
}

/// Compresses a packet.
fn compress_packet<P: ClientPacket>(
    packet: P,
    compression: Option<CompressionInfo>,
) -> Option<EncodedPacket> {
    let id = packet.get_id(ConnectionProtocol::Config);

    EncodedPacket::from_bare(packet, compression, ConnectionProtocol::Config)
        .map_err(|_| {
            log::error!("Failed to encode packet: {id:?}");
        })
        .ok()
}

/// # Panics
/// This function will panic if the compression fails.
fn compress_registry_packets(
    registry_packets: Vec<CRegistryData>,
    compression: Option<CompressionInfo>,
) -> Arc<[EncodedPacket]> {
    let mut compressed_packets = Vec::with_capacity(registry_packets.len());

    for packet in registry_packets {
        compressed_packets
            .push(compress_packet(packet, compression).expect("Failed to compress packet"));
    }

    compressed_packets.into()
}

/// # Panics
/// This function will panic if the compression fails.
#[must_use]
pub fn build_compressed_packets(
    registry_packets: Vec<CRegistryData>,
    tags_packet: CUpdateTags,
    compression: Option<CompressionInfo>,
) -> (Arc<[EncodedPacket]>, EncodedPacket) {
    let compressed_packets = compress_registry_packets(registry_packets, compression);

    let compressed_tags_packet =
        compress_packet(tags_packet, compression).expect("Failed to compress tags packet");

    (compressed_packets, compressed_tags_packet)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_synced_tag_matches_pinned_vanilla_membership() {
        steel_registry::init_vanilla_registry();
        use std::collections::{BTreeMap, BTreeSet};

        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Fixture {
            source_commit: String,
            minecraft_version: String,
            registries: BTreeMap<String, BTreeMap<String, Vec<String>>>,
        }

        let fixture: Fixture =
            serde_json::from_str(include_str!("../../test_assets/26_3_synced_tags.json"))
                .expect("pinned vanilla tag fixture must parse");
        assert_eq!(fixture.minecraft_version, "26.3");
        assert_eq!(
            fixture.source_commit,
            "538b2b167248c648b2198f2c0d56eced10dfc0cf"
        );
        let mut actual = BTreeMap::new();
        let mut seen_registries = BTreeSet::new();
        let packet = RegistryCache::build_tags_packet(&REGISTRY);
        for (registry_key, tags) in packet.tags {
            let name = registry_key.to_string();
            assert!(
                seen_registries.insert(name.clone()),
                "duplicate registry: {name}"
            );
            let mut by_id: BTreeMap<i32, String> = BTreeMap::new();
            macro_rules! index_entries {
                ($field:ident) => {
                    for (_, entry) in REGISTRY.$field.iter() {
                        assert!(
                            by_id
                                .insert(VarInt::from(entry.id()).0, entry.key.to_string())
                                .is_none()
                        );
                    }
                };
            }
            match name.as_str() {
                "minecraft:block" => index_entries!(blocks),
                "minecraft:item" => index_entries!(items),
                "minecraft:fluid" => index_entries!(fluids),
                "minecraft:entity_type" => index_entries!(entity_types),
                "minecraft:game_event" => index_entries!(game_events),
                "minecraft:worldgen/biome" => index_entries!(biomes),
                "minecraft:damage_type" => index_entries!(damage_types),
                "minecraft:enchantment" => index_entries!(enchantments),
                "minecraft:instrument" => index_entries!(instruments),
                "minecraft:painting_variant" => index_entries!(painting_variants),
                "minecraft:banner_pattern" => index_entries!(banner_patterns),
                "minecraft:dialog" => index_entries!(dialogs),
                "minecraft:timeline" => index_entries!(timelines),
                "minecraft:point_of_interest_type" => index_entries!(poi_types),
                "minecraft:potion" => index_entries!(potions),
                _ => panic!("unexpected network tag registry: {name}"),
            }
            let mut actual_tags = BTreeMap::new();
            assert!(
                !tags.is_empty(),
                "empty tag payload must not be sent: {name}"
            );
            for (key, ids) in tags {
                let mut entries = Vec::new();
                let mut seen = BTreeSet::new();
                for id in ids {
                    assert!(seen.insert(id.0), "duplicate ID in {name}/{key}");
                    entries.push(by_id.get(&id.0).expect("tag ID must resolve").clone());
                }
                entries.sort();
                assert!(actual_tags.insert(key.to_string(), entries).is_none());
            }
            // Steel deliberately also loads Fabric common (`c:`) tags. Their
            // IDs were validated above, but they are not vanilla fixture data.
            // Do not suppress them on the wire: vanilla syncs custom tags too.
            actual_tags.retain(|key, _| key.starts_with("minecraft:"));
            assert!(actual.insert(name, actual_tags).is_none());
        }
        assert_eq!(
            actual.keys().collect::<Vec<_>>(),
            fixture.registries.keys().collect::<Vec<_>>()
        );
        for (registry, expected_tags) in fixture.registries {
            let actual_tags = &actual[&registry];
            assert_eq!(
                actual_tags.keys().collect::<Vec<_>>(),
                expected_tags.keys().collect::<Vec<_>>(),
                "tag keys: {registry}"
            );
            for (tag, expected) in expected_tags {
                assert_eq!(
                    actual_tags[&tag], expected,
                    "vanilla tag mismatch: {registry}/{tag}"
                );
            }
        }
    }

    #[test]
    fn synchronized_registries_match_vanilla_order_and_known_pack_elision() {
        steel_registry::init_vanilla_registry();
        let full = RegistryCache::build_registry_packets(&REGISTRY, true);
        let elided = RegistryCache::build_registry_packets(&REGISTRY, false);
        let expected = [
            BIOMES_REGISTRY,
            CHAT_TYPE_REGISTRY,
            TRIM_PATTERN_REGISTRY,
            TRIM_MATERIAL_REGISTRY,
            WOLF_VARIANT_REGISTRY,
            WOLF_SOUND_VARIANT_REGISTRY,
            PIG_VARIANT_REGISTRY,
            PIG_SOUND_VARIANT_REGISTRY,
            FROG_VARIANT_REGISTRY,
            CAT_VARIANT_REGISTRY,
            CAT_SOUND_VARIANT_REGISTRY,
            COW_SOUND_VARIANT_REGISTRY,
            COW_VARIANT_REGISTRY,
            CHICKEN_SOUND_VARIANT_REGISTRY,
            CHICKEN_VARIANT_REGISTRY,
            ZOMBIE_NAUTILUS_VARIANT_REGISTRY,
            PAINTING_VARIANT_REGISTRY,
            SULFUR_CUBE_ARCHETYPE_REGISTRY,
            DIMENSION_TYPE_REGISTRY,
            DAMAGE_TYPE_REGISTRY,
            BANNER_PATTERN_REGISTRY,
            ENCHANTMENT_REGISTRY,
            JUKEBOX_SONG_REGISTRY,
            INSTRUMENT_REGISTRY,
            TEST_ENVIRONMENT_REGISTRY,
            TEST_INSTANCE_REGISTRY,
            DIALOG_REGISTRY,
            WORLD_CLOCK_REGISTRY,
            TIMELINE_REGISTRY,
            DECORATED_POT_PATTERN_REGISTRY,
            BLOCK_TRANSFORMER_REGISTRY,
            BLOCK_STATE_PROVIDER_REGISTRY,
        ];
        assert_eq!(full.len(), expected.len());
        assert_eq!(elided.len(), expected.len());
        for ((full, elided), expected) in full.iter().zip(&elided).zip(&expected) {
            assert_eq!(&full.registry, expected);
            assert_eq!(full.registry, elided.registry);
            assert_eq!(full.entries.len(), elided.entries.len());
            assert!(!full.entries.is_empty(), "empty registry: {expected}");
            for (full, elided) in full.entries.iter().zip(&elided.entries) {
                assert_eq!(full.id, elided.id);
                assert!(full.data.is_some(), "missing full data: {}", full.id);
                assert!(elided.data.is_none(), "unelided entry: {}", elided.id);
            }
        }
    }
}
