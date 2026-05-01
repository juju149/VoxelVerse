use glam::Vec3;
use std::collections::HashSet;
use std::time::Instant;

use vv_core::{BlockId, ChunkKey, LodKey, CHUNK_SIZE};
use vv_mesh::MeshGen;
use vv_planet::CoordSystem;
use vv_world_runtime::PlanetData;

use crate::AnyKey;

use super::types::{MeshJobKind, MeshJobResult};
use super::Renderer;

impl<'a> Renderer<'a> {
    pub fn update_view(&mut self, player_pos: Vec3, planet: &PlanetData) {
        let lod_start = Instant::now();
        let res = planet.resolution;
        let player_id = CoordSystem::pos_to_id(player_pos, planet.geometry);

        let mut upload_count = 0;
        while upload_count < self.lod_cfg.lod_uploads_per_frame
            && self.frame_telemetry.gpu.uploads < self.lod_cfg.max_gpu_uploads_per_frame
        {
            let Ok((job, v, i)) = self.lod_rx.try_recv() else {
                break;
            };
            self.pending_lods.remove(&job.key);
            self.record_mesh_job(job.duration, job.vertices, job.indices, MeshJobKind::Lod);
            if !v.is_empty() {
                self.upload_lod_buffer(job.key, v, i);
                self.frame_telemetry.lod.lods_uploaded += 1;
                upload_count += 1;
            }
        }

        let mut raw_required_voxels: HashSet<ChunkKey> = HashSet::new();
        let mut raw_required_lods: HashSet<LodKey> = HashSet::new();
        let logical_size = res.next_power_of_two();

        for face in 0u8..6 {
            self.process_quadtree(
                face,
                0,
                0,
                logical_size,
                player_pos,
                planet,
                player_id,
                &mut raw_required_voxels,
                &mut raw_required_lods,
            );
        }

        let (required_voxels, dropped_voxels) = self.prioritized_chunk_split(
            raw_required_voxels,
            player_pos,
            planet,
            self.lod_cfg.max_required_chunks,
        );
        let missing_voxels: Vec<ChunkKey> = required_voxels
            .iter()
            .filter(|k| !self.chunks.contains_key(k))
            .cloned()
            .collect();
        let required_chunk_count = required_voxels.len();
        let missing_chunk_count = missing_voxels.len();

        let mut required_lods = self.prioritized_lods(
            raw_required_lods,
            player_pos,
            planet,
            self.lod_cfg.max_required_lods,
        );
        self.add_chunk_fallback_lods(
            missing_voxels
                .iter()
                .copied()
                .chain(dropped_voxels.iter().copied()),
            &mut required_lods,
            player_pos,
            planet,
        );
        let current_lods: Vec<LodKey> = self.lod_chunks.keys().cloned().collect();
        for k in current_lods {
            if required_lods.contains(&k) {
                continue;
            }
            let mut children_missing = false;
            for v_key in &missing_voxels {
                if v_key.face != k.face {
                    continue;
                }
                let vx = v_key.u_idx * CHUNK_SIZE;
                let vy = v_key.v_idx * CHUNK_SIZE;
                let vs = CHUNK_SIZE;
                if k.x < vx + vs && k.x + k.size > vx && k.y < vy + vs && k.y + k.size > vy {
                    children_missing = true;
                    break;
                }
            }
            if children_missing {
                required_lods.insert(k);
            } else if let Some(mesh) = self.lod_chunks.remove(&k) {
                self.animator.retire(AnyKey::Lod(k), mesh);
            }
        }
        self.limit_lod_pressure(player_pos, planet);
        self.animator.limit_retained(
            self.lod_cfg.max_retiring_meshes,
            self.lod_cfg.max_retiring_meshes,
        );

        let required_lod_count = required_lods.len();
        let covered_lod_count = required_lods
            .iter()
            .filter(|key| self.lod_chunks.contains_key(key))
            .count();
        let missing_lod_count = required_lod_count.saturating_sub(covered_lod_count);

        let mut spawn_count = 0;
        let grid_res = self.lod_cfg.tile_grid_res;
        let mut missing_lods: Vec<LodKey> = required_lods
            .iter()
            .copied()
            .filter(|key| !self.lod_chunks.contains_key(key) && !self.pending_lods.contains(key))
            .collect();
        missing_lods.sort_by(|a, b| {
            Self::lod_distance_squared(a, player_pos, planet)
                .partial_cmp(&Self::lod_distance_squared(b, player_pos, planet))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for key in missing_lods {
            if !self.lod_chunks.contains_key(&key) && !self.pending_lods.contains(&key) {
                if spawn_count >= self.lod_cfg.lod_jobs_per_frame
                    || self.pending_lods.len() >= self.lod_cfg.max_pending_lod_jobs
                    || self.lod_chunks.len() + self.pending_lods.len()
                        >= self.lod_cfg.max_active_lods
                {
                    break;
                }
                self.pending_lods.insert(key);
                let tx = self.lod_tx.clone();
                let p = planet.clone();
                let blocks = self.block_content.clone();
                std::thread::spawn(move || {
                    let start = Instant::now();
                    let (v, i) = MeshGen::generate_lod_mesh(key, &p, grid_res, &blocks);
                    let job = MeshJobResult {
                        key,
                        vertices: v.len(),
                        indices: i.len(),
                        duration: start.elapsed(),
                    };
                    let _ = tx.send((job, v, i));
                });
                spawn_count += 1;
            }
        }
        self.frame_telemetry.lod.lod_jobs_started += spawn_count as u32;

        let current_voxels: Vec<ChunkKey> = self.chunks.keys().cloned().collect();
        for k in current_voxels {
            if !required_voxels.contains(&k) {
                if let Some(mesh) = self.chunks.remove(&k) {
                    self.animator.retire(AnyKey::Voxel(k), mesh);
                }
            }
        }
        self.frame_telemetry.lod_coverage_time += lod_start.elapsed();

        let streaming_start = Instant::now();
        let mut queued: Vec<(ChunkKey, f32)> = required_voxels
            .iter()
            .copied()
            .filter(|k| !self.chunks.contains_key(k) && !self.pending_chunks.contains(k))
            .map(|k| (k, Self::chunk_distance_squared(&k, player_pos, planet)))
            .collect();
        queued.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        queued.truncate(self.lod_cfg.max_chunk_queue);
        self.load_queue = queued.into_iter().rev().map(|(key, _)| key).collect();
        self.process_load_queue(player_pos, planet);
        self.frame_telemetry.chunk_streaming_time += streaming_start.elapsed();

        self.frame_telemetry.streaming.active_chunks = self.chunks.len();
        self.frame_telemetry.streaming.required_chunks = required_chunk_count;
        self.frame_telemetry.streaming.missing_chunks = missing_chunk_count;
        self.frame_telemetry.streaming.load_queue = self.load_queue.len();
        self.frame_telemetry.streaming.pending_chunk_jobs = self.pending_chunks.len();
        self.frame_telemetry.lod.active_lods = self.lod_chunks.len();
        self.frame_telemetry.lod.required_lods = required_lod_count;
        self.frame_telemetry.lod.covered_lods = covered_lod_count;
        self.frame_telemetry.lod.missing_lods = missing_lod_count;
        self.frame_telemetry.lod.pending_lod_jobs = self.pending_lods.len();
        self.frame_telemetry.lod.coverage_percent = if required_lod_count == 0 {
            100.0
        } else {
            covered_lod_count as f32 * 100.0 / required_lod_count as f32
        };
    }

fn process_quadtree(
        &self,
        face: u8,
        x: u32,
        y: u32,
        size: u32,
        cam_pos: Vec3,
        planet: &PlanetData,
        player_id: Option<BlockId>,
        voxels: &mut HashSet<ChunkKey>,
        lods: &mut HashSet<LodKey>,
    ) {
        if x >= planet.resolution || y >= planet.resolution {
            return;
        }
        let cu = (x + size / 2).min(planet.resolution - 1);
        let cv = (y + size / 2).min(planet.resolution - 1);
        let h = planet.geometry.surface_layer();
        let world_pos = CoordSystem::get_vertex_pos(face, cu, cv, h, planet.geometry);

        let mut dist = world_pos.distance(cam_pos);
        if let Some(pid) = player_id {
            if pid.face == face && pid.u >= x && pid.u < x + size && pid.v >= y && pid.v < y + size
            {
                dist = 0.0;
            }
        }

        let node_r = (size as f32 * CoordSystem::get_layer_radius(h, planet.geometry))
            / planet.resolution as f32;
        let lod_factor = if size <= CHUNK_SIZE {
            18.0
        } else if size <= CHUNK_SIZE * 2 {
            12.0
        } else if size <= CHUNK_SIZE * 4 {
            7.0
        } else if size <= CHUNK_SIZE * 8 {
            5.0
        } else {
            4.0
        };

        let is_smallest = size <= CHUNK_SIZE;
        if dist < node_r * lod_factor && !is_smallest {
            let half = size / 2;
            self.process_quadtree(face, x, y, half, cam_pos, planet, player_id, voxels, lods);
            self.process_quadtree(
                face,
                x + half,
                y,
                half,
                cam_pos,
                planet,
                player_id,
                voxels,
                lods,
            );
            self.process_quadtree(
                face,
                x,
                y + half,
                half,
                cam_pos,
                planet,
                player_id,
                voxels,
                lods,
            );
            self.process_quadtree(
                face,
                x + half,
                y + half,
                half,
                cam_pos,
                planet,
                player_id,
                voxels,
                lods,
            );
        } else if size <= CHUNK_SIZE {
            let key = ChunkKey {
                face,
                u_idx: x / CHUNK_SIZE,
                v_idx: y / CHUNK_SIZE,
            };
            if key.u_idx * CHUNK_SIZE < planet.resolution
                && key.v_idx * CHUNK_SIZE < planet.resolution
            {
                voxels.insert(key);
            }
        } else {
            lods.insert(LodKey { face, x, y, size });
        }
    }

    fn prioritized_chunk_split(
        &self,
        keys: HashSet<ChunkKey>,
        player_pos: Vec3,
        planet: &PlanetData,
        limit: usize,
    ) -> (HashSet<ChunkKey>, Vec<ChunkKey>) {
        let mut ranked: Vec<(ChunkKey, f32)> = keys
            .into_iter()
            .map(|key| (key, Self::chunk_distance_squared(&key, player_pos, planet)))
            .collect();
        ranked.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let keep = limit.min(self.lod_cfg.max_active_chunks).min(ranked.len());
        let required = ranked[..keep].iter().map(|(key, _)| *key).collect();
        let dropped = ranked[keep..].iter().map(|(key, _)| *key).collect();
        (required, dropped)
    }

    fn prioritized_lods(
        &self,
        keys: HashSet<LodKey>,
        player_pos: Vec3,
        planet: &PlanetData,
        limit: usize,
    ) -> HashSet<LodKey> {
        let mut ranked: Vec<(LodKey, f32)> = keys
            .into_iter()
            .map(|key| (key, Self::lod_distance_squared(&key, player_pos, planet)))
            .collect();
        ranked.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(limit.min(self.lod_cfg.max_active_lods));
        ranked.into_iter().map(|(key, _)| key).collect()
    }

    fn add_chunk_fallback_lods(
        &self,
        chunks: impl Iterator<Item = ChunkKey>,
        lods: &mut HashSet<LodKey>,
        player_pos: Vec3,
        planet: &PlanetData,
    ) {
        let mut ranked: Vec<(LodKey, f32)> = chunks
            .filter_map(|chunk| Self::chunk_fallback_lod(chunk, planet))
            .map(|key| (key, Self::lod_distance_squared(&key, player_pos, planet)))
            .collect();
        ranked.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        for (key, _) in ranked {
            lods.insert(key);
        }
        if lods.len() > self.lod_cfg.max_required_lods {
            let mut ranked_lods: Vec<(LodKey, f32)> = lods
                .iter()
                .copied()
                .map(|key| (key, Self::lod_distance_squared(&key, player_pos, planet)))
                .collect();
            ranked_lods.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            for (key, _) in ranked_lods
                .into_iter()
                .take(lods.len() - self.lod_cfg.max_required_lods)
            {
                lods.remove(&key);
            }
        }
    }

    fn chunk_fallback_lod(chunk: ChunkKey, planet: &PlanetData) -> Option<LodKey> {
        let size = (CHUNK_SIZE * 4).min(planet.resolution.next_power_of_two());
        if size <= CHUNK_SIZE {
            return None;
        }
        let x = (chunk.u_idx * CHUNK_SIZE / size) * size;
        let y = (chunk.v_idx * CHUNK_SIZE / size) * size;
        if x >= planet.resolution || y >= planet.resolution {
            return None;
        }
        Some(LodKey {
            face: chunk.face,
            x,
            y,
            size,
        })
    }

    fn limit_lod_pressure(&mut self, player_pos: Vec3, planet: &PlanetData) {
        if self.lod_chunks.len() <= self.lod_cfg.max_active_lods {
            return;
        }
        let mut ranked: Vec<(LodKey, f32)> = self
            .lod_chunks
            .keys()
            .copied()
            .map(|key| (key, Self::lod_distance_squared(&key, player_pos, planet)))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let remove_count = self.lod_chunks.len() - self.lod_cfg.max_active_lods;
        for (key, _) in ranked.into_iter().take(remove_count) {
            if let Some(mesh) = self.lod_chunks.remove(&key) {
                self.animator.retire(AnyKey::Lod(key), mesh);
            }
        }
    }

    fn chunk_distance_squared(key: &ChunkKey, player_pos: Vec3, planet: &PlanetData) -> f32 {
        CoordSystem::get_vertex_pos(
            key.face,
            key.u_idx * CHUNK_SIZE + CHUNK_SIZE / 2,
            key.v_idx * CHUNK_SIZE + CHUNK_SIZE / 2,
            planet.geometry.surface_layer(),
            planet.geometry,
        )
        .distance_squared(player_pos)
    }

    fn lod_distance_squared(key: &LodKey, player_pos: Vec3, planet: &PlanetData) -> f32 {
        CoordSystem::get_vertex_pos(
            key.face,
            key.x
                .saturating_add(key.size / 2)
                .min(planet.resolution - 1),
            key.y
                .saturating_add(key.size / 2)
                .min(planet.resolution - 1),
            planet.geometry.surface_layer(),
            planet.geometry,
        )
        .distance_squared(player_pos)
    }

        pub fn refresh_neighbors(&mut self, id: BlockId, planet: &PlanetData) {
        let u_c = id.u / CHUNK_SIZE;
        let v_c = id.v / CHUNK_SIZE;
        for key in [
            ChunkKey {
                face: id.face,
                u_idx: u_c,
                v_idx: v_c,
            },
            ChunkKey {
                face: id.face,
                u_idx: u_c.saturating_sub(1),
                v_idx: v_c,
            },
            ChunkKey {
                face: id.face,
                u_idx: u_c + 1,
                v_idx: v_c,
            },
            ChunkKey {
                face: id.face,
                u_idx: u_c,
                v_idx: v_c.saturating_sub(1),
            },
            ChunkKey {
                face: id.face,
                u_idx: u_c,
                v_idx: v_c + 1,
            },
        ] {
            if self.chunks.contains_key(&key) {
                let start = Instant::now();
                let (v, i) = MeshGen::build_chunk(key, planet, &self.block_content);
                self.record_mesh_job(start.elapsed(), v.len(), i.len(), MeshJobKind::Remesh);
                if v.is_empty() {
                    self.chunks.remove(&key);
                    self.frame_telemetry.streaming.empty_chunks += 1;
                } else {
                    self.upload_chunk_buffers(key, v, i);
                }
                self.frame_telemetry.streaming.chunks_invalidated += 1;
            }
        }
    }
}