//! Cave decoration bakery.
//!
//! Places vox props on cave floors and ceilings.  Surface props (grass,
//! flowers, trees) are handled by `ProceduralPlanetTerrain::props_for_chunk`;
//! cave props need subsurface column scanning which is too expensive to inline
//! there, so this focused module owns that responsibility.
//!
//! # Algorithm
//!
//! For each prop scatter that declares `cave_surface: cave_floor` or
//! `cave_surface: cave_ceiling`:
//!
//! 1. Use `placement::for_each_candidate` to get deterministic candidate
//!    columns — same grid as surface props, same hash — no chunk-edge seam.
//! 2. For each candidate column, walk downward from
//!    `surface_height - MIN_DEPTH` in `SCAN_STRIDE`-layer steps.
//! 3. Detect transitions:
//!    - **Floor**: previous sample was cave-air, current sample is solid →
//!      place floor prop at the solid layer.
//!    - **Ceiling**: previous sample was solid, current sample is cave-air →
//!      place ceiling prop at the solid layer (= transition layer + 1).
//! 4. Break after the first matching transition so one column gets at most
//!    one decoration per scatter rule.
//!
//! # Performance
//!
//! A 32×32 chunk with `MAX_SCAN_LAYERS = 64` and `SCAN_STRIDE = 2` yields at
//! most 32×32×32 = 32 768 `voxel_at` calls.  Each call is a cached
//! atomic-load + one noise pass.  In non-cave regions the scan usually finds
//! all-solid and exits after the first couple of steps.  In practice this
//! costs < 2 ms per chunk on a modern CPU, well within the async budget.

use crate::placement::{for_each_candidate, PlacementArea};
use crate::procedural::{ProceduralPlanetTerrain, PropOrientation, PropStamp};
use vv_pack_compiler::{CaveSurface, CompiledVoxPropScatter};
use vv_voxel::{SurfaceChunkKey, VoxelCoord, VoxelId, CHUNK_SIZE};

/// Minimum depth below the terrain surface before we start scanning.
/// Matches the cap-thickness guard in `voxel_resolver::is_cave` (4 voxels)
/// plus a small margin.
const MIN_DEPTH_FROM_SURFACE: u32 = 6;

/// Maximum layers below the surface scanned per column.  Beyond this depth
/// cave content is assumed absent or not relevant to the player's view.
const MAX_SCAN_LAYERS: u32 = 64;

/// Layer sampling stride.  2 = sample every other layer.  The smallest cave
/// carver radius in the default pack is 2 voxels, so stride 2 cannot miss a
/// cave opening.
const SCAN_STRIDE: u32 = 2;

/// Entry point called by `ProceduralPlanetTerrain::props_for_chunk` after
/// surface props have been collected.  Returns additional `PropStamp`s for
/// cave floors and ceilings.
pub(crate) fn cave_props_for_chunk(
    terrain: &ProceduralPlanetTerrain,
    key: SurfaceChunkKey,
) -> Vec<PropStamp> {
    let planet = terrain.planet();

    // Partition registered scatters into floor and ceiling buckets.
    let floor_scatters: Vec<&CompiledVoxPropScatter> = planet
        .vox_prop_scatters
        .iter()
        .map(|&idx| &terrain.registry().vox_prop_scatters[idx])
        .filter(|s| s.placement.cave_surface == CaveSurface::CaveFloor)
        .collect();
    let ceil_scatters: Vec<&CompiledVoxPropScatter> = planet
        .vox_prop_scatters
        .iter()
        .map(|&idx| &terrain.registry().vox_prop_scatters[idx])
        .filter(|s| s.placement.cave_surface == CaveSurface::CaveCeiling)
        .collect();

    if floor_scatters.is_empty() && ceil_scatters.is_empty() {
        return Vec::new();
    }

    let u0 = key.u_idx * CHUNK_SIZE;
    let v0 = key.v_idx * CHUNK_SIZE;
    let u1 = (u0 + CHUNK_SIZE).min(terrain.voxel_res());
    let v1 = (v0 + CHUNK_SIZE).min(terrain.voxel_res());

    let voxel_scale = terrain.voxel_scale();
    let budget = planet.streaming.feature_budget_per_chunk as usize;
    let profile = terrain.profile();

    let mut props: Vec<PropStamp> = Vec::new();

    // ---- FLOOR scatters --------------------------------------------------

    for scatter in &floor_scatters {
        if budget > 0 && props.len() >= budget {
            break;
        }
        for_each_candidate(
            &scatter.placement,
            PlacementArea {
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
                if !terrain.placement_density_hit(&scatter.placement, key.face, &candidate) {
                    return;
                }

                let surface_h = terrain.get_height(key.face, candidate.pu, candidate.pv);
                let scan_top = surface_h.saturating_sub(MIN_DEPTH_FROM_SURFACE);
                let scan_bot = surface_h.saturating_sub(MIN_DEPTH_FROM_SURFACE + MAX_SCAN_LAYERS);

                // Walk downward; look for cave-air above a solid block (floor).
                let mut prev_is_air = false;
                let mut layer = scan_top;
                loop {
                    let coord = VoxelCoord {
                        face: key.face,
                        layer,
                        u: candidate.pu,
                        v: candidate.pv,
                    };
                    let voxel = terrain.voxel_at(coord, profile);
                    let is_air = voxel == VoxelId::AIR;

                    if prev_is_air && !is_air {
                        // Cave air above → solid here: this is a cave floor.
                        if let Some(variant) =
                            scatter.pick_variant(candidate.seed ^ layer ^ 0x000F_100F)
                        {
                            let rotation = ((candidate.rotation / std::f32::consts::TAU * 4.0)
                                .rem_euclid(4.0) as u8)
                                & 3;
                            props.push(PropStamp {
                                face: key.face,
                                u: candidate.pu,
                                v: candidate.pv,
                                surface_layer: layer,
                                model_key: variant.model_key.clone(),
                                rotation,
                                orientation: PropOrientation::Floor,
                            });
                        }
                        return; // one decoration per column per scatter
                    }

                    prev_is_air = is_air;
                    if layer < scan_bot.saturating_add(SCAN_STRIDE) || layer == 0 {
                        break;
                    }
                    layer -= SCAN_STRIDE;
                }
            },
        );
    }

    // ---- CEILING scatters ------------------------------------------------

    for scatter in &ceil_scatters {
        if budget > 0 && props.len() >= budget {
            break;
        }
        for_each_candidate(
            &scatter.placement,
            PlacementArea {
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
                if !terrain.placement_density_hit(&scatter.placement, key.face, &candidate) {
                    return;
                }

                let surface_h = terrain.get_height(key.face, candidate.pu, candidate.pv);
                let scan_top = surface_h.saturating_sub(MIN_DEPTH_FROM_SURFACE);
                let scan_bot = surface_h.saturating_sub(MIN_DEPTH_FROM_SURFACE + MAX_SCAN_LAYERS);

                // Walk downward; look for solid above cave-air (ceiling).
                // At scan_top the terrain is solid, so prev_is_solid = true.
                let mut prev_is_solid = true;
                let mut layer = scan_top;
                loop {
                    let coord = VoxelCoord {
                        face: key.face,
                        layer,
                        u: candidate.pu,
                        v: candidate.pv,
                    };
                    let voxel = terrain.voxel_at(coord, profile);
                    let is_air = voxel == VoxelId::AIR;

                    if prev_is_solid && is_air {
                        // Solid above → cave-air here: the block at `layer + 1` is the ceiling.
                        let ceiling_layer = layer + 1;
                        if ceiling_layer <= scan_top {
                            if let Some(variant) =
                                scatter.pick_variant(candidate.seed ^ layer ^ 0xCEE1_CAFE)
                            {
                                let rotation = ((candidate.rotation / std::f32::consts::TAU * 4.0)
                                    .rem_euclid(4.0)
                                    as u8)
                                    & 3;
                                props.push(PropStamp {
                                    face: key.face,
                                    u: candidate.pu,
                                    v: candidate.pv,
                                    surface_layer: ceiling_layer,
                                    model_key: variant.model_key.clone(),
                                    rotation,
                                    orientation: PropOrientation::Ceiling,
                                });
                            }
                        }
                        return; // one decoration per column per scatter
                    }

                    prev_is_solid = !is_air;
                    if layer < scan_bot.saturating_add(SCAN_STRIDE) || layer == 0 {
                        break;
                    }
                    layer -= SCAN_STRIDE;
                }
            },
        );
    }

    props
}
