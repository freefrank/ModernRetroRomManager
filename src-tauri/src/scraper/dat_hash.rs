use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

const GB_DAT: &str =
    include_str!("../../resources/rom-name-cn/Dats/Nintendo - Game Boy (20250316-082450).dat");
const GBC_DAT: &str = include_str!(
    "../../resources/rom-name-cn/Dats/Nintendo - Game Boy Color (20250314-124712).dat"
);
const GG_DAT: &str =
    include_str!("../../resources/rom-name-cn/Dats/Sega - Game Gear (20241203-185356).dat");

#[derive(Debug, Clone, PartialEq)]
pub struct DatIdentification {
    pub crc32: String,
    pub release_name: String,
    pub scrape_name: String,
    pub confidence: f32,
}

static GB_INDEX: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
static GBC_INDEX: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
static GG_INDEX: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();

fn clean_release_name(name: &str) -> String {
    name.find(" (")
        .map_or(name, |position| &name[..position])
        .trim()
        .to_string()
}

fn build_index(data: &str) -> HashMap<String, Vec<String>> {
    let game_re = regex::Regex::new(r#"(?s)<game name="([^"]+)"[^>]*>(.*?)</game>"#).unwrap();
    let crc_re = regex::Regex::new(r#"crc="([0-9a-fA-F]{8})""#).unwrap();
    let mut values: HashMap<String, HashSet<String>> = HashMap::new();
    for game in game_re.captures_iter(data) {
        let name = game[1].replace("&amp;", "&").replace("&quot;", "\"");
        for crc in crc_re.captures_iter(&game[2]) {
            values
                .entry(crc[1].to_ascii_lowercase())
                .or_default()
                .insert(name.clone());
        }
    }
    values
        .into_iter()
        .map(|(crc, names)| (crc, names.into_iter().collect()))
        .collect()
}

fn index(system: &str) -> Option<&'static HashMap<String, Vec<String>>> {
    match system {
        "GB" => Some(GB_INDEX.get_or_init(|| build_index(GB_DAT))),
        "GBC" => Some(GBC_INDEX.get_or_init(|| build_index(GBC_DAT))),
        "GG" => Some(GG_INDEX.get_or_init(|| build_index(GG_DAT))),
        _ => None,
    }
}

fn rom_crc32(path: &Path, extensions: &[&str]) -> Result<Option<u32>, String> {
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("zip"))
    {
        let file = File::open(path).map_err(|error| error.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|error| error.to_string())?;
        for index in 0..archive.len() {
            let entry = archive.by_index(index).map_err(|error| error.to_string())?;
            let name = entry.name().to_ascii_lowercase();
            if extensions.iter().any(|extension| name.ends_with(extension)) {
                return Ok(Some(entry.crc32()));
            }
        }
        return Ok(None);
    }
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = crc32fast::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let length = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if length == 0 {
            break;
        }
        hasher.update(&buffer[..length]);
    }
    Ok(Some(hasher.finalize()))
}

pub fn identify_dat_rom(system: &str, path: &Path) -> Result<Option<DatIdentification>, String> {
    let extensions: &[&str] = match system {
        "GB" => &[".gb"],
        "GBC" => &[".gbc"],
        "GG" => &[".gg"],
        _ => return Ok(None),
    };
    let Some(crc) = rom_crc32(path, extensions)? else {
        return Ok(None);
    };
    let crc32 = format!("{crc:08x}");
    let Some(names) = index(system).and_then(|values| values.get(&crc32)) else {
        return Ok(None);
    };
    let clean_names: HashSet<_> = names.iter().map(|name| clean_release_name(name)).collect();
    if clean_names.len() != 1 {
        return Ok(None);
    }
    Ok(Some(DatIdentification {
        crc32,
        release_name: names[0].clone(),
        scrape_name: clean_names.into_iter().next().unwrap(),
        confidence: 99.0,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_dat_indexes_are_available() {
        assert!(index("GB").unwrap().len() > 1_000);
        assert!(index("GBC").unwrap().len() > 1_000);
        assert!(index("GG").unwrap().len() > 300);
    }
}
