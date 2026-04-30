

struct Atmosphere {
    sun_direction: vec4<f32>,
    sun_color: vec4<f32>,
    sky_color: vec4<f32>,
    ground_ambient_color: vec4<f32>,
    // Cool fill tint applied to sun-facing surfaces in shadow.
    // Field order must match AtmosphereUniform in atmosphere.rs exactly.
    shadow_tint_color: vec4<f32>,
    fog_color_density: vec4<f32>,
    clear_color: vec4<f32>,
}

struct Global {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    atmosphere: Atmosphere,
    // Inverse view-projection matrix: used by sky shader to reconstruct world-space ray direction.
    // Field order must match GlobalUniform in renderer.rs exactly.
    inv_view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> global: Global;
@group(0) @binding(1) var t_shadow: texture_depth_2d;
@group(0) @binding(2) var s_shadow: sampler_comparison;
@group(0) @binding(3) var t_block_atlas: texture_2d<f32>;
@group(0) @binding(4) var s_block_atlas: sampler;
@group(0) @binding(5) var<storage, read> block_atlas_rects: array<vec4<f32>>;

struct BlockMaterial {
    base_color_flags: vec4<f32>,
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
// (SHADOW_OPACITY removed: shadow tint is now data-driven via atmosphere.shadow_tint_color)

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
            vec4<f32>(1.0, 1.0, 1.0, 0.0),
            vec4<f32>(1.0, 1.0, 1.0, 1.0),
            vec4<f32>(0.03, 0.02, 0.02, 0.015),
            vec4<f32>(0.0, 0.7, 0.0, 0.0),
        );
    }
    return block_materials[u32(block_id)];
}

fn rotated_variant_uv(uv: vec2<f32>, face_hash: f32, material_kind: f32) -> vec2<f32> {
    if (material_kind == 9.0 || material_kind == 10.0 || material_kind == 12.0) {
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

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn material_texture_strength(kind: f32, authored: f32) -> f32 {
    if (kind == 1.0 || kind == 2.0 || kind == 3.0 || kind == 5.0 || kind == 7.0) {
        return authored * 0.16;
    }
    if (kind == 4.0 || kind == 6.0 || kind == 8.0 || kind == 11.0) {
        return authored * 0.24;
    }
    if (kind == 9.0 || kind == 10.0) {
        return authored * 0.34;
    }
    return authored * 0.22;
}

fn material_face_grade(kind: f32, topness: f32, sideness: f32, bottomness: f32) -> vec3<f32> {
    if (kind == 1.0) {
        return vec3<f32>(1.0 + topness * 0.08 - bottomness * 0.14,
                         1.0 + topness * 0.12 - bottomness * 0.10,
                         1.0 - topness * 0.04 - bottomness * 0.08);
    }
    if (kind == 2.0) {
        return vec3<f32>(1.0 + topness * 0.04 - bottomness * 0.08,
                         1.0 + topness * 0.02 - bottomness * 0.08,
                         1.0 - bottomness * 0.08);
    }
    if (kind == 3.0 || kind == 8.0) {
        return vec3<f32>(1.0 + topness * 0.035 - sideness * 0.035,
                         1.0 + topness * 0.045 - sideness * 0.025,
                         1.0 + topness * 0.065);
    }
    if (kind == 4.0 || kind == 9.0 || kind == 11.0) {
        return vec3<f32>(1.0 + topness * 0.03 - bottomness * 0.10);
    }
    if (kind == 5.0) {
        return vec3<f32>(1.0 + topness * 0.06, 1.0 + topness * 0.045, 1.0 - sideness * 0.04);
    }
    if (kind == 6.0 || kind == 10.0) {
        return vec3<f32>(1.0 + topness * 0.025, 1.0 + topness * 0.015, 1.0 - bottomness * 0.045);
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
    macro_b: f32,
) -> vec3<f32> {
    let natural_mix = 0.12 + macro_a * 0.28 + topness * 0.12;
    var color = mix(base, secondary, natural_mix);
    if (kind == 1.0) {
        let sun = vec3<f32>(0.54, 0.80, 0.24);
        let mid = vec3<f32>(0.31, 0.60, 0.18);
        let deep = vec3<f32>(0.13, 0.34, 0.12);
        color = mix(mix(deep, mid, macro_a), sun, topness * 0.46 + macro_b * 0.12);
    } else if (kind == 2.0) {
        color = mix(vec3<f32>(0.23, 0.13, 0.075), vec3<f32>(0.52, 0.30, 0.14), macro_a * 0.72 + macro_b * 0.22);
    } else if (kind == 3.0) {
        color = mix(vec3<f32>(0.76, 0.84, 0.92), vec3<f32>(0.98, 0.98, 0.94), macro_a);
    } else if (kind == 4.0 || kind == 11.0) {
        color = mix(vec3<f32>(0.30, 0.31, 0.30), vec3<f32>(0.58, 0.56, 0.50), macro_a);
    } else if (kind == 5.0) {
        color = mix(vec3<f32>(0.58, 0.48, 0.26), vec3<f32>(0.92, 0.77, 0.43), macro_a);
    } else if (kind == 6.0) {
        color = mix(vec3<f32>(0.30, 0.14, 0.055), vec3<f32>(0.78, 0.42, 0.16), macro_a);
    } else if (kind == 7.0) {
        let shadow_leaf = vec3<f32>(0.055, 0.19, 0.075);
        let body_leaf = vec3<f32>(0.16, 0.43, 0.12);
        let sun_leaf = vec3<f32>(0.48, 0.73, 0.20);
        color = mix(mix(shadow_leaf, body_leaf, macro_a), sun_leaf, topness * 0.34 + macro_b * 0.18);
    } else if (kind == 8.0) {
        color = mix(vec3<f32>(0.30, 0.60, 0.78), vec3<f32>(0.76, 0.93, 0.98), macro_a);
    } else if (kind == 9.0) {
        color = mix(base, secondary, macro_a * 0.22);
    } else if (kind == 10.0) {
        color = mix(base, secondary, macro_a * 0.18 + sideness * 0.04);
    }
    return color * material_face_grade(kind, topness, sideness, 1.0 - max(topness, sideness));
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
    let N = normalize(in.world_normal);
    let radial_up = normalize(in.world_pos);
    let up_dot = clamp(dot(N, radial_up), -1.0, 1.0);
    let topness = smoothstep(0.18, 0.92, up_dot);
    let bottomness = smoothstep(0.15, 0.82, -up_dot);
    let sideness = clamp(1.0 - max(topness, bottomness), 0.0, 1.0);

    let block_cell = floor(in.world_pos * 2.0);
    let face_key = block_cell + floor(abs(N) * 17.0);
    let block_hash = hash13(block_cell + vec3<f32>(f32(in.block_id) * 0.37, 3.1, 9.7));
    let face_hash = hash13(face_key + vec3<f32>(f32(in.block_id) * 0.11, 5.3, 1.7));
    let macro_hash = value_noise(in.world_pos * 0.011 + radial_up * 3.0 + vec3<f32>(f32(in.block_id), 0.0, 0.0));
    let broad_hash = value_noise(in.world_pos * 0.0035 + vec3<f32>(0.0, f32(in.block_id), length(in.world_pos) * 0.002));
    let detail_hash = value_noise(in.world_pos * 2.1 + N * 9.0);

    let vert_color_linear = material.base_color_flags.rgb;
    var identity_color = material_palette(
        kind,
        vert_color_linear,
        material.secondary_color_texture.rgb,
        topness,
        sideness,
        macro_hash,
        broad_hash,
    );
    if (in.texture_id >= 0) {
        let rect = block_atlas_rects[u32(in.texture_id)];
        let varied_uv = rotated_variant_uv(in.uv, face_hash, kind);
        let atlas_uv = mix(rect.xy, rect.zw, varied_uv);
        let tex_color = textureSample(t_block_atlas, s_block_atlas, atlas_uv).rgb;
        let texture_strength = material_texture_strength(kind, material.secondary_color_texture.w);
        let tex_luma = luminance(tex_color);
        let luma_detail = (tex_luma - 0.5) * texture_strength;
        let chroma_detail = (tex_color - vec3<f32>(tex_luma)) * (texture_strength * 0.16);
        identity_color = identity_color * (1.0 + luma_detail) + chroma_detail;
    }
    let organic_shift =
        material_color_shift(kind, block_hash) * material.variation.x +
        material_color_shift(kind, face_hash) * material.variation.y +
        material_color_shift(kind, broad_hash) * material.variation.z;
    let detail = (detail_hash * 2.0 - 1.0) * material.variation.w * (0.55 + sideness * 0.35);
    let exposure_tint = mix(vec3<f32>(0.94, 0.95, 0.96), vec3<f32>(1.04, 1.035, 1.0), topness);
    return clamp(identity_color * exposure_tint * (1.0 + organic_shift + detail), vec3<f32>(0.0), vec3<f32>(3.0));
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
    let radial_up = normalize(in.world_pos);

    // 2. Material & Albedo
    let material = material_for(in.block_id);
    let roughness = material.flags.y;
    let noise = triplanar_detail(in.world_pos, N);
    let albedo = block_albedo(in) * (1.0 + material.variation.w * noise);

    // 3. AO from baked vertex color (mesh encodes 1.0=open, 0.8=one-side, 0.6=two-sides, 0.4=corner)
    let ao_raw = in.color.r;
    // Remap to 0..1 for tinting (0.4..1.0 -> 0..1)
    let ao_t = clamp((ao_raw - 0.4) / 0.6, 0.0, 1.0);
    // Cool blue-violet tint in occluded cavities — richens depth and shadows
    let ao_tint = mix(vec3<f32>(0.62, 0.70, 0.92), vec3<f32>(1.0), ao_t);
    // AO quadratically suppresses ambient (strong), softly suppresses direct (subtle)
    let ao_ambient = ao_raw * ao_raw;
    let ao_direct  = mix(ao_raw, 1.0, 0.50);

    // 4. Shadow
    let NdotL = max(dot(N, L), 0.0);
    let shadow_raw = fetch_shadow_accurate(in.shadow_pos, NdotL);
    let soft_shadow = mix(shadow_raw, smoothstep(0.0, 1.0, shadow_raw),
                          clamp(roughness, 0.0, 1.0) * 0.35);

    // 5. Direct light — warm sun on lit surfaces, cool fill on shadowed sun-facing surfaces
    let sun_lit     = global.atmosphere.sun_color.xyz * NdotL * soft_shadow;
    let shadow_fill = global.atmosphere.shadow_tint_color.xyz * NdotL * (1.0 - soft_shadow);
    let direct_light = (sun_lit + shadow_fill) * ao_direct;

    // 6. Hemispheric Ambient — sky above, cool ground bounce below
    let hemi_factor = dot(N, radial_up) * 0.5 + 0.5;
    let ambient_light = mix(
        global.atmosphere.ground_ambient_color.xyz,
        global.atmosphere.sky_color.xyz,
        hemi_factor
    ) * (0.85 + roughness * 0.18) * ao_ambient;

    // 7. Rim light — not shadow-dependent; serves silhouettes and back-lit geometry
    let fresnel = pow(1.0 - max(dot(N, V), 0.0), 3.0);
    let backlit  = pow(max(dot(N, -L), 0.0), 2.0);
    let rim_light = global.atmosphere.sky_color.xyz * (fresnel * 0.10 + backlit * fresnel * 0.08);

    // 8. Combine
    var final_color = albedo * ao_tint * (direct_light + ambient_light + rim_light);

    // 9. Fog (Atmospheric Scattering)
    let dist = distance(global.camera_pos.xyz, in.world_pos);
    let fog_density = global.atmosphere.fog_color_density.w;
    let fog_factor = 1.0 - exp(-(dist * fog_density) * (dist * fog_density * 0.5));
    final_color = mix(final_color, global.atmosphere.fog_color_density.xyz, clamp(fog_factor, 0.0, 1.0));

    // 10. Tone mapping + Gamma
    final_color = aces_approx(final_color);
    final_color = pow(final_color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(final_color, 1.0);
}

// --- SKY SHADER ---
// Renders a procedural stylized sky as the first draw in the main pass.
// Drawn with depth_write_enabled=false so terrain always occludes it.
// All parameters are driven by the dynamic AtmosphereUniform (updated by SkyState on the CPU),
// so the sky automatically reflects the day/night cycle every frame.

struct SkyOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

// --- Star field ---
// Simple hash-based sparse star field.  Only visible when the sun is below the horizon.
fn star_field(view_dir: vec3<f32>) -> vec3<f32> {
    // Scale direction to control star density.
    let p = view_dir * 165.0;
    let cell = floor(p);
    let h1 = hash13(cell);
    let h2 = hash13(cell + vec3<f32>(13.71, 8.31, 19.17));
    let h3 = hash13(cell + vec3<f32>(7.13, 23.47, 3.91));
    // ~1.5 % of cells host a star.
    let brightness = smoothstep(0.985, 1.0, h1);
    // Vary color: blue-white to warm-white.
    let color = mix(vec3<f32>(0.72, 0.88, 1.00), vec3<f32>(1.00, 0.96, 0.78), h2);
    // Subtle pseudo-twinkling via the third hash.
    let twinkle = 0.70 + h3 * 0.55;
    return color * brightness * twinkle * 1.20;
}

// --- Sky gradient ---
// Computes the sky color for a given view direction.
// All atmosphere parameters are read from the global uniform (CPU-driven per frame).
fn sky_gradient(view_dir: vec3<f32>) -> vec3<f32> {
    let L          = normalize(global.atmosphere.sun_direction.xyz);
    let sky        = global.atmosphere.sky_color.xyz;
    let fog        = global.atmosphere.fog_color_density.xyz;
    let sun_col    = global.atmosphere.sun_color.xyz;
    // On a round planet "up" is radial from the planet centre to the camera.
    let radial_up  = normalize(global.camera_pos.xyz);
    let elevation  = dot(view_dir, radial_up);  // -1 = straight down, +1 = straight up
    let sun_elev   = dot(L, radial_up);          // sun's elevation for this observer

    // =========================================================================
    // NIGHT SKY
    // Fades in when the sun is below the horizon, fades out as it rises.
    // =========================================================================
    // night_t: 1.0 when fully dark, 0.0 when fully day.
    let night_t = smoothstep(0.18, -0.08, sun_elev);

    // Night sky gradient: deep indigo at zenith, slightly warmer near the horizon.
    let night_zenith = vec3<f32>(0.008, 0.012, 0.060);
    let night_horiz  = vec3<f32>(0.022, 0.028, 0.085);
    let night_sky    = mix(night_horiz, night_zenith, clamp(elevation * 2.8, 0.0, 1.0));

    // Stars: fade in with the night, only show above horizon.
    let star_vis    = clamp(elevation * 3.5 + 0.6, 0.0, 1.0);
    let stars       = star_field(view_dir) * night_t * star_vis;

    // Moon: appears directly opposite the sun at night.
    let moon_dir    = -L;
    let moon_dot    = clamp(dot(view_dir, moon_dir), 0.0, 1.0);
    let moon_disk   = smoothstep(0.9996, 1.0, moon_dot) * 0.65;
    let moon_halo   = pow(moon_dot, 22.0) * 0.035;
    let moon_above  = smoothstep(-0.12, 0.06, dot(moon_dir, radial_up));
    let moon_vis    = smoothstep(-0.14, 0.04, elevation);
    let moon        = vec3<f32>(0.86, 0.91, 1.00) * (moon_disk + moon_halo) * moon_above * moon_vis * night_t;

    // =========================================================================
    // DAY SKY
    // =========================================================================
    // Gradient: fog at horizon → richer blue at zenith.
    let zenith_color    = sky * vec3<f32>(0.62, 0.72, 1.22);
    let sky_t           = clamp(smoothstep(-0.08, 0.32, elevation), 0.0, 1.0);
    let sky_base        = mix(fog, zenith_color, sky_t * sky_t);
    // Below horizon: fade into ground haze.
    let below_t         = smoothstep(0.0, -0.22, elevation);
    let sky_ground      = mix(sky_base, fog * vec3<f32>(0.72, 0.78, 0.88), below_t);

    // Sun: disk + inner glow + scatter halo.
    let sun_dot         = clamp(dot(view_dir, L), 0.0, 1.0);
    let sun_disk        = smoothstep(0.9992, 0.9998, sun_dot) * 2.8;
    let sun_glow        = pow(sun_dot, 58.0) * 1.50;
    let sun_halo        = pow(sun_dot,  5.0) * 0.18;
    let sun_above       = smoothstep(-0.15, 0.08, sun_elev);
    let sun_vis         = smoothstep(-0.20, 0.04, elevation);
    let sun_contrib     = sun_col * (sun_disk + sun_glow + sun_halo) * sun_above * sun_vis;

    // Horizon haze band (independent of sun direction).
    let haze_t          = pow(max(0.0, 1.0 - abs(elevation) * 5.5), 2.2);
    let haze            = fog * haze_t * 0.42;

    // Twilight glow: warm orange/pink band near the horizon, aligned towards the sun.
    // Strongest when the sun is very close to the horizon (both sunrise and sunset).
    let abs_sun_elev    = abs(sun_elev);
    let twi_strength    = smoothstep(0.38, 0.0, abs_sun_elev);
    // Sun's horizontal component (azimuth direction on the horizon plane).
    let sun_horiz_dir   = normalize(L - radial_up * sun_elev);
    let view_near_horiz = smoothstep(0.22, 0.0, abs(elevation));
    let view_horiz_dir  = normalize(view_dir - radial_up * elevation);
    let az_align        = max(0.0, dot(view_horiz_dir, sun_horiz_dir));
    // Use fog color (driven by SkyState to be warm orange at sunrise/sunset) for the glow.
    let twi_glow        = fog * vec3<f32>(2.20, 0.75, 0.10) * twi_strength
                          * pow(az_align, 2.0) * view_near_horiz * 0.72;

    let day_sky = sky_ground + sun_contrib + haze + twi_glow;

    // =========================================================================
    // BLEND NIGHT / DAY
    // =========================================================================
    // night_t=1 → full night sky. night_t=0 → full day sky.
    // The transition through twilight is smooth due to the smoothstep threshold above.
    // A tiny minimum of day_sky prevents it from going completely black even at midnight.
    let base_sky = mix(day_sky, night_sky, night_t);
    return base_sky + stars + moon;
}

@vertex
fn vs_sky(@builtin(vertex_index) vi: u32) -> SkyOut {
    // Fullscreen triangle — position selected with switch to satisfy WGSL constant-index rule.
    var pos: vec2<f32>;
    switch vi {
        case 0u: { pos = vec2<f32>(-1.0, -1.0); }
        case 1u: { pos = vec2<f32>( 3.0, -1.0); }
        default: { pos = vec2<f32>(-1.0,  3.0); }
    }
    var out: SkyOut;
    // z=1.0, w=1.0 → NDC depth = far plane. depth_compare=Always so it always draws;
    // depth_write_enabled=false so terrain draw calls overwrite it normally.
    out.clip_pos = vec4<f32>(pos.x, pos.y, 1.0, 1.0);
    out.ndc = pos;
    return out;
}

@fragment
fn fs_sky(in: SkyOut) -> @location(0) vec4<f32> {
    // Reconstruct the world-space view direction from NDC via the inverse VP matrix.
    let clip_far  = vec4<f32>(in.ndc.x, in.ndc.y, 1.0, 1.0);
    let world_far = global.inv_view_proj * clip_far;
    let view_dir  = normalize(world_far.xyz / world_far.w - global.camera_pos.xyz);

    var color = sky_gradient(view_dir);

    // Same tone mapping and gamma correction as the terrain shader → seamless blend.
    color = aces_approx(color);
    color = pow(color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(color, 1.0);
}
