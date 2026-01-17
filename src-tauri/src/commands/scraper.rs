use tauri::AppHandle;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConfigInfo {
    pub id: String,
    pub provider: String,
    pub api_key: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateApiConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub enabled: Option<bool>,
}

#[tauri::command]
pub fn get_api_configs() -> Result<Vec<ApiConfigInfo>, String> {
    Ok(vec![])
    // Err("Scraper config is temporarily unavailable".to_string())
}

#[tauri::command]
pub fn save_api_config(_config: UpdateApiConfig) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn search_game(_query: String, _provider: String) -> Result<Vec<()>, String> {
     Ok(vec![])
}

#[tauri::command]
pub async fn get_scraper_game_details(_source_id: String, _provider: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn get_scraper_game_media(_source_id: String, _provider: String) -> Result<Vec<()>, String> {
    Ok(vec![])
}

#[derive(Debug, Deserialize)]
pub struct ApplyScrapeOptions {
    pub rom_id: String,
}

#[tauri::command]
pub async fn apply_scraped_data(_app: AppHandle, _options: ApplyScrapeOptions) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn batch_scrape(_app: AppHandle, _rom_ids: Vec<String>, _provider: String) -> Result<(), String> {
    Ok(())
}
