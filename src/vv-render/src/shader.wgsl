struct Atmosphere {
    sun_direction: vec4<f32>,
    sun_color: vec4<f32>,
    sky_color: vec4<f32>,
    ground_ambient_color: vec4<f32>,
    shadow_tint_color: vec4<f32>,
    fog_color_density: vec4<f32>,
    clear_color: vec4<f32>,
}

struct Global {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    atmosphere: Atmosphere,
    inv_view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> global: Global;
@group(0) @binding(1) var t_shadow: texture_depth_2d;
@group(0) @binding(2) var s_shadow: sampler_comparison;
@group(0) @binding(3) var t_block_atlas: texture_2d<f32>;
@group(0) @binding(4) var s_block_atlas: sampler;
@group(0) @binding(5) var<storage, read> block_atlas_rects: array<vec4<f32>>;

struct RuntimeVisualVariation {
    per_voxel_tint: f32,
    per_face_tint: f32,
    macro_noise_scale: f32,
    macro_noise_strength: f32,
    micro_noise_scale: f32,
    micro_noise_strength: f32,
    edge_darkening: f32,
    ao_influence: f32,
    biome_tint_strength: f32,
    wetness_response: f32,
    snow_response: f32,
    dust_response: f32,
}

struct BlockVisual {
    base_color_alpha: vec4<f32>,
    emission_roughness_metallic: vec4<f32>,
    variation_a: vec4<f32>,
    variation_b: vec4<f32>,
    response: vec4<f32>,
    palette: vec4<f32>,
    params: vec4<f32>,
    flags: vec4<f32>,
}
@group(0) @binding(6) var<storage, read> block_visuals: array<BlockVisual>;
@group(0) @binding(7) var<storage, read> block_visual_palette: array<vec4<f32>>;

struct Local {
    model: mat4x4<f32>,
    params: vec4<f32>,
}
@group(1) @binding(0) var<uniform> local: Local;

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
};

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) view_pos: vec3<f32>,
    @location(4) shadow_pos: vec3<f32>,
    @location(5) uv: vec2<f32>,
    @location(6) @interpolate(flat) texture_id: i32,
    @location(7) @interpolate(flat) block_id: i32,
    @location(8) @interpolate(flat) block_visual_id: u32,
    @location(9) @interpolate(flat) face_id: u32,
    @location(10) @interpolate(flat) voxel_pos: vec3<i32>,
    @location(11) @interpolate(flat) variation_seed: u32,
    @location(12) ao: f32,
};

struct SkyOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

fn saturate(x: f32) -> f32 {
    return clamp(x, 0.0, 1.0);
}

fn saturate3(v: vec3<f32>) -> vec3<f32> {
    return clamp(v, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn safe_normalize(v: vec3<f32>) -> vec3<f32> {
    let l = max(length(v), 1e-6);
    return v / l;
}

fn linstep(a: f32, b: f32, x: f32) -> f32 {
    return saturate((x - a) / (b - a));
}

fn smooth_remap(a: f32, b: f32, x: f32) -> f32 {
    let t = linstep(a, b, x);
    return t * t * (3.0 - 2.0 * t);
}

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn dither_opacity(pos: vec4<f32>, alpha: f32) -> bool {
    let dither_threshold = dot(vec2<f32>(171.0, 231.0), pos.xy);
    return fract(dither_threshold / 71.0) > alpha;
}

fn hash13(p: vec3<f32>) -> f32 {
    let q = fract(p * vec3<f32>(0.1031, 0.11369, 0.13787));
    let r = q + dot(q, q.yzx + 19.19);
    return fract((r.x + r.y) * r.z);
}

fn value_noise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let n000 = hash13(i + vec3<f32>(0.0, 0.0, 0.0));
    let n100 = hash13(i + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = hash13(i + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = hash13(i + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = hash13(i + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = hash13(i + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = hash13(i + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = hash13(i + vec3<f32>(1.0, 1.0, 1.0));

    let x00 = mix(n000, n100, u.x);
    let x10 = mix(n010, n110, u.x);
    let x01 = mix(n001, n101, u.x);
    let x11 = mix(n011, n111, u.x);
    let y0 = mix(x00, x10, u.y);
    let y1 = mix(x01, x11, u.y);
    return mix(y0, y1, u.z);
}

fn fbm3(p: vec3<f32>) -> f32 {
    let a = value_noise(p);
    let b = value_noise(p * 2.01 + vec3<f32>(17.1, 3.7, 11.3)) * 0.5;
    let c = value_noise(p * 4.03 + vec3<f32>(7.9, 19.4, 5.2)) * 0.25;
    return (a + b + c) / 1.75;
}

fn triplanar_detail(pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    let p = pos * 2.2;
    let n = abs(normal);
    let w = pow(n, vec3<f32>(16.0));
    let weights = w / max(dot(w, vec3<f32>(1.0)), 1e-5);

    let hx = fract(sin(dot(p.yz, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let hy = fract(sin(dot(p.zx, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let hz = fract(sin(dot(p.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);

    return (hx * weights.x + hy * weights.y + hz * weights.z) * 2.0 - 1.0;
}

fn visual_for(block_visual_id: u32) -> BlockVisual {
    return block_visuals[block_visual_id];
}

fn variation_for(visual: BlockVisual) -> RuntimeVisualVariation {
    return RuntimeVisualVariation(
        visual.variation_a.x,
        visual.variation_a.y,
        visual.variation_a.z,
        visual.variation_a.w,
        visual.variation_b.x,
        visual.variation_b.y,
        visual.variation_b.z,
        visual.variation_b.w,
        visual.response.x,
        visual.response.y,
        visual.response.z,
        visual.response.w,
    );
}

fn palette_color(visual: BlockVisual, selector: f32) -> vec3<f32> {
    let len = max(u32(visual.palette.y), 1u);
    let offset = u32(visual.palette.x);
    let index = min(u32(floor(selector * f32(len))), len - 1u);
    return block_visual_palette[offset + index].rgb;
}

fn hash11(x: f32) -> f32 {
    return fract(sin(x * 17.123) * 43758.5453123);
}

fn value_noise_3d(p: vec3<f32>) -> f32 {
    return value_noise(p);
}

fn fbm_3d(p: vec3<f32>) -> f32 {
    return fbm3(p);
}

fn apply_face_tint(color: vec3<f32>, face_id: u32, seed: u32, amount: f32) -> vec3<f32> {
    let f = hash11(f32(seed ^ (face_id * 747796405u)));
    let tint = mix(1.0 - amount, 1.0 + amount, f);
    return color * tint;
}

fn apply_edge_wear(color: vec3<f32>, uv: vec2<f32>, amount: f32) -> vec3<f32> {
    let edge_dist = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
    // Wider edge band for more visible stylized block edges
    let edge = 1.0 - smoothstep(0.0, 0.14, edge_dist);
    // Corner peaks: sharpen the four block corners with extra darkening
    let corner_dist = length(vec2<f32>(uv.x - 0.5, uv.y - 0.5));
    let corner = saturate(1.0 - smoothstep(0.35, 0.50, corner_dist));
    return color * (1.0 - (edge + corner * 0.38) * amount);
}

fn apply_voxel_variation(
    base_color: vec3<f32>,
    world_pos: vec3<f32>,
    block_visual_id: u32,
    face_id: u32,
    seed: u32,
    variation: RuntimeVisualVariation,
) -> vec3<f32> {
    let seed_vec = vec3<f32>(f32(seed & 65535u), f32(face_id), f32(block_visual_id));
    let r = hash13(floor(world_pos * 2.0) + seed_vec * 0.017);
    let tint = mix(1.0 - variation.per_voxel_tint, 1.0 + variation.per_voxel_tint, r);
    // 0.10 instead of 0.04 — macro noise at block-level scale (10–15 blocks per cycle)
    let macro_n = fbm_3d(world_pos * variation.macro_noise_scale * 0.10 + seed_vec * 0.003);
    let micro_n = fbm_3d(world_pos * variation.micro_noise_scale + seed_vec * 0.011);
    var color = base_color * tint;
    // 1.6x and 2.0x amplifiers make variation much more visible
    color = color * (1.0 + ((macro_n - 0.5) * variation.macro_noise_strength * 1.6));
    color = color * (1.0 + ((micro_n - 0.5) * variation.micro_noise_strength * 2.0));
    return color;
}

struct MaterialProfile {
    shadow: vec3<f32>,
    body: vec3<f32>,
    highlight: vec3<f32>,
    accent: vec3<f32>,
    response: vec4<f32>,
    detail: vec4<f32>,
}

fn material_profile(kind: f32, base: vec3<f32>, secondary: vec3<f32>) -> MaterialProfile {
    if (kind == 1.0) {
        return MaterialProfile(
            vec3<f32>(0.10, 0.28, 0.10),
            mix(base, vec3<f32>(0.30, 0.58, 0.16), 0.62),
            mix(secondary, vec3<f32>(0.62, 0.84, 0.25), 0.46),
            vec3<f32>(0.78, 0.86, 0.30),
            vec4<f32>(0.78, 0.54, 0.92, 0.18),
            vec4<f32>(0.38, 0.14, 1.35, 0.18),
        );
    }
    if (kind == 2.0) {
        return MaterialProfile(
            vec3<f32>(0.16, 0.085, 0.045),
            mix(base, vec3<f32>(0.36, 0.19, 0.09), 0.60),
            mix(secondary, vec3<f32>(0.58, 0.34, 0.16), 0.40),
            vec3<f32>(0.70, 0.43, 0.20),
            vec4<f32>(0.62, 0.38, 0.70, 0.10),
            vec4<f32>(0.30, 0.10, 1.00, 0.10),
        );
    }
    if (kind == 3.0) {
        return MaterialProfile(
            vec3<f32>(0.70, 0.78, 0.86),
            mix(base, vec3<f32>(0.86, 0.91, 0.96), 0.56),
            vec3<f32>(1.0, 0.995, 0.94),
            vec3<f32>(0.82, 0.90, 1.0),
            vec4<f32>(0.42, 0.22, 0.48, 0.24),
            vec4<f32>(0.16, 0.05, 0.68, 0.22),
        );
    }
    if (kind == 4.0 || kind == 11.0) {
        return MaterialProfile(
            vec3<f32>(0.22, 0.23, 0.23),
            mix(base, vec3<f32>(0.42, 0.42, 0.39), 0.58),
            mix(secondary, vec3<f32>(0.62, 0.60, 0.54), 0.36),
            vec3<f32>(0.48, 0.52, 0.56),
            vec4<f32>(0.50, 0.34, 0.62, 0.08),
            vec4<f32>(0.24, 0.16, 0.78, 0.12),
        );
    }
    if (kind == 5.0) {
        return MaterialProfile(
            vec3<f32>(0.50, 0.39, 0.19),
            mix(base, vec3<f32>(0.72, 0.57, 0.30), 0.58),
            mix(secondary, vec3<f32>(0.96, 0.82, 0.48), 0.42),
            vec3<f32>(1.0, 0.88, 0.58),
            vec4<f32>(0.48, 0.28, 0.74, 0.16),
            vec4<f32>(0.20, 0.22, 0.86, 0.20),
        );
    }
    if (kind == 6.0) {
        return MaterialProfile(
            vec3<f32>(0.20, 0.085, 0.035),
            mix(base, vec3<f32>(0.52, 0.24, 0.085), 0.52),
            mix(secondary, vec3<f32>(0.88, 0.50, 0.20), 0.34),
            vec3<f32>(0.96, 0.62, 0.24),
            vec4<f32>(0.64, 0.30, 0.78, 0.08),
            vec4<f32>(0.22, 0.42, 1.12, 0.16),
        );
    }
    if (kind == 7.0) {
        return MaterialProfile(
            vec3<f32>(0.035, 0.15, 0.055),
            mix(base, vec3<f32>(0.12, 0.36, 0.11), 0.60),
            mix(secondary, vec3<f32>(0.50, 0.76, 0.20), 0.44),
            vec3<f32>(0.66, 0.86, 0.26),
            vec4<f32>(0.86, 0.60, 1.00, 0.20),
            vec4<f32>(0.42, 0.12, 1.45, 0.24),
        );
    }
    if (kind == 8.0) {
        return MaterialProfile(
            vec3<f32>(0.20, 0.48, 0.66),
            mix(base, vec3<f32>(0.38, 0.70, 0.86), 0.50),
            vec3<f32>(0.78, 0.95, 1.0),
            vec3<f32>(0.92, 1.0, 1.0),
            vec4<f32>(0.36, 0.18, 0.50, 0.28),
            vec4<f32>(0.14, 0.08, 0.72, 0.24),
        );
    }
    if (kind == 9.0 || kind == 10.0) {
        return MaterialProfile(
            base * 0.72,
            base,
            secondary,
            secondary * 1.05,
            vec4<f32>(0.22, 0.14, 0.20, 0.06),
            vec4<f32>(0.10, 0.06, 0.42, 0.06),
        );
    }
    if (kind == 12.0) {
        return MaterialProfile(
            base * 0.72,
            mix(base, secondary, 0.35),
            secondary,
            vec3<f32>(0.78, 0.96, 1.0),
            vec4<f32>(0.20, 0.12, 0.32, 0.30),
            vec4<f32>(0.10, 0.10, 0.55, 0.26),
        );
    }
    return MaterialProfile(
        base * 0.76,
        base,
        secondary,
        secondary,
        vec4<f32>(0.28, 0.18, 0.36, 0.08),
        vec4<f32>(0.16, 0.08, 0.56, 0.08),
    );
}

fn rotated_variant_uv(uv: vec2<f32>, face_hash: f32, material_kind: f32) -> vec2<f32> {
    if (material_kind == 9.0 || material_kind == 10.0 || material_kind == 12.0) {
        return uv;
    }

    let tile = floor(face_hash * 8.0);
    var local = fract(uv);
    if (tile >= 4.0) {
        local.x = 1.0 - local.x;
    }
    let rot = tile - floor(tile * 0.25) * 4.0;

    if (rot < 1.0) {
        return local;
    }
    if (rot < 2.0) {
        return vec2<f32>(1.0 - local.y, local.x);
    }
    if (rot < 3.0) {
        return vec2<f32>(1.0 - local.x, 1.0 - local.y);
    }
    return vec2<f32>(local.y, 1.0 - local.x);
}

fn soft_contrast(x: f32, amount: f32) -> f32 {
    return mix(x, x * x * (3.0 - 2.0 * x), amount);
}

fn soft_ridge(x: f32) -> f32 {
    return 1.0 - abs(x * 2.0 - 1.0);
}

fn oriented_grain(kind: f32, pos: vec3<f32>, normal: vec3<f32>, face_hash: f32) -> f32 {
    let n = abs(normal);
    var axis = pos.y + pos.z * 0.28;
    if (n.y > n.x && n.y > n.z) {
        axis = pos.x + pos.z * 0.22;
    } else if (n.z > n.x) {
        axis = pos.y + pos.x * 0.24;
    }

    if (kind == 6.0 || kind == 10.0) {
        let rings = sin(axis * 13.5 + fbm3(pos * 0.85 + vec3<f32>(face_hash * 7.0)) * 4.0);
        let pores = fbm3(pos * vec3<f32>(1.2, 8.0, 1.2) + vec3<f32>(face_hash, 2.0, 5.0));
        return (rings * 0.5 + 0.5) * 0.58 + pores * 0.42;
    }
    if (kind == 5.0 || kind == 3.0) {
        let drift = fbm3(pos * 0.55 + vec3<f32>(11.0, face_hash, 3.0));
        return fbm3(pos * 3.0 + vec3<f32>(drift * 2.0, face_hash, 0.0));
    }
    if (kind == 4.0 || kind == 11.0 || kind == 8.0) {
        let veins = fbm3(pos * vec3<f32>(0.7, 1.1, 0.9) + normal * 3.0);
        return mix(veins, soft_ridge(fbm3(pos * 2.2 + vec3<f32>(face_hash * 3.0))), 0.35);
    }
    if (kind == 1.0 || kind == 7.0) {
        let blade = fbm3(pos * vec3<f32>(2.2, 5.4, 2.2) + vec3<f32>(face_hash * 4.0));
        return mix(blade, soft_ridge(fbm3(pos * 1.3 + normal * 2.0)), 0.18);
    }
    return fbm3(pos * 1.8 + normal * 4.0 + vec3<f32>(face_hash));
}

fn material_color_shift(kind: f32, n: f32) -> vec3<f32> {
    if (kind == 1.0) {
        return vec3<f32>(-0.06 + n * 0.11, -0.02 + n * 0.13, -0.07 + n * 0.055);
    }
    if (kind == 2.0) {
        return vec3<f32>(-0.04 + n * 0.08, -0.025 + n * 0.055, -0.03 + n * 0.04);
    }
    if (kind == 3.0) {
        let v = -0.025 + n * 0.055;
        return vec3<f32>(v * 0.82, v * 0.9, v);
    }
    if (kind == 4.0) {
        let v = -0.055 + n * 0.105;
        return vec3<f32>(v, v, v * 0.94);
    }
    if (kind == 5.0) {
        return vec3<f32>(-0.02 + n * 0.08, -0.015 + n * 0.065, -0.035 + n * 0.04);
    }
    if (kind == 6.0 || kind == 10.0) {
        return vec3<f32>(-0.035 + n * 0.07, -0.02 + n * 0.045, -0.03 + n * 0.035);
    }
    if (kind == 7.0) {
        return vec3<f32>(-0.045 + n * 0.07, -0.025 + n * 0.11, -0.05 + n * 0.04);
    }
    if (kind == 8.0) {
        return vec3<f32>(-0.025 + n * 0.055, -0.01 + n * 0.04, -0.005 + n * 0.07);
    }
    if (kind == 9.0 || kind == 11.0) {
        let v = -0.04 + n * 0.08;
        return vec3<f32>(v, v, v * 0.96);
    }
    return vec3<f32>(-0.03 + n * 0.06);
}

fn material_texture_strength(kind: f32, authored: f32) -> f32 {
    if (kind == 1.0 || kind == 2.0 || kind == 3.0 || kind == 5.0 || kind == 7.0) {
        return authored * 0.14;
    }
    if (kind == 4.0 || kind == 6.0 || kind == 8.0 || kind == 11.0) {
        return authored * 0.2;
    }
    if (kind == 9.0 || kind == 10.0) {
        return authored * 0.28;
    }
    return authored * 0.18;
}

fn material_face_grade(kind: f32, topness: f32, sideness: f32, bottomness: f32) -> vec3<f32> {
    if (kind == 1.0) {
        return vec3<f32>(
            1.0 + topness * 0.08 - bottomness * 0.16,
            1.0 + topness * 0.12 - bottomness * 0.10,
            1.0 - topness * 0.04 - bottomness * 0.08
        );
    }
    if (kind == 2.0) {
        return vec3<f32>(
            1.0 + topness * 0.03 - bottomness * 0.09,
            1.0 + topness * 0.02 - bottomness * 0.09,
            1.0 - bottomness * 0.08
        );
    }
    if (kind == 3.0 || kind == 8.0) {
        return vec3<f32>(
            1.0 + topness * 0.035 - sideness * 0.03,
            1.0 + topness * 0.045 - sideness * 0.02,
            1.0 + topness * 0.06
        );
    }
    if (kind == 4.0 || kind == 9.0 || kind == 11.0) {
        return vec3<f32>(1.0 + topness * 0.03 - bottomness * 0.11);
    }
    if (kind == 5.0) {
        return vec3<f32>(1.0 + topness * 0.05, 1.0 + topness * 0.04, 1.0 - sideness * 0.04);
    }
    if (kind == 6.0 || kind == 10.0) {
        return vec3<f32>(1.0 + topness * 0.025, 1.0 + topness * 0.015, 1.0 - bottomness * 0.05);
    }
    if (kind == 7.0) {
        return vec3<f32>(1.0 - bottomness * 0.12, 1.0 + topness * 0.05, 1.0 - bottomness * 0.12);
    }
    return vec3<f32>(1.0);
}

fn material_palette(
    kind: f32,
    base: vec3<f32>,
    secondary: vec3<f32>,
    topness: f32,
    sideness: f32,
    macro_a: f32,
    macro_b: f32
) -> vec3<f32> {
    let profile = material_profile(kind, base, secondary);
    let macro_t = soft_contrast(macro_a * 0.76 + macro_b * 0.24, profile.response.z);
    let light_t = saturate(topness * (0.28 + profile.response.w) + macro_b * 0.18);
    let body = mix(profile.shadow, profile.body, macro_t);
    var color = mix(body, profile.highlight, light_t);
    color = mix(color, profile.accent, topness * profile.response.w * 0.42);
    return color * material_face_grade(kind, topness, sideness, 1.0 - max(topness, sideness));
}

fn aces_approx(v: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((v * (a * v + b)) / (v * (c * v + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn atlas_albedo(texture_id: i32, uv: vec2<f32>, face_hash: f32, kind: f32) -> vec3<f32> {
    if (texture_id < 0) {
        return vec3<f32>(1.0);
    }

    let rect = block_atlas_rects[u32(texture_id)];
    let varied_uv = rotated_variant_uv(uv, face_hash, kind);
    let pad = vec2<f32>(0.0012);
    let atlas_uv = mix(rect.xy + pad, rect.zw - pad, varied_uv);
    return textureSample(t_block_atlas, s_block_atlas, atlas_uv).rgb;
}

fn block_albedo(in: VertexOut) -> vec3<f32> {
    let visual = visual_for(in.block_visual_id);
    let variation = variation_for(visual);
    let kind = visual.palette.z;
    let N = safe_normalize(in.world_normal);
    let radial_up = safe_normalize(in.world_pos);
    let up_dot = clamp(dot(N, radial_up), -1.0, 1.0);

    let topness = smooth_remap(0.18, 0.92, up_dot);
    let bottomness = smooth_remap(0.15, 0.82, -up_dot);
    let sideness = clamp(1.0 - max(topness, bottomness), 0.0, 1.0);

    let block_cell = floor(in.world_pos * 2.0);
    let face_key = block_cell + floor(abs(N) * 17.0);

    let block_hash = hash13(block_cell + vec3<f32>(f32(in.variation_seed & 65535u), 3.1, 9.7));
    let face_hash = hash13(face_key + vec3<f32>(f32(in.face_id), 5.3, f32(in.block_visual_id)));
    let macro_warp = fbm_3d(in.world_pos * 0.008 + radial_up * 1.9 + vec3<f32>(f32(in.block_visual_id) * 0.13, 4.0, 9.0));

    // === Biome tint: top-face palette shift ===
    // biome_tint_strength > 0 on blocks like grass and leaves.
    // The last palette entry holds the "top face" color (e.g. #78A83A grass green).
    // Shift the selector toward 0.95 (last entry) based on topness × biome_tint_strength.
    let top_bias = topness * saturate(variation.biome_tint_strength * 3.8);
    let color_selector = mix(block_hash, 0.95, top_bias);

    var identity_color = mix(
        visual.base_color_alpha.rgb,
        palette_color(visual, color_selector),
        smoothstep(0.15, 0.95, face_hash)
    );
    let secondary_color = palette_color(visual, 1.0 - block_hash * 0.65);

    // === Material palette → LIGHTING only (no hue override) ===
    // material_palette() generates material-type colored output with hardcoded hues.
    // Extract its luminance ratio to apply material-type shading (shadow/highlight)
    // without replacing the block's actual color.
    let mat_seed = vec3<f32>(f32(in.block_visual_id) * 0.11, 3.1, 5.7);
    let macro_a = fbm_3d(in.world_pos * variation.macro_noise_scale * 0.10 + mat_seed);
    let macro_b = fbm_3d(in.world_pos * variation.macro_noise_scale * 0.18 + vec3<f32>(7.3, face_hash * 1.9, 2.4));
    let mat_color = material_palette(kind, identity_color, secondary_color, topness, sideness, macro_a, macro_b);
    let mat_luma = max(luminance(mat_color), 0.001);
    let id_luma = max(luminance(identity_color), 0.001);
    // Clamp the ratio to avoid extreme blowouts in very bright or very dark areas
    let lighting = clamp(mat_luma / id_luma, 0.35, 2.2);
    let mat_strength = saturate(variation.macro_noise_strength * 1.6);
    identity_color = identity_color * mix(1.0, lighting, mat_strength * 0.30);

    // === Oriented grain: surface micro-character (stone veins, wood rings, leaf blades) ===
    let grain = oriented_grain(kind, in.world_pos, N, face_hash);
    let grain_shift = material_color_shift(kind, grain);
    let grain_strength = variation.micro_noise_strength * 1.8;
    identity_color = clamp(identity_color + grain_shift * grain_strength, vec3<f32>(0.0), vec3<f32>(2.5));

    // === Texture atlas blend ===
    if (in.texture_id >= 0) {
        let tex_color = atlas_albedo(in.texture_id, in.uv, face_hash, kind);
        let tex_luma = luminance(tex_color);
        let tex_strength = material_texture_strength(kind, variation.micro_noise_strength * 5.0);
        let luma_detail = (tex_luma - 0.5) * tex_strength;
        let chroma_detail = (tex_color - vec3<f32>(tex_luma)) * (tex_strength * 0.12);
        identity_color = identity_color * (1.0 + luma_detail) + chroma_detail;
    }

    // === Stylized face gradient (balanced — not crushing bottom faces) ===
    // Top faces are slightly brighter and cooler; sides slightly warmer
    let broad_gradient = mix(vec3<f32>(0.91, 0.92, 0.94), vec3<f32>(1.08, 1.05, 1.00), topness);
    let side_warmth = mix(vec3<f32>(1.0), vec3<f32>(1.03, 0.98, 0.94), sideness * (1.0 - bottomness));

    var shaped = identity_color * broad_gradient * side_warmth;

    shaped = apply_voxel_variation(
        shaped,
        in.world_pos + vec3<f32>(macro_warp * 0.22),
        in.block_visual_id,
        in.face_id,
        in.variation_seed,
        variation
    );
    shaped = apply_face_tint(shaped, in.face_id, in.variation_seed, variation.per_face_tint);
    shaped = apply_edge_wear(shaped, in.uv, variation.edge_darkening);

    return clamp(shaped, vec3<f32>(0.0), vec3<f32>(3.5));
}

fn shadow_visibility(shadow_pos: vec3<f32>, n_dot_l: f32) -> f32 {
    if (
        shadow_pos.z > 1.0 ||
        shadow_pos.x < 0.0 || shadow_pos.x > 1.0 ||
        shadow_pos.y < 0.0 || shadow_pos.y > 1.0
    ) {
        return 1.0;
    }

    let dim = vec2<f32>(textureDimensions(t_shadow));
    let texel = 1.0 / dim;
    let bias = max(0.00012, 0.00065 * (1.0 - n_dot_l));
    let depth = shadow_pos.z - bias;
    let radius = mix(1.0, 2.2, saturate(1.0 - n_dot_l));

    var sum = 0.0;
    var weight_sum = 0.0;

    for (var ix: i32 = -1; ix <= 1; ix = ix + 1) {
        for (var iy: i32 = -1; iy <= 1; iy = iy + 1) {
            let o = vec2<f32>(f32(ix), f32(iy));
            let dist2 = dot(o, o);
            let w = exp(-dist2 * 0.9);
            let sample_uv = shadow_pos.xy + o * texel * radius;
            let v = textureSampleCompare(t_shadow, s_shadow, sample_uv, depth);
            sum = sum + v * w;
            weight_sum = weight_sum + w;
        }
    }

    return sum / max(weight_sum, 1e-5);
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - cos_theta, 5.0);
}

fn ggx_distribution(n_dot_h: f32, alpha: f32) -> f32 {
    let a2 = alpha * alpha;
    let d = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / max(3.14159265 * d * d, 1e-5);
}

fn smith_visibility(n_dot_v: f32, n_dot_l: f32, alpha: f32) -> f32 {
    let k = (alpha + 1.0);
    let kk = (k * k) * 0.125;
    let gv = n_dot_v / mix(n_dot_v, 1.0, kk);
    let gl = n_dot_l / mix(n_dot_l, 1.0, kk);
    return gv * gl;
}

fn material_subsurface(kind: f32) -> f32 {
    if (kind == 7.0) {
        return 0.22;
    }
    if (kind == 8.0) {
        return 0.08;
    }
    return 0.0;
}

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    let world_pos = local.model * vec4<f32>(in.pos, 1.0);
    out.world_pos = world_pos.xyz;
    out.clip_pos = global.view_proj * world_pos;

    let normal_mat = mat3x3<f32>(
        local.model[0].xyz,
        local.model[1].xyz,
        local.model[2].xyz
    );
    out.world_normal = safe_normalize(normal_mat * in.normal);
    out.color = in.color;
    out.uv = in.uv;
    out.texture_id = in.texture_id;
    out.block_id = in.block_id;
    out.block_visual_id = in.block_visual_id;
    out.face_id = in.face_id;
    out.voxel_pos = in.voxel_pos;
    out.variation_seed = in.variation_seed;
    out.ao = in.ao;
    out.view_pos = global.camera_pos.xyz;

    let normal_offset = out.world_normal * 0.05;
    let pos_light = global.light_view_proj * vec4<f32>(out.world_pos + normal_offset, 1.0);

    out.shadow_pos = vec3<f32>(
        pos_light.x * 0.5 + 0.5,
        -pos_light.y * 0.5 + 0.5,
        pos_light.z
    );

    return out;
}

@fragment
fn fs_feedback(in: VertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, local.params.x * 0.82);
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    if (local.params.x < 1.0 && dither_opacity(in.clip_pos, local.params.x)) {
        discard;
    }

    let visual = visual_for(in.block_visual_id);
    if (visual.base_color_alpha.w < 1.0 && dither_opacity(in.clip_pos, visual.base_color_alpha.w)) {
        discard;
    }
    let variation = variation_for(visual);
    let roughness = clamp(visual.emission_roughness_metallic.w, 0.05, 1.0);

    let N = safe_normalize(in.world_normal);
    let L = safe_normalize(global.atmosphere.sun_direction.xyz);
    let V = safe_normalize(global.camera_pos.xyz - in.world_pos);
    let H = safe_normalize(L + V);
    let radial_up = safe_normalize(in.world_pos);

    let albedo = block_albedo(in) * (1.0 + variation.micro_noise_strength * 0.45 * triplanar_detail(in.world_pos, N));

    let ao_raw = mix(1.0, in.ao, variation.ao_influence);
    let ao_t = clamp((ao_raw - 0.4) / 0.6, 0.0, 1.0);
    // Stylized AO: slight blue tint in shadowed corners, quadratic falloff
    let ao_tint = mix(vec3<f32>(0.62, 0.70, 0.90), vec3<f32>(1.0), ao_t);
    let ao_ambient = ao_raw * ao_raw;
    let ao_direct = mix(ao_raw, 1.0, 0.52);

    let n_dot_l = saturate(dot(N, L));
    let n_dot_v = saturate(dot(N, V));
    let n_dot_h = saturate(dot(N, H));
    let l_dot_h = saturate(dot(L, H));

    let shadow = shadow_visibility(in.shadow_pos, n_dot_l);

    let hemi_factor = dot(N, radial_up) * 0.5 + 0.5;
    let sky_ambient = global.atmosphere.sky_color.xyz;
    let ground_ambient = global.atmosphere.ground_ambient_color.xyz;
    let hemi_ambient = mix(ground_ambient, sky_ambient, hemi_factor);

    let wrap = saturate((dot(N, L) + 0.35) / 1.35);
    let diffuse_term = mix(n_dot_l, wrap, 0.22);
    let direct_sun = global.atmosphere.sun_color.xyz * diffuse_term * shadow;
    let shadow_fill = global.atmosphere.shadow_tint_color.xyz * diffuse_term * (1.0 - shadow);

    let alpha = roughness * roughness;
    let D = ggx_distribution(n_dot_h, alpha);
    let G = smith_visibility(n_dot_v, n_dot_l, alpha);
    let F = fresnel_schlick(l_dot_h, vec3<f32>(0.028));
    let specular = (D * G) * F / max(4.0 * n_dot_l * n_dot_v, 1e-4);
    let specular_term = specular * global.atmosphere.sun_color.xyz * shadow * mix(1.2, 0.35, roughness);

    let fresnel = pow(1.0 - n_dot_v, 3.0);
    let backlit = pow(saturate(dot(N, -L)), 1.8);
    let rim = sky_ambient * (fresnel * 0.10 + fresnel * backlit * 0.10);

    // material_subsurface() returns > 0 for leaves (0.22) and water (0.08)
    let subsurface = material_subsurface(visual.palette.z);
    let transmission = global.atmosphere.sun_color.xyz * backlit * (1.0 - shadow) * subsurface * 0.55;

    let ambient_light = hemi_ambient * (0.85 + roughness * 0.16) * ao_ambient;

    var lit = albedo * ao_tint * ((direct_sun + shadow_fill) * ao_direct + ambient_light + rim) + specular_term + transmission + visual.emission_roughness_metallic.rgb;

    let dist = distance(global.camera_pos.xyz, in.world_pos);
    let fog_density = global.atmosphere.fog_color_density.w;
    let fog_base = global.atmosphere.fog_color_density.xyz;
    let view_dir = safe_normalize(in.world_pos - global.camera_pos.xyz);
    let sun_scatter = pow(saturate(dot(view_dir, L)), 8.0);
    let horizon = pow(1.0 - saturate(abs(dot(view_dir, radial_up))), 2.2);
    let fog_color = mix(fog_base, global.atmosphere.sun_color.xyz * 0.85 + fog_base * 0.35, sun_scatter * 0.18 + horizon * 0.08);
    let fog_factor = 1.0 - exp(-(dist * fog_density) * (dist * fog_density * 0.45));
    lit = mix(lit, fog_color, saturate(fog_factor));

    lit = aces_approx(lit);
    lit = pow(lit, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(lit, 1.0);
}

fn star_field(view_dir: vec3<f32>) -> vec3<f32> {
    let p1 = view_dir * 165.0;
    let c1 = floor(p1);
    let h1 = hash13(c1);
    let h2 = hash13(c1 + vec3<f32>(13.71, 8.31, 19.17));
    let h3 = hash13(c1 + vec3<f32>(7.13, 23.47, 3.91));
    let s1 = smoothstep(0.985, 1.0, h1);
    let col1 = mix(vec3<f32>(0.72, 0.88, 1.00), vec3<f32>(1.00, 0.96, 0.78), h2);
    let tw1 = 0.70 + h3 * 0.55;

    let p2 = view_dir * 320.0;
    let c2 = floor(p2);
    let k1 = hash13(c2 + vec3<f32>(29.1, 11.7, 5.2));
    let k2 = hash13(c2 + vec3<f32>(3.1, 31.4, 17.7));
    let s2 = smoothstep(0.9925, 1.0, k1);
    let col2 = mix(vec3<f32>(0.78, 0.86, 1.0), vec3<f32>(1.0, 0.93, 0.82), k2);

    return col1 * s1 * tw1 * 1.15 + col2 * s2 * 0.65;
}

fn sky_gradient(view_dir: vec3<f32>) -> vec3<f32> {
    let L = safe_normalize(global.atmosphere.sun_direction.xyz);
    let sky = global.atmosphere.sky_color.xyz;
    let fog = global.atmosphere.fog_color_density.xyz;
    let sun_col = global.atmosphere.sun_color.xyz;
    let clear_col = global.atmosphere.clear_color.xyz;
    let radial_up = safe_normalize(global.camera_pos.xyz);

    let elevation = dot(view_dir, radial_up);
    let sun_elev = dot(L, radial_up);
    let day_t = smooth_remap(-0.12, 0.10, sun_elev);
    let night_t = 1.0 - day_t;

    let zenith_day = sky * vec3<f32>(0.58, 0.73, 1.30);
    let mid_day = mix(fog * vec3<f32>(1.04, 1.02, 1.0), zenith_day, smooth_remap(-0.08, 0.34, elevation));
    let below_horizon = smooth_remap(0.0, -0.25, elevation);
    let day_ground = mix(mid_day, fog * vec3<f32>(0.72, 0.78, 0.9), below_horizon);

    let sun_dot = saturate(dot(view_dir, L));
    let sun_disk = smoothstep(0.99915, 0.99986, sun_dot) * 3.2;
    let sun_core = pow(sun_dot, 180.0) * 2.6;
    let sun_glow = pow(sun_dot, 42.0) * 1.35;
    let sun_halo = pow(sun_dot, 7.0) * 0.24;
    let sun_above = smooth_remap(-0.16, 0.07, sun_elev);
    let sun_vis = smooth_remap(-0.22, 0.03, elevation);
    let sun_contrib = sun_col * (sun_disk + sun_core + sun_glow + sun_halo) * sun_above * sun_vis;

    let haze = fog * pow(max(0.0, 1.0 - abs(elevation) * 5.2), 2.1) * 0.46;

    let abs_sun_elev = abs(sun_elev);
    let twilight_strength = smooth_remap(0.34, 0.0, abs_sun_elev);
    let sun_horiz_dir = safe_normalize(L - radial_up * sun_elev);
    let view_horiz_dir = safe_normalize(view_dir - radial_up * elevation);
    let view_near_horizon = smooth_remap(0.22, 0.0, abs(elevation));
    let az_align = saturate(dot(view_horiz_dir, sun_horiz_dir));
    let twilight = mix(
        fog * vec3<f32>(1.8, 0.78, 0.18),
        sun_col * vec3<f32>(1.22, 0.62, 0.22),
        0.5
    ) * twilight_strength * pow(az_align, 2.0) * view_near_horizon * 0.82;

    let day_sky = day_ground + sun_contrib + haze + twilight;

    let night_zenith = mix(vec3<f32>(0.006, 0.010, 0.055), clear_col * vec3<f32>(0.35, 0.45, 1.1), 0.45);
    let night_horizon = mix(vec3<f32>(0.018, 0.024, 0.075), fog * vec3<f32>(0.18, 0.22, 0.34), 0.35);
    let night_base = mix(night_horizon, night_zenith, smooth_remap(-0.12, 0.35, elevation));

    let stars = star_field(view_dir) * night_t * smooth_remap(-0.10, 0.28, elevation);

    let moon_dir = -L;
    let moon_dot = saturate(dot(view_dir, moon_dir));
    let moon_above = smooth_remap(-0.14, 0.03, dot(moon_dir, radial_up));
    let moon_disk = smoothstep(0.99955, 0.99992, moon_dot) * 0.75;
    let moon_glow = pow(moon_dot, 40.0) * 0.06;
    let moon = vec3<f32>(0.84, 0.90, 1.0) * (moon_disk + moon_glow) * moon_above * night_t;

    let night_sky = night_base + stars + moon;

    return mix(night_sky, day_sky, day_t);
}

@vertex
fn vs_sky(@builtin(vertex_index) vi: u32) -> SkyOut {
    var pos: vec2<f32>;
    switch vi {
        case 0u: {
            pos = vec2<f32>(-1.0, -1.0);
        }
        case 1u: {
            pos = vec2<f32>(3.0, -1.0);
        }
        default: {
            pos = vec2<f32>(-1.0, 3.0);
        }
    }

    var out: SkyOut;
    out.clip_pos = vec4<f32>(pos.x, pos.y, 1.0, 1.0);
    out.ndc = pos;
    return out;
}

@fragment
fn fs_sky(in: SkyOut) -> @location(0) vec4<f32> {
    let clip_far = vec4<f32>(in.ndc.x, in.ndc.y, 1.0, 1.0);
    let world_far = global.inv_view_proj * clip_far;
    let view_dir = safe_normalize(world_far.xyz / world_far.w - global.camera_pos.xyz);

    var color = sky_gradient(view_dir);
    color = aces_approx(color);
    color = pow(color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(color, 1.0);
}
