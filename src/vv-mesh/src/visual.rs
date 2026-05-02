use vv_registry::CompiledBlockRender;

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
        self.edge_width > 0.0
    }
}

impl MeshGen {
    pub(crate) fn visual_bevel(render: &CompiledBlockRender) -> VisualBevel {
        let authored_bevel = render.material.bevel.clamp(0.0, 0.18);
        let normal_strength = render.material.normal_strength.clamp(0.0, 1.0);

        let normal_rounding = if authored_bevel > 0.0 {
            (0.18 + authored_bevel * 2.2 + normal_strength * 0.28).clamp(0.0, 0.62)
        } else {
            (normal_strength * 0.34).clamp(0.0, 0.42)
        };

        VisualBevel {
            edge_width: 0.0,
            top_edge: normal_rounding,
            side_edge: normal_rounding * 0.72,
        }
    }
}
