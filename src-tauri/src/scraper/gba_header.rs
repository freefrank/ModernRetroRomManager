//! 从 GBA ROM（含 ZIP）读取 Header，并映射为标准英文抓取名称。

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

const GBA_SERIAL_CSV: &str = include_str!("../../resources/gba-serial.csv");

#[derive(Debug, Clone)]
struct SerialEntry {
    name: String,
    region: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GbaIdentification {
    pub internal_title: String,
    pub game_code: String,
    pub release_name: String,
    pub scrape_name: String,
    pub confidence: f32,
}

struct SerialIndex {
    by_serial: HashMap<String, Vec<SerialEntry>>,
    by_prefix: HashMap<String, Vec<(String, SerialEntry)>>,
}

static SERIAL_INDEX: OnceLock<SerialIndex> = OnceLock::new();

fn clean_release_name(name: &str) -> String {
    name.find(" (")
        .map_or(name, |pos| &name[..pos])
        .trim()
        .to_string()
}

fn has_numbered_installment(name: &str) -> bool {
    name.split(|character: char| !character.is_ascii_alphanumeric())
        .any(|token| {
            token
                .parse::<u8>()
                .is_ok_and(|number| (2..=20).contains(&number))
                || matches!(
                    token.to_ascii_uppercase().as_str(),
                    "II" | "III" | "IV" | "V" | "VI" | "VII" | "VIII" | "IX" | "X"
                )
        })
}

fn choose_regional_scrape_name(release_name: &str, regional_name: String) -> String {
    if has_numbered_installment(release_name) && !has_numbered_installment(&regional_name) {
        release_name.to_string()
    } else {
        regional_name
    }
}

fn serial_index() -> &'static SerialIndex {
    SERIAL_INDEX.get_or_init(|| {
        let mut by_serial: HashMap<String, Vec<SerialEntry>> = HashMap::new();
        let mut by_prefix: HashMap<String, Vec<(String, SerialEntry)>> = HashMap::new();
        let mut reader = csv::Reader::from_reader(GBA_SERIAL_CSV.as_bytes());

        for record in reader.records().flatten() {
            let Some(serial) = record.get(0).filter(|value| value.len() == 4) else {
                continue;
            };
            let entry = SerialEntry {
                name: record.get(1).unwrap_or_default().to_string(),
                region: record.get(2).unwrap_or_default().to_string(),
            };
            by_serial
                .entry(serial.to_string())
                .or_default()
                .push(entry.clone());
            by_prefix
                .entry(serial[..3].to_string())
                .or_default()
                .push((serial.to_string(), entry));
        }
        SerialIndex {
            by_serial,
            by_prefix,
        }
    })
}

fn read_header<R: Read>(reader: &mut R) -> Result<[u8; 0xC0], String> {
    let mut header = [0_u8; 0xC0];
    reader.read_exact(&mut header).map_err(|e| e.to_string())?;
    Ok(header)
}

fn read_zip_header(file: File) -> Result<[u8; 0xC0], String> {
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut best: Option<(usize, u64)> = None;
    for index in 0..archive.len() {
        let entry = archive.by_index(index).map_err(|e| e.to_string())?;
        let is_gba = Path::new(entry.name())
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| {
                value.eq_ignore_ascii_case("gba") || value.eq_ignore_ascii_case("agb")
            });
        if is_gba && best.is_none_or(|(_, size)| entry.size() > size) {
            best = Some((index, entry.size()));
        }
    }
    let (index, _) = best.ok_or_else(|| "ZIP 内没有 GBA ROM".to_string())?;
    let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
    read_header(&mut entry)
}

fn load_header(path: &Path) -> Result<[u8; 0xC0], String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("zip"))
    {
        read_zip_header(file)
    } else {
        read_header(&mut std::io::BufReader::new(file))
    }
}

fn header_text(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).trim().to_string()
}

fn unique_names(entries: &[SerialEntry]) -> Vec<String> {
    let mut seen = HashSet::new();
    entries
        .iter()
        .map(|entry| clean_release_name(&entry.name))
        .filter(|name| seen.insert(name.clone()))
        .collect()
}

pub fn identify_gba_rom(path: &Path) -> Result<Option<GbaIdentification>, String> {
    let header = load_header(path)?;
    let checksum = ((0_u8)
        .wrapping_sub(
            header[0xA0..0xBD]
                .iter()
                .fold(0_u8, |sum, byte| sum.wrapping_add(*byte)),
        )
        .wrapping_sub(0x19))
        == header[0xBD];
    if header[0xB2] != 0x96 || !checksum {
        return Ok(None);
    }

    let internal_title = header_text(&header[0xA0..0xAC]);
    let game_code = header_text(&header[0xAC..0xB0]);
    if game_code.len() != 4 || !game_code.is_ascii() {
        return Ok(None);
    }

    let index = serial_index();
    let Some(direct_entries) = index.by_serial.get(&game_code) else {
        return Ok(None);
    };
    let release_names = unique_names(direct_entries);
    if release_names.len() != 1 {
        return Ok(None);
    }
    let release_name = release_names[0].clone();

    let mut english_candidates = Vec::new();
    if let Some(siblings) = index.by_prefix.get(&game_code[..3]) {
        for preferred_region in ["USA", "Europe"] {
            for (serial, entry) in siblings {
                if serial != &game_code && entry.region == preferred_region {
                    let name = clean_release_name(&entry.name);
                    if !english_candidates.contains(&name) {
                        english_candidates.push(name);
                    }
                }
            }
            if !english_candidates.is_empty() {
                break;
            }
        }
    }

    let (scrape_name, confidence) = if english_candidates.len() == 1 {
        let regional_name = english_candidates.remove(0);
        // 部分地区会省略续作编号（例如 A9CE 的欧版标题没有“2”）。
        // 已识别版本包含明确编号时不能用无编号的跨区标题覆盖它。
        (
            choose_regional_scrape_name(&release_name, regional_name),
            98.0,
        )
    } else {
        // 多个不同英文标题说明前三位代码不足以唯一确定跨区域版本。
        (release_name.clone(), 95.0)
    };
    Ok(Some(GbaIdentification {
        internal_title,
        game_code,
        release_name,
        scrape_name,
        confidence,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_no_intro_suffixes() {
        assert_eq!(
            clean_release_name("Rockman Zero 4 (Japan)"),
            "Rockman Zero 4"
        );
    }

    #[test]
    fn embedded_database_contains_known_games() {
        let index = serial_index();
        assert!(index.by_serial.contains_key("B4ZJ"));
        assert!(index.by_serial.contains_key("BZMJ"));
    }

    #[test]
    fn detects_numbered_installments_in_arabic_and_roman_forms() {
        assert!(has_numbered_installment("CT Special Forces 2"));
        assert!(has_numbered_installment("Final Fantasy VI"));
        assert!(!has_numbered_installment(
            "CT Special Forces - Back to Hell"
        ));
        assert!(!has_numbered_installment("F1 2002"));
    }

    #[test]
    fn keeps_ct_special_forces_2_number_during_regional_mapping() {
        assert_eq!(
            choose_regional_scrape_name(
                "CT Special Forces 2 - Back in the Trenches",
                "CT Special Forces - Back to Hell".to_string(),
            ),
            "CT Special Forces 2 - Back in the Trenches"
        );
    }
}
