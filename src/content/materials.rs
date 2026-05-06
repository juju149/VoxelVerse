pub struct TerrainPalette;

impl TerrainPalette {
    pub const CORE: [f32; 3] = [0.2, 0.2, 0.2];
    pub const LOD_CORE: [f32; 3] = [0.2, 0.22, 0.25];
    pub const GRASS: [f32; 3] = [0.1, 0.7, 0.1];
    pub const LOD_GRASS: [f32; 3] = [0.1, 0.8, 0.1];
    pub const LOD_STEEP_GRASS: [f32; 3] = [0.075, 0.6, 0.075];
    pub const DIRT: [f32; 3] = [0.6, 0.4, 0.2];
    pub const PLAYER: [f32; 3] = [0.0, 0.5, 1.0];
    pub const COLLISION_DEBUG: [f32; 3] = [1.0, 0.0, 0.0];
    pub const CURSOR: [f32; 3] = [1.0, 1.0, 0.0];
    pub const UI_WHITE: [f32; 3] = [1.0, 1.0, 1.0];
}
