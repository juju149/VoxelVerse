use vv_registry::{CompiledBlockRender, CompiledBlockShape};

use crate::MeshGen;

use super::SoftCubeParams;

impl MeshGen {
    pub(crate) fn soft_cube_params(render: &CompiledBlockRender) -> Option<SoftCubeParams> {
        if !matches!(render.shape, CompiledBlockShape::Cube) {
            return None;
        }

        let authored_radius = render.material.bevel;
        let authored_normal = render.material.normal_strength;

        // Pour l'instant, on active le soft cube dès que le block demande du relief.
        // Plus tard, on remplacera ça par un vrai CompiledBlockShapeProfile.
        if authored_radius <= 0.0001 && authored_normal <= 0.0001 {
            return None;
        }

        let radius = authored_radius.clamp(0.0, 0.18);

        Some(SoftCubeParams {
            radius,
            pillow: 0.0,
            // 3 segments per face = 9 quads. Anything higher made the subdivision
            // grid faintly visible across the face once lighting + directional
            // material tint sampled the per-vertex curved normals.
            segments: 3,
        })
    }
}
