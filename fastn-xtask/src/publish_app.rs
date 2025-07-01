pub fn publish_app() -> fastn_core::Result<()> {
    println!("Starting app publishing process...");

    println!("Building WASM...");
    fastn_xtask::build_wasm::build_wasm()?;

    println!("Optimizing WASM...");
    fastn_xtask::optimise_wasm::optimise_wasm()?;

    println!("Updating .gitignore...");
    update_gitignore()?;

    println!("Installing latest fastn...");
    install_fastn()?;

    let current_dir = std::env::current_dir().map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to get current directory: {}", e))
    })?;

    let entries = std::fs::read_dir(&current_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to read workspace directory: {}", e))
    })?;

    let site_dirs: Vec<std::path::PathBuf> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name()?.to_string_lossy();
                if name.ends_with(".fifthtry.site") && !name.ends_with("-template.fifthtry.site") {
                    return Some(path);
                }
            }
            None
        })
        .collect();

    if site_dirs.is_empty() {
        return Err(fastn_core::Error::GenericError(
            "No site directory found (looking for *.fifthtry.site)".to_string(),
        ));
    }

    let site_dir = &site_dirs[0];
    println!("Using site directory: {}", site_dir.display());

    println!("Uploading to fastn...");
    upload_to_fastn(site_dir)?;

    println!("App published successfully!");
    Ok(())
}

fn update_gitignore() -> fastn_core::Result<()> {
    let gitignore_path = ".gitignore";
    if std::fs::metadata(gitignore_path).is_ok() {
        std::fs::remove_file(gitignore_path).map_err(|e| {
            fastn_core::Error::GenericError(format!("Failed to remove existing .gitignore: {}", e))
        })?;
    }

    let mut file = std::fs::File::create(gitignore_path).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to create .gitignore: {}", e))
    })?;

    std::io::Write::write_all(&mut file, b".packages\n").map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to write to .gitignore: {}", e))
    })?;
    std::io::Write::write_all(&mut file, b".fastn\n").map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to write to .gitignore: {}", e))
    })?;
    std::io::Write::write_all(&mut file, b".is-local\n").map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to write to .gitignore: {}", e))
    })?;

    Ok(())
}

fn install_fastn() -> fastn_core::Result<()> {
    let status = std::process::Command::new("sh")
        .args(["-c", "$(curl -fsSL https://fastn.com/install.sh)"])
        .status()
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to install fastn: {}", e)))?;

    if !status.success() {
        return Err(fastn_core::Error::GenericError(
            "Failed to install fastn".to_string(),
        ));
    }

    Ok(())
}

fn upload_to_fastn(site_dir: &std::path::PathBuf) -> fastn_core::Result<()> {
    std::env::set_current_dir(site_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to change to site directory: {}", e))
    })?;

    let status = std::process::Command::new("fastn")
        .args(["upload", "test"])
        .status()
        .map_err(|e| {
            fastn_core::Error::GenericError(format!("Failed to run fastn upload: {}", e))
        })?;

    if !status.success() {
        return Err(fastn_core::Error::GenericError(
            "fastn upload failed".to_string(),
        ));
    }

    Ok(())
}
