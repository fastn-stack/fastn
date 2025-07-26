pub fn run_template() -> fastn_core::Result<()> {
    let template_dir = fastn_xtask::helpers::find_directory(
        |name| name.ends_with("-template.fifthtry.site"),
        "No template directory found (looking for *-template.fifthtry.site)",
    )?;

    let current_dir = fastn_xtask::helpers::with_context(
        std::env::current_dir(),
        "Failed to get current directory",
    )?;

    fastn_xtask::build_wasm::build_wasm()?;
    let fastn_bin = fastn_xtask::helpers::get_fastn_binary()?;
    fastn_xtask::helpers::set_current_dir(&template_dir, "template")?;
    fastn_xtask::helpers::run_command(
        &fastn_bin,
        ["--trace", "serve", "--offline"],
        "fastn serve",
    )?;
    fastn_xtask::helpers::set_current_dir(&current_dir, "original")?;
    Ok(())
}
