use std::path::Path;
use crate::db::models::System;

/// 根据文件扩展名匹配系统
pub fn detect_system(path: &Path, systems: &[System]) -> Option<String> {
    let extension = path.extension()?.to_str()?.to_lowercase();
    let ext_with_dot = format!(".{}", extension);

    // 优先匹配文件名中包含系统名的（简单的启发式，后续可优化）
    // 目前只做扩展名匹配
    
    // 遍历系统列表
    for system in systems {
        if let Ok(exts) = serde_json::from_str::<Vec<String>>(&system.extensions) {
            if exts.contains(&ext_with_dot) {
                return Some(system.id.clone());
            }
        }
    }

    None
}
