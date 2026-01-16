pub mod hash;
pub mod detect;

use std::path::Path;
use walkdir::WalkDir;
use tauri::{AppHandle, Emitter};
use crate::db::{get_connection, models::{Rom, System, ScanDirectory}, schema::{roms, scan_directories, systems}};
use diesel::prelude::*;
use uuid::Uuid;
use chrono::Local;

#[derive(Clone, serde::Serialize)]
struct ScanProgress {
    current: usize,
    total: Option<usize>, // WalkDir 不容易预知总数，或者是扫描到的有效 ROM 数
    message: String,
    finished: bool,
}

pub async fn scan_directory(app: AppHandle, dir_id: String) -> Result<(), String> {
    let mut conn = get_connection().map_err(|e| e.to_string())?;

    // 1. 获取目录配置
    let scan_dir = scan_directories::table
        .filter(scan_directories::id.eq(&dir_id))
        .first::<ScanDirectory>(&mut conn)
        .map_err(|_| "Directory not found")?;

    let path = Path::new(&scan_dir.path);
    if !path.exists() {
        return Err("Directory path does not exist".to_string());
    }

    // 2. 获取所有系统信息用于匹配
    let all_systems = systems::table
        .load::<System>(&mut conn)
        .map_err(|e| e.to_string())?;

    // 3. 遍历文件
    let walker = WalkDir::new(path).into_iter();
    
    let mut count = 0;

    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }

        let file_path = entry.path();
        
        // 发送进度事件
        let _ = app.emit("scan-progress", ScanProgress {
            current: count,
            total: None,
            message: format!("Scanning: {:?}", file_path.file_name().unwrap_or_default()),
            finished: false,
        });

        // 4. 检测系统
        if let Some(system_id) = detect::detect_system(file_path, &all_systems) {
            // 5. 计算 Hash (耗时操作，如果文件很大)
            // 简单优化：如果文件已经存在且大小/时间没变，跳过 Hash？
            // 目前 MVP 版本先全量计算，或后续优化
            
            // 检查数据库是否已存在该路径
            // 这里需要一个新的连接或者复用？Diesel 连接不是线程安全的，但在 async 任务里要注意
            // 由于 scan_directory 是 async 的，我们最好不要长时间持有同步锁。
            // 这里为了简单，先同步执行。
            
            // 计算 Hash
            if let Ok(hashes) = hash::calculate_hashes(file_path) {
                let metadata = std::fs::metadata(file_path).map_err(|e| e.to_string())?;
                let size = metadata.len() as i64;
                
                let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
                let file_path_str = file_path.to_string_lossy().to_string();

                // 插入或更新 ROM
                // 使用 hash 检查是否已存在？或者 path？
                // 通常以 path 为准，如果 path 存在则更新
                
                let new_rom = Rom {
                    id: Uuid::new_v4().to_string(), // 如果存在，这里会被覆盖吗？不，我们需要先查询
                    filename: file_name,
                    path: file_path_str.clone(),
                    system_id,
                    size,
                    crc32: Some(hashes.crc32),
                    md5: Some(hashes.md5),
                    sha1: Some(hashes.sha1),
                    created_at: Local::now().naive_local().to_string(),
                    updated_at: Local::now().naive_local().to_string(),
                };

                // Upsert logic (Sqlite 支持 ON CONFLICT)
                // 但 Diesel 对 SQLite 的 Upsert 支持需要特定写法
                // 这里简单处理：先查询 Path 是否存在
                
                let existing: Option<Rom> = roms::table
                    .filter(roms::path.eq(&file_path_str))
                    .first::<Rom>(&mut conn)
                    .optional()
                    .unwrap_or(None);

                if let Some(existing_rom) = existing {
                    // Update
                    diesel::update(roms::table.filter(roms::id.eq(existing_rom.id)))
                        .set((
                            roms::size.eq(new_rom.size),
                            roms::crc32.eq(new_rom.crc32),
                            roms::md5.eq(new_rom.md5),
                            roms::sha1.eq(new_rom.sha1),
                            roms::updated_at.eq(new_rom.updated_at),
                        ))
                        .execute(&mut conn)
                        .map_err(|e| e.to_string())?;
                } else {
                    // Insert
                    diesel::insert_into(roms::table)
                        .values(&new_rom)
                        .execute(&mut conn)
                        .map_err(|e| e.to_string())?;
                }
                
                count += 1;
            }
        }
    }

    // 更新目录最后扫描时间
    diesel::update(scan_directories::table.filter(scan_directories::id.eq(&dir_id)))
        .set(scan_directories::last_scan.eq(Local::now().naive_local().to_string()))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    // 完成事件
    let _ = app.emit("scan-progress", ScanProgress {
        current: count,
        total: Some(count),
        message: "Scan completed".to_string(),
        finished: true,
    });

    Ok(())
}
