pub struct TerrainPalette;

impl TerrainPalette {
    /// Deep mantle rock color — shown on LOD when below core_layers.
    pub const LOD_CORE: [f32; 3] = [0.18, 0.19, 0.22];
    pub const PLAYER: [f32; 3] = [0.0, 0.5, 1.0];
    pub const COLLISION_DEBUG: [f32; 3] = [1.0, 0.0, 0.0];
    pub const CURSOR: [f32; 3] = [1.0, 1.0, 0.0];
    pub const UI_WHITE: [f32; 3] = [1.0, 1.0, 1.0];
}
