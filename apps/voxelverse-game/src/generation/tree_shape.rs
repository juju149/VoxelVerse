use super::{hash4, range_pick, scale_range};
use crate::content::{CompiledTreeShapeKind, CompiledVegetation};
use crate::voxel::VoxelId;
use glam::Vec3;
use std::f32::consts::TAU;
// ---------------------------------------------------------------------------
// Tree shape — natural-looking generation with full per-tree randomness.
// ---------------------------------------------------------------------------

/// One blob in the canopy.  A canopy is a union of overlapping lobes whose
/// boundary is jittered per-voxel — never a pure sphere.
///
/// `radius` is the maximum reach (used for bounds).  `radii` lets the lobe
/// be anisotropic — a flat disc (`y` shrunk) for spruce/acacia layers, a
/// tall ellipsoid (`y` stretched) for jungle crowns, or a sphere when all
/// three components are equal.  Boundary jitter scales with the local
/// radius so big lobes get bigger surface bites than small ones.
#[derive(Clone, Copy, Debug)]
pub struct CanopyLobe {
    pub center: Vec3, // x = u + 0.5, y = layer + 0.5, z = v + 0.5
    pub radius: f32,
    pub radii: Vec3,
    pub jitter_seed: u32,
}

impl CanopyLobe {
    /// Spherical lobe — kept for the BroadLeaf / Tall / JungleCanopy paths
    /// that author one isotropic `r` value.
    pub fn sphere(center: Vec3, radius: f32, jitter_seed: u32) -> Self {
        Self {
            center,
            radius,
            radii: Vec3::splat(radius),
            jitter_seed,
        }
    }
}

/// One branch growing out of the trunk with an arbitrary horizontal angle and
/// a slight upward slope.  May carry a small canopy lobe at its tip.
#[derive(Clone, Copy, Debug)]
pub struct Branch {
    pub start: Vec3,
    pub direction: Vec3, // unit-ish: (cos θ, slope_y, sin θ)
    pub length: f32,
    pub thickness: f32,
    pub tip: Option<CanopyLobe>,
}

/// Full per-tree shape.  Computed once (deterministically from `(face, pu, pv)`
/// + a per-tree seed) so the bakery and the slow path always agree.
///
/// The struct is silhouette-agnostic: every supported family
/// ([`CompiledTreeShapeKind`]) reduces to the same primitives — a tapered
/// trunk + a list of branches + a list of (anisotropic) canopy lobes.
/// `compute` dispatches on the family to populate those primitives
/// differently (rounded oak, conical spruce, flat-top acacia, …) so the
/// stamper / voxel-resolver code is shape-agnostic.
#[derive(Clone, Debug)]
pub struct TreeShape {
    pub plant_height: u32,
    pub trunk_top_layer: u32,
    pub height: u32,

    pub trunk_block: VoxelId,
    pub leaves_block: VoxelId,

    /// Pivot at the base of the trunk (centre of the planted column).
    pub trunk_pivot: Vec3,
    /// Horizontal lean at the very top (curves quadratically along height).
    pub trunk_lean: (f32, f32),
    pub trunk_base_radius: f32,
    pub trunk_top_radius: f32,
    /// Per-voxel boundary jitter scaler for the trunk (0..=1 fraction of r²).
    pub trunk_seed: u32,
    /// 0..=1 keep-rate for canopy leaf voxels (lower = airier crown).
    pub canopy_density: f32,
    /// Per-tree salt for the canopy density coin-toss (different from
    /// `trunk_seed` so trunks and leaves jitter independently).
    pub canopy_seed: u32,

    pub branches: Vec<Branch>,
    pub lobes: Vec<CanopyLobe>,
}

impl TreeShape {
    pub fn compute(
        veg: &CompiledVegetation,
        face: u8,
        pu: u32,
        pv: u32,
        plant_height: u32,
        scale: f32,
    ) -> Self {
        // -- per-tree seeds & dimensions ------------------------------------
        let height = range_pick(scale_range(veg.height, scale), hash4(face, pu, pv, 33)).max(2);
        let thickness = range_pick(
            scale_range(veg.trunk_thickness, scale),
            hash4(face, pu, pv, 35),
        )
        .max(1) as f32;
        let canopy_radius = range_pick(
            scale_range(veg.canopy_radius, scale),
            hash4(face, pu, pv, 34),
        ) as f32;
        let trunk_seed = hash4(face, pu, pv, 0xA17EE5);
        let canopy_seed = hash4(face, pu, pv, 0xCA7E0_DE);

        let trunk_top_layer = plant_height + height;
        let pivot = Vec3::new(pu as f32 + 0.5, plant_height as f32 + 0.5, pv as f32 + 0.5);

        // -- trunk lean (suppressed for shapes that need a straight trunk) --
        let allow_lean = matches!(
            veg.shape_kind,
            CompiledTreeShapeKind::BroadLeaf | CompiledTreeShapeKind::DenseDark
        );
        let lean_factor = if allow_lean { 1.0 } else { 0.25 };
        let lean_max = (height as f32) * veg.trunk_lean_max * lean_factor;
        let lean_theta = hash01(face, pu, pv, 71) * TAU;
        let lean_amount = hash01(face, pu, pv, 72) * lean_max;
        let trunk_lean = (
            lean_theta.cos() * lean_amount,
            lean_theta.sin() * lean_amount,
        );

        // Tapering: the top is narrower than the base. Always at least 0.5 v.
        let trunk_base_radius = (thickness * 0.5).max(0.5);
        let trunk_top_radius = match veg.shape_kind {
            // Conifers / birches keep an almost cylindrical trunk to read clean.
            CompiledTreeShapeKind::Conical
            | CompiledTreeShapeKind::Tall
            | CompiledTreeShapeKind::JungleCanopy => (trunk_base_radius * 0.85).max(0.5),
            // Acacias fork — top trunk radius is meaningless, branches do the work.
            CompiledTreeShapeKind::FlatTop => (trunk_base_radius * 0.7).max(0.5),
            _ => (trunk_base_radius * 0.6).max(0.5),
        };

        // -- per-shape branch + canopy construction ------------------------
        let (branches, lobes) = match veg.shape_kind {
            CompiledTreeShapeKind::Conical => build_conical(
                veg,
                face,
                pu,
                pv,
                plant_height,
                height,
                canopy_radius,
                &pivot,
                scale,
            ),
            CompiledTreeShapeKind::FlatTop => build_flat_top(
                veg,
                face,
                pu,
                pv,
                plant_height,
                height,
                canopy_radius,
                &pivot,
                trunk_base_radius,
                trunk_top_radius,
                scale,
            ),
            // Tall / JungleCanopy / DenseDark all reuse the BroadLeaf family
            // with their authored parameters carrying the variation (lobe
            // count, canopy_radius, branch_count, canopy_squash, …).  This
            // keeps a single rounded-canopy code path for the "tree-with-a-
            // crown" silhouette.
            _ => build_broad_leaf(
                veg,
                face,
                pu,
                pv,
                plant_height,
                height,
                canopy_radius,
                &pivot,
                trunk_base_radius,
                trunk_top_radius,
                trunk_lean,
                trunk_top_layer,
                scale,
            ),
        };

        Self {
            plant_height,
            trunk_top_layer,
            height,
            trunk_block: veg.trunk,
            leaves_block: veg.leaves,
            trunk_pivot: pivot,
            trunk_lean,
            trunk_base_radius,
            trunk_top_radius,
            trunk_seed,
            canopy_density: veg.canopy_density.clamp(0.05, 1.0),
            canopy_seed,
            branches,
            lobes,
        }
    }

    /// Furthest a single tree can reach horizontally from its plant column.
    /// Used by the bakery to size its neighbourhood scan.
    #[allow(dead_code)]
    pub fn horizontal_reach(&self) -> f32 {
        let lean = (self.trunk_lean.0.powi(2) + self.trunk_lean.1.powi(2)).sqrt();
        let canopy = self
            .lobes
            .iter()
            .map(|l| {
                let dx = l.center.x - self.trunk_pivot.x;
                let dz = l.center.z - self.trunk_pivot.z;
                (dx * dx + dz * dz).sqrt() + l.radius
            })
            .fold(0.0_f32, f32::max);
        let branch = self
            .branches
            .iter()
            .map(|b| b.length + b.tip.map(|t| t.radius).unwrap_or(0.0))
            .fold(0.0_f32, f32::max);
        lean + canopy.max(branch).max(self.trunk_base_radius)
    }

    #[allow(dead_code)]
    pub fn vertical_reach_above(&self) -> f32 {
        let canopy = self
            .lobes
            .iter()
            .map(|l| (l.center.y + l.radius) - self.plant_height as f32)
            .fold(self.height as f32, f32::max);
        let branch = self
            .branches
            .iter()
            .map(|b| b.start.y + b.direction.y * b.length + 1.5)
            .map(|y| y - self.plant_height as f32)
            .fold(canopy, f32::max);
        branch
    }

    /// Walk every voxel that belongs to this tree and forward it to `emit`.
    /// The bakery uses this to populate the chunk feature map.
    pub fn stamp(&self, emit: &mut dyn FnMut(i32, i32, i32, VoxelId)) {
        // Trunk: layer-by-layer disc with lean, taper, and per-voxel jitter.
        for layer in (self.plant_height + 1)..=self.trunk_top_layer {
            let t = (layer - self.plant_height) as f32 / self.height.max(1) as f32;
            let cx = self.trunk_pivot.x + self.trunk_lean.0 * t * t;
            let cz = self.trunk_pivot.z + self.trunk_lean.1 * t * t;
            let r = trunk_radius_at(
                layer as f32,
                self.plant_height as f32,
                self.height as f32,
                self.trunk_base_radius,
                self.trunk_top_radius,
            );
            stamp_disc(
                cx,
                cz,
                layer as i32,
                r,
                self.trunk_seed ^ layer,
                self.trunk_block,
                emit,
            );
        }

        // Branches: trunk-block voxels stepped along the branch direction.
        for (bi, branch) in self.branches.iter().enumerate() {
            let steps = (branch.length.max(1.0).ceil()) as i32;
            for s in 1..=steps {
                let p = branch.start + branch.direction * (s as f32);
                let r = branch.thickness * (1.0 - 0.4 * (s as f32 / steps as f32));
                stamp_disc(
                    p.x,
                    p.z,
                    p.y.round() as i32,
                    r.max(0.55),
                    (self.trunk_seed ^ 0x1234)
                        .wrapping_add((bi as u32).wrapping_mul(7919) + s as u32),
                    self.trunk_block,
                    emit,
                );
            }
            if let Some(tip) = branch.tip {
                stamp_lobe(
                    tip,
                    self.leaves_block,
                    self.canopy_density,
                    self.canopy_seed,
                    emit,
                );
            }
        }

        // Canopy: union of jittered lobes.
        for lobe in &self.lobes {
            stamp_lobe(
                *lobe,
                self.leaves_block,
                self.canopy_density,
                self.canopy_seed,
                emit,
            );
        }
    }

    /// Test whether the voxel at `(u, layer, v)` belongs to this tree.
    /// Mirrors `stamp` exactly so the slow path agrees with the baked map.
    pub fn voxel_at(&self, u: i32, layer: i32, v: i32) -> Option<VoxelId> {
        // Trunk
        if (layer as u32) > self.plant_height && (layer as u32) <= self.trunk_top_layer {
            let l = layer as u32;
            let t = (l - self.plant_height) as f32 / self.height.max(1) as f32;
            let cx = self.trunk_pivot.x + self.trunk_lean.0 * t * t;
            let cz = self.trunk_pivot.z + self.trunk_lean.1 * t * t;
            let r = trunk_radius_at(
                layer as f32,
                self.plant_height as f32,
                self.height as f32,
                self.trunk_base_radius,
                self.trunk_top_radius,
            );
            if disc_contains(u, v, cx, cz, r, self.trunk_seed ^ l) {
                return Some(self.trunk_block);
            }
        }
        // Branches
        for (bi, branch) in self.branches.iter().enumerate() {
            let steps = (branch.length.max(1.0).ceil()) as i32;
            for s in 1..=steps {
                let p = branch.start + branch.direction * (s as f32);
                let r = (branch.thickness * (1.0 - 0.4 * (s as f32 / steps as f32))).max(0.55);
                if (p.y.round() as i32) == layer
                    && disc_contains(
                        u,
                        v,
                        p.x,
                        p.z,
                        r,
                        (self.trunk_seed ^ 0x1234)
                            .wrapping_add((bi as u32).wrapping_mul(7919) + s as u32),
                    )
                {
                    return Some(self.trunk_block);
                }
            }
            if let Some(tip) = branch.tip {
                if lobe_contains(tip, u, layer, v, self.canopy_density, self.canopy_seed) {
                    return Some(self.leaves_block);
                }
            }
        }
        // Canopy
        for lobe in &self.lobes {
            if lobe_contains(*lobe, u, layer, v, self.canopy_density, self.canopy_seed) {
                return Some(self.leaves_block);
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Per-shape builders.  Each one returns `(branches, lobes)` for one tree.
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn build_broad_leaf(
    veg: &CompiledVegetation,
    face: u8,
    pu: u32,
    pv: u32,
    plant_height: u32,
    height: u32,
    canopy_radius: f32,
    pivot: &Vec3,
    trunk_base_radius: f32,
    trunk_top_radius: f32,
    trunk_lean: (f32, f32),
    trunk_top_layer: u32,
    scale: f32,
) -> (Vec<Branch>, Vec<CanopyLobe>) {
    let branch_count = range_pick(veg.branch_count, hash4(face, pu, pv, 36));
    let branch_len_max = range_pick(
        scale_range(veg.branch_length, scale),
        hash4(face, pu, pv, 37),
    )
    .max(1) as f32;

    let mut branches = Vec::with_capacity(branch_count as usize);
    if branch_count > 0 && height >= 3 {
        let band_start = plant_height as f32 + (height as f32) * 0.55;
        let band_end = trunk_top_layer as f32 - 0.5;
        for i in 0..branch_count {
            let h = hash4(face, pu, pv, 401 + i);
            let theta = hash01_u32(h, 0) * TAU;
            let raw_len =
                (1.5 + hash01_u32(h, 1) * (branch_len_max + 1.0)).min(branch_len_max + 2.0);
            let layer_t = hash01_u32(h, 2);
            let start_layer = band_start + layer_t * (band_end - band_start).max(0.0);
            let slope_lo = veg.branch_slope.0;
            let slope = slope_lo + hash01_u32(h, 3) * (veg.branch_slope.1 - slope_lo).max(0.0);
            let dir_xz = Vec3::new(theta.cos(), 0.0, theta.sin());
            let trunk_r_at_start = trunk_radius_at(
                start_layer,
                plant_height as f32,
                height as f32,
                trunk_base_radius,
                trunk_top_radius,
            );
            let start = Vec3::new(
                pivot.x + dir_xz.x * trunk_r_at_start * 0.6,
                start_layer,
                pivot.z + dir_xz.z * trunk_r_at_start * 0.6,
            );
            let direction = Vec3::new(dir_xz.x, slope, dir_xz.z).normalize_or_zero();
            let thickness = (trunk_top_radius * 0.55).max(0.5);
            let tip = Some(CanopyLobe::sphere(
                start + direction * raw_len,
                (canopy_radius * 0.35 + hash01_u32(h, 4) * canopy_radius * 0.35).max(1.2),
                h ^ 0xBEEF,
            ));
            branches.push(Branch {
                start,
                direction,
                length: raw_len,
                thickness,
                tip,
            });
        }
    }

    let mut lobes = Vec::new();
    if canopy_radius > 0.0 {
        let n_lobes_seed = hash4(face, pu, pv, 0x10BE);
        let (lobe_min, lobe_max) = veg.canopy_lobe_count;
        let lobe_range = (lobe_max.saturating_sub(lobe_min) + 1).max(1);
        let n_lobes = lobe_min + n_lobes_seed % lobe_range;
        let canopy_anchor = Vec3::new(
            pivot.x + trunk_lean.0,
            trunk_top_layer as f32 - 0.25,
            pivot.z + trunk_lean.1,
        );
        let squash = veg.canopy_vertical_squash.max(0.3);
        for i in 0..n_lobes {
            let h = hash4(face, pu, pv, 0x501E + i);
            let theta = hash01_u32(h, 0) * TAU;
            let r_off = hash01_u32(h, 1) * canopy_radius * 0.55;
            let y_off = (hash01_u32(h, 2) - 0.35) * canopy_radius * squash;
            let lobe_r = canopy_radius * (0.45 + hash01_u32(h, 3) * 0.55);
            let center = canopy_anchor + Vec3::new(theta.cos() * r_off, y_off, theta.sin() * r_off);
            lobes.push(CanopyLobe {
                center,
                radius: lobe_r.max(1.5),
                radii: Vec3::new(lobe_r, lobe_r * squash, lobe_r),
                jitter_seed: h ^ 0xCAFE,
            });
        }
    }
    (branches, lobes)
}

#[allow(clippy::too_many_arguments)]
fn build_conical(
    veg: &CompiledVegetation,
    face: u8,
    pu: u32,
    pv: u32,
    plant_height: u32,
    height: u32,
    canopy_radius: f32,
    pivot: &Vec3,
    scale: f32,
) -> (Vec<Branch>, Vec<CanopyLobe>) {
    // Spruce-style: stack of flat anisotropic discs whose radius shrinks
    // toward the tip.  No branches — the discs themselves are the canopy.
    // The lowest disc starts ~30% up the trunk so the bottom is bare.
    let _ = scale;
    let trunk_top = plant_height + height;
    let canopy_start = plant_height as f32 + (height as f32) * 0.30;
    let canopy_end = trunk_top as f32 + 0.3;
    let span = (canopy_end - canopy_start).max(1.0);

    // 1 disc per ~1.0 voxel of canopy span (clamped to a sensible range).
    let n_discs = (span.round() as u32).clamp(3, 14);
    let (lobe_min, lobe_max) = veg.canopy_lobe_count;
    // Allow data to override the disc count via canopy_lobe_count if the
    // pack writer wants explicit control — averaged with the geometric one.
    let avg_data = (lobe_min + lobe_max) / 2;
    let n_discs = ((n_discs + avg_data.max(1)) / 2).max(3);

    let mut lobes = Vec::with_capacity(n_discs as usize);
    for i in 0..n_discs {
        let t = i as f32 / (n_discs - 1).max(1) as f32; // 0..=1, base→tip
        let layer = canopy_start + t * span;
        // Disc radius shrinks from canopy_radius at base to ~0.6 v at tip.
        // A small per-disc jitter makes the silhouette less mechanical.
        let jitter = (hash01(face, pu, pv, 0x7E1D + i) - 0.5) * 0.6;
        let r = (canopy_radius * (1.0 - t * 0.85) + jitter).max(0.7);
        let disc_thickness = (canopy_radius * 0.18 + 0.35).max(0.5);
        lobes.push(CanopyLobe {
            center: Vec3::new(pivot.x, layer, pivot.z),
            radius: r,
            radii: Vec3::new(r, disc_thickness, r),
            jitter_seed: hash4(face, pu, pv, 0xC02E + i) ^ 0xCAFE,
        });
    }
    // Tip cap: a small leaf bud above the trunk so the silhouette finishes
    // pointed instead of flat.
    lobes.push(CanopyLobe::sphere(
        Vec3::new(pivot.x, canopy_end + 0.25, pivot.z),
        0.9,
        hash4(face, pu, pv, 0xCAFE) ^ 0x77,
    ));
    (Vec::new(), lobes)
}

#[allow(clippy::too_many_arguments)]
fn build_flat_top(
    veg: &CompiledVegetation,
    face: u8,
    pu: u32,
    pv: u32,
    plant_height: u32,
    height: u32,
    canopy_radius: f32,
    pivot: &Vec3,
    trunk_base_radius: f32,
    trunk_top_radius: f32,
    scale: f32,
) -> (Vec<Branch>, Vec<CanopyLobe>) {
    // Acacia: trunk forks at ~70% height into 2-3 angled limbs, each ending
    // in a flat plate of leaves.  Authored branch_count is reinterpreted as
    // "fork count" (clamped 2..=4) and branch_length as the limb length.
    let fork_count = range_pick(veg.branch_count, hash4(face, pu, pv, 36)).clamp(2, 4);
    let branch_len_max = range_pick(
        scale_range(veg.branch_length, scale),
        hash4(face, pu, pv, 37),
    )
    .max(2) as f32;
    let fork_layer = plant_height as f32 + (height as f32) * 0.70;

    let mut branches = Vec::with_capacity(fork_count as usize);
    let theta0 = hash01(face, pu, pv, 0xACAC) * TAU;
    for i in 0..fork_count {
        let theta = theta0 + (i as f32 / fork_count as f32) * TAU;
        let h = hash4(face, pu, pv, 0xACA0 + i);
        let len = (branch_len_max * (0.7 + hash01_u32(h, 1) * 0.6)).max(2.0);
        // Acacia limbs angle steeply upward (slope 0.55..1.1).
        let slope = veg.branch_slope.0.max(0.55) + hash01_u32(h, 2) * 0.4;
        let dir_xz = Vec3::new(theta.cos(), 0.0, theta.sin());
        let trunk_r = trunk_radius_at(
            fork_layer,
            plant_height as f32,
            height as f32,
            trunk_base_radius,
            trunk_top_radius,
        );
        let start = Vec3::new(
            pivot.x + dir_xz.x * trunk_r * 0.4,
            fork_layer,
            pivot.z + dir_xz.z * trunk_r * 0.4,
        );
        let direction = Vec3::new(dir_xz.x, slope, dir_xz.z).normalize_or_zero();
        // Plate of leaves at the tip — wide, thin disc.
        let plate_r = canopy_radius * (0.55 + hash01_u32(h, 3) * 0.35);
        let tip_center = start + direction * len;
        let plate = CanopyLobe {
            center: tip_center,
            radius: plate_r,
            radii: Vec3::new(plate_r, (canopy_radius * 0.18 + 0.4).max(0.5), plate_r),
            jitter_seed: h ^ 0xACAC,
        };
        branches.push(Branch {
            start,
            direction,
            length: len,
            thickness: (trunk_top_radius * 0.55).max(0.5),
            tip: Some(plate),
        });
    }

    // A flat top "halo" disc at the very top so the silhouette reads as a
    // single plate when limbs fan out, not as separate puffs.
    let halo = CanopyLobe {
        center: Vec3::new(pivot.x, plant_height as f32 + height as f32 + 0.1, pivot.z),
        radius: canopy_radius * 0.85,
        radii: Vec3::new(
            canopy_radius * 0.85,
            (canopy_radius * 0.16 + 0.3).max(0.5),
            canopy_radius * 0.85,
        ),
        jitter_seed: hash4(face, pu, pv, 0xACAC) ^ 0xF0F0,
    };
    (branches, vec![halo])
}

// ---- shape primitives -----------------------------------------------------

fn trunk_radius_at(
    layer: f32,
    plant_height: f32,
    total_height: f32,
    base_r: f32,
    top_r: f32,
) -> f32 {
    let h = total_height.max(0.001);
    let t = ((layer - plant_height) / h).clamp(0.0, 1.0);
    // Slight bulge near the base for natural taper (square-root falloff).
    let bulge = (1.0 - t).sqrt();
    top_r + (base_r - top_r) * bulge
}

fn stamp_disc(
    cx: f32,
    cz: f32,
    layer: i32,
    r: f32,
    seed: u32,
    block: VoxelId,
    emit: &mut dyn FnMut(i32, i32, i32, VoxelId),
) {
    let r2 = r * r;
    let bound = r.ceil() as i32 + 1;
    let bx = cx.floor() as i32;
    let bz = cz.floor() as i32;
    for du in -bound..=bound {
        for dv in -bound..=bound {
            let u = bx + du;
            let v = bz + dv;
            let dx = u as f32 + 0.5 - cx;
            let dz = v as f32 + 0.5 - cz;
            let d2 = dx * dx + dz * dz;
            // Boundary jitter: ±0.45 v² perturbation breaks the perfect circle.
            let j = (hash01_u32(seed, ((du as u32) << 16) ^ (dv as u32 & 0xFFFF)) - 0.5) * 0.45;
            if d2 <= r2 + j {
                emit(u, layer, v, block);
            }
        }
    }
}

fn disc_contains(u: i32, v: i32, cx: f32, cz: f32, r: f32, seed: u32) -> bool {
    let dx = u as f32 + 0.5 - cx;
    let dz = v as f32 + 0.5 - cz;
    let d2 = dx * dx + dz * dz;
    let bx = cx.floor() as i32;
    let bz = cz.floor() as i32;
    let du = u - bx;
    let dv = v - bz;
    let j = (hash01_u32(seed, ((du as u32) << 16) ^ (dv as u32 & 0xFFFF)) - 0.5) * 0.45;
    d2 <= r * r + j
}

/// Anisotropic-aware lobe surface test.  Returns the (jittered) implicit
/// function value scaled so that `<= 0.0` means "inside".  Both `stamp_lobe`
/// and `lobe_contains` use it so they always agree.
fn lobe_implicit(lobe: &CanopyLobe, u: i32, l: i32, v: i32) -> f32 {
    let dx = u as f32 + 0.5 - lobe.center.x;
    let dy = l as f32 + 0.5 - lobe.center.y;
    let dz = v as f32 + 0.5 - lobe.center.z;
    let rx = lobe.radii.x.max(0.4);
    let ry = lobe.radii.y.max(0.4);
    let rz = lobe.radii.z.max(0.4);
    // Squared elliptical distance (1.0 on the surface).
    let nx = dx / rx;
    let ny = dy / ry;
    let nz = dz / rz;
    let d2 = nx * nx + ny * ny + nz * nz;
    let bx = lobe.center.x.floor() as i32;
    let by = lobe.center.y.floor() as i32;
    let bz = lobe.center.z.floor() as i32;
    let du = u - bx;
    let dl = l - by;
    let dv = v - bz;
    // Boundary jitter expressed in normalized space (so it works for discs
    // and spheres alike).  ±0.20 of the unit-radius surface.
    let j = (hash01_u32(
        lobe.jitter_seed,
        ((du as u32) << 20) ^ ((dl as u32) << 10) ^ (dv as u32 & 0x3FF),
    ) - 0.5)
        * 0.40;
    d2 - 1.0 - j
}

fn voxel_offset_key(du: i32, dl: i32, dv: i32) -> u32 {
    ((du as u32) << 20) ^ ((dl as u32) << 10) ^ (dv as u32 & 0x3FF)
}

fn stamp_lobe(
    lobe: CanopyLobe,
    block: VoxelId,
    canopy_density: f32,
    canopy_seed: u32,
    emit: &mut dyn FnMut(i32, i32, i32, VoxelId),
) {
    let bound_x = lobe.radii.x.ceil() as i32 + 1;
    let bound_y = lobe.radii.y.ceil() as i32 + 1;
    let bound_z = lobe.radii.z.ceil() as i32 + 1;
    let bx = lobe.center.x.floor() as i32;
    let by = lobe.center.y.floor() as i32;
    let bz = lobe.center.z.floor() as i32;
    for du in -bound_x..=bound_x {
        for dl in -bound_y..=bound_y {
            for dv in -bound_z..=bound_z {
                let u = bx + du;
                let l = by + dl;
                let v = bz + dv;
                if lobe_implicit(&lobe, u, l, v) <= 0.0 {
                    let key = voxel_offset_key(du, dl, dv);
                    if canopy_density < 0.999
                        && hash01_u32(canopy_seed ^ lobe.jitter_seed, key) >= canopy_density
                    {
                        continue;
                    }
                    emit(u, l, v, block);
                }
            }
        }
    }
}

fn lobe_contains(
    lobe: CanopyLobe,
    u: i32,
    layer: i32,
    v: i32,
    canopy_density: f32,
    canopy_seed: u32,
) -> bool {
    if lobe_implicit(&lobe, u, layer, v) > 0.0 {
        return false;
    }
    if canopy_density >= 0.999 {
        return true;
    }
    let bx = lobe.center.x.floor() as i32;
    let by = lobe.center.y.floor() as i32;
    let bz = lobe.center.z.floor() as i32;
    let key = voxel_offset_key(u - bx, layer - by, v - bz);
    hash01_u32(canopy_seed ^ lobe.jitter_seed, key) < canopy_density
}

fn hash01(face: u8, u: u32, v: u32, salt: u32) -> f32 {
    hash4(face, u, v, salt) as f32 / u32::MAX as f32
}

fn hash01_u32(base: u32, salt: u32) -> f32 {
    let mut x = base ^ salt.wrapping_mul(0x9E37_79B9);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    x as f32 / u32::MAX as f32
}
