use vv_registry::{RuntimeBlockVisual, RUNTIME_SURFACE_PROGRAM_PATTERNED};

pub(crate) fn visual_uses_patterned(visual: &RuntimeBlockVisual) -> bool {
    visual.procedural[3] == RUNTIME_SURFACE_PROGRAM_PATTERNED
}
