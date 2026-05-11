use crate::voxel::{VoxelCoord, VoxelId};
use crate::world::{PlanetData, VoxelEditResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockActionIntent {
    Mine,
    Place,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockAction {
    Mine(VoxelCoord),
    Place { coord: VoxelCoord, voxel: VoxelId },
}

pub struct BlockInteraction;

impl BlockInteraction {
    pub fn resolve(
        intent: BlockActionIntent,
        selected: Option<VoxelCoord>,
        placement: Option<VoxelCoord>,
        place_voxel: Option<VoxelId>,
    ) -> Option<BlockAction> {
        match intent {
            BlockActionIntent::Mine => selected.map(BlockAction::Mine),
            BlockActionIntent::Place => {
                selected?;
                Some(BlockAction::Place {
                    coord: placement?,
                    voxel: place_voxel?,
                })
            }
        }
    }

    pub fn apply(action: BlockAction, planet: &mut PlanetData) -> VoxelEditResult {
        match action {
            BlockAction::Mine(id) => planet.remove_block(id),
            BlockAction::Place { coord, voxel } => planet.place_block(coord, voxel),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockAction, BlockActionIntent, BlockInteraction};
    use crate::voxel::VoxelCoord;

    fn coord(layer: u32) -> VoxelCoord {
        VoxelCoord {
            face: 0,
            layer,
            u: 1,
            v: 2,
        }
    }

    #[test]
    fn mining_uses_selected_voxel() {
        assert_eq!(
            BlockInteraction::resolve(
                BlockActionIntent::Mine,
                Some(coord(3)),
                Some(coord(4)),
                None
            ),
            Some(BlockAction::Mine(coord(3)))
        );
    }

    #[test]
    fn placing_uses_placement_candidate() {
        assert_eq!(
            BlockInteraction::resolve(
                BlockActionIntent::Place,
                Some(coord(3)),
                Some(coord(4)),
                Some(crate::voxel::VoxelId::new(5))
            ),
            Some(BlockAction::Place {
                coord: coord(4),
                voxel: crate::voxel::VoxelId::new(5)
            })
        );
    }

    #[test]
    fn placing_requires_selected_voxel() {
        assert_eq!(
            BlockInteraction::resolve(
                BlockActionIntent::Place,
                None,
                Some(coord(4)),
                Some(crate::voxel::VoxelId::new(5))
            ),
            None
        );
    }

    #[test]
    fn placing_requires_active_voxel() {
        assert_eq!(
            BlockInteraction::resolve(
                BlockActionIntent::Place,
                Some(coord(3)),
                Some(coord(4)),
                None
            ),
            None
        );
    }
}
