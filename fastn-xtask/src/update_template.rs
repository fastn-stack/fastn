pub fn update_template() -> fastn_core::Result<()> {
    println!("Updating template");

    let template_dir = fastn_xtask::helpers::find_directory(
        |name| name.ends_with("-template.fifthtry.site"),
        "No directory matching '*-template.fifthtry.site' found",
    )?;

    let current_dir = std::env::current_dir().map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to get current directory: {}", e))
    })?;

    std::env::set_current_dir(&template_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to change to template directory: {}", e))
    })?;

    let fastn_binary = std::env::var("FASTN_BINARY").unwrap_or_else(|_| "fastn".to_string());

    println!("Running fastn update...");
    let status = std::process::Command::new(&fastn_binary)
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

    std::env::set_current_dir(current_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to return to original directory: {}", e))
    })?;

    println!("Template updated successfully!");
    Ok(())
}
