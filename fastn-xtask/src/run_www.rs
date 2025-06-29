pub fn run_www() -> fastn_core::Result<()> {
    println!("Running WWW site");

    let www_dir = crate::helpers::find_directory(
        |name| name.ends_with(".fifthtry-community.com"),
        "No directory matching '*.fifthtry-community.com' found",
    )?;

    crate::helpers::run_fastn_serve(
        &www_dir,
        &["--trace", "serve", "--port", "8003", "--offline"],
        "www",
    )
}
