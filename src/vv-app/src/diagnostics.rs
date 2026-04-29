use sysinfo::System;
use vv_diagnostics::{emit, DiagnosticConfig, LogDomain, LogLevel};

/// Startup diagnostics printed once at launch.
pub struct SystemDiagnostics;

impl SystemDiagnostics {
    pub fn print_startup_info(config: DiagnosticConfig) {
        let mut sys = System::new_all();
        sys.refresh_all();

        emit(
            config,
            LogLevel::Info,
            LogDomain::Startup,
            format!(
                "system os=\"{} {}\" kernel=\"{}\" host=\"{}\"",
                System::name().unwrap_or_default(),
                System::os_version().unwrap_or_default(),
                System::kernel_version().unwrap_or_default(),
                System::host_name().unwrap_or_default()
            ),
        );
        let cpus = sys.cpus();
        if !cpus.is_empty() {
            emit(
                config,
                LogLevel::Info,
                LogDomain::Startup,
                format!(
                    "cpu=\"{}\" logical_cores={}",
                    cpus[0].brand().trim(),
                    cpus.len()
                ),
            );
        }
        let total = sys.total_memory() as f32 / (1024.0 * 1024.0 * 1024.0);
        let used = sys.used_memory() as f32 / (1024.0 * 1024.0 * 1024.0);
        emit(
            config,
            LogLevel::Info,
            LogDomain::Memory,
            format!("system used={:.2}GB total={:.2}GB", used, total),
        );
    }
}
