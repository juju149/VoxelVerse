struct Global {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    sun_dir: vec4<f32>,
    sky_horizon: vec4<f32>,
    sky_zenith: vec4<f32>,
    render_params: vec4<f32>,
    atmosphere_params: vec4<f32>,
    cloud_params: vec4<f32>,
    water_params: vec4<f32>,
}

@group(0) @binding(0) var<uniform> global: Global;

fn vv_quality_bits() -> u32 {
    return u32(global.render_params.y);
}

