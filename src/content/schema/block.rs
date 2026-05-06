use serde::Deserialize;

/// Raw block definition as written in pack data files.
/// The block's key (e.g. "core:dirt") is derived from its file path — not stored here.
#[derive(Debug, Clone, Deserialize)]
pub struct RawBlockDef {
    pub display_name: String,
    pub solid: bool,
    /// RGB color used for close-up voxel rendering (until texture atlas is ready).
    pub color: [f32; 3],
    /// How many tool hits to break this block. 0.0 = unbreakable.
    pub hardness: f32,
}
