use std::env;
use std::process::Command;
use regex::Regex;
use std::fs;

pub fn run_www() -> fastn_core::Result<()> {
    println!("Running WWW site");

    let current_dir = env::current_dir().map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to get current directory: {}", e))
    })?;

    let re = Regex::new(r".*\.fifthtry-community\.com$").unwrap();
    let entries = fs::read_dir(&current_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to read current directory: {}", e))
    })?;
    let mut www_dir = None;
    for entry in entries {
        let entry = entry.map_err(|e| fastn_core::Error::GenericError(format!("Failed to read entry: {}", e)))?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if re.is_match(name) {
                    www_dir = Some(path);
                    break;
                }
            }
        }
    }
    let www_dir = www_dir.ok_or_else(|| fastn_core::Error::GenericError(
        "No directory matching '*.fifthtry-community.com' found".to_string(),
    ))?;

    env::set_current_dir(&www_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to change to WWW directory: {}", e))
    })?;

    let fastn_binary = env::var("FASTN_BINARY").unwrap_or_else(|_| "fastn".to_string());

    println!(
        "Using {} to serve {}/",
        fastn_binary,
        www_dir.file_name().unwrap().to_string_lossy()
    );

    let status = Command::new(&fastn_binary)
        .args(["--trace", "serve", "--port", "8003", "--offline"])
        .status()
        .map_err(|e| {
            fastn_core::Error::GenericError(format!("Failed to run fastn serve: {}", e))
        })?;

    if !status.success() {
        println!("fastn failed, ensure it's installed, and also consider running update-www");
    }

    env::set_current_dir(current_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to return to original directory: {}", e))
    })?;

    Ok(())
}
