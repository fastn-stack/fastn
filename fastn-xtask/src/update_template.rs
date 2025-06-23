use std::env;
use std::process::Command;
use regex::Regex;
use std::fs;

pub fn update_template() -> fastn_core::Result<()> {
    println!("Updating template");

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
        "No directory matching '*-template.fifthtry.site' found".to_string(),
    ))?;

    env::set_current_dir(&template_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to change to template directory: {}", e))
    })?;

    let fastn_binary = env::var("FASTN_BINARY").unwrap_or_else(|_| "fastn".to_string());

    println!("Running fastn update...");
    let status = Command::new(&fastn_binary)
        .arg("update")
        .status()
        .map_err(|e| {
            fastn_core::Error::GenericError(format!("Failed to run fastn update: {}", e))
        })?;

    if !status.success() {
        return Err(fastn_core::Error::GenericError(
            "fastn update failed".to_string(),
        ));
    }

    env::set_current_dir(current_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to return to original directory: {}", e))
    })?;

    println!("Template updated successfully!");
    Ok(())
}
