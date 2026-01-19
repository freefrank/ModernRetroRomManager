//! ROM 数据服务 - 从 metadata 文件读取 ROM 信息
//!
//! 不存储到数据库，运行时直接解析 metadata 文件

use crate::ps3_sfo;
use crate::scraper::pegasus::{parse_pegasus_file, PegasusGame};
use crate::settings::get_settings;
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
            english_name: game.extra.get("x-english-name").cloned(),
            temp_data: None,

            has_temp_metadata: false,
        }
    }
}


use crate::config::get_temp_dir;

/// 尝试加载并应用临时元数据
fn apply_temp_metadata(roms: &mut [RomInfo], system: &str) {
    let temp_metadata_path = get_temp_dir().join(system).join("metadata.txt");
    if !temp_metadata_path.exists() {
        return;
    }

    if let Ok(temp_metadata) = parse_pegasus_file(&temp_metadata_path) {
        for rom in roms {
            if let Some(temp_game) = temp_metadata.games.iter().find(|g| g.file == Some(rom.file.clone())) {
                rom.has_temp_metadata = true;
                rom.temp_data = Some(temp_game.clone());
            }
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
                        
                        if let Ok(mut roms) = get_roms_from_directory(&sub_path, &format, &system_name) {
                            if !roms.is_empty() {
                                // 尝试加载临时元数据
                                apply_temp_metadata(&mut roms, &system_name);
                                
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
            
            if let Ok(mut roms) = get_roms_from_directory(dir_path, &dir_config.metadata_format, &system_name) {
                if !roms.is_empty() {
                    // 尝试加载临时元数据
                    apply_temp_metadata(&mut roms, &system_name);

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

fn read_pegasus_roms(dir_path: &Path, system_name: &str) -> Result<Vec<RomInfo>, String> {
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
                    resolve_all_media_paths(&mut rom, dir_path);
                    scan_media_directory(&mut rom, dir_path);
                    rom
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
        
        dir_path.join(&normalized_value).to_string_lossy().to_string()
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
                if rom.box_front.is_none() { rom.box_front = Some(full_path); }
            }
            "boxback" | "box_back" => {
                if rom.box_back.is_none() { rom.box_back = Some(full_path); }
            }
            "boxspine" | "box_spine" => {
                if rom.box_spine.is_none() { rom.box_spine = Some(full_path); }
            }
            "boxfull" | "box_full" => {
                if rom.box_full.is_none() { rom.box_full = Some(full_path); }
            }
            "cartridge" | "cart" | "disc" => {
                if rom.cartridge.is_none() { rom.cartridge = Some(full_path); }
            }
            "logo" | "wheel" => {
                if rom.logo.is_none() { rom.logo = Some(full_path); }
            }
            "marquee" | "banner" => {
                if rom.marquee.is_none() { rom.marquee = Some(full_path); }
            }
            "bezel" | "screenmarquee" => {
                if rom.bezel.is_none() { rom.bezel = Some(full_path); }
            }
            "gridicon" | "steam" | "poster" => {
                if rom.gridicon.is_none() { rom.gridicon = Some(full_path); }
            }
            "flyer" => {
                if rom.flyer.is_none() { rom.flyer = Some(full_path); }
            }
            "background" | "fanart" => {
                if rom.background.is_none() { rom.background = Some(full_path); }
            }
            "music" => {
                if rom.music.is_none() { rom.music = Some(full_path); }
            }
            "screenshot" | "screenshots" | "screen" => {
                if rom.screenshot.is_none() { rom.screenshot = Some(full_path); }
            }
            "titlescreen" | "title_screen" | "title" => {
                if rom.titlescreen.is_none() { rom.titlescreen = Some(full_path); }
            }
            "video" | "videos" => {
                if rom.video.is_none() { rom.video = Some(full_path); }
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

            // PS3 特殊处理：扫描文件夹作为 ROM
            if system_name.to_lowercase() == "ps3" && path.is_dir() {
                let folder_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                // 检查是否包含 PS3_GAME 目录
                let ps3_game_dir = path.join("PS3_GAME");
                let is_valid_ps3_folder = ps3_game_dir.exists() && ps3_game_dir.is_dir();

                if is_valid_ps3_folder || true {  // 暂时不做严格验证
                    // 尝试解析 PARAM.SFO 获取游戏信息
                    let param_sfo_path = ps3_game_dir.join("PARAM.SFO");
                    let game_info = if param_sfo_path.exists() {
                        ps3_sfo::parse_param_sfo(&param_sfo_path).ok()
                    } else {
                        None
                    };

                    // 使用解析出的信息或默认值
                    let game_name = game_info.as_ref()
                        .and_then(|info| info.title.clone())
                        .unwrap_or_else(|| folder_name.clone());

                    let game_id = game_info.as_ref()
                        .and_then(|info| info.title_id.clone());

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
                    if rom_extensions.contains(&ext.to_lowercase().as_str()) {
                        let filename = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        let default_name = path.file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        // PS3 ISO 特殊处理：尝试从 ISO 中提取 PARAM.SFO
                        let (game_name, game_description) = if system_name.to_lowercase() == "ps3"
                            && ext.to_lowercase() == "iso" {
                            match ps3_sfo::parse_param_sfo_from_iso(&path) {
                                Ok(game_info) => {
                                    let name = game_info.title.clone().unwrap_or_else(|| default_name.clone());
                                    let desc = game_info.title_id.map(|id| format!("Game ID: {}", id));
                                    (name, desc)
                                }
                                Err(_) => (default_name.clone(), None)
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
    }
    
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
