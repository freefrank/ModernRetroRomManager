mod commands;
mod db;
mod scanner;
mod scraper;

use tauri::Manager;
use tauri_plugin_fs::FsExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 初始化数据库
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            db::init_db(&app_data_dir).expect("Failed to initialize database");

            // 初始化预置系统数据
            init_preset_systems().expect("Failed to initialize preset systems");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_systems,
            commands::get_system,
            commands::get_roms,
            commands::get_rom,
            commands::get_rom_stats,
            commands::add_scan_directory,
            commands::get_scan_directories,
            commands::remove_scan_directory,
            commands::start_scan,
            commands::get_api_configs,
            commands::save_api_config,
            commands::search_game,
            commands::get_scraper_game_details,
            commands::get_scraper_game_media,
            commands::apply_scraped_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 初始化预置游戏系统数据
fn init_preset_systems() -> Result<(), Box<dyn std::error::Error>> {
    use db::{get_connection, schema::systems};
    use diesel::prelude::*;

    let preset_systems = get_preset_systems();

    let mut conn = get_connection()?;

    for system in preset_systems {
        // 检查是否已存在
        let exists: bool = diesel::select(diesel::dsl::exists(
            systems::table.filter(systems::id.eq(&system.id)),
        ))
        .get_result(&mut conn)?;

        if !exists {
            diesel::insert_into(systems::table)
                .values(&system)
                .execute(&mut conn)?;
        }
    }

    Ok(())
}

/// 获取预置游戏系统列表
fn get_preset_systems() -> Vec<db::models::System> {
    vec![
        db::models::System {
            id: "nes".to_string(),
            name: "Nintendo Entertainment System".to_string(),
            short_name: "NES".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1983),
            extensions: r#"[".nes", ".zip", ".7z"]"#.to_string(),
            igdb_platform_id: Some(18),
            thegamesdb_platform_id: Some(7),
            screenscraper_id: Some(3),
        },
        db::models::System {
            id: "snes".to_string(),
            name: "Super Nintendo".to_string(),
            short_name: "SNES".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1990),
            extensions: r#"[".sfc", ".smc", ".zip", ".7z"]"#.to_string(),
            igdb_platform_id: Some(19),
            thegamesdb_platform_id: Some(6),
            screenscraper_id: Some(4),
        },
        db::models::System {
            id: "n64".to_string(),
            name: "Nintendo 64".to_string(),
            short_name: "N64".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1996),
            extensions: r#"[".n64", ".z64", ".v64", ".zip"]"#.to_string(),
            igdb_platform_id: Some(4),
            thegamesdb_platform_id: Some(3),
            screenscraper_id: Some(14),
        },
        db::models::System {
            id: "gb".to_string(),
            name: "Game Boy".to_string(),
            short_name: "GB".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1989),
            extensions: r#"[".gb", ".zip"]"#.to_string(),
            igdb_platform_id: Some(33),
            thegamesdb_platform_id: Some(4),
            screenscraper_id: Some(9),
        },
        db::models::System {
            id: "gbc".to_string(),
            name: "Game Boy Color".to_string(),
            short_name: "GBC".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(1998),
            extensions: r#"[".gbc", ".zip"]"#.to_string(),
            igdb_platform_id: Some(22),
            thegamesdb_platform_id: Some(41),
            screenscraper_id: Some(10),
        },
        db::models::System {
            id: "gba".to_string(),
            name: "Game Boy Advance".to_string(),
            short_name: "GBA".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(2001),
            extensions: r#"[".gba", ".zip"]"#.to_string(),
            igdb_platform_id: Some(24),
            thegamesdb_platform_id: Some(5),
            screenscraper_id: Some(12),
        },
        db::models::System {
            id: "nds".to_string(),
            name: "Nintendo DS".to_string(),
            short_name: "NDS".to_string(),
            manufacturer: Some("Nintendo".to_string()),
            release_year: Some(2004),
            extensions: r#"[".nds", ".zip"]"#.to_string(),
            igdb_platform_id: Some(20),
            thegamesdb_platform_id: Some(8),
            screenscraper_id: Some(15),
        },
        db::models::System {
            id: "genesis".to_string(),
            name: "Sega Genesis / Mega Drive".to_string(),
            short_name: "Genesis".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1988),
            extensions: r#"[".md", ".bin", ".gen", ".zip"]"#.to_string(),
            igdb_platform_id: Some(29),
            thegamesdb_platform_id: Some(18),
            screenscraper_id: Some(1),
        },
        db::models::System {
            id: "saturn".to_string(),
            name: "Sega Saturn".to_string(),
            short_name: "Saturn".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1994),
            extensions: r#"[".iso", ".cue", ".bin"]"#.to_string(),
            igdb_platform_id: Some(32),
            thegamesdb_platform_id: Some(17),
            screenscraper_id: Some(5),
        },
        db::models::System {
            id: "dreamcast".to_string(),
            name: "Sega Dreamcast".to_string(),
            short_name: "DC".to_string(),
            manufacturer: Some("Sega".to_string()),
            release_year: Some(1998),
            extensions: r#"[".cdi", ".gdi", ".iso"]"#.to_string(),
            igdb_platform_id: Some(23),
            thegamesdb_platform_id: Some(16),
            screenscraper_id: Some(40),
        },
        db::models::System {
            id: "psx".to_string(),
            name: "PlayStation".to_string(),
            short_name: "PSX".to_string(),
            manufacturer: Some("Sony".to_string()),
            release_year: Some(1994),
            extensions: r#"[".iso", ".bin", ".cue", ".pbp"]"#.to_string(),
            igdb_platform_id: Some(7),
            thegamesdb_platform_id: Some(10),
            screenscraper_id: Some(57),
        },
        db::models::System {
            id: "ps2".to_string(),
            name: "PlayStation 2".to_string(),
            short_name: "PS2".to_string(),
            manufacturer: Some("Sony".to_string()),
            release_year: Some(2000),
            extensions: r#"[".iso", ".bin"]"#.to_string(),
            igdb_platform_id: Some(8),
            thegamesdb_platform_id: Some(11),
            screenscraper_id: Some(58),
        },
        db::models::System {
            id: "psp".to_string(),
            name: "PlayStation Portable".to_string(),
            short_name: "PSP".to_string(),
            manufacturer: Some("Sony".to_string()),
            release_year: Some(2004),
            extensions: r#"[".iso", ".cso"]"#.to_string(),
            igdb_platform_id: Some(38),
            thegamesdb_platform_id: Some(13),
            screenscraper_id: Some(61),
        },
        db::models::System {
            id: "arcade".to_string(),
            name: "Arcade".to_string(),
            short_name: "Arcade".to_string(),
            manufacturer: None,
            release_year: None,
            extensions: r#"[".zip"]"#.to_string(),
            igdb_platform_id: Some(52),
            thegamesdb_platform_id: Some(23),
            screenscraper_id: Some(75),
        },
        db::models::System {
            id: "neogeo".to_string(),
            name: "Neo Geo".to_string(),
            short_name: "Neo Geo".to_string(),
            manufacturer: Some("SNK".to_string()),
            release_year: Some(1990),
            extensions: r#"[".zip"]"#.to_string(),
            igdb_platform_id: Some(80),
            thegamesdb_platform_id: Some(24),
            screenscraper_id: Some(142),
        },
        db::models::System {
            id: "pce".to_string(),
            name: "PC Engine / TurboGrafx-16".to_string(),
            short_name: "PCE".to_string(),
            manufacturer: Some("NEC".to_string()),
            release_year: Some(1987),
            extensions: r#"[".pce", ".zip"]"#.to_string(),
            igdb_platform_id: Some(86),
            thegamesdb_platform_id: Some(34),
            screenscraper_id: Some(31),
        },
        db::models::System {
            id: "atari2600".to_string(),
            name: "Atari 2600".to_string(),
            short_name: "2600".to_string(),
            manufacturer: Some("Atari".to_string()),
            release_year: Some(1977),
            extensions: r#"[".a26", ".bin", ".zip"]"#.to_string(),
            igdb_platform_id: Some(59),
            thegamesdb_platform_id: Some(22),
            screenscraper_id: Some(26),
        },
    ]
}
