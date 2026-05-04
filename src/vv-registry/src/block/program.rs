use super::RuntimePatternedProgram;

pub const RUNTIME_SURFACE_PROGRAM_FLAT: u32 = 0;
pub const RUNTIME_SURFACE_PROGRAM_PATTERNED: u32 = 1;

pub const RUNTIME_PATTERN_GRID: u32 = 0;
pub const RUNTIME_PATTERN_STRIPS: u32 = 1;
pub const RUNTIME_PATTERN_RUNNING_BOND: u32 = 2;
pub const RUNTIME_PATTERN_RINGS: u32 = 3;
pub const RUNTIME_PATTERN_NATURAL_CELLS: u32 = 4;
pub const RUNTIME_PATTERN_CRACKED_CELLS: u32 = 5;
pub const RUNTIME_PATTERN_LAYERED_SURFACE: u32 = 6;

pub const RUNTIME_PATTERN_ORIENTATION_AUTO: u32 = 0;
pub const RUNTIME_PATTERN_ORIENTATION_HORIZONTAL: u32 = 1;
pub const RUNTIME_PATTERN_ORIENTATION_VERTICAL: u32 = 2;
pub const RUNTIME_PATTERN_ORIENTATION_RADIAL: u32 = 3;

pub const RUNTIME_PATTERN_FLAG_STAGGER: u32 = 1 << 0;
pub const RUNTIME_PATTERN_ORIENTATION_SHIFT: u32 = 8;
pub const RUNTIME_PATTERN_ORIENTATION_MASK: u32 = 0xFF << RUNTIME_PATTERN_ORIENTATION_SHIFT;

/// Whether a pattern kind generates per-cell geometry (recessed mortar, raised
/// panels, bevels) or is purely a shader effect that paints a flat soft cube.
///
/// Geometry patterns (bricks/wall layouts) carve volume out of the face.
/// Shader patterns (rings/wood-like organic surfaces) leave the mesh as a clean
/// soft cube and rely entirely on the fragment shader for visual structure.
pub fn pattern_has_geometry(kind: u32) -> bool {
    !matches!(
        kind,
        RUNTIME_PATTERN_RINGS | RUNTIME_PATTERN_LAYERED_SURFACE,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompiledSurfaceProgram {
    Flat,
    Patterned(RuntimePatternedProgram),
}

impl CompiledSurfaceProgram {
    pub fn flat() -> Self {
        Self::Flat
    }

    pub fn patterned(program: RuntimePatternedProgram) -> Self {
        Self::Patterned(program)
    }

    pub fn runtime_id(self) -> u32 {
        match self {
            Self::Flat => RUNTIME_SURFACE_PROGRAM_FLAT,
            Self::Patterned(_) => RUNTIME_SURFACE_PROGRAM_PATTERNED,
        }
    }

    pub fn patterned_runtime(self) -> RuntimePatternedProgram {
        match self {
            Self::Flat => RuntimePatternedProgram::disabled(),
            Self::Patterned(program) => program,
        }
    }
}

pub const RUNTIME_MODEL_OPERATOR_FLAT: u32 = 0;
pub const RUNTIME_MODEL_OPERATOR_CELLS: u32 = 1;
pub const RUNTIME_MODEL_OPERATOR_BRICKS: u32 = 2;
pub const RUNTIME_MODEL_OPERATOR_TILES: u32 = 3;
pub const RUNTIME_MODEL_OPERATOR_RINGS: u32 = 4;
pub const RUNTIME_MODEL_OPERATOR_STRIPS: u32 = 5;
pub const RUNTIME_MODEL_OPERATOR_WAVES: u32 = 6;
pub const RUNTIME_MODEL_OPERATOR_CRACKS: u32 = 7;
pub const RUNTIME_MODEL_OPERATOR_VEINS: u32 = 8;
pub const RUNTIME_MODEL_OPERATOR_BLOBS: u32 = 9;
pub const RUNTIME_MODEL_OPERATOR_GRANULES: u32 = 10;
pub const RUNTIME_MODEL_OPERATOR_INCLUSIONS: u32 = 11;
pub const RUNTIME_MODEL_OPERATOR_EMISSIVE_FILL: u32 = 12;
pub const RUNTIME_MODEL_OPERATOR_ORNAMENTAL_LINE: u32 = 13;
