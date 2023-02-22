#[derive(thiserror::Error, Debug)]
pub enum CreateError {
    #[error("BuildDirNotFound: {}", _0)]
    BuildDirNotFound(String),
    #[error("TejarCreateError: {}", _0)]
    TejarCreateError(#[from] tejar::error::CreateError),
}

#[derive(thiserror::Error, Debug)]
pub enum UpdateError {}

pub async fn create() -> Result<(), CreateError> {
    let root = build_dir();
    if !root.exists() {
        return Err(CreateError::BuildDirNotFound(
            "Run `fastn build` to create a .build directory before running this".to_string(),
        ));
    }
    println!("{}", root);
    let (list_file, data_file) = create_tejar(root.as_path()).await?;
    println!("{} {}", list_file, data_file);
    // call fastn build
    // read the content of the .build folder
    // pass this content to tejar and create two files LIST and DATA
    // call /api/create/ by passing the content of the LIST and META
    // call /api/upload-new-package by passing the missing entries and DATA
    // save package key and at home folder
    // show the subdomain to user or open browser directly
    println!("publish-static create called");
    Ok(())
}

pub async fn update() -> Result<(), UpdateError> {
    println!("publish-static update called");
    Ok(())
}

fn build_dir() -> camino::Utf8PathBuf {
    camino::Utf8PathBuf::from_path_buf(std::env::current_dir().unwrap())
        .unwrap()
        .join(".build")
}

pub async fn create_tejar(
    root: &camino::Utf8Path,
) -> Result<(camino::Utf8PathBuf, camino::Utf8PathBuf), tejar::error::CreateError> {
    let files: _ = walkdir_util(root)
        .into_iter()
        .map(|file| tejar::create::InputFile {
            path: file.path.strip_prefix(&root).unwrap().to_path_buf(),
            content_type: file.content_type,
            gzip: file.gzip,
        })
        .collect::<Vec<_>>();
    tejar::create::create(&root, files.as_slice())
}

pub fn walkdir_util(root: &camino::Utf8Path) -> Vec<tejar::create::InputFile> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| {
            if !e.as_ref().unwrap().file_type().is_dir() {
                Some(e)
            } else {
                None
            }
        })
        .map(|e| {
            camino::Utf8PathBuf::from(e.as_ref().unwrap().path().as_os_str().to_str().unwrap())
        })
        .map(|x| tejar::create::InputFile {
            content_type: mime_guess::from_path(&x)
                .first_or(mime_guess::mime::TEXT_PLAIN)
                .to_string(),
            path: x,
            gzip: false,
        })
        .collect::<Vec<tejar::create::InputFile>>()
}
