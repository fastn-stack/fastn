/// `find_root_for_file()` starts with the given path, which is the current directory where the
/// application started in, and goes up till it finds a folder that contains `FASTN.ftd` file.
/// TODO: make async
#[async_recursion::async_recursion]
pub(crate) async fn find_root_for_file(
    dir: &fastn_ds::Path,
    file_name: &str,
    ds: &fastn_ds::DocumentStore,
) -> Option<fastn_ds::Path> {
    if ds.exists(&dir.join(file_name)).await {
        Some(dir.clone())
    } else {
        if let Some(p) = dir.parent() {
            return find_root_for_file(&p, file_name, ds).await;
        };
        None
    }
}

pub async fn fastn_doc(
    ds: &fastn_ds::DocumentStore,
    path: &fastn_ds::Path,
) -> fastn_core::Result<ftd::ftd2021::p2::Document> {
    let doc = ds.read_to_string(path).await?;
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

pub(crate) fn is_http_url(url: &str) -> bool {
    url.starts_with("http")
}
