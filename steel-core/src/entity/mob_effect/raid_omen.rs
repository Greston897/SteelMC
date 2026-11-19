//! `RaidOmenMobEffect` behavior.

use super::MobEffectBehavior;
use crate::entity::{Entity as _, LivingEntity};
use crate::world::World;

/// Raid omen mob effect behavior.
///
/// Vanilla applies it on the final tick: the stored omen position becomes the
/// raid center, then the effect is consumed.
pub struct RaidOmenBehavior;

impl MobEffectBehavior for RaidOmenBehavior {
    fn should_apply_effect_tick_this_tick(&self, tick_count: i32, _amplifier: i32) -> bool {
        tick_count == 1
    }

    fn apply_effect_tick(&self, world: &World, user: &dyn LivingEntity, amplifier: i32) -> bool {
        let Some(player) = user.as_player() else {
            return true;
        };
        if player.is_spectator() {
            return true;
        }

        let Some(pos) = player.raid_omen_position.lock().take() else {
            return true;
        };
        if !world.get_game_rule(&steel_registry::vanilla_game_rules::RAIDS)
            || world.dimension_type.can_start_raid == Some(false)
        {
            return false;
        }
        let center = crate::world::raid::village_center_world(world, pos).unwrap_or(pos.0);
        world.raids.raise_omen(center, amplifier);
        player.trigger_advancement_with_context(
            &steel_registry::vanilla_criteria_triggers::VOLUNTARY_EXILE,
            &crate::player::advancement::predicate::TriggerContext::at(
                &player.get_world(),
                player.position(),
            ),
        );
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::mob_effect::{BadOmenBehavior, MobEffectBehavior};
    use crate::entity::{MobEffectInstance, next_entity_id};
    use crate::test_support::{TestPlayerBuilder, fresh_test_world};
    use std::sync::Arc;
    use steel_registry::{REGISTRY, vanilla_mob_effects};
    use steel_utils::BlockPos;

    #[test]
    fn bad_omen_converts_once_and_raid_uses_captured_not_current_position() {
        steel_registry::init_vanilla_registry();
        crate::behavior::init_behaviors();
        let fixture = fresh_test_world("omen_origin");
        let world = &fixture.world;
        let origin = BlockPos::new(4, 64, 4);
        let home = REGISTRY
            .poi_types
            .iter()
            .find(|(_, poi)| poi.key.path.as_ref() == "home")
            .unwrap()
            .0;
        world.poi_storage.lock().add(origin, home, 1);
        assert!(world.poi_storage.lock().reserve_ticket(origin));
        let player =
            TestPlayerBuilder::new(Arc::clone(world), "RaidTest", next_entity_id()).build();
        player
            .try_set_position(glam::DVec3::new(4.0, 64.0, 4.0))
            .unwrap();
        player.add_mob_effect(MobEffectInstance::with_duration(
            vanilla_mob_effects::BAD_OMEN,
            12_000,
            2,
        ));
        player.tick_mob_effects();
        assert!(!player.has_mob_effect(vanilla_mob_effects::BAD_OMEN));
        assert!(player.has_mob_effect(vanilla_mob_effects::RAID_OMEN));
        assert!(world.raids.raids().is_empty());
        assert_eq!(*player.raid_omen_position.lock(), Some(origin));
        player
            .try_set_position(glam::DVec3::new(1_000.0, 70.0, 1_000.0))
            .unwrap();
        for _ in 0..599 {
            player.tick_mob_effects();
        }
        assert!(world.raids.raids().is_empty());
        player.tick_mob_effects();
        let raids = world.raids.raids();
        assert_eq!(raids.len(), 1);
        assert_eq!(raids[0].center, origin.0);
        assert_eq!(raids[0].omen_level, 2);
        assert!(player.raid_omen_position.lock().is_none());
        assert!(!player.has_mob_effect(vanilla_mob_effects::RAID_OMEN));
        assert!(BadOmenBehavior.should_apply_effect_tick_this_tick(1, 0));
    }

    #[test]
    fn manually_given_raid_omen_without_source_cannot_start_a_raid() {
        let fixture = fresh_test_world("omen_no_origin");
        let player =
            TestPlayerBuilder::new(Arc::clone(&fixture.world), "RaidTest", next_entity_id())
                .build();
        assert!(RaidOmenBehavior.apply_effect_tick(&fixture.world, player.as_ref(), 4));
        assert!(fixture.world.raids.raids().is_empty());
    }
}

#[cfg(test)]
mod dimension_tests {
    use super::*;
    use crate::test_support::{TestPlayerBuilder, fresh_test_world_with_dimension_type};
    use crate::entity::next_entity_id;
    use std::sync::Arc;

    #[test]
    fn dimensions_that_forbid_raids_consume_origin_without_starting_encounter() {
        let fixture = fresh_test_world_with_dimension_type("raid", "nether_omen", &steel_registry::vanilla_dimension_types::THE_NETHER);
        let player = TestPlayerBuilder::new(Arc::clone(&fixture.world), "NetherOmen", next_entity_id()).build();
        *player.raid_omen_position.lock() = Some(steel_utils::BlockPos::new(0, 64, 0));
        assert!(!RaidOmenBehavior.apply_effect_tick(&fixture.world, player.as_ref(), 0));
        assert!(fixture.world.raids.raids().is_empty());
        assert!(player.raid_omen_position.lock().is_none());
    }
}
