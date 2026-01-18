//! Tools Commands
//!
//! 工具相关的 Tauri 命令

/// 更新中文 ROM 数据库
#[tauri::command]
pub async fn update_cn_repo() -> Result<(), String> {
    crate::scraper::cn_repo::update_repo()
}
