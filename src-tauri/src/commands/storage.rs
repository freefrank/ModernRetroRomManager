use crate::config;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri_plugin_opener::OpenerExt;
use walkdir::WalkDir;

#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
pub struct DirectoryUsage {
    pub bytes: u64,
    pub files: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StorageStats {
    pub config_dir: String,
    pub cache_dir: String,
    pub total: DirectoryUsage,
    pub cache: DirectoryUsage,
    pub temporary_work: DirectoryUsage,
    pub data: DirectoryUsage,
    pub media: DirectoryUsage,
    pub incomplete: DirectoryUsage,
}

#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
pub struct CleanupResult {
    pub removed_bytes: u64,
    pub removed_files: u64,
}

fn directory_usage(path: &Path) -> DirectoryUsage {
    let mut usage = DirectoryUsage::default();
    if !path.exists() {
        return usage;
    }
    for entry in WalkDir::new(path).follow_links(false).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        if let Ok(metadata) = entry.metadata() {
            usage.files += 1;
            usage.bytes = usage.bytes.saturating_add(metadata.len());
        }
    }
    usage
}

fn is_incomplete_download(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".part"))
}

fn incomplete_usage(roots: &[PathBuf]) -> DirectoryUsage {
    let mut usage = DirectoryUsage::default();
    for root in roots {
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(root).follow_links(false).into_iter().flatten() {
            if !entry.file_type().is_file() || !is_incomplete_download(entry.path()) {
                continue;
            }
            if let Ok(metadata) = entry.metadata() {
                usage.files += 1;
                usage.bytes = usage.bytes.saturating_add(metadata.len());
            }
        }
    }
    usage
}

fn storage_stats_for(config_dir: &Path) -> StorageStats {
    let cache_dir = config_dir.join("cache");
    let temp_dir = config_dir.join("temp");
    let data_dir = config_dir.join("data");
    let media_dir = config_dir.join("media");
    StorageStats {
        config_dir: config_dir.to_string_lossy().to_string(),
        cache_dir: cache_dir.to_string_lossy().to_string(),
        total: directory_usage(config_dir),
        cache: directory_usage(&cache_dir),
        temporary_work: directory_usage(&temp_dir),
        data: directory_usage(&data_dir),
        media: directory_usage(&media_dir),
        incomplete: incomplete_usage(&[cache_dir, temp_dir]),
    }
}

fn remove_empty_directories(root: &Path) {
    let mut directories: Vec<PathBuf> = WalkDir::new(root)
        .min_depth(1)
        .follow_links(false)
        .into_iter()
        .flatten()
        .filter(|entry| entry.file_type().is_dir())
        .map(|entry| entry.into_path())
        .collect();
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for directory in directories {
        let _ = fs::remove_dir(directory);
    }
}

fn cleanup_incomplete_for(config_dir: &Path) -> CleanupResult {
    let roots = [config_dir.join("cache"), config_dir.join("temp")];
    let mut result = CleanupResult::default();
    for root in &roots {
        if !root.exists() {
            continue;
        }
        let files: Vec<PathBuf> = WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .flatten()
            .filter(|entry| entry.file_type().is_file() && is_incomplete_download(entry.path()))
            .map(|entry| entry.into_path())
            .collect();
        for file in files {
            let size = file.metadata().map(|metadata| metadata.len()).unwrap_or(0);
            if fs::remove_file(&file).is_ok() {
                result.removed_files += 1;
                result.removed_bytes = result.removed_bytes.saturating_add(size);
            }
        }
        remove_empty_directories(root);
    }
    result
}

fn clear_cache_for(config_dir: &Path) -> Result<CleanupResult, String> {
    let cache_dir = config_dir.join("cache");
    let usage = directory_usage(&cache_dir);
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)
            .map_err(|error| format!("Failed to clear cache: {error}"))?;
    }
    fs::create_dir_all(&cache_dir)
        .map_err(|error| format!("Failed to recreate cache directory: {error}"))?;
    Ok(CleanupResult {
        removed_bytes: usage.bytes,
        removed_files: usage.files,
    })
}

#[tauri::command]
pub async fn get_storage_stats() -> Result<StorageStats, String> {
    let config_dir = config::get_config_dir();
    tokio::task::spawn_blocking(move || storage_stats_for(&config_dir))
        .await
        .map_err(|error| format!("Failed to calculate storage usage: {error}"))
}

#[tauri::command]
pub async fn cleanup_incomplete_cache() -> Result<CleanupResult, String> {
    let config_dir = config::get_config_dir();
    tokio::task::spawn_blocking(move || cleanup_incomplete_for(&config_dir))
        .await
        .map_err(|error| format!("Failed to clean incomplete cache: {error}"))
}

#[tauri::command]
pub async fn clear_scraper_cache() -> Result<CleanupResult, String> {
    let config_dir = config::get_config_dir();
    tokio::task::spawn_blocking(move || clear_cache_for(&config_dir))
        .await
        .map_err(|error| format!("Failed to clear scraper cache: {error}"))?
}

#[tauri::command]
pub fn open_cache_directory(app: tauri::AppHandle) -> Result<(), String> {
    let cache_dir = config::get_config_dir().join("cache");
    fs::create_dir_all(&cache_dir).map_err(|error| error.to_string())?;
    app.opener()
        .open_path(cache_dir.to_string_lossy(), None::<&str>)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_root(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("mrrm-storage-{tag}-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn stats_keep_cache_and_work_data_separate() {
        let root = test_root("stats");
        fs::create_dir_all(root.join("cache/scraper/assets")).unwrap();
        fs::create_dir_all(root.join("temp/library/gba")).unwrap();
        fs::write(root.join("cache/scraper/assets/cover.png"), vec![0; 12]).unwrap();
        fs::write(root.join("temp/library/gba/metadata.txt"), vec![0; 7]).unwrap();
        let stats = storage_stats_for(&root);
        assert_eq!(
            stats.cache,
            DirectoryUsage {
                bytes: 12,
                files: 1
            }
        );
        assert_eq!(stats.temporary_work, DirectoryUsage { bytes: 7, files: 1 });
        assert_eq!(stats.total.bytes, 19);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn safe_cleanup_only_removes_part_files() {
        let root = test_root("cleanup");
        fs::create_dir_all(root.join("cache/scraper")).unwrap();
        fs::create_dir_all(root.join("temp/library")).unwrap();
        fs::write(root.join("cache/scraper/search.json"), vec![0; 5]).unwrap();
        fs::write(root.join("cache/scraper/search.json.part"), vec![0; 3]).unwrap();
        fs::write(root.join("temp/library/media.png.part"), vec![0; 4]).unwrap();
        let result = cleanup_incomplete_for(&root);
        assert_eq!(result.removed_files, 2);
        assert_eq!(result.removed_bytes, 7);
        assert!(root.join("cache/scraper/search.json").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clear_cache_preserves_temporary_work() {
        let root = test_root("clear");
        fs::create_dir_all(root.join("cache")).unwrap();
        fs::create_dir_all(root.join("temp")).unwrap();
        fs::write(root.join("cache/item.bin"), vec![0; 9]).unwrap();
        fs::write(root.join("temp/metadata.txt"), vec![0; 6]).unwrap();
        let result = clear_cache_for(&root).unwrap();
        assert_eq!(result.removed_bytes, 9);
        assert!(root.join("cache").is_dir());
        assert!(root.join("temp/metadata.txt").exists());
        let _ = fs::remove_dir_all(root);
    }
}
