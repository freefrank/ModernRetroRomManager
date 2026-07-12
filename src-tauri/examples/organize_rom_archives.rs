use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

#[cfg(windows)]
#[link(name = "advapi32")]
unsafe extern "system" {}

fn extensions(system: &str) -> &'static [&'static str] {
    match system.to_ascii_uppercase().as_str() {
        "MD" => &["md", "gen", "smd", "bin"],
        "N64" => &["z64", "n64", "v64"],
        "SFC" => &["sfc", "smc"],
        _ => &[],
    }
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|value| value.to_str()) != Some("_archives") {
                collect_files(&path, files);
            }
        } else {
            files.push(path);
        }
    }
}

fn is_rar(path: &Path) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut magic = [0_u8; 8];
    file.read(&mut magic)
        .is_ok_and(|length| length >= 7 && magic.starts_with(b"Rar!\x1a\x07"))
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        return Err("用法: organize_rom_archives <目录> <MD|N64|SFC> [密码]".to_string());
    }
    let directory = Path::new(&args[1]);
    let allowed = extensions(&args[2]);
    if allowed.is_empty() {
        return Err("目前仅支持 MD、N64、SFC".to_string());
    }
    let password = args.get(3).cloned().unwrap_or_default();
    let mut files = Vec::new();
    collect_files(directory, &mut files);
    let archives: Vec<_> = files.into_iter().filter(|path| is_rar(path)).collect();
    let mut extracted = 0_usize;
    let mut skipped = 0_usize;

    for path in archives {
        let mut recognized = 0_usize;
        let mut archive = if password.is_empty() {
            unrar::Archive::new(&path).open_for_processing()
        } else {
            unrar::Archive::with_password(&path, &password).open_for_processing()
        }
        .map_err(|error| format!("无法打开 {}: {error}", path.display()))?;

        while let Some(header) = archive
            .read_header()
            .map_err(|error| format!("无法读取 {}: {error}", path.display()))?
        {
            let entry = header.entry();
            let matches = entry
                .filename
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| allowed.iter().any(|ext| value.eq_ignore_ascii_case(ext)));
            if !matches {
                archive = header.skip().map_err(|error| error.to_string())?;
                continue;
            }
            recognized += 1;
            let Some(file_name) = entry.filename.file_name() else {
                archive = header.skip().map_err(|error| error.to_string())?;
                continue;
            };
            let target = path.parent().unwrap_or(directory).join(file_name);
            if target.exists() {
                println!("跳过（已存在）: {}", target.display());
                skipped += 1;
                archive = header.skip().map_err(|error| error.to_string())?;
                continue;
            }
            let temporary = target.with_extension(format!(
                "{}.extracting",
                target
                    .extension()
                    .and_then(|value| value.to_str())
                    .unwrap_or("rom")
            ));
            if temporary.exists() {
                return Err(format!("发现未完成的临时文件: {}", temporary.display()));
            }
            archive = header
                .extract_to(&temporary)
                .map_err(|error| format!("解压 {} 失败: {error}", path.display()))?;
            if fs::metadata(&temporary)
                .map_err(|error| error.to_string())?
                .len()
                == 0
            {
                return Err(format!("解压结果为空: {}", temporary.display()));
            }
            fs::rename(&temporary, &target).map_err(|error| error.to_string())?;
            println!("已解压: {} -> {}", path.display(), target.display());
            extracted += 1;
        }
        if recognized > 0 {
            let parent = path.parent().unwrap_or(directory);
            let backup = parent.join("_archives");
            fs::create_dir_all(&backup).map_err(|error| error.to_string())?;
            let destination = backup.join(path.file_name().ok_or("压缩包文件名无效")?);
            if destination.exists() {
                println!("压缩源未移动（备份已存在）: {}", destination.display());
            } else {
                fs::rename(&path, &destination).map_err(|error| error.to_string())?;
                println!("压缩源已归档: {}", destination.display());
            }
        }
    }
    println!("完成: 解压 {extracted}，跳过 {skipped}；压缩源移入 _archives");
    Ok(())
}
