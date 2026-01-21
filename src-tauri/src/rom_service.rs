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
    // 预览数据 (PegasusGame 结构比较接近原始解析结果)
    pub temp_data: Option<PegasusGame>,

    pub has_temp_metadata: bool,
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
        Self {
            file: game.file.unwrap_or_default(),
            name: game.name,
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
            english_name: game
                .extra
                .get("x-mrrm-eng")
                .or_else(|| game.extra.get("x-english-name"))
                .cloned(),
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
    content.push_str(&format!("launch: {{file.path}}\n\n"));

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

        content.push_str("\n");
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

        content.push_str("\n");
    }

    // 写入文件
    let mut file = fs::File::create(&metadata_path)
        .map_err(|e| format!("Failed to write metadata file: {}", e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write metadata file: {}", e))?;

    Ok(())
}

/// 尝试加载并应用临时元数据
fn apply_temp_metadata(roms: &mut [RomInfo], library_path: &Path, system: &str) {
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
        for rom in roms {
            if let Some(temp_game) = temp_metadata
                .games
                .iter()
                .find(|g| g.file == Some(rom.file.clone()))
            {
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

                rom.temp_data = Some(resolved_game);
            }
        }
    }
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

                // 构造 temp_data (用于前端编辑显示)
                let temp_game = PegasusGame {
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
                    video: rom.video.clone(),
                    background: rom.background.clone(),
                    ..Default::default()
                };
                rom.temp_data = Some(temp_game);

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

/// 获取所有目录的 ROM 列表

/// 获取单个目录的ROM列表
pub fn get_roms_for_directory(dir_config: &crate::settings::DirectoryConfig) -> Vec<SystemRoms> {
    let mut systems = Vec::new();
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
                let sub_path = entry.path();
                if sub_path.is_dir() {
                    // 检查是否是PS3游戏文件夹
                    let ps3_game_dir = sub_path.join("PS3_GAME");
                    if ps3_game_dir.exists() && ps3_game_dir.is_dir() {
                        root_ps3_games.push(sub_path);
                        continue;
                    }

                    let system_name = sub_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

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

        let format = detect_metadata_format(dir_path);
        if let Ok(mut roms) = get_roms_from_directory(dir_path, &format, &root_system_name) {
            // 添加根目录下的PS3游戏文件夹
            for ps3_game_path in root_ps3_games {
                let folder_name = ps3_game_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let param_sfo_path = ps3_game_path.join("PS3_GAME").join("PARAM.SFO");
                let game_info = if param_sfo_path.exists() {
                    match ps3::parse_param_sfo(&param_sfo_path) {
                        Ok(info) => Some(info),
                        Err(_) => None,
                    }
                } else {
                    None
                };

                let game_name = game_info
                    .as_ref()
                    .and_then(|info| info.title.clone())
                    .unwrap_or_else(|| folder_name.clone());

                let game_id = game_info.as_ref().and_then(|info| info.title_id.clone());

                roms.push(RomInfo {
                    file: folder_name,
                    name: game_name,
                    description: game_id.map(|id| format!("Game ID: {}", id)),
                    summary: None,
                    developer: None,
                    publisher: None,
                    genre: None,
                    players: None,
                    release: None,
                    rating: None,
                    directory: dir_path.to_string_lossy().to_string(),
                    system: root_system_name.clone(),
                    box_front: None,
                    box_back: None,
                    box_spine: None,
                    box_full: None,
                    cartridge: None,
                    logo: None,
                    marquee: None,
                    bezel: None,
                    gridicon: None,
                    flyer: None,
                    background: None,
                    music: None,
                    screenshot: None,
                    titlescreen: None,
                    video: None,
                    english_name: None,
                    has_temp_metadata: false,
                    temp_data: None,
                });
            }

            if !roms.is_empty() {
                let library_path = dir_path.parent().unwrap_or(dir_path);
                println!(
                    "[DEBUG] Root mode (root itself): dir_path={:?}, library_path={:?}, system={}",
                    dir_path, library_path, root_system_name
                );
                let _ = create_or_update_metadata(library_path, dir_path, &root_system_name, &roms);

                apply_temp_metadata(&mut roms, library_path, &root_system_name);

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

                    println!("[DEBUG] 开始 create_or_update_metadata...");
                    let _ = create_or_update_metadata(library_path, dir_path, &system_name, &roms);
                    println!("[DEBUG] create_or_update_metadata 完成");

                    println!("[DEBUG] 开始 apply_temp_metadata...");
                    apply_temp_metadata(&mut roms, library_path, &system_name);
                    println!("[DEBUG] apply_temp_metadata 完成");

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

pub fn get_all_roms() -> Result<Vec<SystemRoms>, String> {
    let settings = get_settings();

    println!(
        "[DEBUG] get_all_roms() called, scanning {} directories",
        settings.directories.len()
    );

    let all_systems: Vec<SystemRoms> = settings
        .directories
        .par_iter()
        .flat_map(|dir_config| {
            let mut systems = Vec::new();
            let dir_path = Path::new(&dir_config.path);

            println!(
                "[DEBUG] Checking directory: {:?}, is_root_directory={}",
                dir_path, dir_config.is_root_directory
            );

            if !dir_path.exists() {
                println!("[DEBUG] Directory does not exist, skipping");
                return systems;
            }

            if dir_config.is_root_directory {
                // ROMs 根目录模式：扫描子目录和根目录本身
                let mut root_ps3_games = Vec::new(); // 收集根目录下的PS3游戏文件夹

                if let Ok(entries) = std::fs::read_dir(dir_path) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let sub_path = entry.path();
                        if sub_path.is_dir() {
                            // 检查是否是PS3游戏文件夹（包含PS3_GAME目录）
                            let ps3_game_dir = sub_path.join("PS3_GAME");
                            if ps3_game_dir.exists() && ps3_game_dir.is_dir() {
                                // 这是PS3游戏文件夹，归入根目录的PS3系统
                                root_ps3_games.push(sub_path);
                                continue;
                            }

                            let system_name = sub_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Unknown")
                                .to_string();

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

                            if let Ok(mut roms) =
                                get_roms_from_directory(&sub_path, &format, &system_name)
                            {
                                if !roms.is_empty() {
                                    // 创建temp目录和初始metadata文件（如果不存在）
                                    // 根目录模式：library_path = dir_path, system_dir = sub_path
                                    println!(
                                        "[DEBUG] Root mode (subdir): dir_path={:?}, sub_path={:?}, system={}",
                                        dir_path, sub_path, system_name
                                    );
                                    let _ = create_or_update_metadata(
                                        dir_path,
                                        &sub_path,
                                        &system_name,
                                        &roms,
                                    );

                                    // 尝试加载临时元数据
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

                let format = detect_metadata_format(dir_path);
                if let Ok(mut roms) =
                    get_roms_from_directory(dir_path, &format, &root_system_name)
                {
                    // 添加根目录下的PS3游戏文件夹
                    for ps3_game_path in root_ps3_games {
                        let folder_name = ps3_game_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        let param_sfo_path = ps3_game_path.join("PS3_GAME").join("PARAM.SFO");
                        let game_info = if param_sfo_path.exists() {
                            match ps3::parse_param_sfo(&param_sfo_path) {
                                Ok(info) => Some(info),
                                Err(_) => None,
                            }
                        } else {
                            None
                        };

                        let game_name = game_info
                            .as_ref()
                            .and_then(|info| info.title.clone())
                            .unwrap_or_else(|| folder_name.clone());

                        let game_id = game_info.as_ref().and_then(|info| info.title_id.clone());

                        roms.push(RomInfo {
                            file: folder_name,
                            name: game_name,
                            description: game_id.map(|id| format!("Game ID: {}", id)),
                            summary: None,
                            developer: None,
                            publisher: None,
                            genre: None,
                            players: None,
                            release: None,
                            rating: None,
                            directory: dir_path.to_string_lossy().to_string(),
                            system: root_system_name.clone(),
                            box_front: None,
                            box_back: None,
                            box_spine: None,
                            box_full: None,
                            cartridge: None,
                            logo: None,
                            marquee: None,
                            bezel: None,
                            gridicon: None,
                            flyer: None,
                            background: None,
                            music: None,
                            screenshot: None,
                            titlescreen: None,
                            video: None,
                            english_name: None,
                            has_temp_metadata: false,
                            temp_data: None,
                        });
                    }

                    if !roms.is_empty() {
                        // 创建temp目录和初始metadata文件（如果不存在）
                        // 根目录模式：如果目录名是系统名，使用父目录作为library_path
                        let library_path = dir_path.parent().unwrap_or(dir_path);
                        println!(
                            "[DEBUG] Root mode (root itself): dir_path={:?}, library_path={:?}, system={}",
                            dir_path, library_path, root_system_name
                        );
                        let _ = create_or_update_metadata(
                            library_path,
                            dir_path,
                            &root_system_name,
                            &roms,
                        );

                        apply_temp_metadata(&mut roms, library_path, &root_system_name);

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

                println!(
                    "[DEBUG] 开始扫描单系统目录: {:?}, system={}",
                    dir_path, system_name
                );

                // 单系统模式：library_path = 父目录
                let library_path = dir_path.parent().unwrap_or(dir_path);

                // 尝试加载临时元数据
                if let Some(roms) =
                    try_load_from_temp_metadata(library_path, dir_path, &system_name)
                {
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

                    if let Ok(mut roms) = get_roms_from_directory(
                        dir_path,
                        &dir_config.metadata_format,
                        &system_name,
                    ) {
                        println!("[DEBUG] 扫描完成, 找到 {} 个 ROM", roms.len());
                        if !roms.is_empty() {
                            // 单系统模式：library_path = 父目录, system_dir = dir_path
                            println!(
                                "[DEBUG] Single-system mode: dir_path={:?}, library_path={:?}, system={}",
                                dir_path, library_path, system_name
                            );

                            println!("[DEBUG] 开始 create_or_update_metadata...");
                            let _ = create_or_update_metadata(
                                library_path,
                                dir_path,
                                &system_name,
                                &roms,
                            );
                            println!("[DEBUG] create_or_update_metadata 完成");

                            // 尝试加载临时元数据
                            println!("[DEBUG] 开始 apply_temp_metadata...");
                            apply_temp_metadata(&mut roms, library_path, &system_name);
                            println!("[DEBUG] apply_temp_metadata 完成");

                            systems.push(SystemRoms {
                                system: system_name,
                                path: dir_config.path.clone(),
                                roms,
                            });
                            println!("[DEBUG] 已添加到 all_systems");
                        }
                    } else {
                        println!("[DEBUG] get_roms_from_directory 返回错误");
                    }
                }
            }

            systems
        })
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
            *opt = Some(resolve_media_path(dir_path, &v));
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
            "video" | "videos" => {
                if rom.video.is_none() {
                    rom.video = Some(full_path);
                }
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
        developer: Option<String>,
        publisher: Option<String>,
        genre: Option<String>,
        players: Option<String>,
        releasedate: Option<String>,
        rating: Option<f32>,
        #[serde(rename = "english-name")]
        english_name: Option<String>,
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

            Some(RomInfo {
                file,
                name,
                description: g.desc,
                summary: None,
                developer: g.developer,
                publisher: g.publisher,
                genre: g.genre,
                players: g.players,
                release: g.releasedate,
                rating: g.rating.map(|r| format!("{}%", (r * 100.0) as i32)),
                directory: dir_path.to_string_lossy().to_string(),
                system: system_name.to_string(),
                has_temp_metadata: false,
                temp_data: None,
                box_front: g.image.map(|image| resolve_media_path(dir_path, &image)),
                box_back: None,
                box_spine: None,
                box_full: None,
                cartridge: None,
                logo: None,
                marquee: None,
                bezel: None,
                gridicon: None,
                flyer: None,
                background: None,
                music: None,
                screenshot: None,
                titlescreen: None,
                video: None,
                english_name: g.english_name,
            })
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

    println!("[DEBUG] 开始 read_dir...");
    if let Ok(entries) = fs::read_dir(dir_path) {
        println!("[DEBUG] read_dir 成功，开始遍历...");
        let mut count = 0;
        for entry in entries.filter_map(|e| e.ok()) {
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
                        match ps3::parse_param_sfo(&param_sfo_path) {
                            Ok(info) => Some(info),
                            Err(_) => None,
                        }
                    } else {
                        None
                    };

                    // 使用解析出的信息或默认值
                    let game_name = game_info
                        .as_ref()
                        .and_then(|info| info.title.clone())
                        .unwrap_or_else(|| folder_name.clone());

                    let game_id = game_info.as_ref().and_then(|info| info.title_id.clone());

                    roms.push(RomInfo {
                        file: folder_name,
                        name: game_name,
                        description: game_id.map(|id| format!("Game ID: {}", id)),
                        summary: None,
                        developer: None,
                        publisher: None,
                        genre: None,
                        players: None,
                        release: None,
                        rating: None,
                        directory: dir_path.to_string_lossy().to_string(),
                        system: system_name.to_string(),
                        box_front: None,
                        box_back: None,
                        box_spine: None,
                        box_full: None,
                        cartridge: None,
                        logo: None,
                        marquee: None,
                        bezel: None,
                        gridicon: None,
                        flyer: None,
                        background: None,
                        music: None,
                        screenshot: None,
                        titlescreen: None,
                        video: None,
                        english_name: None,
                        has_temp_metadata: false,
                        temp_data: None,
                    });
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

                        roms.push(RomInfo {
                            file: filename,
                            name: game_name,
                            description: game_description,
                            summary: None,
                            developer: None,
                            publisher: None,
                            genre: None,
                            players: None,
                            release: None,
                            rating: None,
                            directory: dir_path.to_string_lossy().to_string(),
                            system: system_name.to_string(),
                            box_front: None,
                            box_back: None,
                            box_spine: None,
                            box_full: None,
                            cartridge: None,
                            logo: None,
                            marquee: None,
                            bezel: None,
                            gridicon: None,
                            flyer: None,
                            background: None,
                            music: None,
                            screenshot: None,
                            titlescreen: None,
                            video: None,
                            english_name: None,
                            has_temp_metadata: false,
                            temp_data: None,
                        });
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
    <rating>0.95</rating>
  </game>
</gameList>
"#;
        fs::write(dir.join("gamelist.xml"), xml).expect("write gamelist.xml");

        let roms = read_emulationstation_roms(&dir, "snes").expect("parse gamelist");
        assert_eq!(roms.len(), 1);
        assert_eq!(roms[0].file, "Super Mario World.sfc");
        assert_eq!(roms[0].name, "Super Mario World");
        assert_eq!(roms[0].system, "snes");

        let _ = fs::remove_dir_all(&dir);
    }
}
