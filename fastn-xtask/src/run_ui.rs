pub fn run_ui() -> fastn_core::Result<()> {
    println!("Running UI");

    let ui_dir = crate::helpers::find_directory(
        |name| name.ends_with(".fifthtry.site") && !name.ends_with("-template.fifthtry.site"),
        "No directory matching '*.fifthtry.site' (excluding *-template.fifthtry.site) found",
    )?;

    crate::helpers::run_fastn_serve(
        &ui_dir,
        &["--trace", "serve", "--port", "8002", "--offline"],
        "ui",
    )
}
