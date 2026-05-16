#include "include/camera/globals.wgsl"
#include "include/math/color_space.wgsl"

// Ghibli final grade.
// AgX tonemap → painterly grade → subtle vignette.
// Optional FXAA (qbit 64) and over-bright bloom lift (qbit 128).
// Single scene sample on the fast path, 5 taps when FXAA is on.

@group(1) @binding(0) var t_scene: texture_2d<f32>;
@group(1) @binding(1) var s_scene: sampler;

struct FullscreenVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

fn vv_sample_scene(uv: vec2<f32>) -> vec3<f32> {
    return textureSample(t_scene, s_scene, uv).rgb;
}

// Painterly day/night grade. Boost saturation slightly, lift midtones,
// keep highlights from blowing to neutral white.
fn vv_ghibli_grade(color: vec3<f32>) -> vec3<f32> {
    let sun_elev = normalize(global.sun_dir.xyz).y;
    let day   = clamp(sun_elev * 3.2 + 0.20, 0.0, 1.0);
    let night = clamp(-sun_elev * 4.0 - 0.06, 0.0, 1.0);

    // Mild s-curve around 0.5: gentle contrast that respects mid-tones.
    let contrast = mix(0.96, 1.06, day) * mix(1.0, 0.92, night);
    var c = (color - vec3<f32>(0.5)) * contrast + vec3<f32>(0.5);
    c = max(c, vec3<f32>(0.0));

    // Saturation lift: paintings live around 1.10.
    let sat = mix(1.04, 1.12, day) * mix(1.0, 0.78, night);
    c = vv_preserve_natural_saturation(c, sat);

    // Warm-cool split: cooler shadows, warmer highlights.
    let luma = dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
    let cool = vec3<f32>(0.96, 0.99, 1.04);
    let warm = vec3<f32>(1.03, 1.00, 0.96);
    let tint = mix(cool, warm, smoothstep(0.25, 0.85, luma));
    c *= mix(vec3<f32>(1.0), tint, 0.35 * day);

    // Night lift keeps things readable without going milky.
    let night_lift = vec3<f32>(0.010, 0.012, 0.020) * night;
    return clamp(c + night_lift, vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: FullscreenVertexOut) -> @location(0) vec4<f32> {
    let qbits = vv_quality_bits();
    let resolution = max(global.render_params.zw, vec2<f32>(1.0));
    let px = 1.0 / resolution;

    var color = vv_sample_scene(in.uv);

    // Cheap FXAA-style box blur on quality flag.
    if (qbits & 64u) != 0u {
        let l = vv_sample_scene(in.uv - vec2<f32>(px.x, 0.0));
        let r = vv_sample_scene(in.uv + vec2<f32>(px.x, 0.0));
        let u = vv_sample_scene(in.uv - vec2<f32>(0.0, px.y));
        let d = vv_sample_scene(in.uv + vec2<f32>(0.0, px.y));
        color = color * 0.60 + (l + r + u + d) * 0.10;
    }

    // Soft over-bright glow (no extra blur pass).
    if (qbits & 128u) != 0u {
        let bloom = max(color - vec3<f32>(1.0), vec3<f32>(0.0));
        color += bloom * 0.22;
    }

    // Painterly vignette.
    let v = smoothstep(0.95, 0.20, distance(in.uv, vec2<f32>(0.5)));
    color *= mix(0.90, 1.0, v);

    // Tonemap then grade. AgX gives clean highlight roll-off.
    color = vv_tonemap_agx(color, global.atmosphere_params.w * 0.80);
    color = vv_ghibli_grade(color);

    // Render target is sRGB; the swapchain encodes for us.
    return vec4<f32>(color, 1.0);
}
