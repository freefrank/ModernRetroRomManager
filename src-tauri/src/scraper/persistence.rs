//! Scraper 数据持久化 - 处理元数据写入和媒体文件下载

use std::path::{Path, PathBuf};
use std::fs;
use crate::config::{get_media_dir, get_temp_dir_for_library};
use crate::scraper::{GameMetadata, MediaAsset, MediaType};
use crate::scraper::pegasus::{PegasusGame, PegasusExportOptions, write_pegasus_file};
use crate::rom_service::RomInfo;

// ... existing code ...

/// 将元数据写入 gamelist.xml (EmulationStation 格式)
pub fn save_metadata_emulationstation(
    rom: &RomInfo,
    metadata: &GameMetadata,
    is_temp: bool,
) -> Result<(), String> {
    let gamelist_path = if is_temp {
        // rom.directory 是 ROM 所在目录，library_path 是其父目录
        let rom_dir = Path::new(&rom.directory);
        let library_path = rom_dir.parent().unwrap_or(rom_dir);
        let temp_sys_dir = get_temp_dir_for_library(library_path, &rom.system);
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

/// 将元数据写入 gamelist.xml (EmulationStation 格式)

/// 将抓取到的媒体资产下载到本地
/// 如果 is_temp 为 true，则保存到程序目录下的 temp/media
pub async fn download_media(
    rom: &RomInfo,
    selected_assets: &[MediaAsset],
    is_temp: bool,
) -> Result<Vec<(MediaType, PathBuf)>, String> {
    let client = reqwest::Client::new();
    let mut downloaded = Vec::new();

    // 确定下载目录: {base_dir}/media/{file_stem}/
    let file_stem = Path::new(&rom.file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&rom.name);

    let base_dir = if is_temp {
        // rom.directory 是 ROM 所在目录 (如 Z:\ps3)
        // library_path 应该是其父目录 (如 Z:\)
        let rom_dir = Path::new(&rom.directory);
        let library_path = rom_dir.parent().unwrap_or(rom_dir);
        get_temp_dir_for_library(library_path, &rom.system)
    } else {
        get_media_dir().join(&rom.system)
    };

    let target_dir = base_dir.join("media").join(file_stem);

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
/// 
/// 使用统一的 pegasus 模块进行文件写入，支持合并模式
pub fn save_metadata_pegasus(
    rom: &RomInfo,
    metadata: &GameMetadata,
    is_temp: bool,
) -> Result<(), String> {
    let metadata_path = if is_temp {
        // rom.directory 是 ROM 所在目录，library_path 是其父目录
        let rom_dir = Path::new(&rom.directory);
        let library_path = rom_dir.parent().unwrap_or(rom_dir);
        let temp_sys_dir = get_temp_dir_for_library(library_path, &rom.system);
        fs::create_dir_all(&temp_sys_dir).map_err(|e| e.to_string())?;
        temp_sys_dir.join("metadata.txt")
    } else {
        Path::new(&rom.directory).join("metadata.txt")
    };

    // 转换为 PegasusGame
    let mut game = PegasusGame {
        name: metadata.name.clone(),
        file: Some(rom.file.clone()),
        developer: metadata.developer.clone(),
        publisher: metadata.publisher.clone(),
        genre: metadata.genres.first().cloned(),
        players: metadata.players.clone(),
        description: metadata.description.clone(),
        release: metadata.release_date.clone(),
        rating: metadata.rating.map(|r| format!("{}%", (r * 100.0) as i32)),
        ..Default::default()
    };
    
    // 添加英文名到 extra 字段
    if let Some(ref en) = metadata.english_name {
        game.extra.insert("x-english-name".to_string(), en.clone());
    }

    // 导出选项：包含 collection header（仅当文件不存在时）
    let options = PegasusExportOptions {
        include_collection: !metadata_path.exists(),
        collection_name: Some(rom.system.clone()),
        ..Default::default()
    };

    // 使用 merge 模式写入，更新已存在的游戏或追加新游戏
    write_pegasus_file(&metadata_path, &[game], &options, true)
}
