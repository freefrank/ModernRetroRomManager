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

/// 已知媒体扩展名(小写),用于校验从 URL 猜出的后缀,过滤掉 `.php` 之类的端点后缀。
const MEDIA_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "gif", "bmp", "avif", "svg", "ico", "mp4", "webm",
];

fn extension_from_mime(mime: &str) -> Option<&'static str> {
    Some(match mime {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" | "image/pjpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/bmp" | "image/x-bmp" | "image/x-ms-bmp" => "bmp",
        "image/avif" => "avif",
        "image/svg+xml" => "svg",
        "image/x-icon" | "image/vnd.microsoft.icon" => "ico",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        _ => return None,
    })
}

/// 根据 HTTP 响应的 `Content-Type` 推断媒体文件扩展名。
///
/// 优先用 `Content-Type`(可靠),否则回退到 URL 路径后缀(先去查询串/锚点,再校验白名单),
/// 最后按是否视频给默认值。这样 ScreenScraper 等以 `.php` 端点返回图片的 URL 不会再被存成 `.php`。
pub fn media_extension(content_type: Option<&str>, url: &str, is_video: bool) -> String {
    if let Some(extension) = content_type
        .map(|value| {
            value
                .split(';')
                .next()
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
        })
        .as_deref()
        .and_then(extension_from_mime)
    {
        return extension.to_string();
    }
    if let Some(extension) = url
        .split(['?', '#'])
        .next()
        .and_then(|value| Path::new(value).extension())
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .filter(|value| MEDIA_EXTENSIONS.contains(&value.as_str()))
    {
        return extension;
    }
    if is_video {
        "mp4".to_string()
    } else {
        "jpg".to_string()
    }
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
    // 空结果可能来自临时限流、Provider 故障或旧版错误查询，不能永久粘住。
    read_json::<Vec<SearchResult>>(&search_path(provider, query))
        .filter(|results| !results.is_empty())
}

pub fn save_search(provider: &str, query: &ScrapeQuery, results: &[SearchResult]) {
    if !results.is_empty() {
        let _ = write_json(&search_path(provider, query), &results);
    }
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

    #[test]
    fn media_extension_prefers_content_type() {
        // ScreenScraper 之类 `.php` 端点返回图片,应按 Content-Type 落成 jpg。
        assert_eq!(
            media_extension(
                Some("image/jpeg"),
                "https://api.screenscraper.fr/api2/mediaJeu.php?media=box-2D",
                false,
            ),
            "jpg"
        );
        assert_eq!(
            media_extension(Some("image/png; charset=binary"), "x.php", false),
            "png"
        );
    }

    #[test]
    fn media_extension_falls_back_to_url_then_default() {
        // 无 Content-Type 时用 URL 白名单后缀。
        assert_eq!(
            media_extension(None, "https://cdn.example.com/art.webp?token=1", false),
            "webp"
        );
        // URL 后缀不在白名单(如 `.php`)时按类型给默认值。
        assert_eq!(media_extension(None, "https://x/media.php", false), "jpg");
        assert_eq!(media_extension(None, "https://x/clip.php", true), "mp4");
    }
}
