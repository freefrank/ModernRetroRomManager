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
/// 优先级: 已存在的 exe 同目录 ./config/ > 环境变量 CONFIG_DIR > 用户 AppData/config
pub fn get_config_dir() -> PathBuf {
    CONFIG_DIR
        .get_or_init(|| {
            select_config_dir(
                &get_exe_dir(),
                std::env::var_os("CONFIG_DIR").map(PathBuf::from),
                &user_config_base(),
            )
        })
        .clone()
}

fn select_config_dir(
    exe_dir: &std::path::Path,
    override_dir: Option<PathBuf>,
    user_base: &std::path::Path,
) -> PathBuf {
    let portable = exe_dir.join("config");
    if portable.is_dir() {
        portable
    } else if let Some(path) = override_dir {
        path
    } else {
        user_base.join("ModernRetroRomManager").join("config")
    }
}

#[cfg(not(test))]
fn user_config_base() -> PathBuf {
    #[cfg(target_os = "windows")]
    if let Some(path) = std::env::var_os("APPDATA") {
        return PathBuf::from(path);
    }
    #[cfg(target_os = "macos")]
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join("Library")
            .join("Application Support");
    }
    if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(path);
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config"))
        .unwrap_or_else(|| get_exe_dir())
}

#[cfg(test)]
fn user_config_base() -> PathBuf {
    // 单元测试可能通过 settings::update_setting 写配置，必须与真实用户 AppData 隔离。
    std::env::temp_dir().join(format!("mrrm-tests-{}", std::process::id()))
}

/// 单文件发行版首次启动时释放的只读内置资源目录。
pub fn get_embedded_resource_dir() -> PathBuf {
    get_data_dir()
        .join("bundled-resources")
        .join(env!("CARGO_PKG_VERSION"))
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
        .replace(['\\', '/'], "_")
        .trim_matches('_')
        .to_string()
}

/// 获取特定ROM库和系统的临时目录路径
/// 例如: get_temp_dir_for_library("z:\", "gba") -> config/temp/z/gba/
pub fn get_temp_dir_for_library(library_path: &std::path::Path, system: &str) -> PathBuf {
    let normalized_library = normalize_path_to_dirname(library_path);
    get_temp_dir().join(normalized_library).join(system)
}

pub fn get_cache_dir_for_library(library_path: &std::path::Path, system: &str) -> PathBuf {
    let normalized_library = normalize_path_to_dirname(library_path);
    get_config_dir()
        .join("cache")
        .join(normalized_library)
        .join(system)
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
#[allow(dead_code)]
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
        assert!(config_dir.starts_with(std::env::temp_dir()));
    }

    #[test]
    fn sidecar_config_wins_and_appdata_is_the_fallback() {
        let root = std::env::temp_dir().join(format!("mrrm-config-{}", std::process::id()));
        let exe = root.join("app");
        let appdata = root.join("appdata");
        let override_dir = root.join("override");
        std::fs::create_dir_all(&exe).unwrap();

        assert_eq!(
            select_config_dir(&exe, None, &appdata),
            appdata.join("ModernRetroRomManager").join("config")
        );
        assert_eq!(
            select_config_dir(&exe, Some(override_dir.clone()), &appdata),
            override_dir
        );

        std::fs::create_dir_all(exe.join("config")).unwrap();
        assert_eq!(
            select_config_dir(&exe, Some(root.join("ignored")), &appdata),
            exe.join("config")
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
