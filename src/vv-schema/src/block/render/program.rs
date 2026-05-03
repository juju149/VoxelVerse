use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockSurfaceProgramDef {
    Flat,
    Patterned(BlockPatternedProgramDef),
}

pub type RawBlockSurfaceProgramDef = BlockSurfaceProgramDef;

impl Default for BlockSurfaceProgramDef {
    fn default() -> Self {
        Self::Flat
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockPatternedProgramDef {
    pub pattern: BlockPatternKind,

    pub rows: u8,
    pub columns: u8,
    pub stagger: bool,

    pub gap_width: f32,
    pub gap_depth: f32,

    pub cell_bevel: f32,
    pub cell_roundness: f32,
    pub cell_pillow: f32,

    pub height_variation: f32,
    pub color_variation: f32,

    pub crack_density: f32,
    pub crack_depth: f32,

    pub orientation: BlockPatternOrientation,
    pub seed: u32,
}

impl Default for BlockPatternedProgramDef {
    fn default() -> Self {
        Self {
            pattern: BlockPatternKind::RunningBond,

            rows: 4,
            columns: 3,
            stagger: true,

            gap_width: 0.045,
            gap_depth: 0.035,

            cell_bevel: 0.025,
            cell_roundness: 0.7,
            cell_pillow: 0.015,

            height_variation: 0.018,
            color_variation: 0.12,

            crack_density: 0.0,
            crack_depth: 0.0,

            orientation: BlockPatternOrientation::Auto,
            seed: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockPatternKind {
    Grid,
    Strips,
    RunningBond,
    Rings,
    NaturalCells,
    CrackedCells,
}

impl Default for BlockPatternKind {
    fn default() -> Self {
        Self::RunningBond
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockPatternOrientation {
    Auto,
    Horizontal,
    Vertical,
    Radial,
}

impl Default for BlockPatternOrientation {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockSurfaceProgramId {
    Flat = 0,
    Patterned = 1,
}

impl BlockSurfaceProgramDef {
    pub fn program_id(&self) -> BlockSurfaceProgramId {
        match self {
            Self::Flat => BlockSurfaceProgramId::Flat,
            Self::Patterned(_) => BlockSurfaceProgramId::Patterned,
        }
    }
}
