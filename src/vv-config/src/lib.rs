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
    pub day_cycle: DayCycleConfig,
}

/// Parameters for the world day/night cycle.
#[derive(Clone, Debug)]
pub struct DayCycleConfig {
    /// Total real-world seconds for one complete day/night cycle.
    pub day_duration_secs: f32,
    /// Time multiplier applied to real elapsed time.
    /// 1.0 = normal speed. Use higher values to debug the full cycle quickly.
    pub time_scale: f32,
    /// Starting moment as a fraction of the day:
    /// 0.0 = midnight, 0.25 = sunrise, 0.5 = noon, 0.75 = sunset.
    pub initial_time: f32,
    /// When true, the clock never advances. Useful for locking a specific time of day.
    pub freeze_time: bool,
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

/// Shadow rendering mode.
///
/// `Off` disables shadow sampling entirely (everything lit, fastest).
/// `Stable` uses a small fixed kernel with stronger biasing — minimises acne
/// and flickering on beveled edges at the cost of softness.
/// `High` uses the full PCF kernel for crisp contact shadows; can be noisy
/// on grazing voxel bevels.
///
/// Override at runtime with `VV_SHADOWS=off|stable|high`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowMode {
    Off,
    Stable,
    High,
}

impl ShadowMode {
    pub fn as_shader_id(self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Stable => 1.0,
            Self::High => 2.0,
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "off" | "0" | "none" | "disabled" => Some(Self::Off),
            "stable" | "1" | "low" | "medium" => Some(Self::Stable),
            "high" | "2" | "full" => Some(Self::High),
            _ => None,
        }
    }
}

/// Rendering quality and visual parameters.
#[derive(Clone, Debug)]
pub struct RenderConfig {
    /// Shadow map texture dimension (power-of-two recommended).
    pub shadow_map_size: u32,
    /// Shadow filter quality. See [`ShadowMode`].
    pub shadow_mode: ShadowMode,
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
#[derive(Debug, Clone)]
pub struct AtmosphereConfig {
    pub sun_direction: [f32; 3],
    pub sun_color: [f32; 3],
    pub sky_color: [f32; 3],
    pub ground_ambient_color: [f32; 3],
    pub shadow_tint_color: [f32; 3],
    pub fog_color: [f32; 3],
    pub fog_density: f32,
    pub clear_color: [f64; 4],

    pub zenith_color: [f32; 3],
    pub horizon_glow_color: [f32; 3],
    pub moon_direction: [f32; 3],
    pub moon_color: [f32; 3],

    pub exposure: f32,
    pub saturation: f32,
    pub contrast: f32,

    pub fog_start_m: f32,
    pub sky_horizon_power: f32,
    pub star_strength: f32,
    pub night_amount: f32,

    pub planet_center: [f32; 3],
    pub atmosphere_height_m: f32,
    pub atmosphere_fade_start_m: f32,
    pub atmosphere_fade_end_m: f32,
    pub terminator_softness: f32,
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
            day_cycle: DayCycleConfig::default(),
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
            shadow_mode: ShadowMode::Stable,
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
        Self::neutral()
    }
}

impl AtmosphereConfig {
    /// Neutral, slightly warm rendering preset. Desaturated palette so per-block
    /// materials carry the color rather than the atmosphere.
    pub fn neutral() -> Self {
        Self {
            sun_direction: [0.58, 0.56, 0.36],
            sun_color: [1.55, 1.42, 1.18],
            sky_color: [0.190, 0.330, 0.560],
            ground_ambient_color: [0.075, 0.082, 0.090],
            shadow_tint_color: [0.060, 0.085, 0.140],
            fog_color: [0.520, 0.580, 0.660],
            fog_density: 0.00030,
            clear_color: [0.030, 0.045, 0.080, 1.0],

            zenith_color: [0.090, 0.190, 0.420],
            horizon_glow_color: [0.880, 0.640, 0.460],
            moon_direction: [-0.58, -0.56, -0.36],
            moon_color: [0.220, 0.270, 0.420],

            exposure: 1.00,
            saturation: 1.00,
            contrast: 1.10,

            fog_start_m: 140.0,
            sky_horizon_power: 0.78,
            star_strength: 0.0,
            night_amount: 0.0,

            planet_center: [0.0, 0.0, 0.0],
            atmosphere_height_m: 90_000.0,
            atmosphere_fade_start_m: 55_000.0,
            atmosphere_fade_end_m: 120_000.0,
            terminator_softness: 0.085,
        }
    }

    /// Legacy dramatic-sunset preset (warm-orange, high saturation).
    pub fn dramatic_sunset() -> Self {
        Self {
            sun_direction: [0.58, 0.56, 0.36],
            sun_color: [2.35, 1.62, 0.86],
            sky_color: [0.105, 0.305, 0.760],
            ground_ambient_color: [0.042, 0.050, 0.070],
            shadow_tint_color: [0.030, 0.060, 0.190],
            fog_color: [0.380, 0.520, 0.760],
            fog_density: 0.00030,
            clear_color: [0.016, 0.028, 0.065, 1.0],

            zenith_color: [0.030, 0.125, 0.520],
            horizon_glow_color: [1.050, 0.610, 0.310],
            moon_direction: [-0.58, -0.56, -0.36],
            moon_color: [0.240, 0.315, 0.610],

            exposure: 0.88,
            saturation: 1.18,
            contrast: 1.34,

            fog_start_m: 140.0,
            sky_horizon_power: 0.78,
            star_strength: 0.0,
            night_amount: 0.0,

            planet_center: [0.0, 0.0, 0.0],
            atmosphere_height_m: 90_000.0,
            atmosphere_fade_start_m: 55_000.0,
            atmosphere_fade_end_m: 120_000.0,
            terminator_softness: 0.085,
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

impl Default for DayCycleConfig {
    fn default() -> Self {
        Self {
            day_duration_secs: 1200.0, // 20 real-world minutes per full day
            time_scale: 1.0,
            initial_time: 0.665,
            freeze_time: false,
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
