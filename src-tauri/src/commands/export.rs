//! 将临时抓取结果导出为 Pegasus 或 EmulationStation 数据包。

use crate::config::get_temp_dir_for_library;
use crate::rom_index::{load_cached_roms_for_library, scan_library_by_id, ScanMode};
use crate::scraper::pegasus::{
    parse_pegasus_file, write_pegasus_file, PegasusExportOptions, PegasusGame,
};
use crate::settings::{get_settings, DirectoryConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

static EXPORT_RUNNING: AtomicBool = AtomicBool::new(false);
static EXPORT_CANCELLED: AtomicBool = AtomicBool::new(false);
const EXPORT_CANCELLED_ERROR: &str = "__MRRM_EXPORT_CANCELLED__";

fn check_export_cancelled() -> Result<(), String> {
    if EXPORT_CANCELLED.load(Ordering::Relaxed) {
        Err(EXPORT_CANCELLED_ERROR.to_string())
    } else {
        Ok(())
    }
}

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

fn paths_equal(left: &Path, right: &Path) -> bool {
    display_path(left)
        .trim_end_matches('/')
        .eq_ignore_ascii_case(display_path(right).trim_end_matches('/'))
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CopyMode {
    All,
    RomAssetsOnly,
}

fn is_rom_or_asset_file(path: &Path) -> bool {
    const EXTENSIONS: &[&str] = &[
        // Archives and common ROM/disc formats.
        "zip", "7z", "rar", "nes", "fds", "unf", "unif", "sfc", "smc", "fig", "gb", "gbc", "gba",
        "agb", "nds", "dsi", "3ds", "cia", "n64", "z64", "v64", "md", "gen", "smd", "32x", "sms",
        "gg", "ws", "wsc", "pce", "sgx", "cue", "bin", "img", "iso", "chd", "cso", "pbp", "gdi",
        "cdi", "ccd", "sub", "mds", "mdf", "rvz", "gcm", "wbfs", "wad", "xci", "nsp", "pkg", "p8",
        "lnx", "a26", "a52", "a78", "col", "vec", "ngc", "ngp", "mgw", "min", "m3u", "rom", "self",
        "sprx", // Artwork, video, and audio assets.
        "png", "jpg", "jpeg", "webp", "gif", "bmp", "svg", "avif", "mp4", "webm", "mkv", "avi",
        "mov", "mp3", "ogg", "opus", "flac", "wav", "m4a", "aac",
    ];
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str()))
}

fn directory_copy_stats(
    source: &Path,
    mode: CopyMode,
    inside_folder_rom: bool,
) -> Result<(usize, u64), String> {
    check_export_cancelled()?;
    let mut files = 0;
    let mut bytes = 0_u64;
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        check_export_cancelled()?;
        let entry = entry.map_err(|error| error.to_string())?;
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_dir() {
            let folder_rom = inside_folder_rom || entry.path().join("PS3_GAME").is_dir();
            let (child_files, child_bytes) = directory_copy_stats(&entry.path(), mode, folder_rom)?;
            files += child_files;
            bytes = bytes.saturating_add(child_bytes);
        } else if file_type.is_file()
            && (mode == CopyMode::All || inside_folder_rom || is_rom_or_asset_file(&entry.path()))
        {
            files += 1;
            bytes =
                bytes.saturating_add(entry.metadata().map_err(|error| error.to_string())?.len());
        }
    }
    Ok((files, bytes))
}

fn copy_directory_recursive(
    source: &Path,
    target: &Path,
    mode: CopyMode,
    inside_folder_rom: bool,
    on_progress: &mut dyn FnMut(u64, u64, bool, bool, &Path),
) -> Result<usize, String> {
    check_export_cancelled()?;
    let mut copied = 0;
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        check_export_cancelled()?;
        let entry = entry.map_err(|error| error.to_string())?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_dir() {
            let folder_rom = inside_folder_rom || source_path.join("PS3_GAME").is_dir();
            copied += copy_directory_recursive(
                &source_path,
                &target_path,
                mode,
                folder_rom,
                on_progress,
            )?;
        } else if file_type.is_file()
            && (mode == CopyMode::All || inside_folder_rom || is_rom_or_asset_file(&source_path))
        {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            let source_size = entry.metadata().map_err(|error| error.to_string())?.len();
            if target_path
                .metadata()
                .is_ok_and(|metadata| metadata.is_file() && metadata.len() == source_size)
            {
                copied += 1;
                on_progress(source_size, 0, true, true, &source_path);
                continue;
            }
            let part_name = format!(
                "{}.mrrm-part",
                target_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("export")
            );
            let part_path = target_path.with_file_name(part_name);
            let result = (|| -> Result<(), String> {
                let mut input = fs::File::open(&source_path).map_err(|error| {
                    format!("读取 {} 失败: {error}", display_path(&source_path))
                })?;
                let mut output = fs::File::create(&part_path)
                    .map_err(|error| format!("创建 {} 失败: {error}", display_path(&part_path)))?;
                let mut buffer = vec![0_u8; 1024 * 1024];
                loop {
                    check_export_cancelled()?;
                    let bytes = input.read(&mut buffer).map_err(|error| error.to_string())?;
                    if bytes == 0 {
                        break;
                    }
                    output
                        .write_all(&buffer[..bytes])
                        .map_err(|error| error.to_string())?;
                    on_progress(bytes as u64, bytes as u64, false, false, &source_path);
                }
                output.flush().map_err(|error| error.to_string())?;
                if target_path.exists() {
                    fs::remove_file(&target_path).map_err(|error| error.to_string())?;
                }
                fs::rename(&part_path, &target_path).map_err(|error| error.to_string())?;
                Ok(())
            })();
            if let Err(error) = result {
                let _ = fs::remove_file(&part_path);
                return Err(if error == EXPORT_CANCELLED_ERROR {
                    error
                } else {
                    format!("复制 {} 失败: {error}", display_path(&source_path))
                });
            }
            copied += 1;
            on_progress(0, 0, true, false, &source_path);
        }
    }
    Ok(copied)
}

fn copy_directory_contents_with_progress(
    source: &Path,
    target: &Path,
    mode: CopyMode,
    progress: &dyn Fn(usize, String),
) -> Result<usize, String> {
    if paths_equal(source, target) {
        return Ok(0);
    }
    let source_text = display_path(source)
        .trim_end_matches('/')
        .to_ascii_lowercase();
    let target_text = display_path(target)
        .trim_end_matches('/')
        .to_ascii_lowercase();
    if target_text.starts_with(&format!("{source_text}/")) {
        return Err("导出目录不能位于源 ROM 目录内部".to_string());
    }
    progress(0, "正在统计待导出文件...".to_string());
    let root_is_folder_rom = source.join("PS3_GAME").is_dir();
    let (total_files, total_bytes) = directory_copy_stats(source, mode, root_is_folder_rom)?;
    if total_files == 0 {
        return Ok(0);
    }
    let started = Instant::now();
    let mut last_emit = started;
    let mut copied_files = 0;
    let mut processed_bytes = 0_u64;
    let mut written_bytes = 0_u64;
    let mut skipped_files = 0;
    copy_directory_recursive(
        source,
        target,
        mode,
        root_is_folder_rom,
        &mut |processed, written, file_finished, skipped, source_path| {
            processed_bytes = processed_bytes.saturating_add(processed);
            written_bytes = written_bytes.saturating_add(written);
            if file_finished {
                copied_files += 1;
                if skipped {
                    skipped_files += 1;
                }
            }
            let now = Instant::now();
            if now.duration_since(last_emit) >= Duration::from_millis(100)
                || copied_files == total_files
            {
                let elapsed = now.duration_since(started).as_secs_f64().max(0.001);
                let speed = (written_bytes as f64 / elapsed) as u64;
                let percent = if total_bytes == 0 {
                    copied_files * 70 / total_files
                } else {
                    (processed_bytes.saturating_mul(70) / total_bytes) as usize
                };
                progress(
                percent,
                format!(
                    "处理文件 {copied_files}/{total_files}（跳过 {skipped_files}）· {} · {}/{} · 写入 {}/s",
                    source_path.file_name().unwrap_or_default().to_string_lossy(),
                    format_bytes(processed_bytes),
                    format_bytes(total_bytes),
                    format_bytes(speed)
                ),
            );
                last_emit = now;
            }
        },
    )
}

#[cfg(test)]
fn copy_directory_contents(source: &Path, target: &Path) -> Result<usize, String> {
    copy_directory_contents_with_progress(source, target, CopyMode::All, &|_, _| {})
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
        check_export_cancelled()?;
        let relative = source_file
            .strip_prefix(source)
            .map_err(|error| error.to_string())?;
        let target_file = target.join(relative);
        if let Some(parent) = target_file.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let source_size = source_file
            .metadata()
            .map_err(|error| error.to_string())?
            .len();
        let unchanged = target_file
            .metadata()
            .is_ok_and(|metadata| metadata.is_file() && metadata.len() == source_size);
        if !unchanged {
            fs::copy(source_file, &target_file)
                .map_err(|error| format!("复制媒体 {} 失败: {error}", source_file.display()))?;
        }
        progress(
            20 + ((index + 1) * 70 / total),
            format!(
                "导出媒体: {}/{}{}",
                index + 1,
                files.len(),
                if unchanged {
                    "（已跳过同大小文件）"
                } else {
                    ""
                }
            ),
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
    #[serde(default, deserialize_with = "crate::rating::deserialize_optional_f64")]
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
        rating: game.rating.as_deref().and_then(crate::rating::parse_rating),
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

fn export_metadata_formats(
    target: &Path,
    system: &str,
    games: &[PegasusGame],
    name_mode: ExportNameMode,
    format: &str,
) -> Result<PathBuf, String> {
    match format {
        "emulationstation" | "es" | "gamelist" => export_emulationstation(target, games, name_mode),
        "pegasus" => export_pegasus(target, system, games, name_mode),
        "both" => {
            export_emulationstation(target, games, name_mode)?;
            export_pegasus(target, system, games, name_mode)?;
            Ok(target.to_path_buf())
        }
        other => Err(format!("不支持的导出格式: {other}")),
    }
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
    rom_assets_only: bool,
    progress: &dyn Fn(usize, String),
) -> Result<ExportOutcome, String> {
    let source_directory = PathBuf::from(&directory);
    let target = PathBuf::from(target_directory.as_deref().unwrap_or(&directory));
    fs::create_dir_all(&target).map_err(|error| error.to_string())?;

    let copies_library = !paths_equal(&source_directory, &target);
    if copies_library {
        copy_directory_contents_with_progress(
            &source_directory,
            &target,
            if rom_assets_only {
                CopyMode::RomAssetsOnly
            } else {
                CopyMode::All
            },
            progress,
        )?;
    }

    let metadata_path = match find_temp_metadata(&source_directory, &system) {
        Ok(path) => path,
        Err(_) if copies_library => {
            return Ok(ExportOutcome {
                games: 0,
                media: 0,
                output: target,
            });
        }
        Err(error) => return Err(error),
    };
    let temp_directory = metadata_path
        .parent()
        .ok_or_else(|| "临时元数据目录无效".to_string())?;

    progress(
        if copies_library { 72 } else { 0 },
        "读取临时抓取数据...".to_string(),
    );
    let mut metadata = parse_pegasus_file(&metadata_path)?;
    if metadata.games.is_empty() {
        return Err("临时元数据中没有可导出的游戏".to_string());
    }
    // 隐藏的 ROM 不写入 gamelist/pegasus,即不被同步到前端与模拟器。
    let hidden_files = crate::commands::rom::hidden_files_for_directory(&directory);
    if !hidden_files.is_empty() {
        metadata.games.retain(|game| {
            game.file
                .as_deref()
                .map(|file| !crate::commands::rom::file_matches_hidden(file, &hidden_files))
                .unwrap_or(true)
        });
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
    progress(
        if copies_library { 75 } else { 10 },
        "写入元数据...".to_string(),
    );
    let output = export_metadata_formats(
        &target,
        &system,
        &metadata.games,
        name_mode,
        resolved_format,
    )?;

    let media_progress = |current, message| {
        progress(
            if copies_library {
                78 + current * 20 / 100
            } else {
                current
            },
            message,
        )
    };
    let media_count = copy_media(
        &temp_directory.join("media"),
        &target.join("media"),
        &media_progress,
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
    rom_assets_only: Option<bool>,
) -> Result<(), String> {
    if EXPORT_RUNNING.swap(true, Ordering::SeqCst) {
        return Err("已有导出任务正在运行".to_string());
    }
    EXPORT_CANCELLED.store(false, Ordering::SeqCst);
    let progress_app = app.clone();
    let joined = tokio::task::spawn_blocking(move || {
        export_system_data(
            system,
            directory,
            format,
            target_directory,
            name_mode,
            rom_assets_only.unwrap_or(false),
            &|current, message| {
                emit_progress(&progress_app, current, message, false);
            },
        )
    })
    .await;
    EXPORT_RUNNING.store(false, Ordering::SeqCst);
    let outcome = joined.map_err(|error| format!("导出任务失败: {error}"))?;
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(error) if error == EXPORT_CANCELLED_ERROR => {
            emit_progress(&app, 100, "导出已停止；已完成的文件已保留", true);
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    let message = if outcome.games == 0 && outcome.media == 0 {
        format!("ROM 平台目录导出完成 -> {}", display_path(&outcome.output))
    } else {
        format!(
            "导出完成: {} 个游戏、{} 个媒体文件 -> {}",
            outcome.games,
            outcome.media,
            display_path(&outcome.output)
        )
    };
    emit_progress(&app, 100, message, true);
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
    rom_assets_only: Option<bool>,
    system_paths: Option<Vec<String>>,
) -> Result<(), String> {
    let library = get_settings()
        .directories
        .into_iter()
        .find(|library| library.id == library_id)
        .ok_or_else(|| format!("Library 不存在: {library_id}"))?;
    let scan_id = library.id.clone();
    let mut systems = if let Some(cached) = load_cached_roms_for_library(&library.id) {
        cached
    } else {
        tokio::task::spawn_blocking(move || scan_library_by_id(&scan_id, ScanMode::Full, |_| {}))
            .await
            .map_err(|error| format!("ROM 扫描任务失败: {error}"))??
    };
    if let Some(system_paths) = system_paths {
        let selected = system_paths
            .into_iter()
            .map(|path| display_path(Path::new(&path)).to_ascii_lowercase())
            .collect::<std::collections::HashSet<_>>();
        systems.retain(|system| {
            selected.contains(&display_path(Path::new(&system.path)).to_ascii_lowercase())
        });
    }

    let target_directory = target_directory
        .filter(|path| !path.trim().is_empty())
        .ok_or_else(|| "请选择 Library 导出目标目录".to_string())?;
    let library_root_text = display_path(Path::new(&library.path))
        .trim_end_matches('/')
        .to_ascii_lowercase();
    let target_root_text = display_path(Path::new(&target_directory))
        .trim_end_matches('/')
        .to_ascii_lowercase();
    if target_root_text == library_root_text
        || target_root_text.starts_with(&format!("{library_root_text}/"))
    {
        return Err("Library 导出目标不能是源目录或其子目录".to_string());
    }
    if systems.is_empty() {
        return Err(format!("Library“{}”中没有选中的可导出平台", library.name));
    }

    if EXPORT_RUNNING.swap(true, Ordering::SeqCst) {
        return Err("已有导出任务正在运行".to_string());
    }
    EXPORT_CANCELLED.store(false, Ordering::SeqCst);

    let total_systems = systems.len();
    let progress_app = app.clone();
    let target_for_task = target_directory.clone();
    let rom_assets_only = rom_assets_only.unwrap_or(false);
    let joined = tokio::task::spawn_blocking(move || {
        let mut total_games = 0;
        let mut total_media = 0;
        for (index, system) in systems.into_iter().enumerate() {
            let source = PathBuf::from(&system.path);
            let target = library_system_target(&library, &source, Some(&target_for_task));
            let base = index * 100 / total_systems;
            let next = (index + 1) * 100 / total_systems;
            let span = next.saturating_sub(base);
            let system_name = system.system.clone();
            if find_temp_metadata(&source, &system.system).is_ok() {
                let outcome = export_system_data(
                    system.system,
                    system.path,
                    format.clone(),
                    Some(display_path(&target)),
                    name_mode.clone(),
                    rom_assets_only,
                    &|current, message| {
                        emit_progress(
                            &progress_app,
                            base + current * span / 100,
                            format!("[{system_name}] {message}"),
                            false,
                        )
                    },
                )?;
                total_games += outcome.games;
                total_media += outcome.media;
            } else {
                emit_progress(
                    &progress_app,
                    base,
                    format!("[{system_name}] 复制 ROM 与现有资源..."),
                    false,
                );
                copy_directory_contents_with_progress(
                    &source,
                    &target,
                    if rom_assets_only {
                        CopyMode::RomAssetsOnly
                    } else {
                        CopyMode::All
                    },
                    &|current, message| {
                        emit_progress(
                            &progress_app,
                            base + current * span / 100,
                            format!("[{system_name}] {message}"),
                            false,
                        )
                    },
                )?;
                total_games += system.roms.len();
            }
        }
        Ok::<_, String>((total_games, total_media))
    })
    .await;
    EXPORT_RUNNING.store(false, Ordering::SeqCst);
    let outcome = joined.map_err(|error| format!("Library 导出任务失败: {error}"))?;
    let (total_games, total_media) = match outcome {
        Ok(outcome) => outcome,
        Err(error) if error == EXPORT_CANCELLED_ERROR => {
            emit_progress(&app, 100, "Library 导出已停止；已完成的文件已保留", true);
            return Ok(());
        }
        Err(error) => return Err(error),
    };

    let destination = display_path(Path::new(&target_directory));
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
pub fn cancel_export() -> bool {
    if !EXPORT_RUNNING.load(Ordering::SeqCst) {
        return false;
    }
    EXPORT_CANCELLED.store(true, Ordering::SeqCst);
    true
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
    fn emulationstation_export_accepts_empty_and_malformed_existing_ratings() {
        let root = std::env::temp_dir().join(format!(
            "mrrm-export-es-empty-rating-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("gamelist.xml"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<gameList>
  <game><path>./empty.gba</path><name>Empty</name><rating/></game>
  <game><path>./blank.gba</path><name>Blank</name><rating></rating></game>
  <game><path>./invalid.gba</path><name>Invalid</name><rating>unknown</rating></game>
  <game><path>./legacy.gba</path><name>Legacy</name><rating>1600</rating></game>
  <game><path>./amplified.gba</path><name>Amplified</name><rating>800000</rating></game>
</gameList>"#,
        )
        .unwrap();

        let path = export_emulationstation(&root, &[game()], ExportNameMode::Original).unwrap();
        let xml = fs::read_to_string(path).unwrap();
        assert!(xml.contains("<path>./empty.gba</path>"));
        assert!(xml.contains("<path>./game.gba</path>"));
        assert_eq!(xml.matches("<rating>0.8</rating>").count(), 2);
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
    fn combined_metadata_export_writes_pegasus_and_gamelist() {
        let root = std::env::temp_dir().join(format!("mrrm-export-both-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        export_metadata_formats(&root, "GBA", &[game()], ExportNameMode::Original, "both").unwrap();

        assert!(root.join("metadata.pegasus.txt").is_file());
        assert!(root.join("gamelist.xml").is_file());
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
            indexed_folders: None,
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

    #[test]
    fn library_copy_preserves_nested_rom_files_and_rejects_recursive_target() {
        let root = std::env::temp_dir().join(format!("mrrm-library-copy-{}", std::process::id()));
        let source = root.join("source");
        let target = root.join("target");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(source.join("hacks")).unwrap();
        fs::write(source.join("game.gba"), b"rom").unwrap();
        fs::write(source.join("hacks").join("hack.gba"), b"hack").unwrap();

        assert_eq!(copy_directory_contents(&source, &target).unwrap(), 2);
        assert_eq!(fs::read(target.join("game.gba")).unwrap(), b"rom");
        assert_eq!(
            fs::read(target.join("hacks").join("hack.gba")).unwrap(),
            b"hack"
        );
        assert!(copy_directory_contents(&source, &source.join("export")).is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn library_copy_skips_existing_files_with_the_same_size() {
        let root = std::env::temp_dir().join(format!("mrrm-library-skip-{}", std::process::id()));
        let source = root.join("source");
        let target = root.join("target");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(source.join("game.gba"), b"new").unwrap();
        fs::write(target.join("game.gba"), b"old").unwrap();

        assert_eq!(copy_directory_contents(&source, &target).unwrap(), 1);
        assert_eq!(fs::read(target.join("game.gba")).unwrap(), b"old");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn filtered_library_copy_keeps_roms_assets_and_folder_rom_contents_only() {
        let root = std::env::temp_dir().join(format!("mrrm-library-filter-{}", std::process::id()));
        let source = root.join("source");
        let target = root.join("target");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(source.join("Game").join("PS3_GAME")).unwrap();
        fs::write(source.join("game.gba"), b"rom").unwrap();
        fs::write(source.join("cover.png"), b"asset").unwrap();
        fs::write(source.join("readme.txt"), b"ignore").unwrap();
        fs::write(
            source.join("Game").join("PS3_GAME").join("required.dat"),
            b"folder-rom",
        )
        .unwrap();

        assert_eq!(
            copy_directory_contents_with_progress(
                &source,
                &target,
                CopyMode::RomAssetsOnly,
                &|_, _| {}
            )
            .unwrap(),
            3
        );
        assert!(target.join("game.gba").exists());
        assert!(target.join("cover.png").exists());
        assert!(!target.join("readme.txt").exists());
        assert!(target
            .join("Game")
            .join("PS3_GAME")
            .join("required.dat")
            .exists());
        let _ = fs::remove_dir_all(&root);
    }
}
