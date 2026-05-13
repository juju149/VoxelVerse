fn vv_face_variation(world_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    let p = floor(world_pos + normal * 0.5);
    return vv_hash31(p);
}

fn vv_face_soft_noise(world_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    let p = world_pos * 0.18 + normal * 3.7;
    let a = vv_hash31(floor(p));
    let b = vv_hash31(floor(p + vec3<f32>(3.1, 7.7, 1.9)));
    return mix(a, b, 0.5);
}
