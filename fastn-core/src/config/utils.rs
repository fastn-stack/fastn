/// `find_root_for_file()` starts with the given path, which is the current directory where the
/// application started in, and goes up till it finds a folder that contains `FASTN.ftd` file.
/// TODO: make async
#[async_recursion::async_recursion]
pub(crate) async fn find_root_for_file(
    dir: &fastn_ds::Path,
    file_name: &str,
    ds: &fastn_ds::DocumentStore,
    session_id: &Option<String>,
) -> Option<fastn_ds::Path> {
    if ds.exists(&dir.join(file_name), session_id).await {
        Some(dir.clone())
    } else {
        if let Some(p) = dir.parent() {
            return find_root_for_file(&p, file_name, ds, session_id).await;
        };
        None
    }
}

pub async fn fastn_doc(
    ds: &fastn_ds::DocumentStore,
    path: &fastn_ds::Path,
    session_id: &Option<String>,
) -> fastn_core::Result<ftd::ftd2021::p2::Document> {
    let doc = ds.read_to_string(path, session_id).await?;
    let lib = fastn_core::FastnLibrary::default();
    match fastn_core::doc::parse_ftd("fastn", doc.as_str(), &lib) {
        Ok(v) => Ok(v),
        Err(e) => Err(fastn_core::Error::PackageError {
            message: format!("failed to parse FASTN.ftd 3: {:?}", &e),
        }),
    }
}

// if path starts with /-/package-name or -/package-name,
// so it trims the package and return the remaining url
#[tracing::instrument]
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
// It will return url with end-point, if package or dependency contains endpoints in them
// url: /-/<package-name>/api/ => (package-name, endpoints/api/, app or package config)
// url: /-/<package-name>/api/ => (package-name, endpoints/api/, app or package config)
pub fn get_clean_url(
    package: &fastn_core::Package,
    req_config: &fastn_core::RequestConfig,
    url: &str,
) -> fastn_core::Result<(
    url::Url,
    Option<String>,
    std::collections::HashMap<String, String>,
)> {
    if url.starts_with("http") {
        return Ok((
            url::Url::parse(url)?,
            None,
            std::collections::HashMap::new(),
        ));
    }

    let cow_1 = std::borrow::Cow::from(url);

    let url = if url.starts_with("/-/") || url.starts_with("-/") {
        cow_1
    } else {
        req_config
            .config
            .get_mountpoint_sanitized_path(url)
            .map(|(u, _, _, _)| u)
            .unwrap_or_else(|| cow_1) // TODO: Error possibly, in that return 404 from proxy
    };

    // This is for current package
    if let Some(remaining_url) = trim_package_name(url.as_ref(), package.name.as_str()) {
        if package.endpoints.is_empty() {
            return Err(fastn_core::Error::GenericError(format!(
                "package does not contain the endpoints: {:?}",
                package.name
            )));
        }

        let mut end_point = None;
        let mut mountpoint = None;
        for e in package.endpoints.iter() {
            if remaining_url.starts_with(e.mountpoint.as_str()) {
                mountpoint = Some(e.mountpoint.to_string());
                end_point = Some(e.endpoint.to_string());
                break;
            }
        }

        if end_point.is_none() {
            return Err(fastn_core::Error::GenericError(format!(
                "No mountpoint matched for url: {}",
                remaining_url.as_str()
            )));
        }

        return Ok((
            url::Url::parse(format!("{}{}", end_point.unwrap(), remaining_url).as_str())?,
            mountpoint,
            std::collections::HashMap::new(), // TODO:
        ));
    }

    // Handle logic for apps
    for app in package.apps.iter() {
        if let Some(ep) = &app.end_point {
            if let Some(remaining_url) = trim_package_name(url.as_ref(), app.package.name.as_str())
            {
                let mut app_conf = app.config.clone();
                if let Some(user_id) = &app.user_id {
                    app_conf.insert("user-id".to_string(), user_id.clone());
                }
                return Ok((
                    url::Url::parse(format!("{}{}", ep, remaining_url).as_str())?,
                    Some(app.mount_point.to_string()),
                    app_conf,
                ));
            }
        }
    }

    if let Some(e) = package
        .endpoints
        .iter()
        .find(|&endpoint| url.starts_with(&endpoint.mountpoint))
    {
        let endpoint_url = e.endpoint.trim_end_matches('/');
        let relative_path = url.trim_start_matches(&e.mountpoint);

        let mut full_url = format!("{}/{}", endpoint_url, relative_path);
        if package.name.ne(&req_config.config.package.name) {
            if let Some(endpoint_url) = endpoint_url.strip_prefix("wasm+proxy://") {
                full_url = format!(
                    "wasm+proxy://.packages/{}/{}/{}",
                    package.name, endpoint_url, relative_path
                );
            }
        }

        let mut mount_point = e.mountpoint.to_string();

        if let Some(value) = req_config
            .request
            .headers()
            .get(fastn_wasm::FASTN_APP_URL_HEADER)
        {
            mount_point = value.to_str().unwrap().to_string();
        }

        return Ok((
            url::Url::parse(&full_url)?,
            Some(mount_point),
            std::collections::HashMap::new(),
        ));
    }

    let msg = format!("http-processor: end-point not found url: {}", url);
    tracing::error!(msg = msg);
    Err(fastn_core::Error::GenericError(msg))
}

pub(crate) fn is_http_url(url: &str) -> bool {
    url.starts_with("http")
}
