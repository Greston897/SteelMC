//! Armor stand entity (`ArmorStand`), the vanilla decoration stand.
//!
//! Vanilla armor stands are living entities (but not mobs): they have 20 health,
//! six equipment slots, six synced pose rotations and a single byte of client
//! flags (`Small`, `ShowArms`, `NoBasePlate`, `Marker`). Any damage destroys the
//! stand in one hit; destroying it drops every equipped item and the
//! `minecraft:armor_stand` item itself, and never awards experience.
//!
//! Documented deviations from vanilla 26.3 (see `PARITY-26.3-RU.md`):
//! * a right click only fills an *empty* slot. Vanilla replaces the item already
//!   in the target slot and hands it to the player; Steel's player inventory has
//!   no "place swapped item into hand" primitive yet, so replacing would delete
//!   the old item. Filling empty slots keeps the same gameplay without silent
//!   item loss;
//! * marker stands keep their vanilla behaviour for interaction (they are not
//!   interactable and not pickable), but stay damageable, exactly like Steel's
//!   other decoration entities.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_data::Rotations;
use steel_registry::entity_type::{EntityDimensions, EntityTypeRef};
use steel_registry::item_stack::ItemStack;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::stat::vanilla_stat_types;
use steel_registry::vanilla_entity_data::ArmorStandEntityData;
use steel_registry::{sound_events, vanilla_items};
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::types::InteractionHand;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey};

use crate::behavior::InteractionResult;
use crate::entity::damage::DamageSource;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntityPose, EntitySyncedData, LivingEntity,
    LivingEntityBase, RemovalReason, SharedEntity,
};
use crate::inventory::equipment::EquipmentSlot;
use crate::player::Player;
use crate::world::World;

/// Vanilla `ArmorStand` client flag: the stand renders at half height.
const FLAG_SMALL: i8 = 0x01;
/// Vanilla `ArmorStand` client flag: the stand shows its arms.
const FLAG_SHOW_ARMS: i8 = 0x04;
/// Vanilla `ArmorStand` client flag: no stone base plate is rendered.
const FLAG_NO_BASE_PLATE: i8 = 0x08;
/// Vanilla `ArmorStand` client flag: the stand is a marker (no hitbox).
const FLAG_MARKER: i8 = 0x10;

/// Vanilla `ArmorStand.SMALL_DIMENSIONS` height factor: small stands keep the
/// normal 0.5 width but render at half the height.
const SMALL_HEIGHT_FACTOR: f32 = 0.5;

/// Equipment slots an armor stand exposes: the four humanoid armor slots plus
/// both hands. `Body` and `Saddle` are not part of the stand's equipment.
const ARMOR_STAND_SLOTS: [EquipmentSlot; 6] = [
    EquipmentSlot::Head,
    EquipmentSlot::Chest,
    EquipmentSlot::Legs,
    EquipmentSlot::Feet,
    EquipmentSlot::MainHand,
    EquipmentSlot::OffHand,
];

/// The vanilla armor stand decoration entity.
#[entity_behavior(class = "ArmorStand")]
pub struct ArmorStandEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    living_base: LivingEntityBase,
    entity_data: SyncMutex<ArmorStandEntityData>,
    /// Set once the stand broke apart, so two damage paths cannot duplicate drops.
    broken: AtomicBool,
}

// SAFETY: This key is owned by Steel and uniquely identifies `ArmorStandEntity`.
unsafe impl DowncastType for ArmorStandEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/armor_stand");
}

impl ArmorStandEntity {
    /// Creates a fresh armor stand from the generic entity factory path.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_with_base(
            EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
        )
    }

    /// Creates an armor stand from persistent entity data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self::new_with_base(EntityBase::from_load(load, entity_type.dimensions), entity_type)
    }

    fn new_with_base(base: EntityBase, entity_type: EntityTypeRef) -> Self {
        let living_base = LivingEntityBase::new(entity_type);
        let mut entity_data = ArmorStandEntityData::new();
        living_base.initialize_synced_data(&mut entity_data);

        Self {
            base,
            entity_type,
            living_base,
            entity_data: SyncMutex::new(entity_data),
            broken: AtomicBool::new(false),
        }
    }

    /// Returns the raw vanilla `DATA_CLIENT_FLAGS` byte.
    #[must_use]
    fn client_flags(&self) -> i8 {
        *self.entity_data.lock().client_flags.get()
    }

    fn has_client_flag(&self, flag: i8) -> bool {
        (self.client_flags() & flag) != 0
    }

    fn set_client_flag(&self, flag: i8, value: bool) {
        let mut data = self.entity_data.lock();
        let flags = *data.client_flags.get();
        let updated = if value { flags | flag } else { flags & !flag };
        data.client_flags.set(updated);
    }

    /// Whether the stand is small (half height).
    #[must_use]
    pub fn is_small(&self) -> bool {
        self.has_client_flag(FLAG_SMALL)
    }

    /// Sets the small flag and refreshes the stand's dimensions, like vanilla
    /// `ArmorStand.setSmall`.
    pub fn set_small(&self, small: bool) {
        if self.is_small() == small {
            return;
        }
        self.set_client_flag(FLAG_SMALL, small);
        Entity::refresh_dimensions(self);
    }

    /// Whether the stand renders its arms.
    #[must_use]
    pub fn show_arms(&self) -> bool {
        self.has_client_flag(FLAG_SHOW_ARMS)
    }

    /// Sets the show-arms flag.
    pub fn set_show_arms(&self, show_arms: bool) {
        self.set_client_flag(FLAG_SHOW_ARMS, show_arms);
    }

    /// Whether the stand renders without a base plate.
    #[must_use]
    pub fn no_base_plate(&self) -> bool {
        self.has_client_flag(FLAG_NO_BASE_PLATE)
    }

    /// Sets the no-base-plate flag.
    pub fn set_no_base_plate(&self, no_base_plate: bool) {
        self.set_client_flag(FLAG_NO_BASE_PLATE, no_base_plate);
    }

    /// Whether the stand is a marker (no hitbox, not interactable).
    #[must_use]
    pub fn is_marker(&self) -> bool {
        self.has_client_flag(FLAG_MARKER)
    }

    /// Sets the marker flag.
    pub fn set_marker(&self, marker: bool) {
        self.set_client_flag(FLAG_MARKER, marker);
    }

    /// Returns the synced head pose.
    #[must_use]
    pub fn head_pose(&self) -> Rotations {
        *self.entity_data.lock().head_pose.get()
    }

    /// Sets the synced head pose.
    pub fn set_head_pose(&self, pose: Rotations) {
        self.entity_data.lock().head_pose.set(pose);
    }

    /// Returns the synced body pose.
    #[must_use]
    pub fn body_pose(&self) -> Rotations {
        *self.entity_data.lock().body_pose.get()
    }

    /// Sets the synced body pose.
    pub fn set_body_pose(&self, pose: Rotations) {
        self.entity_data.lock().body_pose.set(pose);
    }

    /// Returns the synced left arm pose.
    #[must_use]
    pub fn left_arm_pose(&self) -> Rotations {
        *self.entity_data.lock().left_arm_pose.get()
    }

    /// Sets the synced left arm pose.
    pub fn set_left_arm_pose(&self, pose: Rotations) {
        self.entity_data.lock().left_arm_pose.set(pose);
    }

    /// Returns the synced right arm pose.
    #[must_use]
    pub fn right_arm_pose(&self) -> Rotations {
        *self.entity_data.lock().right_arm_pose.get()
    }

    /// Sets the synced right arm pose.
    pub fn set_right_arm_pose(&self, pose: Rotations) {
        self.entity_data.lock().right_arm_pose.set(pose);
    }

    /// Returns the synced left leg pose.
    #[must_use]
    pub fn left_leg_pose(&self) -> Rotations {
        *self.entity_data.lock().left_leg_pose.get()
    }

    /// Sets the synced left leg pose.
    pub fn set_left_leg_pose(&self, pose: Rotations) {
        self.entity_data.lock().left_leg_pose.set(pose);
    }

    /// Returns the synced right leg pose.
    #[must_use]
    pub fn right_leg_pose(&self) -> Rotations {
        *self.entity_data.lock().right_leg_pose.get()
    }

    /// Sets the synced right leg pose.
    pub fn set_right_leg_pose(&self, pose: Rotations) {
        self.entity_data.lock().right_leg_pose.set(pose);
    }

    /// Drops every equipped item at the stand's position, like vanilla
    /// `ArmorStand.dropEquipment`.
    fn drop_equipment(&self) {
        for slot in ARMOR_STAND_SLOTS {
            let stack = self.living_base.equipment().lock().take(slot);
            if !stack.is_empty() {
                self.spawn_at_location(stack, 0.0);
            }
        }
    }

    /// Vanilla `ArmorStand` destruction: play the break sound, drop the
    /// equipment and the stand item, then remove the entity. Armor stands never
    /// drop experience and skip the vanilla 20-tick death animation.
    fn break_apart(&self) {
        if self.broken.swap(true, Ordering::AcqRel) {
            return;
        }

        self.play_sound(&sound_events::ENTITY_ARMOR_STAND_BREAK, 1.0, 1.0);
        self.drop_equipment();
        if let Some(world) = self.level() {
            world.drop_item_stack(
                self.block_position(),
                ItemStack::new(&vanilla_items::ARMOR_STAND),
            );
        }
        self.set_removed(RemovalReason::Killed);
    }

    /// Serializes the vanilla `Pose` compound.
    fn pose_nbt(&self) -> NbtTag {
        let mut pose = NbtCompound::new();
        pose.insert("Head", rotation_list(self.head_pose()));
        pose.insert("Body", rotation_list(self.body_pose()));
        pose.insert("LeftArm", rotation_list(self.left_arm_pose()));
        pose.insert("RightArm", rotation_list(self.right_arm_pose()));
        pose.insert("LeftLeg", rotation_list(self.left_leg_pose()));
        pose.insert("RightLeg", rotation_list(self.right_leg_pose()));
        NbtTag::Compound(pose)
    }

    /// Loads the vanilla `Pose` compound, leaving missing parts untouched.
    fn load_pose(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        let Some(pose) = nbt.compound("Pose") else {
            return;
        };

        let read = |key: &str| {
            pose.list(key)
                .and_then(|list| list.floats())
                .filter(|values| values.len() >= 3)
                .map(|values| Rotations::new(values[0], values[1], values[2]))
        };

        if let Some(rotation) = read("Head") {
            self.set_head_pose(rotation);
        }
        if let Some(rotation) = read("Body") {
            self.set_body_pose(rotation);
        }
        if let Some(rotation) = read("LeftArm") {
            self.set_left_arm_pose(rotation);
        }
        if let Some(rotation) = read("RightArm") {
            self.set_right_arm_pose(rotation);
        }
        if let Some(rotation) = read("LeftLeg") {
            self.set_left_leg_pose(rotation);
        }
        if let Some(rotation) = read("RightLeg") {
            self.set_right_leg_pose(rotation);
        }
    }

    /// Equips the held item into its vanilla target slot, if that slot is free.
    fn interact_with_held_item(&self, player: &Player, hand: InteractionHand) -> InteractionResult {
        let item = {
            let inventory = player.inventory.lock();
            let item = inventory.get_item_in_hand(hand);
            item.copy_with_count(item.count())
        };
        if item.is_empty() || item.is(&vanilla_items::ARMOR_STAND) {
            return InteractionResult::Pass;
        }

        // Vanilla `LivingEntity.getEquipmentSlotForItem`.
        let slot = item
            .get_equippable()
            .map_or(EquipmentSlot::MainHand, |equippable| equippable.slot);
        if !ARMOR_STAND_SLOTS.contains(&slot) {
            return InteractionResult::Pass;
        }
        let slot_occupied = !self.living_base.equipment().lock().get_ref(slot).is_empty();
        if slot_occupied {
            return InteractionResult::Pass;
        }

        let has_infinite_materials = player.has_infinite_materials();
        let equipped = {
            let mut inventory = player.inventory.lock();
            if has_infinite_materials {
                let item = inventory.get_item_in_hand(hand);
                item.copy_with_count(1)
            } else {
                inventory.split_item_in_hand(hand, 1)
            }
        };
        if equipped.is_empty() {
            return InteractionResult::Pass;
        }

        self.living_base
            .equipment()
            .lock()
            .set(slot, equipped.clone());
        self.on_equip_item(slot, &ItemStack::empty(), &equipped);
        player.award_stat(&vanilla_stat_types::ITEM_USED, equipped.item);

        InteractionResult::Success
    }
}

/// Formats one pose as the vanilla three-float rotation list.
fn rotation_list(rotation: Rotations) -> NbtTag {
    NbtTag::List(NbtList::Float(vec![rotation.x, rotation.y, rotation.z]))
}

impl Entity for ArmorStandEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(&self.entity_data)
    }

    fn is_pickable(&self) -> bool {
        !self.is_marker() && !self.is_removed()
    }

    fn is_pushable(&self) -> bool {
        false
    }

    fn is_marker_armor_stand(&self) -> bool {
        self.is_marker()
    }

    fn is_hard_collision_relevant(&self) -> bool {
        false
    }

    fn tick(self: Arc<Self>) {
        let entity: SharedEntity = Arc::<Self>::clone(&self);
        self.tick_living_entity(&entity);
    }

    fn base_tick(&self) {
        LivingEntity::base_tick_living_entity(self);
    }

    fn dimensions_for_pose(&self, _pose: EntityPose) -> EntityDimensions {
        let dimensions = self.entity_type.dimensions;
        if !self.is_small() {
            return dimensions;
        }

        EntityDimensions::new_with_attachments(
            dimensions.width,
            dimensions.height * SMALL_HEIGHT_FACTOR,
            dimensions.eye_height * SMALL_HEIGHT_FACTOR,
            dimensions
                .attachments
                .scale(1.0, SMALL_HEIGHT_FACTOR),
        )
    }

    /// Vanilla `ArmorStand.checkFallDamage`: stands never take fall damage, they
    /// only announce the landing with `entity.armor_stand.fall`.
    fn check_fall_damage(
        self: Arc<Self>,
        vertical_movement: f64,
        on_ground: bool,
        _on_state: BlockStateId,
        _pos: BlockPos,
        world: &Arc<World>,
    ) {
        if !self.is_in_water() && vertical_movement < 0.0 {
            self.base().accumulate_fall_distance(vertical_movement);
        }

        if !on_ground {
            return;
        }

        if self.fall_distance() > 0.0 && !self.is_silent() {
            world.play_sound_at(
                &sound_events::ENTITY_ARMOR_STAND_FALL,
                SoundSource::Neutral,
                self.position(),
                1.0,
                1.0,
                None,
            );
        }
        self.reset_fall_distance();
    }

    fn interact(
        &self,
        player: &Player,
        hand: InteractionHand,
        _location: DVec3,
    ) -> InteractionResult {
        // Vanilla `ArmorStand.interactAt` ignores marker stands entirely.
        if self.is_marker() || self.is_removed() || player.is_spectator() {
            return InteractionResult::Pass;
        }

        self.interact_with_held_item(player, hand)
    }

    fn hurt(&self, world: &World, source: &DamageSource, _amount: f32) -> bool {
        if self.is_removed() || self.is_invulnerable_to(world, source) {
            return false;
        }

        // Vanilla armor stands ignore their health value: any damage destroys
        // the stand immediately, dropping its equipment and itself.
        self.break_apart();
        true
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_equipment(nbt);
        nbt.insert("Invisible", i8::from(self.is_invisible()));
        nbt.insert("Small", i8::from(self.is_small()));
        nbt.insert("ShowArms", i8::from(self.show_arms()));
        nbt.insert("NoBasePlate", i8::from(self.no_base_plate()));
        nbt.insert("Marker", i8::from(self.is_marker()));
        nbt.insert("Pose", self.pose_nbt());
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_equipment(nbt);
        if let Some(invisible) = nbt.byte_value("Invisible") {
            self.set_base_invisible_flag(invisible != 0);
        }
        // The small flag changes the hitbox, so refresh dimensions afterwards.
        self.set_small(nbt.byte_value("Small").is_some_and(|value| value != 0));
        self.set_show_arms(
            nbt.byte_value("ShowArms")
                .is_some_and(|value| value != 0),
        );
        self.set_no_base_plate(
            nbt.byte_value("NoBasePlate")
                .is_some_and(|value| value != 0),
        );
        self.set_marker(nbt.byte_value("Marker").is_some_and(|value| value != 0));
        self.load_pose(nbt);
    }
}

impl LivingEntity for ArmorStandEntity {
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

    fn death_sound(&self) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_ARMOR_STAND_BREAK)
    }

    fn hurt_sound(&self, _source: &DamageSource) -> Option<SoundEventRef> {
        Some(&sound_events::ENTITY_ARMOR_STAND_HIT)
    }

    fn push_entities(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use simdnbt::borrow::read_compound;
    use std::io::Cursor;
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    fn stand() -> ArmorStandEntity {
        init_vanilla_registry();
        ArmorStandEntity::new(&vanilla_entities::ARMOR_STAND, 1, DVec3::ZERO, Weak::new())
    }

    fn reload(nbt: &NbtCompound) -> ArmorStandEntity {
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let borrowed =
            read_compound(&mut Cursor::new(bytes.as_slice())).expect("test NBT should reborrow");
        let stand = stand();
        stand.load_additional((&borrowed).into());
        stand
    }

    #[test]
    fn default_poses_match_vanilla() {
        let stand = stand();

        assert_eq!(stand.head_pose(), Rotations::ZERO);
        assert_eq!(stand.body_pose(), Rotations::ZERO);
        assert_eq!(stand.left_arm_pose(), Rotations::new(-10.0, 0.0, -10.0));
        assert_eq!(stand.right_arm_pose(), Rotations::new(-15.0, 0.0, 10.0));
        assert_eq!(stand.left_leg_pose(), Rotations::new(-1.0, 0.0, -1.0));
        assert_eq!(stand.right_leg_pose(), Rotations::new(1.0, 0.0, 1.0));
    }

    #[test]
    fn small_stands_are_half_height() {
        let stand = stand();
        let normal = stand.dimensions_for_pose(EntityPose::Standing);
        assert!((normal.height - 1.975).abs() < f32::EPSILON);

        stand.set_small(true);
        let small = stand.dimensions_for_pose(EntityPose::Standing);
        assert!((small.width - 0.5).abs() < f32::EPSILON);
        assert!((small.height - 0.9875).abs() < 1.0e-6);
        assert!(stand.base().dimensions().height < normal.height);
    }

    #[test]
    fn flags_can_be_combined_and_cleared() {
        let stand = stand();

        stand.set_small(true);
        stand.set_show_arms(true);
        stand.set_no_base_plate(true);
        stand.set_marker(true);
        assert_eq!(stand.client_flags(), 0x01 | 0x04 | 0x08 | 0x10);

        stand.set_marker(false);
        stand.set_show_arms(false);
        assert_eq!(stand.client_flags(), 0x01 | 0x08);
    }

    #[test]
    fn saving_writes_the_vanilla_keys() {
        let stand = stand();
        stand.set_small(true);
        stand.set_show_arms(true);
        stand.set_marker(true);
        stand.set_head_pose(Rotations::new(10.0, 20.0, 30.0));

        let mut nbt = NbtCompound::new();
        stand.save_additional(&mut nbt);

        assert_eq!(nbt.byte("Small"), Some(1));
        assert_eq!(nbt.byte("ShowArms"), Some(1));
        assert_eq!(nbt.byte("NoBasePlate"), Some(0));
        assert_eq!(nbt.byte("Marker"), Some(1));
        assert_eq!(
            nbt.compound("Pose")
                .and_then(|pose| pose.list("Head"))
                .and_then(|list| list.floats().map(<[f32]>::to_vec)),
            Some(vec![10.0, 20.0, 30.0])
        );
    }

    #[test]
    fn loading_restores_flags_and_poses() {
        let mut nbt = NbtCompound::new();
        nbt.insert("Small", 1_i8);
        nbt.insert("NoBasePlate", 1_i8);
        nbt.insert(
            "Pose",
            {
                let mut pose = NbtCompound::new();
                pose.insert("RightArm", NbtList::Float(vec![1.0, 2.0, 3.0]));
                NbtTag::Compound(pose)
            },
        );

        let stand = reload(&nbt);

        assert!(stand.is_small());
        assert!(stand.no_base_plate());
        assert!(!stand.show_arms());
        assert!(!stand.is_marker());
        assert_eq!(stand.right_arm_pose(), Rotations::new(1.0, 2.0, 3.0));
        // Untouched poses keep their vanilla defaults.
        assert_eq!(stand.left_arm_pose(), Rotations::new(-10.0, 0.0, -10.0));
    }

    #[test]
    fn marker_stands_are_not_pickable_and_not_interactable() {
        let stand = stand();
        assert!(Entity::is_pickable(&stand));
        assert!(!stand.is_marker_armor_stand());

        stand.set_marker(true);
        assert!(!Entity::is_pickable(&stand));
        assert!(stand.is_marker_armor_stand());
    }
}
