use glam::Vec3;
use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;

use vv_core::{ChunkKey, LodKey};
use vv_mesh::{MeshGen, Vertex};
use vv_world_runtime::PlanetData;

use crate::ChunkMesh;

use super::types::{LocalUniform, MeshJobKind, MeshJobResult};
use super::Renderer;

impl<'a> Renderer<'a> {
    pub(super) fn upload_lod_buffer(&mut self, key: LodKey, v: Vec<Vertex>, i: Vec<u32>) {
        let upload_start = Instant::now();
        self.record_upload(v.len(), i.len());
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
        let uniform_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("LOD Uniform"),
                contents: bytemuck::cast_slice(&[LocalUniform {
                    model: glam::Mat4::IDENTITY.to_cols_array(),
                    params: [1.0, 0.0, 0.0, 0.0],
                }]),
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
        let (center, radius) = Self::bounds_from_verts(&v);
        self.lod_chunks.insert(
            key,
            ChunkMesh {
                v_buf,
                i_buf,
                num_inds: i.len() as u32,
                num_verts: v.len(),
                uniform_buf,
                bind_group,
                center,
                radius,
            },
        );
        self.frame_telemetry.gpu.upload_time += upload_start.elapsed();
    }

    pub(super) fn upload_chunk_buffers(&mut self, key: ChunkKey, v: Vec<Vertex>, i: Vec<u32>) {
        let upload_start = Instant::now();
        self.record_upload(v.len(), i.len());
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
        let uniform_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Uniform"),
                contents: bytemuck::cast_slice(&[LocalUniform {
                    model: glam::Mat4::IDENTITY.to_cols_array(),
                    params: [1.0, 0.0, 0.0, 0.0],
                }]),
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
        let (center, radius) = Self::bounds_from_verts(&v);
        self.chunks.insert(
            key,
            ChunkMesh {
                v_buf,
                i_buf,
                num_inds: i.len() as u32,
                num_verts: v.len(),
                uniform_buf,
                bind_group,
                center,
                radius,
            },
        );
        self.frame_telemetry.gpu.upload_time += upload_start.elapsed();
    }

    fn bounds_from_verts(v: &[Vertex]) -> (Vec3, f32) {
        if v.is_empty() {
            return (Vec3::ZERO, 0.0);
        }
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for vert in v {
            let p = Vec3::from_array(vert.pos);
            min = min.min(p);
            max = max.max(p);
        }
        let center = (min + max) * 0.5;
        (center, min.distance(max) * 0.5)
    }

    pub(super) fn process_load_queue(&mut self, _player_pos: Vec3, planet: &PlanetData) {
        let mut uploads = 0usize;
        while uploads < self.lod_cfg.chunk_uploads_per_frame
            && self.frame_telemetry.gpu.uploads < self.lod_cfg.max_gpu_uploads_per_frame
        {
            let Ok((job, v, i)) = self.mesh_rx.try_recv() else {
                break;
            };
            self.pending_chunks.remove(&job.key);
            self.record_mesh_job(job.duration, job.vertices, job.indices, MeshJobKind::Chunk);
            if !v.is_empty() {
                self.upload_chunk_buffers(job.key, v, i);
                self.frame_telemetry.streaming.chunks_uploaded += 1;
                uploads += 1;
            } else {
                self.frame_telemetry.streaming.empty_chunks += 1;
            }
        }
        if self.load_queue.is_empty()
            || self.pending_chunks.len() >= self.lod_cfg.max_pending_chunk_jobs
        {
            return;
        }
        for _ in 0..self.lod_cfg.chunk_jobs_per_frame {
            if self.pending_chunks.len() >= self.lod_cfg.max_pending_chunk_jobs {
                break;
            }
            if let Some(key) = self.load_queue.pop() {
                if self.chunks.contains_key(&key) || self.pending_chunks.contains(&key) {
                    continue;
                }
                self.pending_chunks.insert(key);
                let p = planet.clone();
                let tx = self.mesh_tx.clone();
                let blocks = self.block_content.clone();
                std::thread::spawn(move || {
                    let start = Instant::now();
                    let (v, i) = MeshGen::build_chunk(key, &p, &blocks);
                    let job = MeshJobResult {
                        key,
                        vertices: v.len(),
                        indices: i.len(),
                        duration: start.elapsed(),
                    };
                    let _ = tx.send((job, v, i));
                });
                self.frame_telemetry.streaming.chunk_jobs_started += 1;
            } else {
                break;
            }
        }
    }

    fn record_upload(&mut self, vertices: usize, indices: usize) {
        self.frame_telemetry.gpu.uploads += 1;
        self.frame_telemetry.gpu.upload_vertices += vertices;
        self.frame_telemetry.gpu.upload_indices += indices;
    }

    pub(super) fn record_mesh_job(
        &mut self,
        duration: Duration,
        vertices: usize,
        indices: usize,
        kind: MeshJobKind,
    ) {
        match kind {
            MeshJobKind::Chunk => self.frame_telemetry.mesh.chunk_jobs_completed += 1,
            MeshJobKind::Lod => self.frame_telemetry.mesh.lod_jobs_completed += 1,
            MeshJobKind::Remesh => self.frame_telemetry.mesh.remeshes += 1,
        }
        self.frame_telemetry.mesh.total_job_time += duration;
        self.frame_telemetry.mesh.max_job_time =
            self.frame_telemetry.mesh.max_job_time.max(duration);
        self.frame_telemetry.mesh.vertices += vertices;
        self.frame_telemetry.mesh.indices += indices;
    }
}
