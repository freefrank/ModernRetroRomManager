//! 智能匹配引擎 - 置信度评分算法
//!
//! 用于计算 ROM 文件名与搜索结果的匹配度

use crate::scraper::{ScrapeQuery, SearchResult};

/// 计算两个字符串的相似度 (Jaro-Winkler)
pub fn jaro_winkler_similarity(s1: &str, s2: &str) -> f32 {
    let jaro = jaro_similarity(s1, s2);
    
    // Winkler 修正：共同前缀加权
    let prefix_len = s1
        .chars()
        .zip(s2.chars())
        .take(4) // 最多考虑 4 个字符
        .take_while(|(a, b)| a.eq_ignore_ascii_case(b))
        .count();
    
    let winkler = jaro + (prefix_len as f32 * 0.1 * (1.0 - jaro));
    winkler.min(1.0)
}

/// 计算 Jaro 相似度
fn jaro_similarity(s1: &str, s2: &str) -> f32 {
    if s1.is_empty() && s2.is_empty() {
        return 1.0;
    }
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    let s1_chars: Vec<char> = s1.to_lowercase().chars().collect();
    let s2_chars: Vec<char> = s2.to_lowercase().chars().collect();

    let match_distance = (s1_chars.len().max(s2_chars.len()) / 2).saturating_sub(1);

    let mut s1_matches = vec![false; s1_chars.len()];
    let mut s2_matches = vec![false; s2_chars.len()];

    let mut matches = 0;
    let mut transpositions = 0;

    // 找匹配字符
    for (i, c1) in s1_chars.iter().enumerate() {
        let start = i.saturating_sub(match_distance);
        let end = (i + match_distance + 1).min(s2_chars.len());

        for j in start..end {
            if s2_matches[j] || c1 != &s2_chars[j] {
                continue;
            }
            s1_matches[i] = true;
            s2_matches[j] = true;
            matches += 1;
            break;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    // 计算转置
    let mut k = 0;
    for (i, _) in s1_chars.iter().enumerate() {
        if !s1_matches[i] {
            continue;
        }
        while !s2_matches[k] {
            k += 1;
        }
        if s1_chars[i] != s2_chars[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let matches = matches as f32;
    let transpositions = transpositions as f32 / 2.0;
    let s1_len = s1_chars.len() as f32;
    let s2_len = s2_chars.len() as f32;

    (matches / s1_len + matches / s2_len + (matches - transpositions) / matches) / 3.0
}

/// 清理游戏名称（移除常见后缀和标记）
pub fn normalize_game_name(name: &str) -> String {
    let mut result = name.to_string();
    
    // 移除括号内容 (USA), (Japan), (Rev 1), etc.
    while let Some(start) = result.find('(') {
        if let Some(end) = result[start..].find(')') {
            result = format!("{}{}", &result[..start], &result[start + end + 1..]);
        } else {
            break;
        }
    }
    
    // 移除方括号内容 [!], [T+Eng], etc.
    while let Some(start) = result.find('[') {
        if let Some(end) = result[start..].find(']') {
            result = format!("{}{}", &result[..start], &result[start + end + 1..]);
        } else {
            break;
        }
    }
    
    // 移除文件扩展名
    let extensions = [".zip", ".7z", ".rar", ".iso", ".bin", ".cue", ".nes", ".sfc", ".smc", ".gba", ".gb", ".gbc", ".n64", ".z64", ".v64", ".nds", ".3ds", ".cia", ".xci", ".nsp", ".pbp", ".chd"];
    for ext in extensions {
        if result.to_lowercase().ends_with(ext) {
            result = result[..result.len() - ext.len()].to_string();
        }
    }
    
    // 清理多余空格
    result = result.split_whitespace().collect::<Vec<_>>().join(" ");
    result.trim().to_string()
}

/// 从文件名解析游戏名
pub fn parse_game_name_from_filename(filename: &str) -> String {
    normalize_game_name(filename)
}

/// 计算搜索结果的置信度分数
pub fn calculate_confidence(query: &ScrapeQuery, result: &SearchResult) -> f32 {
    let query_name = normalize_game_name(&query.name);
    let result_name = normalize_game_name(&result.name);
    
    // 基础名称相似度 (权重 0.7)
    let name_similarity = jaro_winkler_similarity(&query_name, &result_name);
    let mut score = name_similarity * 0.7;
    
    // 完全匹配奖励
    if query_name.to_lowercase() == result_name.to_lowercase() {
        score += 0.2;
    }
    
    // 系统匹配奖励 (权重 0.1)
    if let (Some(query_sys), Some(result_sys)) = (&query.system, &result.system) {
        if query_sys.to_lowercase() == result_sys.to_lowercase() {
            score += 0.1;
        }
    }
    
    // 年份匹配可以考虑但目前 query 没有年份信息
    
    score.min(1.0)
}

/// 对搜索结果重新计算置信度并排序
pub fn rank_results(query: &ScrapeQuery, mut results: Vec<SearchResult>) -> Vec<SearchResult> {
    // 重新计算置信度
    for result in &mut results {
        result.confidence = calculate_confidence(query, result);
    }
    
    // 按置信度降序排序
    results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaro_winkler() {
        assert!(jaro_winkler_similarity("Super Mario World", "Super Mario World") > 0.99);
        assert!(jaro_winkler_similarity("Super Mario World", "super mario world") > 0.99);
        assert!(jaro_winkler_similarity("Super Mario World", "Super Mario Bros") > 0.7);
        assert!(jaro_winkler_similarity("Zelda", "Pokemon") < 0.5);
    }

    #[test]
    fn test_normalize_game_name() {
        assert_eq!(
            normalize_game_name("Super Mario World (USA).sfc"),
            "Super Mario World"
        );
        assert_eq!(
            normalize_game_name("Legend of Zelda, The (Japan) [!].zip"),
            "Legend of Zelda, The"
        );
        assert_eq!(
            normalize_game_name("Pokemon Red (USA, Europe) (Rev 1).gbc"),
            "Pokemon Red"
        );
    }

    #[test]
    fn test_calculate_confidence() {
        let query = ScrapeQuery::new(
            "Super Mario World".to_string(),
            "Super Mario World (USA).sfc".to_string(),
        );
        
        let result = SearchResult {
            provider: "test".to_string(),
            source_id: "1".to_string(),
            name: "Super Mario World".to_string(),
            year: None,
            system: None,
            thumbnail: None,
            confidence: 0.0,
        };
        
        let confidence = calculate_confidence(&query, &result);
        assert!(confidence > 0.9);
    }
}
