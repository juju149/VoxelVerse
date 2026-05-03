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
    graded = mix(vec3<f32>(luma), graded, saturation);

    let pivot = vec3<f32>(0.20);
    graded = max((graded - pivot) * contrast + pivot, vec3<f32>(0.0));

    return graded;
}

fn encode_final_color(linear_color: vec3<f32>) -> vec3<f32> {
    let graded = apply_color_grading(linear_color);
    let mapped = aces_approx(graded);
    return pow(max(mapped, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
}
