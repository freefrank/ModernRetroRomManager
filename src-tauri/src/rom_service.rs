//! ROM 数据服务 - 从 metadata 文件读取 ROM 信息
//!
//! 不存储到数据库，运行时直接解析 metadata 文件

use crate::scraper::pegasus::{parse_pegasus_file, PegasusGame};
use crate::settings::{get_settings, DirectoryConfig};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// ROM 信息（运行时数据）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomInfo {
    /// 文件路径（相对于目录）
    pub file: String,
    /// 游戏名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 简介
    pub summary: Option<String>,
    /// 开发商
    pub developer: Option<String>,
    /// 发行商
    pub publisher: Option<String>,
    /// 类型
    pub genre: Option<String>,
    /// 玩家数
    pub players: Option<String>,
    /// 发布日期
    pub release: Option<String>,
    /// 评分
    pub rating: Option<String>,
    /// 封面图路径（相对路径）
    pub boxart: Option<String>,
    /// 所属目录
    pub directory: String,
    /// 系统名称（目录名或指定的 system_id）
    pub system: String,
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
            boxart: None,
            directory: String::new(),
            system: String::new(),
        }
    }
}

/// 获取所有目录的 ROM 列表
pub fn get_all_roms() -> Result<Vec<SystemRoms>, String> {
    let settings = get_settings();
    let mut all_systems = Vec::new();
    
    for dir_config in &settings.directories {
        let dir_path = Path::new(&dir_config.path);
        
        if !dir_path.exists() {
            continue;
        }
        
        if dir_config.is_root_directory {
            // ROMs 根目录模式：扫描子目录
            if let Ok(entries) = std::fs::read_dir(dir_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let sub_path = entry.path();
                    if sub_path.is_dir() {
                        let system_name = sub_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        
                        // 自动检测子目录的 metadata 格式
                        let format = detect_metadata_format(&sub_path);
                        
                        if let Ok(roms) = get_roms_from_directory(&sub_path, &format, &system_name) {
                            if !roms.is_empty() {
                                all_systems.push(SystemRoms {
                                    system: system_name,
                                    path: sub_path.to_string_lossy().to_string(),
                                    roms,
                                });
                            }
                        }
                    }
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
            
            if let Ok(roms) = get_roms_from_directory(dir_path, &dir_config.metadata_format, &system_name) {
                if !roms.is_empty() {
                    all_systems.push(SystemRoms {
                        system: system_name,
                        path: dir_config.path.clone(),
                        roms,
                    });
                }
            }
        }
    }
    
    Ok(all_systems)
}

/// 自动检测目录的 metadata 格式
fn detect_metadata_format(dir_path: &Path) -> String {
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

/// 读取 Pegasus 格式
fn read_pegasus_roms(dir_path: &Path, system_name: &str) -> Result<Vec<RomInfo>, String> {
    // 尝试多个可能的文件名
    let possible_files = ["metadata.pegasus.txt", "metadata.txt"];
    
    for filename in &possible_files {
        let metadata_path = dir_path.join(filename);
        if metadata_path.exists() {
            let metadata = parse_pegasus_file(&metadata_path)?;
            return Ok(metadata
                .games
                .into_iter()
                .map(|g| {
                    let mut rom: RomInfo = g.into();
                    rom.directory = dir_path.to_string_lossy().to_string();
                    rom.system = system_name.to_string();
                    rom
                })
                .collect());
        }
    }
    
    // 没有找到 metadata 文件，扫描文件
    scan_rom_files(dir_path, system_name)
}

/// 读取 EmulationStation gamelist.xml 格式
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
    }
    
    let gamelist_path = dir_path.join("gamelist.xml");
    if !gamelist_path.exists() {
        return scan_rom_files(dir_path, system_name);
    }
    
    let file = File::open(&gamelist_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    
    let game_list: EsGameList = from_reader(reader)
        .map_err(|e| format!("Failed to parse gamelist.xml: {}", e))?;
    
    Ok(game_list
        .games
        .into_iter()
        .map(|g| {
            let file = g.path.trim_start_matches("./").to_string();
            let name = g.name.unwrap_or_else(|| {
                Path::new(&file)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            });
            
            RomInfo {
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
                boxart: g.image,
                directory: dir_path.to_string_lossy().to_string(),
                system: system_name.to_string(),
            }
        })
        .collect())
}

/// 无 metadata 时扫描 ROM 文件
fn scan_rom_files(dir_path: &Path, system_name: &str) -> Result<Vec<RomInfo>, String> {
    use std::fs;
    
    // 常见 ROM 扩展名
    let rom_extensions = [
        "nes", "sfc", "smc", "gba", "gb", "gbc", "n64", "z64", "v64",
        "iso", "bin", "cue", "img", "zip", "7z", "rar",
        "md", "gen", "smd", "gg", "sms",
        "pce", "ngp", "ngc", "ws", "wsc",
        "a26", "a52", "a78", "lnx",
        "nds", "3ds", "cia",
        "psx", "pbp", "chd",
    ];
    
    let mut roms = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if rom_extensions.contains(&ext.to_lowercase().as_str()) {
                        let filename = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        
                        let name = path.file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        
                        roms.push(RomInfo {
                            file: filename,
                            name,
                            description: None,
                            summary: None,
                            developer: None,
                            publisher: None,
                            genre: None,
                            players: None,
                            release: None,
                            rating: None,
                            boxart: None,
                            directory: dir_path.to_string_lossy().to_string(),
                            system: system_name.to_string(),
                        });
                    }
                }
            }
        }
    }
    
    Ok(roms)
}
