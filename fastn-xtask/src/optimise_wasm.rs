use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_BINARYEN_VERSION: &str = "version_119";

pub fn optimise_wasm() -> fastn_core::Result<()> {
    println!("Starting WASM optimization...");

    let binaryen_version =
        env::var("BINARYEN_VERSION").unwrap_or_else(|_| DEFAULT_BINARYEN_VERSION.to_string());

    let wasm_opt_cmd = ensure_wasm_opt(&binaryen_version)?;

    let current_dir = env::current_dir().map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to get current directory: {}", e))
    })?;

    let entries = fs::read_dir(&current_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to read workspace directory: {}", e))
    })?;

    let fifthtry_dirs: Vec<PathBuf> = entries
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
        return Err(fastn_core::Error::GenericError(
            "No directories matching pattern '*.fifthtry.site' found".to_string(),
        ));
    }

    for dir in fifthtry_dirs {
        let wasm_file = dir.join("backend.wasm");
        if wasm_file.exists() {
            optimise_wasm_file(&wasm_opt_cmd, &wasm_file)?;
        } else {
            println!("Warning: No backend.wasm found in {}", dir.display());
        }
    }

    Ok(())
}

fn ensure_wasm_opt(binaryen_version: &str) -> fastn_core::Result<String> {
    if let Ok(output) = Command::new("wasm-opt").arg("--version").output() {
        if output.status.success() {
            println!("Using globally installed wasm-opt");
            return Ok("wasm-opt".to_string());
        }
    }

    println!("wasm-opt not found in PATH. Setting up local version...");

    let os = env::consts::OS;
    let binary_name = match os {
        "linux" => format!("binaryen-{}-x86_64-linux.tar.gz", binaryen_version),
        "macos" | "darwin" => format!("binaryen-{}-x86_64-macos.tar.gz", binaryen_version),
        _ => {
            return Err(fastn_core::Error::GenericError(format!(
                "Unsupported platform: {}",
                os
            )));
        }
    };

    let repo_name = "WebAssembly/binaryen";
    let local_install_dir = format!("./bin/binaryen-{}", binaryen_version);

    fs::create_dir_all("./bin").map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to create bin directory: {}", e))
    })?;

    if !Path::new(&local_install_dir).exists() {
        download_release(repo_name, binaryen_version, &binary_name)?;

        println!("Extracting {}...", binary_name);
        let extract_status = Command::new("tar")
            .args(["-xzf", &binary_name, "-C", "./bin/"])
            .status()
            .map_err(|e| {
                fastn_core::Error::GenericError(format!("Failed to extract archive: {}", e))
            })?;

        if !extract_status.success() {
            return Err(fastn_core::Error::GenericError(
                "Failed to extract binaryen archive".to_string(),
            ));
        }

        println!("Removing {}...", binary_name);
        fs::remove_file(&binary_name).map_err(|e| {
            fastn_core::Error::GenericError(format!("Failed to remove archive: {}", e))
        })?;
    }

    let wasm_opt_path = format!("{}/bin/wasm-opt", local_install_dir);
    if !Path::new(&wasm_opt_path).exists() {
        return Err(fastn_core::Error::GenericError(
            "wasm-opt not found in the extracted files".to_string(),
        ));
    }

    println!("Using local wasm-opt at {}", wasm_opt_path);
    Ok(wasm_opt_path)
}

fn download_release(repo_name: &str, version: &str, binary_name: &str) -> fastn_core::Result<()> {
    let url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        repo_name, version, binary_name
    );
    println!("Downloading release from {}", url);

    let status = Command::new("curl")
        .args(["-L", "-o", binary_name, &url])
        .status()
        .map_err(|e| {
            fastn_core::Error::GenericError(format!("Failed to download binaryen: {}", e))
        })?;

    if !status.success() {
        return Err(fastn_core::Error::GenericError(format!(
            "Failed to download {} from {}",
            binary_name, url
        )));
    }

    Ok(())
}

fn optimise_wasm_file(wasm_opt_cmd: &str, wasm_file: &Path) -> fastn_core::Result<()> {
    println!("Optimizing: {}", wasm_file.display());

    let before_size = fs::metadata(wasm_file)
        .map_err(|e| {
            fastn_core::Error::GenericError(format!("Failed to get file metadata: {}", e))
        })?
        .len();

    let status = Command::new(wasm_opt_cmd)
        .args([
            "-Oz",
            &wasm_file.to_string_lossy(),
            "-o",
            &wasm_file.to_string_lossy(),
        ])
        .status()
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to run wasm-opt: {}", e)))?;

    if !status.success() {
        return Err(fastn_core::Error::GenericError(
            "Optimization failed".to_string(),
        ));
    }

    let after_size = fs::metadata(wasm_file)
        .map_err(|e| {
            fastn_core::Error::GenericError(format!("Failed to get file metadata: {}", e))
        })?
        .len();

    let size_diff = before_size.saturating_sub(after_size);
    let size_diff_percentage = if before_size > 0 {
        (size_diff as f64 * 100.0 / before_size as f64) as u64
    } else {
        0
    };

    println!(
        "{}: Before = {}, After = {}, Reduction = {} ({}%)",
        wasm_file.display(),
        to_human_readable(before_size),
        to_human_readable(after_size),
        to_human_readable(size_diff),
        size_diff_percentage
    );

    Ok(())
}

fn to_human_readable(size: u64) -> String {
    if size >= 1_048_576 {
        format!("{:.3}MB", size as f64 / 1_048_576.0)
    } else if size >= 1_024 {
        format!("{:.3}KB", size as f64 / 1_024.0)
    } else {
        format!("{}B", size)
    }
}
