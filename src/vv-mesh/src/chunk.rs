use std::collections::HashSet;

use vv_registry::{BlockId as ContentBlockId, BlockRenderSource};
use vv_voxel::{BlockId, ChunkKey, CHUNK_SIZE};
use vv_world_runtime::{ChunkMods, PlanetData};

use crate::{overlay::FeatureOverlay, MeshGen, Vertex};

const TERRAIN_VISIBLE_SHELL_LAYERS: u32 = 1;

impl MeshGen {
    pub fn build_chunk(
        key: ChunkKey,
        data: &PlanetData,
        blocks: &impl BlockRenderSource,
    ) -> (Vec<Vertex>, Vec<u32>) {
        let res = data.resolution;

        let u_start = key.u_idx * CHUNK_SIZE;
        let v_start = key.v_idx * CHUNK_SIZE;
        let u_end = (u_start + CHUNK_SIZE).min(res);
        let v_end = (v_start + CHUNK_SIZE).min(res);

        let overlay = FeatureOverlay::new(data, key);

        let chunk_area = ((u_end - u_start) * (v_end - v_start)) as usize;
        let mut candidates: HashSet<BlockId> =
            HashSet::with_capacity(chunk_area.saturating_mul(4).saturating_add(overlay.len()));

        let get_h = |u: u32, v: u32| -> u32 {
            if u >= res || v >= res {
                return 0;
            }

            data.terrain.get_height(key.face, u, v)
        };

        for u in u_start..u_end {
            for v in v_start..v_end {
                let h = get_h(u, v);

                if h == 0 {
                    continue;
                }

                let shell_bottom = h.saturating_sub(TERRAIN_VISIBLE_SHELL_LAYERS);

                for layer in shell_bottom..=h {
                    candidates.insert(BlockId {
                        face: key.face,
                        layer,
                        u,
                        v,
                    });
                }

                let mut min_h = h;

                if u > 0 {
                    min_h = min_h.min(get_h(u - 1, v));
                }

                if u < res - 1 {
                    min_h = min_h.min(get_h(u + 1, v));
                }

                if v > 0 {
                    min_h = min_h.min(get_h(u, v - 1));
                }

                if v < res - 1 {
                    min_h = min_h.min(get_h(u, v + 1));
                }

                if min_h < h {
                    let bottom = min_h.max(h.saturating_sub(20));

                    for layer in (bottom + 1)..h {
                        candidates.insert(BlockId {
                            face: key.face,
                            layer,
                            u,
                            v,
                        });
                    }
                }
            }
        }

        candidates.extend(overlay.target_feature_keys());

        if let Some(mods) = data.chunks.get(&key) {
            for &id in mods.placed.keys() {
                candidates.insert(id);
            }

            Self::add_mined_candidates(mods, &mut candidates, res);
        }

        for n_key in Self::neighbor_chunk_keys(key) {
            if let Some(mods) = data.chunks.get(&n_key) {
                Self::add_mined_candidates(mods, &mut candidates, res);
            }
        }

        let mut verts = Vec::with_capacity(candidates.len().saturating_mul(16));
        let mut inds = Vec::with_capacity(candidates.len().saturating_mul(24));
        let mut idx = 0u32;

        for id in candidates {
            if !overlay.is_in_target(id) {
                continue;
            }

            let Some(block_id) = Self::mesh_block_at(data, id, &overlay) else {
                continue;
            };

            Self::add_voxel(
                id, block_id, data, blocks, &overlay, &mut verts, &mut inds, &mut idx,
            );
        }

        (verts, inds)
    }

    #[inline]
    fn neighbor_chunk_keys(key: ChunkKey) -> [ChunkKey; 4] {
        [
            ChunkKey {
                u_idx: key.u_idx.wrapping_sub(1),
                ..key
            },
            ChunkKey {
                u_idx: key.u_idx + 1,
                ..key
            },
            ChunkKey {
                v_idx: key.v_idx.wrapping_sub(1),
                ..key
            },
            ChunkKey {
                v_idx: key.v_idx + 1,
                ..key
            },
        ]
    }

    fn add_mined_candidates(mods: &ChunkMods, candidates: &mut HashSet<BlockId>, res: u32) {
        for &id in &mods.mined {
            candidates.insert(BlockId {
                layer: id.layer + 1,
                ..id
            });

            if id.layer > 0 {
                candidates.insert(BlockId {
                    layer: id.layer - 1,
                    ..id
                });
            }

            if id.u > 0 {
                candidates.insert(BlockId { u: id.u - 1, ..id });
            }

            if id.u < res - 1 {
                candidates.insert(BlockId { u: id.u + 1, ..id });
            }

            if id.v > 0 {
                candidates.insert(BlockId { v: id.v - 1, ..id });
            }

            if id.v < res - 1 {
                candidates.insert(BlockId { v: id.v + 1, ..id });
            }
        }
    }

    #[inline]
    pub(crate) fn mesh_block_at(
        data: &PlanetData,
        id: BlockId,
        overlay: &FeatureOverlay,
    ) -> Option<ContentBlockId> {
        let key = PlanetData::chunk_key(id);

        if let Some(mods) = data.chunks.get(&key) {
            if let Some(block_id) = mods.placed.get(&id) {
                return Some(*block_id);
            }

            if mods.mined.contains(&id) {
                return None;
            }
        }

        if let Some(block_id) = overlay.get(id) {
            return Some(block_id);
        }

        if id.u >= data.resolution || id.v >= data.resolution {
            return None;
        }

        let height = data.terrain.get_height(id.face, id.u, id.v);

        if id.layer <= height {
            Some(data.terrain.get_block(id.face, id.u, id.v, id.layer))
        } else {
            None
        }
    }
}
