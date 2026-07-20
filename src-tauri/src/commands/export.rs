//! 将临时抓取结果导出为 Pegasus 或 EmulationStation 数据包。

use crate::applog;
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

fn emit_file_progress(app: &AppHandle, progress: ExportFileProgress) {
    let _ = app.emit("export-file-progress", progress);
}

/// 复制阶段的细粒度进度:当前单个文件的进度 + 增量总进度 + 速度 + 预计剩余时间(ETA)。
/// 供 Console 显示单文件进度条、导出页显示剩余时间。
#[derive(Clone, Serialize, Debug, Default)]
pub struct ExportFileProgress {
    /// 当前正在处理的文件名(basename)。
    pub file: String,
    /// 当前文件已处理字节。
    pub file_done: u64,
    /// 当前文件总字节。
    pub file_total: u64,
    /// 本次导出实际已写入字节(增量)。
    pub written: u64,
    /// 本次导出实际待写入字节(增量总量)。
    pub pending: u64,
    /// 写入速度(字节/秒)。
    pub speed_bytes: u64,
    /// 预计剩余秒数;0 表示尚不可估(速度未知)。
    pub eta_secs: u64,
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

/// 统计一次复制的规模。返回 `(总文件数, 总字节, 待写入文件数, 待写入字节)`:
/// 后两项为"增量"——目标已存在同尺寸文件会被复制阶段跳过(不写),故不计入。
/// 进度显示用增量,避免把整库大小当分母、让增量同步看起来像全量拷贝。
fn directory_copy_stats(
    source: &Path,
    target: &Path,
    mode: CopyMode,
    inside_folder_rom: bool,
    skip: &dyn Fn(&Path) -> bool,
) -> Result<(usize, u64, usize, u64), String> {
    check_export_cancelled()?;
    let mut files = 0;
    let mut bytes = 0_u64;
    let mut pending_files = 0;
    let mut pending_bytes = 0_u64;
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        check_export_cancelled()?;
        let entry = entry.map_err(|error| error.to_string())?;
        let source_path = entry.path();
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_dir() {
            if skip(&source_path) {
                continue;
            }
            let folder_rom = inside_folder_rom || source_path.join("PS3_GAME").is_dir();
            let (cf, cb, cpf, cpb) = directory_copy_stats(
                &source_path,
                &target.join(entry.file_name()),
                mode,
                folder_rom,
                skip,
            )?;
            files += cf;
            bytes = bytes.saturating_add(cb);
            pending_files += cpf;
            pending_bytes = pending_bytes.saturating_add(cpb);
        } else if file_type.is_file()
            && !skip(&source_path)
            && (mode == CopyMode::All || inside_folder_rom || is_rom_or_asset_file(&source_path))
        {
            let size = entry.metadata().map_err(|error| error.to_string())?.len();
            files += 1;
            bytes = bytes.saturating_add(size);
            // 与 copy_directory_recursive 的“同尺寸即跳过”一致。
            let unchanged = target
                .join(entry.file_name())
                .metadata()
                .is_ok_and(|meta| meta.is_file() && meta.len() == size);
            if !unchanged {
                pending_files += 1;
                pending_bytes = pending_bytes.saturating_add(size);
            }
        }
    }
    Ok((files, bytes, pending_files, pending_bytes))
}

/// 估算被引用媒体复制到 `target` 时实际需要写入的字节数(同尺寸已存在的不计)。
/// 媒体来源优先取 `primary_root`(临时抓取目录),回退 `fallback_root`(源库)。
fn media_pending_bytes(
    referenced: &std::collections::BTreeSet<String>,
    primary_root: &Path,
    fallback_root: &Path,
    target: &Path,
) -> u64 {
    let mut bytes = 0_u64;
    for relative in referenced {
        let primary = primary_root.join(relative);
        let source_file = if primary.is_file() {
            primary
        } else {
            let fallback = fallback_root.join(relative);
            if fallback.is_file() {
                fallback
            } else {
                continue;
            }
        };
        let Ok(metadata) = source_file.metadata() else {
            continue;
        };
        let source_size = metadata.len();
        let target_file = target.join(relative);
        let unchanged = target_file
            .metadata()
            .is_ok_and(|meta| meta.is_file() && meta.len() == source_size);
        if !unchanged {
            bytes = bytes.saturating_add(source_size);
        }
    }
    bytes
}

/// 校验目标磁盘可用空间足以容纳本次导出实际写入量;不足时报错中止(在任何复制之前)。
fn ensure_sufficient_space(target: &Path, needed: u64) -> Result<(), String> {
    if needed == 0 {
        return Ok(());
    }
    let available =
        fs2::available_space(target).map_err(|error| format!("无法读取目标磁盘可用空间: {error}"))?;
    if available < needed {
        return Err(format!(
            "目标磁盘空间不足:本次导出需写入 {},目标仅剩 {}",
            format_bytes(needed),
            format_bytes(available)
        ));
    }
    Ok(())
}

fn copy_directory_recursive(
    source: &Path,
    target: &Path,
    mode: CopyMode,
    inside_folder_rom: bool,
    skip: &dyn Fn(&Path) -> bool,
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
        if skip(&source_path) {
            continue;
        }
        if file_type.is_dir() {
            let folder_rom = inside_folder_rom || source_path.join("PS3_GAME").is_dir();
            copied += copy_directory_recursive(
                &source_path,
                &target_path,
                mode,
                folder_rom,
                skip,
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
    skip: &dyn Fn(&Path) -> bool,
    progress: &dyn Fn(usize, String),
    on_file: &dyn Fn(ExportFileProgress),
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
    progress(0, "正在统计文件与增量...".to_string());
    let root_is_folder_rom = source.join("PS3_GAME").is_dir();
    // 一次遍历同时得到总量与增量(增量=目标已有同尺寸文件不计):两者都展示。
    let (total_files, total_bytes, pending_files, pending_bytes) =
        directory_copy_stats(source, target, mode, root_is_folder_rom, skip)?;
    if total_files == 0 {
        return Ok(0);
    }
    let started = Instant::now();
    let mut last_emit = started;
    let mut copied_files = 0;
    let mut processed_bytes = 0_u64; // 走查进度(含跳过),抵达 total_bytes
    let mut written_bytes = 0_u64; // 实际写入(增量),抵达 pending_bytes
    let mut skipped_files = 0;
    // 当前单文件追踪:文件切换时重置并读取其总大小。
    let mut cur_file: Option<std::path::PathBuf> = None;
    let mut cur_done = 0_u64;
    let mut cur_total = 0_u64;
    copy_directory_recursive(
        source,
        target,
        mode,
        root_is_folder_rom,
        skip,
        &mut |processed, written, file_finished, skipped, source_path| {
            if cur_file.as_deref() != Some(source_path) {
                cur_file = Some(source_path.to_path_buf());
                cur_done = 0;
                cur_total = fs::metadata(source_path).map(|meta| meta.len()).unwrap_or(0);
            }
            cur_done = cur_done.saturating_add(processed);
            processed_bytes = processed_bytes.saturating_add(processed);
            written_bytes = written_bytes.saturating_add(written);
            if file_finished {
                copied_files += 1;
                cur_done = cur_total; // 收尾对齐,避免分块凑不满
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
                // ETA:按写入速度估算剩余增量所需时间;速度未知时为 0(前端显示“计算中”)。
                let eta_secs = if speed > 0 {
                    pending_bytes.saturating_sub(written_bytes) / speed
                } else {
                    0
                };
                on_file(ExportFileProgress {
                    file: source_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                    file_done: cur_done.min(cur_total),
                    file_total: cur_total,
                    written: written_bytes,
                    pending: pending_bytes,
                    speed_bytes: speed,
                    eta_secs,
                });
                // 进度按“走查总量”平滑推进(跳过的也算走过),避免大量跳过时进度卡在 0。
                let percent = if total_bytes == 0 {
                    copied_files * 70 / total_files.max(1)
                } else {
                    (processed_bytes.saturating_mul(70) / total_bytes).min(70) as usize
                };
                progress(
                percent,
                format!(
                    "复制 {copied_files}/{total_files}（跳过 {skipped_files}）· 总 {}/{} · 增量 {}/{}（{} 个）· {} · 写入 {}/s",
                    format_bytes(processed_bytes),
                    format_bytes(total_bytes),
                    format_bytes(written_bytes),
                    format_bytes(pending_bytes),
                    pending_files,
                    source_path.file_name().unwrap_or_default().to_string_lossy(),
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
    copy_directory_contents_with_progress(
        source,
        target,
        CopyMode::All,
        &|_| false,
        &|_, _| {},
        &|_| {},
    )
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

/// 收集元数据实际引用到的全部媒体相对路径(`media/<game>/<asset>.<ext>`)。
/// 只导出被追踪的资源,既不漏(与扩展名无关)、也不产生孤儿。
fn referenced_media_paths(games: &[PegasusGame]) -> std::collections::BTreeSet<String> {
    let mut set = std::collections::BTreeSet::new();
    let mut push = |value: &Option<String>| {
        if let Some(raw) = value {
            let normalized = raw.replace('\\', "/");
            let trimmed = normalized.strip_prefix("./").unwrap_or(&normalized);
            if !trimmed.is_empty() {
                set.insert(trimmed.to_string());
            }
        }
    };
    for game in games {
        push(&game.box_front);
        push(&game.box_back);
        push(&game.box_spine);
        push(&game.box_full);
        push(&game.cartridge);
        push(&game.logo);
        push(&game.marquee);
        push(&game.bezel);
        push(&game.gridicon);
        push(&game.flyer);
        push(&game.background);
        push(&game.music);
        push(&game.screenshot);
        push(&game.titlescreen);
        push(&game.video);
    }
    set
}

/// 按引用清单复制媒体:每个相对路径优先取 `primary_root`(临时抓取目录,最新),
/// 回退 `fallback_root`(源库已应用媒体);两处都没有则跳过(引用悬空,数据问题)。
fn copy_referenced_media(
    referenced: &std::collections::BTreeSet<String>,
    primary_root: &Path,
    fallback_root: &Path,
    target: &Path,
    progress: &dyn Fn(usize, String),
) -> Result<usize, String> {
    let total = referenced.len().max(1);
    let mut copied = 0usize;
    for (index, relative) in referenced.iter().enumerate() {
        check_export_cancelled()?;
        let primary = primary_root.join(relative);
        let source_file = if primary.is_file() {
            primary
        } else {
            let fallback = fallback_root.join(relative);
            if fallback.is_file() {
                fallback
            } else {
                continue;
            }
        };
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
            fs::copy(&source_file, &target_file)
                .map_err(|error| format!("复制媒体 {} 失败: {error}", source_file.display()))?;
        }
        copied += 1;
        progress(
            20 + ((index + 1) * 70 / total),
            format!("导出媒体: {}/{}", index + 1, referenced.len()),
        );
    }
    Ok(copied)
}

fn export_pegasus(
    target: &Path,
    system: &str,
    games: &[PegasusGame],
    name_mode: ExportNameMode,
    fresh: bool,
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
    // fresh(单向同步)时不合并:只写当前导出集合,清除去重/折叠/BIOS 排除后的陈旧条目。
    write_pegasus_file(&path, &normalized_games, &options, !fresh)?;
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
    fresh: bool,
) -> Result<PathBuf, String> {
    let path = target.join("gamelist.xml");
    // fresh(单向同步)时从空开始:只保留当前导出集合,清除陈旧条目(去重/折叠/BIOS 排除
    // 掉的旧条目、以及路径已不在导出集合里的孤儿);顺带避免损坏的旧 gamelist 解析失败阻塞导出。
    let mut list = if !fresh && path.exists() {
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
    fresh: bool,
) -> Result<PathBuf, String> {
    match format {
        "emulationstation" | "es" | "gamelist" => {
            export_emulationstation(target, games, name_mode, fresh)
        }
        "pegasus" => export_pegasus(target, system, games, name_mode, fresh),
        "both" => {
            export_emulationstation(target, games, name_mode, fresh)?;
            export_pegasus(target, system, games, name_mode, fresh)?;
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

/// 判断是否为 ROM 文件(不含媒体/图片,避免单向同步误删封面等资源)。
fn is_rom_extension(path: &Path) -> bool {
    const ROM_EXTENSIONS: &[&str] = &[
        "zip", "7z", "rar", "nes", "fds", "unf", "unif", "sfc", "smc", "fig", "gb", "gbc", "gba",
        "agb", "nds", "dsi", "3ds", "cia", "n64", "z64", "v64", "md", "gen", "smd", "32x", "sms",
        "gg", "ws", "wsc", "pce", "sgx", "cue", "bin", "img", "iso", "chd", "cso", "pbp", "gdi",
        "cdi", "ccd", "sub", "mds", "mdf", "rvz", "gcm", "wbfs", "wad", "xci", "nsp", "pkg", "p8",
        "lnx", "a26", "a52", "a78", "col", "vec", "ngc", "ngp", "mgw", "min", "m3u", "rom",
    ];
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| ROM_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str()))
}

/// 目录内(递归)是否包含 ROM 文件——用于判断顶层子文件夹是否为游戏文件夹。
fn directory_contains_rom(directory: &Path) -> bool {
    let Ok(entries) = fs::read_dir(directory) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if directory_contains_rom(&path) {
                return true;
            }
        } else if is_rom_extension(&path) {
            return true;
        }
    }
    false
}

/// 单向同步:删除目标里源库已隐藏或已删除的 ROM。
/// - keep 集 = 已过滤隐藏的 metadata 中、源文件仍存在的游戏范围键(顶层文件名/子文件夹名)。
/// - **安全网**:任何"源里仍存在且复制阶段会拷回"的顶层条目一律不删——否则删了又立刻被
///   复制重建,纯属反复删/拷。`copy_skips` 必须与主复制用同一套跳过判定,保证删除决策
///   与复制决策一致:只有源里已不存在、或复制会跳过(如隐藏)的条目才允许删。
/// - BIOS 文件(顶层)保留;媒体/资源目录不动;只删 ROM 文件与确含 ROM 的游戏子文件夹。
fn sync_delete_removed_roms(
    source: &Path,
    target: &Path,
    games: &[crate::scraper::pegasus::PegasusGame],
    copy_skips: &dyn Fn(&Path) -> bool,
) -> Result<usize, String> {
    let scope_key = |file: &str| -> String {
        let normalized = file.replace('\\', "/");
        normalized
            .split('/')
            .next()
            .unwrap_or(&normalized)
            .to_lowercase()
    };
    let mut keep: std::collections::HashSet<String> = std::collections::HashSet::new();
    for game in games {
        if let Some(file) = &game.file {
            if source.join(file).exists() {
                keep.insert(scope_key(file));
                // 多碟游戏折叠后 file 指向 <基名>.m3u,而各碟实际位于 <基名>/ 子文件夹,
                // 其顶层条目名(基名)不同于 m3u 文件名。解析 m3u、把各碟引用路径的顶层
                // 条目一并加入 keep(与下方安全网互为兜底)。
                if file.to_ascii_lowercase().ends_with(".m3u") {
                    if let Ok(content) = fs::read_to_string(source.join(file)) {
                        for line in content.lines() {
                            let entry = line.trim();
                            if !entry.is_empty() && !entry.starts_with('#') {
                                keep.insert(scope_key(entry));
                            }
                        }
                    }
                }
            }
        }
    }
    applog!(
        "sync_delete 开始: source={} target={} games={} keep={:?}",
        source.display(),
        target.display(),
        games.len(),
        keep
    );

    // 源里仍存在且复制阶段会拷回的顶层条目 → 删了也会被立刻重建,一律保留(安全网)。
    let restored_by_copy = |raw_name: &std::ffi::OsStr| -> bool {
        let src = source.join(raw_name);
        src.exists() && !copy_skips(&src)
    };

    let mut removed = 0;
    for entry in fs::read_dir(target).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let raw_name = entry.file_name();
        let name = raw_name.to_string_lossy().to_lowercase();
        if path.is_dir() {
            // 媒体/资源/BIOS 目录不参与同步删除
            if matches!(
                name.as_str(),
                "media" | "images" | "artwork" | "screenshots" | "_archives" | "bios"
            ) {
                continue;
            }
            if keep.contains(&name) {
                continue;
            }
            if restored_by_copy(&raw_name) {
                applog!("sync_delete 保留目录(源存在且会复制): {}", path.display());
                continue;
            }
            // 仅删除确实是游戏文件夹(含 ROM)的目录,避免误删任意目录
            if directory_contains_rom(&path) {
                applog!("sync_delete 删除目录: {}", path.display());
                fs::remove_dir_all(&path).map_err(|error| error.to_string())?;
                removed += 1;
            }
        } else if path.is_file() {
            // BIOS 默认复制且保留,不被同步删除
            if crate::rom_service::is_bios_file(&name) {
                continue;
            }
            if !is_rom_extension(&path) {
                continue;
            }
            if keep.contains(&name) {
                continue;
            }
            if restored_by_copy(&raw_name) {
                applog!("sync_delete 保留文件(源存在且会复制): {}", path.display());
                continue;
            }
            applog!("sync_delete 删除文件: {}", path.display());
            fs::remove_file(&path).map_err(|error| error.to_string())?;
            removed += 1;
        }
    }
    applog!("sync_delete 结束: removed={removed}");
    Ok(removed)
}

fn export_system_data(
    system: String,
    directory: String,
    format: Option<String>,
    target_directory: Option<String>,
    name_mode: Option<String>,
    rom_assets_only: bool,
    sync_delete: bool,
    progress: &dyn Fn(usize, String),
    on_file: &dyn Fn(ExportFileProgress),
) -> Result<ExportOutcome, String> {
    let source_directory = PathBuf::from(&directory);
    let target = PathBuf::from(target_directory.as_deref().unwrap_or(&directory));
    fs::create_dir_all(&target).map_err(|error| error.to_string())?;

    let copies_library = !paths_equal(&source_directory, &target);

    // 目标不得位于源 ROM 目录内部——必须在任何删除/复制**之前**校验,
    // 否则 sync_delete 会先删掉源目录里的文件才在复制阶段报错(审计 #3)。
    if copies_library {
        let source_text = display_path(&source_directory)
            .trim_end_matches('/')
            .to_ascii_lowercase();
        let target_text = display_path(&target)
            .trim_end_matches('/')
            .to_ascii_lowercase();
        if target_text.starts_with(&format!("{source_text}/")) {
            return Err("导出目录不能位于源 ROM 目录内部".to_string());
        }
    }

    // 隐藏的 ROM 既不写入 gamelist/pegasus,也不复制到目标。构造 copy 跳过闭包:
    // 源文件相对路径落在隐藏游戏范围内则跳过;BIOS 不会被隐藏,照常复制。
    let hidden_files = crate::commands::rom::hidden_files_for_directory(&directory);
    let skip_source = source_directory.clone();
    let skip_hidden = hidden_files.clone();
    let skip_hidden_source = move |path: &Path| -> bool {
        if skip_hidden.is_empty() {
            return false;
        }
        match path.strip_prefix(&skip_source) {
            Ok(relative) => {
                crate::commands::rom::file_matches_hidden(&relative.to_string_lossy(), &skip_hidden)
            }
            Err(_) => false,
        }
    };

    // 先解析临时元数据并过滤隐藏项(得到可见游戏集,供清理与写入)。
    // 无临时元数据且是跨目录导出时,退化为纯 ROM/资源复制。
    let metadata_path = find_temp_metadata(&source_directory, &system);
    let mut metadata = match &metadata_path {
        Ok(path) => {
            progress(0, "读取临时抓取数据...".to_string());
            let mut parsed = parse_pegasus_file(path)?;
            // 光盘多轨去重:临时元数据可能把同一张盘的 .bin/.img/.sub 载荷轨与 .cue
            // 描述文件并列成多条。加载(库视图)走 keep_temp_metadata_entry 去过重,但
            // 导出此前直接读文件、绕过了它,导致 gamelist 每个游戏双份列出——载荷轨成了
            // 无法启动、只有原始文件名的"游戏",ES 里看起来就是 metadata 读不到。此处复用
            // 同一规则,一张盘只保留描述文件(cue/ccd/mds 或折叠后的 m3u)。
            {
                let all_files_lower: std::collections::HashSet<String> = parsed
                    .games
                    .iter()
                    .filter_map(|game| game.file.as_deref())
                    .map(|file| file.to_lowercase())
                    .collect();
                let system_lower = system.to_lowercase();
                parsed.games.retain(|game| {
                    game.file
                        .as_deref()
                        .map(|file| {
                            crate::rom_service::keep_temp_metadata_entry(
                                file,
                                &all_files_lower,
                                &system_lower,
                            )
                        })
                        .unwrap_or(true)
                });
            }
            if !hidden_files.is_empty() {
                parsed.games.retain(|game| {
                    game.file
                        .as_deref()
                        .map(|file| !crate::commands::rom::file_matches_hidden(file, &hidden_files))
                        .unwrap_or(true)
                });
            }
            // 防御:只导出源头实际存在的 ROM 文件。临时元数据可能残留陈旧条目(如折叠 bug
            // 留下的双层路径、或改名/删除后未重扫的条目),否则会把这些"幽灵"写进 gamelist,
            // 指向根本没复制过去的文件,ES 里显示为无法启动的坏条目。
            parsed.games.retain(|game| {
                game.file
                    .as_deref()
                    .map(|file| source_directory.join(file.replace('\\', "/")).exists())
                    .unwrap_or(false)
            });
            Some(parsed)
        }
        Err(_) if copies_library => None,
        Err(error) => return Err(error.clone()),
    };

    let copy_mode = if rom_assets_only {
        CopyMode::RomAssetsOnly
    } else {
        CopyMode::All
    };

    // 有临时元数据时,媒体改由 copy_referenced_media 按引用精确导出;主库复制跳过整个
    // media/ 子树,避免白名单漏拷非常规扩展名(如 .ico/.php)以及复制出无人引用的孤儿。
    let skip_media_subtree = metadata.is_some();
    let media_root_dir = source_directory.join("media");
    let skip_for_copy = |path: &Path| -> bool {
        skip_hidden_source(path) || (skip_media_subtree && path == media_root_dir.as_path())
    };

    // 有临时元数据但可见游戏为空(如整平台被隐藏)时,绝不执行 sync_delete——
    // 否则会以空 keep 清空目标平台 ROM 后才在下方报错中止(审计 #2)。
    if let Some(parsed) = &metadata {
        if parsed.games.is_empty() {
            return Err("临时元数据中没有可导出的游戏".to_string());
        }
    }

    applog!(
        "导出开始: system={system} source={} target={} copies_library={copies_library} \
         sync_delete={sync_delete} rom_assets_only={rom_assets_only} games={}",
        source_directory.display(),
        target.display(),
        metadata.as_ref().map(|m| m.games.len()).unwrap_or(0)
    );

    // 导出到外部目录时:先清理(单向同步删除目标中源已隐藏/删除的 ROM),再校验空间,最后复制。
    if copies_library {
        if sync_delete {
            if let Some(parsed) = &metadata {
                progress(2, "单向同步:清理目标中已移除的 ROM...".to_string());
                sync_delete_removed_roms(
                    &source_directory,
                    &target,
                    &parsed.games,
                    &skip_for_copy,
                )?;
            }
        }

        // 清理之后再校验:统计实际待写入体积(主库 + 被引用媒体),与目标可用空间比较。
        // 顺序很关键——清理会释放空间,提前校验才能反映清理后的真实余量(需求)。
        progress(3, "校验目标磁盘可用空间...".to_string());
        let root_is_folder_rom = source_directory.join("PS3_GAME").is_dir();
        // 复用 directory_copy_stats 的增量(pending),与复制进度用同一实现,避免两套算法漂移。
        let (_, _, _, pending) = directory_copy_stats(
            &source_directory,
            &target,
            copy_mode,
            root_is_folder_rom,
            &skip_for_copy,
        )?;
        let mut needed = pending;
        if let Some(parsed) = &metadata {
            if let Some(temp_directory) = metadata_path.as_ref().ok().and_then(|path| path.parent())
            {
                let referenced = referenced_media_paths(&parsed.games);
                needed = needed.saturating_add(media_pending_bytes(
                    &referenced,
                    temp_directory,
                    &source_directory,
                    &target,
                ));
            }
        }
        ensure_sufficient_space(&target, needed)?;

        copy_directory_contents_with_progress(
            &source_directory,
            &target,
            copy_mode,
            &skip_for_copy,
            progress,
            on_file,
        )?;
    }

    // 纯复制场景(无临时元数据):复制已完成,直接返回。
    let Some(mut metadata) = metadata.take() else {
        return Ok(ExportOutcome {
            games: 0,
            media: 0,
            output: target,
        });
    };
    if metadata.games.is_empty() {
        return Err("临时元数据中没有可导出的游戏".to_string());
    }
    let temp_directory = metadata_path
        .as_ref()
        .ok()
        .and_then(|path| path.parent())
        .ok_or_else(|| "临时元数据目录无效".to_string())?;
    for game in &mut metadata.games {
        normalize_game_paths(game);
    }
    let name_mode = ExportNameMode::parse(name_mode.as_deref())?;

    let format = format
        .unwrap_or_else(|| "auto".to_string())
        .to_ascii_lowercase();
    // 默认同时写 gamelist.xml + metadata.pegasus.txt,兼容 ES/batocera 与 Pegasus 前端。
    let resolved_format = if format == "auto" {
        "both"
    } else {
        format.as_str()
    };
    progress(
        if copies_library { 75 } else { 10 },
        "写入元数据...".to_string(),
    );
    // 元数据始终全新写(不合并):metadata.games 是该平台当前全部可见游戏(去隐藏/去重/
    // 折叠后)的权威集合,gamelist/pegasus 就该等于它。合并只会不断堆积陈旧条目(碟轨重复、
    // 折叠掉的旧碟、排除的 BIOS、隐藏游戏残留),与是否单向同步无关。单向同步只是额外再删
    // 目标里的 ROM 文件。
    let output = export_metadata_formats(
        &target,
        &system,
        &metadata.games,
        name_mode,
        resolved_format,
        true,
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
    // 只导出元数据实际引用的媒体:优先取临时抓取目录(最新),回退源库已应用媒体。
    let referenced = referenced_media_paths(&metadata.games);
    let media_count = copy_referenced_media(
        &referenced,
        temp_directory,
        &source_directory,
        &target,
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
    sync_delete: Option<bool>,
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
            sync_delete.unwrap_or(false),
            &|current, message| {
                emit_progress(&progress_app, current, message, false);
            },
            &|file_progress| emit_file_progress(&progress_app, file_progress),
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
    sync_delete: Option<bool>,
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
    let sync_delete = sync_delete.unwrap_or(false);
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
                    sync_delete,
                    &|current, message| {
                        emit_progress(
                            &progress_app,
                            base + current * span / 100,
                            format!("[{system_name}] {message}"),
                            false,
                        )
                    },
                    &|file_progress| emit_file_progress(&progress_app, file_progress),
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
                let copy_mode = if rom_assets_only {
                    CopyMode::RomAssetsOnly
                } else {
                    CopyMode::All
                };
                // 纯复制回退同样先校验空间,避免中途写满目标磁盘。
                fs::create_dir_all(&target).map_err(|error| error.to_string())?;
                let root_is_folder_rom = source.join("PS3_GAME").is_dir();
                let (_, _, _, needed) =
                    directory_copy_stats(&source, &target, copy_mode, root_is_folder_rom, &|_| {
                        false
                    })?;
                ensure_sufficient_space(&target, needed)?;
                copy_directory_contents_with_progress(
                    &source,
                    &target,
                    copy_mode,
                    &|_| false,
                    &|current, message| {
                        emit_progress(
                            &progress_app,
                            base + current * span / 100,
                            format!("[{system_name}] {message}"),
                            false,
                        )
                    },
                    &|file_progress| emit_file_progress(&progress_app, file_progress),
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
    fn copy_skip_excludes_hidden_source_files() {
        let base = std::env::temp_dir().join(format!(
            "mrrm-copyskip-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source = base.join("src");
        let target = base.join("dst");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("visible.sfc"), b"rom").unwrap();
        fs::write(source.join("hidden.sfc"), b"rom").unwrap();
        fs::write(source.join("scph1001.bin"), b"bios").unwrap();

        let hidden = vec!["hidden.sfc".to_string()];
        let src_root = source.clone();
        let skip = move |path: &Path| -> bool {
            path.strip_prefix(&src_root)
                .ok()
                .map(|rel| {
                    crate::commands::rom::file_matches_hidden(&rel.to_string_lossy(), &hidden)
                })
                .unwrap_or(false)
        };
        copy_directory_contents_with_progress(&source, &target, CopyMode::All, &skip, &|_, _| {}, &|_| {})
            .unwrap();

        assert!(target.join("visible.sfc").exists(), "可见 ROM 应复制");
        assert!(!target.join("hidden.sfc").exists(), "隐藏 ROM 不应复制");
        assert!(target.join("scph1001.bin").exists(), "BIOS 应照常复制");
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn sync_delete_removes_hidden_and_deleted_roms_but_keeps_bios_and_visible() {
        let base = std::env::temp_dir().join(format!(
            "mrrm-sync-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source = base.join("src");
        let target = base.join("dst");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&target).unwrap();

        // 源:仅保留 keep.sfc(可见);hidden.sfc 与 gone.sfc 不在 games(隐藏/删除)
        fs::write(source.join("keep.sfc"), b"rom").unwrap();
        // 目标:含可见、隐藏、已删除、BIOS、媒体目录
        fs::write(target.join("keep.sfc"), b"rom").unwrap();
        fs::write(target.join("hidden.sfc"), b"rom").unwrap();
        fs::write(target.join("gone.sfc"), b"rom").unwrap();
        fs::write(target.join("scph1001.bin"), b"bios").unwrap();
        fs::create_dir_all(target.join("media")).unwrap();
        fs::write(target.join("media").join("keep.png"), b"img").unwrap();

        // metadata 只含可见游戏 keep.sfc(隐藏项已在导出前从 games 过滤)
        let visible = PegasusGame {
            name: "Keep".into(),
            file: Some("keep.sfc".into()),
            ..Default::default()
        };
        let removed = sync_delete_removed_roms(&source, &target, &[visible], &|_: &Path| false).unwrap();

        assert_eq!(removed, 2, "应删除 hidden.sfc 与 gone.sfc");
        assert!(target.join("keep.sfc").exists(), "可见 ROM 保留");
        assert!(!target.join("hidden.sfc").exists(), "隐藏 ROM 删除");
        assert!(!target.join("gone.sfc").exists(), "已删除 ROM 删除");
        assert!(target.join("scph1001.bin").exists(), "BIOS 必须保留");
        assert!(
            target.join("media").join("keep.png").exists(),
            "媒体不受影响"
        );
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn sync_delete_removes_hidden_disc_game_folder() {
        let base = std::env::temp_dir().join(format!(
            "mrrm-sync-disc-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source = base.join("src");
        let target = base.join("dst");
        fs::create_dir_all(source.join("KeepGame")).unwrap();
        fs::write(source.join("KeepGame").join("Keep.cue"), b"cue").unwrap();
        // 目标含保留游戏文件夹与一个源已移除的游戏文件夹
        fs::create_dir_all(target.join("KeepGame")).unwrap();
        fs::write(target.join("KeepGame").join("Keep.cue"), b"cue").unwrap();
        fs::write(target.join("KeepGame").join("Keep.bin"), b"bin").unwrap();
        fs::create_dir_all(target.join("HiddenGame")).unwrap();
        fs::write(target.join("HiddenGame").join("Hidden.cue"), b"cue").unwrap();
        fs::write(target.join("HiddenGame").join("Hidden.bin"), b"bin").unwrap();

        let visible = PegasusGame {
            name: "Keep".into(),
            file: Some(r"KeepGame\Keep.cue".into()),
            ..Default::default()
        };
        let removed = sync_delete_removed_roms(&source, &target, &[visible], &|_: &Path| false).unwrap();

        assert_eq!(removed, 1);
        assert!(target.join("KeepGame").join("Keep.cue").exists());
        assert!(
            !target.join("HiddenGame").exists(),
            "整个隐藏游戏文件夹删除"
        );
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn sync_delete_keeps_multidisc_folder_referenced_by_m3u() {
        // 回归:多碟游戏折叠后 metadata.file 指向 <基名>.m3u,各碟在 <基名>/ 子文件夹。
        // 单向同步不得把该碟片文件夹当作"已移除"删掉(此前每次导出都会删一堆已有 ROM)。
        let base = std::env::temp_dir().join(format!(
            "mrrm-sync-m3u-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source = base.join("src");
        let target = base.join("dst");
        // 源:折叠后的结构——FF7.m3u + FF7/ 子文件夹(各碟)
        fs::create_dir_all(source.join("FF7")).unwrap();
        fs::write(
            source.join("FF7.m3u"),
            "FF7/FF7 (Disc 1).cue\nFF7/FF7 (Disc 2).cue\n",
        )
        .unwrap();
        fs::write(source.join("FF7").join("FF7 (Disc 1).cue"), b"cue").unwrap();
        fs::write(source.join("FF7").join("FF7 (Disc 2).cue"), b"cue").unwrap();
        // 目标:上次导出留下的同样结构 + 一个源已移除的游戏文件夹
        fs::create_dir_all(target.join("FF7")).unwrap();
        fs::write(target.join("FF7.m3u"), "FF7/FF7 (Disc 1).cue\n").unwrap();
        fs::write(target.join("FF7").join("FF7 (Disc 1).cue"), b"cue").unwrap();
        fs::write(target.join("FF7").join("FF7 (Disc 2).cue"), b"cue").unwrap();
        fs::create_dir_all(target.join("GoneGame")).unwrap();
        fs::write(target.join("GoneGame").join("Gone.cue"), b"cue").unwrap();

        let game = PegasusGame {
            name: "FF7".into(),
            file: Some("FF7.m3u".into()),
            ..Default::default()
        };
        let removed = sync_delete_removed_roms(&source, &target, &[game], &|_: &Path| false).unwrap();

        assert_eq!(removed, 1, "只应删除源已移除的 GoneGame");
        assert!(
            target.join("FF7").join("FF7 (Disc 1).cue").exists(),
            "m3u 引用的碟片文件夹必须保留"
        );
        assert!(target.join("FF7").join("FF7 (Disc 2).cue").exists());
        assert!(target.join("FF7.m3u").exists(), "m3u 本身保留");
        assert!(!target.join("GoneGame").exists(), "源已移除的游戏应删除");
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn sync_delete_keeps_source_present_rom_even_when_keep_misses_it() {
        // 安全网回归:即便 keep 集因 metadata.file 格式对不上而漏掉某个可见 ROM,
        // 只要它在源里仍存在且复制阶段会拷回,就绝不能删——否则删了又立刻重传(churn)。
        let base = std::env::temp_dir().join(format!(
            "mrrm-sync-safety-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source = base.join("src");
        let target = base.join("dst");
        // 源:一个平铺 ROM + 一个文件夹型游戏,都仍在库里
        fs::create_dir_all(source.join("FolderGame")).unwrap();
        fs::write(source.join("Flat.chd"), b"rom").unwrap();
        fs::write(source.join("FolderGame").join("game.mdf"), b"rom").unwrap();
        // 目标:同样内容 + 一个源已移除的游戏
        fs::create_dir_all(target.join("FolderGame")).unwrap();
        fs::write(target.join("Flat.chd"), b"rom").unwrap();
        fs::write(target.join("FolderGame").join("game.mdf"), b"rom").unwrap();
        fs::create_dir_all(target.join("Removed")).unwrap();
        fs::write(target.join("Removed").join("old.chd"), b"rom").unwrap();

        // 故意传空 games:keep 全空,只能靠安全网保住源里仍在的条目。
        // copy_skips 返回 false = 复制阶段会拷所有源文件。
        let removed =
            sync_delete_removed_roms(&source, &target, &[], &|_: &Path| false).unwrap();

        assert_eq!(removed, 1, "只应删除源已移除的 Removed");
        assert!(target.join("Flat.chd").exists(), "源里仍在的平铺 ROM 必须保留");
        assert!(
            target.join("FolderGame").join("game.mdf").exists(),
            "源里仍在的文件夹型游戏必须保留"
        );
        assert!(!target.join("Removed").exists(), "源已移除的游戏应删除");
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn export_dedupes_multitrack_payload_leaving_descriptor_only() {
        // 回归:光盘平台导出前须复用 keep_temp_metadata_entry 去重,单碟游戏的 .bin/.img
        // 载荷轨不得与 .cue 双份写进 gamelist(此前导致 ES 里每个游戏出现两条、载荷轨无元数据)。
        let files = vec![
            "七风岛物语[简]/七风岛物语.cue".to_string(),
            "七风岛物语[简]/七风岛物语.BIN".to_string(),
            "D之食卓[简].m3u".to_string(),
            "D之食卓[简]/D之食卓[简]CD1/disc1.cue".to_string(),
            "D之食卓[简]/D之食卓[简]CD1/disc1.img".to_string(),
        ];
        let all_lower: std::collections::HashSet<String> =
            files.iter().map(|f| f.to_lowercase()).collect();
        let kept: Vec<&String> = files
            .iter()
            .filter(|f| crate::rom_service::keep_temp_metadata_entry(f, &all_lower, "saturn"))
            .collect();
        // 保留:两个 cue 描述文件 + m3u;剔除:BIN/img 载荷轨。
        assert!(kept.iter().any(|f| f.ends_with("七风岛物语.cue")));
        assert!(kept.iter().any(|f| f.ends_with(".m3u")));
        assert!(kept.iter().any(|f| f.ends_with("disc1.cue")));
        assert!(!kept.iter().any(|f| f.to_lowercase().ends_with(".bin")));
        assert!(!kept.iter().any(|f| f.to_lowercase().ends_with(".img")));
    }

    #[test]
    fn emulationstation_export_contains_metadata_and_assets() {
        let root = std::env::temp_dir().join(format!("mrrm-export-es-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = export_emulationstation(&root, &[game()], ExportNameMode::Original, false).unwrap();
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

        let path = export_emulationstation(&root, &[game()], ExportNameMode::Original, false).unwrap();
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
        let path = export_pegasus(&root, "GBA", &[game()], ExportNameMode::Original, false).unwrap();
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("assets.boxFront: media/game/boxfront.png"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn fresh_export_prunes_stale_gamelist_entries_but_merge_keeps_them() {
        // 单向同步(fresh=true)时 gamelist 只保留当前导出集合,清除路径已不在集合里的陈旧条目;
        // 非 fresh 仍合并保留。修复:去重/折叠/BIOS 排除掉的旧条目此前会一直赖在 gamelist 里。
        let root = std::env::temp_dir().join(format!(
            "mrrm-export-fresh-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let mk = |name: &str, file: &str| PegasusGame {
            name: name.into(),
            file: Some(file.into()),
            ..Default::default()
        };

        // 先写入 old.gba
        export_emulationstation(&root, &[mk("Old", "old.gba")], ExportNameMode::Original, false)
            .unwrap();
        // fresh 写入 new.gba:old 应被清除
        export_emulationstation(&root, &[mk("New", "new.gba")], ExportNameMode::Original, true)
            .unwrap();
        let fresh = fs::read_to_string(root.join("gamelist.xml")).unwrap();
        assert!(fresh.contains("new.gba"), "新条目应在");
        assert!(!fresh.contains("old.gba"), "fresh 应清除陈旧条目");

        // 非 fresh 再写 old.gba:合并保留两者
        export_emulationstation(&root, &[mk("Old", "old.gba")], ExportNameMode::Original, false)
            .unwrap();
        let merged = fs::read_to_string(root.join("gamelist.xml")).unwrap();
        assert!(
            merged.contains("old.gba") && merged.contains("new.gba"),
            "非 fresh 合并保留两者"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn combined_metadata_export_writes_pegasus_and_gamelist() {
        let root = std::env::temp_dir().join(format!("mrrm-export-both-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        export_metadata_formats(&root, "GBA", &[game()], ExportNameMode::Original, "both", false).unwrap();

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
            export_pegasus(&root, "GBA", &[mixed.clone()], ExportNameMode::Original, false).unwrap();
        let pegasus_content = fs::read_to_string(pegasus).unwrap();
        assert!(pegasus_content.contains("file: folder/game.gba"));
        assert!(pegasus_content.contains("assets.boxFront: media/game/boxfront.png"));

        let es = export_emulationstation(&root, &[mixed], ExportNameMode::Original, false).unwrap();
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
            export_pegasus(&root, "GBA", &[named.clone()], ExportNameMode::Original, false).unwrap();
        let content = fs::read_to_string(original).unwrap();
        assert!(content.contains("game: Original Game"));
        assert!(content.contains("x-mrrm-cn: 中文游戏"));

        let chinese = export_emulationstation(&root, &[named], ExportNameMode::Chinese, false).unwrap();
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
    fn referenced_media_copy_pulls_only_tracked_assets_extension_agnostic() {
        let root = std::env::temp_dir().join(format!("mrrm-refmedia-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let temp = root.join("temp");
        let source = root.join("source");
        let target = root.join("target");
        // 追踪的资源:temp 优先命中 boxfront(.php 端点也照拷)、source 回退命中 logo;
        // titlescreen 无对应文件 -> 跳过;orphan.png 未被引用 -> 不拷。
        fs::create_dir_all(temp.join("media/game")).unwrap();
        fs::create_dir_all(source.join("media/game")).unwrap();
        fs::write(temp.join("media/game/boxfront.php"), b"img-new").unwrap();
        fs::write(source.join("media/game/logo.png"), b"img-old").unwrap();
        fs::write(source.join("media/game/orphan.png"), b"orphan").unwrap();

        let mut g = game();
        g.box_front = Some("media/game/boxfront.php".into());
        g.logo = Some("media/game/logo.png".into());
        g.screenshot = None;
        g.titlescreen = Some("media/game/titlescreen.png".into());

        let referenced = referenced_media_paths(std::slice::from_ref(&g));
        let copied =
            copy_referenced_media(&referenced, &temp, &source, &target, &|_, _| {}).unwrap();

        assert_eq!(
            copied, 2,
            "命中 boxfront(temp) 与 logo(source),titlescreen 缺失跳过"
        );
        assert_eq!(
            fs::read(target.join("media/game/boxfront.php")).unwrap(),
            b"img-new",
            ".php 端点媒体照样导出(不受扩展名白名单影响)"
        );
        assert!(target.join("media/game/logo.png").exists(), "回退源库命中");
        assert!(
            !target.join("media/game/orphan.png").exists(),
            "未被引用的孤儿不导出"
        );
        assert!(
            !target.join("media/game/titlescreen.png").exists(),
            "无对应文件的引用跳过"
        );
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
                &|_| false,
                &|_, _| {},
                &|_| {},
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
