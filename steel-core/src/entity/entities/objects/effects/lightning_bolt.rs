//! Vanilla `LightningBolt`.
//!
//! A visual and damaging effect entity. Vanilla keeps two synced flags on
//! `Entity` only, so the whole state lives in NBT: `life` counts down from `2`,
//! and the visual-only flag switches the renderer to the "effect only" model.
//! Damage is applied on the first tick to every entity inside a
//! `3 x 6 x 3` box (vanilla `LIGHTNING_BOLT` bounds).

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::vanilla_entity_data::BaseEntityData;
use steel_registry::{sound_events, vanilla_damage_types};
use steel_utils::BlockPos;
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::{DowncastType, DowncastTypeKey, WorldAabb};

use crate::entity::entities::mobs::passive::MushroomCowEntity;
use crate::entity::damage::DamageSource;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntitySyncedData, RemovalReason,
};
use crate::world::World;

/// Vanilla `LightningBolt.LIFE`.
const LIGHTNING_LIFE: i32 = 2;
/// Vanilla lightning damage.
const LIGHTNING_DAMAGE: f32 = 5.0;
/// Half-width of the damage box.
const DAMAGE_RADIUS: f64 = 3.0;
/// Vanilla `LIGHTNING_BOLT` box height, measured from `y - 3` to `y + 6`.
const DAMAGE_BELOW: f64 = 3.0;
const DAMAGE_ABOVE: f64 = 6.0;

/// Vanilla `net.minecraft.world.entity.LightningBolt`.
#[entity_behavior(class = "LightningBolt")]
pub struct LightningBoltEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    entity_data: SyncMutex<BaseEntityData>,
    state: SyncMutex<LightningBoltState>,
}

#[derive(Debug, Clone, Copy)]
struct LightningBoltState {
    /// Ticks this bolt has been alive.
    life: i32,
    /// Visual-only bolts (from `/summon lightning_bolt {visual_only:1b}`) deal no damage.
    visual_only: bool,
}

impl Default for LightningBoltState {
    fn default() -> Self {
        Self {
            life: LIGHTNING_LIFE,
            visual_only: false,
        }
    }
}

// SAFETY: This key is owned by Steel and uniquely identifies `LightningBoltEntity`.
unsafe impl DowncastType for LightningBoltEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/lightning_bolt");
}

impl LightningBoltEntity {
    /// Creates a new lightning bolt entity.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            entity_data: SyncMutex::new(BaseEntityData::new()),
            state: SyncMutex::new(LightningBoltState::default()),
        }
    }

    /// Creates a lightning bolt entity from saved data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            entity_data: SyncMutex::new(BaseEntityData::new()),
            state: SyncMutex::new(LightningBoltState::default()),
        }
    }

    /// Remaining lifetime in ticks.
    #[must_use]
    pub fn life(&self) -> i32 {
        self.state.lock().life
    }

    /// Whether this bolt is visual-only and never hurts anything.
    #[must_use]
    pub fn is_visual_only(&self) -> bool {
        self.state.lock().visual_only
    }

    /// Marks this bolt as visual-only, like vanilla `EntityDataAccessor` for
    /// lightning's `DATA_VISUAL_ONLY` flag.
    pub fn set_visual_only(&self, visual_only: bool) {
        self.state.lock().visual_only = visual_only;
    }

    /// Vanilla `LightningBolt.tick`'s damage pass.
    fn hit_entities(self: &Arc<Self>) {
        if self.is_visual_only() {
            return;
        }
        let Some(world) = self.level() else {
            return;
        };
        let pos = self.position();
        let box_ = WorldAabb::new(
            pos.x - DAMAGE_RADIUS,
            pos.y - DAMAGE_BELOW,
            pos.z - DAMAGE_RADIUS,
            pos.x + DAMAGE_RADIUS,
            pos.y + DAMAGE_ABOVE,
            pos.z + DAMAGE_RADIUS,
        );
        // Vanilla `LightningBolt.tick` first calls `BlockState.onLightningStrike`
        // on the block it landed on, which is what powers lightning rods.
        let struck_pos = BlockPos::containing(pos.x, pos.y, pos.z);
        let struck_state = world.get_block_state(struck_pos);
        crate::behavior::BLOCK_BEHAVIORS
            .get_behavior(struck_state.get_block())
            .on_lightning_strike(struck_state, &world, struck_pos);

        // Vanilla `LightningBolt.tick` fires `lightning_strike` on the players
        // `ServerLevel.lastLightningBolt` recorded as this strike's cause, while
        // `channeled_lightning` goes to the trident thrower. Steel's bolts carry no
        // owner, so the players inside the same box the damage sweep uses play the
        // striker role: those are the ones vanilla would have notified.
        for entity in world.get_entities_in_aabb(&box_) {
            let Some(player) = entity.as_player() else {
                continue;
            };
            let context = crate::player::advancement::predicate::TriggerContext::at(&world, pos);
            player.trigger_advancement_with_context(
                &steel_registry::vanilla_criteria_triggers::LIGHTNING_STRIKE,
                &context,
            );
            player.trigger_advancement_with_context(
                &steel_registry::vanilla_criteria_triggers::CHANNELED_LIGHTNING,
                &context,
            );
        }

        let source = DamageSource::environment(&vanilla_damage_types::LIGHTNING_BOLT);
        let self_uuid = self.uuid();
        for entity in world.get_entities_in_aabb(&box_) {
            if entity.id() == self.id() || !entity.is_alive() {
                continue;
            }

            // Vanilla runs `LightningBolt.onHitEntity` before the damage call, which
            // is where the mooshroom variant flip lives.
            if let Some(cow) = entity.as_entity_event_source().downcast_ref::<MushroomCowEntity>() {
                cow.thunder_hit(self_uuid);
            }

            entity.hurt(&world, &source, LIGHTNING_DAMAGE);
        }
    }
}

impl Entity for LightningBoltEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
        Some(&self.entity_data)
    }

    fn tick(self: Arc<Self>) {
        let life = {
            let mut state = self.state.lock();
            state.life -= 1;
            state.life
        };
        if life < 0 {
            self.set_removed(RemovalReason::Discarded);
            return;
        }
        if life == LIGHTNING_LIFE - 1 {
            self.play_sound(&sound_events::ENTITY_LIGHTNING_BOLT_THUNDER, 1.0, 1.0);
            self.hit_entities();
        }
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        let state = *self.state.lock();
        nbt.insert("life", state.life);
        nbt.insert("visual_only", i8::from(state.visual_only));
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        let mut state = self.state.lock();
        if let Some(life) = nbt.int_value("life") {
            state.life = life;
        }
        if let Some(visual_only) = nbt.byte_value("visual_only") {
            state.visual_only = visual_only != 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;

    use super::*;

    fn bolt() -> LightningBoltEntity {
        LightningBoltEntity::new(
            &vanilla_entities::LIGHTNING_BOLT,
            1,
            DVec3::new(0.5, 4.0, 0.5),
            Weak::new(),
        )
    }

    #[test]
    fn new_lightning_bolts_are_not_visual_only() {
        init_vanilla_registry();
        let bolt = bolt();
        assert!(!bolt.is_visual_only());
        assert_eq!(bolt.life(), LIGHTNING_LIFE);
    }

    #[test]
    fn visual_only_flag_is_saved_into_nbt() {
        init_vanilla_registry();
        let bolt = bolt();
        bolt.set_visual_only(true);
        let mut nbt = NbtCompound::new();
        bolt.save_additional(&mut nbt);
        assert_eq!(nbt.byte_value("visual_only"), Some(1));
        assert_eq!(nbt.int_value("life"), Some(LIGHTNING_LIFE));

        let loaded = bolt();
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let borrowed = simdnbt::borrow::read_compound(&mut std::io::Cursor::new(&bytes))
            .unwrap_or_else(|error| panic!("reborrow failed: {error}"));
        loaded.load_additional((&borrowed).into());
        assert!(loaded.is_visual_only());
    }

    #[test]
    fn vanilla_lightning_damage_matches() {
        assert_eq!(LIGHTNING_DAMAGE, 5.0);
        assert_eq!(LIGHTNING_LIFE, 2);
    }
}
