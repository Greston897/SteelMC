//! Painting entity (`Painting`).
//!
//! Ports vanilla `PaintingEntity` / `AbstractDecorationEntity`: the painting is a
//! block-attached decoration whose bounding box is the variant's
//! `width × height` block area, one pixel thick against the wall. Placement uses
//! vanilla `placePainting`: every variant from the `placeable` tag is tried,
//! variants that do not fit are dropped, and among the variants with the largest
//! area one is chosen at random.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::painting_variant::{PaintingVariant, PaintingVariantRef};
use steel_registry::vanilla_entity_data::PaintingEntityData;
use steel_registry::{
    REGISTRY, RegistryExt as _, RegistryReference, TaggedRegistryExt as _, sound_events,
    vanilla_items,
};
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtNumeric as _;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::{BlockPos, Direction, DowncastType, DowncastTypeKey, WorldAabb};

use crate::behavior::InteractionResult;
use crate::entity::block_attached_entity::{BlockAttachedEntity, BlockAttachedEntityBase};
use crate::entity::damage::DamageSource;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntityBaseState, EntityMoveError, EntitySyncedData,
    SharedEntity,
};
use crate::player::Player;
use crate::world::World;

/// Vanilla `PaintingEntity.PIXEL_SIZE`: the thickness of a painting against the wall.
pub const PIXEL_SIZE: f64 = 0.0625;

/// Vanilla `PaintingEntity.PAINTING_DEPTH_OFFSET`: distance from the block centre
/// to the painting's wall-facing plane.
pub const PAINTING_DEPTH_OFFSET: f64 = 0.46875;

/// A vanilla painting: a block-attached decoration of a randomly chosen variant.
#[entity_behavior(class = "Painting")]
pub struct PaintingEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<PaintingEntityData>,
    block_attached_entity_base: BlockAttachedEntityBase,
    facing: SyncMutex<Direction>,
    /// Set once the painting dropped its item, so two removal paths cannot duplicate it.
    dropped: AtomicBool,
}

// SAFETY: This key is owned by Steel and uniquely identifies `PaintingEntity`.
unsafe impl DowncastType for PaintingEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/painting");
}

impl PaintingEntity {
    /// Creates a fresh painting from the generic entity factory path.
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
            Self::default_variant(),
            world,
        )
    }

    /// Creates a painting attached to `block_pos` with the given variant.
    #[must_use]
    pub fn new_attached(
        entity_type: EntityTypeRef,
        id: i32,
        block_pos: BlockPos,
        facing: Direction,
        variant: RegistryReference<PaintingVariant>,
        world: Weak<World>,
    ) -> Self {
        let bounding_box = Self::painting_box(block_pos, facing, variant.value());
        let entity = Self {
            base: EntityBase::new_with_state(
                id,
                EntityBaseState::new_with_bounding_box(
                    Self::painting_center(block_pos, facing),
                    entity_type.dimensions,
                    bounding_box,
                )
                .with_rotation(Self::rotation_for_direction(facing)),
                world,
            ),
            entity_type,
            entity_data: SyncMutex::new(PaintingEntityData::new()),
            block_attached_entity_base: BlockAttachedEntityBase::new(block_pos),
            facing: SyncMutex::new(facing),
            dropped: AtomicBool::new(false),
        };
        entity.set_variant(variant);
        entity
    }

    /// Creates a painting from persistent entity data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        let position = load.position;
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(PaintingEntityData::new()),
            block_attached_entity_base: BlockAttachedEntityBase::new(BlockPos::new(
                position.x.floor() as i32,
                position.y.floor() as i32,
                position.z.floor() as i32,
            )),
            facing: SyncMutex::new(Direction::South),
            dropped: AtomicBool::new(false),
        }
    }

    /// Vanilla `Variants.getDefaultOrThrow`: the `kebab` variant is vanilla's default.
    fn default_variant() -> RegistryReference<PaintingVariant> {
        RegistryReference::new(&steel_registry::vanilla_painting_variants::KEBAB)
    }

    /// The currently synced variant.
    #[must_use]
    pub fn variant(&self) -> RegistryReference<PaintingVariant> {
        *self.entity_data.lock().painting().painting_variant.get()
    }

    /// Sets the synced variant.
    pub fn set_variant(&self, variant: RegistryReference<PaintingVariant>) {
        self.entity_data.lock().painting_mut().painting_variant.set(variant);
    }

    /// The direction the painting faces (away from its wall).
    #[must_use]
    pub fn facing(&self) -> Direction {
        *self.facing.lock()
    }

    /// Sets the facing direction, marking nothing dirty (facing is not synced in Steel).
    pub fn set_facing(&self, facing: Direction) {
        *self.facing.lock() = facing;
    }

    /// Vanilla `BlockPos.ofCenter(pos).offset(side, -PAINTING_DEPTH_OFFSET)`.
    fn wall_center(block_pos: BlockPos, facing: Direction) -> DVec3 {
        let offset = facing.offset_vec().as_dvec3() * PAINTING_DEPTH_OFFSET;
        DVec3::new(
            f64::from(block_pos.x()) + 0.5,
            f64::from(block_pos.y()) + 0.5,
            f64::from(block_pos.z()) + 0.5,
        ) - offset
    }

    /// Vanilla `getOffset`: even lengths are centred on a block edge.
    const fn offset_for(length: i32) -> f64 {
        if length % 2 == 0 { 0.5 } else { 0.0 }
    }

    /// Vanilla `PaintingEntity#calculateBoundingBox`.
    #[must_use]
    pub fn painting_box(
        block_pos: BlockPos,
        facing: Direction,
        variant: &PaintingVariant,
    ) -> WorldAabb {
        let value = variant.value();
        let center = Self::wall_center(block_pos, facing);
        let perpendicular = facing.rotate_y_counter_clockwise();
        let origin = center
            + perpendicular.offset_vec().as_dvec3() * Self::offset_for(value.width)
            + DVec3::new(0.0, Self::offset_for(value.height), 0.0);

        let size_x = if facing.get_axis() == steel_utils::axis::Axis::X {
            PIXEL_SIZE
        } else {
            f64::from(value.width)
        };
        let size_z = if facing.get_axis() == steel_utils::axis::Axis::Z {
            PIXEL_SIZE
        } else {
            f64::from(value.width)
        };

        WorldAabb::of_size(origin, size_x, f64::from(value.height), size_z)
    }

    /// The entity position for a painting attached at `block_pos`.
    fn painting_center(block_pos: BlockPos, facing: Direction) -> DVec3 {
        Self::wall_center(block_pos, facing)
    }

    fn rotation_for_direction(direction: Direction) -> (f32, f32) {
        let yaw = match direction {
            Direction::South => 0.0,
            Direction::West => 90.0,
            Direction::North => 180.0,
            Direction::East => 270.0,
            Direction::Up | Direction::Down => 0.0,
        };
        (yaw, 0.0)
    }

    /// Vanilla `Variants.getRandom`, limited to the `placeable` tag, then filtered
    /// exactly like `PaintingEntity.placePainting`.
    #[must_use]
    pub fn place_painting(
        entity_type: EntityTypeRef,
        id: i32,
        block_pos: BlockPos,
        facing: Direction,
        world: Weak<World>,
    ) -> Option<Self> {
        let mut candidates: Vec<PaintingVariantRef> = REGISTRY
            .painting_variants
            .iter_tag(&steel_registry::vanilla_painting_variant_tags::PaintingVariantTag::PLACEABLE)
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Drop variants that do not fit on the wall.
        candidates.retain(|variant| {
            let painting = Self::new_attached(
                entity_type,
                id,
                block_pos,
                facing,
                RegistryReference::new(*variant),
                world.clone(),
            );
            painting.can_stay_attached()
        });

        if candidates.is_empty() {
            return None;
        }

        // Vanilla keeps only the variants with the largest area.
        let max_area = candidates
            .iter()
            .map(|variant| variant.value().width * variant.value().height)
            .max()
            .unwrap_or(0);
        candidates.retain(|variant| variant.value().width * variant.value().height == max_area);

        let index = rand::random_range(0..candidates.len());
        let chosen = candidates[index];
        Some(Self::new_attached(
            entity_type,
            id,
            block_pos,
            facing,
            RegistryReference::new(chosen),
            world,
        ))
    }

    /// Vanilla `AbstractDecorationEntity#getAttachmentBox`, expressed as the blocks
    /// behind the painting.
    fn attachment_blocks(&self) -> Vec<BlockPos> {
        let box_ = self
            .base
            .bounding_box()
            .translate(self.facing().offset_vec().as_dvec3() * -0.5)
            .contract(DVec3::splat(1.0e-7));

        let mut blocks = Vec::new();
        crate::entity::block_effects::for_each_block_in_aabb(box_, |pos| {
            blocks.push(pos);
            true
        });
        blocks
    }

    /// Vanilla `AbstractDecorationEntity#canStayAttached`.
    #[must_use]
    pub fn can_stay_attached(&self) -> bool {
        let Some(world) = self.base.level() else {
            return false;
        };

        let check_box = self.base.bounding_box();
        let collision = crate::physics::collision::WorldCollisionProvider::new(&world);
        if crate::physics::collision::has_block_collision(&collision, check_box) {
            return false;
        }
        if world.has_entity_in_aabb_matching(&check_box, |entity| entity.id() != self.base.id()) {
            return false;
        }

        self.attachment_blocks()
            .into_iter()
            .all(|pos| world.get_block_state(pos).is_solid())
    }

    /// The item this painting drops.
    #[must_use]
    pub fn as_item_stack(&self) -> ItemStack {
        ItemStack::new(&vanilla_items::PAINTING)
    }
}

impl Entity for PaintingEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn spawn_data(&self) -> i32 {
        self.facing().get_3d_data_value()
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
        _player: &Player,
        _hand: steel_utils::types::InteractionHand,
        _location: DVec3,
    ) -> InteractionResult {
        InteractionResult::Pass
    }

    fn ignore_explosion(&self, explosion: &dyn crate::world::Explosion) -> bool {
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

    fn move_entity(
        self: Arc<Self>,
        mover_type: crate::physics::MoverType,
        delta: DVec3,
    ) -> Option<crate::physics::MoveResult> {
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
        nbt.insert("facing", self.facing().get_3d_data_value() as i8);
        nbt.insert("variant", self.variant().value().key.to_string());
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_block_attached_entity(nbt);

        if let Some(facing) = nbt.get("facing").and_then(|tag| tag.codec_i8()) {
            self.set_facing(Direction::from_3d_data_value(i32::from(facing)));
        }

        if let Some(key) = nbt.string("variant")
            && let Ok(identifier) = key.to_str().parse::<steel_utils::Identifier>()
        {
            if let Some(variant) = REGISTRY.painting_variants.by_key(&identifier) {
                self.set_variant(RegistryReference::new(variant));
            }
        } else if let Some(variant) = REGISTRY.painting_variants.by_id(0) {
            self.set_variant(RegistryReference::new(variant));
        }

        self.base
            .set_bounding_box(Self::painting_box(
                self.block_attached_entity_base.pos(),
                self.facing(),
                self.variant().value(),
            ));
    }
}

impl BlockAttachedEntity for PaintingEntity {
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

        self.play_sound(&sound_events::ENTITY_PAINTING_BREAK, 1.0, 1.0);
        world.drop_item_stack(self.block_attached_entity_base.pos(), self.as_item_stack());
    }

    fn recalculate_bounding_box(&self) -> Result<(), EntityMoveError> {
        self.base.set_bounding_box(Self::painting_box(
            self.block_attached_entity_base.pos(),
            self.facing(),
            self.variant().value(),
        ));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use simdnbt::borrow::read_compound;
    use simdnbt::owned::NbtCompound;
    use std::io::Cursor;
    use steel_registry::item_stack::ItemStack;
    use steel_registry::painting_variant::PaintingVariant;
    use steel_registry::vanilla_painting_variants;
    use steel_registry::{REGISTRY, RegistryExt as _, RegistryReference, init_vanilla_registry};
    use steel_registry::vanilla_blocks;
    use steel_registry::vanilla_entities;
    use steel_registry::vanilla_items;
    use steel_utils::types::UpdateFlags;
    use steel_utils::{BlockPos, ChunkPos, Direction, Downcast as _, WorldAabb};

    use super::PaintingEntity;
    use crate::entity::block_attached_entity::CHECK_SURVIVAL_FREQUENCY_TICKS;
    use crate::entity::entities::ItemEntity;
    use crate::entity::{Entity as _, SharedEntity};
    use crate::test_support::{TestWorld, fresh_test_world, insert_ready_full_chunk};

    fn kebab() -> RegistryReference<PaintingVariant> {
        RegistryReference::new(&vanilla_painting_variants::KEBAB)
    }

    fn painting_area() -> WorldAabb {
        WorldAabb::new(-6.0, 60.0, -6.0, 6.0, 72.0, 6.0)
    }

    /// Builds a world with a single stone block at `(0, 64, 0)` acting as a wall.
    fn wall_fixture(name: &str) -> (TestWorld, BlockPos) {
        init_vanilla_registry();

        let world_fixture = fresh_test_world(name);
        let world = Arc::clone(&world_fixture.world);
        let wall = BlockPos::new(0, 64, 0);
        insert_ready_full_chunk(&world, ChunkPos::from_block_pos(wall));
        assert!(world.set_block(
            wall,
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        (world_fixture, wall)
    }

    fn spawn_painting(
        world: &Arc<crate::world::World>,
        block_pos: BlockPos,
        facing: Direction,
        variant: RegistryReference<PaintingVariant>,
    ) -> Arc<PaintingEntity> {
        let painting = Arc::new(PaintingEntity::new_attached(
            &vanilla_entities::PAINTING,
            crate::entity::next_entity_id(),
            block_pos,
            facing,
            variant,
            Arc::downgrade(world),
        ));
        let shared: SharedEntity = painting.clone();
        world
            .try_add_entity(shared)
            .expect("painting should enter a loaded chunk");
        painting
    }

    fn dropped_items(world: &Arc<crate::world::World>) -> Vec<ItemStack> {
        world
            .get_entities_in_aabb(&painting_area())
            .iter()
            .filter_map(|entity| entity.downcast_ref::<ItemEntity>())
            .map(ItemEntity::get_item)
            .collect()
    }

    #[test]
    fn the_bounding_box_matches_the_variant_size() {
        let (fixture, wall) = wall_fixture("painting_bbox");
        let variant = kebab();
        let value = variant.value().value();
        let painting = PaintingEntity::new_attached(
            &vanilla_entities::PAINTING,
            1,
            wall.offset(1, 0, 0),
            Direction::East,
            variant,
            Arc::downgrade(&fixture.world),
        );

        let box_ = painting.base().bounding_box();
        assert!(
            (box_.max_x() - box_.min_x() - super::PIXEL_SIZE).abs() < 1.0e-6,
            "painting depth must be one pixel"
        );
        assert!(
            (box_.max_y() - box_.min_y() - f64::from(value.height)).abs() < 1.0e-6
        );
        assert!(
            (box_.max_z() - box_.min_z() - f64::from(value.width)).abs() < 1.0e-6
        );
    }

    #[test]
    fn a_painting_only_stays_with_a_solid_wall() {
        let (fixture, wall) = wall_fixture("painting_support");
        let painting = spawn_painting(
            &fixture.world,
            wall.offset(1, 0, 0),
            Direction::East,
            kebab(),
        );
        assert!(painting.can_stay_attached());

        Arc::clone(&painting).tick();
        assert!(!painting.is_removed());
    }

    #[test]
    fn removing_the_wall_drops_the_painting_item() {
        let (fixture, wall) = wall_fixture("painting_wall_gone");
        let painting = spawn_painting(
            &fixture.world,
            wall.offset(1, 0, 0),
            Direction::East,
            kebab(),
        );

        assert!(fixture.world.set_block(
            wall,
            vanilla_blocks::AIR.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        for _ in 0..=CHECK_SURVIVAL_FREQUENCY_TICKS {
            Arc::clone(&painting).tick();
        }

        assert!(painting.is_removed());
        let drops = dropped_items(&fixture.world);
        assert_eq!(drops.len(), 1);
        assert!(drops[0].is(&vanilla_items::PAINTING));
    }

    #[test]
    fn dropping_the_painting_twice_does_not_duplicate_it() {
        let (fixture, wall) = wall_fixture("painting_drop_once");
        let painting = spawn_painting(
            &fixture.world,
            wall.offset(1, 0, 0),
            Direction::East,
            kebab(),
        );

        painting.drop_item(None);
        painting.drop_item(None);

        assert_eq!(dropped_items(&fixture.world).len(), 1);
    }

    #[test]
    fn place_painting_requires_a_wall() {
        let (fixture, wall) = wall_fixture("painting_place_wall");
        let world = Arc::clone(&fixture.world);

        let placed = PaintingEntity::place_painting(
            &vanilla_entities::PAINTING,
            1,
            wall.offset(1, 0, 0),
            Direction::East,
            Arc::downgrade(&world),
        );
        let placed = placed.expect("a painting fits on a solid wall");
        assert!(placed.can_stay_attached());

        let floating = PaintingEntity::place_painting(
            &vanilla_entities::PAINTING,
            2,
            BlockPos::new(0, 90, 0),
            Direction::East,
            Arc::downgrade(&world),
        );
        assert!(floating.is_none(), "no wall means no painting");
    }

    #[test]
    fn saving_and_loading_keeps_the_variant_and_facing() {
        let (fixture, wall) = wall_fixture("painting_save");
        let painting = spawn_painting(
            &fixture.world,
            wall.offset(1, 0, 0),
            Direction::East,
            kebab(),
        );

        let mut nbt = NbtCompound::new();
        painting.save_additional(&mut nbt);
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let borrowed =
            read_compound(&mut Cursor::new(bytes.as_slice())).expect("test NBT should reborrow");

        let fresh = PaintingEntity::new_attached(
            &vanilla_entities::PAINTING,
            9,
            wall.offset(1, 0, 0),
            Direction::South,
            RegistryReference::new(REGISTRY.painting_variants.by_id(0).expect("variants")),
            std::sync::Weak::new(),
        );
        assert_eq!(fresh.facing(), Direction::South);

        fresh.load_additional((&borrowed).into());
        assert_eq!(fresh.facing(), Direction::East);
        assert_eq!(fresh.variant().get().key, kebab().get().key);
    }
}
