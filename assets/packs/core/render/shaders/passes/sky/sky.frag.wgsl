#include "include/camera/globals.wgsl"
#include "include/atmosphere/sky.wgsl"

// Ghibli sky pass: painterly gradient + soft sun & moon discs.
// No expensive scattering loops, just clean shaping.

struct FullscreenVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Soft circular glow projected from a world direction.
// Returns (disc, halo) intensities; the caller tints them.
fn vv_celestial_glow(
    uv: vec2<f32>,
    dir: vec3<f32>,
    radius: f32,
    halo: f32,
) -> vec2<f32> {
    let pixel_ndc = uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0);
    let world = global.camera_pos.xyz + dir * 9000.0;
    let clip = global.view_proj * vec4<f32>(world, 1.0);

    if clip.w <= 0.001 || clip.z < 0.0 {
        return vec2<f32>(0.0);
    }

    let ndc = clip.xy / clip.w;
    let d = distance(pixel_ndc, ndc);

    // Disc with a slightly soft edge.
    let disc = 1.0 - smoothstep(radius * 0.85, radius, d);
    // Painterly halo: exponential falloff.
    let glow = exp(-d * halo);
    return vec2<f32>(disc, glow);
}

@fragment
fn fs_main(in: FullscreenVertexOut) -> @location(0) vec4<f32> {
    let sun_dir = normalize(global.sun_dir.xyz);
    let sun_elev = sun_dir.y;

    var sky = vv_sky_color(in.uv);

    // --- Sun: warm at low angle, near-white at noon. -------------------------
    let sun_warm  = vec3<f32>(1.30, 0.55, 0.18);
    let sun_noon  = vec3<f32>(1.35, 1.28, 1.10);
    let sun_col = mix(sun_warm, sun_noon, vv_sky_saturate(sun_elev * 2.4 + 0.18));

    let sun_g = vv_celestial_glow(in.uv, sun_dir, 0.028, 14.0);
    sky += sun_col * (sun_g.x * 1.0 + sun_g.y * 0.55) * global.sky_zenith.w;

    // --- Moon: cool, soft, only visible after dusk. --------------------------
    let moon_dir = normalize(vec3<f32>(-sun_dir.x, -sun_dir.y + 0.06, -sun_dir.z));
    let moon_vis = vv_sky_saturate(-sun_elev * 5.0 - 0.10);
    let moon_col = vec3<f32>(0.85, 0.90, 1.05);
    let moon_g = vv_celestial_glow(in.uv, moon_dir, 0.020, 26.0);
    sky += moon_col * (moon_g.x * 0.55 + moon_g.y * 0.18) * moon_vis;

    return vec4<f32>(max(sky, vec3<f32>(0.0)), 1.0);
}
