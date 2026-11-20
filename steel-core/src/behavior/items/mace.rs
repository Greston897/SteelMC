use std::sync::Arc;

use glam::DVec3;
use steel_macros::item_behavior;
use steel_protocol::packets::game::CSetEntityMotion;
use steel_registry::item_stack::ItemStack;
use steel_registry::sound_event::SoundEventRef;
use steel_registry::{level_events, sound_events, vanilla_damage_types};

use crate::behavior::ItemBehavior;
use crate::enchantment_helper::{self, EnchantmentDamageContext};
use crate::entity::damage::DamageSource;
use crate::entity::{Entity, LivingEntity, SharedEntity};
use crate::inventory::equipment::EquipmentSlot;

/// Vanilla mace item combat behavior.
#[item_behavior]
pub struct MaceItem;

impl MaceItem {
    const SMASH_ATTACK_FALL_THRESHOLD: f64 = 1.5;
    const SMASH_ATTACK_HEAVY_THRESHOLD: f64 = 5.0;
    const SMASH_ATTACK_KNOCKBACK_RADIUS: f64 = 3.5;
    // Vanilla promotes the float constant to double before multiplying.
    const SMASH_ATTACK_KNOCKBACK_POWER: f32 = 0.7;
    const SMASH_ATTACK_PARTICLE_DATA: i32 = 750;

    fn can_smash_attack(attacker: &dyn LivingEntity) -> bool {
        attacker.fall_distance() > Self::SMASH_ATTACK_FALL_THRESHOLD && !attacker.is_fall_flying()
    }

    fn calculate_impact_position(attacker: &dyn LivingEntity) -> DVec3 {
        if let Some(impact_pos) = attacker.current_impulse_impact_pos()
            && impact_pos.y <= attacker.position().y
        {
            return impact_pos;
        }

        attacker.position()
    }

    fn smash_sound(target: &dyn LivingEntity, attacker: &dyn LivingEntity) -> SoundEventRef {
        if !target.on_ground() {
            return &sound_events::ITEM_MACE_SMASH_AIR;
        }

        if attacker.fall_distance() > Self::SMASH_ATTACK_HEAVY_THRESHOLD {
            &sound_events::ITEM_MACE_SMASH_GROUND_HEAVY
        } else {
            &sound_events::ITEM_MACE_SMASH_GROUND
        }
    }

    fn knockback_power(
        attacker: &dyn LivingEntity,
        nearby: &dyn LivingEntity,
        direction: DVec3,
    ) -> f64 {
        let heavy_multiplier = if attacker.fall_distance() > Self::SMASH_ATTACK_HEAVY_THRESHOLD {
            2.0
        } else {
            1.0
        };
        (Self::SMASH_ATTACK_KNOCKBACK_RADIUS - direction.length())
            * f64::from(Self::SMASH_ATTACK_KNOCKBACK_POWER)
            * heavy_multiplier
            * (1.0 - nearby.knockback_resistance())
    }

    fn should_knockback(
        attacker: &dyn LivingEntity,
        target: &dyn LivingEntity,
        nearby: &dyn LivingEntity,
    ) -> bool {
        if nearby.is_spectator() || nearby.id() == attacker.id() || nearby.id() == target.id() {
            return false;
        }
        if attacker.is_allied_to(nearby) || nearby.is_tame_owned_by(target) {
            return false;
        }
        if nearby.is_marker_armor_stand() {
            return false;
        }
        if nearby
            .as_player()
            .is_some_and(|player| player.has_infinite_materials() && player.is_flying())
        {
            return false;
        }

        target.position().distance_squared(nearby.position())
            <= Self::SMASH_ATTACK_KNOCKBACK_RADIUS * Self::SMASH_ATTACK_KNOCKBACK_RADIUS
    }

    fn apply_knockback(attacker: &dyn LivingEntity, target: &dyn LivingEntity) {
        let Some(world) = attacker.level() else {
            return;
        };

        let event_pos = target
            .on_pos(1.0e-5)
            .unwrap_or_else(|| target.block_position());
        world.level_event(
            level_events::PARTICLES_SMASH_ATTACK,
            event_pos,
            Self::SMASH_ATTACK_PARTICLE_DATA,
            None,
        );

        let search_box = target
            .bounding_box()
            .inflate(Self::SMASH_ATTACK_KNOCKBACK_RADIUS);
        for nearby in world.get_entities_in_aabb_matching(&search_box, |entity| {
            let Some(nearby) = entity.as_living_entity() else {
                return false;
            };
            Self::should_knockback(attacker, target, nearby)
        }) {
            let Some(nearby_living) = nearby.as_living_entity() else {
                continue;
            };
            let direction = nearby_living.position() - target.position();
            let knockback_power = Self::knockback_power(attacker, nearby_living, direction);
            if knockback_power <= 0.0 {
                continue;
            }

            let horizontal = if direction.length_squared() > 0.0 {
                direction.normalize() * knockback_power
            } else {
                DVec3::ZERO
            };
            nearby_living.push_impulse(DVec3::new(horizontal.x, f64::from(0.7_f32), horizontal.z));
            if let Some(player) = nearby_living.as_player() {
                player.send_packet(CSetEntityMotion::new(
                    nearby_living.id(),
                    nearby_living.velocity(),
                ));
            }
        }
    }
}

impl ItemBehavior for MaceItem {
    fn get_item_damage_source(&self, attacker: &SharedEntity) -> Option<DamageSource> {
        let living = attacker.as_living_entity()?;
        Self::can_smash_attack(living)
            .then(|| DamageSource::direct(&vanilla_damage_types::MACE_SMASH, Arc::clone(attacker)))
    }

    fn get_attack_damage_bonus(
        &self,
        attacker: &dyn LivingEntity,
        victim: &dyn Entity,
        _base_damage: f32,
        damage_source: &DamageSource,
    ) -> f32 {
        if !Self::can_smash_attack(attacker) {
            return 0.0;
        }

        let fall_distance = attacker.fall_distance();
        let damage = if fall_distance <= 3.0 {
            4.0 * fall_distance
        } else if fall_distance <= 8.0 {
            12.0 + 2.0 * (fall_distance - 3.0)
        } else {
            22.0 + fall_distance - 8.0
        };
        let context = EnchantmentDamageContext::new(
            victim.entity_type(),
            Some(attacker.entity_type()),
            Some(attacker.entity_type()),
            damage_source,
        );
        let mut damage_per_fallen_block = 0.0;
        attacker.with_equipment_slot(EquipmentSlot::MainHand, &mut |item| {
            damage_per_fallen_block =
                enchantment_helper::modify_smash_damage_per_fallen_block(item, &context, 0.0);
        });
        (damage + f64::from(damage_per_fallen_block) * fall_distance) as f32
    }

    fn hurt_enemy(
        &self,
        _stack: &mut ItemStack,
        target: &dyn LivingEntity,
        attacker: &dyn LivingEntity,
    ) {
        if !Self::can_smash_attack(attacker) {
            return;
        }

        let velocity = attacker.velocity();
        attacker.set_velocity(DVec3::new(velocity.x, f64::from(0.01_f32), velocity.z));
        attacker.set_ignore_fall_damage_from_current_impulse(
            true,
            Self::calculate_impact_position(attacker),
        );
        if let Some(player) = attacker.as_player() {
            player.send_packet(CSetEntityMotion::new(attacker.id(), attacker.velocity()));
        }

        if let Some(world) = attacker.level() {
            world.play_sound_at(
                Self::smash_sound(target, attacker),
                attacker.sound_source(),
                attacker.position(),
                1.0,
                1.0,
                None,
            );
        }
        Self::apply_knockback(attacker, target);
    }

    fn post_hurt_enemy(
        &self,
        _stack: &mut ItemStack,
        _target: &dyn LivingEntity,
        attacker: &dyn LivingEntity,
    ) {
        if Self::can_smash_attack(attacker) {
            attacker.reset_fall_distance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::behavior::{ITEM_BEHAVIORS, init_behaviors};
    use crate::test_support::{TestPlayerBuilder, fresh_test_world};
    use steel_registry::vanilla_items;

    #[test]
    fn registered_mace_smash_threshold_and_fall_flying_gate_damage_source() {
        crate::bootstrap::init_globals();
        init_behaviors();
        let fixture = fresh_test_world("mace_smash_source");
        let attacker = TestPlayerBuilder::new(Arc::clone(&fixture.world), "Mace", 1).build();
        let shared: SharedEntity = attacker.clone();
        let behavior = ITEM_BEHAVIORS.get_behavior(&vanilla_items::MACE);
        for (fall, flying, smash) in [
            (0.0, false, false),
            (1.5, false, false),
            (1.5001, false, true),
            (9.0, true, false),
        ] {
            attacker.set_fall_distance(fall);
            attacker.set_fall_flying(flying);
            let source = behavior.get_item_damage_source(&shared);
            assert_eq!(source.is_some(), smash, "fall={fall} flying={flying}");
            if let Some(source) = source {
                assert_eq!(source.damage_type.key, vanilla_damage_types::MACE_SMASH.key);
                assert!(Arc::ptr_eq(source.direct_entity().unwrap(), &shared));
            }
        }
    }

    #[test]
    fn registered_mace_damage_uses_all_three_vanilla_fall_distance_segments() {
        crate::bootstrap::init_globals();
        init_behaviors();
        let fixture = fresh_test_world("mace_smash_bonus");
        let attacker = TestPlayerBuilder::new(Arc::clone(&fixture.world), "Mace", 1).build();
        let victim = TestPlayerBuilder::new(Arc::clone(&fixture.world), "Target", 2).build();
        let source = DamageSource::direct(&vanilla_damage_types::MACE_SMASH, attacker.clone());
        let behavior = ITEM_BEHAVIORS.get_behavior(&vanilla_items::MACE);
        for (fall, expected) in [
            (1.5, 0.0),
            (2.0, 8.0),
            (3.0, 12.0),
            (5.0, 16.0),
            (8.0, 22.0),
            (9.0, 23.0),
        ] {
            attacker.set_fall_distance(fall);
            let bonus =
                behavior.get_attack_damage_bonus(attacker.as_ref(), victim.as_ref(), 6.0, &source);
            assert_eq!(bonus, expected, "fall={fall}");
        }
    }

    #[test]
    fn registered_mace_hit_preserves_horizontal_motion_and_resets_fall_after_hit() {
        crate::bootstrap::init_globals();
        init_behaviors();
        let fixture = fresh_test_world("mace_smash_motion");
        let attacker = TestPlayerBuilder::new(Arc::clone(&fixture.world), "Mace", 1).build();
        let victim = TestPlayerBuilder::new(Arc::clone(&fixture.world), "Target", 2).build();
        attacker
            .base()
            .set_position_local(DVec3::new(8.0, 70.0, 8.0));
        attacker.set_velocity(DVec3::new(0.25, -1.0, -0.5));
        attacker.set_fall_distance(6.0);
        let impact = DVec3::new(8.0, 65.0, 8.0);
        attacker.set_ignore_fall_damage_from_current_impulse(true, impact);
        let behavior = ITEM_BEHAVIORS.get_behavior(&vanilla_items::MACE);
        let mut stack = ItemStack::new(&vanilla_items::MACE);
        behavior.hurt_enemy(&mut stack, victim.as_ref(), attacker.as_ref());
        assert_eq!(
            attacker.velocity(),
            DVec3::new(0.25, f64::from(0.01_f32), -0.5)
        );
        assert_eq!(attacker.current_impulse_impact_pos(), Some(impact));
        assert_eq!(attacker.fall_distance(), 6.0);
        behavior.post_hurt_enemy(&mut stack, victim.as_ref(), attacker.as_ref());
        assert_eq!(attacker.fall_distance(), 0.0);
        attacker.set_velocity(DVec3::new(0.0, -0.5, 0.0));
        behavior.hurt_enemy(&mut stack, victim.as_ref(), attacker.as_ref());
        assert_eq!(attacker.velocity().y, -0.5);
    }

    #[test]
    fn player_attack_dispatches_mace_smash_and_post_hit_fall_reset() {
        crate::bootstrap::init_globals();
        init_behaviors();
        let fixture = fresh_test_world("mace_player_attack");
        let attacker = TestPlayerBuilder::new(Arc::clone(&fixture.world), "Mace", 1).build();
        let victim = TestPlayerBuilder::new(Arc::clone(&fixture.world), "Target", 2).build();
        attacker.set_client_loaded(true);
        victim.set_client_loaded(true);
        attacker
            .inventory
            .lock()
            .set_selected_item(ItemStack::new(&vanilla_items::MACE));
        attacker
            .attributes()
            .lock()
            .set_base_value(steel_registry::vanilla_attributes::ATTACK_DAMAGE, 6.0);
        attacker
            .attributes()
            .lock()
            .set_base_value(steel_registry::vanilla_attributes::ATTACK_SPEED, 4.0);
        victim
            .attributes()
            .lock()
            .set_base_value(steel_registry::vanilla_attributes::MAX_HEALTH, 100.0);
        victim.set_health(100.0);
        attacker.set_on_ground(false);
        attacker.set_fall_distance(6.0);
        attacker.set_velocity(DVec3::new(0.0, -1.0, 0.0));
        for _ in 0..30 {
            attacker.tick_attack_strength();
        }
        let target: SharedEntity = victim.clone();
        assert!(attacker.attack(&target));
        // Vanilla: (base 6 + six-block smash bonus 18) * critical 1.5.
        assert_eq!(victim.get_health(), 64.0);
        assert_eq!(attacker.fall_distance(), 0.0);
        assert_eq!(attacker.velocity().y, f64::from(0.01_f32));
        assert!(attacker.is_ignoring_fall_damage_from_current_impulse());
    }

    #[test]
    fn smash_knockback_uses_heavy_threshold_radius_and_resistance() {
        crate::bootstrap::init_globals();
        let fixture = fresh_test_world("mace_radial_knockback");
        let attacker = TestPlayerBuilder::new(Arc::clone(&fixture.world), "Mace", 1).build();
        let nearby = TestPlayerBuilder::new(Arc::clone(&fixture.world), "Nearby", 2).build();
        nearby.attributes().lock().set_base_value(
            steel_registry::vanilla_attributes::KNOCKBACK_RESISTANCE,
            0.5,
        );
        for (fall, distance, expected) in [
            (5.0, 1.5, f64::from(0.7_f32)),
            (5.001, 1.5, 2.0 * f64::from(0.7_f32)),
            (8.0, 3.5, 0.0),
        ] {
            attacker.set_fall_distance(fall);
            assert_eq!(
                MaceItem::knockback_power(
                    attacker.as_ref(),
                    nearby.as_ref(),
                    DVec3::new(distance, 0.0, 0.0)
                ),
                expected
            );
        }
        nearby.attributes().lock().set_base_value(
            steel_registry::vanilla_attributes::KNOCKBACK_RESISTANCE,
            1.0,
        );
        assert_eq!(
            MaceItem::knockback_power(attacker.as_ref(), nearby.as_ref(), DVec3::X),
            0.0
        );
        assert!(!MaceItem::should_knockback(
            attacker.as_ref(),
            nearby.as_ref(),
            attacker.as_ref()
        ));
        assert!(!MaceItem::should_knockback(
            attacker.as_ref(),
            nearby.as_ref(),
            nearby.as_ref()
        ));
    }
}
