mod chunk;
mod debug_mesh;
mod lod;
mod overlay;
mod primitive_mesh;
mod primitives;
mod shape;
mod vertex;
mod visual;
mod voxel;

pub use vertex::Vertex;

pub struct MeshGen;

#[cfg(test)]
mod tests {
    use super::*;
    use vv_compiler::compile_assets_root;
    use vv_config::WorldGenConfig;
    use vv_core::ChunkKey;
    use vv_world_gen::PlanetTerrain;
    use vv_world_runtime::PlanetData;

    #[test]
    fn chunk_mesh_uses_registry_block_render_color() {
        let geometry = vv_planet::PlanetGeometry::with_resolution(8.0, 0.5, 8);
        let assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let content = compile_assets_root(&assets).expect("core content should compile");
        let block_content = content.to_block_content();
        let terrain = PlanetTerrain::generate_for_geometry(
            geometry,
            &WorldGenConfig::default(),
            &content.worldgen_content(),
        )
        .expect("terrain should generate");
        let planet = PlanetData::new(geometry, terrain, 0);

        let (verts, _) = MeshGen::build_chunk(
            ChunkKey {
                face: 0,
                u_idx: 0,
                v_idx: 0,
            },
            &planet,
            &block_content,
        );

        let vertex = verts
            .iter()
            .find(|vertex| vertex.block_id >= 0)
            .expect("generated chunk should contain block vertices");

        assert!(vertex.ao > 0.0);
        assert_ne!(vertex.variation_seed, 0);
        assert!(
            verts.iter().any(|vertex| vertex.block_visual_id > 0),
            "generated chunk should contain block visual ids"
        );
    }
}
