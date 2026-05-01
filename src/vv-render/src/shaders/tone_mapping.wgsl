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
    let exposure = max(global.atmosphere.grading.x, 0.001);
    let saturation = max(global.atmosphere.grading.y, 0.0);
    let contrast = max(global.atmosphere.grading.z, 0.0);

    var graded = color * exposure;

    let luma = luminance(graded);
    graded = mix(vec3<f32>(luma), graded, saturation * 1.08);

    // Contrast around a low linear grey point, better before tone mapping than using 0.5.
    graded = (graded - vec3<f32>(0.18)) * contrast + vec3<f32>(0.18);

    return max(graded, vec3<f32>(0.0));
}

fn encode_final_color(linear_color: vec3<f32>) -> vec3<f32> {
    let graded = apply_color_grading(linear_color);
    let mapped = aces_approx(graded);
    return pow(max(mapped, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
}