//! Cushion entity (`Cushion`), the 26.3 seat.
//!
//! Vanilla cushions are entities rather than blocks: they carry one of the 16
//! wool colours, rest on top of the block they were placed on, take no
//! collision, let a player sit on them with a right click and pop off as their
//! coloured item when the supporting block disappears or the cushion is
//! attacked.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::FromNbtTag as _;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::data_components::vanilla_components::{CUSHION_COLOR, CUSTOM_NAME};
use steel_registry::dye_color::DyeColor;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::items::ItemRef;
use steel_registry::vanilla_entity_data::CushionEntityData;
use steel_registry::{sound_events, vanilla_game_rules, vanilla_items};
use steel_utils::locks::SyncMutex;
use steel_utils::types::InteractionHand;
use steel_utils::{BlockPos, DowncastType, DowncastTypeKey, WorldAabb};

use crate::behavior::InteractionResult;
use crate::entity::damage::DamageSource;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntityBaseState, EntitySyncedData, RemovalReason,
};
use crate::physics::collision::{CollisionWorld, WorldCollisionProvider};
use crate::player::Player;
use crate::world::World;

/// Subtracts one collider from a world-space box, retaining up to six slabs.
fn subtract_box(source: WorldAabb, collider: WorldAabb, out: &mut Vec<WorldAabb>) {
    if !source.intersects(collider) {
        out.push(source);
        return;
    }
    let x0 = source.min_x().max(collider.min_x());
    let x1 = source.max_x().min(collider.max_x());
    let y0 = source.min_y().max(collider.min_y());
    let y1 = source.max_y().min(collider.max_y());
    let z0 = source.min_z().max(collider.min_z());
    let z1 = source.max_z().min(collider.max_z());
    let mut push = |min_x, min_y, min_z, max_x, max_y, max_z| {
        if min_x < max_x && min_y < max_y && min_z < max_z {
            out.push(WorldAabb::new(min_x, min_y, min_z, max_x, max_y, max_z));
        }
    };
    push(
        source.min_x(),
        source.min_y(),
        source.min_z(),
        x0,
        source.max_y(),
        source.max_z(),
    );
    push(
        x1,
        source.min_y(),
        source.min_z(),
        source.max_x(),
        source.max_y(),
        source.max_z(),
    );
    push(x0, source.min_y(), source.min_z(), x1, y0, source.max_z());
    push(x0, y1, source.min_z(), x1, source.max_y(), source.max_z());
    push(x0, y0, source.min_z(), x1, y1, z0);
    push(x0, y0, z1, x1, y1, source.max_z());
}

/// Returns the vanilla cushion item that matches a wool colour.
#[must_use]
pub fn cushion_item_for_color(color: DyeColor) -> ItemRef {
    match color {
        DyeColor::White => &vanilla_items::WHITE_CUSHION,
        DyeColor::Orange => &vanilla_items::ORANGE_CUSHION,
        DyeColor::Magenta => &vanilla_items::MAGENTA_CUSHION,
        DyeColor::LightBlue => &vanilla_items::LIGHT_BLUE_CUSHION,
        DyeColor::Yellow => &vanilla_items::YELLOW_CUSHION,
        DyeColor::Lime => &vanilla_items::LIME_CUSHION,
        DyeColor::Pink => &vanilla_items::PINK_CUSHION,
        DyeColor::Gray => &vanilla_items::GRAY_CUSHION,
        DyeColor::LightGray => &vanilla_items::LIGHT_GRAY_CUSHION,
        DyeColor::Cyan => &vanilla_items::CYAN_CUSHION,
        DyeColor::Purple => &vanilla_items::PURPLE_CUSHION,
        DyeColor::Blue => &vanilla_items::BLUE_CUSHION,
        DyeColor::Brown => &vanilla_items::BROWN_CUSHION,
        DyeColor::Green => &vanilla_items::GREEN_CUSHION,
        DyeColor::Red => &vanilla_items::RED_CUSHION,
        DyeColor::Black => &vanilla_items::BLACK_CUSHION,
    }
}

/// A vanilla cushion: a coloured, non-colliding seat entity.
#[entity_behavior(class = "Cushion")]
pub struct CushionEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<CushionEntityData>,
    /// Set once the cushion dropped its item, so a second removal path (attack
    /// plus support check) cannot duplicate the drop.
    dropped: AtomicBool,
    /// Whether someone rode the cushion last tick, used to play `get_up`.
    was_ridden: AtomicBool,
    /// Vanilla block-attached entities recheck support every 101 ticks.
    ticks_since_check: AtomicU32,
}

// SAFETY: This key is owned by Steel and uniquely identifies `CushionEntity`.
unsafe impl DowncastType for CushionEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/cushion");
}

impl CushionEntity {
    /// Creates a fresh cushion from the generic entity factory path.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new_with_state(
                id,
                EntityBaseState::new(position, entity_type.dimensions),
                world,
            ),
            entity_type,
            entity_data: SyncMutex::new(CushionEntityData::new()),
            dropped: AtomicBool::new(false),
            was_ridden: AtomicBool::new(false),
            ticks_since_check: AtomicU32::new(0),
        }
    }

    /// Creates a cushion resting on top of `support`.
    #[must_use]
    pub fn new_on_support(
        entity_type: EntityTypeRef,
        id: i32,
        support: BlockPos,
        color: DyeColor,
        world: Weak<World>,
    ) -> Self {
        let entity = Self::new(entity_type, id, Self::support_top(support), world);
        entity.set_color(color);
        entity
    }

    /// Creates a cushion from persistent entity data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(CushionEntityData::new()),
            dropped: AtomicBool::new(false),
            was_ridden: AtomicBool::new(false),
            ticks_since_check: AtomicU32::new(0),
        }
    }

    /// The centre of the top face of `support`: a cushion's resting position.
    #[must_use]
    pub fn support_top(support: BlockPos) -> DVec3 {
        DVec3::new(
            f64::from(support.x()) + 0.5,
            f64::from(support.y()) + 1.0,
            f64::from(support.z()) + 0.5,
        )
    }

    /// The wool colour of this cushion.
    #[must_use]
    pub fn color(&self) -> DyeColor {
        *self.entity_data.lock().color.get()
    }

    /// Sets the wool colour, marking the synced value dirty.
    pub fn set_color(&self, color: DyeColor) {
        self.entity_data.lock().color.set(color);
    }

    /// The block this cushion rests on.
    #[must_use]
    pub fn support_pos(&self) -> BlockPos {
        let pos = self.block_position();
        BlockPos::new(pos.x(), pos.y() - 1, pos.z())
    }

    /// Vanilla checks a thin anchor box against the bounds of nearby outline shapes.
    fn has_anchor_below(world: &World, bounding_box: WorldAabb) -> bool {
        let anchor = WorldAabb::new(
            bounding_box.min_x(),
            bounding_box.min_y() - 0.015625,
            bounding_box.min_z(),
            bounding_box.max_x().next_down(),
            bounding_box.min_y(),
            bounding_box.max_z().next_down(),
        );
        let search = anchor.expand_towards(DVec3::new(0.0, -0.125, 0.0));
        for z in search.min_z().floor() as i32..=search.max_z().floor() as i32 {
            for y in search.min_y().floor() as i32..=search.max_y().floor() as i32 {
                for x in search.min_x().floor() as i32..=search.max_x().floor() as i32 {
                    let pos = BlockPos::new(x, y, z);
                    let shape = world.get_block_state(pos).get_outline_shape_at(pos);
                    if shape
                        .bounds()
                        .is_some_and(|bounds| bounds.at_block(pos).intersects(anchor))
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Mirrors the vanilla support and suffocation checks, using extracted outline shapes.
    #[must_use]
    pub fn would_survive_at(world: &World, bounding_box: WorldAabb) -> bool {
        if !Self::has_anchor_below(world, bounding_box) {
            return false;
        }
        for z in bounding_box.min_z().next_up().floor() as i32
            ..=bounding_box.max_z().next_down().floor() as i32
        {
            for y in bounding_box.min_y().next_up().floor() as i32
                ..=bounding_box.max_y().next_down().floor() as i32
            {
                for x in bounding_box.min_x().next_up().floor() as i32
                    ..=bounding_box.max_x().next_down().floor() as i32
                {
                    if !world.get_block_state(BlockPos::new(x, y, z)).is_suffocating() {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Placement also requires at least part of the resting slice to be exposed.
    #[must_use]
    pub fn can_be_placed_at(world: &Arc<World>, bounding_box: WorldAabb) -> bool {
        if !Self::would_survive_at(world, bounding_box) {
            return false;
        }
        let slice = WorldAabb::new(
            bounding_box.min_x().next_up(),
            bounding_box.min_y().next_up(),
            bounding_box.min_z().next_up(),
            bounding_box.max_x().next_down(),
            (bounding_box.min_y() + 0.015625).next_down(),
            bounding_box.max_z().next_down(),
        );
        let mut exposed = vec![slice];
        for collider in WorldCollisionProvider::new(world).get_block_collisions(&slice) {
            let mut remaining = Vec::new();
            for part in exposed {
                subtract_box(part, collider, &mut remaining);
            }
            if remaining.is_empty() {
                return false;
            }
            exposed = remaining;
        }
        true
    }

    /// The item stack this cushion drops, carrying its colour component.
    #[must_use]
    pub fn as_item_stack(&self) -> ItemStack {
        let color = self.color();
        let mut stack = ItemStack::new(cushion_item_for_color(color));
        stack.set(CUSHION_COLOR, color);
        if let Some(name) = self.custom_name() {
            stack.set(CUSTOM_NAME, name);
        }
        stack
    }

    /// Drops the cushion item once and removes the entity.
    fn drop_and_remove(&self, world: &Arc<World>, creative_attacker: bool) {
        if self.dropped.swap(true, Ordering::AcqRel) {
            return;
        }

        self.play_sound(&sound_events::ENTITY_CUSHION_BREAK, 1.0, 1.0);
        if !creative_attacker && world.get_game_rule(&vanilla_game_rules::ENTITY_DROPS) {
            world.drop_item_stack(self.block_position(), self.as_item_stack());
        }
        self.set_removed(RemovalReason::Discarded);
    }
}

impl Entity for CushionEntity {
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
        true
    }

    fn tick(self: Arc<Self>) {
        let Some(world) = self.base.level() else {
            return;
        };

        // Vanilla plays `entity.cushion.get_up` when the rider climbs off.
        let ridden = self.base.first_passenger().is_some();
        if self.was_ridden.swap(ridden, Ordering::AcqRel) && !ridden {
            self.play_sound(&sound_events::ENTITY_CUSHION_GET_UP, 1.0, 1.0);
        }

        if self.is_removed() {
            return;
        }

        // BlockAttachedEntity checks on the 101st tick, then every 101 ticks.
        if self.ticks_since_check.fetch_add(1, Ordering::Relaxed) >= 100 {
            self.ticks_since_check.store(0, Ordering::Relaxed);
            if !Self::would_survive_at(&world, self.bounding_box()) {
                self.drop_and_remove(&world, false);
            }
        }
    }

    fn interact(
        &self,
        player: &Player,
        _hand: InteractionHand,
        _location: DVec3,
    ) -> InteractionResult {
        // Sneaking keeps the normal "place the held item on the block" behaviour.
        if player.is_secondary_use_active() || self.first_passenger().is_some() {
            return InteractionResult::Pass;
        }

        let Some(world) = self.base.level() else {
            return InteractionResult::Fail;
        };
        let Some(vehicle) = world.get_entity_by_id(self.id()) else {
            return InteractionResult::Fail;
        };

        player.start_riding(&vehicle);
        self.play_sound(&sound_events::ENTITY_CUSHION_SIT, 1.0, 1.0);
        InteractionResult::Success
    }

    fn hurt(&self, _world: &World, source: &DamageSource, _amount: f32) -> bool {
        let Some(world) = self.base.level() else {
            return false;
        };
        let creative_attacker = source
            .causing_entity()
            .and_then(|entity| entity.as_player())
            .is_some_and(Player::has_infinite_materials);
        self.drop_and_remove(&world, creative_attacker);
        true
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        nbt.insert("color", self.color().serialized_name().to_string());
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        if let Some(color) = nbt.get("color").and_then(DyeColor::from_nbt_tag) {
            self.set_color(color);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use glam::DVec3;
    use simdnbt::owned::NbtCompound;
    use steel_registry::data_components::vanilla_components::{CUSHION_COLOR, CUSTOM_NAME};
    use steel_registry::dye_color::DyeColor;
    use steel_registry::item_stack::ItemStack;
    use steel_registry::vanilla_entity_data::CushionEntityData;
    use steel_registry::{init_vanilla_registry, vanilla_blocks};
    use steel_utils::Downcast as _;
    use steel_utils::types::{GameType, InteractionHand, UpdateFlags};
    use steel_utils::{BlockPos, ChunkPos, WorldAabb};
    use text_components::TextComponent;

    use super::{CushionEntity, cushion_item_for_color, subtract_box};
    use crate::behavior::{InteractionResult, init_behaviors};
    use crate::entity;
    use crate::entity::damage::DamageSource;
    use crate::entity::entities::ItemEntity;
    use crate::entity::{Entity as _, SharedEntity};
    use crate::test_support::{
        TestPlayerBuilder, TestWorld, fresh_test_world, insert_ready_full_chunk,
    };
    use crate::world::World;
    use std::sync::Weak;
    use steel_registry::vanilla_damage_types;
    use steel_registry::vanilla_entities;

    fn cushion_area() -> WorldAabb {
        WorldAabb::new(-2.0, 62.0, -2.0, 2.0, 68.0, 2.0)
    }

    /// Builds a world containing a stone support block at `(0, 64, 0)`.
    fn support_fixture(name: &str) -> (TestWorld, BlockPos) {
        init_vanilla_registry();
        init_behaviors();

        let world_fixture = fresh_test_world(name);
        let world = Arc::clone(&world_fixture.world);
        let support = BlockPos::new(0, 64, 0);
        insert_ready_full_chunk(&world, ChunkPos::from_block_pos(support));
        assert!(world.set_block(
            support,
            vanilla_blocks::STONE.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        (world_fixture, support)
    }

    fn spawn_cushion(world: &Arc<World>, support: BlockPos) -> Arc<CushionEntity> {
        let cushion = Arc::new(CushionEntity::new_on_support(
            &vanilla_entities::CUSHION,
            entity::next_entity_id(),
            support,
            DyeColor::Red,
            Arc::downgrade(world),
        ));
        let shared = Arc::clone(&cushion) as SharedEntity;
        world
            .try_add_entity(shared)
            .expect("cushion should enter a loaded chunk");
        cushion
    }

    fn dropped_items(world: &Arc<World>) -> Vec<ItemStack> {
        world
            .get_entities_in_aabb(&cushion_area())
            .iter()
            .filter_map(|entity| entity.downcast_ref::<ItemEntity>())
            .map(ItemEntity::get_item)
            .collect()
    }

    #[test]
    fn buried_slice_checks_the_union_of_colliders() {
        let slice = WorldAabb::new(0.0, 0.0, 0.0, 1.0, 0.015625, 1.0);
        let left = WorldAabb::new(0.0, 0.0, 0.0, 0.5, 1.0, 1.0);
        let right = WorldAabb::new(0.5, 0.0, 0.0, 1.0, 1.0, 1.0);
        let mut exposed = Vec::new();
        subtract_box(slice, left, &mut exposed);
        assert!(!exposed.is_empty());
        let mut remaining = Vec::new();
        for part in exposed {
            subtract_box(part, right, &mut remaining);
        }
        assert!(remaining.is_empty());
    }

    #[test]
    fn cushion_stays_while_its_support_is_solid() {
        let (fixture, support) = support_fixture("cushion_support_ok");
        let cushion = spawn_cushion(&fixture.world, support);

        assert!(CushionEntity::would_survive_at(
            &fixture.world,
            cushion.bounding_box()
        ));
        Arc::clone(&cushion).tick();

        assert!(!cushion.is_removed());
        assert_eq!(dropped_items(&fixture.world), [] as [ItemStack; 0]);
    }

    #[test]
    fn removing_the_support_drops_the_coloured_item_once() {
        let (fixture, support) = support_fixture("cushion_support_gone");
        let cushion = spawn_cushion(&fixture.world, support);
        cushion.set_custom_name(Some(TextComponent::plain("Reading Nook")));

        assert!(fixture.world.set_block(
            support,
            vanilla_blocks::AIR.default_state(),
            UpdateFlags::UPDATE_ALL
        ));

        for _ in 0..100 {
            Arc::clone(&cushion).tick();
        }
        assert!(!cushion.is_removed());
        assert!(dropped_items(&fixture.world).is_empty());
        for _ in 0..3 {
            Arc::clone(&cushion).tick();
        }

        assert!(cushion.is_removed());
        let drops = dropped_items(&fixture.world);
        assert_eq!(drops.len(), 1, "a cushion must drop exactly one item");
        assert!(drops[0].is(cushion_item_for_color(DyeColor::Red)));
        assert_eq!(drops[0].get(CUSHION_COLOR), Some(&DyeColor::Red));
        assert_eq!(
            drops[0].get(CUSTOM_NAME),
            Some(&TextComponent::plain("Reading Nook"))
        );
    }

    #[test]
    fn hitting_a_cushion_drops_its_item_only_once() {
        let (fixture, support) = support_fixture("cushion_hurt_once");
        let cushion = spawn_cushion(&fixture.world, support);
        let source = DamageSource::environment(&vanilla_damage_types::PLAYER_ATTACK);

        assert!(cushion.hurt(&fixture.world, &source, 1.0));
        assert!(cushion.hurt(&fixture.world, &source, 1.0));

        assert_eq!(dropped_items(&fixture.world).len(), 1);
    }

    #[test]
    fn cushion_drops_obey_the_game_rule_and_creative_attacker() {
        for (entity_drops, creative, expected) in [
            (false, false, 0),
            (true, true, 0),
            (true, false, 1),
        ] {
            let (fixture, support) = support_fixture("cushion_drop_rules");
            assert!(fixture
                .world
                .set_game_rule(&steel_registry::vanilla_game_rules::ENTITY_DROPS, entity_drops));
            let cushion = spawn_cushion(&fixture.world, support);
            let player = TestPlayerBuilder::new(Arc::clone(&fixture.world), "Attacker", 1).build();
            if creative {
                player.set_game_mode(GameType::Creative);
            }
            let source = DamageSource::environment(&vanilla_damage_types::PLAYER_ATTACK)
                .with_causing_entity(player);
            assert!(cushion.hurt(&fixture.world, &source, 1.0));
            assert!(cushion.is_removed());
            assert_eq!(dropped_items(&fixture.world).len(), expected);
        }
    }

    #[test]
    fn interacting_makes_the_player_sit_on_the_cushion() {
        let (fixture, support) = support_fixture("cushion_sit");
        let cushion = spawn_cushion(&fixture.world, support);
        let player = TestPlayerBuilder::new(Arc::clone(&fixture.world), "Sitter", 1).build();
        player.set_client_loaded(true);
        // `Entity::start_riding` resolves the passenger through the entity manager.
        fixture
            .world
            .try_add_entity(Arc::clone(&player) as SharedEntity)
            .expect("the passenger enters the world");

        assert_eq!(
            cushion.interact(&player, InteractionHand::MainHand, DVec3::ZERO),
            InteractionResult::Success
        );

        let rider = cushion
            .base()
            .first_passenger()
            .expect("the player should be riding the cushion");
        assert_eq!(rider.id(), player.id());
    }

    #[test]
    fn sneaking_does_not_sit_but_passes() {
        let (fixture, support) = support_fixture("cushion_sit_sneak");
        let cushion = spawn_cushion(&fixture.world, support);
        let player = TestPlayerBuilder::new(Arc::clone(&fixture.world), "Croucher", 1).build();
        player.set_client_loaded(true);
        player.set_shared_shift_key_down(true);

        assert_eq!(
            cushion.interact(&player, InteractionHand::MainHand, DVec3::ZERO),
            InteractionResult::Pass
        );
        assert!(cushion.base().first_passenger().is_none());
    }

    #[test]
    fn the_item_stack_carries_the_cushion_colour() {
        let (fixture, support) = support_fixture("cushion_item_stack");
        let cushion = spawn_cushion(&fixture.world, support);
        let stack = cushion.as_item_stack();

        assert!(stack.is(cushion_item_for_color(DyeColor::Red)));
        assert_eq!(stack.get(CUSHION_COLOR), Some(&DyeColor::Red));
    }

    #[test]
    fn every_colour_has_its_own_item_and_round_trips() {
        let mut seen = Vec::with_capacity(DyeColor::VALUES.len());
        for color in DyeColor::VALUES {
            let item = cushion_item_for_color(color);
            assert!(
                !seen.contains(&item.key),
                "colour {color:?} must map to a unique cushion item"
            );
            seen.push(item.key.clone());

            let mut stack = ItemStack::new(item);
            stack.set(CUSHION_COLOR, color);
            assert_eq!(stack.get(CUSHION_COLOR), Some(&color));
        }
        assert_eq!(seen.len(), 16);
    }

    #[test]
    fn saving_writes_the_colour_name_and_defaults_stay_white() {
        let (fixture, support) = support_fixture("cushion_save");
        let cushion = spawn_cushion(&fixture.world, support);

        let mut nbt = NbtCompound::new();
        cushion.save_additional(&mut nbt);
        assert_eq!(
            nbt.string("color").map(|value| value.to_str().to_string()),
            Some("red".to_owned())
        );

        let fresh = CushionEntity::new(
            &vanilla_entities::CUSHION,
            7,
            DVec3::new(0.5, 65.0, 0.5),
            Weak::new(),
        );
        assert_eq!(fresh.color(), DyeColor::White);
        assert_eq!(CushionEntityData::new().color.get(), &DyeColor::White);
    }
}
