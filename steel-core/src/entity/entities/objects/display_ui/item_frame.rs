//! Minimal persistent item-frame entity used by structure generation.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_math::DEGREE_90;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::data_components::vanilla_components::MAP_ID;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::vanilla_entity_data::ItemFrameEntityData;
use steel_registry::{
    sound_events, vanilla_blocks, vanilla_entities, vanilla_game_events, vanilla_items,
};
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtNumeric as _;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::types::InteractionHand;
use steel_utils::{BlockPos, Direction, DowncastType, DowncastTypeKey, WorldAabb, axis::Axis};

use crate::behavior::InteractionResult;
use crate::entity::block_attached_entity::{BlockAttachedEntity, BlockAttachedEntityBase};
use crate::entity::damage::DamageSource;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntityBaseState, EntityMoveError, EntitySyncedData,
    ItemFrame, SharedEntity,
};
use crate::physics::{MoveResult, MoverType};
use crate::player::Player;
use crate::world::game_event::GameEventContext;
use crate::world::{Explosion, World};
use crate::physics::collision;
use steel_registry::items::ItemRef;

/// Item frame state needed by end-city structure markers.
///
/// This intentionally implements only placement, synced item/facing data,
/// persistence, and comparator integration.
///
/// TODO: Add interaction, drops, map tracking, and support checks.
#[entity_behavior(class = "ItemFrame")]
pub struct ItemFrameEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<ItemFrameEntityData>,
    block_attached_entity_base: BlockAttachedEntityBase,
    /// Set once the frame has dropped its contents, so a second removal path
    /// (hurt + survival tick) cannot duplicate the drops.
    dropped: AtomicBool,
}

// SAFETY: This key is owned by Steel and uniquely identifies `ItemFrameEntity`.
unsafe impl DowncastType for ItemFrameEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/item_frame");
}

impl ItemFrameEntity {
    /// Creates a fresh item frame from the generic entity factory path.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self::new_attached(
            entity_type,
            id,
            BlockPos::new(
                position.x.floor() as i32,
                position.y.floor() as i32,
                position.z.floor() as i32,
            ),
            Direction::South,
            world,
        )
    }

    /// Creates a fresh item frame attached to `block_pos`.
    #[must_use]
    pub fn new_attached(
        entity_type: EntityTypeRef,
        id: i32,
        block_pos: BlockPos,
        direction: Direction,
        world: Weak<World>,
    ) -> Self {
        let entity = Self {
            base: EntityBase::new_with_state(
                id,
                EntityBaseState::new_with_bounding_box(
                    Self::frame_center(block_pos, direction),
                    entity_type.dimensions,
                    Self::frame_bounding_box(block_pos, direction, false),
                )
                .with_rotation(Self::rotation_for_direction(direction)),
                world,
            ),
            entity_type,
            entity_data: SyncMutex::new(ItemFrameEntityData::new()),
            block_attached_entity_base: BlockAttachedEntityBase::new(block_pos),
            dropped: AtomicBool::new(false),
        };
        entity
            .entity_data
            .lock()
            .hanging_entity
            .direction
            .set(direction);
        entity
    }

    /// Creates an item frame from persistent entity data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        let position = load.position;
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(ItemFrameEntityData::new()),
            block_attached_entity_base: BlockAttachedEntityBase::new(BlockPos::new(
                position.x.floor() as i32,
                position.y.floor() as i32,
                position.z.floor() as i32,
            )),
            dropped: AtomicBool::new(false),
        }
    }

    /// Sets the framed item, matching vanilla by storing a single item.
    pub fn set_item(&self, item: ItemStack) {
        self.set_item_with_update(item, true);
    }

    /// Sets the framed item and optionally notifies nearby comparators.
    pub(crate) fn set_item_with_update(&self, mut item: ItemStack, update_comparators: bool) {
        if !item.is_empty() {
            item.set_count(1);
        }
        self.entity_data.lock().item.set(item);
        self.recalculate_position_or_warn();
        if update_comparators && let Some(world) = self.level() {
            world.update_neighbor_for_output_signal(
                self.block_attached_entity_base.pos(),
                &vanilla_blocks::AIR,
            );
        }
    }

    /// Number of rotations a framed item cycles through (vanilla `ROTATION_COUNT`).
    const ROTATION_COUNT: i32 = 8;

    /// Vanilla `ItemFrameEntity#canStayAttached`.
    ///
    /// The frame needs free space for its own volume and a solid support block
    /// directly behind it.
    #[must_use]
    pub fn can_stay_attached(&self) -> bool {
        let Some(world) = self.base.level() else {
            return false;
        };

        let check_box = self.base.bounding_box();
        let collision = crate::physics::collision::WorldCollisionProvider::new(&world);
        if collision::has_block_collision(&collision, check_box) {
            return false;
        }
        // Vanilla `isSpaceBlocked` rejects any other entity inside the frame's own
        // box, which also covers `hasNoIntersectingDecoration` for frames.
        if world.has_entity_in_aabb_matching(&check_box, |entity| entity.id() != self.base.id()) {
            return false;
        }

        let (dx, dy, dz) = self.direction().opposite().offset();
        let support_pos = self.block_attached_entity_base.pos().offset(dx, dy, dz);
        world.get_block_state(support_pos).is_solid()
    }

    /// Returns whether this is a glow item frame.
    fn is_glow(&self) -> bool {
        self.entity_type == &vanilla_entities::GLOW_ITEM_FRAME
    }

    /// Sound played when the frame is broken.
    fn break_sound(&self) -> SoundEventRef {
        if self.is_glow() {
            &sound_events::ENTITY_GLOW_ITEM_FRAME_BREAK
        } else {
            &sound_events::ENTITY_ITEM_FRAME_BREAK
        }
    }

    /// Sound played when the framed item is rotated.
    fn rotate_item_sound(&self) -> SoundEventRef {
        if self.is_glow() {
            &sound_events::ENTITY_GLOW_ITEM_FRAME_ROTATE_ITEM
        } else {
            &sound_events::ENTITY_ITEM_FRAME_ROTATE_ITEM
        }
    }

    /// Sound played when an item is inserted into the frame.
    fn add_item_sound(&self) -> SoundEventRef {
        if self.is_glow() {
            &sound_events::ENTITY_GLOW_ITEM_FRAME_ADD_ITEM
        } else {
            &sound_events::ENTITY_ITEM_FRAME_ADD_ITEM
        }
    }

    /// Sound played when the frame is placed.
    pub fn place_sound(&self) -> SoundEventRef {
        if self.is_glow() {
            &sound_events::ENTITY_GLOW_ITEM_FRAME_PLACE
        } else {
            &sound_events::ENTITY_ITEM_FRAME_PLACE
        }
    }

    /// Returns the item that places this frame (vanilla `getAsItemStack`).
    #[must_use]
    pub fn placeable_item(&self) -> ItemRef {
        if self.is_glow() {
            &vanilla_items::GLOW_ITEM_FRAME
        } else {
            &vanilla_items::ITEM_FRAME
        }
    }

    /// Rotation of the framed item in 45° steps (vanilla `getRotation`).
    #[must_use]
    pub fn rotation(&self) -> i32 {
        *self.entity_data.lock().rotation.get()
    }

    /// The framed item, if any (vanilla `getHeldItemStack`).
    #[must_use]
    pub fn item(&self) -> ItemStack {
        self.entity_data.lock().item.get().clone()
    }

    /// Rotates the framed item by 45° (vanilla `setRotation(getRotation() + 1)`).
    pub fn rotate(&self) {
        let rotation = {
            let entity_data = self.entity_data.lock();
            (*entity_data.rotation.get() + 1).rem_euclid(Self::ROTATION_COUNT)
        };
        self.entity_data.lock().rotation.set(rotation);

        if let Some(world) = self.base.level() {
            world.update_neighbor_for_output_signal(
                self.block_attached_entity_base.pos(),
                &vanilla_blocks::AIR,
            );
        }
    }

    /// Emits the vanilla `BLOCK_CHANGE` game event for the attached block.
    fn emit_block_change(&self, player: Option<&dyn Entity>) {
        let Some(world) = self.base.level() else {
            return;
        };
        world.game_event(
            &vanilla_game_events::BLOCK_CHANGE,
            self.block_attached_entity_base.pos(),
            &GameEventContext::new(player, None),
        );
    }

    /// Vanilla `ItemFrameEntity#interact`: rotate a filled frame, otherwise insert
    /// the held item and consume one of it.
    fn interact_with_player(&self, player: &Player, hand: InteractionHand) -> InteractionResult {
        let held = player.inventory.lock().get_item_in_hand(hand).clone();
        let has_framed_item = !self.entity_data.lock().item.get().is_empty();

        if has_framed_item {
            self.play_sound(self.rotate_item_sound(), 1.0, 1.0);
            self.rotate();
            self.emit_block_change(Some(player));
            return InteractionResult::Success;
        }

        if held.is_empty() || self.is_removed() {
            return InteractionResult::Pass;
        }

        self.set_item(held);
        self.play_sound(self.add_item_sound(), 1.0, 1.0);
        self.emit_block_change(Some(player));

        let infinite = player.has_infinite_materials();
        player
            .inventory
            .lock()
            .mutate_item_in_hand(hand, |stack| stack.consume_one(infinite));

        InteractionResult::Success
    }

    fn set_direction(&self, direction: Direction) {
        self.entity_data
            .lock()
            .hanging_entity
            .direction
            .set(direction);
        self.base
            .set_rotation(Self::rotation_for_direction(direction));
        self.recalculate_position_or_warn();
    }

    fn try_recalculate_position(&self) -> Result<(), EntityMoveError> {
        let block_pos = self.block_attached_entity_base.pos();
        let direction = *self.entity_data.lock().hanging_entity.direction.get();
        let position = Self::frame_center(block_pos, direction);
        self.base.try_set_position(position)?;
        self.base.set_bounding_box(Self::frame_bounding_box(
            block_pos,
            direction,
            self.has_framed_map(),
        ));

        Ok(())
    }

    fn recalculate_position_or_warn(&self) {
        if let Err(error) = self.try_recalculate_position() {
            log::warn!(
                "failed to commit item frame {} position recalculation: {error}",
                self.base.id()
            );
        }
    }

    fn has_framed_map(&self) -> bool {
        self.entity_data.lock().item.get().has(MAP_ID)
    }

    fn frame_center(block_pos: BlockPos, direction: Direction) -> DVec3 {
        let off = direction.offset_vec().as_dvec3() * 0.46875;
        block_pos.0.as_dvec3() + DVec3::splat(0.5) - off
    }

    fn rotation_for_direction(direction: Direction) -> (f32, f32) {
        if direction.is_horizontal() {
            (
                f32::from(direction_2d_data_value(direction)) * DEGREE_90,
                0.0,
            )
        } else {
            let pitch = match direction {
                Direction::Up => -DEGREE_90,
                Direction::Down => DEGREE_90,
                Direction::North | Direction::South | Direction::West | Direction::East => 0.0,
            };
            (0.0, pitch)
        }
    }

    fn frame_bounding_box(
        block_pos: BlockPos,
        direction: Direction,
        has_framed_map: bool,
    ) -> WorldAabb {
        let center = Self::frame_center(block_pos, direction);
        let size = if has_framed_map { 1.0 } else { 0.75 };
        let x_size = if direction.axis() == Axis::X {
            0.0625
        } else {
            size
        };
        let y_size = if direction.axis() == Axis::Y {
            0.0625
        } else {
            size
        };
        let z_size = if direction.axis() == Axis::Z {
            0.0625
        } else {
            size
        };
        WorldAabb::new(
            center.x - x_size / 2.0,
            center.y - y_size / 2.0,
            center.z - z_size / 2.0,
            center.x + x_size / 2.0,
            center.y + y_size / 2.0,
            center.z + z_size / 2.0,
        )
    }
}

impl ItemFrame for ItemFrameEntity {
    fn direction(&self) -> Direction {
        *self.entity_data.lock().hanging_entity.direction.get()
    }

    fn analog_output(&self) -> i32 {
        let entity_data = self.entity_data.lock();
        if entity_data.item.get().is_empty() {
            0
        } else {
            *entity_data.rotation.get() % 8 + 1
        }
    }
}

impl Entity for ItemFrameEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn spawn_data(&self) -> i32 {
        direction_3d_data_value(*self.entity_data.lock().hanging_entity.direction.get())
    }

    fn spawn_position(&self) -> DVec3 {
        let block_pos = self.block_attached_entity_base.pos();
        DVec3::new(
            f64::from(block_pos.x()),
            f64::from(block_pos.y()),
            f64::from(block_pos.z()),
        )
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(&self.entity_data)
    }

    fn tick(self: Arc<Self>) {
        self.tick_block_attached_entity();
    }

    fn is_pickable(&self) -> bool {
        self.is_pickable_block_attached_entity()
    }

    fn interact(
        &self,
        player: &Player,
        hand: InteractionHand,
        _location: DVec3,
    ) -> InteractionResult {
        self.interact_with_player(player, hand)
    }

    fn ignore_explosion(&self, explosion: &dyn Explosion) -> bool {
        explosion
            .direct_source_entity()
            .is_some_and(Entity::is_in_water)
            || !explosion.should_affect_blocklike_entities()
    }

    fn hurt(&self, world: &World, source: &DamageSource, amount: f32) -> bool {
        self.hurt_block_attached_entity(world, source, amount)
    }

    fn skip_attack_interaction(&self, source: &SharedEntity) -> bool {
        self.skip_attack_interaction_block_attached_entity(source)
    }

    fn push_impulse(&self, impulse: DVec3) {
        self.push_impulse_block_attached_entity(impulse);
    }

    fn move_entity(self: Arc<Self>, mover_type: MoverType, delta: DVec3) -> Option<MoveResult> {
        self.move_entity_block_attached_entity(mover_type, delta)
    }

    fn refresh_dimensions(&self) {
        self.refresh_dimensions_block_attached_entity();
    }

    fn try_set_position(&self, pos: DVec3) -> Result<(), EntityMoveError> {
        self.try_set_position_block_attached_entity(pos)
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_block_attached_entity(nbt);

        let entity_data = self.entity_data.lock();
        let item = entity_data.item.get();
        if !item.is_empty() {
            nbt.insert("Item", item.to_nbt_tag_ref());
        }
        nbt.insert("ItemRotation", *entity_data.rotation.get() as i8);
        nbt.insert("ItemDropChance", 1.0_f32);
        nbt.insert(
            "Facing",
            direction_3d_data_value(*entity_data.hanging_entity.direction.get()) as i8,
        );
        nbt.insert("Invisible", 0_i8);
        nbt.insert("Fixed", 0_i8);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_block_attached_entity(nbt);

        if let Some(item_tag) = nbt.compound("Item")
            && let Some(item) = ItemStack::from_borrowed_compound(&item_tag)
        {
            self.set_item_with_update(item, false);
        }

        if let Some(item_rotation) = nbt.byte_value("ItemRotation") {
            self.entity_data
                .lock()
                .rotation
                .set(i32::from(item_rotation) % 8);
        }

        let facing = nbt
            .get("Facing")
            .and_then(|tag| tag.codec_i8())
            .map_or(Direction::Down, |value| {
                Direction::from_3d_data_value(i32::from(value))
            });
        self.set_direction(facing);

        self.recalculate_position_or_warn();
    }
}

impl BlockAttachedEntity for ItemFrameEntity {
    fn block_attached_entity_base(&self) -> &BlockAttachedEntityBase {
        &self.block_attached_entity_base
    }

    fn survives(&self) -> bool {
        self.can_stay_attached()
    }

    fn drop_item(&self, _caused_by: Option<&dyn Entity>) {
        if self.dropped.swap(true, Ordering::AcqRel) {
            return;
        }

        let Some(world) = self.base.level() else {
            return;
        };

        self.play_sound(self.break_sound(), 1.0, 1.0);

        let pos = self.block_attached_entity_base.pos();
        let framed = {
            let entity_data = self.entity_data.lock();
            entity_data.item.get().clone()
        };
        if !framed.is_empty() {
            // The frame is going away: skip the comparator notification.
            self.set_item_with_update(ItemStack::empty(), false);
            world.drop_item_stack(pos, framed);
        }

        world.drop_item_stack(pos, ItemStack::new(self.placeable_item()));
    }

    fn recalculate_bounding_box(&self) -> Result<(), EntityMoveError> {
        self.try_recalculate_position()
    }
}

const fn direction_3d_data_value(direction: Direction) -> i32 {
    match direction {
        Direction::Down => 0,
        Direction::Up => 1,
        Direction::North => 2,
        Direction::South => 3,
        Direction::West => 4,
        Direction::East => 5,
    }
}

const fn direction_2d_data_value(direction: Direction) -> u8 {
    match direction {
        Direction::South | Direction::Down | Direction::Up => 0,
        Direction::West => 1,
        Direction::North => 2,
        Direction::East => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simdnbt::borrow::read_compound;
    use std::io::Cursor;
    use std::string::ToString;
    use steel_registry::{vanilla_entities, vanilla_items};

    fn frame_loaded_from(nbt: &NbtCompound) -> ItemFrameEntity {
        let frame = ItemFrameEntity::new_attached(
            &vanilla_entities::ITEM_FRAME,
            1,
            BlockPos::new(12, 80, 14),
            Direction::West,
            Weak::new(),
        );
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let borrowed =
            read_compound(&mut Cursor::new(bytes.as_slice())).expect("test NBT should reborrow");
        frame.load_additional((&borrowed).into());
        frame
    }

    // --- Placement / support / interaction tests -------------------------------

    use std::sync::Arc;

    use steel_registry::init_vanilla_registry;
    use steel_utils::types::{InteractionHand, UpdateFlags};
    use steel_utils::{ChunkPos, WorldAabb};

    use steel_utils::Downcast as _;

    use crate::behavior::InteractionResult;
    use crate::entity::block_attached_entity::CHECK_SURVIVAL_FREQUENCY_TICKS;
    use crate::entity::entities::ItemEntity;
    use crate::test_support::{
        TestPlayerBuilder, TestWorld, fresh_test_world, insert_ready_full_chunk,
    };
    use crate::entity;

    fn frame_area() -> WorldAabb {
        WorldAabb::new(-2.0, 62.0, -2.0, 2.0, 68.0, 2.0)
    }

    /// Builds a world containing a stone support block at `(0, 64, 0)`.
    fn support_fixture(name: &str) -> (TestWorld, Arc<Player>, BlockPos) {
        init_vanilla_registry();

        let world_fixture = fresh_test_world(name);
        let world = Arc::clone(&world_fixture.world);
        let support = BlockPos::new(0, 64, 0);
        insert_ready_full_chunk(&world, ChunkPos::from_block_pos(support));
        assert!(world.set_block(
            support,
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        let player = TestPlayerBuilder::new(Arc::clone(&world), "FrameOwner", 1).build();
        player.set_client_loaded(true);

        (world_fixture, player, support)
    }

    fn spawn_frame(
        world: &Arc<World>,
        block_pos: BlockPos,
        direction: Direction,
    ) -> Arc<ItemFrameEntity> {
        let frame = Arc::new(ItemFrameEntity::new_attached(
            &vanilla_entities::ITEM_FRAME,
            entity::next_entity_id(),
            block_pos,
            direction,
            Arc::downgrade(world),
        ));
        let shared = Arc::clone(&frame) as SharedEntity;
        world
            .try_add_entity(shared)
            .expect("frame should enter a loaded chunk");
        frame
    }

    fn item_entity_count(world: &Arc<World>) -> usize {
        world
            .get_entities_in_aabb(&frame_area())
            .iter()
            .filter(|entity| entity.downcast_ref::<ItemEntity>().is_some())
            .count()
    }

    #[test]
    fn frame_survives_only_with_a_solid_support_behind_it() {
        let (fixture, _player, support) = support_fixture("frame_support");
        let world = &fixture.world;

        let attached = ItemFrameEntity::new_attached(
            &vanilla_entities::ITEM_FRAME,
            1,
            support.offset(0, 1, 0),
            Direction::Up,
            Arc::downgrade(world),
        );
        assert!(attached.can_stay_attached());

        let floating = ItemFrameEntity::new_attached(
            &vanilla_entities::ITEM_FRAME,
            2,
            support.offset(0, 4, 0),
            Direction::Up,
            Arc::downgrade(world),
        );
        assert!(!floating.can_stay_attached());
    }

    #[test]
    fn broken_support_removes_and_drops_the_frame() {
        let (fixture, player, support) = support_fixture("frame_support_removed");
        let world = &fixture.world;
        let frame = spawn_frame(world, support.offset(0, 1, 0), Direction::Up);

        assert!(world.set_block(
            support,
            vanilla_blocks::AIR.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        // Vanilla only re-checks attachment every `CHECK_SURVIVAL_FREQUENCY_TICKS`.
        for _ in 0..=CHECK_SURVIVAL_FREQUENCY_TICKS {
            Arc::clone(&frame).tick();
        }

        assert!(frame.is_removed());
        assert_eq!(item_entity_count(world), 1, "the frame item should drop");
        let _ = player;
    }

    #[test]
    fn clicking_a_filled_frame_rotates_its_item() {
        let (fixture, player, support) = support_fixture("frame_rotate");
        let world = &fixture.world;
        let frame = spawn_frame(world, support.offset(0, 1, 0), Direction::Up);
        frame.set_item(ItemStack::new(&vanilla_items::APPLE));

        let result = frame.interact(&player, InteractionHand::MainHand, DVec3::ZERO);
        assert_eq!(result, InteractionResult::Success);
        assert_eq!(frame.rotation(), 1);
        assert!(frame.item().is(&vanilla_items::APPLE));
    }

    #[test]
    fn clicking_an_empty_frame_inserts_the_held_item_and_consumes_one() {
        let (fixture, player, support) = support_fixture("frame_insert");
        let world = &fixture.world;
        let frame = spawn_frame(world, support.offset(0, 1, 0), Direction::Up);
        player
            .inventory
            .lock()
            .set_selected_item(ItemStack::with_count(&vanilla_items::APPLE, 2));

        let result = frame.interact(&player, InteractionHand::MainHand, DVec3::ZERO);

        assert_eq!(result, InteractionResult::Success);
        assert!(frame.item().is(&vanilla_items::APPLE));
        assert_eq!(frame.item().count(), 1);
        assert_eq!(player.inventory.lock().get_selected_item().count(), 1);
    }

    #[test]
    fn clicking_an_empty_frame_with_an_empty_hand_passes() {
        let (fixture, player, support) = support_fixture("frame_empty_hand");
        let world = &fixture.world;
        let frame = spawn_frame(world, support.offset(0, 1, 0), Direction::Up);

        assert_eq!(
            frame.interact(&player, InteractionHand::MainHand, DVec3::ZERO),
            InteractionResult::Pass
        );
        assert!(frame.item().is_empty());
    }

    #[test]
    fn dropping_the_frame_twice_does_not_duplicate_items() {
        let (fixture, _player, support) = support_fixture("frame_drop_once");
        let world = &fixture.world;
        let frame = spawn_frame(world, support.offset(0, 1, 0), Direction::Up);
        frame.set_item(ItemStack::new(&vanilla_items::DIAMOND));

        frame.drop_item(None);
        frame.drop_item(None);

        assert_eq!(item_entity_count(world), 2);
    }

    #[test]
    fn removing_a_filled_frame_drops_the_item_and_the_frame() {
        let (fixture, _player, support) = support_fixture("frame_drop");
        let world = &fixture.world;
        let frame = spawn_frame(world, support.offset(0, 1, 0), Direction::Up);
        frame.set_item(ItemStack::new(&vanilla_items::DIAMOND));

        frame.drop_item(None);

        assert!(frame.item().is_empty());
        assert_eq!(item_entity_count(world), 2);
    }

    #[test]
    fn load_wraps_facing_and_keeps_rotation_sign_like_java() {
        let mut nbt = NbtCompound::new();
        nbt.insert("Facing", 7_i8);
        nbt.insert("ItemRotation", -1_i8);
        let frame = frame_loaded_from(&nbt);

        let entity_data = frame.entity_data.lock();
        assert_eq!(*entity_data.hanging_entity.direction.get(), Direction::Up);
        assert_eq!(*entity_data.rotation.get(), -1);
    }

    #[test]
    fn load_defaults_missing_facing_to_down() {
        let frame = frame_loaded_from(&NbtCompound::new());

        let entity_data = frame.entity_data.lock();
        assert_eq!(*entity_data.hanging_entity.direction.get(), Direction::Down);
    }

    #[test]
    fn item_frame_persists_structure_marker_state() {
        let frame = ItemFrameEntity::new_attached(
            &vanilla_entities::ITEM_FRAME,
            1,
            BlockPos::new(12, 80, 14),
            Direction::West,
            Weak::new(),
        );
        frame.set_item(ItemStack::new(&vanilla_items::ELYTRA));

        let mut nbt = NbtCompound::new();
        frame.save_additional(&mut nbt);

        assert_eq!(nbt.byte("Facing"), Some(4));
        assert_eq!(nbt.byte("ItemRotation"), Some(0));
        assert_eq!(nbt.float("ItemDropChance"), Some(1.0));
        assert_eq!(nbt.byte("Invisible"), Some(0));
        assert_eq!(nbt.byte("Fixed"), Some(0));
        let Some(item) = nbt.compound("Item") else {
            panic!("item frame should save framed item");
        };
        assert_eq!(
            item.string("id").map(ToString::to_string),
            Some("minecraft:elytra".to_owned())
        );
        assert_eq!(item.int("count"), Some(1));
    }

    #[test]
    fn item_frame_is_pickable_like_vanilla() {
        let frame = ItemFrameEntity::new_attached(
            &vanilla_entities::ITEM_FRAME,
            1,
            BlockPos::new(12, 80, 14),
            Direction::West,
            Weak::new(),
        );

        assert!(frame.is_pickable());
    }

    #[test]
    fn analog_output_uses_item_presence_and_rotation() {
        let frame = ItemFrameEntity::new_attached(
            &vanilla_entities::ITEM_FRAME,
            1,
            BlockPos::new(12, 80, 14),
            Direction::West,
            Weak::new(),
        );
        assert_eq!(frame.analog_output(), 0);

        frame.set_item_with_update(ItemStack::new(&vanilla_items::ELYTRA), false);
        assert_eq!(frame.analog_output(), 1);
        frame.entity_data.lock().rotation.set(7);
        assert_eq!(frame.analog_output(), 8);
    }
}
