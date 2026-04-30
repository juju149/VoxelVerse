

struct Atmosphere {
    sun_direction: vec4<f32>,
    sun_color: vec4<f32>,
    sky_color: vec4<f32>,
    ground_ambient_color: vec4<f32>,
    fog_color_density: vec4<f32>,
    clear_color: vec4<f32>,
}

struct Global {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    atmosphere: Atmosphere,
}

@group(0) @binding(0) var<uniform> global: Global;
@group(0) @binding(1) var t_shadow: texture_depth_2d;
@group(0) @binding(2) var s_shadow: sampler_comparison;
@group(0) @binding(3) var t_block_atlas: texture_2d<f32>;
@group(0) @binding(4) var s_block_atlas: sampler;
@group(0) @binding(5) var<storage, read> block_atlas_rects: array<vec4<f32>>;

struct BlockMaterial {
    secondary_color_texture: vec4<f32>,
    variation: vec4<f32>,
    flags: vec4<f32>,
}
@group(0) @binding(6) var<storage, read> block_materials: array<BlockMaterial>;

struct Local {
    model: mat4x4<f32>,
    params: vec4<f32>, // x = opacity
}
@group(1) @binding(0) var<uniform> local: Local;

// --- CONSTANTS ---
const SHADOW_OPACITY  = 0.85; // Shadows are not pitch black

// --- VERTEX SHADER ---

struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) texture_id: i32,
    @location(5) block_id: i32,
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
};

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    
    // World Position
    let world_pos = local.model * vec4<f32>(in.pos, 1.0);
    out.world_pos = world_pos.xyz;
    
    // Clip Position (Main Camera)
    out.clip_pos = global.view_proj * world_pos;
    
    // Normal Transformation
    let normal_mat = mat3x3<f32>(
        local.model[0].xyz,
        local.model[1].xyz,
        local.model[2].xyz
    );
    out.world_normal = normalize(normal_mat * in.normal);
    
    // Color (Vertex Color + Baked AO)
    out.color = in.color;
    out.uv = in.uv;
    out.texture_id = in.texture_id;
    out.block_id = in.block_id;
    out.view_pos = global.camera_pos.xyz;

    // Shadow Calculation Space
    // We pre-calculate this to save work in the fragment shader
    // We apply a "Normal Offset" bias here to fix shadow acne on rounded surfaces
    let normal_offset = out.world_normal * 0.05; 
    let pos_light = global.light_view_proj * vec4<f32>(out.world_pos + normal_offset, 1.0);
    
    // Convert to [0, 1] texture space
    out.shadow_pos = vec3<f32>(
        pos_light.x * 0.5 + 0.5,
        -pos_light.y * 0.5 + 0.5,
        pos_light.z
    );

    return out;
}

// --- SHADOW ENGINE (Gaussian PCF) ---

fn fetch_shadow_accurate(shadow_pos: vec3<f32>, NdotL: f32) -> f32 {
    // 1. Cull outside cascade
    if (shadow_pos.z > 1.0 || shadow_pos.x < 0.0 || shadow_pos.x > 1.0 || shadow_pos.y < 0.0 || shadow_pos.y > 1.0) {
        return 1.0;
    }

    // 2. Slope-Scaled Bias
    // Steeper angles need more bias to prevent acne.
    // Base bias matches the texel size of a 4096 map covering ~120 units.
    let bias = max(0.0005 * (1.0 - NdotL), 0.0001);
    let current_depth = shadow_pos.z - bias;

    let tex_dim = vec2<f32>(textureDimensions(t_shadow));
    let texel_size = 1.0 / tex_dim.x;

    // 3. 5x5 Gaussian Weighted PCF
    // We sample a grid, but center samples matter more.
    var shadow_sum = 0.0;
    var total_weight = 0.0;

    // Gaussian weights for range -2 to +2
    // [0.05, 0.25, 0.4, 0.25, 0.05] roughly
    
    for (var x = -1.0; x <= 1.0; x += 1.0) {
        for (var y = -1.0; y <= 1.0; y += 1.0) {
            // Calculate weight based on distance from center (Gaussian-ish)
            let dist_sq = x*x + y*y;
            let weight = exp(-dist_sq * 1.5); // Gaussian Falloff

            let val = textureSampleCompare(
                t_shadow, 
                s_shadow, 
                shadow_pos.xy + vec2<f32>(x, y) * texel_size, 
                current_depth
            );
            
            shadow_sum += val * weight;
            total_weight += weight;
        }
    }

    return shadow_sum / total_weight;
}

// --- UTILS ---

fn dither_opacity(pos: vec4<f32>, alpha: f32) -> bool {
    // 4x4 Ordered Dithering Matrix
    let dither_threshold = dot(vec2<f32>(171.0, 231.0), pos.xy);
    return fract(dither_threshold / 71.0) > alpha;
}

fn triplanar_detail(pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    // Adds subtle grain to voxels so they don't look like plastic
    let p = pos * 2.0;
    let n = abs(normal);
    // Tight blend
    let w = pow(n, vec3<f32>(16.0)); 
    let weights = w / (w.x + w.y + w.z);
    
    // Fast hash noise
    let hx = fract(sin(dot(p.yz, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let hy = fract(sin(dot(p.zx, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let hz = fract(sin(dot(p.xy, vec2<f32>(12.9898, 78.233))) * 43758.5453);

    return (hx * weights.x + hy * weights.y + hz * weights.z) * 2.0 - 1.0;
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

fn material_for(block_id: i32) -> BlockMaterial {
    if (block_id < 0) {
        return BlockMaterial(
            vec4<f32>(1.0, 1.0, 1.0, 1.0),
            vec4<f32>(0.03, 0.02, 0.02, 0.015),
            vec4<f32>(0.0, 0.7, 0.0, 0.0),
        );
    }
    return block_materials[u32(block_id)];
}

fn rotated_variant_uv(uv: vec2<f32>, face_hash: f32, material_kind: f32) -> vec2<f32> {
    if (material_kind == 8.0) {
        return uv;
    }
    let tile = floor(face_hash * 4.0);
    let local = fract(uv);
    if (tile < 1.0) {
        return local;
    }
    if (tile < 2.0) {
        return vec2<f32>(1.0 - local.y, local.x);
    }
    if (tile < 3.0) {
        return vec2<f32>(1.0 - local.x, 1.0 - local.y);
    }
    return vec2<f32>(local.y, 1.0 - local.x);
}

fn material_color_shift(kind: f32, n: f32) -> vec3<f32> {
    if (kind == 1.0) {
        return vec3<f32>(-0.04 + n * 0.08, -0.01 + n * 0.10, -0.05 + n * 0.04);
    }
    if (kind == 2.0) {
        return vec3<f32>(-0.03 + n * 0.07, -0.02 + n * 0.04, -0.025 + n * 0.03);
    }
    if (kind == 3.0 || kind == 7.0 || kind == 9.0) {
        let v = -0.045 + n * 0.09;
        return vec3<f32>(v, v, v * 0.95);
    }
    if (kind == 4.0) {
        return vec3<f32>(-0.02 + n * 0.08, -0.015 + n * 0.065, -0.035 + n * 0.04);
    }
    if (kind == 5.0 || kind == 8.0) {
        return vec3<f32>(-0.035 + n * 0.07, -0.02 + n * 0.045, -0.03 + n * 0.035);
    }
    if (kind == 6.0) {
        return vec3<f32>(-0.045 + n * 0.07, -0.025 + n * 0.11, -0.05 + n * 0.04);
    }
    return vec3<f32>(-0.03 + n * 0.06);
}

// --- TONE MAPPING (ACES) ---
// Industry standard for realistic color reproduction
fn aces_approx(v: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((v * (a * v + b)) / (v * (c * v + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

// --- FRAGMENT SHADER ---

@fragment
fn fs_feedback(in: VertexOut) -> @location(0) vec4<f32> {
    // Gameplay feedback is authored as screen-readable overlay geometry.
    // It should not inherit terrain lighting or shadow grain.
    return vec4<f32>(in.color, local.params.x * 0.82);
}

fn block_albedo(in: VertexOut) -> vec3<f32> {
    let material = material_for(in.block_id);
    let kind = material.flags.x;
    let block_cell = floor(in.world_pos * 2.0);
    let face_key = block_cell + floor(abs(in.world_normal) * 17.0);
    let block_hash = hash13(block_cell + vec3<f32>(f32(in.block_id) * 0.37, 3.1, 9.7));
    let face_hash = hash13(face_key + vec3<f32>(f32(in.block_id) * 0.11, 5.3, 1.7));
    let macro_hash = value_noise(in.world_pos * 0.018 + vec3<f32>(f32(in.block_id), 0.0, 0.0));
    let detail_hash = value_noise(in.world_pos * 2.7 + in.world_normal * 11.0);

    let vert_color_linear = pow(in.color, vec3<f32>(2.2));
    let material_mix = mix(vert_color_linear, material.secondary_color_texture.rgb, face_hash * 0.45);
    var identity_color = material_mix;
    if (in.texture_id >= 0) {
        let rect = block_atlas_rects[u32(in.texture_id)];
        let varied_uv = rotated_variant_uv(in.uv, face_hash, kind);
        let atlas_uv = mix(rect.xy, rect.zw, varied_uv);
        let tex_color = textureSample(t_block_atlas, s_block_atlas, atlas_uv).rgb;
        let texture_influence = material.secondary_color_texture.w;
        identity_color = mix(material_mix, tex_color * vert_color_linear, texture_influence);
    }
    let organic_shift =
        material_color_shift(kind, block_hash) * material.variation.x +
        material_color_shift(kind, face_hash) * material.variation.y +
        material_color_shift(kind, macro_hash) * material.variation.z;
    let detail = (detail_hash * 2.0 - 1.0) * material.variation.w;
    return clamp(identity_color * (1.0 + organic_shift + detail), vec3<f32>(0.0), vec3<f32>(3.0));
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    // 1. Transparency Dithering
    if (local.params.x < 1.0 && dither_opacity(in.clip_pos, local.params.x)) {
        discard;
    }

    let N = normalize(in.world_normal);
    let L = normalize(global.atmosphere.sun_direction.xyz);
    let V = normalize(global.camera_pos.xyz - in.world_pos);

    // 2. Material Setup
    // Apply Detail Noise (Grain)
    let material = material_for(in.block_id);
    let noise = triplanar_detail(in.world_pos, N);
    let albedo = block_albedo(in) * (1.0 + material.variation.w * noise);

    // 3. Lighting Math
    let NdotL = max(dot(N, L), 0.0);
    
    // Shadow Map
    let shadow_raw = fetch_shadow_accurate(in.shadow_pos, NdotL);
    // Smooth transition shadow
    let shadow = mix(1.0 - SHADOW_OPACITY, 1.0, shadow_raw);

    // A. Direct Sun Light
    let direct_light = global.atmosphere.sun_color.xyz * NdotL * shadow;

    // B. Hemispheric Ambient
    // Top of objects gets Sky Color, Bottom gets Ground Bounce
    let up_dot = dot(N, normalize(in.world_pos)); // Relative Up for sphere
    let hemi_factor = up_dot * 0.5 + 0.5;
    let ambient_light = mix(
        global.atmosphere.ground_ambient_color.xyz,
        global.atmosphere.sky_color.xyz,
        hemi_factor
    );

    // C. Fresnel Rim
    // Adds a subtle glow at grazing angles (atmosphere dust effect)
    let fresnel = pow(1.0 - max(dot(N, V), 0.0), 3.0);
    let rim_light = global.atmosphere.sky_color.xyz * fresnel * 0.2 * shadow;

    // Combine
    // Note: Ambient is multiplied by albedo (diffuse reflection)
    var final_color = albedo * (direct_light + ambient_light + rim_light);

    // 4. Fog (Atmospheric Scattering)
    let dist = distance(global.camera_pos.xyz, in.world_pos);
    let fog_density = global.atmosphere.fog_color_density.w;
    let fog_factor = 1.0 - exp(-(dist * fog_density) * (dist * fog_density * 0.5)); // Exp2 fog
    final_color = mix(final_color, global.atmosphere.fog_color_density.xyz, clamp(fog_factor, 0.0, 1.0));

    // 5. Post Processing
    // Tone Mapping (HDR -> LDR)
    final_color = aces_approx(final_color);
    
    // Gamma Correction (Linear -> sRGB)
    final_color = pow(final_color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(final_color, 1.0);
}
