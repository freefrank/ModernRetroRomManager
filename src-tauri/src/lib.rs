mod commands;
mod config;
mod rom_service;
mod scraper;
pub mod settings;

use commands::scraper::ScraperState;
use scraper::local_cn::LocalCnProvider;
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

            // 初始化 LocalCnProvider (注入 Resource 路径)
            let resource_path = app.path().resolve("rom-name-cn", tauri::path::BaseDirectory::Resource).ok();
            if let Some(path) = &resource_path {
                println!("[DEBUG] Resource path for rom-name-cn: {:?}", path);
            }
            
            let extra_paths = resource_path.into_iter().collect::<Vec<_>>();
            let local_cn = LocalCnProvider::new(extra_paths);
            
            // 获取 ScraperState 并注册 Provider
            let state = app.state::<ScraperState>();
            let mut manager = state.manager.blocking_write();
            manager.register(local_cn);

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
            // 新 scraper API
            commands::get_scraper_providers,
            commands::configure_scraper_provider,
            commands::scraper_search,
            commands::scraper_get_metadata,
            commands::scraper_get_media,
            commands::scraper_auto_scrape,
            commands::scraper_set_provider_enabled,
            commands::apply_scraped_data,
            commands::batch_scrape,
            commands::save_temp_metadata,
            commands::get_temp_media_list,
            commands::delete_temp_media,
            commands::update_cn_repo,
            commands::export_scraped_data,
            // Import/Export
            commands::import_gamelist,
            commands::import_pegasus,
            commands::export_to_emulationstation,
            commands::export_to_pegasus,
            // Config commands
            commands::validate_path,
            commands::get_config_dir,
            commands::get_media_dir,
            commands::detect_metadata_files,
            commands::scan_directory,
            // Settings commands
            commands::get_app_settings,
            commands::save_app_settings,
            commands::update_app_setting,
            // Scraper config commands (from settings file)
            commands::get_scraper_configs,
            commands::save_scraper_config,
            // Naming check
            commands::naming_check::scan_directory_for_naming_check,
            commands::naming_check::auto_fix_naming,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// get_preset_systems 已移动到 commands/system.rs 的私有函数
// init_preset_systems 已移除
