const SURFACE_PROGRAM_PATTERNED: u32 = 1u;

const PATTERN_GRID: u32 = 0u;
const PATTERN_STRIPS: u32 = 1u;
const PATTERN_RUNNING_BOND: u32 = 2u;
const PATTERN_RINGS: u32 = 3u;
const PATTERN_NATURAL_CELLS: u32 = 4u;
const PATTERN_CRACKED_CELLS: u32 = 5u;
const PATTERN_LAYERED_SURFACE: u32 = 6u;

const PATTERN_FLAG_STAGGER: u32 = 1u;

const DETAIL_PEBBLE: u32 = 1u;
const DETAIL_ROOT: u32 = 2u;
const DETAIL_LEAF_LOBE: u32 = 3u;
const DETAIL_GRAIN: u32 = 4u;
const DETAIL_SPECKLE: u32 = 5u;
const DETAIL_STAIN: u32 = 6u;
const DETAIL_CRACK: u32 = 7u;

fn vv_value_noise_3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (vec3<f32>(3.0) - 2.0 * f);

    let n000 = hash13(i + vec3<f32>(0.0, 0.0, 0.0));
    let n100 = hash13(i + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = hash13(i + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = hash13(i + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = hash13(i + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = hash13(i + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = hash13(i + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = hash13(i + vec3<f32>(1.0, 1.0, 1.0));

    let nx00 = mix(n000, n100, u.x);
    let nx10 = mix(n010, n110, u.x);
    let nx01 = mix(n001, n101, u.x);
    let nx11 = mix(n011, n111, u.x);
    let nxy0 = mix(nx00, nx10, u.y);
    let nxy1 = mix(nx01, nx11, u.y);

    return mix(nxy0, nxy1, u.z);
}

fn vv_face_bias(visual: BlockVisual, face_id: u32) -> vec3<f32> {
    return face_visual_for(visual, face_id).color_bias.rgb;
}

fn vv_base_color(visual: BlockVisual, face_id: u32, seed: vec3<f32>) -> vec3<f32> {
    var base = visual.base_color.rgb;

    let white = length(base - vec3<f32>(1.0)) < 0.04;
    if (white) {
        base = palette_color(visual, hash13(seed + vec3<f32>(31.0, 17.0, 9.0)));
    }

    let face = vv_face_bias(visual, face_id);
    if (length(face - vec3<f32>(1.0)) > 0.015) {
        base = mix(base, face, 0.86);
    }

    return max(base, vec3<f32>(0.0));
}

fn vv_variation(
    color: vec3<f32>,
    visual: BlockVisual,
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    up: vec3<f32>,
    seed: vec3<f32>,
) -> vec3<f32> {
    let per_voxel = saturate(visual.variation_a.x);
    let per_face = saturate(visual.variation_a.y);
    let macro_scale = safe_positive(visual.variation_a.z, 1.0);
    let macro_strength = saturate(visual.variation_a.w);
    let micro_scale = safe_positive(visual.variation_b.x, 1.0);
    let micro_strength = saturate(visual.variation_b.y);

    let hv = hash13(seed + vec3<f32>(5.0, 11.0, 23.0));
    let hf = hash13(seed + vec3<f32>(31.0, 47.0, 59.0));
    let macro_n = vv_value_noise_3d(world_pos * (0.075 / macro_scale) + seed * 0.011);
    let micro_n = vv_value_noise_3d(world_pos * (0.72 / micro_scale) + seed * 0.023);

    var c = color;
    c *= 1.0 + (hv - 0.5) * per_voxel;
    c *= 1.0 + (hf - 0.5) * per_face;
    c *= 1.0 + (macro_n - 0.5) * macro_strength;
    c *= 1.0 + (micro_n - 0.5) * micro_strength;

    let topness = saturate(dot(normal, up));
    let bottomness = saturate(dot(-normal, up));
    let sideness = saturate(1.0 - topness - bottomness);

    c *= mix(vec3<f32>(1.0), vec3<f32>(1.035, 1.025, 0.970), topness * 0.14);
    c *= mix(vec3<f32>(1.0), vec3<f32>(0.92, 0.94, 1.00), sideness * 0.08);
    c *= mix(vec3<f32>(1.0), vec3<f32>(0.72, 0.76, 0.85), bottomness * 0.24);

    return max(c, vec3<f32>(0.0));
}

fn vv_pattern_seed(
    voxel_pos: vec3<i32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    variation_seed: u32,
    cell: vec2<f32>,
    salt: f32,
) -> vec3<f32> {
    return face_seed(voxel_pos, block_id, block_visual_id, face_id, variation_seed, cell, salt);
}

fn vv_voronoi(p: vec2<f32>, seed: vec3<f32>) -> vec3<f32> {
    let i = floor(p);
    let f = fract(p);

    var closest = 10.0;
    var second = 10.0;
    var cell_hash = 0.0;

    for (var y: i32 = -1; y <= 1; y = y + 1) {
        for (var x: i32 = -1; x <= 1; x = x + 1) {
            let g = vec2<f32>(f32(x), f32(y));
            let h1 = hash13(vec3<f32>(i + g, seed.x + seed.z));
            let h2 = hash13(vec3<f32>(i + g, seed.y + seed.z + 19.0));
            let jitter = vec2<f32>(h1, h2);
            let d = length(g + jitter - f);

            if (d < closest) {
                second = closest;
                closest = d;
                cell_hash = hash13(vec3<f32>(i + g, seed.x + seed.y + seed.z));
            } else if (d < second) {
                second = d;
            }
        }
    }

    return vec3<f32>(closest, second, cell_hash);
}

fn vv_cells_color(
    visual: BlockVisual,
    base: vec3<f32>,
    world_pos: vec3<f32>,
    uv: vec2<f32>,
    voxel_pos: vec3<i32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    variation_seed: u32,
) -> vec3<f32> {
    let seed = vv_pattern_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        vec2<f32>(0.0),
        f32(visual.patterned.seed & 65535u) + 43.0,
    );

    let density = max(max(f32(visual.patterned.rows), f32(visual.patterned.columns)), 2.0);
    let warp = vec2<f32>(
        vv_value_noise_3d(world_pos * 0.31 + seed * 0.017),
        vv_value_noise_3d(world_pos * 0.37 + seed * 0.029),
    ) * 0.12;

    let cells = vv_voronoi((uv + warp) * density, seed);
    let closest = cells.x;
    let second = cells.y;
    let cell_hash = cells.z;

    let boundary = 1.0 - smoothstep(
        0.025,
        0.135 + visual.patterned.gap_width * 1.6,
        second - closest,
    );

    var c = base;
    c *= 1.0 + (cell_hash - 0.5) * 2.0 * saturate(visual.patterned.color_variation);

    // Flat cartoon faces: no cell pillow highlight.

    let crack_amount = max(
        visual.patterned.crack_density,
        select(0.0, 0.18, visual.patterned.kind == PATTERN_CRACKED_CELLS),
    );
    let crack_hash = hash13(seed + vec3<f32>(floor((uv + warp) * density), 71.0));
    let crack = boundary * select(0.0, 1.0, crack_hash < crack_amount + visual.patterned.gap_width);

    c = mix(c, c * vec3<f32>(0.48, 0.48, 0.46), boundary * visual.patterned.gap_depth * 5.0);
    c = mix(c, c * vec3<f32>(0.30, 0.30, 0.28), crack * 0.62);

    return max(c, vec3<f32>(0.0));
}

fn vv_rect_color(
    visual: BlockVisual,
    base: vec3<f32>,
    uv: vec2<f32>,
    voxel_pos: vec3<i32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    variation_seed: u32,
) -> vec3<f32> {
    let rows = max(f32(visual.patterned.rows), 1.0);
    let columns = max(f32(visual.patterned.columns), 1.0);

    var grid_uv = vec2<f32>(uv.x * columns, uv.y * rows);
    let row = floor(grid_uv.y);

    if ((visual.patterned.flags & PATTERN_FLAG_STAGGER) != 0u && (u32(row) % 2u) == 1u) {
        grid_uv.x += 0.5;
    }

    if (visual.patterned.kind == PATTERN_STRIPS) {
        grid_uv = vec2<f32>(uv.x * columns, uv.y);
    }

    let cell = floor(grid_uv);
    let local = fract(grid_uv);
    let seed = vv_pattern_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        cell,
        f32(visual.patterned.seed & 65535u) + 17.0,
    );

    let h = hash13(seed + vec3<f32>(13.0, 31.0, 73.0));
    let h2 = hash13(seed + vec3<f32>(97.0, 11.0, 41.0));

    var c = base;
    c *= 1.0 + (h - 0.5) * 2.0 * saturate(visual.patterned.color_variation);

    // Flat cartoon faces: no center pillow highlight.

    let gap = clamp(visual.patterned.gap_width, 0.0, 0.20);
    let warped = clamp(local + vec2<f32>(h - 0.5, h2 - 0.5) * 0.035, vec2<f32>(0.0), vec2<f32>(1.0));
    let edge = min(min(warped.x, 1.0 - warped.x) / columns, min(warped.y, 1.0 - warped.y) / rows);
    let mortar = 1.0 - smoothstep(gap * 0.40, gap * 1.05 + 0.0001, edge);
    let mortar_tint = mix(vec3<f32>(0.56), vec3<f32>(0.38), saturate(visual.patterned.gap_depth * 7.0));

    c = mix(c, c * mortar_tint, mortar * 0.68);
    return max(c, vec3<f32>(0.0));
}

fn vv_rings_color(
    visual: BlockVisual,
    base: vec3<f32>,
    world_pos: vec3<f32>,
    uv: vec2<f32>,
    face_id: u32,
    voxel_pos: vec3<i32>,
    block_id: i32,
    block_visual_id: u32,
    variation_seed: u32,
) -> vec3<f32> {
    let seed = vv_pattern_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        vec2<f32>(0.0),
        f32(visual.patterned.seed & 65535u) + 101.0,
    );

    if (face_id == 0u || face_id == 1u) {
        let p = uv - vec2<f32>(0.5);
        let r = length(p);
        let rings = max(f32(visual.patterned.rows), 5.0);

        let wobble =
            sin(p.x * 8.7 + hash13(seed) * 6.28318) * 0.012 +
            sin(p.y * 7.9 + hash13(seed + vec3<f32>(9.0, 4.0, 2.0)) * 6.28318) * 0.010;

        let wave = abs(fract((r + wobble) * rings * 1.42) - 0.5);
        let ring = 1.0 - smoothstep(0.060, 0.170, wave);
        let core = 1.0 - smoothstep(0.00, 0.13, r);
        let outer = smoothstep(0.39, 0.54, r);

        var c = base;
        c = mix(c, c * vec3<f32>(1.18, 1.08, 0.86), ring * 0.28);
        c = mix(c, c * vec3<f32>(1.34, 1.18, 0.88), core * 0.32);
        c = mix(c, c * vec3<f32>(0.58, 0.36, 0.18), outer * 0.50);
        return max(c, vec3<f32>(0.0));
    }

    let columns = max(f32(visual.patterned.columns), 6.0);
    let vertical_noise = vv_value_noise_3d(vec3<f32>(uv.x * columns * 0.45, uv.y * 3.2, seed.x * 0.05));
    let fine_noise = vv_value_noise_3d(vec3<f32>(uv.x * columns * 2.2, uv.y * 15.0, seed.y * 0.05));

    let streak = 1.0 - smoothstep(0.20, 0.47, abs(fract(uv.x * columns + vertical_noise * 0.75) - 0.5));
    let dark = 1.0 - smoothstep(0.05, 0.17, abs(fract(uv.x * columns * 1.7 + fine_noise) - 0.5));

    var c = base;
    c = mix(c, c * vec3<f32>(0.60, 0.38, 0.19), streak * 0.28);
    c = mix(c, c * vec3<f32>(0.42, 0.25, 0.12), dark * 0.20);

    let patch_noise = vv_value_noise_3d(world_pos * 0.85 + seed * 0.015);
    c *= 1.0 + (patch_noise - 0.5) * saturate(visual.patterned.color_variation) * 0.55;

    return max(c, vec3<f32>(0.0));
}

fn vv_layered_surface_color(
    visual: BlockVisual,
    base: vec3<f32>,
    world_pos: vec3<f32>,
    uv: vec2<f32>,
    face_id: u32,
    seed: vec3<f32>,
) -> vec3<f32> {
    if (face_id == 0u) {
        let top = vv_face_bias(visual, 0u);
        var c = mix(base, top, select(0.0, 0.88, length(top - vec3<f32>(1.0)) > 0.015));

        let n = vv_value_noise_3d(vec3<f32>(uv * 5.0, seed.z * 0.07));
        let fiber_density = max(f32(visual.patterned.rows), 6.0);
        let fiber = 1.0 - smoothstep(0.10, 0.32, abs(fract((uv.x + n * 0.12) * fiber_density * 0.55) - 0.5));

        c *= 1.0 + (n - 0.5) * saturate(visual.patterned.color_variation) * 0.50;
        c = mix(c, c * vec3<f32>(1.04, 1.12, 0.88), fiber * 0.18);
        return max(c, vec3<f32>(0.0));
    }

    if (face_id == 1u) {
        return base;
    }

    let top_color = vv_face_bias(visual, 0u);
    let has_top = length(top_color - vec3<f32>(1.0)) > 0.015;
    var bleed = base * vec3<f32>(0.62, 1.10, 0.55);
    if (has_top) {
        bleed = top_color;
    }

    let fringe_h = clamp(visual.patterned.gap_width * 2.5, 0.05, 0.50);
    let irregularity = saturate(visual.patterned.height_variation * 4.0) * 0.12;

    let from_top_a = 1.0 - uv.y;
    let from_top_b = uv.y;

    let noise = vv_value_noise_3d(vec3<f32>(uv.x * 3.5, seed.x * 0.05, seed.y * 0.03));
    let edge = fringe_h + (noise - 0.5) * irregularity * 2.0;

    let fringe_a = 1.0 - smoothstep(edge - 0.04, edge + 0.05, from_top_a);
    let fringe_b = 1.0 - smoothstep(edge - 0.04, edge + 0.05, from_top_b);
    let fringe = max(fringe_a, fringe_b * 0.55);

    return max(mix(base, bleed, fringe), vec3<f32>(0.0));
}

fn vv_detail_face_enabled(detail: BlockDetail, face_id: u32) -> bool {
    let mask = detail.kind_data.y;
    let bit = 1u << min(face_id, 5u);
    return mask == 0u || (mask & bit) != 0u;
}

fn vv_detail_mask(
    detail: BlockDetail,
    uv: vec2<f32>,
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    up: vec3<f32>,
    voxel_pos: vec3<i32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    variation_seed: u32,
    index: u32,
) -> f32 {
    if (!vv_detail_face_enabled(detail, face_id)) {
        return 0.0;
    }

    let kind = detail.kind_data.x;
    let density = saturate(detail.params.x);
    let min_size = clamp(detail.params.y, 0.001, 0.50);
    let max_size = clamp(max(detail.params.z, min_size), 0.001, 0.75);
    let slope_bias = saturate(detail.params.w);
    let average = clamp((min_size + max_size) * 0.5, 0.005, 0.60);
    let cell_scale = clamp(1.0 / average, 2.0, 42.0);

    let p = uv * cell_scale;
    let cell = floor(p);
    let local = fract(p);

    let seed = vv_pattern_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        cell,
        f32(detail.kind_data.z & 65535u) + f32(index) * 41.0,
    );

    let spawn = hash13(seed + vec3<f32>(3.0, 7.0, 11.0));
    if (spawn > density) {
        return 0.0;
    }

    let size_hash = hash13(seed + vec3<f32>(13.0, 17.0, 19.0));
    let angle_hash = hash13(seed + vec3<f32>(23.0, 29.0, 31.0));
    let radius = mix(min_size, max_size, size_hash) * cell_scale;

    let center = vec2<f32>(
        hash13(seed + vec3<f32>(37.0, 41.0, 43.0)),
        hash13(seed + vec3<f32>(47.0, 53.0, 59.0)),
    );

    let centered = local - center;
    let angle = angle_hash * 6.28318;
    let ca = cos(angle);
    let sa = sin(angle);
    let rotated = vec2<f32>(
        ca * centered.x - sa * centered.y,
        sa * centered.x + ca * centered.y,
    );

    var m = 0.0;

    if (kind == DETAIL_PEBBLE) {
        let q = rotated / vec2<f32>(max(radius * 0.92, 0.001), max(radius * 0.58, 0.001));
        m = 1.0 - smoothstep(0.72, 1.05, dot(q, q));
    } else if (kind == DETAIL_ROOT) {
        let wobble = vv_value_noise_3d(vec3<f32>(uv * 18.0, seed.z * 0.03)) - 0.5;
        let line = abs(rotated.y + wobble * 0.045);
        let along = 1.0 - smoothstep(0.12, 0.48, abs(rotated.x));
        m = (1.0 - smoothstep(0.010, 0.038, line)) * along;
    } else if (kind == DETAIL_LEAF_LOBE) {
        let q = rotated / vec2<f32>(max(radius * 1.15, 0.001), max(radius * 0.46, 0.001));
        let lobe = 1.0 - smoothstep(0.58, 1.0, dot(q, q));
        let vein = (1.0 - smoothstep(0.010, 0.038, abs(rotated.y))) *
            (1.0 - smoothstep(0.18, 0.48, abs(rotated.x))) * lobe;
        m = saturate(lobe * 0.85 + vein * 0.25);
    } else if (kind == DETAIL_GRAIN) {
        let stripe = abs(fract((uv.x + angle_hash * 0.37) * cell_scale * 0.72) - 0.5);
        m = 1.0 - smoothstep(0.060, 0.19, stripe);
    } else if (kind == DETAIL_SPECKLE) {
        let d = length(centered);
        m = 1.0 - smoothstep(radius * 0.20, radius * 0.52, d);
    } else if (kind == DETAIL_STAIN) {
        let n = vv_value_noise_3d(world_pos * (0.85 + size_hash * 0.55) + seed * 0.017);
        m = smoothstep(0.46, 0.78, n) * (1.0 - smoothstep(0.42, 0.78, length(centered)));
    } else if (kind == DETAIL_CRACK) {
        let wobble = vv_value_noise_3d(vec3<f32>(uv * 22.0, seed.z * 0.05)) - 0.5;
        let line = abs(rotated.y + wobble * 0.055);
        let along = 1.0 - smoothstep(0.16, 0.50, abs(rotated.x));
        m = (1.0 - smoothstep(0.006, 0.026, line)) * along;
    }

    let topness = saturate(dot(normal, up));
    let bottomness = saturate(dot(-normal, up));
    let sideness = saturate(1.0 - topness - bottomness);
    let slope_weight = mix(1.0, saturate(topness + sideness * 0.65 + bottomness * 0.25), slope_bias);

    return saturate(m * slope_weight * detail.color.a);
}

fn vv_apply_details(
    visual: BlockVisual,
    color: vec3<f32>,
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
    var c = color;
    let count = min(visual.procedural.z, 8u);

    for (var i: u32 = 0u; i < count; i = i + 1u) {
        let detail = detail_for(visual, i);
        let mask = vv_detail_mask(
            detail,
            uv,
            world_pos,
            normal,
            up,
            voxel_pos,
            block_id,
            block_visual_id,
            face_id,
            variation_seed,
            i,
        );

        let kind = detail.kind_data.x;
        var detail_color = detail.color.rgb;

        if (kind == DETAIL_CRACK || kind == DETAIL_STAIN || kind == DETAIL_ROOT) {
            detail_color = mix(c * 0.42, detail_color, 0.55);
        }

        c = mix(c, detail_color, mask);
    }

    return max(c, vec3<f32>(0.0));
}


fn vv_cartoon_fake_bevel(
    color: vec3<f32>,
    uv: vec2<f32>,
    normal: vec3<f32>,
    up: vec3<f32>,
    visual: BlockVisual,
) -> vec3<f32> {
    // Shader-only rounded voxel look.
    // No vertex displacement, no holes, no extra mesh cost.
    let edge_x = min(uv.x, 1.0 - uv.x);
    let edge_y = min(uv.y, 1.0 - uv.y);
    let edge = min(edge_x, edge_y);

    let corner_dist = min(
        length(uv - vec2<f32>(0.0, 0.0)),
        min(
            length(uv - vec2<f32>(1.0, 0.0)),
            min(
                length(uv - vec2<f32>(0.0, 1.0)),
                length(uv - vec2<f32>(1.0, 1.0))
            )
        )
    );

    let radius = 0.135;
    let edge_band = 1.0 - smoothstep(0.010, radius, edge);
    let corner_band = 1.0 - smoothstep(0.045, radius * 1.35, corner_dist);

    let bevel = saturate(edge_band * 0.75 + corner_band * 0.55);

    let topness = saturate(dot(normal, up));
    let bottomness = saturate(dot(-normal, up));
    let sideness = saturate(1.0 - topness - bottomness);

    var c = color;

    // Soft candy edge: darker on side/bottom edges, slightly creamy on top edges.
    let dark_edge = vec3<f32>(0.72, 0.76, 0.82);
    let warm_highlight = vec3<f32>(1.10, 1.08, 0.98);

    c = mix(c, c * dark_edge, bevel * (0.22 + sideness * 0.18 + bottomness * 0.24));
    c = mix(c, c * warm_highlight, bevel * topness * 0.13);

    // Preserve readable material colors.
    c = vv_saturate_color(c, 1.06);

    return max(c, vec3<f32>(0.0));
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

    let seed = vv_pattern_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        vec2<f32>(0.0),
        f32(visual.patterned.seed & 65535u) + 7.0,
    );

    var color = vv_base_color(visual, face_id, seed);

    if (visual.procedural.w == SURFACE_PROGRAM_PATTERNED) {
        if (visual.patterned.kind == PATTERN_LAYERED_SURFACE) {
            color = vv_layered_surface_color(visual, color, world_pos, uv, face_id, seed);
        } else if (
            visual.patterned.kind == PATTERN_NATURAL_CELLS ||
            visual.patterned.kind == PATTERN_CRACKED_CELLS
        ) {
            color = vv_cells_color(
                visual,
                color,
                world_pos,
                uv,
                voxel_pos,
                block_id,
                block_visual_id,
                face_id,
                variation_seed,
            );
        } else if (
            visual.patterned.kind == PATTERN_GRID ||
            visual.patterned.kind == PATTERN_RUNNING_BOND ||
            visual.patterned.kind == PATTERN_STRIPS
        ) {
            color = vv_rect_color(
                visual,
                color,
                uv,
                voxel_pos,
                block_id,
                block_visual_id,
                face_id,
                variation_seed,
            );
        } else if (visual.patterned.kind == PATTERN_RINGS) {
            color = vv_rings_color(
                visual,
                color,
                world_pos,
                uv,
                face_id,
                voxel_pos,
                block_id,
                block_visual_id,
                variation_seed,
            );
        }
    }

    color = vv_variation(color, visual, world_pos, normal, up, seed);

    color = vv_apply_details(
        visual,
        color,
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

    color = vv_cartoon_fake_bevel(color, uv, normal, up, visual);

    return clamp(color, vec3<f32>(0.0), vec3<f32>(1.25));
}