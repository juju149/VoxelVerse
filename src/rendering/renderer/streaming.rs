use super::{LocalUniform, Renderer};
use crate::meshing::MeshGen;
use crate::rendering::lod_animation::AnyKey;
use crate::rendering::types::{ChunkMesh, Vertex};
use crate::voxel::{ChunkKey, LodKey, VoxelCoord, CHUNK_SIZE};
use crate::world::PlanetData;
use glam::Vec3;
use wgpu::util::DeviceExt;

impl<'a> Renderer<'a> {
    pub(super) fn upload_lod_buffer(&mut self, key: LodKey, v: Vec<Vertex>, i: Vec<u32>) {
        let v_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&v),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        let i_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&i),
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

        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for vert in &v {
            let p = Vec3::from_array(vert.pos);
            min = min.min(p);
            max = max.max(p);
        }
        let real_center = (min + max) * 0.5;
        let real_radius = min.distance(max) * 0.5;

        self.lod_chunks.insert(
            key,
            ChunkMesh {
                v_buf,
                i_buf,
                num_inds: i.len() as u32,
                num_verts: v.len(),
                uniform_buf,
                bind_group,
                center: real_center, // <--- ADDED
                radius: real_radius, // <--- ADDED
            },
        );
        self.animator.start_spawn(AnyKey::Lod(key));
    }

    pub(super) fn process_load_queue(&mut self, _player_pos: Vec3, planet: &PlanetData) {
        let mut upload_budget = 4;
        while let Ok((key, v, i)) = self.mesh_rx.try_recv() {
            self.pending_chunks.remove(&key);
            if !v.is_empty() {
                self.upload_chunk_buffers(key, v, i);
                upload_budget -= 1;
            }
            if upload_budget <= 0 {
                break;
            }
        }

        if upload_budget <= 0 {
            return;
        }
        if self.load_queue.is_empty() {
            return;
        }
        if self.pending_chunks.len() >= 12 {
            return;
        }

        let chunks_to_spawn = 4;
        for _ in 0..chunks_to_spawn {
            if let Some(key) = self.load_queue.pop() {
                if self.chunks.contains_key(&key) || self.pending_chunks.contains(&key) {
                    continue;
                }
                self.pending_chunks.insert(key);
                let planet_clone = planet.clone();
                let tx = self.mesh_tx.clone();
                std::thread::spawn(move || {
                    let (v, i) = MeshGen::build_chunk(key, &planet_clone);
                    let _ = tx.send((key, v, i));
                });
            } else {
                break;
            }
        }
    }

    pub fn force_reload_all(&mut self, planet: &PlanetData, player_pos: Vec3) {
        self.chunks.clear();
        self.lod_chunks.clear();
        self.load_queue.clear();
        self.pending_chunks.clear();
        self.pending_lods.clear();
        self.player_chunk_pos = None;
        self.update_view(player_pos, planet);
    }

    pub fn refresh_neighbors(&mut self, id: VoxelCoord, planet: &PlanetData) {
        let u_c = id.u / CHUNK_SIZE;
        let v_c = id.v / CHUNK_SIZE;
        let keys = vec![
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
        ];
        for key in keys {
            if self.chunks.contains_key(&key) {
                let (v, i) = MeshGen::build_chunk(key, planet);
                if v.is_empty() {
                    self.chunks.remove(&key);
                } else {
                    self.upload_chunk_buffers(key, v, i);
                }
            }
        }
    }

    fn upload_chunk_buffers(&mut self, key: ChunkKey, v: Vec<Vertex>, i: Vec<u32>) {
        let v_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&v),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        let i_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&i),
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

        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        if v.is_empty() {
            min = Vec3::ZERO;
            max = Vec3::ZERO;
        } else {
            for vert in &v {
                let p = Vec3::from_array(vert.pos);
                min = min.min(p);
                max = max.max(p);
            }
        }
        let real_center = (min + max) * 0.5;
        let real_radius = min.distance(max) * 0.5;

        self.chunks.insert(
            key,
            ChunkMesh {
                v_buf,
                i_buf,
                num_inds: i.len() as u32,
                num_verts: v.len(),
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
