fn face_color_bias(block_visual_id: u32, face_id: u32) -> vec3<f32> {
    let visual = visual_for(block_visual_id);

    // Runtime face order:
    // 0 = top
    // 1 = bottom
    // 2 = north/front
    // 3 = south/back
    // 4 = east/right
    // 5 = west/left
    //
    // Mesh face_id order:
    // 0 = top
    // 1 = bottom
    // 2 = front/north
    // 3 = back/south
    // 4 = left/west
    // 5 = right/east

    if (face_id == 0u) {
        return visual.faces[0].color_bias.rgb;
    }

    if (face_id == 1u) {
        return visual.faces[1].color_bias.rgb;
    }

    if (face_id == 2u) {
        return visual.faces[2].color_bias.rgb;
    }

    if (face_id == 3u) {
        return visual.faces[3].color_bias.rgb;
    }

    if (face_id == 4u) {
        return visual.faces[5].color_bias.rgb;
    }

    if (face_id == 5u) {
        return visual.faces[4].color_bias.rgb;
    }

    return vec3<f32>(1.0);
}

fn flat_block_albedo(block_visual_id: u32, face_id: u32) -> vec3<f32> {
    let visual = visual_for(block_visual_id);

    var color = visual.base_color.rgb;
    color = color * face_color_bias(block_visual_id, face_id);

    return max(color, vec3<f32>(0.0));
}

fn procedural_block_albedo(
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    voxel_pos: vec3<i32>,
    variation_seed: u32,
    up: vec3<f32>,
) -> vec3<f32> {
    // Étape flat pure.
    // On garde tous les paramètres pour ne pas casser l'API shader,
    // mais on ignore volontairement le bruit, les textures et les détails.
    return flat_block_albedo(block_visual_id, face_id);
}
