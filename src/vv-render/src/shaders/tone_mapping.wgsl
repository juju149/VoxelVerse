fn aces_approx(v: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;

    return clamp(
        (v * (a * v + b)) / (v * (c * v + d) + e),
        vec3<f32>(0.0),
        vec3<f32>(1.0),
    );
}

fn apply_color_grading(color: vec3<f32>) -> vec3<f32> {
    let exposure = default_one(global.atmosphere.grading.x);
    let saturation = default_one(global.atmosphere.grading.y);
    let contrast = default_one(global.atmosphere.grading.z);

    var graded = max(color * exposure, vec3<f32>(0.0));

    let luma = luminance(graded);
    let chroma = graded - vec3<f32>(luma);
    let vibrance_mask = saturate(1.0 - length(chroma) * 0.55);
    let final_saturation = saturation * mix(1.0, 1.16, vibrance_mask);

    graded = mix(vec3<f32>(luma), graded, final_saturation);
    graded = (graded - vec3<f32>(0.18)) * contrast + vec3<f32>(0.18);

    return max(graded, vec3<f32>(0.0));
}

fn encode_final_color(linear_color: vec3<f32>) -> vec3<f32> {
    let graded = apply_color_grading(linear_color);
    let mapped = aces_approx(graded);
    return pow(max(mapped, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
}