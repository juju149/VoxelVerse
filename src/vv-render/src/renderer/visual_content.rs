use vv_registry::{CompiledContent, RuntimeBlockVisual};

pub(crate) fn build_block_visuals(content: &CompiledContent) -> Vec<RuntimeBlockVisual> {
    let mut visuals = Vec::with_capacity(content.block_visuals.len().max(1));

    for visual in content.block_visuals.entries() {
        visuals.push(*visual);
    }

    if visuals.is_empty() {
        visuals.push(RuntimeBlockVisual::fallback());
    }

    visuals
}

pub(crate) fn build_block_visual_palette(content: &CompiledContent) -> Vec<[f32; 4]> {
    let mut palette = content.block_visual_palettes.clone();

    if palette.is_empty() {
        palette.push([0.55, 0.55, 0.55, 1.0]);
    }

    palette
}