fn vv_luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn vv_safe_color(c: vec3<f32>) -> vec3<f32> {
    return max(c, vec3<f32>(0.0));
}

fn vv_srgb_encode(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) * 1.055 - vec3<f32>(0.055);
    return select(lo, hi, c > vec3<f32>(0.0031308));
}

fn vv_srgb_decode(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((max(c, vec3<f32>(0.0)) + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(lo, hi, c > vec3<f32>(0.04045));
}

// AgX inspired filmic tonemapper.
// Stable greens, soft highlights, no radioactive saturation.
fn vv_agx_contrast(x: vec3<f32>) -> vec3<f32> {
    let x2 = x * x;
    let x4 = x2 * x2;

    return 15.5 * x4 * x2
        - 40.14 * x4 * x
        + 31.96 * x4
        - 6.868 * x2 * x
        + 0.4298 * x2
        + 0.1191 * x
        - 0.00232;
}

fn vv_tonemap_agx(c: vec3<f32>, exposure: f32) -> vec3<f32> {
    let agx_inset = mat3x3<f32>(
        vec3<f32>(0.842479, 0.042328, 0.042376),
        vec3<f32>(0.078434, 0.878469, 0.079166),
        vec3<f32>(0.079224, 0.079116, 0.879143),
    );

    let min_ev = -12.47393;
    let max_ev = 4.026069;

    var v = agx_inset * max(c * exposure, vec3<f32>(1e-10));
    v = clamp((log2(v) - min_ev) / (max_ev - min_ev), vec3<f32>(0.0), vec3<f32>(1.0));

    return clamp(vv_agx_contrast(v), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn vv_apply_contrast(c: vec3<f32>, contrast: f32) -> vec3<f32> {
    return clamp((c - vec3<f32>(0.5)) * contrast + vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn vv_apply_saturation(c: vec3<f32>, saturation: f32) -> vec3<f32> {
    let luma = vv_luminance(c);
    return mix(vec3<f32>(luma), c, saturation);
}

fn vv_cinematic_grade(c: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
    var color = c;

    // Gentle film contrast, enough to give depth without crushing shadows.
    color = vv_apply_contrast(color, 1.055);

    // Premium voxel look: slightly calm saturation, not candy plastic.
    color = vv_apply_saturation(color, 0.94);

    // Subtle split tone: cool shadows, warm highlights.
    let luma = vv_luminance(color);
    let shadows = vec3<f32>(0.965, 0.982, 1.020);
    let highlights = vec3<f32>(1.025, 1.010, 0.970);
    color *= mix(shadows, highlights, smoothstep(0.18, 0.88, luma));

    // Soft toe, avoids gray-black soup in foggy scenes.
    color = max(color - vec3<f32>(0.006), vec3<f32>(0.0)) / vec3<f32>(0.994);

    // Cinematic vignette, almost invisible, just frames the eye.
    let d = distance(uv, vec2<f32>(0.5));
    let vignette = smoothstep(0.92, 0.24, d);
    color *= mix(0.955, 1.0, vignette);

    return clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));
}