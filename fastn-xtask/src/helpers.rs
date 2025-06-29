pub fn find_directory<F>(predicate: F, error_message: &str) -> fastn_core::Result<std::path::PathBuf>
where
    F: Fn(&str) -> bool,
{
    let current_dir = std::env::current_dir().map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to get current directory: {}", e))
    })?;

    let entries = std::fs::read_dir(&current_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to read current directory: {}", e))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| fastn_core::Error::GenericError(format!("Failed to read entry: {}", e)))?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if predicate(name) {
                    return Ok(path);
                }
            }
        }
    }

    Err(fastn_core::Error::GenericError(error_message.to_string()))
}

pub fn get_fastn_binary() -> fastn_core::Result<String> {
    if let Ok(status) = std::process::Command::new("fastn").arg("--version").status() {
        if status.success() {
            return Ok("fastn".to_string());
        }
    }

    let home_dir = std::env::var("HOME").map_err(|_| {
        fastn_core::Error::GenericError("HOME environment variable not set".to_string())
    })?;

    let cargo_bin = std::path::PathBuf::from(&home_dir).join(".cargo/bin/fastn");
    if cargo_bin.exists() {
        return Ok(cargo_bin.to_string_lossy().to_string());
    }

    let fastn_path = "./target/debug/fastn";
    if std::path::PathBuf::from(fastn_path).exists() {
        return Ok(fastn_path.to_string());
    }

    Err(fastn_core::Error::GenericError(
        "Could not find fastn binary".to_string(),
    ))
}

pub fn run_fastn_serve(
    target_dir: &std::path::PathBuf,
    args: &[&str],
    service_name: &str,
) -> fastn_core::Result<()> {
    let current_dir = std::env::current_dir().map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to get current directory: {}", e))
    })?;

    std::env::set_current_dir(target_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to change to {} directory: {}", service_name, e))
    })?;

    let fastn_binary = std::env::var("FASTN_BINARY").unwrap_or_else(|_| "fastn".to_string());

    println!(
        "Using {} to serve {}/",
        fastn_binary,
        target_dir.file_name().unwrap().to_string_lossy()
    );

    let status = std::process::Command::new(&fastn_binary)
        .args(args)
        .status()
        .map_err(|e| {
            fastn_core::Error::GenericError(format!("Failed to run fastn serve: {}", e))
        })?;

    if !status.success() {
        println!("fastn failed, ensure it's installed, and also consider running update-{}", service_name);
    }

    std::env::set_current_dir(current_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to return to original directory: {}", e))
    })?;

    Ok(())
} 
