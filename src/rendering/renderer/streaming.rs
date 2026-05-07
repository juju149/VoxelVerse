use super::{LocalUniform, Renderer};
use crate::meshing::{CpuMesh, MeshGen};
use crate::rendering::lod_animation::AnyKey;
use crate::rendering::types::{ChunkMesh, Vertex};
use crate::voxel::{LodKey, SurfaceChunkKey, VoxelCoord, CHUNK_SIZE};
use crate::world::PlanetData;
use glam::Vec3;
use wgpu::util::DeviceExt;

// --- per-frame streaming budgets for voxel chunks ---
const UPLOAD_VOXEL_PER_FRAME: usize = 4;
const DISPATCH_VOXEL_PER_FRAME: usize = 4;
const MAX_PENDING_VOXELS: usize = 16;

/// Convert a `CpuMesh` vertex slice to GPU-ready `Vertex` bytes.
fn cpu_verts_to_gpu(cpu: &CpuMesh) -> Vec<Vertex> {
    cpu.vertices.iter().copied().map(Vertex::from).collect()
}

impl<'a> Renderer<'a> {
    pub(super) fn upload_lod_buffer(&mut self, key: LodKey, mesh: CpuMesh) {
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
    }

    pub(super) fn process_load_queue(&mut self, _player_pos: Vec3, planet: &PlanetData) {
        // Drain completed voxel meshes from the channel.
        let mut upload_budget = UPLOAD_VOXEL_PER_FRAME;
        while let Ok((key, mesh)) = self.mesh_rx.try_recv() {
            self.pending_chunks.remove(&key);
            let is_dirty_rebuild = self.pending_dirty.remove(&key);
            // Skip stale initial-load results (chunk was evicted while job was in flight).
            // Always accept dirty rebuilds — they replace an existing chunk with updated geometry.
            if !mesh.is_empty() && (is_dirty_rebuild || !self.chunks.contains_key(&key)) {
                self.upload_chunk_buffers(key, mesh);
                upload_budget -= 1;
            }
            if upload_budget == 0 {
                break;
            }
        }

        // --- DIRTY CHUNKS (player edits) — dispatched with priority ---
        let dirty: Vec<SurfaceChunkKey> = self
            .dirty_chunks
            .drain()
            .filter(|k| !self.pending_chunks.contains(k))
            .collect();

        for key in dirty {
            self.pending_chunks.insert(key);
            self.pending_dirty.insert(key);
            let planet_clone = planet.clone();
            let tx = self.mesh_tx.clone();
            rayon::spawn(move || {
                let mesh = MeshGen::build_chunk(key, &planet_clone);
                let _ = tx.send((key, mesh));
            });
        }

        if upload_budget == 0 || self.load_queue.is_empty() {
            return;
        }
        if self.pending_chunks.len() >= MAX_PENDING_VOXELS {
            return;
        }

        // Dispatch new initial-load jobs.
        for _ in 0..DISPATCH_VOXEL_PER_FRAME {
            let Some(key) = self.load_queue.pop() else {
                break;
            };
            if self.chunks.contains_key(&key) || self.pending_chunks.contains(&key) {
                continue;
            }
            self.pending_chunks.insert(key);
            let planet_clone = planet.clone();
            let tx = self.mesh_tx.clone();
            rayon::spawn(move || {
                let mesh = MeshGen::build_chunk(key, &planet_clone);
                let _ = tx.send((key, mesh));
            });
        }
    }

    pub fn force_reload_all(&mut self, planet: &PlanetData, player_pos: Vec3) {
        self.chunks.clear();
        self.lod_chunks.clear();
        self.load_queue.clear();
        self.pending_chunks.clear();
        self.pending_dirty.clear();
        self.pending_lods.clear();
        self.dirty_chunks.clear();
        self.player_chunk_pos = None;
        self.update_view(player_pos, planet);
    }

    /// Mark a voxel edit as dirty — all affected chunk meshes will be rebuilt
    /// asynchronously on the next process_load_queue call.
    /// Non-blocking: never touches the GPU from the main thread.
    pub fn refresh_neighbors(&mut self, id: VoxelCoord, _planet: &PlanetData) {
        let u_c = id.u / CHUNK_SIZE;
        let v_c = id.v / CHUNK_SIZE;
        let keys = [
            SurfaceChunkKey {
                face: id.face,
                u_idx: u_c,
                v_idx: v_c,
            },
            SurfaceChunkKey {
                face: id.face,
                u_idx: u_c.saturating_sub(1),
                v_idx: v_c,
            },
            SurfaceChunkKey {
                face: id.face,
                u_idx: u_c + 1,
                v_idx: v_c,
            },
            SurfaceChunkKey {
                face: id.face,
                u_idx: u_c,
                v_idx: v_c.saturating_sub(1),
            },
            SurfaceChunkKey {
                face: id.face,
                u_idx: u_c,
                v_idx: v_c + 1,
            },
        ];
        for key in keys {
            self.dirty_chunks.insert(key);
        }
    }

    fn upload_chunk_buffers(&mut self, key: SurfaceChunkKey, mesh: CpuMesh) {
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
            params: [start_opacity, 0.0, 0.0, 0.0],
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
