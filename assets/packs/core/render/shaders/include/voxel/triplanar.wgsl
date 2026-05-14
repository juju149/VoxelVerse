fn vv_triplanar_grain(world_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    let f = 4.75;

    let gx = sin(world_pos.y * f + 0.30) * sin(world_pos.z * f + 0.70);
    let gy = sin(world_pos.x * f + 1.10) * sin(world_pos.z * f + 0.20);
    let gz = sin(world_pos.x * f + 0.80) * sin(world_pos.y * f + 0.50);

    let w = abs(normal);
    let sum_w = max(w.x + w.y + w.z, VV_EPSILON);
    let n = (gx * w.x + gy * w.y + gz * w.z) / sum_w;

    return n * 0.008;
}
