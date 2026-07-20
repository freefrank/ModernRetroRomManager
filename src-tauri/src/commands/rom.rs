use crate::rom_index::{
    load_cached_roms, load_cached_roms_for_library, scan_library as scan_index, scan_library_by_id,
    ScanMode,
};
use crate::rom_service::{get_roms_for_directory, SystemRoms};
use crate::settings::{native_path, DirectoryConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tauri_plugin_opener::OpenerExt;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RomFilter {
    pub system: Option<String>,
    pub search_query: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RomStats {
    pub total_roms: usize,
    pub total_systems: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RomSystemSummary {
    pub system: String,
    pub path: String,
    pub rom_count: usize,
    pub scraped_count: usize,
    pub total_size: u64,
}

/// gamelist.xml 的空标签(如 `<desc/>`)会解析成 Some(""),不能算已抓取。
fn present(value: &Option<String>) -> bool {
    value
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
}

fn summarize(systems: &[SystemRoms]) -> Vec<RomSystemSummary> {
    systems
        .iter()
        .map(|entry| RomSystemSummary {
            system: entry.system.clone(),
            path: native_path(&entry.path),
            rom_count: entry.roms.len(),
            scraped_count: entry
                .roms
                .iter()
                .filter(|rom| {
                    present(&rom.box_front)
                        || present(&rom.description)
                        || rom.temp_data.as_ref().is_some_and(|game| {
                            present(&game.box_front) || present(&game.description)
                        })
                })
                .count(),
            total_size: entry.roms.iter().filter_map(|rom| rom.file_size).sum(),
        })
        .collect()
}

async fn load_cached_roms_async() -> Result<Option<Vec<SystemRoms>>, String> {
    tokio::task::spawn_blocking(load_cached_roms)
        .await
        .map_err(|error| format!("ROM 索引读取任务失败: {error}"))
}

async fn load_library_cached_roms_async(
    library_id: String,
) -> Result<Option<Vec<SystemRoms>>, String> {
    tokio::task::spawn_blocking(move || load_cached_roms_for_library(&library_id))
        .await
        .map_err(|error| format!("Library 索引读取任务失败: {error}"))
}

#[tauri::command]
pub async fn get_library_rom_summary(library_id: String) -> Result<Vec<RomSystemSummary>, String> {
    // 本命令用于切库时即时恢复 UI，不能在缓存缺失时偷偷执行全量扫描。
    // 首次扫描和后台刷新由显式 scan_rom_library 命令负责并报告进度。
    let systems = load_library_cached_roms_async(library_id)
        .await?
        .unwrap_or_default();
    Ok(summarize(&systems))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RomScanProgress {
    pub current: usize,
    pub total: usize,
    pub system: Option<String>,
    pub mode: String,
    pub message: String,
    pub finished: bool,
    pub changed: bool,
    pub library_id: Option<String>,
}

async fn scan_with_events(
    app: AppHandle,
    mode: ScanMode,
    library_id: Option<String>,
) -> Result<Vec<SystemRoms>, String> {
    crate::rom_index::begin_scan();
    let mode_name = match mode {
        ScanMode::Full => "full",
        ScanMode::Incremental => "incremental",
    }
    .to_string();
    let current = Arc::new(AtomicUsize::new(0));
    let total = Arc::new(AtomicUsize::new(0));
    let event_app = app.clone();
    let event_mode = mode_name.clone();
    let event_current = Arc::clone(&current);
    let event_total = Arc::clone(&total);
    let scan_library_id = library_id.clone();

    let _ = app.emit(
        "rom-scan-progress",
        RomScanProgress {
            current: 0,
            total: 0,
            system: None,
            mode: mode_name.clone(),
            message: "正在准备 ROM 扫描".to_string(),
            finished: false,
            changed: false,
            library_id: library_id.clone(),
        },
    );

    let systems = tokio::task::spawn_blocking(move || {
        let on_progress = |update: crate::rom_index::ScanUpdate| {
            event_current.store(update.current, Ordering::Release);
            event_total.store(update.total, Ordering::Release);
            let action = if update.changed { "扫描" } else { "检查" };
            let _ = event_app.emit(
                "rom-scan-progress",
                RomScanProgress {
                    current: update.current,
                    total: update.total,
                    system: Some(update.system.clone()),
                    mode: event_mode.clone(),
                    message: format!("正在{action}: {}", update.system),
                    finished: false,
                    changed: update.changed,
                    library_id: scan_library_id.clone(),
                },
            );
        };
        if let Some(library_id) = scan_library_id.as_deref() {
            scan_library_by_id(library_id, mode, on_progress)
        } else {
            scan_index(mode, on_progress)
        }
    })
    .await
    .map_err(|error| format!("ROM 扫描任务失败: {error}"))??;

    let cancelled = crate::rom_index::scan_cancelled();
    let _ = app.emit(
        "rom-scan-progress",
        RomScanProgress {
            current: current.load(Ordering::Acquire),
            total: total.load(Ordering::Acquire),
            system: None,
            mode: mode_name,
            message: if cancelled {
                "ROM 扫描已停止，已保留完成的索引".to_string()
            } else {
                "ROM 扫描完成".to_string()
            },
            finished: true,
            changed: false,
            library_id,
        },
    );
    crate::rom_index::begin_scan();
    Ok(systems)
}

#[tauri::command]
pub fn cancel_rom_scan() {
    crate::rom_index::cancel_scan();
}

/// 获取 ROM 列表 (按系统分组或扁平化)
#[tauri::command]
pub async fn get_roms(
    app: AppHandle,
    filter: Option<RomFilter>,
) -> Result<Vec<SystemRoms>, String> {
    let all_systems = if let Some(cached) = load_cached_roms_async().await? {
        cached
    } else {
        scan_with_events(app, ScanMode::Full, None).await?
    };

    if let Some(f) = filter {
        let mut filtered_systems = Vec::new();

        for system_roms in all_systems {
            // 系统过滤
            if let Some(sys) = &f.system {
                if &system_roms.system != sys {
                    continue;
                }
            }

            // 搜索过滤
            let roms = if let Some(query) = &f.search_query {
                let lower_query = query.to_lowercase();
                system_roms
                    .roms
                    .into_iter()
                    .filter(|r| {
                        r.name.to_lowercase().contains(&lower_query)
                            || r.chinese_name
                                .as_ref()
                                .is_some_and(|name| name.to_lowercase().contains(&lower_query))
                    })
                    .collect()
            } else {
                system_roms.roms
            };

            if !roms.is_empty() {
                filtered_systems.push(SystemRoms {
                    system: system_roms.system,
                    path: system_roms.path,
                    roms,
                });
            }
        }

        Ok(filtered_systems)
    } else {
        Ok(all_systems)
    }
}

#[tauri::command]
pub async fn get_rom_library_summary(app: AppHandle) -> Result<Vec<RomSystemSummary>, String> {
    let systems = if let Some(cached) = load_cached_roms_async().await? {
        cached
    } else {
        scan_with_events(app, ScanMode::Full, None).await?
    };
    Ok(summarize(&systems))
}

#[tauri::command]
pub async fn get_system_roms(app: AppHandle, system: String) -> Result<SystemRoms, String> {
    let systems = if let Some(cached) = load_cached_roms_async().await? {
        cached
    } else {
        scan_with_events(app, ScanMode::Full, None).await?
    };
    systems
        .into_iter()
        .find(|entry| entry.system.eq_ignore_ascii_case(&system))
        .ok_or_else(|| format!("未找到 ROM 平台: {system}"))
}

/// 获取 ROM 统计信息
#[tauri::command]
pub async fn get_rom_stats() -> Result<RomStats, String> {
    let all_systems = if let Some(cached) = load_cached_roms_async().await? {
        cached
    } else {
        tokio::task::spawn_blocking(|| scan_index(ScanMode::Full, |_| {}))
            .await
            .map_err(|error| format!("ROM 扫描任务失败: {error}"))??
    };

    let total_systems = all_systems.len();
    let total_roms = all_systems.iter().map(|s| s.roms.len()).sum();

    Ok(RomStats {
        total_roms,
        total_systems,
    })
}

#[tauri::command]
pub async fn scan_rom_library(
    app: AppHandle,
    full: bool,
    library_id: Option<String>,
) -> Result<Vec<RomSystemSummary>, String> {
    let systems = scan_with_events(
        app,
        if full {
            ScanMode::Full
        } else {
            ScanMode::Incremental
        },
        library_id,
    )
    .await?;
    Ok(summarize(&systems))
}

/// 获取单个目录的ROM列表
#[tauri::command]
#[allow(non_snake_case)]
pub async fn get_roms_for_single_directory(
    path: String,
    metadataFormat: String,
    isRoot: bool,
    systemId: Option<String>,
) -> Result<Vec<SystemRoms>, String> {
    let dir_config = DirectoryConfig {
        id: String::new(),
        name: String::new(),
        path,
        metadata_format: metadataFormat,
        is_root_directory: isRoot,
        system_id: systemId,
        indexed_folders: None,
    };

    let systems = tokio::task::spawn_blocking(move || get_roms_for_directory(&dir_config))
        .await
        .map_err(|e| format!("Failed to spawn blocking task: {}", e))?;

    Ok(systems)
}

// ============================================================================
// ROM 隐藏与删除
// ============================================================================
//
// 隐藏状态是 UI 层过滤,不写入 ROM 数据本身:独立持久化到
// `<config>/hidden-roms.json`,键为规范化后的平台目录,值为该目录下被隐藏的
// ROM 相对文件名。隐藏的 ROM 默认不显示、且导出时不写入 gamelist/pegasus。

type HiddenMap = HashMap<String, Vec<String>>;

#[derive(Debug, Clone, Serialize)]
pub struct HiddenRom {
    pub directory: String,
    pub file: String,
}

fn hidden_store_path() -> PathBuf {
    crate::config::get_config_dir().join("hidden-roms.json")
}

/// 规范化目录键:统一分隔符为 `/`、去尾部斜杠、转小写。前端需用相同规则匹配。
pub(crate) fn normalize_dir_key(directory: &str) -> String {
    directory
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_lowercase()
}

fn load_hidden_map() -> HiddenMap {
    fs::read(hidden_store_path())
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn save_hidden_map(map: &HiddenMap) -> Result<(), String> {
    let path = hidden_store_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let bytes = serde_json::to_vec_pretty(map).map_err(|error| error.to_string())?;
    fs::write(path, bytes).map_err(|error| error.to_string())
}

// ── 货架页:隐藏/删除整个系统(平台目录) ──────────────────────────────────
fn hidden_systems_path() -> PathBuf {
    crate::config::get_config_dir().join("hidden-systems.json")
}

fn load_hidden_systems() -> std::collections::HashSet<String> {
    fs::read(hidden_systems_path())
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn save_hidden_systems(set: &std::collections::HashSet<String>) -> Result<(), String> {
    let path = hidden_systems_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let bytes = serde_json::to_vec_pretty(set).map_err(|error| error.to_string())?;
    fs::write(path, bytes).map_err(|error| error.to_string())
}

/// 货架页被隐藏的系统目录(规范化路径集合;前端用相同规则匹配过滤)。
#[tauri::command]
pub fn get_hidden_systems() -> Vec<String> {
    load_hidden_systems().into_iter().collect()
}

/// 隐藏/取消隐藏某系统(按其目录路径)。
#[tauri::command]
pub fn set_system_hidden(directory: String, hidden: bool) -> Result<(), String> {
    let key = normalize_dir_key(&directory);
    let mut set = load_hidden_systems();
    if hidden {
        set.insert(key);
    } else {
        set.remove(&key);
    }
    save_hidden_systems(&set)
}

/// 物理删除某系统目录(整个平台文件夹)。不可逆;前端须二次确认。
#[tauri::command]
pub fn delete_system_directory(directory: String) -> Result<(), String> {
    let path = PathBuf::from(&directory);
    if !path.is_dir() {
        return Err("目录不存在".to_string());
    }
    // 安全:必须至少是 <盘符/库/系统> 三级路径,拒绝删除盘根或库根。
    if path.parent().and_then(|p| p.parent()).is_none() {
        return Err("拒绝删除:路径过浅,疑似盘根或库根".to_string());
    }
    fs::remove_dir_all(&path).map_err(|error| error.to_string())?;
    let key = normalize_dir_key(&directory);
    let mut set = load_hidden_systems();
    if set.remove(&key) {
        let _ = save_hidden_systems(&set);
    }
    Ok(())
}

/// 某平台目录下被隐藏的 ROM 相对文件名列表(供导出过滤复用)。
pub(crate) fn hidden_files_for_directory(directory: &str) -> Vec<String> {
    load_hidden_map()
        .remove(&normalize_dir_key(directory))
        .unwrap_or_default()
}

/// game 的相对文件名是否落在某个被隐藏 ROM 的范围内。
/// 子文件夹型光盘游戏(file 含分隔符)按第一段(游戏文件夹)匹配,一个游戏多轨/多碟
/// 文件一并隐藏;单文件游戏精确匹配。
pub(crate) fn file_matches_hidden(file: &str, hidden_files: &[String]) -> bool {
    let normalize = |value: &str| value.replace('\\', "/");
    // 去扩展名取 stem(小写)。ROM 文件均带扩展名,取最后一个 '.' 为分隔即可。
    let stem = |value: &str| -> String {
        value
            .rsplit_once('.')
            .map(|(s, _)| s)
            .unwrap_or(value)
            .to_lowercase()
    };
    let target = normalize(file);
    let target_first = target.split('/').next().unwrap_or(&target).to_string();
    let target_has_subfolder = target.contains('/');
    let target_stem = stem(&target);
    hidden_files.iter().any(|hidden| {
        let hidden = normalize(hidden);
        if hidden.contains('/') {
            let hidden_first = hidden.split('/').next().unwrap_or(&hidden);
            target_first == hidden_first
        } else if target_has_subfolder {
            false
        } else {
            // 平铺型:同名(不同扩展,如 cue+bin+sub)或 "(track N)" 兄弟轨都属同一游戏,
            // 一并隐藏。此前仅精确匹配,导致隐藏游戏的载荷轨仍被复制/不被单向同步删除。
            crate::multidisc::is_sibling_track(&stem(&hidden), &target_stem)
        }
    })
}

/// 获取全部被隐藏的 ROM(扁平列表,供前端标记)。
#[tauri::command]
pub fn get_hidden_roms() -> Vec<HiddenRom> {
    load_hidden_map()
        .into_iter()
        .flat_map(|(directory, files)| {
            files.into_iter().map(move |file| HiddenRom {
                directory: directory.clone(),
                file,
            })
        })
        .collect()
}

/// 设置单个 ROM 的隐藏状态。
#[tauri::command]
pub fn set_rom_hidden(directory: String, file: String, hidden: bool) -> Result<(), String> {
    let key = normalize_dir_key(&directory);
    let mut map = load_hidden_map();
    let list = map.entry(key).or_default();
    if hidden {
        if !list.contains(&file) {
            list.push(file);
        }
    } else {
        list.retain(|value| value != &file);
    }
    map.retain(|_, files| !files.is_empty());
    save_hidden_map(&map)
}

/// 永久删除 ROM 文件(不可恢复)。
/// - 子文件夹型光盘游戏:删除整个游戏子文件夹(第一段目录)。
/// - 单文件游戏:删除该文件及同名兄弟文件(光盘的 cue/bin/img/sub 等)。
/// 删除后一并从隐藏清单移除该条目;临时元数据在下次加载时因文件不存在自动剔除。
#[tauri::command]
pub fn delete_rom(directory: String, file: String) -> Result<(), String> {
    let base = PathBuf::from(&directory);
    if !base.is_dir() {
        return Err(format!("平台目录不存在: {directory}"));
    }
    let normalized = file.replace('\\', "/");
    let mut segments = normalized.splitn(2, '/');
    let first = segments.next().unwrap_or("").trim();
    let has_subfolder = segments.next().is_some();
    if first.is_empty() || first == ".." || first == "." {
        return Err("非法的 ROM 路径".to_string());
    }

    if has_subfolder {
        // 子文件夹型:删除整个游戏文件夹,但绝不删到平台目录本身或其外部。
        let game_dir = base.join(first);
        let safe = game_dir
            .canonicalize()
            .ok()
            .zip(base.canonicalize().ok())
            .is_some_and(|(child, root)| child.starts_with(&root) && child != root);
        if !safe {
            return Err("拒绝删除:路径超出平台目录范围".to_string());
        }
        fs::remove_dir_all(&game_dir).map_err(|error| format!("删除游戏文件夹失败: {error}"))?;
    } else {
        // 单文件型:删除该文件及同 stem 兄弟(同一游戏的多个轨道/描述文件)。
        let stem = Path::new(&normalized)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or(first)
            .to_string();
        let mut removed_any = false;
        if let Ok(entries) = fs::read_dir(&base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(entry_stem) = path.file_stem().and_then(|value| value.to_str()) {
                        if entry_stem == stem {
                            if fs::remove_file(&path).is_ok() {
                                removed_any = true;
                            }
                        }
                    }
                }
            }
        }
        // 保底:直接删除目标文件(read_dir 失败时)。
        let target = base.join(&normalized);
        if target.is_file() {
            let _ = fs::remove_file(&target);
            removed_any = true;
        }
        if !removed_any {
            return Err(format!("未找到要删除的 ROM 文件: {file}"));
        }
    }

    // 从隐藏清单移除
    let key = normalize_dir_key(&directory);
    let mut map = load_hidden_map();
    if let Some(list) = map.get_mut(&key) {
        list.retain(|value| value != &file);
        if list.is_empty() {
            map.remove(&key);
        }
        let _ = save_hidden_map(&map);
    }
    Ok(())
}

/// 在系统文件管理器中定位 ROM(选中该文件);文件不存在时退回打开平台目录。
#[tauri::command]
pub fn open_rom_location(app: AppHandle, directory: String, file: String) -> Result<(), String> {
    let path = PathBuf::from(&directory).join(&file);
    if path.exists() {
        app.opener()
            .reveal_item_in_dir(&path)
            .map_err(|error| error.to_string())
    } else {
        app.opener()
            .open_path(directory, None::<&str>)
            .map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod hidden_tests {
    use super::*;

    #[test]
    fn hidden_match_covers_disc_game_folder_and_single_file() {
        // 子文件夹型:同一游戏文件夹内的多轨/多碟文件全部隐藏。
        let hidden = vec![r"D之食卓CD1\D_CN3_CD1.cue".to_string()];
        assert!(file_matches_hidden(r"D之食卓CD1\D_CN3_CD1.cue", &hidden));
        assert!(file_matches_hidden(r"D之食卓CD1\D_CN3_CD1.img", &hidden));
        assert!(!file_matches_hidden(r"D之食卓CD2\D_CN3_CD2.cue", &hidden));
        // 单文件型:精确匹配,不误伤其它游戏。
        let hidden = vec!["Chrono Trigger.sfc".to_string()];
        assert!(file_matches_hidden("Chrono Trigger.sfc", &hidden));
        assert!(!file_matches_hidden("Final Fantasy.sfc", &hidden));
    }

    #[test]
    fn hidden_match_covers_flat_multitrack_siblings() {
        // 平铺型光盘游戏:隐藏 .cue 时,同名 .bin/.img/.sub 载荷轨与 (track N) 轨都应一并隐藏,
        // 否则隐藏游戏的载荷轨仍被导出复制、且单向同步删不掉(回归)。
        let hidden = vec!["Ridge Racer (Japan).cue".to_string()];
        assert!(file_matches_hidden("Ridge Racer (Japan).cue", &hidden));
        assert!(file_matches_hidden("Ridge Racer (Japan).bin", &hidden));
        assert!(file_matches_hidden("Ridge Racer (Japan).sub", &hidden));
        assert!(file_matches_hidden("Ridge Racer (Japan) (Track 2).bin", &hidden));
        // 不误伤共享前缀但独立的游戏。
        assert!(!file_matches_hidden("Ridge Racer (Japan) 2.cue", &hidden));
        assert!(!file_matches_hidden("Ridge Racer Type 4 (Japan).cue", &hidden));
    }

    #[test]
    fn normalize_dir_key_is_stable_across_separators_and_case() {
        assert_eq!(
            normalize_dir_key(r"G:\ROMS\SS\"),
            normalize_dir_key("g:/roms/ss")
        );
    }

    #[test]
    fn delete_single_file_rom_removes_sibling_tracks() {
        let root = std::env::temp_dir().join(format!(
            "mrrm-del-single-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("Game.cue"), b"cue").unwrap();
        fs::write(root.join("Game.bin"), b"bin").unwrap();
        fs::write(root.join("Other.sfc"), b"rom").unwrap();

        delete_rom(root.to_string_lossy().to_string(), "Game.cue".to_string()).unwrap();
        assert!(!root.join("Game.cue").exists());
        assert!(!root.join("Game.bin").exists(), "同名兄弟轨道应一并删除");
        assert!(root.join("Other.sfc").exists(), "不应误删其它游戏");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn delete_subfolder_rom_removes_whole_game_folder() {
        let root = std::env::temp_dir().join(format!(
            "mrrm-del-folder-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let game = root.join("D之食卓CD1");
        fs::create_dir_all(&game).unwrap();
        fs::write(game.join("D.cue"), b"cue").unwrap();
        fs::write(game.join("D.img"), b"img").unwrap();
        fs::write(game.join("D.sub"), b"sub").unwrap();

        delete_rom(
            root.to_string_lossy().to_string(),
            r"D之食卓CD1\D.cue".to_string(),
        )
        .unwrap();
        assert!(!game.exists(), "整个游戏文件夹应被删除");
        assert!(root.exists(), "平台目录本身不应被删除");
        let _ = fs::remove_dir_all(&root);
    }
}
