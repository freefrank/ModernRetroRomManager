//! Scraper Tauri Commands
//!
//! 前端调用的 Scraper 相关命令

use crate::config::{get_cache_dir_for_library, get_temp_dir, get_temp_dir_for_library};
use crate::rom_service::RomInfo;
use crate::scraper::{
    cartridge_header::identify_cartridge_rom,
    dat_hash::identify_dat_rom,
    gba_header::identify_gba_rom,
    manager::{ProviderInfo, ScraperManager},
    persistence::{download_media, save_metadata_pegasus, save_metadata_pegasus_with_media},
    platform_header::{
        identify_nds_rom, identify_playstation_pbp, identify_psp_iso, identify_sega_disc,
    },
    three_ds_header::identify_3ds_rom,
    types::{GameMetadata, MediaAsset, ScrapeQuery, ScrapeResult, SearchResult},
};
use crate::system_mapping::find_mapping_by_folder;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::sync::RwLock;

use crate::settings::{get_settings, update_setting, ScraperConfig};

fn resolve_scrape_name(name: String, file_name: &str, system: &str, directory: &str) -> String {
    let path = Path::new(directory).join(file_name);
    let canonical = find_mapping_by_folder(system)
        .map(|mapping| mapping.folder_name.to_ascii_uppercase())
        .unwrap_or_else(|| system.to_ascii_uppercase());
    let identified = match canonical.as_str() {
        "GBA" => identify_gba_rom(&path)
            .ok()
            .flatten()
            .map(|value| value.scrape_name),
        "MD" | "N64" | "SFC" => identify_cartridge_rom(&path, &canonical)
            .ok()
            .flatten()
            .map(|value| value.scrape_name),
        "3DS" => identify_3ds_rom(&path)
            .ok()
            .flatten()
            .map(|value| value.scrape_name),
        "NDS" => identify_nds_rom(&path)
            .ok()
            .flatten()
            .map(|value| value.scrape_name),
        "PSP" => identify_psp_iso(&path)
            .ok()
            .flatten()
            .map(|value| value.scrape_name),
        "PS" => identify_playstation_pbp(&path)
            .ok()
            .flatten()
            .map(|value| value.scrape_name),
        "SS" | "DC" => identify_sega_disc(&path, &canonical)
            .ok()
            .flatten()
            .map(|value| value.scrape_name),
        "GB" | "GBC" | "GG" => identify_dat_rom(&canonical, &path)
            .ok()
            .flatten()
            .map(|value| value.scrape_name),
        _ => None,
    };
    identified.unwrap_or(name)
}

// ============================================================================
// State - ScraperManager 全局状态
// ============================================================================

pub struct ScraperState {
    pub manager: Arc<RwLock<ScraperManager>>,
}

impl ScraperState {
    pub fn new() -> Self {
        // 从持久化设置恢复已配置凭证的 provider
        let manager = ScraperManager::from_settings();

        Self {
            manager: Arc::new(RwLock::new(manager)),
        }
    }
}

impl Default for ScraperState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Provider 配置
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCredentials {
    pub api_key: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub developer_mode: Option<bool>,
    pub rate_limit: Option<u32>,
    pub threads: Option<u32>,
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// 获取所有可用的 provider 列表(由 ScraperManager 依据 settings 动态生成)
#[tauri::command]
pub async fn get_scraper_providers(
    state: State<'_, ScraperState>,
) -> Result<Vec<ProviderInfo>, String> {
    let manager = state.manager.read().await;
    Ok(manager.provider_infos())
}

/// 配置 provider 凭证(保存后立即重建 provider 注册表,即时生效)
#[tauri::command]
pub async fn configure_scraper_provider(
    state: State<'_, ScraperState>,
    provider_id: String,
    credentials: ProviderCredentials,
) -> Result<(), String> {
    let mut manager = state.manager.write().await;

    // 保留已有的 enabled / priority 状态
    let current_config = manager.get_credentials(&provider_id).unwrap_or_default();
    let mut new_config = ScraperConfig {
        enabled: current_config.enabled,
        priority: current_config.priority,
        ..Default::default()
    };
    new_config.rate_limit = credentials
        .rate_limit
        .unwrap_or(current_config.rate_limit)
        .clamp(1, 60);
    new_config.threads = credentials
        .threads
        .unwrap_or(current_config.threads)
        .clamp(1, 32);

    match provider_id.as_str() {
        "steamgriddb" | "thegamesdb" => {
            let api_key = credentials
                .api_key
                .or(current_config.api_key)
                .unwrap_or_default();
            if api_key.trim().is_empty() {
                return Err("请输入有效的 SteamGridDB API Key".to_string());
            }
            new_config.api_key = Some(api_key);
        }
        "screenscraper" => {
            let developer_mode = credentials
                .developer_mode
                .unwrap_or(current_config.developer_mode);
            let username = credentials
                .username
                .or(current_config.username)
                .unwrap_or_default();
            let password = credentials
                .password
                .or(current_config.password)
                .unwrap_or_default();
            let client_id = credentials
                .client_id
                .or(current_config.client_id)
                .unwrap_or_default();
            let client_secret = credentials
                .client_secret
                .or(current_config.client_secret)
                .unwrap_or_default();
            if developer_mode {
                if client_id.trim().is_empty() || client_secret.trim().is_empty() {
                    return Err(
                        "请输入 ScreenScraper Developer ID 和 Developer Password".to_string()
                    );
                }
                new_config.client_id = Some(client_id);
                new_config.client_secret = Some(client_secret);
            } else {
                if username.trim().is_empty() || password.trim().is_empty() {
                    return Err("请输入 ScreenScraper 用户名和密码".to_string());
                }
                new_config.username = Some(username);
                new_config.password = Some(password);
            }
            new_config.developer_mode = developer_mode;
        }
        _ => return Err(format!("未知的数据源: {}", provider_id)),
    }

    manager.set_credentials(&provider_id, new_config);
    // 凭证保存后重建注册表,新的凭证即时生效
    manager.rebuild_from_settings();

    Ok(())
}

/// 搜索游戏
#[tauri::command]
pub async fn scraper_search(
    state: State<'_, ScraperState>,
    name: String,
    file_name: String,
    system: Option<String>,
    directory: Option<String>,
) -> Result<Vec<SearchResult>, String> {
    let resolved_name = match (system.as_deref(), directory.as_deref()) {
        (Some(system), Some(directory)) => resolve_scrape_name(name, &file_name, system, directory),
        _ => name,
    };
    let mut query = ScrapeQuery::new(resolved_name, file_name);
    if let Some(sys) = system {
        query = query.with_system(sys);
    }

    let results = state.manager.read().await.search(&query).await;
    Ok(results)
}

/// 获取游戏元数据
#[tauri::command]
pub async fn scraper_get_metadata(
    state: State<'_, ScraperState>,
    provider_id: String,
    source_id: String,
) -> Result<GameMetadata, String> {
    let manager = state.manager.read().await;
    manager.get_metadata(&provider_id, &source_id).await
}

/// 获取游戏媒体资产
#[tauri::command]
pub async fn scraper_get_media(
    state: State<'_, ScraperState>,
    provider_id: String,
    source_id: String,
    rom_directory: Option<String>,
    system: Option<String>,
    rom_id: Option<String>,
    media_types: Option<Vec<String>>,
) -> Result<Vec<MediaAsset>, String> {
    let manager = state.manager.read().await;
    let media = manager
        .get_media(&provider_id, &source_id, media_types.as_deref())
        .await?;
    if let (Some(directory), Some(system), Some(rom_id)) = (rom_directory, system, rom_id) {
        cache_media_candidates(&manager.http_client, &directory, &system, &rom_id, &media).await?;
    }
    Ok(media)
}

#[tauri::command]
pub fn get_scraper_media_types() -> Vec<String> {
    get_settings().scraper_media_types
}

#[tauri::command]
pub async fn set_scraper_media_types(
    state: State<'_, ScraperState>,
    media_types: Vec<String>,
) -> Result<(), String> {
    let allowed = [
        "boxfront",
        "boxback",
        "box3d",
        "screenshot",
        "titlescreen",
        "logo",
        "icon",
        "hero",
        "banner",
        "video",
        "manual",
    ];
    let mut normalized: Vec<String> = media_types
        .into_iter()
        .filter(|value| allowed.contains(&value.as_str()))
        .collect();
    normalized.sort();
    normalized.dedup();
    update_setting(|settings| settings.scraper_media_types = normalized)
        .map(|_| ())
        .map_err(|error| error.to_string())?;
    state.manager.write().await.rebuild_from_settings();
    Ok(())
}

async fn cache_media_candidates(
    client: &reqwest::Client,
    rom_directory: &str,
    system: &str,
    rom_id: &str,
    media: &[MediaAsset],
) -> Result<(), String> {
    let rom_dir = Path::new(rom_directory);
    let library = rom_dir.parent().unwrap_or(rom_dir);
    let rom_stem = Path::new(rom_id)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(rom_id);
    let root = get_cache_dir_for_library(library, system).join(rom_stem);
    let mut counts = std::collections::HashMap::<(String, String), usize>::new();
    for asset in media {
        let key = (
            asset.provider.clone(),
            asset.asset_type.as_str().to_string(),
        );
        let index = counts.entry(key).or_default();
        if *index >= 3 {
            continue;
        }
        *index += 1;
        let extension = asset
            .url
            .split('?')
            .next()
            .and_then(|value| Path::new(value).extension())
            .and_then(|value| value.to_str())
            .filter(|value| value.len() <= 5)
            .unwrap_or(if asset.asset_type.as_str() == "video" {
                "mp4"
            } else {
                "jpg"
            });
        let directory = root.join(&asset.provider).join(asset.asset_type.as_str());
        if fs::create_dir_all(&directory).is_err() {
            continue;
        }
        let target = directory.join(format!(
            "candidate-{}-{:016x}.{}",
            *index,
            asset_cache_key(&asset.url),
            extension
        ));
        if target.exists() {
            continue;
        }
        let Ok(response) = client.get(&asset.url).send().await else {
            continue;
        };
        if !response.status().is_success() {
            continue;
        }
        let Ok(bytes) = response.bytes().await else {
            continue;
        };
        let temporary = target.with_extension(format!("{extension}.part"));
        if fs::write(&temporary, bytes).is_err() {
            continue;
        }
        if fs::rename(&temporary, &target).is_err() {
            let _ = fs::remove_file(&temporary);
        }
    }
    Ok(())
}

fn asset_cache_key(url: &str) -> u64 {
    url.as_bytes()
        .iter()
        .fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        })
}

fn find_cached_asset(
    rom_directory: &str,
    system: &str,
    rom_id: &str,
    asset: &MediaAsset,
) -> Option<PathBuf> {
    let rom_dir = Path::new(rom_directory);
    let library = rom_dir.parent().unwrap_or(rom_dir);
    let rom_stem = Path::new(rom_id)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(rom_id);
    let directory = get_cache_dir_for_library(library, system)
        .join(rom_stem)
        .join(&asset.provider)
        .join(asset.asset_type.as_str());
    let marker = format!("-{:016x}.", asset_cache_key(&asset.url));
    fs::read_dir(directory)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.contains(&marker))
        })
}

fn copy_cached_asset_to_temp(
    rom: &RomInfo,
    asset: &MediaAsset,
    cached: &Path,
) -> Result<(crate::scraper::MediaType, PathBuf), String> {
    let rom_dir = Path::new(&rom.directory);
    let library = rom_dir.parent().unwrap_or(rom_dir);
    let rom_stem = Path::new(&rom.file)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(&rom.name);
    let target_dir = get_temp_dir_for_library(library, &rom.system)
        .join("media")
        .join(rom_stem);
    fs::create_dir_all(&target_dir).map_err(|error| error.to_string())?;
    let extension = cached
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("png");
    let target = target_dir.join(format!("{}.{}", asset.asset_type.as_str(), extension));
    fs::copy(cached, &target).map_err(|error| error.to_string())?;
    Ok((asset.asset_type, target))
}

/// 智能 scrape - 自动匹配并聚合数据
#[tauri::command]
pub async fn scraper_auto_scrape(
    state: State<'_, ScraperState>,
    name: String,
    file_name: String,
    system: Option<String>,
) -> Result<ScrapeResult, String> {
    let manager = state.manager.read().await;

    let mut query = ScrapeQuery::new(name, file_name);
    if let Some(sys) = system {
        query = query.with_system(sys);
    }

    manager.scrape(&query).await
}

/// 启用/禁用 provider
#[tauri::command]
pub async fn scraper_set_provider_enabled(
    state: State<'_, ScraperState>,
    provider_id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut manager = state.manager.write().await;
    manager.set_enabled(&provider_id, enabled);
    Ok(())
}

/// 设置 provider 优先级
#[tauri::command]
pub async fn scraper_set_provider_priority(
    state: State<'_, ScraperState>,
    provider_id: String,
    priority: u32,
) -> Result<(), String> {
    let mut manager = state.manager.write().await;
    manager.set_priority(&provider_id, priority);
    Ok(())
}

#[tauri::command]
pub async fn test_scraper_provider(
    state: State<'_, ScraperState>,
    provider_id: String,
) -> Result<String, String> {
    state.manager.read().await.test_provider(&provider_id).await
}

// ============================================================================
// ScraperManager 控制 (占位)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ApplyScrapedDataOptions {
    pub rom_id: String,    // 文件名
    pub directory: String, // 目录
    pub system: String,    // 系统
    pub metadata: GameMetadata,
    pub selected_media: Vec<MediaAsset>,
}

#[tauri::command]
pub async fn apply_scraped_data(
    state: State<'_, ScraperState>,
    options: ApplyScrapedDataOptions,
) -> Result<(), String> {
    // 1. 构建 RomInfo (用于定位目录和文件)
    let rom = RomInfo {
        file: options.rom_id.clone(),
        directory: options.directory.clone(),
        system: options.system.clone(),
        name: options.metadata.name.clone(),
        ..Default::default()
    };

    // 2. 优先复用点开条目时已下载的候选缓存，仅对缓存缺失项访问网络。
    let mut materialized = Vec::new();
    let mut uncached = Vec::new();
    for asset in &options.selected_media {
        if let Some(cached) =
            find_cached_asset(&options.directory, &options.system, &options.rom_id, asset)
        {
            materialized.push(copy_cached_asset_to_temp(&rom, asset, &cached)?);
        } else {
            uncached.push(asset.clone());
        }
    }
    if !uncached.is_empty() {
        let manager = state.manager.read().await;
        materialized.extend(download_media(&manager.http_client, &rom, &uncached, true).await?);
    }

    // 3. 元数据和已选媒体路径一次写入，ROM 库刷新后即可显示封面。
    save_metadata_pegasus_with_media(&rom, &options.metadata, &materialized, true)?;

    Ok(())
}

/// 批量处理进度
#[derive(Debug, Clone, Serialize)]
pub struct BatchProgress {
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub finished: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BatchScrapeRom {
    pub file_name: String,
    pub search_name: String,
    #[serde(default)]
    pub system: String,
    #[serde(default)]
    pub directory: String,
}

fn select_batch_media(media: &[MediaAsset], allowed_media: &[String]) -> Vec<MediaAsset> {
    use crate::scraper::MediaType;

    let mut selected: Vec<MediaAsset> = media
        .iter()
        .filter(|asset| {
            allowed_media
                .iter()
                .any(|value| value == asset.asset_type.as_str())
        })
        .fold(Vec::new(), |mut selected, asset| {
            if !selected
                .iter()
                .any(|current| current.asset_type == asset.asset_type)
            {
                selected.push(asset.clone());
            }
            selected
        });

    // 即使用户取消了类型勾选，也维持批量抓取至少包含封面和一项其他美术资源。
    if let Some(boxart) = media
        .iter()
        .find(|asset| asset.asset_type == MediaType::BoxFront)
    {
        if !selected
            .iter()
            .any(|asset| asset.asset_type == MediaType::BoxFront)
        {
            selected.push(boxart.clone());
        }
    }
    let has_other_artwork = selected.iter().any(|asset| {
        !matches!(
            asset.asset_type,
            MediaType::BoxFront | MediaType::Video | MediaType::Manual | MediaType::Other
        )
    });
    if !has_other_artwork {
        for preferred_type in [
            MediaType::Screenshot,
            MediaType::TitleScreen,
            MediaType::Hero,
            MediaType::Logo,
            MediaType::Banner,
            MediaType::Icon,
            MediaType::BoxBack,
            MediaType::Box3D,
        ] {
            if let Some(artwork) = media
                .iter()
                .find(|asset| asset.asset_type == preferred_type)
            {
                selected.push(artwork.clone());
                break;
            }
        }
    }
    selected
}

#[tauri::command]
pub async fn batch_scrape(
    app: tauri::AppHandle,
    state: State<'_, ScraperState>,
    roms: Vec<BatchScrapeRom>,
    system: String,
    directory: String,
    provider_ids: Vec<String>,
    media_types: Option<Vec<String>>,
) -> Result<(), String> {
    if provider_ids.is_empty() {
        return Err("请至少选择一个抓取来源".to_string());
    }
    let manager_arc = Arc::clone(&state.manager);
    let total = roms.len();
    let settings = get_settings();
    let concurrency = provider_ids
        .iter()
        .filter_map(|provider_id| settings.scrapers.get(provider_id))
        .map(|config| config.threads)
        .min()
        .unwrap_or(1)
        .clamp(1, 32) as usize;
    let allowed_media = media_types.unwrap_or(settings.scraper_media_types);

    tokio::spawn(async move {
        let completed = Arc::new(AtomicUsize::new(0));
        futures::stream::iter(roms.into_iter())
            .for_each_concurrent(concurrency, |rom_item| {
                let app = app.clone();
                let manager_arc = Arc::clone(&manager_arc);
                let fallback_system = system.clone();
                let fallback_directory = directory.clone();
                let allowed_media = allowed_media.clone();
                let provider_ids = provider_ids.clone();
                let completed = Arc::clone(&completed);
                async move {
                    let file_name = rom_item.file_name;
                    let system = if rom_item.system.trim().is_empty() {
                        fallback_system
                    } else {
                        rom_item.system
                    };
                    let directory = if rom_item.directory.trim().is_empty() {
                        fallback_directory
                    } else {
                        rom_item.directory
                    };
                    let search_name = if rom_item.search_name.trim().is_empty() {
                        file_name.clone()
                    } else {
                        rom_item.search_name
                    };
                    let search_name =
                        resolve_scrape_name(search_name, &file_name, &system, &directory);

                    let _ = app.emit(
                        "batch-scrape-progress",
                        BatchProgress {
                            current: completed.load(Ordering::Relaxed) + 1,
                            total,
                            message: format!("正在抓取: {}", search_name),
                            finished: false,
                        },
                    );

                    let query = ScrapeQuery::new(search_name, file_name.clone())
                        .with_system(system.clone());

                    let scrape_res = {
                        let manager = manager_arc.read().await;
                        manager.scrape_with_providers(&query, &provider_ids).await
                    };

                    if let Ok(result) = scrape_res {
                        let rom = RomInfo {
                            file: file_name.clone(),
                            name: result.metadata.name.clone(),
                            system: system.clone(),
                            directory: directory.clone(),
                            ..Default::default()
                        };
                        let client = manager_arc.read().await.http_client.clone();
                        let selected_media = select_batch_media(&result.media, &allowed_media);
                        let mut media: Vec<_> = result
                            .media
                            .into_iter()
                            .filter(|asset| {
                                allowed_media
                                    .iter()
                                    .any(|value| value == asset.asset_type.as_str())
                            })
                            .collect();
                        for asset in &selected_media {
                            if !media.iter().any(|candidate| candidate.url == asset.url) {
                                media.push(asset.clone());
                            }
                        }
                        let _ = cache_media_candidates(
                            &client, &directory, &system, &file_name, &media,
                        )
                        .await;

                        let mut materialized = Vec::new();
                        let mut uncached = Vec::new();
                        for asset in &selected_media {
                            if let Some(cached) =
                                find_cached_asset(&directory, &system, &file_name, asset)
                            {
                                match copy_cached_asset_to_temp(&rom, asset, &cached) {
                                    Ok(copied) => materialized.push(copied),
                                    Err(_) => uncached.push(asset.clone()),
                                }
                            } else {
                                uncached.push(asset.clone());
                            }
                        }
                        if !uncached.is_empty() {
                            if let Ok(downloaded) =
                                download_media(&client, &rom, &uncached, true).await
                            {
                                materialized.extend(downloaded);
                            }
                        }
                        let _ = save_metadata_pegasus_with_media(
                            &rom,
                            &result.metadata,
                            &materialized,
                            true,
                        );
                    }
                    let current = completed.fetch_add(1, Ordering::Relaxed) + 1;
                    let _ = app.emit(
                        "batch-scrape-progress",
                        BatchProgress {
                            current,
                            total,
                            message: format!("已处理: {}", file_name),
                            finished: false,
                        },
                    );
                }
            })
            .await;

        let _ = app.emit(
            "batch-scrape-progress",
            BatchProgress {
                current: total,
                total,
                message: "批量处理完成".to_string(),
                finished: true,
            },
        );
    });

    Ok(())
}

#[cfg(test)]
mod batch_tests {
    use super::*;
    use crate::scraper::MediaType;

    fn asset(asset_type: MediaType, suffix: &str) -> MediaAsset {
        MediaAsset {
            provider: "test".into(),
            url: format!("https://example.com/{suffix}.png"),
            asset_type,
            width: None,
            height: None,
        }
    }

    #[test]
    fn batch_media_selects_one_asset_per_allowed_type() {
        let media = vec![
            asset(MediaType::Logo, "logo"),
            asset(MediaType::Screenshot, "screen-1"),
            asset(MediaType::Screenshot, "screen-2"),
            asset(MediaType::BoxFront, "box"),
            asset(MediaType::Hero, "hero"),
        ];
        let allowed = vec![
            "boxfront".into(),
            "logo".into(),
            "screenshot".into(),
            "hero".into(),
        ];
        let selected = select_batch_media(&media, &allowed);
        assert_eq!(selected.len(), 4);
        assert_eq!(
            selected
                .iter()
                .filter(|asset| asset.asset_type == MediaType::Screenshot)
                .count(),
            1
        );
    }

    #[test]
    fn batch_primary_media_falls_back_to_other_artwork() {
        let media = vec![
            asset(MediaType::BoxFront, "box"),
            asset(MediaType::Logo, "logo"),
        ];
        let selected = select_batch_media(&media, &[]);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[1].asset_type, MediaType::Logo);
    }
}

#[tauri::command]
pub async fn save_temp_metadata(
    system: String,
    directory: String,
    metadata: GameMetadata,
    rom_id: String,
) -> Result<(), String> {
    let rom = RomInfo {
        file: rom_id,
        directory,
        system,
        name: metadata.name.clone(),
        ..Default::default()
    };

    save_metadata_pegasus(&rom, &metadata, true)
}

#[tauri::command]
pub async fn delete_temp_media(
    system: String,
    rom_id: String,
    asset_type: String,
) -> Result<(), String> {
    let file_stem = Path::new(&rom_id)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&rom_id);

    let media_dir = get_temp_dir().join("media").join(&system).join(file_stem);

    if !media_dir.exists() {
        return Ok(());
    }

    // 查找匹配 asset_type 的文件 (忽略扩展名)
    for entry in fs::read_dir(media_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem == asset_type {
                    fs::remove_file(path).map_err(|e| e.to_string())?;
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct TempMediaInfo {
    pub asset_type: String,
    pub path: String,
}

#[tauri::command]
pub async fn get_temp_media_list(
    system: String,
    rom_id: String,
    rom_directory: String,
) -> Result<Vec<TempMediaInfo>, String> {
    // 从 rom_id (filename) 提取文件名主体
    let file_stem = Path::new(&rom_id)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&rom_id);

    // 计算 library_path (rom_directory 的父目录)
    let rom_dir = Path::new(&rom_directory);
    let library_path = rom_dir.parent().unwrap_or(rom_dir);

    // 媒体存储在: {temp_dir}/{library}/{system}/media/{file_stem}/
    let media_dir = get_temp_dir_for_library(library_path, &system)
        .join("media")
        .join(file_stem);

    let mut list = Vec::new();
    if media_dir.exists() && media_dir.is_dir() {
        for entry in fs::read_dir(&media_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_file() {
                // asset_type 是文件名主体 (e.g., "boxfront" from "boxfront.png")
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    list.push(TempMediaInfo {
                        asset_type: stem.to_string(),
                        path: path.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }
    Ok(list)
}

/// 导出任务进度
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct ExportProgress {
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub finished: bool,
}

#[allow(dead_code)]
fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, files);
            } else {
                files.push(path);
            }
        }
    }
}
