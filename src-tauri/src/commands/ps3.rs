use crate::config::get_temp_dir_for_library;
use crate::ps3;
use crate::rom_service::update_rom_media_in_metadata;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateBoxartRequest {
    /// ROM 文件名或文件夹名
    pub rom_file: String,
    /// ROM 所在目录
    pub rom_directory: String,
    /// 系统名称
    pub system: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateBoxartResponse {
    /// 生成的 boxart 绝对路径
    pub boxart_path: String,
    /// 生成的 boxart 相对路径（相对于temp目录，用于前端显示）
    pub relative_boxart_path: String,
    /// 生成的 logo 绝对路径
    pub logo_path: Option<String>,
    /// 生成的 logo 相对路径
    pub relative_logo_path: Option<String>,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    pub error: Option<String>,
}

/// 为 PS3 ROM 生成 boxart 和 logo
///
/// 从 PS3_GAME 目录提取 PIC1.PNG 和 ICON0.PNG
/// - boxart: PIC1.PNG 背景 + ICON0.PNG 叠加
/// - logo: ICON0.PNG 直接使用
#[tauri::command]
pub async fn generate_ps3_boxart(request: GenerateBoxartRequest) -> Result<GenerateBoxartResponse, String> {
    // 在后台线程执行，避免阻塞 UI
    tokio::task::spawn_blocking(move || {
        generate_ps3_boxart_impl(request)
    })
    .await
    .map_err(|e| format!("Failed to spawn blocking task: {}", e))?
}

fn generate_ps3_boxart_impl(request: GenerateBoxartRequest) -> Result<GenerateBoxartResponse, String> {
    // 构建 ROM 路径
    let rom_path = Path::new(&request.rom_directory).join(&request.rom_file);

    // 提取文件名主体 (不含扩展名)
    let file_stem = Path::new(&request.rom_file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&request.rom_file);

    // 构建输出路径（使用标准结构: media/{file_stem}/boxfront.png）
    // 使用父目录作为library_path，与rom_service.rs的逻辑保持一致
    let rom_dir = Path::new(&request.rom_directory);
    let library_path = rom_dir.parent().unwrap_or(rom_dir);
    let media_dir = get_temp_dir_for_library(library_path, &request.system)
        .join("media")
        .join(file_stem);

    // 确保media/{file_stem}/目录存在
    if let Err(e) = std::fs::create_dir_all(&media_dir) {
        return Ok(GenerateBoxartResponse {
            boxart_path: String::new(),
            relative_boxart_path: String::new(),
            logo_path: None,
            relative_logo_path: None,
            success: false,
            error: Some(format!("Failed to create media directory: {}", e)),
        });
    }

    let boxart_output_path = media_dir.join("boxfront.png");
    let logo_output_path = media_dir.join("logo.png");

    // 根据 ROM 类型生成 boxart 和 logo
    let result = if rom_path.is_dir() {
        // ROM 是文件夹（PS3 游戏文件夹）
        let ps3_game_dir = rom_path.join("PS3_GAME");

        // 检查 PS3_GAME 目录是否存在
        if !ps3_game_dir.exists() || !ps3_game_dir.is_dir() {
            return Ok(GenerateBoxartResponse {
                boxart_path: String::new(),
                relative_boxart_path: String::new(),
                logo_path: None,
                relative_logo_path: None,
                success: false,
                error: Some("PS3_GAME 目录不存在".to_string()),
            });
        }

        // 生成 boxart
        let boxart_result = ps3::generate_ps3_boxart(&ps3_game_dir, &boxart_output_path);
        
        // 生成 logo (直接复制 ICON0.PNG)
        let logo_result = ps3::extract_ps3_logo(&ps3_game_dir, &logo_output_path);
        
        (boxart_result, logo_result)
    } else if rom_path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()) == Some("iso".to_string()) {
        // ROM 是 ISO 文件，从 ISO 中提取图片
        let boxart_result = ps3::generate_ps3_boxart_from_iso(&rom_path, &boxart_output_path);
        let logo_result = ps3::extract_ps3_logo_from_iso(&rom_path, &logo_output_path);
        
        (boxart_result, logo_result)
    } else {
        return Ok(GenerateBoxartResponse {
            boxart_path: String::new(),
            relative_boxart_path: String::new(),
            logo_path: None,
            relative_logo_path: None,
            success: false,
            error: Some("不支持的 ROM 格式".to_string()),
        });
    };

    let (boxart_result, logo_result) = result;

    // 处理 boxart 结果
    match boxart_result {
        Ok(_) => {
            // 更新metadata文件，添加boxart路径
            let relative_boxart = format!("media/{}/boxfront.png", file_stem);
            if let Err(e) = update_rom_media_in_metadata(
                library_path,
                &request.system,
                &request.rom_file,
                "boxfront",
                &relative_boxart,
            ) {
                eprintln!("Warning: Failed to update boxart metadata: {}", e);
            }

            // 处理 logo 结果
            let (logo_path, relative_logo) = if logo_result.is_ok() {
                let relative_logo = format!("media/{}/logo.png", file_stem);
                if let Err(e) = update_rom_media_in_metadata(
                    library_path,
                    &request.system,
                    &request.rom_file,
                    "logo",
                    &relative_logo,
                ) {
                    eprintln!("Warning: Failed to update logo metadata: {}", e);
                }
                (Some(logo_output_path.to_string_lossy().to_string()), Some(relative_logo))
            } else {
                (None, None)
            };

            Ok(GenerateBoxartResponse {
                boxart_path: boxart_output_path.to_string_lossy().to_string(),
                relative_boxart_path: relative_boxart,
                logo_path,
                relative_logo_path: relative_logo,
                success: true,
                error: None,
            })
        }
        Err(e) => Ok(GenerateBoxartResponse {
            boxart_path: String::new(),
            relative_boxart_path: String::new(),
            logo_path: None,
            relative_logo_path: None,
            success: false,
            error: Some(e),
        }),
    }
}
