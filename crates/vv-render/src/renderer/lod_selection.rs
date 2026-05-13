use super::{MeshJobResult, QuadContext, QuadNode, Renderer};
use vv_math::CoordSystem;
use vv_meshing::MeshGen;
use crate::lod_animation::AnyKey;
use vv_voxel::{LodKey, SurfaceChunkKey, CHUNK_SIZE};
use vv_world::PlanetData;
use glam::Vec3;
use std::collections::HashSet;

impl<'a> Renderer<'a> {
    pub fn update_view(&mut self, player_pos: Vec3, planet: &PlanetData) {
        let update_started = std::time::Instant::now();
        self.reset_streaming_frame_stats();

        let res = planet.resolution;
        let player_id = CoordSystem::pos_to_id(player_pos, planet.profile);
        let player_surface_key = player_id.map(|id| SurfaceChunkKey {
            face: id.face,
            u_idx: id.u / CHUNK_SIZE,
            v_idx: id.v / CHUNK_SIZE,
        });
        let should_rebuild_required =
            self.player_chunk_pos != player_surface_key || self.required_voxels.is_empty();
        self.player_chunk_pos = player_surface_key;

        if should_rebuild_required {
            self.rebuild_required_sets(player_pos, planet, player_id, res);
        }

        // 2. Receive completed LOD meshes.
        //    Only upload if the chunk is still needed — stale meshes are dropped.
        let mut uploaded_lods = 0;
        while self.scheduler.can_upload_lod(uploaded_lods) {
            let Ok(result) = self.lod_rx.try_recv() else {
                break;
            };
            let MeshJobResult {
                key,
                mesh,
                elapsed_ms,
            } = result;
            self.record_mesh_time(elapsed_ms);
            self.pending_lods.remove(&key);
            if self.required_lods.contains(&key) {
                self.upload_lod_buffer(key, mesh);
                uploaded_lods += 1;
                self.scheduler_stats.uploaded_lod += 1;
            }
            // stale mesh: drop without uploading
        }

        // 3. Evict stale meshes only when the required topology changes.
        //    Keep any LOD that covers a voxel chunk still in flight (children_missing guard).
        if should_rebuild_required {
            let missing_voxels: Vec<SurfaceChunkKey> = self
                .required_voxels
                .iter()
                .filter(|k| !self.chunks.contains_key(k))
                .cloned()
                .collect();

            let current_lods: Vec<LodKey> = self.lod_chunks.keys().cloned().collect();
            for k in current_lods {
                if self.required_lods.contains(&k) {
                    continue;
                }
                let mut children_missing = false;
                for v_key in &missing_voxels {
                    if v_key.face != k.face {
                        continue;
                    }
                    let v_x = v_key.u_idx * CHUNK_SIZE;
                    let v_y = v_key.v_idx * CHUNK_SIZE;
                    let overlap = k.x < v_x + CHUNK_SIZE
                        && k.x + k.size > v_x
                        && k.y < v_y + CHUNK_SIZE
                        && k.y + k.size > v_y;
                    if overlap {
                        children_missing = true;
                        break;
                    }
                }
                if children_missing {
                    self.required_lods.insert(k);
                } else if let Some(mesh) = self.lod_chunks.remove(&k) {
                    self.animator.retire(AnyKey::Lod(k), mesh);
                }
            }
        }

        // 4. Dispatch new LOD jobs — sorted nearest-first so the most visible
        //    chunks are meshed before distant ones.
        let mut to_spawn: Vec<(LodKey, f32)> = self
            .required_lods
            .iter()
            .filter(|k| !self.lod_chunks.contains_key(k) && !self.pending_lods.contains(k))
            .map(|k| {
                let cx = (k.x + k.size / 2).min(planet.resolution.saturating_sub(1));
                let cy = (k.y + k.size / 2).min(planet.resolution.saturating_sub(1));
                let h = planet.profile.surface_layer;
                let wp = CoordSystem::get_vertex_pos(k.face, cx, cy, h, planet.profile);
                (*k, wp.distance_squared(player_pos))
            })
            .collect();
        // nearest first (smallest distance_sq at front)
        to_spawn.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        for (key, _) in to_spawn {
            if !self
                .scheduler
                .can_dispatch_lod(self.scheduler_stats.dispatched_lod, self.pending_lods.len())
            {
                break;
            }
            self.pending_lods.insert(key);
            let tx = self.lod_tx.clone();
            let p = planet.clone();
            rayon::spawn(move || {
                let started = std::time::Instant::now();
                let mesh = MeshGen::generate_lod_mesh(key, &p);
                let elapsed_ms = started.elapsed().as_secs_f32() * 1000.0;
                let _ = tx.send(MeshJobResult {
                    key,
                    mesh,
                    elapsed_ms,
                });
            });
            self.scheduler_stats.dispatched_lod += 1;
        }

        // 5. Evict voxel chunks no longer required and rebuild load queue.
        if should_rebuild_required {
            let current_voxels: Vec<SurfaceChunkKey> = self.chunks.keys().cloned().collect();
            for k in current_voxels {
                if !self.required_voxels.contains(&k) {
                    if let Some(mesh) = self.chunks.remove(&k) {
                        self.animator.retire(AnyKey::Voxel(k), mesh);
                    }
                }
            }

            self.rebuild_load_queue(player_pos, planet);
        }

        self.process_load_queue(player_pos, planet);
        self.scheduler_stats.pending_voxel = self.pending_chunks.len();
        self.scheduler_stats.pending_lod = self.pending_lods.len();
        self.update_view_ms = update_started.elapsed().as_secs_f32() * 1000.0;
    }

    fn rebuild_required_sets(
        &mut self,
        player_pos: Vec3,
        planet: &PlanetData,
        player_id: Option<vv_voxel::VoxelCoord>,
        resolution: u32,
    ) {
        self.required_voxels.clear();
        self.required_lods.clear();

        let logical_size = resolution.next_power_of_two();
        let quad_context = QuadContext {
            cam_pos: player_pos,
            planet,
            player_id,
        };
        let mut required_voxels = std::mem::take(&mut self.required_voxels);
        let mut required_lods = std::mem::take(&mut self.required_lods);

        for face in 0..6 {
            self.process_quadtree(
                QuadNode {
                    face,
                    x: 0,
                    y: 0,
                    size: logical_size,
                },
                &quad_context,
                &mut required_voxels,
                &mut required_lods,
            );
        }

        self.required_voxels = required_voxels;
        self.required_lods = required_lods;
    }

    fn rebuild_load_queue(&mut self, player_pos: Vec3, planet: &PlanetData) {
        self.load_queue.clear();
        self.load_queue_set.clear();

        for key in &self.required_voxels {
            if self.chunks.contains_key(key) || self.pending_chunks.contains(key) {
                continue;
            }
            self.load_queue.push(*key);
            self.load_queue_set.insert(*key);
        }

        // Sort farthest-first so pop() returns the nearest chunk.
        self.load_queue.sort_by(|a, b| {
            let da = chunk_center(*a, planet).distance_squared(player_pos);
            let db = chunk_center(*b, planet).distance_squared(player_pos);
            db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    fn process_quadtree(
        &self,
        node: QuadNode,
        context: &QuadContext<'_>,
        voxels: &mut HashSet<SurfaceChunkKey>,
        lods: &mut HashSet<LodKey>,
    ) {
        let QuadNode { face, x, y, size } = node;
        let planet = context.planet;

        if x >= planet.resolution || y >= planet.resolution {
            return;
        }

        let center_u = (x + size / 2).min(planet.resolution - 1);
        let center_v = (y + size / 2).min(planet.resolution - 1);
        let h = planet.profile.surface_layer;

        let world_pos = CoordSystem::get_vertex_pos(face, center_u, center_v, h, planet.profile);

        let mut dist = world_pos.distance(context.cam_pos);

        if let Some(pid) = context.player_id {
            if pid.face == face && pid.u >= x && pid.u < x + size && pid.v >= y && pid.v < y + size
            {
                dist = 0.0;
            }
        }

        let node_radius_world =
            (size as f32 * planet.profile.layer_radius(h)) / planet.resolution as f32;

        let mut lod_factor = 4.0;
        if size <= CHUNK_SIZE * 8 {
            lod_factor = 5.0;
        }
        if size <= CHUNK_SIZE * 4 {
            lod_factor = 7.0;
        }
        if size <= CHUNK_SIZE * 2 {
            lod_factor = 12.0;
        }
        if size <= CHUNK_SIZE {
            lod_factor = 18.0;
        }

        let split_distance = node_radius_world * lod_factor * self.lod_distance_scale;
        let is_smallest = size <= CHUNK_SIZE;

        if dist < split_distance && !is_smallest {
            let half = size / 2;
            self.process_quadtree(
                QuadNode {
                    face,
                    x,
                    y,
                    size: half,
                },
                context,
                voxels,
                lods,
            );
            self.process_quadtree(
                QuadNode {
                    face,
                    x: x + half,
                    y,
                    size: half,
                },
                context,
                voxels,
                lods,
            );
            self.process_quadtree(
                QuadNode {
                    face,
                    x,
                    y: y + half,
                    size: half,
                },
                context,
                voxels,
                lods,
            );
            self.process_quadtree(
                QuadNode {
                    face,
                    x: x + half,
                    y: y + half,
                    size: half,
                },
                context,
                voxels,
                lods,
            );
        } else if size <= CHUNK_SIZE {
            let key = SurfaceChunkKey {
                face,
                u_idx: x / CHUNK_SIZE,
                v_idx: y / CHUNK_SIZE,
            };
            if (key.u_idx * CHUNK_SIZE) < planet.resolution
                && (key.v_idx * CHUNK_SIZE) < planet.resolution
            {
                voxels.insert(key);
            }
        } else {
            lods.insert(LodKey { face, x, y, size });
        }
    }
}

fn chunk_center(key: SurfaceChunkKey, planet: &PlanetData) -> Vec3 {
    let u = key.u_idx * CHUNK_SIZE + CHUNK_SIZE / 2;
    let v = key.v_idx * CHUNK_SIZE + CHUNK_SIZE / 2;
    let h = planet.profile.surface_layer;
    CoordSystem::get_vertex_pos(key.face, u, v, h, planet.profile)
}

