const SURFACE_PROGRAM_PATTERNED: u32 = 1u;

const PATTERN_GRID: u32 = 0u;
const PATTERN_STRIPS: u32 = 1u;
const PATTERN_RUNNING_BOND: u32 = 2u;
const PATTERN_RINGS: u32 = 3u;
const PATTERN_NATURAL_CELLS: u32 = 4u;
const PATTERN_CRACKED_CELLS: u32 = 5u;

const PATTERN_FLAG_STAGGER: u32 = 1u;

fn face_color_bias(block_visual_id: u32, face_id: u32) -> vec3<f32> {
    let visual = visual_for(block_visual_id);

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
    return max(visual.base_color.rgb * face_color_bias(block_visual_id, face_id), vec3<f32>(0.0));
}

fn vv_soft_voxel_variation(
    visual: BlockVisual,
    world_pos: vec3<f32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    voxel_pos: vec3<i32>,
    variation_seed: u32,
) -> f32 {
    let seed = face_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        vec2<f32>(0.0),
        f32(visual.patterned.seed & 65535u),
    );

    let voxel_hash = hash13(seed + vec3<f32>(3.1, 7.7, 11.3));
    let face_hash = hash13(seed + vec3<f32>(17.0, 29.0, 43.0));

    // Very low frequency world variation. No UV cells, no mortar, no grid.
    let macro_cell = floor(world_pos * 0.115 + seed * 0.013);
    let macro_hash = hash13(macro_cell);

    let authored_strength = max(
        max(visual.variation_a.x, visual.variation_a.y),
        max(visual.patterned.color_variation, visual.variation_a.w),
    );

    let strength = clamp(authored_strength, 0.025, 0.135);

    let combined =
        (voxel_hash - 0.5) * 0.72 +
        (face_hash - 0.5) * 0.28 +
        (macro_hash - 0.5) * 0.34;

    return combined * strength;
}

fn vv_soft_directional_tint(
    visual: BlockVisual,
    normal: vec3<f32>,
    up: vec3<f32>,
    face_id: u32,
) -> vec3<f32> {
    let topness = saturate(dot(normal, up));
    let bottomness = saturate(dot(-normal, up));
    let sideness = saturate(1.0 - topness - bottomness);

    var tint = vec3<f32>(1.0);

    // Subtle readable face separation, not a pattern.
    tint *= mix(vec3<f32>(1.0), vec3<f32>(1.045, 1.035, 0.970), topness * 0.18);
    tint *= mix(vec3<f32>(1.0), vec3<f32>(0.870, 0.900, 1.020), sideness * 0.10);
    tint *= mix(vec3<f32>(1.0), vec3<f32>(0.720, 0.760, 0.850), bottomness * 0.28);

    return tint;
}

fn non_repeating_patterned_color(
    visual: BlockVisual,
    base: vec3<f32>,
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
    var color = base;

    let variation = vv_soft_voxel_variation(
        visual,
        world_pos,
        block_id,
        block_visual_id,
        face_id,
        voxel_pos,
        variation_seed,
    );

    color *= 1.0 + variation;

    // Keep Patterned data alive, but do NOT draw authored rows/columns yet.
    // Phase 3 will reintroduce bricks/logs/natural stone with non-repeating rules.
    color *= vv_soft_directional_tint(visual, normal, up, face_id);

    // Tiny non-grid material response. Uses UV only as a smooth gradient, never as cells.
    let soft_uv = (uv.x - 0.5) * (uv.y - 0.5);
    color *= 1.0 + soft_uv * 0.018;

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
    let visual = visual_for(block_visual_id);
    let base = flat_block_albedo(block_visual_id, face_id);

    if (visual.procedural.w == SURFACE_PROGRAM_PATTERNED) {
        return non_repeating_patterned_color(
            visual,
            base,
            world_pos,
            normal,
            uv,
            block_id,
            block_visual_id,
            face_id,
            voxel_pos,
            variation_seed,
            up,
        );
    }

    return base;
}