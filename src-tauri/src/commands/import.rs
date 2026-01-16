use crate::db::{get_connection, models::{Rom, RomMetadata, MediaAsset}, schema::{roms, rom_metadata, media_assets}};
use diesel::prelude::*;
use serde::Deserialize;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Deserialize)]
struct EsGameList {
    #[serde(rename = "game", default)]
    games: Vec<EsGame>,
}

#[derive(Debug, Deserialize)]
struct EsGame {
    path: String,
    name: Option<String>,
    desc: Option<String>,
    image: Option<String>,
    video: Option<String>,
    rating: Option<f32>,
    releasedate: Option<String>,
    developer: Option<String>,
    publisher: Option<String>,
    genre: Option<String>,
    players: Option<String>,
}

#[tauri::command]
pub fn import_gamelist(xml_path: String) -> Result<usize, String> {
    let file = File::open(&xml_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    
    let game_list: EsGameList = quick_xml::de::from_reader(reader)
        .map_err(|e| format!("Failed to parse XML: {}", e))?;

    let mut conn = get_connection().map_err(|e| e.to_string())?;
    let mut imported_count = 0;

    let xml_dir = Path::new(&xml_path).parent().unwrap_or(Path::new(""));

    for game in game_list.games {
        // 1. 解析 Path，尝试匹配数据库中的 ROM
        // ES path 通常是 "./rom.zip" 或相对路径
        let clean_path = game.path.trim_start_matches("./").replace("\\", "/");
        let filename = Path::new(&clean_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        // 查找 ROM (通过文件名匹配)
        // 注意：可能有重名文件在不同系统，这里简单处理，如果能匹配到多个，可能需要更严格的逻辑
        // 理想情况：用户导入时指定了 System，或者我们遍历所有 ROM
        let matched_roms: Vec<Rom> = roms::table
            .filter(roms::filename.eq(filename))
            .load::<Rom>(&mut conn)
            .map_err(|e| e.to_string())?;

        for rom in matched_roms {
            // 2. 插入/更新 Metadata
            let metadata = RomMetadata {
                rom_id: rom.id.clone(),
                name: game.name.clone().unwrap_or(filename.to_string()),
                description: game.desc.clone(),
                release_date: game.releasedate.clone().map(|d| d.chars().take(8).collect()), // YYYYMMDD
                developer: game.developer.clone(),
                publisher: game.publisher.clone(),
                genre: game.genre.clone(),
                players: game.players.clone().and_then(|p| p.split('-').last().and_then(|n| n.parse().ok())), // "1-2" -> 2
                rating: game.rating.map(|r| r as f64),
                region: None,
                scraper_source: Some("import_es".to_string()),
                scraped_at: Some(chrono::Local::now().naive_local().to_string()),
            };

            diesel::replace_into(rom_metadata::table)
                .values(&metadata)
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;

            // 3. 处理媒体路径 (Image/Video)
            // ES xml 中的路径通常是相对 xml 文件的
            let mut assets = Vec::new();
            
            if let Some(img_path) = &game.image {
                let abs_path = resolve_path(xml_dir, img_path);
                if abs_path.exists() {
                    assets.push(("boxfront", abs_path));
                }
            }
            
            if let Some(vid_path) = &game.video {
                let abs_path = resolve_path(xml_dir, vid_path);
                if abs_path.exists() {
                    assets.push(("video", abs_path));
                }
            }

            for (asset_type, path) in assets {
                let new_asset = MediaAsset {
                    id: Uuid::new_v4().to_string(),
                    rom_id: rom.id.clone(),
                    asset_type: asset_type.to_string(),
                    path: path.to_string_lossy().to_string(),
                    width: None, 
                    height: None,
                    file_size: Some(std::fs::metadata(&path).map(|m| m.len() as i64).unwrap_or(0)),
                    source_url: None,
                    downloaded_at: chrono::Local::now().naive_local().to_string(),
                };

                diesel::insert_into(media_assets::table)
                    .values(&new_asset)
                    .execute(&mut conn)
                    .map_err(|e| e.to_string())?;
            }

            imported_count += 1;
        }
    }

    Ok(imported_count)
}

fn resolve_path(base: &Path, relative: &str) -> std::path::PathBuf {
    // 处理 "./images/..." 这种路径
    let rel = relative.trim_start_matches("./").replace("\\", "/");
    base.join(rel)
}
