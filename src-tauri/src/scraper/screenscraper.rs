//! ScreenScraper Provider - 全能型 Scraper，支持 Hash 匹配

use crate::scraper::rate_limit::{response_error, ProviderRateLimiter};
use crate::scraper::{
    Capabilities, GameMetadata, MediaAsset, MediaType, ProviderCapability, RomHash, ScrapeQuery,
    ScraperProvider, SearchResult,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::time::Duration;

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
            client: Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
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
            chinese_name: None,
            description,
            release_date: jeu.dates.first().map(|d| d.date.clone()),
            developer: jeu.developpeur.clone(),
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
            rating: jeu
                .note
                .as_deref()
                .and_then(|note| note.parse::<f64>().ok()),
            translated_languages: Vec::new(),
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

        let body = read_response_body(resp).await?;
        parse_game_response(&body)
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
        let body = read_response_body(resp).await?;
        parse_search_response(&body)
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
        let body = read_response_body(resp).await?;
        if !parse_user_response(&body)? {
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

        let games = parse_search_response(body).unwrap();
        let results = ScreenScraperClient::build_search_results(games, Some("gba".into()));

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source_id, "42");
        assert_eq!(results[0].name, "Valid result");
        assert_eq!(results[0].year.as_deref(), Some("2001"));
        assert_eq!(results[0].confidence, 0.9);
    }

    #[test]
    fn search_response_accepts_text_fields_returned_as_objects() {
        let body = r#"{
            "response": {
                "jeux": [{
                    "id": 42,
                    "noms": [{ "text": { "text": "Object-shaped name" } }],
                    "editeur": { "id": "7", "text": "Publisher" },
                    "developpeur": { "id": "8", "text": "Developer" },
                    "joueurs": { "text": "1-2" },
                    "note": { "text": 18 }
                }]
            }
        }"#;

        let mut games = parse_search_response(body).unwrap();
        let game = games.remove(0);
        let metadata = ScreenScraperClient::parse_metadata(&game);

        assert_eq!(game.id.as_deref(), Some("42"));
        assert_eq!(metadata.name, "Object-shaped name");
        assert_eq!(metadata.publisher.as_deref(), Some("Publisher"));
        assert_eq!(metadata.developer.as_deref(), Some("Developer"));
        assert_eq!(metadata.players.as_deref(), Some("1-2"));
        assert_eq!(metadata.rating, Some(18.0));
    }

    #[test]
    fn search_response_skips_malformed_items_and_accepts_single_objects() {
        let body = r#"{
            "response": {
                "jeux": [
                    "broken item",
                    {
                        "id": 42,
                        "noms": { "text": "Single name" },
                        "dates": { "text": 2001 },
                        "medias": [
                            { "type": "box-2D", "url": { "text": "https://example.test/box.png" } },
                            123
                        ],
                        "genres": null,
                        "note": 18
                    }
                ]
            }
        }"#;

        let games = parse_search_response(body).unwrap();
        assert_eq!(games.len(), 1);
        let metadata = ScreenScraperClient::parse_metadata(&games[0]);
        assert_eq!(metadata.name, "Single name");
        assert_eq!(metadata.release_date.as_deref(), Some("2001"));
        assert_eq!(metadata.rating, Some(18.0));
        assert_eq!(ScreenScraperClient::parse_media(&games[0]).len(), 1);
    }

    #[test]
    fn response_parsers_catch_invalid_and_error_payloads() {
        assert!(matches!(
            parse_search_response("<html>maintenance</html>"),
            Err(error) if error.contains("非 JSON")
        ));
        assert!(matches!(
            parse_game_response(""),
            Err(error) if error.contains("空响应")
        ));
        assert!(parse_user_response(r#"{"error":"invalid credentials"}"#)
            .unwrap_err()
            .contains("invalid credentials"));
        let redacted = parse_user_response(
            r#"{"error":"failed https://api.example/?devid=secret&sspassword=secret"}"#,
        )
        .unwrap_err();
        assert!(!redacted.contains("secret"));
        assert!(!redacted.contains("https://"));
        assert!(parse_search_response(r#"{"response":{"jeux":null}}"#)
            .unwrap()
            .is_empty());
        assert!(matches!(
            parse_search_response(r#"{"response":{"jeux":[{}]}}"#),
            Err(error) if error.contains("结构无法识别")
        ));
    }

    #[tokio::test]
    #[ignore = "requires configured ScreenScraper member credentials and network access"]
    async fn live_api_smoke_covers_all_used_endpoints() {
        let username = std::env::var("MRRM_SS_USERNAME")
            .expect("MRRM_SS_USERNAME must be set for the live test");
        let password = std::env::var("MRRM_SS_PASSWORD")
            .expect("MRRM_SS_PASSWORD must be set for the live test");
        let (devid, devpassword) = bundled_developer_credentials();
        let client = ScreenScraperClient::new(username, password, devid, devpassword, 1, 1);

        client.test_member_connection().await.unwrap();
        eprintln!("ScreenScraper live: ssuserInfos OK");

        let games = client
            .search_games("Super Mario Advance", "12")
            .await
            .unwrap();
        let game_id = games
            .iter()
            .find_map(|game| game.id.as_deref())
            .expect("jeuRecherche should return a game id");
        eprintln!(
            "ScreenScraper live: jeuRecherche OK ({} results)",
            games.len()
        );

        let love_hina = client
            .search_games("Love Hina Advance - Shukufuku no Kane wa Naru Kana", "12")
            .await
            .unwrap();
        assert!(
            !love_hina.is_empty(),
            "the canonical ALHJ title should be searchable"
        );

        let game = client
            .fetch_game_info(vec![("gameid", game_id.to_string())])
            .await
            .unwrap()
            .expect("jeuInfos should return the selected game");
        assert!(!game.noms.is_empty());
        eprintln!(
            "ScreenScraper live: jeuInfos OK ({} media assets)",
            game.medias.len()
        );
    }
}

// ============================================================================
// API Response Types
// ============================================================================

#[derive(Deserialize)]
struct SSGame {
    #[serde(default, deserialize_with = "deserialize_optional_text")]
    id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_lossy_vec")]
    noms: Vec<SSName>,
    #[serde(default, deserialize_with = "deserialize_lossy_vec")]
    synopsis: Vec<SSSynopsis>,
    #[serde(default, deserialize_with = "deserialize_optional_text")]
    editeur: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_text")]
    developpeur: Option<String>,
    #[serde(default, deserialize_with = "deserialize_lossy_vec")]
    dates: Vec<SSDate>,
    #[serde(default, deserialize_with = "deserialize_lossy_vec")]
    medias: Vec<SSMedia>,
    #[serde(default, deserialize_with = "deserialize_lossy_vec")]
    genres: Vec<SSGenre>,
    #[serde(default, deserialize_with = "deserialize_optional_text")]
    joueurs: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_text")]
    note: Option<String>,
}

#[derive(Deserialize)]
struct SSName {
    #[serde(rename = "text", default, deserialize_with = "deserialize_text")]
    nom: String,
}

#[derive(Deserialize)]
struct SSSynopsis {
    #[serde(rename = "text", default, deserialize_with = "deserialize_text")]
    texte: String,
    #[serde(default, deserialize_with = "deserialize_text")]
    langue: String,
}

#[derive(Deserialize)]
struct SSDate {
    #[serde(rename = "text", default, deserialize_with = "deserialize_text")]
    date: String,
}

#[derive(Deserialize)]
struct SSMedia {
    #[serde(rename = "type", default, deserialize_with = "deserialize_text")]
    media_type: String,
    #[serde(default, deserialize_with = "deserialize_text")]
    url: String,
}

#[derive(Deserialize)]
struct SSGenre {
    #[serde(default, deserialize_with = "deserialize_lossy_vec")]
    noms: Vec<SSGenreName>,
}

#[derive(Deserialize)]
struct SSGenreName {
    #[serde(rename = "text", default, deserialize_with = "deserialize_text")]
    text: String,
    #[serde(default, deserialize_with = "deserialize_text")]
    langue: String,
}

fn json_text(value: Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) => Some(value),
        serde_json::Value::Number(value) => Some(value.to_string()),
        serde_json::Value::Bool(value) => Some(value.to_string()),
        serde_json::Value::Object(mut value) => ["text", "nom", "name", "id"]
            .into_iter()
            .find_map(|key| value.remove(key).and_then(json_text)),
        serde_json::Value::Array(value) => value.into_iter().find_map(json_text),
        serde_json::Value::Null => None,
    }
}

fn values_from_list(value: Value) -> Vec<Value> {
    match value {
        Value::Array(values) => values,
        Value::Null => Vec::new(),
        Value::Object(values) if values.is_empty() => Vec::new(),
        value => vec![value],
    }
}

fn deserialize_lossy_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    Value::deserialize(deserializer).map(|value| {
        values_from_list(value)
            .into_iter()
            .filter_map(|item| serde_json::from_value(item).ok())
            .collect()
    })
}

fn deserialize_text<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Value::deserialize(deserializer).map(|value| json_text(value).unwrap_or_default())
}

fn deserialize_optional_text<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<Value>::deserialize(deserializer).map(|value| {
        value
            .and_then(json_text)
            .filter(|text| !text.trim().is_empty())
    })
}

async fn read_response_body(response: reqwest::Response) -> Result<String, String> {
    response
        .text()
        .await
        .map_err(|_| "ScreenScraper 响应读取失败".to_string())
}

fn parse_json_response(body: &str) -> Result<Value, String> {
    let body = body.trim();
    if body.is_empty() {
        return Err("ScreenScraper 返回空响应".to_string());
    }
    serde_json::from_str(body).map_err(|_| "ScreenScraper 返回非 JSON 响应".to_string())
}

fn api_error_message(value: &Value) -> Option<String> {
    let object = value.as_object()?;
    ["error", "erreur", "message"]
        .into_iter()
        .find_map(|key| object.get(key).cloned().and_then(json_text))
        .filter(|message| !message.trim().is_empty())
        .map(sanitize_api_message)
}

fn sanitize_api_message(message: String) -> String {
    let normalized = message.split_whitespace().collect::<Vec<_>>().join(" ");
    let lowercase = normalized.to_ascii_lowercase();
    if [
        "password",
        "devpassword",
        "sspassword",
        "devid=",
        "ssid=",
        "http://",
        "https://",
    ]
    .iter()
    .any(|marker| lowercase.contains(marker))
    {
        "服务端返回错误".to_string()
    } else {
        normalized.chars().take(200).collect()
    }
}

fn game_has_content(game: &SSGame) -> bool {
    game.id.is_some()
        || !game.noms.is_empty()
        || !game.synopsis.is_empty()
        || !game.dates.is_empty()
        || !game.medias.is_empty()
}

fn parse_search_response(body: &str) -> Result<Vec<SSGame>, String> {
    let root = parse_json_response(body)?;
    if let Some(message) = api_error_message(&root) {
        return Err(format!("ScreenScraper API: {message}"));
    }
    let games = root
        .get("response")
        .and_then(|response| response.get("jeux"))
        .cloned()
        .unwrap_or(Value::Null);
    let raw_games = values_from_list(games);
    let raw_count = raw_games.len();
    let parsed: Vec<SSGame> = raw_games
        .into_iter()
        .filter_map(|game| serde_json::from_value(game).ok())
        .filter(game_has_content)
        .collect();
    if raw_count > 0 && parsed.is_empty() {
        return Err("ScreenScraper 搜索结果结构无法识别".to_string());
    }
    Ok(parsed)
}

fn parse_game_response(body: &str) -> Result<Option<SSGame>, String> {
    let root = parse_json_response(body)?;
    if let Some(message) = api_error_message(&root) {
        return Err(format!("ScreenScraper API: {message}"));
    }
    let game = root
        .get("response")
        .and_then(|response| response.get("jeu"))
        .cloned();
    match game {
        None | Some(Value::Null) => Ok(None),
        Some(game) => values_from_list(game)
            .into_iter()
            .find_map(|item| serde_json::from_value(item).ok())
            .filter(game_has_content)
            .map(Some)
            .ok_or_else(|| "ScreenScraper 游戏详情结构无法识别".to_string()),
    }
}

fn parse_user_response(body: &str) -> Result<bool, String> {
    let root = parse_json_response(body)?;
    if let Some(message) = api_error_message(&root) {
        return Err(format!("ScreenScraper API: {message}"));
    }
    Ok(root
        .get("response")
        .and_then(|response| response.get("ssuser"))
        .is_some_and(|user| !user.is_null()))
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
