//! 智能匹配引擎 - 置信度评分算法
//!
//! 用于计算 ROM 文件名与搜索结果的匹配度

use crate::scraper::{ScrapeQuery, SearchResult};

fn canonical_system(system: &str) -> String {
    match system
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-', '_'], "")
        .as_str()
    {
        "3" | "n64" | "nintendo64" => "n64",
        "5" | "gba" | "gameboyadvance" => "gba",
        "6" | "sfc" | "snes" | "superfamicom" => "sfc",
        "7" | "nes" | "fc" | "famicom" => "fc",
        "8" | "nds" | "nintendods" => "nds",
        "gb" | "gameboy" => "gb",
        "gbc" | "gameboycolor" => "gbc",
        "4912" | "3ds" | "nintendo3ds" => "3ds",
        "18" | "md" | "genesis" | "megadrive" => "md",
        "ps" | "ps1" | "playstation" => "ps",
        "psp" | "playstationportable" => "psp",
        "ss" | "saturn" | "segasaturn" => "ss",
        "dc" | "dreamcast" => "dc",
        value => value,
    }
    .to_string()
}

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
    let extensions = [
        ".zip", ".7z", ".rar", ".iso", ".bin", ".cue", ".nes", ".sfc", ".smc", ".gba", ".gb",
        ".gbc", ".n64", ".z64", ".v64", ".nds", ".3ds", ".cia", ".xci", ".nsp", ".pbp", ".chd",
    ];
    for ext in extensions {
        if result.to_lowercase().ends_with(ext) {
            result = result[..result.len() - ext.len()].to_string();
        }
    }

    // 清理多余空格
    result = result.split_whitespace().collect::<Vec<_>>().join(" ");
    result.trim().to_string()
}

fn installment_token_number(token: &str) -> Option<u8> {
    if !token.starts_with('0') {
        if let Ok(number) = token.parse::<u8>() {
            if (2..=20).contains(&number) {
                return Some(number);
            }
        }
    }
    match token.to_ascii_uppercase().as_str() {
        "II" => Some(2),
        "III" => Some(3),
        "IV" => Some(4),
        "V" => Some(5),
        "VI" => Some(6),
        "VII" => Some(7),
        "VIII" => Some(8),
        "IX" => Some(9),
        "X" => Some(10),
        _ => None,
    }
}

fn sequel_identity(name: &str) -> Option<(u8, String)> {
    let tokens: Vec<_> = name
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect();
    let (position, number) = tokens
        .iter()
        .enumerate()
        // 标题开头的“2 Games in 1”“007”不是续作标识。
        .skip(1)
        .find_map(|(position, token)| {
            installment_token_number(token).map(|number| (position, number))
        })?;
    let stem = tokens
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != position)
        .flat_map(|(_, token)| token.chars())
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    Some((number, stem))
}

fn compact_name(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// 计算搜索结果的置信度分数
pub fn calculate_confidence(query: &ScrapeQuery, result: &SearchResult) -> f32 {
    let query_name = normalize_game_name(&query.name);
    let result_name = normalize_game_name(&result.name);

    // 名称仍是主要信号，但主机平台是排除同名移植版和无关媒体条目的强信号。
    let name_similarity = jaro_winkler_similarity(&query_name, &result_name);
    let mut score = name_similarity * 0.55;

    // 完全匹配奖励
    if query_name.to_lowercase() == result_name.to_lowercase() {
        score += 0.15;
    }

    // 续作编号是强身份信号：二代不能与无编号的一代或其他续作获得相同评分。
    match (sequel_identity(&query_name), sequel_identity(&result_name)) {
        (Some((query_number, query_stem)), Some((result_number, result_stem)))
            if query_stem == result_stem && query_number == result_number =>
        {
            score += 0.1
        }
        (Some((_, query_stem)), Some((_, result_stem))) if query_stem == result_stem => {
            score -= 0.4
        }
        (Some((_, query_stem)), None) if query_stem == compact_name(&result_name) => score -= 0.3,
        (None, Some((_, result_stem))) if compact_name(&query_name) == result_stem => score -= 0.3,
        _ => {}
    }

    // 平台一致奖励 30%；明确冲突则扣 40%，让已确认的当前主机平台显著优先。
    if let (Some(query_sys), Some(result_sys)) = (&query.system, &result.system) {
        if canonical_system(query_sys) == canonical_system(result_sys) {
            score += 0.3;
        } else {
            score -= 0.4;
        }
    }

    // 无可用封面预览的条目降低 15%，优先展示可以直接确认和应用的结果。
    if result
        .thumbnail
        .as_deref()
        .is_none_or(|thumbnail| thumbnail.trim().is_empty())
    {
        score -= 0.15;
    }

    // 年份匹配可以考虑但目前 query 没有年份信息

    score.clamp(0.0, 1.0)
}

/// 资产完整度占最终评分的 10%，8 个及以上可用资产计满。
pub fn apply_asset_count_confidence(base_confidence: f32, asset_count: usize) -> f32 {
    let asset_quality = asset_count.min(8) as f32 / 8.0;
    (base_confidence * 0.9 + asset_quality * 0.1).clamp(0.0, 1.0)
}

/// 对搜索结果重新计算置信度并排序
pub fn rank_results(query: &ScrapeQuery, mut results: Vec<SearchResult>) -> Vec<SearchResult> {
    // 重新计算置信度
    for result in &mut results {
        result.confidence = calculate_confidence(query, result);
    }

    // 按置信度降序排序
    results.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

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
    fn test_rank_results_orders_by_recomputed_confidence() {
        let query = ScrapeQuery::new(
            "Super Mario World".to_string(),
            "Super Mario World (USA).sfc".to_string(),
        );

        let make_result = |name: &str, confidence: f32| SearchResult {
            provider: "test".to_string(),
            source_id: name.to_string(),
            name: name.to_string(),
            year: None,
            system: None,
            thumbnail: None,
            asset_count: None,
            confidence,
        };

        // 故意给低匹配度结果一个虚高的初始 confidence,验证 rank_results 会重算并排序
        let results = vec![
            make_result("Zelda", 0.99),
            make_result("Super Mario World", 0.0),
            make_result("Super Mario Bros", 0.5),
        ];

        let ranked = rank_results(&query, results);
        assert_eq!(ranked[0].name, "Super Mario World");
        assert_eq!(ranked[2].name, "Zelda");
        // 置信度已重算且降序排列
        assert!(ranked[0].confidence >= 0.54);
        assert!(ranked[0].confidence >= ranked[1].confidence);
        assert!(ranked[1].confidence >= ranked[2].confidence);
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
            asset_count: None,
            confidence: 0.0,
        };

        let confidence = calculate_confidence(&query, &result);
        // 无平台和封面信息时保守降权，避免盖过信息完整的结果。
        assert!((confidence - 0.55).abs() < 0.001);
    }

    #[test]
    fn platform_aliases_and_numeric_ids_receive_stronger_weight() {
        let query = ScrapeQuery::new("Sonic".into(), "Sonic.zip".into()).with_system("MD");
        let result = |system: Option<&str>| SearchResult {
            provider: "test".into(),
            source_id: "1".into(),
            name: "Sonic".into(),
            year: None,
            system: system.map(str::to_string),
            thumbnail: None,
            asset_count: None,
            confidence: 0.0,
        };
        assert!((calculate_confidence(&query, &result(Some("18"))) - 0.85).abs() < 0.001);
        assert!((calculate_confidence(&query, &result(Some("Genesis"))) - 0.85).abs() < 0.001);
        assert!(calculate_confidence(&query, &result(Some("SNES"))) < 0.2);
        assert!(calculate_confidence(&query, &result(None)) < 0.7);
    }

    #[test]
    fn missing_or_blank_boxart_reduces_confidence() {
        let query = ScrapeQuery::new("Metroid Fusion".into(), "Metroid Fusion.gba".into())
            .with_system("GBA");
        let result = |thumbnail: Option<&str>| SearchResult {
            provider: "test".into(),
            source_id: "1".into(),
            name: "Metroid Fusion".into(),
            year: None,
            system: Some("GBA".into()),
            thumbnail: thumbnail.map(str::to_string),
            asset_count: None,
            confidence: 0.0,
        };

        let with_boxart =
            calculate_confidence(&query, &result(Some("https://example.com/box.png")));
        let without_boxart = calculate_confidence(&query, &result(None));
        let blank_boxart = calculate_confidence(&query, &result(Some("  ")));

        assert!((with_boxart - without_boxart - 0.15).abs() < 0.001);
        assert!((without_boxart - blank_boxart).abs() < 0.001);
    }

    #[test]
    fn sequel_number_distinguishes_ct_special_forces_entries() {
        let make_result = |name: &str| SearchResult {
            provider: "test".into(),
            source_id: name.into(),
            name: name.into(),
            year: None,
            system: Some("GBA".into()),
            thumbnail: Some("https://example.com/box.png".into()),
            asset_count: None,
            confidence: 0.0,
        };
        let sequel_query =
            ScrapeQuery::new("CT Special Forces 2".into(), "ct2.gba".into()).with_system("GBA");
        let original_query =
            ScrapeQuery::new("CT Special Forces".into(), "ct.gba".into()).with_system("GBA");

        let original = make_result("CT Special Forces");
        let sequel = make_result("CT Special Forces 2");
        assert!(
            calculate_confidence(&sequel_query, &sequel)
                > calculate_confidence(&sequel_query, &original) + 0.2
        );
        assert!(
            calculate_confidence(&original_query, &original)
                > calculate_confidence(&original_query, &sequel) + 0.2
        );
    }

    #[test]
    fn sequel_detection_ignores_leading_numbers_and_regional_subtitles() {
        assert_eq!(sequel_identity("007 - Everything or Nothing"), None);
        assert_eq!(sequel_identity("2 Games in 1 - Sonic Advance"), None);

        let query = ScrapeQuery::new("Luigis Mansion: Dark Moon".into(), "luigi.3ds".into())
            .with_system("3DS");
        let result = SearchResult {
            provider: "test".into(),
            source_id: "luigi-2".into(),
            name: "Luigis Mansion 2".into(),
            year: None,
            system: Some("3DS".into()),
            thumbnail: Some("https://example.com/box.png".into()),
            asset_count: None,
            confidence: 0.0,
        };
        let expected_without_sequel_penalty =
            jaro_winkler_similarity("Luigis Mansion: Dark Moon", "Luigis Mansion 2") * 0.55 + 0.3;
        assert!(
            (calculate_confidence(&query, &result) - expected_without_sequel_penalty).abs() < 0.001
        );
    }

    #[test]
    fn asset_count_breaks_equal_confidence_ties_without_saturation() {
        let one_asset = apply_asset_count_confidence(1.0, 1);
        let eight_assets = apply_asset_count_confidence(1.0, 8);
        assert!(eight_assets > one_asset);
        assert!((one_asset - 0.9125).abs() < 0.001);
        assert!((eight_assets - 1.0).abs() < 0.001);
    }
}
