//! ScreenScraper Provider - 全能型 Scraper，支持 Hash 匹配

use crate::scraper::{
    ScraperProvider, ScrapeQuery, SearchResult, GameMetadata, MediaAsset,
    MediaType, Capabilities, ProviderCapability, RomHash,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const PROVIDER_ID: &str = "screenscraper";
const PROVIDER_NAME: &str = "ScreenScraper";

pub struct ScreenScraperClient {
    ssid: String,
    sspassword: String,
    devid: String,
    devpassword: String,
    softname: String,
    client: Client,
}

impl ScreenScraperClient {
    pub fn new(ssid: String, sspassword: String) -> Self {
        Self {
            ssid,
            sspassword,
            devid: "anonymous".to_string(),
            devpassword: "anonymous".to_string(),
            softname: "ModernRetroRomManager".to_string(),
            client: Client::new(),
        }
    }

    fn build_url(&self, endpoint: &str, params: Vec<(&str, String)>) -> String {
        let mut url = format!(
            "https://api.screenscraper.fr/api2/{}.php?output=json",
            endpoint
        );
        url.push_str(&format!(
            "&devid={}&devpassword={}&softname={}",
            self.devid, self.devpassword, self.softname
        ));
        url.push_str(&format!(
            "&ssid={}&sspassword={}",
            self.ssid, self.sspassword
        ));
        for (key, value) in params {
            url.push_str(&format!("&{}={}", key, urlencoding::encode(&value)));
        }
        url
    }

    /// 从 SSGame 解析元数据
    fn parse_metadata(jeu: &SSGame) -> GameMetadata {
        let name = jeu.noms.first().map(|n| n.nom.clone()).unwrap_or_default();
        let description = jeu
            .synopsis
            .iter()
            .find(|s| s.langue == "en")
            .or_else(|| jeu.synopsis.first())
            .map(|s| s.texte.clone());

        GameMetadata {
            name,
            description,
            release_date: jeu.dates.first().map(|d| d.date.clone()),
            developer: match &jeu.developpeur {
                Some(OptionValue::String(s)) => Some(s.clone()),
                _ => None,
            },
            publisher: jeu.editeur.clone(),
            genres: jeu
                .genres
                .iter()
                .filter_map(|g| {
                    g.noms
                        .iter()
                        .find(|n| n.langue == "en")
                        .or_else(|| g.noms.first())
                        .map(|n| n.text.clone())
                })
                .collect(),
            players: jeu.joueurs.clone(),
            rating: jeu.note.as_ref().and_then(|n| n.text.parse::<f64>().ok()),
        }
    }

    /// 从 SSGame 解析媒体
    fn parse_media(jeu: &SSGame) -> Vec<MediaAsset> {
        jeu.medias
            .iter()
            .filter_map(|m| {
                let media_type = match m.media_type.as_str() {
                    "box-2D" | "box-2d" => MediaType::BoxFront,
                    "box-3D" | "box-3d" => MediaType::Box3D,
                    "box-back" | "box-arriere" => MediaType::BoxBack,
                    "ss" | "screenshot" => MediaType::Screenshot,
                    "sstitle" => MediaType::TitleScreen,
                    "wheel" | "wheel-hd" => MediaType::Logo,
                    "video" | "video-normalized" => MediaType::Video,
                    "manuel" => MediaType::Manual,
                    _ => MediaType::Other,
                };

                if media_type == MediaType::Other {
                    return None;
                }

                Some(MediaAsset {
                    provider: PROVIDER_ID.to_string(),
                    url: m.url.clone(),
                    asset_type: media_type,
                    width: None,
                    height: None,
                })
            })
            .collect()
    }

    /// 调用 jeuInfos API
    async fn fetch_game_info(&self, params: Vec<(&str, String)>) -> Result<Option<SSGame>, String> {
        let url = self.build_url("jeuInfos", params);
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;

        if resp.status() == 404 || resp.status() == 430 {
            return Ok(None);
        }

        let ss_resp: SSResponse = resp.json().await.map_err(|e| e.to_string())?;

        Ok(ss_resp.response.and_then(|r| r.jeu))
    }
}

// ============================================================================
// API Response Types
// ============================================================================

#[derive(Deserialize)]
struct SSResponse {
    response: Option<SSResponseData>,
}

#[derive(Deserialize)]
struct SSResponseData {
    jeu: Option<SSGame>,
}

#[derive(Deserialize)]
struct SSGame {
    id: String,
    #[serde(default)]
    noms: Vec<SSName>,
    #[serde(default)]
    synopsis: Vec<SSSynopsis>,
    #[serde(default)]
    editeur: Option<String>,
    #[serde(default)]
    developpeur: Option<OptionValue>,
    #[serde(default)]
    dates: Vec<SSDate>,
    #[serde(default)]
    medias: Vec<SSMedia>,
    #[serde(default)]
    genres: Vec<SSGenre>,
    #[serde(default)]
    joueurs: Option<String>,
    #[serde(default)]
    note: Option<SSNote>,
}

#[derive(Deserialize)]
struct SSName {
    #[serde(rename = "text")]
    nom: String,
}

#[derive(Deserialize)]
struct SSSynopsis {
    #[serde(rename = "text")]
    texte: String,
    langue: String,
}

#[derive(Deserialize)]
struct SSDate {
    #[serde(rename = "text")]
    date: String,
}

#[derive(Deserialize)]
struct SSMedia {
    #[serde(rename = "type")]
    media_type: String,
    url: String,
}

#[derive(Deserialize)]
struct SSGenre {
    #[serde(default)]
    noms: Vec<SSGenreName>,
}

#[derive(Deserialize)]
struct SSGenreName {
    #[serde(rename = "text")]
    text: String,
    langue: String,
}

#[derive(Deserialize)]
struct SSNote {
    #[serde(rename = "text")]
    text: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OptionValue {
    String(String),
    Object(serde_json::Value),
}

// ============================================================================
// 新 ScraperProvider 实现
// ============================================================================

#[async_trait]
impl ScraperProvider for ScreenScraperClient {
    fn id(&self) -> &'static str {
        PROVIDER_ID
    }

    fn display_name(&self) -> &'static str {
        PROVIDER_NAME
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::new()
            .with(ProviderCapability::Search)
            .with(ProviderCapability::HashLookup)
            .with(ProviderCapability::Metadata)
            .with(ProviderCapability::Media)
    }

    async fn search(&self, query: &ScrapeQuery) -> Result<Vec<SearchResult>, String> {
        // ScreenScraper 使用 romnom 参数搜索
        let jeu = self
            .fetch_game_info(vec![("romnom", query.file_name.clone())])
            .await?;

        match jeu {
            Some(game) => {
                let name = game.noms.first().map(|n| n.nom.clone()).unwrap_or_default();
                let year = game.dates.first().map(|d| {
                    // 提取年份 (格式可能是 YYYY-MM-DD 或 YYYY)
                    d.date.split('-').next().unwrap_or(&d.date).to_string()
                });

                Ok(vec![SearchResult {
                    provider: PROVIDER_ID.to_string(),
                    source_id: game.id,
                    name,
                    year,
                    system: query.system.clone(),
                    thumbnail: game
                        .medias
                        .iter()
                        .find(|m| m.media_type == "box-2D" || m.media_type == "box-2d")
                        .map(|m| m.url.clone()),
                    confidence: 0.9, // 文件名匹配置信度较高
                }])
            }
            None => Ok(vec![]),
        }
    }

    async fn get_metadata(&self, source_id: &str) -> Result<GameMetadata, String> {
        let jeu = self
            .fetch_game_info(vec![("gameid", source_id.to_string())])
            .await?
            .ok_or_else(|| "Game not found".to_string())?;

        Ok(Self::parse_metadata(&jeu))
    }

    async fn get_media(&self, source_id: &str) -> Result<Vec<MediaAsset>, String> {
        let jeu = self
            .fetch_game_info(vec![("gameid", source_id.to_string())])
            .await?
            .ok_or_else(|| "Game not found".to_string())?;

        Ok(Self::parse_media(&jeu))
    }

    async fn lookup_by_hash(
        &self,
        hash: &RomHash,
        system: Option<&str>,
    ) -> Result<Option<SearchResult>, String> {
        // 构建 Hash 查询参数
        let mut params = Vec::new();

        if let Some(ref crc) = hash.crc32 {
            params.push(("crc", crc.clone()));
        }
        if let Some(ref md5) = hash.md5 {
            params.push(("md5", md5.clone()));
        }
        if let Some(ref sha1) = hash.sha1 {
            params.push(("sha1", sha1.clone()));
        }

        if params.is_empty() {
            return Ok(None);
        }

        // 添加系统 ID (如果有)
        if let Some(sys) = system {
            // TODO: 映射系统名到 ScreenScraper systemid
            params.push(("systemeid", sys.to_string()));
        }

        let jeu = self.fetch_game_info(params).await?;

        match jeu {
            Some(game) => {
                let name = game.noms.first().map(|n| n.nom.clone()).unwrap_or_default();
                Ok(Some(SearchResult {
                    provider: PROVIDER_ID.to_string(),
                    source_id: game.id,
                    name,
                    year: game.dates.first().map(|d| d.date.clone()),
                    system: system.map(String::from),
                    thumbnail: game
                        .medias
                        .iter()
                        .find(|m| m.media_type == "box-2D")
                        .map(|m| m.url.clone()),
                    confidence: 1.0, // Hash 精确匹配
                }))
            }
            None => Ok(None),
        }
    }
}
