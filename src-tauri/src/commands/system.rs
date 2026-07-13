use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RomType {
    File,   // 文件格式ROM
    Folder, // 文件夹格式ROM（如PS3）
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub manufacturer: Option<String>,
    pub release_year: Option<i32>,
    pub extensions: Vec<String>,
    pub logo: Option<String>, // Logo文件名（相对于resources/logo/）
    pub rom_type: RomType,    // ROM处理方式
}

pub(crate) fn get_preset_systems_data() -> Vec<SystemInfo> {
    vec![
        // Nintendo - FC/NES 系列
        SystemInfo {
            id: "nes".to_string(),
            name: "Nintendo Entertainment System".to_string(),
            short_name: "NES".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1983),
            extensions: vec![".nes".to_string(), ".zip".to_string()],
            logo: Some("FC.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "fc".to_string(),
            name: "FC / NES".to_string(),
            short_name: "FC".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1983),
            extensions: vec![".nes".to_string(), ".zip".to_string()],
            logo: Some("FC.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "fc-hd".to_string(),
            name: "FC HD".to_string(),
            short_name: "FC HD".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: None,
            extensions: vec![".nes".to_string(), ".zip".to_string()],
            logo: Some("FC-HD.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "fc hack".to_string(),
            name: "FC Hack".to_string(),
            short_name: "FC Hack".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: None,
            extensions: vec![".nes".to_string(), ".zip".to_string()],
            logo: Some("FC hack.png".to_string()),
            rom_type: RomType::File,
        },
        // Nintendo - SFC/SNES 系列
        SystemInfo {
            id: "snes".to_string(),
            name: "Super Nintendo Entertainment System".to_string(),
            short_name: "SNES".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1990),
            extensions: vec![
                ".sfc".to_string(),
                ".smc".to_string(),
                ".fig".to_string(),
                ".smd".to_string(),
                ".zip".to_string(),
            ],
            logo: Some("SFC.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "sfc".to_string(),
            name: "SFC / SNES".to_string(),
            short_name: "SFC".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1990),
            extensions: vec![
                ".sfc".to_string(),
                ".smc".to_string(),
                ".fig".to_string(),
                ".smd".to_string(),
                ".zip".to_string(),
            ],
            logo: Some("SFC.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "sfc hack".to_string(),
            name: "SFC Hack".to_string(),
            short_name: "SFC Hack".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: None,
            extensions: vec![".sfc".to_string(), ".smc".to_string(), ".zip".to_string()],
            logo: Some("SFC hack.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "sfc-msu1".to_string(),
            name: "SFC MSU-1".to_string(),
            short_name: "SFC MSU-1".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: None,
            extensions: vec![".sfc".to_string(), ".smc".to_string(), ".zip".to_string()],
            logo: Some("SFC-MSU1.png".to_string()),
            rom_type: RomType::File,
        },
        // Nintendo - N64
        SystemInfo {
            id: "n64".to_string(),
            name: "Nintendo 64".to_string(),
            short_name: "N64".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1996),
            extensions: vec![
                ".z64".to_string(),
                ".n64".to_string(),
                ".v64".to_string(),
                ".zip".to_string(),
            ],
            logo: Some("N64.png".to_string()),
            rom_type: RomType::File,
        },
        // Nintendo - GameCube
        SystemInfo {
            id: "gc".to_string(),
            name: "Nintendo GameCube".to_string(),
            short_name: "GameCube".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(2001),
            extensions: vec![".iso".to_string(), ".gcm".to_string()],
            logo: Some("NGC.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "ngc".to_string(),
            name: "Nintendo GameCube".to_string(),
            short_name: "NGC".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(2001),
            extensions: vec![".iso".to_string(), ".gcm".to_string()],
            logo: Some("NGC.png".to_string()),
            rom_type: RomType::File,
        },
        // Nintendo - Wii
        SystemInfo {
            id: "wii".to_string(),
            name: "Nintendo Wii".to_string(),
            short_name: "Wii".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(2006),
            extensions: vec![".iso".to_string(), ".wbfs".to_string()],
            logo: Some("Wii.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "wii ware".to_string(),
            name: "Wii Ware".to_string(),
            short_name: "WiiWare".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: None,
            extensions: vec![".wad".to_string()],
            logo: Some("Wii.png".to_string()),
            rom_type: RomType::File,
        },
        // Nintendo - 掌机系列
        SystemInfo {
            id: "gb".to_string(),
            name: "Game Boy".to_string(),
            short_name: "Game Boy".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1989),
            extensions: vec![".gb".to_string(), ".zip".to_string()],
            logo: Some("GB.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "gbc".to_string(),
            name: "Game Boy Color".to_string(),
            short_name: "GBC".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1998),
            extensions: vec![".gbc".to_string(), ".zip".to_string()],
            logo: Some("GBC.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "gba".to_string(),
            name: "Game Boy Advance".to_string(),
            short_name: "GBA".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(2001),
            extensions: vec![".gba".to_string(), ".zip".to_string()],
            logo: Some("GBA.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "nds".to_string(),
            name: "Nintendo DS".to_string(),
            short_name: "NDS".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(2004),
            extensions: vec![".nds".to_string(), ".zip".to_string()],
            logo: Some("NDS.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "3ds".to_string(),
            name: "Nintendo 3DS".to_string(),
            short_name: "3DS".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(2011),
            extensions: vec![".3ds".to_string(), ".cia".to_string()],
            logo: Some("3DS.png".to_string()),
            rom_type: RomType::File,
        },
        // Nintendo - 其他
        SystemInfo {
            id: "virtual boy".to_string(),
            name: "Virtual Boy".to_string(),
            short_name: "VB".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1995),
            extensions: vec![".vb".to_string(), ".zip".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "game watch".to_string(),
            name: "Game & Watch".to_string(),
            short_name: "G&W".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1980),
            extensions: vec![".mgw".to_string(), ".zip".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "poke mini".to_string(),
            name: "Pokemon Mini".to_string(),
            short_name: "PokeMini".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(2001),
            extensions: vec![".min".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        // Sega - 主机系列
        SystemInfo {
            id: "sms".to_string(),
            name: "Sega Master System".to_string(),
            short_name: "SMS".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1985),
            extensions: vec![".sms".to_string(), ".zip".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "genesis".to_string(),
            name: "Sega Genesis / Mega Drive".to_string(),
            short_name: "Genesis".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1988),
            extensions: vec![
                ".md".to_string(),
                ".bin".to_string(),
                ".gen".to_string(),
                ".smd".to_string(),
                ".zip".to_string(),
            ],
            logo: Some("MD.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "md".to_string(),
            name: "Mega Drive / Genesis".to_string(),
            short_name: "MD".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1988),
            extensions: vec![
                ".md".to_string(),
                ".bin".to_string(),
                ".gen".to_string(),
                ".smd".to_string(),
                ".zip".to_string(),
            ],
            logo: Some("MD.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "md hack".to_string(),
            name: "MD Hack".to_string(),
            short_name: "MD Hack".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: None,
            extensions: vec![".md".to_string(), ".bin".to_string(), ".zip".to_string()],
            logo: Some("MD hack.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "md-32x".to_string(),
            name: "Sega 32X".to_string(),
            short_name: "32X".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1994),
            extensions: vec![".32x".to_string(), ".zip".to_string()],
            logo: Some("MD-32X.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "gg".to_string(),
            name: "Sega Game Gear".to_string(),
            short_name: "GG".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1990),
            extensions: vec![".gg".to_string(), ".zip".to_string()],
            logo: Some("GG.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "saturn".to_string(),
            name: "Sega Saturn".to_string(),
            short_name: "Saturn".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1994),
            extensions: vec![
                ".iso".to_string(),
                ".cue".to_string(),
                ".bin".to_string(),
                ".chd".to_string(),
                ".img".to_string(),
            ],
            logo: Some("SS.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "ss".to_string(),
            name: "Sega Saturn".to_string(),
            short_name: "SS".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1994),
            extensions: vec![
                ".iso".to_string(),
                ".cue".to_string(),
                ".bin".to_string(),
                ".chd".to_string(),
                ".img".to_string(),
            ],
            logo: Some("SS.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "dreamcast".to_string(),
            name: "Sega Dreamcast".to_string(),
            short_name: "DC".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1998),
            extensions: vec![
                ".cdi".to_string(),
                ".gdi".to_string(),
                ".iso".to_string(),
                ".chd".to_string(),
                ".zip".to_string(),
            ],
            logo: Some("DC.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "dc".to_string(),
            name: "Sega Dreamcast".to_string(),
            short_name: "DC".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1998),
            extensions: vec![
                ".cdi".to_string(),
                ".gdi".to_string(),
                ".iso".to_string(),
                ".chd".to_string(),
                ".zip".to_string(),
            ],
            logo: Some("DC.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "dc hack".to_string(),
            name: "DC Hack".to_string(),
            short_name: "DC Hack".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: None,
            extensions: vec![".cdi".to_string(), ".gdi".to_string()],
            logo: Some("DC hack.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "naomi".to_string(),
            name: "Sega NAOMI".to_string(),
            short_name: "NAOMI".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1998),
            extensions: vec![".zip".to_string(), ".bin".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        // Sony - PlayStation 系列
        SystemInfo {
            id: "psx".to_string(),
            name: "PlayStation".to_string(),
            short_name: "PSX".to_string(),
            manufacturer: Some("Sony".to_string()),
            release_year: Some(1994),
            extensions: vec![
                ".iso".to_string(),
                ".bin".to_string(),
                ".cue".to_string(),
                ".pbp".to_string(),
            ],
            logo: Some("PS1.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "ps1".to_string(),
            name: "PlayStation".to_string(),
            short_name: "PS1".to_string(),
            manufacturer: Some("Sony".to_string()),
            release_year: Some(1994),
            extensions: vec![
                ".iso".to_string(),
                ".bin".to_string(),
                ".cue".to_string(),
                ".pbp".to_string(),
            ],
            logo: Some("PS1.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "ps1 hack".to_string(),
            name: "PS1 Hack".to_string(),
            short_name: "PS1 Hack".to_string(),
            manufacturer: Some("Sony".to_string()),
            release_year: None,
            extensions: vec![".iso".to_string(), ".bin".to_string(), ".cue".to_string()],
            logo: Some("PS1 hack.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "ps2".to_string(),
            name: "PlayStation 2".to_string(),
            short_name: "PS2".to_string(),
            manufacturer: Some("Sony".to_string()),
            release_year: Some(2000),
            extensions: vec![".iso".to_string(), ".bin".to_string()],
            logo: Some("PS2.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "ps3".to_string(),
            name: "PlayStation 3".to_string(),
            short_name: "PS3".to_string(),
            manufacturer: Some("Sony".to_string()),
            release_year: Some(2006),
            extensions: vec![".iso".to_string(), ".pkg".to_string(), ".bin".to_string()],
            logo: Some("PS3.png".to_string()),
            rom_type: RomType::Folder,
        },
        SystemInfo {
            id: "psp".to_string(),
            name: "PlayStation Portable".to_string(),
            short_name: "PSP".to_string(),
            manufacturer: Some("Sony".to_string()),
            release_year: Some(2004),
            extensions: vec![".iso".to_string(), ".cso".to_string()],
            logo: Some("PSP.png".to_string()),
            rom_type: RomType::File,
        },
        // NEC
        SystemInfo {
            id: "pce".to_string(),
            name: "PC Engine / TurboGrafx-16".to_string(),
            short_name: "PCE".to_string(),
            manufacturer: Some("NEC".to_string()),
            release_year: Some(1987),
            extensions: vec![".pce".to_string(), ".zip".to_string()],
            logo: Some("PCE.png".to_string()),
            rom_type: RomType::File,
        },
        // SNK
        SystemInfo {
            id: "neogeo".to_string(),
            name: "Neo Geo".to_string(),
            short_name: "Neo Geo".to_string(),
            manufacturer: Some("SNK".to_string()),
            release_year: Some(1990),
            extensions: vec![".zip".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "ngpc".to_string(),
            name: "Neo Geo Pocket Color".to_string(),
            short_name: "NGPC".to_string(),
            manufacturer: Some("SNK".to_string()),
            release_year: Some(1999),
            extensions: vec![".ngc".to_string(), ".ngp".to_string(), ".zip".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        // Bandai
        SystemInfo {
            id: "ws".to_string(),
            name: "WonderSwan".to_string(),
            short_name: "WS".to_string(),
            manufacturer: Some("Bandai".to_string()),
            release_year: Some(1999),
            extensions: vec![".ws".to_string(), ".zip".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "wsc".to_string(),
            name: "WonderSwan Color".to_string(),
            short_name: "WSC".to_string(),
            manufacturer: Some("Bandai".to_string()),
            release_year: Some(2000),
            extensions: vec![".wsc".to_string(), ".zip".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        // Atari
        SystemInfo {
            id: "atari2600".to_string(),
            name: "Atari 2600".to_string(),
            short_name: "2600".to_string(),
            manufacturer: Some("Atari".to_string()),
            release_year: Some(1977),
            extensions: vec![".a26".to_string(), ".bin".to_string(), ".zip".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "atari5200".to_string(),
            name: "Atari 5200".to_string(),
            short_name: "5200".to_string(),
            manufacturer: Some("Atari".to_string()),
            release_year: Some(1982),
            extensions: vec![".a52".to_string(), ".bin".to_string(), ".zip".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "atari7800".to_string(),
            name: "Atari 7800".to_string(),
            short_name: "7800".to_string(),
            manufacturer: Some("Atari".to_string()),
            release_year: Some(1986),
            extensions: vec![".a78".to_string(), ".bin".to_string(), ".zip".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "lynx".to_string(),
            name: "Atari Lynx".to_string(),
            short_name: "Lynx".to_string(),
            manufacturer: Some("Atari".to_string()),
            release_year: Some(1989),
            extensions: vec![".lnx".to_string(), ".zip".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        // Arcade - FBNeo 分类
        SystemInfo {
            id: "fbneo act".to_string(),
            name: "FBNeo 动作".to_string(),
            short_name: "FBNeo ACT".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: Some("动作街机.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "fbneo stg".to_string(),
            name: "FBNeo 射击".to_string(),
            short_name: "FBNeo STG".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: Some("射击街机.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "fbneo ftg".to_string(),
            name: "FBNeo 格斗".to_string(),
            short_name: "FBNeo FTG".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: Some("格斗街机.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "fbneo fly".to_string(),
            name: "FBNeo 飞行".to_string(),
            short_name: "FBNeo FLY".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: Some("飞行街机.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "fbneo rac".to_string(),
            name: "FBNeo 竞速".to_string(),
            short_name: "FBNeo RAC".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: Some("竞速街机.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "fbneo spo".to_string(),
            name: "FBNeo 体育".to_string(),
            short_name: "FBNeo SPO".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: Some("体育街机.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "fbneo etc".to_string(),
            name: "FBNeo 其他".to_string(),
            short_name: "FBNeo ETC".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: Some("其他街机.png".to_string()),
            rom_type: RomType::File,
        },
        // Arcade - MAME 分类
        SystemInfo {
            id: "mame act".to_string(),
            name: "MAME 动作".to_string(),
            short_name: "MAME ACT".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: Some("动作街机.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "mame stg".to_string(),
            name: "MAME 射击".to_string(),
            short_name: "MAME STG".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: Some("射击街机.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "mame ftg".to_string(),
            name: "MAME 格斗".to_string(),
            short_name: "MAME FTG".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: Some("格斗街机.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "mame fly".to_string(),
            name: "MAME 飞行".to_string(),
            short_name: "MAME FLY".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: Some("飞行街机.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "mame rac".to_string(),
            name: "MAME 竞速".to_string(),
            short_name: "MAME RAC".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: Some("竞速街机.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "mame spo".to_string(),
            name: "MAME 体育".to_string(),
            short_name: "MAME SPO".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: Some("体育街机.png".to_string()),
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "mame etc".to_string(),
            name: "MAME 其他".to_string(),
            short_name: "MAME ETC".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: Some("其他街机.png".to_string()),
            rom_type: RomType::File,
        },
        // Arcade - 其他
        SystemInfo {
            id: "arcade".to_string(),
            name: "Arcade".to_string(),
            short_name: "Arcade".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "light gun".to_string(),
            name: "光枪游戏".to_string(),
            short_name: "Light Gun".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
            logo: Some("光枪街机.png".to_string()),
            rom_type: RomType::File,
        },
        // 实际整合包常见平台别名与独立引擎
        SystemInfo {
            id: "ms".to_string(),
            name: "Sega Master System".to_string(),
            short_name: "MS".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1985),
            extensions: vec![".sms".to_string(), ".zip".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "gw".to_string(),
            name: "Game & Watch".to_string(),
            short_name: "GW".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1980),
            extensions: vec![".mgw".to_string(), ".zip".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "openbor".to_string(),
            name: "OpenBOR".to_string(),
            short_name: "OPENBOR".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".pak".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "pgm".to_string(),
            name: "PolyGame Master".to_string(),
            short_name: "PGM".to_string(),
            manufacturer: Some("IGS".to_string()),
            release_year: Some(1997),
            extensions: vec![".zip".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "pico8".to_string(),
            name: "PICO-8".to_string(),
            short_name: "PICO8".to_string(),
            manufacturer: Some("Lexaloffle".to_string()),
            release_year: Some(2015),
            extensions: vec![".p8".to_string(), ".p8.png".to_string()],
            logo: None,
            rom_type: RomType::File,
        },
        SystemInfo {
            id: "easyrpg".to_string(),
            name: "EasyRPG".to_string(),
            short_name: "EASYRPG".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![],
            logo: None,
            rom_type: RomType::Folder,
        },
        SystemInfo {
            id: "ps".to_string(),
            name: "PlayStation".to_string(),
            short_name: "PS".to_string(),
            manufacturer: Some("Sony".to_string()),
            release_year: Some(1994),
            extensions: vec![
                ".pbp".to_string(),
                ".chd".to_string(),
                ".cue".to_string(),
                ".bin".to_string(),
                ".img".to_string(),
                ".iso".to_string(),
            ],
            logo: Some("PS1.png".to_string()),
            rom_type: RomType::File,
        },
        // PC
        SystemInfo {
            id: "dos".to_string(),
            name: "DOS".to_string(),
            short_name: "DOS".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".exe".to_string(), ".bat".to_string(), ".com".to_string()],
            logo: Some("DOS.png".to_string()),
            rom_type: RomType::File,
        },
    ]
}

/// 用 system_mapping 中的 logo_name 补齐缺失的系统 logo(按 id / short_name 匹配,不覆盖已有值)
fn fill_missing_logos(mut systems: Vec<SystemInfo>) -> Vec<SystemInfo> {
    use crate::system_mapping::find_logo_name_by_folder;

    for system in &mut systems {
        if system.logo.is_none() {
            system.logo = find_logo_name_by_folder(&system.id)
                .or_else(|| find_logo_name_by_folder(&system.short_name))
                .map(|name| name.to_string());
        }
    }
    systems
}

/// 获取所有游戏系统
#[tauri::command]
pub fn get_systems() -> Result<Vec<SystemInfo>, String> {
    Ok(fill_missing_logos(get_preset_systems_data()))
}

/// 获取单个游戏系统
#[tauri::command]
pub fn get_system(id: String) -> Result<Option<SystemInfo>, String> {
    let systems = fill_missing_logos(get_preset_systems_data());
    Ok(systems.into_iter().find(|s| s.id == id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system_mapping::find_logo_name_by_folder;

    #[test]
    fn get_systems_fills_logo_from_system_mapping() {
        let systems = get_systems().unwrap();

        // Virtual Boy 在预置数据中 logo 为 None,但 system_mapping 提供了 VB.png
        let vb = systems.iter().find(|s| s.id == "virtual boy").unwrap();
        assert_eq!(vb.logo.as_deref(), Some("VB.png"));

        // 凡是能在 system_mapping 中(按 id 或 short_name)找到 logo_name 的系统,logo 均非空
        for system in &systems {
            let mapped = find_logo_name_by_folder(&system.id)
                .or_else(|| find_logo_name_by_folder(&system.short_name));
            if mapped.is_some() {
                assert!(
                    system.logo.is_some(),
                    "系统 {} 在 system_mapping 中有 logo_name,但 get_systems 输出的 logo 为空",
                    system.id
                );
            }
        }
    }

    #[test]
    fn system_info_serializes_as_camel_case() {
        let systems = get_systems().unwrap();
        let nes = systems.iter().find(|s| s.id == "nes").unwrap();
        let value = serde_json::to_value(nes).unwrap();
        let obj = value.as_object().unwrap();

        // camelCase 字段存在
        assert_eq!(obj.get("shortName").and_then(|v| v.as_str()), Some("NES"));
        assert_eq!(obj.get("releaseYear").and_then(|v| v.as_i64()), Some(1983));
        assert_eq!(obj.get("romType").and_then(|v| v.as_str()), Some("File"));

        // snake_case 字段不再输出
        assert!(!obj.contains_key("short_name"));
        assert!(!obj.contains_key("release_year"));
        assert!(!obj.contains_key("rom_type"));
    }

    #[test]
    fn fill_missing_logos_keeps_existing_logo() {
        let systems = get_systems().unwrap();
        // 预置数据里已有 logo 的系统不被 mapping 覆盖(合并语义:只补缺)
        let nes = systems.iter().find(|s| s.id == "nes").unwrap();
        assert_eq!(nes.logo.as_deref(), Some("FC.png"));
    }
}
