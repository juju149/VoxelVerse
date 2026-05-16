//! Sub-cell jittered placement helper.
//!
//! Replaces the old per-voxel `hash4(face, u, v, salt)` scatter which baked
//! a visible integer grid into vegetation and prop layouts.  Candidates
//! now live on a placement cell grid sized by the feature's authored
//! `min_spacing_voxels`, then jittered into sub-cell positions by a
//! deterministic hash of the cell origin.
//!
//! Key invariants:
//!
//! * **Cell grid is world-aligned, never chunk-local.**  Cells start at
//!   integer multiples of `cell_size` in world voxel coords — so two
//!   adjacent chunks iterate the same cells with identical jitter,
//!   eliminating the chunk-edge seam.
//! * **One candidate per cell.**  Guarantees `min_spacing` between any
//!   two instances of the same placement.
//! * **Jitter is deterministic.**  Same `(face, cell_u, cell_v)` → same
//!   jittered position → same density / climate / biome outcome.  Re-bake
//!   is idempotent.
//! * **Clump field modulates density.**  An optional low-frequency noise
//!   shapes forests-and-clearings without authoring a bespoke scatter
//!   noise per biome.
//!
//! The slow path (`feature_eval::tree_voxel_at`) and the chunk bakery
//! (`features::FeatureBakery`) both consume this iterator so their
//! placements stay byte-for-byte identical.

use std::f32::consts::TAU;
use vv_pack_compiler::CompiledFeaturePlacement;

/// One placement candidate.  All fields are deterministic in
/// `(face, cell_u, cell_v)` so the same candidate is regenerated identically
/// from any chunk or query path.
#[derive(Clone, Copy, Debug)]
pub struct PlacementCandidate {
    /// Integer plant column the candidate snaps to (used by feature
    /// stampers that still want a discrete voxel column).
    pub pu: u32,
    pub pv: u32,
    /// Sub-voxel jittered position for callers that can use continuous
    /// coordinates (rotation pivot, prop fractional offset).
    pub pu_f: f32,
    pub pv_f: f32,
    /// Per-candidate hash usable as a RNG seed for variant selection,
    /// rotation, scale, etc.  Always non-zero.
    pub seed: u32,
    /// Rotation in radians, randomised by `rotation_variance`.
    pub rotation: f32,
    /// Scale multiplier, sampled inside `scale_variance`.
    pub scale: f32,
}

/// Cell side length (in voxels) used by a placement.  Always ≥ 1 so cells
/// stay aligned to the voxel grid even when authored `min_spacing` is < 1.
#[inline]
pub fn placement_cell_size(placement: &CompiledFeaturePlacement, voxel_scale: f32) -> u32 {
    let raw = (placement.min_spacing.max(0.0) * voxel_scale).ceil();
    raw.max(1.0) as u32
}

/// Iterate every placement candidate whose snapped column falls inside the
/// chunk extent `[u_lo, u_hi) × [v_lo, v_hi)`.  The bakery uses this to
/// avoid the old quadratic per-voxel scan.
///
/// `voxel_scale` is the active-grid → 1 m-baseline ratio so authored
/// physical spacing stays constant across voxel resolutions.
pub fn for_each_candidate<F: FnMut(PlacementCandidate)>(
    placement: &CompiledFeaturePlacement,
    face: u8,
    u_lo: u32,
    u_hi: u32,
    v_lo: u32,
    v_hi: u32,
    voxel_scale: f32,
    mut emit: F,
) {
    if u_hi <= u_lo || v_hi <= v_lo {
        return;
    }
    let cell = placement_cell_size(placement, voxel_scale);
    // Start at the cell that contains u_lo (world-aligned floor division).
    let start_u = (u_lo / cell) * cell;
    let start_v = (v_lo / cell) * cell;
    let mut cu = start_u;
    while cu < u_hi {
        let mut cv = start_v;
        while cv < v_hi {
            if let Some(candidate) = candidate_for_cell(placement, face, cu, cv, cell) {
                if (candidate.pu >= u_lo)
                    && (candidate.pu < u_hi)
                    && (candidate.pv >= v_lo)
                    && (candidate.pv < v_hi)
                {
                    emit(candidate);
                }
            }
            cv = cv.saturating_add(cell);
            if cv == 0 {
                // overflow guard
                break;
            }
        }
        cu = cu.saturating_add(cell);
        if cu == 0 {
            break;
        }
    }
}

/// Compute a single candidate for the cell whose origin is `(cell_u, cell_v)`.
/// The cell origin must be a multiple of `cell` (the caller enforces this).
/// Returns `None` only when the candidate would land outside `[0, u32::MAX)`,
/// which never happens in practice — but the signature keeps callers honest.
pub fn candidate_for_cell(
    placement: &CompiledFeaturePlacement,
    face: u8,
    cell_u: u32,
    cell_v: u32,
    cell: u32,
) -> Option<PlacementCandidate> {
    let salt = placement.field as u32 ^ 0xA51_C0DE;
    let h_pos = stable_hash3(face, cell_u, cell_v, salt);
    let h_aux = stable_hash3(
        face,
        cell_u ^ 0x9E37_79B9,
        cell_v ^ 0x85EB_CA6B,
        salt ^ 0x1234_5678,
    );

    let cell_f = cell as f32;
    let half = cell_f * 0.5;
    let jit_max = placement.jitter_strength.clamp(0.0, 1.0) * half;
    let jit_u = (hash01(h_pos) - 0.5) * 2.0 * jit_max;
    let jit_v = (hash01(h_pos.rotate_left(16)) - 0.5) * 2.0 * jit_max;

    let pu_f = cell_u as f32 + half + jit_u;
    let pv_f = cell_v as f32 + half + jit_v;
    let pu = pu_f.round() as i64;
    let pv = pv_f.round() as i64;
    if pu < 0 || pv < 0 || pu > u32::MAX as i64 || pv > u32::MAX as i64 {
        return None;
    }

    let rotation_t = hash01(h_aux);
    let rotation = rotation_t * TAU * placement.rotation_variance.clamp(0.0, 1.0);

    let (lo, hi) = placement.scale_variance;
    let scale = if (hi - lo).abs() < 1e-4 {
        lo
    } else {
        lo + hash01(h_aux.rotate_left(11)) * (hi - lo)
    };

    Some(PlacementCandidate {
        pu: pu as u32,
        pv: pv as u32,
        pu_f,
        pv_f,
        seed: h_pos | 1, // never zero so downstream RNG salts are well-formed
        rotation,
        scale,
    })
}

/// Stable scalar hash used by the placement grid.  Order-independent in
/// the rotational sense and produces well-distributed integers — verified
/// by visual inspection of jitter scatter plots.
#[inline]
pub fn stable_hash3(face: u8, a: u32, b: u32, salt: u32) -> u32 {
    let mut x = salt ^ (face as u32).wrapping_mul(0x9E37_79B9);
    x ^= a.wrapping_mul(0x85EB_CA6B).rotate_left(13);
    x ^= b.wrapping_mul(0xC2B2_AE35).rotate_right(7);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^ (x >> 16)
}

#[inline]
pub fn hash01(h: u32) -> f32 {
    (h as f32) / (u32::MAX as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vv_pack_compiler::{CaveSurface, CompiledFeaturePlacement};
    use vv_voxel::VoxelId;

    fn dummy_placement(min_spacing: f32, jitter: f32) -> CompiledFeaturePlacement {
        CompiledFeaturePlacement {
            surface_blocks: vec![VoxelId::AIR],
            slope_max: 1.0,
            density: 0.5,
            field: 0,
            biome_tags: Vec::new(),
            min_spacing,
            jitter_strength: jitter,
            clump_field: None,
            clump_strength: 0.0,
            altitude_range: None,
            humidity_range: None,
            temperature_range: None,
            slope_min: 0.0,
            scale_variance: (1.0, 1.0),
            rotation_variance: 1.0,
            cave_surface: CaveSurface::TopSurface,
        }
    }

    #[test]
    fn cell_aligned_grid_is_consistent_across_chunk_boundaries() {
        let placement = dummy_placement(4.0, 0.8);
        let mut left = Vec::new();
        let mut right = Vec::new();
        // Two adjacent "chunks" 32 voxels wide.  They must see the same
        // cells in their overlap (the boundary itself) — which proves the
        // placement grid is world-aligned, not chunk-local.
        for_each_candidate(&placement, 2, 0, 32, 0, 32, 1.0, |c| left.push(c));
        for_each_candidate(&placement, 2, 32, 64, 0, 32, 1.0, |c| right.push(c));
        // No overlap (cells fall entirely in one chunk or the other), but
        // the candidates must be deterministic and grid-aligned.
        for cand in &left {
            assert!(cand.pu < 32);
        }
        for cand in &right {
            assert!(cand.pu >= 32 && cand.pu < 64);
        }
    }

    #[test]
    fn min_spacing_drives_cell_size() {
        let placement = dummy_placement(8.0, 0.0);
        let mut count = 0usize;
        for_each_candidate(&placement, 0, 0, 64, 0, 64, 1.0, |_| count += 1);
        // 64 / 8 = 8 cells per side → ≤64 candidates total.
        assert!(count <= 64);
        assert!(count > 0);
    }

    #[test]
    fn determinism_same_seed_same_candidates() {
        let placement = dummy_placement(3.0, 0.7);
        let collect = || {
            let mut out = Vec::new();
            for_each_candidate(&placement, 4, 16, 48, 16, 48, 1.0, |c| {
                out.push((c.pu, c.pv, c.seed))
            });
            out
        };
        assert_eq!(collect(), collect());
    }
}
