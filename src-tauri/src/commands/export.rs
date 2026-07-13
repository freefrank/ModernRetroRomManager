//! 将临时抓取结果导出为 Pegasus 或 EmulationStation 数据包。

use crate::config::get_temp_dir_for_library;
use crate::scraper::pegasus::{
    parse_pegasus_file, write_pegasus_file, PegasusExportOptions, PegasusGame,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize, Debug)]
pub struct ExportProgress {
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub finished: bool,
}

fn emit_progress(app: &AppHandle, current: usize, message: impl Into<String>, finished: bool) {
    let _ = app.emit(
        "export-progress",
        ExportProgress {
            current,
            total: 100,
            message: message.into(),
            finished,
        },
    );
}

fn find_temp_metadata(source_directory: &Path, system: &str) -> Result<PathBuf, String> {
    let library = source_directory.parent().unwrap_or(source_directory);
    let temp_dir = get_temp_dir_for_library(library, system);
    ["metadata.pegasus.txt", "metadata.txt"]
        .into_iter()
        .map(|name| temp_dir.join(name))
        .find(|path| path.exists())
        .ok_or_else(|| format!("没有找到 {system} 的临时抓取元数据"))
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if !directory.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

fn copy_media(source: &Path, target: &Path, app: &AppHandle) -> Result<usize, String> {
    let mut files = Vec::new();
    collect_files(source, &mut files)?;
    let total = files.len().max(1);
    for (index, source_file) in files.iter().enumerate() {
        let relative = source_file
            .strip_prefix(source)
            .map_err(|error| error.to_string())?;
        let target_file = target.join(relative);
        if let Some(parent) = target_file.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::copy(source_file, &target_file)
            .map_err(|error| format!("复制媒体 {} 失败: {error}", source_file.display()))?;
        emit_progress(
            app,
            20 + ((index + 1) * 70 / total),
            format!("导出媒体: {}/{}", index + 1, files.len()),
            false,
        );
    }
    Ok(files.len())
}

fn export_pegasus(target: &Path, system: &str, games: &[PegasusGame]) -> Result<PathBuf, String> {
    let path = target.join("metadata.pegasus.txt");
    let options = PegasusExportOptions {
        include_collection: true,
        collection_name: Some(system.to_string()),
        include_assets: true,
        ..Default::default()
    };
    write_pegasus_file(&path, games, &options, true)?;
    Ok(path)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct EsGame {
    path: String,
    name: Option<String>,
    desc: Option<String>,
    image: Option<String>,
    thumbnail: Option<String>,
    video: Option<String>,
    marquee: Option<String>,
    developer: Option<String>,
    publisher: Option<String>,
    genre: Option<String>,
    players: Option<String>,
    releasedate: Option<String>,
    rating: Option<f64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename = "gameList")]
struct EsGameList {
    #[serde(rename = "game", default)]
    games: Vec<EsGame>,
}

fn local_asset(value: &Option<String>) -> Option<String> {
    value.as_ref().map(|path| {
        let normalized = path.replace('\\', "/");
        if normalized.starts_with("./") {
            normalized
        } else {
            format!("./{normalized}")
        }
    })
}

fn pegasus_to_es(game: &PegasusGame) -> Option<EsGame> {
    let file = game.file.as_ref()?;
    Some(EsGame {
        path: if file.starts_with("./") {
            file.clone()
        } else {
            format!("./{file}")
        },
        name: Some(game.name.clone()),
        desc: game.description.clone().or_else(|| game.summary.clone()),
        image: local_asset(&game.box_front).or_else(|| local_asset(&game.screenshot)),
        thumbnail: local_asset(&game.screenshot),
        video: local_asset(&game.video),
        marquee: local_asset(&game.logo).or_else(|| local_asset(&game.marquee)),
        developer: game.developer.clone(),
        publisher: game.publisher.clone(),
        genre: game.genre.clone(),
        players: game.players.clone(),
        releasedate: game.release.clone(),
        rating: game.rating.as_ref().and_then(|rating| {
            let value = rating.trim_end_matches('%').parse::<f64>().ok()?;
            Some(if rating.contains('%') {
                value / 100.0
            } else {
                value
            })
        }),
    })
}

fn export_emulationstation(target: &Path, games: &[PegasusGame]) -> Result<PathBuf, String> {
    let path = target.join("gamelist.xml");
    let mut list = if path.exists() {
        quick_xml::de::from_str::<EsGameList>(
            &fs::read_to_string(&path).map_err(|error| error.to_string())?,
        )
        .map_err(|error| format!("解析 gamelist.xml 失败: {error}"))?
    } else {
        EsGameList::default()
    };

    for game in games.iter().filter_map(pegasus_to_es) {
        if let Some(existing) = list.games.iter_mut().find(|entry| entry.path == game.path) {
            *existing = game;
        } else {
            list.games.push(game);
        }
    }
    let xml = quick_xml::se::to_string(&list).map_err(|error| error.to_string())?;
    fs::write(
        &path,
        format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{xml}"),
    )
    .map_err(|error| error.to_string())?;
    Ok(path)
}

/// 导出指定系统的全部临时 metadata 和相关媒体资产。
#[tauri::command]
pub async fn export_scraped_data(
    app: AppHandle,
    system: String,
    directory: String,
    format: Option<String>,
    target_directory: Option<String>,
) -> Result<(), String> {
    let source_directory = PathBuf::from(&directory);
    let metadata_path = find_temp_metadata(&source_directory, &system)?;
    let temp_directory = metadata_path
        .parent()
        .ok_or_else(|| "临时元数据目录无效".to_string())?;
    let target = PathBuf::from(target_directory.as_deref().unwrap_or(&directory));
    fs::create_dir_all(&target).map_err(|error| error.to_string())?;

    emit_progress(&app, 0, "读取临时抓取数据...", false);
    let metadata = parse_pegasus_file(&metadata_path)?;
    if metadata.games.is_empty() {
        return Err("临时元数据中没有可导出的游戏".to_string());
    }

    let format = format
        .unwrap_or_else(|| "auto".to_string())
        .to_ascii_lowercase();
    let resolved_format = if format == "auto" {
        if target.join("gamelist.xml").exists() {
            "emulationstation"
        } else {
            "pegasus"
        }
    } else {
        format.as_str()
    };
    emit_progress(&app, 10, "写入元数据...", false);
    let output = match resolved_format {
        "emulationstation" | "es" | "gamelist" => {
            export_emulationstation(&target, &metadata.games)?
        }
        "pegasus" => export_pegasus(&target, &system, &metadata.games)?,
        other => return Err(format!("不支持的导出格式: {other}")),
    };

    let media_count = copy_media(&temp_directory.join("media"), &target.join("media"), &app)?;
    emit_progress(
        &app,
        100,
        format!(
            "导出完成: {} 个游戏、{} 个媒体文件 -> {}",
            metadata.games.len(),
            media_count,
            output.display()
        ),
        true,
    );
    Ok(())
}

#[tauri::command]
pub async fn export_to_emulationstation(app: AppHandle, target_dir: String) -> Result<(), String> {
    emit_progress(
        &app,
        100,
        format!("请从导出页面选择源系统: {target_dir}"),
        true,
    );
    Err("请使用 export_scraped_data 并指定源系统".to_string())
}

#[tauri::command]
pub async fn export_to_pegasus(app: AppHandle, target_dir: String) -> Result<(), String> {
    emit_progress(
        &app,
        100,
        format!("请从导出页面选择源系统: {target_dir}"),
        true,
    );
    Err("请使用 export_scraped_data 并指定源系统".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> PegasusGame {
        PegasusGame {
            name: "Test Game".into(),
            file: Some("game.gba".into()),
            box_front: Some("media/game/boxfront.png".into()),
            screenshot: Some("media/game/screenshot.png".into()),
            logo: Some("media/game/logo.png".into()),
            ..Default::default()
        }
    }

    #[test]
    fn emulationstation_export_contains_metadata_and_assets() {
        let root = std::env::temp_dir().join(format!("mrrm-export-es-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = export_emulationstation(&root, &[game()]).unwrap();
        let xml = fs::read_to_string(path).unwrap();
        assert!(xml.contains("<path>./game.gba</path>"));
        assert!(xml.contains("<image>./media/game/boxfront.png</image>"));
        assert!(xml.contains("<thumbnail>./media/game/screenshot.png</thumbnail>"));
        assert!(xml.contains("<marquee>./media/game/logo.png</marquee>"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn pegasus_export_contains_asset_paths() {
        let root = std::env::temp_dir().join(format!("mrrm-export-pg-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = export_pegasus(&root, "GBA", &[game()]).unwrap();
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("assets.boxFront: media/game/boxfront.png"));
        fs::remove_dir_all(root).unwrap();
    }
}
