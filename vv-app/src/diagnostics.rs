use sysinfo::System;

/// Startup diagnostics printed once at launch.
pub struct SystemDiagnostics;

impl SystemDiagnostics {
    pub fn print_startup_info() {
        let mut sys = System::new_all();
        sys.refresh_all();

        println!("\n==========================================");
        println!("           SYSTEM DIAGNOSTICS            ");
        println!("==========================================");
        println!("OS       : {} {}", System::name().unwrap_or_default(), System::os_version().unwrap_or_default());
        println!("Kernel   : {}", System::kernel_version().unwrap_or_default());
        println!("Hostname : {}", System::host_name().unwrap_or_default());
        let cpus = sys.cpus();
        if !cpus.is_empty() {
            println!("CPU      : {}", cpus[0].brand().trim());
            println!("Cores    : {} logical", cpus.len());
        }
        let total = sys.total_memory() as f32 / (1024.0 * 1024.0 * 1024.0);
        let used  = sys.used_memory()  as f32 / (1024.0 * 1024.0 * 1024.0);
        println!("Memory   : {:.2} GB used / {:.2} GB total", used, total);
        println!("==========================================\n");
    }
}
