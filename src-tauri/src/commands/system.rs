use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemInfo {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub manufacturer: Option<String>,
    pub release_year: Option<i32>,
    pub extensions: Vec<String>,
}

fn get_preset_systems_data() -> Vec<SystemInfo> {
    vec![
        SystemInfo {
            id: "nes".to_string(),
            name: "Nintendo Entertainment System".to_string(),
            short_name: "NES".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1983),
            extensions: vec![".nes".to_string(), ".zip".to_string()],
        },
        SystemInfo {
            id: "snes".to_string(),
            name: "Super Nintendo Entertainment System".to_string(),
            short_name: "SNES".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1990),
            extensions: vec![".sfc".to_string(), ".smc".to_string(), ".zip".to_string()],
        },
        SystemInfo {
            id: "n64".to_string(),
            name: "Nintendo 64".to_string(),
            short_name: "N64".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1996),
            extensions: vec![".z64".to_string(), ".n64".to_string(), ".zip".to_string()],
        },
        SystemInfo {
            id: "gc".to_string(),
            name: "Nintendo GameCube".to_string(),
            short_name: "GameCube".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(2001),
            extensions: vec![".iso".to_string(), ".gcm".to_string()],
        },
        SystemInfo {
            id: "gb".to_string(),
            name: "Game Boy".to_string(),
            short_name: "Game Boy".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1989),
            extensions: vec![".gb".to_string(), ".zip".to_string()],
        },
        SystemInfo {
            id: "gbc".to_string(),
            name: "Game Boy Color".to_string(),
            short_name: "GBC".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1998),
            extensions: vec![".gbc".to_string(), ".zip".to_string()],
        },
        SystemInfo {
            id: "gba".to_string(),
            name: "Game Boy Advance".to_string(),
            short_name: "GBA".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(2001),
            extensions: vec![".gba".to_string(), ".zip".to_string()],
        },
        SystemInfo {
            id: "nds".to_string(),
            name: "Nintendo DS".to_string(),
            short_name: "NDS".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(2004),
            extensions: vec![".nds".to_string(), ".zip".to_string()],
        },
        SystemInfo {
            id: "genesis".to_string(),
            name: "Sega Genesis / Mega Drive".to_string(),
            short_name: "Genesis".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1988),
            extensions: vec![".md".to_string(), ".bin".to_string(), ".gen".to_string(), ".zip".to_string()],
        },
        SystemInfo {
            id: "saturn".to_string(),
            name: "Sega Saturn".to_string(),
            short_name: "Saturn".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1994),
            extensions: vec![".iso".to_string(), ".cue".to_string(), ".bin".to_string()],
        },
        SystemInfo {
            id: "dreamcast".to_string(),
            name: "Sega Dreamcast".to_string(),
            short_name: "DC".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1998),
            extensions: vec![".cdi".to_string(), ".gdi".to_string(), ".iso".to_string()],
        },
        SystemInfo {
            id: "psx".to_string(),
            name: "PlayStation".to_string(),
            short_name: "PSX".to_string(),
            manufacturer: Some("Sony".to_string()),
            release_year: Some(1994),
            extensions: vec![".iso".to_string(), ".bin".to_string(), ".cue".to_string(), ".pbp".to_string()],
        },
        SystemInfo {
            id: "ps2".to_string(),
            name: "PlayStation 2".to_string(),
            short_name: "PS2".to_string(),
            manufacturer: Some("Sony".to_string()),
            release_year: Some(2000),
            extensions: vec![".iso".to_string(), ".bin".to_string()],
        },
        SystemInfo {
            id: "ps3".to_string(),
            name: "PlayStation 3".to_string(),
            short_name: "PS3".to_string(),
            manufacturer: Some("Sony".to_string()),
            release_year: Some(2006),
            extensions: vec![".iso".to_string(), ".pkg".to_string(), ".bin".to_string()],
        },
        SystemInfo {
            id: "psp".to_string(),
            name: "PlayStation Portable".to_string(),
            short_name: "PSP".to_string(),
            manufacturer: Some("Sony".to_string()),
            release_year: Some(2004),
            extensions: vec![".iso".to_string(), ".cso".to_string()],
        },
        SystemInfo {
            id: "arcade".to_string(),
            name: "Arcade".to_string(),
            short_name: "Arcade".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: vec![".zip".to_string()],
        },
        SystemInfo {
            id: "neogeo".to_string(),
            name: "Neo Geo".to_string(),
            short_name: "Neo Geo".to_string(),
            manufacturer: Some("SNK".to_string()),
            release_year: Some(1990),
            extensions: vec![".zip".to_string()],
        },
        SystemInfo {
            id: "pce".to_string(),
            name: "PC Engine / TurboGrafx-16".to_string(),
            short_name: "PCE".to_string(),
            manufacturer: Some("NEC".to_string()),
            release_year: Some(1987),
            extensions: vec![".pce".to_string(), ".zip".to_string()],
        },
        SystemInfo {
            id: "atari2600".to_string(),
            name: "Atari 2600".to_string(),
            short_name: "2600".to_string(),
            manufacturer: Some("Atari".to_string()),
            release_year: Some(1977),
            extensions: vec![".a26".to_string(), ".bin".to_string(), ".zip".to_string()],
        },
    ]
}

/// 获取所有游戏系统
#[tauri::command]
pub fn get_systems() -> Result<Vec<SystemInfo>, String> {
    Ok(get_preset_systems_data())
}

/// 获取单个游戏系统
#[tauri::command]
pub fn get_system(id: String) -> Result<Option<SystemInfo>, String> {
    let systems = get_preset_systems_data();
    Ok(systems.into_iter().find(|s| s.id == id))
}
