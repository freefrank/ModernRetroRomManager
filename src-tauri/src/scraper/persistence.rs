//! Scraper 数据持久化 - 处理元数据写入和媒体文件下载

use std::path::{Path, PathBuf};
use std::fs;
use crate::config::{get_media_dir, get_temp_dir};
use crate::scraper::{GameMetadata, MediaAsset, MediaType};
use crate::rom_service::RomInfo;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::Cursor;

// ... existing code ...

/// 将元数据写入 gamelist.xml (EmulationStation 格式)
pub fn save_metadata_emulationstation(
    rom: &RomInfo,
    metadata: &GameMetadata,
    is_temp: bool,
) -> Result<(), String> {
    let gamelist_path = if is_temp {
        let temp_sys_dir = get_temp_dir().join(&rom.system);
        fs::create_dir_all(&temp_sys_dir).map_err(|e| e.to_string())?;
        temp_sys_dir.join("gamelist.xml")
    } else {
        Path::new(&rom.directory).join("gamelist.xml")
    };

    let content = if gamelist_path.exists() {
        fs::read_to_string(&gamelist_path).map_err(|e| e.to_string())?
    } else {
        r#"<?xml version="1.0"?>
<gameList>
</gameList>"#.to_string()
    };

    // 重构策略：使用 quick-xml 反序列化 -> 修改 -> 序列化
    // 这样虽然会重置格式，但最稳健。
    use quick_xml::de::from_str;
    use quick_xml::se::to_string;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename = "gameList")]
    struct EsGameList {
        #[serde(rename = "game", default)]
        games: Vec<EsGame>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct EsGame {
        path: String,
        name: Option<String>,
        desc: Option<String>,
        image: Option<String>,
        developer: Option<String>,
        publisher: Option<String>,
        genre: Option<String>,
        players: Option<String>,
        releasedate: Option<String>,
        rating: Option<f32>,
        #[serde(rename = "english-name")]
        english_name: Option<String>,
    }

    let mut list: EsGameList = from_str(&content).map_err(|e| format!("XML Parse error: {}", e))?;
    let mut found = false;

    for game in &mut list.games {
        // 匹配逻辑：path 结尾匹配文件名
        if game.path.ends_with(&rom.file) || game.path == rom.file {
            game.name = Some(metadata.name.clone());
            if let Some(desc) = &metadata.description { game.desc = Some(desc.clone()); }
            if let Some(dev) = &metadata.developer { game.developer = Some(dev.clone()); }
            if let Some(pub_) = &metadata.publisher { game.publisher = Some(pub_.clone()); }
            if !metadata.genres.is_empty() { game.genre = Some(metadata.genres[0].clone()); }
            if let Some(en) = &metadata.english_name { game.english_name = Some(en.clone()); }
            found = true;
            break;
        }
    }

    if !found {
        list.games.push(EsGame {
            path: format!("./{}", rom.file),
            name: Some(metadata.name.clone()),
            desc: metadata.description.clone(),
            image: None,
            developer: metadata.developer.clone(),
            publisher: metadata.publisher.clone(),
            genre: metadata.genres.first().cloned(),
            players: metadata.players.clone(),
            releasedate: metadata.release_date.clone(),
            rating: metadata.rating.map(|r| r as f32),
            english_name: metadata.english_name.clone(),
        });
    }

    // 序列化回写
    let new_xml = to_string(&list).map_err(|e| e.to_string())?;
    // quick-xml 默认没有 xml header
    let final_xml = format!("<?xml version=\"1.0\"?>\n{}", new_xml);
    
    fs::write(gamelist_path, final_xml).map_err(|e| e.to_string())?;

    Ok(())
}

/// 将抓取到的媒体资产下载到本地
/// 如果 is_temp 为 true，则保存到程序目录下的 temp/media
pub async fn download_media(
    rom: &RomInfo,
    selected_assets: &[MediaAsset],
    is_temp: bool,
) -> Result<Vec<(MediaType, PathBuf)>, String> {
    let client = reqwest::Client::new();
    let mut downloaded = Vec::new();

    // 确定下载目录: {base_dir}/{system}/{file_stem}/
    let file_stem = Path::new(&rom.file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&rom.name);
    
    let base_dir = if is_temp {
        get_temp_dir().join("media")
    } else {
        get_media_dir()
    };

    let target_dir = base_dir
        .join(&rom.system)
        .join(file_stem);

    fs::create_dir_all(&target_dir).map_err(|e| format!("无法创建媒体目录: {}", e))?;

    for asset in selected_assets {
        let extension = asset.url.split('.').last().unwrap_or("png");
        let filename = format!("{}.{}", asset.asset_type.as_str(), extension);
        let save_path = target_dir.join(filename);

        let resp = client.get(&asset.url).send().await.map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
            fs::write(&save_path, bytes).map_err(|e| e.to_string())?;
            downloaded.push((asset.asset_type, save_path));
        }
    }

    Ok(downloaded)
}

/// 将元数据写入 metadata.txt (Pegasus 格式)
/// 如果 is_temp 为 true，则写入程序目录下的 temp/{system}/metadata.txt
pub fn save_metadata_pegasus(
    rom: &RomInfo,
    metadata: &GameMetadata,
    is_temp: bool,
) -> Result<(), String> {
    let metadata_path = if is_temp {
        let temp_sys_dir = get_temp_dir().join(&rom.system);
        fs::create_dir_all(&temp_sys_dir).map_err(|e| e.to_string())?;
        temp_sys_dir.join("metadata.txt")
    } else {
        Path::new(&rom.directory).join("metadata.txt")
    };

    // 简单实现：由于 Pegasus 格式是文本追加，我们先读取全部内容
    let mut content = if metadata_path.exists() {
        fs::read_to_string(&metadata_path).map_err(|e| e.to_string())?
    } else {
        format!("collection: {}\n", rom.system)
    };

    // 查找是否已存在该文件
    let file_marker = format!("file: {}", rom.file);
    
    let mut game_entry = String::new();
    game_entry.push_str(&format!("\ngame: {}\n", metadata.name));
    game_entry.push_str(&format!("file: {}\n", rom.file));
    if let Some(ref en) = metadata.english_name { game_entry.push_str(&format!("x-english-name: {}\n", en)); }
    if let Some(ref d) = metadata.description { game_entry.push_str(&format!("description: {}\n", d.replace('\n', " "))); }
    if let Some(ref d) = metadata.developer { game_entry.push_str(&format!("developer: {}\n", d)); }
    if let Some(ref p) = metadata.publisher { game_entry.push_str(&format!("publisher: {}\n", p)); }
    if !metadata.genres.is_empty() { game_entry.push_str(&format!("genres: {}\n", metadata.genres.join(", "))); }
    if let Some(ref r) = metadata.release_date { game_entry.push_str(&format!("release: {}\n", r)); }
    if let Some(ref p) = metadata.players { game_entry.push_str(&format!("players: {}\n", p)); }
    if let Some(ref r) = metadata.rating { game_entry.push_str(&format!("rating: {}%\n", (r * 100.0) as i32)); }

    // 极简处理：如果包含文件名则替换
    if content.contains(&file_marker) {
        let lines: Vec<&str> = content.lines().collect();
        let mut new_lines = Vec::new();
        let mut in_target_block = false;

        // 首先找到包含 file_marker 的 block 的起始索引
        let mut target_start_idx = None;
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("game:") {
                let mut j = i + 1;
                while j < lines.len() && !lines[j].starts_with("game:") {
                    if lines[j].trim() == file_marker {
                        target_start_idx = Some(i);
                        break;
                    }
                    j += 1;
                }
            }
            if target_start_idx.is_some() { break; }
        }

        if let Some(start_idx) = target_start_idx {
            for (i, line) in lines.iter().enumerate() {
                if i == start_idx {
                    in_target_block = true;
                    new_lines.push(game_entry.trim());
                    continue;
                }
                if in_target_block && line.starts_with("game:") {
                    in_target_block = false;
                }
                if !in_target_block {
                    new_lines.push(line);
                }
            }
            content = new_lines.join("\n");
        } else {
            content.push_str(&game_entry);
        }
    } else {
        content.push_str(&game_entry);
    }

    fs::write(metadata_path, content).map_err(|e| e.to_string())?;
    Ok(())
}
