//! 目录检查工具命令
//! 
//! 用于检查目录下的 ROM 命名情况 (中英文对照)

use crate::rom_service::get_roms_from_directory;
use crate::scraper::cn_repo::{find_csv_in_dir, read_csv, CnRomEntry};
use crate::scraper::local_cn::smart_cn_similarity;
use crate::config::{get_temp_dir, get_data_dir};
use regex::Regex;
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
    let mut depth: i32 = 0;

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

/// 清理文件夹/文件名，去除版本号、汉化组等信息
fn clean_folder_name(name: &str) -> String {
    let mut result = name.to_string();
    
    // 去除括号内容：(xxx), [xxx]
    let bracket_re = Regex::new(r"\s*[\(\[][^\)\]]*[\)\]]").unwrap();
    result = bracket_re.replace_all(&result, "").to_string();
    
    // 去除常见汉化组标识
    let groups = [
        "汉化", "中文", "简体", "繁体", "简中", "繁中", "CN", "SC", "TC",
        "老游戏", "怀旧游戏", "翻译", "民间", "完美", "正式",
    ];
    for g in groups {
        result = result.replace(g, "");
    }
    
    // 去除版本号: v1.0, V2.1, ver1.0 等
    let version_re = Regex::new(r"(?i)\s*v(er)?\.?\s*\d+(\.\d+)*").unwrap();
    result = version_re.replace_all(&result, "").to_string();
    
    // 去除尾部的分隔符和空格
    result = result.trim_end_matches(|c: char| c == '_' || c == '-' || c == '.' || c.is_whitespace()).to_string();
    
    // 去除多余空格
    let multi_space_re = Regex::new(r"\s+").unwrap();
    result = multi_space_re.replace_all(&result, " ").to_string();
    
    result.trim().to_string()
}

/// ROM 扫描条目（包含文件夹信息）
#[derive(Debug, Clone)]
struct RomScanEntry {
    file: String,
    /// ROM 所在的子文件夹路径（相对于扫描根目录）
    subfolder: Option<String>,
    /// 清理后的文件夹名（如果ROM在单独文件夹中）
    cleaned_folder_name: Option<String>,
}

/// 扫描目录，检测子文件夹中的ROM
/// 如果ROM在子文件夹中，使用清理后的文件夹名作为游戏名
/// 如果子文件夹有多个文件，只返回最大的那个（主ROM）
fn scan_directory_with_folders(dir_path: &Path) -> Vec<RomScanEntry> {
    let mut entries = Vec::new();
    
    // ROM 文件扩展名
    let rom_extensions: std::collections::HashSet<&str> = [
        "iso", "cso", "chd", "bin", "cue", "img", "mdf", "nrg",
        "nes", "sfc", "smc", "gba", "gbc", "gb", "nds", "3ds",
        "n64", "z64", "v64", "gcm", "wbfs", "wad", "rvz",
        "psx", "pbp", "pkg",
        "zip", "7z", "rar",
    ].iter().cloned().collect();
    
    if let Ok(dir_entries) = fs::read_dir(dir_path) {
        for entry in dir_entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            
            if path.is_file() {
                // 根目录下的文件
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if rom_extensions.contains(ext.to_lowercase().as_str()) {
                        let filename = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();
                        entries.push(RomScanEntry {
                            file: filename,
                            subfolder: None,
                            cleaned_folder_name: None,
                        });
                    }
                }
            } else if path.is_dir() {
                // 子文件夹
                let folder_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                
                // 跳过常见的媒体资源目录
                let skip_dirs = [
                    "media", "images", "artwork", "videos", "screenshots",
                    "boxart", "snap", "wheel", "marquee", "named_boxarts", "named_snaps",
                ];
                if skip_dirs.iter().any(|&d| folder_name.eq_ignore_ascii_case(d)) {
                    continue;
                }
                
                // 扫描子文件夹中的ROM文件，记录文件大小
                let mut subfolder_roms: Vec<(String, u64)> = Vec::new();
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.filter_map(|e| e.ok()) {
                        let sub_path = sub_entry.path();
                        if sub_path.is_file() {
                            if let Some(ext) = sub_path.extension().and_then(|e| e.to_str()) {
                                if rom_extensions.contains(ext.to_lowercase().as_str()) {
                                    let filename = sub_path.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let file_size = sub_path.metadata()
                                        .map(|m| m.len())
                                        .unwrap_or(0);
                                    subfolder_roms.push((filename, file_size));
                                }
                            }
                        }
                    }
                }
                
                if subfolder_roms.is_empty() {
                    continue;
                }
                
                // 清理文件夹名
                let cleaned_name = clean_folder_name(&folder_name);
                
                // 如果只有一个ROM，直接使用
                // 如果有多个ROM，只取最大的那个（主ROM），其他是补丁
                if subfolder_roms.len() == 1 {
                    entries.push(RomScanEntry {
                        file: subfolder_roms[0].0.clone(),
                        subfolder: Some(folder_name),
                        cleaned_folder_name: Some(cleaned_name),
                    });
                } else {
                    // 找最大的文件作为主ROM
                    if let Some((largest_file, _)) = subfolder_roms.iter().max_by_key(|(_, size)| size) {
                        entries.push(RomScanEntry {
                            file: largest_file.clone(),
                            subfolder: Some(folder_name),
                            cleaned_folder_name: Some(cleaned_name),
                        });
                    }
                }
            }
        }
    }
    
    entries
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

    // 使用新的扫描函数，检测子文件夹ROM
    let scan_entries = scan_directory_with_folders(dir_path);

    // 读取临时元数据
    let temp_entries = load_temp_cn_metadata(&path).unwrap_or_default();

    // 转换为检查结果，合并临时数据
    let results = scan_entries.into_iter().map(|entry| {
        // 优先使用清理后的文件夹名，否则从文件名提取
        let extracted = if let Some(ref folder_name) = entry.cleaned_folder_name {
            Some(folder_name.clone())
        } else {
            parse_cn_name_from_filename(&entry.file)
        };

        // 查找临时数据中的匹配项
        let temp_data = temp_entries.iter().find(|e| e.file == entry.file);

        NamingCheckResult {
            file: entry.file.clone(),
            name: temp_data.and_then(|t| t.name.clone()).or_else(|| extracted.clone()).unwrap_or_else(|| entry.file.clone()),
            english_name: temp_data.and_then(|t| t.english_name.clone()),
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

    // 使用新的扫描函数
    let scan_entries = scan_directory_with_folders(dir_path);
    let total = scan_entries.len();

    // 优先使用传入的系统名，否则从目录名获取
    let system_name = system.unwrap_or_else(|| {
        dir_path.file_name().unwrap_or_default().to_string_lossy().to_string()
    });

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

    // 加载现有的临时元数据，保留用户手动编辑的数据
    let mut entries = load_temp_cn_metadata(&path).unwrap_or_default();

    for (idx, scan_entry) in scan_entries.into_iter().enumerate() {
        // 发送进度事件
        let _ = app.emit("naming-match-progress", MatchProgress {
            current: idx + 1,
            total,
        });

        // 检查是否已有用户手动编辑的数据（confidence = 100）
        let existing_entry = entries.iter().find(|e| e.file == scan_entry.file);
        if let Some(entry) = existing_entry {
            // 用户手动编辑的数据（满分），跳过自动匹配
            if entry.confidence == Some(100.0) && entry.english_name.is_some() {
                continue;
            }
        }

        // 优先使用清理后的文件夹名，否则从文件名提取
        let extracted_cn = if let Some(ref folder_name) = scan_entry.cleaned_folder_name {
            Some(folder_name.clone())
        } else {
            parse_cn_name_from_filename(&scan_entry.file)
        };
        let english_suffix = extract_english_suffix(&scan_entry.file);

        let query_name = extracted_cn.clone().unwrap_or_else(|| scan_entry.file.clone());

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
                let new_confidence = confidence * 100.0;

                // 更新或新增条目
                if let Some(entry) = entries.iter_mut().find(|e| e.file == scan_entry.file) {
                    entry.english_name = Some(cleaned_eng_name);
                    entry.confidence = Some(new_confidence);
                    // 保留现有的 name（如果用户已设置）
                    if entry.name.is_none() {
                        entry.name = extracted_cn.clone();
                    }
                } else {
                    entries.push(TempMetadataEntry {
                        file: scan_entry.file.clone(),
                        name: extracted_cn.clone(),
                        english_name: Some(cleaned_eng_name),
                        confidence: Some(new_confidence),
                    });
                }
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

/// 更新单个 ROM 的英文名 (写入临时 metadata)
#[tauri::command]
pub async fn update_english_name(
    directory: String,
    file: String,
    english_name: String,
) -> Result<(), String> {
    let mut entries = load_temp_cn_metadata(&directory).unwrap_or_default();

    // 查找或创建条目
    if let Some(entry) = entries.iter_mut().find(|e| e.file == file) {
        // 更新现有条目
        entry.english_name = if english_name.is_empty() {
            None
        } else {
            Some(english_name)
        };
        // 用户手动编辑的自动设置为满分
        entry.confidence = Some(100.0);
    } else {
        // 创建新条目
        entries.push(TempMetadataEntry {
            file,
            name: None,
            english_name: if english_name.is_empty() {
                None
            } else {
                Some(english_name)
            },
            confidence: Some(100.0), // 用户手动编辑的自动设置为满分
        });
    }

    save_temp_cn_metadata(&directory, &entries)?;
    Ok(())
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
