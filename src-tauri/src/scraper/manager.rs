//! ScraperManager - 统一调度层
//!
//! 管理多个 ScraperProvider，提供统一的搜索、元数据获取、媒体获取接口

use futures::future::join_all;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

use super::{
    matcher::rank_results, screenscraper::ScreenScraperClient, steamgriddb::SteamGridDBClient,
    GameMetadata, MediaAsset, ProviderCapability, RomHash, ScrapeQuery, ScrapeResult,
    ScraperProvider, SearchResult,
};
use crate::settings::{get_settings, update_setting, AppSettings, ScraperConfig};

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
    pub capabilities: Vec<String>,
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
        "screenscraper" => {
            config
                .username
                .as_deref()
                .is_some_and(|u| !u.trim().is_empty())
                && config
                    .password
                    .as_deref()
                    .is_some_and(|p| !p.trim().is_empty())
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
}

impl ScraperManager {
    /// 创建新的 ScraperManager
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            configs: HashMap::new(),
            http_client: reqwest::Client::new(),
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
                        self.register_with_config(SteamGridDBClient::new(api_key), provider_config);
                    }
                }
                "screenscraper" => {
                    if let (Some(username), Some(password)) =
                        (config.username.clone(), config.password.clone())
                    {
                        self.register_with_config(
                            ScreenScraperClient::new(username, password),
                            provider_config,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    /// 遍历受支持的 provider 目录,结合给定设置生成 provider 信息列表
    pub fn provider_infos_from(settings: &AppSettings) -> Vec<ProviderInfo> {
        PROVIDER_CATALOG
            .iter()
            .map(|descriptor| {
                let config = settings.scrapers.get(descriptor.id);
                ProviderInfo {
                    id: descriptor.id.to_string(),
                    name: descriptor.name.to_string(),
                    enabled: config.map(|c| c.enabled).unwrap_or(true),
                    priority: config.map(|c| c.priority).unwrap_or(100),
                    requires_credentials: descriptor.requires_credentials,
                    has_credentials: credentials_present(descriptor.id, config),
                    capabilities: descriptor
                        .capabilities
                        .iter()
                        .map(|c| c.to_string())
                        .collect(),
                }
            })
            .collect()
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
    fn enabled_providers(&self) -> Vec<Arc<dyn ScraperProvider>> {
        let mut providers: Vec<_> = self
            .providers
            .iter()
            .filter(|(id, _)| self.configs.get(*id).map(|c| c.enabled).unwrap_or(true))
            .map(|(id, p)| {
                let priority = self.configs.get(id).map(|c| c.priority).unwrap_or(100);
                (priority, Arc::clone(p))
            })
            .collect();

        providers.sort_by_key(|(priority, _)| *priority);
        providers.into_iter().map(|(_, p)| p).collect()
    }

    /// 统一搜索 - 并行查询所有启用的 providers,结果经评分排序
    pub async fn search(&self, query: &ScrapeQuery) -> Vec<SearchResult> {
        let providers = self.enabled_providers();

        // 并行查询所有 provider
        let futures: Vec<_> = providers
            .iter()
            .filter(|p| p.capabilities().has(ProviderCapability::Search))
            .map(|p| {
                let provider = Arc::clone(p);
                let q = query.clone();
                async move { provider.search(&q).await }
            })
            .collect();

        let results: Vec<Result<Vec<SearchResult>, String>> = join_all(futures).await;

        // 合并结果并按重算后的置信度降序排序
        let merged: Vec<SearchResult> = results
            .into_iter()
            .filter_map(|r: Result<Vec<SearchResult>, String>| r.ok())
            .flatten()
            .collect();

        rank_results(query, merged)
    }

    /// 通过 Hash 精确查找
    pub async fn lookup_by_hash(
        &self,
        hash: &RomHash,
        system: Option<&str>,
    ) -> Option<SearchResult> {
        let providers = self.enabled_providers();

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
        let provider = self
            .providers
            .get(provider_id)
            .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;

        provider.get_metadata(source_id).await
    }

    /// 获取媒体 - 指定 provider
    pub async fn get_media(
        &self,
        provider_id: &str,
        source_id: &str,
    ) -> Result<Vec<MediaAsset>, String> {
        let provider = self
            .providers
            .get(provider_id)
            .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;

        provider.get_media(source_id).await
    }

    /// 聚合多个 provider 的元数据（按优先级合并）
    async fn aggregate_metadata(&self, best_match: &SearchResult) -> (GameMetadata, Vec<String>) {
        let providers = self.enabled_providers();

        // 并行获取所有支持元数据的 provider 的数据
        let futures: Vec<_> = providers
            .iter()
            .filter(|p| p.capabilities().has(ProviderCapability::Metadata))
            .map(|p| {
                let provider = Arc::clone(p);
                let source_id = best_match.source_id.clone();
                let provider_id = p.id().to_string();
                async move {
                    let result = provider.get_metadata(&source_id).await;
                    (provider_id, result)
                }
            })
            .collect();

        let results = join_all(futures).await;

        // 合并元数据（按优先级，providers 已排序）
        self.merge_metadata(results)
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
        // 1. 先尝试 Hash 精确匹配
        let best_match = if let Some(ref hash) = query.hash {
            self.lookup_by_hash(hash, query.system.as_deref()).await
        } else {
            None
        };

        // 2. 如果没有 Hash 匹配，进行名称搜索
        let best_match = match best_match {
            Some(m) => m,
            None => {
                let results = self.search(query).await;
                if results.is_empty() {
                    return Err("No results found".to_string());
                }
                // 选择置信度最高的结果
                results
                    .into_iter()
                    .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
                    .ok_or_else(|| "No results found".to_string())?
            }
        };

        // 3. 聚合多个 provider 的元数据
        let (metadata, metadata_sources) = self.aggregate_metadata(&best_match).await;

        // 4. 并行获取所有 provider 的媒体
        let providers = self.enabled_providers();
        let media_futures: Vec<_> = providers
            .iter()
            .filter(|p| p.capabilities().has(ProviderCapability::Media))
            .map(|p| {
                let provider = Arc::clone(p);
                let source_id = best_match.source_id.clone();
                async move { provider.get_media(&source_id).await }
            })
            .collect();

        let media_results: Vec<Result<Vec<MediaAsset>, String>> = join_all(media_futures).await;
        let media: Vec<MediaAsset> = media_results
            .into_iter()
            .filter_map(|r: Result<Vec<MediaAsset>, String>| r.ok())
            .flatten()
            .collect();

        Ok(ScrapeResult {
            metadata,
            media,
            sources: metadata_sources,
        })
    }

    /// 更新 provider 配置
    #[allow(dead_code)]
    pub fn set_config(&mut self, provider_id: &str, config: ProviderConfig) {
        if self.providers.contains_key(provider_id) {
            self.configs.insert(provider_id.to_string(), config);
        }
    }

    /// 启用/禁用 provider
    pub fn set_enabled(&mut self, provider_id: &str, enabled: bool) {
        // 更新内存中的配置（如果存在）
        if let Some(config) = self.configs.get_mut(provider_id) {
            config.enabled = enabled;
        }

        // 始终持久化保存到 settings.json（即使 provider 未注册）
        let provider_id_owned = provider_id.to_string();
        let _ = update_setting(move |settings| {
            let entry = settings.scrapers.entry(provider_id_owned).or_default();
            entry.enabled = enabled;
        });
    }

    /// 设置 provider 优先级
    pub fn set_priority(&mut self, provider_id: &str, priority: u32) {
        // 更新内存中的配置（如果存在）
        if let Some(config) = self.configs.get_mut(provider_id) {
            config.priority = priority;
        }

        // 始终持久化保存到 settings.json
        let provider_id_owned = provider_id.to_string();
        let _ = update_setting(move |settings| {
            let entry = settings.scrapers.entry(provider_id_owned).or_default();
            entry.priority = priority;
        });
    }

    /// 获取 Provider 的持久化配置 (API Key 等)
    pub fn get_credentials(&self, provider_id: &str) -> Option<ScraperConfig> {
        let settings = get_settings();
        settings.scrapers.get(provider_id).cloned()
    }

    /// 更新 Provider 的凭证配置
    pub fn set_credentials(&mut self, provider_id: &str, config: ScraperConfig) {
        let provider_id_owned = provider_id.to_string();
        let config_clone = config.clone();

        let _ = update_setting(move |settings| {
            settings.scrapers.insert(provider_id_owned, config_clone);
        });

        // 同时也更新内存中的启用状态
        if let Some(mem_config) = self.configs.get_mut(provider_id) {
            mem_config.enabled = config.enabled;
        }
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
        let sgdb = infos.iter().find(|i| i.id == "steamgriddb").unwrap();
        assert!(!sgdb.enabled);
        assert_eq!(sgdb.priority, 5);
        assert!(sgdb.has_credentials);

        let ss = infos.iter().find(|i| i.id == "screenscraper").unwrap();
        assert!(ss.enabled);
        assert!(!ss.has_credentials);
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
        ]);
        manager.rebuild_from(&settings);
        let mut ids = manager.provider_ids();
        ids.sort();
        assert_eq!(ids, vec!["screenscraper", "steamgriddb"]);

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
        results: Vec<SearchResult>,
    }

    #[async_trait]
    impl ScraperProvider for MockProvider {
        fn id(&self) -> &'static str {
            "mock"
        }

        fn capabilities(&self) -> Capabilities {
            Capabilities::new().with(ProviderCapability::Search)
        }

        async fn search(&self, _query: &ScrapeQuery) -> Result<Vec<SearchResult>, String> {
            Ok(self.results.clone())
        }

        async fn get_metadata(&self, _source_id: &str) -> Result<GameMetadata, String> {
            Err("不支持".to_string())
        }

        async fn get_media(&self, _source_id: &str) -> Result<Vec<MediaAsset>, String> {
            Ok(vec![])
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
            confidence,
        };

        let mut manager = ScraperManager::new();
        manager.register_with_config(
            MockProvider {
                // 故意乱序且给低匹配度结果虚高置信度
                results: vec![
                    make_result("Zelda", 0.99),
                    make_result("Super Mario World", 0.0),
                ],
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
}
