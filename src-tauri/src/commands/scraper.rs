//! Scraper Tauri Commands
//!
//! 前端调用的 Scraper 相关命令

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::fs;
use tauri::{State, Emitter};
use tokio::sync::RwLock;
use crate::config::{get_temp_dir, get_temp_dir_for_library};
use crate::scraper::{
    manager::ScraperManager,
    types::{ScrapeQuery, SearchResult, GameMetadata, MediaAsset, MediaType, ScrapeResult},
    steamgriddb::SteamGridDBClient,
    screenscraper::ScreenScraperClient,
    pegasus::parse_pegasus_file, // Add parser import
    persistence::{download_media, save_metadata_pegasus, save_metadata_emulationstation},
    hash::{compute_rom_hash, DEFAULT_HASH_SIZE_LIMIT},
    matcher::normalize_game_name,
};
use crate::rom_service::RomInfo;


use crate::settings::ScraperConfig;

// ============================================================================
// State - ScraperManager 全局状态
// ============================================================================

pub struct ScraperState {
    pub manager: Arc<RwLock<ScraperManager>>,
    /// 批量抓取取消标记，置 true 后正在运行的批量任务会在当前项完成后停止
    pub batch_cancel: Arc<std::sync::atomic::AtomicBool>,
}

impl ScraperState {
    pub fn new() -> Self {
        // 确保配置已从磁盘加载（ScraperState 在 setup 之前创建）
        let _ = crate::settings::load_settings();

        let mut manager = ScraperManager::new();

        // 从持久化设置恢复已配置凭证的 provider，
        // 否则重启后 manager 为空，抓取功能静默失效
        let settings = crate::settings::get_settings();

        if let Some(config) = settings.scrapers.get("steamgriddb") {
            if let Some(api_key) = config.api_key.as_ref().filter(|k| !k.is_empty()) {
                manager.register(SteamGridDBClient::new(api_key.clone()));
            }
        }

        if let Some(config) = settings.scrapers.get("screenscraper") {
            if let (Some(username), Some(password)) = (
                config.username.as_ref().filter(|u| !u.is_empty()),
                config.password.as_ref().filter(|p| !p.is_empty()),
            ) {
                manager.register(ScreenScraperClient::new(username.clone(), password.clone()));
            }
        }

        Self {
            manager: Arc::new(RwLock::new(manager)),
            batch_cancel: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}

impl Default for ScraperState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Provider 配置
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: u32,
    pub has_credentials: bool,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCredentials {
    pub api_key: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// 获取所有可用的 provider 列表
#[tauri::command]
pub async fn get_scraper_providers(
    state: State<'_, ScraperState>,
) -> Result<Vec<ProviderInfo>, String> {
    let manager = state.manager.read().await;
    let provider_ids = manager.provider_ids();
    
    // 动态生成 provider 信息
    let mut providers = Vec::new();
    
    // SteamGridDB
    if let Some(config) = manager.get_credentials("steamgriddb") {
        providers.push(ProviderInfo {
            id: "steamgriddb".to_string(),
            name: "SteamGridDB".to_string(),
            enabled: config.enabled,
            priority: config.priority,
            has_credentials: config.api_key.is_some() && !config.api_key.unwrap().is_empty(),
            capabilities: vec!["search".to_string(), "media".to_string()],
        });
    } else {
        providers.push(ProviderInfo {
            id: "steamgriddb".to_string(),
            name: "SteamGridDB".to_string(),
            enabled: true, // 默认启用
            priority: 100, // 默认优先级
            has_credentials: false,
            capabilities: vec!["search".to_string(), "media".to_string()],
        });
    }

    // ScreenScraper
    if let Some(config) = manager.get_credentials("screenscraper") {
        providers.push(ProviderInfo {
            id: "screenscraper".to_string(),
            name: "ScreenScraper".to_string(),
            enabled: config.enabled,
            priority: config.priority,
            has_credentials: config.username.is_some() && !config.username.unwrap().is_empty()
                && config.password.is_some() && !config.password.unwrap().is_empty(),
            capabilities: vec![
                "search".to_string(),
                "hash_lookup".to_string(),
                "metadata".to_string(),
                "media".to_string(),
            ],
        });
    } else {
        providers.push(ProviderInfo {
            id: "screenscraper".to_string(),
            name: "ScreenScraper".to_string(),
            enabled: true, // 默认启用
            priority: 100, // 默认优先级
            has_credentials: false,
            capabilities: vec![
                "search".to_string(),
                "hash_lookup".to_string(),
                "metadata".to_string(),
                "media".to_string(),
            ],
        });
    }

    Ok(providers)
}

/// 配置 provider 凭证
#[tauri::command]
pub async fn configure_scraper_provider(
    state: State<'_, ScraperState>,
    provider_id: String,
    credentials: ProviderCredentials,
) -> Result<(), String> {
    let mut manager = state.manager.write().await;
    
    // 获取当前配置以保留 enabled 状态
    let current_config = manager.get_credentials(&provider_id).unwrap_or_default();
    
    let mut new_config = ScraperConfig {
        enabled: current_config.enabled,
        ..Default::default()
    };

    match provider_id.as_str() {
        "steamgriddb" => {
            if let Some(api_key) = credentials.api_key {
                if !api_key.is_empty() {
                    let client = SteamGridDBClient::new(api_key.clone());
                    manager.register(client);
                    new_config.api_key = Some(api_key);
                    manager.set_credentials(&provider_id, new_config);
                }
            }
        }
        "screenscraper" => {
            if let (Some(username), Some(password)) = (credentials.username, credentials.password) {
                if !username.is_empty() && !password.is_empty() {
                    let client = ScreenScraperClient::new(username.clone(), password.clone());
                    manager.register(client);
                    new_config.username = Some(username);
                    new_config.password = Some(password);
                    manager.set_credentials(&provider_id, new_config);
                }
            }
        }
        _ => return Err(format!("Unknown provider: {}", provider_id)),
    }
    
    Ok(())
}

/// 搜索游戏
#[tauri::command]
pub async fn scraper_search(
    state: State<'_, ScraperState>,
    name: String,
    file_name: String,
    system: Option<String>,
) -> Result<Vec<SearchResult>, String> {
    let manager = state.manager.read().await;
    
    let mut query = ScrapeQuery::new(name, file_name);
    if let Some(sys) = system {
        query = query.with_system(sys);
    }
    
    let results = manager.search(&query).await;
    Ok(results)
}

/// 获取游戏元数据
#[tauri::command]
pub async fn scraper_get_metadata(
    state: State<'_, ScraperState>,
    provider_id: String,
    source_id: String,
) -> Result<GameMetadata, String> {
    let manager = state.manager.read().await;
    manager.get_metadata(&provider_id, &source_id).await
}

/// 获取游戏媒体资产
#[tauri::command]
pub async fn scraper_get_media(
    state: State<'_, ScraperState>,
    provider_id: String,
    source_id: String,
) -> Result<Vec<MediaAsset>, String> {
    let manager = state.manager.read().await;
    manager.get_media(&provider_id, &source_id).await
}

/// 计算 ROM 文件 Hash（在阻塞线程中执行，避免阻塞异步运行时）
async fn compute_hash_for_rom(directory: &str, file_name: &str) -> Option<crate::scraper::RomHash> {
    let rom_path = Path::new(directory).join(file_name);
    tokio::task::spawn_blocking(move || compute_rom_hash(&rom_path, DEFAULT_HASH_SIZE_LIMIT))
        .await
        .ok()
        .flatten()
}

/// 智能 scrape - 自动匹配并聚合数据
///
/// 提供 directory 时会计算 ROM 文件 Hash，优先走精确匹配
#[tauri::command]
pub async fn scraper_auto_scrape(
    state: State<'_, ScraperState>,
    name: String,
    file_name: String,
    system: Option<String>,
    directory: Option<String>,
) -> Result<ScrapeResult, String> {
    let mut query = ScrapeQuery::new(name, file_name.clone());
    if let Some(sys) = system {
        query = query.with_system(sys);
    }
    if let Some(dir) = directory {
        if let Some(hash) = compute_hash_for_rom(&dir, &file_name).await {
            query = query.with_hash(hash);
        }
    }

    let manager = state.manager.read().await;
    manager.scrape(&query).await
}

/// 启用/禁用 provider
#[tauri::command]
pub async fn scraper_set_provider_enabled(
    state: State<'_, ScraperState>,
    provider_id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut manager = state.manager.write().await;
    manager.set_enabled(&provider_id, enabled);
    Ok(())
}

/// 设置 provider 优先级
#[tauri::command]
pub async fn scraper_set_provider_priority(
    state: State<'_, ScraperState>,
    provider_id: String,
    priority: u32,
) -> Result<(), String> {
    let mut manager = state.manager.write().await;
    manager.set_priority(&provider_id, priority);
    Ok(())
}

// ============================================================================
// ScraperManager 控制 (占位)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ApplyScrapedDataOptions {
    pub rom_id: String, // 文件名
    pub directory: String, // 目录
    pub system: String, // 系统
    pub metadata: GameMetadata,
    pub selected_media: Vec<MediaAsset>,
}

#[tauri::command]
pub async fn apply_scraped_data(
    state: State<'_, ScraperState>,
    options: ApplyScrapedDataOptions,
) -> Result<(), String> {
    // 1. 构建 RomInfo (用于定位目录和文件)
    let rom = RomInfo {
        file: options.rom_id.clone(),
        directory: options.directory.clone(),
        system: options.system.clone(),
        name: options.metadata.name.clone(),
        ..Default::default()
    };

    // 2. 下载媒体文件到临时目录
    if !options.selected_media.is_empty() {
        let manager = state.manager.read().await;
        download_media(&manager.http_client, &rom, &options.selected_media, true).await?;
    }

    // 3. 写入元数据到临时目录
    save_metadata_pegasus(&rom, &options.metadata, true)?;

    Ok(())
}

/// 批量处理进度
#[derive(Debug, Clone, Serialize)]
pub struct BatchProgress {
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub finished: bool,
}

#[tauri::command]
pub async fn batch_scrape(
    app: tauri::AppHandle,
    state: State<'_, ScraperState>,
    rom_ids: Vec<String>,
    system: String,
    directory: String,
    _provider_id: String,
) -> Result<(), String> {
    use std::sync::atomic::Ordering;

    let manager_arc = Arc::clone(&state.manager);
    let cancel_flag = Arc::clone(&state.batch_cancel);
    cancel_flag.store(false, Ordering::SeqCst);
    let total = rom_ids.len();

    tokio::spawn(async move {
        let mut success = 0usize;
        let mut failed = 0usize;
        let mut cancelled = false;

        for (i, file_name) in rom_ids.into_iter().enumerate() {
            if cancel_flag.load(Ordering::SeqCst) {
                cancelled = true;
                let _ = app.emit("batch-scrape-progress", BatchProgress {
                    current: i,
                    total,
                    message: format!("已取消批量抓取 (成功 {}, 失败 {})", success, failed),
                    finished: true,
                });
                break;
            }

            let current = i + 1;

            let _ = app.emit("batch-scrape-progress", BatchProgress {
                current,
                total,
                message: format!("正在抓取: {}", file_name),
                finished: false,
            });

            // 计算 Hash 走精确匹配；名称搜索使用清洗后的游戏名
            let mut query = ScrapeQuery::new(normalize_game_name(&file_name), file_name.clone())
                .with_system(system.clone());
            if let Some(hash) = compute_hash_for_rom(&directory, &file_name).await {
                query = query.with_hash(hash);
            }

            let scrape_res = {
                let manager = manager_arc.read().await;
                manager.scrape(&query).await
            };

            match scrape_res {
                Ok(result) => {
                    let rom = RomInfo {
                        file: file_name.clone(),
                        name: result.metadata.name.clone(),
                        system: system.clone(),
                        directory: directory.clone(),
                        ..Default::default()
                    };

                    // 先下载默认媒体（封面/截图/标题画面/Logo 各取优先级最高的一个），
                    // 再写元数据，让媒体路径能关联进 assets 字段
                    let assets = select_default_assets(&result.media);
                    if !assets.is_empty() {
                        let _ = app.emit("batch-scrape-progress", BatchProgress {
                            current,
                            total,
                            message: format!("正在下载媒体: {} ({} 个)", file_name, assets.len()),
                            finished: false,
                        });

                        let client = {
                            let manager = manager_arc.read().await;
                            manager.http_client.clone()
                        };
                        // 媒体下载失败不影响元数据抓取结果
                        let _ = download_media(&client, &rom, &assets, true).await;
                    }

                    match save_metadata_pegasus(&rom, &result.metadata, true) {
                        Ok(_) => success += 1,
                        Err(_) => failed += 1,
                    }
                }
                Err(_) => failed += 1,
            }
        }

        if !cancelled {
            let _ = app.emit("batch-scrape-progress", BatchProgress {
                current: total,
                total,
                message: format!("批量处理完成 (成功 {}, 失败 {})", success, failed),
                finished: true,
            });
        }
    });

    Ok(())
}

/// 批量抓取默认下载的媒体类型（按优先级排列）
const BATCH_MEDIA_TYPES: [MediaType; 5] = [
    MediaType::BoxFront,
    MediaType::Screenshot,
    MediaType::TitleScreen,
    MediaType::Logo,
    MediaType::Banner,
];

/// 从抓取结果中为每种默认媒体类型挑选第一个资产
/// （media 列表已按 provider 优先级排序）
fn select_default_assets(media: &[MediaAsset]) -> Vec<MediaAsset> {
    BATCH_MEDIA_TYPES
        .iter()
        .filter_map(|t| media.iter().find(|m| m.asset_type == *t).cloned())
        .collect()
}

/// 取消正在运行的批量抓取任务
#[tauri::command]
pub async fn cancel_batch_scrape(state: State<'_, ScraperState>) -> Result<(), String> {
    state
        .batch_cancel
        .store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn save_temp_metadata(
    system: String,
    directory: String,
    metadata: GameMetadata,
    rom_id: String,
) -> Result<(), String> {
    let rom = RomInfo {
        file: rom_id,
        directory,
        system,
        name: metadata.name.clone(),
        ..Default::default()
    };

    save_metadata_pegasus(&rom, &metadata, true)
}

#[tauri::command]
pub async fn delete_temp_media(
    system: String,
    rom_id: String,
    rom_directory: String,
    asset_type: String,
) -> Result<(), String> {
    let file_stem = Path::new(&rom_id)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&rom_id);

    // 与 download_media / get_temp_media_list 保持一致的库级临时目录
    let rom_dir = Path::new(&rom_directory);
    let library_path = rom_dir.parent().unwrap_or(rom_dir);
    let media_dir = get_temp_dir_for_library(library_path, &system)
        .join("media")
        .join(file_stem);

    if !media_dir.exists() {
        return Ok(());
    }

    // 查找匹配 asset_type 的文件 (忽略扩展名)
    for entry in fs::read_dir(media_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem == asset_type {
                    fs::remove_file(path).map_err(|e| e.to_string())?;
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct TempMediaInfo {
    pub asset_type: String,
    pub path: String,
}

#[tauri::command]
pub async fn get_temp_media_list(
    system: String,
    rom_id: String,
    rom_directory: String,
) -> Result<Vec<TempMediaInfo>, String> {
    // 从 rom_id (filename) 提取文件名主体
    let file_stem = Path::new(&rom_id)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&rom_id);

    // 计算 library_path (rom_directory 的父目录)
    let rom_dir = Path::new(&rom_directory);
    let library_path = rom_dir.parent().unwrap_or(rom_dir);
    
    // 媒体存储在: {temp_dir}/{library}/{system}/media/{file_stem}/
    let media_dir = get_temp_dir_for_library(library_path, &system)
        .join("media")
        .join(file_stem);

    let mut list = Vec::new();
    if media_dir.exists() && media_dir.is_dir() {
        for entry in fs::read_dir(&media_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_file() {
                // asset_type 是文件名主体 (e.g., "boxfront" from "boxfront.png")
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    list.push(TempMediaInfo {
                        asset_type: stem.to_string(),
                        path: path.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }
    Ok(list)
}

/// 导出任务进度
#[derive(Debug, Clone, Serialize)]
pub struct ExportProgress {
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub finished: bool,
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, files);
            } else {
                files.push(path);
            }
        }
    }
}
