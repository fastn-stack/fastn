#![allow(dead_code)]

pub fn root() -> camino::Utf8PathBuf {
    camino::Utf8PathBuf::from_path_buf(
        std::env::current_dir().expect("Issue while reading current dir"),
    )
    .expect("issue while reading current dir")
}

pub fn home() -> camino::Utf8PathBuf {
    let home = match home::home_dir() {
        Some(h) => h,
        None => {
            eprintln!("Impossible to get your home directory");
            std::process::exit(1);
        }
    };
    camino::Utf8PathBuf::from_path_buf(home).expect("Issue while reading your home directory")
}

pub fn build_dir(ds: &fastn_ds::DocumentStore) -> fastn_ds::Path {
    ds.root().join(".build")
}

pub fn cw_id(ds: &fastn_ds::DocumentStore) -> fastn_ds::Path {
    ds.root().join(".fastn/cw-id")
}

pub fn sid(ds: &fastn_ds::DocumentStore) -> fastn_ds::Path {
    ds.home().join(".fastn/sid.json")
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
