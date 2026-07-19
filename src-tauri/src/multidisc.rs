//! 多碟游戏检测与整理:从 ROM 文件/文件夹名解析碟号,把同一游戏的多张碟归为一组,
//! 并在保存时物理整理源库(各碟塞进 `<基名>/` 子文件夹、外层写 `<基名>.m3u`)、
//! 把临时元数据里的各碟折叠成一条指向 m3u 的记录。
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
        // 同碟号有多个文件(cue + bin/img/sub 等载荷轨)时,保留可加载的描述文件作为代表。
        discs.sort_by(|a, b| {
            a.disc
                .cmp(&b.disc)
                .then_with(|| ext_rank(&a.file).cmp(&ext_rank(&b.file)))
                .then_with(|| a.file.cmp(&b.file))
        });
        discs.dedup_by_key(|d| d.disc);
        if discs.len() >= 2 {
            result.push(MultiDiscGame { base, discs });
        }
    }
    result
}

/// 光盘代表文件优先级:可直接加载的描述文件优先,载荷/子码轨最次。
fn ext_rank(file: &str) -> u8 {
    let ext = file.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "cue" => 1,
        "chd" => 2,
        "gdi" => 3,
        "iso" => 4,
        "pbp" => 5,
        "ccd" => 6,
        "mds" => 7,
        "cdi" => 8,
        "bin" | "img" | "mdf" => 20,
        "sub" => 30,
        _ => 15,
    }
}

// ============================================================================
// 物理整理 + 元数据折叠
// ============================================================================

use crate::scraper::pegasus::PegasusGame;
use std::path::Path;

/// 多碟整理报告(供命令层序列化返回)。
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct OrganizeReport {
    /// 是否发生改动(有多碟组被整理)。
    pub changed: bool,
    /// 整理的多碟游戏组数。
    pub groups: usize,
    /// 物理移动的顶层条目数(文件夹或平铺文件)。
    pub files_moved: usize,
    /// 写出的 m3u 文件数。
    pub m3u_written: usize,
}

/// m3u 文件相对平台目录的路径 = `<基名>.m3u`。
fn m3u_relpath(base: &str) -> String {
    format!("{base}.m3u")
}

/// 某碟文件搬入 `<基名>/` 后的新相对路径(保留原顶层条目名)。
fn reorg_target(base: &str, disc_file: &str) -> String {
    format!("{base}/{}", disc_file.replace('\\', "/"))
}

/// 去掉游戏显示名里的碟号标记(用作折叠后的名字);无标记则原样返回。
fn stripped_name(name: &str) -> String {
    parse_disc_marker(name)
        .map(|m| m.base)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| name.to_string())
}

/// 把某组各碟的顶层条目移动到 `<基名>/` 下,并在平台目录写出 `<基名>.m3u`。
/// 返回移动的顶层条目数;目标已存在(已整理)时跳过对应移动,保证幂等。
fn apply_group_reorg(dir: &Path, group: &MultiDiscGame) -> Result<usize, String> {
    let base = &group.base;
    let base_dir = dir.join(base);

    // 去重收集各碟的顶层条目(相对平台目录的第一段)。
    let mut tops: Vec<String> = Vec::new();
    for disc in &group.discs {
        let top = disc
            .file
            .split('/')
            .next()
            .unwrap_or(&disc.file)
            .to_string();
        if !top.is_empty() && !tops.contains(&top) {
            tops.push(top);
        }
    }

    let mut moved = 0usize;
    for top in &tops {
        let src = dir.join(top);
        if !src.exists() {
            continue; // 已整理或缺失
        }
        std::fs::create_dir_all(&base_dir).map_err(|e| e.to_string())?;
        if src.is_dir() {
            // 文件夹型:整个碟目录搬入 <基名>/(保留目录名,无碰撞、可逆)。
            let dst = base_dir.join(top);
            if dst.exists() {
                continue;
            }
            std::fs::rename(&src, &dst)
                .map_err(|e| format!("移动 {} 失败: {e}", src.display()))?;
            moved += 1;
        } else {
            // 平铺型:代表文件 + 同前缀兄弟轨(cue+bin/sub 等)一并搬入。
            let stem = Path::new(top)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(top)
                .to_lowercase();
            let mut siblings: Vec<String> = Vec::new();
            for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                let name_stem = Path::new(&name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&name)
                    .to_lowercase();
                // 前缀匹配,但紧随其后不能是数字,避免 "(Disc 1)" 误吞 "(Disc 10)"。
                let hit = match name_stem.strip_prefix(&stem) {
                    Some(rest) => rest.chars().next().is_none_or(|c| !c.is_ascii_digit()),
                    None => false,
                };
                if hit {
                    siblings.push(name);
                }
            }
            for name in siblings {
                let s = dir.join(&name);
                let dst = base_dir.join(&name);
                if dst.exists() {
                    continue;
                }
                std::fs::rename(&s, &dst)
                    .map_err(|e| format!("移动 {} 失败: {e}", s.display()))?;
            }
            moved += 1;
        }
    }

    // 写 m3u:按碟序逐行列出各碟代表文件(相对平台目录、正斜杠)。
    let m3u_path = dir.join(m3u_relpath(base));
    let mut content = String::new();
    for disc in &group.discs {
        content.push_str(&reorg_target(base, &disc.file));
        content.push('\n');
    }
    std::fs::write(&m3u_path, content)
        .map_err(|e| format!("写 m3u {} 失败: {e}", m3u_path.display()))?;

    Ok(moved)
}

/// 对光盘平台的游戏列表做多碟折叠:物理整理源目录 + 各碟合并成一条指向 m3u 的记录。
/// 就地修改 `games`;非光盘平台或无多碟组时不改动、`changed=false`。幂等(已整理再跑为空操作)。
pub fn organize_and_collapse(
    dir: &Path,
    games: &mut Vec<PegasusGame>,
    is_disc_platform: bool,
) -> Result<OrganizeReport, String> {
    let mut report = OrganizeReport::default();
    if !is_disc_platform {
        return Ok(report);
    }

    let files: Vec<String> = games
        .iter()
        .filter_map(|g| g.file.clone())
        .map(|f| f.replace('\\', "/"))
        .collect();
    let groups = detect_multidisc_groups(&files);
    if groups.is_empty() {
        return Ok(report);
    }

    let group_bases: std::collections::HashSet<String> =
        groups.iter().map(|g| g.base.clone()).collect();
    let mut m3u_files: std::collections::HashSet<String> = std::collections::HashSet::new();

    for group in &groups {
        // 物理整理 + 写 m3u。
        let moved = apply_group_reorg(dir, group)?;
        report.files_moved += moved;
        report.m3u_written += 1;
        report.groups += 1;

        let m3u = m3u_relpath(&group.base);
        m3u_files.insert(m3u.clone());
        let keep_file = group.discs[0].file.clone();

        // 保留碟序最小者:file 改为 m3u、名字去碟号;其余同组条目稍后一并删除。
        if let Some(keep_idx) = games.iter().position(|g| {
            g.file.as_deref().map(|f| f.replace('\\', "/")) == Some(keep_file.clone())
        }) {
            let new_name = stripped_name(&games[keep_idx].name);
            games[keep_idx].name = new_name;
            games[keep_idx].file = Some(m3u);
            games[keep_idx].files.clear();
        }
    }

    // 删除所有归入某组、且不是保留 m3u 条目的游戏(含各碟代表与散落的载荷轨条目)。
    games.retain(|g| {
        let file = g
            .file
            .as_deref()
            .map(|f| f.replace('\\', "/"))
            .unwrap_or_default();
        if m3u_files.contains(&file) {
            return true;
        }
        match disc_key(&file) {
            Some((key, _)) => !group_bases.contains(&key),
            None => true,
        }
    });

    report.changed = report.groups > 0;
    Ok(report)
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

    #[test]
    fn descriptor_wins_over_payload_track_as_representative() {
        // 同碟号既有 cue 又有 bin 时,代表应为 cue。
        let files = vec![
            "世纪末Disc A/game.bin".to_string(),
            "世纪末Disc A/game.cue".to_string(),
            "世纪末Disc B/game.bin".to_string(),
            "世纪末Disc B/game.cue".to_string(),
        ];
        let groups = detect_multidisc_groups(&files);
        assert_eq!(groups.len(), 1);
        assert!(groups[0].discs.iter().all(|d| d.file.ends_with(".cue")));
    }

    // --- 整理 + 折叠 ---

    fn disc_game(name: &str, file: &str) -> PegasusGame {
        PegasusGame {
            name: name.to_string(),
            file: Some(file.to_string()),
            ..Default::default()
        }
    }

    fn tmp_dir(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!(
            "mrrm-md-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn organize_folder_per_disc_moves_folders_writes_m3u_and_collapses() {
        let dir = tmp_dir("folder");
        for (folder, cue) in [
            ("世纪末[简]Disc A", "世纪末A.cue"),
            ("世纪末[简]Disc B", "世纪末B.cue"),
        ] {
            std::fs::create_dir_all(dir.join(folder)).unwrap();
            std::fs::write(dir.join(folder).join(cue), b"cue").unwrap();
            std::fs::write(dir.join(folder).join("game.bin"), b"bin").unwrap();
        }
        let mut games = vec![
            disc_game("世纪末[简]", "世纪末[简]Disc A/世纪末A.cue"),
            disc_game("世纪末[简]", "世纪末[简]Disc B/世纪末B.cue"),
        ];

        let report = organize_and_collapse(&dir, &mut games, true).unwrap();
        assert!(report.changed);
        assert_eq!(report.groups, 1);
        assert_eq!(report.files_moved, 2);

        // 折叠成一条,指向 m3u。
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].file.as_deref(), Some("世纪末[简].m3u"));

        // m3u 落在平台根,列出两碟(相对平台目录)。
        let m3u = std::fs::read_to_string(dir.join("世纪末[简].m3u")).unwrap();
        assert!(m3u.contains("世纪末[简]/世纪末[简]Disc A/世纪末A.cue"));
        assert!(m3u.contains("世纪末[简]/世纪末[简]Disc B/世纪末B.cue"));

        // 各碟文件夹已搬入 <基名>/,原位置不再存在。
        assert!(dir
            .join("世纪末[简]")
            .join("世纪末[简]Disc A")
            .join("世纪末A.cue")
            .exists());
        assert!(!dir.join("世纪末[简]Disc A").exists());

        // 幂等:再跑一次无改动。
        let again = organize_and_collapse(&dir, &mut games, true).unwrap();
        assert!(!again.changed);
        assert_eq!(games.len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn organize_flat_files_moves_siblings_into_subfolder() {
        let dir = tmp_dir("flat");
        for f in [
            "FF7 (Disc 1).cue",
            "FF7 (Disc 1).bin",
            "FF7 (Disc 2).cue",
            "FF7 (Disc 2).bin",
        ] {
            std::fs::write(dir.join(f), b"x").unwrap();
        }
        let mut games = vec![
            disc_game("Final Fantasy VII", "FF7 (Disc 1).cue"),
            disc_game("Final Fantasy VII", "FF7 (Disc 2).cue"),
        ];

        let report = organize_and_collapse(&dir, &mut games, true).unwrap();
        assert!(report.changed);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].file.as_deref(), Some("FF7.m3u"));

        // 代表 cue 及其兄弟 bin 都搬进 FF7/。
        assert!(dir.join("FF7").join("FF7 (Disc 1).cue").exists());
        assert!(dir.join("FF7").join("FF7 (Disc 1).bin").exists());
        assert!(dir.join("FF7").join("FF7 (Disc 2).bin").exists());
        assert!(!dir.join("FF7 (Disc 1).cue").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn non_disc_platform_is_untouched() {
        let dir = tmp_dir("nondisc");
        let mut games = vec![
            disc_game("三国志2 (Disc 1)", "三国志2 (Disc 1).sfc"),
            disc_game("三国志2 (Disc 2)", "三国志2 (Disc 2).sfc"),
        ];
        let report = organize_and_collapse(&dir, &mut games, false).unwrap();
        assert!(!report.changed);
        assert_eq!(games.len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
