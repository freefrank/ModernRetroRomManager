use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

const GB_DAT: &str = "rom-name-cn/Dats/Nintendo - Game Boy (20250316-082450).dat";
const GBC_DAT: &str = "rom-name-cn/Dats/Nintendo - Game Boy Color (20250314-124712).dat";
const GG_DAT: &str = "rom-name-cn/Dats/Sega - Game Gear (20241203-185356).dat";

/// 平台(canonical 文件夹名的大写形式)对应的内置 No-Intro DAT 相对路径。
/// 变体平台(hack/HD/MSU1 等)映射到同一份基础 DAT;
/// 一个平台可对应多个 DAT(如 NES 的 Headered/Headerless),名称合并去重。
fn nointro_dat_paths(system: &str) -> &'static [&'static str] {
    match system {
        // Nintendo 系列
        "3DS" => &[
            "rom-name-cn/Dats/Nintendo - Nintendo 3DS (Decrypted) (20250502-021715).dat",
            "rom-name-cn/Dats/Nintendo - Nintendo 3DS (Encrypted) (20250502-021715).dat",
        ],
        "FC" | "FC HACK" | "FC-HD" => &[
            "rom-name-cn/Dats/Nintendo - Nintendo Entertainment System (Headered) (20250317-022847).dat",
            "rom-name-cn/Dats/Nintendo - Nintendo Entertainment System (Headerless) (20250317-022847).dat",
        ],
        // Sufami Turbo 小卡带运行于 SFC,并入 SFC 名单
        "SFC" | "SFC HACK" | "SFC-MSU1" => &[
            "rom-name-cn/Dats/Nintendo - Super Nintendo Entertainment System (20250318-222534).dat",
            "rom-name-cn/Dats/Nintendo - Sufami Turbo (20240622-035607).dat",
        ],
        "GB" => &[GB_DAT],
        "GBC" => &[GBC_DAT],
        "GBA" => &["rom-name-cn/Dats/Nintendo - Game Boy Advance (20250313-191605).dat"],
        "N64" => &[
            "rom-name-cn/Dats/Nintendo - Nintendo 64 (BigEndian) (20250208-050759).dat",
            "rom-name-cn/Dats/Nintendo - Nintendo 64 (ByteSwapped) (20250208-050759).dat",
        ],
        "NDS" => &["rom-name-cn/Dats/Nintendo - Nintendo DS (Decrypted) (20250304-133524).dat"],
        "WII" | "WII WARE" => {
            &["rom-name-cn/Dats/Nintendo - Wii - Datfile (3776) (2025-05-17 10-15-59).dat"]
        }
        "VIRTUAL BOY" => &["rom-name-cn/Dats/Nintendo - Virtual Boy (20240829-133848).dat"],
        "GAME WATCH" => &["rom-name-cn/Dats/Nintendo - Game & Watch (20241105-120946).dat"],
        "POKE MINI" => &["rom-name-cn/Dats/Nintendo - Pokemon Mini (20250407-153358).dat"],
        // Sega 系列
        "MD" | "MD HACK" | "MD HACK(PICODRIVE)" => {
            &["rom-name-cn/Dats/Sega - Mega Drive - Genesis (20250305-122836).dat"]
        }
        "MD-32X" => &["rom-name-cn/Dats/Sega - 32X (20250126-142831).dat"],
        "GG" => &[GG_DAT],
        // Sony 系列
        "PS1" | "PS1 HACK" => {
            &["rom-name-cn/Dats/Sony - PlayStation - Datfile (10850) (2025-03-29 23-07-14).dat"]
        }
        "PSP" => &[
            "rom-name-cn/Dats/Sony - PlayStation Portable - Datfile (3131) (2025-03-29 15-23-59).dat",
        ],
        // Bandai 系列
        "WS" => &["rom-name-cn/Dats/Bandai - WonderSwan (20241208-053316).dat"],
        "WSC" => &["rom-name-cn/Dats/Bandai - WonderSwan Color (20250117-025245).dat"],
        // SNK 系列:NGPC 向下兼容 NGP,合并两份 DAT
        "NGPC" => &[
            "rom-name-cn/Dats/SNK - NeoGeo Pocket (20250222-080429).dat",
            "rom-name-cn/Dats/SNK - NeoGeo Pocket Color (20240506-123728).dat",
        ],
        // NEC 系列:SuperGrafx 卡带通常与 PCE 同目录存放,合并
        "PCE" => &[
            "rom-name-cn/Dats/NEC - PC Engine - TurboGrafx-16 (20250224-150029).dat",
            "rom-name-cn/Dats/NEC - PC Engine SuperGrafx (20250121-134304).dat",
        ],
        // Atari 系列:ATARI 目录泛指 Atari 卡带主机,合并三份 DAT
        "ATARI" => &[
            "rom-name-cn/Dats/Atari - Atari 2600 (20250501-125025).dat",
            "rom-name-cn/Dats/Atari - Atari 5200 (20250406-021055).dat",
            "rom-name-cn/Dats/Atari - Atari 7800 (BIN) (20250416-124715).dat",
        ],
        _ => &[],
    }
}

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

/// 反转义 DAT 中常见的 XML 实体。
fn unescape_xml(value: &str) -> String {
    value
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// 从 DAT 文本中收集基础英文名(跳过 BIOS 条目)。
fn collect_base_names(data: &str, output: &mut BTreeSet<String>) {
    static NAME_RE: OnceLock<regex::Regex> = OnceLock::new();
    let name_re = NAME_RE.get_or_init(|| regex::Regex::new(r#"<game name="([^"]+)""#).unwrap());
    for capture in name_re.captures_iter(data) {
        let name = unescape_xml(&capture[1]);
        if name.starts_with("[BIOS]") {
            continue;
        }
        let base = clean_release_name(&name);
        if !base.is_empty() {
            output.insert(base);
        }
    }
}

/// 基础名清单缓存:以 DAT 路径列表为键,变体平台共享同一份结果。
static BASE_NAME_CACHE: OnceLock<Mutex<HashMap<String, &'static [String]>>> = OnceLock::new();

/// 返回平台 No-Intro DAT 中的全部游戏基础英文名(去重、按字典序)。
/// 基础名 = 去掉第一个 " (" 起的区域/语言/版本标记(保留 No-Intro 逗号冠词格式)。
/// 平台无对应 DAT 时返回空切片。
pub fn nointro_base_names(system: &str) -> &'static [String] {
    let paths = nointro_dat_paths(system);
    if paths.is_empty() {
        return &[];
    }
    let cache = BASE_NAME_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().unwrap();
    let key = paths.join("|");
    if let Some(names) = guard.get(&key).copied() {
        return names;
    }
    let mut unique = BTreeSet::new();
    for path in paths {
        if let Ok(data) = crate::embedded_resources::read_database_text(path) {
            collect_base_names(&data, &mut unique);
        }
    }
    let leaked: &'static [String] =
        Box::leak(unique.into_iter().collect::<Vec<_>>().into_boxed_slice());
    guard.insert(key, leaked);
    leaked
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
    let load = |path| {
        crate::embedded_resources::read_database_text(path)
            .map(|data| build_index(&data))
            .unwrap_or_default()
    };
    match system {
        "GB" => Some(GB_INDEX.get_or_init(|| load(GB_DAT))),
        "GBC" => Some(GBC_INDEX.get_or_init(|| load(GBC_DAT))),
        "GG" => Some(GG_INDEX.get_or_init(|| load(GG_DAT))),
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

    #[test]
    fn nointro_base_names_covers_snes_library() {
        // SNES DAT 原始条目约 4000+,按基础名去重后约 2300+
        let names = nointro_base_names("SFC");
        assert!(names.len() > 2_000);
        assert!(names.iter().any(|name| name == "Lion King, The"));
    }

    #[test]
    fn nointro_base_names_supports_mega_drive() {
        assert!(!nointro_base_names("MD").is_empty());
    }

    #[test]
    fn nointro_base_names_variant_shares_base_dat() {
        assert_eq!(nointro_base_names("SFC HACK"), nointro_base_names("SFC"));
    }

    #[test]
    fn nointro_base_names_unknown_platform_is_empty() {
        assert!(nointro_base_names("UNKNOWN").is_empty());
    }
}
