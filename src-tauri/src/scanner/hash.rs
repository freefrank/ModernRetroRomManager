use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use sha1::Digest;

/// 计算文件的 CRC32, MD5, SHA1
pub struct FileHashes {
    pub crc32: String,
    pub md5: String,
    pub sha1: String,
}

pub fn calculate_hashes<P: AsRef<Path>>(path: P) -> io::Result<FileHashes> {
    let mut file = File::open(path)?;
    let mut buffer = [0; 8192]; // 8KB buffer

    let mut crc32 = crc32fast::Hasher::new();
    let mut md5 = md5::Context::new();
    let mut sha1 = sha1::Sha1::new();

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let chunk = &buffer[..count];
        crc32.update(chunk);
        md5.consume(chunk);
        sha1.update(chunk);
    }

    Ok(FileHashes {
        crc32: format!("{:08x}", crc32.finalize()),
        md5: format!("{:x}", md5.finalize()),
        sha1: hex::encode(sha1.finalize()),
    })
}
