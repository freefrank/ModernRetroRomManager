//! ROM 文件 Hash 计算 - 用于 ScreenScraper 等支持 Hash 精确匹配的源
//!
//! 单次流式读取同时计算 CRC32 / MD5 / SHA1，避免多次读盘。

use std::fs::File;
use std::io::Read;
use std::path::Path;

use sha1::Digest;

use crate::scraper::RomHash;

/// 默认 Hash 计算的文件大小上限 (256 MiB)
///
/// 超过上限的文件（如 PS2/PS3 镜像）跳过 Hash，回退到名称搜索，
/// 避免批量抓取时长时间读盘。
pub const DEFAULT_HASH_SIZE_LIMIT: u64 = 256 * 1024 * 1024;

/// 计算 ROM 文件的 CRC32 / MD5 / SHA1
///
/// 文件不存在、读取失败或大小超过 `max_size` 时返回 None。
pub fn compute_rom_hash(path: &Path, max_size: u64) -> Option<RomHash> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > max_size {
        return None;
    }

    let mut file = File::open(path).ok()?;
    let mut crc = crc32fast::Hasher::new();
    let mut md5_ctx = md5::Context::new();
    let mut sha1_ctx = sha1::Sha1::new();

    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        crc.update(&buf[..n]);
        md5_ctx.consume(&buf[..n]);
        sha1_ctx.update(&buf[..n]);
    }

    Some(RomHash {
        crc32: Some(format!("{:08X}", crc.finalize())),
        md5: Some(format!("{:x}", md5_ctx.finalize())),
        sha1: Some(hex::encode(sha1_ctx.finalize())),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_rom_hash() {
        let dir = std::env::temp_dir().join("mrrm_hash_test");
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("test.bin");
        std::fs::write(&path, b"hello world").expect("write test file");

        let hash = compute_rom_hash(&path, DEFAULT_HASH_SIZE_LIMIT).expect("hash computed");
        // 已知 "hello world" 的标准摘要
        assert_eq!(hash.crc32.as_deref(), Some("0D4A1185"));
        assert_eq!(hash.md5.as_deref(), Some("5eb63bbbe01eeed093cb22bb8f5acdc3"));
        assert_eq!(
            hash.sha1.as_deref(),
            Some("2aae6c35c94fcfb415dbe95f408b9ce91ee846ed")
        );

        // 超过大小上限返回 None
        assert!(compute_rom_hash(&path, 5).is_none());
    }
}
