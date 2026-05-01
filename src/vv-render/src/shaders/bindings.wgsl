struct Atmosphere {
    sun_direction: vec4<f32>,
    sun_color: vec4<f32>,
    sky_color: vec4<f32>,
    ground_ambient_color: vec4<f32>,
    shadow_tint_color: vec4<f32>,
    fog_color_density: vec4<f32>,
    clear_color: vec4<f32>,

    // Phase 1 additions.
    // Keep as vec4 fields only to preserve clean uniform alignment.
    zenith_color: vec4<f32>,
    horizon_glow_color: vec4<f32>,
    moon_direction: vec4<f32>,
    moon_color: vec4<f32>,

    // x = exposure
    // y = saturation
    // z = contrast
    // w = reserved
    grading: vec4<f32>,

    // x = fog_start
    // y = sky_horizon_power
    // z = star_strength
    // w = night_amount
    sky_params: vec4<f32>,
}

struct Global {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    atmosphere: Atmosphere,
    inv_view_proj: mat4x4<f32>,
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

struct Local {
    model: mat4x4<f32>,
    params: vec4<f32>,
}

@group(0) @binding(0) var<uniform> global: Global;
@group(0) @binding(1) var t_shadow: texture_depth_2d;
@group(0) @binding(2) var s_shadow: sampler_comparison;
@group(0) @binding(3) var<storage, read> block_visuals: array<BlockVisual>;
@group(0) @binding(4) var<storage, read> block_visual_palette: array<vec4<f32>>;
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
}

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) shadow_pos: vec3<f32>,
    @location(4) uv: vec2<f32>,
    @location(5) @interpolate(flat) block_id: i32,
    @location(6) @interpolate(flat) block_visual_id: u32,
    @location(7) @interpolate(flat) face_id: u32,
    @location(8) @interpolate(flat) voxel_pos: vec3<i32>,
    @location(9) @interpolate(flat) variation_seed: u32,
    @location(10) ao: f32,
}

struct SkyOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,
}