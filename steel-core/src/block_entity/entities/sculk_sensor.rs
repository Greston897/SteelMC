//! Sculk sensor block entity.
//!
//! Vanilla `SculkSensorBlockEntity` stores the last vibration frequency it
//! received (used by the comparator output and the client listener) and owns the
//! vibration listener. Calibrated sensors share the class and differ only in how
//! they filter vibrations.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::BaseNbtCompound as BorrowedNbtCompound;
use simdnbt::owned::NbtCompound;
use steel_registry::block_entity_type::BlockEntityTypeRef;
use steel_registry::game_events::GameEventRef;
use steel_registry::vanilla_block_entity_types;
use steel_utils::{BlockPos, BlockStateId, DowncastType, DowncastTypeKey, locks::SyncMutex};

use crate::block_entity::{BlockEntity, BlockEntityBase};
use crate::entity::Entity as _;
use crate::world::World;
use crate::world::game_event::{GameEventContext, GameEventListener, SharedGameEventListener};

/// Sculk sensor and calibrated sculk sensor block entity.
pub struct SculkSensorBlockEntity {
    base: Arc<BlockEntityBase>,
    /// Last received vibration frequency, matching vanilla's `lastVibrationFrequency`.
    last_vibration_frequency: SyncMutex<i32>,
}

// SAFETY: This key is owned by Steel and uniquely identifies `SculkSensorBlockEntity`.
unsafe impl DowncastType for SculkSensorBlockEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:block_entity/sculk_sensor");
}

impl SculkSensorBlockEntity {
    /// Creates a new sculk sensor block entity.
    #[must_use]
    pub fn new(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self::new_with_type(&vanilla_block_entity_types::SCULK_SENSOR, level, pos, state)
    }

    /// Creates a new calibrated sculk sensor block entity.
    #[must_use]
    pub fn new_calibrated(level: Weak<World>, pos: BlockPos, state: BlockStateId) -> Self {
        Self::new_with_type(
            &vanilla_block_entity_types::CALIBRATED_SCULK_SENSOR,
            level,
            pos,
            state,
        )
    }

    fn new_with_type(
        block_entity_type: BlockEntityTypeRef,
        level: Weak<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) -> Self {
        Self {
            base: Arc::new(BlockEntityBase::new(block_entity_type, level, pos, state)),
            last_vibration_frequency: SyncMutex::new(0),
        }
    }

    /// Last received vibration frequency (0 when no vibration was received).
    #[must_use]
    pub fn last_vibration_frequency(&self) -> i32 {
        *self.last_vibration_frequency.lock()
    }

    /// Records a received vibration frequency.
    pub fn set_last_vibration_frequency(&self, frequency: i32) {
        *self.last_vibration_frequency.lock() = frequency;
    }
}

/// Vanilla `SculkSensorBlockEntity.VibrationListener` as far as Steel needs it:
/// the sensor hears a vibration, records its frequency and the entity that
/// caused it fires `avoid_vibration`, which is vanilla
/// `CriteriaTriggers.AVOID_VIBRATION` ("walk near a sculk sensor or shrieker
/// while sneaking").
struct SculkSensorListener {
    position: DVec3,
}

impl GameEventListener for SculkSensorListener {
    fn listener_pos(&self) -> Option<DVec3> {
        Some(self.position)
    }

    fn listener_radius(&self) -> i32 {
        SCULK_SENSOR_RADIUS
    }

    fn handle_game_event(
        &self,
        _world: &Arc<World>,
        event: GameEventRef,
        context: &GameEventContext<'_>,
        _source_pos: DVec3,
    ) -> bool {
        // Vanilla records the vibration frequency for the comparator; Steel's
        // game events do not carry a frequency yet, so only the criterion fires.
        let _ = event;
        // Vanilla credits the *source entity* of the vibration, so a player who
        // sneaks past a sensor earns the criterion even though the sensor heard
        // them.
        if let Some(source) = context.source_entity()
            && let Some(player) = source.as_player()
        {
            player.trigger_advancement_with_context(
                &steel_registry::vanilla_criteria_triggers::AVOID_VIBRATION,
                &crate::player::advancement::predicate::TriggerContext::at(
                    &player.get_world(),
                    player.position(),
                ),
            );
        }
        true
    }
}

/// Vanilla `SculkSensorBlockEntity.VibrationListener` radius.
const SCULK_SENSOR_RADIUS: i32 = 8;

impl BlockEntity for SculkSensorBlockEntity {
    fn base(&self) -> &BlockEntityBase {
        &self.base
    }

    fn game_event_listener(&self) -> Option<SharedGameEventListener> {
        let position = self.base.pos();
        Some(Arc::new(SculkSensorListener {
            position: DVec3::new(
                f64::from(position.x()),
                f64::from(position.y()),
                f64::from(position.z()),
            ),
        }))
    }

    fn load_additional(&self, nbt: &BorrowedNbtCompound<'_>) {
        let view: simdnbt::borrow::NbtCompound<'_, '_> = nbt.into();
        *self.last_vibration_frequency.lock() = view.int("last_vibration_frequency").unwrap_or(0);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        nbt.insert("last_vibration_frequency", *self.last_vibration_frequency.lock());
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks};

    use super::*;

    #[test]
    fn sensor_records_frequency() {
        init_vanilla_registry();
        let sensor = SculkSensorBlockEntity::new(
            Weak::new(),
            BlockPos::new(0, 0, 0),
            vanilla_blocks::SCULK_SENSOR.default_state(),
        );
        assert_eq!(sensor.last_vibration_frequency(), 0);
        sensor.set_last_vibration_frequency(7);
        assert_eq!(sensor.last_vibration_frequency(), 7);
    }
}
