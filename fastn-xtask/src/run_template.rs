use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::build_wasm;

pub fn run_template() -> fastn_core::Result<()> {
    // Find the template directory (any directory ending with -template.fifthtry.site)
    let current_dir = env::current_dir().map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to get current directory: {}", e))
    })?;

    let entries = std::fs::read_dir(&current_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to read current directory: {}", e))
    })?;

    let template_dirs: Vec<PathBuf> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name()?.to_string_lossy();
                if name.ends_with("-template.fifthtry.site") {
                    return Some(path);
                }
            }
            None
        })
        .collect();

    if template_dirs.is_empty() {
        return Err(fastn_core::Error::GenericError(
            "No template directory found (looking for *-template.fifthtry.site)".to_string(),
        ));
    }

    let template_dir = &template_dirs[0]; // Use the first matching template directory
    println!("Using template directory: {}", template_dir.display());

    // Change to the template directory
    env::set_current_dir(template_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to change to template directory: {}", e))
    })?;

    // Run build_wasm
    println!("Building WASM...");
    build_wasm::build_wasm()?;

    // Run fastn serve with offline mode
    println!("Starting fastn serve in offline mode...");
    let fastn_bin = get_fastn_binary()?;

    let status = Command::new(fastn_bin)
        .args(["--trace", "serve", "--offline"])
        .status()
        .map_err(|e| {
            fastn_core::Error::GenericError(format!("Failed to execute fastn serve: {}", e))
        })?;

    if !status.success() {
        return Err(fastn_core::Error::GenericError(
            "fastn serve failed".to_string(),
        ));
    }

    Ok(())
}

fn get_fastn_binary() -> fastn_core::Result<String> {
    // Try to find fastn in PATH first
    if let Ok(status) = Command::new("fastn").arg("--version").status() {
        if status.success() {
            return Ok("fastn".to_string());
        }
    }

    // Try cargo-installed fastn
    let home_dir = env::var("HOME").map_err(|_| {
        fastn_core::Error::GenericError("HOME environment variable not set".to_string())
    })?;

    let cargo_bin = PathBuf::from(&home_dir).join(".cargo/bin/fastn");
    if cargo_bin.exists() {
        return Ok(cargo_bin.to_string_lossy().to_string());
    }

    // Use relative path as last resort
    let fastn_path = "./target/debug/fastn";
    if PathBuf::from(fastn_path).exists() {
        return Ok(fastn_path.to_string());
    }

    Err(fastn_core::Error::GenericError(
        "Could not find fastn binary".to_string(),
    ))
}
