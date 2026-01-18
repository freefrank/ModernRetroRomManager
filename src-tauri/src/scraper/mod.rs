pub mod steamgriddb;
pub mod screenscraper;
pub mod pegasus;
pub mod types;
pub mod manager;
pub mod matcher;
pub mod persistence;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// 重新导出类型
pub use types::*;

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
