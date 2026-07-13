use crate::config::get_config_dir;
use crate::rom_service::{
    detect_metadata_format, get_roms_for_directory, get_roms_for_directory_with_progress,
    SystemRoms,
};
use crate::settings::{get_settings, DirectoryConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

const INDEX_VERSION: u32 = 2;

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

fn index_path() -> PathBuf {
    get_config_dir()
        .join("cache")
        .join("rom-library-index.json")
}

fn directories_signature(directories: &[DirectoryConfig]) -> u64 {
    let serialized = serde_json::to_vec(directories).unwrap_or_default();
    stable_hash(&serialized)
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

fn hash_directory(path: &Path, state: &mut impl Hasher) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase());
    for entry in entries {
        let entry_path = entry.path();
        entry
            .file_name()
            .to_string_lossy()
            .to_ascii_lowercase()
            .hash(state);
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        metadata.is_dir().hash(state);
        metadata.len().hash(state);
        metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .map(|value| value.as_nanos())
            .hash(state);
        if metadata.is_dir() {
            hash_directory(&entry_path, state);
        }
    }
}

fn directory_fingerprint(path: &Path) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    normalized_path(path).hash(&mut hasher);
    hash_directory(path, &mut hasher);
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
        if root_has_direct_games(path) {
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
            candidates.push(ScanCandidate {
                config: DirectoryConfig {
                    path: system_path.to_string_lossy().into_owned(),
                    is_root_directory: false,
                    metadata_format: detect_metadata_format(&system_path),
                    system_id: Some(system),
                },
                fingerprint_path: system_path,
            });
        }
    }
    candidates
}

fn load_index() -> Option<RomLibraryIndex> {
    let settings = get_settings();
    let content = fs::read(index_path()).ok()?;
    let index: RomLibraryIndex = serde_json::from_slice(&content).ok()?;
    (index.version == INDEX_VERSION
        && index.directories_signature == directories_signature(&settings.directories))
    .then_some(index)
}

fn flatten_directories(directories: &[IndexedDirectory]) -> Vec<SystemRoms> {
    let mut systems: Vec<_> = directories
        .iter()
        .flat_map(|directory| directory.systems.iter().cloned())
        .collect();
    systems.sort_by(|left, right| left.system.cmp(&right.system));
    systems
}

fn save_index(directories: Vec<IndexedDirectory>) -> Result<Vec<SystemRoms>, String> {
    let settings = get_settings();
    let index = RomLibraryIndex {
        version: INDEX_VERSION,
        directories_signature: directories_signature(&settings.directories),
        directories,
    };
    let path = index_path();
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
    load_index().map(|index| flatten_directories(&index.directories))
}

pub fn invalidate_index() {
    let _ = fs::remove_file(index_path());
}

pub fn scan_library(
    mode: ScanMode,
    on_progress: impl Fn(ScanUpdate),
) -> Result<Vec<SystemRoms>, String> {
    let settings = get_settings();
    let candidates = discover_candidates(&settings.directories);
    let total = candidates.len();
    let previous: HashMap<String, IndexedDirectory> = if mode == ScanMode::Incremental {
        load_index()
            .map(|index| {
                index
                    .directories
                    .into_iter()
                    .map(|item| (normalized_path(Path::new(&item.path)), item))
                    .collect()
            })
            .unwrap_or_default()
    } else {
        HashMap::new()
    };

    let mut indexed = Vec::new();
    for (position, candidate) in candidates.into_iter().enumerate() {
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
        if let Some(cached) = previous
            .get(&key)
            .filter(|item| item.fingerprint == fingerprint)
        {
            on_progress(ScanUpdate {
                current,
                total,
                system,
                changed: false,
            });
            indexed.push(cached.clone());
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
        indexed.push(IndexedDirectory {
            path,
            fingerprint,
            systems,
        });
    }

    indexed.sort_by(|left, right| left.path.cmp(&right.path));
    save_index(indexed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn fingerprint_changes_when_file_changes() {
        let directory = std::env::temp_dir().join(format!("mrrm_index_{}", Uuid::new_v4()));
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("game.gba"), b"one").unwrap();
        let first = directory_fingerprint(&directory);
        fs::write(directory.join("game.gba"), b"changed-size").unwrap();
        let second = directory_fingerprint(&directory);
        assert_ne!(first, second);
        fs::remove_dir_all(directory).unwrap();
    }
}
