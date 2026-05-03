const SURFACE_PROGRAM_PATTERNED: u32 = 1u;

const PATTERN_GRID: u32 = 0u;
const PATTERN_STRIPS: u32 = 1u;
const PATTERN_RUNNING_BOND: u32 = 2u;
const PATTERN_RINGS: u32 = 3u;
const PATTERN_NATURAL_CELLS: u32 = 4u;
const PATTERN_CRACKED_CELLS: u32 = 5u;
const PATTERN_LAYERED_SURFACE: u32 = 6u;

const PATTERN_FLAG_STAGGER: u32 = 1u;

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

fn vv_program_seed(
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

fn vv_face_bias(visual: BlockVisual, face_id: u32) -> vec3<f32> {
    return face_visual_for(visual, face_id).color_bias.rgb;
}

fn vv_authored_base_color(
    visual: BlockVisual,
    face_id: u32,
    seed: vec3<f32>,
) -> vec3<f32> {
    let palette_selector = hash13(seed + vec3<f32>(31.0, 17.0, 9.0));
    let palette = palette_color(visual, palette_selector);

    let authored_base = visual.base_color.rgb;
    let base_is_white = length(authored_base - vec3<f32>(1.0)) < 0.04;

    var base = authored_base;
    if (base_is_white) {
        base = palette;
    }

    let bias = vv_face_bias(visual, face_id);
    let has_face_bias = length(bias - vec3<f32>(1.0)) > 0.015;

    if (has_face_bias) {
        if (base_is_white) {
            base = mix(base, bias, 0.72);
        } else {
            base = base * bias;
        }
    }

    return max(base, vec3<f32>(0.0));
}

fn vv_directional_material_tint(normal: vec3<f32>, up: vec3<f32>) -> vec3<f32> {
    let topness = saturate(dot(normal, up));
    let bottomness = saturate(dot(-normal, up));
    let sideness = saturate(1.0 - topness - bottomness);

    var tint = vec3<f32>(1.0);
    tint *= mix(vec3<f32>(1.0), vec3<f32>(1.035, 1.025, 0.970), topness * 0.20);
    tint *= mix(vec3<f32>(1.0), vec3<f32>(0.900, 0.925, 1.020), sideness * 0.12);
    tint *= mix(vec3<f32>(1.0), vec3<f32>(0.720, 0.760, 0.850), bottomness * 0.30);

    return tint;
}

fn vv_apply_variation_pipeline(
    color: vec3<f32>,
    visual: BlockVisual,
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    up: vec3<f32>,
    voxel_pos: vec3<i32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    variation_seed: u32,
) -> vec3<f32> {
    let seed = vv_program_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        vec2<f32>(0.0, 0.0),
        19.37,
    );

    let per_voxel_tint = saturate(visual.variation_a.x);
    let per_face_tint = saturate(visual.variation_a.y);
    let macro_scale = safe_positive(visual.variation_a.z, 1.0);
    let macro_strength = saturate(visual.variation_a.w);
    let micro_scale = safe_positive(visual.variation_b.x, 1.0);
    let micro_strength = saturate(visual.variation_b.y);

    let voxel_hash = hash13(seed + vec3<f32>(5.0, 11.0, 23.0));
    let face_hash = hash13(seed + vec3<f32>(31.0, 47.0, 59.0));

    let macro_noise = vv_value_noise_3d(world_pos * (0.075 / macro_scale) + seed * 0.011);
    let micro_noise = vv_value_noise_3d(world_pos * (0.72 / micro_scale) + seed * 0.023);

    var varied = color;
    varied *= 1.0 + (voxel_hash - 0.5) * per_voxel_tint;
    varied *= 1.0 + (face_hash - 0.5) * per_face_tint;
    varied *= 1.0 + (macro_noise - 0.5) * macro_strength;
    varied *= 1.0 + (micro_noise - 0.5) * micro_strength;

    varied *= vv_directional_material_tint(normal, up);

    return max(varied, vec3<f32>(0.0));
}

fn flat_block_albedo(
    visual: BlockVisual,
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    voxel_pos: vec3<i32>,
    variation_seed: u32,
    up: vec3<f32>,
) -> vec3<f32> {
    let seed = vv_program_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        vec2<f32>(0.0, 0.0),
        7.0,
    );

    let base = vv_authored_base_color(visual, face_id, seed);

    return vv_apply_variation_pipeline(
        base,
        visual,
        world_pos,
        normal,
        up,
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
    );
}

fn vv_patterned_cell_coords(visual: BlockVisual, uv: vec2<f32>) -> vec4<f32> {
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

fn vv_rect_mortar_mask(visual: BlockVisual, cell_uv: vec2<f32>, jitter: vec2<f32>) -> f32 {
    let rows = max(f32(visual.patterned.rows), 1.0);
    let columns = max(f32(visual.patterned.columns), 1.0);
    let gap = clamp(visual.patterned.gap_width, 0.0, 0.20);

    let warped_uv = clamp(cell_uv + jitter * 0.045, vec2<f32>(0.0), vec2<f32>(1.0));

    let edge_x = min(warped_uv.x, 1.0 - warped_uv.x) / columns;
    let edge_y = min(warped_uv.y, 1.0 - warped_uv.y) / rows;
    let edge = min(edge_x, edge_y);

    return 1.0 - smoothstep(gap * 0.40, gap * 1.05 + 0.0001, edge);
}

fn vv_rect_pattern_color(
    visual: BlockVisual,
    base: vec3<f32>,
    uv: vec2<f32>,
    voxel_pos: vec3<i32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    variation_seed: u32,
) -> vec3<f32> {
    let coords = vv_patterned_cell_coords(visual, uv);
    let cell = coords.xy;
    let cell_uv = coords.zw;

    let seed = vv_program_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        cell,
        f32(visual.patterned.seed & 65535u),
    );

    let h = hash13(seed + vec3<f32>(13.0, 31.0, 73.0));
    let h2 = hash13(seed + vec3<f32>(97.0, 11.0, 41.0));

    var color = base;
    color *= 1.0 + (h - 0.5) * 2.0 * saturate(visual.patterned.color_variation);

    let center_distance = length(cell_uv - vec2<f32>(0.5));
    let center_highlight = 1.0 - smoothstep(0.10, 0.72, center_distance);
    color *= 1.0 + center_highlight * visual.patterned.cell_pillow * 1.65;

    // Rectangular programs are allowed to show authored structure,
    // but only for blocks that explicitly ask for grid/running_bond/strips.
    let jitter = vec2<f32>(h - 0.5, h2 - 0.5);
    let mortar = vv_rect_mortar_mask(visual, cell_uv, jitter);

    let mortar_tint = mix(vec3<f32>(0.56), vec3<f32>(0.38), saturate(visual.patterned.gap_depth * 7.0));
    color = mix(color, color * mortar_tint, mortar * 0.68);

    return max(color, vec3<f32>(0.0));
}

fn vv_voronoi_cells(p: vec2<f32>, seed: vec3<f32>) -> vec3<f32> {
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

fn vv_natural_cell_color(
    visual: BlockVisual,
    base: vec3<f32>,
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
    voxel_pos: vec3<i32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    variation_seed: u32,
) -> vec3<f32> {
    let seed = vv_program_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        vec2<f32>(0.0, 0.0),
        f32(visual.patterned.seed & 65535u) + 43.0,
    );

    let density = max(max(f32(visual.patterned.rows), f32(visual.patterned.columns)), 2.0);
    let warped_uv = uv +
        vec2<f32>(
            vv_value_noise_3d(world_pos * 0.31 + seed * 0.017),
            vv_value_noise_3d(world_pos * 0.37 + seed * 0.029),
        ) * 0.12;

    let cells = vv_voronoi_cells(warped_uv * density, seed);
    let closest = cells.x;
    let second = cells.y;
    let cell_hash = cells.z;

    let boundary = 1.0 - smoothstep(
        0.025,
        0.135 + visual.patterned.gap_width * 1.6,
        second - closest,
    );

    let cell_color_variation = (cell_hash - 0.5) * 2.0 * saturate(visual.patterned.color_variation);

    var color = base;
    color *= 1.0 + cell_color_variation;

    let pillow = 1.0 - smoothstep(0.05, 0.62, closest);
    color *= 1.0 + pillow * visual.patterned.cell_pillow * 1.4;

    let crack_amount = max(
        visual.patterned.crack_density,
        select(0.0, 0.18, visual.patterned.kind == PATTERN_CRACKED_CELLS),
    );

    let crack_hash = hash13(seed + vec3<f32>(floor(warped_uv * density), 71.0));
    let crack = boundary * select(0.0, 1.0, crack_hash < crack_amount + visual.patterned.gap_width);

    color = mix(color, color * vec3<f32>(0.48, 0.48, 0.46), boundary * visual.patterned.gap_depth * 5.0);
    color = mix(color, color * vec3<f32>(0.30, 0.30, 0.28), crack * 0.62);

    return max(color, vec3<f32>(0.0));
}

// Radial cross-section pattern for the rings shader pattern.
// Used on faces perpendicular to the rings axis (top/bottom of a log).
fn vv_rings_radial_color(
    visual: BlockVisual,
    base: vec3<f32>,
    uv: vec2<f32>,
    voxel_pos: vec3<i32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    variation_seed: u32,
) -> vec3<f32> {
    let p = uv - vec2<f32>(0.5);
    let r = length(p);

    let seed = vv_program_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        vec2<f32>(0.0, 0.0),
        f32(visual.patterned.seed & 65535u) + 101.0,
    );

    let rings = max(f32(visual.patterned.rows), 5.0);
    let wobble =
        sin(p.x * 8.7 + hash13(seed) * 6.28318) * 0.012 +
        sin(p.y * 7.9 + hash13(seed + vec3<f32>(9.0, 4.0, 2.0)) * 6.28318) * 0.010;

    let rr = r + wobble;
    let ring_wave = abs(fract(rr * rings * 1.42) - 0.5);
    let ring_mask = 1.0 - smoothstep(0.060, 0.170, ring_wave);

    let core = 1.0 - smoothstep(0.00, 0.13, r);
    let outer = smoothstep(0.39, 0.54, r);

    var color = base;
    color = mix(color, color * vec3<f32>(1.18, 1.08, 0.86), ring_mask * 0.28);
    color = mix(color, color * vec3<f32>(1.34, 1.18, 0.88), core * 0.32);
    color = mix(color, color * vec3<f32>(0.58, 0.36, 0.18), outer * 0.50);

    let grain = hash13(seed + vec3<f32>(14.0, 71.0, 6.0));
    color *= 1.0 + (grain - 0.5) * saturate(visual.patterned.color_variation) * 0.55;

    return max(color, vec3<f32>(0.0));
}

// Vertical streak pattern for the rings shader pattern.
// Used on faces parallel to the rings axis (sides of a log).
fn vv_rings_axial_color(
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
    let seed = vv_program_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        vec2<f32>(0.0, 0.0),
        f32(visual.patterned.seed & 65535u) + 211.0,
    );

    let columns = max(f32(visual.patterned.columns), 6.0);

    let vertical_noise = vv_value_noise_3d(vec3<f32>(uv.x * columns * 0.45, uv.y * 3.2, seed.x * 0.05));
    let fine_noise = vv_value_noise_3d(vec3<f32>(uv.x * columns * 2.2, uv.y * 15.0, seed.y * 0.05));

    let streak = 1.0 - smoothstep(0.20, 0.47, abs(fract(uv.x * columns + vertical_noise * 0.75) - 0.5));
    let dark_streak = 1.0 - smoothstep(0.05, 0.17, abs(fract(uv.x * columns * 1.7 + fine_noise) - 0.5));

    var color = base;
    color = mix(color, color * vec3<f32>(0.60, 0.38, 0.19), streak * 0.28);
    color = mix(color, color * vec3<f32>(0.42, 0.25, 0.12), dark_streak * 0.20);

    let patch_noise = vv_value_noise_3d(world_pos * 0.85 + seed * 0.015);
    color *= 1.0 + (patch_noise - 0.5) * saturate(visual.patterned.color_variation) * 0.55;

    return max(color, vec3<f32>(0.0));
}

// Generic two-zone surface pattern.
//
// Top face (face_id == 0): authored top color with horizontal fiber/blade
//   stripes for relief. Density = visual.patterned.rows.
// Side faces (face_id >= 2): the top color (read from face 0's bias) bleeds
//   down from the top edge as a fringe. Fringe height comes from
//   visual.patterned.gap_width (treated as a fraction in [0, 0.5] of UV.y).
//   The fringe boundary is irregularised by value noise driven by
//   visual.patterned.height_variation.
// Bottom face (face_id == 1): unmodified — uses the authored bottom color.
//
// This pattern is purely shader-driven; the mesh stays a clean soft cube.
// It supersedes the legacy "grass" hardcoded program and can drive grass,
// snow caps, moss, etc. solely from the .ron.
fn vv_layered_surface_color(
    visual: BlockVisual,
    base: vec3<f32>,
    uv: vec2<f32>,
    face_id: u32,
    seed: vec3<f32>,
) -> vec3<f32> {
    var color = base;

    if (face_id == 0u) {
        let fiber_density = max(f32(visual.patterned.rows), 6.0);
        let jitter = hash13(seed + vec3<f32>(43.0, 7.0, 19.0));
        let fiber_wave = abs(fract((uv.x + jitter * 0.31) * fiber_density * 0.55) - 0.5);
        let fiber_mask = 1.0 - smoothstep(0.10, 0.32, fiber_wave);

        let patch_h = hash13(vec3<f32>(floor(uv.x * 3.0), floor(uv.y * 3.0), seed.z));
        color *= 1.0 + (patch_h - 0.5) * saturate(visual.patterned.color_variation) * 0.50;
        color = mix(color, color * vec3<f32>(1.10, 1.20, 0.86), fiber_mask * 0.20);
        return max(color, vec3<f32>(0.0));
    }

    if (face_id == 1u) {
        return color;
    }

    // Side face — bleed the top color down as an irregular fringe.
    let top_color = vv_face_bias(visual, 0u);
    let has_top_bias = length(top_color - vec3<f32>(1.0)) > 0.015;
    let target = select(color * vec3<f32>(0.62, 1.10, 0.55), top_color, has_top_bias);

    // gap_width is .ron-clamped to [0, 0.2]; treat that range as the full
    // [0, 0.5] fringe-height span so layered_surface can author tall fringes.
    let fringe_h = clamp(visual.patterned.gap_width * 2.5, 0.05, 0.50);
    // height_variation is .ron-clamped to [0, 0.15]; map it to [0, 0.6] so
    // small authored values produce visibly irregular fringe edges.
    let irregularity = saturate(visual.patterned.height_variation * 4.0) * 0.10;
    let v_from_top = 1.0 - uv.y;

    let noise = vv_value_noise_3d(vec3<f32>(uv.x * 14.0, seed.x * 0.05, seed.y * 0.03));
    let fringe_edge = fringe_h + (noise - 0.5) * irregularity * 2.0;
    let fringe = 1.0 - smoothstep(fringe_edge - 0.03, fringe_edge + 0.04, v_from_top);

    return max(mix(color, target, fringe), vec3<f32>(0.0));
}

fn patterned_block_albedo(
    visual: BlockVisual,
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
    let seed = vv_program_seed(
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
        variation_seed,
        vec2<f32>(0.0, 0.0),
        f32(visual.patterned.seed & 65535u),
    );

    let base = vv_authored_base_color(visual, face_id, seed);

    var patterned = base;

    if (visual.patterned.kind == PATTERN_RINGS) {
        if (face_id == 0u || face_id == 1u) {
            patterned = vv_rings_radial_color(
                visual,
                base,
                uv,
                voxel_pos,
                block_id,
                block_visual_id,
                face_id,
                variation_seed,
            );
        } else {
            patterned = vv_rings_axial_color(
                visual,
                base,
                world_pos,
                uv,
                voxel_pos,
                block_id,
                block_visual_id,
                face_id,
                variation_seed,
            );
        }
    } else if (visual.patterned.kind == PATTERN_LAYERED_SURFACE) {
        patterned = vv_layered_surface_color(visual, base, uv, face_id, seed);
    } else if (
        visual.patterned.kind == PATTERN_NATURAL_CELLS ||
        visual.patterned.kind == PATTERN_CRACKED_CELLS
    ) {
        patterned = vv_natural_cell_color(
            visual,
            base,
            world_pos,
            normal,
            uv,
            voxel_pos,
            block_id,
            block_visual_id,
            face_id,
            variation_seed,
        );
    } else {
        patterned = vv_rect_pattern_color(
            visual,
            base,
            uv,
            voxel_pos,
            block_id,
            block_visual_id,
            face_id,
            variation_seed,
        );
    }

    return vv_apply_variation_pipeline(
        patterned,
        visual,
        world_pos,
        normal,
        up,
        voxel_pos,
        block_id,
        block_visual_id,
        face_id,
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

    if (visual.procedural.w == SURFACE_PROGRAM_PATTERNED) {
        return patterned_block_albedo(
            visual,
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

    return flat_block_albedo(
        visual,
        world_pos,
        normal,
        block_id,
        block_visual_id,
        face_id,
        voxel_pos,
        variation_seed,
        up,
    );
}