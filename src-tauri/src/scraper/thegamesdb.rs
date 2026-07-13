use crate::scraper::rate_limit::{response_error, ProviderRateLimiter};
use crate::scraper::{
    Capabilities, GameMetadata, MediaAsset, MediaType, ProviderCapability, RomHash, ScrapeQuery,
    ScraperProvider, SearchResult,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

const PROVIDER_ID: &str = "thegamesdb";

pub struct TheGamesDbClient {
    api_key: String,
    client: Client,
    limiter: ProviderRateLimiter,
    include_video: bool,
}

impl TheGamesDbClient {
    pub fn new(api_key: String, media_types: &[String]) -> Self {
        Self {
            api_key,
            client: Client::new(),
            limiter: ProviderRateLimiter::per_second(4),
            include_video: media_types.iter().any(|value| value == "video"),
        }
    }

    async fn request<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        params: &[(&str, String)],
    ) -> Result<T, String> {
        self.limiter.acquire().await;
        let mut query = vec![("apikey", self.api_key.clone())];
        query.extend_from_slice(params);
        let response = self
            .client
            .get(format!("https://api.thegamesdb.net{endpoint}"))
            .query(&query)
            .send()
            .await
            .map_err(|error| error.to_string())?;
        if !response.status().is_success() {
            return Err(response_error("TheGamesDB", response).await);
        }
        response.json().await.map_err(|error| error.to_string())
    }

    async fn games(&self, endpoint: &str, key: &str, value: String) -> Result<TgdbGames, String> {
        self.request(
            endpoint,
            &[
                (key, value),
                (
                    "fields",
                    "players,overview,rating,platform,youtube,genres,publishers,developers".into(),
                ),
                ("include", "boxart,platform".into()),
            ],
        )
        .await
    }

    fn platform_id(system: &str) -> Option<&'static str> {
        match system.to_ascii_lowercase().as_str() {
            "n64" | "nintendo64" => Some("3"),
            "gba" | "gameboyadvance" => Some("5"),
            "sfc" | "snes" | "superfamicom" => Some("6"),
            "nes" | "fc" | "famicom" => Some("7"),
            "nds" | "nintendods" => Some("8"),
            "md" | "genesis" | "megadrive" => Some("18"),
            "3ds" | "nintendo3ds" => Some("4912"),
            _ => None,
        }
    }

    fn metadata(game: &TgdbGame) -> GameMetadata {
        GameMetadata {
            name: game.game_title.clone(),
            english_name: Some(game.game_title.clone()),
            description: game.overview.clone(),
            release_date: game.release_date.clone(),
            developer: None,
            publisher: None,
            genres: Vec::new(),
            players: game.players.map(|value| value.to_string()),
            rating: game.rating.as_deref().and_then(|value| value.parse().ok()),
        }
    }

    fn image_asset(base: &str, image: &TgdbImage) -> Option<MediaAsset> {
        let asset_type = match (image.image_type.as_str(), image.side.as_deref()) {
            ("boxart", Some("front")) => MediaType::BoxFront,
            ("boxart", Some("back")) => MediaType::BoxBack,
            ("screenshot", _) => MediaType::Screenshot,
            ("titlescreen", _) => MediaType::TitleScreen,
            ("clearlogo", _) => MediaType::Logo,
            ("banner", _) => MediaType::Banner,
            ("fanart", _) => MediaType::Hero,
            _ => return None,
        };
        let (width, height) = image
            .resolution
            .as_deref()
            .and_then(|value| value.split_once('x'))
            .map(|(width, height)| (width.parse().ok(), height.parse().ok()))
            .unwrap_or((None, None));
        Some(MediaAsset {
            provider: PROVIDER_ID.into(),
            url: format!(
                "{}{}",
                base.trim_end_matches('/').to_string() + "/",
                image.filename.trim_start_matches('/')
            ),
            asset_type,
            width,
            height,
        })
    }
}

#[async_trait]
impl ScraperProvider for TheGamesDbClient {
    fn id(&self) -> &'static str {
        PROVIDER_ID
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::new()
            .with(ProviderCapability::Search)
            .with(ProviderCapability::HashLookup)
            .with(ProviderCapability::Metadata)
            .with(ProviderCapability::Media)
    }

    async fn search(&self, query: &ScrapeQuery) -> Result<Vec<SearchResult>, String> {
        let mut params = vec![
            ("name", query.name.clone()),
            ("mode", "natural".into()),
            ("fields", "players,overview,rating,platform".into()),
            ("include", "boxart,platform".into()),
        ];
        if let Some(platform_id) = query.system.as_deref().and_then(Self::platform_id) {
            params.push(("filter[platform]", platform_id.to_string()));
        }
        let response: TgdbGames = self.request("/v1.1/Games/ByGameName", &params).await?;
        Ok(response
            .data
            .games
            .into_iter()
            .enumerate()
            .map(|(index, game)| SearchResult {
                provider: PROVIDER_ID.into(),
                source_id: game.id.to_string(),
                name: game.game_title,
                year: game
                    .release_date
                    .and_then(|date| date.split('-').next().map(str::to_string)),
                system: game.platform.map(|id| id.to_string()),
                thumbnail: None,
                confidence: (0.92 - index as f32 * 0.05).max(0.5),
            })
            .collect())
    }

    async fn get_metadata(&self, source_id: &str) -> Result<GameMetadata, String> {
        let response = self
            .games("/v1/Games/ByGameID", "id", source_id.to_string())
            .await?;
        response
            .data
            .games
            .first()
            .map(Self::metadata)
            .ok_or_else(|| "TheGamesDB 未找到游戏".into())
    }

    async fn get_media(&self, source_id: &str) -> Result<Vec<MediaAsset>, String> {
        let response: TgdbImages = self
            .request("/v1/Games/Images", &[("games_id", source_id.to_string())])
            .await?;
        let images = response
            .data
            .images
            .get(source_id)
            .cloned()
            .unwrap_or_default();
        let mut media: Vec<MediaAsset> = images
            .iter()
            .filter_map(|image| Self::image_asset(&response.data.base_url.original, image))
            .collect();
        if self.include_video {
            let response: TgdbVideos = self
                .request("/v1/Games/Videos", &[("games_id", source_id.to_string())])
                .await?;
            for video in response.data.videos.get(source_id).into_iter().flatten() {
                media.push(MediaAsset {
                    provider: PROVIDER_ID.into(),
                    url: format!(
                        "{}/{}",
                        response.data.base_url.trim_end_matches('/'),
                        video.filename.trim_start_matches('/')
                    ),
                    asset_type: MediaType::Video,
                    width: None,
                    height: None,
                });
            }
        }
        Ok(media)
    }

    async fn lookup_by_hash(
        &self,
        hash: &RomHash,
        _system: Option<&str>,
    ) -> Result<Option<SearchResult>, String> {
        let Some((value, hash_type)) = hash
            .md5
            .as_ref()
            .map(|value| (value, "md5"))
            .or_else(|| hash.crc32.as_ref().map(|value| (value, "crc")))
            .or_else(|| hash.sha1.as_ref().map(|value| (value, "sha1")))
        else {
            return Ok(None);
        };
        let mut params = vec![
            ("hash", value.clone()),
            ("filter[type]", hash_type.to_string()),
            ("fields", "players,overview,rating,platform".into()),
            ("include", "boxart,platform".into()),
        ];
        if let Some(platform_id) = _system.and_then(Self::platform_id) {
            params.push(("filter[platform]", platform_id.to_string()));
        }
        let response: TgdbGames = self.request("/v1/Games/ByGameHash", &params).await?;
        Ok(response
            .data
            .games
            .into_iter()
            .next()
            .map(|game| SearchResult {
                provider: PROVIDER_ID.into(),
                source_id: game.id.to_string(),
                name: game.game_title,
                year: game.release_date,
                system: game.platform.map(|id| id.to_string()),
                thumbnail: None,
                confidence: 1.0,
            }))
    }
}

#[derive(Deserialize)]
struct TgdbGames {
    data: TgdbGameData,
}
#[derive(Deserialize)]
struct TgdbGameData {
    #[serde(default)]
    games: Vec<TgdbGame>,
}
#[derive(Deserialize)]
struct TgdbGame {
    id: u64,
    game_title: String,
    release_date: Option<String>,
    platform: Option<u64>,
    players: Option<u32>,
    overview: Option<String>,
    rating: Option<String>,
}
#[derive(Deserialize)]
struct TgdbImages {
    data: TgdbImageData,
}
#[derive(Deserialize)]
struct TgdbImageData {
    base_url: TgdbBaseUrl,
    #[serde(default)]
    images: HashMap<String, Vec<TgdbImage>>,
}
#[derive(Deserialize)]
struct TgdbBaseUrl {
    original: String,
}
#[derive(Clone, Deserialize)]
struct TgdbImage {
    #[serde(rename = "type")]
    image_type: String,
    side: Option<String>,
    filename: String,
    resolution: Option<String>,
}
#[derive(Deserialize)]
struct TgdbVideos {
    data: TgdbVideoData,
}
#[derive(Deserialize)]
struct TgdbVideoData {
    base_url: String,
    #[serde(default)]
    videos: HashMap<String, Vec<TgdbVideo>>,
}
#[derive(Deserialize)]
struct TgdbVideo {
    filename: String,
}
