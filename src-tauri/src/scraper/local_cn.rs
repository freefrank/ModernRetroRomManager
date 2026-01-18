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
use pinyin::ToPinyin;

const PROVIDER_ID: &str = "local_cn_repo";
const PROVIDER_NAME: &str = "Chinese ROM DB (Local)";

/// 从字符串中提取所有数字
fn extract_numbers(s: &str) -> Vec<String> {
    let mut numbers = Vec::new();
    let mut current_num = String::new();

    for c in s.chars() {
        if c.is_ascii_digit() {
            current_num.push(c);
        } else if !current_num.is_empty() {
            numbers.push(current_num.clone());
            current_num.clear();
        }
    }

    if !current_num.is_empty() {
        numbers.push(current_num);
    }

    numbers
}

/// 移除字符串中的所有数字，用于核心名称比较
fn remove_numbers(s: &str) -> String {
    s.chars().filter(|c| !c.is_ascii_digit()).collect()
}

/// 提取英文单词的首字母缩写（如 OriginalGeneration -> OG）
fn extract_abbreviation(s: &str) -> String {
    let mut abbr = String::new();
    let mut prev_was_lower = false;

    for c in s.chars() {
        if c.is_ascii_uppercase() {
            abbr.push(c);
            prev_was_lower = false;
        } else if c.is_ascii_lowercase() {
            // 如果前一个不是小写（即这是一个新单词的开始）
            if !prev_was_lower && abbr.is_empty() {
                abbr.push(c.to_ascii_uppercase());
            }
            prev_was_lower = true;
        } else {
            prev_was_lower = false;
        }
    }

    abbr
}

/// 移除字符串中的所有英文字母
fn remove_english(s: &str) -> String {
    s.chars().filter(|c| !c.is_ascii_alphabetic()).collect()
}

/// 提取字符串中的所有英文部分
fn extract_english(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_alphabetic()).collect()
}

/// 将中文字符串转换为拼音（不带声调）
fn to_pinyin_string(s: &str) -> String {
    s.chars()
        .map(|c| {
            if let Some(pinyin) = c.to_pinyin() {
                pinyin.plain().to_string()
            } else {
                c.to_string()
            }
        })
        .collect()
}

/// 计算字符集合的 Jaccard 相似度（词序无关）
fn jaccard_char_similarity(s1: &str, s2: &str) -> f32 {
    use std::collections::HashSet;

    let chars1: HashSet<char> = s1.chars().filter(|c| !c.is_whitespace()).collect();
    let chars2: HashSet<char> = s2.chars().filter(|c| !c.is_whitespace()).collect();

    if chars1.is_empty() && chars2.is_empty() {
        return 1.0;
    }

    let intersection = chars1.intersection(&chars2).count();
    let union = chars1.union(&chars2).count();

    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// 计算拼音的 Jaccard 相似度
fn pinyin_jaccard_similarity(s1: &str, s2: &str) -> f32 {
    let py1 = to_pinyin_string(s1);
    let py2 = to_pinyin_string(s2);

    // 按拼音音节分割（每个汉字对应一个拼音）
    use std::collections::HashSet;

    // 简单地用字符集合比较拼音
    let chars1: HashSet<char> = py1.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    let chars2: HashSet<char> = py2.chars().filter(|c| c.is_ascii_alphabetic()).collect();

    if chars1.is_empty() && chars2.is_empty() {
        return 1.0;
    }

    let intersection = chars1.intersection(&chars2).count();
    let union = chars1.union(&chars2).count();

    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// 常见同音字/异体字映射
fn normalize_homophones(s: &str) -> String {
    s.replace('非', "菲")
        .replace('飛', "飞")
        .replace('極', "极")
        .replace('戰', "战")
        .replace('傳', "传")
        .replace('説', "说")
        .replace('説', "说")
        .replace('機', "机")
        .replace('車', "车")
        .replace('東', "东")
        .replace('島', "岛")
        .replace('國', "国")
        .replace('龍', "龙")
        .replace('馬', "马")
        .replace('電', "电")
        .replace('記', "记")
        .replace('時', "时")
        .replace('開', "开")
        .replace('門', "门")
        .replace('見', "见")
        .replace('長', "长")
        .replace('無', "无")
        .replace('為', "为")
        .replace('從', "从")
        .replace('頭', "头")
        .replace('實', "实")
        .replace('樂', "乐")
}

/// 移除常见前缀（如"神游"）
fn remove_common_prefixes(s: &str) -> String {
    let prefixes = ["神游", "神遊", "ique", "iQue"];
    let mut result = s.to_string();
    for prefix in prefixes {
        if result.to_lowercase().starts_with(&prefix.to_lowercase()) {
            result = result[prefix.len()..].trim_start().to_string();
            break;
        }
    }
    result
}

/// 计算两个中文名的智能相似度
/// 考虑数字匹配、Jaccard 字符集合、拼音匹配、英文缩写等
pub fn smart_cn_similarity(query: &str, target: &str) -> f32 {
    // 0. 预处理：移除常见前缀、标准化同音字
    let query_clean = remove_common_prefixes(query);
    let target_clean = remove_common_prefixes(target);

    let query_normalized = normalize_homophones(&query_clean.to_lowercase());
    let target_normalized = normalize_homophones(&target_clean.to_lowercase());

    // 1. 提取数字并比较
    let query_nums = extract_numbers(&query_normalized);
    let target_nums = extract_numbers(&target_normalized);

    // 如果都有数字，检查数字是否匹配
    let num_match = if !query_nums.is_empty() && !target_nums.is_empty() {
        query_nums.iter().any(|qn| target_nums.contains(qn))
    } else {
        true
    };

    // 如果数字不匹配，直接返回很低的分数
    if !num_match {
        return 0.0;
    }

    // 2. 提取英文部分并比较
    let query_eng = extract_english(&query_normalized);
    let target_eng = extract_english(&target_normalized);

    // 检查英文缩写匹配（如 OriginalGeneration -> OG）
    let query_abbr = extract_abbreviation(&query_clean);
    let eng_abbr_match = if !query_eng.is_empty() && !target_eng.is_empty() {
        // 完全匹配或缩写匹配
        query_eng.to_lowercase() == target_eng.to_lowercase()
            || query_abbr.to_lowercase() == target_eng.to_lowercase()
            || query_eng.to_lowercase() == extract_abbreviation(&target_clean).to_lowercase()
    } else {
        false
    };

    // 3. 移除数字和英文后比较中文核心名称
    let query_cn_core = remove_english(&remove_numbers(&query_normalized));
    let target_cn_core = remove_english(&remove_numbers(&target_normalized));

    // 4. 计算多种相似度
    // 4.1 中文部分的 Jaccard 字符集合相似度（词序无关）
    let jaccard_score = jaccard_char_similarity(&query_cn_core, &target_cn_core);

    // 4.2 拼音 Jaccard 相似度（处理同音字）
    let pinyin_score = pinyin_jaccard_similarity(&query_cn_core, &target_cn_core);

    // 4.3 基础 Jaro-Winkler 相似度
    let jw_score = jaro_winkler_similarity(&query_cn_core, &target_cn_core);

    // 4.4 检查是否为子串关系
    let is_substring = query_cn_core.contains(&target_cn_core) || target_cn_core.contains(&query_cn_core);

    // 5. 综合评分
    let mut final_score = jaccard_score.max(pinyin_score).max(jw_score);

    // 如果英文缩写匹配且中文相似度不错，大幅提升分数
    if eng_abbr_match && final_score > 0.5 {
        final_score = final_score.max(0.95);
    }

    // 如果是子串关系，提升分数
    if is_substring && final_score > 0.5 {
        final_score = final_score.max(0.9);
    }

    // 如果 Jaccard 或拼音相似度很高，确保分数足够
    if jaccard_score > 0.8 || pinyin_score > 0.8 {
        final_score = final_score.max(0.85);
    }

    final_score
}

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
        eprintln!("[LocalCnProvider] Searching in {} paths", self.search_paths.len());
        for root_path in &self.search_paths {
            eprintln!("[LocalCnProvider] Checking path: {:?}", root_path);
            if let Some(path) = find_csv_in_dir(root_path, system) {
                eprintln!("[LocalCnProvider] Found CSV at: {:?}", path);
                if let Ok(entries) = read_csv(&path) {
                    eprintln!("[LocalCnProvider] Successfully loaded {} entries", entries.len());
                    cache.insert(system.to_string(), entries);
                    return cache.get(system).unwrap().iter().map(|e| CnRomEntry {
                        english_name: e.english_name.clone(),
                        chinese_name: e.chinese_name.clone(),
                    }).collect();
                } else {
                    eprintln!("[LocalCnProvider] Failed to read CSV");
                }
            } else {
                eprintln!("[LocalCnProvider] No CSV found for system: {}", system);
            }
        }

        eprintln!("[LocalCnProvider] No entries found after checking all paths");
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

        eprintln!("[LocalCnProvider] Searching for system: {}", system);

        let entries = self.get_entries(system);

        eprintln!("[LocalCnProvider] Loaded {} entries from CSV", entries.len());

        if entries.is_empty() {
            eprintln!("[LocalCnProvider] No entries found for system: {}", system);
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

            // 1. 精确匹配 - 优先匹配中文名
            if chinese_lower == query_name_lower || chinese_lower == query_file_stem {
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

            // 也支持英文名精确匹配
            if english_lower == query_name_lower || english_lower == query_file_stem {
                results.push(SearchResult {
                    provider: PROVIDER_ID.to_string(),
                    source_id: entry.english_name.clone(),
                    name: entry.chinese_name.clone(),
                    year: None,
                    system: Some(system.to_string()),
                    thumbnail: None,
                    confidence: 1.0,
                });
                continue;
            }

            // 2. 智能模糊匹配
            // 使用 smart_cn_similarity 处理数字位置不同、关键词相似等情况
            let score_cn_smart = smart_cn_similarity(&query_name_lower, &chinese_lower);
            let score_cn_file_smart = smart_cn_similarity(&query_file_stem, &chinese_lower);

            // 也支持英文名匹配（基础 Jaro-Winkler）
            let score_en = jaro_winkler_similarity(&query_name_lower, &english_lower);
            let score_file = jaro_winkler_similarity(&query_file_stem, &english_lower);

            // 处理英文名中的括号 (e.g. "Game Name (USA)" -> "Game Name")
            let english_clean = if let Some(idx) = english_lower.find('(') {
                english_lower[..idx].trim()
            } else {
                &english_lower
            };
            let score_en_clean = jaro_winkler_similarity(&query_name_lower, english_clean);

            // 取最高分（中文智能匹配优先）
            let final_score = score_cn_smart
                .max(score_cn_file_smart)
                .max(score_en)
                .max(score_file)
                .max(score_en_clean);

            if final_score > 0.75 { // 阈值 0.75
                results.push(SearchResult {
                    provider: PROVIDER_ID.to_string(),
                    source_id: entry.english_name.clone(),
                    name: entry.chinese_name.clone(),
                    year: None,
                    system: Some(system.to_string()),
                    thumbnail: None,
                    confidence: final_score,
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
                    english_name: Some(entry.english_name.clone()),
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
