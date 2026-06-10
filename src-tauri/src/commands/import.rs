#[tauri::command]
pub fn import_gamelist(_xml_path: String) -> Result<usize, String> {
    Err("手动导入 gamelist.xml 已废弃，请在 ROM 库中添加目录并选择 EmulationStation 格式。".to_string())
}

#[tauri::command]
pub fn import_pegasus(_file_path: String) -> Result<usize, String> {
    Err("手动导入 Pegasus 元数据已废弃，请在 ROM 库中添加目录并选择 Pegasus 格式。".to_string())
}
