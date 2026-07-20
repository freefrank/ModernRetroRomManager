//! 轻量文件日志:发行版没有控制台,导出/单向同步等疑难需要落盘日志排查。
//! 写入 `<config>/logs/mrrm.log`,追加、带时间戳、超限自动从头开始;best-effort,失败静默。

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

static LOG_LOCK: Mutex<()> = Mutex::new(());
const MAX_LOG_BYTES: u64 = 5 * 1024 * 1024;

/// 日志目录:`<config>/logs`。
pub fn log_dir() -> PathBuf {
    crate::config::get_config_dir().join("logs")
}

/// 日志文件完整路径。
pub fn log_path() -> PathBuf {
    log_dir().join("mrrm.log")
}

/// 追加一行带时间戳的日志。任何 IO 错误都静默忽略,绝不影响主流程。
pub fn log(message: &str) {
    let _guard = LOG_LOCK.lock();
    let dir = log_dir();
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = dir.join("mrrm.log");
    // 超过上限则清空重来,避免无限增长。
    if fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0) > MAX_LOG_BYTES {
        let _ = fs::write(&path, b"");
    }
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "[{timestamp}] {message}");
    }
}

/// `log(format!(...))` 的便捷宏。
#[macro_export]
macro_rules! applog {
    ($($arg:tt)*) => {
        $crate::logging::log(&format!($($arg)*))
    };
}
