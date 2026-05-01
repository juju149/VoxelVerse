fn face_seed(
    voxel_pos: vec3<i32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    variation_seed: u32,
    cell: vec2<f32>,
    salt: f32,
) -> vec3<f32> {
    return vec3<f32>(
        f32(voxel_pos.x) * 11.7 + f32(block_id) * 0.37 + f32(variation_seed & 65535u) * 0.013 + cell.x * 1.97 + salt,
        f32(voxel_pos.y) * 7.3 + f32(face_id) * 3.11 + f32(variation_seed >> 16u) * 0.017 + cell.y * 2.41 + salt * 1.7,
        f32(voxel_pos.z) * 5.9 + f32(block_visual_id) * 0.53 + f32(face_id) * 0.19 + salt * 2.3,
    );
}

fn macro_cluster_hash(cell: vec2<f32>, seed: vec3<f32>, grid_size: u32, authored_scale: f32) -> f32 {
    let cluster_span = clamp(
        floor(f32(grid_size) / max(authored_scale * 3.0, 1.0)),
        1.0,
        f32(grid_size),
    );
    let cluster = floor(cell / cluster_span);
    return hash13(vec3<f32>(cluster, seed.x + seed.y + seed.z));
}

fn edge_factor(uv: vec2<f32>) -> f32 {
    let edge_dist = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));

    // Phase 1 tweak:
    // 0.10 instead of 0.14 gives sharper voxel separation.
    return 1.0 - smoothstep(0.0, 0.10, edge_dist);
}

fn square_ring(cell_uv: vec2<f32>, grid_size: f32) -> f32 {
    let centered = abs((cell_uv + vec2<f32>(0.5 / grid_size)) * 2.0 - 1.0);
    let ring = max(centered.x, centered.y) * grid_size * 0.55;
    return abs(fract(ring) - 0.5);
}

fn detail_strength(
    detail: BlockDetail,
    face_id: u32,
    cell: vec2<f32>,
    grid_size: f32,
    uv: vec2<f32>,
    topness: f32,
    sideness: f32,
    bottomness: f32,
    seed: vec3<f32>,
) -> f32 {
    let density = saturate(detail.params.x);
    let slope_pref = mix(1.0, saturate(topness + sideness * 0.65), saturate(detail.params.w));
    let cell_hash = hash13(seed + vec3<f32>(f32(detail.kind_data.y), 1.7, 4.9));
    let cluster_hash = hash13(floor(vec3<f32>(cell / max(1.0, detail.params.z * grid_size * 6.0), f32(detail.kind_data.y & 255u))));
    var strength = 0.0;

    switch detail.kind_data.x {
        case 1u: {
            let ridge_col = abs(fract((cell.x + f32(detail.kind_data.y & 7u)) * 0.33) - 0.5);
            let ridge = 1.0 - smoothstep(0.12, 0.34, ridge_col);
            strength = ridge * saturate(0.35 + density) * mix(0.5, 1.0, cluster_hash);
        }
        case 2u: {
            let ring = 1.0 - smoothstep(0.10, 0.24, square_ring(cell / grid_size, grid_size));
            strength = ring * saturate(0.25 + density);
        }
        case 3u: {
            let stripe = 1.0 - smoothstep(0.10, 0.34, abs(fract((cell.x + f32(detail.kind_data.y & 15u)) * 0.5) - 0.5));
            strength = stripe * (1.0 - smoothstep(0.18, 0.42, uv.y)) * saturate(0.3 + density);
        }
        case 4u: {
            let streak = 1.0 - smoothstep(0.12, 0.30, abs(fract((cell.x + f32(detail.kind_data.y & 31u)) * 0.4) - 0.5));
            strength = streak * smoothstep(0.52, 0.95, uv.y) * saturate(0.2 + density);
        }
        case 5u: {
            strength = select(0.0, 1.0, cell_hash > (1.0 - density)) * mix(0.6, 1.0, cluster_hash);
        }
        case 6u: {
            strength = select(0.0, 1.0, cluster_hash > (1.0 - density * 0.9)) * saturate(0.35 + topness * 0.45 + sideness * 0.2);
        }
        case 7u: {
            strength = select(0.0, 1.0, cell_hash > (0.55 - density * 0.4)) * 0.45;
        }
        case 8u: {
            let speck = select(0.0, 1.0, cell_hash > (1.0 - density * 0.85));
            let ring = 1.0 - smoothstep(0.14, 0.30, square_ring(cell / grid_size, grid_size));
            strength = max(speck * 0.75, ring * 0.45);
        }
        case 9u: {
            strength = select(0.0, 1.0, cell_hash > (1.0 - density * 0.45)) * 0.9;
        }
        default: {
            strength = select(0.0, 1.0, cluster_hash > (1.0 - density)) * mix(0.45, 1.0, cell_hash);
        }
    }

    if (face_id == 0u) {
        strength = strength * mix(0.8, 1.0, topness);
    } else if (face_id == 1u) {
        strength = strength * mix(0.4, 0.8, bottomness);
    } else {
        strength = strength * mix(0.65, 1.0, sideness);
    }

    return saturate(strength * slope_pref * detail.color.a);
}