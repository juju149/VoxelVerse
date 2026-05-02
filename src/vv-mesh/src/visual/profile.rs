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
    pub(crate) fn disabled() -> Self {
        Self {
            edge_width: 0.0,
            top_edge: 0.0,
            side_edge: 0.0,
        }
    }

    #[inline]
    pub(crate) fn is_enabled(self) -> bool {
        self.edge_width > 0.0001
    }
}

impl MeshGen {
    pub(crate) fn visual_bevel(render: &CompiledBlockRender) -> VisualBevel {
        let is_cube = matches!(render.shape, CompiledBlockShape::Cube);

        if !is_cube {
            return VisualBevel::disabled();
        }

        // Règle moteur :
        // Un cube visible ne doit jamais être un cube mathématique coupant.
        // Le .ron peut augmenter le bevel, mais pas descendre sous ce minimum.
        let engine_min_bevel = if render.meshing.occludes {
            0.035
        } else {
            0.018
        };

        let authored_bevel = render.material.bevel;
        let edge_width = authored_bevel.max(engine_min_bevel).clamp(0.0, 0.18);

        if edge_width <= 0.0001 {
            return VisualBevel::disabled();
        }

        let authored_normal = render.material.normal_strength.clamp(0.0, 1.0);

        // Même si le .ron oublie normal_strength, on garde une base douce.
        let normal_strength = authored_normal.max(0.62);

        VisualBevel {
            edge_width,
            top_edge: normal_strength,
            side_edge: normal_strength * 0.92,
        }
    }
}
