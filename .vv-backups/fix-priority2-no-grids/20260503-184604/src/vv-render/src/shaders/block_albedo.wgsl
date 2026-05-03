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

fn vv_cell_seed(
    voxel_pos: vec3<i32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    variation_seed: u32,
    cell: vec2<f32>,
    salt: f32,
) -> vec3<f32> {
    return face_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        cell,
        salt,
    );
}

fn wood_cut_face_color(
    visual: BlockVisual,
    base: vec3<f32>,
    uv: vec2<f32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    voxel_pos: vec3<i32>,
    variation_seed: u32,
) -> vec3<f32> {
    let p = uv - vec2<f32>(0.5);
    let r = length(p);

    let rings = max(f32(visual.patterned.rows), 5.0);

    let seed = vv_cell_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        vec2<f32>(floor(r * rings), 0.0),
        f32(visual.patterned.seed & 65535u),
    );

    let wobble =
        sin((p.x * 7.0 + hash13(seed) * 6.28318)) * 0.010 +
        sin((p.y * 9.0 + hash13(seed + vec3<f32>(2.0, 4.0, 8.0)) * 6.28318)) * 0.008;

    let rr = r + wobble;

    let ring_wave = abs(fract(rr * rings * 1.55) - 0.5);
    let ring_mask = 1.0 - smoothstep(0.065, 0.17, ring_wave);

    let core = 1.0 - smoothstep(0.00, 0.13, r);
    let outer = smoothstep(0.38, 0.53, r);

    var color = base;

    color = mix(color, color * vec3<f32>(1.24, 1.10, 0.82), ring_mask * 0.30);
    color = mix(color, color * vec3<f32>(1.40, 1.20, 0.86), core * 0.35);
    color = mix(color, color * vec3<f32>(0.58, 0.35, 0.17), outer * 0.55);

    let square_edge = smoothstep(0.42, 0.50, max(abs(p.x), abs(p.y)));
    color = mix(color, color * vec3<f32>(0.70, 0.43, 0.20), square_edge * 0.45);

    let grain = hash13(seed + vec3<f32>(14.0, 71.0, 6.0));
    color *= 1.0 + (grain - 0.5) * visual.patterned.color_variation * 0.50;

    return max(color, vec3<f32>(0.0));
}

fn wood_bark_side_color(
    visual: BlockVisual,
    base: vec3<f32>,
    uv: vec2<f32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    voxel_pos: vec3<i32>,
    variation_seed: u32,
) -> vec3<f32> {
    let stripe_count = 14.0;
    let stripe = uv.x * stripe_count;
    let stripe_id = floor(stripe);

    let seed = vv_cell_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        vec2<f32>(stripe_id, floor(uv.y * 7.0)),
        f32(visual.patterned.seed & 65535u),
    );

    let h = hash13(seed);
    let vnoise = hash13(seed + vec3<f32>(floor(uv.y * 11.0), 7.0, 19.0));

    let thin = 1.0 - smoothstep(0.035, 0.12, abs(fract(stripe + h * 0.45) - 0.5));
    let broad = 1.0 - smoothstep(0.14, 0.42, abs(fract(stripe * 0.34 + h) - 0.5));
    let bark_patch = select(0.0, 1.0, vnoise > 0.74) * 0.12;

    var color = base;
    color = mix(color, color * vec3<f32>(0.56, 0.34, 0.16), broad * 0.35);
    color = mix(color, color * vec3<f32>(0.38, 0.22, 0.10), thin * 0.28);
    color *= 1.0 + (h - 0.5) * visual.patterned.color_variation * 0.75;
    color *= 1.0 + bark_patch;

    let center_light = smoothstep(0.18, 0.45, uv.y) * (1.0 - smoothstep(0.72, 1.0, uv.y));
    color *= 1.0 + center_light * 0.05;

    return max(color, vec3<f32>(0.0));
}

fn patterned_cell_coords(visual: BlockVisual, uv: vec2<f32>) -> vec4<f32> {
    let rows = max(f32(visual.patterned.rows), 1.0);
    let columns = max(f32(visual.patterned.columns), 1.0);

    if (visual.patterned.kind == PATTERN_STRIPS) {
        let x = uv.x * columns;
        return vec4<f32>(floor(x), 0.0, fract(x), uv.y);
    }

    let row = floor(uv.y * rows);
    var x = uv.x * columns;

    if ((visual.patterned.flags & PATTERN_FLAG_STAGGER) != 0u && (u32(row) % 2u) == 1u) {
        x = x + 0.5;
    }

    return vec4<f32>(floor(x), row, fract(x), fract(uv.y * rows));
}

fn patterned_mortar_mask(visual: BlockVisual, cell_uv: vec2<f32>) -> f32 {
    let rows = max(f32(visual.patterned.rows), 1.0);
    let columns = max(f32(visual.patterned.columns), 1.0);
    let gap = clamp(visual.patterned.gap_width, 0.0, 0.20);

    let edge_x = min(cell_uv.x, 1.0 - cell_uv.x) / columns;
    let edge_y = min(cell_uv.y, 1.0 - cell_uv.y) / rows;
    let edge = min(edge_x, edge_y);

    return 1.0 - smoothstep(gap * 0.45, gap * 0.95 + 0.0001, edge);
}

fn generic_patterned_color(
    visual: BlockVisual,
    base: vec3<f32>,
    uv: vec2<f32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    voxel_pos: vec3<i32>,
    variation_seed: u32,
) -> vec3<f32> {
    let coords = patterned_cell_coords(visual, uv);
    let cell = coords.xy;
    let cell_uv = coords.zw;

    let seed = vv_cell_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        cell,
        f32(visual.patterned.seed & 65535u),
    );

    let h = hash13(seed + vec3<f32>(13.0, 31.0, 73.0));

    var color = base;
    color *= 1.0 + (h - 0.5) * 2.0 * visual.patterned.color_variation;

    let center_distance = length(cell_uv - vec2<f32>(0.5));
    let center_highlight = 1.0 - smoothstep(0.10, 0.72, center_distance);
    color *= 1.0 + center_highlight * visual.patterned.cell_pillow * 2.0;

    let mortar = patterned_mortar_mask(visual, cell_uv);
    color = mix(color, color * vec3<f32>(0.42), mortar * 0.72);

    return max(color, vec3<f32>(0.0));
}

fn patterned_color(
    visual: BlockVisual,
    base: vec3<f32>,
    uv: vec2<f32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    voxel_pos: vec3<i32>,
    variation_seed: u32,
) -> vec3<f32> {
    if (visual.patterned.kind == PATTERN_RINGS) {
        if (face_id == 0u || face_id == 1u) {
            return wood_cut_face_color(
                visual,
                base,
                uv,
                block_id,
                block_visual_id,
                face_id,
                voxel_pos,
                variation_seed,
            );
        }

        return wood_bark_side_color(
            visual,
            base,
            uv,
            block_id,
            block_visual_id,
            face_id,
            voxel_pos,
            variation_seed,
        );
    }

    return generic_patterned_color(
        visual,
        base,
        uv,
        block_id,
        block_visual_id,
        face_id,
        voxel_pos,
        variation_seed,
    );
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
        return patterned_color(
            visual,
            base,
            uv,
            block_id,
            block_visual_id,
            face_id,
            voxel_pos,
            variation_seed,
        );
    }

    return base;
}