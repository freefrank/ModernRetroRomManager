fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-lib=advapi32");
    }
    generate_embedded_resources();
    tauri_build::build()
}

fn generate_embedded_resources() {
    use std::fmt::Write as _;
    use std::path::Path;

    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let resources = manifest.join("resources");
    let mut files = Vec::new();
    collect_files(&resources, &resources, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));

    let mut generated = String::from("pub const EMBEDDED_RESOURCES: &[(&str, &[u8])] = &[\n");
    for (relative, absolute) in files {
        writeln!(
            generated,
            "    ({relative:?}, include_bytes!({absolute:?})),",
            relative = relative,
            absolute = absolute.to_string_lossy()
        )
        .unwrap();
    }
    generated.push_str("];\n");
    let output =
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("embedded_resources.rs");
    std::fs::write(output, generated).unwrap();
    println!("cargo:rerun-if-changed={}", resources.display());

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
