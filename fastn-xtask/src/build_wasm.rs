pub fn build_wasm() -> fastn_xtask::Result<()> {
    fastn_xtask::helpers::run_command(
        "cargo",
        [
            "build",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
            "--package",
            "backend",
        ],
        "cargo build",
    )?;

    let current_dir = fastn_xtask::helpers::with_context(
        std::env::current_dir(),
        "Failed to get current directory",
    )?;

    let source1 = std::path::PathBuf::from("./target/wasm32-unknown-unknown/release");
    let home_dir = fastn_xtask::helpers::with_context(
        std::env::var("HOME"),
        "HOME environment variable not set",
    )?;
    let source2 = std::path::PathBuf::from(&home_dir).join("target/wasm32-unknown-unknown/release");

    let source_dir = if source1.exists() {
        source1
    } else if source2.exists() {
        source2
    } else {
        return Err(fastn_xtask::Error::GenericError(
            "Source folder not found".to_string(),
        ));
    };

    let dest_dirs = {
        let entries = fastn_xtask::helpers::with_context(
            std::fs::read_dir(&current_dir),
            "Failed to read current directory",
        )?;
        entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name()?.to_string_lossy();
                    if name.ends_with(".fifthtry.site") {
                        return Some(path);
                    }
                }
                None
            })
            .collect::<Vec<_>>()
    };

    if dest_dirs.is_empty() {
        return Err(fastn_xtask::Error::GenericError(
            "No destination directories matching pattern '*.fifthtry.site' found".to_string(),
        ));
    }

    let wasm_file = source_dir.join("backend.wasm");
    if !wasm_file.exists() {
        return Err(fastn_xtask::Error::GenericError(format!(
            "WASM file not found at {:?}",
            wasm_file
        )));
    }

    for dest_dir in dest_dirs {
        fastn_xtask::helpers::with_context(
            std::fs::copy(&wasm_file, dest_dir.join("backend.wasm")),
            &format!("Failed to copy WASM file to {:?}", dest_dir),
        )?;
    }

    Ok(())
}
