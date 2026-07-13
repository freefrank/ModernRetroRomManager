use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::OnceLock;

const NDS_DAT: &str = include_str!(
    "../../resources/rom-name-cn/Dats/Nintendo - Nintendo DS (Decrypted) (20250304-133524).dat"
);

#[derive(Debug, Clone, PartialEq)]
pub struct PlatformIdentification {
    pub internal_title: String,
    pub product_code: String,
    pub release_name: String,
    pub scrape_name: String,
    pub confidence: f32,
}

static NDS_SERIALS: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();

fn xml_unescape(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn clean_release_name(name: &str) -> String {
    name.find(" (")
        .map_or(name, |position| &name[..position])
        .trim()
        .to_string()
}

fn nds_serials() -> &'static HashMap<String, Vec<String>> {
    NDS_SERIALS.get_or_init(|| {
        let game_re = regex::Regex::new(r#"(?s)<game name="([^"]+)"[^>]*>(.*?)</game>"#).unwrap();
        let serial_re = regex::Regex::new(r#"serial="([A-Za-z0-9]{4})""#).unwrap();
        let mut values: HashMap<String, HashSet<String>> = HashMap::new();
        for game in game_re.captures_iter(NDS_DAT) {
            let name = xml_unescape(&game[1]);
            for serial in serial_re.captures_iter(&game[2]) {
                values
                    .entry(serial[1].to_ascii_uppercase())
                    .or_default()
                    .insert(name.clone());
            }
        }
        values
            .into_iter()
            .map(|(serial, names)| (serial, names.into_iter().collect()))
            .collect()
    })
}

fn read_nds_prefix(path: &Path) -> Result<Vec<u8>, String> {
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("zip"))
    {
        let file = File::open(path).map_err(|error| error.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|error| error.to_string())?;
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
            if entry.name().to_ascii_lowercase().ends_with(".nds") {
                let mut prefix = vec![0_u8; 0x20];
                entry
                    .read_exact(&mut prefix)
                    .map_err(|error| error.to_string())?;
                return Ok(prefix);
            }
        }
        return Err("ZIP 内没有 NDS ROM".to_string());
    }
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut prefix = vec![0_u8; 0x20];
    file.read_exact(&mut prefix)
        .map_err(|error| error.to_string())?;
    Ok(prefix)
}

pub fn identify_nds_rom(path: &Path) -> Result<Option<PlatformIdentification>, String> {
    let data = read_nds_prefix(path)?;
    let internal_title = String::from_utf8_lossy(&data[..12])
        .trim_matches('\0')
        .trim()
        .to_string();
    let product_code = String::from_utf8_lossy(&data[12..16]).to_ascii_uppercase();
    if !product_code
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric())
    {
        return Ok(None);
    }
    let Some(names) = nds_serials().get(&product_code) else {
        return Ok(None);
    };
    let clean_names: HashSet<_> = names.iter().map(|name| clean_release_name(name)).collect();
    if clean_names.len() != 1 {
        return Ok(None);
    }
    let scrape_name = clean_names.into_iter().next().unwrap();
    Ok(Some(PlatformIdentification {
        internal_title,
        product_code,
        release_name: names[0].clone(),
        scrape_name,
        confidence: 99.0,
    }))
}

pub fn identify_psp_iso(path: &Path) -> Result<Option<PlatformIdentification>, String> {
    if !path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("iso"))
    {
        return Ok(None);
    }
    let sfo = read_iso9660_file(path, "PSP_GAME", "PARAM.SFO")?;
    let info = crate::ps3::sfo::parse_param_sfo_from_bytes(&sfo)?;
    let Some(title) = info.title.filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };
    Ok(Some(PlatformIdentification {
        internal_title: title.clone(),
        product_code: info.title_id.unwrap_or_default(),
        release_name: title.clone(),
        scrape_name: title,
        confidence: 98.0,
    }))
}

pub fn identify_playstation_pbp(path: &Path) -> Result<Option<PlatformIdentification>, String> {
    if !path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("pbp"))
    {
        return Ok(None);
    }
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut header = [0_u8; 16];
    file.read_exact(&mut header)
        .map_err(|error| error.to_string())?;
    if &header[..4] != b"\0PBP" {
        return Ok(None);
    }
    let sfo_start = u32::from_le_bytes(header[8..12].try_into().unwrap()) as u64;
    let sfo_end = u32::from_le_bytes(header[12..16].try_into().unwrap()) as u64;
    if sfo_end <= sfo_start || sfo_end - sfo_start > 4 * 1024 * 1024 {
        return Err("PBP PARAM.SFO 偏移无效".to_string());
    }
    file.seek(SeekFrom::Start(sfo_start))
        .map_err(|error| error.to_string())?;
    let mut sfo = vec![0_u8; (sfo_end - sfo_start) as usize];
    file.read_exact(&mut sfo)
        .map_err(|error| error.to_string())?;
    identification_from_sfo(&sfo, 98.0)
}

fn identification_from_sfo(
    data: &[u8],
    confidence: f32,
) -> Result<Option<PlatformIdentification>, String> {
    let info = crate::ps3::sfo::parse_param_sfo_from_bytes(data)?;
    let Some(title) = info.title.filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };
    Ok(Some(PlatformIdentification {
        internal_title: title.clone(),
        product_code: info.title_id.unwrap_or_default(),
        release_name: title.clone(),
        scrape_name: title,
        confidence,
    }))
}

pub fn identify_sega_disc(
    path: &Path,
    system: &str,
) -> Result<Option<PlatformIdentification>, String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if !["bin", "img", "iso", "cdi"]
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
    {
        return Ok(None);
    }
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut data = Vec::new();
    file.by_ref()
        .take(16 * 1024 * 1024)
        .read_to_end(&mut data)
        .map_err(|error| error.to_string())?;
    let (magic, code_offset, code_length, title_offset, title_length) = match system {
        "SS" => (b"SEGA SEGASATURN".as_slice(), 0x20, 10, 0x60, 112),
        "DC" => (b"SEGA SEGAKATANA".as_slice(), 0x40, 10, 0x80, 128),
        _ => return Ok(None),
    };
    let Some(start) = data
        .windows(magic.len())
        .rposition(|window| window == magic)
    else {
        return Ok(None);
    };
    let end = start + title_offset + title_length;
    if end > data.len() {
        return Ok(None);
    }
    let clean = |bytes: &[u8]| {
        String::from_utf8_lossy(bytes)
            .trim_matches(|c: char| c == '\0' || c.is_whitespace())
            .to_string()
    };
    let product_code = clean(&data[start + code_offset..start + code_offset + code_length]);
    let title = clean(&data[start + title_offset..end]);
    if title.is_empty() {
        return Ok(None);
    }
    Ok(Some(PlatformIdentification {
        internal_title: title.clone(),
        product_code,
        release_name: title.clone(),
        scrape_name: title,
        confidence: 96.0,
    }))
}

fn iso_record(record: &[u8]) -> Option<(u32, u32, bool, String)> {
    if record.len() < 34 || record[0] as usize > record.len() {
        return None;
    }
    let extent = u32::from_le_bytes(record[2..6].try_into().ok()?);
    let size = u32::from_le_bytes(record[10..14].try_into().ok()?);
    let is_directory = record[25] & 0x02 != 0;
    let name_length = record[32] as usize;
    let name = record.get(33..33 + name_length)?;
    Some((
        extent,
        size,
        is_directory,
        String::from_utf8_lossy(name)
            .trim_end_matches(";1")
            .to_ascii_uppercase(),
    ))
}

fn find_iso_entry(data: &[u8], expected: &str) -> Option<(u32, u32, bool)> {
    let mut offset = 0_usize;
    while offset < data.len() {
        let length = data[offset] as usize;
        if length == 0 {
            offset = ((offset / 2048) + 1) * 2048;
            continue;
        }
        let record = data.get(offset..offset + length)?;
        if let Some((extent, size, is_directory, name)) = iso_record(record) {
            if name == expected {
                return Some((extent, size, is_directory));
            }
        }
        offset += length;
    }
    None
}

fn read_extent(file: &mut File, extent: u32, size: u32) -> Result<Vec<u8>, String> {
    file.seek(SeekFrom::Start(extent as u64 * 2048))
        .map_err(|error| error.to_string())?;
    let mut data = vec![0_u8; size as usize];
    file.read_exact(&mut data)
        .map_err(|error| error.to_string())?;
    Ok(data)
}

fn read_iso9660_file(path: &Path, directory: &str, filename: &str) -> Result<Vec<u8>, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    file.seek(SeekFrom::Start(16 * 2048))
        .map_err(|error| error.to_string())?;
    let mut pvd = vec![0_u8; 2048];
    file.read_exact(&mut pvd)
        .map_err(|error| error.to_string())?;
    if pvd.get(1..6) != Some(b"CD001") {
        return Err("不是标准 ISO9660 镜像".to_string());
    }
    let (root_extent, root_size, _, _) = iso_record(&pvd[156..]).ok_or("ISO 根目录无效")?;
    let root = read_extent(&mut file, root_extent, root_size)?;
    let (directory_extent, directory_size, is_directory) =
        find_iso_entry(&root, &directory.to_ascii_uppercase()).ok_or("ISO 内缺少游戏目录")?;
    if !is_directory {
        return Err("ISO 游戏目录类型无效".to_string());
    }
    let directory_data = read_extent(&mut file, directory_extent, directory_size)?;
    let (file_extent, file_size, is_directory) =
        find_iso_entry(&directory_data, &filename.to_ascii_uppercase())
            .ok_or("ISO 内缺少 PARAM.SFO")?;
    if is_directory {
        return Err("PARAM.SFO 类型无效".to_string());
    }
    read_extent(&mut file, file_extent, file_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nds_database_contains_known_serial() {
        assert!(nds_serials().contains_key("BKAJ"));
    }
}
