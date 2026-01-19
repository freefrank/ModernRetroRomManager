use crate::rom_service::{get_all_roms, SystemRoms};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RomFilter {
    pub system: Option<String>,
    pub search_query: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RomStats {
    pub total_roms: usize,
    pub total_systems: usize,
}

/// 获取 ROM 列表 (按系统分组或扁平化)
#[tauri::command]
pub async fn get_roms(filter: Option<RomFilter>) -> Result<Vec<SystemRoms>, String> {
    // 在后台线程中执行IO密集型操作，避免阻塞UI
    let all_systems = tokio::task::spawn_blocking(|| {
        get_all_roms()
    })
    .await
    .map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
    
    if let Some(f) = filter {
        let mut filtered_systems = Vec::new();
        
        for system_roms in all_systems {
            // 系统过滤
            if let Some(sys) = &f.system {
                if &system_roms.system != sys {
                    continue;
                }
            }
            
            // 搜索过滤
            let roms = if let Some(query) = &f.search_query {
                let lower_query = query.to_lowercase();
                system_roms.roms.into_iter()
                    .filter(|r| r.name.to_lowercase().contains(&lower_query))
                    .collect()
            } else {
                system_roms.roms
            };
            
            if !roms.is_empty() {
                filtered_systems.push(SystemRoms {
                    system: system_roms.system,
                    path: system_roms.path,
                    roms,
                });
            }
        }
        
        Ok(filtered_systems)
    } else {
        Ok(all_systems)
    }
}

/// 获取 ROM 统计信息
#[tauri::command]
pub async fn get_rom_stats() -> Result<RomStats, String> {
    // 在后台线程中执行IO密集型操作，避免阻塞UI
    let all_systems = tokio::task::spawn_blocking(|| {
        get_all_roms()
    })
    .await
    .map_err(|e| format!("Failed to spawn blocking task: {}", e))??;
    
    let total_systems = all_systems.len();
    let total_roms = all_systems.iter().map(|s| s.roms.len()).sum();
    
    Ok(RomStats {
        total_roms,
        total_systems,
    })
}
