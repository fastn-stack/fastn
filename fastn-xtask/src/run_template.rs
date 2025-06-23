use std::env;
use std::path::PathBuf;
use std::process::Command;
use regex::Regex;
use std::fs;

use crate::build_wasm;

pub fn run_template() -> fastn_core::Result<()> {
    let current_dir = env::current_dir().map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to get current directory: {}", e))
    })?;

    let re = Regex::new(r".*-template\.fifthtry\.site$").unwrap();
    let entries = fs::read_dir(&current_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to read current directory: {}", e))
    })?;
    let mut template_dir = None;
    for entry in entries {
        let entry = entry.map_err(|e| fastn_core::Error::GenericError(format!("Failed to read entry: {}", e)))?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if re.is_match(name) {
                    template_dir = Some(path);
                    break;
                }
            }
        }
    }
    let template_dir = template_dir.ok_or_else(|| fastn_core::Error::GenericError(
        "No template directory found (looking for *-template.fifthtry.site)".to_string(),
    ))?;
    println!("Using template directory: {}", template_dir.display());

    env::set_current_dir(&template_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to change to template directory: {}", e))
    })?;

    println!("Building WASM...");
    build_wasm::build_wasm()?;

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
    if let Ok(status) = Command::new("fastn").arg("--version").status() {
        if status.success() {
            return Ok("fastn".to_string());
        }
    }

    let home_dir = env::var("HOME").map_err(|_| {
        fastn_core::Error::GenericError("HOME environment variable not set".to_string())
    })?;

    let cargo_bin = PathBuf::from(&home_dir).join(".cargo/bin/fastn");
    if cargo_bin.exists() {
        return Ok(cargo_bin.to_string_lossy().to_string());
    }

    let fastn_path = "./target/debug/fastn";
    if PathBuf::from(fastn_path).exists() {
        return Ok(fastn_path.to_string());
    }

    Err(fastn_core::Error::GenericError(
        "Could not find fastn binary".to_string(),
    ))
}
