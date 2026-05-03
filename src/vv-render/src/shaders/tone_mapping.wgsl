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

    let luma_before = luminance(graded);
    let chroma = graded - vec3<f32>(luma_before);
    let vibrance_mask = saturate(1.0 - length(chroma) * 0.62);
    let final_saturation = saturation * mix(1.0, 1.10, vibrance_mask);

    graded = mix(vec3<f32>(luma_before), graded, final_saturation);

    let pivot = vec3<f32>(0.20);
    graded = max((graded - pivot) * contrast + pivot, vec3<f32>(0.0));

    let luma_after = luminance(graded);
    let shadow_mask = 1.0 - smoothstep(0.045, 0.320, luma_after);
    let highlight_mask = smoothstep(0.420, 1.650, luma_after);

    let shadow_tone = vec3<f32>(0.94, 0.98, 1.08);
    let highlight_tone = vec3<f32>(1.055, 1.020, 0.955);

    graded = mix(graded, graded * shadow_tone, shadow_mask * 0.10);
    graded = mix(graded, graded * highlight_tone, highlight_mask * 0.06);

    return max(graded, vec3<f32>(0.0));
}

fn encode_final_color(linear_color: vec3<f32>) -> vec3<f32> {
    let graded = apply_color_grading(linear_color);
    let mapped = aces_approx(graded);
    return pow(max(mapped, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
}