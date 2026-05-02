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
            (authored_bevel * 2.45).clamp(0.012, 0.075)
        };

        let normal_rounding = if edge_width > 0.0 {
            (0.20 + edge_width * 3.25 + normal_strength * 0.34).clamp(0.0, 0.72)
        } else {
            (normal_strength * 0.36).clamp(0.0, 0.42)
        };

        VisualBevel {
            edge_width,
            top_edge: normal_rounding,
            side_edge: normal_rounding * 0.76,
        }
    }
}
