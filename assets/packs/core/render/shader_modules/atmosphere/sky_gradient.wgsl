// VoxelVerse — Atmospheric Sky
//
// Renders a physically-inspired sky gradient with sun disc and corona.
// Called as a fullscreen triangle (no vertex buffer); receives UV from
// the shared final_composite vertex stage.
//
// GlobalUniform layout (must match renderer.rs / GlobalUniform):
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

struct SkyFragmentIn {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0)       uv:       vec2<f32>,
}

// Smooth quintic curve: slower at ends, faster in middle.
fn smoothstep5(t: f32) -> f32 {
    let c = clamp(t, 0.0, 1.0);
    return c * c * c * (c * (c * 6.0 - 15.0) + 10.0);
}

@fragment
fn fs_main(in: SkyFragmentIn) -> @location(0) vec4<f32> {
    let sun_dir      = normalize(global.sun_dir.xyz);
    let sun_intensity = global.sky_zenith.w;
    let horizon_col  = global.sky_horizon.rgb;
    let zenith_col   = global.sky_zenith.rgb;

    // UV from the vertex stage:
    //   uv.y = 0  → top of screen  (zenith)
    //   uv.y = 1  → bottom (horizon / below horizon)
    // Use a power curve so the gradient is richer near zenith.
    let zenith_t = pow(clamp(1.0 - in.uv.y, 0.0, 1.0), 0.55);
    var sky = mix(horizon_col, zenith_col, zenith_t);

    // ── Sun disc + corona ────────────────────────────────────────────────
    // Project a point far in the sun direction through the camera VP matrix
    // to get its screen-space NDC position.
    let sun_world = global.camera_pos.xyz + sun_dir * 9000.0;
    let sun_clip  = global.view_proj * vec4<f32>(sun_world, 1.0);

    // sun_clip.w > 0 means the sun is in front of the near plane.
    // sun_clip.z > 0 means it hasn't been clipped behind the camera.
    if sun_clip.w > 0.001 && sun_clip.z >= 0.0 {
        let sun_ndc   = sun_clip.xy / sun_clip.w;
        // Convert current pixel UV → NDC (y-flipped: NDC y+ is up, UV y+ is down)
        let pixel_ndc = in.uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0);
        let to_sun    = pixel_ndc - sun_ndc;
        let dist_sq   = dot(to_sun, to_sun);

        // Wide atmospheric glow — visible over a large area around the sun.
        let wide_glow  = exp(-dist_sq * 0.9) * 0.50 * sun_intensity;
        // Tight inner corona
        let inner_glow = exp(-dist_sq * 12.0) * 0.65 * sun_intensity;

        let warm = vec3<f32>(1.10, 0.80, 0.42); // warm orange-gold
        sky = mix(sky, warm, clamp(wide_glow,  0.0, 0.75));
        sky += warm * clamp(inner_glow, 0.0, 0.50);

        // Hard sun disc (very bright centre)
        let disc = 1.0 - smoothstep(0.0005, 0.0025, dist_sq);
        sky = mix(sky, vec3<f32>(1.12, 1.06, 0.95), disc * sun_intensity);
    }

    // ── Horizon haze brightening ─────────────────────────────────────────
    // Slightly brighter band right at the horizon, mimicking Rayleigh scatter.
    let haze_t = pow(clamp((in.uv.y - 0.78) * 4.55, 0.0, 1.0), 2.5) * 0.18;
    sky = mix(sky, horizon_col * 1.18, haze_t);

    // ── sRGB gamma encode ─────────────────────────────────────────────────
    // Sky colors are specified in linear space in the uniform; encode here so
    // the output matches the sRGB swapchain format.
    let out_linear = max(sky, vec3<f32>(0.0));
    let out_srgb   = pow(out_linear, vec3<f32>(1.0 / 2.2));
    return vec4<f32>(out_srgb, 1.0);
}

