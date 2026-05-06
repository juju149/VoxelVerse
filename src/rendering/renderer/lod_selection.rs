use super::{QuadContext, QuadNode, Renderer};
use crate::generation::CoordSystem;
use crate::meshing::MeshGen;
use crate::rendering::lod_animation::AnyKey;
use crate::voxel::{ChunkKey, LodKey, CHUNK_SIZE};
use crate::world::PlanetData;
use glam::Vec3;
use std::collections::HashSet;

impl<'a> Renderer<'a> {
    pub fn update_view(&mut self, player_pos: Vec3, planet: &PlanetData) {
        let res = planet.resolution;
        let player_id = CoordSystem::pos_to_id(player_pos, res);
        let mut upload_count = 0;
        while let Ok((key, v, i)) = self.lod_rx.try_recv() {
            self.pending_lods.remove(&key);
            self.upload_lod_buffer(key, v, i);
            upload_count += 1;
            if upload_count > 20 {
                break;
            }
        }
        let mut required_voxels: HashSet<ChunkKey> = HashSet::new();
        let mut required_lods: HashSet<LodKey> = HashSet::new();
        let logical_size = res.next_power_of_two();

        let quad_context = QuadContext {
            cam_pos: player_pos,
            planet,
            player_id,
        };

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

        let missing_voxels: Vec<ChunkKey> = required_voxels
            .iter()
            .filter(|k| !self.chunks.contains_key(k))
            .cloned()
            .collect();

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
                let v_x = v_key.u_idx * CHUNK_SIZE;
                let v_y = v_key.v_idx * CHUNK_SIZE;
                let v_s = CHUNK_SIZE;
                let overlap =
                    k.x < v_x + v_s && k.x + k.size > v_x && k.y < v_y + v_s && k.y + k.size > v_y;
                if overlap {
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

        let mut spawn_count = 0;
        for key in required_lods {
            if !self.lod_chunks.contains_key(&key) && !self.pending_lods.contains(&key) {
                if spawn_count >= 8 {
                    break;
                }
                self.pending_lods.insert(key);
                let tx = self.lod_tx.clone();
                let p = planet.clone();
                std::thread::spawn(move || {
                    let (v, i) = MeshGen::generate_lod_mesh(key, &p);
                    let _ = tx.send((key, v, i));
                });
                spawn_count += 1;
            }
        }

        let current_voxels: Vec<ChunkKey> = self.chunks.keys().cloned().collect();
        for k in current_voxels {
            if !required_voxels.contains(&k) {
                if let Some(mesh) = self.chunks.remove(&k) {
                    self.animator.retire(AnyKey::Voxel(k), mesh);
                }
            }
        }

        self.load_queue.retain(|k| required_voxels.contains(k));
        for k in required_voxels {
            if !self.chunks.contains_key(&k) && !self.load_queue.contains(&k) {
                self.load_queue.push(k);
            }
        }

        self.load_queue.sort_by(|a, b| {
            let get_center = |k: &ChunkKey| -> glam::Vec3 {
                let u = k.u_idx * CHUNK_SIZE + CHUNK_SIZE / 2;
                let v = k.v_idx * CHUNK_SIZE + CHUNK_SIZE / 2;
                let h = planet.profile.surface_layer;
                CoordSystem::get_vertex_pos(k.face, u, v, h, planet.resolution)
            };
            let da = get_center(a).distance_squared(player_pos);
            let db = get_center(b).distance_squared(player_pos);
            db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
        });

        self.process_load_queue(player_pos, planet);
    }

    fn process_quadtree(
        &self,
        node: QuadNode,
        context: &QuadContext<'_>,
        voxels: &mut HashSet<ChunkKey>,
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

        let world_pos = CoordSystem::get_vertex_pos(face, center_u, center_v, h, planet.resolution);

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

        let split_distance = node_radius_world * lod_factor;
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
            let key = ChunkKey {
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
