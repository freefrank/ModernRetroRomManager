//! SteamGridDB Provider - 高质量游戏封面/Logo/图标

use crate::scraper::{
    ScraperProvider, ScrapeQuery, SearchResult, GameMetadata, MediaAsset,
    MediaType, Capabilities, ProviderCapability, RomHash,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const PROVIDER_ID: &str = "steamgriddb";
const PROVIDER_NAME: &str = "SteamGridDB";

pub struct SteamGridDBClient {
    api_key: String,
    client: Client,
}

impl SteamGridDBClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    async fn fetch_images(&self, endpoint: &str, source_id: &str) -> Result<Vec<SGDBImage>, String> {
        let url = format!(
            "https://www.steamgriddb.com/api/v2/{}/game/{}",
            endpoint, source_id
        );
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let sgdb_resp: SGDBResponse<Vec<SGDBImage>> =
            resp.json().await.map_err(|e| e.to_string())?;

        if sgdb_resp.success {
            Ok(sgdb_resp.data)
        } else {
            Ok(vec![])
        }
    }
}

// ============================================================================
// API Response Types
// ============================================================================

#[derive(Deserialize)]
struct SGDBResponse<T> {
    success: bool,
    data: T,
}

#[derive(Deserialize)]
struct SGDBGame {
    id: i64,
    name: String,
    #[serde(default)]
    release_date: Option<i64>,
}

#[derive(Deserialize)]
struct SGDBImage {
    url: String,
    width: i32,
    height: i32,
}

// ============================================================================
// 新 ScraperProvider 实现
// ============================================================================

#[async_trait]
impl ScraperProvider for SteamGridDBClient {
    fn id(&self) -> &'static str {
        PROVIDER_ID
    }

    fn display_name(&self) -> &'static str {
        PROVIDER_NAME
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::new()
            .with(ProviderCapability::Search)
            .with(ProviderCapability::Media)
        // 注意：SteamGridDB 主要提供媒体，元数据有限
    }

    async fn search(&self, query: &ScrapeQuery) -> Result<Vec<SearchResult>, String> {
        let url = format!(
            "https://www.steamgriddb.com/api/v2/search/autocomplete/{}",
            urlencoding::encode(&query.name)
        );
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let sgdb_resp: SGDBResponse<Vec<SGDBGame>> =
            resp.json().await.map_err(|e| e.to_string())?;

        if !sgdb_resp.success {
            return Err("SteamGridDB API error".to_string());
        }

        Ok(sgdb_resp
            .data
            .into_iter()
            .enumerate()
            .map(|(i, g)| {
                // 简单置信度：第一个结果置信度最高
                let confidence = 1.0 - (i as f32 * 0.1).min(0.5);
                SearchResult {
                    provider: PROVIDER_ID.to_string(),
                    source_id: g.id.to_string(),
                    name: g.name,
                    year: g.release_date.map(|ts| {
                        // Unix timestamp to year
                        let secs = ts;
                        let year = 1970 + (secs / 31536000);
                        year.to_string()
                    }),
                    system: None, // SteamGridDB 不区分系统
                    thumbnail: None,
                    confidence,
                }
            })
            .collect())
    }

    async fn get_metadata(&self, source_id: &str) -> Result<GameMetadata, String> {
        // SteamGridDB 主要用于媒体，元数据有限
        let url = format!(
            "https://www.steamgriddb.com/api/v2/games/id/{}",
            source_id
        );
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let sgdb_resp: SGDBResponse<SGDBGame> =
            resp.json().await.map_err(|e| e.to_string())?;

        Ok(GameMetadata {
            name: sgdb_resp.data.name,
            description: None,
            release_date: sgdb_resp.data.release_date.map(|ts| {
                let year = 1970 + (ts / 31536000);
                year.to_string()
            }),
            developer: None,
            publisher: None,
            genres: vec![],
            players: None,
            rating: None,
        })
    }

    async fn get_media(&self, source_id: &str) -> Result<Vec<MediaAsset>, String> {
        let mut all_media = Vec::new();

        // 端点 -> 媒体类型映射
        let endpoints = [
            ("grids", MediaType::BoxFront),
            ("heroes", MediaType::Hero),
            ("logos", MediaType::Logo),
            ("icons", MediaType::Icon),
        ];

        for (endpoint, media_type) in endpoints {
            if let Ok(images) = self.fetch_images(endpoint, source_id).await {
                for img in images {
                    all_media.push(MediaAsset {
                        provider: PROVIDER_ID.to_string(),
                        url: img.url,
                        asset_type: media_type,
                        width: Some(img.width as u32),
                        height: Some(img.height as u32),
                    });
                }
            }
        }

        Ok(all_media)
    }

    async fn lookup_by_hash(
        &self,
        _hash: &RomHash,
        _system: Option<&str>,
    ) -> Result<Option<SearchResult>, String> {
        // SteamGridDB 不支持 Hash 查找
        Ok(None)
    }
}
