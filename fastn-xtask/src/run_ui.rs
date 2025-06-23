use std::env;
use std::process::Command;
use regex::Regex;
use std::fs;

pub fn run_ui() -> fastn_core::Result<()> {
    println!("Running UI");

    let current_dir = env::current_dir().map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to get current directory: {}", e))
    })?;

    let re = Regex::new(r".*\.fifthtry\.site$").unwrap();
    let template_re = Regex::new(r"-template\.fifthtry\.site$").unwrap();
    let entries = fs::read_dir(&current_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to read current directory: {}", e))
    })?;
    let mut ui_dir = None;
    for entry in entries {
        let entry = entry.map_err(|e| fastn_core::Error::GenericError(format!("Failed to read entry: {}", e)))?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if re.is_match(name) && !template_re.is_match(name) {
                    ui_dir = Some(path);
                    break;
                }
            }
        }
    }
    let ui_dir = ui_dir.ok_or_else(|| fastn_core::Error::GenericError(
        "No directory matching '*.fifthtry.site' (excluding *-template.fifthtry.site) found".to_string(),
    ))?;

    env::set_current_dir(&ui_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to change to UI directory: {}", e))
    })?;

    let fastn_binary = env::var("FASTN_BINARY").unwrap_or_else(|_| "fastn".to_string());

    println!("Using {} to serve {}/", fastn_binary, ui_dir.file_name().unwrap().to_string_lossy());

    let status = Command::new(&fastn_binary)
        .args(["--trace", "serve", "--port", "8002", "--offline"])
        .status()
        .map_err(|e| {
            fastn_core::Error::GenericError(format!("Failed to run fastn serve: {}", e))
        })?;

    if !status.success() {
        println!("fastn failed, ensure it's installed, and also consider running update-ui");
    }

    env::set_current_dir(current_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to return to original directory: {}", e))
    })?;

    Ok(())
}
