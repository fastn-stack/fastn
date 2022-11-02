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
            Err(e) => Err(fpm::Error::PackageError {
                message: format!("failed to parse FPM.ftd 3: {:?}", &e),
            }),
        }
    }
}

// url can be start with /-/package-name/ or  -/package-name/
// It will return url with end-point, if package or dependency contains endpoint in them
// url: /-/<package-name>/api/ => endpoint/api/
// url: /-/<package-name>/api/ => endpoint/api/
pub fn get_clean_url(config: &fpm::Config, url: &str) -> fpm::Result<url::Url> {
    if url.starts_with("http") {
        return Ok(url::Url::parse(url)?);
    }

    // if path starts with /-/package-name or -/package-name, so it trim the package and return the remaining url
    fn path_start_with(path: &str, package_name: &str) -> Option<String> {
        let package_name = format!("-/{}", package_name.trim().trim_matches('/'));
        let path = path.trim().trim_start_matches('/');
        if path.starts_with(package_name.as_str()) {
            return Some(path.trim_start_matches(package_name.as_str()).to_string());
        }
        None
    }

    // This is for current package
    if let Some(remaining_url) = path_start_with(url, config.package.name.as_str()) {
        let end_point = match config.package.endpoint.as_ref() {
            Some(ep) => ep,
            None => {
                return Err(fpm::Error::GenericError(format!(
                    "package does not contain the endpoint: {:?}",
                    config.package.name
                )));
            }
        };
        return Ok(url::Url::parse(
            format!("{}{}", end_point, remaining_url).as_str(),
        )?);
    }

    // This is for dependency packages
    let deps_ep = config.package.dep_with_ep();
    for (dep, ep) in deps_ep {
        if let Some(remaining_url) = path_start_with(url, dep.name.as_str()) {
            return Ok(url::Url::parse(
                format!("{}{}", ep, remaining_url).as_str(),
            )?);
        }
    }

    Err(fpm::Error::GenericError(format!(
        "http-processor: end-point not found url: {}",
        url
    )))
}
