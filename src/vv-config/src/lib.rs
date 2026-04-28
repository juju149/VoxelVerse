/// Central engine configuration.
///
/// All tunable parameters live here. Change a value once; it propagates
/// to every system that uses it. No magic numbers scattered across files.
#[derive(Clone, Debug)]
pub struct EngineConfig {
    /// Face grid resolution of the planet (also the radial layer count).
    /// Increase for a larger, more detailed planet; decrease for faster gen.
    pub planet_resolution: u32,
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
}

// --- Default implementations ------------------------------------------------

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            planet_resolution: 10000,
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
            lod_fade_duration: 2.0,
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
        Self { tile_grid_res: 64 }
    }
}
