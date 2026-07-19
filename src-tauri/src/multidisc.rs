// 骨架阶段:检测原语先落地,分组/m3u/导出接线后移除本 allow。
#![allow(dead_code)]
//! 多碟游戏检测:从 ROM 文件/文件夹名解析碟号,把同一游戏的多张碟归为一组。
//!
//! 支持的碟号标记(大小写不敏感):
//! - `(Disc 1)` / `(Disk 1)` / `(CD 1)`     —— 括号内数字
//! - `(Disc A)` / `Disc B`(文件夹后缀)     —— 字母 A/B/C… → 1/2/3…
//! - `CD1` / `CD2阴之章`                      —— CD 紧跟数字(后可接文本)
//! - `碟1` / `第1碟` / `第一张`               —— 中文碟号

/// 解析结果:去掉碟号标记后的**游戏基名**与**碟序**(从 1 起)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscMarker {
    pub base: String,
    pub disc: u32,
}

fn cn_numeral(s: &str) -> Option<u32> {
    match s {
        "一" => Some(1),
        "二" => Some(2),
        "三" => Some(3),
        "四" => Some(4),
        "五" => Some(5),
        "六" => Some(6),
        "七" => Some(7),
        "八" => Some(8),
        "九" => Some(9),
        "十" => Some(10),
        _ => None,
    }
}

/// 把 A/B/C… 映射为 1/2/3…(仅单个 ASCII 字母)。
fn letter_index(s: &str) -> Option<u32> {
    let mut chars = s.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    if c.is_ascii_alphabetic() {
        Some((c.to_ascii_uppercase() as u32) - ('A' as u32) + 1)
    } else {
        None
    }
}

/// 从单个名字段(文件名去扩展名,或文件夹名)提取碟号标记。
/// 命中返回去标记后的基名(已 trim)与碟序;未命中返回 None。
pub fn parse_disc_marker(name: &str) -> Option<DiscMarker> {
    let trimmed = name.trim();
    // 依次尝试各类标记;正则用手写扫描以避免额外依赖。
    if let Some(m) = match_parenthetical(trimmed) {
        return Some(m);
    }
    if let Some(m) = match_cd_prefix(trimmed) {
        return Some(m);
    }
    if let Some(m) = match_trailing_disc(trimmed) {
        return Some(m);
    }
    if let Some(m) = match_chinese(trimmed) {
        return Some(m);
    }
    None
}

/// `(Disc 1)` / `(Disk A)` / `(CD 2)` —— 括号包裹。
fn match_parenthetical(name: &str) -> Option<DiscMarker> {
    let lower = name.to_lowercase();
    for kw in ["disc", "disk", "cd"] {
        let mut from = 0;
        while let Some(rel) = lower[from..].find(&format!("({kw}")) {
            let open = from + rel;
            // 找到对应右括号
            let close_rel = name[open..].find(')')?;
            let close = open + close_rel;
            let inner = name[open + 1 + kw.len()..close].trim(); // kw 后到 ) 前
            if let Some(disc) = inner.parse::<u32>().ok().or_else(|| letter_index(inner)) {
                let mut base = String::new();
                base.push_str(name[..open].trim_end());
                base.push_str(name[close + 1..].trim_start());
                return Some(DiscMarker {
                    base: base.trim().to_string(),
                    disc,
                });
            }
            from = close + 1;
        }
    }
    None
}

/// 结尾裸标记:`... Disc A` / `...Disc 2` / `... Disk B`(无括号,常见于文件夹名)。
fn match_trailing_disc(name: &str) -> Option<DiscMarker> {
    let lower = name.to_lowercase();
    for kw in ["disc", "disk"] {
        if let Some(pos) = lower.rfind(kw) {
            let after = name[pos + kw.len()..].trim();
            if let Some(disc) = after.parse::<u32>().ok().or_else(|| letter_index(after)) {
                let base = name[..pos].trim().to_string();
                if !base.is_empty() {
                    return Some(DiscMarker { base, disc });
                }
            }
        }
    }
    None
}

/// `CD1` / `CD2阴之章` —— CD 紧跟数字,数字后可接任意文本(章节名)。
fn match_cd_prefix(name: &str) -> Option<DiscMarker> {
    let lower = name.to_lowercase();
    let mut from = 0;
    while let Some(rel) = lower[from..].find("cd") {
        let pos = from + rel;
        let rest = &name[pos + 2..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            if let Ok(disc) = digits.parse::<u32>() {
                // 去掉 "CD<digits>" 及其后紧跟的章节文本(到下一个分隔或结尾)。
                let tail = &name[pos + 2 + digits.len()..];
                let base = name[..pos].trim().to_string();
                // base 为空说明 CD 号在最前,一般不是多碟游戏名,跳过。
                if !base.is_empty() {
                    let _ = tail;
                    return Some(DiscMarker { base, disc });
                }
            }
        }
        from = pos + 2;
    }
    None
}

/// 中文:`碟1` / `第1碟` / `第一张` / `第二碟`。
fn match_chinese(name: &str) -> Option<DiscMarker> {
    // 第X张/第X碟
    if let Some(pos) = name.find('第') {
        let rest = &name[pos + '第'.len_utf8()..];
        let mut chars = rest.chars();
        if let Some(c) = chars.next() {
            let num = c.to_digit(10).or_else(|| cn_numeral(&c.to_string()));
            if let Some(disc) = num {
                let next: String = chars.clone().take(1).collect();
                if next == "张" || next == "碟" || next == "盘" {
                    let base = name[..pos].trim().to_string();
                    if !base.is_empty() {
                        return Some(DiscMarker { base, disc });
                    }
                }
            }
        }
    }
    // 碟X(结尾)
    if let Some(pos) = name.rfind('碟') {
        let rest = &name[pos + '碟'.len_utf8()..];
        if let Some(c) = rest.chars().next() {
            if let Some(disc) = c.to_digit(10).or_else(|| cn_numeral(&c.to_string())) {
                let base = name[..pos].trim().to_string();
                if !base.is_empty() {
                    return Some(DiscMarker { base, disc });
                }
            }
        }
    }
    None
}

/// 一张碟在多碟游戏中的条目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscEntry {
    /// 碟序(从 1 起)。
    pub disc: u32,
    /// 该碟的代表文件(cue/描述文件等),相对平台目录、正斜杠。
    pub file: String,
}

/// 一个多碟游戏组。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiDiscGame {
    /// 去碟号后的游戏基名(用于 m3u 文件名/子文件夹名)。
    pub base: String,
    /// 按碟序排序的各碟。
    pub discs: Vec<DiscEntry>,
}

/// 从单个(已单碟去重的)代表文件路径解析 (分组键, 碟序)。
/// 优先父文件夹的碟号标记(folder-per-disc),否则退回文件名标记(flat)。
fn disc_key(file: &str) -> Option<(String, u32)> {
    let norm = file.replace('\\', "/");
    let segs: Vec<&str> = norm.split('/').filter(|s| !s.is_empty()).collect();
    if segs.is_empty() {
        return None;
    }
    // 1) 父文件夹标记(每碟一个文件夹)
    if segs.len() >= 2 {
        let folder = segs[segs.len() - 2];
        if let Some(m) = parse_disc_marker(folder) {
            let prefix = segs[..segs.len() - 2].join("/");
            let key = if prefix.is_empty() {
                m.base
            } else {
                format!("{prefix}/{}", m.base)
            };
            return Some((key, m.disc));
        }
    }
    // 2) 文件名标记(平铺)
    let filename = segs[segs.len() - 1];
    let stem = filename
        .rsplit_once('.')
        .map(|(s, _)| s)
        .unwrap_or(filename);
    if let Some(m) = parse_disc_marker(stem) {
        let prefix = segs[..segs.len() - 1].join("/");
        let key = if prefix.is_empty() {
            m.base
        } else {
            format!("{prefix}/{}", m.base)
        };
        return Some((key, m.disc));
    }
    None
}

/// 从代表文件列表中识别多碟游戏组(仅返回碟数 ≥2 的组)。
/// 调用方应只对光盘平台的文件调用本函数。
pub fn detect_multidisc_groups(files: &[String]) -> Vec<MultiDiscGame> {
    use std::collections::BTreeMap;
    // 保持稳定顺序:按基名排序;组内按碟序排序、同碟号去重(保留首个)。
    let mut groups: BTreeMap<String, Vec<DiscEntry>> = BTreeMap::new();
    for file in files {
        if let Some((key, disc)) = disc_key(file) {
            let entry = DiscEntry {
                disc,
                file: file.replace('\\', "/"),
            };
            groups.entry(key).or_default().push(entry);
        }
    }
    let mut result = Vec::new();
    for (base, mut discs) in groups {
        discs.sort_by(|a, b| a.disc.cmp(&b.disc).then_with(|| a.file.cmp(&b.file)));
        discs.dedup_by_key(|d| d.disc);
        if discs.len() >= 2 {
            result.push(MultiDiscGame { base, discs });
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(base: &str, disc: u32) -> Option<DiscMarker> {
        Some(DiscMarker {
            base: base.to_string(),
            disc,
        })
    }

    #[test]
    fn parses_parenthetical_numeric_and_letter() {
        assert_eq!(
            parse_disc_marker("Final Fantasy VII (Disc 1)"),
            mk("Final Fantasy VII", 1)
        );
        assert_eq!(
            parse_disc_marker("D no Shokutaku (Japan) (Disc 3)"),
            mk("D no Shokutaku (Japan)", 3)
        );
        assert_eq!(parse_disc_marker("Game (Disk B)"), mk("Game", 2));
        assert_eq!(parse_disc_marker("Game (CD 2)"), mk("Game", 2));
    }

    #[test]
    fn parses_trailing_folder_style() {
        assert_eq!(
            parse_disc_marker("世纪末吸血鬼[简][xjsxjs197]Disc A"),
            mk("世纪末吸血鬼[简][xjsxjs197]", 1)
        );
        assert_eq!(parse_disc_marker("My Game Disc C"), mk("My Game", 3));
    }

    #[test]
    fn parses_cd_prefix_with_chapter_text() {
        assert_eq!(
            parse_disc_marker("东京魔人学园 剑风帖CD1阳之章"),
            mk("东京魔人学园 剑风帖", 1)
        );
        assert_eq!(
            parse_disc_marker("东京魔人学园 剑风帖CD2阴之章"),
            mk("东京魔人学园 剑风帖", 2)
        );
    }

    #[test]
    fn parses_chinese_markers() {
        assert_eq!(parse_disc_marker("某游戏第1张"), mk("某游戏", 1));
        assert_eq!(parse_disc_marker("某游戏第二碟"), mk("某游戏", 2));
        assert_eq!(parse_disc_marker("某游戏碟3"), mk("某游戏", 3));
    }

    #[test]
    fn ignores_non_disc_names() {
        assert_eq!(parse_disc_marker("三国志2 (简) (v20120913)"), None);
        assert_eq!(parse_disc_marker("Sonic CD"), None); // CD 无数字
        assert_eq!(parse_disc_marker("Discovery"), None); // 非独立 disc 标记
    }

    #[test]
    fn groups_folder_per_disc_with_inner_disc_marker() {
        // D之食卓:文件夹 Disc A/B/C,内层文件名又带 (Disc N)——应按文件夹分同一组。
        let files = vec![
            "D之食卓[简]Disc A/D no Shokutaku (Disc 1).cue".to_string(),
            "D之食卓[简]Disc B/D no Shokutaku (Disc 2).cue".to_string(),
            "D之食卓[简]Disc C/D no Shokutaku (Disc 3).cue".to_string(),
        ];
        let groups = detect_multidisc_groups(&files);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].base, "D之食卓[简]");
        assert_eq!(
            groups[0].discs.iter().map(|d| d.disc).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn groups_folder_per_disc_simple() {
        let files = vec![
            "世纪末吸血鬼[简][xjsxjs197]Disc A/世纪末吸血鬼A.cue".to_string(),
            "世纪末吸血鬼[简][xjsxjs197]Disc B/世纪末吸血鬼B.cue".to_string(),
        ];
        let groups = detect_multidisc_groups(&files);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].base, "世纪末吸血鬼[简][xjsxjs197]");
        assert_eq!(groups[0].discs.len(), 2);
    }

    #[test]
    fn groups_cd_chapter_folders() {
        let files = vec![
            "东京魔人学园CD1阳之章/game1.cue".to_string(),
            "东京魔人学园CD2阴之章/game2.cue".to_string(),
        ];
        let groups = detect_multidisc_groups(&files);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].base, "东京魔人学园");
    }

    #[test]
    fn groups_flat_disc_files() {
        let files = vec![
            "Final Fantasy VII (Disc 1).chd".to_string(),
            "Final Fantasy VII (Disc 2).chd".to_string(),
            "Final Fantasy VII (Disc 3).chd".to_string(),
        ];
        let groups = detect_multidisc_groups(&files);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].base, "Final Fantasy VII");
        assert_eq!(groups[0].discs.len(), 3);
    }

    #[test]
    fn single_disc_and_unrelated_not_grouped() {
        let files = vec![
            "单碟游戏/game.cue".to_string(),
            "三国志2.md".to_string(),
            "孤单 (Disc 1).cue".to_string(), // 只有一碟,不成组
        ];
        assert!(detect_multidisc_groups(&files).is_empty());
    }
}
