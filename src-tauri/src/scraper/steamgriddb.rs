use crate::scraper::{ScrapedGame, ScrapedMedia, Scraper};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct SteamGridDBClient {
    api_key: String,
    client: Client,
}

impl SteamGridDBClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct SGDBResponse<T> {
    success: bool,
    data: T,
}

#[derive(Deserialize)]
struct SGDBGame {
    id: i64,
    name: String,
}

#[derive(Deserialize)]
struct SGDBImage {
    url: String,
    width: i32,
    height: i32,
    // style: String,
}

#[async_trait]
impl Scraper for SteamGridDBClient {
    fn name(&self) -> &'static str {
        "steamgriddb"
    }

    async fn search(&self, query: &str) -> Result<Vec<ScrapedGame>, String> {
        let url = format!("https://www.steamgriddb.com/api/v2/search/autocomplete/{}", query);
        let resp = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let sgdb_resp: SGDBResponse<Vec<SGDBGame>> = resp.json().await.map_err(|e| e.to_string())?;

        if !sgdb_resp.success {
            return Err("SGDB API error".to_string());
        }

        Ok(sgdb_resp.data.into_iter().map(|g| ScrapedGame {
            source_id: g.id.to_string(),
            name: g.name,
            description: None,
            release_date: None,
            developer: None,
            publisher: None,
            genres: vec![],
            rating: None,
        }).collect())
    }

    async fn get_details(&self, source_id: &str) -> Result<ScrapedGame, String> {
        // SGDB is mainly for images, details are limited.
        // We can use a game detail endpoint if exists, but usually search is enough for ID.
        // Let's just return a placeholder or implement if needed.
        let url = format!("https://www.steamgriddb.com/api/v2/games/id/{}", source_id);
        let resp = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let sgdb_resp: SGDBResponse<SGDBGame> = resp.json().await.map_err(|e| e.to_string())?;

        Ok(ScrapedGame {
            source_id: sgdb_resp.data.id.to_string(),
            name: sgdb_resp.data.name,
            description: None,
            release_date: None,
            developer: None,
            publisher: None,
            genres: vec![],
            rating: None,
        })
    }

    async fn get_media(&self, source_id: &str) -> Result<Vec<ScrapedMedia>, String> {
        let mut all_media = Vec::new();

        // Fetch grids (boxart)
        let endpoints = vec![
            ("grids", "boxfront"),
            ("heroes", "hero"),
            ("logos", "logo"),
            ("icons", "icon"),
        ];

        for (endpoint, asset_type) in endpoints {
            let url = format!("https://www.steamgriddb.com/api/v2/{}/game/{}", endpoint, source_id);
            let resp = self.client.get(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .send()
                .await
                .map_err(|e| e.to_string())?;

            let sgdb_resp: SGDBResponse<Vec<SGDBImage>> = resp.json().await.map_err(|e| e.to_string())?;

            if sgdb_resp.success {
                for img in sgdb_resp.data {
                    all_media.push(ScrapedMedia {
                        url: img.url,
                        asset_type: asset_type.to_string(),
                        width: Some(img.width),
                        height: Some(img.height),
                    });
                }
            }
        }

        Ok(all_media)
    }
}
