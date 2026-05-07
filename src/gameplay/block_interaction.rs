use crate::voxel::VoxelCoord;
use crate::world::{PlanetData, VoxelEditResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockActionIntent {
    Mine,
    Place,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockAction {
    Mine(VoxelCoord),
    Place(VoxelCoord),
}

pub struct BlockInteraction;

impl BlockInteraction {
    pub fn resolve(
        intent: BlockActionIntent,
        selected: Option<VoxelCoord>,
        placement: Option<VoxelCoord>,
    ) -> Option<BlockAction> {
        match intent {
            BlockActionIntent::Mine => selected.map(BlockAction::Mine),
            BlockActionIntent::Place => {
                selected?;
                placement.map(BlockAction::Place)
            }
        }
    }

    pub fn apply(action: BlockAction, planet: &mut PlanetData) -> VoxelEditResult {
        match action {
            BlockAction::Mine(id) => planet.remove_block(id),
            BlockAction::Place(id) => planet.add_block(id),
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
            BlockInteraction::resolve(BlockActionIntent::Mine, Some(coord(3)), Some(coord(4))),
            Some(BlockAction::Mine(coord(3)))
        );
    }

    #[test]
    fn placing_uses_placement_candidate() {
        assert_eq!(
            BlockInteraction::resolve(BlockActionIntent::Place, Some(coord(3)), Some(coord(4))),
            Some(BlockAction::Place(coord(4)))
        );
    }

    #[test]
    fn placing_requires_selected_voxel() {
        assert_eq!(
            BlockInteraction::resolve(BlockActionIntent::Place, None, Some(coord(4))),
            None
        );
    }
}
