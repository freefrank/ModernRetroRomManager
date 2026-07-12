use std::env;
use std::path::Path;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        return Err("用法: organize_rom_archives <目录> <MD|N64|SFC> [密码]".to_string());
    }
    let result = modern_retro_rom_manager_lib::rom_archive::organize_rom_archives(
        Path::new(&args[1]),
        &args[2],
        args.get(3).map(String::as_str).unwrap_or_default(),
    )?;
    println!(
        "完成: 发现 {} 个压缩包，解压 {}，跳过 {}，归档 {}",
        result.archives, result.extracted, result.skipped, result.archived
    );
    Ok(())
}
