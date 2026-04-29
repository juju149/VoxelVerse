/// Central engine configuration.
///
/// All tunable parameters live here. Change a value once; it propagates
/// to every system that uses it. No magic numbers scattered across files.
#[derive(Clone, Debug)]
pub struct EngineConfig {
    /// Optional development cap for generated planet resolution.
    /// Physical planet size comes from content; voxel size controls density.
    pub max_debug_resolution: Option<u32>,
    pub physics: PhysicsConfig,
    pub player: PlayerConfig,
    pub render: RenderConfig,
    pub worldgen: WorldGenConfig,
    pub lod: LodConfig,
}

/// Physical constants used by the movement and collision solver.
#[derive(Clone, Debug)]
pub struct PhysicsConfig {
    /// Gravitational acceleration in units/s².
    pub gravity: f32,
    /// Player capsule height in world units.
    pub player_height: f32,
    /// Camera eye position above player feet.
    pub eye_height: f32,
    /// Player capsule radius.
    pub player_radius: f32,
    /// Maximum step-up height for automatic stair climbing.
    pub step_height: f32,
    /// Radial layers from planet centre that cannot be mined.
    pub core_protection_layers: u32,
}

/// Player movement and control parameters.
#[derive(Clone, Debug)]
pub struct PlayerConfig {
    /// Default walking speed in units/s.
    pub move_speed: f32,
    /// Jump impulse in units/s.
    pub jump_force: f32,
    /// Mouse look sensitivity in radians per pixel.
    pub mouse_sensitivity: f32,
    /// Height above terrain surface at which the player spawns.
    pub spawn_height_offset: f32,
    /// Horizontal reach of the block placement/mining ray in world units.
    pub reach_distance: f32,
}

/// Rendering quality and visual parameters.
#[derive(Clone, Debug)]
pub struct RenderConfig {
    /// Shadow map texture dimension (power-of-two recommended).
    pub shadow_map_size: u32,
    /// Vertical field of view in first-person mode, in degrees.
    pub fov_first_person_deg: f32,
    /// Vertical field of view in orbit mode, in degrees.
    pub fov_orbit_deg: f32,
    /// Camera near clip plane.
    pub near_plane: f32,
    /// Camera far clip plane.
    pub far_plane: f32,
    /// Duration of LOD cross-fade transitions in seconds.
    pub lod_fade_duration: f32,
    /// Global atmospheric lighting and background parameters.
    pub atmosphere: AtmosphereConfig,
}

/// Visual atmosphere parameters consumed by the renderer.
///
/// These values are intentionally render configuration for now. Later systems
/// such as biome fog, weather, or a day-night cycle can author a resolved
/// atmosphere and feed the same renderer path.
#[derive(Clone, Debug)]
pub struct AtmosphereConfig {
    /// Direction from the world toward the sun.
    pub sun_direction: [f32; 3],
    /// Linear sun radiance used by direct lighting.
    pub sun_color: [f32; 3],
    /// Linear ambient sky color used by hemispheric lighting.
    pub sky_color: [f32; 3],
    /// Linear lower hemisphere ambient bounce color.
    pub ground_ambient_color: [f32; 3],
    /// Linear fog color.
    pub fog_color: [f32; 3],
    /// Exponential squared fog density in world units.
    pub fog_density: f32,
    /// Main pass clear color, used until a dedicated sky pass owns the background.
    pub clear_color: [f64; 4],
}

/// Procedural world generation parameters.
#[derive(Clone, Debug)]
pub struct WorldGenConfig {
    /// Deterministic noise seed.
    pub noise_seed: u32,
    /// Number of fractal noise octaves.
    pub noise_octaves: u32,
    /// Amplitude multiplier per octave (persistence).
    pub noise_persistence: f32,
    /// Frequency multiplier per octave (lacunarity).
    pub noise_lacunarity: f32,
}

/// LOD system parameters.
#[derive(Clone, Debug)]
pub struct LodConfig {
    /// Vertex grid resolution used when rasterising LOD heightmap tiles.
    pub tile_grid_res: u32,
    /// Maximum voxel chunks the renderer is allowed to require in one frame.
    pub max_required_chunks: usize,
    /// Maximum chunk keys retained in the streaming queue.
    pub max_chunk_queue: usize,
    /// Maximum chunk meshes kept alive by the current coverage set.
    pub max_active_chunks: usize,
    /// Maximum new chunk mesh jobs started per frame.
    pub chunk_jobs_per_frame: usize,
    /// Maximum chunk mesh jobs in flight.
    pub max_pending_chunk_jobs: usize,
    /// Maximum chunk mesh uploads accepted per frame.
    pub chunk_uploads_per_frame: usize,
    /// Maximum LOD tiles the renderer is allowed to require in one frame.
    pub max_required_lods: usize,
    /// Maximum LOD meshes kept alive by the current coverage set.
    pub max_active_lods: usize,
    /// Maximum new LOD mesh jobs started per frame.
    pub lod_jobs_per_frame: usize,
    /// Maximum LOD mesh jobs in flight.
    pub max_pending_lod_jobs: usize,
    /// Maximum LOD mesh uploads accepted per frame.
    pub lod_uploads_per_frame: usize,
    /// Hard cap for GPU buffer creations in one frame.
    pub max_gpu_uploads_per_frame: u32,
    /// Maximum retiring meshes kept for cross-fade fallback.
    pub max_retiring_meshes: usize,
}

// --- Default implementations ------------------------------------------------

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_debug_resolution: None,
            physics: PhysicsConfig::default(),
            player: PlayerConfig::default(),
            render: RenderConfig::default(),
            worldgen: WorldGenConfig::default(),
            lod: LodConfig::default(),
        }
    }
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: 12.0,
            player_height: 1.8,
            eye_height: 1.6,
            player_radius: 0.3,
            step_height: 0.6,
            core_protection_layers: 6,
        }
    }
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            jump_force: 8.0,
            mouse_sensitivity: 0.002,
            spawn_height_offset: 10.0,
            reach_distance: 8.0,
        }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            shadow_map_size: 4096,
            fov_first_person_deg: 80.0,
            fov_orbit_deg: 45.0,
            near_plane: 0.1,
            far_plane: 20000.0,
            lod_fade_duration: 0.0,
            atmosphere: AtmosphereConfig::default(),
        }
    }
}

impl Default for AtmosphereConfig {
    fn default() -> Self {
        Self {
            sun_direction: [0.5, 0.8, 0.4],
            sun_color: [1.6, 1.5, 1.3],
            sky_color: [0.15, 0.3, 0.6],
            ground_ambient_color: [0.05, 0.04, 0.03],
            fog_color: [0.56, 0.68, 0.82],
            fog_density: 0.0015,
            clear_color: [0.02, 0.03, 0.05, 1.0],
        }
    }
}

impl Default for WorldGenConfig {
    fn default() -> Self {
        Self {
            noise_seed: 42,
            noise_octaves: 4,
            noise_persistence: 0.5,
            noise_lacunarity: 2.0,
        }
    }
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            tile_grid_res: 32,
            max_required_chunks: 160,
            max_chunk_queue: 96,
            max_active_chunks: 192,
            chunk_jobs_per_frame: 1,
            max_pending_chunk_jobs: 2,
            chunk_uploads_per_frame: 2,
            max_required_lods: 192,
            max_active_lods: 224,
            lod_jobs_per_frame: 4,
            max_pending_lod_jobs: 8,
            lod_uploads_per_frame: 4,
            max_gpu_uploads_per_frame: 4,
            max_retiring_meshes: 64,
        }
    }
}
