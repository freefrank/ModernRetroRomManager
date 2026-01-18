//! 目录检查工具命令
//! 
//! 用于检查目录下的 ROM 命名情况 (中英文对照)

use crate::rom_service::get_roms_from_directory;
use crate::scraper::local_cn::LocalCnProvider;
use crate::scraper::{ScrapeQuery, ScraperProvider};
use crate::config::get_temp_dir;
use crate::commands::scraper::ScraperState;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use tauri::State;

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

    let _manager = state.manager.read().await;
    let provider = LocalCnProvider::new(vec![]); 

    let mut success_count = 0;
    let mut failed_count = 0;
    
    // 收集需要写入的数据
    let mut entries: Vec<TempMetadataEntry> = Vec::new();

    for rom in roms {
        // 如果已经有英文名，跳过
        if rom.english_name.is_some() && rom.name != rom.file {
             continue;
        }

        let extracted_cn = parse_cn_name_from_filename(&rom.file);
        
        let query = ScrapeQuery {
            name: extracted_cn.clone().unwrap_or_else(|| rom.name.clone()),
            file_name: rom.file.clone(),
            system: Some(system_name.clone()),
            ..Default::default()
        };

        match provider.search(&query).await {
            Ok(results) => {
                if let Some(best_match) = results.iter().find(|r| r.confidence > 0.95) {
                    entries.push(TempMetadataEntry {
                        file: rom.file.clone(),
                        name: extracted_cn.clone(),
                        english_name: Some(best_match.source_id.clone()),
                    });
                    success_count += 1;
                } else {
                    failed_count += 1;
                }
            }
            Err(_) => failed_count += 1,
        }
    }

    // 保存到 temp 目录
    save_temp_cn_metadata(&path, &entries)?;

    Ok(AutoFixResult {
        success: success_count,
        failed: failed_count,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TempMetadataEntry {
    file: String,
    name: Option<String>,
    english_name: Option<String>,
}

/// 保存临时元数据到 temp 目录
fn save_temp_cn_metadata(source_dir: &str, entries: &[TempMetadataEntry]) -> Result<(), String> {
    let temp_dir = get_temp_dir();
    let dir_name = Path::new(source_dir)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    let target_dir = temp_dir.join("cn_metadata").join(&dir_name);
    fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
    
    let target_file = target_dir.join("metadata.json");
    let json = serde_json::to_string_pretty(entries).map_err(|e| e.to_string())?;
    fs::write(&target_file, json).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 读取临时元数据
fn load_temp_cn_metadata(source_dir: &str) -> Result<Vec<TempMetadataEntry>, String> {
    let temp_dir = get_temp_dir();
    let dir_name = Path::new(source_dir)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    let target_file = temp_dir.join("cn_metadata").join(&dir_name).join("metadata.json");
    
    if !target_file.exists() {
        return Ok(Vec::new());
    }
    
    let content = fs::read_to_string(&target_file).map_err(|e| e.to_string())?;
    let entries: Vec<TempMetadataEntry> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(entries)
}

/// 将提取的中文名设置为 ROM 名 (写入临时 metadata)
#[tauri::command]
pub async fn set_extracted_cn_as_name(directory: String) -> Result<AutoFixResult, String> {
    let dir_path = Path::new(&directory);
    if !dir_path.exists() {
        return Err("Directory does not exist".to_string());
    }

    let format = detect_format(dir_path);
    let system_name = dir_path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let roms = get_roms_from_directory(dir_path, &format, &system_name)?;

    let mut entries = load_temp_cn_metadata(&directory).unwrap_or_default();
    let mut success_count = 0;

    for rom in roms {
        let extracted_cn = parse_cn_name_from_filename(&rom.file);
        if let Some(cn_name) = extracted_cn {
            // 更新或新增
            if let Some(entry) = entries.iter_mut().find(|e| e.file == rom.file) {
                entry.name = Some(cn_name);
            } else {
                entries.push(TempMetadataEntry {
                    file: rom.file,
                    name: Some(cn_name),
                    english_name: None,
                });
            }
            success_count += 1;
        }
    }

    save_temp_cn_metadata(&directory, &entries)?;

    Ok(AutoFixResult {
        success: success_count,
        failed: 0,
    })
}

/// 将英文名添加为额外 tag (写入临时 metadata)
#[tauri::command]
pub async fn add_english_as_tag(directory: String) -> Result<AutoFixResult, String> {
    let entries = load_temp_cn_metadata(&directory)?;
    
    // 这里 tag 信息已经在 english_name 字段中，导出时会转换为 x-mrrm-eng / <eng>
    // 所以这个命令实际上只是确认操作，不需要额外处理
    // 但如果需要明确标记，可以添加一个 has_eng_tag 字段
    
    let count = entries.iter().filter(|e| e.english_name.is_some()).count();
    
    Ok(AutoFixResult {
        success: count,
        failed: 0,
    })
}

/// 导出临时 metadata 到指定位置
#[tauri::command]
pub async fn export_cn_metadata(
    directory: String,
    target_path: String,
    format: String,
) -> Result<(), String> {
    let entries = load_temp_cn_metadata(&directory)?;
    
    if entries.is_empty() {
        return Err("No metadata to export. Please run 'Match English Names' first.".to_string());
    }

    match format.as_str() {
        "pegasus" => export_pegasus_format(&target_path, &entries),
        "gamelist" => export_gamelist_format(&target_path, &entries),
        _ => Err(format!("Unsupported format: {}", format)),
    }
}

fn export_pegasus_format(target_path: &str, entries: &[TempMetadataEntry]) -> Result<(), String> {
    let mut content = String::new();
    content.push_str("# Generated by ModernRetroRomManager - CN ROM Tool\n\n");
    
    for entry in entries {
        content.push_str(&format!("game: {}\n", entry.name.as_ref().unwrap_or(&entry.file)));
        content.push_str(&format!("file: {}\n", entry.file));
        
        if let Some(eng_name) = &entry.english_name {
            content.push_str(&format!("x-mrrm-eng: {}\n", eng_name));
        }
        
        content.push('\n');
    }
    
    fs::write(target_path, content).map_err(|e| e.to_string())
}

fn export_gamelist_format(target_path: &str, entries: &[TempMetadataEntry]) -> Result<(), String> {
    let mut content = String::new();
    content.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    content.push_str("<gameList>\n");
    content.push_str("  <!-- Generated by ModernRetroRomManager - CN ROM Tool -->\n");
    
    for entry in entries {
        let name = entry.name.as_ref().unwrap_or(&entry.file);
        let escaped_name = escape_xml(name);
        let escaped_file = escape_xml(&entry.file);
        
        content.push_str("  <game>\n");
        content.push_str(&format!("    <path>./{}</path>\n", escaped_file));
        content.push_str(&format!("    <name>{}</name>\n", escaped_name));
        
        if let Some(eng_name) = &entry.english_name {
            content.push_str(&format!("    <eng>{}</eng>\n", escape_xml(eng_name)));
        }
        
        content.push_str("  </game>\n");
    }
    
    content.push_str("</gameList>\n");
    
    fs::write(target_path, content).map_err(|e| e.to_string())
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
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
