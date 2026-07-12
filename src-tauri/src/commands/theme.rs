use serde::Serialize;
use std::fs;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// 主题包解压后的总大小上限(20MB)
const MAX_UNPACKED_SIZE: u64 = 20 * 1024 * 1024;

/// 已导入自定义主题信息(字段名与前端契约保持 snake_case)
#[derive(Debug, Clone, Serialize)]
pub struct CustomThemeInfo {
    pub manifest_json: String,
    pub dir: String,
    pub custom_css: Option<String>,
}

/// 主题存储根目录:config/themes/(与 settings.json 同级,复用 config 目录取法)
fn themes_root() -> PathBuf {
    crate::config::get_config_dir().join("themes")
}

/// 主题 id 校验:^[a-z0-9][a-z0-9-]*$(同时防止路径穿越)
fn is_valid_theme_id(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// 校验 theme.json 清单(语义与前端 src/theme/validate.ts 对齐),返回主题 id
fn validate_manifest_json(json: &str) -> Result<String, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|_| "主题清单不是合法的 JSON".to_string())?;
    let obj = value
        .as_object()
        .ok_or_else(|| "主题清单不是对象".to_string())?;
    if obj.get("schemaVersion").and_then(|v| v.as_i64()) != Some(1) {
        return Err("不支持的主题 schemaVersion,请升级应用后重试".to_string());
    }
    let id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "主题清单缺少 id".to_string())?;
    if !is_valid_theme_id(id) {
        return Err("主题 id 非法:仅允许小写字母、数字和连字符".to_string());
    }
    match obj.get("name").and_then(|v| v.as_str()) {
        Some(name) if !name.is_empty() => {}
        _ => return Err("主题清单缺少 name".to_string()),
    }
    if !obj.get("tokens").map(|v| v.is_object()).unwrap_or(false) {
        return Err("主题清单缺少 tokens".to_string());
    }
    Ok(id.to_string())
}

fn re_import() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"(?is)@import[^;]*;").unwrap())
}

fn re_external_url() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r#"(?is)url\(\s*['"]?\s*(?:https?:|//)[^)]*\)"#).unwrap())
}

fn re_asset_url() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r#"(?i)url\(\s*['"]?assets/([^)'"]+?)['"]?\s*\)"#).unwrap())
}

/// 清洗主题自定义 CSS:
/// - 含 `</style` 或 `javascript:` 直接拒绝
/// - 剥离全部 `@import …;`
/// - 外部 `url(http…)` / `url(//…)` 替换为空 `url()` 并计数
/// - `url(assets/xxx)` 相对路径重写为 `url(__RR_ASSET__/xxx)`(前端用 convertFileSrc 替换)
///
/// 返回 (清洗后的 CSS, 被移除的外部引用数量)
fn sanitize_css(css: &str) -> Result<(String, usize), String> {
    let lower = css.to_lowercase();
    if lower.contains("</style") {
        return Err("主题 CSS 含被禁止的内容(</style),已拒绝导入".to_string());
    }
    if lower.contains("javascript:") {
        return Err("主题 CSS 含被禁止的内容(javascript:),已拒绝导入".to_string());
    }

    let css = re_import().replace_all(css, "");

    let mut removed = 0usize;
    let css = re_external_url().replace_all(&css, |_: &regex::Captures| {
        removed += 1;
        "url()".to_string()
    });

    let css = re_asset_url().replace_all(&css, "url(__RR_ASSET__/$1)");
    Ok((css.into_owned(), removed))
}

/// 读取 zip 内指定条目为 UTF-8 文本;条目不存在返回 Ok(None)
fn read_entry_string<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> Result<Option<String>, String> {
    match archive.by_name(name) {
        Ok(mut entry) => {
            let mut s = String::new();
            entry
                .read_to_string(&mut s)
                .map_err(|_| format!("主题包内 {} 不是有效的 UTF-8 文本", name))?;
            Ok(Some(s))
        }
        Err(zip::result::ZipError::FileNotFound) => Ok(None),
        Err(e) => Err(format!("读取主题包条目失败:{}", e)),
    }
}

/// 从任意 Read+Seek 源导入主题包(便于单元测试使用内存数据)
fn import_theme_pack_from_reader<R: Read + Seek>(
    reader: R,
    themes_root: &Path,
) -> Result<CustomThemeInfo, String> {
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|_| "无法读取主题包:不是有效的压缩文件".to_string())?;

    // 安全预检:zip-slip 路径穿越 + 解压总大小上限
    let mut declared_total: u64 = 0;
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| format!("读取主题包条目失败:{}", e))?;
        let name = entry.name();
        if name.split(['/', '\\']).any(|part| part == "..")
            || name.starts_with('/')
            || name.contains(':')
        {
            return Err("主题包内含非法路径,已拒绝导入".to_string());
        }
        declared_total += entry.size();
    }
    if declared_total > MAX_UNPACKED_SIZE {
        return Err("主题包解压后超过 20MB 上限,已拒绝导入".to_string());
    }

    // 校验清单
    let manifest_json = read_entry_string(&mut archive, "theme.json")?
        .ok_or_else(|| "主题包缺少 theme.json".to_string())?;
    let id = validate_manifest_json(&manifest_json)?;

    // CSS 清洗(可选文件)
    let custom_css = match read_entry_string(&mut archive, "style.css")? {
        Some(css) => Some(sanitize_css(&css)?.0),
        None => None,
    };

    // 解压到 themes/<id>/,同 id 整目录覆盖
    let dest = themes_root.join(&id);
    if dest.exists() {
        fs::remove_dir_all(&dest).map_err(|e| format!("清理旧主题目录失败:{}", e))?;
    }
    fs::create_dir_all(&dest).map_err(|e| format!("创建主题目录失败:{}", e))?;

    let mut written_total: u64 = 0;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("读取主题包条目失败:{}", e))?;
        let Some(rel) = entry.enclosed_name() else {
            let _ = fs::remove_dir_all(&dest);
            return Err("主题包内含非法路径,已拒绝导入".to_string());
        };
        let out_path = dest.join(&rel);
        if entry.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| format!("创建主题子目录失败:{}", e))?;
            continue;
        }
        // style.css 稍后写入清洗后的版本
        if rel == Path::new("style.css") {
            continue;
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建主题子目录失败:{}", e))?;
        }
        let mut out = fs::File::create(&out_path).map_err(|e| format!("写入主题文件失败:{}", e))?;
        let budget = MAX_UNPACKED_SIZE - written_total + 1;
        let copied = std::io::copy(&mut (&mut entry).take(budget), &mut out)
            .map_err(|e| format!("写入主题文件失败:{}", e))?;
        written_total += copied;
        if written_total > MAX_UNPACKED_SIZE {
            let _ = fs::remove_dir_all(&dest);
            return Err("主题包解压后超过 20MB 上限,已拒绝导入".to_string());
        }
    }
    if let Some(css) = &custom_css {
        fs::write(dest.join("style.css"), css).map_err(|e| format!("写入主题样式失败:{}", e))?;
    }

    Ok(CustomThemeInfo {
        manifest_json,
        dir: dest.to_string_lossy().to_string(),
        custom_css,
    })
}

/// 扫描 themes 根目录,损坏的主题跳过并继续
fn list_custom_themes_impl(themes_root: &Path) -> Result<Vec<CustomThemeInfo>, String> {
    let mut result = Vec::new();
    if !themes_root.exists() {
        return Ok(result);
    }
    let entries = fs::read_dir(themes_root).map_err(|e| format!("读取主题目录失败:{}", e))?;
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let Ok(manifest_json) = fs::read_to_string(dir.join("theme.json")) else {
            continue;
        };
        if validate_manifest_json(&manifest_json).is_err() {
            continue;
        }
        let custom_css = fs::read_to_string(dir.join("style.css")).ok();
        result.push(CustomThemeInfo {
            manifest_json,
            dir: dir.to_string_lossy().to_string(),
            custom_css,
        });
    }
    result.sort_by(|a, b| a.dir.cmp(&b.dir));
    Ok(result)
}

fn delete_custom_theme_impl(themes_root: &Path, id: &str) -> Result<(), String> {
    if !is_valid_theme_id(id) {
        return Err("主题 id 非法".to_string());
    }
    let dir = themes_root.join(id);
    if !dir.is_dir() {
        return Err("主题不存在或已被删除".to_string());
    }
    fs::remove_dir_all(&dir).map_err(|e| format!("删除主题失败:{}", e))
}

/// 列出全部已导入的自定义主题
#[tauri::command]
pub fn list_custom_themes(_app: tauri::AppHandle) -> Result<Vec<CustomThemeInfo>, String> {
    list_custom_themes_impl(&themes_root())
}

/// 导入 .rrtheme 主题包(zip):校验清单、清洗 CSS、解压到 config/themes/<id>/
#[tauri::command]
pub fn import_theme_pack(
    _app: tauri::AppHandle,
    file_path: String,
) -> Result<CustomThemeInfo, String> {
    let file =
        fs::File::open(&file_path).map_err(|_| "无法打开主题包文件,请确认路径有效".to_string())?;
    import_theme_pack_from_reader(file, &themes_root())
}

/// 删除已导入的自定义主题
#[tauri::command]
pub fn delete_custom_theme(_app: tauri::AppHandle, id: String) -> Result<(), String> {
    delete_custom_theme_impl(&themes_root(), &id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    fn temp_root(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("rr-theme-test-{}-{}", tag, uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn build_zip(entries: &[(&str, &[u8])]) -> Cursor<Vec<u8>> {
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut cursor);
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            for (name, data) in entries {
                writer.start_file(*name, opts).unwrap();
                writer.write_all(data).unwrap();
            }
            writer.finish().unwrap();
        }
        cursor.set_position(0);
        cursor
    }

    const VALID_MANIFEST: &str = r##"{
        "schemaVersion": 1,
        "id": "retro-arcade",
        "name": "复古游戏厅",
        "tokens": { "accent-primary": "#e94560" }
    }"##;

    // ---------- sanitize_css ----------

    #[test]
    fn sanitize_strips_import() {
        let css = "@import url(\"https://fonts.googleapis.com/css2\");\n.rr-button { color: red; }";
        let (out, _) = sanitize_css(css).unwrap();
        assert!(!out.contains("@import"));
        assert!(out.contains(".rr-button { color: red; }"));
    }

    #[test]
    fn sanitize_blanks_external_urls_and_counts() {
        let css = ".a { background: url(https://evil.com/x.png); }\n.b { background: url('//evil.com/y.png'); }";
        let (out, removed) = sanitize_css(css).unwrap();
        assert_eq!(removed, 2);
        assert!(!out.contains("evil.com"));
        assert_eq!(out.matches("url()").count(), 2);
    }

    #[test]
    fn sanitize_rejects_style_close_and_javascript() {
        assert!(sanitize_css(".a {} </style><script>alert(1)</script>").is_err());
        assert!(sanitize_css(".a { background: url('javascript:alert(1)'); }").is_err());
    }

    #[test]
    fn sanitize_rewrites_asset_urls() {
        let css =
            ".c { background: url(assets/bg.png); }\n.d { src: url(\"assets/fonts/a.woff2\"); }";
        let (out, removed) = sanitize_css(css).unwrap();
        assert_eq!(removed, 0);
        assert!(out.contains("url(__RR_ASSET__/bg.png)"));
        assert!(out.contains("url(__RR_ASSET__/fonts/a.woff2)"));
    }

    #[test]
    fn sanitize_keeps_plain_css() {
        let css = ".rr-card { box-shadow: 3px 3px 0 var(--border-strong); }";
        let (out, removed) = sanitize_css(css).unwrap();
        assert_eq!(out, css);
        assert_eq!(removed, 0);
    }

    // ---------- validate_manifest_json ----------

    #[test]
    fn manifest_valid_passes() {
        assert_eq!(
            validate_manifest_json(VALID_MANIFEST).unwrap(),
            "retro-arcade"
        );
    }

    #[test]
    fn manifest_rejects_wrong_schema_version() {
        let json = r#"{ "schemaVersion": 2, "id": "a", "name": "A", "tokens": {} }"#;
        let err = validate_manifest_json(json).unwrap_err();
        assert!(err.contains("schemaVersion"));
    }

    #[test]
    fn manifest_rejects_invalid_id() {
        for bad in ["../evil", "UPPER", "汉字", "a/b", "", "-lead"] {
            let json = format!(
                r#"{{ "schemaVersion": 1, "id": "{}", "name": "A", "tokens": {{}} }}"#,
                bad
            );
            assert!(
                validate_manifest_json(&json).is_err(),
                "id {:?} 应被拒绝",
                bad
            );
        }
    }

    #[test]
    fn manifest_rejects_missing_fields() {
        assert!(validate_manifest_json("not json").is_err());
        assert!(
            validate_manifest_json(r#"{ "schemaVersion": 1, "id": "a", "tokens": {} }"#).is_err()
        );
        assert!(
            validate_manifest_json(r#"{ "schemaVersion": 1, "id": "a", "name": "A" }"#).is_err()
        );
    }

    // ---------- zip 解压安全 ----------

    #[test]
    fn import_rejects_zip_slip() {
        let root = temp_root("slip");
        let zip = build_zip(&[
            ("theme.json", VALID_MANIFEST.as_bytes()),
            ("../evil.txt", b"pwned"),
        ]);
        let err = import_theme_pack_from_reader(zip, &root).unwrap_err();
        assert!(err.contains("非法路径"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn import_rejects_oversized_pack() {
        let root = temp_root("size");
        let big = vec![0u8; (MAX_UNPACKED_SIZE + 1) as usize];
        let zip = build_zip(&[
            ("theme.json", VALID_MANIFEST.as_bytes()),
            ("assets/big.bin", &big),
        ]);
        let err = import_theme_pack_from_reader(zip, &root).unwrap_err();
        assert!(err.contains("20MB"));
        let _ = fs::remove_dir_all(&root);
    }

    // ---------- 导入 / 列表 / 删除 ----------

    #[test]
    fn import_list_delete_roundtrip() {
        let root = temp_root("roundtrip");
        let css = ".rr-card { background: url(assets/bg.png); }\n.x { background: url(https://e.com/a.png); }";
        let zip = build_zip(&[
            ("theme.json", VALID_MANIFEST.as_bytes()),
            ("style.css", css.as_bytes()),
            ("assets/bg.png", b"fakepng"),
        ]);
        let info = import_theme_pack_from_reader(zip, &root).unwrap();
        assert_eq!(info.manifest_json, VALID_MANIFEST);
        assert!(info.dir.ends_with("retro-arcade"));
        let saved_css = info.custom_css.unwrap();
        assert!(saved_css.contains("url(__RR_ASSET__/bg.png)"));
        assert!(!saved_css.contains("e.com"));

        // 磁盘上的 style.css 是清洗后的版本
        let disk_css = fs::read_to_string(root.join("retro-arcade/style.css")).unwrap();
        assert_eq!(disk_css, saved_css);
        assert!(root.join("retro-arcade/assets/bg.png").exists());

        // 同 id 再导入 = 整目录覆盖(旧资产被清掉)
        fs::write(root.join("retro-arcade/stale.txt"), "old").unwrap();
        let zip2 = build_zip(&[("theme.json", VALID_MANIFEST.as_bytes())]);
        let info2 = import_theme_pack_from_reader(zip2, &root).unwrap();
        assert!(info2.custom_css.is_none());
        assert!(!root.join("retro-arcade/stale.txt").exists());

        // list:损坏主题跳过并继续
        fs::create_dir_all(root.join("broken")).unwrap();
        fs::write(root.join("broken/theme.json"), "{ garbage").unwrap();
        let listed = list_custom_themes_impl(&root).unwrap();
        assert_eq!(listed.len(), 1);
        assert!(listed[0].dir.ends_with("retro-arcade"));

        // delete
        delete_custom_theme_impl(&root, "retro-arcade").unwrap();
        assert!(!root.join("retro-arcade").exists());
        assert!(delete_custom_theme_impl(&root, "retro-arcade").is_err());
        assert!(delete_custom_theme_impl(&root, "../evil").is_err());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn import_rejects_missing_manifest() {
        let root = temp_root("nomanifest");
        let zip = build_zip(&[("style.css", b".a {}")]);
        let err = import_theme_pack_from_reader(zip, &root).unwrap_err();
        assert!(err.contains("theme.json"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn list_missing_root_returns_empty() {
        let root = std::env::temp_dir().join(format!("rr-theme-none-{}", uuid::Uuid::new_v4()));
        assert!(list_custom_themes_impl(&root).unwrap().is_empty());
    }
}
