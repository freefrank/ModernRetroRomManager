use crate::db::{get_connection, models::{ApiConfig, MediaAsset, Rom, RomMetadata}, schema::{api_configs, media_assets, rom_metadata, roms}};
use crate::scraper::{Scraper, ScrapedGame, ScrapedMedia, steamgriddb::SteamGridDBClient, screenscraper::ScreenScraperClient};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tauri::{AppHandle, Manager, Emitter};
use std::path::PathBuf;

// ... (ApiConfig structs remain same) ...

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConfigInfo {
    pub id: String,
    pub provider: String,
    pub api_key: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub enabled: bool,
    pub priority: i32,
}

impl From<ApiConfig> for ApiConfigInfo {
    fn from(c: ApiConfig) -> Self {
        Self {
            id: c.id,
            provider: c.provider,
            api_key: c.api_key,
            client_id: c.username,
            client_secret: c.api_secret,
            enabled: c.enabled,
            priority: c.priority,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApiConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub enabled: Option<bool>,
}

/// 获取所有 API 配置
#[tauri::command]
pub fn get_api_configs() -> Result<Vec<ApiConfigInfo>, String> {
    let mut conn = get_connection().map_err(|e| e.to_string())?;

    let results = api_configs::table
        .load::<ApiConfig>(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(results.into_iter().map(ApiConfigInfo::from).collect())
}

/// 保存 API 配置 (Upsert)
#[tauri::command]
pub fn save_api_config(config: UpdateApiConfig) -> Result<(), String> {
    let mut conn = get_connection().map_err(|e| e.to_string())?;

    let existing: Option<ApiConfig> = api_configs::table
        .filter(api_configs::provider.eq(&config.provider))
        .first::<ApiConfig>(&mut conn)
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some(existing_config) = existing {
        diesel::update(api_configs::table.filter(api_configs::id.eq(existing_config.id)))
            .set((
                api_configs::api_key.eq(config.api_key),
                api_configs::username.eq(config.client_id.or(config.username)),
                api_configs::api_secret.eq(config.client_secret),
                api_configs::password.eq(config.password),
                api_configs::enabled.eq(config.enabled.unwrap_or(true)),
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
    } else {
        let new_config = ApiConfig {
            id: Uuid::new_v4().to_string(),
            provider: config.provider,
            api_key: config.api_key,
            api_secret: config.client_secret,
            username: config.client_id.or(config.username),
            password: config.password,
            enabled: config.enabled.unwrap_or(true),
            priority: 0,
        };

        diesel::insert_into(api_configs::table)
            .values(&new_config)
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

async fn get_scraper(provider: &str) -> Result<Box<dyn Scraper>, String> {
    let mut conn = get_connection().map_err(|e| e.to_string())?;

    let config: ApiConfig = api_configs::table
        .filter(api_configs::provider.eq(provider))
        .first::<ApiConfig>(&mut conn)
        .map_err(|_| format!("Scraper config not found for {}", provider))?;

    if !config.enabled {
        return Err(format!("Scraper {} is disabled", provider));
    }

    match provider {
        "steamgriddb" => {
            let api_key = config.api_key.ok_or("SteamGridDB API key missing")?;
            Ok(Box::new(SteamGridDBClient::new(api_key)))
        }
        "screenscraper" => {
            let username = config.username.ok_or("ScreenScraper username missing")?;
            let password = config.password.ok_or("ScreenScraper password missing")?;
            Ok(Box::new(ScreenScraperClient::new(username, password)))
        }
        _ => Err(format!("Provider {} not implemented", provider)),
    }
}

/// 搜索游戏
#[tauri::command]
pub async fn search_game(query: String, provider: String) -> Result<Vec<ScrapedGame>, String> {
    let scraper = get_scraper(&provider).await?;
    scraper.search(&query).await
}

/// 获取游戏详情
#[tauri::command]
pub async fn get_scraper_game_details(source_id: String, provider: String) -> Result<ScrapedGame, String> {
    let scraper = get_scraper(&provider).await?;
    scraper.get_details(&source_id).await
}

/// 获取游戏媒体资源
#[tauri::command]
pub async fn get_scraper_game_media(source_id: String, provider: String) -> Result<Vec<ScrapedMedia>, String> {
    let scraper = get_scraper(&provider).await?;
    scraper.get_media(&source_id).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyScrapeOptions {
    pub rom_id: String,
    pub game_details: ScrapedGame,
    pub selected_media: Vec<ScrapedMedia>,
}

/// 核心数据应用逻辑（复用）
async fn process_scrape_result(app: &AppHandle, options: &ApplyScrapeOptions) -> Result<(), String> {
    let mut conn = get_connection().map_err(|e| e.to_string())?;

    // 1. 更新元数据
    let metadata = RomMetadata {
        rom_id: options.rom_id.clone(),
        name: options.game_details.name.clone(),
        description: options.game_details.description.clone(),
        release_date: options.game_details.release_date.clone(),
        developer: options.game_details.developer.clone(),
        publisher: options.game_details.publisher.clone(),
        genre: Some(serde_json::to_string(&options.game_details.genres).unwrap_or_default()),
        players: None,
        rating: options.game_details.rating,
        region: None,
        scraper_source: Some("scraper".to_string()),
        scraped_at: Some(chrono::Local::now().naive_local().to_string()),
    };

    diesel::replace_into(rom_metadata::table)
        .values(&metadata)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    // 2. 处理媒体文件
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let media_base_path = app_data_dir.join("media").join(&options.rom_id);
    std::fs::create_dir_all(&media_base_path).map_err(|e| e.to_string())?;

    let client = reqwest::Client::new();

    for media in &options.selected_media {
        let extension = match media.asset_type.as_str() {
            "video" => "mp4",
            _ => "png", // Default to png for images
        };
        
        let filename = format!("{}.{}", media.asset_type, extension);
        let save_path = media_base_path.join(&filename);

        // 下载文件
        let resp = client.get(&media.url).send().await.map_err(|e| e.to_string())?;
        let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
        std::fs::write(&save_path, bytes).map_err(|e| e.to_string())?;

        // 存入数据库
        let new_asset = MediaAsset {
            id: Uuid::new_v4().to_string(),
            rom_id: options.rom_id.clone(),
            asset_type: media.asset_type.clone(),
            path: save_path.to_string_lossy().to_string(),
            width: media.width,
            height: media.height,
            file_size: Some(std::fs::metadata(&save_path).map(|m| m.len() as i64).unwrap_or(0)),
            source_url: Some(media.url.clone()),
            downloaded_at: chrono::Local::now().naive_local().to_string(),
        };

        diesel::insert_into(media_assets::table)
            .values(&new_asset)
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// 应用抓取到的数据 (Frontend调用)
#[tauri::command]
pub async fn apply_scraped_data(app: AppHandle, options: ApplyScrapeOptions) -> Result<(), String> {
    process_scrape_result(&app, &options).await
}

#[derive(Clone, serde::Serialize)]
struct BatchProgress {
    current: usize,
    total: usize,
    message: String,
    finished: bool,
}

/// 自动抓取单个 ROM
async fn auto_scrape_rom(app: &AppHandle, rom_id: &str, provider: &str) -> Result<(), String> {
    let mut conn = get_connection().map_err(|e| e.to_string())?;
    
    // 1. 获取 ROM 信息
    let rom = roms::table
        .filter(roms::id.eq(rom_id))
        .first::<Rom>(&mut conn)
        .map_err(|e| e.to_string())?;

    // 2. 搜索游戏 (使用文件名，去除扩展名)
    // 简单的文件名清理：去除括号内容
    let clean_name = rom.filename
        .rsplitn(2, '.').last().unwrap_or(&rom.filename) // remove extension
        .split('(').next().unwrap_or("") // remove (region) etc
        .trim();

    let scraper = get_scraper(provider).await?;
    let search_results = scraper.search(clean_name).await?;

    if let Some(game) = search_results.first() {
        // 3. 获取详情 (如果需要)
        let details = scraper.get_details(&game.source_id).await?;
        
        // 4. 获取媒体
        let media_list = scraper.get_media(&game.source_id).await?;
        
        // 5. 自动选择最佳媒体
        // 策略：每种类型取第一个
        let mut selected_media = Vec::new();
        let types = vec!["boxfront", "screenshot", "logo", "video", "hero"];
        
        for t in types {
            if let Some(m) = media_list.iter().find(|m| m.asset_type == t) {
                selected_media.push(m.clone());
            }
        }

        // 6. 应用数据
        process_scrape_result(app, &ApplyScrapeOptions {
            rom_id: rom_id.to_string(),
            game_details: details,
            selected_media,
        }).await?;
    } else {
        return Err(format!("No match found for {}", clean_name));
    }

    Ok(())
}

/// 批量 Scrape
#[tauri::command]
pub async fn batch_scrape(app: AppHandle, rom_ids: Vec<String>, provider: String) -> Result<(), String> {
    let total = rom_ids.len();
    
    for (idx, rom_id) in rom_ids.into_iter().enumerate() {
        // 发送进度
        let _ = app.emit("batch-scrape-progress", BatchProgress {
            current: idx + 1,
            total,
            message: format!("Processing item {}/{}", idx + 1, total),
            finished: false,
        });

        // 执行自动抓取，忽略单个失败
        if let Err(e) = auto_scrape_rom(&app, &rom_id, &provider).await {
            println!("Failed to auto-scrape rom {}: {}", rom_id, e);
        }
        
        // 简单的限速，避免 API 封禁 (特别是 SteamGridDB)
        // 生产环境应该用更好的 Rate Limiter
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // 完成
    let _ = app.emit("batch-scrape-progress", BatchProgress {
        current: total,
        total,
        message: "Batch scrape completed".to_string(),
        finished: true,
    });

    Ok(())
}
