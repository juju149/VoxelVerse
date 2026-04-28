use vv_core::BlockId as VoxelId;
use vv_registry::{BlockId as ContentBlockId, CompiledContent, CompiledItemKind};
use vv_world_runtime::PlanetData;

use crate::Inventory;

pub fn selected_placeable_block(
    inventory: &Inventory,
    selected_hotbar_slot: usize,
    content: &CompiledContent,
) -> Option<ContentBlockId> {
    let stack = inventory.selected_stack(selected_hotbar_slot)?;
    match content.items.get(stack.item)?.kind {
        CompiledItemKind::Block { block } => Some(block),
        _ => None,
    }
}

pub fn can_place_block(planet: &PlanetData, id: VoxelId) -> bool {
    !planet.exists(id) && id.layer < planet.resolution
}
