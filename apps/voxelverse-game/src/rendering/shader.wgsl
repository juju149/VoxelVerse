// Stylized PBR-lite shading.
struct Global {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    sun_dir: vec4<f32>,
}

@group(0) @binding(0) var<uniform> global: Global;
@group(0) @binding(1) var t_shadow: texture_depth_2d;
@group(0) @binding(2) var s_shadow: sampler_comparison;

struct Local {
    model: mat4x4<f32>,
    params: vec4<f32>, // x = opacity, y = rounded edge radius in voxel UV
}
@group(1) @binding(0) var<uniform> local: Local;

@group(2) @binding(0) var t_albedo: texture_2d_array<f32>;
@group(2) @binding(1) var t_normal: texture_2d_array<f32>;
@group(2) @binding(2) var t_roughness: texture_2d_array<f32>;
@group(2) @binding(3) var s_material: sampler;
// One vec4 per atlas layer — flat per-block color used by the Fn debug toggle
// to skip texture sampling entirely (perf A/B comparison vs. textured mode).
@group(2) @binding(4) var<storage, read> material_colors: array<vec4<f32>>;

// --- CONSTANTS ---
const SUN_COLOR       = vec3<f32>(1.25, 1.12, 0.82);
const SKY_COLOR       = vec3<f32>(0.28, 0.48, 0.86);
const GROUND_COLOR    = vec3<f32>(0.11, 0.09, 0.06);
const SHADOW_OPACITY  = 0.62;
// Sentinel material index for prop voxels — vertex colour carries the full
// albedo; no texture or material_colors[] lookup should be performed.
const PROP_VERTEX_COLOR_ONLY = 0xFFFFu;

// --- VERTEX SHADER ---

struct VertexIn {
    @location(0) pos:       vec3<f32>,
    @location(1) uv:        vec2<f32>,
    @location(2) normal:    vec3<f32>,
    @location(3) color:     vec3<f32>,
    @location(4) tex_index: u32,
};

struct VertexOut {
    @builtin(position)              clip_pos:    vec4<f32>,
    @location(0)                    uv:          vec2<f32>,
    @location(1)                    world_normal: vec3<f32>,
    @location(2)                    world_pos:   vec3<f32>,
    @location(3)                    view_pos:    vec3<f32>,
    @location(4)                    shadow_pos:       vec3<f32>,
    @location(5)                    color:            vec3<f32>,
    @location(6) @interpolate(flat) packed_tex_index: u32,
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
    
    // Pass-through
    out.color     = in.color;
    out.uv        = in.uv;
    out.packed_tex_index = in.tex_index;
    out.view_pos  = global.camera_pos.xyz;

    // Shadow Calculation Space
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

fn fetch_shadow_accurate(shadow_pos: vec3<f32>, NdotL: f32, pcf_radius: i32) -> f32 {
    // 1. Cull outside cascade
    if (shadow_pos.z > 1.0 || shadow_pos.x < 0.0 || shadow_pos.x > 1.0 || shadow_pos.y < 0.0 || shadow_pos.y > 1.0) {
        return 1.0;
    }

    // 2. Slope-Scaled Bias — steeper angles need more bias to avoid acne.
    let bias = max(0.0005 * (1.0 - NdotL), 0.0001);
    let current_depth = shadow_pos.z - bias;

    let tex_dim = vec2<f32>(textureDimensions(t_shadow));
    let texel_size = 1.0 / tex_dim.x;

    // 3. Gaussian-weighted PCF.  Radius is dynamic so the engine can dial
    //    quality up or down without recompiling: 1 = 3x3, 2 = 5x5, 3 = 7x7.
    let r = max(pcf_radius, 0);
    var shadow_sum = 0.0;
    var total_weight = 0.0;
    for (var x: i32 = -r; x <= r; x++) {
        for (var y: i32 = -r; y <= r; y++) {
            let dist_sq = f32(x*x + y*y);
            let weight = exp(-dist_sq * 1.5);
            let val = textureSampleCompare(
                t_shadow,
                s_shadow,
                shadow_pos.xy + vec2<f32>(f32(x), f32(y)) * texel_size,
                current_depth
            );
            shadow_sum += val * weight;
            total_weight += weight;
        }
    }
    return shadow_sum / total_weight;
}

// --- UTILS ---

const MATERIAL_INDEX_MASK = 0x0000FFFFu;
const EDGE_MIN_U = 1u;
const EDGE_MAX_U = 2u;
const EDGE_MIN_V = 4u;
const EDGE_MAX_V = 8u;

struct SurfaceBasis {
    tangent: vec3<f32>,
    bitangent: vec3<f32>,
};

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

fn surface_basis(world_pos: vec3<f32>, uv: vec2<f32>, world_normal: vec3<f32>) -> SurfaceBasis {
    let dpdx_v = dpdx(world_pos);
    let dpdy_v = dpdy(world_pos);
    let duvdx_v = dpdx(uv);
    let duvdy_v = dpdy(uv);
    let det = duvdx_v.x * duvdy_v.y - duvdx_v.y * duvdy_v.x;

    var tangent: vec3<f32>;
    var bitangent: vec3<f32>;
    if (abs(det) > 0.000001) {
        let inv_det = 1.0 / det;
        tangent = normalize((dpdx_v * duvdy_v.y - dpdy_v * duvdx_v.y) * inv_det);
        bitangent = normalize((dpdy_v * duvdx_v.x - dpdx_v * duvdy_v.x) * inv_det);
    } else {
        let up_ref = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(world_normal.y) > 0.92);
        tangent = normalize(cross(up_ref, world_normal));
        bitangent = normalize(cross(world_normal, tangent));
    }

    return SurfaceBasis(tangent, bitangent);
}

fn smootherstep01(x: f32) -> f32 {
    let t = clamp(x, 0.0, 1.0);
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn bevel_profile(enabled: bool, distance_to_edge: f32, radius: f32, feather: f32) -> f32 {
    // Screen-space feather keeps the normal transition stable at distance.
    // The silhouette stays a cube; only lighting gets a rounded-box response.
    let inside_radius = 1.0 - smoothstep(radius - feather, radius + feather, distance_to_edge);
    let amount = smootherstep01(inside_radius);
    return select(0.0, amount, enabled && radius > 0.0001);
}

fn rounded_edge_normal(
    world_normal: vec3<f32>,
    uv: vec2<f32>,
    edge_mask: u32,
    radius: f32,
    basis: SurfaceBasis,
    edge_feather: vec2<f32>,
) -> vec3<f32> {
    let safe_radius = clamp(radius, 0.0, 0.35);
    if (edge_mask == 0u || safe_radius <= 0.0001) {
        return world_normal;
    }

    let min_u = (edge_mask & EDGE_MIN_U) != 0u;
    let max_u = (edge_mask & EDGE_MAX_U) != 0u;
    let min_v = (edge_mask & EDGE_MIN_V) != 0u;
    let max_v = (edge_mask & EDGE_MAX_V) != 0u;

    let u_min = bevel_profile(min_u, uv.x, safe_radius, edge_feather.x);
    let u_max = bevel_profile(max_u, 1.0 - uv.x, safe_radius, edge_feather.x);
    let v_min = bevel_profile(min_v, uv.y, safe_radius, edge_feather.y);
    let v_max = bevel_profile(max_v, 1.0 - uv.y, safe_radius, edge_feather.y);

    let signed_u = u_max - u_min;
    let signed_v = v_max - v_min;
    let edge_strength = max(max(u_min, u_max), max(v_min, v_max));
    let corner_strength = max(u_min, u_max) * max(v_min, v_max);

    // Keep a strong face component so the result reads as a soft radius, not a
    // geometric chamfer. Corners receive a little extra diagonal bend.
    let face_keep = mix(1.0, 0.76, edge_strength) - corner_strength * 0.08;
    let side_gain = mix(0.72, 0.92, corner_strength);
    let bend = (basis.tangent * signed_u + basis.bitangent * signed_v) * side_gain;
    return normalize(world_normal * max(face_keep, 0.62) + bend);
}

fn rounded_edge_amount(
    uv: vec2<f32>,
    edge_mask: u32,
    radius: f32,
    edge_feather: vec2<f32>,
) -> f32 {
    let safe_radius = clamp(radius, 0.0, 0.35);
    if (edge_mask == 0u || safe_radius <= 0.0001) {
        return 0.0;
    }

    let min_u = (edge_mask & EDGE_MIN_U) != 0u;
    let max_u = (edge_mask & EDGE_MAX_U) != 0u;
    let min_v = (edge_mask & EDGE_MIN_V) != 0u;
    let max_v = (edge_mask & EDGE_MAX_V) != 0u;

    let u_amount = max(
        bevel_profile(min_u, uv.x, safe_radius, edge_feather.x),
        bevel_profile(max_u, 1.0 - uv.x, safe_radius, edge_feather.x)
    );
    let v_amount = max(
        bevel_profile(min_v, uv.y, safe_radius, edge_feather.y),
        bevel_profile(max_v, 1.0 - uv.y, safe_radius, edge_feather.y)
    );
    return max(u_amount, v_amount);
}

fn material_normal(world_normal: vec3<f32>, uv: vec2<f32>, layer: u32, basis: SurfaceBasis) -> vec3<f32> {
    let n_tex = textureSample(t_normal, s_material, uv, i32(layer)).xyz * 2.0 - vec3<f32>(1.0);
    return normalize(basis.tangent * n_tex.x + basis.bitangent * n_tex.y + world_normal * max(n_tex.z, 0.12));
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
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let edge_feather = max(fwidth(in.uv) * 1.5, vec2<f32>(0.0005));

    // 1. Transparency Dithering
    if (local.params.x < 1.0 && dither_opacity(in.clip_pos, local.params.x)) {
        discard;
    }

    var N = normalize(in.world_normal);
    let L = normalize(global.sun_dir.xyz);
    let V = normalize(global.camera_pos.xyz - in.world_pos);

    // Decode runtime quality knobs packed into camera_pos.w
    //   bit 0      = triplanar grain enable
    //   bits 1..2  = pcf radius level (0/1/2 → 3x3/5x5/7x7)
    //   bit 3      = color-only mode (skip texture sampling entirely)
    let quality_bits   = u32(global.camera_pos.w);
    let triplanar_on   = (quality_bits & 1u) != 0u;
    let pcf_radius     = i32(((quality_bits >> 1u) & 3u) + 1u);
    let color_only     = (quality_bits & 8u) != 0u;

    // 2. Material Setup
    let material_layer = in.packed_tex_index & MATERIAL_INDEX_MASK;
    let edge_mask = (in.packed_tex_index >> 16u) & 0xFu;
    let basis = surface_basis(in.world_pos, in.uv, N);

    var albedo_rgb: vec3<f32>;
    var roughness: f32;
    if (material_layer == PROP_VERTEX_COLOR_ONLY) {
        // Prop voxel path — vertex colour is the full pre-darkened sRGB colour.
        // No texture array fetch, no material_colors buffer read, no normal map,
        // no bevel (prop faces are tiny — rounded edges would be invisible).
        // The shader's gamma-expansion below handles sRGB→linear conversion.
        albedo_rgb = vec3<f32>(1.0, 1.0, 1.0); // vertex colour × 1.0 = vertex colour
        roughness = 0.78; // matte-ish, similar to leaves/grass
    } else if (color_only) {
        // Cheap path: one storage-buffer fetch, no texture sampling, no
        // normal-map perturbation. The vertex `in.color` already carries
        // AO × skylight tint, so we keep it as a multiplier.
        albedo_rgb = material_colors[material_layer].rgb;
        roughness = 0.7;
        N = rounded_edge_normal(N, in.uv, edge_mask, local.params.y, basis, edge_feather);
    } else {
        let albedo_sample = textureSample(t_albedo, s_material, in.uv, i32(material_layer));
        roughness = textureSample(t_roughness, s_material, in.uv, i32(material_layer)).r;
        N = material_normal(N, in.uv, material_layer, basis);
        N = rounded_edge_normal(N, in.uv, edge_mask, local.params.y, basis, edge_feather);
        albedo_rgb = albedo_sample.rgb;
    }
    let bevel_amount = rounded_edge_amount(in.uv, edge_mask, local.params.y, edge_feather);
    let vert_color_linear = pow(in.color, vec3<f32>(2.2));

    // Detail grain — gated behind the triplanar quality flag.  3 sin() per
    // fragment is non-trivial on weak GPUs, so it's optional.
    let bevel_contrast = mix(1.0, 0.96, bevel_amount);
    var albedo = vert_color_linear * albedo_rgb * bevel_contrast;
    if (triplanar_on && !color_only) {
        let noise = triplanar_detail(in.world_pos, N);
        albedo = albedo * (1.0 + 0.025 * noise);
    }

    // 3. Lighting Math
    let NdotL = max(dot(N, L) * 0.82 + 0.18, 0.0);

    // Shadow Map
    let shadow_raw = fetch_shadow_accurate(in.shadow_pos, NdotL, pcf_radius);
    // Smooth transition shadow
    let shadow = mix(1.0 - SHADOW_OPACITY, 1.0, shadow_raw);

    let matte = mix(1.05, 0.62, roughness);
    let direct_light = SUN_COLOR * NdotL * shadow * matte;

    // B. Hemispheric Ambient
    // Top of objects gets Sky Color, Bottom gets Ground Bounce
    let up_dot = dot(N, normalize(in.world_pos)); // Relative Up for sphere
    let hemi_factor = up_dot * 0.5 + 0.5;
    let ambient_light = mix(GROUND_COLOR, SKY_COLOR, hemi_factor) * mix(0.92, 1.22, roughness);

    // C. Fresnel Rim
    // Adds a subtle glow at grazing angles (atmosphere dust effect)
    let fresnel = pow(1.0 - max(dot(N, V), 0.0), 3.0);
    let rim_light = SKY_COLOR * fresnel * 0.12 * shadow * (1.0 - roughness * 0.35);

    // Combine
    // Note: Ambient is multiplied by albedo (diffuse reflection)
    var final_color = albedo * (direct_light + ambient_light + rim_light);

    // 4. Fog (Atmospheric Scattering)
    // fog_density is passed in sun_dir.w, computed per-planet from surface radius.
    let dist = distance(global.camera_pos.xyz, in.world_pos);
    let fog_density = global.sun_dir.w;
    let fog_factor = 1.0 - exp(-(dist * fog_density) * (dist * fog_density * 0.5));

    // Horizon color: warm haze near the sun mixes into the sky blue.
    let fog_col = mix(SKY_COLOR * 0.9, vec3<f32>(0.72, 0.82, 1.0), 0.25);
    final_color = mix(final_color, fog_col, clamp(fog_factor, 0.0, 1.0));

    // 5. Post Processing
    // Tone Mapping (HDR -> LDR)
    final_color = aces_approx(final_color);
    
    // Gamma Correction (Linear -> sRGB)
    final_color = pow(final_color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(final_color, 1.0);
}
