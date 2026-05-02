use vv_core::BlockId;
use vv_registry::BlockId as ContentBlockId;

use crate::MeshGen;

impl MeshGen {
    pub fn stable_variation_seed(
        voxel: BlockId,
        block: ContentBlockId,
        face_id: u32,
        planet_seed: u32,
    ) -> u32 {
        let mut hash = 0x811c_9dc5u32 ^ planet_seed;

        for value in [
            voxel.face as u32,
            voxel.u,
            voxel.v,
            voxel.layer,
            block.raw(),
            face_id,
        ] {
            hash ^= value.wrapping_mul(0x9e37_79b9);
            hash = hash.rotate_left(13).wrapping_mul(0x85eb_ca6b);
        }

        hash ^ (hash >> 16)
    }
}
