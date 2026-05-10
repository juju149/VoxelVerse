// GlobalUniform layout (must match renderer.rs / GlobalUniform — 192 bytes):
//   view_proj        mat4   (bytes   0–63)
//   light_view_proj  mat4   (bytes  64–127)
//   camera_pos       vec4   xyz=cam_pos,  w=quality_bits   (bytes 128–143)
//   sun_dir          vec4   xyz=sun_dir,  w=fog_density    (bytes 144–159)
//   sky_horizon      vec4   xyz=horizon,  w=time_of_day    (bytes 160–175)
//   sky_zenith       vec4   xyz=zenith,   w=sun_intensity  (bytes 176–191)
struct Global {
    view_proj:       mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    camera_pos:      vec4<f32>,
    sun_dir:         vec4<f32>,
    sky_horizon:     vec4<f32>,
    sky_zenith:      vec4<f32>,
}

@group(0) @binding(0) var<uniform> global: Global;

struct Local {
    model: mat4x4<f32>,
    params: vec4<f32>,
}

@group(1) @binding(0) var<uniform> local: Local;

struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) color: vec3<f32>,
    @location(4) tex_index: u32,
}

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

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    let world_pos = local.model * vec4<f32>(in.pos, 1.0);
    let normal_mat = mat3x3<f32>(local.model[0].xyz, local.model[1].xyz, local.model[2].xyz);
    let world_normal = normalize(normal_mat * in.normal);
    let light_clip = global.light_view_proj * vec4<f32>(world_pos.xyz + world_normal * 0.05, 1.0);

    out.clip_pos = global.view_proj * world_pos;
    out.uv = in.uv;
    out.world_normal = world_normal;
    out.world_pos = world_pos.xyz;
    out.view_pos = global.camera_pos.xyz;
    out.shadow_pos = vec3<f32>(light_clip.x * 0.5 + 0.5, -light_clip.y * 0.5 + 0.5, light_clip.z);
    out.color = in.color;
    out.packed_tex_index = in.tex_index;
    return out;
}

