// VoxelVerse — Water surface shader
//
// Features:
//   • Schlick Fresnel — transparent shallow, reflective at grazing angles
//   • Sun specular highlight (Blinn-Phong)
//   • Sky horizon reflection color matched to atmospheric uniform
//   • Curvature fog consistent with terrain shader
//   • Subtle sin-wave normal perturbation for animated-look water
//
// GlobalUniform layout (192 bytes — must match renderer.rs / GlobalUniform):
//   view_proj        mat4  (bytes   0–63)
//   light_view_proj  mat4  (bytes  64–127)
//   camera_pos       vec4  xyz=cam_pos,  w=quality_bits  (bytes 128–143)
//   sun_dir          vec4  xyz=sun_dir,  w=fog_density   (bytes 144–159)
//   sky_horizon      vec4  xyz=horizon,  w=time_of_day   (bytes 160–175)
//   sky_zenith       vec4  xyz=zenith,   w=sun_intensity (bytes 176–191)

struct Global {
    view_proj:       mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    camera_pos:      vec4<f32>,
    sun_dir:         vec4<f32>,
    sky_horizon:     vec4<f32>,
    sky_zenith:      vec4<f32>,
}

@group(0) @binding(0) var<uniform> global: Global;

struct WaterFragmentIn {
    @builtin(position) clip_pos:    vec4<f32>,
    @location(0)       uv:          vec2<f32>,
    @location(1)       world_normal: vec3<f32>,
    @location(2)       world_pos:   vec3<f32>,
}

// Schlick Fresnel approximation
fn schlick_fresnel(cos_theta: f32, f0: f32) -> f32 {
    let c = 1.0 - clamp(cos_theta, 0.0, 1.0);
    return f0 + (1.0 - f0) * (c * c * c * c * c);
}

@fragment
fn fs_main(in: WaterFragmentIn) -> @location(0) vec4<f32> {
    let sun_dir      = normalize(global.sun_dir.xyz);
    let sun_intensity = global.sky_zenith.w;

    // Perturb the geometric normal with sin-wave to simulate gentle waves.
    // Two overlapping waves at different frequencies for visual richness.
    let wx  = in.world_pos.x;
    let wz  = in.world_pos.z;
    let w1  = sin(wx * 0.10 + wz * 0.07) * 0.14;
    let w2  = sin(wx * 0.17 - wz * 0.11) * 0.10;
    let base_normal  = normalize(in.world_normal);
    let perturbed    = normalize(base_normal + vec3<f32>(w1, 0.0, w2));

    let view_dir  = normalize(global.camera_pos.xyz - in.world_pos);
    let cos_theta = max(dot(view_dir, perturbed), 0.0);
    let fresnel   = schlick_fresnel(cos_theta, 0.04);

    // Sun specular (Blinn-Phong, sharp on water)
    let half_dir  = normalize(sun_dir + view_dir);
    let spec_raw  = max(dot(perturbed, half_dir), 0.0);
    let specular  = pow(spec_raw, 80.0) * 0.90 * sun_intensity;

    // Water body color: deep teal at normal incidence, lighter at shallow angle
    let deep_col    = vec3<f32>(0.03, 0.13, 0.22);
    let shallow_col = vec3<f32>(0.15, 0.50, 0.70);
    let depth_blend = 0.25 + (1.0 - cos_theta) * 0.35;
    let water_body  = mix(deep_col, shallow_col, clamp(depth_blend, 0.0, 1.0));

    // Reflection: sky horizon color (matched to the atmospheric sky)
    let reflect_col = global.sky_horizon.rgb;

    // Combine body + Fresnel reflection + specular
    var color  = mix(water_body, reflect_col, fresnel * 0.62);
    color     += vec3<f32>(1.10, 1.04, 0.95) * specular;

    // Fog: same density formula as the terrain shader
    let dist        = distance(global.camera_pos.xyz, in.world_pos);
    let fog_density = global.sun_dir.w;
    let fog_factor  = 1.0 - exp(-(dist * fog_density) * (dist * fog_density * 0.5));
    color = mix(color, global.sky_horizon.rgb, clamp(fog_factor, 0.0, 1.0));

    // sRGB encode (water is rendered without post-process pass)
    color = pow(max(color, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));

    // Alpha: opaque at grazing angles (Fresnel-like), translucent head-on
    let alpha = mix(0.60, 0.96, fresnel);
    return vec4<f32>(color, alpha);
}

