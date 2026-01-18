//! 目录检查工具命令
//! 
//! 用于检查目录下的 ROM 命名情况 (中英文对照)

use crate::rom_service::get_roms_from_directory;
use crate::scraper::cn_repo::{find_csv_in_dir, read_csv, CnRomEntry};
use crate::scraper::local_cn::smart_cn_similarity;
use crate::config::{get_temp_dir, get_data_dir};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use tauri::{AppHandle, Emitter, Manager};

/// 获取 rom-name-cn 数据目录列表（优先打包资源，其次用户数据目录）
fn get_cn_repo_paths(app: &AppHandle) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // 1. 优先检查打包的资源目录
    if let Ok(resource_path) = app.path().resolve("rom-name-cn", tauri::path::BaseDirectory::Resource) {
        if resource_path.exists() {
            eprintln!("[get_cn_repo_paths] Found bundled resource at: {:?}", resource_path);
            paths.push(resource_path);
        }
    }

    // 2. 用户数据目录作为后备（支持用户自行更新）
    let user_data_path = get_data_dir().join("rom-name-cn");
    if user_data_path.exists() {
        eprintln!("[get_cn_repo_paths] Found user data at: {:?}", user_data_path);
        paths.push(user_data_path);
    }

    paths
}

#[derive(Debug, Serialize)]
pub struct NamingCheckResult {
    pub file: String,
    pub name: String,
    pub english_name: Option<String>,
    pub extracted_cn_name: Option<String>,
    pub confidence: Option<f32>,
}

/// 从文件名提取英文后缀信息（如 "Original Generation 2" -> "OG2"）
fn extract_english_suffix(filename: &str) -> Option<String> {
    let stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);

    let clean_name = if let Some(idx) = stem.find(|c| c == '[' || c == '(') {
        &stem[..idx]
    } else {
        stem
    };

    // 查找中文后的英文部分
    let parts: Vec<&str> = clean_name.split(&['-', '–', '—', ':', '：'][..]).collect();
    if parts.len() < 2 {
        return None;
    }

    // 获取最后一部分（英文部分）
    let english_part = parts.last()?.trim();

    // 提取首字母缩写 + 数字
    let mut abbreviation = String::new();
    for word in english_part.split_whitespace() {
        if let Some(first_char) = word.chars().next() {
            if first_char.is_ascii_alphabetic() {
                abbreviation.push(first_char.to_ascii_uppercase());
            } else if first_char.is_ascii_digit() {
                abbreviation.push_str(word);
            }
        }
    }

    if abbreviation.is_empty() {
        None
    } else {
        Some(abbreviation)
    }
}

/// 清理英文名，去除括号中的区域信息
/// 例如: "Super Mario Bros. (USA)" -> "Super Mario Bros."
fn clean_english_name(name: &str) -> String {
    // 去除 () 和 [] 中的内容
    let mut result = String::new();
    let mut depth = 0;

    for ch in name.chars() {
        match ch {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            _ => {
                if depth == 0 {
                    result.push(ch);
                }
            }
        }
    }

    // 去除首尾空格
    result.trim().to_string()
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

    // 3. 处理全角字符，去除空格
    let normalized = clean_name
        .replace('－', "-")
        .replace('　', "")   // 全角空格移除
        .replace(' ', "")    // 半角空格移除
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

#[derive(Debug, Clone, Serialize)]
pub struct MatchProgress {
    pub current: usize,
    pub total: usize,
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

    // 读取临时元数据
    let temp_entries = load_temp_cn_metadata(&path).unwrap_or_default();

    // 转换为检查结果，合并临时数据
    let results = roms.into_iter().map(|r| {
        let extracted = parse_cn_name_from_filename(&r.file);

        // 查找临时数据中的匹配项
        let temp_data = temp_entries.iter().find(|e| e.file == r.file);

        NamingCheckResult {
            file: r.file.clone(),
            name: temp_data.and_then(|t| t.name.clone()).unwrap_or(r.name),
            english_name: temp_data.and_then(|t| t.english_name.clone()).or(r.english_name),
            extracted_cn_name: extracted,
            confidence: temp_data.and_then(|t| t.confidence),
        }
    }).collect();

    Ok(results)
}

/// 在内存中进行快速匹配
fn fast_match(
    query_cn: &str,
    english_suffix: Option<&str>,
    csv_entries: &[CnRomEntry],
) -> Option<(String, String, f32)> {
    let query_lower = query_cn.to_lowercase();
    let mut best_match: Option<(String, String, f32)> = None;

    for entry in csv_entries {
        let chinese_lower = entry.chinese_name.to_lowercase();

        // 1. 精确匹配
        if chinese_lower == query_lower {
            return Some((entry.english_name.clone(), entry.chinese_name.clone(), 1.0));
        }

        // 2. 如果有英文后缀（如 OG2），优先匹配包含该后缀的中文名
        if let Some(suffix) = english_suffix {
            if entry.chinese_name.contains(suffix) {
                let score = smart_cn_similarity(&query_lower, &chinese_lower);
                if score > 0.5 {
                    // 后缀匹配优先
                    return Some((entry.english_name.clone(), entry.chinese_name.clone(), 0.98));
                }
            }
        }

        // 3. 智能相似度匹配
        let score = smart_cn_similarity(&query_lower, &chinese_lower);
        if score > 0.75 {
            if let Some((_, _, best_score)) = &best_match {
                if score > *best_score {
                    best_match = Some((entry.english_name.clone(), entry.chinese_name.clone(), score));
                }
            } else {
                best_match = Some((entry.english_name.clone(), entry.chinese_name.clone(), score));
            }
        }
    }

    best_match
}

#[tauri::command]
pub async fn auto_fix_naming(
    app: AppHandle,
    path: String,
    system: Option<String>,
) -> Result<AutoFixResult, String> {
    let dir_path = Path::new(&path);
    if !dir_path.exists() {
        return Err("Directory does not exist".to_string());
    }

    let format = detect_format(dir_path);
    // 优先使用传入的系统名，否则从目录名获取
    let system_name = system.unwrap_or_else(|| {
        dir_path.file_name().unwrap_or_default().to_string_lossy().to_string()
    });
    let roms = get_roms_from_directory(dir_path, &format, &system_name)?;
    let total = roms.len();

    // 一次性加载 CSV 到内存（优先使用打包资源）
    let repo_paths = get_cn_repo_paths(&app);
    let csv_entries = {
        let mut entries = Vec::new();
        for repo_path in &repo_paths {
            if let Some(csv_path) = find_csv_in_dir(repo_path, &system_name) {
                eprintln!("[auto_fix_naming] Found CSV at: {:?}", csv_path);
                if let Ok(loaded) = read_csv(&csv_path) {
                    entries = loaded;
                    break;
                }
            }
        }
        if entries.is_empty() {
            eprintln!("[auto_fix_naming] No CSV found for system: {} in paths: {:?}", system_name, repo_paths);
            return Err(format!("No CSV database found for system: {}", system_name));
        }
        entries
    };

    eprintln!("[auto_fix_naming] Loaded {} entries from CSV", csv_entries.len());

    let mut success_count = 0;
    let mut failed_count = 0;

    // 收集需要写入的数据
    let mut entries: Vec<TempMetadataEntry> = Vec::new();

    for (idx, rom) in roms.into_iter().enumerate() {
        // 发送进度事件
        let _ = app.emit("naming-match-progress", MatchProgress {
            current: idx + 1,
            total,
        });

        // 如果已经有英文名，跳过
        if rom.english_name.is_some() && rom.name != rom.file {
             continue;
        }

        let extracted_cn = parse_cn_name_from_filename(&rom.file);
        let english_suffix = extract_english_suffix(&rom.file);

        let query_name = extracted_cn.clone().unwrap_or_else(|| rom.name.clone());

        // 使用内存中的快速匹配
        if let Some((eng_name, _cn_name, confidence)) = fast_match(
            &query_name,
            english_suffix.as_deref(),
            &csv_entries,
        ) {
            // 只有一个匹配且置信度 > 0.75，或高置信度 > 0.95
            if confidence > 0.95 || confidence > 0.75 {
                // 清理英文名，去除括号中的区域信息
                let cleaned_eng_name = clean_english_name(&eng_name);

                entries.push(TempMetadataEntry {
                    file: rom.file.clone(),
                    name: extracted_cn.clone(),
                    english_name: Some(cleaned_eng_name),
                    confidence: Some(confidence * 100.0), // 转换为百分比
                });
                success_count += 1;
            } else {
                failed_count += 1;
            }
        } else {
            failed_count += 1;
        }
    }

    // 保存到 temp 目录
    save_temp_cn_metadata(&path, &entries)?;

    eprintln!("[auto_fix_naming] Done: {} success, {} failed", success_count, failed_count);

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
    confidence: Option<f32>,
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
                // 保留原有的 english_name 和 confidence
            } else {
                entries.push(TempMetadataEntry {
                    file: rom.file,
                    name: Some(cn_name),
                    english_name: None,
                    confidence: None,
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
