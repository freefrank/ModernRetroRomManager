//! ROM 数据服务 - 从 metadata 文件读取 ROM 信息
//!
//! 不存储到数据库，运行时直接解析 metadata 文件

use crate::commands::system::get_preset_systems_data;
use crate::ps3;
use crate::scraper::pegasus::{parse_pegasus_file, PegasusGame};
use crate::settings::get_settings;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RomInfo {
    pub file: String,
    pub name: String,
    pub description: Option<String>,
    pub summary: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<String>,
    pub players: Option<String>,
    pub release: Option<String>,
    pub rating: Option<String>,
    pub directory: String,
    pub system: String,
    pub box_front: Option<String>,
    pub box_back: Option<String>,
    pub box_spine: Option<String>,
    pub box_full: Option<String>,
    pub cartridge: Option<String>,
    pub logo: Option<String>,
    pub marquee: Option<String>,
    pub bezel: Option<String>,
    pub gridicon: Option<String>,
    pub flyer: Option<String>,
    pub background: Option<String>,
    pub music: Option<String>,
    pub screenshot: Option<String>,
    pub titlescreen: Option<String>,
    pub video: Option<String>,
    pub english_name: Option<String>,
    pub chinese_name: Option<String>,
    pub translated_languages: Vec<String>,
    /// ROM 文件大小(bytes,文件夹型 ROM 为 None,读取失败为 None)
    pub file_size: Option<u64>,
    /// ROM 文件修改时间(unix 秒,读取失败为 None)
    pub modified_at: Option<i64>,
    // 预览数据 (PegasusGame 结构比较接近原始解析结果)
    pub temp_data: Option<PegasusGame>,

    pub has_temp_metadata: bool,
}

/// 从文件系统补齐 ROM 的大小与修改时间(失败给 None,不中断扫描)
fn fill_file_metadata(rom: &mut RomInfo, rom_path: &Path) {
    if let Ok(meta) = fs::metadata(rom_path) {
        // 文件夹型 ROM(如 PS3)目录本身的 len 无意义,只对文件记录大小
        if meta.is_file() {
            rom.file_size = Some(meta.len());
        }
        rom.modified_at = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);
    }
}

/// 系统 ROM 列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRoms {
    /// 系统名称
    pub system: String,
    /// 目录路径
    pub path: String,
    /// ROMs 列表
    pub roms: Vec<RomInfo>,
}

impl From<PegasusGame> for RomInfo {
    fn from(game: PegasusGame) -> Self {
        let english_name = game
            .extra
            .get("x-mrrm-eng")
            .or_else(|| game.extra.get("x-english-name"))
            .cloned();
        let explicit_chinese_name = game
            .chinese_name
            .clone()
            .or_else(|| game.extra.get("x-mrrm-cn").cloned());
        // 旧 CN ROM Tool 把中文名写进 game，同时把英文名写入 x-mrrm-eng。
        let legacy_chinese_name = explicit_chinese_name
            .is_none()
            .then(|| game.extra.get("x-mrrm-eng").map(|_| game.name.clone()))
            .flatten();
        let name = if explicit_chinese_name.is_none() && legacy_chinese_name.is_some() {
            english_name.clone().unwrap_or_else(|| game.name.clone())
        } else {
            game.name.clone()
        };
        Self {
            file: game.file.unwrap_or_default(),
            name,
            description: game.description,
            summary: game.summary,
            developer: game.developer,
            publisher: game.publisher,
            genre: game.genre,
            players: game.players,
            release: game.release,
            rating: game.rating,
            directory: String::new(),
            system: String::new(),
            box_front: game.box_front,
            box_back: game.box_back,
            box_spine: game.box_spine,
            box_full: game.box_full,
            cartridge: game.cartridge,
            logo: game.logo,
            marquee: game.marquee,
            bezel: game.bezel,
            gridicon: game.gridicon,
            flyer: game.flyer,
            background: game.background,
            music: game.music,
            screenshot: game.screenshot,
            titlescreen: game.titlescreen,
            video: game.video,
            english_name,
            chinese_name: explicit_chinese_name.or(legacy_chinese_name),
            translated_languages: game.translated_languages,
            file_size: None,
            modified_at: None,
            temp_data: None,

            has_temp_metadata: false,
        }
    }
}

use crate::config::{get_temp_dir, get_temp_dir_for_library};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// 确保temp目录存在
fn ensure_temp_metadata_dir(library_path: &Path, system: &str) -> Result<PathBuf, String> {
    let temp_dir = get_temp_dir_for_library(library_path, system);
    fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp directory: {}", e))?;
    Ok(temp_dir)
}

/// 创建或更新metadata.pegasus.txt文件
/// 优先复制ROM目录下的现有metadata，如果不存在则创建新的
fn create_or_update_metadata(
    library_path: &Path,
    system_dir: &Path,
    system: &str,
    roms: &[RomInfo],
) -> Result<(), String> {
    let temp_dir = ensure_temp_metadata_dir(library_path, system)?;
    let metadata_path = temp_dir.join("metadata.pegasus.txt");

    // 如果temp metadata文件已存在，跳过创建
    if metadata_path.exists() {
        return Ok(());
    }

    // 检查系统目录下是否有现有的metadata文件
    let source_metadata_pegasus = system_dir.join("metadata.pegasus.txt");
    let source_metadata_txt = system_dir.join("metadata.txt");

    // 如果ROM目录下有metadata文件，复制到temp目录
    if source_metadata_pegasus.exists() {
        fs::copy(&source_metadata_pegasus, &metadata_path)
            .map_err(|e| format!("Failed to copy metadata.pegasus.txt: {}", e))?;
        return Ok(());
    } else if source_metadata_txt.exists() {
        fs::copy(&source_metadata_txt, &metadata_path)
            .map_err(|e| format!("Failed to copy metadata.txt: {}", e))?;
        return Ok(());
    }

    // 如果没有现有metadata，生成新的Pegasus格式的metadata内容
    let mut content = String::new();
    content.push_str(&format!("collection: {}\n", system));
    content.push_str("launch: {file.path}\n\n");

    for rom in roms {
        content.push_str(&format!("game: {}\n", rom.name));
        content.push_str(&format!("file: {}\n", rom.file));

        if let Some(desc) = &rom.description {
            content.push_str(&format!("description: {}\n", desc));
        }
        if let Some(dev) = &rom.developer {
            content.push_str(&format!("developer: {}\n", dev));
        }
        if let Some(pub_) = &rom.publisher {
            content.push_str(&format!("publisher: {}\n", pub_));
        }
        if let Some(genre) = &rom.genre {
            content.push_str(&format!("genre: {}\n", genre));
        }
        if let Some(release) = &rom.release {
            content.push_str(&format!("release: {}\n", release));
        }
        if let Some(rating) = &rom.rating {
            content.push_str(&format!("rating: {}\n", rating));
        }

        content.push('\n');
    }

    // 写入文件
    let mut file = fs::File::create(&metadata_path)
        .map_err(|e| format!("Failed to create metadata file: {}", e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write metadata file: {}", e))?;

    Ok(())
}

/// 更新metadata文件中特定ROM的媒体资源路径
pub fn update_rom_media_in_metadata(
    library_path: &Path,
    system: &str,
    rom_file: &str,
    asset_type: &str,
    asset_path: &str,
) -> Result<(), String> {
    let temp_dir = ensure_temp_metadata_dir(library_path, system)?;
    let metadata_path = temp_dir.join("metadata.pegasus.txt");

    // 读取现有metadata文件
    let mut metadata = if metadata_path.exists() {
        parse_pegasus_file(&metadata_path).unwrap_or_default()
    } else {
        // 如果文件不存在，创建一个新的
        crate::scraper::pegasus::PegasusMetadata {
            collections: vec![],
            games: vec![],
        }
    };

    // 查找或创建对应的ROM条目
    let game = metadata
        .games
        .iter_mut()
        .find(|g| g.file == Some(rom_file.to_string()));

    if let Some(game) = game {
        // 更新现有ROM的媒体资源
        match asset_type {
            "boxfront" => game.box_front = Some(asset_path.to_string()),
            "boxback" => game.box_back = Some(asset_path.to_string()),
            "logo" => game.logo = Some(asset_path.to_string()),
            "screenshot" => game.screenshot = Some(asset_path.to_string()),
            "video" => game.video = Some(asset_path.to_string()),
            "background" => game.background = Some(asset_path.to_string()),
            _ => {}
        }
    } else {
        // 创建新的ROM条目
        let mut new_game = PegasusGame {
            name: rom_file.to_string(),
            file: Some(rom_file.to_string()),
            ..Default::default()
        };

        match asset_type {
            "boxfront" => new_game.box_front = Some(asset_path.to_string()),
            "boxback" => new_game.box_back = Some(asset_path.to_string()),
            "logo" => new_game.logo = Some(asset_path.to_string()),
            "screenshot" => new_game.screenshot = Some(asset_path.to_string()),
            "video" => new_game.video = Some(asset_path.to_string()),
            "background" => new_game.background = Some(asset_path.to_string()),
            _ => {}
        }

        metadata.games.push(new_game);
    }

    // 重新生成metadata文件内容
    let mut content = String::new();

    // 输出collection信息（如果有）
    if !metadata.collections.is_empty() {
        for collection in &metadata.collections {
            content.push_str(&format!("collection: {}\n", collection.name));
        }
    } else {
        // 如果没有collection，使用系统名称
        content.push_str(&format!("collection: {}\n", system));
    }
    content.push_str("launch: {file.path}\n\n");

    for game in &metadata.games {
        content.push_str(&format!("game: {}\n", game.name));
        if let Some(file) = &game.file {
            content.push_str(&format!("file: {}\n", file));
        }
        if let Some(desc) = &game.description {
            content.push_str(&format!("description: {}\n", desc));
        }
        if let Some(dev) = &game.developer {
            content.push_str(&format!("developer: {}\n", dev));
        }
        if let Some(pub_) = &game.publisher {
            content.push_str(&format!("publisher: {}\n", pub_));
        }
        if let Some(genre) = &game.genre {
            content.push_str(&format!("genre: {}\n", genre));
        }
        if let Some(release) = &game.release {
            content.push_str(&format!("release: {}\n", release));
        }
        if let Some(rating) = &game.rating {
            content.push_str(&format!("rating: {}\n", rating));
        }

        // 媒体资源
        if let Some(box_front) = &game.box_front {
            content.push_str(&format!("assets.boxFront: {}\n", box_front));
        }
        if let Some(box_back) = &game.box_back {
            content.push_str(&format!("assets.boxBack: {}\n", box_back));
        }
        if let Some(logo) = &game.logo {
            content.push_str(&format!("assets.logo: {}\n", logo));
        }
        if let Some(screenshot) = &game.screenshot {
            content.push_str(&format!("assets.screenshot: {}\n", screenshot));
        }
        if let Some(video) = &game.video {
            content.push_str(&format!("assets.video: {}\n", video));
        }
        if let Some(background) = &game.background {
            content.push_str(&format!("assets.background: {}\n", background));
        }

        content.push('\n');
    }

    // 写入文件
    let mut file = fs::File::create(&metadata_path)
        .map_err(|e| format!("Failed to write metadata file: {}", e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write metadata file: {}", e))?;

    Ok(())
}

/// 尝试加载并应用临时元数据
pub(crate) fn apply_temp_metadata(roms: &mut [RomInfo], library_path: &Path, system: &str) {
    // 使用新的目录结构: temp/{library}/{system}/metadata.pegasus.txt
    let temp_dir = get_temp_dir_for_library(library_path, system);
    let temp_metadata_path = temp_dir.join("metadata.pegasus.txt");

    // 兼容旧的目录结构: temp/{system}/metadata.txt
    let legacy_temp_metadata_path = get_temp_dir().join(system).join("metadata.txt");

    // 优先使用新结构，如果不存在则尝试旧结构
    let (metadata_path, base_dir) = if temp_metadata_path.exists() {
        (temp_metadata_path, temp_dir.clone())
    } else if legacy_temp_metadata_path.exists() {
        (legacy_temp_metadata_path, get_temp_dir().join(system))
    } else {
        return;
    };

    if let Ok(temp_metadata) = parse_pegasus_file(&metadata_path) {
        let games_by_file: std::collections::HashMap<String, PegasusGame> = temp_metadata
            .games
            .into_iter()
            .filter_map(|game| game.file.clone().map(|file| (file, game)))
            .collect();
        let media_index = build_temp_media_index(&base_dir);
        for rom in roms {
            if let Some(temp_game) = games_by_file.get(&rom.file) {
                rom.has_temp_metadata = true;
                // 克隆并解析媒体路径为绝对路径
                let mut resolved_game = temp_game.clone();

                // 解析所有媒体路径
                let resolve = |path: &mut Option<String>| {
                    if let Some(p) = path.as_ref() {
                        if !p.starts_with("http") && !Path::new(p).is_absolute() {
                            *path = Some(base_dir.join(p).to_string_lossy().to_string());
                        }
                    }
                };

                resolve(&mut resolved_game.box_front);
                resolve(&mut resolved_game.box_back);
                resolve(&mut resolved_game.logo);
                resolve(&mut resolved_game.screenshot);
                resolve(&mut resolved_game.video);
                resolve(&mut resolved_game.background);

                fill_missing_temp_media_from_index(&mut resolved_game, &media_index, &rom.file);

                rom.temp_data = Some(resolved_game);
            }
        }
    }
}

type TempMediaIndex = std::collections::HashMap<String, std::collections::HashMap<String, String>>;

fn build_temp_media_index(base_dir: &Path) -> TempMediaIndex {
    let mut index = TempMediaIndex::new();
    let Ok(game_dirs) = fs::read_dir(base_dir.join("media")) else {
        return index;
    };
    for game_dir in game_dirs.filter_map(Result::ok) {
        let path = game_dir.path();
        if !path.is_dir() {
            continue;
        }
        let key = game_dir.file_name().to_string_lossy().to_ascii_lowercase();
        let Ok(assets) = fs::read_dir(&path) else {
            continue;
        };
        let entries = index.entry(key).or_default();
        for asset in assets.filter_map(Result::ok) {
            let asset_path = asset.path();
            if !asset_path.is_file() {
                continue;
            }
            if let Some(stem) = asset_path.file_stem().and_then(|value| value.to_str()) {
                entries.insert(
                    stem.to_ascii_lowercase(),
                    asset_path.to_string_lossy().into_owned(),
                );
            }
        }
    }
    index
}

fn find_temp_media_in_index(
    index: &TempMediaIndex,
    rom_file: &str,
    asset_type: &str,
) -> Option<String> {
    let rom_stem = Path::new(rom_file)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(rom_file);
    index
        .get(&rom_stem.to_ascii_lowercase())?
        .get(&asset_type.to_ascii_lowercase())
        .cloned()
}

fn fill_missing_temp_media_from_index(
    game: &mut PegasusGame,
    index: &TempMediaIndex,
    rom_file: &str,
) {
    let missing_local_asset = |asset: &Option<String>| {
        asset
            .as_ref()
            .is_none_or(|path| !path.starts_with("http") && !Path::new(path).is_file())
    };

    if missing_local_asset(&game.box_front) {
        game.box_front = find_temp_media_in_index(index, rom_file, "boxfront");
    }
    if missing_local_asset(&game.box_back) {
        game.box_back = find_temp_media_in_index(index, rom_file, "boxback");
    }
    if missing_local_asset(&game.logo) {
        game.logo = find_temp_media_in_index(index, rom_file, "logo");
    }
    if missing_local_asset(&game.screenshot) {
        game.screenshot = find_temp_media_in_index(index, rom_file, "screenshot");
    }
    if missing_local_asset(&game.titlescreen) {
        game.titlescreen = find_temp_media_in_index(index, rom_file, "titlescreen");
    }
    if missing_local_asset(&game.video) {
        game.video = find_temp_media_in_index(index, rom_file, "video");
    }
    if missing_local_asset(&game.background) {
        game.background = find_temp_media_in_index(index, rom_file, "hero");
    }
}

#[cfg(test)]
fn fill_missing_temp_media(game: &mut PegasusGame, base_dir: &Path, rom_file: &str) {
    let index = build_temp_media_index(base_dir);
    fill_missing_temp_media_from_index(game, &index, rom_file);
}

/// 尝试从临时元数据加载 ROM 列表
fn try_load_from_temp_metadata(
    library_path: &Path,
    rom_dir: &Path,
    system_name: &str,
) -> Option<Vec<RomInfo>> {
    // 1. 获取临时元数据路径
    let temp_dir = get_temp_dir_for_library(library_path, system_name);
    let metadata_path = temp_dir.join("metadata.pegasus.txt");

    // 兼容旧路径
    let legacy_path = get_temp_dir().join(system_name).join("metadata.txt");

    let (path_to_read, base_dir) = if metadata_path.exists() {
        (metadata_path, temp_dir)
    } else if legacy_path.exists() {
        let legacy_dir = get_temp_dir().join(system_name);
        (legacy_path, legacy_dir)
    } else {
        return None;
    };

    println!("[DEBUG] 发现临时元数据，尝试加载: {:?}", path_to_read);

    if let Ok(metadata) = parse_pegasus_file(&path_to_read) {
        // Temp metadata is authoritative for scraped values, but it may contain
        // entries without artwork. Keep a lightweight index of the original ES
        // media so those gaps can still fall back to <boxart>/<marquee> without
        // rescanning every ROM file on a network library.
        let source_media = read_emulationstation_media_assets(rom_dir);
        let media_index = build_temp_media_index(&base_dir);
        let roms: Vec<RomInfo> = metadata
            .games
            .into_iter()
            .filter_map(|g| {
                // 必须有文件名
                let file_name = g.file.clone()?;
                let rom_path = rom_dir.join(&file_name);

                // 验证文件存在性 (快速检查)
                if !rom_path.exists() {
                    // 如果是 PS3 文件夹形式，也要检查
                    if !rom_dir.join(&file_name).is_dir() {
                        return None;
                    }
                }

                let mut rom: RomInfo = g.into();
                rom.directory = rom_dir.to_string_lossy().to_string();
                rom.system = system_name.to_string();
                // 标记为已拥有临时数据
                rom.has_temp_metadata = true;
                fill_file_metadata(&mut rom, &rom_path);

                // 解析媒体路径为绝对路径
                let resolve = |path: &mut Option<String>| {
                    if let Some(p) = path.as_ref() {
                        if !p.starts_with("http") && !Path::new(p).is_absolute() {
                            *path = Some(base_dir.join(p).to_string_lossy().to_string());
                        }
                    }
                };

                resolve(&mut rom.box_front);
                resolve(&mut rom.box_back);
                resolve(&mut rom.logo);
                resolve(&mut rom.screenshot);
                resolve(&mut rom.video);
                resolve(&mut rom.background);
                resolve(&mut rom.titlescreen);
                resolve(&mut rom.marquee);
                resolve(&mut rom.bezel);
                resolve(&mut rom.gridicon);
                resolve(&mut rom.flyer);
                resolve(&mut rom.music);
                resolve(&mut rom.cartridge);
                resolve(&mut rom.box_spine);
                resolve(&mut rom.box_full);

                let mut temp_game = PegasusGame {
                    name: rom.name.clone(),
                    file: Some(rom.file.clone()),
                    description: rom.description.clone(),
                    developer: rom.developer.clone(),
                    publisher: rom.publisher.clone(),
                    genre: rom.genre.clone(),
                    release: rom.release.clone(),
                    rating: rom.rating.clone(),
                    box_front: rom.box_front.clone(),
                    box_back: rom.box_back.clone(),
                    logo: rom.logo.clone(),
                    screenshot: rom.screenshot.clone(),
                    titlescreen: rom.titlescreen.clone(),
                    video: rom.video.clone(),
                    background: rom.background.clone(),
                    ..Default::default()
                };
                fill_missing_temp_media_from_index(&mut temp_game, &media_index, &rom.file);

                if let Some(source) = source_media.get(&rom.file) {
                    if temp_game.box_front.is_none() {
                        temp_game.box_front = source.box_front.clone();
                    }
                    if temp_game.logo.is_none() {
                        temp_game.logo = source.logo.clone();
                    }
                    if rom.box_front.is_none() {
                        rom.box_front = source.box_front.clone();
                    }
                    if rom.logo.is_none() {
                        rom.logo = source.logo.clone();
                    }
                    if rom.marquee.is_none() {
                        rom.marquee = source.logo.clone();
                    }
                }
                rom.temp_data = Some(temp_game);

                Some(rom)
            })
            .collect();

        if !roms.is_empty() {
            println!("[DEBUG] 从临时元数据成功加载 {} 个 ROM", roms.len());
            return Some(roms);
        }
    }

    None
}

#[derive(Debug, Default)]
struct EsMediaAssets {
    box_front: Option<String>,
    logo: Option<String>,
}

fn read_emulationstation_media_assets(
    dir_path: &Path,
) -> std::collections::HashMap<String, EsMediaAssets> {
    use quick_xml::de::from_reader;
    use std::fs::File;
    use std::io::BufReader;

    #[derive(Deserialize)]
    struct EsMediaList {
        #[serde(rename = "game", default)]
        games: Vec<EsMediaGame>,
    }

    #[derive(Deserialize)]
    struct EsMediaGame {
        path: String,
        image: Option<String>,
        boxart: Option<String>,
        marquee: Option<String>,
    }

    let Ok(file) = File::open(dir_path.join("gamelist.xml")) else {
        return std::collections::HashMap::new();
    };
    let Ok(list) = from_reader::<_, EsMediaList>(BufReader::new(file)) else {
        return std::collections::HashMap::new();
    };

    list.games
        .into_iter()
        .map(|game| {
            let file = game.path.trim_start_matches("./").to_string();
            let image = game.image.map(|value| resolve_media_path(dir_path, &value));
            let boxart = game
                .boxart
                .map(|value| resolve_media_path(dir_path, &value));
            let logo = game
                .marquee
                .map(|value| resolve_media_path(dir_path, &value));
            (
                file,
                EsMediaAssets {
                    box_front: boxart.or(image),
                    logo,
                },
            )
        })
        .collect()
}

/// 获取单个目录的ROM列表
pub fn get_roms_for_directory(dir_config: &crate::settings::DirectoryConfig) -> Vec<SystemRoms> {
    get_roms_for_directory_with_progress(dir_config, &|_| {})
}

pub fn get_roms_for_directory_with_progress(
    dir_config: &crate::settings::DirectoryConfig,
    on_system: &dyn Fn(&str),
) -> Vec<SystemRoms> {
    let mut systems = Vec::new();
    if dir_config
        .indexed_folders
        .as_ref()
        .is_some_and(Vec::is_empty)
    {
        return systems;
    }
    let dir_path = Path::new(&dir_config.path);

    println!(
        "[DEBUG] scan_single_directory: {:?}, is_root_directory={}",
        dir_path, dir_config.is_root_directory
    );

    if !dir_path.exists() {
        println!("[DEBUG] Directory does not exist, skipping");
        return systems;
    }

    if dir_config.is_root_directory {
        // ROMs 根目录模式：扫描子目录和根目录本身
        let mut root_ps3_games = Vec::new();

        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                if crate::rom_index::scan_cancelled() {
                    break;
                }
                let sub_path = entry.path();
                if sub_path.is_dir() {
                    let system_name = sub_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    if dir_config.indexed_folders.as_ref().is_some_and(|folders| {
                        !folders
                            .iter()
                            .any(|folder| folder.eq_ignore_ascii_case(&system_name))
                    }) {
                        continue;
                    }
                    // 检查是否是PS3游戏文件夹
                    let ps3_game_dir = sub_path.join("PS3_GAME");
                    if ps3_game_dir.exists() && ps3_game_dir.is_dir() {
                        root_ps3_games.push(sub_path);
                        continue;
                    }

                    on_system(&system_name);

                    // 尝试加载临时元数据
                    if let Some(roms) =
                        try_load_from_temp_metadata(dir_path, &sub_path, &system_name)
                    {
                        println!(
                            "[DEBUG] Root mode (subdir): 使用临时元数据跳过扫描: system={}",
                            system_name
                        );
                        systems.push(SystemRoms {
                            system: system_name,
                            path: sub_path.to_string_lossy().to_string(),
                            roms,
                        });
                        continue;
                    }

                    // 自动检测子目录的 metadata 格式
                    let format = detect_metadata_format(&sub_path);

                    if let Ok(mut roms) = get_roms_from_directory(&sub_path, &format, &system_name)
                    {
                        if !roms.is_empty() {
                            println!(
                                "[DEBUG] Root mode (subdir): dir_path={:?}, sub_path={:?}, system={}",
                                dir_path, sub_path, system_name
                            );
                            let _ =
                                create_or_update_metadata(dir_path, &sub_path, &system_name, &roms);

                            apply_temp_metadata(&mut roms, dir_path, &system_name);

                            systems.push(SystemRoms {
                                system: system_name,
                                path: sub_path.to_string_lossy().to_string(),
                                roms,
                            });
                        }
                    }
                }
            }
        }

        // 扫描根目录本身的文件和收集到的PS3游戏文件夹
        let root_system_name = dir_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        if !root_ps3_games.is_empty()
            || std::fs::read_dir(dir_path).ok().is_some_and(|entries| {
                entries
                    .filter_map(Result::ok)
                    .any(|entry| entry.path().is_file())
            })
        {
            on_system(&root_system_name);
        }

        let format = detect_metadata_format(dir_path);
        let root_scan = if dir_config.indexed_folders.is_some() {
            Ok(Vec::new())
        } else if format == "none" {
            // 根目录模式的子目录已经作为独立系统处理，这里只读取直属文件，
            // 避免把所有平台递归复制成一个额外的根目录系统。
            scan_rom_files_internal(dir_path, &root_system_name, false)
        } else {
            get_roms_from_directory(dir_path, &format, &root_system_name)
        };
        if let Ok(mut roms) = root_scan {
            // 添加根目录下的PS3游戏文件夹
            for ps3_game_path in root_ps3_games {
                if crate::rom_index::scan_cancelled() {
                    break;
                }
                let folder_name = ps3_game_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let param_sfo_path = ps3_game_path.join("PS3_GAME").join("PARAM.SFO");
                let game_info = if param_sfo_path.exists() {
                    ps3::parse_param_sfo(&param_sfo_path).ok()
                } else {
                    None
                };

                let game_name = game_info
                    .as_ref()
                    .and_then(|info| info.title.clone())
                    .unwrap_or_else(|| folder_name.clone());

                let game_id = game_info.as_ref().and_then(|info| info.title_id.clone());

                let mut rom = RomInfo {
                    file: folder_name,
                    name: game_name,
                    description: game_id.map(|id| format!("Game ID: {}", id)),
                    directory: dir_path.to_string_lossy().to_string(),
                    system: root_system_name.clone(),
                    ..Default::default()
                };
                fill_file_metadata(&mut rom, &ps3_game_path);
                roms.push(rom);
            }

            if !roms.is_empty() {
                let library_path = dir_path.parent().unwrap_or(dir_path);
                println!(
                    "[DEBUG] Root mode (root itself): dir_path={:?}, library_path={:?}, system={}",
                    dir_path, library_path, root_system_name
                );
                if !crate::rom_index::scan_cancelled() {
                    let _ =
                        create_or_update_metadata(library_path, dir_path, &root_system_name, &roms);
                    apply_temp_metadata(&mut roms, library_path, &root_system_name);
                }

                systems.push(SystemRoms {
                    system: root_system_name,
                    path: dir_config.path.clone(),
                    roms,
                });
            }
        }
    } else {
        // 单系统目录模式
        let system_name = dir_config.system_id.clone().unwrap_or_else(|| {
            dir_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string()
        });
        on_system(&system_name);

        println!(
            "[DEBUG] 开始扫描单系统目录: {:?}, system={}",
            dir_path, system_name
        );

        let library_path = dir_path.parent().unwrap_or(dir_path);

        // 尝试加载临时元数据
        if let Some(roms) = try_load_from_temp_metadata(library_path, dir_path, &system_name) {
            println!(
                "[DEBUG] Single-system mode: 使用临时元数据跳过扫描: system={}",
                system_name
            );
            systems.push(SystemRoms {
                system: system_name,
                path: dir_config.path.clone(),
                roms,
            });
        } else {
            println!("[DEBUG] metadata_format={}", dir_config.metadata_format);

            if let Ok(mut roms) =
                get_roms_from_directory(dir_path, &dir_config.metadata_format, &system_name)
            {
                println!("[DEBUG] 扫描完成, 找到 {} 个 ROM", roms.len());
                if !roms.is_empty() {
                    println!(
                        "[DEBUG] Single-system mode: dir_path={:?}, library_path={:?}, system={}",
                        dir_path, library_path, system_name
                    );

                    if !crate::rom_index::scan_cancelled() {
                        println!("[DEBUG] 开始 create_or_update_metadata...");
                        let _ =
                            create_or_update_metadata(library_path, dir_path, &system_name, &roms);
                        println!("[DEBUG] create_or_update_metadata 完成");

                        println!("[DEBUG] 开始 apply_temp_metadata...");
                        apply_temp_metadata(&mut roms, library_path, &system_name);
                        println!("[DEBUG] apply_temp_metadata 完成");
                    }

                    systems.push(SystemRoms {
                        system: system_name,
                        path: dir_config.path.clone(),
                        roms,
                    });
                    println!("[DEBUG] 已添加到 systems");
                }
            } else {
                println!("[DEBUG] get_roms_from_directory 返回错误");
            }
        }
    }

    systems
}

#[allow(dead_code)]
pub fn get_all_roms() -> Result<Vec<SystemRoms>, String> {
    let settings = get_settings();
    let active_directories: Vec<_> = settings.active_library().cloned().into_iter().collect();

    println!(
        "[DEBUG] get_all_roms() called, scanning {} directories",
        active_directories.len()
    );

    let all_systems: Vec<SystemRoms> = active_directories
        .par_iter()
        .flat_map(get_roms_for_directory)
        .collect();

    Ok(all_systems)
}

/// 自动检测目录的 metadata 格式
pub fn detect_metadata_format(dir_path: &Path) -> String {
    if dir_path.join("metadata.pegasus.txt").exists() || dir_path.join("metadata.txt").exists() {
        "pegasus".to_string()
    } else if dir_path.join("gamelist.xml").exists() {
        "emulationstation".to_string()
    } else {
        "none".to_string()
    }
}

/// 从目录读取 ROM 列表
pub fn get_roms_from_directory(
    dir_path: &Path,
    metadata_format: &str,
    system_name: &str,
) -> Result<Vec<RomInfo>, String> {
    match metadata_format {
        "pegasus" => read_pegasus_roms(dir_path, system_name),
        "emulationstation" => read_emulationstation_roms(dir_path, system_name),
        "none" => scan_rom_files(dir_path, system_name),
        _ => Err(format!("Unknown metadata format: {}", metadata_format)),
    }
}

fn read_pegasus_roms(dir_path: &Path, system_name: &str) -> Result<Vec<RomInfo>, String> {
    let possible_files = ["metadata.pegasus.txt", "metadata.txt"];

    for filename in &possible_files {
        let metadata_path = dir_path.join(filename);
        if metadata_path.exists() {
            let metadata = parse_pegasus_file(&metadata_path)?;
            return Ok(metadata
                .games
                .into_iter()
                .filter_map(|g| {
                    if crate::rom_index::scan_cancelled() {
                        return None;
                    }
                    let mut rom: RomInfo = g.into();
                    rom.directory = dir_path.to_string_lossy().to_string();
                    rom.system = system_name.to_string();

                    // 验证 ROM 文件是否存在
                    if !rom.file.is_empty() {
                        let rom_path = dir_path.join(&rom.file);
                        if !rom_path.exists() {
                            // ROM 文件不存在，跳过此条目
                            return None;
                        }
                        fill_file_metadata(&mut rom, &rom_path);
                    }

                    resolve_all_media_paths(&mut rom, dir_path);
                    scan_media_directory(&mut rom, dir_path);
                    Some(rom)
                })
                .collect());
        }
    }

    scan_rom_files(dir_path, system_name)
}

fn resolve_media_path(dir_path: &Path, value: &str) -> String {
    let path = Path::new(value);
    if path.is_absolute() {
        // Normalize path separators on Windows
        #[cfg(windows)]
        return value.replace('/', "\\");
        #[cfg(not(windows))]
        return value.to_string();
    } else {
        // Normalize the relative path first, then join
        #[cfg(windows)]
        let normalized_value = value.replace('/', "\\");
        #[cfg(not(windows))]
        let normalized_value = value.to_string();

        dir_path
            .join(&normalized_value)
            .to_string_lossy()
            .to_string()
    }
}

fn resolve_all_media_paths(rom: &mut RomInfo, dir_path: &Path) {
    let resolve = |opt: &mut Option<String>| {
        if let Some(v) = opt.take() {
            // gamelist.xml 的空标签(如 <image/>)解析为空串,join 后会变成
            // 指向库目录的“伪路径”,让 ROM 被误判为已有资源,必须丢弃。
            if !v.trim().is_empty() {
                *opt = Some(resolve_media_path(dir_path, &v));
            }
        }
    };

    resolve(&mut rom.box_front);
    resolve(&mut rom.box_back);
    resolve(&mut rom.box_spine);
    resolve(&mut rom.box_full);
    resolve(&mut rom.cartridge);
    resolve(&mut rom.logo);
    resolve(&mut rom.marquee);
    resolve(&mut rom.bezel);
    resolve(&mut rom.gridicon);
    resolve(&mut rom.flyer);
    resolve(&mut rom.background);
    resolve(&mut rom.music);
    resolve(&mut rom.screenshot);
    resolve(&mut rom.titlescreen);
    resolve(&mut rom.video);
}

fn scan_media_directory(rom: &mut RomInfo, dir_path: &Path) {
    // Use file stem (filename without extension) as lookup key, like Pegasus does
    let file_stem = Path::new(&rom.file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&rom.name);

    let media_dir = dir_path.join("media").join(file_stem);
    if !media_dir.exists() {
        return;
    }

    let entries = match std::fs::read_dir(&media_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let full_path = path.to_string_lossy().to_string();

        match stem.to_lowercase().as_str() {
            "boxfront" | "box_front" | "boxart" | "cover" => {
                if rom.box_front.is_none() {
                    rom.box_front = Some(full_path);
                }
            }
            "boxback" | "box_back" => {
                if rom.box_back.is_none() {
                    rom.box_back = Some(full_path);
                }
            }
            "boxspine" | "box_spine" => {
                if rom.box_spine.is_none() {
                    rom.box_spine = Some(full_path);
                }
            }
            "boxfull" | "box_full" => {
                if rom.box_full.is_none() {
                    rom.box_full = Some(full_path);
                }
            }
            "cartridge" | "cart" | "disc" => {
                if rom.cartridge.is_none() {
                    rom.cartridge = Some(full_path);
                }
            }
            "logo" | "wheel" => {
                if rom.logo.is_none() {
                    rom.logo = Some(full_path);
                }
            }
            "marquee" | "banner" => {
                if rom.marquee.is_none() {
                    rom.marquee = Some(full_path);
                }
            }
            "bezel" | "screenmarquee" => {
                if rom.bezel.is_none() {
                    rom.bezel = Some(full_path);
                }
            }
            "gridicon" | "steam" | "poster" => {
                if rom.gridicon.is_none() {
                    rom.gridicon = Some(full_path);
                }
            }
            "flyer" => {
                if rom.flyer.is_none() {
                    rom.flyer = Some(full_path);
                }
            }
            "background" | "fanart" => {
                if rom.background.is_none() {
                    rom.background = Some(full_path);
                }
            }
            "music" => {
                if rom.music.is_none() {
                    rom.music = Some(full_path);
                }
            }
            "screenshot" | "screenshots" | "screen" => {
                if rom.screenshot.is_none() {
                    rom.screenshot = Some(full_path);
                }
            }
            "titlescreen" | "title_screen" | "title" => {
                if rom.titlescreen.is_none() {
                    rom.titlescreen = Some(full_path);
                }
            }
            "video" | "videos" if rom.video.is_none() => {
                rom.video = Some(full_path);
            }
            _ => {}
        }
    }
}

fn read_emulationstation_roms(dir_path: &Path, system_name: &str) -> Result<Vec<RomInfo>, String> {
    use quick_xml::de::from_reader;
    use std::fs::File;
    use std::io::BufReader;

    #[derive(Debug, Deserialize)]
    struct EsGameList {
        #[serde(rename = "game", default)]
        games: Vec<EsGame>,
    }

    #[derive(Debug, Deserialize)]
    struct EsGame {
        path: String,
        name: Option<String>,
        desc: Option<String>,
        image: Option<String>,
        boxart: Option<String>,
        marquee: Option<String>,
        developer: Option<String>,
        publisher: Option<String>,
        genre: Option<String>,
        players: Option<String>,
        releasedate: Option<String>,
        #[serde(default, deserialize_with = "crate::rating::deserialize_optional_f32")]
        rating: Option<f32>,
        #[serde(rename = "eng", alias = "english-name")]
        english_name: Option<String>,
        #[serde(rename = "chinese-name", alias = "x-mrrm-cn")]
        chinese_name: Option<String>,
    }

    let gamelist_path = dir_path.join("gamelist.xml");

    if !gamelist_path.exists() {
        return scan_rom_files(dir_path, system_name);
    }

    let file = File::open(&gamelist_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    let game_list: EsGameList =
        from_reader(reader).map_err(|e| format!("Failed to parse gamelist.xml: {}", e))?;

    Ok(game_list
        .games
        .into_iter()
        .filter_map(|g| {
            if crate::rom_index::scan_cancelled() {
                return None;
            }
            let file = g.path.trim_start_matches("./").to_string();

            // 验证 ROM 文件是否存在
            if !file.is_empty() {
                let rom_path = dir_path.join(&file);
                if !rom_path.exists() {
                    // ROM 文件不存在，跳过此条目
                    return None;
                }
            }

            let name = g.name.unwrap_or_else(|| {
                Path::new(&file)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            });

            // 空标签(<image/>)解析为空串,join 后会变成指向库目录的“伪路径”,先滤掉。
            let image = g
                .image
                .filter(|value| !value.trim().is_empty())
                .map(|image| resolve_media_path(dir_path, &image));
            let boxart = g
                .boxart
                .filter(|value| !value.trim().is_empty())
                .map(|boxart| resolve_media_path(dir_path, &boxart));
            let marquee = g
                .marquee
                .filter(|value| !value.trim().is_empty())
                .map(|marquee| resolve_media_path(dir_path, &marquee));

            let mut rom = RomInfo {
                file,
                name,
                description: g.desc,
                developer: g.developer,
                publisher: g.publisher,
                genre: g.genre,
                players: g.players,
                release: g.releasedate,
                rating: g
                    .rating
                    .and_then(|r| crate::rating::normalize_rating(r as f64))
                    .map(|r| format!("{}%", (r * 100.0).round() as i32)),
                directory: dir_path.to_string_lossy().to_string(),
                system: system_name.to_string(),
                // EmulationStation themes often store a pre-composed horizontal
                // "mix" in <image>, while <boxart> is the actual portrait cover.
                // Prefer the real cover for MRRM's 3:4 cards and keep <image> as
                // a fallback for libraries that only expose the standard field.
                box_front: boxart.or(image),
                logo: marquee.clone(),
                marquee,
                english_name: g.english_name,
                chinese_name: g.chinese_name,
                ..Default::default()
            };
            if !rom.file.is_empty() {
                let rom_path = dir_path.join(&rom.file);
                fill_file_metadata(&mut rom, &rom_path);
            }
            Some(rom)
        })
        .collect())
}

fn normalize_extension(ext: &str) -> String {
    ext.trim().trim_start_matches('.').to_lowercase()
}

fn get_system_extensions(system_name: &str) -> Option<Vec<String>> {
    let target = system_name.to_lowercase();
    let systems = get_preset_systems_data();

    systems
        .into_iter()
        .find(|s| {
            s.id.to_lowercase() == target
                || s.short_name.to_lowercase() == target
                || s.name.to_lowercase() == target
        })
        .map(|s| s.extensions)
}

/// 无 metadata 时扫描 ROM 文件
fn scan_rom_files(dir_path: &Path, system_name: &str) -> Result<Vec<RomInfo>, String> {
    scan_rom_files_internal(dir_path, system_name, true)
}

fn scan_rom_files_internal(
    dir_path: &Path,
    system_name: &str,
    recursive: bool,
) -> Result<Vec<RomInfo>, String> {
    use std::collections::HashSet;
    use std::fs;

    println!(
        "[DEBUG] scan_rom_files 开始: {:?}, system={}",
        dir_path, system_name
    );

    // 常见 ROM 扩展名（兜底用）
    let fallback_extensions = [
        "nes", "sfc", "smc", "gba", "gb", "gbc", "n64", "z64", "v64", "iso", "bin", "cue", "img",
        "zip", "7z", "rar", "md", "gen", "smd", "gg", "sms", "pce", "ngp", "ngc", "ws", "wsc",
        "a26", "a52", "a78", "lnx", "nds", "3ds", "cia", "psx", "pbp", "chd",
    ];

    let system_extensions = get_system_extensions(system_name);
    if let Some(exts) = &system_extensions {
        println!("[DEBUG] 使用系统扩展名: {:?}", exts);
    } else {
        println!("[DEBUG] 使用默认扩展名列表");
    }

    let allowed_extensions: HashSet<String> = if let Some(exts) = system_extensions {
        exts.into_iter().map(|e| normalize_extension(&e)).collect()
    } else {
        fallback_extensions.iter().map(|e| e.to_string()).collect()
    };

    let mut roms = Vec::new();
    let system_lower = system_name.to_lowercase();

    if system_lower == "ports" || system_lower.contains("ports (") || system_lower == "psbios" {
        return Ok(roms);
    }

    // EasyRPG 一个直属文件夹就是一个游戏，只使用文件夹名，不扫描内部素材。
    if system_lower == "easyrpg" {
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.filter_map(|entry| entry.ok()) {
                if crate::rom_index::scan_cancelled() {
                    break;
                }
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let folder_name = entry.file_name().to_string_lossy().into_owned();
                let mut rom = RomInfo {
                    file: folder_name.clone(),
                    name: folder_name,
                    directory: dir_path.to_string_lossy().to_string(),
                    system: system_name.to_string(),
                    ..Default::default()
                };
                fill_file_metadata(&mut rom, &path);
                roms.push(rom);
            }
        }
        return Ok(roms);
    }

    // 普通文件型平台允许分类子目录，保留相对路径并跳过媒体/备份目录。
    if !system_lower.contains("ps3") {
        fn collect_rom_paths(
            directory: &Path,
            allowed: &HashSet<String>,
            paths: &mut Vec<PathBuf>,
            recursive: bool,
        ) {
            if crate::rom_index::scan_cancelled() {
                return;
            }
            let Ok(entries) = fs::read_dir(directory) else {
                return;
            };
            for entry in entries.filter_map(|entry| entry.ok()) {
                if crate::rom_index::scan_cancelled() {
                    break;
                }
                let path = entry.path();
                if path.is_dir() {
                    if !recursive {
                        continue;
                    }
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    if ["media", "images", "artwork", "screenshots", "_archives"]
                        .contains(&name.as_str())
                    {
                        continue;
                    }
                    collect_rom_paths(&path, allowed, paths, recursive);
                } else {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    if allowed
                        .iter()
                        .any(|extension| name.ends_with(&format!(".{extension}")))
                    {
                        paths.push(path);
                    }
                }
            }
        }

        let mut paths = Vec::new();
        collect_rom_paths(dir_path, &allowed_extensions, &mut paths, recursive);
        if matches!(
            system_lower.as_str(),
            "ps" | "ps1" | "psx" | "playstation" | "ps1 hack"
        ) {
            // CUE/BIN 是同一张光盘，不应把数据轨、音轨和附带 BIOS 分别列成游戏。
            // 同目录存在 CUE 时只索引 CUE；没有 CUE 的单独 BIN 仍然保留。
            let cue_directories: HashSet<PathBuf> = paths
                .iter()
                .filter(|path| {
                    path.extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|value| value.eq_ignore_ascii_case("cue"))
                })
                .filter_map(|path| path.parent().map(Path::to_path_buf))
                .collect();
            let referenced_bins: HashSet<String> = paths
                .iter()
                .filter(|path| {
                    path.extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|value| value.eq_ignore_ascii_case("cue"))
                })
                .filter_map(|cue| std::fs::read_to_string(cue).ok().map(|text| (cue, text)))
                .flat_map(|(cue, text)| {
                    text.lines()
                        .filter_map(|line| {
                            let line = line.trim();
                            if !line.to_ascii_uppercase().starts_with("FILE ") {
                                return None;
                            }
                            let file_name = line
                                .split_once('"')
                                .and_then(|(_, rest)| rest.split_once('"'))
                                .map(|(name, _)| name)
                                .or_else(|| line.split_whitespace().nth(1))?;
                            Some(
                                cue.parent()
                                    .unwrap_or_else(|| Path::new(""))
                                    .join(file_name)
                                    .to_string_lossy()
                                    .to_ascii_lowercase(),
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .collect();
            paths.retain(|path| {
                let is_bin = path
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.eq_ignore_ascii_case("bin"));
                let referenced =
                    referenced_bins.contains(&path.to_string_lossy().to_ascii_lowercase());
                let is_sidecar_bios = path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.to_ascii_lowercase().ends_with("_bios"))
                    && path
                        .parent()
                        .is_some_and(|parent| cue_directories.contains(parent));
                !is_bin || (!referenced && !is_sidecar_bios)
            });
        }
        paths.sort();
        for path in paths {
            if crate::rom_index::scan_cancelled() {
                break;
            }
            let relative = path.strip_prefix(dir_path).unwrap_or(&path);
            let file = relative.to_string_lossy().into_owned();
            let name = path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("Unknown")
                .to_string();
            let mut rom = RomInfo {
                file,
                name,
                directory: dir_path.to_string_lossy().to_string(),
                system: system_name.to_string(),
                ..Default::default()
            };
            fill_file_metadata(&mut rom, &path);
            roms.push(rom);
        }
        return Ok(roms);
    }

    println!("[DEBUG] 开始 read_dir...");
    if let Ok(entries) = fs::read_dir(dir_path) {
        println!("[DEBUG] read_dir 成功，开始遍历...");
        let mut count = 0;
        for entry in entries.filter_map(|e| e.ok()) {
            if crate::rom_index::scan_cancelled() {
                break;
            }
            count += 1;
            let path = entry.path();

            if count % 10 == 0 {
                println!("[DEBUG] 已扫描 {} 个条目...", count);
            }

            // PS3 特殊处理：扫描文件夹作为 ROM
            if system_name.to_lowercase().contains("ps3") && path.is_dir() {
                let folder_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                // 检查是否包含 PS3_GAME 目录
                let ps3_game_dir = path.join("PS3_GAME");
                let is_valid_ps3_folder = ps3_game_dir.exists() && ps3_game_dir.is_dir();

                if is_valid_ps3_folder || true {
                    // 暂时不做严格验证
                    // 尝试解析 PARAM.SFO 获取游戏信息
                    let param_sfo_path = ps3_game_dir.join("PARAM.SFO");
                    let game_info = if param_sfo_path.exists() {
                        ps3::parse_param_sfo(&param_sfo_path).ok()
                    } else {
                        None
                    };

                    // 使用解析出的信息或默认值
                    let game_name = game_info
                        .as_ref()
                        .and_then(|info| info.title.clone())
                        .unwrap_or_else(|| folder_name.clone());

                    let game_id = game_info.as_ref().and_then(|info| info.title_id.clone());

                    let mut rom = RomInfo {
                        file: folder_name,
                        name: game_name,
                        description: game_id.map(|id| format!("Game ID: {}", id)),
                        directory: dir_path.to_string_lossy().to_string(),
                        system: system_name.to_string(),
                        ..Default::default()
                    };
                    fill_file_metadata(&mut rom, &path);
                    roms.push(rom);
                }
            }
            // 常规文件处理
            else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_normalized = normalize_extension(ext);
                    if allowed_extensions.contains(&ext_normalized) {
                        let filename = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        let default_name = path
                            .file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        // PS3 ISO 特殊处理：尝试从 ISO 中提取 PARAM.SFO
                        let (game_name, game_description) =
                            if system_name.to_lowercase().contains("ps3")
                                && ext.to_lowercase() == "iso"
                            {
                                match ps3::parse_param_sfo_from_iso(&path) {
                                    Ok(game_info) => {
                                        let name = game_info
                                            .title
                                            .clone()
                                            .unwrap_or_else(|| default_name.clone());
                                        let desc =
                                            game_info.title_id.map(|id| format!("Game ID: {}", id));
                                        (name, desc)
                                    }
                                    Err(_) => (default_name.clone(), None),
                                }
                            } else {
                                (default_name, None)
                            };

                        let mut rom = RomInfo {
                            file: filename,
                            name: game_name,
                            description: game_description,
                            directory: dir_path.to_string_lossy().to_string(),
                            system: system_name.to_string(),
                            ..Default::default()
                        };
                        fill_file_metadata(&mut rom, &path);
                        roms.push(rom);
                    }
                }
            }
        }
    } else {
        println!("[DEBUG] read_dir 失败");
    }

    println!("[DEBUG] scan_rom_files 完成, 找到 {} 个 ROM", roms.len());
    Ok(roms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use uuid::Uuid;

    fn create_temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("mrrm_test_{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn parse_emulationstation_gamelist() {
        let dir = create_temp_dir();
        let xml = r#"
<gameList>
  <game>
    <path>./Super Mario World.sfc</path>
    <name>Super Mario World</name>
    <desc>Test description</desc>
    <developer>Nintendo</developer>
    <genre>Platform</genre>
    <players>2</players>
    <releasedate>1990</releasedate>
    <rating>1600</rating>
    <eng>Super Mario World (English)</eng>
  </game>
</gameList>
"#;
        fs::write(dir.join("gamelist.xml"), xml).expect("write gamelist.xml");
        // 解析器会跳过磁盘上不存在的 ROM 条目,需要真实创建文件
        fs::write(dir.join("Super Mario World.sfc"), b"rom").expect("write rom file");

        let roms = read_emulationstation_roms(&dir, "snes").expect("parse gamelist");
        assert_eq!(roms.len(), 1);
        assert_eq!(roms[0].file, "Super Mario World.sfc");
        assert_eq!(roms[0].name, "Super Mario World");
        assert_eq!(roms[0].system, "snes");
        assert_eq!(roms[0].rating.as_deref(), Some("80%"));
        assert_eq!(
            roms[0].english_name.as_deref(),
            Some("Super Mario World (English)")
        );
        // gamelist 路径同样补齐文件元数据
        assert_eq!(roms[0].file_size, Some(3));
        assert!(roms[0].modified_at.is_some());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn emulationstation_prefers_boxart_over_composed_image_and_reads_marquee() {
        let dir = create_temp_dir();
        fs::create_dir_all(dir.join("media")).expect("create media directory");
        fs::write(dir.join("Test.gba"), b"rom").expect("write rom");
        fs::write(dir.join("media/mix.png"), b"mix").expect("write mix");
        fs::write(dir.join("media/boxart.png"), b"boxart").expect("write boxart");
        fs::write(dir.join("media/logo.png"), b"logo").expect("write logo");
        fs::write(
            dir.join("gamelist.xml"),
            r#"<gameList><game>
  <path>./Test.gba</path>
  <name>Test Game</name>
  <image>./media/mix.png</image>
  <boxart>./media/boxart.png</boxart>
  <marquee>./media/logo.png</marquee>
</game></gameList>"#,
        )
        .expect("write gamelist");

        let roms = read_emulationstation_roms(&dir, "gba").expect("parse gamelist");
        assert_eq!(roms.len(), 1);
        assert_eq!(
            roms[0].box_front,
            Some(resolve_media_path(&dir, "./media/boxart.png"))
        );
        assert_eq!(
            roms[0].logo,
            Some(resolve_media_path(&dir, "./media/logo.png"))
        );
        assert_eq!(roms[0].logo, roms[0].marquee);

        let media = read_emulationstation_media_assets(&dir);
        let source = media.get("Test.gba").expect("source media");
        assert_eq!(
            source.box_front,
            Some(resolve_media_path(&dir, "./media/boxart.png"))
        );
        assert_eq!(
            source.logo,
            Some(resolve_media_path(&dir, "./media/logo.png"))
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn scan_rom_files_fills_file_size_and_modified_at() {
        let dir = create_temp_dir();
        let rom_bytes = b"fake rom content";
        fs::write(dir.join("Test Game.sfc"), rom_bytes).expect("write rom file");

        let roms = scan_rom_files(&dir, "sfc").expect("scan roms");
        assert_eq!(roms.len(), 1);
        assert_eq!(roms[0].file_size, Some(rom_bytes.len() as u64));

        let modified_at = roms[0].modified_at.expect("modified_at 应存在");
        assert!(modified_at > 0, "modified_at 应为正的 unix 秒");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn scan_rom_files_supports_category_subdirectories() {
        let dir = create_temp_dir();
        fs::create_dir_all(dir.join("RPG")).unwrap();
        fs::write(dir.join("RPG").join("Test Game.zip"), b"rom").unwrap();
        fs::write(dir.join("RPG").join("cover.png"), b"image").unwrap();

        let roms = scan_rom_files(&dir, "FC").unwrap();
        assert_eq!(roms.len(), 1);
        assert_eq!(
            roms[0].file,
            Path::new("RPG").join("Test Game.zip").to_string_lossy()
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ps1_scan_uses_cue_instead_of_duplicate_bin_tracks() {
        let dir = create_temp_dir();
        let game = dir.join("Game");
        let bin_only = dir.join("Bin Only");
        fs::create_dir_all(&game).unwrap();
        fs::create_dir_all(&bin_only).unwrap();
        fs::write(game.join("Game.cue"), b"FILE \"Game.bin\" BINARY").unwrap();
        fs::write(game.join("Game.bin"), b"disc").unwrap();
        fs::write(game.join("Game_BIOS.BIN"), b"bios").unwrap();
        fs::write(game.join("Independent.bin"), b"disc").unwrap();
        fs::write(bin_only.join("Standalone.bin"), b"disc").unwrap();

        let roms = scan_rom_files(&dir, "PS1").unwrap();
        assert_eq!(roms.len(), 3);
        assert!(roms.iter().any(|rom| rom.file.ends_with("Game.cue")));
        assert!(roms.iter().any(|rom| rom.file.ends_with("Independent.bin")));
        assert!(roms.iter().any(|rom| rom.file.ends_with("Standalone.bin")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn easyrpg_uses_direct_child_folder_names_only() {
        let dir = create_temp_dir();
        let game = dir.join("测试游戏");
        fs::create_dir_all(game.join("Picture")).unwrap();
        fs::write(game.join("Picture").join("asset.png"), b"image").unwrap();

        let roms = scan_rom_files(&dir, "EASYRPG").unwrap();
        assert_eq!(roms.len(), 1);
        assert_eq!(roms[0].file, "测试游戏");
        assert_eq!(roms[0].name, "测试游戏");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn legacy_cn_tool_metadata_migrates_to_chinese_name() {
        let mut game = PegasusGame {
            name: "实况足球".into(),
            file: Some("game.gba".into()),
            ..Default::default()
        };
        game.extra
            .insert("x-mrrm-eng".into(), "Winning Eleven".into());

        let rom: RomInfo = game.into();
        assert_eq!(rom.name, "Winning Eleven");
        assert_eq!(rom.english_name.as_deref(), Some("Winning Eleven"));
        assert_eq!(rom.chinese_name.as_deref(), Some("实况足球"));
    }

    #[test]
    fn temp_media_fallback_recovers_existing_boxart() {
        let dir = create_temp_dir();
        let media_dir = dir.join("media").join("CT特种部队1");
        fs::create_dir_all(&media_dir).unwrap();
        fs::write(media_dir.join("boxfront.png"), b"image").unwrap();
        let mut game = PegasusGame::default();

        fill_missing_temp_media(&mut game, &dir, "CT特种部队1.zip");

        assert_eq!(
            game.box_front.as_deref(),
            Some(media_dir.join("boxfront.png").to_string_lossy().as_ref())
        );

        game.box_front = Some(
            dir.join("media")
                .join("CT特种部队1")
                .to_string_lossy()
                .into_owned(),
        );
        fill_missing_temp_media(&mut game, &dir, "CT特种部队1.zip");
        assert_eq!(
            game.box_front.as_deref(),
            Some(media_dir.join("boxfront.png").to_string_lossy().as_ref())
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
