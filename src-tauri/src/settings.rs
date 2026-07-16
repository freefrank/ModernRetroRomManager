use crate::config;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::RwLock;

/// 目录配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryConfig {
    /// 稳定的 Library ID；旧配置加载时根据路径自动补齐。
    #[serde(default)]
    pub id: String,
    /// 用户可编辑的 Library 名称。
    #[serde(default)]
    pub name: String,
    /// 目录路径
    pub path: String,
    /// 是否为 ROMs 根目录（包含多个系统子目录）
    /// true: 扫描子目录作为独立系统
    /// false: 当前目录就是单个系统目录
    #[serde(default)]
    pub is_root_directory: bool,
    /// 元数据格式: emulationstation, pegasus, launchbox, none
    /// 对于 root 目录，每个子目录可能有不同格式，在运行时自动检测
    pub metadata_format: String,
    /// 系统 ID (可选，仅用于单系统目录)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_id: Option<String>,
}

/// Scraper API 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperConfig {
    pub enabled: bool,
    #[serde(default = "default_priority")]
    pub priority: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u32,
    #[serde(default = "default_threads")]
    pub threads: u32,
}

fn default_priority() -> u32 {
    100
}

fn default_rate_limit() -> u32 {
    1
}
fn default_threads() -> u32 {
    1
}

impl Default for ScraperConfig {
    fn default() -> Self {
        Self {
            // Provider 默认启用,与未配置任何 ScraperConfig 时的展示语义保持一致
            enabled: true,
            priority: default_priority(),
            api_key: None,
            username: None,
            password: None,
            rate_limit: default_rate_limit(),
            threads: default_threads(),
        }
    }
}

/// OpenAI-compatible metadata translation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTranslationConfig {
    #[serde(default = "default_ai_endpoint")]
    pub endpoint: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub model: String,
    #[serde(default = "default_ai_target_language")]
    pub target_language: String,
    /// Batch translation may combine several ROM entries into one API request.
    #[serde(default)]
    pub merge_batch_requests: bool,
    /// Approximate context budget used when grouping batch translation requests.
    #[serde(default = "default_ai_batch_token_limit")]
    pub batch_token_limit: u32,
    /// OpenAI-compatible reasoning_effort value; auto omits the parameter.
    #[serde(default = "default_ai_reasoning_effort")]
    pub reasoning_effort: String,
}

fn default_ai_endpoint() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_ai_target_language() -> String {
    "简体中文（zh-CN）".to_string()
}

fn default_ai_batch_token_limit() -> u32 {
    50_000
}

fn default_ai_reasoning_effort() -> String {
    "auto".to_string()
}

impl Default for AiTranslationConfig {
    fn default() -> Self {
        Self {
            endpoint: default_ai_endpoint(),
            api_key: String::new(),
            model: String::new(),
            target_language: default_ai_target_language(),
            merge_batch_requests: false,
            batch_token_limit: default_ai_batch_token_limit(),
            reasoning_effort: default_ai_reasoning_effort(),
        }
    }
}

/// Provider 累计运行统计，便于确认抓取源是否实际参与工作。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScraperStats {
    #[serde(default)]
    pub matched_count: u64,
    #[serde(default)]
    pub selected_count: u64,
    #[serde(default)]
    pub cache_hit_count: u64,
    #[serde(default)]
    pub error_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 界面主题: light, dark, cyberpunk, ocean, forest, sunset, rose, nord
    pub theme: String,
    /// 界面语言: zh-CN, zh-TW, en, fr, de, it, es, ru（兼容旧值 zh）
    pub language: String,
    /// 视图模式: grid, list
    pub view_mode: String,
    /// 动效等级: off, low, full(旧版 settings.json 无此字段,缺省为 None)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion_level: Option<String>,
    /// 目录列表
    #[serde(default)]
    pub directories: Vec<DirectoryConfig>,
    /// 当前激活的 Library；旧配置默认激活第一条目录。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_library_id: Option<String>,
    /// Scraper API 配置 (key: provider id)
    #[serde(default)]
    pub scrapers: HashMap<String, ScraperConfig>,
    /// Provider 累计匹配、采用、缓存和错误统计。
    #[serde(default)]
    pub scraper_stats: HashMap<String, ScraperStats>,
    /// 自动抓取和候选缓存允许的媒体类型
    #[serde(default = "default_scraper_media_types")]
    pub scraper_media_types: Vec<String>,
    /// OpenAI-compatible metadata translation settings.
    #[serde(default)]
    pub ai_translation: AiTranslationConfig,
}

fn default_scraper_media_types() -> Vec<String> {
    ["boxfront", "logo", "screenshot", "titlescreen", "hero"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            language: "zh".to_string(),
            view_mode: "grid".to_string(),
            motion_level: None,
            directories: Vec::new(),
            active_library_id: None,
            scrapers: HashMap::new(),
            scraper_stats: HashMap::new(),
            scraper_media_types: default_scraper_media_types(),
            ai_translation: AiTranslationConfig::default(),
        }
    }
}

static SETTINGS: RwLock<Option<AppSettings>> = RwLock::new(None);

/// 根据规范化路径生成跨启动稳定的 Library ID。
pub fn library_id_for_path(path: &str) -> String {
    let normalized = path
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_ascii_lowercase();
    let hash = normalized
        .as_bytes()
        .iter()
        .fold(0xcbf29ce484222325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        });
    format!("library-{hash:016x}")
}

/// 对外展示和持久化使用当前平台的原生路径分隔符。
pub fn native_path(path: &str) -> String {
    #[cfg(windows)]
    {
        path.replace('/', "\\")
    }
    #[cfg(not(windows))]
    {
        path.replace('\\', "/")
    }
}

/// 新 Library 默认使用末级目录名；盘符根目录直接显示盘符。
pub fn default_library_name(path: &str, position: usize) -> String {
    let normalized = path.replace('\\', "/");
    normalized
        .trim_end_matches('/')
        .rsplit('/')
        .find(|part| !part.trim().is_empty())
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("Library {}", position + 1))
}

/// 补齐旧版目录记录，并保证 ID、名称和激活项始终有效。
fn normalize_library_settings(settings: &mut AppSettings) -> bool {
    let mut changed = false;
    let mut used_ids = HashSet::new();

    for (position, library) in settings.directories.iter_mut().enumerate() {
        let normalized_path = native_path(&library.path);
        if library.path != normalized_path {
            library.path = normalized_path;
            changed = true;
        }
        let base_id = if library.id.trim().is_empty() {
            changed = true;
            library_id_for_path(&library.path)
        } else {
            library.id.trim().to_string()
        };
        let mut unique_id = base_id.clone();
        let mut suffix = 2;
        while used_ids.contains(&unique_id) {
            unique_id = format!("{base_id}-{suffix}");
            suffix += 1;
        }
        if library.id != unique_id {
            library.id = unique_id.clone();
            changed = true;
        }
        used_ids.insert(unique_id);

        let trimmed_name = library.name.trim();
        let normalized_name = if trimmed_name.is_empty() {
            default_library_name(&library.path, position)
        } else {
            trimmed_name.to_string()
        };
        if library.name != normalized_name {
            library.name = normalized_name;
            changed = true;
        }
    }

    let active_is_valid = settings
        .active_library_id
        .as_ref()
        .is_some_and(|active| settings.directories.iter().any(|item| &item.id == active));
    if !active_is_valid {
        let fallback = settings.directories.first().map(|item| item.id.clone());
        if settings.active_library_id != fallback {
            settings.active_library_id = fallback;
            changed = true;
        }
    }

    changed
}

impl AppSettings {
    pub fn active_library(&self) -> Option<&DirectoryConfig> {
        let active_id = self.active_library_id.as_ref()?;
        self.directories.iter().find(|item| &item.id == active_id)
    }
}

fn write_settings_file(
    path: &std::path::Path,
    settings: &AppSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let serialized = serde_json::to_string_pretty(settings)?;
    let temporary = path.with_extension("json.part");
    let backup = path.with_extension("json.bak");
    fs::write(&temporary, serialized)?;

    // 每次替换前保留最后一份完整配置，升级或异常写入后仍可人工恢复。
    if path.is_file() {
        fs::copy(path, &backup)?;
        fs::remove_file(path)?;
    }
    fs::rename(&temporary, path)?;
    Ok(())
}

fn read_settings_file_with_backup(
    path: &std::path::Path,
) -> Result<AppSettings, Box<dyn std::error::Error>> {
    let parse = |candidate: &std::path::Path| -> Result<AppSettings, Box<dyn std::error::Error>> {
        Ok(serde_json::from_str(&fs::read_to_string(candidate)?)?)
    };

    match parse(path) {
        Ok(settings) => Ok(settings),
        Err(primary_error) => {
            let backup = path.with_extension("json.bak");
            if !backup.is_file() {
                return Err(primary_error);
            }
            let settings = parse(&backup)?;
            if path.is_file() {
                fs::copy(path, path.with_extension("json.corrupt"))?;
                fs::remove_file(path)?;
            }
            // 主文件损坏或丢失时恢复最后一份完整备份，不覆盖 .bak。
            write_settings_file(path, &settings)?;
            Ok(settings)
        }
    }
}

/// 加载配置（如果不存在则创建默认配置）
pub fn load_settings() -> Result<AppSettings, Box<dyn std::error::Error>> {
    let path = config::get_settings_path();

    if path.exists() || path.with_extension("json.bak").exists() {
        let mut settings = read_settings_file_with_backup(&path)?;
        if normalize_library_settings(&mut settings) {
            write_settings_file(&path, &settings)?;
        }
        *SETTINGS.write().unwrap() = Some(settings.clone());
        Ok(settings)
    } else {
        let settings = AppSettings::default();
        save_settings(&settings)?;
        *SETTINGS.write().unwrap() = Some(settings.clone());
        Ok(settings)
    }
}

/// 保存配置
pub fn save_settings(settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
    let path = config::get_settings_path();
    let mut normalized = settings.clone();
    normalize_library_settings(&mut normalized);
    write_settings_file(&path, &normalized)?;
    *SETTINGS.write().unwrap() = Some(normalized);
    Ok(())
}

/// 获取当前配置（内存缓存）
pub fn get_settings() -> AppSettings {
    SETTINGS.read().unwrap().clone().unwrap_or_default()
}

/// 更新单个配置项
pub fn update_setting<F>(updater: F) -> Result<AppSettings, Box<dyn std::error::Error>>
where
    F: FnOnce(&mut AppSettings),
{
    let mut guard = SETTINGS.write().unwrap();
    let mut settings = guard.clone().unwrap_or_default();
    updater(&mut settings);
    normalize_library_settings(&mut settings);
    let path = config::get_settings_path();
    write_settings_file(&path, &settings)?;
    *guard = Some(settings.clone());
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motion_level_serde_roundtrip_and_backward_compat() {
        // 序列化/反序列化往返
        let mut s = AppSettings::default();
        assert!(s.motion_level.is_none());
        s.motion_level = Some("low".to_string());
        let json = serde_json::to_string(&s).unwrap();
        let loaded: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.motion_level.as_deref(), Some("low"));

        // 旧版 settings.json(无 motion_level 字段)向后兼容
        let legacy = r#"{"theme":"dark","language":"zh","view_mode":"grid"}"#;
        let loaded: AppSettings = serde_json::from_str(legacy).unwrap();
        assert!(loaded.motion_level.is_none());
    }

    #[test]
    fn legacy_directories_are_migrated_to_libraries() {
        let legacy = r#"{
            "theme":"dark",
            "language":"zh",
            "view_mode":"grid",
            "directories":[
                {"path":"G:/ROMS","is_root_directory":true,"metadata_format":"auto"},
                {"path":"Y:/3DS","is_root_directory":false,"metadata_format":"none"}
            ]
        }"#;
        let mut loaded: AppSettings = serde_json::from_str(legacy).unwrap();

        assert!(normalize_library_settings(&mut loaded));
        assert_eq!(loaded.directories[0].name, "ROMS");
        assert_eq!(loaded.directories[1].name, "3DS");
        assert!(!loaded.directories[0].id.is_empty());
        assert_ne!(loaded.directories[0].id, loaded.directories[1].id);
        assert_eq!(
            loaded.active_library_id.as_deref(),
            Some(loaded.directories[0].id.as_str())
        );
        assert!(!normalize_library_settings(&mut loaded));
    }

    #[test]
    fn settings_writes_are_atomic_and_keep_previous_backup() {
        let root = std::env::temp_dir().join(format!(
            "mrrm-settings-backup-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = root.join("settings.json");
        let _ = fs::remove_dir_all(&root);

        let first = AppSettings::default();
        write_settings_file(&path, &first).unwrap();
        let mut second = first.clone();
        second.theme = "retro-arcade".into();
        write_settings_file(&path, &second).unwrap();

        let saved: AppSettings = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let backup: AppSettings =
            serde_json::from_str(&fs::read_to_string(path.with_extension("json.bak")).unwrap())
                .unwrap();
        assert_eq!(saved.theme, "retro-arcade");
        assert_eq!(backup.theme, first.theme);
        assert!(!path.with_extension("json.part").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn invalid_settings_restore_the_last_valid_backup() {
        let root = std::env::temp_dir().join(format!(
            "mrrm-settings-recovery-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = root.join("settings.json");
        fs::create_dir_all(&root).unwrap();
        fs::write(&path, "{invalid").unwrap();
        let expected = AppSettings {
            theme: "retro-arcade".into(),
            ..Default::default()
        };
        fs::write(
            path.with_extension("json.bak"),
            serde_json::to_string_pretty(&expected).unwrap(),
        )
        .unwrap();

        let recovered = read_settings_file_with_backup(&path).unwrap();
        assert_eq!(recovered.theme, expected.theme);
        assert!(path.with_extension("json.corrupt").is_file());
        assert_eq!(
            serde_json::from_str::<AppSettings>(&fs::read_to_string(&path).unwrap())
                .unwrap()
                .theme,
            expected.theme
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn update_motion_level_then_reload_reads_back() {
        let updated = update_setting(|s| s.motion_level = Some("off".to_string())).unwrap();
        assert_eq!(updated.motion_level.as_deref(), Some("off"));

        // 重新从磁盘加载,能读回 motion_level
        let reloaded = load_settings().unwrap();
        assert_eq!(reloaded.motion_level.as_deref(), Some("off"));
    }
}
