fn vv_foliage_tint(base: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let biome_warmth = 0.5 + 0.5 * sin(dot(normalize(world_pos), vec3<f32>(2.1, 3.7, 1.4)));
    let tint = mix(vec3<f32>(0.78, 0.94, 0.72), vec3<f32>(0.55, 0.78, 0.45), biome_warmth);
    return base * tint;
}

