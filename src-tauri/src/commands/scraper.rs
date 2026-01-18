//! Scraper Tauri Commands
//!
//! 前端调用的 Scraper 相关命令

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::fs;
use tauri::{State, Emitter};
use tokio::sync::RwLock;
use crate::config::get_temp_dir;
use crate::scraper::{
    manager::ScraperManager,
    types::{ScrapeQuery, SearchResult, GameMetadata, MediaAsset, ScrapeResult},
    steamgriddb::SteamGridDBClient,
    screenscraper::ScreenScraperClient,
    pegasus::parse_pegasus_file, // Add parser import
    persistence::{download_media, save_metadata_pegasus, save_metadata_emulationstation},
};
use crate::rom_service::RomInfo;


use crate::settings::ScraperConfig;

// ============================================================================
// State - ScraperManager 全局状态
// ============================================================================

pub struct ScraperState {
    pub manager: Arc<RwLock<ScraperManager>>,
}

impl ScraperState {
    pub fn new() -> Self {
        let manager = ScraperManager::new();

        Self {
            manager: Arc::new(RwLock::new(manager)),
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

/// 智能 scrape - 自动匹配并聚合数据
#[tauri::command]
pub async fn scraper_auto_scrape(
    state: State<'_, ScraperState>,
    name: String,
    file_name: String,
    system: Option<String>,
) -> Result<ScrapeResult, String> {
    let manager = state.manager.read().await;
    
    let mut query = ScrapeQuery::new(name, file_name);
    if let Some(sys) = system {
        query = query.with_system(sys);
    }
    
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
    _state: State<'_, ScraperState>,
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
        download_media(&rom, &options.selected_media, true).await?;
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
    let manager_arc = Arc::clone(&state.manager);
    let total = rom_ids.len();

    tokio::spawn(async move {
        for (i, file_name) in rom_ids.into_iter().enumerate() {
            let current = i + 1;
            
            let _ = app.emit("batch-scrape-progress", BatchProgress {
                current,
                total,
                message: format!("正在抓取: {}", file_name),
                finished: false,
            });

            let query = ScrapeQuery::new(file_name.clone(), file_name.clone()).with_system(system.clone());
            
            let scrape_res = {
                let manager = manager_arc.read().await;
                manager.scrape(&query).await
            };

            if let Ok(result) = scrape_res {
                let rom = RomInfo {
                    file: file_name.clone(),
                    name: result.metadata.name.clone(),
                    system: system.clone(),
                    directory: directory.clone(),
                    ..Default::default()
                };
                let _ = save_metadata_pegasus(&rom, &result.metadata, true);
            }
        }

        let _ = app.emit("batch-scrape-progress", BatchProgress {
            current: total,
            total,
            message: "批量处理完成".to_string(),
            finished: true,
        });
    });

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
    asset_type: String,
) -> Result<(), String> {
    let file_stem = Path::new(&rom_id)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&rom_id);

    let media_dir = get_temp_dir()
        .join("media")
        .join(&system)
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
) -> Result<Vec<TempMediaInfo>, String> {
    let file_stem = Path::new(&rom_id)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&rom_id);

    let media_dir = get_temp_dir()
        .join("media")
        .join(&system)
        .join(file_stem);

    let mut list = Vec::new();
    if media_dir.exists() {
        for entry in fs::read_dir(media_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_file() {
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
