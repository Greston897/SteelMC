use super::super::prelude::*;
use super::super::runner::FeatureDecorationRunner;

impl FeatureDecorationRunner {
    pub(in crate::worldgen::feature) fn place_end_podium_feature(
        region: &mut WorldGenRegion<'_>,
        origin: BlockPos,
        active: bool,
    ) -> bool {
        let bedrock = vanilla_blocks::BEDROCK.default_state();
        let end_stone = vanilla_blocks::END_STONE.default_state();
        let air = vanilla_blocks::AIR.default_state();
        let end_portal = vanilla_blocks::END_PORTAL.default_state();

        for pos in BlockPos::between_closed(origin.offset(-4, -1, -4), origin.offset(4, 32, 4)) {
            let inside_rim = Self::end_podium_closer_than(pos, origin, 2.5);
            if !inside_rim && !Self::end_podium_closer_than(pos, origin, 3.5) {
                continue;
            }

            if pos.y() < origin.y() {
                if inside_rim {
                    let _ = region.set_block_state(pos, bedrock, UpdateFlags::UPDATE_CLIENTS);
                } else if active {
                    Self::drop_previous_and_set(region, pos, end_stone);
                } else {
                    let _ = region.set_block_state(pos, end_stone, UpdateFlags::UPDATE_CLIENTS);
                }
            } else if pos.y() > origin.y() {
                if active {
                    Self::drop_previous_and_set(region, pos, air);
                } else {
                    let _ = region.set_block_state(pos, air, UpdateFlags::UPDATE_CLIENTS);
                }
            } else if !inside_rim {
                let _ = region.set_block_state(pos, bedrock, UpdateFlags::UPDATE_CLIENTS);
            } else if active {
                Self::drop_previous_and_set(region, pos, end_portal);
            } else {
                let _ = region.set_block_state(pos, air, UpdateFlags::UPDATE_CLIENTS);
            }
        }

        for y in 0..4 {
            let _ = region.set_block_state(origin.above_n(y), bedrock, UpdateFlags::UPDATE_CLIENTS);
        }

        let center = origin.above_n(2);
        for facing in Self::VANILLA_HORIZONTAL_DIRECTIONS {
            let torch = vanilla_blocks::WALL_TORCH
                .default_state()
                .set_value(&BlockStateProperties::HORIZONTAL_FACING, facing);
            let _ =
                region.set_block_state(center.relative(facing), torch, UpdateFlags::UPDATE_CLIENTS);
        }

        true
    }

    fn end_podium_closer_than(pos: BlockPos, origin: BlockPos, distance: f64) -> bool {
        let dx = f64::from(pos.x() - origin.x());
        let dy = f64::from(pos.y() - origin.y());
        let dz = f64::from(pos.z() - origin.z());
        dx * dx + dy * dy + dz * dz < distance * distance
    }

    fn drop_previous_and_set(region: &mut WorldGenRegion<'_>, pos: BlockPos, state: BlockStateId) {
        if region.block_state(pos).get_block() == state.get_block() {
            return;
        }

        let _ = region.destroy_block(pos, true);
        let _ = region.set_block_state(pos, state, UpdateFlags::UPDATE_CLIENTS);
    }
}

/// Vanilla `EndPodiumFeature.place` for a runtime world.
///
/// The fight's exit-portal spawn is exactly the worldgen feature with an
/// `active` flag, so it is executed here against the live world rather than
/// rebuilt with a second block loop that could drift from worldgen.
pub(crate) fn place_end_podium_in_world(
    world: &Arc<crate::world::World>,
    origin: BlockPos,
    active: bool,
) {
    let bedrock = REGISTRY
        .blocks
        .get_default_state_id(&vanilla_blocks::BEDROCK);
    let end_stone = REGISTRY
        .blocks
        .get_default_state_id(&vanilla_blocks::END_STONE);
    let air = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR);
    let end_portal = REGISTRY
        .blocks
        .get_default_state_id(&vanilla_blocks::END_PORTAL);

    let closer_than = |pos: BlockPos, distance: f64| {
        let dx = f64::from(pos.x() - origin.x());
        let dy = f64::from(pos.y() - origin.y());
        let dz = f64::from(pos.z() - origin.z());
        dx * dx + dy * dy + dz * dz < distance * distance
    };

    for pos in BlockPos::between_closed(origin.offset(-4, -1, -4), origin.offset(4, 32, 4)) {
        let inside_rim = closer_than(pos, 2.5);
        if !inside_rim && !closer_than(pos, 3.5) {
            continue;
        }

        if pos.y() < origin.y() {
            if inside_rim {
                let _ = world.set_block(pos, bedrock, UpdateFlags::UPDATE_CLIENTS);
            } else if active {
                let _ = world.destroy_block(pos, true);
                let _ = world.set_block(pos, end_stone, UpdateFlags::UPDATE_CLIENTS);
            } else {
                let _ = world.set_block(pos, end_stone, UpdateFlags::UPDATE_CLIENTS);
            }
        } else if pos.y() > origin.y() {
            if active {
                let _ = world.destroy_block(pos, true);
            }
            let _ = world.set_block(pos, air, UpdateFlags::UPDATE_CLIENTS);
        } else if !inside_rim {
            let _ = world.set_block(pos, bedrock, UpdateFlags::UPDATE_CLIENTS);
        } else if active {
            let _ = world.destroy_block(pos, true);
            let _ = world.set_block(pos, end_portal, UpdateFlags::UPDATE_CLIENTS);
        } else {
            let _ = world.set_block(pos, air, UpdateFlags::UPDATE_CLIENTS);
        }
    }

    for y in 0..4 {
        let _ = world.set_block(origin.above_n(y), bedrock, UpdateFlags::UPDATE_CLIENTS);
    }

    let center = origin.above_n(2);
    for facing in FeatureDecorationRunner::VANILLA_HORIZONTAL_DIRECTIONS {
        let torch = REGISTRY.blocks.get_default_state_id(
            vanilla_blocks::WALL_TORCH
                .default_state()
                .set_value(&BlockStateProperties::HORIZONTAL_FACING, facing),
        );
        let _ = world.set_block(center.relative(facing), torch, UpdateFlags::UPDATE_CLIENTS);
    }
}
