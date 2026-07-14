use std::fs;

include!(concat!(env!("OUT_DIR"), "/embedded_resources.rs"));

pub fn ensure_embedded_resources() -> Result<(), String> {
    let root = crate::config::get_embedded_resource_dir();
    let marker = root.join(".complete");
    if marker.is_file() {
        return Ok(());
    }
    for (relative, bytes) in EMBEDDED_RESOURCES {
        let destination = root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("创建内置资源目录失败: {error}"))?;
        }
        fs::write(&destination, bytes)
            .map_err(|error| format!("释放内置资源 {} 失败: {error}", destination.display()))?;
    }
    fs::write(marker, b"ok").map_err(|error| format!("写入内置资源标记失败: {error}"))?;
    Ok(())
}
