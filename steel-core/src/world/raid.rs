//! Persistent, entity-driven raid waves. Wave progress is never inferred from elapsed time.

use std::{collections::BTreeMap, sync::Arc};
use std::collections::BTreeSet;
use std::f32::consts::PI;
use std::f32::consts::TAU;

use glam::IVec3;
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, types::Difficulty};
use uuid::Uuid;
use steel_math::trig::cos;
use steel_math::trig::sin;
use steel_protocol::packets::game::CTakeItemEntity;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::vanilla_blocks::SNOW;
use steel_registry::vanilla_criteria_triggers::HERO_OF_THE_VILLAGE;
use steel_registry::vanilla_game_rules::RAIDS;
use steel_registry::vanilla_items::WHITE_BANNER;
use steel_utils::random::legacy_random::LegacyRandom;

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
use crate::entity::Entity::is_alive;
use crate::entity::LivingEntity::is_alive;
use crate::entity::PathfinderMob;
use crate::entity::RemovalReason::Discarded;
use crate::entity::SharedEntity;
use crate::entity::entities::objects::items::ItemEntity;
use crate::inventory::equipment::EquipmentSlot::Head;
use crate::inventory::equipment::EquipmentSlot::MainHand;
use crate::player::advancement::predicate::TriggerContext;
use crate::world::SignalQueryContext;

/// Maximum zero-based effect amplifier; the raid's internal level is amplifier + 1.
pub const MAX_OMEN_LEVEL: i32 = 4;
const VILLAGE_SCAN_RADIUS: i32 = 64;
const RAID_TIMEOUT_TICKS: i32 = 48_000;
const PRE_WAVE_TICKS: i32 = 300;

/// Lifecycle state, persisted so a restart cannot turn a lost raid into a victory.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaidStatus {
    /// Waiting for or fighting a wave.
    #[default]
    Ongoing,
    /// All waves defeated and the post-raid delay completed.
    Victory,
    /// The village was destroyed after the encounter started.
    Loss,
}

/// One raid, including the UUIDs of every member of its current wave.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Raid {
    /// Stable vanilla RaidId used by saved raiders; zero marks legacy saves.
    pub id: i32,
    /// Village center used for membership and spawn searches.
    pub center: IVec3,
    /// Persistent wire identity of the raid boss bar.
    pub uuid: Uuid,
    /// Initial total health of the current wave, for boss-bar progress.
    pub total_health: i32,
    /// Captain UUID for the most recently spawned wave.
    pub leader: Option<Uuid>,
    /// Captains are scoped to their wave, including loaded older members.
    #[serde(with = "wave_leaders_save")]
    pub wave_leaders: BTreeMap<i32, Uuid>,
    /// Persisted wave identity must not change when a newer group spawns.
    pub member_waves: BTreeMap<Uuid, i32>,
    /// Zero-based omen amplifier, retained for compatibility with older saves.
    pub omen_level: i32,
    /// Loaded active ticks counted against the global timeout.
    pub ticks_active: i32,
    /// Number of spawned groups with no remaining members.
    pub waves_cleared: i32,
    /// Number of groups actually inserted into the world.
    pub waves_spawned: i32,
    /// Base group count selected from the starting difficulty.
    pub num_groups: i32,
    /// Ticks remaining before the next wave.
    pub cooldown: i32,
    /// Transient vanilla wave-position cache; rebuilt after loading a save.
    #[serde(skip)]
    pub wave_spawn_pos: Option<IVec3>,
    /// Runtime Raider.ticksOutsideRaid counters; vanilla does not persist them.
    #[serde(skip)]
    pub outside_raid_checks: BTreeMap<Uuid, u8>,
    /// Empty final-wave ticks before victory is declared.
    pub post_raid_ticks: i32,
    /// Ticks since entering victory or loss.
    pub celebration_ticks: i32,
    /// Current persistent lifecycle state.
    pub status: RaidStatus,
    /// UUIDs whose deaths or departures gate wave progression.
    pub raiders: Vec<Uuid>,
    /// Unique credited player killers eligible for the victory reward.
    pub heroes: Vec<Uuid>,
}

impl Default for Raid {
    fn default() -> Self {
        Self {
            id: 0,
            center: IVec3::ZERO,
            uuid: Uuid::new_v4(),
            total_health: 0,
            leader: None,
            wave_leaders: BTreeMap::new(),
            member_waves: BTreeMap::new(),
            omen_level: 0,
            ticks_active: 0,
            waves_cleared: 0,
            waves_spawned: 0,
            num_groups: 5,
            cooldown: PRE_WAVE_TICKS,
            wave_spawn_pos: None,
            outside_raid_checks: BTreeMap::new(),
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

    /// Whether the post-raid delay has completed with every wave defeated.
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

pub(crate) fn is_raider_type(entity_type: EntityTypeRef) -> bool {
    entity_type == &vanilla_entities::PILLAGER
        || entity_type == &vanilla_entities::VINDICATOR
        || entity_type == &vanilla_entities::EVOKER
        || entity_type == &vanilla_entities::WITCH
        || entity_type == &vanilla_entities::RAVAGER
        || entity_type == &vanilla_entities::ILLUSIONER
}

// TOML table keys are strings; encode integer wave keys as pairs instead.
mod wave_leaders_save {
    use super::*;
    pub fn serialize<S: serde::Serializer>(
        value: &BTreeMap<i32, Uuid>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value
            .iter()
            .map(|(wave, uuid)| (*wave, *uuid))
            .collect::<Vec<_>>()
            .serialize(serializer)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<i32, Uuid>, D::Error> {
        Ok(Vec::<(i32, Uuid)>::deserialize(deserializer)?
            .into_iter()
            .collect())
    }
}

/// TOML documents require a table root, including when the raid list is empty.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RaidSavedData {
    #[serde(default)]
    pub raids: Vec<Raid>,
    #[serde(default)]
    pub next_id: i32,
}

/// Raids currently alive per world.
#[derive(Debug, Default)]
pub struct RaidManager {
    raids: SyncMutex<Vec<Raid>>,
    viewers: SyncMutex<BTreeMap<Uuid, Vec<Uuid>>>,
    next_id: SyncMutex<i32>,
}

impl RaidManager {
    /// Creates an empty manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a manager over raids loaded from disk.
    #[must_use]
    pub fn from_raids(mut raids: Vec<Raid>) -> Self {
        let mut used: BTreeSet<i32> = raids
            .iter()
            .map(|raid| raid.id)
            .filter(|id| *id > 0)
            .collect();
        let mut seen = BTreeSet::new();
        let mut next = 1;
        for raid in &mut raids {
            for member in &raid.raiders {
                raid.member_waves
                    .entry(*member)
                    .or_insert(raid.waves_spawned);
            }
            if let Some(leader) = raid.leader {
                raid.wave_leaders
                    .entry(raid.waves_spawned)
                    .or_insert(leader);
            }
            if raid.id <= 0 || !seen.insert(raid.id) {
                while used.contains(&next) {
                    next += 1;
                }
                raid.id = next;
                used.insert(next);
                seen.insert(next);
            }
        }
        let next = used.last().copied().unwrap_or(0).saturating_add(1);
        Self {
            raids: SyncMutex::new(raids),
            viewers: SyncMutex::new(BTreeMap::new()),
            next_id: SyncMutex::new(next),
        }
    }

    pub(crate) fn from_saved_data(data: RaidSavedData) -> Self {
        let manager = Self::from_raids(data.raids);
        let next = (*manager.next_id.lock()).max(data.next_id);
        *manager.next_id.lock() = next;
        manager
    }

    pub(crate) fn to_saved_data(&self) -> RaidSavedData {
        RaidSavedData {
            raids: self.to_persistent(),
            next_id: *self.next_id.lock(),
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
        let mut next_id = self.next_id.lock();
        let id = (*next_id).max(1);
        *next_id = id.checked_add(1).expect("raid id space exhausted");
        let raid = Raid {
            id,
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

    /// Whether an ongoing encounter owns this raider, including unloaded members.
    pub fn contains_raider(&self, uuid: Uuid) -> bool {
        self.raids
            .lock()
            .iter()
            .any(|raid| raid.status == RaidStatus::Ongoing && raid.raiders.contains(&uuid))
    }

    /// Vanilla Raider.aiStep: eligible nearby mobs join once, without respawning
    /// or replacing their equipment. The closest raid within 96 blocks owns them.
    pub fn try_join_raider(&self, entity: &SharedEntity) -> bool {
        let Some(mob) = entity.as_mob() else {
            return false;
        };
        if !is_raider_type(entity.entity_type())
            || !entity.is_alive()
            || !mob.can_join_raid()
            || mob.no_action_time() > 2_400
        {
            return false;
        }
        let mut raids = self.raids.lock();
        let uuid = entity.uuid();
        if raids.iter().any(|raid| raid.raiders.contains(&uuid)) {
            return false;
        }
        let pos = entity.block_position().0;
        let Some(raid) = raids
            .iter_mut()
            .filter(|raid| {
                raid.status == RaidStatus::Ongoing && within_village_radius(raid.center, pos)
            })
            .min_by_key(|raid| (raid.center.as_i64vec3() - pos.as_i64vec3()).length_squared())
        else {
            return false;
        };
        raid.raiders.push(uuid);
        raid.member_waves.insert(uuid, raid.waves_spawned);
        raid.total_health += mob.get_health().max(0.0).ceil() as i32;
        true
    }

    /// Celebration is a loss-only AI state, not victory animation on any raid.
    pub fn is_loss_member(&self, uuid: Uuid) -> bool {
        self.raids
            .lock()
            .iter()
            .any(|raid| raid.status == RaidStatus::Loss && raid.raiders.contains(&uuid))
    }

    /// Vanilla raider save fields, including legacy worlds whose entities did
    /// not yet carry RaidId. Membership already in the manager is authoritative.
    pub fn member_save_data(&self, uuid: Uuid) -> Option<(i32, i32, bool)> {
        self.raids
            .lock()
            .iter()
            .find(|raid| raid.raiders.contains(&uuid))
            .map(|raid| {
                let wave = *raid.member_waves.get(&uuid).unwrap_or(&raid.waves_spawned);
                (
                    raid.id,
                    wave,
                    raid.wave_leaders.get(&wave) == Some(&uuid) || raid.leader == Some(uuid),
                )
            })
    }

    /// Reconnects loaded entities by RaidId, never by proximity. Repeated loads
    /// must not add health twice or turn a saved captain into a second leader.
    pub fn restore_raider(
        &self,
        entity: &SharedEntity,
        id: i32,
        wave: i32,
        leader: bool,
    ) {
        if !is_raider_type(entity.entity_type()) || !entity.is_alive() {
            return;
        }
        let mut raids = self.raids.lock();
        let Some(raid) = raids
            .iter_mut()
            .find(|raid| raid.id == id && raid.status == RaidStatus::Ongoing)
        else {
            return;
        };
        if wave < 0 || wave > raid.waves_spawned {
            return;
        }
        if !raid.raiders.contains(&entity.uuid()) {
            raid.raiders.push(entity.uuid());
        }
        raid.member_waves.insert(entity.uuid(), wave);
        if leader {
            raid.wave_leaders.entry(wave).or_insert(entity.uuid());
        }
        let is_leader = raid.wave_leaders.get(&wave) == Some(&entity.uuid());
        if wave == raid.waves_spawned && is_leader {
            raid.leader = Some(entity.uuid());
        }
        drop(raids);
        if is_leader && let Some(mob) = entity.as_mob() {
            let head = Head;
            mob.with_equipment_slot_mut(head, &mut |item| *item = ominous_banner());
            mob.set_equipment_drop_chance(head, 2.0);
        }
    }

    /// Shared eligibility for the MOVE-controlled captain-banner goal.
    pub(crate) fn can_pick_up_captain_banner(
        &self,
        mob: &dyn PathfinderMob,
    ) -> bool {
        use crate::inventory::equipment::EquipmentSlot;
        use steel_registry::item_stack::ItemStack;
        if !is_alive(mob)
            || !(mob.entity_type() == &vanilla_entities::PILLAGER
                || mob.entity_type() == &vanilla_entities::VINDICATOR
                || mob.entity_type() == &vanilla_entities::EVOKER
                || mob.entity_type() == &vanilla_entities::ILLUSIONER)
        {
            return false;
        }
        if ItemStack::matches(
            &mob.equipment_in_slot(EquipmentSlot::Head),
            &ominous_banner(),
        ) {
            return false;
        }
        self.raids.lock().iter().any(|raid| {
            let wave = *raid
                .member_waves
                .get(&mob.uuid())
                .unwrap_or(&raid.waves_spawned);
            raid.status == RaidStatus::Ongoing
                && raid.raiders.contains(&mob.uuid())
                && !raid.wave_leaders.contains_key(&wave)
                && !(wave == raid.waves_spawned && raid.leader.is_some())
        })
    }

    /// Atomically claim the pursued banner; competing goals cannot both promote.
    pub(crate) fn claim_captain_banner(
        &self,
        entity: &SharedEntity,
        item: &ItemEntity,
        world: &Arc<World>,
    ) {
        use crate::inventory::equipment::EquipmentSlot;
        use steel_registry::item_stack::ItemStack;
        let Some(mob) = entity.as_pathfinder_mob() else {
            return;
        };
        if !self.can_pick_up_captain_banner(mob)
            || item.has_pickup_delay()
            || !ItemStack::matches(&item.get_item(), &ominous_banner())
            || entity.position().distance_squared(item.position()) >= 1.414_f64.powi(2)
        {
            return;
        }
        {
            let mut raids = self.raids.lock();
            let Some(raid) = raids.iter_mut().find(|raid| {
                raid.status == RaidStatus::Ongoing && raid.raiders.contains(&entity.uuid())
            }) else {
                return;
            };
            let wave = *raid
                .member_waves
                .get(&entity.uuid())
                .unwrap_or(&raid.waves_spawned);
            if raid.wave_leaders.contains_key(&wave) || item.is_removed() {
                return;
            }
            raid.wave_leaders.insert(wave, entity.uuid());
            if wave == raid.waves_spawned {
                raid.leader = Some(entity.uuid());
            }
            item.set_removed(Discarded);
        }
        let old = mob.equipment_in_slot(EquipmentSlot::Head);
        if !old.is_empty()
            && (rand::random::<f32>() - 0.1).max(0.0)
                < mob.equipment_drop_chance(EquipmentSlot::Head)
        {
            mob.spawn_at_location(old, 0.0);
        }
        mob.with_equipment_slot_mut(EquipmentSlot::Head, &mut |head| *head = ominous_banner());
        mob.set_equipment_drop_chance(EquipmentSlot::Head, 2.0);
        world.broadcast_to_nearby(
            steel_utils::ChunkPos::from_entity_pos(item.position()),
            CTakeItemEntity::new(item.id(), entity.id(), 1),
            None,
        );
    }

    /// Removal with no death (discard or dimension transfer) must not leave an
    /// unreachable UUID holding the wave open, or award hero credit.
    pub fn raider_left_world(&self, uuid: Uuid, health: f32) {
        for raid in self.raids.lock().iter_mut() {
            if let Some(index) = raid.raiders.iter().position(|member| *member == uuid) {
                raid.raiders.swap_remove(index);
                raid.member_waves.remove(&uuid);
                raid.wave_leaders.retain(|_, leader| *leader != uuid);
                raid.outside_raid_checks.remove(&uuid);
                raid.total_health = (raid.total_health - health.max(0.0).ceil() as i32).max(0);
                if raid.leader == Some(uuid) {
                    raid.leader = None;
                }
            }
        }
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
                raid.member_waves.remove(&raider);
                raid.wave_leaders.retain(|_, leader| *leader != raider);
                if raid.leader == Some(raider) {
                    raid.leader = None;
                }
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
            if !world.get_game_rule(&RAIDS)
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
            if raid.ticks_active % 20 == 0 {
                update_raiders(world, raid);
            }
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
                if raid.waves_spawned < raid.wave_count() {
                    // Vanilla resets the inter-wave timer and returns; its
                    // first decrement happens on the following tick.
                    return true;
                }
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
                update_wave_spawn_cache(world, raid);
                raid.cooldown -= 1;
                if raid.cooldown > 0 {
                    return true;
                }
            }
            let cached = raid.wave_spawn_pos.take().map(BlockPos);
            // Vanilla makes six independent 20-candidate attempts before stopping.
            let position = cached.or_else(|| {
                (0..6).find_map(|_| find_spawn_pos(world, raid.center, raid.cooldown, 20))
            });
            if let Some(pos) = position {
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
                || !is_alive(player.as_ref())
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
                &HERO_OF_THE_VILLAGE,
                &TriggerContext::at(
                    &player.get_world(),
                    player.position(),
                ),
            );
            true
        });
    }
}

fn wave_spawn_column(
    center: IVec3,
    cooldown: i32,
    start_angle: f32,
    attempt: i32,
    jitter_x: i32,
    jitter_z: i32,
) -> (i32, i32) {
    let how_far = 0.22_f32 * (cooldown / 20) as f32 - 0.24_f32;
    let angle = start_angle + PI * attempt as f32 / 8.0;
    let jitter_scale = how_far.floor() as i32;
    (
        center.x
            + (cos(angle) * 32.0 * how_far).floor() as i32
            + jitter_x * jitter_scale,
        center.z
            + (sin(angle) * 32.0 * how_far).floor() as i32
            + jitter_z * jitter_scale,
    )
}

fn update_wave_spawn_cache(world: &Arc<World>, raid: &mut Raid) {
    let retry =
        raid.wave_spawn_pos.map_or(raid.cooldown % 5 == 0, |pos| {
            !world.chunk_map.is_entity_ticking_full_chunk_loaded(
                steel_utils::ChunkPos::from_block_pos(BlockPos(pos)),
            )
        });
    if retry {
        raid.wave_spawn_pos = find_spawn_pos(world, raid.center, raid.cooldown, 8).map(|pos| pos.0);
    }
}

fn spawn_region_loaded(world: &Arc<World>, pos: BlockPos) -> bool {
    for x in (pos.x() - 10).div_euclid(16)..=(pos.x() + 10).div_euclid(16) {
        for z in (pos.z() - 10).div_euclid(16)..=(pos.z() + 10).div_euclid(16) {
            if world
                .chunk_map
                .with_full_chunk(steel_utils::ChunkPos::new(x, z), |_| ())
                .is_none()
            {
                return false;
            }
        }
    }
    world
        .chunk_map
        .is_entity_ticking_full_chunk_loaded(steel_utils::ChunkPos::from_block_pos(pos))
}

fn valid_empty_raid_spawn_block(world: &Arc<World>, pos: BlockPos) -> bool {
    use crate::behavior::{BLOCK_BEHAVIORS, BlockCollisionContext};
    use crate::entity::ai::walk::WalkPathEvaluator;
    use steel_registry::blocks::shapes::is_shape_full_block;
    use steel_registry::{vanilla_blocks, vanilla_fluids};
    let state = world.get_block_state(pos);
    let block = state.get_block();
    let behavior = BLOCK_BEHAVIORS.get_behavior(block);
    let dangerous = WalkPathEvaluator::is_burning_block(state)
        || block == &vanilla_blocks::WITHER_ROSE
        || block == &vanilla_blocks::SWEET_BERRY_BUSH
        || block == &vanilla_blocks::CACTUS
        || block == &vanilla_blocks::POWDER_SNOW;
    !is_shape_full_block(behavior.get_collision_shape(
        state,
        world.as_ref(),
        pos,
        BlockCollisionContext::empty(),
    )) && !behavior.is_signal_source(state, SignalQueryContext::DEFAULT)
        && state.get_fluid_state().fluid_id == &vanilla_fluids::EMPTY
        && !block.has_tag(&steel_utils::Identifier::vanilla_static(
            "prevent_mob_spawning_inside",
        ))
        && !dangerous
}

fn valid_raid_spawn_surface(world: &Arc<World>, pos: BlockPos) -> bool {
    use crate::behavior::BLOCK_BEHAVIORS;
    let below = pos.below();
    let state = world.get_block_state(below);
    let on_ground = world.is_block_within_world_border(pos)
        && BLOCK_BEHAVIORS
            .get_behavior(state.get_block())
            .is_valid_spawn(state, world.as_ref(), below, &vanilla_entities::RAVAGER)
        && valid_empty_raid_spawn_block(world, pos)
        && valid_empty_raid_spawn_block(world, pos.above());
    on_ground
        || (state.get_block() == &SNOW
            && world.get_block_state(pos).is_air())
}

/// Vanilla 26.3 cooldown-dependent surface search, with loaded simulation gating.
fn find_spawn_pos(
    world: &Arc<World>,
    center: IVec3,
    cooldown: i32,
    max_tries: i32,
) -> Option<BlockPos> {
    let start_angle = rand::random::<f32>() * TAU;
    for attempt in 0..max_tries {
        let (x, z) = wave_spawn_column(
            center,
            cooldown,
            start_angle,
            attempt,
            rand::random_range(0..3),
            rand::random_range(0..3),
        );
        let Some(y) = world.as_ref().height_at(HeightmapType::WorldSurface, x, z) else {
            continue;
        };
        if (y - center.y).abs() > 96 {
            continue;
        }
        let pos = BlockPos::new(x, y, z);
        if cooldown / 20 > 7 && is_village(world, pos.0) {
            continue;
        }
        if spawn_region_loaded(world, pos) && valid_raid_spawn_surface(world, pos) {
            return Some(pos);
        }
    }
    None
}

/// Shared vanilla captain banner, including all eight ordered pattern layers.
pub(crate) fn ominous_banner() -> ItemStack {
    use steel_registry::data_components::vanilla_components::{
        BANNER_PATTERNS, ITEM_NAME, RARITY, Rarity,
    };
    use steel_registry::data_components::{BannerPatternLayer, BannerPatternLayers};
    use steel_registry::{DyeColor, RegistryHolder, vanilla_banner_patterns as patterns};
    use text_components::{TextComponent, translation::TranslatedMessage};
    let layers = [
        (&patterns::RHOMBUS, DyeColor::Cyan),
        (&patterns::STRIPE_BOTTOM, DyeColor::LightGray),
        (&patterns::STRIPE_CENTER, DyeColor::Gray),
        (&patterns::BORDER, DyeColor::LightGray),
        (&patterns::STRIPE_MIDDLE, DyeColor::Black),
        (&patterns::HALF_HORIZONTAL, DyeColor::LightGray),
        (&patterns::CIRCLE, DyeColor::LightGray),
        (&patterns::BORDER, DyeColor::Black),
    ]
    .into_iter()
    .map(|(pattern, color)| BannerPatternLayer::new(RegistryHolder::reference(pattern), color))
    .collect();
    let mut banner =
        ItemStack::new(&WHITE_BANNER);
    banner.set(BANNER_PATTERNS, BannerPatternLayers::new(layers));
    banner.set(
        ITEM_NAME,
        TextComponent::translated(TranslatedMessage {
            key: "block.minecraft.ominous_banner".into(),
            fallback: None,
            args: None,
        }),
    );
    banner.set(RARITY, Rarity::Uncommon);
    banner
}

// Vanilla Pillager/Vindicator.applyRaidBuffs selects extracted providers by wave.
fn raid_weapon(
    kind: usize,
    wave: i32,
    omen: i32,
    roll: f32,
) -> Option<ItemStack> {
    use steel_registry::data_components::vanilla_components::{ENCHANTMENTS, ItemEnchantments};
    use steel_registry::{vanilla_enchantment_providers as providers, vanilla_items};
    let odds = match omen {
        1 => 0.1,
        2 => 0.25,
        3 => 0.5,
        4 => 0.75,
        _ => 0.0,
    };
    let (item, provider) = match kind {
        0 => (
            &vanilla_items::IRON_AXE,
            Some(if wave > 5 {
                &*providers::RAID_VINDICATOR_POST_WAVE_5
            } else {
                &*providers::RAID_VINDICATOR
            }),
        ),
        2 => (
            &vanilla_items::CROSSBOW,
            if wave > 5 {
                Some(&*providers::RAID_PILLAGER_POST_WAVE_5)
            } else if wave > 3 {
                Some(&*providers::RAID_PILLAGER_POST_WAVE_3)
            } else {
                None
            },
        ),
        _ => return None,
    };
    let mut stack = ItemStack::new(item);
    if roll <= odds
        && let Some(provider) = provider
    {
        let mut rng = LegacyRandom::from_seed(rand::random());
        let mut enchantments = ItemEnchantments::empty();
        for (enchantment, level) in provider.sample(0.0, &mut rng) {
            enchantments.set(enchantment.key.clone(), level as u32);
        }
        stack.set(ENCHANTMENTS, enchantments);
    }
    Some(stack)
}

fn horn_position(listener: glam::DVec3, origin: glam::DVec3) -> glam::DVec3 {
    let delta = glam::DVec3::new(origin.x - listener.x, 0.0, origin.z - listener.z);
    if delta.length_squared() == 0.0 {
        return listener;
    }
    listener + delta.normalize() * 13.0
}

fn play_raid_horn(world: &Arc<World>, raid: &Raid, pos: BlockPos) {
    use steel_protocol::packets::game::{CSound, SoundSource};
    let Some(sound) = steel_registry::REGISTRY
        .sound_events
        .by_key(&steel_utils::Identifier::vanilla_static("event.raid.horn"))
    else {
        return;
    };
    let origin = pos.0.as_dvec3() + glam::DVec3::splat(0.5);
    let seed = rand::random();
    world.players.iter_players(|_, player| {
        let position = player.position();
        let dx = origin.x - position.x;
        let dz = origin.z - position.z;
        if dx * dx + dz * dz <= 64.0 * 64.0
            || within_village_radius(raid.center, player.block_position().0)
        {
            player.send_packet(CSound::new(
                sound,
                SoundSource::Neutral,
                horn_position(position, origin),
                64.0,
                1.0,
                seed,
            ));
        }
        true
    });
}

fn spawn_wave(world: &Arc<World>, raid: &mut Raid, pos: BlockPos) {
    let wave = raid.waves_spawned + 1;
    raid.total_health = 0;
    raid.outside_raid_checks.clear();
    raid.leader = None;
    let bonus = wave > raid.num_groups;
    let index = if bonus { raid.num_groups } else { wave }.clamp(0, 7) as usize;
    let types = [
        &vanilla_entities::VINDICATOR,
        &vanilla_entities::EVOKER,
        &vanilla_entities::PILLAGER,
        &vanilla_entities::WITCH,
        &vanilla_entities::RAVAGER,
    ];
    for (kind, entity_type) in types.into_iter().enumerate() {
        let count = WAVE_COUNTS[kind][index] + bonus_spawns(kind, wave, world.difficulty(), bonus);
        for number in 0..count {
            let request = EntitySpawnRequest {
                entity_type,
                placement: EntitySpawnPlacement::Block {
                    pos,
                    try_move_down: false,
                    moved_up: false,
                },
                reason: EntitySpawnReason::Event,
                finalize_spawn: true,
                play_ambient_sound: false,
                item_stack: None,
                user_is_operator: false,
            };
            if let Ok(entity) = spawn_entity(world, request) {
                if let Some(mob) = entity.as_mob() {
                    mob.set_persistence_required();
                }
                raid.total_health += entity
                    .as_living_entity()
                    .map_or(0, |living| living.get_health().ceil() as i32);
                if let Some(living) = entity.as_living_entity() {
                    if let Some(weapon) = raid_weapon(kind, wave, raid.omen_level, rand::random())
                        && (kind == 0 || weapon.get(ENCHANTMENTS).is_some_and(|value| !value.levels.is_empty()))
                    {
                        living.with_equipment_slot_mut(MainHand,
                            &mut |item| *item = weapon.clone());
                    }
                    if raid.leader.is_none() && kind <= 2 {
                        living.with_equipment_slot_mut(
                            Head,
                            &mut |item| *item = ominous_banner(),
                        );
                        if let Some(mob) = entity.as_mob() {
                            mob.set_equipment_drop_chance(
                                Head,
                                2.0,
                            );
                        }
                        raid.leader = Some(entity.uuid());
                        raid.wave_leaders.insert(wave, entity.uuid());
                    }
                }
                raid.raiders.push(entity.uuid());
                raid.member_waves.insert(entity.uuid(), wave);
                if kind == 4 {
                    let rider = if wave == 5 {
                        Some(&vanilla_entities::PILLAGER)
                    } else if wave >= 7 {
                        Some(if number == 0 {
                            &vanilla_entities::EVOKER
                        } else {
                            &vanilla_entities::VINDICATOR
                        })
                    } else {
                        None
                    };
                    if let Some(entity_type) = rider
                        && let Ok(rider) = spawn_entity(
                            world,
                            EntitySpawnRequest {
                                entity_type,
                                ..request
                            },
                        )
                    {
                        if let Some(mob) = rider.as_mob() {
                            mob.set_persistence_required();
                        }
                        let rider_kind = if entity_type == &vanilla_entities::VINDICATOR {
                            0
                        } else if entity_type == &vanilla_entities::PILLAGER {
                            2
                        } else {
                            1
                        };
                        if let Some(living) = rider.as_living_entity()
                            && let Some(weapon) = raid_weapon(rider_kind, wave, raid.omen_level, rand::random())
                            && (rider_kind == 0 || weapon.get(ENCHANTMENTS).is_some_and(|value| !value.levels.is_empty()))
                        {
                            living.with_equipment_slot_mut(MainHand,
                                &mut |item| *item = weapon.clone());
                        }
                        rider.start_riding(&entity);
                        raid.total_health += rider
                            .as_living_entity()
                            .map_or(0, |living| living.get_health().ceil() as i32);
                        raid.raiders.push(rider.uuid());
                        raid.member_waves.insert(rider.uuid(), wave);
                    }
                }
            }
        }
    }
    // Failure to create mobs must never manufacture progress.
    if !raid.raiders.is_empty() {
        raid.waves_spawned = wave;
        play_raid_horn(world, raid, pos);
    }
}

/// Vanilla checks membership every second, not every simulation tick. Missing
/// UUIDs alone are deliberately not deaths: unloaded entities need reconnection.
fn update_raiders(world: &Arc<World>, raid: &mut Raid) {
    let mut departed_health = 0;
    raid.raiders.retain(|id| {
        let Some(entity) = world.get_entity_by_uuid(id) else {
            return true;
        };
        let delta = entity.block_position().0.as_i64vec3() - raid.center.as_i64vec3();
        let different_world = entity
            .level()
            .is_some_and(|level| !Arc::ptr_eq(&level, world));
        let outside_expired = if entity.tick_count() > 600 {
            let checks = raid.outside_raid_checks.entry(*id).or_default();
            if !is_village(world, entity.block_position().0)
                && entity
                    .as_living_entity()
                    .is_some_and(|living| living.no_action_time() > 2_400)
            {
                *checks = checks.saturating_add(1);
            }
            *checks >= 30
        } else {
            false
        };
        let departed = entity.is_removed()
            || different_world
            || delta.length_squared() >= 12_544
            || outside_expired;
        if departed {
            departed_health += entity
                .as_living_entity()
                .map_or(0, |living| living.get_health().max(0.0).ceil() as i32);
        }
        !departed
    });
    raid.outside_raid_checks
        .retain(|id, _| raid.raiders.contains(id));
    raid.total_health = (raid.total_health - departed_health).max(0);
    raid.wave_leaders
        .retain(|_, leader| raid.raiders.contains(leader));
    raid.member_waves
        .retain(|member, _| raid.raiders.contains(member));
    if raid
        .leader
        .is_some_and(|leader| !raid.raiders.contains(&leader))
    {
        raid.leader = None;
    }
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
    steel_registry::REGISTRY
        .poi_types
        .by_id(type_id)
        .is_some_and(|poi| {
            steel_registry::REGISTRY
                .poi_types
                .is_in_tag(poi, &steel_utils::Identifier::vanilla_static("village"))
        })
}

/// Whether this position is within one section of an occupied village POI.
#[must_use]
pub fn is_village(world: &Arc<World>, pos: IVec3) -> bool {
    is_village_world(world, BlockPos(pos))
}

/// Village distance is measured in sections, including the vertical axis.
#[must_use]
pub fn is_village_world(world: &World, pos: BlockPos) -> bool {
    world
        .poi_storage
        .lock()
        .get_in_horizontal_square(&village_poi, pos, 32, OccupationStatus::Occupied)
        .iter()
        .any(|(poi, _)| {
            let delta = (poi.0 >> 4_i32) - (pos.0 >> 4_i32);
            delta.abs().max_element() <= 1
        })
}

/// Vanilla averages occupied village POIs in a 64-block sphere to center a raid.
#[must_use]
pub fn village_center_world(world: &World, pos: BlockPos) -> Option<IVec3> {
    let matches = world.poi_storage.lock().get_in_horizontal_square(
        &village_poi,
        pos,
        VILLAGE_SCAN_RADIUS,
        OccupationStatus::Occupied,
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
    (count > 0).then(|| {
        IVec3::new(
            total.x.div_euclid(count) as i32,
            total.y.div_euclid(count) as i32,
            total.z.div_euclid(count) as i32,
        )
    })
}

/// Arc-world convenience wrapper for the averaged occupied village center.
#[must_use]
pub fn village_center(world: &Arc<World>, pos: IVec3) -> Option<IVec3> {
    village_center_world(world, BlockPos(pos))
}

#[cfg(test)]
mod persistence_tests {
    use super::*;
    use steel_utils::saved_data::{SavedDataManager, names};
use std::env::temp_dir;
use std::fs::remove_dir_all;
    #[tokio::test]
    async fn empty_and_active_raids_round_trip_through_real_disk_toml() {
        let directory =
            temp_dir().join(format!("steel-raid-save-{}", uuid::Uuid::new_v4()));
        let manager = SavedDataManager::new(Some(&directory));
        for raids in [
            Vec::new(),
            vec![Raid {
                center: IVec3::new(-5, 64, 8),
                omen_level: 3,
                ticks_active: 129,
                waves_cleared: 1,
                ..Raid::default()
            }],
        ] {
            let data = RaidSavedData {
                raids,
                ..RaidSavedData::default()
            };
            manager.save(names::RAIDS, &data).await.unwrap();
            let loaded: RaidSavedData = manager.load_or_default(names::RAIDS).await.unwrap();
            assert_eq!(loaded, data);
        }
        remove_dir_all(directory).unwrap();
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use crate::behavior::init_behaviors;
    use crate::test_support::{fresh_test_world_with_difficulty, insert_ready_full_chunk};
    use steel_utils::ChunkPos;
use steel_registry::vanilla_damage_types::GENERIC;
use crate::entity::damage::DamageSource;
use crate::inventory::equipment::EquipmentSlot::Head;

    #[test]
    fn difficulty_not_omen_selects_base_groups_and_omen_adds_only_one_bonus() {
        for (difficulty, expected) in [
            (Difficulty::Easy, 3),
            (Difficulty::Normal, 5),
            (Difficulty::Hard, 7),
        ] {
            for omen_level in 0..=4 {
                let raid = Raid {
                    num_groups: difficulty_groups(difficulty),
                    omen_level,
                    ..Raid::default()
                };
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
        let manager = RaidManager::from_raids(vec![Raid {
            raiders: vec![member, other],
            ..Raid::default()
        }]);
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
        let home = steel_registry::REGISTRY
            .poi_types
            .iter()
            .find(|(_, poi)| poi.key.path.as_ref() == "home")
            .unwrap()
            .0;
        world.poi_storage.lock().add(BlockPos(center), home, 1);
        assert!(world.poi_storage.lock().reserve_ticket(BlockPos(center)));
        assert!(is_village(world, center));
        let mut raid = Raid {
            center,
            ..Raid::default()
        };
        spawn_wave(world, &mut raid, BlockPos::new(2, 64, 2));
        // Normal difficulty independently adds 0..=1 pillagers AND 0..=1
        // vindicators, including on the first wave (Raid.getPotentialBonusSpawns).
        assert!((4..=6).contains(&raid.raiders.len()));
        let members_of_type = |entity_type| {
            raid.raiders
                .iter()
                .filter_map(|id| world.get_entity_by_uuid(id))
                .filter(|entity| entity.entity_type() == entity_type)
                .count()
        };
        assert!((4..=5).contains(&members_of_type(&vanilla_entities::PILLAGER)));
        assert!(members_of_type(&vanilla_entities::VINDICATOR) <= 1);
        let captain = world
            .get_entity_by_uuid(&raid.leader.expect("wave captain"))
            .unwrap();
        let banner = captain
            .as_living_entity()
            .unwrap()
            .living_base()
            .equipment()
            .lock()
            .get_ref(Head)
            .clone();
        assert_eq!(
            banner
                .get(BANNER_PATTERNS)
                .unwrap()
                .layers()
                .len(),
            8
        );
        let members = raid.raiders.clone();
        *world.raids.raids.lock() = vec![raid];
        for _ in 0..1_201 {
            world.raids.tick(world);
        }
        let ongoing = world.raids.raids();
        assert_eq!(ongoing[0].waves_spawned, 1);
        assert_eq!(ongoing[0].waves_cleared, 0);
        assert_eq!(ongoing[0].raiders.len(), members.len());
        for id in members {
            let entity = world.get_entity_by_uuid(&id).unwrap();
            let living = entity.as_living_entity().unwrap();
            living.set_health(0.0);
            living.die(&DamageSource::environment(
                &GENERIC,
            ));
        }
        world.raids.tick(world);
        assert!(world.raids.raids()[0].raiders.is_empty());
        assert_eq!(world.raids.raids()[0].waves_cleared, 1);
        assert!(!world.raids.raids()[0].is_victory());
    }

    #[test]
    fn missing_members_survive_serialization_without_false_clear() {
        let raid = Raid {
            waves_spawned: 2,
            waves_cleared: 1,
            raiders: vec![Uuid::new_v4()],
            ..Raid::default()
        };
        let document = toml::to_string(&RaidSavedData {
            raids: vec![raid.clone()],
            ..RaidSavedData::default()
        })
        .unwrap();
        let restored: RaidSavedData = toml::from_str(&document).unwrap();
        assert_eq!(restored.raids, vec![raid]);
        assert!(!restored.raids[0].is_victory());
    }
}

#[cfg(test)]
mod outcome_tests {
    use super::*;
    use crate::entity::next_entity_id;
    use crate::test_support::{TestPlayerBuilder, fresh_test_world, insert_ready_full_chunk};
    use steel_utils::ChunkPos;
use crate::behavior::init_behaviors;

    #[test]
    fn victory_waits_forty_ticks_rewards_only_credited_hero_then_expires() {
        steel_registry::init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("raid_victory");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let center = IVec3::new(0, 64, 0);
        let home = steel_registry::REGISTRY
            .poi_types
            .iter()
            .find(|(_, poi)| poi.key.path.as_ref() == "home")
            .unwrap()
            .0;
        world.poi_storage.lock().add(BlockPos(center), home, 1);
        assert!(world.poi_storage.lock().reserve_ticket(BlockPos(center)));
        let hero = TestPlayerBuilder::new(Arc::clone(world), "Hero", next_entity_id()).build();
        let bystander =
            TestPlayerBuilder::new(Arc::clone(world), "Bystander", next_entity_id()).build();
        world.players.insert(Arc::clone(&hero));
        world.players.insert(Arc::clone(&bystander));
        // Hard raid with omen III: seven normal groups and exactly one bonus.
        *world.raids.raids.lock() = vec![Raid {
            center,
            num_groups: 7,
            omen_level: 2,
            waves_spawned: 8,
            waves_cleared: 8,
            heroes: vec![hero.uuid()],
            ..Raid::default()
        }];
        for _ in 0..40 {
            world.raids.tick(world);
        }
        assert!(!world.raids.raids()[0].is_victory());
        assert!(!hero.has_mob_effect(vanilla_mob_effects::HERO_OF_THE_VILLAGE));
        world.raids.tick(world);
        assert!(world.raids.raids()[0].is_victory());
        let effect = hero
            .mob_effect(vanilla_mob_effects::HERO_OF_THE_VILLAGE)
            .unwrap();
        assert_eq!(effect.duration(), 48_000);
        assert_eq!(effect.amplifier(), 2);
        assert!(!bystander.has_mob_effect(vanilla_mob_effects::HERO_OF_THE_VILLAGE));
        for _ in 0..600 {
            world.raids.tick(world);
        }
        assert!(world.raids.raids().is_empty());
        assert!(world.raids.viewers.lock().is_empty());
    }

    #[test]
    fn destroyed_village_is_loss_and_timeout_is_global_not_per_wave() {
        let fixture = fresh_test_world("raid_loss_timeout");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        *world.raids.raids.lock() = vec![Raid {
            waves_spawned: 1,
            ..Raid::default()
        }];
        world.raids.tick(world);
        assert_eq!(world.raids.raids()[0].status, RaidStatus::Loss);
        assert!(!world.raids.raids()[0].is_victory());
        *world.raids.raids.lock() = vec![Raid {
            ticks_active: 47_999,
            ..Raid::default()
        }];
        world.raids.tick(world);
        assert!(world.raids.raids().is_empty());
    }
}

#[cfg(test)]
mod complete_wave_tests {
    use super::*;
    use crate::test_support::{fresh_test_world_with_difficulty, insert_ready_full_chunk};
    use steel_registry::vanilla_blocks;
    use steel_utils::{ChunkPos, types::UpdateFlags};
use steel_registry::vanilla_damage_types::GENERIC;
use crate::behavior::init_behaviors;
use crate::chunk::chunk_holder::TickingReadiness::EntityTicking;
use crate::chunk::chunk_ticket_manager::ChunkTicketLevel;
use crate::entity::damage::DamageSource;

    #[test]
    fn all_difficulties_and_bonus_waves_require_real_member_deaths() {
        steel_registry::init_vanilla_registry();
        init_behaviors();
        for (difficulty, omen, waves) in [
            (Difficulty::Easy, 0, 3),
            (Difficulty::Easy, 1, 4),
            (Difficulty::Normal, 0, 5),
            (Difficulty::Normal, 1, 6),
            (Difficulty::Hard, 0, 7),
            (Difficulty::Hard, 4, 8),
        ] {
            run_encounter(difficulty, omen, waves);
        }
    }

    fn run_encounter(difficulty: Difficulty, omen: i32, waves: i32) {
        let key = format!("raid_{difficulty:?}_{omen}").to_lowercase();
        let fixture = fresh_test_world_with_difficulty(&key, difficulty);
        let world = &fixture.world;
        for x in -1..=1 {
            for z in -1..=1 {
                let holder = insert_ready_full_chunk(world, ChunkPos::new(x, z));
                holder.set_simulation_level(Some(
                    ChunkTicketLevel::ENTITY_TICKING_CHUNK,
                ));
                let _ = holder.transition_ticking_readiness(
                    EntityTicking,
                );
            }
        }
        for x in -16..32 {
            for z in -16..32 {
                world.set_block(
                    BlockPos::new(x, 63, z),
                    vanilla_blocks::STONE.default_state(),
                    UpdateFlags::UPDATE_NONE,
                );
            }
        }
        let home = steel_registry::REGISTRY
            .poi_types
            .iter()
            .find(|(_, poi)| poi.key.path.as_ref() == "home")
            .unwrap()
            .0;
        let center = IVec3::new(0, 64, 0);
        world.poi_storage.lock().add(BlockPos(center), home, 1);
        assert!(world.poi_storage.lock().reserve_ticket(BlockPos(center)));
        world.raids.raise_omen(center, omen);
        for wave in 1..=waves {
            let wait = if wave == 1 { 300 } else { 301 };
            for _ in 0..wait - 1 {
                world.raids.tick(world);
            }
            assert_eq!(
                world.raids.raids()[0].waves_spawned,
                wave - 1,
                "wave must not spawn early"
            );
            world.raids.tick(world);
            let raid = world.raids.raids()[0].clone();
            assert_eq!(raid.waves_spawned, wave);
            assert_eq!(raid.waves_cleared, wave - 1);
            assert!(!raid.raiders.is_empty());
            assert!(!raid.is_victory());
            if wave == 5 || wave >= 7 {
                for id in &raid.raiders {
                    let entity = world.get_entity_by_uuid(id).unwrap();
                    if entity.entity_type() == &vanilla_entities::RAVAGER {
                        assert_eq!(entity.passengers().len(), 1, "ravager rider in wave {wave}");
                        assert!(raid.raiders.contains(&entity.passengers()[0].uuid()));
                    }
                }
            }
            for id in raid.raiders {
                let entity = world.get_entity_by_uuid(&id).unwrap();
                let living = entity.as_living_entity().unwrap();
                living.set_health(0.0);
                living.die(&DamageSource::environment(
                    &GENERIC,
                ));
            }
        }
        for _ in 0..41 {
            world.raids.tick(world);
        }
        let raid = &world.raids.raids()[0];
        assert_eq!(raid.waves_cleared, waves);
        assert!(raid.is_victory());
        assert!(
            raid.heroes.is_empty(),
            "environmental kills do not fabricate hero credit"
        );
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
        let meeting = steel_registry::REGISTRY
            .poi_types
            .iter()
            .find(|(_, poi)| poi.key.path.as_ref() == "meeting")
            .unwrap()
            .0;
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
        let home = steel_registry::REGISTRY
            .poi_types
            .iter()
            .find(|(_, poi)| poi.key.path.as_ref() == "home")
            .unwrap()
            .0;
        let center = BlockPos::new(0, 64, 0);
        world.poi_storage.lock().add(center, home, 1);
        assert!(world.poi_storage.lock().reserve_ticket(center));
        let member = Uuid::new_v4();
        *world.raids.raids.lock() = vec![Raid {
            center: center.0,
            waves_spawned: 1,
            raiders: vec![member],
            ..Raid::default()
        }];
        for _ in 0..1_201 {
            world.raids.tick(world);
        }
        let raid = &world.raids.raids()[0];
        assert_eq!(raid.raiders, vec![member]);
        assert_eq!(raid.waves_cleared, 0);
        assert!(!raid.is_victory());
    }
}

#[cfg(test)]
mod raid_enchantment_tests {
    use super::*;
    use steel_registry::{data_components::vanilla_components::ENCHANTMENTS, vanilla_enchantments};

    #[test]
    fn extracted_raid_weapon_providers_follow_wave_thresholds_and_omen_probability() {
        steel_registry::init_vanilla_registry();
        for (kind, wave, level) in [
            (0, 1, 1),
            (0, 5, 1),
            (0, 6, 2),
            (2, 3, 0),
            (2, 4, 1),
            (2, 5, 1),
            (2, 6, 2),
        ] {
            let weapon = raid_weapon(kind, wave, 4, 0.1).unwrap();
            let enchantment = if kind == 0 {
                &vanilla_enchantments::SHARPNESS
            } else {
                &vanilla_enchantments::QUICK_CHARGE
            };
            assert_eq!(
                weapon
                    .get(ENCHANTMENTS)
                    .map_or(0, |value| value.get_level(&enchantment.key)),
                level
            );
            let plain = raid_weapon(kind, wave, 0, 0.1).unwrap();
            assert_eq!(
                plain
                    .get(ENCHANTMENTS)
                    .map_or(0, |value| value.get_level(&enchantment.key)),
                0
            );
        }
        for (omen, odds) in [(1, 0.1_f32), (2, 0.25), (3, 0.5), (4, 0.75)] {
            assert_eq!(
                raid_weapon(0, 6, omen, odds)
                    .unwrap()
                    .get(ENCHANTMENTS)
                    .unwrap()
                    .get_level(&vanilla_enchantments::SHARPNESS.key),
                2
            );
            assert_eq!(
                raid_weapon(0, 6, omen, odds + 0.01)
                    .unwrap()
                    .get(ENCHANTMENTS)
                    .map_or(0, |value| value
                        .get_level(&vanilla_enchantments::SHARPNESS.key)),
                0
            );
        }
    }
}

#[cfg(test)]
mod horn_tests {
    use super::*;

    #[test]
    fn horn_projects_toward_wave_thirteen_blocks_at_listener_height() {
        steel_registry::init_vanilla_registry();
        assert!(
            steel_registry::REGISTRY
                .sound_events
                .by_key(&steel_utils::Identifier::vanilla_static("event.raid.horn"))
                .is_some()
        );
        let listener = glam::DVec3::new(1.0, 64.0, 2.0);
        let projected = horn_position(listener, glam::DVec3::new(31.0, 90.0, 42.0));
        assert!((projected.distance(listener) - 13.0).abs() < 1.0e-10);
        assert_eq!(projected.y, 64.0);
        assert_eq!(horn_position(listener, listener), listener);
    }
}

#[cfg(test)]
mod spawn_search_tests {
    use super::*;
    use crate::chunk::chunk_holder::TickingReadiness;
    use crate::chunk::chunk_ticket_manager::ChunkTicketLevel;
    use crate::test_support::{fresh_test_world, insert_ready_full_chunk};
    use steel_registry::vanilla_blocks;
    use steel_utils::{ChunkPos, types::UpdateFlags};
use crate::behavior::init_behaviors;

    #[test]
    fn search_radius_uses_whole_countdown_seconds_and_signed_floor_jitter() {
        assert_eq!(wave_spawn_column(IVec3::ZERO, 300, 0.0, 0, 0, 0), (97, 0));
        assert_eq!(wave_spawn_column(IVec3::ZERO, 300, 0.0, 0, 2, 1), (103, 3));
        assert_eq!(wave_spawn_column(IVec3::ZERO, 299, 0.0, 0, 0, 0), (90, 0));
        assert_eq!(wave_spawn_column(IVec3::ZERO, 0, 0.0, 0, 2, 2), (-10, -2));
        assert_eq!(wave_spawn_column(IVec3::ZERO, 19, 0.0, 0, 2, 2), (-10, -2));
    }

    #[test]
    fn spawn_requires_loaded_ten_block_margin_and_entity_not_only_block_ticks() {
        let fixture = fresh_test_world("raid_spawn_simulation");
        let world = &fixture.world;
        for x in -1..=1 {
            for z in -1..=1 {
                insert_ready_full_chunk(world, ChunkPos::new(x, z));
            }
        }
        let pos = BlockPos::new(0, 64, 0);
        assert!(!spawn_region_loaded(world, pos));
        let holder = world
            .chunk_map
            .chunks
            .read_sync(&ChunkPos::new(0, 0), |_, holder| Arc::clone(holder))
            .unwrap();
        holder.set_simulation_level(Some(ChunkTicketLevel::ENTITY_TICKING_CHUNK));
        let _ = holder.transition_ticking_readiness(TickingReadiness::EntityTicking);
        assert!(spawn_region_loaded(world, pos));
        assert!(!spawn_region_loaded(world, BlockPos::new(30, 64, 30)));
    }

    #[test]
    fn cached_position_survives_countdown_but_not_loss_of_entity_simulation() {
        let fixture = fresh_test_world("raid_spawn_cache");
        let world = &fixture.world;
        let holder = insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        holder.set_simulation_level(Some(ChunkTicketLevel::ENTITY_TICKING_CHUNK));
        let _ = holder.transition_ticking_readiness(TickingReadiness::EntityTicking);
        let cached = IVec3::new(0, 64, 0);
        let mut raid = Raid {
            cooldown: 121,
            wave_spawn_pos: Some(cached),
            ..Raid::default()
        };
        update_wave_spawn_cache(world, &mut raid);
        assert_eq!(raid.wave_spawn_pos, Some(cached));
        holder.set_simulation_level(Some(ChunkTicketLevel::BLOCK_TICKING_CHUNK));
        update_wave_spawn_cache(world, &mut raid);
        assert!(raid.wave_spawn_pos.is_none());
    }

    #[test]
    fn ground_placement_accepts_non_air_vegetation_and_rejects_hazards_and_signals() {
        steel_registry::init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("raid_spawn_surface");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let pos = BlockPos::new(1, 64, 1);
        world.set_block(
            pos.below(),
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_NONE,
        );
        for block in [&vanilla_blocks::AIR, &vanilla_blocks::SHORT_GRASS] {
            world.set_block(pos, block.default_state(), UpdateFlags::UPDATE_NONE);
            assert!(valid_raid_spawn_surface(world, pos), "{}", block.key);
        }
        for block in [
            &vanilla_blocks::STONE,
            &vanilla_blocks::WATER,
            &vanilla_blocks::WITHER_ROSE,
            &vanilla_blocks::STONE_PRESSURE_PLATE,
        ] {
            world.set_block(pos, block.default_state(), UpdateFlags::UPDATE_NONE);
            assert!(!valid_raid_spawn_surface(world, pos), "{}", block.key);
        }
        world.set_block(
            pos,
            vanilla_blocks::AIR.default_state(),
            UpdateFlags::UPDATE_NONE,
        );
        world.set_block(
            pos.below(),
            vanilla_blocks::SNOW.default_state(),
            UpdateFlags::UPDATE_NONE,
        );
        assert!(
            valid_raid_spawn_surface(world, pos),
            "vanilla snow fallback"
        );
    }

    #[test]
    fn spawn_position_cache_is_transient_like_vanilla() {
        let data = RaidSavedData {
            raids: vec![Raid {
                wave_spawn_pos: Some(IVec3::new(50, 64, -60)),
                ..Raid::default()
            }],
            ..RaidSavedData::default()
        };
        let encoded = toml::to_string(&data).unwrap();
        let decoded: RaidSavedData = toml::from_str(&encoded).unwrap();
        assert!(decoded.raids[0].wave_spawn_pos.is_none());
        assert_eq!(decoded.raids[0].uuid, data.raids[0].uuid);
    }
}

#[cfg(test)]
mod joining_and_killer_tests {
    use super::*;
    use crate::entity::damage::DamageSource;
    use crate::entity::{SharedEntity, next_entity_id};
    use crate::test_support::{TestPlayerBuilder, fresh_test_world, insert_ready_full_chunk};
    use steel_registry::{entity_type::EntityTypeRef, vanilla_damage_types};
    use steel_utils::ChunkPos;
use simdnbt::borrow::read_compound;
use simdnbt::owned::NbtCompound;
use crate::behavior::init_behaviors;
use std::io::Cursor;

    fn spawn(
        world: &Arc<World>,
        entity_type: EntityTypeRef,
        reason: EntitySpawnReason,
    ) -> SharedEntity {
        spawn_entity(
            world,
            EntitySpawnRequest {
                entity_type,
                placement: EntitySpawnPlacement::Block {
                    pos: BlockPos::new(2, 64, 2),
                    try_move_down: false,
                    moved_up: false,
                },
                reason,
                finalize_spawn: true,
                play_ambient_sound: false,
                item_stack: None,
                user_is_operator: false,
            },
        )
        .unwrap()
    }

    #[test]
    fn raider_ai_joins_once_blocks_despawning_and_respects_idle_and_witch_exclusions() {
        steel_registry::init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("raid_external_join");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let center = IVec3::new(0, 64, 0);
        world.raids.raise_omen(center, 0);
        let raider = spawn(
            world,
            &vanilla_entities::PILLAGER,
            EntitySpawnReason::Command,
        );
        let mob = raider.as_mob().unwrap();
        mob.mob_server_ai_step(&raider);
        assert!(world.raids.contains_raider(raider.uuid()));
        assert_eq!(world.raids.raids()[0].total_health, 24);
        assert!(!world.raids.try_join_raider(&raider));
        assert!(!mob.remove_when_far_away(1_000_000.0));
        let witch = spawn(world, &vanilla_entities::WITCH, EntitySpawnReason::Natural);
        assert!(!witch.as_mob().unwrap().can_join_raid());
        assert!(!world.raids.try_join_raider(&witch));
        let event_witch = spawn(world, &vanilla_entities::WITCH, EntitySpawnReason::Event);
        assert!(world.raids.try_join_raider(&event_witch));
        let idle = spawn(
            world,
            &vanilla_entities::VINDICATOR,
            EntitySpawnReason::Command,
        );
        idle.as_living_entity().unwrap().set_no_action_time(2_401);
        assert!(!world.raids.try_join_raider(&idle));
        idle.as_living_entity().unwrap().set_no_action_time(2_400);
        assert!(world.raids.try_join_raider(&idle));
        let pig = spawn(world, &vanilla_entities::PIG, EntitySpawnReason::Command);
        pig.as_mob().unwrap().set_can_join_raid(true);
        assert!(!world.raids.try_join_raider(&pig));
        assert!(world.raids.stop(center));
        assert!(mob.remove_when_far_away(1_000_000.0));
    }

    #[test]
    fn raider_activity_uses_double_increment_and_only_raid_player_or_golem_resets() {
        let fixture = fresh_test_world("raid_activity_timer");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        world.raids.raise_omen(IVec3::new(0, 64, 0), 0);
        let entity = spawn(world, &vanilla_entities::PILLAGER, EntitySpawnReason::Event);
        let mob = entity.as_mob().unwrap();
        let golem = spawn(
            world,
            &vanilla_entities::IRON_GOLEM,
            EntitySpawnReason::Event,
        );
        mob.mob_base().set_target(Some(&golem), |_| true);
        mob.set_no_action_time(100);
        mob.update_mob_no_action_time();
        assert_eq!(mob.no_action_time(), 102); // not yet in a raid
        assert!(world.raids.try_join_raider(&entity));
        mob.update_mob_no_action_time();
        assert_eq!(mob.no_action_time(), 2);
        let pig = spawn(world, &vanilla_entities::PIG, EntitySpawnReason::Command);
        mob.mob_base().set_target(Some(&pig), |_| true);
        mob.set_no_action_time(100);
        mob.update_mob_no_action_time();
        assert_eq!(mob.no_action_time(), 102); // not a player or golem
        mob.mob_base().set_target(None, |_| true);
        mob.update_mob_no_action_time();
        assert_eq!(mob.no_action_time(), 104);
        pig.as_mob().unwrap().set_no_action_time(100);
        pig.as_mob().unwrap().update_mob_no_action_time();
        assert_eq!(pig.as_mob().unwrap().no_action_time(), 101);
    }

    #[test]
    fn eligibility_round_trips_and_missing_old_nbt_flag_defaults_to_false() {
        let fixture = fresh_test_world("raid_eligibility_nbt");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let raider = spawn(
            world,
            &vanilla_entities::PILLAGER,
            EntitySpawnReason::Command,
        );
        let mob = raider.as_mob().unwrap();
        for eligible in [false, true] {
            mob.set_can_join_raid(eligible);
            let mut nbt = NbtCompound::new();
            mob.save_mob(&mut nbt);
            let mut bytes = Vec::new();
            nbt.write(&mut bytes);
            let borrowed =
                read_compound(&mut Cursor::new(&bytes)).unwrap();
            mob.set_can_join_raid(!eligible);
            mob.load_mob((&borrowed).into());
            assert_eq!(mob.can_join_raid(), eligible);
        }
        let empty = NbtCompound::new();
        let mut bytes = Vec::new();
        empty.write(&mut bytes);
        let borrowed = read_compound(&mut Cursor::new(&bytes)).unwrap();
        mob.load_mob((&borrowed).into());
        assert!(!mob.can_join_raid());
    }

    #[test]
    fn hero_credit_uses_killing_damage_source_not_remembered_previous_player_hit() {
        steel_registry::init_vanilla_registry();
        init_behaviors();
        let fixture = fresh_test_world("raid_killing_source");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        world.raids.raise_omen(IVec3::new(0, 64, 0), 0);
        let hero = TestPlayerBuilder::new(Arc::clone(world), "Hero", next_entity_id()).build();
        world.players.insert(Arc::clone(&hero));
        let first = spawn(world, &vanilla_entities::PILLAGER, EntitySpawnReason::Event);
        assert!(world.raids.try_join_raider(&first));
        let living = first.as_living_entity().unwrap();
        living.set_last_hurt_by_player(hero.uuid(), 100);
        living.set_health(0.0);
        living.die(&DamageSource::environment(&vanilla_damage_types::GENERIC));
        assert!(world.raids.raids()[0].heroes.is_empty());
        let second = spawn(world, &vanilla_entities::PILLAGER, EntitySpawnReason::Event);
        assert!(world.raids.try_join_raider(&second));
        world.raids.raids.lock()[0].leader = Some(second.uuid());
        second.as_living_entity().unwrap().set_health(0.0);
        let hero_entity: SharedEntity = hero.clone();
        second
            .as_living_entity()
            .unwrap()
            .die(&DamageSource::direct(
                &vanilla_damage_types::PLAYER_ATTACK,
                hero_entity,
            ));
        let raid = &world.raids.raids()[0];
        assert_eq!(raid.heroes, vec![hero.uuid()]);
        assert!(raid.raiders.is_empty());
        assert!(raid.leader.is_none());
    }
}

#[cfg(test)]
mod departure_tests {
    use super::*;
    use crate::test_support::{fresh_test_world, insert_ready_full_chunk};
    use steel_utils::ChunkPos;

    #[test]
    fn departed_member_subtracts_current_health_clears_leader_and_never_credits_kills() {
        let fixture = fresh_test_world("raid_member_departure");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let entity = spawn_entity(
            world,
            EntitySpawnRequest {
                entity_type: &vanilla_entities::PILLAGER,
                placement: EntitySpawnPlacement::Block {
                    pos: BlockPos::new(2, 64, 2),
                    try_move_down: false,
                    moved_up: false,
                },
                reason: EntitySpawnReason::Event,
                finalize_spawn: true,
                play_ambient_sound: false,
                item_stack: None,
                user_is_operator: false,
            },
        )
        .unwrap();
        entity.as_living_entity().unwrap().set_health(13.0);
        let missing = Uuid::new_v4();
        let mut raid = Raid {
            center: IVec3::new(114, 64, 2),
            total_health: 48,
            raiders: vec![entity.uuid(), missing],
            leader: Some(entity.uuid()),
            ..Raid::default()
        };
        update_raiders(world, &mut raid);
        assert_eq!(raid.raiders, vec![missing]); // distance exactly 112 blocks
        assert_eq!(raid.total_health, 35); // current health, not max health
        assert!(raid.leader.is_none());
        assert!(raid.heroes.is_empty());
        update_raiders(world, &mut raid);
        assert_eq!(raid.total_health, 35); // departure is charged only once
        raid.center = IVec3::new(113, 64, 2);
        raid.raiders.push(entity.uuid());
        update_raiders(world, &mut raid);
        assert!(raid.raiders.contains(&entity.uuid())); // 111 is still a member
    }
    #[test]
    fn inactive_outside_members_require_age_and_thirty_checks_not_thirty_ticks() {
        let fixture = fresh_test_world("raid_inactive_departure");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));
        let entity = spawn_entity(
            world,
            EntitySpawnRequest {
                entity_type: &vanilla_entities::PILLAGER,
                placement: EntitySpawnPlacement::Block {
                    pos: BlockPos::new(2, 64, 2),
                    try_move_down: false,
                    moved_up: false,
                },
                reason: EntitySpawnReason::Event,
                finalize_spawn: true,
                play_ambient_sound: false,
                item_stack: None,
                user_is_operator: false,
            },
        )
        .unwrap();
        let mut raid = Raid {
            center: IVec3::new(2, 64, 2),
            total_health: 24,
            raiders: vec![entity.uuid()],
            ..Raid::default()
        };
        entity.as_living_entity().unwrap().set_no_action_time(2_401);
        for _ in 0..35 {
            update_raiders(world, &mut raid);
        }
        assert_eq!(raid.raiders.len(), 1); // <=600 ticks old
        for _ in 0..601 {
            entity.advance_tick_count();
        }
        entity.as_living_entity().unwrap().set_no_action_time(2_400);
        update_raiders(world, &mut raid);
        assert_eq!(raid.outside_raid_checks[&entity.uuid()], 0);
        entity.as_living_entity().unwrap().set_no_action_time(2_401);
        for _ in 0..29 {
            update_raiders(world, &mut raid);
        }
        assert_eq!(raid.raiders.len(), 1);
        update_raiders(world, &mut raid);
        assert!(raid.raiders.is_empty());
        assert_eq!(raid.total_health, 0);
        assert!(raid.outside_raid_checks.is_empty());
        assert!(raid.heroes.is_empty());
    }
}
