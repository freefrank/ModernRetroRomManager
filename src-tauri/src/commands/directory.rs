use crate::settings::{get_settings, update_setting, DirectoryConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryInfo {
    pub path: String,
    pub is_root_directory: bool,
    pub metadata_format: String,
    pub system_id: Option<String>,
}

impl From<DirectoryConfig> for DirectoryInfo {
    fn from(c: DirectoryConfig) -> Self {
        Self {
            path: c.path,
            is_root_directory: c.is_root_directory,
            metadata_format: c.metadata_format,
            system_id: c.system_id,
        }
    }
}

/// 添加目录到配置
#[tauri::command]
pub fn add_directory(
    path: String,
    metadata_format: String,
    is_root: bool,
    system_id: Option<String>,
) -> Result<DirectoryInfo, String> {
    update_setting(|settings| {
        // 检查是否已存在
        if !settings.directories.iter().any(|d| d.path == path) {
            settings.directories.push(DirectoryConfig {
                path: path.clone(),
                metadata_format: metadata_format.clone(),
                is_root_directory: is_root,
                system_id: system_id.clone(),
            });
        }
    })
    .map_err(|e| e.to_string())?;

    Ok(DirectoryInfo {
        path,
        is_root_directory: is_root,
        metadata_format,
        system_id,
    })
}

/// 从配置移除目录
#[tauri::command]
pub fn remove_directory(path: String) -> Result<(), String> {
    update_setting(|settings| {
        settings.directories.retain(|d| d.path != path);
    })
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 获取所有配置的目录
#[tauri::command]
pub fn get_directories() -> Vec<DirectoryInfo> {
    get_settings()
        .directories
        .into_iter()
        .map(DirectoryInfo::from)
        .collect()
}
