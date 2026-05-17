//! Per-chunk vox-prop scattering (flowers, rocks, debris on top surface).
//!
//! Surface-side companion to `cave_decoration` — emits `PropStamp` instances
//! for floor scatters declared on the planet. Cave floor/ceiling scatters
//! are appended via `cave_decoration::cave_props_for_chunk`.
//!
//! Cell grids are world-aligned (not chunk-aligned), so adjacent chunks
//! agree on every candidate and no chunk-edge prop seam appears.

use super::{ProceduralPlanetTerrain, PropOrientation, PropStamp};
use vv_voxel::{SurfaceChunkKey, CHUNK_SIZE};

impl ProceduralPlanetTerrain {
    /// Return the vox prop instances that should appear in the given chunk.
    /// Props are procedurally derived (deterministic) and are NOT stored in
    /// the voxel grid — this is the authoritative placement query.
    pub fn props_for_chunk(&self, key: SurfaceChunkKey) -> Vec<PropStamp> {
        let planet = self.planet();

        let u0 = key.u_idx * CHUNK_SIZE;
        let v0 = key.v_idx * CHUNK_SIZE;
        let u1 = (u0 + CHUNK_SIZE).min(self.voxel_res);
        let v1 = (v0 + CHUNK_SIZE).min(self.voxel_res);

        let voxel_scale = self.voxel_scale();
        let budget = planet.streaming.feature_budget_per_chunk as usize;
        let mut props = Vec::new();
        let mut emitted_at: Vec<(u32, u32)> = Vec::new(); // for one-prop-per-column

        for scatter_idx in &planet.vox_prop_scatters {
            if budget > 0 && props.len() >= budget {
                break;
            }
            let scatter = &self.registry.vox_prop_scatters[*scatter_idx];
            // Cave scatters are handled by CaveDecorationBakery — skip here.
            if scatter.placement.cave_surface != vv_pack_compiler::CaveSurface::TopSurface {
                continue;
            }
            crate::placement::for_each_candidate(
                &scatter.placement,
                crate::placement::PlacementArea {
                    face: key.face,
                    u_lo: u0,
                    u_hi: u1,
                    v_lo: v0,
                    v_hi: v1,
                    voxel_scale,
                },
                |candidate| {
                    if budget > 0 && props.len() >= budget {
                        return;
                    }
                    // One prop per column max — different scatters can still
                    // share a chunk, just not the same column.
                    if emitted_at
                        .iter()
                        .any(|(u, v)| *u == candidate.pu && *v == candidate.pv)
                    {
                        return;
                    }

                    let surface = self.surface_sample(key.face, candidate.pu, candidate.pv);
                    let biome = self.registry.biome(surface.primary_biome);
                    if !scatter.placement.allowed_in_biome(biome) {
                        self.stats.record_reject();
                        return;
                    }
                    let top = biome.surface.top;
                    if !scatter.placement.surface_blocks.contains(&top) {
                        self.stats.record_reject();
                        return;
                    }
                    if !self.placement_density_hit(&scatter.placement, key.face, &candidate) {
                        return;
                    }
                    if let Some(variant) = scatter.pick_variant(candidate.seed) {
                        // Quarter-turn rotation — the current renderer only
                        // supports 4 cardinal directions for vox props.  The
                        // jittered position itself already breaks the grid.
                        let rotation = ((candidate.rotation / std::f32::consts::TAU * 4.0)
                            .rem_euclid(4.0) as u8)
                            & 3;
                        props.push(PropStamp {
                            face: key.face,
                            u: candidate.pu,
                            v: candidate.pv,
                            surface_layer: surface.height,
                            model_key: variant.model_key.clone(),
                            rotation,
                            orientation: PropOrientation::Floor,
                        });
                        emitted_at.push((candidate.pu, candidate.pv));
                        self.stats.record_prop();
                    }
                },
            );
        }

        // Append cave floor / ceiling props (subsurface column scanning).
        let mut cave = crate::cave_decoration::cave_props_for_chunk(self, key);
        props.append(&mut cave);

        props
    }
}
