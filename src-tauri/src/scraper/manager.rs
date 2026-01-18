//! ScraperManager - 统一调度层
//!
//! 管理多个 ScraperProvider，提供统一的搜索、元数据获取、媒体获取接口

use std::collections::HashMap;
use std::sync::Arc;
use futures::future::join_all;

use super::{
    ScraperProvider, ScrapeQuery, SearchResult, GameMetadata, MediaAsset,
    ScrapeResult, RomHash, ProviderCapability,
};
use super::matcher::rank_results;
use crate::settings::{get_settings, update_setting, ScraperConfig};

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

/// ScraperManager - 统一调度层
pub struct ScraperManager {
    /// 已注册的 providers
    providers: HashMap<String, Arc<dyn ScraperProvider>>,
    /// Provider 配置
    configs: HashMap<String, ProviderConfig>,
}

impl ScraperManager {
    /// 创建新的 ScraperManager
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    /// 注册 provider
    pub fn register<P: ScraperProvider + 'static>(&mut self, provider: P) {
        let id = provider.id().to_string();
        self.providers.insert(id.clone(), Arc::new(provider));
        
        // 从持久化设置中加载配置，如果不存在则使用默认值
        let settings = get_settings();
        let config = if let Some(saved_config) = settings.scrapers.get(&id) {
            ProviderConfig {
                enabled: saved_config.enabled,
                priority: 100, // 暂时默认优先级
            }
        } else {
            ProviderConfig::default()
        };
        
        self.configs.insert(id, config);
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
    pub fn unregister(&mut self, id: &str) {
        self.providers.remove(id);
        self.configs.remove(id);
    }

    /// 获取所有已注册的 provider ID
    pub fn provider_ids(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// 获取已启用的 providers (按优先级排序)
    fn enabled_providers(&self) -> Vec<Arc<dyn ScraperProvider>> {
        let mut providers: Vec<_> = self
            .providers
            .iter()
            .filter(|(id, _)| {
                self.configs
                    .get(*id)
                    .map(|c| c.enabled)
                    .unwrap_or(true)
            })
            .map(|(id, p)| {
                let priority = self.configs.get(id).map(|c| c.priority).unwrap_or(100);
                (priority, Arc::clone(p))
            })
            .collect();

        providers.sort_by_key(|(priority, _)| *priority);
        providers.into_iter().map(|(_, p)| p).collect()
    }

    /// 统一搜索 - 并行查询所有启用的 providers
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

        // 合并结果
        results
            .into_iter()
            .filter_map(|r: Result<Vec<SearchResult>, String>| r.ok())
            .flatten()
            .collect()
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
    fn merge_metadata(&self, results: Vec<(String, Result<GameMetadata, String>)>) -> (GameMetadata, Vec<String>) {
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

    #[test]
    fn test_manager_creation() {
        let manager = ScraperManager::new();
        assert!(manager.provider_ids().is_empty());
    }
}
