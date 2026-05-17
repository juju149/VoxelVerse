use vv_math::SphericalGrid;
use vv_voxel::LodKey;

/// Pre-computed colors for one LOD macro-cell.
#[derive(Clone, Copy, Debug)]
pub struct LodCellColors {
    /// Top-face color (biome-driven surface color).
    pub top: [f32; 3],
    /// Wall color (subsurface / cliff color).
    pub wall: [f32; 3],
    pub is_water: bool,
}

/// Everything the LOD mesher needs for one tile.  No world references.
///
/// The world layer builds this by calling `PlanetSnapshot::prepare_lod_mesh_input`.
/// All biome color computation, height sampling, and skirt estimation happen there.
pub struct LodMeshInput {
    pub key: LodKey,
    /// Spherical grid for vertex position computation.
    pub grid: SphericalGrid,
    /// (n+1) × (n+1) corner heights (n = LOD_GRID_RES = CHUNK_SIZE).
    /// Indexed as `corner_heights[j * (n+1) + i]` where i = column, j = row.
    pub corner_heights: Vec<u32>,
    /// n × n cell heights (max of 4 corner samples per cell).
    pub cell_heights: Vec<u32>,
    /// n × n pre-computed cell colors.
    pub cell_colors: Vec<LodCellColors>,
    pub sea_level: u32,
    /// Skirt depth in layers for tile-boundary walls.
    pub skirt_layers: u32,
}
