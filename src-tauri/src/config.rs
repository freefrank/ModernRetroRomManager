use std::path::PathBuf;
use std::sync::OnceLock;

static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

/// 获取可执行文件所在目录
fn get_exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// 获取配置目录路径
/// 优先级: 环境变量 CONFIG_DIR > exe 所在目录 ./config/
pub fn get_config_dir() -> PathBuf {
    CONFIG_DIR
        .get_or_init(|| {
            // 1. 检查环境变量
            if let Ok(env_path) = std::env::var("CONFIG_DIR") {
                return PathBuf::from(env_path);
            }

            // 2. 使用 exe 所在目录下的 config/
            get_exe_dir().join("config")
        })
        .clone()
}

/// 获取媒体资产目录路径
pub fn get_media_dir() -> PathBuf {
    get_config_dir().join("media")
}

/// 获取临时数据目录路径
pub fn get_temp_dir() -> PathBuf {
    get_config_dir().join("temp")
}

/// 将路径规范化为合法的目录名
/// 例如: z:\ -> z, d:\games\ -> d_games
fn normalize_path_to_dirname(path: &std::path::Path) -> String {
    let path_str = path.to_string_lossy();

    // 移除驱动器冒号和路径分隔符，用下划线替换
    path_str
        .replace(':', "")
        .replace('\\', "_")
        .replace('/', "_")
        .trim_matches('_')
        .to_string()
}

/// 获取特定ROM库和系统的临时目录路径
/// 例如: get_temp_dir_for_library("z:\", "gba") -> config/temp/z/gba/
pub fn get_temp_dir_for_library(library_path: &std::path::Path, system: &str) -> PathBuf {
    let normalized_library = normalize_path_to_dirname(library_path);
    get_temp_dir().join(normalized_library).join(system)
}

/// 获取设置文件路径
pub fn get_settings_path() -> PathBuf {
    get_config_dir().join("settings.json")
}

/// 获取数据目录 (用于存放 scraper 数据库等)
pub fn get_data_dir() -> PathBuf {
    get_config_dir().join("data")
}

/// 确保配置目录结构存在
pub fn ensure_config_dirs() -> Result<(), std::io::Error> {
    std::fs::create_dir_all(get_media_dir())?;
    std::fs::create_dir_all(get_temp_dir())?;
    std::fs::create_dir_all(get_data_dir())?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_paths() {
        let config_dir = get_config_dir();
        assert!(config_dir.ends_with("config"));
    }
}

