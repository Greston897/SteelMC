//! Persistent, entity-driven raid waves. Wave progress is never inferred from elapsed time.

use std::{collections::BTreeMap, sync::Arc};

use glam::IVec3;
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, types::Difficulty};
use uuid::Uuid;

use serde::{Deserialize, Serialize};
use steel_registry::blocks::block_state_ext::BlockStateExt as _;
use steel_registry::{
    RegistryExt as _, TaggedRegistryExt as _, vanilla_entities, vanilla_mob_effects,
};

use crate::chunk::heightmap::HeightmapType;
use crate::entity::spawn::{EntitySpawnPlacement, EntitySpawnReason, EntitySpawnRequest};
use crate::entity::spawn_entity;
use crate::entity::{Entity as _, LivingEntity as _, MobEffectInstance};
use crate::poi::OccupationStatus;
use crate::world::{LevelReader, World};

/// Maximum zero-based effect amplifier; the raid's internal level is amplifier + 1.
pub const MAX_OMEN_LEVEL: i32 = 4;
const VILLAGE_SCAN_RADIUS: i32 = 64;
const RAID_TIMEOUT_TICKS: i32 = 48_000;
const PRE_WAVE_TICKS: i32 = 300;

/// Lifecycle state, persisted so a restart cannot turn a lost raid into a victory.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaidStatus {
    #[default]
    Ongoing,
    Victory,
    Loss,
}

/// One raid, including the UUIDs of every member of its current wave.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Raid {
    pub center: IVec3,
    pub uuid: Uuid,
    pub total_health: i32,
    pub leader: Option<Uuid>,
    /// Zero-based omen amplifier, retained for compatibility with older saves.
    pub omen_level: i32,
    pub ticks_active: i32,
    pub waves_cleared: i32,
    pub waves_spawned: i32,
    pub num_groups: i32,
    pub cooldown: i32,
    pub post_raid_ticks: i32,
    pub celebration_ticks: i32,
    pub status: RaidStatus,
    pub raiders: Vec<Uuid>,
    pub heroes: Vec<Uuid>,
}

impl Default for Raid {
    fn default() -> Self {
        Self {
            center: IVec3::ZERO,
            uuid: Uuid::new_v4(),
            total_health: 0,
            leader: None,
            omen_level: 0,
            ticks_active: 0,
            waves_cleared: 0,
            waves_spawned: 0,
            num_groups: 5,
            cooldown: PRE_WAVE_TICKS,
            post_raid_ticks: 0,
            celebration_ticks: 0,
            status: RaidStatus::Ongoing,
            raiders: Vec::new(),
            heroes: Vec::new(),
        }
    }
}

impl Raid {
    /// Difficulty-selected base groups, plus one bonus wave for omen II or higher.
    #[must_use]
    pub const fn wave_count(&self) -> i32 {
        self.num_groups + if self.omen_level > 0 { 1 } else { 0 }
    }

    #[must_use]
    pub const fn is_victory(&self) -> bool {
        matches!(self.status, RaidStatus::Victory)
    }
}

const fn difficulty_groups(difficulty: Difficulty) -> i32 {
    match difficulty {
        Difficulty::Peaceful => 0,
        Difficulty::Easy => 3,
        Difficulty::Normal => 5,
        Difficulty::Hard => 7,
    }
}

// Vanilla RaiderType order: vindicator, evoker, pillager, witch, ravager.
const WAVE_COUNTS: [[i32; 8]; 5] = [
    [0, 0, 2, 0, 1, 4, 2, 5],
    [0, 0, 0, 0, 0, 1, 1, 2],
    [0, 4, 3, 3, 4, 4, 4, 2],
    [0, 0, 0, 0, 3, 0, 0, 1],
    [0, 0, 0, 1, 0, 1, 0, 2],
];

fn bonus_spawns(kind: usize, wave: i32, difficulty: Difficulty, bonus: bool) -> i32 {
    let max = match kind {
        0 | 2 => match difficulty {
            Difficulty::Easy => rand::random_range(0..=1),
            Difficulty::Normal => 1,
            Difficulty::Hard => 2,
            Difficulty::Peaceful => 0,
        },
        3 if difficulty != Difficulty::Easy && wave > 2 && wave != 4 => 1,
        4 if difficulty != Difficulty::Easy && bonus => 1,
        _ => 0,
    };
    rand::random_range(0..=max)
}

/// TOML documents require a table root, including when the raid list is empty.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RaidSavedData {
    #[serde(default)]
    pub raids: Vec<Raid>,
}

/// Raids currently alive per world.
#[derive(Debug, Default)]
pub struct RaidManager {
    raids: SyncMutex<Vec<Raid>>,
    viewers: SyncMutex<BTreeMap<Uuid, Vec<Uuid>>>,
}

impl RaidManager {
    /// Creates an empty manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a manager over raids loaded from disk.
    #[must_use]
    pub const fn from_raids(raids: Vec<Raid>) -> Self {
        Self {
            raids: SyncMutex::new(raids),
            viewers: SyncMutex::new(BTreeMap::new()),
        }
    }

    /// Snapshot of the raids in the form persisted to disk.
    #[must_use]
    pub fn to_persistent(&self) -> Vec<Raid> {
        self.raids.lock().clone()
    }

    /// Returns the raid whose center lies within the village radius of `pos`.
    #[must_use]
    pub fn raid_at(&self, pos: IVec3) -> Option<Raid> {
        self.raids
            .lock()
            .iter()
            .find(|raid| within_village_radius(raid.center, pos))
            .cloned()
    }

    /// Returns every active raid, in start order.
    #[must_use]
    pub fn raids(&self) -> Vec<Raid> {
        self.raids.lock().clone()
    }

    /// Starts (or extends) a raid centered on `center`, mirroring vanilla
    /// `Raid.raiseOmenLevel` when a raid already covers the village.
    ///
    /// Returns the resulting raid.
    pub fn raise_omen(&self, center: IVec3, omen_level: i32) -> Raid {
        let omen_level = omen_level.clamp(0, MAX_OMEN_LEVEL);
        let mut raids = self.raids.lock();
        if let Some(existing) = raids
            .iter_mut()
            .find(|raid| within_village_radius(raid.center, center))
        {
            existing.omen_level = (existing.omen_level + omen_level + 1).min(MAX_OMEN_LEVEL);
            return existing.clone();
        }
        let raid = Raid {
            center,
            omen_level,
            ticks_active: 0,
            waves_cleared: 0,
            ..Raid::default()
        };
        raids.push(raid.clone());
        raid
    }

    /// Removes a raid once it is defeated, matching vanilla `Raid.stop`.
    pub fn stop(&self, center: IVec3) -> bool {
        let mut raids = self.raids.lock();
        let before = raids.len();
        raids.retain(|raid| !within_village_radius(raid.center, center));
        raids.len() != before
    }

    /// Called synchronously from the death path, before entities disappear from
    /// the manager. Only actual kills earn hero credit; unloading never does.
    pub fn raider_died(&self, raider: Uuid, killer: Option<Uuid>) {
        for raid in self.raids.lock().iter_mut() {
            if raid.status != RaidStatus::Ongoing {
                continue;
            }
            if let Some(index) = raid.raiders.iter().position(|id| *id == raider) {
                raid.raiders.swap_remove(index);
                if let Some(killer) = killer
                    && !raid.heroes.contains(&killer)
                {
                    raid.heroes.push(killer);
                }
            }
        }
    }

    /// Advances only loaded raids. Surviving or unloaded members cannot be
    /// converted into cleared waves by a timer or by restarting the server.
    pub fn tick(&self, world: &Arc<World>) {
        let mut victories = Vec::new();
        self.raids.lock().retain_mut(|raid| {
            if !world.get_game_rule(&steel_registry::vanilla_game_rules::RAIDS)
                || world.difficulty() == Difficulty::Peaceful
            {
                return false;
            }
            if world
                .as_ref()
                .height_at(HeightmapType::WorldSurface, raid.center.x, raid.center.z)
                .is_none()
            {
                return true;
            }
            if raid.status != RaidStatus::Ongoing {
                raid.celebration_ticks += 1;
                return raid.celebration_ticks < 600;
            }
            raid.ticks_active += 1;
            if raid.ticks_active >= RAID_TIMEOUT_TICKS {
                return false;
            }
            if !is_village(world, raid.center) {
                if raid.waves_spawned == 0 {
                    return false;
                }
                raid.status = RaidStatus::Loss;
                return true;
            }
            if raid.waves_spawned == 0 {
                raid.num_groups = difficulty_groups(world.difficulty());
            }
            // Keep missing UUIDs: a member may be in an unloaded chunk. Deaths
            // are removed by raider_died, not by failure of an entity lookup.
            raid.raiders.retain(|id| {
                let Some(entity) = world.get_entity_by_uuid(id) else {
                    return true;
                };
                let pos = entity.block_position().0;
                let delta = pos.as_i64vec3() - raid.center.as_i64vec3();
                !entity.is_removed() && delta.length_squared() < 12_544
            });
            if !raid.raiders.is_empty() {
                if raid.ticks_active % 20 == 0 {
                    for id in &raid.raiders {
                        if let Some(entity) = world.get_entity_by_uuid(id)
                            && let Some(mob) = entity.as_pathfinder_mob()
                            && mob.target().is_none()
                        {
                            mob.move_to_pos(raid.center.as_dvec3(), 1.0);
                        }
                    }
                }
                return true;
            }
            if raid.waves_spawned > raid.waves_cleared {
                raid.waves_cleared = raid.waves_spawned;
                raid.cooldown = PRE_WAVE_TICKS;
            }
            if raid.waves_spawned >= raid.wave_count() {
                raid.post_raid_ticks += 1;
                if raid.post_raid_ticks > 40 {
                    raid.status = RaidStatus::Victory;
                    victories.push(raid.clone());
                }
                return true;
            }
            if raid.cooldown > 0 {
                raid.cooldown -= 1;
                return true;
            }
            if let Some(pos) = find_spawn_pos(world, raid.center) {
                spawn_wave(world, raid, pos);
            } else {
                // No legal terrain is not a cleared wave or a victory.
                return false;
            }
            true
        });
        for raid in victories {
            Self::award_hero_of_the_village(world, &raid);
        }
        self.sync_boss_bars(world);
    }

    fn sync_boss_bars(&self, world: &Arc<World>) {
        use steel_protocol::packets::game::{BossBarColor, BossBarOverlay, CBossEvent};
        use text_components::{TextComponent, translation::TranslatedMessage};
        let raids = self.raids();
        let mut viewers = self.viewers.lock();
        viewers.retain(|uuid, shown| {
            if raids.iter().any(|raid| raid.uuid == *uuid) {
                return true;
            }
            for player in shown.iter().filter_map(|id| world.players.get_by_uuid(id)) {
                player.send_packet(CBossEvent::remove(*uuid));
            }
            false
        });
        for raid in &raids {
            let shown = viewers.entry(raid.uuid).or_default();
            shown.retain(|id| {
                let Some(player) = world.players.get_by_uuid(id) else {
                    return false;
                };
                if within_village_radius(raid.center, player.block_position().0) {
                    return true;
                }
                player.send_packet(CBossEvent::remove(raid.uuid));
                false
            });
            let health: f32 = raid
                .raiders
                .iter()
                .filter_map(|id| world.get_entity_by_uuid(id))
                .filter_map(|entity| {
                    entity
                        .as_living_entity()
                        .map(|living| living.get_health().max(0.0))
                })
                .sum();
            let progress = if raid.raiders.is_empty() && raid.status == RaidStatus::Ongoing {
                1.0 - raid.cooldown as f32 / PRE_WAVE_TICKS as f32
            } else if raid.total_health > 0 {
                health / raid.total_health as f32
            } else {
                0.0
            };
            let key = match raid.status {
                RaidStatus::Ongoing => "event.minecraft.raid",
                RaidStatus::Victory => "event.minecraft.raid.victory",
                RaidStatus::Loss => "event.minecraft.raid.defeat",
            };
            let name = TextComponent::translated(TranslatedMessage {
                key: key.into(),
                fallback: None,
                args: None,
            });
            world.players.iter_players(|id, player| {
                if !within_village_radius(raid.center, player.block_position().0) {
                    return true;
                }
                if !shown.contains(id) {
                    player.send_packet(CBossEvent::add(
                        raid.uuid,
                        name.clone(),
                        progress.clamp(0.0, 1.0),
                        BossBarColor::Red,
                        BossBarOverlay::Progress,
                        0,
                    ));
                    shown.push(*id);
                } else if raid.ticks_active % 20 == 0 || raid.status != RaidStatus::Ongoing {
                    player.send_packet(CBossEvent::update_progress(
                        raid.uuid,
                        progress.clamp(0.0, 1.0),
                    ));
                    player.send_packet(CBossEvent::update_name(raid.uuid, name.clone()));
                }
                true
            });
        }
    }

    fn award_hero_of_the_village(world: &Arc<World>, raid: &Raid) {
        world.players.iter_players(|uuid, player| {
            if !raid.heroes.contains(uuid)
                || !crate::entity::Entity::is_alive(player.as_ref())
                || player.is_spectator()
            {
                return true;
            }
            player.add_mob_effect(MobEffectInstance::with_duration(
                vanilla_mob_effects::HERO_OF_THE_VILLAGE,
                48_000,
                raid.omen_level,
            ));
            player.trigger_advancement_with_context(
                &steel_registry::vanilla_criteria_triggers::HERO_OF_THE_VILLAGE,
                &crate::player::advancement::predicate::TriggerContext::at(
                    &player.get_world(),
                    player.position(),
                ),
            );
            true
        });
    }
}

/// Searches loaded surface terrain; never creates a wave in missing chunks.
fn find_spawn_pos(world: &Arc<World>, center: IVec3) -> Option<BlockPos> {
    let angle: f64 = rand::random_range(0.0..std::f64::consts::TAU);
    for radius in [64.0, 32.0, 16.0] {
        for attempt in 0..20 {
            let angle = angle + f64::from(attempt) * std::f64::consts::PI / 8.0;
            let x = center.x + (angle.cos() * radius).floor() as i32;
            let z = center.z + (angle.sin() * radius).floor() as i32;
            let Some(y) = world.as_ref().height_at(HeightmapType::WorldSurface, x, z) else {
                continue;
            };
            if (y - center.y).abs() > 96 {
                continue;
            }
            let pos = BlockPos::new(x, y, z);
            if !world.get_block_state(BlockPos::new(x, y - 1, z)).is_solid()
                || !world.get_block_state(pos).is_air()
                || !world.get_block_state(BlockPos::new(x, y + 1, z)).is_air()
                || !world.get_block_state(BlockPos::new(x, y + 2, z)).is_air()
            {
                continue;
            }
            return Some(pos);
        }
    }
    None
}

/// Shared vanilla captain banner, including all eight ordered pattern layers.
pub(crate) fn ominous_banner() -> steel_registry::item_stack::ItemStack {
    use steel_registry::{DyeColor, RegistryHolder, vanilla_banner_patterns as patterns};
    use steel_registry::data_components::{BannerPatternLayer, BannerPatternLayers, Rarity};
    use steel_registry::data_components::vanilla_components::{BANNER_PATTERNS, ITEM_NAME, RARITY};
    use text_components::{TextComponent, translation::TranslatedMessage};
    let layers = [
        (&patterns::RHOMBUS_MIDDLE, DyeColor::Cyan),
        (&patterns::STRIPE_BOTTOM, DyeColor::LightGray),
        (&patterns::STRIPE_CENTER, DyeColor::Gray),
        (&patterns::BORDER, DyeColor::LightGray),
        (&patterns::STRIPE_MIDDLE, DyeColor::Black),
        (&patterns::HALF_HORIZONTAL, DyeColor::LightGray),
        (&patterns::CIRCLE_MIDDLE, DyeColor::LightGray),
        (&patterns::BORDER, DyeColor::Black),
    ].into_iter().map(|(pattern, color)| BannerPatternLayer::new(RegistryHolder::reference(pattern), color)).collect();
    let mut banner = steel_registry::item_stack::ItemStack::new(&steel_registry::vanilla_items::WHITE_BANNER);
    banner.set(BANNER_PATTERNS, BannerPatternLayers::new(layers));
    banner.set(ITEM_NAME, TextComponent::translated(TranslatedMessage { key: "block.minecraft.ominous_banner".into(), fallback: None, args: None }));
    banner.set(RARITY, Rarity::Uncommon);
    banner
}

fn spawn_wave(world: &Arc<World>, raid: &mut Raid, pos: BlockPos) {
    let wave = raid.waves_spawned + 1;
    raid.total_health = 0;
    raid.leader = None;
    let bonus = wave > raid.num_groups;
    let index = if bonus { raid.num_groups } else { wave }.clamp(0, 7) as usize;
    let types = [&vanilla_entities::VINDICATOR, &vanilla_entities::EVOKER,
        &vanilla_entities::PILLAGER, &vanilla_entities::WITCH, &vanilla_entities::RAVAGER];
    for (kind, entity_type) in types.into_iter().enumerate() {
        let count = WAVE_COUNTS[kind][index] + bonus_spawns(kind, wave, world.difficulty(), bonus);
        for number in 0..count {
            let request = EntitySpawnRequest {
                entity_type, placement: EntitySpawnPlacement::Block {
                    pos, try_move_down: false, moved_up: false,
                },
                reason: EntitySpawnReason::Event, finalize_spawn: true,
                play_ambient_sound: false, item_stack: None, user_is_operator: false,
            };
            if let Ok(entity) = spawn_entity(world, request) {
                if let Some(mob) = entity.as_mob() { mob.set_persistence_required(); }
                raid.total_health += entity.as_living_entity().map_or(0, |living| living.get_health().ceil() as i32);
                if let Some(living) = entity.as_living_entity() {
                    if kind == 0 {
                        living.with_equipment_slot_mut(crate::inventory::equipment::EquipmentSlot::MainHand,
                            &mut |item| *item = steel_registry::item_stack::ItemStack::new(&steel_registry::vanilla_items::IRON_AXE));
                    }
                    if raid.leader.is_none() && kind <= 2 {
                        living.with_equipment_slot_mut(crate::inventory::equipment::EquipmentSlot::Head,
                            &mut |item| *item = ominous_banner());
                        if let Some(mob) = entity.as_mob() {
                            mob.set_equipment_drop_chance(crate::inventory::equipment::EquipmentSlot::Head, 2.0);
                        }
                        raid.leader = Some(entity.uuid());
                    }
                }
                raid.raiders.push(entity.uuid());
                if kind == 4 {
                    let rider = if wave == 5 { Some(&vanilla_entities::PILLAGER) }
                        else if wave >= 7 { Some(if number == 0 { &vanilla_entities::EVOKER }
                            else { &vanilla_entities::VINDICATOR }) } else { None };
                    if let Some(entity_type) = rider
                        && let Ok(rider) = spawn_entity(world, EntitySpawnRequest { entity_type, ..request }) {
                        if let Some(mob) = rider.as_mob() { mob.set_persistence_required(); }
                        rider.start_riding(&entity);
                        raid.total_health += rider.as_living_entity().map_or(0, |living| living.get_health().ceil() as i32);
                        raid.raiders.push(rider.uuid());
                    }
                }
            }
        }
    }
    // Failure to create mobs must never manufacture progress.
    if !raid.raiders.is_empty() { raid.waves_spawned = wave; }
}

/// Vanilla `Raid.isVillage` distance rule: the same radius village detection uses.
const fn within_village_radius(center: IVec3, pos: IVec3) -> bool {
    let dx = center.x as i64 - pos.x as i64;
    let dy = center.y as i64 - pos.y as i64;
    let dz = center.z as i64 - pos.z as i64;
    dx * dx + dy * dy + dz * dz < 9_216
}

/// An occupied village POI, not an unclaimed bell, establishes a village.
fn village_poi(type_id: usize) -> bool {
    steel_registry::REGISTRY.poi_types.by_id(type_id).is_some_and(|poi| {
        steel_registry::REGISTRY.poi_types.is_in_tag(
            poi, &steel_utils::Identifier::vanilla_static("village"),
        )
    })
}

#[must_use]
pub fn is_village(world: &Arc<World>, pos: IVec3) -> bool {
    is_village_world(world, BlockPos(pos))
}

/// Village distance is measured in sections, including the vertical axis.
#[must_use]
pub fn is_village_world(world: &World, pos: BlockPos) -> bool {
    world.poi_storage.lock().get_in_horizontal_square(
        &village_poi, pos, 32, OccupationStatus::Occupied,
    ).iter().any(|(poi, _)| {
        let delta = (poi.0 >> 4_i32) - (pos.0 >> 4_i32);
        delta.abs().max_element() <= 1
    })
}

/// Vanilla averages occupied village POIs in a 64-block sphere to center a raid.
#[must_use]
pub fn village_center_world(world: &World, pos: BlockPos) -> Option<IVec3> {
    let matches = world.poi_storage.lock().get_in_horizontal_square(
        &village_poi, pos, VILLAGE_SCAN_RADIUS, OccupationStatus::Occupied,
    );
    let mut total = glam::I64Vec3::ZERO;
    let mut count = 0;
    for (poi, _) in matches {
        let delta = poi.0.as_i64vec3() - pos.0.as_i64vec3();
        if delta.length_squared() <= i64::from(VILLAGE_SCAN_RADIUS).pow(2) {
            total += poi.0.as_i64vec3();
            count += 1;
        }
    }
    (count > 0).then(|| IVec3::new(
        total.x.div_euclid(count) as i32, total.y.div_euclid(count) as i32,
        total.z.div_euclid(count) as i32,
    ))
}

#[must_use]
pub fn village_center(world: &Arc<World>, pos: IVec3) -> Option<IVec3> {
    village_center_world(world, BlockPos(pos))
}

#[cfg(test)]
mod persistence_tests {
    use super::*;
    use steel_utils::saved_data::{SavedDataManager, names};
    #[tokio::test]
    async fn empty_and_active_raids_round_trip_through_real_disk_toml() {
        let directory = std::env::temp_dir().join(format!("steel-raid-save-{}", uuid::Uuid::new_v4()));
        let manager = SavedDataManager::new(Some(&directory));
        for raids in [Vec::new(), vec![Raid { center: IVec3::new(-5, 64, 8), omen_level: 3, ticks_active: 129, waves_cleared: 1, ..Raid::default() }]] {
            let data = RaidSavedData { raids };
            manager.save(names::RAIDS, &data).await.unwrap();
            let loaded: RaidSavedData = manager.load_or_default(names::RAIDS).await.unwrap();
            assert_eq!(loaded, data);
        }
        std::fs::remove_dir_all(directory).unwrap();
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use crate::behavior::init_behaviors;
    use crate::test_support::{fresh_test_world_with_difficulty, insert_ready_full_chunk};
    use steel_utils::ChunkPos;

    #[test]
    fn difficulty_not_omen_selects_base_groups_and_omen_adds_only_one_bonus() {
        for (difficulty, expected) in [(Difficulty::Easy, 3), (Difficulty::Normal, 5), (Difficulty::Hard, 7)] {
            for omen_level in 0..=4 {
                let raid = Raid { num_groups: difficulty_groups(difficulty), omen_level, ..Raid::default() };
                assert_eq!(raid.wave_count(), expected + i32::from(omen_level > 0));
                assert!(!raid.is_victory());
            }
        }
        assert_eq!(difficulty_groups(Difficulty::Peaceful), 0);
    }

    #[test]
    fn omen_absorption_is_additive_and_capped() {
        let manager = RaidManager::new();
        assert_eq!(manager.raise_omen(IVec3::ZERO, 0).omen_level, 0);
        assert_eq!(manager.raise_omen(IVec3::ZERO, 0).omen_level, 1);
        assert_eq!(manager.raise_omen(IVec3::ZERO, 1).omen_level, 3);
        assert_eq!(manager.raise_omen(IVec3::ZERO, 4).omen_level, 4);
        assert_eq!(manager.raids().len(), 1);
    }

    #[test]
    fn death_removes_only_members_and_credits_unique_killers() {
        let member = Uuid::new_v4();
        let other = Uuid::new_v4();
        let killer = Uuid::new_v4();
        let manager = RaidManager::from_raids(vec![Raid { raiders: vec![member, other], ..Raid::default() }]);
        manager.raider_died(Uuid::new_v4(), Some(killer));
        assert!(manager.raids()[0].heroes.is_empty());
        manager.raider_died(member, Some(killer));
        assert_eq!(manager.raids()[0].raiders, vec![other]);
        manager.raider_died(other, Some(killer));
        assert_eq!(manager.raids()[0].heroes, vec![killer]);
        assert_eq!(manager.raids()[0].waves_cleared, 0);
    }

    #[test]
    fn real_spawned_wave_does_not_clear_with_elapsed_time_and_death_hook_removes_it() {
        steel_registry::init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world_with_difficulty("raid_real_wave", Difficulty::Normal);
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let center = IVec3::new(0, 64, 0);
        let home = steel_registry::REGISTRY.poi_types.iter()
            .find(|(_, poi)| poi.key.path.as_ref() == "home").unwrap().0;
        world.poi_storage.lock().add(BlockPos(center), home, 1);
        assert!(world.poi_storage.lock().reserve_ticket(BlockPos(center)));
        assert!(is_village(world, center));
        let mut raid = Raid { center, ..Raid::default() };
        spawn_wave(world, &mut raid, BlockPos::new(2, 64, 2));
        assert!((4..=5).contains(&raid.raiders.len()));
        let captain = world.get_entity_by_uuid(&raid.leader.expect("wave captain")).unwrap();
        let banner = captain.as_living_entity().unwrap().equipment_in_slot(crate::inventory::equipment::EquipmentSlot::Head);
        assert_eq!(banner.get(steel_registry::data_components::vanilla_components::BANNER_PATTERNS).unwrap().layers().len(), 8);
        let members = raid.raiders.clone();
        *world.raids.raids.lock() = vec![raid];
        for _ in 0..1_201 { world.raids.tick(world); }
        let ongoing = world.raids.raids();
        assert_eq!(ongoing[0].waves_spawned, 1);
        assert_eq!(ongoing[0].waves_cleared, 0);
        assert_eq!(ongoing[0].raiders.len(), members.len());
        for id in members {
            let entity = world.get_entity_by_uuid(&id).unwrap();
            let living = entity.as_living_entity().unwrap();
            living.set_health(0.0);
            living.die(&crate::entity::damage::DamageSource::environment(steel_registry::vanilla_damage_types::GENERIC));
        }
        world.raids.tick(world);
        assert!(world.raids.raids()[0].raiders.is_empty());
        assert_eq!(world.raids.raids()[0].waves_cleared, 1);
        assert!(!world.raids.raids()[0].is_victory());
    }

    #[test]
    fn missing_members_survive_serialization_without_false_clear() {
        let raid = Raid { waves_spawned: 2, waves_cleared: 1, raiders: vec![Uuid::new_v4()], ..Raid::default() };
        let document = toml::to_string(&RaidSavedData { raids: vec![raid.clone()] }).unwrap();
        let restored: RaidSavedData = toml::from_str(&document).unwrap();
        assert_eq!(restored.raids, vec![raid]);
        assert!(!restored.raids[0].is_victory());
    }
}

#[cfg(test)]
mod outcome_tests {
    use super::*;
    use crate::test_support::{fresh_test_world, insert_ready_full_chunk, TestPlayerBuilder};
    use crate::entity::next_entity_id;
    use steel_utils::ChunkPos;

    #[test]
    fn victory_waits_forty_ticks_rewards_only_credited_hero_then_expires() {
        steel_registry::init_vanilla_registry();
        crate::behavior::init_behaviors();
        let fixture = fresh_test_world("raid_victory");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let center = IVec3::new(0, 64, 0);
        let home = steel_registry::REGISTRY.poi_types.iter().find(|(_, poi)| poi.key.path.as_ref() == "home").unwrap().0;
        world.poi_storage.lock().add(BlockPos(center), home, 1);
        world.poi_storage.lock().reserve_ticket(BlockPos(center));
        let hero = TestPlayerBuilder::new(Arc::clone(world), "Hero", next_entity_id()).build();
        let bystander = TestPlayerBuilder::new(Arc::clone(world), "Bystander", next_entity_id()).build();
        world.players.insert(Arc::clone(&hero));
        world.players.insert(Arc::clone(&bystander));
        // Hard raid with omen III: seven normal groups and exactly one bonus.
        *world.raids.raids.lock() = vec![Raid { center, num_groups: 7, omen_level: 2,
            waves_spawned: 8, waves_cleared: 8, heroes: vec![hero.uuid()], ..Raid::default() }];
        for _ in 0..40 { world.raids.tick(world); }
        assert!(!world.raids.raids()[0].is_victory());
        assert!(!hero.has_mob_effect(vanilla_mob_effects::HERO_OF_THE_VILLAGE));
        world.raids.tick(world);
        assert!(world.raids.raids()[0].is_victory());
        let effect = hero.mob_effect(vanilla_mob_effects::HERO_OF_THE_VILLAGE).unwrap();
        assert_eq!(effect.duration(), 48_000);
        assert_eq!(effect.amplifier(), 2);
        assert!(!bystander.has_mob_effect(vanilla_mob_effects::HERO_OF_THE_VILLAGE));
        for _ in 0..600 { world.raids.tick(world); }
        assert!(world.raids.raids().is_empty());
        assert!(world.raids.viewers.lock().is_empty());
    }

    #[test]
    fn destroyed_village_is_loss_and_timeout_is_global_not_per_wave() {
        let fixture = fresh_test_world("raid_loss_timeout");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        *world.raids.raids.lock() = vec![Raid { waves_spawned: 1, ..Raid::default() }];
        world.raids.tick(world);
        assert_eq!(world.raids.raids()[0].status, RaidStatus::Loss);
        assert!(!world.raids.raids()[0].is_victory());
        *world.raids.raids.lock() = vec![Raid { ticks_active: 47_999, ..Raid::default() }];
        world.raids.tick(world);
        assert!(world.raids.raids().is_empty());
    }
}

#[cfg(test)]
mod complete_wave_tests {
    use super::*;
    use crate::test_support::{fresh_test_world_with_difficulty, insert_ready_full_chunk};
    use steel_utils::{ChunkPos, types::UpdateFlags};
    use steel_registry::vanilla_blocks;

    #[test]
    fn easy_raid_spawns_three_real_waves_and_requires_all_three_to_die() {
        steel_registry::init_vanilla_registry();
        crate::behavior::init_behaviors();
        let fixture = fresh_test_world_with_difficulty("raid_three_waves", Difficulty::Easy);
        let world = &fixture.world;
        for x in -1..=1 { for z in -1..=1 { insert_ready_full_chunk(world, ChunkPos::new(x, z)); } }
        for x in -16..32 { for z in -16..32 {
            world.set_block(BlockPos::new(x, 63, z), vanilla_blocks::STONE.default_state(), UpdateFlags::UPDATE_NONE);
        } }
        let home = steel_registry::REGISTRY.poi_types.iter().find(|(_, poi)| poi.key.path.as_ref() == "home").unwrap().0;
        let center = IVec3::new(0, 64, 0);
        world.poi_storage.lock().add(BlockPos(center), home, 1);
        world.poi_storage.lock().reserve_ticket(BlockPos(center));
        world.raids.raise_omen(center, 0);
        for wave in 1..=3 {
            for _ in 0..301 { world.raids.tick(world); }
            let raid = world.raids.raids()[0].clone();
            assert_eq!(raid.waves_spawned, wave);
            assert_eq!(raid.waves_cleared, wave - 1);
            assert!(!raid.raiders.is_empty());
            assert!(!raid.is_victory());
            for id in raid.raiders {
                let entity = world.get_entity_by_uuid(&id).unwrap();
                let living = entity.as_living_entity().unwrap();
                living.set_health(0.0);
                living.die(&crate::entity::damage::DamageSource::environment(steel_registry::vanilla_damage_types::GENERIC));
            }
        }
        for _ in 0..41 { world.raids.tick(world); }
        let raid = &world.raids.raids()[0];
        assert_eq!(raid.waves_cleared, 3);
        assert!(raid.is_victory());
        assert!(raid.heroes.is_empty(), "environmental kills do not fabricate hero credit");
    }
}

#[cfg(test)]
mod village_and_unload_tests {
    use super::*;
    use crate::test_support::{fresh_test_world, insert_ready_full_chunk};
    use steel_utils::ChunkPos;

    #[test]
    fn unclaimed_bell_is_not_a_village_and_section_distance_includes_height() {
        let fixture = fresh_test_world("raid_village_sections");
        let world = &fixture.world;
        let meeting = steel_registry::REGISTRY.poi_types.iter().find(|(_, poi)| poi.key.path.as_ref() == "meeting").unwrap().0;
        let center = BlockPos::new(0, 64, 0);
        world.poi_storage.lock().add(center, meeting, 32);
        assert!(!is_village(world, center.0));
        assert!(world.poi_storage.lock().reserve_ticket(center));
        assert!(is_village(world, center.0));
        assert!(is_village(world, IVec3::new(31, 95, 31)));
        assert!(!is_village(world, IVec3::new(32, 64, 0)));
        assert!(!is_village(world, IVec3::new(0, 96, 0)));
    }

    #[test]
    fn loaded_raid_with_missing_member_does_not_turn_unload_into_wave_clear() {
        let fixture = fresh_test_world("raid_missing_member");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let home = steel_registry::REGISTRY.poi_types.iter().find(|(_, poi)| poi.key.path.as_ref() == "home").unwrap().0;
        let center = BlockPos::new(0, 64, 0);
        world.poi_storage.lock().add(center, home, 1);
        world.poi_storage.lock().reserve_ticket(center);
        let member = Uuid::new_v4();
        *world.raids.raids.lock() = vec![Raid { center: center.0, waves_spawned: 1, raiders: vec![member], ..Raid::default() }];
        for _ in 0..1_201 { world.raids.tick(world); }
        let raid = &world.raids.raids()[0];
        assert_eq!(raid.raiders, vec![member]);
        assert_eq!(raid.waves_cleared, 0);
        assert!(!raid.is_victory());
    }
}
