//! Scraper Tauri Commands
//!
//! 前端调用的 Scraper 相关命令

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::Path;
use std::fs;
use tauri::{State, Emitter};
use tokio::sync::RwLock;
use crate::config::get_temp_dir;
use crate::scraper::{
    manager::ScraperManager,
    types::{ScrapeQuery, SearchResult, GameMetadata, MediaAsset, ScrapeResult},
    steamgriddb::SteamGridDBClient,
    screenscraper::ScreenScraperClient,
    persistence::{download_media, save_metadata_pegasus},
};
use crate::rom_service::RomInfo;


// ============================================================================
// State - ScraperManager 全局状态
// ============================================================================

pub struct ScraperState {
    pub manager: Arc<RwLock<ScraperManager>>,
}

impl ScraperState {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(RwLock::new(ScraperManager::new())),
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
    
    // 目前返回硬编码的 provider 信息
    // TODO: 从 manager 动态获取
    let providers = vec![
        ProviderInfo {
            id: "steamgriddb".to_string(),
            name: "SteamGridDB".to_string(),
            enabled: provider_ids.contains(&"steamgriddb".to_string()),
            has_credentials: false,
            capabilities: vec!["search".to_string(), "media".to_string()],
        },
        ProviderInfo {
            id: "screenscraper".to_string(),
            name: "ScreenScraper".to_string(),
            enabled: provider_ids.contains(&"screenscraper".to_string()),
            has_credentials: false,
            capabilities: vec![
                "search".to_string(),
                "hash_lookup".to_string(),
                "metadata".to_string(),
                "media".to_string(),
            ],
        },
    ];
    
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
    
    match provider_id.as_str() {
        "steamgriddb" => {
            if let Some(api_key) = credentials.api_key {
                if !api_key.is_empty() {
                    let client = SteamGridDBClient::new(api_key);
                    manager.register(client);
                }
            }
        }
        "screenscraper" => {
            if let (Some(username), Some(password)) = (credentials.username, credentials.password) {
                if !username.is_empty() && !password.is_empty() {
                    let client = ScreenScraperClient::new(username, password);
                    manager.register(client);
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
pub async fn export_scraped_data(
    system: String,
    directory: String,
) -> Result<(), String> {
    let temp_metadata_path = get_temp_dir().join(&system).join("metadata.txt");
    if !temp_metadata_path.exists() {
        return Err("No temporary data to export".to_string());
    }

    // 1. 读取临时元数据
    let content = fs::read_to_string(&temp_metadata_path).map_err(|e| e.to_string())?;
    
    // 2. 写入到目标目录 (Pegasus 模式)
    let target_path = Path::new(&directory).join("metadata.txt");
    
    // 如果目标文件存在，我们执行合并/追加逻辑
    // 简单起见，目前直接追加
    let mut target_content = if target_path.exists() {
        fs::read_to_string(&target_path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    target_content.push_str("\n# Exported from ModernRetroRomManager\n");
    target_content.push_str(&content);

    fs::write(&target_path, target_content).map_err(|e| e.to_string())?;

    // 3. 移动媒体文件
    let temp_media_dir = get_temp_dir().join("media").join(&system);
    let target_media_dir = Path::new(&directory).join("media");

    if temp_media_dir.exists() {
        fs::create_dir_all(&target_media_dir).map_err(|e| e.to_string())?;
        // 简单移动整个系统目录下的媒体
        // 这里可以使用更精细的按文件移动逻辑
        copy_dir_recursive(&temp_media_dir, &target_media_dir)?;
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if !dst.exists() {
        fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    }

    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name())).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
