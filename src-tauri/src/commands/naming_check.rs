//! 目录检查工具命令
//! 
//! 用于检查目录下的 ROM 命名情况 (中英文对照)

use crate::rom_service::{get_roms_from_directory, RomInfo};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct NamingCheckResult {
    pub file: String,
    pub name: String,
    pub english_name: Option<String>,
}

#[tauri::command]
pub fn scan_directory_for_naming_check(path: String) -> Result<Vec<NamingCheckResult>, String> {
    let dir_path = Path::new(&path);
    if !dir_path.exists() {
        return Err("Directory does not exist".to_string());
    }

    // 检测元数据格式
    let format = if dir_path.join("metadata.pegasus.txt").exists() || dir_path.join("metadata.txt").exists() {
        "pegasus"
    } else if dir_path.join("gamelist.xml").exists() {
        "emulationstation"
    } else {
        "none"
    };

    // 读取 ROM 列表
    let roms = get_roms_from_directory(dir_path, format, "unknown")?;

    // 转换为检查结果
    let results = roms.into_iter().map(|r| NamingCheckResult {
        file: r.file,
        name: r.name,
        english_name: r.english_name,
    }).collect();

    Ok(results)
}
