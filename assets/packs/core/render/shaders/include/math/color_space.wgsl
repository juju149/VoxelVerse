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

fn vv_tonemap_agx(color: vec3<f32>, exposure: f32) -> vec3<f32> {
    let agx_mat = mat3x3<f32>(
        vec3<f32>(0.842479, 0.042328, 0.042376),
        vec3<f32>(0.078434, 0.878469, 0.079166),
        vec3<f32>(0.079224, 0.079116, 0.879143),
    );

    let min_ev = -12.47393;
    let max_ev = 4.026069;

    var v = agx_mat * max(color * exposure, vec3<f32>(1e-10));
    v = clamp((log2(v) - min_ev) / (max_ev - min_ev), vec3<f32>(0.0), vec3<f32>(1.0));

    return clamp(vv_agx_contrast(v), vec3<f32>(0.0), vec3<f32>(1.0));
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

fn vv_preserve_natural_saturation(color: vec3<f32>, amount: f32) -> vec3<f32> {
    let luma = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
    return mix(vec3<f32>(luma), color, amount);
}
