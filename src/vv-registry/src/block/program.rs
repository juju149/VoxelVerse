use super::RuntimePatternedProgram;

pub const RUNTIME_SURFACE_PROGRAM_FLAT: u32 = 0;
pub const RUNTIME_SURFACE_PROGRAM_PATTERNED: u32 = 1;

pub const RUNTIME_PATTERN_GRID: u32 = 0;
pub const RUNTIME_PATTERN_STRIPS: u32 = 1;
pub const RUNTIME_PATTERN_RUNNING_BOND: u32 = 2;
pub const RUNTIME_PATTERN_RINGS: u32 = 3;
pub const RUNTIME_PATTERN_NATURAL_CELLS: u32 = 4;
pub const RUNTIME_PATTERN_CRACKED_CELLS: u32 = 5;

pub const RUNTIME_PATTERN_ORIENTATION_AUTO: u32 = 0;
pub const RUNTIME_PATTERN_ORIENTATION_HORIZONTAL: u32 = 1;
pub const RUNTIME_PATTERN_ORIENTATION_VERTICAL: u32 = 2;
pub const RUNTIME_PATTERN_ORIENTATION_RADIAL: u32 = 3;

pub const RUNTIME_PATTERN_FLAG_STAGGER: u32 = 1 << 0;
pub const RUNTIME_PATTERN_ORIENTATION_SHIFT: u32 = 8;
pub const RUNTIME_PATTERN_ORIENTATION_MASK: u32 = 0xFF << RUNTIME_PATTERN_ORIENTATION_SHIFT;

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
