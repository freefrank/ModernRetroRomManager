mod commands;
mod config;
mod embedded_resources;
mod ps3;
mod rating;
pub mod rom_archive;
mod rom_index;
mod rom_service;
mod scraper;
pub mod settings;
pub mod system_mapping;

#[cfg(windows)]
#[link(name = "advapi32")]
unsafe extern "system" {}

use commands::scraper::ScraperState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Debug: 输出配置目录位置
    println!("[DEBUG] Config directory: {:?}", config::get_config_dir());
    println!("[DEBUG] Settings file: {:?}", config::get_settings_path());

    if let Err(error) = embedded_resources::ensure_embedded_resources() {
        eprintln!("[ERROR] {error}");
    }

    // 加载应用配置(如果不存在则创建默认配置)。
    // 必须先于 ScraperState 初始化,否则已保存的 Scraper 凭证无法在启动时恢复。
    settings::load_settings().expect("Failed to load settings");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(ScraperState::new())
        .invoke_handler(tauri::generate_handler![
            commands::system::get_systems,
            commands::system::get_system,
            commands::system::get_system_logo_map,
            commands::system::get_system_logo_path,
            commands::rom::get_roms,
            commands::rom::get_rom_library_summary,
            commands::rom::get_library_rom_summary,
            commands::rom::get_system_roms,
            commands::rom::get_rom_stats,
            commands::rom::get_roms_for_single_directory,
            commands::rom::scan_rom_library,
            commands::rom::cancel_rom_scan,
            commands::directory::add_directory,
            commands::directory::get_directories,
            commands::directory::remove_directory,
            commands::directory::set_active_library,
            commands::directory::rename_library,
            commands::directory::get_library_folders,
            commands::directory::set_library_indexed_folders,
            commands::directory::scan_directory,
            commands::directory::detect_metadata_files,
            // Config
            commands::config::validate_path,
            commands::config::list_directory,
            commands::config::get_config_dir,
            commands::config::get_media_dir,
            commands::config::get_app_settings,
            commands::config::save_app_settings,
            commands::config::update_app_setting,
            commands::config::get_scraper_configs,
            commands::config::save_scraper_config,
            commands::storage::get_storage_stats,
            commands::storage::cleanup_incomplete_cache,
            commands::storage::clear_rebuildable_cache,
            commands::storage::scan_orphaned_library_assets,
            commands::storage::cleanup_orphaned_library_assets,
            commands::storage::open_cache_directory,
            commands::media_cache::resolve_cached_library_asset,
            // AI metadata translation
            commands::ai::get_ai_translation_config,
            commands::ai::save_ai_translation_config,
            commands::ai::translate_metadata,
            commands::ai::translate_metadata_batch,
            // Scraper (Updated)
            commands::scraper::get_scraper_providers,
            commands::scraper::configure_scraper_provider,
            commands::scraper::scraper_search,
            commands::scraper::scraper_get_metadata,
            commands::scraper::scraper_get_media,
            commands::scraper::get_scraper_media_types,
            commands::scraper::set_scraper_media_types,
            commands::scraper::scraper_auto_scrape,
            commands::scraper::scraper_set_provider_enabled,
            commands::scraper::scraper_set_provider_priority,
            commands::scraper::test_scraper_provider,
            commands::scraper::apply_scraped_data,
            commands::scraper::batch_scrape,
            commands::scraper::cancel_batch_scrape,
            commands::scraper::save_temp_metadata,
            commands::scraper::get_temp_media_list,
            commands::scraper::delete_temp_media,
            // Export (New location)
            commands::export::export_scraped_data,
            commands::export::export_library_scraped_data,
            commands::export::cancel_export,
            commands::export::export_to_emulationstation, // Placeholder
            commands::export::export_to_pegasus,          // Placeholder
            // Naming check / CN ROM Tool
            commands::naming_check::scan_directory_for_naming_check,
            commands::naming_check::get_naming_check_results,
            commands::naming_check::auto_fix_naming,
            commands::naming_check::set_extracted_cn_as_name,
            commands::naming_check::add_english_as_tag,
            commands::naming_check::export_cn_metadata,
            commands::naming_check::update_english_name,
            commands::naming_check::update_extracted_cn_name,
            // Tools
            commands::tools::update_cn_repo,
            commands::tools::organize_rom_archives,
            // PS3
            commands::ps3::generate_ps3_boxart,
            // Theme
            commands::theme::list_custom_themes,
            commands::theme::import_theme_pack,
            commands::theme::delete_custom_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// get_preset_systems 已移动到 commands/system.rs 的私有函数
// init_preset_systems 已移除
