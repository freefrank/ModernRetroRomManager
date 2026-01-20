use crate::settings::{get_settings, update_setting, DirectoryConfig};
use crate::system_mapping::find_mapping_by_folder;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 规范化路径：统一使用正斜杠，小写盘符
fn normalize_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    
    // 处理 Windows 盘符 (如 Z:/ -> z:/)
    if normalized.len() >= 2 && normalized.chars().nth(1) == Some(':') {
        let mut chars: Vec<char> = normalized.chars().collect();
        chars[0] = chars[0].to_ascii_lowercase();
        chars.into_iter().collect()
    } else {
        normalized
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataFileInfo {
    pub format: String,
    pub format_name: String,
    pub file_path: String,
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubDirectoryInfo {
    pub name: String,
    pub path: String,
    pub metadata_files: Vec<MetadataFileInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryScanResult {
    pub is_root_directory: bool,
    pub metadata_files: Vec<MetadataFileInfo>,
    pub sub_directories: Vec<SubDirectoryInfo>,
}


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
#[allow(non_snake_case)]
pub fn add_directory(
    path: String,
    metadataFormat: String,
    isRoot: bool,
    systemId: Option<String>,
) -> Result<DirectoryInfo, String> {
    let normalized = normalize_path(&path);

    update_setting(|settings| {
        // 使用规范化路径去重
        if !settings.directories.iter().any(|d| normalize_path(&d.path) == normalized) {
            settings.directories.push(DirectoryConfig {
                path: normalized.clone(),
                metadata_format: metadataFormat.clone(),
                is_root_directory: isRoot,
                system_id: systemId.clone(),
            });
        }
    })
    .map_err(|e| e.to_string())?;

    Ok(DirectoryInfo {
        path: normalized,
        is_root_directory: isRoot,
        metadata_format: metadataFormat,
        system_id: systemId,
    })

}

/// 从配置移除目录
#[tauri::command]
pub fn remove_directory(path: String) -> Result<(), String> {
    let normalized = normalize_path(&path);
    update_setting(|settings| {
        settings.directories.retain(|d| normalize_path(&d.path) != normalized);
    })
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_directories() -> Vec<DirectoryInfo> {
    get_settings()
        .directories
        .into_iter()
        .map(DirectoryInfo::from)
        .collect()
}

fn detect_metadata_in_dir(dir_path: &Path) -> Vec<MetadataFileInfo> {
    let mut results = Vec::new();

    let pegasus_path = dir_path.join("metadata.pegasus.txt");
    if pegasus_path.exists() {
        results.push(MetadataFileInfo {
            format: "pegasus".to_string(),
            format_name: "Pegasus".to_string(),
            file_path: pegasus_path.to_string_lossy().to_string(),
            file_name: "metadata.pegasus.txt".to_string(),
        });
    }

    let metadata_path = dir_path.join("metadata.txt");
    if metadata_path.exists() {
        results.push(MetadataFileInfo {
            format: "pegasus".to_string(),
            format_name: "Pegasus".to_string(),
            file_path: metadata_path.to_string_lossy().to_string(),
            file_name: "metadata.txt".to_string(),
        });
    }

    let gamelist_path = dir_path.join("gamelist.xml");
    if gamelist_path.exists() {
        results.push(MetadataFileInfo {
            format: "emulationstation".to_string(),
            format_name: "EmulationStation".to_string(),
            file_path: gamelist_path.to_string_lossy().to_string(),
            file_name: "gamelist.xml".to_string(),
        });
    }

    results
}

#[tauri::command]
pub fn detect_metadata_files(path: String) -> Result<Vec<MetadataFileInfo>, String> {
    let base_path = Path::new(&path);

    if !base_path.exists() {
        return Err("Path does not exist".to_string());
    }

    if !base_path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    Ok(detect_metadata_in_dir(base_path))
}

#[tauri::command]
pub fn scan_directory(path: String) -> Result<DirectoryScanResult, String> {
    let base_path = Path::new(&path);

    if !base_path.exists() {
        return Err("Path does not exist".to_string());
    }

    if !base_path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    let metadata_files = detect_metadata_in_dir(base_path);
    
    // 如果当前目录有 metadata 文件，这是单系统目录
    if !metadata_files.is_empty() {
        return Ok(DirectoryScanResult {
            is_root_directory: false,
            metadata_files,
            sub_directories: Vec::new(),
        });
    }

    // 检查当前目录名或父目录名是否匹配已定义的 retro system
    let dir_name = base_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    
    if find_mapping_by_folder(dir_name).is_some() {
        // 当前目录名匹配已定义的系统，这是单系统目录
        return Ok(DirectoryScanResult {
            is_root_directory: false,
            metadata_files: Vec::new(),
            sub_directories: Vec::new(),
        });
    }

    // 扫描子目录
    let mut sub_directories = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(base_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let sub_path = entry.path();
            if sub_path.is_dir() {
                let sub_metadata = detect_metadata_in_dir(&sub_path);
                let name = sub_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                sub_directories.push(SubDirectoryInfo {
                    name,
                    path: sub_path.to_string_lossy().to_string(),
                    metadata_files: sub_metadata,
                });
            }
        }
    }

    sub_directories.sort_by(|a, b| a.name.cmp(&b.name));

    // 判断是否为根目录：子目录名匹配已定义的系统，或子目录中有 metadata 文件
    let is_root = sub_directories.iter().any(|d| {
        !d.metadata_files.is_empty() || find_mapping_by_folder(&d.name).is_some()
    });

    Ok(DirectoryScanResult {
        is_root_directory: is_root,
        metadata_files: Vec::new(),
        sub_directories,
    })
}

