use super::chunk_input::{ChunkBorderSamples, ChunkVoxelView};
use super::material_packing::MeshMaterialTable;
use vv_voxel::{VoxelId, CHUNK_SIZE};

/// Wraps the pre-resolved voxel data and material table.
/// Replaces the old `ChunkAccessor<'a>` which pulled data from `PlanetSnapshot`.
pub(crate) struct VoxelAccessor<'a> {
    pub(crate) voxels: &'a ChunkVoxelView,
    pub(crate) materials: &'a MeshMaterialTable,
    pub(crate) border: &'a ChunkBorderSamples,
}

impl<'a> VoxelAccessor<'a> {
    pub(crate) fn new(
        voxels: &'a ChunkVoxelView,
        materials: &'a MeshMaterialTable,
        border: &'a ChunkBorderSamples,
    ) -> Self {
        Self {
            voxels,
            materials,
            border,
        }
    }

    pub(crate) fn voxel_id(&self, layer: u32, u: u32, v: u32) -> VoxelId {
        self.voxels.get(layer, u, v)
    }

    pub(crate) fn has_renderable(&self, layer: u32, u: u32, v: u32) -> bool {
        self.materials.is_renderable(self.voxel_id(layer, u, v))
    }

    pub(crate) fn uses_greedy(&self, layer: u32, u: u32, v: u32) -> bool {
        self.materials.uses_greedy(self.voxel_id(layer, u, v))
    }

    /// Check if neighbor at signed-offset is solid (for face culling + AO).
    /// Returns `true` if coordinate is below layer 0 (planet core = solid).
    pub(crate) fn check_solid(&self, layer: i32, u: i32, v: i32) -> bool {
        match self.voxels.get_signed(layer, u, v) {
            None => true, // core below layer 0
            Some(id) => self.materials.is_opaque_cube(id),
        }
    }

    /// Whether the face between `current` and signed neighbor is hidden.
    pub(crate) fn check_hides(
        &self,
        current: VoxelId,
        n_layer: i32,
        n_u: i32,
        n_v: i32,
    ) -> bool {
        match self.voxels.get_signed(n_layer, n_u, n_v) {
            None => true, // core = hides everything
            Some(other) => self.materials.hides_face_between(current, other),
        }
    }

    pub(crate) fn surface_height(&self, u: u32, v: u32) -> u32 {
        self.border.surface_height(u, v)
    }
}

/// A (face, layer, u, v) coordinate on the planet surface.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Coord {
    pub face: u8,
    pub layer: u32,
    pub u: u32,
    pub v: u32,
}

/// Buffer for the set of voxels that need meshing.
#[derive(Default)]
pub(crate) struct CandidateBuffer {
    coords: Vec<Coord>,
}

impl CandidateBuffer {
    pub(crate) fn with_capacity(n: usize) -> Self {
        Self {
            coords: Vec::with_capacity(n),
        }
    }

    pub(crate) fn push(&mut self, c: Coord) {
        self.coords.push(c);
    }

    /// Sort, dedup, and return the finished list.
    pub(crate) fn finish(mut self) -> Vec<Coord> {
        self.coords
            .sort_by_key(|c| (c.face, c.layer, c.u, c.v));
        self.coords
            .dedup_by_key(|c| (c.face, c.layer, c.u, c.v));
        self.coords
    }
}

/// Build the candidate list from the border-sample height data.
///
/// This mirrors the previous `build_chunk` surface-scan, but operates
/// entirely on pre-resolved data — no world references.
pub(crate) fn build_candidates(
    accessor: &VoxelAccessor<'_>,
    face: u8,
    u_start: u32,
    v_start: u32,
    cliff_fill_depth: u32,
    resolution: u32,
) -> CandidateBuffer {
    let u_end = (u_start + CHUNK_SIZE).min(resolution);
    let v_end = (v_start + CHUNK_SIZE).min(resolution);

    let mut buf = CandidateBuffer::with_capacity((CHUNK_SIZE * CHUNK_SIZE * 2) as usize);

    let gh = |u: u32, v: u32| -> u32 { accessor.surface_height(u, v) };

    for u in u_start..u_end {
        for v in v_start..v_end {
            let h = gh(u, v);
            if h == 0 {
                continue;
            }

            buf.push(Coord { face, layer: h, u, v });

            // Cliff fill: expose blocks down to the lowest neighbor height.
            let mut min_h = h;
            if u > 0 {
                min_h = min_h.min(gh(u - 1, v));
            }
            if u + 1 < resolution {
                min_h = min_h.min(gh(u + 1, v));
            }
            if v > 0 {
                min_h = min_h.min(gh(u, v - 1));
            }
            if v + 1 < resolution {
                min_h = min_h.min(gh(u, v + 1));
            }
            if min_h < h {
                let bottom = min_h.max(h.saturating_sub(cliff_fill_depth));
                for l in (bottom + 1)..h {
                    buf.push(Coord { face, layer: l, u, v });
                }
            }

            // Water surface at sea level.
            let sea = accessor.border.sea_level;
            let water = accessor.border.water_voxel;
            if water != VoxelId::AIR && h < sea {
                buf.push(Coord { face, layer: sea, u, v });
            }
        }
    }

    // Above-surface feature voxels: any non-AIR voxel stored in the view
    // that is above the surface (layer > surface_height).
    for (layer, u, v, _id) in accessor.voxels.iter_voxels() {
        if u >= u_start
            && u < u_end
            && v >= v_start
            && v < v_end
            && layer > gh(u, v)
        {
            buf.push(Coord { face, layer, u, v });
        }
    }

    buf
}

/// Push the 7-neighbourhood of a modified voxel (the voxel itself + its
/// 6-connected neighbors), used to ensure dirty chunks get correct faces.
pub(crate) fn add_modified_candidates(
    face: u8,
    layer: u32,
    u: u32,
    v: u32,
    buf: &mut CandidateBuffer,
    resolution: u32,
) {
    let c = Coord { face, layer, u, v };
    buf.push(c);
    if layer + 1 < resolution {
        buf.push(Coord { layer: layer + 1, ..c });
    }
    if layer > 0 {
        buf.push(Coord { layer: layer - 1, ..c });
    }
    if u > 0 {
        buf.push(Coord { u: u - 1, ..c });
    }
    if u + 1 < resolution {
        buf.push(Coord { u: u + 1, ..c });
    }
    if v > 0 {
        buf.push(Coord { v: v - 1, ..c });
    }
    if v + 1 < resolution {
        buf.push(Coord { v: v + 1, ..c });
    }
}
