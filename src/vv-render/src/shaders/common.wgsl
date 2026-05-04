fn saturate(x: f32) -> f32 {
    return clamp(x, 0.0, 1.0);
}

fn safe_normalize(v: vec3<f32>) -> vec3<f32> {
    return v / max(length(v), 1e-6);
}

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn safe_positive(value: f32, fallback: f32) -> f32 {
    if (value <= 0.0001) {
        return fallback;
    }
    return value;
}

fn hash13(p: vec3<f32>) -> f32 {
    let q = fract(p * vec3<f32>(0.1031, 0.11369, 0.13787));
    let r = q + dot(q, q.yzx + 19.19);
    return fract((r.x + r.y) * r.z);
}

fn hash11(x: f32) -> f32 {
    return fract(sin(x * 17.123) * 43758.5453123);
}

fn dither_opacity(pos: vec4<f32>, alpha: f32) -> bool {
    let dither_threshold = dot(vec2<f32>(171.0, 231.0), pos.xy);
    return fract(dither_threshold / 71.0) > alpha;
}

fn vv_saturate_color(color: vec3<f32>, amount: f32) -> vec3<f32> {
    let luma = luminance(color);
    return mix(vec3<f32>(luma), color, amount);
}

fn vv_toon_band(x: f32, softness: f32) -> f32 {
    return smoothstep(0.0, softness, x);
}

fn face_seed(
    voxel_pos: vec3<i32>,
    block_id: i32,
    block_visual_id: u32,
    face_id: u32,
    variation_seed: u32,
    cell: vec2<f32>,
    salt: f32,
) -> vec3<f32> {
    return vec3<f32>(
        f32(voxel_pos.x) * 11.7 + f32(block_id) * 0.37 + f32(variation_seed & 65535u) * 0.013 + cell.x * 1.97 + salt,
        f32(voxel_pos.y) * 7.3 + f32(face_id) * 3.11 + f32(variation_seed >> 16u) * 0.017 + cell.y * 2.41 + salt * 1.7,
        f32(voxel_pos.z) * 5.9 + f32(block_visual_id) * 0.53 + f32(face_id) * 0.19 + salt * 2.3,
    );
}