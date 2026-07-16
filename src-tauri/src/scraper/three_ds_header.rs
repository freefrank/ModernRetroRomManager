//! 从 CIA/CCI/3DS 文件头读取 Title ID，并映射为标准发行名称。

use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::OnceLock;

const TITLE_CSV: &str = "3ds-title-id.csv";

#[derive(Debug, Clone, PartialEq)]
pub struct ThreeDsIdentification {
    pub title_id: String,
    pub content_type: String,
    pub release_name: String,
    pub scrape_name: String,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
struct TitleEntry {
    title_id: String,
    serial: String,
    name: String,
    region: String,
}

static TITLE_INDEX: OnceLock<Vec<TitleEntry>> = OnceLock::new();

fn title_index() -> &'static [TitleEntry] {
    TITLE_INDEX.get_or_init(|| {
        let data = crate::embedded_resources::read_database(TITLE_CSV).unwrap_or_default();
        let mut reader = csv::Reader::from_reader(data.as_slice());
        reader
            .records()
            .flatten()
            .map(|record| TitleEntry {
                title_id: record.get(0).unwrap_or_default().to_ascii_uppercase(),
                serial: record.get(1).unwrap_or_default().to_ascii_uppercase(),
                name: record.get(2).unwrap_or_default().to_string(),
                region: record.get(3).unwrap_or_default().to_ascii_uppercase(),
            })
            .collect()
    })
}

fn align64(value: usize) -> Option<usize> {
    value.checked_add(63).map(|value| value & !63)
}

fn signature_block_size(signature_type: u32) -> Option<usize> {
    match signature_type {
        0x0001_0000 | 0x0001_0003 => Some(0x240),
        0x0001_0001 | 0x0001_0004 => Some(0x140),
        0x0001_0002 | 0x0001_0005 => Some(0x80),
        _ => None,
    }
}

fn parse_cia_title_id(data: &[u8]) -> Option<String> {
    if data.len() < 0x20 {
        return None;
    }
    let read_u32_le = |offset: usize| -> Option<u32> {
        Some(u32::from_le_bytes(
            data.get(offset..offset + 4)?.try_into().ok()?,
        ))
    };
    let header_size = usize::try_from(read_u32_le(0)?).ok()?;
    let cert_size = usize::try_from(read_u32_le(8)?).ok()?;
    let ticket_size = usize::try_from(read_u32_le(12)?).ok()?;
    let ticket_offset = align64(header_size)?.checked_add(align64(cert_size)?)?;
    if ticket_size < 0xe4 || data.len() < ticket_offset + 4 {
        return None;
    }
    let signature_type =
        u32::from_be_bytes(data[ticket_offset..ticket_offset + 4].try_into().ok()?);
    let body_offset = ticket_offset.checked_add(signature_block_size(signature_type)?)?;
    let title_id = data.get(body_offset + 0x9c..body_offset + 0xa4)?;
    Some(hex::encode_upper(title_id))
}

fn read_cci_title_id(file: &mut File) -> Result<Option<String>, String> {
    let mut ncsd_header = [0_u8; 0x200];
    file.read_exact(&mut ncsd_header)
        .map_err(|error| error.to_string())?;
    if &ncsd_header[0x100..0x104] != b"NCSD" {
        return Ok(None);
    }
    let partition_units = u32::from_le_bytes(ncsd_header[0x120..0x124].try_into().unwrap());
    let partition_offset = u64::from(partition_units) * 0x200;
    file.seek(SeekFrom::Start(partition_offset))
        .map_err(|error| error.to_string())?;
    let mut ncch_header = [0_u8; 0x200];
    file.read_exact(&mut ncch_header)
        .map_err(|error| error.to_string())?;
    if &ncch_header[0x100..0x104] != b"NCCH" {
        return Ok(None);
    }
    let program_id = u64::from_le_bytes(ncch_header[0x118..0x120].try_into().unwrap());
    Ok(Some(format!("{program_id:016X}")))
}

fn normalize_base_title_id(title_id: &str) -> Option<(String, &'static str)> {
    if title_id.len() != 16 {
        return None;
    }
    match &title_id[..8] {
        "00040000" => Some((title_id.to_string(), "本体")),
        "0004000E" => Some((format!("00040000{}", &title_id[8..]), "更新")),
        "0004008C" => Some((format!("00040000{}", &title_id[8..]), "DLC")),
        _ => Some((title_id.to_string(), "其他")),
    }
}

fn serial_family(serial: &str) -> Option<&str> {
    let code = serial.strip_prefix("CTR-")?;
    (code.len() >= 4).then_some(&code[..code.len() - 1])
}

fn identify_title(title_id: String) -> Option<ThreeDsIdentification> {
    let (base_title_id, content_type) = normalize_base_title_id(&title_id)?;
    let direct: Vec<_> = title_index()
        .iter()
        .filter(|entry| entry.title_id == base_title_id)
        .collect();
    let mut names = HashSet::new();
    let release_names: Vec<_> = direct
        .iter()
        .map(|entry| entry.name.clone())
        .filter(|name| names.insert(name.clone()))
        .collect();
    if release_names.len() != 1 {
        return None;
    }
    let release_name = release_names[0].clone();
    let family = direct.iter().find_map(|entry| serial_family(&entry.serial));
    let mut candidates = Vec::new();
    if let Some(family) = family {
        for preferred_region in ["USA", "EUR"] {
            for entry in title_index() {
                if entry.region == preferred_region
                    && serial_family(&entry.serial) == Some(family)
                    && !candidates.contains(&entry.name)
                {
                    candidates.push(entry.name.clone());
                }
            }
            if !candidates.is_empty() {
                break;
            }
        }
    }
    let scrape_name = if candidates.len() == 1 {
        candidates.remove(0)
    } else {
        release_name.clone()
    };
    Some(ThreeDsIdentification {
        title_id,
        content_type: content_type.to_string(),
        release_name,
        scrape_name,
        confidence: 99.0,
    })
}

pub fn identify_3ds_rom(path: &Path) -> Result<Option<ThreeDsIdentification>, String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let title_id = if extension.eq_ignore_ascii_case("cia") {
        let mut data = Vec::new();
        file.by_ref()
            .take(0x10000)
            .read_to_end(&mut data)
            .map_err(|error| error.to_string())?;
        parse_cia_title_id(&data)
    } else if extension.eq_ignore_ascii_case("3ds") || extension.eq_ignore_ascii_case("cci") {
        read_cci_title_id(&mut file)?
    } else {
        None
    };
    Ok(title_id.and_then(identify_title))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_update_and_dlc_ids() {
        assert_eq!(
            normalize_base_title_id("0004000E001C1800"),
            Some(("00040000001C1800".into(), "更新"))
        );
        assert_eq!(
            normalize_base_title_id("0004008C001C1800"),
            Some(("00040000001C1800".into(), "DLC"))
        );
    }

    #[test]
    fn embedded_database_is_not_empty() {
        assert!(title_index().len() > 1_500);
    }

    #[test]
    fn parses_cia_ticket_and_identifies_dlc() {
        let mut data = vec![0_u8; 0x4000];
        data[0..4].copy_from_slice(&0x2020_u32.to_le_bytes());
        data[8..12].copy_from_slice(&0x0a00_u32.to_le_bytes());
        data[12..16].copy_from_slice(&0x0350_u32.to_le_bytes());
        let ticket_offset = 0x2a40;
        data[ticket_offset..ticket_offset + 4].copy_from_slice(&0x0001_0004_u32.to_be_bytes());
        data[ticket_offset + 0x140 + 0x9c..ticket_offset + 0x140 + 0xa4].copy_from_slice(
            &u64::from_str_radix("0004008C001C1800", 16)
                .unwrap()
                .to_be_bytes(),
        );

        let title_id = parse_cia_title_id(&data).unwrap();
        assert_eq!(title_id, "0004008C001C1800");
        let identification = identify_title(title_id).unwrap();
        assert_eq!(identification.content_type, "DLC");
        assert_eq!(identification.release_name, "The Snack World: TreJarers");
    }
}
