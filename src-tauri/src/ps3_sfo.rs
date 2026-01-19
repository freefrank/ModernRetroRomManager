//! PS3 PARAM.SFO 文件解析器
//!
//! PARAM.SFO 是 PS3 游戏的元数据文件，包含游戏标题、ID等信息

use std::fs::File;
use std::io::{Read as StdRead, Seek, SeekFrom};
use std::path::Path;
use iso9660_simple::ISO9660;

/// 文件包装器，实现 iso9660_simple 的 Read trait
struct FileReader {
    file: File,
}

impl FileReader {
    fn new(file: File) -> Self {
        Self { file }
    }
}

impl iso9660_simple::io::Read for FileReader {
    fn read(&mut self, position: usize, buffer: &mut [u8]) -> Option<()> {
        self.file.seek(SeekFrom::Start(position as u64)).ok()?;
        self.file.read_exact(buffer).ok()?;
        Some(())
    }
}

/// SFO 数据类型
#[derive(Debug, Clone, Copy, PartialEq)]
enum SfoDataType {
    Utf8 = 0x0004,
    Utf8Special = 0x0204,
    Int32 = 0x0404,
}

impl SfoDataType {
    fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0004 => Some(SfoDataType::Utf8),
            0x0204 => Some(SfoDataType::Utf8Special),
            0x0404 => Some(SfoDataType::Int32),
            _ => None,
        }
    }
}

/// SFO 文件头
#[derive(Debug)]
struct SfoHeader {
    magic: u32,
    version: u32,
    key_table_offset: u32,
    data_table_offset: u32,
    entry_count: u32,
}

/// SFO 索引条目
#[derive(Debug)]
struct SfoIndexEntry {
    key_offset: u16,
    data_type: SfoDataType,
    data_length: u32,
    data_max_length: u32,
    data_offset: u32,
}

/// PS3 游戏信息
#[derive(Debug, Clone, Default)]
pub struct Ps3GameInfo {
    pub title: Option<String>,
    pub title_id: Option<String>,
    pub version: Option<String>,
    pub app_ver: Option<String>,
    pub category: Option<String>,
}

/// 读取小端序 u16
fn read_u16_le(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

/// 读取小端序 u32
fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

/// 解析 PARAM.SFO 文件
pub fn parse_param_sfo(path: &Path) -> Result<Ps3GameInfo, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open SFO file: {}", e))?;

    // 读取整个文件到内存
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read SFO file: {}", e))?;

    // 使用字节数组解析函数
    parse_param_sfo_from_bytes(&buffer)
}

/// 解析文件头
fn parse_header(buffer: &[u8]) -> Result<SfoHeader, String> {
    Ok(SfoHeader {
        magic: read_u32_le(buffer, 0),
        version: read_u32_le(buffer, 4),
        key_table_offset: read_u32_le(buffer, 8),
        data_table_offset: read_u32_le(buffer, 12),
        entry_count: read_u32_le(buffer, 16),
    })
}

/// 解析索引条目
fn parse_entries(buffer: &[u8], header: &SfoHeader) -> Result<Vec<SfoIndexEntry>, String> {
    let mut entries = Vec::new();
    let index_table_offset = 20; // 文件头后立即是索引表

    for i in 0..header.entry_count {
        let offset = index_table_offset + (i * 16) as usize;

        if offset + 16 > buffer.len() {
            return Err("Index table out of bounds".to_string());
        }

        let key_offset = read_u16_le(buffer, offset);
        let data_type_raw = read_u16_le(buffer, offset + 2);
        let data_length = read_u32_le(buffer, offset + 4);
        let data_max_length = read_u32_le(buffer, offset + 8);
        let data_offset = read_u32_le(buffer, offset + 12);

        let data_type = SfoDataType::from_u16(data_type_raw)
            .ok_or_else(|| format!("Unknown data type: 0x{:04x}", data_type_raw))?;

        entries.push(SfoIndexEntry {
            key_offset,
            data_type,
            data_length,
            data_max_length,
            data_offset,
        });
    }

    Ok(entries)
}

/// 提取游戏信息
fn extract_game_info(
    buffer: &[u8],
    header: &SfoHeader,
    entries: &[SfoIndexEntry],
) -> Result<Ps3GameInfo, String> {
    let mut info = Ps3GameInfo::default();

    for entry in entries {
        // 读取键名
        let key_start = (header.key_table_offset as usize) + (entry.key_offset as usize);
        let key = read_null_terminated_string(buffer, key_start)?;

        // 读取数据值
        let data_start = (header.data_table_offset as usize) + (entry.data_offset as usize);

        match entry.data_type {
            SfoDataType::Utf8 | SfoDataType::Utf8Special => {
                let value = read_string(buffer, data_start, entry.data_length as usize)?;

                match key.as_str() {
                    "TITLE" => info.title = Some(value),
                    "TITLE_ID" => info.title_id = Some(value),
                    "VERSION" => info.version = Some(value),
                    "APP_VER" => info.app_ver = Some(value),
                    "CATEGORY" => info.category = Some(value),
                    _ => {}
                }
            }
            SfoDataType::Int32 => {
                // 暂时不处理整数类型
            }
        }
    }

    Ok(info)
}

/// 读取以null结尾的字符串
fn read_null_terminated_string(buffer: &[u8], start: usize) -> Result<String, String> {
    let mut end = start;
    while end < buffer.len() && buffer[end] != 0 {
        end += 1;
    }

    if end >= buffer.len() {
        return Err("String not null-terminated".to_string());
    }

    String::from_utf8(buffer[start..end].to_vec())
        .map_err(|e| format!("Invalid UTF-8: {}", e))
}

/// 读取固定长度的字符串（可能包含null填充）
fn read_string(buffer: &[u8], start: usize, length: usize) -> Result<String, String> {
    if start + length > buffer.len() {
        return Err("String data out of bounds".to_string());
    }

    // 找到第一个null字节或到达长度限制
    let mut end = start;
    while end < start + length && buffer[end] != 0 {
        end += 1;
    }

    String::from_utf8(buffer[start..end].to_vec())
        .map_err(|e| format!("Invalid UTF-8: {}", e))
}

/// 从字节数组解析 PARAM.SFO
fn parse_param_sfo_from_bytes(buffer: &[u8]) -> Result<Ps3GameInfo, String> {
    if buffer.len() < 20 {
        return Err("SFO data too small".to_string());
    }

    // 解析文件头
    let header = parse_header(buffer)?;

    // 验证魔数
    if header.magic != 0x46535000 {
        return Err("Invalid SFO magic number".to_string());
    }

    // 解析所有条目
    let entries = parse_entries(buffer, &header)?;

    // 提取游戏信息
    extract_game_info(buffer, &header, &entries)
}

/// 从 ISO 文件中提取并解析 PARAM.SFO
pub fn parse_param_sfo_from_iso(iso_path: &Path) -> Result<Ps3GameInfo, String> {
    let file = File::open(iso_path)
        .map_err(|e| format!("Failed to open ISO file: {}", e))?;

    let reader = FileReader::new(file);
    let mut iso = ISO9660::from_device(reader)
        .ok_or_else(|| "Failed to parse ISO9660 filesystem".to_string())?;

    // 查找并读取 PS3_GAME/PARAM.SFO 文件
    let sfo_data = find_and_read_sfo(&mut iso)
        .map_err(|e| format!("Failed to find PARAM.SFO in ISO: {}", e))?;

    // 解析 SFO 数据
    parse_param_sfo_from_bytes(&sfo_data)
}

/// 在 ISO 文件系统中查找并读取 PARAM.SFO
fn find_and_read_sfo(iso: &mut ISO9660) -> Result<Vec<u8>, String> {
    // 读取根目录
    let root_entries: Vec<_> = iso.read_root().collect();

    // 查找 PS3_GAME 目录（不区分大小写）
    let ps3_game_entry = root_entries.iter()
        .find(|entry| entry.name.to_uppercase() == "PS3_GAME" && entry.is_folder())
        .ok_or_else(|| "PS3_GAME directory not found".to_string())?;

    // 读取 PS3_GAME 目录内容
    let ps3_game_lba = ps3_game_entry.lsb_position() as usize;
    let ps3_game_entries: Vec<_> = iso.read_directory(ps3_game_lba).collect();

    // 查找 PARAM.SFO 文件（不区分大小写）
    let param_sfo_entry = ps3_game_entries.iter()
        .find(|entry| entry.name.to_uppercase() == "PARAM.SFO" && entry.is_file())
        .ok_or_else(|| "PARAM.SFO file not found".to_string())?;

    // 读取 PARAM.SFO 文件内容
    let file_size = param_sfo_entry.file_size() as usize;
    let mut sfo_data = vec![0u8; file_size];

    iso.read_file(param_sfo_entry, 0, &mut sfo_data)
        .ok_or_else(|| "Failed to read PARAM.SFO file".to_string())?;

    Ok(sfo_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u16_le() {
        let data = [0x34, 0x12];
        assert_eq!(read_u16_le(&data, 0), 0x1234);
    }

    #[test]
    fn test_read_u32_le() {
        let data = [0x78, 0x56, 0x34, 0x12];
        assert_eq!(read_u32_le(&data, 0), 0x12345678);
    }
}
