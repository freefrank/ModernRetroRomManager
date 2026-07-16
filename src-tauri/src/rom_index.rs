use crate::config::get_config_dir;
use crate::rom_service::{
    detect_metadata_format, get_roms_for_directory, get_roms_for_directory_with_progress,
    SystemRoms,
};
use crate::settings::{get_settings, DirectoryConfig};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::UNIX_EPOCH;

const INDEX_VERSION: u32 = 4;
const FINGERPRINT_VERSION: u8 = 1;
static SCAN_CANCELLED: AtomicBool = AtomicBool::new(false);

pub fn begin_scan() {
    SCAN_CANCELLED.store(false, Ordering::Release);
}

pub fn cancel_scan() {
    SCAN_CANCELLED.store(true, Ordering::Release);
}

pub(crate) fn scan_cancelled() -> bool {
    SCAN_CANCELLED.load(Ordering::Acquire)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanMode {
    Incremental,
    Full,
}

#[derive(Debug, Clone)]
pub struct ScanUpdate {
    pub current: usize,
    pub total: usize,
    pub system: String,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexedDirectory {
    path: String,
    fingerprint: u64,
    #[serde(default)]
    fingerprint_version: u8,
    systems: Vec<SystemRoms>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RomLibraryIndex {
    version: u32,
    directories_signature: u64,
    directories: Vec<IndexedDirectory>,
}

#[derive(Debug, Clone)]
struct ScanCandidate {
    config: DirectoryConfig,
    fingerprint_path: PathBuf,
}

fn index_path_for(library_id: &str) -> PathBuf {
    let id_hash = stable_hash(library_id.as_bytes());
    get_config_dir()
        .join("cache")
        .join(format!("rom-library-index-{id_hash:016x}.json"))
}

fn directories_signature_with_path(
    directories: &[DirectoryConfig],
    map_path: impl Fn(&str) -> String,
) -> u64 {
    // 名称和 ID 不影响扫描内容，重命名 Library 不应使索引失效。
    let scan_configs: Vec<_> = directories
        .iter()
        .map(|item| {
            (
                map_path(&item.path),
                item.is_root_directory,
                item.metadata_format.clone(),
                item.system_id.clone(),
                item.indexed_folders.clone(),
            )
        })
        .collect();
    let serialized = serde_json::to_vec(&scan_configs).unwrap_or_default();
    stable_hash(&serialized)
}

fn directories_signature(directories: &[DirectoryConfig]) -> u64 {
    directories_signature_with_path(directories, |path| normalized_path(Path::new(path)))
}

fn compatible_directories_signatures(directories: &[DirectoryConfig]) -> [u64; 3] {
    // 0.8.4 及更早版本会把原始路径字符串写入签名；兼容两种分隔符，
    // 避免 Windows 路径统一为反斜杠后无意义地丢弃已有索引。
    [
        directories_signature(directories),
        directories_signature_with_path(directories, |path| path.replace('\\', "/")),
        directories_signature_with_path(directories, |path| path.replace('/', "\\")),
    ]
}

fn stable_hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

fn normalized_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_ascii_lowercase()
}

fn hash_directory_shallow(path: &Path, state: &mut impl Hasher) {
    // 增量扫描只观察当前目录的成员。旧实现会递归读取并 stat 每个 ROM，
    // 在 SD 卡和 Samba 上一次“校验”实际等同于完整扫描，会让切库长时间无响应。
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase());
    for entry in entries {
        if scan_cancelled() {
            break;
        }
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        name.hash(state);
        let is_dir = entry.file_type().ok().is_some_and(|kind| kind.is_dir());
        is_dir.hash(state);

        // 目录时间戳可以低成本发现其直属成员变化；元数据清单则需要发现内容更新。
        // 普通 ROM 原地改写由用户触发“全量扫描”处理，避免常规切库读取每个文件。
        let is_metadata_file = matches!(
            name.as_str(),
            "gamelist.xml" | "metadata.pegasus.txt" | "metadata.txt"
        );
        if is_dir || is_metadata_file {
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            metadata.len().hash(state);
            metadata
                .modified()
                .ok()
                .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                .map(|value| value.as_nanos())
                .hash(state);
        }
    }
}

fn directory_fingerprint(path: &Path) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    normalized_path(path).hash(&mut hasher);
    hash_directory_shallow(path, &mut hasher);
    hasher.finish()
}

fn root_has_direct_games(path: &Path) -> bool {
    const ROM_EXTENSIONS: &[&str] = &[
        "zip", "7z", "rar", "nes", "fds", "sfc", "smc", "gb", "gbc", "gba", "nds", "3ds", "cia",
        "n64", "z64", "v64", "md", "gen", "sms", "gg", "ws", "wsc", "pce", "cue", "chd", "iso",
        "cso", "pbp", "gdi", "cdi", "rvz", "wbfs", "wad", "xci", "nsp", "p8", "lnx", "a26", "a52",
        "a78", "col", "vec", "ngc", "ngp",
    ];
    fs::read_dir(path).ok().is_some_and(|entries| {
        entries.filter_map(Result::ok).any(|entry| {
            let entry_path = entry.path();
            (entry_path.is_file()
                && entry_path
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|extension| {
                        ROM_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
                    }))
                || (entry_path.is_dir() && entry_path.join("PS3_GAME").is_dir())
        })
    })
}

fn discover_candidates(directories: &[DirectoryConfig]) -> Vec<ScanCandidate> {
    let mut candidates = Vec::new();
    for directory in directories {
        if directory
            .indexed_folders
            .as_ref()
            .is_some_and(Vec::is_empty)
        {
            continue;
        }
        let path = Path::new(&directory.path);
        if !path.is_dir() {
            continue;
        }
        if !directory.is_root_directory {
            candidates.push(ScanCandidate {
                config: directory.clone(),
                fingerprint_path: path.to_path_buf(),
            });
            continue;
        }

        // 根目录直属游戏（文件或 PS3 文件夹）较少见；存在时保守回退为该根目录全量扫描。
        if directory.indexed_folders.is_none() && root_has_direct_games(path) {
            candidates.push(ScanCandidate {
                config: directory.clone(),
                fingerprint_path: path.to_path_buf(),
            });
            continue;
        }

        let Ok(entries) = fs::read_dir(path) else {
            continue;
        };
        let mut system_dirs: Vec<_> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|entry_path| entry_path.is_dir())
            .collect();
        system_dirs.sort();
        for system_path in system_dirs {
            let system = system_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("Unknown")
                .to_string();
            if directory.indexed_folders.as_ref().is_some_and(|folders| {
                !folders
                    .iter()
                    .any(|folder| folder.eq_ignore_ascii_case(&system))
            }) {
                continue;
            }
            candidates.push(ScanCandidate {
                config: DirectoryConfig {
                    id: directory.id.clone(),
                    name: directory.name.clone(),
                    path: system_path.to_string_lossy().into_owned(),
                    is_root_directory: false,
                    metadata_format: detect_metadata_format(&system_path),
                    system_id: Some(system),
                    indexed_folders: None,
                },
                fingerprint_path: system_path,
            });
        }
    }
    candidates
}

fn load_index_for(library: &DirectoryConfig) -> Option<RomLibraryIndex> {
    let content = fs::read(index_path_for(&library.id)).ok()?;
    let index: RomLibraryIndex = serde_json::from_slice(&content).ok()?;
    let signatures = compatible_directories_signatures(std::slice::from_ref(library));
    (index.version == INDEX_VERSION && signatures.contains(&index.directories_signature))
        .then_some(index)
}

fn flatten_directories(directories: &[IndexedDirectory]) -> Vec<SystemRoms> {
    let mut systems: Vec<_> = directories
        .iter()
        .flat_map(|directory| directory.systems.iter().cloned())
        .collect();
    migrate_legacy_chinese_names(&mut systems);
    systems.sort_by(|left, right| left.system.cmp(&right.system));
    systems
}

fn contains_han(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character as u32, 0x3400..=0x4dbf | 0x4e00..=0x9fff))
}

fn migrate_legacy_chinese_names(systems: &mut [SystemRoms]) {
    for system in systems {
        for rom in &mut system.roms {
            if rom.chinese_name.is_none() && contains_han(&rom.name) {
                if let Some(original) = rom
                    .english_name
                    .as_ref()
                    .filter(|name| !contains_han(name) && *name != &rom.name)
                    .cloned()
                {
                    rom.chinese_name = Some(rom.name.clone());
                    rom.name = original;
                }
            }
            if let Some(preview) = rom.temp_data.as_mut() {
                if preview.chinese_name.is_none() && contains_han(&preview.name) {
                    let original = preview
                        .extra
                        .get("x-mrrm-eng")
                        .cloned()
                        .or_else(|| rom.english_name.clone());
                    if let Some(original) = original.filter(|name| !contains_han(name)) {
                        preview.chinese_name = Some(preview.name.clone());
                        preview.name = original;
                    }
                }
            }
        }
    }
}

fn save_index(
    library: &DirectoryConfig,
    directories: Vec<IndexedDirectory>,
) -> Result<Vec<SystemRoms>, String> {
    let index = RomLibraryIndex {
        version: INDEX_VERSION,
        directories_signature: directories_signature(std::slice::from_ref(library)),
        directories,
    };
    let path = index_path_for(&library.id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(
        &path,
        serde_json::to_vec(&index).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(flatten_directories(&index.directories))
}

pub fn load_cached_roms() -> Option<Vec<SystemRoms>> {
    let settings = get_settings();
    let library = settings.active_library()?;
    load_index_for(library).map(|index| flatten_directories(&index.directories))
}

pub fn load_cached_roms_for_library(library_id: &str) -> Option<Vec<SystemRoms>> {
    let settings = get_settings();
    let library = settings
        .directories
        .iter()
        .find(|library| library.id == library_id)?;
    load_index_for(library).map(|index| flatten_directories(&index.directories))
}

pub fn invalidate_index() {
    if let Some(library) = get_settings().active_library() {
        invalidate_library_index(&library.id);
    }
}

pub fn invalidate_library_index(library_id: &str) {
    let _ = fs::remove_file(index_path_for(library_id));
}

pub fn scan_library(
    mode: ScanMode,
    on_progress: impl Fn(ScanUpdate),
) -> Result<Vec<SystemRoms>, String> {
    let settings = get_settings();
    let Some(library) = settings.active_library().cloned() else {
        return Ok(Vec::new());
    };
    scan_library_config(library, mode, on_progress)
}

pub fn scan_library_by_id(
    library_id: &str,
    mode: ScanMode,
    on_progress: impl Fn(ScanUpdate),
) -> Result<Vec<SystemRoms>, String> {
    let settings = get_settings();
    let library = settings
        .directories
        .into_iter()
        .find(|library| library.id == library_id)
        .ok_or_else(|| format!("Library 不存在: {library_id}"))?;
    scan_library_config(library, mode, on_progress)
}

fn scan_library_config(
    library: DirectoryConfig,
    mode: ScanMode,
    on_progress: impl Fn(ScanUpdate),
) -> Result<Vec<SystemRoms>, String> {
    let candidates = discover_candidates(std::slice::from_ref(&library));
    let total = candidates.len();
    let candidate_keys: HashSet<_> = candidates
        .iter()
        .map(|candidate| normalized_path(&candidate.fingerprint_path))
        .collect();
    let previous: HashMap<String, IndexedDirectory> = load_index_for(&library)
        .map(|index| {
            index
                .directories
                .into_iter()
                .map(|item| (normalized_path(Path::new(&item.path)), item))
                .collect()
        })
        .unwrap_or_default();

    let mut indexed = Vec::new();
    for (position, candidate) in candidates.into_iter().enumerate() {
        if scan_cancelled() {
            break;
        }
        let current = position + 1;
        let system = candidate
            .config
            .system_id
            .clone()
            .or_else(|| {
                candidate
                    .fingerprint_path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "Unknown".to_string());
        let fingerprint = directory_fingerprint(&candidate.fingerprint_path);
        let path = candidate.fingerprint_path.to_string_lossy().into_owned();
        let key = normalized_path(&candidate.fingerprint_path);
        if let Some(cached) = (mode == ScanMode::Incremental)
            .then(|| previous.get(&key))
            .flatten()
            .filter(|item| item.fingerprint_version == 0 || item.fingerprint == fingerprint)
        {
            on_progress(ScanUpdate {
                current,
                total,
                system,
                changed: false,
            });
            let mut migrated = cached.clone();
            migrated.fingerprint = fingerprint;
            migrated.fingerprint_version = FINGERPRINT_VERSION;
            indexed.push(migrated);
            continue;
        }

        on_progress(ScanUpdate {
            current,
            total,
            system,
            changed: true,
        });
        let systems = if candidate.config.is_root_directory {
            get_roms_for_directory_with_progress(&candidate.config, &|_| {})
        } else {
            get_roms_for_directory(&candidate.config)
        };
        let scanned = IndexedDirectory {
            path,
            fingerprint,
            fingerprint_version: FINGERPRINT_VERSION,
            systems,
        };
        if scan_cancelled() {
            // An interrupted refresh must not replace a complete existing platform
            // with a partial one. A brand-new library still keeps what was found.
            indexed.push(previous.get(&key).cloned().unwrap_or(scanned));
            break;
        }
        indexed.push(scanned);
    }

    if scan_cancelled() {
        let indexed_keys: HashSet<_> = indexed
            .iter()
            .map(|item| normalized_path(Path::new(&item.path)))
            .collect();
        indexed.extend(previous.into_iter().filter_map(|(key, item)| {
            (candidate_keys.contains(&key) && !indexed_keys.contains(&key)).then_some(item)
        }));
    }

    indexed.sort_by(|left, right| left.path.cmp(&right.path));
    save_index(&library, indexed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn fingerprint_changes_when_directory_entries_change() {
        let directory = std::env::temp_dir().join(format!("mrrm_index_{}", Uuid::new_v4()));
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("game.gba"), b"one").unwrap();
        let first = directory_fingerprint(&directory);
        fs::write(directory.join("another.gba"), b"two").unwrap();
        let second = directory_fingerprint(&directory);
        assert_ne!(first, second);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn fingerprint_tracks_metadata_file_changes() {
        let directory = std::env::temp_dir().join(format!("mrrm_index_{}", Uuid::new_v4()));
        fs::create_dir_all(&directory).unwrap();
        let metadata = directory.join("metadata.pegasus.txt");
        fs::write(&metadata, b"one").unwrap();
        let first = directory_fingerprint(&directory);
        fs::write(&metadata, b"changed-size").unwrap();
        let second = directory_fingerprint(&directory);
        assert_ne!(first, second);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn cached_legacy_names_are_split_without_rescanning() {
        let mut systems = vec![SystemRoms {
            system: "gba".into(),
            path: "Y:/gba".into(),
            roms: vec![crate::rom_service::RomInfo {
                file: "game.gba".into(),
                name: "实况足球".into(),
                english_name: Some("Winning Eleven".into()),
                ..Default::default()
            }],
        }];
        migrate_legacy_chinese_names(&mut systems);
        assert_eq!(systems[0].roms[0].name, "Winning Eleven");
        assert_eq!(systems[0].roms[0].chinese_name.as_deref(), Some("实况足球"));
    }
}
