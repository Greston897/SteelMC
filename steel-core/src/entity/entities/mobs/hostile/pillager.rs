//! Vanilla `net.minecraft.world.entity.monster.Pillager`.
//!
//! Crossbow-wielding illager with a 25-tick charge cycle and ranged combat.
//! Attributes and dimensions come from `entities.json`; daylight burning is
//! driven by the `minecraft:burn_in_daylight` entity type tag.
//!
//! Deviations (deliberate, documented in `PARITY-26.3-RU.md`):
//! * natural spawning is not wired yet: `SpawnRule` hands a `&dyn LevelReader`,
//!   while `Monster::check_monster_spawn_rules` needs `&World`, so the entity
//!   type is registered without a spawn rule and only spawners/commands create it.
//! * raid patrols, raid waves, and natural spawning are separate systems; shots
//!   use the shared mob-arrow helper rather than the player crossbow pipeline.

use std::iter::empty;
use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::item_stack::ItemStack;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::PillagerEntityData;
use steel_registry::{sound_events, vanilla_attributes, vanilla_entities, vanilla_items};
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    FloatGoal, Goal, GoalControls, HurtByTargetGoal, LookAtPlayerGoal,
    NearestAttackableTargetGoal, RandomLookAroundGoal, WaterAvoidingRandomStrollGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::ranged_attack;
use crate::entity::{
    DefaultEntityPush, Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntityPose,
    EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase, Monster, PathfinderMob,
    SharedEntity, sync_dirty_mob_effects,
};
use crate::physics::MoveResult;
use crate::world::World;

const DEFAULT_STEP_HEIGHT: f32 = 0.6;
const CROSSBOW_CHARGE_TICKS: i32 = 25;
const CROSSBOW_ATTACK_RADIUS: f64 = 15.0;
const CROSSBOW_ATTACK_RADIUS_SQR: f64 = CROSSBOW_ATTACK_RADIUS * CROSSBOW_ATTACK_RADIUS;
const CROSSBOW_RELOAD_TICKS: i32 = 20;
const LOOK_YAW_SPEED: f32 = 30.0;
const LOOK_PITCH_SPEED: f32 = 30.0;

/// Vanilla `net.minecraft.world.entity.monster.Pillager`.
#[entity_behavior(class = "Pillager")]
pub struct PillagerEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: Arc<SyncMutex<PillagerEntityData>>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `PillagerEntity`.
unsafe impl DowncastType for PillagerEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/pillager");
}

impl PillagerEntity {
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

        let mut entity_data = PillagerEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);
        let entity_data = Arc::new(SyncMutex::new(entity_data));
        living_base.equipment().lock().set(
            crate::inventory::equipment::EquipmentSlot::MainHand,
            ItemStack::new(&vanilla_items::CROSSBOW),
        );

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(2, PillagerCrossbowAttackGoal::new(Arc::clone(&entity_data)));
            goal_selector.add_goal(8, WaterAvoidingRandomStrollGoal::new(1.0));
            goal_selector.add_goal(9, LookAtPlayerGoal::new(8.0));
            goal_selector.add_goal(10, RandomLookAroundGoal::new());

            let mut target_selector = mob_base.target_selector().lock();
            target_selector.add_goal(1, HurtByTargetGoal::new().set_alert_others(empty()));
            target_selector.add_goal(
                2,
                NearestAttackableTargetGoal::new_for_players(true, |_, _| true),
            );
            target_selector.add_goal(
                3,
                NearestAttackableTargetGoal::new(true, |entity, _| {
                    entity.entity_type() == &vanilla_entities::VILLAGER
                        || entity.entity_type() == &vanilla_entities::WANDERING_TRADER
                }),
            );
            target_selector.add_goal(
                4,
                NearestAttackableTargetGoal::new(true, |entity, _| {
                    entity.entity_type() == &vanilla_entities::IRON_GOLEM
                }),
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

    fn update_dirty_mob_effect_entity_data(&self) {
        sync_dirty_mob_effects(&self.living_base, &self.entity_data);
    }
}


struct PillagerCrossbowAttackGoal {
    entity_data: Arc<SyncMutex<PillagerEntityData>>,
    charge_ticks: i32,
    reload_ticks: i32,
}

impl PillagerCrossbowAttackGoal {
    fn new(entity_data: Arc<SyncMutex<PillagerEntityData>>) -> Self {
        Self { entity_data, charge_ticks: 0, reload_ticks: 0 }
    }

    fn set_charging(&self, charging: bool) {
        let mut data = self.entity_data.lock();
        if *data.is_charging_crossbow.get() != charging {
            data.is_charging_crossbow.set(charging);
        }
    }
}

impl Goal for PillagerCrossbowAttackGoal {
    fn controls(&self) -> GoalControls {
        GoalControls::MOVE | GoalControls::LOOK
    }

    fn can_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        mob.target().is_some_and(|target| target.is_alive())
    }

    fn can_continue_to_use(&mut self, mob: &dyn PathfinderMob) -> bool {
        mob.target().is_some_and(|target| target.is_alive())
    }

    fn start(&mut self, mob: &dyn PathfinderMob) {
        mob.set_aggressive(true);
        self.charge_ticks = 0;
        self.reload_ticks = 0;
        self.set_charging(false);
    }

    fn stop(&mut self, mob: &dyn PathfinderMob) {
        mob.set_aggressive(false);
        mob.mob_base().navigation().lock().stop();
        self.charge_ticks = 0;
        self.reload_ticks = 0;
        self.set_charging(false);
    }

    fn requires_update_every_tick(&self) -> bool { true }

    fn tick(&mut self, mob: &dyn PathfinderMob, entity: &SharedEntity) {
        let Some(target) = mob.target() else { return; };
        let target_position = target.position();
        let distance_sqr = mob.distance_to_sqr(target_position);
        let can_see = mob.has_line_of_sight_cached(target.as_ref());
        if distance_sqr > CROSSBOW_ATTACK_RADIUS_SQR || !can_see {
            mob.move_to_pos(target_position, 1.0);
            self.set_charging(false);
            self.charge_ticks = 0;
        } else {
            mob.mob_base().navigation().lock().stop();
            if self.reload_ticks > 0 {
                self.reload_ticks -= 1;
                self.set_charging(false);
            } else {
                self.set_charging(true);
                self.charge_ticks += 1;
                if self.charge_ticks >= CROSSBOW_CHARGE_TICKS {
                    mob.perform_ranged_attack(entity, &target, 1.0);
                    self.charge_ticks = 0;
                    self.reload_ticks = CROSSBOW_RELOAD_TICKS;
                    self.set_charging(false);
                }
            }
        }
        mob.mob_base().controls().lock().look_control.set_look_at(
            DVec3::new(target_position.x, target.get_eye_y(), target_position.z),
            LOOK_YAW_SPEED,
            LOOK_PITCH_SPEED,
        );
    }
}

impl DefaultEntityPush for PillagerEntity {}

impl Entity for PillagerEntity {
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

    fn play_step_sound(&self, _pos: BlockPos, _block_state: BlockStateId) {
        self.play_sound(&sound_events::ENTITY_ZOMBIE_STEP, 0.15, 1.0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
    }
}

impl LivingEntity for PillagerEntity {
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
        Some(&sound_events::ENTITY_PILLAGER_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_PILLAGER_DEATH)
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

impl Mob for PillagerEntity {
    fn mob_base(&self) -> &MobBase {
        &self.mob_base
    }

    fn tick_goal_selectors(&self, entity: &SharedEntity) {
        PathfinderMob::tick_pathfinder_goal_selectors(self, entity);
    }

    fn tick_path_navigation(&self) {
        PathfinderMob::tick_pathfinder_path_navigation(self);
    }

    fn ambient_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_PILLAGER_AMBIENT)
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }
}

impl Monster for PillagerEntity {}

impl PathfinderMob for PillagerEntity {
    fn perform_ranged_attack(
        &self,
        self_entity: &SharedEntity,
        target: &SharedEntity,
        _power: f32,
    ) {
        if ranged_attack::shoot_arrow(self_entity, target, ranged_attack::MOB_ARROW_DAMAGE).is_some() {
            self.play_sound(&sound_events::ITEM_CROSSBOW_SHOOT, 1.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::Weak;

    use glam::DVec3;
    use simdnbt::borrow::read_compound;
    use simdnbt::owned::NbtCompound;
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::PillagerEntity;
    use crate::entity::{Entity, LivingEntity, Mob};
    use crate::inventory::equipment::EquipmentSlot;
    use steel_registry::vanilla_items;

    #[test]
    fn pillager_uses_registry_attributes_and_round_trips_nbt() {
        init_vanilla_registry();
        let pillager =
            PillagerEntity::new(&vanilla_entities::PILLAGER, 1, DVec3::ZERO, Weak::new());
        assert!((pillager.get_max_health() - 24.0).abs() < 1e-4);
        assert!(pillager.equipment_in_slot(EquipmentSlot::MainHand).is(&vanilla_items::CROSSBOW));
        assert!(!*pillager.entity_data.lock().is_charging_crossbow.get());

        let mut nbt = NbtCompound::new();
        pillager.save_additional(&mut nbt);

        let loaded =
            PillagerEntity::new(&vanilla_entities::PILLAGER, 2, DVec3::ZERO, Weak::new());
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let borrowed = read_compound(&mut Cursor::new(&bytes))
            .unwrap_or_else(|error| panic!("reborrow failed: {error}"));
        loaded.load_additional((&borrowed).into());
        assert!(loaded.get_health() > 0.0);
    }
}
