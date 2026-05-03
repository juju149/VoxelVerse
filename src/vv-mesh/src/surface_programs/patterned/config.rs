use vv_registry::RuntimePatternedProgram;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PatternedMeshConfig {
    pub kind: u32,
    pub rows: u32,
    pub columns: u32,
    pub flags: u32,

    pub gap_width: f32,
    pub gap_depth: f32,
    pub cell_bevel: f32,
    pub cell_roundness: f32,

    pub cell_pillow: f32,
    pub height_variation: f32,
    pub color_variation: f32,
    pub crack_density: f32,

    pub crack_depth: f32,
    pub seed: u32,
}

impl PatternedMeshConfig {
    pub(crate) fn from_runtime(program: RuntimePatternedProgram) -> Self {
        Self {
            kind: program.kind,
            rows: program.rows.max(1),
            columns: program.columns.max(1),
            flags: program.flags,

            gap_width: program.gap_width,
            gap_depth: program.gap_depth,
            cell_bevel: program.cell_bevel,
            cell_roundness: program.cell_roundness,

            cell_pillow: program.cell_pillow,
            height_variation: program.height_variation,
            color_variation: program.color_variation,
            crack_density: program.crack_density,

            crack_depth: program.crack_depth,
            seed: program.seed,
        }
    }
}
