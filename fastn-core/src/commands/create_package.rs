async fn template_contents(
    project_name: &str,
    download_base_url: Option<&str>,
) -> (String, String, String) {
    let ftd = format!(
        r#"-- import: fastn

-- fastn.package: {}
{}
"#,
        project_name,
        download_base_url
            .map(|v| format!("download-base-url: {}", v))
            .unwrap_or_default()
    );
    let index = "-- ftd.text: Hello world".to_string();
    let gitignore = r#".build/
.env
    "#
    .to_string();

    (ftd, index, gitignore)
}

pub async fn create_package(
    name: &str,
    path: Option<&str>,
    download_base_url: Option<&str>,
) -> fastn_core::Result<()> {
    use colored::Colorize;

    let current_dir: camino::Utf8PathBuf = std::env::current_dir()?.canonicalize()?.try_into()?;
    let ds = fastn_ds::DocumentStore::new(current_dir);

    let base_path = {
        match std::env::current_dir() {
            Ok(bp) => match bp.to_str() {
                Some(fbp) => fbp.to_string(),
                None => "None".to_string(),
            },
            Err(_) => panic!("Error cannot find the current working directory!!"),
        }
    };

    // Not using config for base path as it requires manifest or FASTN.ftd file for building and will throw error
    // and since this command should work from anywhere within the system
    // so we dont need to rely on config for using it

    // name is a required field so it will always be some defined string (cant be None)
    // name can be any package url or standard project name
    // path is an optional field and if no path is provided then current directory is used

    let final_dir = {
        match path {
            Some(p) => fastn_ds::Path::new(base_path).join(p).join(name),
            None => fastn_ds::Path::new(base_path).join(name),
        }
    };

    // Create all directories if not present

    let (tmp_fastn, tmp_index, tmp_gitignore) = template_contents(name, download_base_url).await;

    fastn_core::utils::update(&final_dir.join("FASTN.ftd"), tmp_fastn.as_bytes(), &ds).await?;
    fastn_core::utils::update(&final_dir.join("index.ftd"), tmp_index.as_bytes(), &ds).await?;
    fastn_core::utils::update(&final_dir.join(".gitignore"), tmp_gitignore.as_bytes(), &ds).await?;

    println!(
        "fastn Package Created: {}\nPath: {}",
        name.green(),
        final_dir.to_string().yellow()
    );

    Ok(())
}
