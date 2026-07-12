//! Tools Commands
//!
//! 工具相关的 Tauri 命令

/// 更新中文 ROM 数据库
#[tauri::command]
pub async fn update_cn_repo() -> Result<(), String> {
    crate::scraper::cn_repo::update_repo()
}

#[tauri::command]
pub async fn organize_rom_archives(
    directory: String,
    system: String,
    password: String,
) -> Result<crate::rom_archive::ArchiveOrganizeResult, String> {
    tokio::task::spawn_blocking(move || {
        crate::rom_archive::organize_rom_archives(
            std::path::Path::new(&directory),
            &system,
            &password,
        )
    })
    .await
    .map_err(|error| error.to_string())?
}
