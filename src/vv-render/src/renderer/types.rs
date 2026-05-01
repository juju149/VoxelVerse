use bytemuck::{Pod, Zeroable};
use std::time::Duration;

use vv_diagnostics::{
    GpuCounters, LodCounters, MeshCounters, StreamingCounters,
};

use crate::atmosphere::AtmosphereUniform;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(crate) struct GlobalUniform {
    pub view_proj: [f32; 16],
    pub light_view_proj: [f32; 16],
    pub cam_pos: [f32; 4],
    pub atmosphere: AtmosphereUniform,
    /// Inverse of view_proj, used by the sky shader to reconstruct world-space ray directions.
    pub inv_view_proj: [f32; 16],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(crate) struct LocalUniform {
    pub model: [f32; 16],
    /// x = opacity
    pub params: [f32; 4],
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct MeshJobResult<K> {
    pub key: K,
    pub vertices: usize,
    pub indices: usize,
    pub duration: Duration,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RendererFrameTelemetry {
    pub streaming: StreamingCounters,
    pub lod: LodCounters,
    pub mesh: MeshCounters,
    pub gpu: GpuCounters,
    pub render_prep_time: Duration,
    pub lod_coverage_time: Duration,
    pub chunk_streaming_time: Duration,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum MeshJobKind {
    Chunk,
    Lod,
    Remesh,
}