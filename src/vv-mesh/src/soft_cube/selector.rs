use vv_registry::CompiledSurfaceProgram;

use crate::MeshGen;

use super::SoftCubeParams;

impl MeshGen {
    pub(crate) fn soft_cube_params(
        render: &vv_registry::CompiledBlockRender,
    ) -> Option<SoftCubeParams> {
        match render.material.surface_program {
            CompiledSurfaceProgram::Patterned(program)
                if vv_registry::pattern_has_geometry(program.kind)
                    && program.gap_width > 0.0001
                    && program.gap_depth > 0.0001 =>
            {
                // Surface relief mode:
                // - radius 0 keeps the cube perfectly sealed
                // - pillow 0 prevents bombed faces
                // - segments 1 is enough because patterned/emit.rs creates the relief mesh itself
                Some(SoftCubeParams {
                    radius: 0.0,
                    pillow: 0.0,
                    segments: 1,
                })
            }
            _ => None,
        }
    }
}
