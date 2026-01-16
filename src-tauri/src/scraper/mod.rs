pub mod steamgriddb;
pub mod screenscraper;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedGame {
    pub source_id: String,
    pub name: String,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genres: Vec<String>,
    pub rating: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedMedia {
    pub url: String,
    pub asset_type: String, // boxfront, boxback, screenshot, logo, icon, hero, video
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[async_trait]
pub trait Scraper: Send + Sync {
    fn name(&self) -> &'static str;
    async fn search(&self, query: &str) -> Result<Vec<ScrapedGame>, String>;
    async fn get_details(&self, source_id: &str) -> Result<ScrapedGame, String>;
    async fn get_media(&self, source_id: &str) -> Result<Vec<ScrapedMedia>, String>;
}
