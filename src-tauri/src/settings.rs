use crate::config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::RwLock;

/// 目录配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryConfig {
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
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

fn default_priority() -> u32 {
    100
}

impl Default for ScraperConfig {
    fn default() -> Self {
        Self {
            // Provider 默认启用,与未配置任何 ScraperConfig 时的展示语义保持一致
            enabled: true,
            priority: default_priority(),
            api_key: None,
            client_id: None,
            client_secret: None,
            username: None,
            password: None,
        }
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 界面主题: light, dark, cyberpunk, ocean, forest, sunset, rose, nord
    pub theme: String,
    /// 界面语言: zh, en
    pub language: String,
    /// 视图模式: grid, list
    pub view_mode: String,
    /// 动效等级: off, low, full(旧版 settings.json 无此字段,缺省为 None)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motion_level: Option<String>,
    /// 目录列表
    #[serde(default)]
    pub directories: Vec<DirectoryConfig>,
    /// Scraper API 配置 (key: provider id)
    #[serde(default)]
    pub scrapers: HashMap<String, ScraperConfig>,
    /// 自动抓取和候选缓存允许的媒体类型
    #[serde(default = "default_scraper_media_types")]
    pub scraper_media_types: Vec<String>,
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
            scrapers: HashMap::new(),
            scraper_media_types: default_scraper_media_types(),
        }
    }
}

static SETTINGS: RwLock<Option<AppSettings>> = RwLock::new(None);

/// 加载配置（如果不存在则创建默认配置）
pub fn load_settings() -> Result<AppSettings, Box<dyn std::error::Error>> {
    let path = config::get_settings_path();

    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let settings: AppSettings = serde_json::from_str(&content)?;
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

    // 确保目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(settings)?;
    fs::write(&path, content)?;
    *SETTINGS.write().unwrap() = Some(settings.clone());
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
    let mut settings = get_settings();
    updater(&mut settings);
    save_settings(&settings)?;
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
    fn update_motion_level_then_reload_reads_back() {
        let updated = update_setting(|s| s.motion_level = Some("off".to_string())).unwrap();
        assert_eq!(updated.motion_level.as_deref(), Some("off"));

        // 重新从磁盘加载,能读回 motion_level
        let reloaded = load_settings().unwrap();
        assert_eq!(reloaded.motion_level.as_deref(), Some("off"));
    }
}
