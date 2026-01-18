//! 中文 ROM 命名数据库管理
//!
//! 负责管理 yingw/rom-name-cn 仓库的本地副本，并提供 CSV 读取功能

use std::path::PathBuf;
use std::process::Command;
use std::fs;
use crate::config::get_data_dir;
use crate::system_mapping::find_csv_name_by_folder;

const REPO_URL: &str = "https://github.com/yingw/rom-name-cn.git";
const REPO_DIR_NAME: &str = "rom-name-cn";

/// 获取仓库本地路径
pub fn get_cn_repo_path() -> PathBuf {
    get_data_dir().join(REPO_DIR_NAME)
}

/// 检查仓库是否存在
pub fn is_repo_ready() -> bool {
    let path = get_cn_repo_path();
    path.exists() && path.join(".git").exists()
}

/// 初始化或更新仓库
pub fn update_repo() -> Result<(), String> {
    let path = get_cn_repo_path();
    let data_dir = get_data_dir();

    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    }

    if is_repo_ready() {
        // Git pull
        let output = Command::new("git")
            .current_dir(&path)
            .args(&["pull"])
            .output()
            .map_err(|e| format!("Git pull failed: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
    } else {
        // Git clone
        // 如果目录存在但不是 git 仓库（例如空目录），先清理
        if path.exists() {
            let _ = fs::remove_dir_all(&path);
        }

        let output = Command::new("git")
            .current_dir(&data_dir)
            .args(&["clone", REPO_URL])
            .output()
            .map_err(|e| format!("Git clone failed: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
    }

    Ok(())
}

/// CSV 记录
#[derive(Debug, serde::Deserialize)]
pub struct CnRomEntry {
    #[serde(rename = "English Name")]
    pub english_name: String,
    #[serde(rename = "Chinese Name")]
    pub chinese_name: String,
}

/// 在指定目录查找系统对应的 CSV 文件
pub fn find_csv_in_dir(root_path: &PathBuf, system: &str) -> Option<PathBuf> {
    if !root_path.exists() {
        return None;
    }

    // 使用 system_mapping 查找 CSV 文件名
    let csv_name = find_csv_name_by_folder(system)?;

    // 直接查找精确的 CSV 文件名
    let csv_path = root_path.join(csv_name);
    if csv_path.exists() {
        return Some(csv_path);
    }

    None
}

/// 查找系统对应的 CSV 文件 (旧接口，仅搜索用户目录)
pub fn find_csv_for_system(system: &str) -> Option<PathBuf> {
    find_csv_in_dir(&get_cn_repo_path(), system)
}

/// 读取 CSV 内容
pub fn read_csv(path: &PathBuf) -> Result<Vec<CnRomEntry>, String> {
    let mut entries = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true) // CSV 文件有 header: Name EN,Name CN
        .from_path(path)
        .map_err(|e| e.to_string())?;

    for result in rdr.records() {
        if let Ok(record) = result {
            if record.len() >= 2 {
                let english_name = record[0].trim().to_string();
                let chinese_name = record[1].trim().to_string();

                // 跳过空行或无效数据
                if !english_name.is_empty() && !chinese_name.is_empty() {
                    entries.push(CnRomEntry {
                        english_name,
                        chinese_name,
                    });
                }
            }
        }
    }
    Ok(entries)
}
