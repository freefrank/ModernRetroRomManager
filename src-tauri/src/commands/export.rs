use tauri::AppHandle;

#[derive(Clone, serde::Serialize)]
struct ExportProgress {
    current: usize,
    total: usize,
    message: String,
    finished: bool,
}

#[tauri::command]
pub async fn export_to_emulationstation(_app: AppHandle, _target_dir: String) -> Result<(), String> {
    Err("Export to EmulationStation is temporarily disabled in this version due to architecture migration.".to_string())
}

#[tauri::command]
pub async fn export_to_pegasus(_app: AppHandle, _target_dir: String) -> Result<(), String> {
    Err("Export to Pegasus is temporarily disabled in this version due to architecture migration.".to_string())
}
