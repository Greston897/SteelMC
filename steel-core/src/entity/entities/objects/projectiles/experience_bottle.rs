//! Thrown experience bottle projectile entity (`ExperienceBottle`).
//!
//! Mirrors vanilla `ExperienceBottleEntity`: a `ThrownItemEntity` with a
//! steeper 0.07 gravity that, on impact, plays the splash world event and
//! scatters `3 + rand(5) + rand(5)` experience as orbs before discarding
//! itself.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::items::ItemRef;
use steel_registry::vanilla_entity_data::ExperienceBottleEntityData;
use steel_registry::vanilla_items;
use steel_utils::locks::SyncMutex;
use steel_utils::{BlockPos, DowncastType, DowncastTypeKey};

use crate::entity::entities::ExperienceOrbEntity;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntitySyncedData, Projectile, ProjectileBase,
    ProjectileHit, RemovalReason, SharedEntity, ThrowableItemProjectile, ThrowableProjectile,
};
use crate::world::World;

/// Vanilla `ExperienceBottleEntity.getGravity`.
const GRAVITY: f64 = 0.07;

/// Vanilla `WorldEvents.SPLASH_POTION_SPLASH`.
const SPLASH_LEVEL_EVENT: i32 = 2002;

/// Vanilla `ExperienceBottleEntity` splash particle color.
const SPLASH_PARTICLE_COLOR: i32 = -13_083_194;

/// Vanilla `ExperienceBottleEntity.MIN_EXPERIENCE`.
const MIN_EXPERIENCE: i32 = 3;

/// Vanilla `ExperienceBottleEntity.EXPERIENCE_RANDOM_RANGE`, rolled twice.
const EXPERIENCE_RANDOM_RANGE: i32 = 5;

/// Half of the average side length of the orb hitbox: vanilla
/// `ExperienceOrbEntity` offsets itself by this along the hit normal.
const ORB_SPAWN_OFFSET: f64 = 0.125;

/// A thrown experience bottle.
#[entity_behavior(class = "ThrownExperienceBottle")]
pub struct ExperienceBottleEntity {
    /// Common entity fields (id, uuid, position, etc.).
    base: EntityBase,
    /// Vanilla entity type registered for this implementation.
    entity_type: EntityTypeRef,
    /// Synced data carrying the rendered item stack.
    entity_data: SyncMutex<ExperienceBottleEntityData>,
    /// Shared `Projectile` state (owner / left-owner / has-been-shot).
    projectile_base: ProjectileBase,
}

// SAFETY: This key is owned by Steel and uniquely identifies `ExperienceBottleEntity`.
unsafe impl DowncastType for ExperienceBottleEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/experience_bottle");
}

impl ExperienceBottleEntity {
    /// Creates a new thrown experience bottle with no owner.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            entity_data: SyncMutex::new(ExperienceBottleEntityData::new()),
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Creates a thrown experience bottle from saved base data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(ExperienceBottleEntityData::new()),
            projectile_base: ProjectileBase::new(),
        }
    }

    /// Vanilla `MIN_EXPERIENCE + rand(5) + rand(5)`.
    fn experience_amount() -> i32 {
        MIN_EXPERIENCE
            + rand::random_range(0..EXPERIENCE_RANDOM_RANGE)
            + rand::random_range(0..EXPERIENCE_RANDOM_RANGE)
    }
}

impl Entity for ExperienceBottleEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn tick(self: Arc<Self>) {
        self.throwable_projectile_tick();
    }

    fn get_default_gravity(&self) -> f64 {
        GRAVITY
    }

    fn sound_source(&self) -> SoundSource {
        SoundSource::Neutral
    }

    fn spawn_data(&self) -> i32 {
        self.get_owner().map_or(0, |owner| owner.id())
    }

    fn restore_owner_reference(&self, owner: &SharedEntity) {
        self.cache_owner_entity(owner);
    }

    fn projectile_owner_uuid(&self) -> Option<uuid::Uuid> {
        self.owner_uuid()
    }

    fn projectile_owner(&self) -> Option<SharedEntity> {
        self.get_owner()
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(&self.entity_data)
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_projectile(nbt);
        self.save_throwable_item(nbt);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_projectile(nbt);
        self.load_throwable_item(nbt);
    }
}

impl Projectile for ExperienceBottleEntity {
    fn projectile_base(&self) -> &ProjectileBase {
        &self.projectile_base
    }

    fn on_hit(self: Arc<Self>, hit: &ProjectileHit) {
        // Vanilla `ThrownExperienceBottle.onCollision`: base collision handling
        // first (which is what breaks the bottle), then the splash event and
        // the experience orbs, then `discard()`.
        Arc::clone(&self).projectile_on_hit(hit);

        let Some(world) = self.level() else {
            self.set_removed(RemovalReason::Discarded);
            return;
        };
        let location = hit.location();
        let direction = match hit {
            ProjectileHit::Block { hit, .. } => {
                let (dx, dy, dz) = hit.direction.offset();
                DVec3::new(f64::from(dx), f64::from(dy), f64::from(dz))
            }
            ProjectileHit::Entity(_) => -self.velocity(),
        };

        world.level_event(
            SPLASH_LEVEL_EVENT,
            BlockPos::containing(location.x, location.y, location.z),
            SPLASH_PARTICLE_COLOR,
            None,
        );
        ExperienceOrbEntity::award(
            &world,
            location + direction * ORB_SPAWN_OFFSET,
            Self::experience_amount(),
        );

        self.set_removed(RemovalReason::Discarded);
    }
}

impl ThrowableProjectile for ExperienceBottleEntity {}

impl ThrowableItemProjectile for ExperienceBottleEntity {
    fn get_default_item(&self) -> ItemRef {
        &*vanilla_items::EXPERIENCE_BOTTLE
    }

    fn set_item(&self, item: ItemStack) {
        self.entity_data
            .lock()
            .throwable_item_projectile
            .item_stack
            .set(item);
    }

    fn get_item(&self) -> ItemStack {
        self.entity_data
            .lock()
            .throwable_item_projectile
            .item_stack
            .get()
            .clone()
    }
}

#[cfg(test)]
mod tests {

    use glam::DVec3;
    use steel_registry::vanilla_entities;
    use steel_utils::{BlockPos, ChunkPos, Direction, Downcast as _};

    use super::*;
    use crate::test_support::{fresh_test_world, insert_ready_full_chunk};
    use crate::world::ClipHitResult;

    #[test]
    fn experience_amount_matches_vanilla_range() {
        for _ in 0..64 {
            let amount = ExperienceBottleEntity::experience_amount();
            assert!((3..=11).contains(&amount));
        }
    }

    #[test]
    fn impact_spawns_experience_orbs_and_discards_the_bottle() {
        steel_registry::init_vanilla_registry();

        let fixture = fresh_test_world("experience_bottle_break");
        let world = &fixture.world;
        insert_ready_full_chunk(world, ChunkPos::new(0, 0));

        let bottle = Arc::new(ExperienceBottleEntity::new(
            &vanilla_entities::EXPERIENCE_BOTTLE,
            1,
            DVec3::new(8.0, 65.0, 8.0),
            Arc::downgrade(world),
        ));
        let hit = ProjectileHit::Block {
            location: DVec3::new(8.0, 64.9, 8.0),
            hit: ClipHitResult {
                location: DVec3::new(8.0, 64.9, 8.0),
                direction: Direction::Up,
                block_pos: BlockPos::new(8, 64, 8),
                miss: false,
                inside: false,
                world_border_hit: false,
            },
        };

        Arc::clone(&bottle).on_hit(&hit);

        assert!(bottle.is_removed());
        let orbs: i32 = world
            .get_entities_in_aabb(&steel_utils::WorldAabb::new(
                6.0, 63.0, 6.0, 10.0, 67.0, 10.0,
            ))
            .iter()
            .filter_map(|entity| entity.as_ref().downcast_ref::<ExperienceOrbEntity>())
            .map(ExperienceOrbEntity::value)
            .sum();
        assert!(
            (3..=11).contains(&orbs),
            "expected 3..=11 experience points, got {orbs}"
        );
    }
}
