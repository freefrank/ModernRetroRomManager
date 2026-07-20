//! 诊断相关命令:定位/打开应用日志,便于用户在无控制台的发行版里取回日志排查问题。

use tauri_plugin_opener::OpenerExt;

/// 返回日志文件的完整路径(字符串)。
#[tauri::command]
pub fn get_log_path() -> String {
    crate::logging::log_path().to_string_lossy().to_string()
}

/// 在系统文件管理器中打开日志所在目录。
#[tauri::command]
pub fn open_log_directory(app: tauri::AppHandle) -> Result<(), String> {
    let dir = crate::logging::log_dir();
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|error| error.to_string())
}
