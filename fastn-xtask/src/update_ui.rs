use std::env;
use std::process::Command;

pub fn update_ui() -> fastn_core::Result<()> {
    println!("Updating UI in test.fifthtry.site...");

    let current_dir = env::current_dir()
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to get current directory: {}", e)))?;
    
    let ui_dir = current_dir.join("test.fifthtry.site");
    env::set_current_dir(&ui_dir)
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to change to UI directory: {}", e)))?;
    
    let fastn_binary = env::var("FASTN_BINARY").unwrap_or_else(|_| "fastn".to_string());
    
    println!("Running fastn update...");
    let status = Command::new(&fastn_binary)
        .arg("update")
        .status()
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to run fastn update: {}", e)))?;
    
    if !status.success() {
        return Err(fastn_core::Error::GenericError("fastn update failed".to_string()));
    }
    
    env::set_current_dir(current_dir)
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to return to original directory: {}", e)))?;
    
    println!("UI updated successfully!");
    Ok(())
} 