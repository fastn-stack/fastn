pub const INDEX_FILE: &str = "index.html";
pub const BUILD_FOLDER: &str = ".build";

pub async fn post_build_check(config: &fastn_core::Config) -> fastn_core::Result<()> {
    let build_path = config.root.join(BUILD_FOLDER);

    if build_path.is_dir() {
        if !build_path.join(INDEX_FILE).exists() {
            return Err(fastn_core::Error::NotFound(format!(
                "Couldn't find {} in package root folder",
                INDEX_FILE
            )));
        }
        check_index_in_folders(build_path)
            .await
            .map_err(|e| fastn_core::Error::GenericError(e.to_string()))?;
    }

    Ok(())
}

#[async_recursion::async_recursion]
async fn check_index_in_folders(folder: camino::Utf8PathBuf) -> Result<(), fastn_core::Error> {
    use colored::Colorize;

    if folder.is_dir() {
        let mut entries = tokio::fs::read_dir(folder).await?;
        while let Some(current_entry) = entries.next_entry().await? {
            let entry_path = camino::Utf8PathBuf::from_path_buf(current_entry.path())
                .expect(format!("failed to read path: {:?}", current_entry.path()).as_str());

            if entry_path.is_dir() {
                let index_html_path = entry_path.join(INDEX_FILE);
                if !index_html_path.exists() {
                    let warning_msg = format!(
                        "Warning: Folder {:?} does not have an index.html file.",
                        entry_path
                    );
                    println!("{}", warning_msg.yellow());
                }

                check_index_in_folders(entry_path).await?;
            }
        }
    }
    Ok(())
}
