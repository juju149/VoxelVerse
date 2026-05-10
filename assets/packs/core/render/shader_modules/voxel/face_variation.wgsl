fn vv_face_variation(world_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    let seed = dot(floor(world_pos + normal * 0.5), vec3<f32>(12.9898, 78.233, 37.719));
    return fract(sin(seed) * 43758.5453);
}

