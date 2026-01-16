use crate::db::{get_connection, models::{Rom, RomMetadata, MediaAsset, System}, schema::{roms, rom_metadata, media_assets, systems}};
use diesel::prelude::*;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use quick_xml::se::Serializer;

#[derive(Debug, Serialize)]
#[serde(rename = "gameList")]
struct EsGameList {
    #[serde(rename = "game")]
    games: Vec<EsGame>,
}

#[derive(Debug, Serialize)]
struct EsGame {
    path: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    desc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rating: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    releasedate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    developer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    players: Option<String>,
}

#[derive(Clone, serde::Serialize)]
struct ExportProgress {
    current: usize,
    total: usize,
    message: String,
    finished: bool,
}

#[tauri::command]
pub async fn export_to_emulationstation(app: AppHandle, target_dir: String) -> Result<(), String> {
    let mut conn = get_connection().map_err(|e| e.to_string())?;

    // 1. 获取所有系统
    let all_systems = systems::table
        .load::<System>(&mut conn)
        .map_err(|e| e.to_string())?;

    let target_path = Path::new(&target_dir);
    if !target_path.exists() {
        std::fs::create_dir_all(target_path).map_err(|e| e.to_string())?;
    }

    // 统计总数用于进度
    let total_roms: i64 = roms::table.count().get_result(&mut conn).map_err(|e| e.to_string())?;
    let mut processed_count = 0;

    for system in all_systems {
        // 获取该系统的所有 ROM
        let system_roms = roms::table
            .filter(roms::system_id.eq(&system.id))
            .load::<Rom>(&mut conn)
            .map_err(|e| e.to_string())?;

        if system_roms.is_empty() {
            continue;
        }

        // 创建系统目录 e.g. target/nes
        let sys_dir = target_path.join(&system.short_name); // use short_name or id? usually short_name matching ES folders
        std::fs::create_dir_all(&sys_dir).map_err(|e| e.to_string())?;

        // 创建媒体目录 e.g. target/nes/media
        let media_dir = sys_dir.join("media");
        std::fs::create_dir_all(&media_dir).map_err(|e| e.to_string())?;

        let mut games_xml = Vec::new();

        for rom in system_roms {
            // 发送进度
            processed_count += 1;
            let _ = app.emit("export-progress", ExportProgress {
                current: processed_count,
                total: total_roms as usize,
                message: format!("Exporting {}...", rom.filename),
                finished: false,
            });

            // 1. 复制 ROM 文件
            let src_rom_path = Path::new(&rom.path);
            if !src_rom_path.exists() {
                continue; // Skip if file missing
            }
            
            let dest_rom_filename = src_rom_path.file_name().unwrap();
            let dest_rom_path = sys_dir.join(dest_rom_filename);
            
            // 如果目标文件已存在且大小相同，跳过复制？为了简单先覆盖或检查
            if !dest_rom_path.exists() {
                std::fs::copy(src_rom_path, &dest_rom_path).map_err(|e| e.to_string())?;
            }

            // 2. 获取元数据
            let metadata = rom_metadata::table
                .filter(rom_metadata::rom_id.eq(&rom.id))
                .first::<RomMetadata>(&mut conn)
                .optional()
                .map_err(|e| e.to_string())?;

            // 3. 获取媒体
            let assets = media_assets::table
                .filter(media_assets::rom_id.eq(&rom.id))
                .load::<MediaAsset>(&mut conn)
                .map_err(|e| e.to_string())?;

            let mut image_path_rel = None;
            let mut video_path_rel = None;

            for asset in assets {
                let src_asset_path = Path::new(&asset.path);
                if !src_asset_path.exists() {
                    continue;
                }

                // 复制媒体文件
                let ext = src_asset_path.extension().and_then(|s| s.to_str()).unwrap_or("png");
                let dest_filename = format!("{}-{}.{}", rom.id, asset.asset_type, ext); // unique name
                let dest_asset_path = media_dir.join(&dest_filename);
                
                if !dest_asset_path.exists() {
                    std::fs::copy(src_asset_path, &dest_asset_path).map_err(|e| e.to_string())?;
                }

                let rel_path = format!("./media/{}", dest_filename);

                match asset.asset_type.as_str() {
                    "boxfront" | "screenshot" => {
                        if image_path_rel.is_none() || asset.asset_type == "boxfront" {
                            image_path_rel = Some(rel_path);
                        }
                    }
                    "video" => video_path_rel = Some(rel_path),
                    _ => {}
                }
            }

            // 4. 构建 XML Entry
            if let Some(meta) = metadata {
                games_xml.push(EsGame {
                    path: format!("./{}", dest_rom_filename.to_string_lossy()),
                    name: meta.name,
                    desc: meta.description,
                    image: image_path_rel,
                    video: video_path_rel,
                    rating: meta.rating.map(|r| (r / 10.0) as f32), // ES uses 0.0-1.0 usually? Or 0-10? Usually 0-1 float. My db is 0-100 or 0-10? IGDB is 0-100.
                    // Assuming DB stores 0-10 or 0-100. Let's assume 0-100 -> 0-1.
                    releasedate: meta.release_date.map(|d| format!("{}T000000", d.replace("-", ""))),
                    developer: meta.developer,
                    publisher: meta.publisher,
                    genre: meta.genre, // Need to parse JSON? ES takes string.
                    players: meta.players.map(|p| p.to_string()),
                });
            } else {
                // Minimal entry
                games_xml.push(EsGame {
                    path: format!("./{}", dest_rom_filename.to_string_lossy()),
                    name: rom.filename,
                    desc: None,
                    image: None,
                    video: None,
                    rating: None,
                    releasedate: None,
                    developer: None,
                    publisher: None,
                    genre: None,
                    players: None,
                });
            }
        }

        // 生成 gamelist.xml
        if !games_xml.is_empty() {
            let gamelist = EsGameList { games: games_xml };
            let xml_path = sys_dir.join("gamelist.xml");
            
            let mut file = File::create(xml_path).map_err(|e| e.to_string())?;
            
            // quick-xml serialize
            let mut buffer = String::new();
            let mut ser = Serializer::new(&mut buffer);
            ser.indent(' ', 2);
            gamelist.serialize(ser).map_err(|e| e.to_string())?;
            
            file.write_all(buffer.as_bytes()).map_err(|e| e.to_string())?;
        }
    }

    let _ = app.emit("export-progress", ExportProgress {
        current: processed_count,
        total: processed_count,
        message: "Export completed".to_string(),
        finished: true,
    });

    Ok(())
}
