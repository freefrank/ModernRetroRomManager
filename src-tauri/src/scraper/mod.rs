pub mod cache;
pub mod cartridge_header;
pub mod cn_repo;
pub mod dat_hash;
pub mod gba_header;
pub mod jy6d_dz;
pub mod local_cn;
pub mod manager;
pub mod matcher;
pub mod pegasus;
pub mod persistence;
pub mod platform_header;
pub mod rate_limit;
pub mod screenscraper;
pub mod steamgriddb;
pub mod thegamesdb;
pub mod three_ds_header;
pub mod types;

use async_trait::async_trait;

// 重新导出类型
pub use types::*;

/// Scraper Provider trait - 可扩展接口
#[async_trait]
pub trait ScraperProvider: Send + Sync {
    /// Provider 唯一标识符
    fn id(&self) -> &'static str;

    /// Provider 支持的能力
    fn capabilities(&self) -> Capabilities;

    /// 搜索游戏
    async fn search(&self, query: &ScrapeQuery) -> Result<Vec<SearchResult>, String>;

    /// 测试 Provider 凭据与网络连接。
    async fn test_connection(&self) -> Result<String, String> {
        let query = ScrapeQuery::new(
            "Super Mario World".to_string(),
            "Super Mario World.sfc".to_string(),
        )
        .with_system("snes");
        let results = self.search(&query).await?;
        Ok(format!("连接正常，返回 {} 个结果", results.len()))
    }

    /// 获取游戏详细元数据
    async fn get_metadata(&self, source_id: &str) -> Result<GameMetadata, String>;

    /// 获取媒体资产
    async fn get_media(&self, source_id: &str) -> Result<Vec<MediaAsset>, String>;

    /// 通过 Hash 精确查找（可选实现）
    async fn lookup_by_hash(
        &self,
        _hash: &RomHash,
        _system: Option<&str>,
    ) -> Result<Option<SearchResult>, String> {
        Ok(None) // 默认不支持
    }
}
