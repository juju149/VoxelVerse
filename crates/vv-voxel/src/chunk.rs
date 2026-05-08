use crate::{LocalVoxelCoord, VoxelCoord, VoxelId, CHUNK_SIZE};

pub const CHUNK_VOLUME: usize =
    (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize);

#[derive(Clone)]
pub struct VoxelChunk {
    overrides: Box<[VoxelId]>,
    override_count: u16,
}

impl VoxelChunk {
    pub fn new() -> Self {
        Self {
            overrides: vec![VoxelId::UNSET; CHUNK_VOLUME].into_boxed_slice(),
            override_count: 0,
        }
    }

    pub fn get_override(&self, local: LocalVoxelCoord) -> Option<VoxelId> {
        let voxel = self.overrides[Self::index(local)];
        (!voxel.is_unset()).then_some(voxel)
    }

    pub fn set_override(&mut self, local: LocalVoxelCoord, voxel: Option<VoxelId>) {
        let idx = Self::index(local);
        let old = self.overrides[idx];
        let new = voxel.unwrap_or(VoxelId::UNSET);

        if old.is_unset() && !new.is_unset() {
            self.override_count += 1;
        } else if !old.is_unset() && new.is_unset() {
            self.override_count -= 1;
        }

        self.overrides[idx] = new;
    }

    pub fn is_empty(&self) -> bool {
        self.override_count == 0
    }

    pub fn iter_overrides(
        &self,
        face: u8,
        layer_base: u32,
        u_base: u32,
        v_base: u32,
    ) -> impl Iterator<Item = (VoxelCoord, VoxelId)> + '_ {
        self.overrides
            .iter()
            .copied()
            .enumerate()
            .filter_map(move |(idx, voxel)| {
                if voxel.is_unset() {
                    return None;
                }

                let local = Self::coord_from_index(idx);
                Some((
                    VoxelCoord {
                        face,
                        layer: layer_base + local.layer as u32,
                        u: u_base + local.u as u32,
                        v: v_base + local.v as u32,
                    },
                    voxel,
                ))
            })
    }

    #[inline]
    fn index(local: LocalVoxelCoord) -> usize {
        let layer = local.layer as usize;
        let u = local.u as usize;
        let v = local.v as usize;
        (layer * CHUNK_SIZE as usize * CHUNK_SIZE as usize) + (v * CHUNK_SIZE as usize) + u
    }

    #[inline]
    fn coord_from_index(idx: usize) -> LocalVoxelCoord {
        let plane = CHUNK_SIZE as usize * CHUNK_SIZE as usize;
        let layer = idx / plane;
        let rem = idx % plane;
        let v = rem / CHUNK_SIZE as usize;
        let u = rem % CHUNK_SIZE as usize;

        LocalVoxelCoord {
            layer: layer as u8,
            u: u as u8,
            v: v as u8,
        }
    }
}
