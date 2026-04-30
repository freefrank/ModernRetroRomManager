//! jy6d-dz 中英文对照 CSV 读取器
//!
//! 读取 resources/cn-mapping/{system}.csv 文件，提供中英文名称映射。
//! 数据来源: http://emu.jy6d.com/dz/

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// jy6d-dz 中英文对照条目
#[derive(Debug, Clone)]
pub struct Jy6dDzEntry {
    pub english_name: String,
    pub chinese_name: String,
}

/// 从 CSV 文件加载 jy6d-dz 中英文映射
///
/// CSV 格式: english_name,chinese_name,source_id,extra_json
/// 只读取前两列
pub fn load_jy6d_csv(csv_path: &Path) -> Result<Vec<Jy6dDzEntry>, String> {
    let file = File::open(csv_path)
        .map_err(|e| format!("Failed to open jy6d CSV {}: {}", csv_path.display(), e))?;

    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut is_first_line = true;

    for line_result in reader.lines() {
        let line = line_result.map_err(|e| format!("Failed to read line: {}", e))?;

        // 跳过表头
        if is_first_line {
            is_first_line = false;
            continue;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // 简单 CSV 解析（处理可能的引号）
        let fields = parse_csv_line(line);
        if fields.len() < 2 {
            continue;
        }

        let english_name = fields[0].trim().to_string();
        let chinese_name = fields[1].trim().to_string();

        // 跳过空条目
        if english_name.is_empty() && chinese_name.is_empty() {
            continue;
        }

        entries.push(Jy6dDzEntry {
            english_name,
            chinese_name,
        });
    }

    Ok(entries)
}

/// 构建中文名 -> 英文名的映射表（用于快速查找）
pub fn build_cn_to_en_map(entries: &[Jy6dDzEntry]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for entry in entries {
        if !entry.chinese_name.is_empty() && !entry.english_name.is_empty() {
            // 用小写作为 key 提高匹配率
            map.insert(
                entry.chinese_name.to_lowercase(),
                entry.english_name.clone(),
            );
        }
    }
    map
}

/// 构建英文名 -> 中文名的映射表
pub fn build_en_to_cn_map(entries: &[Jy6dDzEntry]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for entry in entries {
        if !entry.english_name.is_empty() && !entry.chinese_name.is_empty() {
            map.insert(
                entry.english_name.to_lowercase(),
                entry.chinese_name.clone(),
            );
        }
    }
    map
}

/// 简单 CSV 行解析，支持双引号字段
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '"' {
            if in_quotes {
                // 检查是否是转义的引号
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                in_quotes = true;
            }
        } else if ch == ',' && !in_quotes {
            fields.push(current.clone());
            current.clear();
        } else {
            current.push(ch);
        }
    }

    fields.push(current);
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_line() {
        let line = r#"Hello World,你好世界,"with,comma","with ""quotes"""#;
        let fields = parse_csv_line(line);
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "Hello World");
        assert_eq!(fields[1], "你好世界");
        assert_eq!(fields[2], "with,comma");
        assert_eq!(fields[3], r#"with "quotes""#);
    }
}
