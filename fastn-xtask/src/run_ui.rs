pub fn run_ui() -> fastn_core::Result<()> {
    let ui_dir = fastn_xtask::helpers::find_directory(
        |name| name.ends_with(".fifthtry.site") && !name.ends_with("-template.fifthtry.site"),
        "No directory matching '*.fifthtry.site' (excluding *-template.fifthtry.site) found",
    )?;

    fastn_xtask::helpers::run_fastn_serve(
        &ui_dir,
        &["--trace", "serve", "--port", "8002", "--offline"],
        "ui",
    )
}
