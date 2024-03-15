use snafu::ResultExt;

pub async fn from_fastn_doc(
    ds: &fastn_ds::DocumentStore,
    fastn_path: &fastn_ds::Path,
) -> fastn_core::Result<fastn_core::Package> {
    let doc = ds.read_to_string(fastn_path).await?;
    let lib = fastn_core::FastnLibrary::default();
    let fastn_doc = match fastn_core::doc::parse_ftd("fastn", doc.as_str(), &lib) {
        Ok(v) => Ok(v),
        Err(e) => Err(fastn_core::Error::PackageError {
            message: format!("failed to parse FASTN.ftd 3: {:?}", &e),
        }),
    }?;
    let package = fastn_core::Package::from_fastn_doc(ds, &fastn_doc)?;

    Ok(package)
}

pub async fn read_current_package(
    ds: &fastn_ds::DocumentStore,
) -> fastn_core::Result<fastn_core::Package> {
    let fastn_path = fastn_ds::Path::new("FASTN.ftd");

    from_fastn_doc(ds, &fastn_path).await
}

pub(crate) async fn download_archive(
    url: String,
) -> fastn_core::Result<zip::ZipArchive<std::io::Cursor<bytes::Bytes>>> {
    use std::io::Seek;

    let zipball = fastn_core::http::http_get(&url).await?;
    let mut zipball_cursor = std::io::Cursor::new(zipball);
    zipball_cursor.seek(std::io::SeekFrom::Start(0))?;
    let archive = zip::ZipArchive::new(zipball_cursor)?;
    Ok(archive)
}

pub(crate) fn read_manifest(
    bytes: &[u8],
    package_name: &str,
) -> Result<fastn_core::Manifest, fastn_update::ManifestError> {
    let manifest: fastn_core::Manifest =
        serde_json::de::from_slice(bytes).context(fastn_update::DeserializeManifestSnafu {
            package: package_name,
        })?;

    Ok(manifest)
}

/// Download manifest of the package `<package-name>/manifest.json`
/// Resolve to `fastn_core::Manifest` struct
pub(crate) async fn get_manifest(
    package_name: &str,
) -> Result<(fastn_core::Manifest, bytes::Bytes), fastn_update::ManifestError> {
    let manifest_bytes = fastn_core::http::http_get(&format!(
        "https://{}/{}",
        package_name,
        fastn_core::manifest::MANIFEST_FILE
    ))
        .await
        .context(fastn_update::DownloadManifestSnafu {
            package: package_name,
        })?;
    let manifest = read_manifest(&manifest_bytes, package_name)?;

    Ok((manifest, manifest_bytes))
}

pub(crate) async fn resolve_dependency_package(
    ds: &fastn_ds::DocumentStore,
    dependency: &fastn_core::package::dependency::Dependency,
    dependency_path: &fastn_ds::Path,
) -> Result<fastn_core::Package, fastn_update::DependencyError> {
    let mut dep_package = dependency.package.clone();
    let fastn_path = dependency_path.join("FASTN.ftd");
    dep_package
        .resolve(&fastn_path, ds)
        .await
        .context(fastn_update::ResolveDependencySnafu {
            package: dependency.package.name.clone(),
        })?;
    Ok(dep_package)
}
