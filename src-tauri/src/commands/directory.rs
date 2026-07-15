use crate::rom_index::invalidate_library_index;
use crate::settings::{
    default_library_name, get_settings, library_id_for_path, native_path, update_setting,
    DirectoryConfig,
};
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
#[serde(rename_all = "camelCase")]
pub struct DirectoryInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub is_root_directory: bool,
    pub metadata_format: String,
    pub system_id: Option<String>,
    pub is_active: bool,
}

impl DirectoryInfo {
    fn from_config(c: DirectoryConfig, active_id: Option<&str>) -> Self {
        let is_active = active_id.is_some_and(|id| id == c.id);
        Self {
            id: c.id,
            name: c.name,
            path: c.path,
            is_root_directory: c.is_root_directory,
            metadata_format: c.metadata_format,
            system_id: c.system_id,
            is_active,
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
    let stored_path = native_path(&path);

    let updated = update_setting(|settings| {
        // 使用规范化路径去重
        let existing_id = settings
            .directories
            .iter()
            .find(|d| normalize_path(&d.path) == normalized)
            .map(|item| item.id.clone());
        let library_id = existing_id.unwrap_or_else(|| {
            let base_id = library_id_for_path(&stored_path);
            let mut id = base_id.clone();
            let mut suffix = 2;
            while settings.directories.iter().any(|item| item.id == id) {
                id = format!("{base_id}-{suffix}");
                suffix += 1;
            }
            let name = default_library_name(&stored_path, settings.directories.len());
            settings.directories.push(DirectoryConfig {
                id: id.clone(),
                name,
                path: stored_path.clone(),
                metadata_format: metadataFormat.clone(),
                is_root_directory: isRoot,
                system_id: systemId.clone(),
            });
            id
        });
        // 新增目录需要立即成为当前 Library，随后的首次扫描才会落到正确目录。
        settings.active_library_id = Some(library_id);
    })
    .map_err(|e| e.to_string())?;
    let active_id = updated.active_library_id.as_deref();
    updated
        .active_library()
        .cloned()
        .map(|item| DirectoryInfo::from_config(item, active_id))
        .ok_or_else(|| "添加 Library 后无法找到激活项".to_string())
}

/// 从配置移除目录
#[tauri::command]
#[allow(non_snake_case)]
pub fn remove_directory(libraryId: String) -> Result<(), String> {
    if !get_settings()
        .directories
        .iter()
        .any(|item| item.id == libraryId)
    {
        return Err("Library 不存在".to_string());
    }
    update_setting(|settings| {
        settings.directories.retain(|item| item.id != libraryId);
    })
    .map_err(|e| e.to_string())?;
    invalidate_library_index(&libraryId);

    Ok(())
}

/// 切换当前激活的 Library。
#[tauri::command]
#[allow(non_snake_case)]
pub fn set_active_library(libraryId: String) -> Result<DirectoryInfo, String> {
    if !get_settings()
        .directories
        .iter()
        .any(|item| item.id == libraryId)
    {
        return Err("Library 不存在".to_string());
    }
    let updated = update_setting(|settings| {
        settings.active_library_id = Some(libraryId.clone());
    })
    .map_err(|error| error.to_string())?;
    let active_id = updated.active_library_id.as_deref();
    updated
        .active_library()
        .cloned()
        .map(|item| DirectoryInfo::from_config(item, active_id))
        .ok_or_else(|| "激活 Library 失败".to_string())
}

/// 修改 Library 展示名称。
#[tauri::command]
#[allow(non_snake_case)]
pub fn rename_library(libraryId: String, name: String) -> Result<DirectoryInfo, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Library 名称不能为空".to_string());
    }
    if !get_settings()
        .directories
        .iter()
        .any(|item| item.id == libraryId)
    {
        return Err("Library 不存在".to_string());
    }
    let updated = update_setting(|settings| {
        if let Some(library) = settings
            .directories
            .iter_mut()
            .find(|item| item.id == libraryId)
        {
            library.name = name.clone();
        }
    })
    .map_err(|error| error.to_string())?;
    let active_id = updated.active_library_id.clone();
    updated
        .directories
        .into_iter()
        .find(|item| item.id == libraryId)
        .map(|item| DirectoryInfo::from_config(item, active_id.as_deref()))
        .ok_or_else(|| "重命名 Library 失败".to_string())
}

#[tauri::command]
pub fn get_directories() -> Vec<DirectoryInfo> {
    let settings = get_settings();
    let active_id = settings.active_library_id.clone();
    settings
        .directories
        .into_iter()
        .map(|item| DirectoryInfo::from_config(item, active_id.as_deref()))
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

fn detect_metadata_files_blocking(path: String) -> Result<Vec<MetadataFileInfo>, String> {
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
pub async fn detect_metadata_files(path: String) -> Result<Vec<MetadataFileInfo>, String> {
    tokio::task::spawn_blocking(move || detect_metadata_files_blocking(path))
        .await
        .map_err(|error| format!("Metadata detection task failed: {error}"))?
}

fn scan_directory_blocking(path: String) -> Result<DirectoryScanResult, String> {
    println!("[DEBUG] scan_directory 开始, path: {}", path);
    let base_path = Path::new(&path);

    if !base_path.exists() {
        return Err("Path does not exist".to_string());
    }

    if !base_path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    let metadata_files = detect_metadata_in_dir(base_path);
    println!("[DEBUG] 检测到 metadata 文件数量: {}", metadata_files.len());

    // 如果当前目录有 metadata 文件，这是单系统目录
    if !metadata_files.is_empty() {
        println!("[DEBUG] 返回: 单系统目录 (有 metadata)");
        return Ok(DirectoryScanResult {
            is_root_directory: false,
            metadata_files,
            sub_directories: Vec::new(),
        });
    }

    // 检查当前目录名或父目录名是否匹配已定义的 retro system
    let dir_name = base_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    println!(
        "[DEBUG] 目录名: '{}', find_mapping_by_folder 结果: {:?}",
        dir_name,
        find_mapping_by_folder(dir_name).is_some()
    );

    if find_mapping_by_folder(dir_name).is_some() {
        // 当前目录名匹配已定义的系统，这是单系统目录
        println!("[DEBUG] 返回: 单系统目录 (目录名匹配系统)");
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
    println!("[DEBUG] 子目录数量: {}", sub_directories.len());

    // 判断是否为根目录：子目录名匹配已定义的系统，或子目录中有 metadata 文件
    let is_root = sub_directories
        .iter()
        .any(|d| !d.metadata_files.is_empty() || find_mapping_by_folder(&d.name).is_some());

    println!("[DEBUG] 返回: is_root_directory={}", is_root);

    Ok(DirectoryScanResult {
        is_root_directory: is_root,
        metadata_files: Vec::new(),
        sub_directories,
    })
}

#[tauri::command]
pub async fn scan_directory(path: String) -> Result<DirectoryScanResult, String> {
    tokio::task::spawn_blocking(move || scan_directory_blocking(path))
        .await
        .map_err(|error| format!("Directory scan task failed: {error}"))?
}
