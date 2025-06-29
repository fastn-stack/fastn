pub fn run_template() -> fastn_core::Result<()> {
    let template_dir = crate::helpers::find_directory(
        |name| name.ends_with("-template.fifthtry.site"),
        "No template directory found (looking for *-template.fifthtry.site)",
    )?;
    println!("Using template directory: {}", template_dir.display());

    let current_dir = std::env::current_dir().map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to get current directory: {}", e))
    })?;

    std::env::set_current_dir(&template_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to change to template directory: {}", e))
    })?;

    println!("Building WASM...");
    crate::build_wasm::build_wasm()?;

    println!("Starting fastn serve in offline mode...");
    let fastn_bin = crate::helpers::get_fastn_binary()?;

    let status = std::process::Command::new(fastn_bin)
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

    std::env::set_current_dir(current_dir).map_err(|e| {
        fastn_core::Error::GenericError(format!("Failed to return to original directory: {}", e))
    })?;

    Ok(())
}
