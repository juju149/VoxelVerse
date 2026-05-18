pub mod chunk_input;
pub(crate) mod face_culling;
pub(crate) mod face_emitter;
pub(crate) mod lighting;
pub mod material_packing;
pub mod prop_integration;

use std::sync::OnceLock;

use crate::cpu_mesh::CpuMesh;
use crate::VoxelMeshingConfig;
use chunk_input::ChunkMeshInput;
use face_culling::{build_candidates, VoxelAccessor};
use face_emitter::{emit_cube_voxel, FaceEmitter, QuadFace};
use material_packing::VoxelMeshKind;
use prop_integration::bake_prop_instances;

use vv_math::SphericalGrid;
use vv_voxel::CHUNK_SIZE;

use crate::greedy_mesher::GreedyMesher;

pub struct MeshGen;

impl MeshGen {
    /// Whether to bake prop geometry for the given chunk based on its
    /// distance to the player.  Chunks on other faces are always skipped.
    pub fn should_bake_props_for_chunk(
        key: vv_voxel::SurfaceChunkKey,
        player_key: Option<vv_voxel::SurfaceChunkKey>,
        config: VoxelMeshingConfig,
    ) -> bool {
        let Some(player_key) = player_key else {
            return true;
        };
        if key.face != player_key.face {
            return false;
        }
        key.u_idx.abs_diff(player_key.u_idx) <= config.prop_lod_chunk_radius
            && key.v_idx.abs_diff(player_key.v_idx) <= config.prop_lod_chunk_radius
    }

    /// Build the CPU mesh for one voxel chunk.
    ///
    /// All world knowledge was already baked into `input` by the caller.
    /// This function is pure geometry — no world, no worldgen, no registry.
    pub fn build_chunk(input: &ChunkMeshInput, config: VoxelMeshingConfig) -> CpuMesh {
        let key = input.key;
        let resolution = input.voxels.resolution;

        let u_start = key.u_idx * CHUNK_SIZE;
        let v_start = key.v_idx * CHUNK_SIZE;
        let u_end = (u_start + CHUNK_SIZE).min(resolution);
        let v_end = (v_start + CHUNK_SIZE).min(resolution);

        let accessor =
            VoxelAccessor::new(&input.voxels, &input.material_table, &input.border_samples);

        let mut candidate_buf = build_candidates(
            &accessor,
            key.face,
            u_start,
            v_start,
            config.cliff_fill_depth,
            resolution,
        );

        // Modified voxels inserted by the world layer are already in the view;
        // add their 6-connected neighbors as candidates so faces are correct.
        for (layer, u, v, _) in input.voxels.iter_voxels() {
            if u < u_start.saturating_sub(1)
                || u > u_end
                || v < v_start.saturating_sub(1)
                || v > v_end
            {
                continue;
            }
            face_culling::add_modified_candidates(
                key.face,
                layer,
                u,
                v,
                &mut candidate_buf,
                resolution,
            );
        }

        let candidates = candidate_buf.finish();

        let mut verts = Vec::with_capacity((CHUNK_SIZE * CHUNK_SIZE * 4) as usize);
        let mut inds = Vec::with_capacity((CHUNK_SIZE * CHUNK_SIZE * 6) as usize);
        let mut idx = 0u32;

        // Pull the profile for geometry generation.
        let profile = input.border_samples.profile;
        let greedy_enabled = greedy_meshing_enabled();

        if greedy_enabled {
            GreedyMesher::append_opaque_cubes(
                &accessor,
                &candidates,
                profile,
                &mut verts,
                &mut inds,
                &mut idx,
            );
        }

        for c in &candidates {
            if c.u < u_start || c.u >= u_end || c.v < v_start || c.v >= v_end {
                continue;
            }
            if !accessor.has_renderable(c.layer, c.u, c.v) {
                continue;
            }
            let voxel_id = accessor.voxel_id(c.layer, c.u, c.v);
            let kind = input.material_table.mesh_kind(voxel_id);
            if greedy_enabled && input.material_table.uses_greedy(voxel_id) {
                continue;
            }
            match kind {
                VoxelMeshKind::Cube | VoxelMeshKind::CubeColumn => {
                    emit_cube_voxel(
                        c.face, c.layer, c.u, c.v, profile, &accessor, &mut verts, &mut inds,
                        &mut idx,
                    );
                }
                VoxelMeshKind::None => {
                    debug_assert!(false, "mesher visited a None-mesh block");
                }
            }
        }

        let mut mesh = CpuMesh::new(verts, inds);

        if !input.prop_instances.is_empty() {
            let grid = SphericalGrid::new(
                profile.resolution,
                profile.inner_radius,
                profile.layer_height,
            );
            bake_prop_instances(&input.prop_instances, grid, &mut mesh, config);
        }

        mesh
    }

    pub(crate) fn quad_tiled(
        verts: &mut Vec<crate::cpu_mesh::CpuVertex>,
        inds: &mut Vec<u32>,
        idx: &mut u32,
        face: QuadFace,
        uv_span: [f32; 2],
    ) {
        FaceEmitter::quad_tiled(verts, inds, idx, face, uv_span);
    }
}

fn greedy_meshing_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        !std::env::var("VV_DISABLE_GREEDY_MESH")
            .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    })
}

#[cfg(test)]
mod tests {
    use super::{MeshGen, VoxelMeshingConfig};
    use vv_voxel::SurfaceChunkKey;

    fn chunk(face: u8, u_idx: u32, v_idx: u32) -> SurfaceChunkKey {
        SurfaceChunkKey { face, u_idx, v_idx }
    }

    #[test]
    fn prop_lod_keeps_only_near_same_face_chunks() {
        let player = Some(chunk(2, 10, 10));
        let config = VoxelMeshingConfig::default();
        assert!(MeshGen::should_bake_props_for_chunk(
            chunk(2, 15, 10),
            player,
            config
        ));
        assert!(!MeshGen::should_bake_props_for_chunk(
            chunk(2, 16, 10),
            player,
            config
        ));
        assert!(!MeshGen::should_bake_props_for_chunk(
            chunk(3, 10, 10),
            player,
            config
        ));
    }

    #[test]
    fn prop_lod_defaults_to_baking_when_player_key_is_unknown() {
        assert!(MeshGen::should_bake_props_for_chunk(
            chunk(5, 80, 80),
            None,
            VoxelMeshingConfig::default()
        ));
    }

    #[test]
    fn prop_lod_uses_configured_chunk_radius() {
        let config = VoxelMeshingConfig {
            prop_lod_chunk_radius: 2,
            ..VoxelMeshingConfig::default()
        };
        let player = Some(chunk(1, 10, 10));
        assert!(MeshGen::should_bake_props_for_chunk(
            chunk(1, 12, 10),
            player,
            config
        ));
        assert!(!MeshGen::should_bake_props_for_chunk(
            chunk(1, 13, 10),
            player,
            config
        ));
    }
}
