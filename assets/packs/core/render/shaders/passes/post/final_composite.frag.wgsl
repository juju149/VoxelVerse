#include "include/camera/globals.wgsl"
#include "include/math/color_space.wgsl"

@group(1) @binding(0) var t_scene: texture_2d<f32>;
@group(1) @binding(1) var s_scene: sampler;

struct FullscreenVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

fn vv_scene_resolution() -> vec2<f32> {
    let dims = textureDimensions(t_scene);
    return max(vec2<f32>(f32(dims.x), f32(dims.y)), vec2<f32>(1.0));
}

fn vv_sample_scene(uv: vec2<f32>) -> vec3<f32> {
    return textureSample(t_scene, s_scene, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))).rgb;
}

// Lightweight FXAA-inspired resolve.
// It avoids the old full-screen blur look and only softens strong edges.
fn vv_fxaa_resolve(uv: vec2<f32>, px: vec2<f32>) -> vec3<f32> {
    let c = vv_sample_scene(uv);

    let n = vv_sample_scene(uv + vec2<f32>(0.0, -px.y));
    let s = vv_sample_scene(uv + vec2<f32>(0.0,  px.y));
    let e = vv_sample_scene(uv + vec2<f32>( px.x, 0.0));
    let w = vv_sample_scene(uv + vec2<f32>(-px.x, 0.0));

    let lc = vv_luminance(c);
    let ln = vv_luminance(n);
    let ls = vv_luminance(s);
    let le = vv_luminance(e);
    let lw = vv_luminance(w);

    let l_min = min(lc, min(min(ln, ls), min(le, lw)));
    let l_max = max(lc, max(max(ln, ls), max(le, lw)));
    let contrast = l_max - l_min;

    if contrast < 0.045 {
        return c;
    }

    let grad_h = abs(lw - le);
    let grad_v = abs(ln - ls);

    let horizontal_edge = grad_h > grad_v;

    let edge_blend = clamp(contrast * 2.75, 0.0, 0.55);

    let along_edge = select(
        (e + w) * 0.5,
        (n + s) * 0.5,
        horizontal_edge
    );

    return mix(c, along_edge, edge_blend);
}

// Cheap local bloom. Not a full bloom chain, but gives sunlight and bright fog
// a soft cinematic lift until the dedicated bloom pass is wired.
fn vv_cheap_bloom(uv: vec2<f32>, px: vec2<f32>) -> vec3<f32> {
    let r1 = px * 1.5;
    let r2 = px * 3.5;

    var glow = vec3<f32>(0.0);

    glow += max(vv_sample_scene(uv + vec2<f32>( r1.x,  0.0)) - vec3<f32>(1.0), vec3<f32>(0.0));
    glow += max(vv_sample_scene(uv + vec2<f32>(-r1.x,  0.0)) - vec3<f32>(1.0), vec3<f32>(0.0));
    glow += max(vv_sample_scene(uv + vec2<f32>( 0.0,  r1.y)) - vec3<f32>(1.0), vec3<f32>(0.0));
    glow += max(vv_sample_scene(uv + vec2<f32>( 0.0, -r1.y)) - vec3<f32>(1.0), vec3<f32>(0.0));

    glow += max(vv_sample_scene(uv + vec2<f32>( r2.x,  r2.y)) - vec3<f32>(1.0), vec3<f32>(0.0));
    glow += max(vv_sample_scene(uv + vec2<f32>(-r2.x,  r2.y)) - vec3<f32>(1.0), vec3<f32>(0.0));
    glow += max(vv_sample_scene(uv + vec2<f32>( r2.x, -r2.y)) - vec3<f32>(1.0), vec3<f32>(0.0));
    glow += max(vv_sample_scene(uv + vec2<f32>(-r2.x, -r2.y)) - vec3<f32>(1.0), vec3<f32>(0.0));

    return glow * 0.022;
}

@fragment
fn fs_main(in: FullscreenVertexOut) -> @location(0) vec4<f32> {
    let qbits = vv_quality_bits();
    let resolution = vv_scene_resolution();
    let px = 1.0 / resolution;

    var color = vv_sample_scene(in.uv);

    // Bit 64: FXAA.
    if (qbits & 64u) != 0u {
        color = vv_fxaa_resolve(in.uv, px);
    }

    // Bit 128: cheap bloom until the real bloom chain is connected.
    if (qbits & 128u) != 0u {
        color += vv_cheap_bloom(in.uv, px);
    }

    // Exposure comes from atmosphere_params.w.
    // Fallback prevents a black frame if the uniform is not initialized yet.
    let exposure = max(global.atmosphere_params.w, 0.85);

    color = vv_tonemap_agx(vv_safe_color(color), exposure);
    color = vv_cinematic_grade(color, in.uv);
    color = vv_srgb_encode(color);

    return vec4<f32>(color, 1.0);
}