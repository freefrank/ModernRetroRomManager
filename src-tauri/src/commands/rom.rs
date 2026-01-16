use crate::db::{get_connection, models::{Rom, RomMetadata, MediaAsset}, schema::{roms, rom_metadata, media_assets}};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RomInfo {
    pub id: String,
    pub filename: String,
    pub path: String,
    pub system_id: String,
    pub size: i64,
    pub crc32: Option<String>,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub metadata: Option<RomMetadataInfo>,
    pub media: Vec<MediaAssetInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RomMetadataInfo {
    pub name: String,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<Vec<String>>,
    pub players: Option<i32>,
    pub rating: Option<f64>,
    pub region: Option<String>,
    pub scraper_source: Option<String>,
    pub scraped_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaAssetInfo {
    pub id: String,
    pub asset_type: String,
    pub path: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl From<RomMetadata> for RomMetadataInfo {
    fn from(m: RomMetadata) -> Self {
        Self {
            name: m.name,
            description: m.description,
            release_date: m.release_date,
            developer: m.developer,
            publisher: m.publisher,
            genre: m.genre.and_then(|g| serde_json::from_str(&g).ok()),
            players: m.players,
            rating: m.rating,
            region: m.region,
            scraper_source: m.scraper_source,
            scraped_at: m.scraped_at,
        }
    }
}

impl From<MediaAsset> for MediaAssetInfo {
    fn from(m: MediaAsset) -> Self {
        Self {
            id: m.id,
            asset_type: m.asset_type,
            path: m.path,
            width: m.width,
            height: m.height,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RomFilter {
    pub system_id: Option<String>,
    pub has_metadata: Option<bool>,
    pub search_query: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RomStats {
    pub total_roms: i64,
    pub scraped_roms: i64,
    pub total_size: i64,
}

/// 获取 ROM 列表
#[tauri::command]
pub fn get_roms(filter: Option<RomFilter>) -> Result<Vec<RomInfo>, String> {
    let mut conn = get_connection().map_err(|e| e.to_string())?;
    let mut pattern = String::new();

    let mut query = roms::table.into_boxed();

    if let Some(ref f) = filter {
        if let Some(ref system_id) = f.system_id {
            query = query.filter(roms::system_id.eq(system_id));
        }
        if let Some(ref search) = f.search_query {
            pattern = format!("%{}%", search);
            query = query.filter(roms::filename.like(&pattern));
        }
    }

    let rom_list: Vec<Rom> = query
        .order(roms::filename.asc())
        .load(&mut conn)
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();

    for rom in rom_list {
        let metadata = rom_metadata::table
            .filter(rom_metadata::rom_id.eq(&rom.id))
            .first::<RomMetadata>(&mut conn)
            .optional()
            .map_err(|e| e.to_string())?
            .map(RomMetadataInfo::from);

        let media = media_assets::table
            .filter(media_assets::rom_id.eq(&rom.id))
            .load::<MediaAsset>(&mut conn)
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(MediaAssetInfo::from)
            .collect();

        result.push(RomInfo {
            id: rom.id,
            filename: rom.filename,
            path: rom.path,
            system_id: rom.system_id,
            size: rom.size,
            crc32: rom.crc32,
            md5: rom.md5,
            sha1: rom.sha1,
            created_at: rom.created_at,
            updated_at: rom.updated_at,
            metadata,
            media,
        });
    }

    Ok(result)
}

/// 获取单个 ROM 详情
#[tauri::command]
pub fn get_rom(id: String) -> Result<Option<RomInfo>, String> {
    let mut conn = get_connection().map_err(|e| e.to_string())?;

    let rom = roms::table
        .filter(roms::id.eq(&id))
        .first::<Rom>(&mut conn)
        .optional()
        .map_err(|e| e.to_string())?;

    match rom {
        Some(rom) => {
            let metadata = rom_metadata::table
                .filter(rom_metadata::rom_id.eq(&rom.id))
                .first::<RomMetadata>(&mut conn)
                .optional()
                .map_err(|e| e.to_string())?
                .map(RomMetadataInfo::from);

            let media = media_assets::table
                .filter(media_assets::rom_id.eq(&rom.id))
                .load::<MediaAsset>(&mut conn)
                .map_err(|e| e.to_string())?
                .into_iter()
                .map(MediaAssetInfo::from)
                .collect();

            Ok(Some(RomInfo {
                id: rom.id,
                filename: rom.filename,
                path: rom.path,
                system_id: rom.system_id,
                size: rom.size,
                crc32: rom.crc32,
                md5: rom.md5,
                sha1: rom.sha1,
                created_at: rom.created_at,
                updated_at: rom.updated_at,
                metadata,
                media,
            }))
        }
        None => Ok(None),
    }
}

/// 获取 ROM 统计信息
#[tauri::command]
pub fn get_rom_stats() -> Result<RomStats, String> {
    let mut conn = get_connection().map_err(|e| e.to_string())?;

    let total_roms: i64 = roms::table
        .count()
        .get_result(&mut conn)
        .map_err(|e| e.to_string())?;

    let scraped_roms: i64 = rom_metadata::table
        .count()
        .get_result(&mut conn)
        .map_err(|e| e.to_string())?;

    let total_size: i64 = roms::table
        .select(roms::size)
        .load::<i64>(&mut conn)
        .map_err(|e| e.to_string())?
        .iter()
        .sum();

    Ok(RomStats {
        total_roms,
        scraped_roms,
        total_size,
    })
}
