use std::fs;
use std::io::{self, Cursor};
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/embedded_resources.rs"));

pub fn ensure_embedded_resources() -> Result<(), String> {
    let root = crate::config::get_embedded_resource_dir();
    let packed_marker = root.join(".packed-databases");
    if root.is_dir() && !packed_marker.is_file() {
        remove_legacy_database_files(&root)?;
        fs::write(&packed_marker, b"ok")
            .map_err(|error| format!("写入内置数据库迁移标记失败: {error}"))?;
    }
    let marker = root.join(".complete");
    if marker.is_file() {
        return Ok(());
    }
    extract_archive(&root, EMBEDDED_ASSET_ARCHIVE)?;
    fs::write(packed_marker, b"ok")
        .map_err(|error| format!("写入内置数据库迁移标记失败: {error}"))?;
    fs::write(marker, b"ok").map_err(|error| format!("写入内置资源标记失败: {error}"))?;
    Ok(())
}

fn extract_archive(root: &Path, bytes: &[u8]) -> Result<(), String> {
    let reader = Cursor::new(bytes);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|error| format!("读取内置资源压缩包失败: {error}"))?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("读取内置资源条目失败: {error}"))?;
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| format!("内置资源路径不安全: {}", entry.name()))?;
        let destination = root.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&destination)
                .map_err(|error| format!("创建内置资源目录失败: {error}"))?;
            continue;
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("创建内置资源目录失败: {error}"))?;
        }
        let mut output = fs::File::create(&destination)
            .map_err(|error| format!("创建内置资源 {} 失败: {error}", destination.display()))?;
        io::copy(&mut entry, &mut output)
            .map_err(|error| format!("释放内置资源 {} 失败: {error}", destination.display()))?;
    }
    Ok(())
}

pub(crate) fn read_database(relative: &str) -> Result<Vec<u8>, String> {
    let reader = Cursor::new(EMBEDDED_DATABASE_ARCHIVE);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|error| format!("读取内置数据库包失败: {error}"))?;
    let name = relative.replace('\\', "/");
    let mut entry = archive
        .by_name(&name)
        .map_err(|error| format!("读取内置数据库 {name} 失败: {error}"))?;
    let mut bytes = Vec::with_capacity(entry.size() as usize);
    io::copy(&mut entry, &mut bytes)
        .map_err(|error| format!("解压内置数据库 {name} 失败: {error}"))?;
    Ok(bytes)
}

pub(crate) fn read_database_text(relative: &str) -> Result<String, String> {
    String::from_utf8(read_database(relative)?)
        .map_err(|error| format!("内置数据库 {relative} 不是 UTF-8: {error}"))
}

fn remove_legacy_database_files(root: &Path) -> Result<(), String> {
    if !root.is_dir() {
        return Ok(());
    }
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let is_database = entry
            .path()
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("csv") || extension.eq_ignore_ascii_case("dat")
            });
        if is_database {
            fs::remove_file(entry.path()).map_err(|error| {
                format!(
                    "清理旧版内置数据库 {} 失败: {error}",
                    entry.path().display()
                )
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_archives_are_valid_and_non_empty() {
        let databases = zip::ZipArchive::new(Cursor::new(EMBEDDED_DATABASE_ARCHIVE)).unwrap();
        let assets = zip::ZipArchive::new(Cursor::new(EMBEDDED_ASSET_ARCHIVE)).unwrap();
        assert!(!databases.is_empty());
        assert!(!assets.is_empty());
        assert!(!read_database("gba-serial.csv").unwrap().is_empty());
    }
}
