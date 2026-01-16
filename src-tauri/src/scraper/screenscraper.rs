use crate::scraper::{ScrapedGame, ScrapedMedia, Scraper};
use async_trait::async_trait;
use reqwest::Client;
// use serde::Deserialize;

pub struct ScreenScraperClient {
    #[allow(dead_code)]
    username: String,
    #[allow(dead_code)]
    password: String,
    #[allow(dead_code)]
    client: Client,
}

impl ScreenScraperClient {
    pub fn new(username: String, password: String) -> Self {
        Self {
            username,
            password,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Scraper for ScreenScraperClient {
    fn name(&self) -> &'static str {
        "screenscraper"
    }

    async fn search(&self, _query: &str) -> Result<Vec<ScrapedGame>, String> {
        // TODO: Implement ScreenScraper API
        Err("ScreenScraper not fully implemented yet".to_string())
    }

    async fn get_details(&self, _source_id: &str) -> Result<ScrapedGame, String> {
        Err("Not implemented".to_string())
    }

    async fn get_media(&self, _source_id: &str) -> Result<Vec<ScrapedMedia>, String> {
        // This is where videos would come from
        Ok(vec![])
    }
}
