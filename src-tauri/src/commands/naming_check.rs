//! 目录检查工具命令
//! 
//! 用于检查目录下的 ROM 命名情况 (中英文对照)

use crate::rom_service::{detect_metadata_format, get_roms_from_directory, RomInfo};
use crate::scraper::cn_repo::{find_csv_in_dir, read_csv, CnRomEntry};
use crate::scraper::jy6d_dz::{load_jy6d_csv, Jy6dDzEntry};
use crate::scraper::pegasus::parse_pegasus_file;
use crate::scraper::local_cn::smart_cn_similarity;
use crate::config::{get_data_dir, get_temp_dir, get_temp_dir_for_library};
use crate::system_mapping::find_mapping_by_folder;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter, Manager};

static BRACKET_RE: OnceLock<Regex> = OnceLock::new();
static VERSION_RE: OnceLock<Regex> = OnceLock::new();
static MULTI_SPACE_RE: OnceLock<Regex> = OnceLock::new();

fn find_preferred_pegasus_metadata_path(dir_path: &Path, system_name: &str) -> Option<PathBuf> {
    // App 逻辑可能会将系统目录下的 metadata 复制到 temp/{library}/{system}/metadata.pegasus.txt
    // 这里用于兜底读取（例如源目录没有 metadata，但 temp 里还保留着）

    let mut candidates: Vec<PathBuf> = Vec::new();

    // 1) 猜测 library_path = dir_path.parent() (常见: library=.../samba, system=.../psp)
    if let Some(parent) = dir_path.parent() {
        candidates.push(get_temp_dir_for_library(parent, system_name).join("metadata.pegasus.txt"));
    }

    // 2) 兼容：library_path = dir_path（如果用户把 system 目录本身当作 library）
    candidates.push(get_temp_dir_for_library(dir_path, system_name).join("metadata.pegasus.txt"));

    // 3) 兼容旧结构: temp/{system}/metadata.*
    candidates.push(get_temp_dir().join(system_name).join("metadata.pegasus.txt"));
    candidates.push(get_temp_dir().join(system_name).join("metadata.txt"));

    candidates.into_iter().find(|p| p.exists())
}

fn sync_source_metadata_to_temp(dir_path: &Path, system_name: &str) -> Result<Option<PathBuf>, String> {
    // 用户期望：如果系统目录里存在 metadata，则直接复制到 temp（覆盖更新）。
    // 这样 CN ROM Tool 显示与后续处理都基于 temp 的 metadata。

    let source_pegasus = dir_path.join("metadata.pegasus.txt");
    let source_txt = dir_path.join("metadata.txt");
    let source = if source_pegasus.exists() {
        source_pegasus
    } else if source_txt.exists() {
        source_txt
    } else {
        return Ok(None);
    };

    // 不覆盖已存在的 temp metadata（避免擦掉用户在 temp 中的编辑）
    let target = temp_pegasus_metadata_path(dir_path, system_name);
    if target.exists() {
        return Ok(Some(target));
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    // 无论源文件名是什么，都复制到 temp 的 metadata.pegasus.txt，保持一致
    fs::copy(&source, &target).map_err(|e| format!("Failed to copy metadata to temp: {}", e))?;

    Ok(Some(target))
}

fn temp_pegasus_metadata_path(dir_path: &Path, system_name: &str) -> PathBuf {
    // 约定：temp metadata 按 library/system 分层；library 通常是 system 目录的父级
    let library_path = dir_path.parent().unwrap_or(dir_path);
    get_temp_dir_for_library(library_path, system_name).join("metadata.pegasus.txt")
}

fn ensure_temp_pegasus_metadata_exists(dir_path: &Path, system_name: &str) -> Result<PathBuf, String> {
    let metadata_path = temp_pegasus_metadata_path(dir_path, system_name);

    if metadata_path.exists() {
        return Ok(metadata_path);
    }

    // 如果源目录有 metadata，复制一份到 temp
    if let Some(p) = sync_source_metadata_to_temp(dir_path, system_name)? {
        return Ok(p);
    }

    // 否则创建一个最小可用的 metadata（写入到 temp）
    if let Some(parent) = metadata_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let content = format!("collection: {}\nlaunch: {{file.path}}\n\n", system_name);
    fs::write(&metadata_path, content).map_err(|e| e.to_string())?;
    Ok(metadata_path)
}

fn write_updated_pegasus_games(
    metadata_path: &Path,
    system_name: &str,
    metadata: crate::scraper::pegasus::PegasusMetadata,
) -> Result<(), String> {
    use crate::scraper::pegasus::{PegasusExportOptions, write_pegasus_file};

    let (collection_name, extensions_vec, launch_command, workdir) = metadata
        .collections
        .first()
        .map(|c| {
            (
                Some(c.name.clone()),
                c.extensions.clone(),
                c.launch_command.clone(),
                c.workdir.clone(),
            )
        })
        .unwrap_or((Some(system_name.to_string()), Vec::new(), None, None));

    let extensions = if extensions_vec.is_empty() {
        None
    } else {
        Some(extensions_vec)
    };

    let options = PegasusExportOptions {
        include_collection: true,
        collection_name,
        extensions,
        launch_command,
        workdir,
        include_assets: true,
    };

    write_pegasus_file(metadata_path, &metadata.games, &options, false)
}

fn upsert_temp_pegasus_game_english_name(
    dir_path: &Path,
    system_name: &str,
    rom_file: &str,
    english_name: Option<&str>,
) -> Result<(), String> {
    use crate::scraper::pegasus::PegasusGame;

    let metadata_path = ensure_temp_pegasus_metadata_exists(dir_path, system_name)?;
    let mut metadata = parse_pegasus_file(&metadata_path).unwrap_or_default();

    let game = metadata
        .games
        .iter_mut()
        .find(|g| g.file.as_deref() == Some(rom_file));

    if let Some(game) = game {
        match english_name {
            Some(v) if !v.is_empty() => {
                game.extra.insert("x-mrrm-eng".to_string(), v.to_string());
            }
            _ => {
                game.extra.remove("x-mrrm-eng");
                game.extra.remove("x-english-name");
            }
        }
    } else {
        let mut new_game = PegasusGame {
            name: rom_file.to_string(),
            file: Some(rom_file.to_string()),
            ..Default::default()
        };
        if let Some(v) = english_name {
            if !v.is_empty() {
                new_game.extra.insert("x-mrrm-eng".to_string(), v.to_string());
            }
        }
        metadata.games.push(new_game);
    }

    write_updated_pegasus_games(&metadata_path, system_name, metadata)
}

fn upsert_temp_pegasus_game_name(
    dir_path: &Path,
    system_name: &str,
    rom_file: &str,
    new_name: &str,
) -> Result<(), String> {
    use crate::scraper::pegasus::PegasusGame;

    let metadata_path = ensure_temp_pegasus_metadata_exists(dir_path, system_name)?;
    let mut metadata = parse_pegasus_file(&metadata_path).unwrap_or_default();

    let game = metadata
        .games
        .iter_mut()
        .find(|g| g.file.as_deref() == Some(rom_file));

    if let Some(game) = game {
        if !new_name.is_empty() {
            game.name = new_name.to_string();
        }
    } else {
        let new_game = PegasusGame {
            name: new_name.to_string(),
            file: Some(rom_file.to_string()),
            ..Default::default()
        };
        metadata.games.push(new_game);
    }

    write_updated_pegasus_games(&metadata_path, system_name, metadata)
}

fn read_pegasus_roms_light(dir_path: &Path, system_name: &str) -> Result<Vec<RomInfo>, String> {
    // source 目录存在 metadata 时：确保 temp metadata 存在（只在不存在时复制），然后读取 temp
    // 这样显示/编辑都基于 temp，并且不会覆盖 temp 中的用户修改。
    let metadata_path = ensure_temp_pegasus_metadata_exists(dir_path, system_name)?;

    let metadata = parse_pegasus_file(&metadata_path)?;
    Ok(metadata
        .games
        .into_iter()
        .filter_map(|g| {
            let mut rom: RomInfo = g.into();
            rom.directory = dir_path.to_string_lossy().to_string();
            rom.system = system_name.to_string();

            // 轻量模式：只验证 ROM 文件存在，不做媒体扫描
            if !rom.file.is_empty() {
                let rom_path = dir_path.join(&rom.file);
                if !rom_path.exists() {
                    return None;
                }
            }

            Some(rom)
        })
        .collect())
}

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

/// 获取 jy6d-dz CSV 文件路径（打包资源目录）
fn get_jy6d_csv_path(app: &AppHandle, csv_name: &str) -> Option<PathBuf> {
    if let Ok(resource_path) = app.path().resolve("cn-mapping", tauri::path::BaseDirectory::Resource) {
        let csv_path = resource_path.join(csv_name);
        if csv_path.exists() {
            eprintln!("[get_jy6d_csv_path] Found jy6d CSV at: {:?}", csv_path);
            return Some(csv_path);
        }
    }
    None
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

/// 统一的游戏名提取函数
/// - 如果 ROM 在子文件夹中，从文件夹名提取
/// - 如果 ROM 直接在平台文件夹中，从文件名提取
/// 
/// # Arguments
/// * `name` - 文件名或文件夹名
/// * `is_filename` - true 表示输入是文件名（需要先去除扩展名）
fn extract_game_name(name: &str, is_filename: bool) -> Option<String> {
    let mut result = if is_filename {
        // 去除扩展名
        std::path::Path::new(name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(name)
            .to_string()
    } else {
        name.to_string()
    };
    
    // 去除括号内容：(xxx), [xxx]
    let bracket_re = BRACKET_RE.get_or_init(|| Regex::new(r"\s*[\(\[][^\)\]]*[\)\]]").unwrap());
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
    let version_re = VERSION_RE.get_or_init(|| Regex::new(r"(?i)\s*v(er)?\.?\s*\d+(\.\d+)*").unwrap());
    result = version_re.replace_all(&result, "").to_string();
    
    // 处理全角字符
    result = result
        .replace('－', "-")
        .replace('　', " ");  // 全角空格转半角
    
    // 去除尾部的分隔符和空格
    result = result.trim_end_matches(|c: char| c == '_' || c == '-' || c == '.' || c.is_whitespace()).to_string();
    
    // 去除多余空格
    let multi_space_re = MULTI_SPACE_RE.get_or_init(|| Regex::new(r"\s+").unwrap());
    result = multi_space_re.replace_all(&result, " ").to_string();
    
    let result = result.trim().to_string();
    
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn parse_cn_name_from_filename(filename: &str) -> Option<String> {
    extract_game_name(filename, true)
}

/// 清理文件夹/文件名，去除版本号、汉化组等信息
fn clean_folder_name(name: &str) -> String {
    extract_game_name(name, false).unwrap_or_else(|| name.to_string())
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

/// 统一的目录扫描函数
fn scan_directory_internal<F>(dir_path: &Path, callback: Option<F>) -> Vec<RomScanEntry>
where
    F: Fn(usize, usize, &str),
{
    eprintln!("[DEBUG] scan_directory_internal: {:?}", dir_path);
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
        let dir_entries: Vec<_> = dir_entries.filter_map(|e| e.ok()).collect();
        let total = dir_entries.len();
        eprintln!("[DEBUG] Found {} entries in directory", total);

        for (idx, entry) in dir_entries.into_iter().enumerate() {
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            let folder_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // 如果有回调，发送进度
            if let Some(ref cb) = callback {
                cb(idx + 1, total, &folder_name);
            }

            if file_type.is_file() {
                // 根目录下的文件
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if rom_extensions.contains(ext.to_lowercase().as_str()) {
                        entries.push(RomScanEntry {
                            file: folder_name,
                            subfolder: None,
                            cleaned_folder_name: None,
                        });
                    }
                }
            } else if file_type.is_dir() {
                // 子文件夹
                let skip_dirs = [
                    "media", "images", "artwork", "videos", "screenshots",
                    "boxart", "snap", "wheel", "marquee", "named_boxarts", "named_snaps",
                ];
                if skip_dirs.iter().any(|&d| folder_name.eq_ignore_ascii_case(d)) {
                    continue;
                }

                // 扫描子文件夹中的ROM
                let mut subfolder_rom_paths: Vec<PathBuf> = Vec::new();
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.filter_map(|e| e.ok()) {
                        let sub_path = sub_entry.path();
                        let sub_file_type = match sub_entry.file_type() {
                            Ok(ft) => ft,
                            Err(_) => continue,
                        };
                        if sub_file_type.is_file() {
                            if let Some(ext) = sub_path.extension().and_then(|e| e.to_str()) {
                                if rom_extensions.contains(ext.to_lowercase().as_str()) {
                                    subfolder_rom_paths.push(sub_path);
                                }
                            }
                        }
                    }
                }

                if subfolder_rom_paths.is_empty() {
                    continue;
                }

                let cleaned_name = clean_folder_name(&folder_name);

                if subfolder_rom_paths.len() == 1 {
                    let filename = subfolder_rom_paths[0].file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    entries.push(RomScanEntry {
                        file: filename,
                        subfolder: Some(folder_name),
                        cleaned_folder_name: Some(cleaned_name),
                    });
                } else {
                    // 多个ROM，并行获取文件大小找最大的
                    let sizes: Vec<(String, u64)> = subfolder_rom_paths
                        .par_iter()
                        .map(|sub_path| {
                            let filename = sub_path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .to_string();
                            let file_size = sub_path.metadata()
                                .map(|m| m.len())
                                .unwrap_or(0);
                            (filename, file_size)
                        })
                        .collect();

                    if let Some((largest_filename, _)) = sizes.into_iter().max_by_key(|(_, size)| *size) {
                        entries.push(RomScanEntry {
                            file: largest_filename,
                            subfolder: Some(folder_name),
                            cleaned_folder_name: Some(cleaned_name),
                        });
                    }
                }
            }
        }
    }

    eprintln!("[DEBUG] scan_directory_internal complete, found {} ROM entries", entries.len());
    entries
}

fn scan_directory_with_folders(dir_path: &Path) -> Vec<RomScanEntry> {
    scan_directory_internal(dir_path, None::<fn(usize, usize, &str)>)
}

fn scan_directory_with_folders_progress(dir_path: &Path, app: &AppHandle) -> Vec<RomScanEntry> {
    let app_handle = app.clone();
    scan_directory_internal(dir_path, Some(move |current, total, folder: &str| {
        let _ = app_handle.emit("scan-progress", ScanProgress {
            current,
            total,
            current_folder: folder.to_string(),
        });
    }))
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

/// 扫描进度事件
#[derive(Clone, Serialize)]
pub struct ScanProgress {
    pub current: usize,
    pub total: usize,
    pub current_folder: String,
}

#[tauri::command]
pub async fn scan_directory_for_naming_check(
    app: AppHandle,
    path: String,
) -> Result<Vec<NamingCheckResult>, String> {
    let path_clone = path.clone();
    
    // 先快速获取目录条目数量用于进度显示
    let dir_path = Path::new(&path);
    if !dir_path.exists() {
        return Err("Directory does not exist".to_string());
    }
    
    // 在后台线程执行IO密集型操作
    tokio::task::spawn_blocking(move || {
        let dir_path = Path::new(&path_clone);

        // 1. 检测是否存在 metadata 文件
        let metadata_format = detect_metadata_format(dir_path);
        
        // 2. 根据 metadata 存在与否决定扫描方式
        let (scan_entries, metadata_entries) = if metadata_format != "none" {
            let system_name = dir_path.file_name().unwrap_or_default().to_string_lossy().to_string();
            eprintln!("[naming_check] Found metadata ({}), loading directly...", metadata_format);
            
            let roms_result = if metadata_format == "pegasus" {
                // CN ROM Tool 只需要 file/name/english_name；避免 get_roms_from_directory 的媒体扫描
                read_pegasus_roms_light(dir_path, &system_name)
            } else {
                get_roms_from_directory(dir_path, &metadata_format, &system_name)
            };

            if let Ok(roms) = roms_result {
                // 将 RomInfo 转换为 RomScanEntry
                let entries: Vec<RomScanEntry> = roms.iter().map(|rom| {
                    let path = Path::new(&rom.file);
                    // 如果文件路径包含目录分隔符，则认为是在子文件夹中
                    let subfolder = path.parent()
                        .and_then(|p| p.to_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());
                    
                    let cleaned_folder_name = subfolder.as_ref().map(|s| clean_folder_name(s));
                    
                    // file 只存储纯文件名，subfolder 单独存储
                    // 这样后续逻辑可以统一处理：file_path = subfolder/file
                    let filename = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&rom.file)
                        .to_string();
                    
                    RomScanEntry {
                        file: filename,
                        subfolder,
                        cleaned_folder_name,
                    }
                }).collect();
                
                // 将 RomInfo 转换为 TempMetadataEntry 以预填充数据
                let meta_entries: Vec<TempMetadataEntry> = roms.iter().map(|rom| {
                    TempMetadataEntry {
                        file: rom.file.clone(),
                        name: Some(rom.name.clone()),
                        english_name: rom.english_name.clone(),
                        // 如果 metadata 中有英文名，我们假设它是正确的（confidence=100）
                        confidence: if rom.english_name.is_some() { Some(100.0) } else { None },
                        extracted_cn_name: None, // 将在后续循环中重新计算
                    }
                }).collect();
                
                (entries, Some(meta_entries))
            } else {
                eprintln!("[naming_check] Failed to parse metadata, falling back to file scan.");
                (scan_directory_with_folders_progress(dir_path, &app), None)
            }
        } else {
            // 没有 metadata，执行常规文件扫描
            (scan_directory_with_folders_progress(dir_path, &app), None)
        };

        // 读取现有的临时元数据
        let mut temp_entries = load_temp_cn_metadata(&path_clone).unwrap_or_default();

        // 如果从 metadata 加载了数据，将其合并到 temp_entries 中（如果不冲突）
        if let Some(meta_entries) = metadata_entries {
            for meta in meta_entries {
                // 如果 temp 中没有这个文件，或者 temp 中没有 english_name 但 metadata 有
                if let Some(existing) = temp_entries.iter_mut().find(|e| e.file == meta.file) {
                    if existing.english_name.is_none() && meta.english_name.is_some() {
                        existing.english_name = meta.english_name;
                        existing.confidence = meta.confidence;
                    }
                    if existing.name.is_none() && meta.name.is_some() {
                        existing.name = meta.name;
                    }
                } else {
                    temp_entries.push(meta);
                }
            }
        }

        // 转换为检查结果，合并临时数据
        // 使用 HashMap 进行去重，以 file 字段为 key
        let mut results_map: std::collections::HashMap<String, NamingCheckResult> = std::collections::HashMap::new();
        // 同时构建需要保存的临时条目
        let mut new_temp_entries: Vec<TempMetadataEntry> = Vec::new();
        
        for entry in scan_entries {
            // 优先使用清理后的文件夹名，否则从文件名提取
            let extracted = if let Some(ref folder_name) = entry.cleaned_folder_name {
                Some(folder_name.clone())
            } else {
                parse_cn_name_from_filename(&entry.file)
            };

            // 查找临时数据中的匹配项
            // file 字段使用完整相对路径: subfolder/filename 或纯 filename
            let file_path = if let Some(ref subfolder) = entry.subfolder {
                format!("{}/{}", subfolder, entry.file)
            } else {
                entry.file.clone()
            };
            // 优先按完整路径匹配，如果没找到再按纯文件名匹配（兼容旧数据）
            let temp_data = temp_entries.iter().find(|e| e.file == file_path)
                .or_else(|| temp_entries.iter().find(|e| e.file == entry.file));

            let result = NamingCheckResult {
                file: file_path.clone(),
                name: temp_data.and_then(|t| t.name.clone()).or_else(|| extracted.clone()).unwrap_or_else(|| entry.file.clone()),
                english_name: temp_data.and_then(|t| t.english_name.clone()),
                extracted_cn_name: extracted.clone(),
                confidence: temp_data.and_then(|t| t.confidence),
            };
            
            // 构建临时条目（保留已有数据，更新 extracted_cn_name）
            let temp_entry = TempMetadataEntry {
                file: file_path.clone(),
                name: temp_data.and_then(|t| t.name.clone()),
                english_name: temp_data.and_then(|t| t.english_name.clone()),
                confidence: temp_data.and_then(|t| t.confidence),
                extracted_cn_name: extracted,
            };
            new_temp_entries.push(temp_entry);
            
            // 去重：如果已存在，保留有更多信息的条目
            if let Some(existing) = results_map.get(&file_path) {
                // 保留有 english_name 或更高 confidence 的条目
                let should_replace = match (&result.english_name, &existing.english_name) {
                    (Some(_), None) => true,
                    (Some(_), Some(_)) => {
                        // 都有 english_name，比较 confidence
                        result.confidence.unwrap_or(0.0) > existing.confidence.unwrap_or(0.0)
                    }
                    _ => false,
                };
                if should_replace {
                    results_map.insert(file_path, result);
                }
            } else {
                results_map.insert(file_path, result);
            }
        }
        
        // 保存扫描结果到临时 metadata（包含 extracted_cn_name）
        let _ = save_temp_cn_metadata(&path_clone, &new_temp_entries);
        
        let results: Vec<NamingCheckResult> = results_map.into_values().collect();

        Ok(results)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
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

/// 在 jy6d-dz 数据中进行快速匹配
fn fast_match_jy6d(
    query_cn: &str,
    english_suffix: Option<&str>,
    jy6d_entries: &[Jy6dDzEntry],
) -> Option<(String, String, f32)> {
    let query_lower = query_cn.to_lowercase();
    let mut best_match: Option<(String, String, f32)> = None;

    for entry in jy6d_entries {
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

    // 从临时 metadata 读取已扫描的条目（不再重新扫描文件系统）
    let mut entries = load_temp_cn_metadata(&path).unwrap_or_default();
    if entries.is_empty() {
        return Err("No scan data found. Please scan the directory first.".to_string());
    }
    let total = entries.len();

    // 优先使用传入的系统名，否则从目录名获取
    let system_name = system.unwrap_or_else(|| {
        dir_path.file_name().unwrap_or_default().to_string_lossy().to_string()
    });

    // 一次性加载 cn_repo CSV 到内存（优先使用打包资源）
    let repo_paths = get_cn_repo_paths(&app);
    let csv_entries: Vec<CnRomEntry> = {
        let mut csv_data = Vec::new();
        for repo_path in &repo_paths {
            if let Some(csv_path) = find_csv_in_dir(repo_path, &system_name) {
                eprintln!("[auto_fix_naming] Found cn_repo CSV at: {:?}", csv_path);
                if let Ok(loaded) = read_csv(&csv_path) {
                    csv_data = loaded;
                    break;
                }
            }
        }
        csv_data
    };

    // 加载 jy6d-dz CSV 到内存（作为补充数据源）
    let jy6d_entries: Vec<Jy6dDzEntry> = {
        let mut jy6d_data = Vec::new();
        if let Some(mapping) = find_mapping_by_folder(&system_name) {
            if let Some(jy6d_csv_name) = mapping.jy6d_csv_name {
                if let Some(csv_path) = get_jy6d_csv_path(&app, jy6d_csv_name) {
                    eprintln!("[auto_fix_naming] Found jy6d CSV at: {:?}", csv_path);
                    if let Ok(loaded) = load_jy6d_csv(&csv_path) {
                        jy6d_data = loaded;
                    }
                }
            }
        }
        jy6d_data
    };

    // 如果两个数据源都没有数据，返回错误
    if csv_entries.is_empty() && jy6d_entries.is_empty() {
        eprintln!("[auto_fix_naming] No CSV found for system: {} in paths: {:?}", system_name, repo_paths);
        return Err(format!("No CSV database found for system: {}", system_name));
    }

    eprintln!("[auto_fix_naming] Loaded {} entries from cn_repo, {} entries from jy6d", csv_entries.len(), jy6d_entries.len());

    let mut success_count = 0;
    let mut failed_count = 0;

    for (idx, entry) in entries.iter_mut().enumerate() {
        // 发送进度事件
        let _ = app.emit("naming-match-progress", MatchProgress {
            current: idx + 1,
            total,
        });

        // 检查是否已有用户手动编辑的数据（confidence = 100）
        if entry.confidence == Some(100.0) && entry.english_name.is_some() {
            continue;
        }

        // 查询名优先级：name（用户编辑/已生成）> extracted_cn_name > 从文件名提取
        let query_name = entry.name.clone()
            .or_else(|| entry.extracted_cn_name.clone())
            .or_else(|| parse_cn_name_from_filename(&entry.file))
            .unwrap_or_else(|| entry.file.clone());
        let english_suffix = extract_english_suffix(&entry.file);

        // 双数据源匹配：先查 cn_repo，再查 jy6d，取最高置信度结果
        let cn_repo_match = if !csv_entries.is_empty() {
            fast_match(&query_name, english_suffix.as_deref(), &csv_entries)
        } else {
            None
        };

        let jy6d_match = if !jy6d_entries.is_empty() {
            fast_match_jy6d(&query_name, english_suffix.as_deref(), &jy6d_entries)
        } else {
            None
        };

        // 选择置信度更高的结果
        let best_result = match (cn_repo_match, jy6d_match) {
            (Some((eng1, cn1, conf1)), Some((eng2, cn2, conf2))) => {
                if conf1 >= conf2 {
                    Some((eng1, cn1, conf1))
                } else {
                    Some((eng2, cn2, conf2))
                }
            }
            (Some(r), None) => Some(r),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        };

        if let Some((eng_name, _cn_name, confidence)) = best_result {
            // 只有一个匹配且置信度 > 0.75，或高置信度 > 0.95
            if confidence > 0.95 || confidence > 0.75 {
                // 清理英文名，去除括号中的区域信息
                let cleaned_eng_name = clean_english_name(&eng_name);
                let new_confidence = confidence * 100.0;

                entry.english_name = Some(cleaned_eng_name);
                entry.confidence = Some(new_confidence);
                // name 已在扫描时设置，无需再修改
                success_count += 1;
            } else {
                failed_count += 1;
            }
        } else {
            failed_count += 1;
        }
    }

    // 同步写入 temp pegasus metadata（将 english_name 落盘到 x-mrrm-eng）
    {
        let metadata_path = ensure_temp_pegasus_metadata_exists(dir_path, &system_name)?;
        let mut pegasus_metadata = parse_pegasus_file(&metadata_path).unwrap_or_default();

        for entry in entries.iter() {
            let Some(ref eng_name) = entry.english_name else {
                continue;
            };

            if let Some(game) = pegasus_metadata
                .games
                .iter_mut()
                .find(|g| g.file.as_deref() == Some(entry.file.as_str()))
            {
                game.extra.insert("x-mrrm-eng".to_string(), eng_name.clone());
            } else {
                let mut new_game = crate::scraper::pegasus::PegasusGame {
                    name: entry.name.clone().unwrap_or_else(|| entry.file.clone()),
                    file: Some(entry.file.clone()),
                    ..Default::default()
                };
                new_game
                    .extra
                    .insert("x-mrrm-eng".to_string(), eng_name.clone());
                pegasus_metadata.games.push(new_game);
            }
        }

        write_updated_pegasus_games(&metadata_path, &system_name, pegasus_metadata)?;
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
    /// 从文件夹名或文件名提取的中文名（用于匹配查询）
    #[serde(default)]
    extracted_cn_name: Option<String>,
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

/// 只读取临时元数据，不扫描文件系统（用于快速刷新）
#[tauri::command]
pub fn get_naming_check_results(path: String) -> Result<Vec<NamingCheckResult>, String> {
    let entries = load_temp_cn_metadata(&path).unwrap_or_default();
    
    // 使用 HashMap 去重，以 file 字段为 key
    let mut results_map: std::collections::HashMap<String, NamingCheckResult> = std::collections::HashMap::new();
    
    for entry in entries {
        let result = NamingCheckResult {
            file: entry.file.clone(),
            name: entry.name.unwrap_or_else(|| entry.file.clone()),
            english_name: entry.english_name,
            // 直接使用已保存的 extracted_cn_name，不再重复提取
            extracted_cn_name: entry.extracted_cn_name,
            confidence: entry.confidence,
        };
        
        // 去重：保留有更多信息的条目
        if let Some(existing) = results_map.get(&entry.file) {
            let should_replace = match (&result.english_name, &existing.english_name) {
                (Some(_), None) => true,
                (Some(_), Some(_)) => {
                    result.confidence.unwrap_or(0.0) > existing.confidence.unwrap_or(0.0)
                }
                _ => false,
            };
            if should_replace {
                results_map.insert(entry.file, result);
            }
        } else {
            results_map.insert(entry.file, result);
        }
    }
    
    let results: Vec<NamingCheckResult> = results_map.into_values().collect();
    
    Ok(results)
}

/// 将提取的中文名设置为 ROM 名 (写入临时 metadata)
#[tauri::command]
pub async fn set_extracted_cn_as_name(directory: String) -> Result<AutoFixResult, String> {
    let dir_path = Path::new(&directory);
    if !dir_path.exists() {
        return Err("Directory does not exist".to_string());
    }
    let system_name = dir_path.file_name().unwrap_or_default().to_string_lossy().to_string();

    let mut entries = load_temp_cn_metadata(&directory).unwrap_or_default();
    let mut success_count = 0;

    // 同步写入 temp 的 pegasus metadata（不扫描媒体）
    let metadata_path = ensure_temp_pegasus_metadata_exists(dir_path, &system_name)?;
    let mut pegasus_metadata = parse_pegasus_file(&metadata_path).unwrap_or_default();

    for entry in entries.iter_mut() {
        // "设置ROM名" 以提取的中文名为准（extracted_cn_name）
        let Some(cn_name) = entry.extracted_cn_name.clone() else {
            continue;
        };

        entry.name = Some(cn_name.clone());
        success_count += 1;

        let entry_basename = Path::new(&entry.file)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(entry.file.as_str());

        // 写入 pegasus: 更新 game name
        // 兼容：metadata 里可能是纯文件名，也可能是 subfolder/filename
        if let Some(game) = pegasus_metadata.games.iter_mut().find(|g| {
            let Some(ref gf) = g.file else {
                return false;
            };
            if gf == &entry.file {
                return true;
            }
            Path::new(gf)
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|bn| bn == entry_basename)
        }) {
            game.name = cn_name;

            // 如果 pegasus 里还是纯文件名，顺便升级成完整相对路径
            if let Some(ref gf) = game.file {
                if !gf.contains('/') && entry.file.contains('/') {
                    game.file = Some(entry.file.clone());
                }
            }
        } else {
            pegasus_metadata.games.push(crate::scraper::pegasus::PegasusGame {
                name: cn_name,
                file: Some(entry.file.clone()),
                ..Default::default()
            });
        }
    }

    save_temp_cn_metadata(&directory, &entries)?;

    write_updated_pegasus_games(&metadata_path, &system_name, pegasus_metadata)?;

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

    let english_opt = if english_name.is_empty() {
        None
    } else {
        Some(english_name.clone())
    };

    // 查找或创建条目
    if let Some(entry) = entries.iter_mut().find(|e| e.file == file) {
        // 更新现有条目
        entry.english_name = english_opt.clone();
        // 用户手动编辑的自动设置为满分
        entry.confidence = Some(100.0);
    } else {
        // 创建新条目
        entries.push(TempMetadataEntry {
            file: file.clone(),
            name: None,
            english_name: english_opt.clone(),
            confidence: Some(100.0), // 用户手动编辑的自动设置为满分
            extracted_cn_name: None,
        });
    }

    save_temp_cn_metadata(&directory, &entries)?;

    // 同步写入 temp pegasus metadata
    let dir_path = Path::new(&directory);
    let system_name = dir_path.file_name().unwrap_or_default().to_string_lossy().to_string();
    upsert_temp_pegasus_game_english_name(dir_path, &system_name, &file, english_opt.as_deref())?;
    Ok(())
}

/// 更新提取的中文名（仅影响 CN ROM Tool 的 extracted_cn_name 字段）
/// 该字段会被 "设置ROM名" 功能用于写入 pegasus metadata 的 game name。
#[tauri::command]
pub async fn update_extracted_cn_name(
    directory: String,
    file: String,
    extracted_cn_name: String,
) -> Result<(), String> {
    let mut entries = load_temp_cn_metadata(&directory).unwrap_or_default();

    let extracted_opt = if extracted_cn_name.trim().is_empty() {
        None
    } else {
        Some(extracted_cn_name.trim().to_string())
    };

    if let Some(entry) = entries.iter_mut().find(|e| e.file == file) {
        entry.extracted_cn_name = extracted_opt;
    } else {
        // 兼容：如果临时数据里还没有该文件，则创建新条目
        entries.push(TempMetadataEntry {
            file,
            name: None,
            english_name: None,
            confidence: None,
            extracted_cn_name: extracted_opt,
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
    use crate::scraper::pegasus::{PegasusGame, PegasusExportOptions, write_pegasus_file};
    use std::path::Path;
    
    // 转换为 PegasusGame 列表
    let games: Vec<PegasusGame> = entries.iter().map(|entry| {
        let mut game = PegasusGame {
            name: entry.name.clone().unwrap_or_else(|| entry.file.clone()),
            file: Some(entry.file.clone()),
            ..Default::default()
        };
        
        // 添加英文名到 x-mrrm-eng 字段
        if let Some(ref eng_name) = entry.english_name {
            game.extra.insert("x-mrrm-eng".to_string(), eng_name.clone());
        }
        
        game
    }).collect();
    
    // 不包含 collection header（由用户自行添加或合并到现有文件）
    let options = PegasusExportOptions {
        include_collection: false,
        ..Default::default()
    };
    
    // 写入文件（不合并，直接覆盖）
    write_pegasus_file(Path::new(target_path), &games, &options, false)
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
