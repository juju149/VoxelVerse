use vv_registry::CompiledBlockRender;

use crate::MeshGen;

#[derive(Clone, Copy, Debug)]
pub(crate) struct VisualBevel {
    pub edge_width: f32,
    pub top_edge: f32,
    pub side_edge: f32,
}

impl MeshGen {
    pub(crate) fn visual_bevel(render: &CompiledBlockRender) -> VisualBevel {
        let edge_width = render.material.bevel.clamp(0.0, 0.12);

        if edge_width <= 0.0 {
            return VisualBevel {
                edge_width: 0.0,
                top_edge: 0.0,
                side_edge: 0.0,
            };
        }

        let normal_strength = render.material.normal_strength.clamp(0.0, 1.0);

        VisualBevel {
            edge_width,
            top_edge: 0.08 + normal_strength * 0.36,
            side_edge: 0.04 + normal_strength * 0.22,
        }
    }
}
