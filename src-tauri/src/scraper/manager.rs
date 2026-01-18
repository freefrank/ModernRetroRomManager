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
        self.configs.insert(id, ProviderConfig::default());
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

        // 3. 获取元数据
        let metadata = self
            .get_metadata(&best_match.provider, &best_match.source_id)
            .await
            .unwrap_or_default();

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
            sources: vec![best_match.provider],
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
        if let Some(config) = self.configs.get_mut(provider_id) {
            config.enabled = enabled;
        }
    }

    /// 设置 provider 优先级
    pub fn set_priority(&mut self, provider_id: &str, priority: u32) {
        if let Some(config) = self.configs.get_mut(provider_id) {
            config.priority = priority;
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
