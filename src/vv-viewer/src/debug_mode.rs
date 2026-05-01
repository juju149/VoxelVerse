/// Debug visualization modes for the viewer.
/// The u32 value maps directly to the WGSL shader debug_mode uniform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DebugMode {
    Beauty = 0,
    FlatColor = 1,
    Palette = 2,
    Noise = 3,
    AoOnly = 4,
    FaceId = 5,
    Uv = 6,
    EdgesOnly = 7,
    NoVariation = 8,
    MacroNoise = 9,
    MicroNoise = 10,
    RuntimeIds = 11,
}

impl Default for DebugMode {
    fn default() -> Self {
        Self::Beauty
    }
}

impl DebugMode {
    pub const ALL: &'static [DebugMode] = &[
        Self::Beauty,
        Self::FlatColor,
        Self::Palette,
        Self::FaceId,
        Self::Uv,
        Self::AoOnly,
        Self::NoVariation,
        Self::MacroNoise,
        Self::MicroNoise,
        Self::EdgesOnly,
        Self::Noise,
        Self::RuntimeIds,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Beauty => "Beauty",
            Self::FlatColor => "Flat Color",
            Self::Palette => "Palette",
            Self::Noise => "Noise (combined)",
            Self::AoOnly => "AO Only",
            Self::FaceId => "Face ID",
            Self::Uv => "UV",
            Self::EdgesOnly => "Edges Only",
            Self::NoVariation => "No Variation",
            Self::MacroNoise => "Macro Noise",
            Self::MicroNoise => "Micro Noise",
            Self::RuntimeIds => "Runtime IDs",
        }
    }

    pub fn as_u32(self) -> u32 {
        self as u32
    }

    pub fn from_u32(v: u32) -> Self {
        match v {
            1 => Self::FlatColor,
            2 => Self::Palette,
            3 => Self::Noise,
            4 => Self::AoOnly,
            5 => Self::FaceId,
            6 => Self::Uv,
            7 => Self::EdgesOnly,
            8 => Self::NoVariation,
            9 => Self::MacroNoise,
            10 => Self::MicroNoise,
            11 => Self::RuntimeIds,
            _ => Self::Beauty,
        }
    }
}
