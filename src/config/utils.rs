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

// if path starts with /-/package-name or -/package-name,
// so it trims the package and return the remaining url
pub fn trim_package_name(path: &str, package_name: &str) -> Option<String> {
    let package_name1 = format!("-/{}", package_name.trim().trim_matches('/'));
    let path = path.trim().trim_start_matches('/');
    if path.starts_with(package_name1.as_str()) {
        return Some(path.trim_start_matches(package_name1.as_str()).to_string());
    }

    let package_name2 = format!("/-/{}", package_name.trim().trim_matches('/'));
    if path.starts_with(package_name2.as_str()) {
        return Some(path.trim_start_matches(package_name2.as_str()).to_string());
    }

    None
}

// url can be start with /-/package-name/ or  -/package-name/
// It will return url with end-point, if package or dependency contains endpoint in them
// url: /-/<package-name>/api/ => (package-name, endpoint/api/, app or package config)
// url: /-/<package-name>/api/ => (package-name, endpoint/api/, app or package config)
pub fn get_clean_url(
    config: &fpm::Config,
    url: &str,
) -> fpm::Result<(
    Option<String>,
    url::Url,
    std::collections::HashMap<String, String>,
)> {
    if url.starts_with("http") {
        return Ok((
            None,
            url::Url::parse(url)?,
            std::collections::HashMap::new(),
        ));
    }

    let url = if url.starts_with("/-/") || url.starts_with("-/") {
        url.to_string()
    } else {
        config
            .get_mountpoint_sanitized_path(&config.package, url)
            .map(|(u, _, _, _)| u)
            .unwrap_or_else(|| url.to_string()) // TODO: Error possibly, in that return 404 from proxy
    };

    // This is for current package
    if let Some(remaining_url) = trim_package_name(url.as_str(), config.package.name.as_str()) {
        let end_point = match config.package.endpoint.as_ref() {
            Some(ep) => ep,
            None => {
                return Err(fpm::Error::GenericError(format!(
                    "package does not contain the endpoint: {:?}",
                    config.package.name
                )));
            }
        };
        return Ok((
            Some(config.package.name.to_string()),
            url::Url::parse(format!("{}{}", end_point, remaining_url).as_str())?,
            std::collections::HashMap::new(), // TODO:
        ));
    }

    // Handle logic for apps
    for app in config.package.apps.iter() {
        if let Some(ep) = &app.end_point {
            if let Some(remaining_url) = trim_package_name(url.as_str(), app.package.name.as_str())
            {
                return Ok((
                    Some(app.package.name.to_string()),
                    url::Url::parse(format!("{}{}", ep, remaining_url).as_str())?,
                    app.config.clone(),
                ));
            }
        }
    }

    Err(fpm::Error::GenericError(format!(
        "http-processor: end-point not found url: {}",
        url
    )))
}
