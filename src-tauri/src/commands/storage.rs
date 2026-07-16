use crate::config;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::BufReader;
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
    pub scraper_cache: DirectoryUsage,
    pub library_assets: DirectoryUsage,
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

#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
pub struct OrphanAssetReport {
    pub library_id: String,
    pub library_name: String,
    pub orphaned: DirectoryUsage,
    pub scanned_assets: u64,
    pub indexed_assets: u64,
    pub skipped_systems: u64,
    pub sample_files: Vec<String>,
    #[serde(skip)]
    orphaned_files: Vec<PathBuf>,
    #[serde(skip)]
    asset_roots: Vec<PathBuf>,
}

const ASSET_DIRECTORY_NAMES: &[&str] = &[
    "media",
    "images",
    "image",
    "artwork",
    "screenshots",
    "videos",
    "assets",
    "boxart",
    "covers",
    "marquees",
    "downloaded_images",
];

const ASSET_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "gif", "bmp", "svg", "avif", "mp4", "webm", "mkv", "avi", "mov",
    "mp3", "ogg", "opus", "flac", "wav", "m4a", "aac", "pdf",
];

fn is_asset_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            ASSET_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
        })
}

fn path_key(path: &Path) -> String {
    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let normalized = path.to_string_lossy().replace('\\', "/");
    #[cfg(windows)]
    return normalized.to_lowercase();
    #[cfg(not(windows))]
    normalized
}

fn referenced_path(system_dir: &Path, value: &str) -> Option<PathBuf> {
    let value = value.trim().trim_matches(['"', '\'']);
    if value.is_empty()
        || value.starts_with("http://")
        || value.starts_with("https://")
        || value.starts_with("data:")
    {
        return None;
    }
    let path = Path::new(value);
    Some(if path.is_absolute() {
        path.to_path_buf()
    } else {
        system_dir.join(value.trim_start_matches("./").trim_start_matches(".\\"))
    })
}

fn insert_reference(references: &mut HashSet<String>, system_dir: &Path, value: &Option<String>) {
    if let Some(path) = value
        .as_deref()
        .and_then(|value| referenced_path(system_dir, value))
    {
        references.insert(path_key(&path));
    }
}

fn collect_pegasus_references(
    system_dir: &Path,
    metadata_path: &Path,
    references: &mut HashSet<String>,
) -> Result<(), String> {
    let metadata = crate::scraper::pegasus::parse_pegasus_file(metadata_path)?;
    for game in metadata.games {
        for value in [
            game.box_front,
            game.box_back,
            game.box_spine,
            game.box_full,
            game.cartridge,
            game.logo,
            game.marquee,
            game.bezel,
            game.gridicon,
            game.flyer,
            game.background,
            game.music,
            game.screenshot,
            game.titlescreen,
            game.video,
        ] {
            insert_reference(references, system_dir, &value);
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct AssetGameList {
    #[serde(rename = "game", default)]
    games: Vec<AssetGame>,
}

#[derive(Debug, Deserialize)]
struct AssetGame {
    image: Option<String>,
    thumbnail: Option<String>,
    video: Option<String>,
    marquee: Option<String>,
    fanart: Option<String>,
    boxart: Option<String>,
    boxback: Option<String>,
    titleshot: Option<String>,
    screenshot: Option<String>,
    cartridge: Option<String>,
    wheel: Option<String>,
    bezel: Option<String>,
    manual: Option<String>,
    map: Option<String>,
    music: Option<String>,
}

fn collect_gamelist_references(
    system_dir: &Path,
    metadata_path: &Path,
    references: &mut HashSet<String>,
) -> Result<(), String> {
    let file = File::open(metadata_path)
        .map_err(|error| format!("读取 {} 失败: {error}", metadata_path.display()))?;
    let list: AssetGameList = quick_xml::de::from_reader(BufReader::new(file))
        .map_err(|error| format!("解析 {} 失败: {error}", metadata_path.display()))?;
    for game in list.games {
        for value in [
            game.image,
            game.thumbnail,
            game.video,
            game.marquee,
            game.fanart,
            game.boxart,
            game.boxback,
            game.titleshot,
            game.screenshot,
            game.cartridge,
            game.wheel,
            game.bezel,
            game.manual,
            game.map,
            game.music,
        ] {
            insert_reference(references, system_dir, &value);
        }
    }
    Ok(())
}

fn metadata_references(system_dir: &Path) -> Result<Option<HashSet<String>>, String> {
    let mut references = HashSet::new();
    let mut found = false;
    for file_name in ["metadata.pegasus.txt", "metadata.txt"] {
        let path = system_dir.join(file_name);
        if path.is_file() {
            found = true;
            collect_pegasus_references(system_dir, &path, &mut references)?;
        }
    }
    let gamelist = system_dir.join("gamelist.xml");
    if gamelist.is_file() {
        found = true;
        collect_gamelist_references(system_dir, &gamelist, &mut references)?;
    }
    Ok(found.then_some(references))
}

fn library_system_directories(
    library: &crate::settings::DirectoryConfig,
) -> Result<Vec<PathBuf>, String> {
    let root = PathBuf::from(&library.path);
    if !root.is_dir() {
        return Err(format!("Library 目录不存在: {}", root.display()));
    }
    if !library.is_root_directory {
        return Ok(vec![root]);
    }

    let enabled = library.indexed_folders.as_ref().map(|folders| {
        folders
            .iter()
            .map(|folder| folder.to_lowercase())
            .collect::<HashSet<_>>()
    });
    let mut directories = fs::read_dir(&root)
        .map_err(|error| format!("读取 Library 目录失败: {error}"))?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter(|entry| {
            enabled.as_ref().is_none_or(|folders| {
                folders.contains(&entry.file_name().to_string_lossy().to_lowercase())
            })
        })
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    directories.sort();
    Ok(directories)
}

fn orphaned_assets_for_library(
    library: &crate::settings::DirectoryConfig,
) -> Result<OrphanAssetReport, String> {
    let library_root = PathBuf::from(&library.path);
    let mut report = OrphanAssetReport {
        library_id: library.id.clone(),
        library_name: library.name.clone(),
        ..Default::default()
    };

    for system_dir in library_system_directories(library)? {
        let references = match metadata_references(&system_dir) {
            Ok(Some(references)) => references,
            Ok(None) | Err(_) => {
                report.skipped_systems += 1;
                continue;
            }
        };
        report.indexed_assets = report
            .indexed_assets
            .saturating_add(references.len() as u64);

        let asset_roots = fs::read_dir(&system_dir)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .filter(|entry| {
                let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
                ASSET_DIRECTORY_NAMES.contains(&name.as_str())
            })
            .map(|entry| entry.path())
            .collect::<Vec<_>>();

        for asset_root in asset_roots {
            report.asset_roots.push(asset_root.clone());
            for entry in WalkDir::new(&asset_root)
                .follow_links(false)
                .into_iter()
                .flatten()
                .filter(|entry| entry.file_type().is_file() && is_asset_file(entry.path()))
            {
                report.scanned_assets += 1;
                if references.contains(&path_key(entry.path())) {
                    continue;
                }
                let size = entry.metadata().map(|metadata| metadata.len()).unwrap_or(0);
                report.orphaned.files += 1;
                report.orphaned.bytes = report.orphaned.bytes.saturating_add(size);
                if report.sample_files.len() < 20 {
                    report.sample_files.push(
                        entry
                            .path()
                            .strip_prefix(&library_root)
                            .unwrap_or(entry.path())
                            .to_string_lossy()
                            .to_string(),
                    );
                }
                report.orphaned_files.push(entry.into_path());
            }
        }
    }
    Ok(report)
}

fn cleanup_orphaned_assets_for_library(
    library: &crate::settings::DirectoryConfig,
) -> Result<CleanupResult, String> {
    let report = orphaned_assets_for_library(library)?;
    let mut result = CleanupResult::default();
    for path in report.orphaned_files {
        let size = path.metadata().map(|metadata| metadata.len()).unwrap_or(0);
        fs::remove_file(&path)
            .map_err(|error| format!("删除未引用资源 {} 失败: {error}", path.display()))?;
        result.removed_files += 1;
        result.removed_bytes = result.removed_bytes.saturating_add(size);
    }
    for root in report.asset_roots {
        remove_empty_directories(&root);
    }
    Ok(result)
}

fn find_library(library_id: &str) -> Result<crate::settings::DirectoryConfig, String> {
    crate::settings::get_settings()
        .directories
        .into_iter()
        .find(|library| library.id == library_id)
        .ok_or_else(|| format!("未找到 Library: {library_id}"))
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
        scraper_cache: directory_usage(&cache_dir.join("scraper")),
        library_assets: directory_usage(&cache_dir.join("library-assets")),
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

fn clear_rebuildable_cache_for(config_dir: &Path) -> Result<CleanupResult, String> {
    let cache_dir = config_dir.join("cache");
    let mut usage = DirectoryUsage::default();
    for directory in [cache_dir.join("scraper"), cache_dir.join("library-assets")] {
        let current = directory_usage(&directory);
        usage.bytes = usage.bytes.saturating_add(current.bytes);
        usage.files = usage.files.saturating_add(current.files);
        if directory.exists() {
            fs::remove_dir_all(&directory)
                .map_err(|error| format!("Failed to clear cache: {error}"))?;
        }
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
pub async fn clear_rebuildable_cache() -> Result<CleanupResult, String> {
    let _asset_io_permits = crate::commands::media_cache::CACHE_IO_LIMIT
        .acquire_many(crate::commands::media_cache::LIBRARY_ASSET_IO_CONCURRENCY)
        .await
        .map_err(|error| format!("Failed to pause library asset cache: {error}"))?;
    let config_dir = config::get_config_dir();
    tokio::task::spawn_blocking(move || clear_rebuildable_cache_for(&config_dir))
        .await
        .map_err(|error| format!("Failed to clear scraper cache: {error}"))?
}

#[tauri::command]
pub async fn scan_orphaned_library_assets(library_id: String) -> Result<OrphanAssetReport, String> {
    let library = find_library(&library_id)?;
    tokio::task::spawn_blocking(move || orphaned_assets_for_library(&library))
        .await
        .map_err(|error| format!("扫描未引用资源任务失败: {error}"))?
}

#[tauri::command]
pub async fn cleanup_orphaned_library_assets(library_id: String) -> Result<CleanupResult, String> {
    let library = find_library(&library_id)?;
    tokio::task::spawn_blocking(move || cleanup_orphaned_assets_for_library(&library))
        .await
        .map_err(|error| format!("清理未引用资源任务失败: {error}"))?
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
        fs::create_dir_all(root.join("cache/library-assets/files")).unwrap();
        fs::create_dir_all(root.join("temp/library/gba")).unwrap();
        fs::write(root.join("cache/scraper/assets/cover.png"), vec![0; 12]).unwrap();
        fs::write(
            root.join("cache/library-assets/files/cover.png"),
            vec![0; 4],
        )
        .unwrap();
        fs::write(root.join("temp/library/gba/metadata.txt"), vec![0; 7]).unwrap();
        let stats = storage_stats_for(&root);
        assert_eq!(
            stats.cache,
            DirectoryUsage {
                bytes: 16,
                files: 2
            }
        );
        assert_eq!(
            stats.scraper_cache,
            DirectoryUsage {
                bytes: 12,
                files: 1
            }
        );
        assert_eq!(stats.library_assets, DirectoryUsage { bytes: 4, files: 1 });
        assert_eq!(stats.temporary_work, DirectoryUsage { bytes: 7, files: 1 });
        assert_eq!(stats.total.bytes, 23);
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
    fn clear_cache_preserves_indexes_and_temporary_work() {
        let root = test_root("clear");
        fs::create_dir_all(root.join("cache/scraper")).unwrap();
        fs::create_dir_all(root.join("cache/library-assets")).unwrap();
        fs::create_dir_all(root.join("temp")).unwrap();
        fs::write(root.join("cache/scraper/item.bin"), vec![0; 9]).unwrap();
        fs::write(root.join("cache/library-assets/cover.png"), vec![0; 4]).unwrap();
        fs::write(root.join("cache/rom-library-index.json"), vec![0; 3]).unwrap();
        fs::write(root.join("temp/metadata.txt"), vec![0; 6]).unwrap();
        let result = clear_rebuildable_cache_for(&root).unwrap();
        assert_eq!(result.removed_bytes, 13);
        assert!(root.join("cache").is_dir());
        assert!(root.join("cache/rom-library-index.json").exists());
        assert!(root.join("temp/metadata.txt").exists());
        let _ = fs::remove_dir_all(root);
    }

    fn test_library(path: &Path, is_root_directory: bool) -> crate::settings::DirectoryConfig {
        crate::settings::DirectoryConfig {
            id: "test-library".into(),
            name: "Test Library".into(),
            path: path.to_string_lossy().to_string(),
            is_root_directory,
            metadata_format: "auto".into(),
            system_id: None,
            indexed_folders: None,
        }
    }

    #[test]
    fn orphan_cleanup_keeps_assets_referenced_by_both_metadata_formats() {
        let root = test_root("orphan-assets");
        fs::create_dir_all(root.join("media/game")).unwrap();
        fs::write(root.join("media/game/boxfront.png"), vec![0; 4]).unwrap();
        fs::write(root.join("media/game/video.mp4"), vec![0; 5]).unwrap();
        fs::write(root.join("media/game/orphan.jpg"), vec![0; 7]).unwrap();
        fs::write(
            root.join("metadata.pegasus.txt"),
            "game: Test\nfile: game.zip\nassets.boxFront: media/game/boxfront.png\n",
        )
        .unwrap();
        fs::write(
            root.join("gamelist.xml"),
            "<gameList><game><path>./game.zip</path><video>./media/game/video.mp4</video></game></gameList>",
        )
        .unwrap();

        let library = test_library(&root, false);
        let report = orphaned_assets_for_library(&library).unwrap();
        assert_eq!(report.scanned_assets, 3);
        assert_eq!(report.indexed_assets, 2);
        assert_eq!(report.orphaned, DirectoryUsage { bytes: 7, files: 1 });

        let result = cleanup_orphaned_assets_for_library(&library).unwrap();
        assert_eq!(
            result,
            CleanupResult {
                removed_bytes: 7,
                removed_files: 1
            }
        );
        assert!(root.join("media/game/boxfront.png").is_file());
        assert!(root.join("media/game/video.mp4").is_file());
        assert!(!root.join("media/game/orphan.jpg").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn orphan_cleanup_skips_systems_without_valid_metadata() {
        let root = test_root("orphan-safety");
        fs::create_dir_all(root.join("gba/media/game")).unwrap();
        fs::create_dir_all(root.join("psp/media/game")).unwrap();
        fs::write(root.join("gba/media/game/cover.png"), vec![0; 4]).unwrap();
        fs::write(root.join("psp/media/game/cover.png"), vec![0; 5]).unwrap();
        fs::write(root.join("psp/gamelist.xml"), "<broken>").unwrap();

        let library = test_library(&root, true);
        let report = orphaned_assets_for_library(&library).unwrap();
        assert_eq!(report.skipped_systems, 2);
        assert_eq!(report.scanned_assets, 0);
        assert_eq!(report.orphaned.files, 0);
        assert!(root.join("gba/media/game/cover.png").is_file());
        assert!(root.join("psp/media/game/cover.png").is_file());
        let _ = fs::remove_dir_all(root);
    }
}
