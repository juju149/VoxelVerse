use serde::Serialize;

pub type MetricName = &'static str;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub enum DiagnosticsProfile {
    Off,
    #[default]
    Normal,
    High,
    Verbose,
}

impl DiagnosticsProfile {
    pub fn captures_scopes(self) -> bool {
        matches!(self, Self::High | Self::Verbose)
    }

    pub fn captures_verbose(self) -> bool {
        matches!(self, Self::Verbose)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum DiagnosticCategory {
    Frame,
    App,
    Input,
    Gameplay,
    World,
    Worldgen,
    Meshing,
    Streaming,
    Renderer,
    Gpu,
    RenderPass,
    Lod,
    Chunks,
    Props,
    Content,
    Ui,
    Audio,
    Memory,
    Warning,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct CounterSample {
    pub name: MetricName,
    pub category: DiagnosticCategory,
    pub value: u64,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct GaugeSample {
    pub name: MetricName,
    pub category: DiagnosticCategory,
    pub value: f64,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct CpuScopeTiming {
    pub name: MetricName,
    pub category: DiagnosticCategory,
    pub elapsed_ms: f32,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct RenderPassTiming {
    pub name: MetricName,
    pub elapsed_ms: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiagnosticWarning {
    pub category: DiagnosticCategory,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct PlayerCameraSnapshot {
    pub player_position: [f32; 3],
    pub camera_direction: [f32; 3],
    pub current_chunk: Option<[i32; 3]>,
    pub current_face: Option<u8>,
    pub current_lod: Option<u8>,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct WorkloadSnapshot {
    pub pending_jobs: u32,
    pub pending_chunks: u32,
    pub pending_lods: u32,
    pub uploaded_meshes: u32,
    pub upload_bytes: u64,
    pub draw_calls: u32,
    pub gpu_memory_bytes: u64,
    pub dirty_chunks: u32,
    pub worldgen_samples: u32,
    pub meshed_chunks: u32,
    pub baked_props: u32,
    pub missing_model_lookups: u32,
}
