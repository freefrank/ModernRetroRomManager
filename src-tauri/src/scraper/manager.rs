//! ScraperManager - 统一调度层
//!
//! 管理多个 ScraperProvider，提供统一的搜索、元数据获取、媒体获取接口

use futures::future::join_all;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock as AsyncRwLock;

use super::{
    cache,
    matcher::{apply_asset_count_confidence, normalize_game_name, rank_results},
    screenscraper::{bundled_developer_credentials, ScreenScraperClient},
    steamgriddb::SteamGridDBClient,
    thegamesdb::TheGamesDbClient,
    GameMetadata, MediaAsset, ProviderCapability, ProviderSearchOutcome, RomHash, ScrapeQuery,
    ScrapeResult, ScraperProvider, SearchResult,
};
use crate::settings::{get_settings, update_setting, AppSettings, ScraperConfig};

fn usable_asset(asset: &MediaAsset) -> bool {
    let url = asset.url.trim();
    !url.is_empty()
        && (url.starts_with("https://") || url.starts_with("http://"))
        && asset.asset_type.as_str() != "other"
}

fn record_search_stats(provider: &str, matched: usize, cache_hit: bool, error: Option<&str>) {
    let provider = provider.to_string();
    let error = error.map(str::to_string);
    if let Err(write_error) = update_setting(move |settings| {
        let stats = settings.scraper_stats.entry(provider).or_default();
        stats.matched_count = stats.matched_count.saturating_add(matched as u64);
        if cache_hit {
            stats.cache_hit_count = stats.cache_hit_count.saturating_add(1);
        }
        if let Some(error) = error {
            stats.error_count = stats.error_count.saturating_add(1);
            stats.last_error = Some(error);
        }
    }) {
        eprintln!("[scraper] 保存 Provider 搜索统计失败: {write_error}");
    }
}

pub fn record_selected(provider: &str) {
    let provider = provider.to_string();
    if let Err(error) = update_setting(move |settings| {
        let stats = settings.scraper_stats.entry(provider).or_default();
        stats.selected_count = stats.selected_count.saturating_add(1);
    }) {
        eprintln!("[scraper] 保存 Provider 采用统计失败: {error}");
    }
}

/// Provider 配置
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// 是否启用
    pub enabled: bool,
    /// 优先级 (数字越小优先级越高)
    pub priority: u32,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            priority: 100,
        }
    }
}

/// 受支持 provider 的静态描述(全集目录,与是否已注册无关)
struct ProviderDescriptor {
    id: &'static str,
    name: &'static str,
    requires_credentials: bool,
    capabilities: &'static [&'static str],
}

/// 受支持的 provider 目录
const PROVIDER_CATALOG: &[ProviderDescriptor] = &[
    ProviderDescriptor {
        id: "thegamesdb",
        name: "TheGamesDB",
        requires_credentials: true,
        capabilities: &["search", "hash_lookup", "metadata", "media"],
    },
    ProviderDescriptor {
        id: "steamgriddb",
        name: "SteamGridDB",
        requires_credentials: true,
        capabilities: &["search", "media"],
    },
    ProviderDescriptor {
        id: "screenscraper",
        name: "ScreenScraper",
        requires_credentials: true,
        capabilities: &["search", "hash_lookup", "metadata", "media"],
    },
];

/// 面向前端的 provider 信息
#[derive(Debug, Clone, Serialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: u32,
    pub requires_credentials: bool,
    pub has_credentials: bool,
    pub configured_username: Option<String>,
    pub has_saved_password: bool,
    pub capabilities: Vec<String>,
    pub rate_limit: u32,
    pub threads: u32,
    pub matched_count: u64,
    pub selected_count: u64,
    pub cache_hit_count: u64,
    pub error_count: u64,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SearchReport {
    pub results: Vec<SearchResult>,
    pub outcomes: Vec<ProviderSearchOutcome>,
}

/// 判断给定 provider 的凭证是否已配置完整
fn credentials_present(provider_id: &str, config: Option<&ScraperConfig>) -> bool {
    let Some(config) = config else {
        return false;
    };
    match provider_id {
        "steamgriddb" => config
            .api_key
            .as_deref()
            .is_some_and(|k| !k.trim().is_empty()),
        "thegamesdb" => config
            .api_key
            .as_deref()
            .is_some_and(|k| !k.trim().is_empty()),
        "screenscraper" => {
            config
                .username
                .as_deref()
                .is_some_and(|v| !v.trim().is_empty())
                && config
                    .password
                    .as_deref()
                    .is_some_and(|v| !v.trim().is_empty())
        }
        _ => false,
    }
}

/// ScraperManager - 统一调度层
pub struct ScraperManager {
    /// 已注册的 providers
    providers: HashMap<String, Arc<dyn ScraperProvider>>,
    /// Provider 配置
    configs: HashMap<String, ProviderConfig>,
    /// Shared HTTP Client
    pub http_client: reqwest::Client,
    /// Provider 媒体清单缓存，搜索排名与详情页共用，避免重复请求。
    media_cache: AsyncRwLock<HashMap<(String, String), Vec<MediaAsset>>>,
}

impl ScraperManager {
    /// 创建新的 ScraperManager
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            configs: HashMap::new(),
            http_client: reqwest::Client::new(),
            media_cache: AsyncRwLock::new(HashMap::new()),
        }
    }

    /// 根据当前持久化设置创建 ScraperManager(注册所有凭证完整的 provider)
    pub fn from_settings() -> Self {
        let mut manager = Self::new();
        manager.rebuild_from_settings();
        manager
    }

    /// 根据当前持久化设置重建 provider 注册表(凭证/启用态即时生效)
    pub fn rebuild_from_settings(&mut self) {
        self.rebuild_from(&get_settings());
    }

    /// 根据给定设置重建 provider 注册表
    pub fn rebuild_from(&mut self, settings: &AppSettings) {
        self.providers.clear();
        self.configs.clear();
        self.media_cache.get_mut().clear();

        for descriptor in PROVIDER_CATALOG {
            let Some(config) = settings.scrapers.get(descriptor.id) else {
                continue;
            };
            if !credentials_present(descriptor.id, Some(config)) {
                continue;
            }
            let provider_config = ProviderConfig {
                enabled: config.enabled,
                priority: config.priority,
            };
            match descriptor.id {
                "steamgriddb" => {
                    if let Some(api_key) = config.api_key.clone() {
                        self.register_with_config(
                            SteamGridDBClient::new(
                                api_key,
                                &settings.scraper_media_types,
                                config.rate_limit,
                                config.threads,
                            ),
                            provider_config,
                        );
                    }
                }
                "thegamesdb" => {
                    if let Some(api_key) = config.api_key.clone() {
                        self.register_with_config(
                            TheGamesDbClient::new(
                                api_key,
                                &settings.scraper_media_types,
                                config.rate_limit,
                                config.threads,
                            ),
                            provider_config,
                        );
                    }
                }
                "screenscraper" => {
                    let (devid, devpassword) = bundled_developer_credentials();
                    self.register_with_config(
                        ScreenScraperClient::new(
                            config.username.clone().unwrap_or_default(),
                            config.password.clone().unwrap_or_default(),
                            devid,
                            devpassword,
                            config.rate_limit,
                            config.threads,
                        ),
                        provider_config,
                    );
                }
                _ => {}
            }
        }
    }

    /// 遍历受支持的 provider 目录,结合给定设置生成 provider 信息列表
    pub fn provider_infos_from(settings: &AppSettings) -> Vec<ProviderInfo> {
        let mut infos: Vec<_> = PROVIDER_CATALOG
            .iter()
            .map(|descriptor| {
                let config = settings.scrapers.get(descriptor.id);
                let stats = settings
                    .scraper_stats
                    .get(descriptor.id)
                    .cloned()
                    .unwrap_or_default();
                ProviderInfo {
                    id: descriptor.id.to_string(),
                    name: descriptor.name.to_string(),
                    enabled: config.map(|c| c.enabled).unwrap_or(true),
                    priority: config.map(|c| c.priority).unwrap_or(100),
                    requires_credentials: descriptor.requires_credentials,
                    has_credentials: credentials_present(descriptor.id, config),
                    configured_username: (descriptor.id == "screenscraper")
                        .then(|| config.and_then(|item| item.username.clone()))
                        .flatten(),
                    has_saved_password: descriptor.id == "screenscraper"
                        && config
                            .and_then(|item| item.password.as_deref())
                            .is_some_and(|value| !value.trim().is_empty()),
                    capabilities: descriptor
                        .capabilities
                        .iter()
                        .map(|c| c.to_string())
                        .collect(),
                    rate_limit: config.map(|c| c.rate_limit).unwrap_or(1),
                    threads: config.map(|c| c.threads).unwrap_or(1),
                    matched_count: stats.matched_count,
                    selected_count: stats.selected_count,
                    cache_hit_count: stats.cache_hit_count,
                    error_count: stats.error_count,
                    last_error: stats.last_error,
                }
            })
            .collect();
        // Provider 列表是前端所有抓取入口的权威顺序，不应泄漏
        // 内置目录的固定声明顺序。相同优先级保留目录顺序。
        infos.sort_by_key(|info| info.priority);
        infos
    }

    /// 基于当前持久化设置生成 provider 信息列表
    pub fn provider_infos(&self) -> Vec<ProviderInfo> {
        Self::provider_infos_from(&get_settings())
    }

    /// 注册 provider 并指定配置
    pub fn register_with_config<P: ScraperProvider + 'static>(
        &mut self,
        provider: P,
        config: ProviderConfig,
    ) {
        let id = provider.id().to_string();
        self.providers.insert(id.clone(), Arc::new(provider));
        self.configs.insert(id, config);
    }

    /// 注销 provider
    #[allow(dead_code)]
    pub fn unregister(&mut self, id: &str) {
        self.providers.remove(id);
        self.configs.remove(id);
    }

    /// 获取所有已注册的 provider ID(目前仅测试使用)
    #[cfg(test)]
    pub fn provider_ids(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// 获取已启用的 providers (按优先级排序)
    #[cfg(test)]
    fn enabled_providers(&self) -> Vec<Arc<dyn ScraperProvider>> {
        self.enabled_providers_matching(&[])
    }

    fn enabled_providers_matching(&self, provider_ids: &[String]) -> Vec<Arc<dyn ScraperProvider>> {
        let mut providers: Vec<_> = self
            .providers
            .iter()
            .filter(|(id, _)| self.configs.get(*id).map(|c| c.enabled).unwrap_or(true))
            .filter(|(id, _)| {
                provider_ids.is_empty() || provider_ids.iter().any(|value| value == *id)
            })
            .map(|(id, p)| {
                let priority = self.configs.get(id).map(|c| c.priority).unwrap_or(100);
                (priority, Arc::clone(p))
            })
            .collect();

        providers.sort_by_key(|(priority, _)| *priority);
        providers.into_iter().map(|(_, p)| p).collect()
    }

    /// 统一搜索 - 并行查询所有启用的 providers,结果经评分排序
    #[allow(dead_code)]
    pub async fn search(&self, query: &ScrapeQuery) -> Vec<SearchResult> {
        self.search_with_providers_internal(query, &[], false)
            .await
            .results
    }

    /// 用户主动点击搜索时跳过持久缓存，重新查询 Provider 并刷新缓存。
    #[allow(dead_code)]
    pub async fn search_fresh(&self, query: &ScrapeQuery) -> Vec<SearchResult> {
        self.search_fresh_report(query).await.results
    }

    pub async fn search_fresh_report(&self, query: &ScrapeQuery) -> SearchReport {
        self.search_with_providers_internal(query, &[], true).await
    }

    async fn search_with_providers_internal(
        &self,
        query: &ScrapeQuery,
        provider_ids: &[String],
        force_refresh: bool,
    ) -> SearchReport {
        let providers = self.enabled_providers_matching(provider_ids);

        // 并行查询所有 provider
        let futures: Vec<_> = providers
            .iter()
            .filter(|p| p.capabilities().has(ProviderCapability::Search))
            .map(|p| {
                let provider = Arc::clone(p);
                let provider_id = provider.id().to_string();
                let q = query.clone();
                async move {
                    if !force_refresh {
                        if let Some(results) = cache::load_search(&provider_id, &q) {
                            return (provider_id, Ok(results), true);
                        }
                    }
                    let result = provider.search(&q).await;
                    if let Ok(results) = &result {
                        cache::save_search(&provider_id, &q, results);
                    }
                    (provider_id, result, false)
                }
            })
            .collect();

        let results = join_all(futures).await;

        // 合并结果并按重算后的置信度降序排序
        let mut outcomes = Vec::with_capacity(results.len());
        let merged: Vec<SearchResult> = results
            .into_iter()
            .filter_map(|(provider, result, cache_hit)| match result {
                Ok(results) => {
                    outcomes.push(ProviderSearchOutcome {
                        provider: provider.clone(),
                        matched: results.len(),
                        cache_hit,
                        error: None,
                    });
                    record_search_stats(&provider, results.len(), cache_hit, None);
                    Some(results)
                }
                Err(error) => {
                    eprintln!("[scraper] {provider} 搜索失败: {error}");
                    outcomes.push(ProviderSearchOutcome {
                        provider: provider.clone(),
                        matched: 0,
                        cache_hit,
                        error: Some(error.clone()),
                    });
                    record_search_stats(&provider, 0, cache_hit, Some(&error));
                    None
                }
            })
            .flatten()
            .collect();

        let mut ranked = rank_results(query, merged);

        // Provider 已在搜索响应中给出资产数量时直接计入评分。
        for result in &mut ranked {
            if let Some(asset_count) = result.asset_count {
                result.confidence = apply_asset_count_confidence(result.confidence, asset_count);
            }
        }

        // 仅展开同 Provider 的同名重复候选，避免对所有搜索结果制造 N+1 请求。
        let mut duplicate_counts: HashMap<(String, String), usize> = HashMap::new();
        for result in &ranked {
            *duplicate_counts
                .entry((
                    result.provider.clone(),
                    normalize_game_name(&result.name).to_ascii_lowercase(),
                ))
                .or_default() += 1;
        }
        let media_futures: Vec<_> = ranked
            .iter()
            .enumerate()
            .filter(|(_, result)| result.asset_count.is_none())
            .filter(|(_, result)| {
                duplicate_counts
                    .get(&(
                        result.provider.clone(),
                        normalize_game_name(&result.name).to_ascii_lowercase(),
                    ))
                    .is_some_and(|count| *count > 1)
            })
            .map(|(index, result)| async move {
                (
                    index,
                    self.fetch_media_cached(&result.provider, &result.source_id)
                        .await,
                )
            })
            .collect();
        for (index, media) in join_all(media_futures).await {
            if let Ok(media) = media {
                let asset_count = media.len();
                ranked[index].asset_count = Some(asset_count);
                ranked[index].confidence =
                    apply_asset_count_confidence(ranked[index].confidence, asset_count);
            }
        }
        ranked.sort_by(|left, right| {
            right
                .confidence
                .partial_cmp(&left.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        SearchReport {
            results: ranked,
            outcomes,
        }
    }

    #[allow(dead_code)]
    pub async fn search_with_providers(
        &self,
        query: &ScrapeQuery,
        provider_ids: &[String],
    ) -> Vec<SearchResult> {
        self.search_with_providers_internal(query, provider_ids, false)
            .await
            .results
    }

    pub async fn search_with_providers_report(
        &self,
        query: &ScrapeQuery,
        provider_ids: &[String],
    ) -> SearchReport {
        self.search_with_providers_internal(query, provider_ids, false)
            .await
    }

    async fn lookup_by_hash_with_providers(
        &self,
        hash: &RomHash,
        system: Option<&str>,
        provider_ids: &[String],
    ) -> Option<SearchResult> {
        let providers = self.enabled_providers_matching(provider_ids);

        for provider in providers
            .iter()
            .filter(|p| p.capabilities().has(ProviderCapability::HashLookup))
        {
            if let Ok(Some(result)) = provider.lookup_by_hash(hash, system).await {
                return Some(result);
            }
        }

        None
    }

    /// 获取元数据 - 指定 provider
    pub async fn get_metadata(
        &self,
        provider_id: &str,
        source_id: &str,
    ) -> Result<GameMetadata, String> {
        self.fetch_metadata_cached(provider_id, source_id).await
    }

    pub async fn test_provider(&self, provider_id: &str) -> Result<String, String> {
        let provider = self
            .providers
            .get(provider_id)
            .ok_or_else(|| format!("Provider '{provider_id}' 未配置完整凭证"))?;
        provider.test_connection().await
    }

    /// 获取媒体 - 指定 provider
    pub async fn get_media(
        &self,
        provider_id: &str,
        source_id: &str,
        media_types: Option<&[String]>,
    ) -> Result<Vec<MediaAsset>, String> {
        let configured = get_settings().scraper_media_types;
        let allowed = media_types.unwrap_or(&configured);
        Ok(self
            .fetch_media_cached(provider_id, source_id)
            .await?
            .into_iter()
            .filter(|asset| {
                allowed
                    .iter()
                    .any(|value| value == asset.asset_type.as_str())
            })
            .collect())
    }

    async fn fetch_media_cached(
        &self,
        provider_id: &str,
        source_id: &str,
    ) -> Result<Vec<MediaAsset>, String> {
        let key = (provider_id.to_string(), source_id.to_string());
        if let Some(media) = self.media_cache.read().await.get(&key).cloned() {
            return Ok(media);
        }
        if let Some(media) = cache::load_media(provider_id, source_id) {
            self.media_cache.write().await.insert(key, media.clone());
            return Ok(media);
        }
        let provider = self
            .providers
            .get(provider_id)
            .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;
        let media: Vec<_> = provider
            .get_media(source_id)
            .await?
            .into_iter()
            .filter(usable_asset)
            .collect();
        cache::save_media(provider_id, source_id, &media);
        self.media_cache.write().await.insert(key, media.clone());
        Ok(media)
    }

    async fn fetch_metadata_cached(
        &self,
        provider_id: &str,
        source_id: &str,
    ) -> Result<GameMetadata, String> {
        if let Some(metadata) = cache::load_metadata(provider_id, source_id) {
            return Ok(metadata);
        }
        let provider = self
            .providers
            .get(provider_id)
            .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;
        let metadata = provider.get_metadata(source_id).await?;
        cache::save_metadata(provider_id, source_id, &metadata);
        Ok(metadata)
    }

    /// 合并多个 provider 的元数据（优先级从高到低）
    fn merge_metadata(
        &self,
        results: Vec<(String, Result<GameMetadata, String>)>,
    ) -> (GameMetadata, Vec<String>) {
        let mut merged = GameMetadata::default();
        let mut sources = Vec::new();

        // 按优先级顺序处理（providers 已排序）
        for (provider_id, result) in results {
            if let Ok(metadata) = result {
                // 记录贡献的 provider
                sources.push(provider_id);

                // 合并字段（只填充空字段，已有数据的字段保持不变）
                if merged.name.is_empty() {
                    merged.name = metadata.name;
                }
                if merged.english_name.is_none() && metadata.english_name.is_some() {
                    merged.english_name = metadata.english_name;
                }
                if merged.description.is_none() && metadata.description.is_some() {
                    merged.description = metadata.description;
                }
                if merged.release_date.is_none() && metadata.release_date.is_some() {
                    merged.release_date = metadata.release_date;
                }
                if merged.developer.is_none() && metadata.developer.is_some() {
                    merged.developer = metadata.developer;
                }
                if merged.publisher.is_none() && metadata.publisher.is_some() {
                    merged.publisher = metadata.publisher;
                }
                if merged.players.is_none() && metadata.players.is_some() {
                    merged.players = metadata.players;
                }
                if merged.rating.is_none() && metadata.rating.is_some() {
                    merged.rating = metadata.rating;
                }
                // genres 合并（去重）
                for genre in metadata.genres {
                    if !merged.genres.contains(&genre) {
                        merged.genres.push(genre);
                    }
                }
            }
        }

        (merged, sources)
    }

    /// 智能 scrape - 自动匹配 + 聚合多源数据
    pub async fn scrape(&self, query: &ScrapeQuery) -> Result<ScrapeResult, String> {
        self.scrape_with_providers(query, &[]).await
    }

    pub async fn scrape_with_providers(
        &self,
        query: &ScrapeQuery,
        provider_ids: &[String],
    ) -> Result<ScrapeResult, String> {
        // 每个 provider 必须使用自己的 source_id，不能跨平台复用 ID。
        let hash_match = if let Some(ref hash) = query.hash {
            self.lookup_by_hash_with_providers(hash, query.system.as_deref(), provider_ids)
                .await
        } else {
            None
        };
        let mut matches: HashMap<String, SearchResult> = HashMap::new();
        let search_report = self.search_with_providers_report(query, provider_ids).await;
        for result in search_report.results {
            matches.entry(result.provider.clone()).or_insert(result);
        }
        if let Some(result) = hash_match {
            matches.insert(result.provider.clone(), result);
        }
        if matches.is_empty() {
            return Err("No results found".to_string());
        }
        let providers = self.enabled_providers_matching(provider_ids);
        let metadata_futures: Vec<_> = providers
            .iter()
            .filter(|provider| provider.capabilities().has(ProviderCapability::Metadata))
            .filter_map(|provider| {
                let source_id = matches.get(provider.id())?.source_id.clone();
                let provider_id = provider.id().to_string();
                Some(async move {
                    let metadata = self.fetch_metadata_cached(&provider_id, &source_id).await;
                    (provider_id, metadata)
                })
            })
            .collect();
        let (metadata, metadata_sources) = self.merge_metadata(join_all(metadata_futures).await);

        let media_futures: Vec<_> = providers
            .iter()
            .filter(|p| p.capabilities().has(ProviderCapability::Media))
            .filter_map(|provider| {
                let source_id = matches.get(provider.id())?.source_id.clone();
                let provider_id = provider.id().to_string();
                Some(async move { self.fetch_media_cached(&provider_id, &source_id).await })
            })
            .collect();

        let media_results: Vec<Result<Vec<MediaAsset>, String>> = join_all(media_futures).await;
        let media: Vec<MediaAsset> = media_results
            .into_iter()
            .filter_map(|r: Result<Vec<MediaAsset>, String>| r.ok())
            .flatten()
            .filter(usable_asset)
            .filter(|asset| {
                get_settings()
                    .scraper_media_types
                    .iter()
                    .any(|value| value == asset.asset_type.as_str())
            })
            .collect();

        Ok(ScrapeResult {
            metadata,
            media,
            sources: metadata_sources,
            provider_outcomes: search_report.outcomes,
        })
    }

    /// 启用/禁用 provider
    pub fn set_enabled(&mut self, provider_id: &str, enabled: bool) {
        // 更新内存中的配置（如果存在）
        if let Some(config) = self.configs.get_mut(provider_id) {
            config.enabled = enabled;
        }

        // 始终持久化保存到 settings.json（即使 provider 未注册）
        // 内存状态先行:持久化失败仅记录日志,不影响本次会话的内存配置
        let provider_id_owned = provider_id.to_string();
        if let Err(e) = update_setting(move |settings| {
            let entry = settings.scrapers.entry(provider_id_owned).or_default();
            entry.enabled = enabled;
        }) {
            eprintln!("[ScraperManager] 持久化 provider '{provider_id}' 启用状态失败: {e}");
        }
    }

    /// 设置 provider 优先级
    pub fn set_priority(&mut self, provider_id: &str, priority: u32) {
        // 更新内存中的配置（如果存在）
        if let Some(config) = self.configs.get_mut(provider_id) {
            config.priority = priority;
        }

        // 始终持久化保存到 settings.json
        // 内存状态先行:持久化失败仅记录日志,不影响本次会话的内存配置
        let provider_id_owned = provider_id.to_string();
        if let Err(e) = update_setting(move |settings| {
            let entry = settings.scrapers.entry(provider_id_owned).or_default();
            entry.priority = priority;
        }) {
            eprintln!("[ScraperManager] 持久化 provider '{provider_id}' 优先级失败: {e}");
        }
    }

    /// 获取 Provider 的持久化配置 (API Key 等)
    pub fn get_credentials(&self, provider_id: &str) -> Option<ScraperConfig> {
        let settings = get_settings();
        settings.scrapers.get(provider_id).cloned()
    }

    /// 更新 Provider 的凭证配置
    pub fn set_credentials(
        &mut self,
        provider_id: &str,
        config: ScraperConfig,
    ) -> Result<(), String> {
        let provider_id_owned = provider_id.to_string();
        let config_clone = config.clone();

        update_setting(move |settings| {
            settings.scrapers.insert(provider_id_owned, config_clone);
        })
        .map_err(|error| format!("保存 {provider_id} 凭据失败: {error}"))?;

        // 同时也更新内存中的启用状态
        if let Some(mem_config) = self.configs.get_mut(provider_id) {
            mem_config.enabled = config.enabled;
        }
        Ok(())
    }
}

impl Default for ScraperManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scraper::Capabilities;
    use async_trait::async_trait;

    #[test]
    fn empty_or_invalid_media_assets_are_filtered() {
        let asset = |url: &str| MediaAsset {
            provider: "test".into(),
            url: url.into(),
            asset_type: crate::scraper::MediaType::BoxFront,
            width: None,
            height: None,
        };
        assert!(!usable_asset(&asset("")));
        assert!(!usable_asset(&asset("   ")));
        assert!(!usable_asset(&asset("file:///missing.png")));
        assert!(usable_asset(&asset("https://cdn.example/cover.png")));
    }

    #[test]
    fn test_manager_creation() {
        let manager = ScraperManager::new();
        assert!(manager.provider_ids().is_empty());
    }

    fn settings_with(entries: &[(&str, ScraperConfig)]) -> AppSettings {
        let mut settings = AppSettings::default();
        for (id, config) in entries {
            settings.scrapers.insert(id.to_string(), config.clone());
        }
        settings
    }

    #[test]
    fn provider_infos_default_settings() {
        // 未配置任何 scraper 时:目录全集列出,默认启用,无凭证
        let infos = ScraperManager::provider_infos_from(&AppSettings::default());
        assert_eq!(infos.len(), PROVIDER_CATALOG.len());
        let ids: Vec<&str> = infos.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&"steamgriddb"));
        assert!(ids.contains(&"thegamesdb"));
        assert!(ids.contains(&"screenscraper"));
        for info in &infos {
            assert!(info.enabled);
            assert!(info.requires_credentials);
            assert!(!info.has_credentials);
        }
    }

    #[test]
    fn provider_infos_follow_settings_changes() {
        // provider 列表随 settings 变化:凭证/启用态/优先级均来自 settings
        let settings = settings_with(&[
            (
                "steamgriddb",
                ScraperConfig {
                    enabled: false,
                    priority: 5,
                    api_key: Some("test-key".to_string()),
                    ..Default::default()
                },
            ),
            (
                "screenscraper",
                ScraperConfig {
                    enabled: true,
                    // 只有用户名没有密码,凭证视为不完整
                    username: Some("user".to_string()),
                    password: None,
                    ..Default::default()
                },
            ),
        ]);

        let infos = ScraperManager::provider_infos_from(&settings);
        assert_eq!(
            infos.first().map(|info| info.id.as_str()),
            Some("steamgriddb")
        );
        let sgdb = infos.iter().find(|i| i.id == "steamgriddb").unwrap();
        assert!(!sgdb.enabled);
        assert_eq!(sgdb.priority, 5);
        assert!(sgdb.has_credentials);

        let ss = infos.iter().find(|i| i.id == "screenscraper").unwrap();
        assert!(ss.enabled);
        assert!(!ss.has_credentials);
    }

    #[test]
    fn screenscraper_requires_member_credentials() {
        let member_only = ScraperConfig {
            username: Some("member".to_string()),
            password: Some("member-pass".to_string()),
            ..Default::default()
        };
        assert!(credentials_present("screenscraper", Some(&member_only)));
    }

    #[test]
    fn rebuild_registers_providers_from_credentials() {
        let mut manager = ScraperManager::new();

        // 凭证完整的 provider 注册进来
        let settings = settings_with(&[
            (
                "steamgriddb",
                ScraperConfig {
                    api_key: Some("key".to_string()),
                    ..Default::default()
                },
            ),
            (
                "screenscraper",
                ScraperConfig {
                    username: Some("user".to_string()),
                    password: Some("pass".to_string()),
                    ..Default::default()
                },
            ),
            (
                "thegamesdb",
                ScraperConfig {
                    api_key: Some("tgdb-key".to_string()),
                    ..Default::default()
                },
            ),
        ]);
        manager.rebuild_from(&settings);
        let mut ids = manager.provider_ids();
        ids.sort();
        assert_eq!(ids, vec!["screenscraper", "steamgriddb", "thegamesdb"]);

        // 保存后再次重建即时生效:清空凭证 → provider 注销
        manager.rebuild_from(&AppSettings::default());
        assert!(manager.provider_ids().is_empty());

        // 空白凭证视为未配置
        let blank = settings_with(&[(
            "steamgriddb",
            ScraperConfig {
                api_key: Some("   ".to_string()),
                ..Default::default()
            },
        )]);
        manager.rebuild_from(&blank);
        assert!(manager.provider_ids().is_empty());
    }

    #[test]
    fn rebuild_respects_enabled_flag() {
        // 禁用的 provider 仍会注册,但不参与 enabled_providers
        let settings = settings_with(&[(
            "steamgriddb",
            ScraperConfig {
                enabled: false,
                api_key: Some("key".to_string()),
                ..Default::default()
            },
        )]);
        let mut manager = ScraperManager::new();
        manager.rebuild_from(&settings);
        assert_eq!(manager.provider_ids(), vec!["steamgriddb"]);
        assert!(manager.enabled_providers().is_empty());
    }

    /// 返回固定结果的 mock provider,用于验证搜索聚合排序
    struct MockProvider {
        id: &'static str,
        results: Vec<SearchResult>,
        media_counts: HashMap<String, usize>,
    }

    #[async_trait]
    impl ScraperProvider for MockProvider {
        fn id(&self) -> &'static str {
            self.id
        }

        fn capabilities(&self) -> Capabilities {
            Capabilities::new()
                .with(ProviderCapability::Search)
                .with(ProviderCapability::Metadata)
                .with(ProviderCapability::Media)
        }

        async fn search(&self, _query: &ScrapeQuery) -> Result<Vec<SearchResult>, String> {
            Ok(self.results.clone())
        }

        async fn get_metadata(&self, source_id: &str) -> Result<GameMetadata, String> {
            Ok(GameMetadata {
                name: format!("{}-{source_id}", self.id),
                genres: vec![source_id.to_string()],
                ..Default::default()
            })
        }

        async fn get_media(&self, source_id: &str) -> Result<Vec<MediaAsset>, String> {
            Ok((0..self.media_counts.get(source_id).copied().unwrap_or(0))
                .map(|index| MediaAsset {
                    provider: self.id.into(),
                    url: format!("https://example.com/{source_id}/{index}.png"),
                    asset_type: crate::scraper::MediaType::BoxFront,
                    width: None,
                    height: None,
                })
                .collect())
        }
    }

    #[tokio::test]
    async fn search_results_are_ranked_by_confidence() {
        let make_result = |name: &str, confidence: f32| SearchResult {
            provider: "mock".to_string(),
            source_id: name.to_string(),
            name: name.to_string(),
            year: None,
            system: None,
            thumbnail: None,
            asset_count: None,
            confidence,
        };

        let mut manager = ScraperManager::new();
        manager.register_with_config(
            MockProvider {
                id: "mock",
                // 故意乱序且给低匹配度结果虚高置信度
                results: vec![
                    make_result("Zelda", 0.99),
                    make_result("Super Mario World", 0.0),
                ],
                media_counts: HashMap::new(),
            },
            ProviderConfig::default(),
        );

        let query = ScrapeQuery::new(
            "Super Mario World".to_string(),
            "Super Mario World (USA).sfc".to_string(),
        );
        let results = manager.search(&query).await;
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "Super Mario World");
        assert!(results[0].confidence >= results[1].confidence);
    }

    #[tokio::test]
    async fn scrape_uses_each_provider_own_source_id() {
        let mut manager = ScraperManager::new();
        for (id, source_id, priority) in [("first", "id-a", 10), ("second", "id-b", 20)] {
            manager.register_with_config(
                MockProvider {
                    id,
                    results: vec![SearchResult {
                        provider: id.to_string(),
                        source_id: source_id.to_string(),
                        name: "Test Game".to_string(),
                        year: None,
                        system: Some("snes".to_string()),
                        thumbnail: None,
                        asset_count: None,
                        confidence: 1.0,
                    }],
                    media_counts: HashMap::new(),
                },
                ProviderConfig {
                    enabled: true,
                    priority,
                },
            );
        }
        let result = manager
            .scrape(&ScrapeQuery::new("Test Game".into(), "test.sfc".into()).with_system("snes"))
            .await
            .unwrap();
        assert!(result.metadata.genres.contains(&"id-a".to_string()));
        assert!(result.metadata.genres.contains(&"id-b".to_string()));
    }

    #[tokio::test]
    async fn scrape_with_providers_only_uses_selected_sources() {
        let mut manager = ScraperManager::new();
        for (id, source_id) in [("first", "id-a"), ("second", "id-b")] {
            manager.register_with_config(
                MockProvider {
                    id,
                    results: vec![SearchResult {
                        provider: id.to_string(),
                        source_id: source_id.to_string(),
                        name: "Test Game".to_string(),
                        year: None,
                        system: Some("snes".to_string()),
                        thumbnail: None,
                        asset_count: None,
                        confidence: 1.0,
                    }],
                    media_counts: HashMap::new(),
                },
                ProviderConfig::default(),
            );
        }

        let result = manager
            .scrape_with_providers(
                &ScrapeQuery::new("Test Game".into(), "test.sfc".into()).with_system("snes"),
                &["second".to_string()],
            )
            .await
            .unwrap();
        assert_eq!(result.metadata.genres, vec!["id-b".to_string()]);
    }

    #[tokio::test]
    async fn duplicate_results_use_asset_count_for_ranking() {
        let result = |source_id: &str| SearchResult {
            provider: "mock".into(),
            source_id: source_id.into(),
            name: "CT Special Forces 2".into(),
            year: None,
            system: Some("GBA".into()),
            thumbnail: Some("https://example.com/thumb.png".into()),
            asset_count: None,
            confidence: 0.0,
        };
        let mut manager = ScraperManager::new();
        manager.register_with_config(
            MockProvider {
                id: "mock",
                results: vec![result("one"), result("eight")],
                media_counts: HashMap::from([("one".into(), 1), ("eight".into(), 8)]),
            },
            ProviderConfig::default(),
        );

        let query =
            ScrapeQuery::new("CT Special Forces 2".into(), "ct2.gba".into()).with_system("GBA");
        let results = manager.search(&query).await;
        assert_eq!(results[0].source_id, "eight");
        assert_eq!(results[0].asset_count, Some(8));
        assert_eq!(results[1].asset_count, Some(1));
        assert!(results[0].confidence > results[1].confidence);
    }
}
