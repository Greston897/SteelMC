//! Vanilla `Bucketable` interface: mobs that can be caught in a bucket.
//!
//! Vanilla stores the caught mob's state in the `minecraft:bucket_entity_data`
//! data component (`BUCKET_ENTITY_DATA` in Steel) and flips the tracked
//! `FROM_BUCKET` flag, which makes the mob persistent (`cannotDespawn`) and
//! skips the immediate-despawn check.

use simdnbt::owned;
use steel_registry::items::ItemRef;
use steel_utils::nbt::NbtValueInput as _;

use crate::entity::{LivingEntity, Mob};

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
