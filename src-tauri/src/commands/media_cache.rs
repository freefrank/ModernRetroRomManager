use crate::config;
use crate::scraper::cache::stable_key;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use tokio::sync::Semaphore;

const MAX_CACHED_ASSET_BYTES: u64 = 32 * 1024 * 1024;
pub(crate) const LIBRARY_ASSET_IO_CONCURRENCY: u32 = 2;
const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "gif", "bmp", "avif", "svg", "ico",
];
pub(crate) static CACHE_IO_LIMIT: Semaphore =
    Semaphore::const_new(LIBRARY_ASSET_IO_CONCURRENCY as usize);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedAssetRecord {
    source_path: String,
    source_size: u64,
    source_modified_ns: u128,
    cached_file: String,
}

fn cache_root() -> PathBuf {
    config::get_config_dir()
        .join("cache")
        .join("library-assets")
}

fn normalize_for_key(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    if cfg!(windows) {
        normalized.to_ascii_lowercase()
    } else {
        normalized
    }
}

fn source_key(path: &Path) -> String {
    stable_key(&normalize_for_key(path))
}

fn record_path(root: &Path, key: &str) -> PathBuf {
    root.join("index").join(format!("{key}.json"))
}

fn read_record(root: &Path, key: &str) -> Option<CachedAssetRecord> {
    serde_json::from_slice(&fs::read(record_path(root, key)).ok()?).ok()
}

fn write_record(root: &Path, key: &str, record: &CachedAssetRecord) -> Result<(), String> {
    let path = record_path(root, key);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temporary = path.with_extension("json.part");
    fs::write(
        &temporary,
        serde_json::to_vec(record).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    fs::rename(temporary, path).map_err(|error| error.to_string())
}

fn cached_path_from_record(root: &Path, record: &CachedAssetRecord) -> Option<PathBuf> {
    let path = root.join(&record.cached_file);
    path.is_file().then_some(path)
}

fn is_inside(path: &Path, directory: &Path) -> bool {
    let path = normalize_for_key(path);
    let directory = normalize_for_key(directory);
    path == directory || path.starts_with(&format!("{directory}/"))
}

fn cacheable_extension(path: &Path) -> Option<String> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    IMAGE_EXTENSIONS
        .contains(&extension.as_str())
        .then_some(extension)
}

fn source_modified_ns(metadata: &fs::Metadata) -> u128 {
    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

fn cache_library_asset_for(path: &Path, root: &Path, config_dir: &Path) -> Result<PathBuf, String> {
    if is_inside(path, config_dir) {
        return Ok(path.to_path_buf());
    }
    let Some(extension) = cacheable_extension(path) else {
        return Ok(path.to_path_buf());
    };
    let key = source_key(path);
    let existing = read_record(root, &key);

    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => {
            return existing
                .as_ref()
                .and_then(|record| cached_path_from_record(root, record))
                .ok_or_else(|| format!("Failed to read library asset: {error}"));
        }
    };
    if !metadata.is_file() || metadata.len() > MAX_CACHED_ASSET_BYTES {
        return Ok(path.to_path_buf());
    }

    let modified_ns = source_modified_ns(&metadata);
    if let Some(record) = existing.as_ref().filter(|record| {
        record.source_size == metadata.len() && record.source_modified_ns == modified_ns
    }) {
        if let Some(cached) = cached_path_from_record(root, record) {
            return Ok(cached);
        }
    }

    let version = stable_key(&format!("{}:{modified_ns}", metadata.len()));
    let cached_file = format!("files/{key}-{version}.{extension}");
    let target = root.join(&cached_file);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temporary = target.with_extension(format!("{extension}.part"));
    fs::copy(path, &temporary).map_err(|error| error.to_string())?;
    if target.exists() {
        let _ = fs::remove_file(&target);
    }
    fs::rename(&temporary, &target).map_err(|error| error.to_string())?;

    if let Some(previous) = existing
        .as_ref()
        .and_then(|record| cached_path_from_record(root, record))
        .filter(|previous| previous != &target)
    {
        let _ = fs::remove_file(previous);
    }
    write_record(
        root,
        &key,
        &CachedAssetRecord {
            source_path: path.to_string_lossy().into_owned(),
            source_size: metadata.len(),
            source_modified_ns: modified_ns,
            cached_file,
        },
    )?;
    Ok(target)
}

fn cache_library_asset(path: &Path) -> Result<PathBuf, String> {
    cache_library_asset_for(path, &cache_root(), &config::get_config_dir())
}

fn existing_cached_asset(path: &Path) -> Option<PathBuf> {
    let root = cache_root();
    read_record(&root, &source_key(path))
        .as_ref()
        .and_then(|record| cached_path_from_record(&root, record))
}

#[tauri::command]
pub async fn resolve_cached_library_asset(path: String) -> Result<String, String> {
    let original = PathBuf::from(&path);
    if is_inside(&original, &config::get_config_dir()) || cacheable_extension(&original).is_none() {
        return Ok(path);
    }

    // 已缓存图片立即走本地磁盘，远端 metadata 校验和更新放到后台。
    // 这样 Samba/SD 暂时离线或响应缓慢时不会挡住卡片布局切换。
    if let Some(cached) = existing_cached_asset(&original) {
        let refresh_source = original.clone();
        tauri::async_runtime::spawn(async move {
            let Ok(_permit) = CACHE_IO_LIMIT.acquire().await else {
                return;
            };
            let _ = tokio::task::spawn_blocking(move || cache_library_asset(&refresh_source)).await;
        });
        return Ok(cached.to_string_lossy().into_owned());
    }

    let _permit = CACHE_IO_LIMIT
        .acquire()
        .await
        .map_err(|error| error.to_string())?;
    let fallback = path.clone();
    tokio::task::spawn_blocking(move || cache_library_asset(&original))
        .await
        .map_err(|error| format!("Library asset cache task failed: {error}"))?
        .map(|cached| cached.to_string_lossy().into_owned())
        .or(Ok(fallback))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_image_extensions_are_cacheable() {
        assert_eq!(
            cacheable_extension(Path::new("cover.PNG")).as_deref(),
            Some("png")
        );
        assert!(cacheable_extension(Path::new("video.mp4")).is_none());
        assert!(cacheable_extension(Path::new("game.gba")).is_none());
    }

    #[test]
    fn source_keys_are_stable_for_separator_and_case_on_windows() {
        let left = source_key(Path::new("D:\\Roms\\GBA\\cover.PNG"));
        let right = source_key(Path::new("D:/Roms/GBA/cover.PNG"));
        assert_eq!(left, right);
    }

    #[test]
    fn cached_image_is_reused_and_invalidated_by_source_metadata() {
        let root =
            std::env::temp_dir().join(format!("mrrm-library-assets-{}", uuid::Uuid::new_v4()));
        let source_dir = root.join("source");
        let config_dir = root.join("config");
        let cache_dir = config_dir.join("cache/library-assets");
        fs::create_dir_all(&source_dir).unwrap();
        let source = source_dir.join("cover.png");
        fs::write(&source, b"first").unwrap();

        let first = cache_library_asset_for(&source, &cache_dir, &config_dir).unwrap();
        let reused = cache_library_asset_for(&source, &cache_dir, &config_dir).unwrap();
        assert_eq!(first, reused);
        assert_eq!(fs::read(&reused).unwrap(), b"first");

        fs::write(&source, b"updated-content").unwrap();
        let updated = cache_library_asset_for(&source, &cache_dir, &config_dir).unwrap();
        assert_ne!(first, updated);
        assert_eq!(fs::read(&updated).unwrap(), b"updated-content");
        assert!(!first.exists());
        let _ = fs::remove_dir_all(root);
    }
}
