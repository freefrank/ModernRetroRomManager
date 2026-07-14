//! ScreenScraper Provider - 全能型 Scraper，支持 Hash 匹配

use crate::scraper::rate_limit::{response_error, ProviderRateLimiter};
use crate::scraper::{
    Capabilities, GameMetadata, MediaAsset, MediaType, ProviderCapability, RomHash, ScrapeQuery,
    ScraperProvider, SearchResult,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const PROVIDER_ID: &str = "screenscraper";
const SOFTNAME: &str = "ModernRetroRomManager";

// ScreenScraper requires application-level developer credentials on every API call.
// Keep the bundled values out of source/binary string tables as plain text. This is
// deliberately lightweight obfuscation: a desktop binary cannot make an embedded
// shared secret truly private, but this avoids accidental disclosure via strings/logs.
const BUNDLED_DEV_USERNAME: [u8; 9] = [140, 219, 228, 4, 55, 94, 82, 139, 128];
const BUNDLED_DEV_PASSWORD: [u8; 11] = [218, 228, 194, 85, 18, 107, 81, 191, 217, 164, 133];

fn decode_bundled_credential(encoded: &[u8]) -> String {
    let mask_source = SOFTNAME.as_bytes();
    let decoded = encoded
        .iter()
        .enumerate()
        .map(|(index, byte)| {
            let mask = 0xA7u8.wrapping_add((index as u8).wrapping_mul(31))
                ^ mask_source[index % mask_source.len()];
            byte ^ mask
        })
        .collect();
    String::from_utf8(decoded).expect("bundled ScreenScraper credentials must be UTF-8")
}

pub(crate) fn bundled_developer_credentials() -> (String, String) {
    (
        decode_bundled_credential(&BUNDLED_DEV_USERNAME),
        decode_bundled_credential(&BUNDLED_DEV_PASSWORD),
    )
}

pub struct ScreenScraperClient {
    ssid: String,
    sspassword: String,
    devid: String,
    devpassword: String,
    softname: String,
    client: Client,
    limiter: ProviderRateLimiter,
}

impl ScreenScraperClient {
    pub fn new(
        ssid: String,
        sspassword: String,
        devid: String,
        devpassword: String,
        rate_limit: u32,
        threads: u32,
    ) -> Self {
        Self {
            ssid,
            sspassword,
            devid,
            devpassword,
            softname: SOFTNAME.to_string(),
            client: Client::new(),
            limiter: ProviderRateLimiter::per_second(rate_limit, threads),
        }
    }

    fn system_id(system: &str) -> Option<&'static str> {
        match system.to_ascii_lowercase().as_str() {
            "md" | "genesis" | "megadrive" => Some("1"),
            "nes" | "fc" | "famicom" => Some("3"),
            "sfc" | "snes" | "superfamicom" => Some("4"),
            "gb" | "gameboy" => Some("9"),
            "gbc" | "gameboycolor" => Some("10"),
            "gba" | "gameboyadvance" => Some("12"),
            "n64" | "nintendo64" => Some("14"),
            "nds" | "nintendods" => Some("15"),
            "3ds" | "nintendo3ds" => Some("17"),
            _ => None,
        }
    }

    fn build_url(&self, endpoint: &str, params: Vec<(&str, String)>) -> String {
        let mut url = format!(
            "https://api.screenscraper.fr/api2/{}.php?output=json",
            endpoint
        );
        url.push_str(&format!(
            "&devid={}&devpassword={}&softname={}",
            urlencoding::encode(&self.devid),
            urlencoding::encode(&self.devpassword),
            urlencoding::encode(&self.softname)
        ));
        if !self.ssid.trim().is_empty() && !self.sspassword.trim().is_empty() {
            url.push_str(&format!(
                "&ssid={}&sspassword={}",
                urlencoding::encode(&self.ssid),
                urlencoding::encode(&self.sspassword)
            ));
        }
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
            english_name: None, // ScreenScraper API 返回的多语言名称中可能包含英文，这里暂不提取
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

    fn build_search_results(games: Vec<SSGame>, system: Option<String>) -> Vec<SearchResult> {
        games
            .into_iter()
            .filter_map(|game| {
                let source_id = game.id.as_deref()?.trim().to_string();
                if source_id.is_empty() {
                    return None;
                }
                Some((game, source_id))
            })
            .enumerate()
            .map(|(index, (game, source_id))| {
                let name = game.noms.first().map(|n| n.nom.clone()).unwrap_or_default();
                let year = game.dates.first().map(|d| {
                    // 提取年份 (格式可能是 YYYY-MM-DD 或 YYYY)
                    d.date.split('-').next().unwrap_or(&d.date).to_string()
                });

                SearchResult {
                    provider: PROVIDER_ID.to_string(),
                    source_id,
                    name,
                    year,
                    system: system.clone(),
                    thumbnail: game
                        .medias
                        .iter()
                        .find(|m| m.media_type == "box-2D" || m.media_type == "box-2d")
                        .map(|m| m.url.clone()),
                    asset_count: Some(game.medias.len()),
                    confidence: (0.9_f32 - index as f32 * 0.01).max(0.6),
                }
            })
            .collect()
    }

    /// 调用 jeuInfos API
    async fn fetch_game_info(&self, params: Vec<(&str, String)>) -> Result<Option<SSGame>, String> {
        let _permit = self.limiter.acquire().await;
        let url = self.build_url("jeuInfos", params);
        let resp = self.client.get(&url).send().await.map_err(|error| {
            let kind = if error.is_timeout() {
                "请求超时"
            } else if error.is_connect() {
                "连接失败"
            } else {
                "请求失败"
            };
            // reqwest errors may include the full request URL. Never surface it
            // because ScreenScraper credentials are query parameters.
            format!("ScreenScraper {kind}")
        })?;

        if resp.status() == 404 {
            return Ok(None);
        }

        if !resp.status().is_success() {
            return Err(response_error("ScreenScraper", resp).await);
        }

        let body = resp.text().await.map_err(|e| e.to_string())?;
        if !body.trim_start().starts_with('{') {
            return Err(format!(
                "ScreenScraper API: {}",
                body.chars().take(200).collect::<String>()
            ));
        }
        let ss_resp: SSResponse = serde_json::from_str(&body).map_err(|e| e.to_string())?;

        Ok(ss_resp.response.and_then(|r| r.jeu))
    }

    async fn search_games(&self, query: &str, system_id: &str) -> Result<Vec<SSGame>, String> {
        let _permit = self.limiter.acquire().await;
        let url = self.build_url(
            "jeuRecherche",
            vec![
                ("recherche", query.to_string()),
                ("systemeid", system_id.to_string()),
            ],
        );
        let resp = self.client.get(&url).send().await.map_err(|error| {
            let kind = if error.is_timeout() {
                "请求超时"
            } else if error.is_connect() {
                "连接失败"
            } else {
                "请求失败"
            };
            format!("ScreenScraper {kind}")
        })?;
        if resp.status() == 404 {
            return Ok(Vec::new());
        }
        if !resp.status().is_success() {
            return Err(response_error("ScreenScraper", resp).await);
        }
        let body = resp.text().await.map_err(|error| error.to_string())?;
        let response: SSSearchResponse = serde_json::from_str(&body)
            .map_err(|error| format!("ScreenScraper 搜索结果解析失败: {error}"))?;
        Ok(response
            .response
            .map(|response| response.jeux)
            .unwrap_or_default())
    }

    async fn test_member_connection(&self) -> Result<String, String> {
        let _permit = self.limiter.acquire().await;
        let url = self.build_url("ssuserInfos", Vec::new());
        let resp = self.client.get(&url).send().await.map_err(|error| {
            let kind = if error.is_timeout() {
                "请求超时"
            } else if error.is_connect() {
                "连接失败"
            } else {
                "请求失败"
            };
            format!("ScreenScraper {kind}")
        })?;
        if !resp.status().is_success() {
            return Err(response_error("ScreenScraper", resp).await);
        }
        let body = resp.text().await.map_err(|error| error.to_string())?;
        let response: SSUserResponse = serde_json::from_str(&body)
            .map_err(|error| format!("ScreenScraper 账户结果解析失败: {error}"))?;
        if response
            .response
            .and_then(|response| response.ssuser)
            .is_none()
        {
            return Err("ScreenScraper 未返回用户信息".to_string());
        }
        Ok("连接正常，ScreenScraper 账号鉴权已通过".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_credentials_decode_to_expected_shape() {
        let (username, password) = bundled_developer_credentials();
        assert_eq!(username.len(), BUNDLED_DEV_USERNAME.len());
        assert_eq!(password.len(), BUNDLED_DEV_PASSWORD.len());
        assert!(username.chars().all(|value| value.is_ascii_alphanumeric()));
        assert!(password.chars().all(|value| value.is_ascii_alphanumeric()));
    }

    #[test]
    fn api_url_combines_application_and_member_credentials() {
        let client = ScreenScraperClient::new(
            "member user".into(),
            "member pass".into(),
            "developer user".into(),
            "developer pass".into(),
            1,
            1,
        );

        let url = client.build_url("jeuInfos", vec![("romnom", "Test Game".into())]);
        assert!(url.contains("devid=developer%20user"));
        assert!(url.contains("devpassword=developer%20pass"));
        assert!(url.contains("softname=ModernRetroRomManager"));
        assert!(url.contains("ssid=member%20user"));
        assert!(url.contains("sspassword=member%20pass"));
    }

    #[test]
    fn search_response_skips_games_without_an_id() {
        let body = r#"{
            "response": {
                "jeux": [
                    { "noms": [{ "text": "Incomplete result" }] },
                    { "id": null, "noms": [{ "text": "Null id" }] },
                    {
                        "id": "42",
                        "noms": [{ "text": "Valid result" }],
                        "dates": [{ "text": "2001-01-01" }]
                    }
                ]
            }
        }"#;

        let response: SSSearchResponse = serde_json::from_str(body).unwrap();
        let games = response.response.unwrap().jeux;
        let results = ScreenScraperClient::build_search_results(games, Some("gba".into()));

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source_id, "42");
        assert_eq!(results[0].name, "Valid result");
        assert_eq!(results[0].year.as_deref(), Some("2001"));
        assert_eq!(results[0].confidence, 0.9);
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
struct SSSearchResponse {
    response: Option<SSSearchResponseData>,
}

#[derive(Deserialize)]
struct SSSearchResponseData {
    #[serde(default)]
    jeux: Vec<SSGame>,
}

#[derive(Deserialize)]
struct SSUserResponse {
    response: Option<SSUserResponseData>,
}

#[derive(Deserialize)]
struct SSUserResponseData {
    ssuser: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct SSGame {
    #[serde(default)]
    id: Option<String>,
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
#[allow(dead_code)]
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

    fn capabilities(&self) -> Capabilities {
        Capabilities::new()
            .with(ProviderCapability::Search)
            .with(ProviderCapability::HashLookup)
            .with(ProviderCapability::Metadata)
            .with(ProviderCapability::Media)
    }

    async fn test_connection(&self) -> Result<String, String> {
        self.test_member_connection().await
    }

    async fn search(&self, query: &ScrapeQuery) -> Result<Vec<SearchResult>, String> {
        let system_id = query
            .system
            .as_deref()
            .and_then(Self::system_id)
            .ok_or_else(|| "ScreenScraper 缺少受支持的平台映射".to_string())?;
        let games = self.search_games(&query.name, system_id).await?;
        Ok(Self::build_search_results(games, query.system.clone()))
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
            if let Some(system_id) = Self::system_id(sys) {
                params.push(("systemeid", system_id.to_string()));
            }
        }

        let jeu = self.fetch_game_info(params).await?;

        match jeu {
            Some(game) => {
                let source_id = match game.id.as_deref().map(str::trim) {
                    Some(source_id) if !source_id.is_empty() => source_id.to_string(),
                    _ => return Ok(None),
                };
                let name = game.noms.first().map(|n| n.nom.clone()).unwrap_or_default();
                Ok(Some(SearchResult {
                    provider: PROVIDER_ID.to_string(),
                    source_id,
                    name,
                    year: game.dates.first().map(|d| d.date.clone()),
                    system: system.map(String::from),
                    thumbnail: game
                        .medias
                        .iter()
                        .find(|m| m.media_type == "box-2D")
                        .map(|m| m.url.clone()),
                    asset_count: Some(game.medias.len()),
                    confidence: 1.0, // Hash 精确匹配
                }))
            }
            None => Ok(None),
        }
    }
}
