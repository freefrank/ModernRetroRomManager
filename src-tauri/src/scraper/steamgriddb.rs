//! SteamGridDB Provider - 高质量游戏封面/Logo/图标

use crate::scraper::rate_limit::{response_error, ProviderRateLimiter};
use crate::scraper::{
    Capabilities, GameMetadata, MediaAsset, MediaType, ProviderCapability, RomHash, ScrapeQuery,
    ScraperProvider, SearchResult,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;

const PROVIDER_ID: &str = "steamgriddb";

pub struct SteamGridDBClient {
    api_key: String,
    client: Client,
    limiter: ProviderRateLimiter,
    media_types: HashSet<String>,
}

impl SteamGridDBClient {
    pub fn new(api_key: String, media_types: &[String], rate_limit: u32, threads: u32) -> Self {
        Self {
            api_key,
            client: Client::new(),
            limiter: ProviderRateLimiter::per_second(rate_limit, threads),
            media_types: media_types.iter().cloned().collect(),
        }
    }

    async fn fetch_images(
        &self,
        endpoint: &str,
        source_id: &str,
    ) -> Result<Vec<SGDBImage>, String> {
        let url = format!(
            "https://www.steamgriddb.com/api/v2/{}/game/{}",
            endpoint, source_id
        );
        let _permit = self.limiter.acquire().await;
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(response_error("SteamGridDB", resp).await);
        }

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
        let _permit = self.limiter.acquire().await;
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(response_error("SteamGridDB", resp).await);
        }

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
                    // autocomplete 不返回平台字段，保留本次 ROM 平台作为搜索上下文，
                    // 让统一排序能优先当前平台而不是把结果当成完全未知平台。
                    system: query.system.clone(),
                    thumbnail: None,
                    asset_count: None,
                    confidence,
                }
            })
            .collect())
    }

    async fn get_metadata(&self, source_id: &str) -> Result<GameMetadata, String> {
        // SteamGridDB 主要用于媒体，元数据有限
        let url = format!("https://www.steamgriddb.com/api/v2/games/id/{}", source_id);
        let _permit = self.limiter.acquire().await;
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(response_error("SteamGridDB", resp).await);
        }

        let sgdb_resp: SGDBResponse<SGDBGame> = resp.json().await.map_err(|e| e.to_string())?;

        Ok(GameMetadata {
            name: sgdb_resp.data.name,
            english_name: None,
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
            if !self.media_types.contains(media_type.as_str()) {
                continue;
            }
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
