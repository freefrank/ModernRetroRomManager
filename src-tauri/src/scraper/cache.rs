//! Scraper 全局持久缓存。
//!
//! 缓存键不包含 ROM 库路径，因此同一平台/游戏在不同库之间可以复用搜索、
//! 元数据、媒体清单和已下载的资产文件。

use super::{GameMetadata, MediaAsset, ScrapeQuery, SearchResult};
use crate::config::get_config_dir;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    get_config_dir().join("cache").join("scraper")
}

pub fn asset_root() -> PathBuf {
    root().join("assets")
}

pub fn stable_key(value: &str) -> String {
    format!(
        "{:016x}",
        value
            .as_bytes()
            .iter()
            .fold(0xcbf29ce484222325_u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
            })
    )
}

fn normalized_query(query: &ScrapeQuery) -> String {
    format!(
        "{}\n{}",
        query
            .system
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase(),
        query.name.trim().to_ascii_lowercase()
    )
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    let temporary = path.with_extension("json.part");
    fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    if path.exists() {
        fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    fs::rename(temporary, path).map_err(|error| error.to_string())
}

fn search_path(provider: &str, query: &ScrapeQuery) -> PathBuf {
    root()
        .join("search")
        .join(provider)
        .join(format!("{}.json", stable_key(&normalized_query(query))))
}

pub fn load_search(provider: &str, query: &ScrapeQuery) -> Option<Vec<SearchResult>> {
    read_json(&search_path(provider, query))
}

pub fn save_search(provider: &str, query: &ScrapeQuery, results: &[SearchResult]) {
    let _ = write_json(&search_path(provider, query), &results);
}

fn provider_item_path(kind: &str, provider: &str, source_id: &str) -> PathBuf {
    root()
        .join(kind)
        .join(provider)
        .join(format!("{}.json", stable_key(source_id)))
}

pub fn load_metadata(provider: &str, source_id: &str) -> Option<GameMetadata> {
    read_json(&provider_item_path("metadata", provider, source_id))
}

pub fn save_metadata(provider: &str, source_id: &str, metadata: &GameMetadata) {
    let _ = write_json(
        &provider_item_path("metadata", provider, source_id),
        metadata,
    );
}

pub fn load_media(provider: &str, source_id: &str) -> Option<Vec<MediaAsset>> {
    read_json(&provider_item_path("media", provider, source_id))
}

pub fn save_media(provider: &str, source_id: &str, media: &[MediaAsset]) {
    let _ = write_json(&provider_item_path("media", provider, source_id), &media);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_key_ignores_library_and_file_name() {
        let left = ScrapeQuery::new("Mario Kart".into(), "a.zip".into()).with_system("SNES");
        let right = ScrapeQuery::new(" mario kart ".into(), "other.sfc".into()).with_system("snes");
        assert_eq!(normalized_query(&left), normalized_query(&right));
    }

    #[test]
    fn stable_key_is_repeatable_and_sensitive() {
        assert_eq!(stable_key("same"), stable_key("same"));
        assert_ne!(stable_key("same"), stable_key("different"));
    }
}
