pub fn update_ui() -> fastn_xtask::Result<()> {
    let ui_dir = fastn_xtask::helpers::find_directory(
        |name| name.ends_with(".fifthtry.site") && !name.ends_with("-template.fifthtry.site"),
        "No directory matching '*.fifthtry.site' (excluding *-template.fifthtry.site) found",
    )?;

    let current_dir = fastn_xtask::helpers::with_context(
        std::env::current_dir(),
        "Failed to get current directory",
    )?;

    fastn_xtask::helpers::set_current_dir(&ui_dir, "UI")?;
    let fastn_binary = std::env::var("FASTN_BINARY").unwrap_or_else(|_| "fastn".to_string());
    fastn_xtask::helpers::run_command(&fastn_binary, ["update"], "fastn update")?;
    fastn_xtask::helpers::set_current_dir(&current_dir, "original")?;
    Ok(())
}
