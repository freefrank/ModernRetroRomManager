mod commands;
mod config;
mod rom_service;
mod scraper;
pub mod settings;

use tauri::Manager;
use tauri_plugin_fs::FsExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| {
            // Debug: 输出配置目录位置
            println!("[DEBUG] Config directory: {:?}", config::get_config_dir());
            println!("[DEBUG] Settings file: {:?}", config::get_settings_path());
            
            // 加载应用配置（如果不存在则创建默认配置）
            settings::load_settings().expect("Failed to load settings");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_systems,
            commands::get_system,
            commands::get_roms,
            commands::get_rom_stats,
            commands::add_directory,
            commands::get_directories,
            commands::remove_directory,
            commands::get_api_configs,
            commands::save_api_config,
            commands::search_game,
            commands::get_scraper_game_details,
            commands::get_scraper_game_media,
            commands::apply_scraped_data,
            commands::batch_scrape,
            commands::import_gamelist,
            commands::import_pegasus,
            commands::export_to_emulationstation,
            commands::export_to_pegasus,
            // Config commands
            commands::validate_path,
            commands::get_config_dir,
            commands::get_media_dir,
            // Settings commands
            commands::get_app_settings,
            commands::save_app_settings,
            commands::update_app_setting,
            // Scraper config commands (from settings file)
            commands::get_scraper_configs,
            commands::save_scraper_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// get_preset_systems 已移动到 commands/system.rs 的私有函数
// init_preset_systems 已移除
