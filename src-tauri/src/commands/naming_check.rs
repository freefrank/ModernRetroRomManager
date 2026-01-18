//! 目录检查工具命令
//! 
//! 用于检查目录下的 ROM 命名情况 (中英文对照)

use crate::rom_service::{get_roms_from_directory, RomInfo};
use crate::scraper::local_cn::LocalCnProvider;
use crate::scraper::{ScrapeQuery, ScraperProvider};
use crate::scraper::persistence::{save_metadata_pegasus, save_metadata_emulationstation};
use crate::commands::scraper::ScraperState;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct NamingCheckResult {
    pub file: String,
    pub name: String,
    pub english_name: Option<String>,
    pub extracted_cn_name: Option<String>,
}

fn parse_cn_name_from_filename(filename: &str) -> Option<String> {
    // 1. 去除扩展名
    let stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);

    // 2. 清理括号及内容 []()
    // 策略：找到第一个 [ 或 (，截断
    let clean_name = if let Some(idx) = stem.find(|c| c == '[' || c == '(') {
        &stem[..idx]
    } else {
        stem
    };

    // 3. 处理全角字符，去除所有空格
    let normalized = clean_name
        .replace('－', "-")
        .replace('　', "")  // 全角空格直接移除
        .replace(' ', "")   // 半角空格直接移除
        .trim()
        .to_string();

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AutoFixResult {
    pub success: usize,
    pub failed: usize,
}

#[tauri::command]
pub fn scan_directory_for_naming_check(path: String) -> Result<Vec<NamingCheckResult>, String> {
    let dir_path = Path::new(&path);
    if !dir_path.exists() {
        return Err("Directory does not exist".to_string());
    }

    // 检测元数据格式
    let format = detect_format(dir_path);

    // 读取 ROM 列表
    let roms = get_roms_from_directory(dir_path, &format, "unknown")?;

    // 转换为检查结果
    let results = roms.into_iter().map(|r| {
        let extracted = parse_cn_name_from_filename(&r.file);
        NamingCheckResult {
            file: r.file,
            name: r.name,
            english_name: r.english_name,
            extracted_cn_name: extracted,
        }
    }).collect();

    Ok(results)
}

#[tauri::command]
pub async fn auto_fix_naming(
    state: State<'_, ScraperState>,
    path: String,
) -> Result<AutoFixResult, String> {
    let dir_path = Path::new(&path);
    if !dir_path.exists() {
        return Err("Directory does not exist".to_string());
    }

    let format = detect_format(dir_path);
    let system_name = dir_path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let roms = get_roms_from_directory(dir_path, &format, &system_name)?;

    let manager = state.manager.read().await;
    // 获取 LocalCnProvider，通常它是注册的
    // 为了简化，我们假设 LocalCnProvider 始终可用，或者我们直接在这里实例化一个新的?
    // 由于 manager 抽象了 provider，我们需要通过 ID 获取
    // 或者更简单的，我们直接实例化 LocalCnProvider，因为它只依赖本地文件
    // 但为了使用 setup 中注入的 Resource 路径，最好从 manager 获取
    // 不过 manager 目前没有直接暴露特定 provider 的接口...
    // 作为一个权宜之计，我们新建一个 LocalCnProvider，只用 data 目录下的库
    // 更好的做法是在 manager 中增加 get_provider 方法，或者让 auto_fix_naming 接收 provider_id
    
    // 这里我们直接创建一个新的 LocalCnProvider，只扫描用户目录
    // 资源目录的路径在 command 中难以获取，除非通过 AppHandle 传递
    // 考虑到 "更新数据库" 功能会将最新数据下载到 data 目录，这应该是够用的
    let provider = LocalCnProvider::new(vec![]); 

    let mut success_count = 0;
    let mut failed_count = 0;

    for rom in roms {
        // 如果已经有英文名，跳过 (或者策略是覆盖？)
        // 假设这里只修复缺失的
        if rom.english_name.is_some() && rom.name != rom.file {
             continue;
        }

        // 尝试从文件名提取中文名，如果有，使用它进行搜索
        let extracted_cn = parse_cn_name_from_filename(&rom.file);
        
        let query = ScrapeQuery {
            name: extracted_cn.clone().unwrap_or_else(|| rom.name.clone()),
            file_name: rom.file.clone(),
            system: Some(system_name.clone()),
            ..Default::default()
        };

        match provider.search(&query).await {
            Ok(results) => {
                // 查找高置信度结果
                if let Some(best_match) = results.iter().find(|r| r.confidence > 0.95) {
                    // 获取元数据
                    // name 优先使用提取的中文名 (如果存在)，否则使用匹配到的中文名
                    // english_name 使用匹配到的英文名 (source_id)
                    let metadata = crate::scraper::GameMetadata {
                        name: extracted_cn.unwrap_or(best_match.name.clone()),
                        english_name: Some(best_match.source_id.clone()),
                        description: None,
                        developer: None,
                        publisher: None,
                        genres: vec![],
                        release_date: None,
                        players: None,
                        rating: None,
                    };

                    // 持久化 (直接写入原始文件，非 temp)
                    let save_result = match format.as_str() {
                        "pegasus" => save_metadata_pegasus(&rom, &metadata, false),
                        "emulationstation" => save_metadata_emulationstation(&rom, &metadata, false),
                        _ => Err("Unsupported format for auto-fix".to_string()),
                    };

                    if save_result.is_ok() {
                        success_count += 1;
                    } else {
                        failed_count += 1;
                    }
                } else {
                    failed_count += 1;
                }
            }
            Err(_) => failed_count += 1,
        }
    }

    Ok(AutoFixResult {
        success: success_count,
        failed: failed_count,
    })
}

fn detect_format(dir_path: &Path) -> String {
    if dir_path.join("metadata.pegasus.txt").exists() || dir_path.join("metadata.txt").exists() {
        "pegasus".to_string()
    } else if dir_path.join("gamelist.xml").exists() {
        "emulationstation".to_string()
    } else {
        "none".to_string()
    }
}
