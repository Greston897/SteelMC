//! Vanilla `net.minecraft.world.entity.monster.Enderman`.
//!
//! Tall hostile mob with stare-triggered player targeting, evasive
//! projectile-teleportation, and synchronized carried-block and anger state.
//! Dimensions and attributes come from the extracted `entities.json`; daytime
//! behavior is vanilla (Endermen are not daylight-burning mobs).
//!
//! Deviations (documented in `PARITY-26.3-RU.md`): natural spawning and the
//! block pickup/place goals are separate follow-up systems.

use std::iter::empty;
use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::EndermanEntityData;
use steel_registry::{sound_events, vanilla_attributes};
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockStateId, DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    FloatGoal, Goal, GoalControls, HurtByTargetGoal, LookAtPlayerGoal, MeleeAttackGoal,
    RandomLookAroundGoal, WaterAvoidingRandomStrollGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::{
    DefaultEntityPush, Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntityPose,
    EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase, Monster, PathfinderMob,
    SharedEntity, sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::world::World;

const DEFAULT_STEP_HEIGHT: f32 = 0.6;
const STARE_THRESHOLD: f64 = 0.025;

/// Vanilla `net.minecraft.world.entity.monster.Enderman`.
#[entity_behavior(class = "Enderman")]
pub struct EndermanEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: Arc<SyncMutex<EndermanEntityData>>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `EndermanEntity`.
unsafe impl DowncastType for EndermanEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/enderman");
}

impl EndermanEntity {
    /// Creates a new entity of this vanilla type.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Loads a saved entity of this vanilla type.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self::new_with_base(
            EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
        )
    }

    fn new_with_base(base: EntityBase, entity_type: EntityTypeRef) -> Self {
        let living_base = LivingEntityBase::new(entity_type);
        let mob_base = MobBase::new();

        let mut entity_data = EndermanEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);
        let entity_data = Arc::new(SyncMutex::new(entity_data));

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(2, MeleeAttackGoal::new(1.0, false));
            goal_selector.add_goal(7, WaterAvoidingRandomStrollGoal::new(1.0));
            goal_selector.add_goal(8, LookAtPlayerGoal::new(8.0));
            goal_selector.add_goal(9, RandomLookAroundGoal::new());

            let mut target_selector = mob_base.target_selector().lock();
            target_selector.add_goal(1, HurtByTargetGoal::new().set_alert_others(empty()));
            target_selector.add_goal(
                2,
                EndermanStareTargetGoal::new(Arc::clone(&entity_data)),
            );
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            entity_data,
        }
    }

    /// Returns the optional block state carried in vanilla's synced data.
    #[must_use]
    pub fn carried_block(&self) -> Option<BlockStateId> {
        *self.entity_data.lock().carry_state.get()
    }

    /// Sets the optional carried block state shown by the client.
    pub fn set_carried_block(&self, state: Option<BlockStateId>) {
        self.entity_data.lock().carry_state.set(state);
    }

    /// Whether the enderman's synced anger animation is active.
    #[must_use]
    pub fn is_creepy(&self) -> bool {
        *self.entity_data.lock().creepy.get()
    }

    /// Whether the synced stare reaction is active.
    #[must_use]
    pub fn is_stared_at(&self) -> bool {
        *self.entity_data.lock().stared_at.get()
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        sync_dirty_mob_effects(&self.living_base, &self.entity_data);
    }

    fn teleport_away_from_projectile(&self) {
        let Some(world) = self.level() else { return; };
        let origin = self.position();
        for _ in 0..16 {
            let x = origin.x + rand::random_range(-16.0..16.0);
            let y = origin.y + rand::random_range(-8.0..8.0);
            let z = origin.z + rand::random_range(-16.0..16.0);
            if self.random_teleport(&world, x, y, z, true) {
                self.play_sound(&sound_events::ENTITY_ENDERMAN_TELEPORT, 1.0, 1.0);
                return;
            }
        }
    }

}


struct EndermanStareTargetGoal {
    entity_data: Arc<SyncMutex<EndermanEntityData>>,
    target: Option<SharedEntity>,
}

impl EndermanStareTargetGoal {
    fn new(entity_data: Arc<SyncMutex<EndermanEntityData>>) -> Self {
        Self { entity_data, target: None }
    }

    fn set_stare_flags(&self, stared_at: bool, creepy: bool) {
        let mut data = self.entity_data.lock();
        if *data.stared_at.get() != stared_at {
            data.stared_at.set(stared_at);
        }
        if *data.creepy.get() != creepy {
            data.creepy.set(creepy);
        }
    }
}

impl Goal for EndermanStareTargetGoal {
    fn controls(&self) -> GoalControls { GoalControls::TARGET }

    fn can_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        let Some(world) = mob.level() else { return false; };
        let position = mob.position();
        let eye = DVec3::new(position.x, mob.get_eye_y(), position.z);
        let follow_range = mob.attributes().lock()
            .get_value(vanilla_attributes::FOLLOW_RANGE).unwrap_or(64.0);
        self.target = world.nearest_player(eye, follow_range, |player| {
            !player.is_spectator()
                && mob.has_line_of_sight_cached(player)
                && is_staring_at(player.position(), player.get_eye_y(), player.look_angle(), eye)
        }).map(|player| -> SharedEntity { player });
        self.target.is_some()
    }

    fn can_continue_to_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        mob.target().is_some_and(|target| target.is_alive())
    }

    fn start(&mut self, mob: &dyn PathfinderMob) {
        let _ = mob.set_target(self.target.as_ref());
        self.set_stare_flags(true, true);
    }

    fn stop(&mut self, mob: &dyn PathfinderMob) {
        if mob.target().is_some_and(|target| self.target.as_ref().is_some_and(|ours| target.uuid() == ours.uuid())) {
            let _ = mob.set_target(None);
        }
        self.target = None;
        self.set_stare_flags(false, false);
    }
}

fn is_staring_at(player_position: DVec3, player_eye_y: f64, look: DVec3, target_eye: DVec3) -> bool {
    let toward = target_eye - DVec3::new(player_position.x, player_eye_y, player_position.z);
    let distance_sqr = toward.length_squared();
    if distance_sqr <= f64::EPSILON { return true; }
    let distance = distance_sqr.sqrt();
    look.dot(toward / distance) > 1.0 - STARE_THRESHOLD / distance
}

impl DefaultEntityPush for EndermanEntity {}

impl Entity for EndermanEntity {
    fn is_hard_collision_relevant(&self) -> bool {
        false
    }

    fn as_default_entity_push(&self) -> Option<&dyn DefaultEntityPush> {
        Some(self)
    }

    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn movement_emission(&self) -> EntityMovementEmission {
        EntityMovementEmission::Events
    }

    fn base_tick(&self) {
        Mob::base_tick_mob(self);
    }

    fn hurt(&self, world: &World, source: &DamageSource, amount: f32) -> bool {
        let hurt = LivingEntity::hurt_server(self, world, source, amount);
        if hurt && source.direct_entity().is_some_and(|entity| entity.as_projectile().is_some()) {
            self.teleport_away_from_projectile();
        }
        hurt
    }

    fn dimensions_for_pose(&self, _pose: EntityPose) -> EntityDimensions {
        let scale = LivingEntity::get_scale(self);
        if self.entity_type.fixed {
            self.entity_type.dimensions
        } else {
            self.entity_type.dimensions.scale(scale)
        }
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(self.entity_data.as_ref())
    }

    fn update_data_before_sync(&self) {
        self.update_dirty_mob_effect_entity_data();
    }

    fn max_up_step(&self) -> f32 {
        self.attributes()
            .lock()
            .get_value(vanilla_attributes::STEP_HEIGHT)
            .unwrap_or(f64::from(DEFAULT_STEP_HEIGHT)) as f32
    }

    fn sound_source(&self) -> SoundSource {
        Monster::sound_source_monster(self)
    }

    fn fall_sounds(&self) -> (SoundEventRef, SoundEventRef) {
        Monster::fall_sounds_monster(self)
    }

    fn swim_sound(&self) -> SoundEventRef {
        Monster::swim_sound_monster(self)
    }

    fn swim_splash_sound(&self) -> SoundEventRef {
        Monster::swim_splash_sound_monster(self)
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
    }
}

impl LivingEntity for EndermanEntity {
    fn living_base(&self) -> &LivingEntityBase {
        &self.living_base
    }

    fn get_health(&self) -> f32 {
        *self.entity_data.lock().living_entity().health.get()
    }

    fn set_health(&self, health: f32) {
        let max_health = self.get_max_health();
        let clamped = health.clamp(0.0, max_health);
        self.entity_data
            .lock()
            .living_entity_mut()
            .health
            .set(clamped);
    }

    fn hurt_sound(&self, _source: &DamageSource) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_ENDERMAN_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_ENDERMAN_DEATH)
    }

    fn should_drop_experience(&self) -> bool {
        Monster::should_drop_experience_monster(self)
    }

    fn should_drop_loot(&self, world: &World) -> bool {
        Monster::should_drop_loot_monster(self, world)
    }

    fn server_ai_step(&self, entity: &SharedEntity) {
        Mob::mob_server_ai_step(self, entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        Monster::monster_ai_step(self, entity)
    }
}

impl Mob for EndermanEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn set_target(&self, target: Option<&SharedEntity>) -> bool {
        let assigned = self
            .mob_base
            .set_target(target, |living| self.is_valid_target(living));
        if assigned {
            let mut data = self.entity_data.lock();
            let creepy = target.is_some();
            if *data.creepy.get() != creepy {
                data.creepy.set(creepy);
            }
            if target.is_none() && *data.stared_at.get() {
                data.stared_at.set(false);
            }
        }
        assigned
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_ENDERMAN_AMBIENT)
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl Monster for EndermanEntity {}

impl PathfinderMob for EndermanEntity {}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::Weak;

    use glam::DVec3;
    use simdnbt::borrow::read_compound;
    use simdnbt::owned::NbtCompound;
    use steel_registry::{init_vanilla_registry, vanilla_entities};
    use steel_utils::BlockStateId;

    use super::{EndermanEntity, is_staring_at};
    use crate::entity::{Entity, LivingEntity};

    #[test]
    fn gaze_test_matches_forward_and_away_vectors() {
        let player = DVec3::new(0.0, 0.0, 0.0);
        let target = DVec3::new(0.0, 1.6, -8.0);
        assert!(is_staring_at(player, 1.6, DVec3::NEG_Z, target));
        assert!(!is_staring_at(player, 1.6, DVec3::Z, target));
    }

    #[test]
    fn enderman_uses_registry_attributes_and_round_trips_nbt() {
        init_vanilla_registry();
        let enderman =
            EndermanEntity::new(&vanilla_entities::ENDERMAN, 1, DVec3::ZERO, Weak::new());
        assert!((enderman.get_max_health() - 40.0).abs() < 1e-4);
        assert_eq!(enderman.carried_block(), None);
        enderman.set_carried_block(Some(BlockStateId(1)));
        assert_eq!(enderman.carried_block(), Some(BlockStateId(1)));
        enderman.set_carried_block(None);
        assert!(!enderman.is_creepy());
        assert!(!enderman.is_stared_at());

        let mut nbt = NbtCompound::new();
        enderman.save_additional(&mut nbt);

        let loaded =
            EndermanEntity::new(&vanilla_entities::ENDERMAN, 2, DVec3::ZERO, Weak::new());
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let borrowed = read_compound(&mut Cursor::new(&bytes))
            .unwrap_or_else(|error| panic!("reborrow failed: {error}"));
        loaded.load_additional((&borrowed).into());
        assert!(loaded.get_health() > 0.0);
    }
}
