const SURFACE_PROGRAM_FLAT: u32 = 0u;
const SURFACE_PROGRAM_GRASS: u32 = 1u;
const SURFACE_PROGRAM_DIRT: u32 = 2u;
const SURFACE_PROGRAM_STONE: u32 = 3u;
const SURFACE_PROGRAM_STONE_BRICKS: u32 = 4u;
const SURFACE_PROGRAM_WOOD_LOG: u32 = 5u;
const SURFACE_PROGRAM_WOOD_PLANKS: u32 = 6u;
const SURFACE_PROGRAM_SAND: u32 = 7u;
const SURFACE_PROGRAM_SNOW: u32 = 8u;
const SURFACE_PROGRAM_ICE: u32 = 9u;
const SURFACE_PROGRAM_LEAVES: u32 = 10u;
const SURFACE_PROGRAM_LAVA: u32 = 11u;
const SURFACE_PROGRAM_CRYSTAL: u32 = 12u;
const SURFACE_PROGRAM_ORE: u32 = 13u;
const SURFACE_PROGRAM_MUSHROOM: u32 = 14u;

fn visual_surface_program(visual: BlockVisual) -> u32 {
    return visual.procedural.w;
}

fn visual_geometry_profile(visual: BlockVisual) -> u32 {
    return u32(max(visual.shape.z + 0.5, 0.0));
}

fn visual_edge_roundness(visual: BlockVisual) -> f32 {
    return saturate(visual.shape.w);
}

fn visual_face_pillow(visual: BlockVisual) -> f32 {
    return saturate(visual.surface.w);
}

fn vv_program_edge_factor(uv: vec2<f32>) -> f32 {
    let edge_dist = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
    return 1.0 - smoothstep(0.0, 0.18, edge_dist);
}

fn vv_program_square_ring(cell_uv: vec2<f32>, grid_size: f32) -> f32 {
    let centered = abs((cell_uv + vec2<f32>(0.5 / grid_size)) * 2.0 - 1.0);
    let ring = max(centered.x, centered.y) * grid_size * 0.55;
    return abs(fract(ring) - 0.5);
}

fn vv_program_cell_center_mask(uv: vec2<f32>) -> f32 {
    let d = length(uv - vec2<f32>(0.5));
    return 1.0 - smoothstep(0.10, 0.78, d);
}

fn vv_program_soft_noise(seed: vec3<f32>, amount: f32) -> f32 {
    return (hash13(seed) - 0.5) * 2.0 * amount;
}

fn program_wood_log_color(
    visual: BlockVisual,
    color: vec3<f32>,
    uv: vec2<f32>,
    face_id: u32,
    cell: vec2<f32>,
    grid_size: f32,
    seed: vec3<f32>,
) -> vec3<f32> {
    var c = color;
    let h0 = hash13(seed + vec3<f32>(7.0, 11.0, 13.0));
    let h1 = hash13(seed + vec3<f32>(17.0, 19.0, 23.0));

    if (face_id == 0u || face_id == 1u) {
        let p = uv - vec2<f32>(0.5);
        let r = length(p);
        let ring_wave = abs(fract(r * grid_size * 1.75 + h0 * 0.22) - 0.5);
        let ring = 1.0 - smoothstep(0.060, 0.210, ring_wave);
        let core = 1.0 - smoothstep(0.00, 0.18, r);
        let radial_warmth = 1.0 - smoothstep(0.20, 0.76, r);

        c *= 1.0 + radial_warmth * 0.18;
        c = mix(c, c * vec3<f32>(1.28, 1.12, 0.82), ring * 0.28);
        c = mix(c, c * vec3<f32>(1.42, 1.22, 0.92), core * 0.38);

        let tiny_cut = select(0.0, 1.0, h1 > 0.86) * 0.10;
        c *= 1.0 + tiny_cut;
    } else {
        let ridge_wave = abs(fract((uv.x + h0 * 0.18) * grid_size * 0.42) - 0.5);
        let ridge = 1.0 - smoothstep(0.08, 0.30, ridge_wave);
        let vertical_break = mix(0.72, 1.0, hash13(vec3<f32>(floor(uv.x * grid_size * 0.55), floor(uv.y * grid_size * 0.22), seed.z)));
        let knot_center = vec2<f32>(0.30 + h0 * 0.40, 0.25 + h1 * 0.48);
        let knot = 1.0 - smoothstep(0.055, 0.165, length(uv - knot_center));

        c *= 1.0 - ridge * vertical_break * 0.18;
        c = mix(c, c * vec3<f32>(0.58, 0.42, 0.28), knot * 0.58);

        let side_highlight = smoothstep(0.15, 0.52, uv.y) * (1.0 - smoothstep(0.62, 1.0, uv.y));
        c *= 1.0 + side_highlight * 0.045;
    }

    return max(c, vec3<f32>(0.0));
}

fn program_grass_color(
    visual: BlockVisual,
    color: vec3<f32>,
    uv: vec2<f32>,
    face_id: u32,
    cell: vec2<f32>,
    grid_size: f32,
    seed: vec3<f32>,
) -> vec3<f32> {
    var c = color;
    let h = hash13(seed + vec3<f32>(31.0, 37.0, 41.0));

    if (face_id == 0u) {
        let blade_wave = abs(fract((uv.x + h * 0.15) * grid_size * 0.58) - 0.5);
        let blades = 1.0 - smoothstep(0.10, 0.32, blade_wave);
        let patch_v = hash13(vec3<f32>(floor(cell.x / 2.0), floor(cell.y / 2.0), seed.z));
        c *= 1.0 + (patch_v - 0.5) * 0.18;
        c = mix(c, c * vec3<f32>(1.14, 1.24, 0.86), blades * 0.18);
    } else if (face_id >= 2u) {
        let fringe = 1.0 - smoothstep(0.10, 0.42, uv.y);
        c = mix(c, c * vec3<f32>(0.72, 1.12, 0.58), fringe * 0.50);
    }

    return max(c, vec3<f32>(0.0));
}

fn program_stone_color(
    visual: BlockVisual,
    color: vec3<f32>,
    uv: vec2<f32>,
    face_id: u32,
    cell: vec2<f32>,
    grid_size: f32,
    seed: vec3<f32>,
) -> vec3<f32> {
    var c = color;
    let vein = hash13(vec3<f32>(floor(cell.x / 2.0), floor(cell.y / 2.0), seed.z));
    let chip = hash13(seed + vec3<f32>(53.0, 59.0, 61.0));
    let plate = abs(fract((uv.x + uv.y * 0.73 + chip * 0.20) * grid_size * 0.22) - 0.5);
    let plate_mask = 1.0 - smoothstep(0.10, 0.34, plate);

    c *= 1.0 + (vein - 0.5) * 0.16;
    c = mix(c, c * vec3<f32>(1.18), plate_mask * 0.06);
    c = mix(c, c * vec3<f32>(0.70), select(0.0, 1.0, chip > 0.88) * 0.12);

    return max(c, vec3<f32>(0.0));
}

fn program_leaves_color(
    visual: BlockVisual,
    color: vec3<f32>,
    uv: vec2<f32>,
    face_id: u32,
    cell: vec2<f32>,
    grid_size: f32,
    seed: vec3<f32>,
) -> vec3<f32> {
    var c = color;
    let leaf_cluster = hash13(vec3<f32>(floor(cell.x / 2.0), floor(cell.y / 2.0), seed.z));
    let vein_wave = abs(fract((uv.x - uv.y * 0.55 + leaf_cluster * 0.21) * grid_size * 0.34) - 0.5);
    let vein = 1.0 - smoothstep(0.08, 0.22, vein_wave);
    c *= 1.0 + (leaf_cluster - 0.5) * 0.20;
    c = mix(c, c * vec3<f32>(1.10, 1.20, 0.84), vein * 0.13);
    return max(c, vec3<f32>(0.0));
}

fn apply_surface_program_color(
    visual: BlockVisual,
    color: vec3<f32>,
    uv: vec2<f32>,
    face_id: u32,
    cell: vec2<f32>,
    grid_size: f32,
    seed: vec3<f32>,
) -> vec3<f32> {
    let program = visual_surface_program(visual);

    switch program {
        case SURFACE_PROGRAM_GRASS: {
            return program_grass_color(visual, color, uv, face_id, cell, grid_size, seed);
        }
        case SURFACE_PROGRAM_DIRT: {
            return program_stone_color(visual, color, uv, face_id, cell, grid_size, seed) * vec3<f32>(1.04, 0.88, 0.70);
        }
        case SURFACE_PROGRAM_STONE: {
            return program_stone_color(visual, color, uv, face_id, cell, grid_size, seed);
        }
        case SURFACE_PROGRAM_STONE_BRICKS: {
            let brick_line = min(
                abs(fract(uv.y * grid_size * 0.22) - 0.5),
                abs(fract((uv.x + floor(uv.y * grid_size * 0.22) * 0.5) * grid_size * 0.22) - 0.5),
            );
            let mortar = 1.0 - smoothstep(0.035, 0.085, brick_line);
            return mix(program_stone_color(visual, color, uv, face_id, cell, grid_size, seed), color * 0.58, mortar * 0.55);
        }
        case SURFACE_PROGRAM_WOOD_LOG: {
            return program_wood_log_color(visual, color, uv, face_id, cell, grid_size, seed);
        }
        case SURFACE_PROGRAM_WOOD_PLANKS: {
            let plank_line = 1.0 - smoothstep(0.020, 0.070, abs(fract(uv.y * 4.0) - 0.5));
            let grain = abs(fract((uv.x + hash13(seed) * 0.2) * grid_size * 0.34) - 0.5);
            let grain_mask = 1.0 - smoothstep(0.08, 0.24, grain);
            var c = color * (1.0 - plank_line * 0.20);
            c = mix(c, c * vec3<f32>(1.18, 1.06, 0.82), grain_mask * 0.12);
            return c;
        }
        case SURFACE_PROGRAM_SAND: {
            let speck = hash13(seed + vec3<f32>(67.0, 71.0, 73.0));
            return color * (1.0 + (speck - 0.5) * 0.12);
        }
        case SURFACE_PROGRAM_SNOW: {
            let sparkle = select(0.0, 1.0, hash13(seed + vec3<f32>(79.0, 83.0, 89.0)) > 0.94);
            return color * vec3<f32>(1.08, 1.10, 1.18) + sparkle * vec3<f32>(0.07);
        }
        case SURFACE_PROGRAM_LEAVES: {
            return program_leaves_color(visual, color, uv, face_id, cell, grid_size, seed);
        }
        default: {
            return color;
        }
    }
}

fn apply_stylized_voxel_finish(
    visual: BlockVisual,
    color: vec3<f32>,
    uv: vec2<f32>,
    face_id: u32,
    seed: vec3<f32>,
) -> vec3<f32> {
    var c = color;

    let edge = vv_program_edge_factor(uv);
    let center = vv_program_cell_center_mask(uv);

    let pillow = visual_face_pillow(visual);
    let roundness = visual_edge_roundness(visual);

    c *= 1.0 + center * pillow * 0.10;
    c *= 1.0 - edge * roundness * 0.12;

    let profile = visual_geometry_profile(visual);
    if (profile == 1u || profile == 2u || profile == 7u) {
        c *= 1.0 + center * 0.035;
    }

    if (profile == 3u || profile == 4u || profile == 5u) {
        c *= 1.0 - edge * 0.045;
    }

    let tiny = vv_program_soft_noise(seed + vec3<f32>(97.0, 101.0, 103.0), 0.025);
    c *= 1.0 + tiny;

    return max(c, vec3<f32>(0.0));
}
