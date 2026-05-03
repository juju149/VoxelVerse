use bytemuck::{Pod, Zeroable};

pub const BLOCK_VISUAL_FACE_COUNT: usize = 6;
pub const BLOCK_VISUAL_DETAIL_COUNT: usize = 8;

pub const RUNTIME_BLOCK_DETAIL_NONE: u32 = 0;
pub const RUNTIME_BLOCK_DETAIL_PEBBLE: u32 = 1;
pub const RUNTIME_BLOCK_DETAIL_ROOT: u32 = 2;
pub const RUNTIME_BLOCK_DETAIL_LEAF_LOBE: u32 = 3;
pub const RUNTIME_BLOCK_DETAIL_GRAIN: u32 = 4;
pub const RUNTIME_BLOCK_DETAIL_SPECKLE: u32 = 5;
pub const RUNTIME_BLOCK_DETAIL_STAIN: u32 = 6;
pub const RUNTIME_BLOCK_DETAIL_CRACK: u32 = 7;

pub const RUNTIME_BLOCK_DETAIL_FACE_TOP: u32 = 1 << 0;
pub const RUNTIME_BLOCK_DETAIL_FACE_BOTTOM: u32 = 1 << 1;
pub const RUNTIME_BLOCK_DETAIL_FACE_NORTH: u32 = 1 << 2;
pub const RUNTIME_BLOCK_DETAIL_FACE_SOUTH: u32 = 1 << 3;
pub const RUNTIME_BLOCK_DETAIL_FACE_EAST: u32 = 1 << 4;
pub const RUNTIME_BLOCK_DETAIL_FACE_WEST: u32 = 1 << 5;
pub const RUNTIME_BLOCK_DETAIL_FACE_SIDE: u32 = RUNTIME_BLOCK_DETAIL_FACE_NORTH
    | RUNTIME_BLOCK_DETAIL_FACE_SOUTH
    | RUNTIME_BLOCK_DETAIL_FACE_EAST
    | RUNTIME_BLOCK_DETAIL_FACE_WEST;
pub const RUNTIME_BLOCK_DETAIL_FACE_ALL: u32 = RUNTIME_BLOCK_DETAIL_FACE_TOP
    | RUNTIME_BLOCK_DETAIL_FACE_BOTTOM
    | RUNTIME_BLOCK_DETAIL_FACE_SIDE;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct BlockProceduralConfig {
    pub grid_size: u32,
    pub face_blend: u32,
    pub _padding: [u32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct RuntimePatternedProgram {
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
    pub _padding: [u32; 2],
}

impl RuntimePatternedProgram {
    pub fn disabled() -> Self {
        Self {
            kind: 0,
            rows: 1,
            columns: 1,
            flags: 0,

            gap_width: 0.0,
            gap_depth: 0.0,
            cell_bevel: 0.0,
            cell_roundness: 0.0,

            cell_pillow: 0.0,
            height_variation: 0.0,
            color_variation: 0.0,
            crack_density: 0.0,

            crack_depth: 0.0,
            seed: 0,
            _padding: [0; 2],
        }
    }
}

impl BlockProceduralConfig {
    pub fn new(grid_size: u32, face_blend: bool) -> Self {
        Self {
            grid_size,
            face_blend: u32::from(face_blend),
            _padding: [0; 2],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RuntimeBlockFaceVisual {
    pub color_bias: [f32; 4],
    pub detail_mask: u32,
    pub _padding: [u32; 3],
}

impl Default for RuntimeBlockFaceVisual {
    fn default() -> Self {
        Self {
            color_bias: [1.0, 1.0, 1.0, 1.0],
            detail_mask: 0,
            _padding: [0; 3],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RuntimeBlockDetail {
    pub color: [f32; 4],
    pub params: [f32; 4],
    pub meta: [u32; 4],
}

impl Default for RuntimeBlockDetail {
    fn default() -> Self {
        Self {
            color: [0.0; 4],
            params: [0.0; 4],
            meta: [0; 4],
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct BlockVisualFlags(pub u32);

impl BlockVisualFlags {
    pub const TRANSPARENT: u32 = 1 << 0;
    pub const EMISSIVE: u32 = 1 << 1;
    pub const BIOME_TINTED: u32 = 1 << 2;
    pub const OCCLUDES: u32 = 1 << 3;
    pub const RECEIVES_AO: u32 = 1 << 4;

    pub fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    pub fn contains(self, bit: u32) -> bool {
        self.0 & bit != 0
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct RuntimeBlockVisual {
    pub base_color: [f32; 4],
    pub emission: [f32; 4],

    // x = roughness
    // y = metallic
    // z = alpha
    // w = face_depth
    pub surface: [f32; 4],

    // x = bevel
    // y = normal_strength
    // z = shape_profile_id
    // w = roundness
    pub shape: [f32; 4],

    pub variation_a: [f32; 4],
    pub variation_b: [f32; 4],
    pub response: [f32; 4],

    // x = palette_offset
    // y = palette_len
    // z = material_id
    // w = flags
    pub palette: [u32; 4],

    // x = grid_size legacy
    // y = face_blend legacy
    // z = detail_count
    // w = surface_program_id
    pub procedural: [u32; 4],
    pub patterned: RuntimePatternedProgram,

    pub faces: [RuntimeBlockFaceVisual; BLOCK_VISUAL_FACE_COUNT],
    pub details: [RuntimeBlockDetail; BLOCK_VISUAL_DETAIL_COUNT],
}

impl RuntimeBlockVisual {
    pub fn fallback() -> Self {
        Self {
            base_color: [0.55, 0.55, 0.55, 1.0],
            emission: [0.0; 4],
            surface: [1.0, 0.0, 1.0, 0.0],
            shape: [0.0; 4],
            variation_a: [0.0, 0.0, 1.0, 0.0],
            variation_b: [1.0, 0.0, 0.0, 1.0],
            response: [0.0; 4],
            palette: [0, 1, 0, 0],
            procedural: [10, 0, 0, 0],
            patterned: RuntimePatternedProgram::disabled(),
            faces: [RuntimeBlockFaceVisual::default(); BLOCK_VISUAL_FACE_COUNT],
            details: [RuntimeBlockDetail::default(); BLOCK_VISUAL_DETAIL_COUNT],
        }
    }
}
