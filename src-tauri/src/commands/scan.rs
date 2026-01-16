use crate::db::{get_connection, models::ScanDirectory, schema::scan_directories};
use crate::scanner;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanDirectoryInfo {
    pub id: String,
    pub path: String,
    pub system_id: Option<String>,
    pub recursive: bool,
    pub enabled: bool,
    pub last_scan: Option<String>,
}

impl From<ScanDirectory> for ScanDirectoryInfo {
    fn from(d: ScanDirectory) -> Self {
        Self {
            id: d.id,
            path: d.path,
            system_id: d.system_id,
            recursive: d.recursive,
            enabled: d.enabled,
            last_scan: d.last_scan,
        }
    }
}

/// 获取所有扫描目录
#[tauri::command]
pub fn get_scan_directories() -> Result<Vec<ScanDirectoryInfo>, String> {
    let mut conn = get_connection().map_err(|e| e.to_string())?;

    let results = scan_directories::table
        .load::<ScanDirectory>(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(results.into_iter().map(ScanDirectoryInfo::from).collect())
}

/// 添加扫描目录
#[tauri::command]
pub fn add_scan_directory(path: String, system_id: Option<String>) -> Result<ScanDirectoryInfo, String> {
    let mut conn = get_connection().map_err(|e| e.to_string())?;

    // 检查是否已存在
    let existing = scan_directories::table
        .filter(scan_directories::path.eq(&path))
        .first::<ScanDirectory>(&mut conn)
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some(dir) = existing {
        return Ok(ScanDirectoryInfo::from(dir));
    }

    let new_dir = ScanDirectory {
        id: Uuid::new_v4().to_string(),
        path,
        system_id,
        recursive: true,
        enabled: true,
        last_scan: None,
    };

    diesel::insert_into(scan_directories::table)
        .values(&new_dir)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(ScanDirectoryInfo::from(new_dir))
}

/// 删除扫描目录
#[tauri::command]
pub fn remove_scan_directory(id: String) -> Result<(), String> {
    let mut conn = get_connection().map_err(|e| e.to_string())?;

    diesel::delete(scan_directories::table.filter(scan_directories::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 触发扫描 (异步任务)
#[tauri::command]
pub async fn start_scan(app: AppHandle, dir_id: String) -> Result<(), String> {
    // 放在 tokio 线程中执行
    scanner::scan_directory(app, dir_id).await
}
