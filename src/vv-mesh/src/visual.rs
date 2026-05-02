use vv_registry::{CompiledBlockRender, CompiledBlockShape};

use crate::MeshGen;

#[derive(Clone, Copy, Debug)]
pub(crate) struct VisualBevel {
    pub edge_width: f32,
    pub top_edge: f32,
    pub side_edge: f32,
}

impl VisualBevel {
    #[inline]
    pub(crate) fn is_enabled(self) -> bool {
        self.edge_width > 0.0001
    }
}

impl MeshGen {
    pub(crate) fn visual_bevel(render: &CompiledBlockRender) -> VisualBevel {
        if !matches!(render.shape, CompiledBlockShape::Cube) {
            return VisualBevel {
                edge_width: 0.0,
                top_edge: 0.0,
                side_edge: 0.0,
            };
        }

        let authored_bevel = render.material.bevel.clamp(0.0, 0.18);
        let normal_strength = render.material.normal_strength.clamp(0.0, 1.0);

        if authored_bevel <= 0.001 && normal_strength <= 0.001 {
            return VisualBevel {
                edge_width: 0.0,
                top_edge: 0.0,
                side_edge: 0.0,
            };
        }

        let edge_width = if authored_bevel <= 0.001 {
            0.0
        } else {
            (authored_bevel * 1.80).clamp(0.015, 0.18)
        };

        // Strength is used as slerp t ∈ [0, 1]: recalibrated so typical blocks sit
        // at ~0.35 and high-bevel blocks can reach ~0.85 for a visually round edge.
        let normal_rounding = if edge_width > 0.0 {
            (0.10 + edge_width * 1.80 + normal_strength * 0.40).clamp(0.0, 0.90)
        } else {
            (normal_strength * 0.45).clamp(0.0, 0.50)
        };

        VisualBevel {
            edge_width,
            top_edge: normal_rounding,
            side_edge: normal_rounding * 0.76,
        }
    }
}
