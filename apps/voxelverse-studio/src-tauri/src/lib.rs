#[tauri::command]
fn studio_version() -> &'static str {
    "0.1.0"
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![studio_version])
        .run(tauri::generate_context!())
        .expect("failed to run VoxelVerse Studio");
}
