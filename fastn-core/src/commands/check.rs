pub const INDEX_FILE: &str = "index.html";
pub const BUILD_FOLDER: &str = ".build";
pub const IGNORED_DIRECTORIES: [&str; 4] = ["-", "images", "static", "assets"];

pub async fn post_build_check(config: &fastn_core::Config) -> fastn_core::Result<()> {
    let build_path = config.ds.root().join(BUILD_FOLDER);
    let build_directory = build_path.as_str().to_string();
    println!("Post build index assertion started ...");

    if build_path.is_dir() {
        if !build_path.join(INDEX_FILE).exists() {
            return Err(fastn_core::Error::NotFound(format!(
                "Couldn't find {} in package root folder",
                INDEX_FILE
            )));
        }
        check_index_in_folders(build_path, build_directory.as_str())
            .await
            .map_err(|e| fastn_core::Error::GenericError(e.to_string()))?;
    }

    Ok(())
}

#[async_recursion::async_recursion]
async fn check_index_in_folders(
    folder: camino::Utf8PathBuf,
    build_path: &str,
) -> Result<(), fastn_core::Error> {
    use colored::Colorize;
    let mut file_count = 0;
    let mut has_ignored_directory = false;

    if folder.is_dir() {
        // Todo: Use config.ds.read_dir instead of tokio::fs::read_dir
        let mut entries = tokio::fs::read_dir(&folder).await?;
        while let Some(current_entry) = entries.next_entry().await? {
            let current_entry_path = current_entry.path();
            let entry_path = camino::Utf8PathBuf::from_path_buf(current_entry_path)
                .unwrap_or_else(|_| panic!("failed to read path: {:?}", current_entry.path()));

            let is_ignored_directory = entry_path.is_dir() && is_ignored_directory(&entry_path);

            if !has_ignored_directory {
                has_ignored_directory = is_ignored_directory;
            }
            if entry_path.is_file() {
                file_count += 1;
            }
            if entry_path.is_dir() && !is_ignored_directory {
                check_index_in_folders(entry_path, build_path).await?;
            }
        }
        if file_count > 0 || !has_ignored_directory {
            let index_html_path = folder.join(INDEX_FILE);
            if !index_html_path.exists() {
                let warning_msg = format!(
                    "Warning: Directory {:?} does not have an index.html file.",
                    folder.as_str().trim_start_matches(build_path)
                );
                println!("{}", warning_msg.yellow());
            }
        }
    }
    Ok(())
}

fn is_ignored_directory(path: &camino::Utf8PathBuf) -> bool {
    IGNORED_DIRECTORIES.iter().any(|dir| path.ends_with(dir))
}
