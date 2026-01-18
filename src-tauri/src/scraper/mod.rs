pub mod steamgriddb;
pub mod screenscraper;
pub mod pegasus;
pub mod types;
pub mod manager;
pub mod matcher;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// 重新导出类型
pub use types::*;

// ============================================================================
// 旧类型 (保留兼容性，后续逐步迁移)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedGame {
    pub source_id: String,
    pub name: String,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genres: Vec<String>,
    pub rating: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedMedia {
    pub url: String,
    pub asset_type: String, // boxfront, boxback, screenshot, logo, icon, hero, video
    pub width: Option<i32>,
    pub height: Option<i32>,
}

// ============================================================================
// 新 Provider Trait
// ============================================================================

/// Scraper Provider trait - 可扩展接口
#[async_trait]
pub trait ScraperProvider: Send + Sync {
    /// Provider 唯一标识符
    fn id(&self) -> &'static str;

    /// Provider 显示名称
    fn display_name(&self) -> &'static str;

    /// Provider 支持的能力
    fn capabilities(&self) -> Capabilities;

    /// 搜索游戏
    async fn search(&self, query: &ScrapeQuery) -> Result<Vec<SearchResult>, String>;

    /// 获取游戏详细元数据
    async fn get_metadata(&self, source_id: &str) -> Result<GameMetadata, String>;

    /// 获取媒体资产
    async fn get_media(&self, source_id: &str) -> Result<Vec<MediaAsset>, String>;

    /// 通过 Hash 精确查找（可选实现）
    async fn lookup_by_hash(&self, _hash: &RomHash, _system: Option<&str>) -> Result<Option<SearchResult>, String> {
        Ok(None) // 默认不支持
    }
}

// ============================================================================
// 旧 Trait (保留兼容性)
// ============================================================================

#[async_trait]
pub trait Scraper: Send + Sync {
    fn name(&self) -> &'static str;
    async fn search(&self, query: &str) -> Result<Vec<ScrapedGame>, String>;
    async fn get_details(&self, source_id: &str) -> Result<ScrapedGame, String>;
    async fn get_media(&self, source_id: &str) -> Result<Vec<ScrapedMedia>, String>;
}
