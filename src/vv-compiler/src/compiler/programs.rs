use super::prelude::*;
use super::ContentCompiler;

impl ContentCompiler {
    pub(super) fn compile_surface_program(
        &mut self,
        doc: &RawDocument<BlockDef>,
        program: &BlockSurfaceProgramDef,
    ) -> CompiledSurfaceProgram {
        match program {
            BlockSurfaceProgramDef::Flat => CompiledSurfaceProgram::flat(),
            BlockSurfaceProgramDef::Patterned(patterned) => {
                CompiledSurfaceProgram::patterned(self.compile_patterned_program(doc, patterned))
            }
        }
    }

    fn compile_patterned_program(
        &mut self,
        doc: &RawDocument<BlockDef>,
        program: &BlockPatternedProgramDef,
    ) -> RuntimePatternedProgram {
        let rows = self.clamp_u8_range(doc, "render.program.rows", program.rows, 1, 12) as u32;
        let columns =
            self.clamp_u8_range(doc, "render.program.columns", program.columns, 1, 12) as u32;

        RuntimePatternedProgram {
            kind: compiled_pattern_kind(program.pattern),
            rows,
            columns,
            flags: compiled_pattern_flags(program.stagger, program.orientation),

            gap_width: self.clamp_range(
                doc,
                "render.program.gap_width",
                program.gap_width,
                0.0,
                0.20,
            ),
            gap_depth: self.clamp_range(
                doc,
                "render.program.gap_depth",
                program.gap_depth,
                0.0,
                0.20,
            ),
            cell_bevel: self.clamp_range(
                doc,
                "render.program.cell_bevel",
                program.cell_bevel,
                0.0,
                0.15,
            ),
            cell_roundness: self.clamp_unit(
                doc,
                "render.program.cell_roundness",
                program.cell_roundness,
            ),

            cell_pillow: self.clamp_range(
                doc,
                "render.program.cell_pillow",
                program.cell_pillow,
                0.0,
                0.10,
            ),
            height_variation: self.clamp_range(
                doc,
                "render.program.height_variation",
                program.height_variation,
                0.0,
                0.15,
            ),
            color_variation: self.clamp_unit(
                doc,
                "render.program.color_variation",
                program.color_variation,
            ),
            crack_density: self.clamp_unit(
                doc,
                "render.program.crack_density",
                program.crack_density,
            ),

            crack_depth: self.clamp_range(
                doc,
                "render.program.crack_depth",
                program.crack_depth,
                0.0,
                0.15,
            ),
            seed: program.seed,
            _padding: [0; 2],
        }
    }

    fn clamp_u8_range(
        &mut self,
        doc: &RawDocument<BlockDef>,
        field: &str,
        value: u8,
        min: u8,
        max: u8,
    ) -> u8 {
        if value >= min && value <= max {
            value
        } else {
            self.invalid_value(
                doc,
                field,
                &value.to_string(),
                &format!("expected an integer between {min} and {max}"),
            );
            value.clamp(min, max)
        }
    }
}

pub(super) fn compiled_pattern_kind(kind: BlockPatternKind) -> u32 {
    match kind {
        BlockPatternKind::Grid => RUNTIME_PATTERN_GRID,
        BlockPatternKind::Strips => RUNTIME_PATTERN_STRIPS,
        BlockPatternKind::RunningBond => RUNTIME_PATTERN_RUNNING_BOND,
        BlockPatternKind::Rings => RUNTIME_PATTERN_RINGS,
        BlockPatternKind::NaturalCells => RUNTIME_PATTERN_NATURAL_CELLS,
        BlockPatternKind::CrackedCells => RUNTIME_PATTERN_CRACKED_CELLS,
        BlockPatternKind::LayeredSurface => RUNTIME_PATTERN_LAYERED_SURFACE,
    }
}

pub(super) fn compiled_pattern_orientation(orientation: BlockPatternOrientation) -> u32 {
    match orientation {
        BlockPatternOrientation::Auto => RUNTIME_PATTERN_ORIENTATION_AUTO,
        BlockPatternOrientation::Horizontal => RUNTIME_PATTERN_ORIENTATION_HORIZONTAL,
        BlockPatternOrientation::Vertical => RUNTIME_PATTERN_ORIENTATION_VERTICAL,
        BlockPatternOrientation::Radial => RUNTIME_PATTERN_ORIENTATION_RADIAL,
    }
}

fn compiled_pattern_flags(stagger: bool, orientation: BlockPatternOrientation) -> u32 {
    let stagger_flag = u32::from(stagger) * RUNTIME_PATTERN_FLAG_STAGGER;
    let orientation_bits =
        compiled_pattern_orientation(orientation) << RUNTIME_PATTERN_ORIENTATION_SHIFT;

    stagger_flag | (orientation_bits & RUNTIME_PATTERN_ORIENTATION_MASK)
}
