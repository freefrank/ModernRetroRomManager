//! 从 MD、N64、SFC ROM（含 ZIP）读取稳定产品码，并映射为标准发行名称。

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

const MD_SERIAL_CSV: &str = "md-serial.csv";
const N64_SERIAL_CSV: &str = "n64-serial.csv";
const SFC_SERIAL_CSV: &str = "sfc-serial.csv";

#[derive(Debug, Clone, PartialEq)]
pub struct CartridgeIdentification {
    pub internal_title: String,
    pub game_code: String,
    pub release_name: String,
    pub scrape_name: String,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
struct SerialEntry {
    serial: String,
    name: String,
    region: String,
}

static MD_INDEX: OnceLock<Vec<SerialEntry>> = OnceLock::new();
static N64_INDEX: OnceLock<Vec<SerialEntry>> = OnceLock::new();
static SFC_INDEX: OnceLock<Vec<SerialEntry>> = OnceLock::new();

fn index(system: &str) -> &'static [SerialEntry] {
    let (lock, data) = match system {
        "MD" => (&MD_INDEX, MD_SERIAL_CSV),
        "N64" => (&N64_INDEX, N64_SERIAL_CSV),
        _ => (&SFC_INDEX, SFC_SERIAL_CSV),
    };
    lock.get_or_init(|| {
        let data = crate::embedded_resources::read_database(data).unwrap_or_default();
        let mut reader = csv::Reader::from_reader(data.as_slice());
        reader
            .records()
            .flatten()
            .map(|record| SerialEntry {
                serial: record.get(0).unwrap_or_default().to_ascii_uppercase(),
                name: record.get(1).unwrap_or_default().to_string(),
                region: record.get(2).unwrap_or_default().to_string(),
            })
            .collect()
    })
}

fn clean_release_name(name: &str) -> String {
    name.find(" (")
        .map_or(name, |position| &name[..position])
        .trim()
        .to_string()
}

fn header_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .trim_matches(|character: char| character == '\0' || character.is_whitespace())
        .to_string()
}

fn read_file_prefix(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut data = Vec::new();
    file.by_ref()
        .take(limit as u64)
        .read_to_end(&mut data)
        .map_err(|error| error.to_string())?;
    Ok(data)
}

fn read_zip_prefix(path: &Path, extensions: &[&str], limit: usize) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| error.to_string())?;
    let mut best = None;
    for position in 0..archive.len() {
        let entry = archive
            .by_index(position)
            .map_err(|error| error.to_string())?;
        let matches = Path::new(entry.name())
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                extensions
                    .iter()
                    .any(|candidate| extension.eq_ignore_ascii_case(candidate))
            });
        if matches && best.is_none_or(|(_, size)| entry.size() > size) {
            best = Some((position, entry.size()));
        }
    }
    let (position, _) = best.ok_or_else(|| "ZIP 内没有对应平台的 ROM".to_string())?;
    let mut entry = archive
        .by_index(position)
        .map_err(|error| error.to_string())?;
    let mut data = Vec::new();
    entry
        .by_ref()
        .take(limit as u64)
        .read_to_end(&mut data)
        .map_err(|error| error.to_string())?;
    Ok(data)
}

fn read_rar_prefix(path: &Path, extensions: &[&str], limit: usize) -> Result<Vec<u8>, String> {
    let mut archive = unrar::Archive::new(path)
        .open_for_processing()
        .map_err(|error| error.to_string())?;
    while let Some(header) = archive.read_header().map_err(|error| error.to_string())? {
        let matches = header
            .entry()
            .filename
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                extensions
                    .iter()
                    .any(|candidate| extension.eq_ignore_ascii_case(candidate))
            });
        if matches {
            let (mut data, _) = header.read().map_err(|error| error.to_string())?;
            data.truncate(limit);
            return Ok(data);
        }
        archive = header.skip().map_err(|error| error.to_string())?;
    }
    Err("RAR 内没有对应平台的 ROM".to_string())
}

fn archive_magic(path: &Path) -> Result<[u8; 8], String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut magic = [0_u8; 8];
    let length = file.read(&mut magic).map_err(|error| error.to_string())?;
    if length < magic.len() {
        magic[length..].fill(0);
    }
    Ok(magic)
}

fn load_prefix(path: &Path, extensions: &[&str], limit: usize) -> Result<Vec<u8>, String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let magic = archive_magic(path)?;
    if magic.starts_with(b"Rar!\x1a\x07") || extension.eq_ignore_ascii_case("rar") {
        read_rar_prefix(path, extensions, limit)
    } else if magic.starts_with(b"PK\x03\x04") || extension.eq_ignore_ascii_case("zip") {
        read_zip_prefix(path, extensions, limit)
    } else {
        read_file_prefix(path, limit)
    }
}

fn normalize_md_code(value: &str) -> String {
    let code = value
        .to_ascii_uppercase()
        .replace("GM", "")
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches('-')
        .to_string();
    if code.matches('-').count() >= 2 {
        code.rsplit_once('-')
            .map_or(code.clone(), |(product, _)| product.to_string())
    } else {
        code
    }
}

fn parse_md(data: &[u8]) -> Option<(String, String)> {
    if data.len() < 0x190 || !data[0x100..0x110].starts_with(b"SEGA") {
        return None;
    }
    let title = header_text(&data[0x150..0x180]);
    let code = normalize_md_code(&header_text(&data[0x180..0x18e]));
    (!title.is_empty() || !code.is_empty()).then_some((title, code))
}

fn normalize_n64_byte_order(data: &mut [u8]) -> bool {
    match data.get(..4) {
        Some([0x80, 0x37, 0x12, 0x40]) => true,
        Some([0x37, 0x80, 0x40, 0x12]) => {
            for pair in data.chunks_exact_mut(2) {
                pair.swap(0, 1);
            }
            true
        }
        Some([0x40, 0x12, 0x37, 0x80]) => {
            for word in data.chunks_exact_mut(4) {
                word.reverse();
            }
            true
        }
        _ => false,
    }
}

fn parse_n64(data: &mut [u8]) -> Option<(String, String)> {
    if data.len() < 0x40 || !normalize_n64_byte_order(data) {
        return None;
    }
    Some((
        header_text(&data[0x20..0x34]),
        header_text(&data[0x3b..0x3f]),
    ))
}

fn parse_sfc(data: &[u8]) -> Option<(String, String)> {
    let copier_header = usize::from(data.len() % 0x8000 == 512) * 512;
    for base in [
        0x7fc0 + copier_header,
        0xffc0 + copier_header,
        0x40ffc0 + copier_header,
    ] {
        if base < 0x10 || data.len() < base + 0x20 {
            continue;
        }
        let checksum = u16::from_le_bytes([data[base + 0x1e], data[base + 0x1f]]);
        let complement = u16::from_le_bytes([data[base + 0x1c], data[base + 0x1d]]);
        if checksum ^ complement != 0xffff {
            continue;
        }
        let code = header_text(&data[base - 0x0e..base - 0x0a]).to_ascii_uppercase();
        let valid_code = code.len() == 4 && code.bytes().all(|byte| byte.is_ascii_alphanumeric());
        return Some((
            header_text(&data[base..base + 21]),
            if valid_code { code } else { String::new() },
        ));
    }
    None
}

fn serial_code(system: &str, serial: &str) -> String {
    match system {
        "MD" => normalize_md_code(serial),
        "N64" => serial.split('-').nth(1).unwrap_or(serial).to_string(),
        "SFC" => serial.split('-').nth(1).unwrap_or(serial).to_string(),
        _ => String::new(),
    }
}

fn identify(
    system: &str,
    internal_title: String,
    game_code: String,
    name_hint: &str,
) -> Option<CartridgeIdentification> {
    let direct: Vec<_> = index(system)
        .iter()
        .filter(|entry| serial_code(system, &entry.serial) == game_code)
        .collect();
    if direct.is_empty() && system == "N64" {
        if let Some(identification) = identify_n64_ique(internal_title.clone(), game_code.clone()) {
            return Some(identification);
        }
    }
    let mut names = HashSet::new();
    let release_names: Vec<_> = direct
        .iter()
        .map(|entry| clean_release_name(&entry.name))
        .filter(|name| names.insert(name.clone()))
        .collect();
    let release_name = if release_names.len() == 1 {
        release_names[0].clone()
    } else if system == "N64" {
        let lower_hint = name_hint.to_ascii_lowercase();
        let requests_master_quest = lower_hint.contains("master quest")
            || lower_hint.contains("ura")
            || name_hint.contains('里')
            || name_hint.contains('裏');
        let master_quest = release_names
            .iter()
            .find(|name| name.to_ascii_lowercase().contains("master quest"));
        if requests_master_quest {
            master_quest.cloned()
        } else {
            let shortest = release_names.iter().min_by_key(|name| name.len());
            shortest
                .filter(|base| {
                    release_names
                        .iter()
                        .all(|name| name == *base || name.starts_with(&format!("{} - ", base)))
                })
                .cloned()
        }
        .or_else(|| {
            identify_by_internal_title(system, internal_title.clone(), game_code.clone())
                .map(|value| value.release_name)
        })?
    } else {
        return identify_by_internal_title(system, internal_title, game_code);
    };

    let family = if system == "N64" && game_code.len() == 4 {
        &game_code[..3]
    } else {
        &game_code
    };
    let mut candidates: HashMap<String, String> = HashMap::new();
    for preferred_region in ["USA", "Europe"] {
        for entry in index(system) {
            let code = serial_code(system, &entry.serial);
            if code.starts_with(family) && entry.region == preferred_region {
                candidates
                    .entry(clean_release_name(&entry.name))
                    .or_insert(entry.region.clone());
            }
        }
        if !candidates.is_empty() {
            break;
        }
    }
    let scrape_name = if candidates.len() == 1 {
        candidates
            .into_keys()
            .next()
            .unwrap_or_else(|| release_name.clone())
    } else {
        release_name.clone()
    };
    Some(CartridgeIdentification {
        internal_title,
        game_code,
        release_name,
        scrape_name,
        confidence: 98.0,
    })
}

fn normalize_title(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn identify_by_internal_title(
    system: &str,
    internal_title: String,
    game_code: String,
) -> Option<CartridgeIdentification> {
    let normalized = normalize_title(&internal_title);
    if normalized.len() < 5 {
        return None;
    }
    let mut names = HashSet::new();
    for entry in index(system) {
        let name = clean_release_name(&entry.name);
        let candidate = normalize_title(&name);
        if candidate == normalized || (normalized.len() >= 10 && candidate.starts_with(&normalized))
        {
            names.insert(name);
        }
    }
    if names.len() != 1 {
        return None;
    }
    let release_name = names.into_iter().next()?;
    Some(CartridgeIdentification {
        internal_title,
        game_code,
        scrape_name: release_name.clone(),
        release_name,
        confidence: 94.0,
    })
}

fn identify_n64_ique(internal_title: String, game_code: String) -> Option<CartridgeIdentification> {
    let bytes = game_code.as_bytes();
    if bytes.len() != 4 || bytes[3] != b'C' || !matches!(bytes[0], b'B' | b'N') {
        return None;
    }
    let family = &game_code[1..3];
    for preferred_region in ["USA", "Europe", "Japan"] {
        let mut names = Vec::new();
        for entry in index("N64") {
            let code = serial_code("N64", &entry.serial);
            if code.len() == 4 && &code[1..3] == family && entry.region == preferred_region {
                let name = clean_release_name(&entry.name);
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }
        if names.len() == 1 {
            let release_name = names.remove(0);
            return Some(CartridgeIdentification {
                internal_title,
                game_code,
                scrape_name: release_name.clone(),
                release_name,
                confidence: 97.0,
            });
        }
        if names.len() > 1 {
            return None;
        }
    }
    None
}

pub fn identify_cartridge_rom(
    path: &Path,
    system: &str,
) -> Result<Option<CartridgeIdentification>, String> {
    let canonical = system.to_ascii_uppercase();
    let (extensions, limit): (&[&str], usize) = match canonical.as_str() {
        "MD" => (&["md", "gen", "bin"], 0x200),
        "N64" => (&["z64", "n64", "v64"], 0x1000),
        "SFC" => (&["sfc", "smc"], 0x410000),
        _ => return Ok(None),
    };
    let mut data = load_prefix(path, extensions, limit)?;
    let parsed = match canonical.as_str() {
        "MD" => parse_md(&data),
        "N64" => parse_n64(&mut data),
        "SFC" => parse_sfc(&data),
        _ => None,
    };
    let name_hint = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    Ok(parsed.and_then(|(title, code)| identify(&canonical, title, code, name_hint)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_mega_drive_product_codes() {
        assert_eq!(normalize_md_code("GM T-76023 -10"), "T-76023");
    }

    #[test]
    fn embedded_databases_are_not_empty() {
        assert!(index("MD").len() > 800);
        assert!(index("N64").len() > 500);
        assert!(index("SFC").len() > 1_000);
    }

    #[test]
    fn identifies_known_product_codes() {
        assert!(identify("MD", "SANGOKUSHI II".into(), "T-76023".into(), "").is_some());
        assert!(identify("N64", "GOLDENEYE".into(), "NGEJ".into(), "").is_some());
        assert!(identify("SFC", "SOCCER".into(), "AY2J".into(), "").is_some());
        let ique = identify("N64", "SUPER MARIO 64".into(), "BSMC".into(), "").unwrap();
        assert_eq!(ique.scrape_name, "Super Mario 64");
        assert_eq!(ique.confidence, 97.0);
        let internal = identify("SFC", "F-ZERO".into(), String::new(), "").unwrap();
        assert_eq!(internal.scrape_name, "F-Zero");
        assert_eq!(internal.confidence, 94.0);
    }

    #[test]
    fn disambiguates_ocarina_master_quest_from_the_archive_name() {
        let master_quest = identify(
            "N64",
            "THE LEGEND OF ZELDA".into(),
            "CZLE".into(),
            "塞尔达传说 时之笛 里[简][atomic_]",
        )
        .unwrap();
        assert_eq!(
            master_quest.scrape_name,
            "Legend of Zelda, The - Ocarina of Time - Master Quest"
        );

        let regular = identify(
            "N64",
            "THE LEGEND OF ZELDA".into(),
            "CZLE".into(),
            "塞尔达传说 时之笛[简]",
        )
        .unwrap();
        assert_eq!(
            regular.scrape_name,
            "Legend of Zelda, The - Ocarina of Time"
        );
    }
}
