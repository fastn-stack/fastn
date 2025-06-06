pub async fn run_template_command(app_name: &str) -> fastn_core::Result<()> {
    println!("Creating new app: {}", app_name);

    let zip_url = "https://github.com/fifthtry-community/app-template/zipball/main";
    let target_dir = app_name.to_string();
    
    // Download the zip file
    let response = reqwest::get(zip_url).await?;
    if !response.status().is_success() {
        return Err(fastn_core::Error::GenericError(format!(
            "Failed to download template: HTTP {}", 
            response.status()
        )));
    }
    let zip_bytes = response.bytes().await.map_err(|e| fastn_core::Error::GenericError(e.to_string()))?;

    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to read zip archive: {}", e)))?;

    let temp_dir = std::env::temp_dir().join(format!("fastn-template-{}", app_name));
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)
            .map_err(|e| fastn_core::Error::GenericError(format!("Failed to clean temp directory: {}", e)))?;
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| fastn_core::Error::GenericError(format!("Failed to read zip entry: {}", e)))?;
        
        let outpath = match file.enclosed_name() {
            Some(path) => temp_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| fastn_core::Error::GenericError(format!("Failed to create directory: {}", e)))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)
                        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to create parent directory: {}", e)))?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| fastn_core::Error::GenericError(format!("Failed to create file: {}", e)))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| fastn_core::Error::GenericError(format!("Failed to extract file: {}", e)))?;
        }
    }

    // Find the extracted directory (it will have a name like "fifthtry-community-app-template-<commit-hash>")
    let extracted_dirs: Vec<_> = std::fs::read_dir(&temp_dir)
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to read temp directory: {}", e)))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_dir() {
                Some(entry.path())
            } else {
                None
            }
        })
        .collect();

    if extracted_dirs.is_empty() {
        return Err(fastn_core::Error::GenericError("No directories found in extracted zip".to_string()));
    }

    let source_dir = &extracted_dirs[0];
    let target_path = std::path::Path::new(&target_dir);

    // Move the extracted content to target directory
    std::fs::rename(source_dir, target_path)
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to move extracted files: {}", e)))?;

    // Clean up temp directory
    std::fs::remove_dir_all(&temp_dir)
        .map_err(|e| eprintln!("Failed to clean up temp directory: {}", e))
        .ok();

    // Update scripts folder files
    let scripts_dir = target_path.join("scripts");
    if scripts_dir.exists() {
        let script_files = ["auto.sh", "build-wasm.sh", "optimise-wasm.sh", "publish-app.sh"];
        
        for script_file in &script_files {
            let script_path = scripts_dir.join(script_file);
            if script_path.exists() {
                if let Ok(contents) = std::fs::read_to_string(&script_path) {
                    let new_contents = contents.replace("lets-XXX", app_name);
                    std::fs::write(&script_path, new_contents)
                        .map_err(|e| eprintln!("Failed to update {} in scripts: {}", script_file, e))
                        .ok();
                }
            }
        }
    }

    let dir_patterns = [
        "lets-XXX.fifthtry.site",
        "lets-XXX.fifthtry-community.com", 
        "lets-XXX-template.fifthtry.site",
    ];

    // Process directories and FASTN.ftd files in a single loop
    let mut renamed_dirs = Vec::new();
    for pattern in &dir_patterns {
        let old_dir = pattern.to_string();
        let new_dir = old_dir.replace("lets-XXX", app_name);
        let old_path = target_path.join(&old_dir);
        let new_path = target_path.join(&new_dir);

        if old_path.exists() {
            // Rename directory
            if let Err(e) = std::fs::rename(&old_path, &new_path) {
                eprintln!("Failed to rename directory {}: {}", old_dir, e);
                continue;
            }
            renamed_dirs.push(new_path.clone());

            // Update FASTN.ftd file
            let fastn_ftd_path = new_path.join("FASTN.ftd");
            if fastn_ftd_path.exists() {
                if let Ok(contents) = std::fs::read_to_string(&fastn_ftd_path) {
                    let new_contents = contents.replace("lets-XXX", app_name);
                    std::fs::write(&fastn_ftd_path, new_contents)
                        .map_err(|e| eprintln!("Failed to update FASTN.ftd in {}: {}", new_dir, e))
                        .ok();
                }
            }
        }
    }

    // Create symlink
    let template_dir_name = format!("{}-template.fifthtry.site", app_name);
    let packages_dir = target_path.join(&template_dir_name).join(".packages");
    
    // Create .packages directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&packages_dir) {
        eprintln!("Failed to create .packages directory: {}", e);
    }
    
    let symlink_name = format!("{}.fifthtry.site", app_name);
    let symlink_target = format!("../../{}.fifthtry.site", app_name);
    let symlink_path = packages_dir.join(&symlink_name);

    if symlink_path.exists() {
        std::fs::remove_file(&symlink_path).ok();
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(&symlink_target, &symlink_path)
            .map_err(|e| eprintln!("Failed to create symlink: {}", e))
            .ok();
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_dir;
        symlink_dir(&symlink_target, &symlink_path)
            .map_err(|e| eprintln!("Failed to create symlink: {}", e))
            .ok();
    }

    println!("App '{}' created successfully!", app_name);
    Ok(())
}
