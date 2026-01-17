use crate::config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::RwLock;

/// Scraper API 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScraperConfig {
    pub enabled: bool,
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

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 界面主题: light, dark, cyberpunk, ocean, forest, sunset, rose, nord
    pub theme: String,
    /// 界面语言: zh, en
    pub language: String,
    /// 视图模式: grid, list
    pub view_mode: String,
    /// 扫描目录列表
    pub scan_directories: Vec<String>,
    /// Scraper API 配置 (key: provider id)
    #[serde(default)]
    pub scrapers: HashMap<String, ScraperConfig>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            language: "zh".to_string(),
            view_mode: "grid".to_string(),
            scan_directories: Vec::new(),
            scrapers: HashMap::new(),
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
    SETTINGS
        .read()
        .unwrap()
        .clone()
        .unwrap_or_default()
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
