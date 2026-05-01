fn palette_color(visual: BlockVisual, selector: f32) -> vec3<f32> {
    let len = max(visual.palette.y, 1u);
    let index = min(u32(floor(selector * f32(len))), len - 1u);
    return block_visual_palette[visual.palette.x + index].rgb;
}

fn visual_for(block_visual_id: u32) -> BlockVisual {
    return block_visuals[block_visual_id];
}

fn face_visual_for(visual: BlockVisual, face_id: u32) -> BlockFaceVisual {
    switch min(face_id, 5u) {
        case 0u: { return visual.faces[0]; }
        case 1u: { return visual.faces[1]; }
        case 2u: { return visual.faces[2]; }
        case 3u: { return visual.faces[3]; }
        case 4u: { return visual.faces[4]; }
        default: { return visual.faces[5]; }
    }
}

fn detail_for(visual: BlockVisual, index: u32) -> BlockDetail {
    switch index {
        case 0u: { return visual.details[0]; }
        case 1u: { return visual.details[1]; }
        case 2u: { return visual.details[2]; }
        case 3u: { return visual.details[3]; }
        case 4u: { return visual.details[4]; }
        case 5u: { return visual.details[5]; }
        case 6u: { return visual.details[6]; }
        default: { return visual.details[7]; }
    }
}

fn detail_enabled(mask: u32, index: u32) -> bool {
    return (mask & (1u << index)) != 0u;
}

fn default_or_face_color(face_visual: BlockFaceVisual, fallback: vec3<f32>) -> vec3<f32> {
    let delta = length(face_visual.color_bias.rgb - vec3<f32>(1.0));
    if (delta > 0.01) {
        return face_visual.color_bias.rgb;
    }
    return fallback;
}