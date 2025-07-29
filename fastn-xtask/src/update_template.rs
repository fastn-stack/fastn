pub fn update_template() -> fastn_xtask::Result<()> {
    let template_dir = fastn_xtask::helpers::find_directory(
        |name| name.ends_with("-template.fifthtry.site"),
        "No directory matching '*-template.fifthtry.site' found",
    )?;

    let current_dir = fastn_xtask::helpers::with_context(
        std::env::current_dir(),
        "Failed to get current directory",
    )?;

    fastn_xtask::helpers::set_current_dir(&template_dir, "template")?;
    let fastn_binary = std::env::var("FASTN_BINARY").unwrap_or_else(|_| "fastn".to_string());
    fastn_xtask::helpers::run_command(&fastn_binary, ["update"], "fastn update")?;
    fastn_xtask::helpers::set_current_dir(&current_dir, "original")?;
    Ok(())
}
