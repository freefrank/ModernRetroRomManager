//! 将临时抓取结果导出为 Pegasus 或 EmulationStation 数据包。

use crate::config::get_temp_dir_for_library;
use crate::rom_index::{load_cached_roms_for_library, scan_library_by_id, ScanMode};
use crate::scraper::pegasus::{
    parse_pegasus_file, write_pegasus_file, PegasusExportOptions, PegasusGame,
};
use crate::settings::{get_settings, DirectoryConfig};
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

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn normalize_optional_path(value: &mut Option<String>) {
    if let Some(path) = value {
        *path = path.replace('\\', "/");
    }
}

fn normalize_game_paths(game: &mut PegasusGame) {
    normalize_optional_path(&mut game.file);
    for file in &mut game.files {
        *file = file.replace('\\', "/");
    }
    normalize_optional_path(&mut game.box_front);
    normalize_optional_path(&mut game.box_back);
    normalize_optional_path(&mut game.box_spine);
    normalize_optional_path(&mut game.box_full);
    normalize_optional_path(&mut game.cartridge);
    normalize_optional_path(&mut game.logo);
    normalize_optional_path(&mut game.marquee);
    normalize_optional_path(&mut game.bezel);
    normalize_optional_path(&mut game.gridicon);
    normalize_optional_path(&mut game.flyer);
    normalize_optional_path(&mut game.background);
    normalize_optional_path(&mut game.music);
    normalize_optional_path(&mut game.screenshot);
    normalize_optional_path(&mut game.titlescreen);
    normalize_optional_path(&mut game.video);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExportNameMode {
    Original,
    Chinese,
}

impl ExportNameMode {
    fn parse(value: Option<&str>) -> Result<Self, String> {
        match value.unwrap_or("original").to_ascii_lowercase().as_str() {
            "original" => Ok(Self::Original),
            "chinese" => Ok(Self::Chinese),
            other => Err(format!("不支持的导出名称模式: {other}")),
        }
    }
}

fn prepare_export_names(games: &mut [PegasusGame], mode: ExportNameMode) {
    for game in games {
        let explicit_chinese = game
            .chinese_name
            .clone()
            .or_else(|| game.extra.get("x-mrrm-cn").cloned());
        let legacy_chinese = explicit_chinese
            .is_none()
            .then(|| game.extra.get("x-mrrm-eng").map(|_| game.name.clone()))
            .flatten();
        let chinese = explicit_chinese.or(legacy_chinese);
        let original = if game.chinese_name.is_none()
            && game.extra.get("x-mrrm-cn").is_none()
            && chinese.is_some()
        {
            game.extra
                .get("x-mrrm-eng")
                .cloned()
                .unwrap_or_else(|| game.name.clone())
        } else {
            game.name.clone()
        };
        if let Some(chinese) = chinese {
            game.chinese_name = Some(chinese.clone());
            if mode == ExportNameMode::Chinese {
                game.name = chinese;
                continue;
            }
        }
        game.name = original;
    }
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

fn copy_media(
    source: &Path,
    target: &Path,
    progress: &dyn Fn(usize, String),
) -> Result<usize, String> {
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
        progress(
            20 + ((index + 1) * 70 / total),
            format!("导出媒体: {}/{}", index + 1, files.len()),
        );
    }
    Ok(files.len())
}

fn export_pegasus(
    target: &Path,
    system: &str,
    games: &[PegasusGame],
    name_mode: ExportNameMode,
) -> Result<PathBuf, String> {
    let path = target.join("metadata.pegasus.txt");
    let mut normalized_games = games.to_vec();
    for game in &mut normalized_games {
        normalize_game_paths(game);
    }
    prepare_export_names(&mut normalized_games, name_mode);
    let options = PegasusExportOptions {
        include_collection: true,
        collection_name: Some(system.to_string()),
        include_assets: true,
        ..Default::default()
    };
    write_pegasus_file(&path, &normalized_games, &options, true)?;
    Ok(path)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct EsGame {
    path: String,
    name: Option<String>,
    #[serde(rename = "chinese-name", skip_serializing_if = "Option::is_none")]
    chinese_name: Option<String>,
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
    let file = game.file.as_ref()?.replace('\\', "/");
    Some(EsGame {
        path: if file.starts_with("./") {
            file
        } else {
            format!("./{file}")
        },
        name: Some(game.name.clone()),
        chinese_name: game
            .chinese_name
            .clone()
            .or_else(|| game.extra.get("x-mrrm-cn").cloned()),
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

fn export_emulationstation(
    target: &Path,
    games: &[PegasusGame],
    name_mode: ExportNameMode,
) -> Result<PathBuf, String> {
    let path = target.join("gamelist.xml");
    let mut list = if path.exists() {
        quick_xml::de::from_str::<EsGameList>(
            &fs::read_to_string(&path).map_err(|error| error.to_string())?,
        )
        .map_err(|error| format!("解析 gamelist.xml 失败: {error}"))?
    } else {
        EsGameList::default()
    };

    let mut named_games = games.to_vec();
    prepare_export_names(&mut named_games, name_mode);
    for game in named_games.iter().filter_map(pegasus_to_es) {
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

struct ExportOutcome {
    games: usize,
    media: usize,
    output: PathBuf,
}

fn export_system_data(
    system: String,
    directory: String,
    format: Option<String>,
    target_directory: Option<String>,
    name_mode: Option<String>,
    progress: &dyn Fn(usize, String),
) -> Result<ExportOutcome, String> {
    let source_directory = PathBuf::from(&directory);
    let metadata_path = find_temp_metadata(&source_directory, &system)?;
    let temp_directory = metadata_path
        .parent()
        .ok_or_else(|| "临时元数据目录无效".to_string())?;
    let target = PathBuf::from(target_directory.as_deref().unwrap_or(&directory));
    fs::create_dir_all(&target).map_err(|error| error.to_string())?;

    progress(0, "读取临时抓取数据...".to_string());
    let mut metadata = parse_pegasus_file(&metadata_path)?;
    if metadata.games.is_empty() {
        return Err("临时元数据中没有可导出的游戏".to_string());
    }
    for game in &mut metadata.games {
        normalize_game_paths(game);
    }
    let name_mode = ExportNameMode::parse(name_mode.as_deref())?;

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
    progress(10, "写入元数据...".to_string());
    let output = match resolved_format {
        "emulationstation" | "es" | "gamelist" => {
            export_emulationstation(&target, &metadata.games, name_mode)?
        }
        "pegasus" => export_pegasus(&target, &system, &metadata.games, name_mode)?,
        other => return Err(format!("不支持的导出格式: {other}")),
    };

    let media_count = copy_media(
        &temp_directory.join("media"),
        &target.join("media"),
        progress,
    )?;
    Ok(ExportOutcome {
        games: metadata.games.len(),
        media: media_count,
        output,
    })
}

/// 导出指定系统的全部临时 metadata 和相关媒体资产。
#[tauri::command]
pub async fn export_scraped_data(
    app: AppHandle,
    system: String,
    directory: String,
    format: Option<String>,
    target_directory: Option<String>,
    name_mode: Option<String>,
) -> Result<(), String> {
    let outcome = export_system_data(
        system,
        directory,
        format,
        target_directory,
        name_mode,
        &|current, message| {
            emit_progress(&app, current, message, false);
        },
    )?;
    emit_progress(
        &app,
        100,
        format!(
            "导出完成: {} 个游戏、{} 个媒体文件 -> {}",
            outcome.games,
            outcome.media,
            display_path(&outcome.output)
        ),
        true,
    );
    Ok(())
}

fn library_system_target(
    library: &DirectoryConfig,
    source_directory: &Path,
    target_root: Option<&str>,
) -> PathBuf {
    let Some(target_root) = target_root.filter(|value| !value.trim().is_empty()) else {
        return source_directory.to_path_buf();
    };
    let target_root = PathBuf::from(target_root);
    let library_root = Path::new(&library.path);
    match source_directory.strip_prefix(library_root) {
        Ok(relative) if !relative.as_os_str().is_empty() => target_root.join(relative),
        Ok(_) => target_root,
        Err(_) => source_directory
            .file_name()
            .map(|name| target_root.join(name))
            .unwrap_or(target_root),
    }
}

/// 导出指定 Library 下所有已抓取的平台。
#[tauri::command]
pub async fn export_library_scraped_data(
    app: AppHandle,
    library_id: String,
    format: Option<String>,
    target_directory: Option<String>,
    name_mode: Option<String>,
) -> Result<(), String> {
    let library = get_settings()
        .directories
        .into_iter()
        .find(|library| library.id == library_id)
        .ok_or_else(|| format!("Library 不存在: {library_id}"))?;
    let scan_id = library.id.clone();
    let systems = if let Some(cached) = load_cached_roms_for_library(&library.id) {
        cached
    } else {
        tokio::task::spawn_blocking(move || scan_library_by_id(&scan_id, ScanMode::Full, |_| {}))
            .await
            .map_err(|error| format!("ROM 扫描任务失败: {error}"))??
    };

    let exportable: Vec<_> = systems
        .into_iter()
        .filter(|system| find_temp_metadata(Path::new(&system.path), &system.system).is_ok())
        .collect();
    if exportable.is_empty() {
        return Err(format!("Library“{}”中没有可导出的抓取数据", library.name));
    }

    let total_systems = exportable.len();
    let mut total_games = 0;
    let mut total_media = 0;
    for (index, system) in exportable.into_iter().enumerate() {
        let source = PathBuf::from(&system.path);
        let target = library_system_target(&library, &source, target_directory.as_deref());
        let base = index * 100 / total_systems;
        let next = (index + 1) * 100 / total_systems;
        let span = next.saturating_sub(base);
        let system_name = system.system.clone();
        let outcome = export_system_data(
            system.system,
            system.path,
            format.clone(),
            Some(display_path(&target)),
            name_mode.clone(),
            &|current, message| {
                emit_progress(
                    &app,
                    base + current * span / 100,
                    format!("[{system_name}] {message}"),
                    false,
                );
            },
        )?;
        total_games += outcome.games;
        total_media += outcome.media;
    }

    let destination = target_directory
        .as_deref()
        .map(|path| display_path(Path::new(path)))
        .unwrap_or_else(|| "各 ROM 平台目录".to_string());
    emit_progress(
        &app,
        100,
        format!(
            "Library 导出完成: {total_systems} 个平台、{total_games} 个游戏、{total_media} 个媒体文件 -> {destination}"
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
        let path = export_emulationstation(&root, &[game()], ExportNameMode::Original).unwrap();
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
        let path = export_pegasus(&root, "GBA", &[game()], ExportNameMode::Original).unwrap();
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("assets.boxFront: media/game/boxfront.png"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn exported_metadata_uses_forward_slashes() {
        let root = std::env::temp_dir().join(format!("mrrm-export-paths-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let mut mixed = game();
        mixed.file = Some("folder\\game.gba".into());
        mixed.box_front = Some("media\\game\\boxfront.png".into());

        let pegasus =
            export_pegasus(&root, "GBA", &[mixed.clone()], ExportNameMode::Original).unwrap();
        let pegasus_content = fs::read_to_string(pegasus).unwrap();
        assert!(pegasus_content.contains("file: folder/game.gba"));
        assert!(pegasus_content.contains("assets.boxFront: media/game/boxfront.png"));

        let es = export_emulationstation(&root, &[mixed], ExportNameMode::Original).unwrap();
        let es_content = fs::read_to_string(es).unwrap();
        assert!(es_content.contains("<path>./folder/game.gba</path>"));
        assert!(es_content.contains("<image>./media/game/boxfront.png</image>"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn export_name_mode_selects_original_or_chinese_name() {
        let root = std::env::temp_dir().join(format!("mrrm-export-names-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let mut named = game();
        named.name = "Original Game".into();
        named.extra.insert("x-mrrm-cn".into(), "中文游戏".into());

        let original =
            export_pegasus(&root, "GBA", &[named.clone()], ExportNameMode::Original).unwrap();
        let content = fs::read_to_string(original).unwrap();
        assert!(content.contains("game: Original Game"));
        assert!(content.contains("x-mrrm-cn: 中文游戏"));

        let chinese = export_emulationstation(&root, &[named], ExportNameMode::Chinese).unwrap();
        let content = fs::read_to_string(chinese).unwrap();
        assert!(content.contains("<name>中文游戏</name>"));
        assert!(content.contains("<chinese-name>中文游戏</chinese-name>"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn whole_library_target_preserves_platform_subdirectory() {
        let library = DirectoryConfig {
            id: "library-test".into(),
            name: "Test".into(),
            path: "D:/Retro/Roms".into(),
            is_root_directory: true,
            metadata_format: "auto".into(),
            system_id: None,
        };
        assert_eq!(
            library_system_target(&library, Path::new("D:/Retro/Roms/DC"), Some("E:/Export")),
            PathBuf::from("E:/Export/DC")
        );
        assert_eq!(
            library_system_target(&library, Path::new("D:/Retro/Roms"), Some("E:/Export")),
            PathBuf::from("E:/Export")
        );
    }
}
