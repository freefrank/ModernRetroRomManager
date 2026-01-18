mod commands;
mod config;
mod rom_service;
mod scraper;
pub mod settings;
pub mod system_mapping;

use commands::scraper::ScraperState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(ScraperState::new())
        .setup(|app| {
            // Debug: 输出配置目录位置
            println!("[DEBUG] Config directory: {:?}", config::get_config_dir());
            println!("[DEBUG] Settings file: {:?}", config::get_settings_path());

            // 加载应用配置（如果不存在则创建默认配置）
            settings::load_settings().expect("Failed to load settings");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::system::get_systems,
            commands::system::get_system,
            commands::rom::get_roms,
            commands::rom::get_rom_stats,
            commands::directory::add_directory,
            commands::directory::get_directories,
            commands::directory::remove_directory,
            // Scraper (Updated)
            commands::scraper::get_scraper_providers,
            commands::scraper::configure_scraper_provider,
            commands::scraper::scraper_search,
            commands::scraper::scraper_get_metadata,
            commands::scraper::scraper_get_media,
            commands::scraper::scraper_auto_scrape,
            commands::scraper::scraper_set_provider_enabled,
            commands::scraper::apply_scraped_data,
            commands::scraper::batch_scrape,
            commands::scraper::save_temp_metadata,
            commands::scraper::get_temp_media_list,
            commands::scraper::delete_temp_media,
            commands::scraper::update_cn_repo,

            // Export (New location)
            commands::export::export_scraped_data,
            commands::export::export_to_emulationstation, // Placeholder
            commands::export::export_to_pegasus, // Placeholder
            
            // Naming check / CN ROM Tool
            commands::naming_check::scan_directory_for_naming_check,
            commands::naming_check::auto_fix_naming,
            commands::naming_check::set_extracted_cn_as_name,
            commands::naming_check::add_english_as_tag,
            commands::naming_check::export_cn_metadata,
            commands::naming_check::update_english_name,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// get_preset_systems 已移动到 commands/system.rs 的私有函数
// init_preset_systems 已移除
