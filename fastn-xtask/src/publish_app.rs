pub fn publish_app() -> fastn_core::Result<()> {
    fastn_xtask::build_wasm::build_wasm()?;
    fastn_xtask::optimise_wasm::optimise_wasm()?;

    let gitignore_path = ".gitignore";
    if std::fs::metadata(gitignore_path).is_ok() {
        fastn_xtask::helpers::with_context(
            std::fs::remove_file(gitignore_path),
            "Failed to remove existing .gitignore",
        )?;
    }
    let mut file = fastn_xtask::helpers::with_context(
        std::fs::File::create(gitignore_path),
        "Failed to create .gitignore",
    )?;
    fastn_xtask::helpers::with_context(
        std::io::Write::write_all(&mut file, b".packages\n"),
        "Failed to write to .gitignore",
    )?;
    fastn_xtask::helpers::with_context(
        std::io::Write::write_all(&mut file, b".fastn\n"),
        "Failed to write to .gitignore",
    )?;
    fastn_xtask::helpers::with_context(
        std::io::Write::write_all(&mut file, b".is-local\n"),
        "Failed to write to .gitignore",
    )?;

    fastn_xtask::helpers::run_command(
        "sh",
        ["-c", "curl -fsSL https://fastn.com/install.sh | sh"],
        "install fastn",
    )?;

    let site_dir = fastn_xtask::helpers::find_directory(
        |name| name.ends_with(".fifthtry.site") && !name.ends_with("-template.fifthtry.site"),
        "No site directory found (looking for *.fifthtry.site)",
    )?;
    let js_dir = site_dir.join("js");
    if js_dir.is_dir() {
        fastn_xtask::helpers::set_current_dir(&js_dir, "js")?;
        fastn_xtask::helpers::run_command("npm", ["install"], "npm install")?;
        fastn_xtask::helpers::run_command("npm", ["run", "build"], "npm run build")?;
        fastn_xtask::helpers::set_current_dir(&site_dir, "site")?;
    }

    let site_name = site_dir
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.strip_suffix(".fifthtry.site"))
        .ok_or_else(|| {
            fastn_core::Error::GenericError(
                "Failed to extract site name from directory".to_string(),
            )
        })?;

    fastn_xtask::helpers::set_current_dir(&site_dir, "site")?;
    fastn_xtask::helpers::run_command("fastn", ["upload", site_name], "fastn upload")?;
    Ok(())
}
