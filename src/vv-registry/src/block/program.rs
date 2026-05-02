#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledSurfaceProgramKind {
    Flat,
}

impl CompiledSurfaceProgramKind {
    pub fn runtime_id(self) -> u32 {
        match self {
            Self::Flat => RUNTIME_SURFACE_PROGRAM_FLAT,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompiledSurfaceProgram {
    pub kind: CompiledSurfaceProgramKind,

    // Reserved for future programs:
    // stone_brick, wood_log, grass, ore, crystal...
    pub params_a: [f32; 4],
    pub params_b: [f32; 4],
}

impl CompiledSurfaceProgram {
    pub fn flat() -> Self {
        Self {
            kind: CompiledSurfaceProgramKind::Flat,
            params_a: [0.0; 4],
            params_b: [0.0; 4],
        }
    }

    pub fn runtime_id(self) -> u32 {
        self.kind.runtime_id()
    }
}

pub const RUNTIME_SURFACE_PROGRAM_FLAT: u32 = 0;
