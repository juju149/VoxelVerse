//! Auto-detected hardware performance tier and the rendering knobs derived
//! from it.  Single responsibility: pick a [`PerfTier`] at startup based on
//! CPU/RAM/GPU class, and expose every render-side setting that scales with
//! it (shadow map size, scheduler budgets, PCF kernel, LOD aggressiveness).
//!
//! The user can force a tier with the `VV_PERF=low|medium|high|ultra` env
//! variable; otherwise we fall back to [`PerfTier::detect`].

use crate::lod_streaming::{LodSplitCurve, LodStreamingConfig};
use crate::quality::{PcfQuality, QualitySettings, RenderQualityProfile};
use vv_meshing::SchedulerBudget;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PerfTier {
    Low,
    Medium,
    High,
    Ultra,
}

impl PerfTier {
    pub fn label(self) -> &'static str {
        match self {
            PerfTier::Low => "Low",
            PerfTier::Medium => "Medium",
            PerfTier::High => "High",
            PerfTier::Ultra => "Ultra",
        }
    }

    /// Parse `low|medium|high|ultra` (case-insensitive). Returns `None` on
    /// unknown input so callers can fall back to auto-detection.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "low" | "lo" | "1" => Some(PerfTier::Low),
            "medium" | "med" | "2" => Some(PerfTier::Medium),
            "high" | "hi" | "3" => Some(PerfTier::High),
            "ultra" | "max" | "4" => Some(PerfTier::Ultra),
            _ => None,
        }
    }

    /// Heuristic: rank the host as Low/Medium/High/Ultra from CPU cores, RAM
    /// and the wgpu adapter's device type.  Errs on the conservative side —
    /// integrated GPUs never go above Medium even with lots of RAM, since the
    /// shadow pass and triplanar grain are fill-rate bound.
    pub fn detect(adapter_info: &wgpu::AdapterInfo) -> Self {
        use sysinfo::System;
        let mut sys = System::new();
        sys.refresh_memory();
        sys.refresh_cpu();
        let ram_gb = sys.total_memory() as f32 / (1024.0 * 1024.0 * 1024.0);
        let cores = sys.cpus().len() as u32;

        let is_discrete = matches!(adapter_info.device_type, wgpu::DeviceType::DiscreteGpu);
        let is_integrated = matches!(adapter_info.device_type, wgpu::DeviceType::IntegratedGpu);
        let is_software = matches!(
            adapter_info.device_type,
            wgpu::DeviceType::Cpu | wgpu::DeviceType::Other
        );

        if is_software {
            return PerfTier::Low;
        }

        if is_discrete && ram_gb >= 24.0 && cores >= 12 {
            return PerfTier::Ultra;
        }
        if is_discrete && ram_gb >= 16.0 && cores >= 8 {
            return PerfTier::High;
        }
        if is_discrete || (is_integrated && ram_gb >= 16.0 && cores >= 8) {
            return PerfTier::Medium;
        }
        PerfTier::Low
    }

    /// Resolve a tier honouring the `VV_PERF` env override.
    pub fn resolve(adapter_info: &wgpu::AdapterInfo) -> Self {
        if let Ok(forced) = std::env::var("VV_PERF") {
            if let Some(t) = Self::from_str(&forced) {
                println!("[perf] tier forced via VV_PERF = {}", t.label());
                return t;
            }
            eprintln!(
                "[perf] VV_PERF=\"{}\" unrecognised — falling back to auto-detect",
                forced
            );
        }
        Self::detect(adapter_info)
    }
}

/// Bundle of every render-side knob that scales with the perf tier.
#[derive(Clone, Copy, Debug)]
pub struct PerfProfile {
    pub tier: PerfTier,
    /// Edge length (and width) of the square shadow depth texture.
    pub shadow_map_size: u32,
    pub lod_streaming: LodStreamingConfig,
    pub quality: QualitySettings,
    pub scheduler: SchedulerBudget,
}

impl PerfProfile {
    pub fn for_tier(tier: PerfTier) -> Self {
        match tier {
            PerfTier::Low => Self {
                tier,
                shadow_map_size: 1024,
                lod_streaming: LodStreamingConfig {
                    lod_near_radius: 64.0,
                    lod_split_curve: LodSplitCurve {
                        far_factor: 2.2,
                        mid_factor: 3.9,
                        near_factor: 6.6,
                        voxel_factor: 9.9,
                    },
                    lod_hysteresis: 0.18,
                    lod_transition_time: 1.0,
                    max_visible_voxel_chunks: 144,
                    max_visible_lod_tiles: 1024,
                },
                quality: QualitySettings {
                    profile: RenderQualityProfile::Potato,
                    triplanar_grain: false,
                    pcf: PcfQuality::Low,
                    color_only_mode: false,
                    volumetric_fog: false,
                    volumetric_clouds: false,
                    fxaa: false,
                    bloom: false,
                    cloud_steps: 0,
                },
                scheduler: SchedulerBudget {
                    upload_voxel: 3,
                    dispatch_voxel: 3,
                    max_pending_voxel: 16,
                    upload_lod: 5,
                    dispatch_lod: 6,
                    max_pending_lod: 16,
                    upload_bytes_per_frame: 6 * 1024 * 1024,
                    upload_time_budget_ms: 3.5,
                },
            },
            PerfTier::Medium => Self {
                tier,
                shadow_map_size: 2048,
                lod_streaming: LodStreamingConfig {
                    lod_near_radius: 88.0,
                    lod_split_curve: LodSplitCurve {
                        far_factor: 3.2,
                        mid_factor: 5.6,
                        near_factor: 9.6,
                        voxel_factor: 14.4,
                    },
                    lod_hysteresis: 0.16,
                    lod_transition_time: 1.15,
                    max_visible_voxel_chunks: 256,
                    max_visible_lod_tiles: 2048,
                },
                quality: QualitySettings {
                    profile: RenderQualityProfile::Balanced,
                    triplanar_grain: false,
                    pcf: PcfQuality::Low,
                    color_only_mode: false,
                    volumetric_fog: true,
                    volumetric_clouds: false,
                    fxaa: true,
                    bloom: false,
                    cloud_steps: 6,
                },
                scheduler: SchedulerBudget::default(),
            },
            PerfTier::High => Self {
                tier,
                shadow_map_size: 4096,
                lod_streaming: LodStreamingConfig {
                    lod_near_radius: 112.0,
                    lod_split_curve: LodSplitCurve {
                        far_factor: 4.0,
                        mid_factor: 7.0,
                        near_factor: 12.0,
                        voxel_factor: 18.0,
                    },
                    lod_hysteresis: 0.15,
                    lod_transition_time: 1.25,
                    max_visible_voxel_chunks: 384,
                    max_visible_lod_tiles: 3072,
                },
                quality: QualitySettings {
                    profile: RenderQualityProfile::High,
                    triplanar_grain: false,
                    pcf: PcfQuality::Medium,
                    color_only_mode: false,
                    volumetric_fog: true,
                    volumetric_clouds: true,
                    fxaa: true,
                    bloom: true,
                    cloud_steps: 10,
                },
                scheduler: SchedulerBudget {
                    upload_voxel: 10,
                    dispatch_voxel: 10,
                    max_pending_voxel: 64,
                    upload_lod: 14,
                    dispatch_lod: 16,
                    max_pending_lod: 40,
                    upload_bytes_per_frame: 20 * 1024 * 1024,
                    upload_time_budget_ms: 6.0,
                },
            },
            PerfTier::Ultra => Self {
                tier,
                shadow_map_size: 4096,
                lod_streaming: LodStreamingConfig {
                    lod_near_radius: 144.0,
                    lod_split_curve: LodSplitCurve {
                        far_factor: 5.0,
                        mid_factor: 8.75,
                        near_factor: 15.0,
                        voxel_factor: 22.5,
                    },
                    lod_hysteresis: 0.14,
                    lod_transition_time: 1.35,
                    max_visible_voxel_chunks: 512,
                    max_visible_lod_tiles: 4096,
                },
                quality: QualitySettings {
                    profile: RenderQualityProfile::Ultra,
                    triplanar_grain: false,
                    pcf: PcfQuality::High,
                    color_only_mode: false,
                    volumetric_fog: true,
                    volumetric_clouds: true,
                    fxaa: true,
                    bloom: true,
                    cloud_steps: 14,
                },
                scheduler: SchedulerBudget {
                    upload_voxel: 14,
                    dispatch_voxel: 12,
                    max_pending_voxel: 96,
                    upload_lod: 18,
                    dispatch_lod: 22,
                    max_pending_lod: 56,
                    upload_bytes_per_frame: 28 * 1024 * 1024,
                    upload_time_budget_ms: 7.0,
                },
            },
        }
    }

    pub fn print(&self) {
        println!(
            "[perf] tier={} profile={:?} shadow={}px lod_near={:.1} hyst={:.2} transition={:.2}s max_chunks={} max_lods={} pcf={:?} triplanar={} fog={} clouds={} fxaa={} bloom={} \
             voxel(disp/up/pend)={}/{}/{} lod(disp/up/pend)={}/{}/{}",
            self.tier.label(),
            self.quality.profile,
            self.shadow_map_size,
            self.lod_streaming.lod_near_radius,
            self.lod_streaming.lod_hysteresis,
            self.lod_streaming.lod_transition_time,
            self.lod_streaming.max_visible_voxel_chunks,
            self.lod_streaming.max_visible_lod_tiles,
            self.quality.pcf,
            self.quality.triplanar_grain,
            self.quality.volumetric_fog,
            self.quality.volumetric_clouds,
            self.quality.fxaa,
            self.quality.bloom,
            self.scheduler.dispatch_voxel,
            self.scheduler.upload_voxel,
            self.scheduler.max_pending_voxel,
            self.scheduler.dispatch_lod,
            self.scheduler.upload_lod,
            self.scheduler.max_pending_lod,
        );
    }
}
