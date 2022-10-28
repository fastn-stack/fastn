/// `find_root_for_file()` starts with the given path, which is the current directory where the
/// application started in, and goes up till it finds a folder that contains `FPM.ftd` file.
/// TODO: make async
pub(crate) fn find_root_for_file(
    dir: &camino::Utf8Path,
    file_name: &str,
) -> Option<camino::Utf8PathBuf> {
    if dir.join(file_name).exists() {
        Some(dir.into())
    } else {
        if let Some(p) = dir.parent() {
            return find_root_for_file(p, file_name);
        };
        None
    }
}

pub async fn fpm_doc(path: &camino::Utf8Path) -> fpm::Result<ftd::p2::Document> {
    {
        let doc = tokio::fs::read_to_string(path);
        let lib = fpm::FPMLibrary::default();
        match fpm::doc::parse_ftd("FPM", doc.await?.as_str(), &lib) {
            Ok(v) => Ok(v),
            Err(e) => {
                return Err(fpm::Error::PackageError {
                    message: format!("failed to parse FPM.ftd 3: {:?}", &e),
                });
            }
        }
    }
}
