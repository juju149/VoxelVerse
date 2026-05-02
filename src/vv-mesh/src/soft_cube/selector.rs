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

        let radius = authored_radius.max(0.10).clamp(0.04, 0.22);
        let pillow = (radius * 0.28).clamp(0.018, 0.055);

        Some(SoftCubeParams {
            radius,
            pillow,
            segments: 6,
        })
    }
}
