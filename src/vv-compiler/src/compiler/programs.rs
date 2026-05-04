use super::helpers::*;
use super::prelude::*;
use super::ContentCompiler;

impl ContentCompiler {
    pub(super) fn compile_render_surface_program(
        &mut self,
        doc: &RawDocument<BlockDef>,
        render: &BlockRenderDef,
    ) -> CompiledSurfaceProgram {
        if let Some(model) = &render.model {
            if let Some(program) = self.compile_model_surface_program(doc, model) {
                return program;
            }
        }

        self.compile_surface_program(doc, &render.program)
    }

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

    fn compile_model_surface_program(
        &mut self,
        doc: &RawDocument<BlockDef>,
        model: &BlockProceduralModelDef,
    ) -> Option<CompiledSurfaceProgram> {
        let layer = model.layers.iter().find(|layer| layer.enabled)?;
        let patterned = self.patterned_from_model_layer(doc, model, layer)?;
        Some(CompiledSurfaceProgram::patterned(
            self.compile_patterned_program(doc, &patterned),
        ))
    }

    fn patterned_from_model_layer(
        &mut self,
        doc: &RawDocument<BlockDef>,
        model: &BlockProceduralModelDef,
        layer: &BlockLayerDef,
    ) -> Option<BlockPatternedProgramDef> {
        let seed = if layer.seed != 0 {
            layer.seed
        } else if model.seed != 0 {
            model.seed
        } else {
            stable_hash32(&format!("{}:{}", doc.relative_path.display(), layer.id))
        };

        let mut program = BlockPatternedProgramDef {
            seed,
            ..BlockPatternedProgramDef::default()
        };

        match &layer.operator {
            BlockLayerOperatorDef::Flat { .. } => return None,

            BlockLayerOperatorDef::Cells {
                cell_size,
                irregularity,
                bevel,
                height,
                ..
            } => {
                let cells = cells_from_size(*cell_size);
                program.pattern = BlockPatternKind::NaturalCells;
                program.rows = cells;
                program.columns = cells;
                program.stagger = false;
                program.gap_width = 0.008 + irregularity.clamp(0.0, 1.0) * 0.035;
                program.gap_depth = (*height * 0.45).clamp(0.0, 0.20);
                program.cell_bevel = (*bevel).clamp(0.0, 0.15);
                program.cell_roundness =
                    (0.45 + irregularity.clamp(0.0, 1.0) * 0.45).clamp(0.0, 1.0);
                program.cell_pillow = (*height * 0.40).clamp(0.0, 0.10);
                program.height_variation = (*height).clamp(0.0, 0.15);
                program.color_variation =
                    (0.08 + irregularity.clamp(0.0, 1.0) * 0.22).clamp(0.0, 1.0);
            }

            BlockLayerOperatorDef::Bricks {
                rows,
                columns,
                stagger,
                mortar_width,
                mortar_depth,
                bevel,
                height,
                ..
            } => {
                program.pattern = BlockPatternKind::RunningBond;
                program.rows = (*rows).clamp(1, 12);
                program.columns = (*columns).clamp(1, 12);
                program.stagger = *stagger;
                program.gap_width = (*mortar_width).clamp(0.0, 0.20);
                program.gap_depth = (*mortar_depth).clamp(0.0, 0.20);
                program.cell_bevel = (*bevel).clamp(0.0, 0.15);
                program.cell_pillow = (*height * 0.20).clamp(0.0, 0.10);
                program.height_variation = (*height).clamp(0.0, 0.15);
                program.color_variation = 0.14;
            }

            BlockLayerOperatorDef::Tiles {
                rows,
                columns,
                gap_width,
                gap_depth,
                bevel,
                height,
                ..
            } => {
                program.pattern = BlockPatternKind::Grid;
                program.rows = (*rows).clamp(1, 12);
                program.columns = (*columns).clamp(1, 12);
                program.stagger = false;
                program.gap_width = (*gap_width).clamp(0.0, 0.20);
                program.gap_depth = (*gap_depth).clamp(0.0, 0.20);
                program.cell_bevel = (*bevel).clamp(0.0, 0.15);
                program.cell_pillow = (*height * 0.20).clamp(0.0, 0.10);
                program.height_variation = (*height).clamp(0.0, 0.15);
                program.color_variation = 0.10;
            }

            BlockLayerOperatorDef::Rings {
                rings,
                wobble,
                height,
                ..
            } => {
                program.pattern = BlockPatternKind::Rings;
                program.rows = (*rings).clamp(1, 12);
                program.columns = (*rings).clamp(1, 12);
                program.stagger = false;
                program.gap_width = 0.0;
                program.gap_depth = 0.0;
                program.cell_roundness = 0.85;
                program.cell_pillow = (*height * 0.20).clamp(0.0, 0.10);
                program.height_variation = (*height).clamp(0.0, 0.15);
                program.color_variation = (0.12 + wobble.clamp(0.0, 1.0) * 0.22).clamp(0.0, 1.0);
                program.orientation = BlockPatternOrientation::Radial;
            }

            BlockLayerOperatorDef::Strips {
                count,
                vertical,
                wobble,
                height,
                ..
            } => {
                program.pattern = BlockPatternKind::Strips;
                program.rows = (*count).clamp(1, 12);
                program.columns = (*count).clamp(1, 12);
                program.stagger = false;
                program.gap_width = 0.015;
                program.gap_depth = (*height * 0.40).clamp(0.0, 0.20);
                program.cell_roundness = 0.65;
                program.height_variation = (*height).clamp(0.0, 0.15);
                program.color_variation = (0.10 + wobble.clamp(0.0, 1.0) * 0.22).clamp(0.0, 1.0);
                program.orientation = if *vertical {
                    BlockPatternOrientation::Vertical
                } else {
                    BlockPatternOrientation::Horizontal
                };
            }

            BlockLayerOperatorDef::Waves {
                count,
                amplitude,
                height,
                ..
            } => {
                program.pattern = BlockPatternKind::Strips;
                program.rows = (*count).clamp(1, 12);
                program.columns = (*count).clamp(1, 12);
                program.stagger = true;
                program.gap_width = 0.012 + amplitude.clamp(0.0, 1.0) * 0.05;
                program.gap_depth = (*height * 0.35).clamp(0.0, 0.20);
                program.cell_roundness = 0.80;
                program.height_variation = (*height).clamp(0.0, 0.15);
                program.color_variation = 0.12;
                program.orientation = BlockPatternOrientation::Horizontal;
            }

            BlockLayerOperatorDef::Cracks {
                density,
                depth,
                scale,
                ..
            } => {
                let cells = cells_from_size(0.14 / scale.max(0.25));
                program.pattern = BlockPatternKind::CrackedCells;
                program.rows = cells;
                program.columns = cells;
                program.gap_width = 0.010;
                program.gap_depth = (*depth).clamp(0.0, 0.20);
                program.cell_bevel = 0.008;
                program.cell_roundness = 0.50;
                program.height_variation = (*depth * 0.50).clamp(0.0, 0.15);
                program.crack_density = (*density).clamp(0.0, 1.0);
                program.crack_depth = (*depth).clamp(0.0, 0.15);
                program.color_variation = 0.18;
            }

            BlockLayerOperatorDef::Veins {
                scale, strength, ..
            } => {
                let cells = cells_from_size(0.16 / scale.max(0.25));
                program.pattern = BlockPatternKind::CrackedCells;
                program.rows = cells;
                program.columns = cells;
                program.gap_width = 0.008;
                program.gap_depth = (strength.clamp(0.0, 1.0) * 0.04).clamp(0.0, 0.20);
                program.cell_roundness = 0.65;
                program.height_variation = 0.015;
                program.color_variation = strength.clamp(0.0, 1.0);
                program.crack_density = 0.35;
                program.crack_depth = 0.025;
            }

            BlockLayerOperatorDef::Blobs {
                density,
                roundness,
                height,
                ..
            } => {
                let cells = (4.0 + density.clamp(0.0, 1.0) * 8.0).round() as u8;
                program.pattern = BlockPatternKind::NaturalCells;
                program.rows = cells.clamp(1, 12);
                program.columns = cells.clamp(1, 12);
                program.gap_width = 0.010;
                program.gap_depth = (*height * 0.35).clamp(0.0, 0.20);
                program.cell_roundness = (*roundness).clamp(0.0, 1.0);
                program.cell_pillow = (*height * 0.55).clamp(0.0, 0.10);
                program.height_variation = (*height).clamp(0.0, 0.15);
                program.color_variation = 0.20;
            }

            BlockLayerOperatorDef::Granules {
                density,
                min_size,
                max_size,
                height,
                ..
            } => {
                let average = ((*min_size + *max_size) * 0.5).max(0.025);
                let cells = cells_from_size(average);
                program.pattern = BlockPatternKind::NaturalCells;
                program.rows = cells;
                program.columns = cells;
                program.gap_width = 0.004 + density.clamp(0.0, 1.0) * 0.012;
                program.gap_depth = (*height * 0.25).clamp(0.0, 0.20);
                program.cell_roundness = 0.85;
                program.cell_pillow = (*height * 0.35).clamp(0.0, 0.10);
                program.height_variation = (*height).clamp(0.0, 0.15);
                program.color_variation = 0.16;
            }

            BlockLayerOperatorDef::Inclusions {
                density,
                size,
                height,
                ..
            } => {
                let cells = cells_from_size((*size).max(0.04));
                program.pattern = BlockPatternKind::NaturalCells;
                program.rows = cells;
                program.columns = cells;
                program.gap_width = 0.010 + density.clamp(0.0, 1.0) * 0.030;
                program.gap_depth = (*height * 0.45).clamp(0.0, 0.20);
                program.cell_roundness = 0.70;
                program.cell_pillow = (*height * 0.40).clamp(0.0, 0.10);
                program.height_variation = (*height).clamp(0.0, 0.15);
                program.color_variation = 0.30;
            }

            BlockLayerOperatorDef::EmissiveFill { pulse, .. } => {
                program.pattern = BlockPatternKind::CrackedCells;
                program.rows = 5;
                program.columns = 5;
                program.gap_width = 0.060;
                program.gap_depth = 0.060;
                program.cell_roundness = 0.75;
                program.cell_pillow = 0.025;
                program.height_variation = 0.055;
                program.color_variation = (0.35 + pulse.clamp(0.0, 1.0) * 0.30).clamp(0.0, 1.0);
                program.crack_density = 0.55;
                program.crack_depth = 0.065;
            }

            BlockLayerOperatorDef::OrnamentalLine { depth, repeat, .. } => {
                program.pattern = BlockPatternKind::Grid;
                program.rows = 3;
                program.columns = (*repeat).clamp(1, 12);
                program.gap_width = 0.025;
                program.gap_depth = (*depth).clamp(0.0, 0.20);
                program.cell_bevel = 0.008;
                program.cell_roundness = 0.35;
                program.height_variation = (*depth * 0.35).clamp(0.0, 0.15);
                program.color_variation = 0.08;
                program.orientation = BlockPatternOrientation::Horizontal;
            }
        }

        Some(program)
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

fn cells_from_size(size: f32) -> u8 {
    let safe = if size.is_finite() {
        size.clamp(0.04, 1.0)
    } else {
        0.18
    };

    ((1.0 / safe).round() as u8).clamp(1, 12)
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
