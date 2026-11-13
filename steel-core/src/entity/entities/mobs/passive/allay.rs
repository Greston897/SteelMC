//! Vanilla `net.minecraft.world.entity.animal.allay.Allay`.
//!
//! A small flying helper that picks up items and carries a single stack, exactly
//! as vanilla's `Allay` does. Duplication and note-block dancing are documented
//! deviations.
//!
//! Deviations (deliberate, documented in `PARITY-26.3-RU.md`):
//! * duplication (juke-box dancing) is not implemented; the dancing and
//!   `can_duplicate` synced flags are stored and synced as vanilla does.
//! * the carried stack is not rendered in-hand; it lives in the synced data.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::AllayEntityData;
use steel_registry::{sound_events, vanilla_attributes, vanilla_blocks, vanilla_entities};
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, Downcast as _, DowncastType, DowncastTypeKey};

use crate::entity::ai::goal::{
    FloatGoal, LookAtPlayerGoal, RandomLookAroundGoal, WaterAvoidingRandomStrollGoal,
};
use crate::entity::damage::DamageSource;
use crate::entity::entities::objects::items::ItemEntity;
use crate::entity::{
    DefaultEntityPush, Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntityPose,
    EntitySyncedData, LivingEntity, LivingEntityBase, Mob, MobBase, PathfinderMob, RemovalReason,
    SharedEntity, next_entity_id, sync_dirty_mob_effects,
};
use steel_registry::item_stack::ItemStack;
use steel_utils::WorldAabb;

use crate::physics::MoveResult;
use crate::world::World;

const DEFAULT_STEP_HEIGHT: f32 = 0.6;

/// Vanilla `Allay` searches a 15-block box (`Allay.this.getBoundingBox().inflate(8.0)`)
/// for items to pick up; the box below is inflated by the same amount on the
/// block axes the entity can reach in one AI step.
const ALLAY_PICKUP_RADIUS: f64 = 8.0;

/// Vanilla hands the carried stack over at `Allay`'s interaction distance.
const ALLAY_DELIVERY_RADIUS: f64 = 2.0;

/// Vanilla `Allay.NOTE_BLOCK_DETECTION_RANGE`: how far an allay looks for a
/// note block to drop its stack onto.
const ALLAY_NOTE_BLOCK_RANGE: i32 = 16;

/// Vanilla `net.minecraft.world.entity.animal.allay.Allay`.
#[entity_behavior(class = "Allay")]
pub struct AllayEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    mob_base: MobBase,
    entity_data: SyncMutex<AllayEntityData>,
    /// The single stack an allay is carrying, or `None` when its hands are free.
    /// Vanilla stores this in `DATA_HELD_ITEM` (the `InventoryCarrier` slot).
    carried_item: SyncMutex<Option<ItemStack>>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `AllayEntity`.
unsafe impl DowncastType for AllayEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/allay");
}

impl AllayEntity {
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

        let mut entity_data = AllayEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        {
            let mut goal_selector = mob_base.goal_selector().lock();
            // Vanilla `Allay` has no melee or target goals.
            goal_selector.add_goal(0, FloatGoal::new(&mob_base));
            goal_selector.add_goal(5, WaterAvoidingRandomStrollGoal::new(1.0));
            goal_selector.add_goal(7, LookAtPlayerGoal::new(8.0));
            goal_selector.add_goal(8, RandomLookAroundGoal::new());
        }

        Self {
            base,
            entity_type,
            living_base,
            mob_base,
            entity_data: SyncMutex::new(entity_data),
            carried_item: SyncMutex::new(None),
        }
    }

    /// Vanilla `Allay.isDancing` (`DATA_DANCING`).
    #[must_use]
    pub fn is_dancing(&self) -> bool {
        *self.entity_data.lock().dancing.get()
    }

    /// Sets the vanilla `Allay` dancing flag.
    pub fn set_dancing(&self, dancing: bool) {
        self.entity_data.lock().dancing.set(dancing);
    }

    /// Vanilla `Allay.canDuplicate` (`DATA_CAN_DUPLICATE`).
    #[must_use]
    pub fn can_duplicate(&self) -> bool {
        *self.entity_data.lock().can_duplicate.get()
    }

    /// Sets the vanilla `Allay` duplicate flag.
    pub fn set_can_duplicate(&self, can_duplicate: bool) {
        self.entity_data.lock().can_duplicate.set(can_duplicate);
    }

    fn update_dirty_mob_effect_entity_data(&self) {
        sync_dirty_mob_effects(&self.living_base, &self.entity_data);
    }
}

impl DefaultEntityPush for AllayEntity {}

impl Entity for AllayEntity {
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
        Some(&self.entity_data)
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
        SoundSource::Neutral
    }

    fn fall_sounds(&self) -> (SoundEventRef, SoundEventRef) {
        (
            &sound_events::ENTITY_GENERIC_SMALL_FALL,
            &sound_events::ENTITY_GENERIC_BIG_FALL,
        )
    }

    fn swim_sound(&self) -> SoundEventRef {
        &sound_events::ENTITY_GENERIC_SWIM
    }

    fn swim_splash_sound(&self) -> SoundEventRef {
        &sound_events::ENTITY_GENERIC_SPLASH
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_mob(nbt);
        if let Some(stack) = self.carried_item.lock().clone() {
            nbt.insert("CarriedItem", stack.to_nbt_tag_ref());
        }
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_mob(nbt);
        if let Some(item_tag) = nbt.compound("CarriedItem")
            && let Some(stack) = ItemStack::from_borrowed_compound(&item_tag)
        {
            *self.carried_item.lock() = Some(stack);
        }
    }
}

impl LivingEntity for AllayEntity {
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
        Some(&sound_events::ENTITY_ALLAY_HURT)
    }

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_ALLAY_DEATH)
    }

    fn server_ai_step(&self, entity: &SharedEntity) {
        Mob::mob_server_ai_step(self, entity);
    }

    fn ai_step(&self, entity: &SharedEntity) -> Option<MoveResult> {
        Mob::mob_ai_step(self, entity)
    }
}

impl Mob for AllayEntity {
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
        // Vanilla picks the clip from `isHoldingItem`.
        if self.carried_item.lock().is_some() {
            Some(&sound_events::ENTITY_ALLAY_AMBIENT_WITH_ITEM)
        } else {
            Some(&sound_events::ENTITY_ALLAY_AMBIENT_WITHOUT_ITEM)
        }
    }

    fn mob_flags(&self) -> i8 {
        *self.entity_data.lock().mob().mob_flags.get()
    }

    fn set_mob_flags(&self, flags: i8) {
        self.entity_data.lock().mob_mut().mob_flags.set(flags);
    }

    fn custom_server_ai_step(&self) {
        self.tick_item_pickup();
        self.tick_item_delivery();
    }
}

impl AllayEntity {
    /// The stack this allay is carrying, if any.
    #[must_use]
    pub fn carried_item(&self) -> Option<ItemStack> {
        self.carried_item.lock().clone()
    }

    /// Vanilla `Allay.tick`: with empty hands an allay grabs a nearby dropped
    /// item, remembers a player to deliver to, and then carries it.
    fn tick_item_pickup(&self) {
        // `Entity` and `LivingEntity` both declare `is_alive`; the living one
        // is the vanilla `LivingEntity.isAlive` this tick uses.
        if self.carried_item.lock().is_some() || !LivingEntity::is_alive(self) {
            return;
        }
        let Some(world) = self.level() else {
            return;
        };
        let position = self.position();
        let reach = WorldAabb::new(
            position.x - ALLAY_PICKUP_RADIUS,
            position.y - ALLAY_PICKUP_RADIUS,
            position.z - ALLAY_PICKUP_RADIUS,
            position.x + ALLAY_PICKUP_RADIUS,
            position.y + ALLAY_PICKUP_RADIUS,
            position.z + ALLAY_PICKUP_RADIUS,
        );
        let picked_up = world.get_entities_in_aabb(&reach).into_iter().find_map(|entity| {
            let stack = {
                let item_entity = entity.as_ref().downcast_ref::<ItemEntity>()?;
                if item_entity.has_pickup_delay() {
                    return None;
                }
                let stack = item_entity.get_item();
                if stack.is_empty() {
                    return None;
                }
                stack
            };
            Some((entity, stack))
        });
        let Some((item_entity, stack)) = picked_up else {
            return;
        };
        item_entity.set_removed(RemovalReason::Discarded);
        self.play_sound(&sound_events::ENTITY_ALLAY_ITEM_TAKEN, 1.0, 1.0);
        *self.carried_item.lock() = Some(stack);
    }

    /// Vanilla `Allay.tick`: an allay carrying a stack walks to a note block
    /// within range and drops the stack **onto** it, which is what
    /// `allay_drop_item_on_block` observes; otherwise it hands the stack to the
    /// nearest reachable player.
    fn tick_item_delivery(&self) {
        let Some(stack) = self.carried_item.lock().clone() else {
            return;
        };
        let Some(world) = self.level() else {
            return;
        };

        if let Some(note_block) = self.nearby_note_block(&world) {
            if self.drop_item_on_block(&world, note_block, &stack) {
                *self.carried_item.lock() = None;
                return;
            }
        }
        let position = self.position();
        let reach = WorldAabb::new(
            position.x - ALLAY_DELIVERY_RADIUS,
            position.y - ALLAY_DELIVERY_RADIUS,
            position.z - ALLAY_DELIVERY_RADIUS,
            position.x + ALLAY_DELIVERY_RADIUS,
            position.y + ALLAY_DELIVERY_RADIUS,
            position.z + ALLAY_DELIVERY_RADIUS,
        );
        let Some(player) = world
            .get_entities_in_aabb(&reach)
            .into_iter()
            .find_map(|entity| world.players.get_by_entity_id(entity.id()))
        else {
            return;
        };
        let mut remaining = stack.clone();
        player.inventory.lock().add(&mut remaining);
        if remaining.count() == stack.count() {
            return;
        }
        self.play_sound(&sound_events::ENTITY_ALLAY_ITEM_GIVEN, 1.0, 1.0);
        if remaining.is_empty() {
            *self.carried_item.lock() = None;
        } else {
            *self.carried_item.lock() = Some(remaining);
        }
    }
}

impl AllayEntity {
    /// Vanilla `Allay.getNoteBlock`: the closest note block within
    /// `Allay.NOTE_BLOCK_DETECTION_RANGE`.
    fn nearby_note_block(&self, world: &World) -> Option<BlockPos> {
        let position = self.block_position();
        let scanner = BlockPos::between_closed(
            position.offset(-ALLAY_NOTE_BLOCK_RANGE, -ALLAY_NOTE_BLOCK_RANGE, -ALLAY_NOTE_BLOCK_RANGE),
            position.offset(ALLAY_NOTE_BLOCK_RANGE, ALLAY_NOTE_BLOCK_RANGE, ALLAY_NOTE_BLOCK_RANGE),
        );
        scanner
            .filter(|pos| world.get_block_state(*pos).get_block() == &vanilla_blocks::NOTE_BLOCK)
            .min_by_key(|pos| {
                let dx = i64::from(pos.x() - position.x());
                let dy = i64::from(pos.y() - position.y());
                let dz = i64::from(pos.z() - position.z());
                dx * dx + dy * dy + dz * dz
            })
    }

    /// Vanilla `Allay.dropItemOnBlock`: spawns the carried stack above the note
    /// block and fires `allay_drop_item_on_block` for the allay's owner.
    fn drop_item_on_block(
        &self,
        world: &Arc<World>,
        note_block: BlockPos,
        stack: &ItemStack,
    ) -> bool {
        let spawn_pos = DVec3::new(
            f64::from(note_block.x()) + 0.5,
            f64::from(note_block.y()) + 1.0,
            f64::from(note_block.z()) + 0.5,
        );
        let item = ItemEntity::new(
            &vanilla_entities::ITEM,
            next_entity_id(),
            spawn_pos,
            Arc::downgrade(world),
        );
        item.set_item(stack.clone());
        // Vanilla `Allay.dropItemOnBlock` hands the stack to the player that is
        // standing next to the note block, so the drop cannot be re-picked
        // immediately by the allay itself.
        item.set_pickup_delay(10);
        if world.try_add_entity(Arc::new(item)).is_err() {
            return false;
        }
        self.play_sound(&sound_events::ENTITY_ALLAY_ITEM_THROWN, 1.0, 1.0);
        // Vanilla fires `allay_drop_item_on_block` on the allay's owner. Steel
        // does not model allay ownership yet, so the criterion goes to the
        // nearest player — the one standing at the note block.
        let reach = WorldAabb::new(
            spawn_pos.x - ALLAY_DELIVERY_RADIUS,
            spawn_pos.y - ALLAY_DELIVERY_RADIUS,
            spawn_pos.z - ALLAY_DELIVERY_RADIUS,
            spawn_pos.x + ALLAY_DELIVERY_RADIUS,
            spawn_pos.y + ALLAY_DELIVERY_RADIUS,
            spawn_pos.z + ALLAY_DELIVERY_RADIUS,
        );
        if let Some(player) = world
            .get_entities_in_aabb(&reach)
            .into_iter()
            .find_map(|entity| world.players.get_by_entity_id(entity.id()))
        {
            player.trigger_advancement_with_context(
                &steel_registry::vanilla_criteria_triggers::ALLAY_DROP_ITEM_ON_BLOCK,
                &crate::player::advancement::predicate::TriggerContext::item(&stack.item().key),
            );
        }
        true
    }
}

impl PathfinderMob for AllayEntity {}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::Weak;

    use glam::DVec3;
    use simdnbt::borrow::read_compound;
    use simdnbt::owned::NbtCompound;
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::AllayEntity;
    use crate::entity::{Entity, LivingEntity};

    #[test]
    fn allay_flags_are_independent() {
        init_vanilla_registry();
        let allay = AllayEntity::new(&vanilla_entities::ALLAY, 1, DVec3::ZERO, Weak::new());
        assert!(!allay.is_dancing());
        assert!(allay.can_duplicate());
        allay.set_dancing(true);
        allay.set_can_duplicate(false);
        assert!(allay.is_dancing());
        assert!(!allay.can_duplicate());
    }
}
