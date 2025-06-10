use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn build_wasm() -> fastn_core::Result<()> {
    // Build the WASM target
    println!("Building WASM target...");
    let build_status = Command::new("cargo")
        .args(&["build", "--release", "--target", "wasm32-unknown-unknown"])
        .status()
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to execute cargo build: {}", e)))?;

    if !build_status.success() {
        return Err(fastn_core::Error::GenericError("Cargo build failed".to_string()));
    }

    // Define the possible source directories
    let current_dir = env::current_dir()
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to get current directory: {}", e)))?;
    let source1 = current_dir.join("target/wasm32-unknown-unknown/release");
    
    let home_dir = env::var("HOME")
        .map_err(|_| fastn_core::Error::GenericError("HOME environment variable not set".to_string()))?;
    let source2 = PathBuf::from(&home_dir).join("target/wasm32-unknown-unknown/release");

    let source_dir = if source1.exists() {
        source1
    } else if source2.exists() {
        source2
    } else {
        return Err(fastn_core::Error::GenericError("Source folder not found".to_string()));
    };

    // Find directories matching the "*.fifthtry.site" pattern
    let workspace_root = current_dir.clone();
    let entries = fs::read_dir(&workspace_root)
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to read workspace directory: {}", e)))?;

    let dest_dirs: Vec<PathBuf> = entries
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

    if dest_dirs.is_empty() {
        return Err(fastn_core::Error::GenericError(
            "No destination directories matching pattern '*.fifthtry.site' found".to_string()
        ));
    }

    // Copy the WASM file to each matching destination
    let wasm_file = source_dir.join("backend.wasm");
    if !wasm_file.exists() {
        return Err(fastn_core::Error::GenericError(
            format!("WASM file not found at {:?}", wasm_file)
        ));
    }

    for dest_dir in dest_dirs {
        fs::copy(&wasm_file, &dest_dir.join("backend.wasm"))
            .map_err(|e| fastn_core::Error::GenericError(
                format!("Failed to copy WASM file to {:?}: {}", dest_dir, e)
            ))?;
        
        println!("WASM file copied successfully to {}", dest_dir.display());
    }

    Ok(())
}