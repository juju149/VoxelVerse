//! Plain data types shared across the procedural pipeline.

use super::MAX_BIOME_WEIGHTS;
use glam::Vec3;
use vv_voxel::{VoxelCoord, VoxelId};

#[derive(Clone, Copy, Debug, Default)]
pub struct BiomeWeight {
    pub biome: u16,
    pub weight: f32,
}

/// Surface sample carried through the per-voxel pipeline.  `biome_weights`
/// is an inline fixed-size buffer — no `Vec` allocation in the hot path.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceSample {
    pub height: u32,
    pub primary_biome: usize,
    pub biome_weights: [BiomeWeight; MAX_BIOME_WEIGHTS],
    pub weight_count: u8,
    pub temperature: f32,
    pub humidity: f32,
    pub roughness: f32,
}

impl SurfaceSample {
    pub fn weights(&self) -> &[BiomeWeight] {
        &self.biome_weights[..self.weight_count as usize]
    }
}

#[derive(Clone, Debug)]
pub struct GeneratedVoxelContext {
    pub face: u8,
    pub u: u32,
    pub v: u32,
    pub layer: u32,
    pub dir: Vec3,
    pub depth_from_surface: i32,
    pub surface: SurfaceSample,
}

#[derive(Clone, Debug)]
pub enum FeatureStamp {
    Tree {
        coord: VoxelCoord,
        trunk: VoxelId,
        leaves: VoxelId,
        height: u32,
        canopy_radius: u32,
        priority: i32,
    },
    Structure {
        coord: VoxelCoord,
        stamp: String,
        priority: i32,
    },
}

/// How a prop is oriented relative to its anchor voxel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PropOrientation {
    /// Sits on a solid block, oriented radially outward (normal above-ground).
    #[default]
    Floor,
    /// Hangs from a solid block above, oriented radially inward (cave ceiling).
    Ceiling,
}

/// A vox prop instance to be rendered above the terrain surface.
/// Props are not in the voxel grid — they are rendered separately.
#[derive(Clone, Debug)]
pub struct PropStamp {
    pub face: u8,
    pub u: u32,
    pub v: u32,
    /// Layer index of the anchor solid block.
    /// For `Floor` props the prop sits at `surface_layer + 1`;
    /// for `Ceiling` props it hangs below at `surface_layer - 1`.
    pub surface_layer: u32,
    /// Content ref to a .vox asset, e.g. `"core:voxel/vegetation/flowers/flower_blue_1"`.
    pub model_key: String,
    /// Quarter-turn rotation around the radial axis (0–3).
    pub rotation: u8,
    /// Placement orientation — floor (default) or ceiling.
    pub orientation: PropOrientation,
}
