struct ViewerGlobal {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    sun_direction: vec4<f32>,
    sun_color: vec4<f32>,
    sky_color: vec4<f32>,
}

struct ViewerLocal {
    model: mat4x4<f32>,
    params: vec4<f32>,
    sliders: vec4<f32>,
}

struct BlockFaceVisual {
    color_bias: vec4<f32>,
    detail_mask: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

struct BlockDetail {
    color: vec4<f32>,
    params: vec4<f32>,
    kind_data: vec4<u32>,
}

struct BlockVisual {
    base_color: vec4<f32>,
    emission: vec4<f32>,
    surface: vec4<f32>,
    shape: vec4<f32>,
    variation_a: vec4<f32>,
    variation_b: vec4<f32>,
    response: vec4<f32>,
    palette: vec4<u32>,
    procedural: vec4<u32>,
    faces: array<BlockFaceVisual, 6>,
    details: array<BlockDetail, 8>,
}

@group(0) @binding(0) var<uniform> g: ViewerGlobal;
@group(0) @binding(1) var t_dummy_shadow: texture_depth_2d;
@group(0) @binding(2) var s_dummy_shadow: sampler_comparison;
@group(0) @binding(3) var<storage, read> block_visuals: array<BlockVisual>;
@group(0) @binding(4) var<storage, read> block_visual_palette: array<vec4<f32>>;
@group(1) @binding(0) var<uniform> local: ViewerLocal;

struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) texture_id: i32,
    @location(5) block_id: i32,
    @location(6) block_visual_id: u32,
    @location(7) face_id: u32,
    @location(8) voxel_pos: vec3<i32>,
    @location(9) variation_seed: u32,
    @location(10) ao: f32,
}

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) block_id: i32,
    @location(4) @interpolate(flat) block_visual_id: u32,
    @location(5) @interpolate(flat) face_id: u32,
    @location(6) @interpolate(flat) voxel_pos: vec3<i32>,
    @location(7) @interpolate(flat) variation_seed: u32,
    @location(8) ao: f32,
}

struct GridOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec4<f32>,
}

fn saturate(x: f32) -> f32 {
    return clamp(x, 0.0, 1.0);
}

fn safe_normalize(v: vec3<f32>) -> vec3<f32> {
    return v / max(length(v), 1e-6);
}

fn hash13(p: vec3<f32>) -> f32 {
    let q = fract(p * vec3<f32>(0.1031, 0.11369, 0.13787));
    let r = q + dot(q, q.yzx + 19.19);
    return fract((r.x + r.y) * r.z);
}

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
    return 1.0 - smoothstep(0.0, 0.14, edge_dist);
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

fn procedural_block_albedo(
    world_pos: vec3<f32>,
    world_normal: vec3<f32>,
    uv: vec2<f32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    voxel_pos: vec3<i32>,
    variation_seed: u32,
    variation_scale: f32,
    edge_mult: f32,
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
        color = mix(color, palette_color(visual, cell_hash), 0.82);
    }

    let bias_delta = length(face_visual.color_bias.rgb - vec3<f32>(1.0));
    if (bias_delta > 0.01) {
        color = mix(color, face_visual.color_bias.rgb, clamp(0.55 + bias_delta * 0.18, 0.0, 0.92));
    }

    color = color * (1.0 + (cell_hash - 0.5) * 2.0 * visual.variation_a.x * variation_scale);
    color = color * (1.0 + (face_hash - 0.5) * 2.0 * visual.variation_a.y * variation_scale);
    color = color * (1.0 + (macro_hash - 0.5) * 2.0 * visual.variation_a.w * variation_scale);
    color = color * (1.0 + (micro_hash - 0.5) * 2.0 * visual.variation_b.y * variation_scale);

    let up_dot = clamp(dot(safe_normalize(world_normal), vec3<f32>(0.0, 1.0, 0.0)), -1.0, 1.0);
    let topness = saturate(up_dot);
    let bottomness = saturate(-up_dot);
    let sideness = saturate(1.0 - max(topness, bottomness));

    if (visual.response.x > 0.0 && face_id == 0u) {
        color = mix(color, palette_color(visual, 0.999), saturate(visual.response.x));
    }

    if (visual.procedural.y != 0u && face_id >= 2u) {
        let top_face = visual.faces[0];
        let fringe_color = default_or_face_color(top_face, palette_color(visual, 0.999));
        let fringe_noise = hash13(seed + vec3<f32>(41.0, 43.0, 47.0));
        let fringe = (1.0 - smoothstep(0.14, 0.40, uv.y)) * smoothstep(0.22, 0.78, fringe_noise);
        color = mix(color, fringe_color, fringe * 0.92);
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

    color = color * (1.0 - edge_factor(uv) * visual.variation_b.z * edge_mult);
    return clamp(color, vec3<f32>(0.0), vec3<f32>(3.5));
}

fn face_id_color(face_id: u32) -> vec3<f32> {
    switch face_id {
        case 0u: { return vec3<f32>(0.9, 0.2, 0.2); }
        case 1u: { return vec3<f32>(0.2, 0.4, 0.9); }
        case 2u: { return vec3<f32>(0.2, 0.9, 0.2); }
        case 3u: { return vec3<f32>(0.9, 0.9, 0.2); }
        case 4u: { return vec3<f32>(0.9, 0.5, 0.2); }
        case 5u: { return vec3<f32>(0.8, 0.2, 0.9); }
        default: { return vec3<f32>(1.0); }
    }
}

fn aces(v: vec3<f32>) -> vec3<f32> {
    return clamp((v * (2.51 * v + 0.03)) / (v * (2.43 * v + 0.59) + 0.14), vec3<f32>(0.0), vec3<f32>(1.0));
}

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    let world_pos = local.model * vec4<f32>(in.pos, 1.0);
    out.world_pos = world_pos.xyz;
    out.clip_pos = g.view_proj * world_pos;
    let nm = mat3x3<f32>(local.model[0].xyz, local.model[1].xyz, local.model[2].xyz);
    out.world_normal = safe_normalize(nm * in.normal);
    out.uv = in.uv;
    out.block_id = in.block_id;
    out.block_visual_id = in.block_visual_id;
    out.face_id = in.face_id;
    out.voxel_pos = in.voxel_pos;
    out.variation_seed = in.variation_seed;
    out.ao = in.ao;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let debug_mode = i32(local.params.x);
    let variation_scale = max(local.params.y, 0.0);
    let edge_mult = max(local.params.z, 0.0);
    let exposure = max(local.params.w, 0.1);

    if (debug_mode == 5) {
        return vec4<f32>(face_id_color(in.face_id), 1.0);
    }
    if (debug_mode == 6) {
        return vec4<f32>(in.uv.x, in.uv.y, 0.0, 1.0);
    }
    if (debug_mode == 4) {
        return vec4<f32>(vec3<f32>(in.ao), 1.0);
    }

    let visual = visual_for(in.block_visual_id);
    if (debug_mode == 2 && visual.palette.y > 0u) {
        let index = min(u32(floor(in.uv.x * f32(visual.palette.y))), max(visual.palette.y, 1u) - 1u);
        return vec4<f32>(block_visual_palette[visual.palette.x + index].rgb, 1.0);
    }

    let N = safe_normalize(in.world_normal);
    let L = safe_normalize(g.sun_direction.xyz);
    let V = safe_normalize(g.camera_pos.xyz - in.world_pos);

    var albedo = procedural_block_albedo(
        in.world_pos,
        N,
        in.uv,
        in.block_id,
        in.block_visual_id,
        in.face_id,
        in.voxel_pos,
        in.variation_seed,
        select(variation_scale, 0.0, debug_mode == 8),
        edge_mult,
    );

    if (debug_mode == 1) {
        return vec4<f32>(albedo, 1.0);
    }

    let ao_mult = max(local.sliders.x, 0.0);
    let ao_direct = mix(1.0, in.ao, visual.variation_b.w * ao_mult);
    let ao_ambient = ao_direct * ao_direct;
    let n_dot_l = saturate(dot(N, L));
    let hemi = dot(N, vec3<f32>(0.0, 1.0, 0.0)) * 0.5 + 0.5;
    let ambient = mix(vec3<f32>(0.16, 0.14, 0.12), g.sky_color.rgb, hemi) * 0.78 * ao_ambient;
    let diffuse = g.sun_color.rgb * mix(n_dot_l, saturate((n_dot_l + 0.25) / 1.25), 0.24) * ao_direct;
    let spec = g.sun_color.rgb * pow(saturate(dot(reflect(-L, N), V)), mix(8.0, 48.0, pow(1.0 - visual.surface.x, 2.0))) * 0.06;
    let rim = g.sky_color.rgb * pow(1.0 - saturate(dot(N, V)), 3.0) * 0.06;

    var lit = albedo * (ambient + diffuse + rim) + spec + visual.emission.rgb;
    lit = aces(lit * exposure);
    lit = pow(max(lit, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
    return vec4<f32>(lit, visual.surface.z);
}

@vertex
fn vs_line(in: VertexIn) -> GridOut {
    var out: GridOut;
    out.clip_pos = g.view_proj * (local.model * vec4<f32>(in.pos, 1.0));
    out.color = vec4<f32>(in.color, 1.0);
    return out;
}

@fragment
fn fs_line(in: GridOut) -> @location(0) vec4<f32> {
    return in.color;
}
