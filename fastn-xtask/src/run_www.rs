use std::env;
use std::process::Command;

pub fn run_www() -> fastn_core::Result<()> {
    println!("Running WWW site in test.fifthtry-community.com...");

    let current_dir = env::current_dir()
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to get current directory: {}", e)))?;
    
    let www_dir = current_dir.join("test.fifthtry-community.com");
    env::set_current_dir(&www_dir)
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to change to WWW directory: {}", e)))?;
    
    let fastn_binary = env::var("FASTN_BINARY").unwrap_or_else(|_| "fastn".to_string());
    
    println!("Using {} to serve test.fifthtry-community.com/", fastn_binary);
    
    let status = Command::new(&fastn_binary)
        .args(["--trace", "serve", "--port", "8003", "--offline"])
        .status()
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to run fastn serve: {}", e)))?;
    
    if !status.success() {
        println!("fastn failed, ensure it's installed, and also consider running update-www");
    }
    
    env::set_current_dir(current_dir)
        .map_err(|e| fastn_core::Error::GenericError(format!("Failed to return to original directory: {}", e)))?;
    
    Ok(())
} 