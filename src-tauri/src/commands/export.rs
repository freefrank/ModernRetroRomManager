//! 导出命令模块
//! 
//! 处理将抓取的元数据和媒体导出到目标前端 (EmulationStation, Pegasus)

use std::path::{Path, PathBuf};
use std::fs;
use tauri::{AppHandle, Emitter};
use crate::config::get_temp_dir;
use crate::rom_service::RomInfo;
use crate::scraper::{
    GameMetadata,
    pegasus::parse_pegasus_file,
    persistence::{save_metadata_emulationstation, save_metadata_pegasus},
};

#[derive(Clone, serde::Serialize, Debug)]
pub struct ExportProgress {
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub finished: bool,
}

/// 导出抓取的数据 (统一入口，自动识别格式)
/// 
/// 1. 自动检测目标目录是否存在 gamelist.xml，存在则导出为 ES 格式
/// 2. 否则默认为 Pegasus 格式 (metadata.txt)
/// 3. 同步媒体文件
#[tauri::command]
pub async fn export_scraped_data(
    app: AppHandle,
    system: String,
    directory: String,
) -> Result<(), String> {
    let temp_metadata_path = get_temp_dir().join(&system).join("metadata.txt");
    if !temp_metadata_path.exists() {
        return Err("No temporary data to export".to_string());
    }

    let app_clone = app.clone();
    
    // 启动异步导出
    tokio::spawn(async move {
        let _ = app_clone.emit("export-progress", ExportProgress {
            current: 0,
            total: 100,
            message: "准备导出元数据...".to_string(),
            finished: false,
        });

        // 1. 读取临时元数据
        if let Ok(pegasus_data) = parse_pegasus_file(&temp_metadata_path) {
            let target_gamelist = Path::new(&directory).join("gamelist.xml");
            
            if target_gamelist.exists() {
                // Target is EmulationStation: Need to convert Pegasus data
                let _ = app_clone.emit("export-progress", ExportProgress {
                    current: 5,
                    total: 100,
                    message: "正在转换格式 (Pegasus -> ES)...".to_string(),
                    finished: false,
                });

                let total_games = pegasus_data.games.len();
                for (idx, game) in pegasus_data.games.iter().enumerate() {
                    // Use filename from game or fallback (should have file)
                    if let Some(file) = &game.file {
                        // Convert to GameMetadata
                        let metadata: GameMetadata = game.clone().into();
                        
                        // Create dummy RomInfo for path resolution in save_metadata_emulationstation
                        let rom = RomInfo {
                            file: file.clone(),
                            directory: directory.clone(),
                            system: system.clone(),
                            name: metadata.name.clone(),
                            ..Default::default()
                        };
                        
                        // Save individually to gamelist.xml (append/update)
                        // 注意：这里频繁读写同一个文件效率较低，但对于增量更新是安全的
                        // 优化方案：批量读取 -> 批量更新 -> 一次写入 (后续迭代)
                        let _ = save_metadata_emulationstation(&rom, &metadata, false);
                        
                        if idx % 10 == 0 {
                             let progress = 5 + ((idx as f32 / total_games as f32) * 15.0) as usize;
                             let _ = app_clone.emit("export-progress", ExportProgress {
                                current: progress,
                                total: 100,
                                message: format!("更新元数据: {}/{}", idx + 1, total_games),
                                finished: false,
                            });
                        }
                    }
                }
            } else {
                // Target is Pegasus (default): Append metadata.txt
                let _ = app_clone.emit("export-progress", ExportProgress {
                    current: 10,
                    total: 100,
                    message: "写入 Pegasus 元数据...".to_string(),
                    finished: false,
                });

                let target_path = Path::new(&directory).join("metadata.txt");
                let content = fs::read_to_string(&temp_metadata_path).unwrap_or_default();
                
                let mut target_content = if target_path.exists() {
                    fs::read_to_string(&target_path).unwrap_or_default()
                } else {
                    String::new()
                };

                target_content.push_str("\n# Exported from ModernRetroRomManager\n");
                target_content.push_str(&content);
                let _ = fs::write(&target_path, target_content);
            }
        }

        // 2. 处理媒体文件导出
        let temp_media_dir = get_temp_dir().join("media").join(&system);
        let target_media_dir = Path::new(&directory).join("media");

        if temp_media_dir.exists() {
            let _ = app_clone.emit("export-progress", ExportProgress {
                current: 20,
                total: 100,
                message: "正在同步媒体资产...".to_string(),
                finished: false,
            });

            // 获取文件列表以计算进度
            let mut files_to_copy: Vec<PathBuf> = Vec::new();
            collect_files(&temp_media_dir, &mut files_to_copy);
            
            let total_files = files_to_copy.len();
            for (i, src_path) in files_to_copy.into_iter().enumerate() {
                let relative = src_path.strip_prefix(&temp_media_dir).unwrap_or(&src_path);
                let dst_path = target_media_dir.join(relative);
                
                if let Some(parent) = dst_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                
                if let Ok(_) = fs::copy(&src_path, &dst_path) {
                    let progress = 20 + ((i + 1) as f32 / total_files as f32 * 75.0) as usize;
                    let _ = app_clone.emit("export-progress", ExportProgress {
                        current: progress,
                        total: 100,
                        message: format!("导出媒体: {} ({}/{})", 
                            src_path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                            i + 1, total_files),
                        finished: false,
                    });
                }
            }
        }

        // 3. 完成
        let _ = app_clone.emit("export-progress", ExportProgress {
            current: 100,
            total: 100,
            message: "导出完成".to_string(),
            finished: true,
        });
    });

    Ok(())
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, files);
            } else {
                files.push(path);
            }
        }
    }
}

// 占位符: 旧接口保留以防前端调用，实际逻辑已合并到 export_scraped_data
#[tauri::command]
pub async fn export_to_emulationstation(_app: AppHandle, _target_dir: String) -> Result<(), String> {
    Err("Please use export_scraped_data instead.".to_string())
}

#[tauri::command]
pub async fn export_to_pegasus(_app: AppHandle, _target_dir: String) -> Result<(), String> {
    Err("Please use export_scraped_data instead.".to_string())
}
