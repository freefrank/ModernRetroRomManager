//! Scraper Tauri Commands
//!
//! 前端调用的 Scraper 相关命令

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

use crate::scraper::{
    manager::ScraperManager,
    types::{ScrapeQuery, SearchResult, GameMetadata, MediaAsset, ScrapeResult},
    steamgriddb::SteamGridDBClient,
    screenscraper::ScreenScraperClient,
};

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

#[tauri::command]
pub async fn apply_scraped_data(_rom_id: String) -> Result<(), String> {
    // TODO: 实现保存 scrape 数据到 metadata 文件
    Ok(())
}

#[tauri::command]
pub async fn batch_scrape(_rom_ids: Vec<String>, _provider_id: String) -> Result<(), String> {
    // TODO: 实现批量 scrape
    Ok(())
}
