#include "include/math/random.wgsl"

fn vv_face_variation(world_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    return vv_hash31(floor(world_pos + normal * 0.5));
}

