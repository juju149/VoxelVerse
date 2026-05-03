// The legacy per-detail-layer pattern code (rings, ridges, specks,
// square-ring circles, …) used to live here and was the most likely
// source of the visible "dot grid" artifact on cube faces. It was only
// ever called from the standalone viewer; the in-game renderer never
// used it, so the pattern code has been deleted.
//
// `face_seed` survives because block_albedo.wgsl still calls it through
// vv_program_seed.

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
