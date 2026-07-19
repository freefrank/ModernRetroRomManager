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

/// 多碟游戏整理:物理把各碟塞进 `<基名>/` 子文件夹、外层写 `<基名>.m3u`,
/// 并把临时元数据里的各碟折叠成一条指向 m3u 的记录。保存时逐平台调用,
/// 非光盘平台/无临时元数据/无多碟组时为空操作,幂等。
#[tauri::command]
pub async fn organize_multidisc_games(
    directory: String,
    system: String,
) -> Result<crate::multidisc::OrganizeReport, String> {
    tokio::task::spawn_blocking(move || {
        let dir = std::path::Path::new(&directory);
        let system_lower = system.to_lowercase();
        if !crate::rom_service::is_disc_platform(&system_lower) {
            return Ok(crate::multidisc::OrganizeReport::default());
        }

        // 临时元数据是库视图权威来源:整理后回写它,库/导出都随之变一条。
        let library = dir.parent().unwrap_or(dir);
        let temp_dir = crate::config::get_temp_dir_for_library(library, &system);
        let Some(metadata_path) = ["metadata.pegasus.txt", "metadata.txt"]
            .into_iter()
            .map(|name| temp_dir.join(name))
            .find(|path| path.exists())
        else {
            return Ok(crate::multidisc::OrganizeReport::default());
        };

        let mut metadata = crate::scraper::pegasus::parse_pegasus_file(&metadata_path)?;
        let report =
            crate::multidisc::organize_and_collapse(dir, &mut metadata.games, true)?;
        if report.changed {
            let options = crate::scraper::pegasus::PegasusExportOptions {
                include_assets: true,
                ..Default::default()
            };
            crate::scraper::pegasus::write_pegasus_file(
                &metadata_path,
                &metadata.games,
                &options,
                false,
            )?;
        }
        Ok(report)
    })
    .await
    .map_err(|error| error.to_string())?
}
