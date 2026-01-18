//! 本地中文 ROM 数据库 Provider
//! 
//! 基于 yingw/rom-name-cn 仓库的 CSV 文件提供中文名称查找

use crate::scraper::{
    ScraperProvider, ScrapeQuery, SearchResult, GameMetadata, MediaAsset,
    Capabilities, ProviderCapability,
};
use crate::scraper::cn_repo::{find_csv_in_dir, read_csv, CnRomEntry};
use crate::scraper::matcher::jaro_winkler_similarity;
use crate::config::get_data_dir;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::PathBuf;

const PROVIDER_ID: &str = "local_cn_repo";
const PROVIDER_NAME: &str = "Chinese ROM DB (Local)";

/// 缓存 CSV 数据，避免频繁 IO
/// Key: System Name, Value: List of Entries
type CsvCache = HashMap<String, Vec<CnRomEntry>>;

pub struct LocalCnProvider {
    cache: Arc<Mutex<CsvCache>>,
    search_paths: Vec<PathBuf>,
}

impl LocalCnProvider {
    /// 创建新的 LocalCnProvider
    /// 
    /// # Arguments
    /// * `extra_paths` - 额外的搜索路径 (例如 bundled resources)，优先级低于用户数据目录
    pub fn new(extra_paths: Vec<PathBuf>) -> Self {
        // 默认总是包含用户数据目录下的 rom-name-cn
        let user_repo_path = get_data_dir().join("rom-name-cn");
        
        let mut search_paths = vec![user_repo_path];
        search_paths.extend(extra_paths);

        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            search_paths,
        }
    }

    /// 获取或加载系统对应的 CSV 数据
    fn get_entries(&self, system: &str) -> Vec<CnRomEntry> {
        let mut cache = self.cache.lock().unwrap();
        
        if let Some(entries) = cache.get(system) {
            return entries.iter().map(|e| CnRomEntry {
                english_name: e.english_name.clone(),
                chinese_name: e.chinese_name.clone(),
            }).collect();
        }

        // 遍历所有搜索路径
        for root_path in &self.search_paths {
            if let Some(path) = find_csv_in_dir(root_path, system) {
                if let Ok(entries) = read_csv(&path) {
                    cache.insert(system.to_string(), entries);
                    return cache.get(system).unwrap().iter().map(|e| CnRomEntry {
                        english_name: e.english_name.clone(),
                        chinese_name: e.chinese_name.clone(),
                    }).collect();
                }
            }
        }

        vec![]
    }
}

#[async_trait]
impl ScraperProvider for LocalCnProvider {
    fn id(&self) -> &'static str {
        PROVIDER_ID
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::new()
            .with(ProviderCapability::Search)
            .with(ProviderCapability::Metadata) // 仅提供名称
    }

    async fn search(&self, query: &ScrapeQuery) -> Result<Vec<SearchResult>, String> {
        // 移除 is_repo_ready 检查，改由 get_entries 内部处理
        let system = query.system.as_deref().unwrap_or("unknown");
        let entries = self.get_entries(system);

        if entries.is_empty() {
            return Ok(vec![]);
        }

        let mut results = Vec::new();
        let query_name_lower = query.name.to_lowercase();
        // 如果有文件名，尝试清理扩展名
        let query_file_stem = std::path::Path::new(&query.file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&query.name)
            .to_lowercase();

        for entry in entries {
            let english_lower = entry.english_name.to_lowercase();
            let chinese_lower = entry.chinese_name.to_lowercase();

            // 1. 精确匹配
            if english_lower == query_name_lower || english_lower == query_file_stem {
                results.push(SearchResult {
                    provider: PROVIDER_ID.to_string(),
                    source_id: entry.english_name.clone(), // 使用英文名作为 ID
                    name: entry.chinese_name.clone(),
                    year: None,
                    system: Some(system.to_string()),
                    thumbnail: None,
                    confidence: 1.0,
                });
                continue;
            }

            // 2. 模糊匹配 (Jaro-Winkler)
            // 比较 query.name 和 CSV 中的 English Name
            let score_en = jaro_winkler_similarity(&query_name_lower, &english_lower);
            // 比较 query.file_name 和 CSV 中的 English Name
            let score_file = jaro_winkler_similarity(&query_file_stem, &english_lower);
            
            // 同时也比较中文名（万一用户搜索的是中文）
            let score_cn = jaro_winkler_similarity(&query_name_lower, &chinese_lower);

            let max_score = score_en.max(score_file).max(score_cn);

            if max_score > 0.85 { // 阈值可调
                results.push(SearchResult {
                    provider: PROVIDER_ID.to_string(),
                    source_id: entry.english_name.clone(),
                    name: entry.chinese_name.clone(),
                    year: None,
                    system: Some(system.to_string()),
                    thumbnail: None,
                    confidence: max_score,
                });
            }
        }

        // 按置信度排序
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        // 限制返回数量
        Ok(results.into_iter().take(5).collect())
    }

    async fn get_metadata(&self, source_id: &str) -> Result<GameMetadata, String> {
        // 由于我们把 english_name 作为 source_id，我们可以直接尝试查找
        // 但我们需要 system context，而 get_metadata 接口没有 system 参数
        // 这是一个设计上的小缺陷，不过我们可以遍历缓存或者...
        // 简单起见，我们假定 source_id 包含了足够的信息，或者我们返回最基本的 Metadata
        // 在 LocalCnProvider 中，source_id 就是英文原名，我们实际上应该返回中文名作为 name
        
        // 由于无法直接定位到 system，我们可能需要搜索所有缓存，或者暂时只返回 name
        // 考虑到 search 已经返回了中文名，get_metadata 主要是为了详情页
        // 这里我们可以尝试再次进行一次全局查找（如果缓存中有）
        
        let cache = self.cache.lock().unwrap();
        for entries in cache.values() {
            if let Some(entry) = entries.iter().find(|e| e.english_name == source_id) {
                return Ok(GameMetadata {
                    name: entry.chinese_name.clone(),
                    description: Some(format!("中文名称: {}", entry.chinese_name)), // 简单的描述
                    release_date: None,
                    developer: None,
                    publisher: None,
                    genres: vec![],
                    players: None,
                    rating: None,
                });
            }
        }

        // 如果找不到，直接返回 source_id 作为 name (虽然不太可能发生，因为是 search 结果来的)
        Err("Metadata not found".to_string())
    }

    async fn get_media(&self, _source_id: &str) -> Result<Vec<MediaAsset>, String> {
        Ok(vec![]) // 不提供媒体
    }
}
