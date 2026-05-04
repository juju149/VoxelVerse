use vv_registry::{RuntimeBlockVisual, RUNTIME_SURFACE_PROGRAM_PATTERNED};

pub(crate) fn visual_uses_patterned(visual: &RuntimeBlockVisual) -> bool {
    // VoxelForge model layers are compiled into the current patterned runtime path.
    // This keeps the first vertical slice simple: complete .ron -> compiled visual -> mesh/rander path.
    visual.procedural[3] == RUNTIME_SURFACE_PROGRAM_PATTERNED
}
