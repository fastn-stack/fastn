pub fn root() -> camino::Utf8PathBuf {
    camino::Utf8PathBuf::from_path_buf(std::env::current_dir().unwrap()).unwrap()
}

pub fn build_dir() -> camino::Utf8PathBuf {
    root().join(".build")
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
