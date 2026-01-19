use crate::config::get_temp_dir;
use crate::ps3;
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
    /// 生成的 boxart 路径
    pub boxart_path: String,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    pub error: Option<String>,
}

/// 为 PS3 ROM 生成 boxart
///
/// 从 PS3_GAME 目录提取 PIC1.PNG 和 ICON0.PNG，合成为 boxart
/// 生成的 boxart 保存到 temp 目录
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

    // 确定 PS3_GAME 目录位置
    let ps3_game_dir = if rom_path.is_dir() {
        // ROM 是文件夹（PS3 游戏文件夹）
        rom_path.join("PS3_GAME")
    } else if rom_path.extension().and_then(|e| e.to_str()) == Some("iso") {
        // ROM 是 ISO 文件，暂不支持从 ISO 提取图片
        return Ok(GenerateBoxartResponse {
            boxart_path: String::new(),
            success: false,
            error: Some("暂不支持从 ISO 文件生成 boxart".to_string()),
        });
    } else {
        return Ok(GenerateBoxartResponse {
            boxart_path: String::new(),
            success: false,
            error: Some("不支持的 ROM 格式".to_string()),
        });
    };

    // 检查 PS3_GAME 目录是否存在
    if !ps3_game_dir.exists() || !ps3_game_dir.is_dir() {
        return Ok(GenerateBoxartResponse {
            boxart_path: String::new(),
            success: false,
            error: Some("PS3_GAME 目录不存在".to_string()),
        });
    }

    // 构建输出路径（保存到 temp 目录）
    let temp_dir = get_temp_dir().join(&request.system).join("media");
    let output_filename = format!("{}_boxart.png", request.rom_file);
    let output_path = temp_dir.join(&output_filename);

    // 生成 boxart
    match ps3::generate_ps3_boxart(&ps3_game_dir, &output_path) {
        Ok(_) => Ok(GenerateBoxartResponse {
            boxart_path: output_path.to_string_lossy().to_string(),
            success: true,
            error: None,
        }),
        Err(e) => Ok(GenerateBoxartResponse {
            boxart_path: String::new(),
            success: false,
            error: Some(e),
        }),
    }
}
