//! PS3 Boxart 自动生成
//!
//! 从 PS3_GAME 目录或 ISO 文件中提取 PIC1.PNG (背景) 和 ICON0.PNG (图标)
//! 合成为 boxart 图片

use image::{DynamicImage, GenericImageView, ImageBuffer, RgbaImage};
use std::fs::File;
use std::io::{Read as StdRead, Seek, SeekFrom};
use std::path::Path;
use iso9660_simple::ISO9660;

/// Boxart 标准尺寸
const BOXART_WIDTH: u32 = 512;
const BOXART_HEIGHT: u32 = 512;

/// ICON0 在 boxart 中的位置和大小
const ICON_SIZE: u32 = 128;
const ICON_MARGIN: u32 = 16;

/// 生成 PS3 boxart
///
/// # 参数
/// - `ps3_game_dir`: PS3_GAME 目录路径
/// - `output_path`: 输出 boxart 图片路径
///
/// # 返回
/// - `Ok(())`: 成功生成
/// - `Err(String)`: 错误信息
pub fn generate_ps3_boxart(ps3_game_dir: &Path, output_path: &Path) -> Result<(), String> {
    // 加载 PIC1.PNG (背景)
    let pic1_path = ps3_game_dir.join("PIC1.PNG");
    if !pic1_path.exists() {
        return Err("PIC1.PNG not found".to_string());
    }

    let pic1 = image::open(&pic1_path)
        .map_err(|e| format!("Failed to load PIC1.PNG: {}", e))?;

    // 加载 ICON0.PNG (图标)
    let icon0_path = ps3_game_dir.join("ICON0.PNG");
    let icon0 = if icon0_path.exists() {
        Some(
            image::open(&icon0_path)
                .map_err(|e| format!("Failed to load ICON0.PNG: {}", e))?,
        )
    } else {
        None
    };

    // 创建 boxart
    let boxart = composite_boxart(&pic1, icon0.as_ref())?;

    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // 保存 boxart
    boxart
        .save(output_path)
        .map_err(|e| format!("Failed to save boxart: {}", e))?;

    Ok(())
}

/// 合成 boxart 图片
///
/// # 参数
/// - `pic1`: PIC1.PNG 背景图片
/// - `icon0`: ICON0.PNG 图标 (可选)
///
/// # 返回
/// - 合成后的 boxart 图片
fn composite_boxart(pic1: &DynamicImage, icon0: Option<&DynamicImage>) -> Result<DynamicImage, String> {
    // 创建 boxart 画布
    let mut canvas: RgbaImage = ImageBuffer::new(BOXART_WIDTH, BOXART_HEIGHT);

    // 1. 处理背景 (PIC1.PNG) - 居中裁切
    let bg = center_crop(pic1, BOXART_WIDTH, BOXART_HEIGHT);

    // 将背景绘制到画布
    image::imageops::overlay(&mut canvas, &bg, 0, 0);

    // 2. 如果有 ICON0.PNG，绘制到左下角
    if let Some(icon) = icon0 {
        let icon_resized = icon.resize_exact(ICON_SIZE, ICON_SIZE, image::imageops::FilterType::Lanczos3);
        let icon_rgba = icon_resized.to_rgba8();

        // 计算左下角位置
        let x = ICON_MARGIN as i64;
        let y = (BOXART_HEIGHT - ICON_SIZE - ICON_MARGIN) as i64;

        image::imageops::overlay(&mut canvas, &icon_rgba, x, y);
    }

    Ok(DynamicImage::ImageRgba8(canvas))
}

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

/// 从 ISO 文件中提取 PIC1.PNG 和 ICON0.PNG
fn extract_images_from_iso(iso_path: &Path) -> Result<(Vec<u8>, Option<Vec<u8>>), String> {
    let file = File::open(iso_path)
        .map_err(|e| format!("Failed to open ISO file: {}", e))?;

    let reader = FileReader::new(file);
    let mut iso = ISO9660::from_device(reader)
        .ok_or_else(|| "Failed to parse ISO9660 filesystem".to_string())?;

    // 读取根目录
    let root_entries: Vec<_> = iso.read_root().collect();

    // 查找 PS3_GAME 目录
    let ps3_game_entry = root_entries.iter()
        .find(|entry| {
            let name = entry.name.to_uppercase();
            (name == "PS3_GAME" || name.starts_with("PS3_GAME;")) && entry.is_folder()
        })
        .ok_or_else(|| "PS3_GAME directory not found in ISO".to_string())?;

    // 读取 PS3_GAME 目录内容
    let ps3_game_lba = ps3_game_entry.lsb_position() as usize;
    let ps3_game_entries: Vec<_> = iso.read_directory(ps3_game_lba).collect();

    // 查找 PIC1.PNG
    let pic1_entry = ps3_game_entries.iter()
        .find(|entry| {
            let name = entry.name.to_uppercase();
            (name == "PIC1.PNG" || name.starts_with("PIC1.PNG;")) && entry.is_file()
        })
        .ok_or_else(|| "PIC1.PNG not found in ISO".to_string())?;

    // 读取 PIC1.PNG 内容
    let pic1_size = pic1_entry.file_size() as usize;
    let mut pic1_data = vec![0u8; pic1_size];
    iso.read_file(pic1_entry, 0, &mut pic1_data)
        .ok_or_else(|| "Failed to read PIC1.PNG from ISO".to_string())?;

    // 查找 ICON0.PNG (可选)
    let icon0_data = ps3_game_entries.iter()
        .find(|entry| {
            let name = entry.name.to_uppercase();
            (name == "ICON0.PNG" || name.starts_with("ICON0.PNG;")) && entry.is_file()
        })
        .and_then(|icon0_entry| {
            let icon0_size = icon0_entry.file_size() as usize;
            let mut icon0_data = vec![0u8; icon0_size];
            iso.read_file(icon0_entry, 0, &mut icon0_data)?;
            Some(icon0_data)
        });

    Ok((pic1_data, icon0_data))
}

/// 从 ISO 文件生成 PS3 boxart
pub fn generate_ps3_boxart_from_iso(iso_path: &Path, output_path: &Path) -> Result<(), String> {
    // 从 ISO 中提取图片
    let (pic1_data, icon0_data) = extract_images_from_iso(iso_path)?;

    // 加载图片
    let pic1 = image::load_from_memory(&pic1_data)
        .map_err(|e| format!("Failed to load PIC1.PNG: {}", e))?;

    let icon0 = icon0_data.and_then(|data| {
        image::load_from_memory(&data).ok()
    });

    // 合成 boxart
    let boxart = composite_boxart(&pic1, icon0.as_ref())?;

    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // 保存 boxart
    boxart
        .save(output_path)
        .map_err(|e| format!("Failed to save boxart: {}", e))?;

    Ok(())
}


/// 居中裁切图片
///
/// # 参数
/// - `img`: 原始图片
/// - `target_width`: 目标宽度
/// - `target_height`: 目标高度
///
/// # 返回
/// - 裁切后的图片
fn center_crop(img: &DynamicImage, target_width: u32, target_height: u32) -> RgbaImage {
    let (img_width, img_height) = img.dimensions();

    // 计算缩放比例，确保图片完全覆盖目标区域
    let scale_x = target_width as f32 / img_width as f32;
    let scale_y = target_height as f32 / img_height as f32;
    let scale = scale_x.max(scale_y);

    // 缩放图片
    let scaled_width = (img_width as f32 * scale) as u32;
    let scaled_height = (img_height as f32 * scale) as u32;
    let scaled = img.resize_exact(scaled_width, scaled_height, image::imageops::FilterType::Lanczos3);

    // 计算裁切起始位置（居中）
    let crop_x = (scaled_width - target_width) / 2;
    let crop_y = (scaled_height - target_height) / 2;

    // 裁切
    let cropped = image::imageops::crop_imm(&scaled, crop_x, crop_y, target_width, target_height);

    cropped.to_image()
}

/// 从 PS3_GAME 目录提取 ICON0.PNG 作为 logo
pub fn extract_ps3_logo(ps3_game_dir: &Path, output_path: &Path) -> Result<(), String> {
    let icon0_path = ps3_game_dir.join("ICON0.PNG");
    if !icon0_path.exists() {
        return Err("ICON0.PNG not found".to_string());
    }

    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // 直接复制 ICON0.PNG 作为 logo
    std::fs::copy(&icon0_path, output_path)
        .map_err(|e| format!("Failed to copy ICON0.PNG: {}", e))?;

    Ok(())
}

/// 从 ISO 文件中提取 ICON0.PNG 作为 logo
pub fn extract_ps3_logo_from_iso(iso_path: &Path, output_path: &Path) -> Result<(), String> {
    // 从 ISO 中提取图片
    let (_pic1_data, icon0_data) = extract_images_from_iso(iso_path)?;

    let icon0_data = icon0_data.ok_or_else(|| "ICON0.PNG not found in ISO".to_string())?;

    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // 保存 ICON0.PNG 作为 logo
    std::fs::write(output_path, &icon0_data)
        .map_err(|e| format!("Failed to save logo: {}", e))?;

    Ok(())
}
