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

fn vv_has_color(color: vec3<f32>) -> bool {
    return length(color - vec3<f32>(1.0)) > 0.018;
}

fn vv_seed(
    voxel_pos: vec3<i32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    variation_seed: u32,
    salt: f32,
) -> vec3<f32> {
    return face_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        vec2<f32>(0.0),
        f32(visual_for(block_visual_id).patterned.seed & 65535u) + salt,
    );
}

fn vv_base_color(visual: BlockVisual, face_id: u32, seed: vec3<f32>) -> vec3<f32> {
    var base = visual.base_color.rgb;

    if (length(base - vec3<f32>(1.0)) < 0.04) {
        base = palette_color(visual, hash13(seed + vec3<f32>(31.0, 17.0, 9.0)));
    }

    let face = vv_face_bias(visual, face_id);
    if (vv_has_color(face)) {
        base = mix(base, face, 0.92);
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

    c *= mix(vec3<f32>(1.0), vec3<f32>(1.035, 1.025, 0.97), topness * 0.12);
    c *= mix(vec3<f32>(1.0), vec3<f32>(0.92, 0.94, 0.98), sideness * 0.08);
    c *= mix(vec3<f32>(1.0), vec3<f32>(0.74, 0.76, 0.82), bottomness * 0.20);

    return max(c, vec3<f32>(0.0));
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

fn vv_cells_material(
    base: vec3<f32>,
    uv: vec2<f32>,
    world_pos: vec3<f32>,
    density: f32,
    gap: f32,
    contrast: f32,
    seed: vec3<f32>,
) -> vec3<f32> {
    let warp = vec2<f32>(
        vv_value_noise_3d(world_pos * 0.25 + seed * 0.017),
        vv_value_noise_3d(world_pos * 0.31 + seed * 0.029),
    ) * 0.10;

    let cells = vv_voronoi((uv + warp) * density, seed);
    let closest = cells.x;
    let second = cells.y;
    let cell_hash = cells.z;

    let boundary = 1.0 - smoothstep(0.020, gap, second - closest);

    var c = base;
    c *= 1.0 + (cell_hash - 0.5) * contrast;

    let center = 1.0 - smoothstep(0.06, 0.62, closest);
    c *= 1.0 + center * 0.040;

    c = mix(c, c * vec3<f32>(0.48, 0.40, 0.32), boundary * 0.58);
    return max(c, vec3<f32>(0.0));
}

fn vv_rect_material(
    visual: BlockVisual,
    base: vec3<f32>,
    uv: vec2<f32>,
    seed: vec3<f32>,
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
    let local_uv = fract(grid_uv);
    let h = hash13(seed + vec3<f32>(cell, 13.0));
    let h2 = hash13(seed + vec3<f32>(cell, 97.0));

    var c = base;
    c *= 1.0 + (h - 0.5) * 2.0 * saturate(visual.patterned.color_variation);

    let gap = clamp(visual.patterned.gap_width, 0.0, 0.20);
    let warped = clamp(local_uv + vec2<f32>(h - 0.5, h2 - 0.5) * 0.035, vec2<f32>(0.0), vec2<f32>(1.0));
    let edge = min(min(warped.x, 1.0 - warped.x) / columns, min(warped.y, 1.0 - warped.y) / rows);
    let mortar = 1.0 - smoothstep(gap * 0.40, gap * 1.05 + 0.0001, edge);

    c = mix(c, c * vec3<f32>(0.44, 0.38, 0.34), mortar * 0.70);
    return max(c, vec3<f32>(0.0));
}

fn vv_leaf_shape(uv: vec2<f32>, center: vec2<f32>, angle: f32, radius: vec2<f32>) -> f32 {
    let p = uv - center;
    let ca = cos(angle);
    let sa = sin(angle);
    let q = vec2<f32>(
        ca * p.x - sa * p.y,
        sa * p.x + ca * p.y,
    ) / radius;

    let body = 1.0 - smoothstep(0.62, 1.0, dot(q, q));
    let tip = smoothstep(-0.95, 0.75, q.x) * (1.0 - smoothstep(0.48, 1.10, abs(q.y)));
    return saturate(body * tip);
}

fn vv_grass_top(
    visual: BlockVisual,
    base: vec3<f32>,
    uv: vec2<f32>,
    world_pos: vec3<f32>,
    seed: vec3<f32>,
) -> vec3<f32> {
    var grass = vec3<f32>(0.33, 0.61, 0.15);

    let face_top = vv_face_bias(visual, 0u);
    if (vv_has_color(face_top)) {
        grass = face_top;
    }

    let broad = vv_value_noise_3d(vec3<f32>(uv * 3.2, seed.z * 0.05));
    let fine = vv_value_noise_3d(vec3<f32>(uv * 11.0, seed.x * 0.03));
    grass *= 0.92 + broad * 0.16 + fine * 0.045;

    let cells = vv_voronoi((uv + broad * 0.025) * 6.2, seed + vec3<f32>(3.0, 9.0, 17.0));
    let mound = 1.0 - smoothstep(0.10, 0.54, cells.x);
    grass = mix(grass, grass * vec3<f32>(1.10, 1.18, 0.82), mound * 0.18);

    // Big readable leaf clusters, like the reference top.
    let tile = floor(uv * 5.3);
    let local_uv = fract(uv * 5.3);
    let tile_seed = seed + vec3<f32>(tile, 41.0);

    var leaf_mix = 0.0;
    var leaf_shadow = 0.0;

    for (var i: u32 = 0u; i < 4u; i = i + 1u) {
        let s = tile_seed + vec3<f32>(f32(i) * 19.0, f32(i) * 7.0, f32(i) * 3.0);
        let spawn = hash13(s + vec3<f32>(1.0, 2.0, 3.0));

        let center = vec2<f32>(
            hash13(s + vec3<f32>(5.0, 7.0, 11.0)),
            hash13(s + vec3<f32>(13.0, 17.0, 19.0)),
        );

        let angle = hash13(s + vec3<f32>(23.0, 29.0, 31.0)) * 6.28318;
        let size = mix(0.135, 0.235, hash13(s + vec3<f32>(37.0, 41.0, 43.0)));

        let leaf = vv_leaf_shape(local_uv, center, angle, vec2<f32>(size, size * 0.34));
        let enabled = select(0.0, 1.0, spawn > 0.20);
        leaf_mix = max(leaf_mix, leaf * enabled);

        let shadow_leaf = vv_leaf_shape(
            local_uv,
            center + vec2<f32>(0.025, -0.030),
            angle,
            vec2<f32>(size * 1.08, size * 0.40)
        );
        leaf_shadow = max(leaf_shadow, shadow_leaf * enabled);
    }

    let leaf_light = vec3<f32>(0.58, 0.78, 0.22);
    let leaf_dark = vec3<f32>(0.20, 0.43, 0.10);

    grass = mix(grass, grass * leaf_dark, leaf_shadow * 0.22);
    grass = mix(grass, leaf_light, leaf_mix * 0.30);

    // Strong candy bevel on the top border.
    let edge_x = min(uv.x, 1.0 - uv.x);
    let edge_y = min(uv.y, 1.0 - uv.y);
    let edge = 1.0 - smoothstep(0.020, 0.145, min(edge_x, edge_y));
    grass = mix(grass, grass * vec3<f32>(0.56, 0.78, 0.32), edge * 0.24);
    grass = mix(grass, grass * vec3<f32>(1.16, 1.14, 0.88), edge * 0.10);

    return max(grass, vec3<f32>(0.0));
}

fn vv_grass_side(
    visual: BlockVisual,
    soil_base: vec3<f32>,
    uv: vec2<f32>,
    world_pos: vec3<f32>,
    seed: vec3<f32>,
) -> vec3<f32> {
    // Darker chocolate soil, closer to the reference.
    var soil = vec3<f32>(0.36, 0.21, 0.11);
    let side_bias = vv_face_bias(visual, 2u);
    if (vv_has_color(side_bias)) {
        soil = side_bias;
    }

    // Large rounded chunks instead of thin beige cracks.
    let density = 4.15;
    let cell_seed = seed + vec3<f32>(71.0, 13.0, 5.0);
    let warp = vec2<f32>(
        vv_value_noise_3d(world_pos * 0.22 + cell_seed * 0.017),
        vv_value_noise_3d(world_pos * 0.29 + cell_seed * 0.029)
    ) * 0.08;

    let cells = vv_voronoi((uv + warp) * density, cell_seed);
    let closest = cells.x;
    let second = cells.y;
    let cell_hash = cells.z;

    let joint = 1.0 - smoothstep(0.018, 0.125, second - closest);
    let center = 1.0 - smoothstep(0.06, 0.58, closest);

    soil *= 0.84 + cell_hash * 0.30;
    soil = mix(soil, soil * vec3<f32>(1.28, 1.14, 0.88), center * 0.15);
    soil = mix(soil, vec3<f32>(0.17, 0.10, 0.055), joint * 0.68);

    // Rounded pebble patches on the blocks.
    let pebble_grid = vec2<f32>(5.3, 4.8);
    let pebble_cell = floor(uv * pebble_grid);
    let pebble_local = fract(uv * pebble_grid);
    let pebble_seed = seed + vec3<f32>(pebble_cell, 101.0);
    let pebble_spawn = hash13(pebble_seed);

    let pebble_center = vec2<f32>(
        hash13(pebble_seed + vec3<f32>(2.0, 4.0, 6.0)),
        hash13(pebble_seed + vec3<f32>(8.0, 10.0, 12.0))
    );

    let pebble_q = (pebble_local - pebble_center) / vec2<f32>(0.25, 0.17);
    let pebble = (1.0 - smoothstep(0.50, 1.0, dot(pebble_q, pebble_q))) *
        select(0.0, 1.0, pebble_spawn > 0.72);

    let pebble_shadow = (1.0 - smoothstep(0.64, 1.12, dot((pebble_local - pebble_center - vec2<f32>(0.035, -0.030)) / vec2<f32>(0.28, 0.20), (pebble_local - pebble_center - vec2<f32>(0.035, -0.030)) / vec2<f32>(0.28, 0.20)))) *
        select(0.0, 1.0, pebble_spawn > 0.72);

    soil = mix(soil, soil * vec3<f32>(0.45, 0.34, 0.25), pebble_shadow * 0.22);
    soil = mix(soil, vec3<f32>(0.58, 0.34, 0.17), pebble * 0.40);

    // Grass cap from the real top of the side face.
    let side_from_top = 1.0 - uv.y;

    var grass = vec3<f32>(0.33, 0.61, 0.15);
    let grass_top = vv_face_bias(visual, 0u);
    if (vv_has_color(grass_top)) {
        grass = grass_top;
    }

    let x_noise = vv_value_noise_3d(vec3<f32>(uv.x * 7.0, seed.x * 0.03, seed.y * 0.05));
    let thin_noise = vv_value_noise_3d(vec3<f32>(uv.x * 19.0, seed.z * 0.04, 9.0));

    // Scalloped leaf curtain. Stronger in columns, like dripping grass lobes.
    let scallop = 1.0 - smoothstep(0.12, 0.36, abs(fract(uv.x * 8.0 + x_noise * 0.32) - 0.5));
    let drip_depth = 0.155 + x_noise * 0.135 + scallop * 0.105 + thin_noise * 0.030;

    let cap = 1.0 - smoothstep(drip_depth - 0.032, drip_depth + 0.042, side_from_top);
    let lower_shadow = smoothstep(drip_depth - 0.010, drip_depth + 0.085, side_from_top) *
        (1.0 - smoothstep(drip_depth + 0.05, drip_depth + 0.16, side_from_top));

    var out_color = mix(soil, grass, cap * 0.96);

    // Dark shadow just under the grass cap, huge visual difference.
    out_color = mix(out_color, soil * vec3<f32>(0.48, 0.34, 0.22), lower_shadow * 0.46);

    // Individual darker lobes.
    let lobe = cap * scallop * smoothstep(0.035, 0.18, side_from_top);
    out_color = mix(out_color, grass * vec3<f32>(0.48, 0.68, 0.30), lobe * 0.26);

    // Small highlights on grass curtain.
    let cap_highlight = cap * (1.0 - smoothstep(0.00, 0.075, side_from_top));
    out_color = mix(out_color, grass * vec3<f32>(1.20, 1.16, 0.84), cap_highlight * 0.11);

    return max(out_color, vec3<f32>(0.0));
}

fn vv_rings_material(
    visual: BlockVisual,
    base: vec3<f32>,
    uv: vec2<f32>,
    face_id: u32,
    seed: vec3<f32>,
) -> vec3<f32> {
    if (face_id == 0u || face_id == 1u) {
        let p = uv - vec2<f32>(0.5);
        let r = length(p);
        let rings = max(f32(visual.patterned.rows), 5.0);
        let wobble = sin(p.x * 8.7 + hash13(seed) * 6.28318) * 0.012 +
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
        var top = vv_face_bias(visual, 0u);
        if (!vv_has_color(top)) {
            top = vec3<f32>(0.36, 0.64, 0.16);
        }

        return vv_grass_top(visual, top, uv, world_pos, seed);
    }

    if (face_id == 1u) {
        var bottom = vv_face_bias(visual, 1u);
        if (!vv_has_color(bottom)) {
            bottom = base * vec3<f32>(0.60, 0.48, 0.38);
        }
        return bottom;
    }

    var side = vv_face_bias(visual, face_id);
    if (!vv_has_color(side)) {
        side = vv_face_bias(visual, 2u);
    }
    if (!vv_has_color(side)) {
        side = vec3<f32>(0.46, 0.26, 0.13);
    }

    return vv_grass_side(visual, side, uv, world_pos, seed);
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
    let average = clamp((min_size + max_size) * 0.5, 0.005, 0.60);
    let cell_scale = clamp(1.0 / average, 2.0, 42.0);

    let p = uv * cell_scale;
    let cell = floor(p);
    let local_uv = fract(p);

    let seed = face_seed(
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

    let centered = local_uv - center;
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
        let leaf = vv_leaf_shape(local_uv, center, angle, vec2<f32>(radius * 0.95, radius * 0.36));
        m = leaf;
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
    let slope_weight = saturate(topness + sideness * 0.78 + bottomness * 0.30);

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

        c = mix(c, detail_color, mask * 0.75);
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

    let radius = 0.150;
    let edge_band = 1.0 - smoothstep(0.012, radius, edge);
    let corner_band = 1.0 - smoothstep(0.040, radius * 1.34, corner_dist);
    let bevel = saturate(edge_band * 0.82 + corner_band * 0.58);

    let topness = saturate(dot(normal, up));
    let bottomness = saturate(dot(-normal, up));
    let sideness = saturate(1.0 - topness - bottomness);

    var c = color;

    // Stronger dark outline at side/bottom edges, closer to toy render.
    c = mix(c, c * vec3<f32>(0.58, 0.62, 0.70), bevel * (0.18 + sideness * 0.25 + bottomness * 0.30));

    // Creamy highlight on top edges.
    c = mix(c, c * vec3<f32>(1.18, 1.14, 0.94), bevel * topness * 0.18);

    return max(vv_saturate_color(c, 1.12), vec3<f32>(0.0));
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
    let seed = vv_seed(voxel_pos, block_id, block_visual_id, face_id, variation_seed, 7.0);

    var color = vv_base_color(visual, face_id, seed);

    if (visual.procedural.w == SURFACE_PROGRAM_PATTERNED) {
        if (visual.patterned.kind == PATTERN_LAYERED_SURFACE) {
            color = vv_layered_surface_color(visual, color, world_pos, uv, face_id, seed);
        } else if (
            visual.patterned.kind == PATTERN_NATURAL_CELLS ||
            visual.patterned.kind == PATTERN_CRACKED_CELLS
        ) {
            let density = max(max(f32(visual.patterned.rows), f32(visual.patterned.columns)), 2.0);
            color = vv_cells_material(
                color,
                uv,
                world_pos,
                density,
                0.135 + visual.patterned.gap_width,
                saturate(visual.patterned.color_variation) * 0.75,
                seed,
            );
        } else if (
            visual.patterned.kind == PATTERN_GRID ||
            visual.patterned.kind == PATTERN_RUNNING_BOND ||
            visual.patterned.kind == PATTERN_STRIPS
        ) {
            color = vv_rect_material(visual, color, uv, seed);
        } else if (visual.patterned.kind == PATTERN_RINGS) {
            color = vv_rings_material(visual, color, uv, face_id, seed);
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

    return clamp(color, vec3<f32>(0.0), vec3<f32>(1.35));
}