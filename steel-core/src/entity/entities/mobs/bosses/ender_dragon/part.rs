//! The dragon's eight hitbox parts (`EnderDragonPart`).
//!
//! Vanilla models the dragon as one `EnderDragon` plus eight `EnderDragonPart`
//! entities that follow the parent every tick. Which part was hit decides the
//! damage: the head and neck take full damage, every other part takes
//! `damage / 4 + min(damage, 1)`. The parts are `isPickable`, are registered in
//! `ServerLevel.dragonParts` so level-wide entity queries find them, and are
//! never saved.
//!
//! Steel has no generic sub-entity concept, so the parts are per-dragon state
//! rather than extra entity-manager entries:
//!
//! * **Geometry is derived, never stored.** [`DragonParts::bounding_box`] computes
//!   a part's box from the parent position plus the offsets the parent recomputes
//!   every tick, so the parts cannot drift out of sync and need no resync.
//! * **Hit routing.** [`DragonParts::hit_part`] answers "which part does this ray
//!   hit", playing the role vanilla's `Level.getEntities` + `EntityHitResult`
//!   gives for parts.
//! * **Wing sweep.** [`DragonParts::wing_sweep_boxes`] exposes the two enlarged
//!   wing boxes vanilla uses to shove and damage entities the dragon flies past.
//!
//! Deviation from vanilla, recorded in `STATE-26.3-RU.md`: because the parts are
//! not entities they do not appear in `Level.getEntities` results, in
//! `getEntity(id)` lookups, or in entity-tracking packets.

use glam::DVec3;
use steel_registry::entity_type::EntityDimensions;
use steel_utils::{WorldAabb, axis::Axis};

/// Index of the head part.
pub const DRAGON_PART_HEAD: usize = 0;
/// Index of the neck part.
pub const DRAGON_PART_NECK: usize = 1;
/// Index of the body part.
pub const DRAGON_PART_BODY: usize = 2;
/// Index of the first tail part.
pub const DRAGON_PART_TAIL1: usize = 3;
/// Index of the second tail part.
pub const DRAGON_PART_TAIL2: usize = 4;
/// Index of the third tail part.
pub const DRAGON_PART_TAIL3: usize = 5;
/// Index of the first wing.
pub const DRAGON_PART_WING1: usize = 6;
/// Index of the second wing.
pub const DRAGON_PART_WING2: usize = 7;

/// Number of parts, matching vanilla `EnderDragon.subEntities.length`.
pub const DRAGON_PART_COUNT: usize = 8;

/// Height of the head part, i.e. `EnderDragon.head.getBbHeight()`.
pub const DRAGON_PART_HEAD_HEIGHT: f32 = 1.0;

/// One dragon hitbox part: its vanilla name and size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragonPart {
    /// Vanilla `EnderDragonPart.name`.
    pub name: &'static str,
    /// The part's width in blocks.
    pub width: f32,
    /// The part's height in blocks.
    pub height: f32,
}

impl DragonPart {
    /// Returns this part's size as a scalar entity dimension.
    #[must_use]
    pub const fn dimensions(self) -> EntityDimensions {
        EntityDimensions::new(self.width, self.height, self.height)
    }
}

/// The eight parts in vanilla `subEntities` order.
///
/// The order is load-bearing: vanilla numbers the part entities as
/// `dragon.getId() + 1 + index`, and the head/tail math indexes into it.
pub const DRAGON_PARTS: [DragonPart; DRAGON_PART_COUNT] = [
    DragonPart {
        name: "head",
        width: 1.0,
        height: 1.0,
    },
    DragonPart {
        name: "neck",
        width: 3.0,
        height: 3.0,
    },
    DragonPart {
        name: "body",
        width: 5.0,
        height: 3.0,
    },
    DragonPart {
        name: "tail",
        width: 2.0,
        height: 2.0,
    },
    DragonPart {
        name: "tail",
        width: 2.0,
        height: 2.0,
    },
    DragonPart {
        name: "tail",
        width: 2.0,
        height: 2.0,
    },
    DragonPart {
        name: "wing",
        width: 4.0,
        height: 2.0,
    },
    DragonPart {
        name: "wing",
        width: 4.0,
        height: 2.0,
    },
];

/// The dragon's eight part hitboxes, as absolute world positions.
///
/// [`super::phases::run_ai_step`] rewrites every position once per AI step.
#[derive(Debug, Clone)]
pub struct DragonParts {
    positions: [DVec3; DRAGON_PART_COUNT],
}

impl Default for DragonParts {
    fn default() -> Self {
        Self::new()
    }
}

impl DragonParts {
    /// Creates parts at the origin, before the first tick places them.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            positions: [DVec3::ZERO; DRAGON_PART_COUNT],
        }
    }

    /// Creates parts centred on `position`, like a freshly spawned dragon.
    #[must_use]
    pub const fn at(position: DVec3) -> Self {
        Self {
            positions: [position; DRAGON_PART_COUNT],
        }
    }

    /// Returns the absolute position of `index`, or `None` when out of range.
    #[must_use]
    pub fn position(&self, index: usize) -> Option<DVec3> {
        self.positions.get(index).copied()
    }

    /// Returns every part position in `subEntities` order.
    #[must_use]
    pub const fn all_positions(&self) -> &[DVec3; DRAGON_PART_COUNT] {
        &self.positions
    }

    /// Vanilla `EnderDragon.tickPart`: places one part relative to the parent.
    pub fn tick_part(&mut self, index: usize, dragon_position: DVec3, offset: DVec3) {
        if let Some(slot) = self.positions.get_mut(index) {
            *slot = dragon_position + offset;
        }
    }

    /// Vanilla `EnderDragonPart.makeBoundingBox`, in `Entity.makeBoundingBox`
    /// shape: centred horizontally on the part's position, rising from it.
    #[must_use]
    pub fn bounding_box(&self, index: usize) -> Option<WorldAabb> {
        let part = DRAGON_PARTS.get(index)?;
        let position = self.position(index)?;
        let half_width = f64::from(part.width) / 2.0;
        Some(WorldAabb::new(
            position.x - half_width,
            position.y,
            position.z - half_width,
            position.x + half_width,
            position.y + f64::from(part.height),
            position.z + half_width,
        ))
    }

    /// The head position, which the breath attack measures from.
    #[must_use]
    pub fn head_position(&self) -> DVec3 {
        self.positions[DRAGON_PART_HEAD]
    }

    /// The middle of the head box: vanilla's `head.getY(0.5)`.
    #[must_use]
    pub fn head_center(&self) -> DVec3 {
        let head = self.positions[DRAGON_PART_HEAD];
        head + DVec3::new(0.0, f64::from(DRAGON_PARTS[DRAGON_PART_HEAD].height) / 2.0, 0.0)
    }

    /// The two enlarged wing boxes vanilla sweeps for push and damage.
    ///
    /// `wing.getBoundingBox().inflate(4.0, 2.0, 4.0).move(0.0, -2.0, 0.0)`.
    #[must_use]
    pub fn wing_sweep_boxes(&self) -> [Option<WorldAabb>; 2] {
        [DRAGON_PART_WING1, DRAGON_PART_WING2].map(|index| {
            self.bounding_box(index).map(|box_| {
                box_
                    .inflate_xyz(4.0, 2.0, 4.0)
                    .translate(DVec3::new(0.0, -2.0, 0.0))
            })
        })
    }

    /// Tests a segment against every part and returns the closest hit.
    ///
    /// Mirrors what vanilla gets from `ProjectileUtil.getEntityHitResult` over the
    /// part entities: the earliest intersection along the segment wins, and a
    /// segment that starts inside a box hits it immediately.
    #[must_use]
    pub fn hit_part(&self, from: DVec3, to: DVec3) -> Option<usize> {
        let mut best: Option<(usize, f64)> = None;
        for index in 0..DRAGON_PART_COUNT {
            let Some(box_) = self.bounding_box(index) else {
                continue;
            };
            let Some(distance) = segment_box_entry_distance(box_, from, to) else {
                continue;
            };
            if best.is_none_or(|(_, current)| distance < current) {
                best = Some((index, distance));
            }
        }
        best.map(|(index, _)| index)
    }

    /// Returns the part whose box contains `point`.
    #[must_use]
    pub fn part_containing(&self, point: DVec3) -> Option<usize> {
        (0..DRAGON_PART_COUNT).find(|index| {
            self.bounding_box(*index)
                .is_some_and(|box_| box_.contains(point))
        })
    }
}

/// Vanilla `EnderDragon.hurt`'s per-part scaling.
///
/// Returns `(multiplier, flat)` so callers apply `damage * multiplier +
/// min(damage, flat)`: the head and neck keep full damage, everything else drops
/// to a quarter plus at most one point.
#[must_use]
pub const fn part_damage_scaling(index: usize) -> (f32, f32) {
    if index == DRAGON_PART_HEAD || index == DRAGON_PART_NECK {
        (1.0, 0.0)
    } else {
        (0.25, 1.0)
    }
}

/// Returns the distance along `from -> to` at which the segment first enters
/// `box_`, or `None` when it misses.
///
/// This is the slab test vanilla's `AABB.clip` performs.
#[must_use]
pub fn segment_box_entry_distance(box_: WorldAabb, from: DVec3, to: DVec3) -> Option<f64> {
    let direction = to - from;
    let mut entry: f64 = 0.0;
    let mut exit: f64 = 1.0;

    for axis in [Axis::X, Axis::Y, Axis::Z] {
        let start = axis_component(from, axis);
        let delta = axis_component(direction, axis);
        let (min, max) = (box_.min(axis), box_.max(axis));

        if delta.abs() < f64::EPSILON {
            // Parallel to this slab: either always inside it or never.
            if start < min || start > max {
                return None;
            }
            continue;
        }

        let inv = 1.0 / delta;
        let mut t1 = (min - start) * inv;
        let mut t2 = (max - start) * inv;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }
        entry = entry.max(t1);
        exit = exit.min(t2);
        if entry > exit {
            return None;
        }
    }

    Some(entry * direction.length())
}

const fn axis_component(vector: DVec3, axis: Axis) -> f64 {
    match axis {
        Axis::X => vector.x,
        Axis::Y => vector.y,
        Axis::Z => vector.z,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part_sizes_match_vanilla_sub_entities() {
        let names: Vec<_> = DRAGON_PARTS.iter().map(|part| part.name).collect();
        assert_eq!(
            names,
            ["head", "neck", "body", "tail", "tail", "tail", "wing", "wing"]
        );

        let sizes: Vec<_> = DRAGON_PARTS
            .iter()
            .map(|part| (part.width, part.height))
            .collect();
        assert_eq!(
            sizes,
            [
                (1.0, 1.0),
                (3.0, 3.0),
                (5.0, 3.0),
                (2.0, 2.0),
                (2.0, 2.0),
                (2.0, 2.0),
                (4.0, 2.0),
                (4.0, 2.0),
            ]
        );
    }

    #[test]
    fn only_the_head_and_neck_avoid_the_part_damage_penalty() {
        assert_eq!(part_damage_scaling(DRAGON_PART_HEAD), (1.0, 0.0));
        assert_eq!(part_damage_scaling(DRAGON_PART_NECK), (1.0, 0.0));
        for index in [DRAGON_PART_BODY, DRAGON_PART_TAIL1, DRAGON_PART_WING1] {
            assert_eq!(part_damage_scaling(index), (0.25, 1.0));
        }
    }

    #[test]
    fn part_boxes_use_the_vanilla_entity_box_shape() {
        let mut parts = DragonParts::new();
        parts.tick_part(DRAGON_PART_BODY, DVec3::new(0.0, 100.0, 0.0), DVec3::ZERO);

        let box_ = parts.bounding_box(DRAGON_PART_BODY).expect("body exists");
        assert_eq!(box_.min_x(), -2.5);
        assert_eq!(box_.max_x(), 2.5);
        assert_eq!(box_.min_y(), 100.0);
        assert_eq!(box_.max_y(), 103.0);
    }

    #[test]
    fn a_segment_reports_the_nearest_part_it_crosses() {
        let mut parts = DragonParts::new();
        let origin = DVec3::new(0.0, 100.0, 0.0);
        parts.tick_part(DRAGON_PART_BODY, origin, DVec3::new(10.0, 0.0, 0.0));
        parts.tick_part(DRAGON_PART_HEAD, origin, DVec3::new(20.0, 0.0, 0.0));
        // Park every other part far away so only body and head can be hit.
        for index in [
            DRAGON_PART_NECK,
            DRAGON_PART_TAIL1,
            DRAGON_PART_TAIL2,
            DRAGON_PART_TAIL3,
            DRAGON_PART_WING1,
            DRAGON_PART_WING2,
        ] {
            parts.tick_part(index, DVec3::new(0.0, 500.0, 0.0), DVec3::ZERO);
        }

        assert_eq!(
            parts.hit_part(DVec3::new(-10.0, 100.5, 0.0), DVec3::new(40.0, 100.5, 0.0)),
            Some(DRAGON_PART_BODY),
            "the nearer body box wins"
        );
        assert_eq!(
            parts.hit_part(DVec3::new(15.0, 100.5, 0.0), DVec3::new(40.0, 100.5, 0.0)),
            Some(DRAGON_PART_HEAD)
        );
        assert_eq!(
            parts.hit_part(DVec3::new(-10.0, 300.0, 0.0), DVec3::new(40.0, 300.0, 0.0)),
            None
        );
    }

    #[test]
    fn a_segment_starting_inside_a_part_hits_it_immediately() {
        let parts = DragonParts::at(DVec3::new(0.0, 100.0, 0.0));
        let inside = DVec3::new(0.0, 100.5, 0.0);

        assert_eq!(
            parts.hit_part(inside, DVec3::new(0.0, 300.0, 0.0)),
            Some(DRAGON_PART_HEAD),
            "the head is first in subEntities order, so it wins the tie"
        );
        assert_eq!(parts.part_containing(inside), Some(DRAGON_PART_HEAD));
    }

    #[test]
    fn wing_sweep_boxes_grow_and_drop_like_vanilla() {
        let mut parts = DragonParts::new();
        parts.tick_part(DRAGON_PART_WING1, DVec3::new(0.0, 100.0, 0.0), DVec3::ZERO);

        let box_ = parts.bounding_box(DRAGON_PART_WING1).expect("wing exists");
        let sweep = parts.wing_sweep_boxes()[0].expect("wing sweep exists");

        // inflate(4, 2, 4) then move(0, -2, 0).
        assert_eq!(sweep.min_x(), box_.min_x() - 4.0);
        assert_eq!(sweep.max_x(), box_.max_x() + 4.0);
        assert_eq!(sweep.min_y(), box_.min_y() - 2.0 - 2.0);
        assert_eq!(sweep.max_y(), box_.max_y() + 2.0 - 2.0);
    }
}
