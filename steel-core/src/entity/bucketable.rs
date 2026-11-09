//! Vanilla `Bucketable` interface: mobs that can be caught in a bucket.
//!
//! Vanilla stores the caught mob's state in the `minecraft:bucket_entity_data`
//! data component (`BUCKET_ENTITY_DATA` in Steel) and flips the tracked
//! `FROM_BUCKET` flag, which makes the mob persistent (`cannotDespawn`) and
//! skips the immediate-despawn check.

use simdnbt::owned;
use steel_registry::data_components::components::CustomData;
use steel_registry::item_stack::ItemStack;
use steel_registry::data_components::vanilla_components;
use steel_registry::items::ItemRef;
use steel_registry::vanilla_items;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::types::InteractionHand;
use steel_utils::Downcast as _;

use crate::behavior::InteractionResult;
use crate::entity::entities::mobs::passive::{
    CodEntity, PufferfishEntity, SalmonEntity,
};
use crate::entity::{Entity, LivingEntity, Mob, RemovalReason};
use crate::player::Player;

/// Vanilla `Bucketable`.
pub trait Bucketable: Mob {
    /// Vanilla `fromBucket()`.
    fn is_from_bucket(&self) -> bool;
    /// Vanilla `setFromBucket(boolean)`.
    fn set_from_bucket(&self, from_bucket: bool);
    /// The item produced when the mob is caught.
    fn bucket_item(&self) -> ItemRef;

    /// Vanilla `Bucketable.copyDataToNbt` — base entity state stored in the
    /// bucket component. Subclasses extend this with their own data.
    fn copy_bucket_data_to_nbt(&self, nbt: &mut owned::NbtCompound) {
        write_base_bucket_data(self, nbt);
    }

    /// Vanilla `Bucketable.copyDataFromNbt` counterpart.
    fn read_bucket_data_from_nbt(&self, nbt: &simdnbt::borrow::NbtCompound<'_, '_>) {
        self.set_from_bucket(nbt.byte_value("FromBucket").unwrap_or(1) != 0);
        if let Some(health) = nbt.float_value("Health") {
            LivingEntity::set_health(self, health);
        }
    }
}

// TODO(bucketable): `Bucketable.tryBucket` (water bucket -> mob bucket exchange,
// `RemovalReason::Discarded`, `apply_filled_result`, bucket NBT component) and the
// catch path inside `BucketItem::use_empty_bucket` land together with the
// `MobBucketItem` behavior; the interaction recipe is taken from
// `behavior/items/boat.rs` + `behavior/items/bucket.rs` so it matches the
// existing inventory-exchange semantics exactly.

/// Base `Bucketable.copyDataToNbt` payload; also callable from an override so a
/// subclass can append its own keys after the shared ones.
pub fn write_base_bucket_data<T>(entity: &T, nbt: &mut owned::NbtCompound)
where
    T: Bucketable + ?Sized,
{
    nbt.insert("FromBucket", 1i8);
    nbt.insert("Health", LivingEntity::get_health(entity));
}

/// Vanilla `Bucketable.tryBucket` for one concrete mob type.
pub fn try_bucket<T>(
    player: &Player,
    hand: InteractionHand,
    entity: &T,
) -> Option<InteractionResult>
where
    T: Bucketable + ?Sized,
{
    let held = {
        let inventory = player.inventory.lock();
        let stack = inventory.get_item_in_hand(hand);
        stack.copy_with_count(stack.count())
    };

    if !held.is(&vanilla_items::BUCKET) || !Entity::is_alive(entity) {
        return None;
    }

    entity.make_sound(Mob::ambient_sound(entity));

    let bucket_stack = bucket_stack_for(entity);
    let has_infinite_materials = player.has_infinite_materials();
    player
        .inventory
        .lock()
        .apply_filled_result(hand, bucket_stack, has_infinite_materials, true);

    Entity::set_removed(entity, RemovalReason::Discarded);

    Some(InteractionResult::Success)
}

/// Builds the bucket stack vanilla hands back after a catch: the mob item with
/// its `minecraft:bucket_entity_data` payload (`Bucketable.copyDataToNbt`).
fn bucket_stack_for<T>(entity: &T) -> ItemStack
where
    T: Bucketable + ?Sized,
{
    let mut nbt = owned::NbtCompound::new();
    entity.copy_bucket_data_to_nbt(&mut nbt);

    let mut stack = ItemStack::new(entity.bucket_item());
    if let Some(data) = CustomData::try_from_compound(nbt) {
        stack.set(vanilla_components::BUCKET_ENTITY_DATA, data);
    }
    stack
}

/// Vanilla `EntityBucketItem` release half: the freshly spawned mob is marked as
/// coming from a bucket so it does not immediately despawn.
fn restore_from_bucket<T>(entity: &T)
where
    T: Bucketable + ?Sized,
{
    if !entity.is_from_bucket() {
        entity.set_from_bucket(true);
    }
}

/// Dispatches [`restore_from_bucket`] for any entity implementing [`Bucketable`].
pub fn release_from_bucket(entity: &dyn Entity) {
    if let Some(cod) = entity.downcast_ref::<CodEntity>() {
        restore_from_bucket(cod);
        return;
    }
    if let Some(salmon) = entity.downcast_ref::<SalmonEntity>() {
        restore_from_bucket(salmon);
        return;
    }
    if let Some(pufferfish) = entity.downcast_ref::<PufferfishEntity>() {
        restore_from_bucket(pufferfish);
    }
}

/// Dispatches [`try_bucket`] for any entity that implements [`Bucketable`].
///
/// Steel has no trait-object form of `Bucketable` yet, so the dispatch is a
/// downcast chain over the implemented water mobs.
pub fn try_bucket_entity(
    player: &Player,
    hand: InteractionHand,
    entity: &dyn Entity,
) -> Option<InteractionResult> {
    if let Some(cod) = entity.downcast_ref::<CodEntity>() {
        return try_bucket(player, hand, cod);
    }
    if let Some(salmon) = entity.downcast_ref::<SalmonEntity>() {
        return try_bucket(player, hand, salmon);
    }
    if let Some(pufferfish) = entity.downcast_ref::<PufferfishEntity>() {
        return try_bucket(player, hand, pufferfish);
    }
    None
}

/// Whether the entity can currently be caught in a bucket.
#[must_use]
pub fn is_bucketable(entity: &dyn Entity) -> bool {
    entity.downcast_ref::<CodEntity>().is_some()
        || entity.downcast_ref::<SalmonEntity>().is_some()
        || entity.downcast_ref::<PufferfishEntity>().is_some()
}
