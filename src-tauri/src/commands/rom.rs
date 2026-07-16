use crate::rom_index::{
    load_cached_roms, load_cached_roms_for_library, scan_library as scan_index, scan_library_by_id,
    ScanMode,
};
use crate::rom_service::{get_roms_for_directory, SystemRoms};
use crate::settings::{native_path, DirectoryConfig};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

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
                    rom.box_front.is_some()
                        || rom.description.is_some()
                        || rom.temp_data.as_ref().is_some_and(|game| {
                            game.box_front.is_some() || game.description.is_some()
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
