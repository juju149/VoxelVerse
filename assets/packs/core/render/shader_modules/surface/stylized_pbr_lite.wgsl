struct Global {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    sun_dir: vec4<f32>,
}

@group(0) @binding(0) var<uniform> global: Global;
@group(0) @binding(1) var t_shadow: texture_depth_2d;
@group(0) @binding(2) var s_shadow: sampler_comparison;

@group(2) @binding(0) var t_albedo: texture_2d_array<f32>;
@group(2) @binding(1) var t_normal: texture_2d_array<f32>;
@group(2) @binding(2) var t_roughness: texture_2d_array<f32>;
@group(2) @binding(3) var s_material: sampler;
@group(2) @binding(4) var<storage, read> material_colors: array<vec4<f32>>;

const MATERIAL_INDEX_MASK: u32 = 0x0000FFFFu;
const PROP_VERTEX_COLOR_ONLY: u32 = 0xFFFFu;

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) view_pos: vec3<f32>,
    @location(4) shadow_pos: vec3<f32>,
    @location(5) color: vec3<f32>,
    @location(6) @interpolate(flat) packed_tex_index: u32,
}

fn vv_tonemap_aces(color: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((color * (a * color + b)) / (color * (c * color + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn vv_linear_to_srgb(color: vec3<f32>) -> vec3<f32> {
    return pow(max(color, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
}

fn vv_shadow(shadow_pos: vec3<f32>, ndotl: f32) -> f32 {
    if (shadow_pos.z > 1.0 || shadow_pos.x < 0.0 || shadow_pos.x > 1.0 || shadow_pos.y < 0.0 || shadow_pos.y > 1.0) {
        return 1.0;
    }
    let bias = max(0.0005 * (1.0 - ndotl), 0.0001);
    return textureSampleCompare(t_shadow, s_shadow, shadow_pos.xy, shadow_pos.z - bias);
}

fn vv_apply_curvature_fog(color: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let dist = distance(global.camera_pos.xyz, world_pos);
    let fog_density = global.sun_dir.w;
    let fog_factor = 1.0 - exp(-(dist * fog_density) * (dist * fog_density * 0.5));
    let fog_color = mix(vec3<f32>(0.25, 0.46, 0.86), vec3<f32>(0.72, 0.82, 1.0), 0.25);
    return mix(color, fog_color, clamp(fog_factor, 0.0, 1.0));
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let layer = in.packed_tex_index & MATERIAL_INDEX_MASK;
    let quality_bits = u32(global.camera_pos.w);
    let color_only = (quality_bits & 8u) != 0u;

    var albedo: vec3<f32>;
    var roughness: f32;
    if (layer == PROP_VERTEX_COLOR_ONLY) {
        albedo = in.color;
        roughness = 0.78;
    } else if (color_only) {
        albedo = material_colors[layer].rgb * in.color;
        roughness = 0.72;
    } else {
        albedo = textureSample(t_albedo, s_material, in.uv, i32(layer)).rgb * in.color;
        roughness = textureSample(t_roughness, s_material, in.uv, i32(layer)).r;
    }

    let normal = normalize(in.world_normal);
    let sun_dir = normalize(global.sun_dir.xyz);
    let ndotl = max(dot(normal, sun_dir) * 0.82 + 0.18, 0.0);
    let shadow = mix(0.38, 1.0, vv_shadow(in.shadow_pos, ndotl));
    let sun = vec3<f32>(1.25, 1.12, 0.82) * ndotl * shadow * mix(1.05, 0.62, roughness);
    let up_dot = dot(normal, normalize(in.world_pos)) * 0.5 + 0.5;
    let ambient = mix(vec3<f32>(0.11, 0.09, 0.06), vec3<f32>(0.28, 0.48, 0.86), up_dot) * mix(0.92, 1.22, roughness);

    var color = albedo * (sun + ambient);
    color = vv_apply_curvature_fog(color, in.world_pos);
    color = vv_linear_to_srgb(vv_tonemap_aces(color));
    return vec4<f32>(color, 1.0);
}

