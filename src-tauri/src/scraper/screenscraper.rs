use crate::scraper::{ScrapedGame, ScrapedMedia, Scraper};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

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
            devid: "anonymous".to_string(), // Placeholder, real apps should use their own
            devpassword: "anonymous".to_string(),
            softname: "ModernRetroRomManager".to_string(),
            client: Client::new(),
        }
    }

    fn build_url(&self, endpoint: &str, params: Vec<(&str, String)>) -> String {
        let mut url = format!("https://api.screenscraper.fr/api2/{}.php?output=json", endpoint);
        url.push_str(&format!("&devid={}&devpassword={}&softname={}", self.devid, self.devpassword, self.softname));
        url.push_str(&format!("&ssid={}&sspassword={}", self.ssid, self.sspassword));
        for (key, value) in params {
            url.push_str(&format!("&{}={}", key, urlencoding::encode(&value)));
        }
        url
    }
}

#[derive(Deserialize)]
struct SSResponse {
    response: Option<SSResponseData>,
}

#[derive(Deserialize)]
struct SSResponseData {
    jeu: Option<SSGame>,
    // jeux: Option<Vec<SSGame>>, // For list endpoints
}

#[derive(Deserialize)]
struct SSGame {
    id: String,
    noms: Vec<SSName>,
    synopsis: Vec<SSSynopsis>,
    editeur: Option<String>,
    developpeur: Option<OptionValue>,
    dates: Vec<SSDate>,
    medias: Vec<SSMedia>,
}

#[derive(Deserialize)]
struct SSName {
    nom: String,
    // region: String,
}

#[derive(Deserialize)]
struct SSSynopsis {
    texte: String,
    langue: String,
}

#[derive(Deserialize)]
struct SSDate {
    date: String,
    // region: String,
}

#[derive(Deserialize)]
struct SSMedia {
    #[serde(rename = "type")]
    media_type: String,
    url: String,
    // region: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OptionValue {
    String(String),
    Object(serde_json::Value),
}

#[async_trait]
impl Scraper for ScreenScraperClient {
    fn name(&self) -> &'static str {
        "screenscraper"
    }

    async fn search(&self, query: &str) -> Result<Vec<ScrapedGame>, String> {
        // ScreenScraper doesn't have a great "autocomplete" search.
        // We use jeuInfos.php with romnom or jeuListe.php.
        // For simple search, jeuListe.php?romnom=... is common.
        let url = self.build_url("jeuInfos", vec![("romnom", query.to_string())]);
        
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        
        if resp.status() == 404 {
            return Ok(vec![]);
        }

        let ss_resp: SSResponse = resp.json().await.map_err(|e| e.to_string())?;

        if let Some(data) = ss_resp.response {
            if let Some(jeu) = data.jeu {
                let name = jeu.noms.first().map(|n| n.nom.clone()).unwrap_or_default();
                let description = jeu.synopsis.iter()
                    .find(|s| s.langue == "en")
                    .or_else(|| jeu.synopsis.first())
                    .map(|s| s.texte.clone());

                return Ok(vec![ScrapedGame {
                    source_id: jeu.id,
                    name,
                    description,
                    release_date: jeu.dates.first().map(|d| d.date.clone()),
                    developer: match jeu.developpeur {
                        Some(OptionValue::String(s)) => Some(s),
                        _ => None,
                    },
                    publisher: jeu.editeur,
                    genres: vec![], // TODO: map genres
                    rating: None,
                }]);
            }
        }

        Ok(vec![])
    }

    async fn get_details(&self, source_id: &str) -> Result<ScrapedGame, String> {
        let url = self.build_url("jeuInfos", vec![("gameid", source_id.to_string())]);
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        let ss_resp: SSResponse = resp.json().await.map_err(|e| e.to_string())?;

        if let Some(data) = ss_resp.response {
            if let Some(jeu) = data.jeu {
                let name = jeu.noms.first().map(|n| n.nom.clone()).unwrap_or_default();
                let description = jeu.synopsis.iter()
                    .find(|s| s.langue == "en")
                    .or_else(|| jeu.synopsis.first())
                    .map(|s| s.texte.clone());

                return Ok(ScrapedGame {
                    source_id: jeu.id,
                    name,
                    description,
                    release_date: jeu.dates.first().map(|d| d.date.clone()),
                    developer: match jeu.developpeur {
                        Some(OptionValue::String(s)) => Some(s),
                        _ => None,
                    },
                    publisher: jeu.editeur,
                    genres: vec![],
                    rating: None,
                });
            }
        }

        Err("Game not found".to_string())
    }

    async fn get_media(&self, source_id: &str) -> Result<Vec<ScrapedMedia>, String> {
        let url = self.build_url("jeuInfos", vec![("gameid", source_id.to_string())]);
        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        let ss_resp: SSResponse = resp.json().await.map_err(|e| e.to_string())?;

        let mut media = Vec::new();
        if let Some(data) = ss_resp.response {
            if let Some(jeu) = data.jeu {
                for m in jeu.medias {
                    let asset_type = match m.media_type.as_str() {
                        "box-2d" | "box-3d" => "boxfront",
                        "box-back" => "boxback",
                        "screenshot" => "screenshot",
                        "video" => "video",
                        "logo" => "logo",
                        "wheel" => "logo",
                        _ => "other",
                    };
                    
                    if asset_type != "other" {
                        media.push(ScrapedMedia {
                            url: m.url,
                            asset_type: asset_type.to_string(),
                            width: None,
                            height: None,
                        });
                    }
                }
            }
        }

        Ok(media)
    }
}
