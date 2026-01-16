use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// 游戏系统/平台
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, Selectable)]
#[diesel(table_name = crate::db::schema::systems)]
pub struct System {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub manufacturer: Option<String>,
    pub release_year: Option<i32>,
    pub extensions: String, // JSON 数组
    pub igdb_platform_id: Option<i32>,
    pub thegamesdb_platform_id: Option<i32>,
}

/// ROM 文件
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, Selectable)]
#[diesel(table_name = crate::db::schema::roms)]
pub struct Rom {
    pub id: String,
    pub filename: String,
    pub path: String,
    pub system_id: String,
    pub size: i64,
    pub crc32: Option<String>,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// ROM 元数据
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, Selectable)]
#[diesel(table_name = crate::db::schema::rom_metadata)]
pub struct RomMetadata {
    pub rom_id: String,
    pub name: String,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<String>, // JSON 数组
    pub players: Option<i32>,
    pub rating: Option<f64>,
    pub region: Option<String>,
    pub scraper_source: Option<String>,
    pub scraped_at: Option<String>,
}

/// 媒体资产
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, Selectable)]
#[diesel(table_name = crate::db::schema::media_assets)]
pub struct MediaAsset {
    pub id: String,
    pub rom_id: String,
    pub asset_type: String, // boxfront, boxback, screenshot, video, logo, manual
    pub path: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_size: Option<i64>,
    pub source_url: Option<String>,
    pub downloaded_at: String,
}

/// API 配置
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, Selectable)]
#[diesel(table_name = crate::db::schema::api_configs)]
pub struct ApiConfig {
    pub id: String,
    pub provider: String, // igdb, thegamesdb, mobygames, screenscraper, ai
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub enabled: bool,
    pub priority: i32,
}

/// 扫描目录配置
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, Selectable)]
#[diesel(table_name = crate::db::schema::scan_directories)]
pub struct ScanDirectory {
    pub id: String,
    pub path: String,
    pub system_id: Option<String>,
    pub recursive: bool,
    pub enabled: bool,
    pub last_scan: Option<String>,
}
