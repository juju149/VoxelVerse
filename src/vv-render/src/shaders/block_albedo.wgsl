fn procedural_block_albedo(
    world_pos: vec3<f32>,
    world_normal: vec3<f32>,
    uv: vec2<f32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    voxel_pos: vec3<i32>,
    variation_seed: u32,
    up_dir: vec3<f32>,
) -> vec3<f32> {
    let visual = visual_for(block_visual_id);
    let face_visual = face_visual_for(visual, face_id);

    let grid_size = max(1u, visual.procedural.x);
    let grid_size_f = f32(grid_size);
    let cell = clamp(floor(uv * grid_size_f), vec2<f32>(0.0), vec2<f32>(grid_size_f - 1.0));
    let seed = face_seed(voxel_pos, block_id, block_visual_id, face_id, variation_seed, cell, 0.0);

    let cell_hash = hash13(seed);
    let face_hash = hash13(seed + vec3<f32>(3.1, 5.7, 7.3));
    let macro_hash = macro_cluster_hash(cell, seed + vec3<f32>(11.0, 17.0, 23.0), grid_size, visual.variation_a.z);
    let micro_hash = hash13(seed + vec3<f32>(19.0, 29.0, 31.0));

    var color = visual.base_color.rgb;

    if (visual.palette.y > 1u) {
        color = mix(color, palette_color(visual, cell_hash), 0.78);
    }

    let bias_delta = length(face_visual.color_bias.rgb - vec3<f32>(1.0));
    if (bias_delta > 0.01) {
        color = mix(color, face_visual.color_bias.rgb, clamp(0.55 + bias_delta * 0.18, 0.0, 0.92));
    }

    // Variation hierarchy.
    // Macro variation is intentionally softened to avoid visual soup.
    color = color * (1.0 + (cell_hash - 0.5) * 2.0 * visual.variation_a.x);
    color = color * (1.0 + (face_hash - 0.5) * 2.0 * visual.variation_a.y);
    color = color * (1.0 + (macro_hash - 0.5) * 2.0 * visual.variation_a.w * 0.65);
    color = color * (1.0 + (micro_hash - 0.5) * 2.0 * visual.variation_b.y);

    let up_dot = clamp(dot(safe_normalize(world_normal), up_dir), -1.0, 1.0);
    let topness = saturate(up_dot);
    let bottomness = saturate(-up_dot);
    let sideness = saturate(1.0 - max(topness, bottomness));

    if (visual.response.x > 0.0 && face_id == 0u) {
        color = mix(color, palette_color(visual, 0.999), saturate(visual.response.x));
    }

    // Grass top fringe on side faces.
    if (visual.procedural.y != 0u && face_id >= 2u) {
        let top_face = visual.faces[0];
        let fringe_color = default_or_face_color(top_face, palette_color(visual, 0.999));
        let fringe_noise = hash13(seed + vec3<f32>(41.0, 43.0, 47.0));

        // Phase 1 tweak:
        // Stronger and lower transition than previous version.
        let fringe = (1.0 - smoothstep(0.05, 0.35, uv.y)) * smoothstep(0.15, 0.85, fringe_noise);
        color = mix(color, fringe_color, fringe);
    }

    let detail_count = min(visual.procedural.z, 8u);
    for (var i: u32 = 0u; i < detail_count; i = i + 1u) {
        if (!detail_enabled(face_visual.detail_mask, i)) {
            continue;
        }

        let detail = detail_for(visual, i);
        let strength = detail_strength(
            detail,
            face_id,
            cell,
            grid_size_f,
            uv,
            topness,
            sideness,
            bottomness,
            seed + vec3<f32>(f32(i) * 13.0, 2.0, 9.0),
        );

        color = mix(color, detail.color.rgb, strength);
    }

    color = color * (1.0 - edge_factor(uv) * visual.variation_b.z);

    return clamp(color, vec3<f32>(0.0), vec3<f32>(3.5));
}