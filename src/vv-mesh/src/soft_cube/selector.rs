use vv_registry::CompiledBlockRender;

use crate::MeshGen;

use super::SoftCubeParams;

impl MeshGen {
    pub(crate) fn soft_cube_params(_render: &CompiledBlockRender) -> Option<SoftCubeParams> {
        // Disabled on terrain chunks.
        //
        // Real geometric rounded cubes create physical holes between adjacent voxels:
        // every corner retracts inward, so the clear color becomes visible between blocks.
        // They also multiply the mesh cost by segments² per face, which tanks FPS.
        //
        // VoxelVerse now uses sealed hard voxel geometry + shader fake bevel.
        // This keeps worlds watertight, square, fast, and still visually toy/cartoon.
        None
    }
}
