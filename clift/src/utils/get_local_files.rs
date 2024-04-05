#[derive(Debug, thiserror::Error)]
pub enum GetLocalFilesError {
    #[error("CanNotReadFile {1}: {0}")]
    CantReadFile(std::io::Error, String),
}

pub async fn get_local_files(
    current_dir: &std::path::Path,
    folder: &str,
) -> Result<Vec<clift::api::ContentToUpload>, GetLocalFilesError> {
    let ignore_path = ignore::WalkBuilder::new(current_dir.join(folder))
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .git_global(true)
        .ignore(true)
        .parents(true)
        .build();

    let mut files = vec![];
    for path in ignore_path.flatten() {
        if path.path().is_dir() {
            continue;
        }

        let content = path_to_content(current_dir, path.path()).await?;

        if content.file_name.starts_with(".git/")
            || content.file_name.starts_with(".github/")
            || content.file_name.eq(".gitignore")
        {
            continue;
        }

        files.push(content);
    }

    Ok(files)
}

pub async fn path_to_content(
    current_dir: &std::path::Path,
    path: &std::path::Path,
) -> Result<clift::api::ContentToUpload, GetLocalFilesError> {
    let path_without_package_dir = path
        .to_str()
        .unwrap()
        .to_string()
        .trim_start_matches(current_dir.to_str().unwrap())
        .trim_start_matches('/')
        .to_string();

    let content = tokio::fs::read(path)
        .await
        .map_err(|e| GetLocalFilesError::CantReadFile(e, path.to_string_lossy().to_string()))?;

    Ok(clift::api::ContentToUpload {
        file_name: path_without_package_dir,
        // TODO: create the hash using file stream instead of reading entire
        //       file content into memory
        sha256_hash: clift::utils::generate_hash(&content),
        file_size: content.len(),
    })
}
