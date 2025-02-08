use snafu::ResultExt;

pub async fn from_fastn_doc(
    ds: &fastn_ds::DocumentStore,
    fastn_path: &fastn_ds::Path,
) -> fastn_core::Result<fastn_core::Package> {
    let doc = ds.read_to_string(fastn_path, &None).await?;
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
    ds: &fastn_ds::DocumentStore,
    url: &str,
) -> fastn_core::Result<zip::ZipArchive<std::io::Cursor<bytes::Bytes>>> {
    use std::io::Seek;

    let zipball = fastn_core::http::http_get(ds, url).await?;
    let mut zipball_cursor = std::io::Cursor::new(zipball);
    zipball_cursor.seek(std::io::SeekFrom::Start(0))?;
    let archive = zip::ZipArchive::new(zipball_cursor)?;
    Ok(archive)
}

pub(crate) async fn resolve_dependency_package(
    ds: &fastn_ds::DocumentStore,
    dependency: &fastn_core::package::dependency::Dependency,
    dependency_path: &fastn_ds::Path,
) -> Result<fastn_core::Package, fastn_update::DependencyError> {
    let mut dep_package = dependency.package.clone();
    let fastn_path = dependency_path.join("FASTN.ftd");
    dep_package.resolve(&fastn_path, ds, &None).await.context(
        fastn_update::ResolveDependencySnafu {
            package: dependency.package.name.clone(),
        },
    )?;
    Ok(dep_package)
}
