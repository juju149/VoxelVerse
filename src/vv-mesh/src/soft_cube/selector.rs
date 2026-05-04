use vv_registry::CompiledBlockRender;

use crate::MeshGen;

use super::SoftCubeParams;

impl MeshGen {
    pub(crate) fn soft_cube_params(_render: &CompiledBlockRender) -> Option<SoftCubeParams> {
        // Terrain chunks stay sealed hard cubes.
        // Rounded look is shader-only to avoid gaps and keep performance stable.
        None
    }
}
