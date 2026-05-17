//! Mutation methods on `PlanetData`: voxel edits, block damage, world time,
//! and the developer resize operation.

use crate::{BlockDamageResult, PlanetData, VoxelEditResult};
use std::sync::Arc;
use vv_voxel::{VoxelCoord, VoxelId};
use vv_worldgen::ProceduralPlanetTerrain;

impl PlanetData {
    // --- world time -------------------------------------------------------

    pub fn tick_world_time(&mut self, dt_seconds: f32) {
        self.world_time.tick(dt_seconds);
    }

    pub fn set_day_length_seconds(&mut self, seconds: f32) {
        self.world_time.set_day_length_seconds(seconds);
    }

    pub fn set_day_phase(&mut self, phase: f32) {
        self.world_time.set_day_phase(phase);
    }

    pub fn set_fixed_elapsed_seconds(&mut self, elapsed_seconds: f32) {
        self.world_time.set_fixed_elapsed_seconds(elapsed_seconds);
    }

    // --- voxel edits ------------------------------------------------------

    pub fn place_block(&mut self, coord: VoxelCoord, voxel: VoxelId) -> VoxelEditResult {
        self.set_voxel(coord, voxel)
    }

    pub fn remove_block(&mut self, coord: VoxelCoord) -> VoxelEditResult {
        if self.has_core && coord.layer < self.profile.core_layers {
            return VoxelEditResult {
                changed: coord,
                dirty_chunks: Vec::new(),
            };
        }

        // If a prop is sitting directly above the broken block, destroy it too.
        // Props sit at `surface_layer + 1`, so a prop whose surface_layer == coord.layer
        // is supported by this block.
        Arc::make_mut(&mut self.broken_props).break_prop(coord.face, coord.u, coord.v);

        self.set_voxel(coord, VoxelId::AIR)
    }

    pub fn set_voxel(&mut self, coord: VoxelCoord, voxel: VoxelId) -> VoxelEditResult {
        self.block_damage.clear(coord);
        let generated = self.generated_voxel(coord);
        let override_voxel = (voxel != generated).then_some(voxel);
        Arc::make_mut(&mut self.voxels).set_override(coord, override_voxel);
        VoxelEditResult {
            changed: coord,
            dirty_chunks: Self::dirty_chunks_for_coord(self.resolution, coord),
        }
    }

    // --- block damage -----------------------------------------------------

    pub fn apply_block_damage(
        &mut self,
        coord: VoxelCoord,
        amount: f32,
        break_threshold: f32,
    ) -> BlockDamageResult {
        let voxel = self.get_voxel(coord);
        self.block_damage
            .apply_hit(coord, voxel, amount, break_threshold)
    }

    pub fn clear_block_damage(&mut self, coord: VoxelCoord) {
        self.block_damage.clear(coord);
    }

    // --- dev resize -------------------------------------------------------

    /// Regenerate the planet at a new resolution. Used by the dev resize hotkey;
    /// clears all sparse overrides and block damage as a side effect.
    pub fn resize(&mut self, increase: bool) {
        if increase {
            let new_res = (self.resolution as f32 * 1.2) as u32;
            self.resolution = new_res.max(self.resolution + 1).min(16384);
        } else {
            let new_res = (self.resolution as f32 / 1.2) as u32;
            self.resolution = new_res.max(8);
        }

        Arc::make_mut(&mut self.voxels).clear();
        self.block_damage.clear_all();
        self.planet_def = self.planet_def.with_resolution(self.resolution);
        self.planet_def.seed = self.seed;
        self.profile = self.planet_def.to_planet_profile();
        self.resolution = self.profile.resolution;

        println!("Regenerating terrain for resolution {}…", self.resolution);
        self.terrain = ProceduralPlanetTerrain::new(
            self.profile,
            self.procedural.clone(),
            self.procedural_planet_index,
        );
    }
}
