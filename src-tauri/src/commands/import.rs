// 两个命令均未注册进 invoke_handler,仅作为占位保留
#![allow(dead_code)]

#[tauri::command]
pub fn import_gamelist(_xml_path: String) -> Result<usize, String> {
    Err("Importing gamelist.xml manually is deprecated. Please adding the directory as a library with 'EmulationStation' format selected.".to_string())
}

#[tauri::command]
pub fn import_pegasus(_file_path: String) -> Result<usize, String> {
    Err("Importing pegasus metadata manually is deprecated. Please adding the directory as a library with 'Pegasus' format selected.".to_string())
}
