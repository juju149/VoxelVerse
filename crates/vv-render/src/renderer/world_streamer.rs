use super::{LocalUniform, MeshJobResult, Renderer};
use crate::lod_animation::AnyKey;
use crate::types::{ChunkMesh, Vertex};
use crate::world_streaming::StreamingView;
use glam::Vec3;
use vv_meshing::{CpuMesh, MeshGen, UploadBudgetState};
use vv_voxel::{LodKey, SurfaceChunkKey};
use vv_world::PlanetData;
use wgpu::util::DeviceExt;

fn mesh_byte_size(mesh: &CpuMesh) -> usize {
    mesh.vertices.len() * std::mem::size_of::<Vertex>() + mesh.indices.len() * 4
}

/// Convert a `CpuMesh` vertex slice to GPU-ready `Vertex` bytes.
fn cpu_verts_to_gpu(cpu: &CpuMesh) -> Vec<Vertex> {
    cpu.vertices.iter().copied().map(Vertex::from).collect()
}

impl<'a> Renderer<'a> {
    pub(super) fn upload_lod_buffer(&mut self, key: LodKey, mesh: CpuMesh) {
        let upload_started = std::time::Instant::now();
        let gpu_verts = cpu_verts_to_gpu(&mesh);
        let v_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&gpu_verts),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        let i_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });

        let uniform_data = LocalUniform {
            model: glam::Mat4::IDENTITY.to_cols_array(),
            params: [0.0, 0.0, 0.0, 0.0],
        };

        let uniform_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("LOD Uniform"),
                contents: bytemuck::cast_slice(&[uniform_data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.local_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
            label: None,
        });

        let real_center = Vec3::from_array(mesh.bounds.center);
        let real_radius = mesh.bounds.radius;
        let num_inds = mesh.indices.len() as u32;
        let num_verts = mesh.vertices.len();

        self.lod_chunks.insert(
            key,
            ChunkMesh {
                v_buf,
                i_buf,
                num_inds,
                num_verts,
                uniform_buf,
                bind_group,
                center: real_center,
                radius: real_radius,
            },
        );
        self.animator.start_spawn(AnyKey::Lod(key));
        self.gpu_upload_ms += upload_started.elapsed().as_secs_f32() * 1000.0;
    }

    pub(super) fn process_load_queue(&mut self, _player_pos: Vec3, planet: &PlanetData) {
        // Drain completed voxel meshes from the channel.  Upload budget is
        // tracked in (count, bytes, ms) — any single ceiling can stop the loop
        // so a giant chunk cannot snowball into a frame spike.
        let upload_started = std::time::Instant::now();
        let mut budget = UploadBudgetState::default();
        loop {
            budget.elapsed_ms = upload_started.elapsed().as_secs_f32() * 1000.0;
            if !self.scheduler.can_upload_voxel(&budget) {
                break;
            }
            let Ok(result) = self.mesh_rx.try_recv() else {
                break;
            };
            let MeshJobResult {
                key,
                mesh,
                elapsed_ms,
            } = result;
            self.record_voxel_mesh_time(elapsed_ms);
            self.pending_chunks.remove(&key);
            let is_dirty_rebuild = self.pending_dirty.remove(&key);
            let still_needed = self.required_voxels.contains(&key);
            if !mesh.is_empty()
                && still_needed
                && (is_dirty_rebuild || !self.chunks.contains_key(&key))
            {
                let bytes = mesh_byte_size(&mesh);
                self.upload_chunk_buffers(key, mesh, planet.profile.edge_rounding_radius_voxels);
                budget.count += 1;
                budget.bytes += bytes;
                self.scheduler_stats.uploaded_voxel += 1;
            }
        }

        // --- DIRTY CHUNKS (player edits) — dispatched with priority ---
        let dirty: Vec<SurfaceChunkKey> = self
            .dirty_chunks
            .drain()
            .filter(|k| !self.pending_chunks.contains(k))
            .collect();

        for key in dirty {
            if !self.scheduler.can_dispatch_voxel(
                self.scheduler_stats.dispatched_voxel,
                self.pending_chunks.len(),
            ) {
                self.dirty_chunks.insert(key);
                continue;
            }
            self.pending_chunks.insert(key);
            self.pending_dirty.insert(key);
            let mut snapshot = planet.snapshot();
            snapshot.player_surface_key = self.player_chunk_pos;
            let tx = self.mesh_tx.clone();
            let meshing = self.meshing;
            rayon::spawn(move || {
                let started = std::time::Instant::now();
                let mesh = MeshGen::build_chunk(key, &snapshot, meshing);
                let elapsed_ms = started.elapsed().as_secs_f32() * 1000.0;
                let _ = tx.send(MeshJobResult {
                    key,
                    mesh,
                    elapsed_ms,
                });
            });
            self.scheduler_stats.dispatched_voxel += 1;
        }

        if !self.scheduler.can_upload_voxel(&budget) || self.load_queue.is_empty() {
            return;
        }
        if !self.scheduler.can_dispatch_voxel(
            self.scheduler_stats.dispatched_voxel,
            self.pending_chunks.len(),
        ) {
            return;
        }

        // Dispatch new initial-load jobs.
        while self.scheduler.can_dispatch_voxel(
            self.scheduler_stats.dispatched_voxel,
            self.pending_chunks.len(),
        ) {
            let Some(key) = self.load_queue.pop() else {
                break;
            };
            self.load_queue_set.remove(&key);
            if self.chunks.contains_key(&key) || self.pending_chunks.contains(&key) {
                continue;
            }
            self.pending_chunks.insert(key);
            let mut snapshot = planet.snapshot();
            snapshot.player_surface_key = self.player_chunk_pos;
            let tx = self.mesh_tx.clone();
            let meshing = self.meshing;
            rayon::spawn(move || {
                let started = std::time::Instant::now();
                let mesh = MeshGen::build_chunk(key, &snapshot, meshing);
                let elapsed_ms = started.elapsed().as_secs_f32() * 1000.0;
                let _ = tx.send(MeshJobResult {
                    key,
                    mesh,
                    elapsed_ms,
                });
            });
            self.scheduler_stats.dispatched_voxel += 1;
        }
    }

    pub fn force_reload_all(&mut self, planet: &PlanetData, player_pos: Vec3) {
        self.chunks.clear();
        self.lod_chunks.clear();
        self.load_queue.clear();
        self.load_queue_set.clear();
        self.pending_chunks.clear();
        self.pending_dirty.clear();
        self.pending_lods.clear();
        self.dirty_chunks.clear();
        self.player_chunk_pos = None;
        self.required_voxels.clear();
        self.required_lods.clear();
        self.update_view(
            StreamingView {
                player_pos,
                camera_pos: player_pos,
                view_dir: player_pos.normalize_or_zero(),
                cursor_id: None,
            },
            planet,
        );
    }

    /// Queue dirty chunks produced by world edits.
    pub fn refresh_dirty_chunks(&mut self, keys: Vec<SurfaceChunkKey>) {
        for key in keys {
            self.dirty_chunks.insert(key);
        }
    }

    fn upload_chunk_buffers(
        &mut self,
        key: SurfaceChunkKey,
        mesh: CpuMesh,
        edge_rounding_radius_voxels: f32,
    ) {
        let upload_started = std::time::Instant::now();
        let gpu_verts = cpu_verts_to_gpu(&mesh);
        let v_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&gpu_verts),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        let i_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });

        let is_update = self.chunks.contains_key(&key);
        let start_opacity = if is_update { 1.0 } else { 0.0 };

        let uniform_data = LocalUniform {
            model: glam::Mat4::IDENTITY.to_cols_array(),
            params: [start_opacity, edge_rounding_radius_voxels, 0.0, 0.0],
        };

        let uniform_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Uniform"),
                contents: bytemuck::cast_slice(&[uniform_data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.local_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
            label: None,
        });

        let real_center = Vec3::from_array(mesh.bounds.center);
        let real_radius = mesh.bounds.radius;
        let num_inds = mesh.indices.len() as u32;
        let num_verts = mesh.vertices.len();

        self.chunks.insert(
            key,
            ChunkMesh {
                v_buf,
                i_buf,
                num_inds,
                num_verts,
                uniform_buf,
                bind_group,
                center: real_center,
                radius: real_radius,
            },
        );

        if !is_update {
            self.animator.start_spawn(AnyKey::Voxel(key));
        }
        self.gpu_upload_ms += upload_started.elapsed().as_secs_f32() * 1000.0;
    }

    pub fn log_memory(&self, planet: &PlanetData) {
        let stats = self.render_stats(0, 0);
        println!("------------------------------------------");
        println!("RESOLUTION: {}", planet.resolution);
        println!("Active Chunks: {}", stats.active_chunks);
        println!("Active LODs: {}", stats.active_lods);
        println!("Pending Chunks: {}", stats.pending_chunks);
        println!("Pending LODs: {}", stats.pending_lods);
        println!("GPU Memory: {}", stats.gpu_memory_label());
        println!("------------------------------------------");
    }
}
