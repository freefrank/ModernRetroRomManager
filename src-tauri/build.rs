fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-lib=advapi32");
    }
    generate_embedded_resources();
    tauri_build::build()
}

fn generate_embedded_resources() {
    use std::io::{Read, Write};
    use std::path::Path;

    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let resources = manifest.join("resources");
    let mut files = Vec::new();
    collect_files(&resources, &resources, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));

    let output_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let (databases, assets): (Vec<_>, Vec<_>) = files
        .into_iter()
        .partition(|(relative, _)| relative.ends_with(".csv") || relative.ends_with(".dat"));
    let database_archive = output_dir.join("embedded_databases.zip");
    let asset_archive = output_dir.join("embedded_assets.zip");
    write_archive(&database_archive, &databases);
    write_archive(&asset_archive, &assets);
    let generated = format!(
        "pub const EMBEDDED_DATABASE_ARCHIVE: &[u8] = include_bytes!({:?});\n\
         pub const EMBEDDED_ASSET_ARCHIVE: &[u8] = include_bytes!({:?});\n",
        database_archive.to_string_lossy(),
        asset_archive.to_string_lossy()
    );
    std::fs::write(output_dir.join("embedded_resources.rs"), generated).unwrap();
    println!("cargo:rerun-if-changed={}", resources.display());

    fn write_archive(path: &Path, files: &[(String, std::path::PathBuf)]) {
        let archive_file = std::fs::File::create(path).unwrap();
        let mut archive = zip::ZipWriter::new(archive_file);
        for (relative, absolute) in files {
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .compression_level(Some(9));
            archive.start_file(relative, options).unwrap();
            let mut source = std::fs::File::open(absolute).unwrap();
            let mut buffer = [0_u8; 64 * 1024];
            loop {
                let read = source.read(&mut buffer).unwrap();
                if read == 0 {
                    break;
                }
                archive.write_all(&buffer[..read]).unwrap();
            }
        }
        archive.finish().unwrap();
    }

    fn collect_files(
        root: &Path,
        directory: &Path,
        output: &mut Vec<(String, std::path::PathBuf)>,
    ) {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files(root, &path, output);
            } else if path.is_file() {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                output.push((relative, path));
            }
        }
    }
}
