use super::{MeshJobResult, QuadContext, QuadNode, Renderer};
use crate::lod_animation::AnyKey;
use crate::types::Vertex;
use crate::world_streaming::StreamingView;
use glam::Vec3;
use std::collections::HashSet;
use vv_meshing::{CpuMesh, MeshGen, UploadBudgetState};
use vv_voxel::{LodKey, SurfaceChunkKey, VoxelCoord, CHUNK_SIZE};
use vv_world::{PlanetData, PlanetGeometry};

fn lod_mesh_byte_size(mesh: &CpuMesh) -> usize {
    mesh.vertices.len() * std::mem::size_of::<Vertex>() + mesh.indices.len() * 4
}

impl<'a> Renderer<'a> {
    pub fn update_view(&mut self, view: StreamingView, planet: &PlanetData) {
        let update_started = std::time::Instant::now();
        self.reset_streaming_frame_stats();
        self.animator
            .set_fade_duration(self.world_streaming.lod_transition_time);

        let res = planet.resolution();
        let player_id = PlanetGeometry::pos_to_id(view.player_pos, planet.profile());
        let previous_player_chunk_pos = self.player_chunk_pos;
        let player_surface_key = player_id.map(|id| SurfaceChunkKey {
            face: id.face,
            u_idx: id.u / CHUNK_SIZE,
            v_idx: id.v / CHUNK_SIZE,
        });
        let should_rebuild_required =
            self.player_chunk_pos != player_surface_key || self.required_voxels.is_empty();
        self.player_chunk_pos = player_surface_key;

        if should_rebuild_required {
            self.refresh_prop_lod_chunks(previous_player_chunk_pos, player_surface_key);
            let selection_started = std::time::Instant::now();
            self.rebuild_required_sets(view, planet, player_id, res);
            self.lod_selection_ms = selection_started.elapsed().as_secs_f32() * 1000.0;
        }

        self.receive_lod_meshes();
        self.dispatch_lod_jobs(view, planet);

        if should_rebuild_required {
            self.evict_stale_lods();
            self.evict_stale_voxels();
            self.rebuild_load_queue(view, planet);
        }

        self.process_load_queue(view.player_pos, planet);
        self.scheduler_stats.pending_voxel = self.pending_chunks.len();
        self.scheduler_stats.pending_lod = self.pending_lods.len();
        self.update_view_ms = update_started.elapsed().as_secs_f32() * 1000.0;
    }

    fn receive_lod_meshes(&mut self) {
        let upload_started = std::time::Instant::now();
        let mut budget = UploadBudgetState::default();
        loop {
            budget.elapsed_ms = upload_started.elapsed().as_secs_f32() * 1000.0;
            if !self.scheduler.can_upload_lod(&budget) {
                break;
            }
            let Ok(result) = self.lod_rx.try_recv() else {
                break;
            };
            let MeshJobResult {
                key,
                mesh,
                elapsed_ms,
            } = result;
            self.record_lod_mesh_time(elapsed_ms);
            self.pending_lods.remove(&key);
            if self.required_lods.contains(&key) {
                let bytes = lod_mesh_byte_size(&mesh);
                self.upload_lod_buffer(key, mesh);
                budget.count += 1;
                budget.bytes += bytes;
                self.scheduler_stats.uploaded_lod += 1;
            }
        }
    }

    fn evict_stale_lods(&mut self) {
        // A LOD that the quadtree no longer needs is only safe to retire once
        // every smaller replacement covering its region — voxel chunks *and*
        // finer LODs — is actually meshed.  Otherwise its retirement opens a
        // visible void at exactly the spots the player is approaching.
        let missing_voxels: Vec<SurfaceChunkKey> = self
            .required_voxels
            .iter()
            .filter(|k| !self.chunks.contains_key(k))
            .copied()
            .collect();
        let missing_lods: Vec<LodKey> = self
            .required_lods
            .iter()
            .filter(|k| !self.lod_chunks.contains_key(k))
            .copied()
            .collect();

        let current_lods: Vec<LodKey> = self.lod_chunks.keys().copied().collect();
        for k in current_lods {
            if self.required_lods.contains(&k) {
                continue;
            }
            if lod_covers_any_missing_voxel(k, &missing_voxels)
                || lod_covers_any_missing_smaller_lod(k, &missing_lods)
            {
                self.required_lods.insert(k);
            } else if let Some(mesh) = self.lod_chunks.remove(&k) {
                self.animator.retire(AnyKey::Lod(k), mesh);
            }
        }
    }

    fn dispatch_lod_jobs(&mut self, view: StreamingView, planet: &PlanetData) {
        let mut to_spawn: Vec<(LodKey, f32)> = self
            .required_lods
            .iter()
            .filter(|k| !self.lod_chunks.contains_key(k) && !self.pending_lods.contains(k))
            .map(|k| (*k, lod_priority(*k, view, planet)))
            .collect();
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
            let snapshot = planet.snapshot();
            rayon::spawn(move || {
                let started = std::time::Instant::now();
                let mesh = MeshGen::generate_lod_mesh(key, &snapshot);
                let elapsed_ms = started.elapsed().as_secs_f32() * 1000.0;
                let _ = tx.send(MeshJobResult {
                    key,
                    mesh,
                    elapsed_ms,
                });
            });
            self.scheduler_stats.dispatched_lod += 1;
        }
    }

    fn evict_stale_voxels(&mut self) {
        let current_voxels: Vec<SurfaceChunkKey> = self.chunks.keys().copied().collect();
        for k in current_voxels {
            if !self.required_voxels.contains(&k) {
                if let Some(mesh) = self.chunks.remove(&k) {
                    self.animator.retire(AnyKey::Voxel(k), mesh);
                }
            }
        }
    }

    fn rebuild_required_sets(
        &mut self,
        view: StreamingView,
        planet: &PlanetData,
        player_id: Option<VoxelCoord>,
        resolution: u32,
    ) {
        let previous_voxels = self.required_voxels.clone();
        let previous_lods = self.required_lods.clone();
        let mut required_voxels = HashSet::new();
        let mut required_lods = HashSet::new();

        let logical_size = resolution.next_power_of_two();
        let quad_context = QuadContext {
            view,
            planet,
            player_id,
            previous_voxels: &previous_voxels,
            previous_lods: &previous_lods,
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

        enforce_streaming_budget(
            self.world_streaming.max_visible_voxel_chunks,
            self.world_streaming.max_visible_lod_tiles,
            view,
            planet,
            &mut required_voxels,
            &mut required_lods,
        );

        self.required_voxels = required_voxels;
        self.required_lods = required_lods;
    }

    fn rebuild_load_queue(&mut self, view: StreamingView, planet: &PlanetData) {
        self.load_queue.clear();
        self.load_queue_set.clear();

        for key in &self.required_voxels {
            if self.chunks.contains_key(key) || self.pending_chunks.contains(key) {
                continue;
            }
            self.load_queue.push(*key);
            self.load_queue_set.insert(*key);
        }

        self.load_queue.sort_by(|a, b| {
            let da = voxel_priority(*a, view, planet);
            let db = voxel_priority(*b, view, planet);
            db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    fn refresh_prop_lod_chunks(
        &mut self,
        previous_player_key: Option<SurfaceChunkKey>,
        current_player_key: Option<SurfaceChunkKey>,
    ) {
        if previous_player_key == current_player_key {
            return;
        }

        let changed: Vec<SurfaceChunkKey> = self
            .chunks
            .keys()
            .copied()
            .filter(|key| {
                MeshGen::should_bake_props_for_chunk(*key, previous_player_key, self.meshing)
                    != MeshGen::should_bake_props_for_chunk(*key, current_player_key, self.meshing)
            })
            .collect();
        self.dirty_chunks.extend(changed);
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

        if x >= planet.resolution() || y >= planet.resolution() {
            return;
        }

        let center_u = (x + size / 2).min(planet.resolution() - 1);
        let center_v = (y + size / 2).min(planet.resolution() - 1);
        let h = planet.profile().surface_layer;
        let world_pos =
            PlanetGeometry::get_vertex_pos(face, center_u, center_v, h, planet.profile());

        let mut dist = world_pos.distance(context.view.camera_pos);
        let cursor_inside = context
            .view
            .cursor_id
            .is_some_and(|cursor| voxel_in_node(cursor, node));

        if let Some(pid) = context.player_id {
            if voxel_in_node(pid, node) {
                dist = 0.0;
            }
        }

        let node_radius_world =
            (size as f32 * planet.profile().layer_radius(h)) / planet.resolution() as f32;
        let node_size_chunks = (size / CHUNK_SIZE).max(1);
        let split_distance = self
            .world_streaming
            .split_distance(node_radius_world, node_size_chunks);
        let hysteresis = self.world_streaming.lod_hysteresis.clamp(0.0, 0.45);
        let was_split = node_was_split(node, context.previous_voxels, context.previous_lods);
        let split_threshold = if was_split {
            split_distance * (1.0 + hysteresis)
        } else {
            split_distance * (1.0 - hysteresis)
        };
        let is_smallest = size <= CHUNK_SIZE;

        if (cursor_inside || dist < split_threshold) && !is_smallest {
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
        } else if is_smallest {
            let key = SurfaceChunkKey {
                face,
                u_idx: x / CHUNK_SIZE,
                v_idx: y / CHUNK_SIZE,
            };
            if (key.u_idx * CHUNK_SIZE) < planet.resolution()
                && (key.v_idx * CHUNK_SIZE) < planet.resolution()
            {
                voxels.insert(key);
            }
        } else {
            lods.insert(LodKey { face, x, y, size });
        }
    }
}

fn enforce_streaming_budget(
    max_voxels: usize,
    max_lods: usize,
    view: StreamingView,
    planet: &PlanetData,
    voxels: &mut HashSet<SurfaceChunkKey>,
    lods: &mut HashSet<LodKey>,
) {
    if voxels.len() > max_voxels {
        let mut ranked: Vec<(SurfaceChunkKey, f32)> = voxels
            .iter()
            .map(|k| (*k, voxel_priority(*k, view, planet)))
            .collect();
        ranked.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let keep: HashSet<SurfaceChunkKey> =
            ranked.iter().take(max_voxels).map(|(k, _)| *k).collect();
        for dropped in voxels.difference(&keep) {
            if let Some(fallback) = fallback_lod_for_voxel(*dropped, planet.resolution()) {
                lods.insert(fallback);
            }
        }
        *voxels = keep;
    }

    if lods.len() > max_lods {
        // Coarse LODs are the planet's only coverage at the horizon — losing
        // one leaves a visible void.  Fine LODs near the player are about to
        // be replaced by voxel chunks anyway, so dropping them is harmless.
        // Rank by size descending (coarsest kept first), tie-break with the
        // usual distance/view priority.
        let mut ranked: Vec<(LodKey, f32)> = lods
            .iter()
            .map(|k| (*k, lod_priority(*k, view, planet)))
            .collect();
        ranked.sort_by(|a, b| {
            b.0.size
                .cmp(&a.0.size)
                .then_with(|| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        });
        *lods = ranked.iter().take(max_lods).map(|(k, _)| *k).collect();
    }
}

fn voxel_priority(key: SurfaceChunkKey, view: StreamingView, planet: &PlanetData) -> f32 {
    let center = chunk_center(key, planet);
    let cursor_bonus = view
        .cursor_id
        .filter(|c| voxel_in_chunk(*c, key))
        .map_or(0.0, |_| 20_000.0);
    priority_score(
        center,
        chunk_radius_world(planet),
        view,
        planet,
        cursor_bonus,
    )
}

fn lod_priority(key: LodKey, view: StreamingView, planet: &PlanetData) -> f32 {
    let center = lod_center(key, planet);
    let radius = lod_radius_world(key, planet);
    let cursor_bonus = view
        .cursor_id
        .filter(|c| voxel_in_lod(*c, key))
        .map_or(0.0, |_| 15_000.0);
    priority_score(center, radius, view, planet, cursor_bonus)
}

fn priority_score(
    center: Vec3,
    radius: f32,
    view: StreamingView,
    planet: &PlanetData,
    cursor_bonus: f32,
) -> f32 {
    let to_center = center - view.camera_pos;
    let dist = to_center.length().max(0.001);
    let dir = to_center / dist;
    let view_dir = view.view_dir.normalize_or_zero();
    let angle_penalty = (1.0 - view_dir.dot(dir).clamp(-1.0, 1.0)) * dist * 0.55;
    let horizon_bonus = horizon_importance(center, radius, view, planet) * dist * 0.25;
    let landmark_bonus = terrain_landmark_importance(radius, planet) * dist * 0.18;
    dist + angle_penalty - horizon_bonus - landmark_bonus - cursor_bonus
}

fn horizon_importance(center: Vec3, radius: f32, view: StreamingView, planet: &PlanetData) -> f32 {
    let cam_dist = view.camera_pos.length();
    let surface_radius = planet.profile().surface_radius;
    if cam_dist <= surface_radius * 1.001 {
        return 0.0;
    }
    let dist = center.length().max(0.001);
    let cos_horizon = surface_radius / cam_dist;
    let cos_angle = view.camera_pos.normalize_or_zero().dot(center / dist);
    let angular_radius = radius / dist;
    let band = (2.5 * angular_radius).max(0.015);
    (1.0 - ((cos_angle - cos_horizon).abs() / band)).clamp(0.0, 1.0)
}

fn terrain_landmark_importance(radius: f32, planet: &PlanetData) -> f32 {
    (radius / planet.profile().surface_radius.max(1.0)).clamp(0.0, 1.0)
}

fn node_was_split(
    node: QuadNode,
    previous_voxels: &HashSet<SurfaceChunkKey>,
    previous_lods: &HashSet<LodKey>,
) -> bool {
    previous_voxels.iter().any(|k| chunk_in_node(*k, node))
        || previous_lods
            .iter()
            .any(|k| k.face == node.face && k.size < node.size && lod_inside_node(*k, node))
}

fn lod_covers_any_missing_voxel(key: LodKey, missing_voxels: &[SurfaceChunkKey]) -> bool {
    missing_voxels.iter().any(|v_key| {
        let v_x = v_key.u_idx * CHUNK_SIZE;
        let v_y = v_key.v_idx * CHUNK_SIZE;
        key.face == v_key.face
            && key.x < v_x + CHUNK_SIZE
            && key.x + key.size > v_x
            && key.y < v_y + CHUNK_SIZE
            && key.y + key.size > v_y
    })
}

/// True if any *smaller* required LOD that has not been meshed yet falls
/// inside `parent`'s area.  The parent is the quadtree's only coverage of
/// that region until the finer tile uploads — retiring it early would tear
/// a hole in the horizon for several frames.
fn lod_covers_any_missing_smaller_lod(parent: LodKey, missing_lods: &[LodKey]) -> bool {
    missing_lods.iter().any(|child| {
        child.size < parent.size
            && child.face == parent.face
            && parent.x < child.x + child.size
            && parent.x + parent.size > child.x
            && parent.y < child.y + child.size
            && parent.y + parent.size > child.y
    })
}

fn fallback_lod_for_voxel(key: SurfaceChunkKey, resolution: u32) -> Option<LodKey> {
    let size = CHUNK_SIZE * 2;
    if size >= resolution.next_power_of_two() {
        return None;
    }
    let x = (key.u_idx * CHUNK_SIZE / size) * size;
    let y = (key.v_idx * CHUNK_SIZE / size) * size;
    Some(LodKey {
        face: key.face,
        x,
        y,
        size,
    })
}

fn chunk_in_node(key: SurfaceChunkKey, node: QuadNode) -> bool {
    let x = key.u_idx * CHUNK_SIZE;
    let y = key.v_idx * CHUNK_SIZE;
    key.face == node.face
        && x >= node.x
        && y >= node.y
        && x < node.x + node.size
        && y < node.y + node.size
}

fn lod_inside_node(key: LodKey, node: QuadNode) -> bool {
    key.x >= node.x
        && key.y >= node.y
        && key.x + key.size <= node.x + node.size
        && key.y + key.size <= node.y + node.size
}

fn voxel_in_node(coord: VoxelCoord, node: QuadNode) -> bool {
    coord.face == node.face
        && coord.u >= node.x
        && coord.u < node.x + node.size
        && coord.v >= node.y
        && coord.v < node.y + node.size
}

fn voxel_in_chunk(coord: VoxelCoord, key: SurfaceChunkKey) -> bool {
    coord.face == key.face && coord.u / CHUNK_SIZE == key.u_idx && coord.v / CHUNK_SIZE == key.v_idx
}

fn voxel_in_lod(coord: VoxelCoord, key: LodKey) -> bool {
    coord.face == key.face
        && coord.u >= key.x
        && coord.u < key.x + key.size
        && coord.v >= key.y
        && coord.v < key.y + key.size
}

fn chunk_center(key: SurfaceChunkKey, planet: &PlanetData) -> Vec3 {
    let u = key.u_idx * CHUNK_SIZE + CHUNK_SIZE / 2;
    let v = key.v_idx * CHUNK_SIZE + CHUNK_SIZE / 2;
    let h = planet.profile().surface_layer;
    PlanetGeometry::get_vertex_pos(key.face, u, v, h, planet.profile())
}

fn lod_center(key: LodKey, planet: &PlanetData) -> Vec3 {
    let u = (key.x + key.size / 2).min(planet.resolution().saturating_sub(1));
    let v = (key.y + key.size / 2).min(planet.resolution().saturating_sub(1));
    let h = planet.profile().surface_layer;
    PlanetGeometry::get_vertex_pos(key.face, u, v, h, planet.profile())
}

fn chunk_radius_world(planet: &PlanetData) -> f32 {
    (CHUNK_SIZE as f32
        * planet
            .profile()
            .layer_radius(planet.profile().surface_layer))
        / planet.resolution() as f32
}

fn lod_radius_world(key: LodKey, planet: &PlanetData) -> f32 {
    (key.size as f32
        * planet
            .profile()
            .layer_radius(planet.profile().surface_layer))
        / planet.resolution() as f32
}
