use steel_utils::Downcast as _;

use super::super::prelude::*;
use super::super::runner::FeatureDecorationRunner;

impl FeatureDecorationRunner {
    pub(in crate::worldgen::feature) fn place_end_gateway_feature(
        region: &WorldGenRegion<'_>,
        config: &EndGatewayConfiguration,
        origin: BlockPos,
    ) -> bool {
        for dy in -2..=2 {
            for dx in -1..=1 {
                for dz in -1..=1 {
                    let pos = origin.offset(dx, dy, dz);
                    let same_x = dx == 0;
                    let same_y = dy == 0;
                    let same_z = dz == 0;
                    let end = dy.abs() == 2;
                    if same_x && same_y && same_z {
                        let state = vanilla_blocks::END_GATEWAY.default_state();
                        let _ = region.set_block_state(pos, state, UpdateFlags::UPDATE_ALL);
                        let exit = config.exit.map(BlockPos);
                        Self::set_end_gateway_block_entity(region, pos, state, exit, config.exact);
                    } else if same_y {
                        let _ = region.set_block_state(
                            pos,
                            vanilla_blocks::AIR.default_state(),
                            UpdateFlags::UPDATE_ALL,
                        );
                    } else if (end && same_x && same_z) || ((same_x || same_z) && !end) {
                        let _ = region.set_block_state(
                            pos,
                            vanilla_blocks::BEDROCK.default_state(),
                            UpdateFlags::UPDATE_ALL,
                        );
                    } else {
                        let _ = region.set_block_state(
                            pos,
                            vanilla_blocks::AIR.default_state(),
                            UpdateFlags::UPDATE_ALL,
                        );
                    }
                }
            }
        }

        true
    }
}

/// Vanilla `EndGatewayFeature.place` for a runtime world.
///
/// `EnderDragonFight.spawnNewGateway` places the `end_gateway_delayed` feature, so
/// the fight uses this instead of writing the bedrock capsule by hand. The block
/// entity is created the same way the worldgen path creates it, which is what
/// gives the gateway its exit search on first teleport.
pub(crate) fn place_end_gateway_in_world(
    world: &Arc<crate::world::World>,
    origin: BlockPos,
    config: &EndGatewayConfiguration,
) {
    let gateway = REGISTRY
        .blocks
        .get_default_state_id(&vanilla_blocks::END_GATEWAY);
    let bedrock = REGISTRY
        .blocks
        .get_default_state_id(&vanilla_blocks::BEDROCK);
    let air = REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR);

    for dy in -2..=2 {
        for dx in -1..=1 {
            for dz in -1..=1 {
                let pos = origin.offset(dx, dy, dz);
                let same_x = dx == 0;
                let same_y = dy == 0;
                let same_z = dz == 0;
                let end = dy.abs() == 2;
                let state = if same_x && same_y && same_z {
                    gateway
                } else if same_y {
                    air
                } else if (end && same_x && same_z) || ((same_x || same_z) && !end) {
                    bedrock
                } else {
                    air
                };
                let _ = world.set_block(pos, state, UpdateFlags::UPDATE_ALL);
                if same_x
                    && same_y
                    && same_z
                    && let Some(exit) = config.exit
                    && let Some(block_entity) = world.get_block_entity(pos)
                    && let Some(gateway) = block_entity
                        .downcast_ref::<crate::block_entity::entities::EndGatewayBlockEntity>()
                {
                    let exit = BlockPos::new(exit[0], exit[1], exit[2]);
                    gateway.set_exit_position(exit, config.exact);
                }
            }
        }
    }
}
