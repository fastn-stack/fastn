const DEFAULT_BINARYEN_VERSION: &str = "version_119";

pub fn optimise_wasm() -> fastn_xtask::Result<()> {
    let binaryen_version = std::env::var("BINARYEN_VERSION")
        .unwrap_or_else(|_| DEFAULT_BINARYEN_VERSION.to_string());

    let wasm_opt_cmd = if let Ok(output) = std::process::Command::new("wasm-opt").arg("--version").output() {
        if output.status.success() {
            "wasm-opt".to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    let wasm_opt_cmd = if !wasm_opt_cmd.is_empty() {
        wasm_opt_cmd
    } else {
        let os = std::env::consts::OS;
        let binary_name = match os {
            "linux" => format!("binaryen-{}-x86_64-linux.tar.gz", binaryen_version),
            "macos" | "darwin" => format!("binaryen-{}-x86_64-macos.tar.gz", binaryen_version),
            _ => {
                return Err(fastn_xtask::Error::GenericError(format!(
                    "Unsupported platform: {}",
                    os
                )));
            }
        };
        let repo_name = "WebAssembly/binaryen";
        let local_install_dir = format!("./bin/binaryen-{}", binaryen_version);
        fastn_xtask::helpers::with_context(
            std::fs::create_dir_all("./bin"),
            "Failed to create bin directory",
        )?;
        if !std::path::Path::new(&local_install_dir).exists() {
            // Inline download_release
            let url = format!(
                "https://github.com/{}/releases/download/{}/{}",
                repo_name, binaryen_version, binary_name
            );
            fastn_xtask::helpers::run_command(
                "curl",
                ["-L", "-o", &binary_name, &url],
                "download binaryen",
            )?;
            fastn_xtask::helpers::run_command(
                "tar",
                ["-xzf", &binary_name, "-C", "./bin/"],
                "extract binaryen archive",
            )?;
            fastn_xtask::helpers::with_context(
                std::fs::remove_file(&binary_name),
                "Failed to remove archive",
            )?;
        }
        let wasm_opt_path = format!("{}/bin/wasm-opt", local_install_dir);
        if !std::path::Path::new(&wasm_opt_path).exists() {
            return Err(fastn_xtask::Error::GenericError(
                "wasm-opt not found in the extracted files".to_string(),
            ));
        }
        wasm_opt_path
    };

    let current_dir = fastn_xtask::helpers::with_context(
        std::env::current_dir(),
        "Failed to get current directory",
    )?;
    let entries = fastn_xtask::helpers::with_context(
        std::fs::read_dir(&current_dir),
        "Failed to read workspace directory",
    )?;
    let fifthtry_dirs: Vec<std::path::PathBuf> = entries
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
        .collect();
    if fifthtry_dirs.is_empty() {
        return Err(fastn_xtask::Error::GenericError(
            "No directories matching pattern '*.fifthtry.site' found".to_string(),
        ));
    }
    for dir in fifthtry_dirs {
        let wasm_file = dir.join("backend.wasm");
        if wasm_file.exists() {
            // Inline optimise_wasm_file
            let before_size = fastn_xtask::helpers::with_context(
                std::fs::metadata(&wasm_file),
                "Failed to get file metadata",
            )?.len();
            fastn_xtask::helpers::run_command(
                &wasm_opt_cmd,
                ["-Oz", &wasm_file.to_string_lossy(), "-o", &wasm_file.to_string_lossy()],
                "wasm-opt",
            )?;
            let after_size = fastn_xtask::helpers::with_context(
                std::fs::metadata(&wasm_file),
                "Failed to get file metadata",
            )?.len();
            let size_diff = before_size.saturating_sub(after_size);
            let size_diff_percentage = if before_size > 0 {
                (size_diff as f64 * 100.0 / before_size as f64) as u64
            } else {
                0
            };
            let to_human_readable = |size: u64| {
                if size >= 1_048_576 {
                    format!("{:.1}MB", size as f64 / 1_048_576.0)
                } else if size >= 1_024 {
                    format!("{:.1}KB", size as f64 / 1_024.0)
                } else {
                    format!("{}B", size)
                }
            };
            println!(
                "{}: {} -> {} ({}% reduction)",
                wasm_file.display(),
                to_human_readable(before_size),
                to_human_readable(after_size),
                size_diff_percentage
            );
        } else {
            eprintln!("Warning: No backend.wasm found in {}", dir.display());
        }
    }
    Ok(())
}
