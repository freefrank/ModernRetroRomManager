use crate::config;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct PathValidation {
    pub path: String,
    pub exists: bool,
    pub is_directory: bool,
    pub readable: bool,
    pub writable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: Option<u64>,
}

/// 验证路径是否有效
#[tauri::command]
pub fn validate_path(path: String) -> Result<PathValidation, String> {
    let path_buf = PathBuf::from(&path);
    
    let exists = path_buf.exists();
    let is_directory = path_buf.is_dir();
    
    // 检查可读性
    let readable = if exists {
        std::fs::read_dir(&path_buf).is_ok()
    } else {
        false
    };
    
    // 检查可写性（尝试在目标目录创建临时文件）
    let writable = if exists && is_directory {
        let temp_file = path_buf.join(".write_test");
        match std::fs::write(&temp_file, "") {
            Ok(_) => {
                let _ = std::fs::remove_file(&temp_file);
                true
            }
            Err(_) => false,
        }
    } else {
        false
    };
    
    Ok(PathValidation {
        path,
        exists,
        is_directory,
        readable,
        writable,
    })
}

/// 列出目录内容
#[tauri::command]
pub fn list_directory(path: String) -> Result<Vec<DirectoryEntry>, String> {
    let path_buf = PathBuf::from(&path);
    
    if !path_buf.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    
    if !path_buf.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }
    
    let entries = std::fs::read_dir(&path_buf)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    let mut result: Vec<DirectoryEntry> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            
            // 跳过隐藏文件（以 . 开头）
            if name.starts_with('.') {
                return None;
            }
            
            Some(DirectoryEntry {
                name,
                path: entry.path().to_string_lossy().to_string(),
                is_directory: metadata.is_dir(),
                size: if metadata.is_file() { Some(metadata.len()) } else { None },
            })
        })
        .collect();
    
    // 目录优先，然后按名称排序
    result.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
    
    Ok(result)
}

/// 获取配置目录路径
#[tauri::command]
pub fn get_config_dir() -> String {
    config::get_config_dir().to_string_lossy().to_string()
}

/// 获取媒体目录路径
#[tauri::command]
pub fn get_media_dir() -> String {
    config::get_media_dir().to_string_lossy().to_string()
}

/// 获取应用配置
#[tauri::command]
pub fn get_app_settings() -> Result<crate::settings::AppSettings, String> {
    Ok(crate::settings::get_settings())
}

/// 保存应用配置
#[tauri::command]
pub fn save_app_settings(settings: crate::settings::AppSettings) -> Result<(), String> {
    crate::settings::save_settings(&settings).map_err(|e| e.to_string())
}

/// 更新单个配置项
#[tauri::command]
pub fn update_app_setting(key: String, value: String) -> Result<crate::settings::AppSettings, String> {
    crate::settings::update_setting(|s| {
        match key.as_str() {
            "theme" => s.theme = value.clone(),
            "language" => s.language = value.clone(),
            "view_mode" => s.view_mode = value.clone(),
            _ => {}
        }
    }).map_err(|e| e.to_string())
}

/// 获取所有 Scraper 配置
#[tauri::command]
pub fn get_scraper_configs() -> Result<std::collections::HashMap<String, crate::settings::ScraperConfig>, String> {
    Ok(crate::settings::get_settings().scrapers)
}

/// 保存单个 Scraper 配置
#[tauri::command]
pub fn save_scraper_config(provider: String, config: crate::settings::ScraperConfig) -> Result<(), String> {
    crate::settings::update_setting(|s| {
        s.scrapers.insert(provider, config);
    }).map_err(|e| e.to_string())?;
    Ok(())
}
