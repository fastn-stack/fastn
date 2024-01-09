extern crate self as fastn_update;

mod utils;

#[derive(thiserror::Error, Debug)]
pub enum ManifestError {
    #[error("Failed to download manifest.json for package '{package}'")]
    DownloadError {
        package: String,
        #[source]
        source: fastn_core::Error,
    },
    #[error("Failed to deserialize manifest.json for package '{package}'")]
    DeserializationError {
        package: String,
        #[source]
        source: serde_json::Error,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum ArchiveError {
    #[error("Failed to read archive for package '{package}'")]
    IoReadError {
        package: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to unpack archive for package '{package}'")]
    ZipError {
        package: String,
        #[source]
        source: zip::result::ZipError,
    },
    #[error("Failed to download archive for package '{package}'")]
    DownloadError {
        package: String,
        #[source]
        source: fastn_core::Error,
    },
    #[error("Failed to write archive content for package '{package}'")]
    DsWriteError {
        package: String,
        #[source]
        source: fastn_ds::WriteError,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum DependencyError {
    #[error("Failed to resolve dependency '{package}'")]
    ResolveError {
        package: String,
        #[source]
        source: fastn_core::Error,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum UpdateError {
    #[error("Manifest error: {0}")]
    Manifest(#[from] ManifestError),

    #[error("Archive error: {0}")]
    Archive(#[from] ArchiveError),

    #[error("Dependency error: {0}")]
    Dependency(#[from] DependencyError),
}

async fn resolve_dependency_package(
    ds: &fastn_ds::DocumentStore,
    dependency: &fastn_core::package::dependency::Dependency,
    dependency_path: &fastn_ds::Path,
    package: String,
) -> Result<fastn_core::Package, DependencyError> {
    let mut dep_package = dependency.package.clone();
    let fastn_path = dependency_path.join("FASTN.ftd");
    if let Err(e) = dep_package.resolve(&fastn_path, ds).await {
        return Err(DependencyError::ResolveError { package, source: e });
    }
    Ok(dep_package)
}

async fn update_dependencies(
    ds: &fastn_ds::DocumentStore,
    packages_root: &fastn_ds::Path,
    current_package: &fastn_core::Package,
    pb: &indicatif::ProgressBar,
) -> Result<(), UpdateError> {
    let mut stack = vec![current_package.clone()];
    let mut resolved = std::collections::HashSet::new();
    resolved.insert(current_package.name.to_string());

    while let Some(package) = stack.pop() {
        for dependency in package.dependencies {
            if resolved.contains(&dependency.package.name)
                || dependency.package.name.eq(&fastn_core::FASTN_UI_INTERFACE)
            {
                continue;
            }
            let package_name = dependency.package.name.clone();
            let dependency_path = &packages_root.join(&package_name);

            pb.set_message(format!("Resolving {}/manifest.json", &package_name));

            let manifest = get_manifest(package_name.clone()).await?;

            pb.set_message(format!("Downloading {} archive", &package_name));

            download_and_unpack_zip(
                ds,
                &packages_root.join(&package_name),
                &manifest,
                package_name.clone(),
            )
            .await?;

            let dep_package =
                resolve_dependency_package(ds, &dependency, dependency_path, package_name.clone())
                    .await?;
            resolved.insert(package_name.to_string());
            stack.push(dep_package);
            pb.inc_length(1);
        }

        pb.inc(1);
    }
    Ok(())
}

async fn download_and_unpack_zip(
    ds: &fastn_ds::DocumentStore,
    dependency_path: &fastn_ds::Path,
    manifest: &fastn_core::Manifest,
    package_name: String,
) -> Result<(), ArchiveError> {
    let mut archive = fastn_update::utils::download_archive(manifest.zip_url.clone())
        .await
        .map_err(|e| ArchiveError::DownloadError {
            package: package_name.clone(),
            source: e,
        })?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| ArchiveError::ZipError {
            package: package_name.clone(),
            source: e,
        })?;

        if entry.is_file() {
            let mut buffer = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut buffer).map_err(|e| {
                ArchiveError::IoReadError {
                    package: package_name.clone(),
                    source: e,
                }
            })?;
            if let Some(path) = entry.enclosed_name() {
                let path = path.to_str().unwrap();
                let path_without_prefix = match path.split_once(std::path::MAIN_SEPARATOR) {
                    Some((_, path)) => path,
                    None => path,
                };
                let output_path = &dependency_path.join(path_without_prefix);
                ds.write_content(output_path, buffer).await.map_err(|e| {
                    ArchiveError::DsWriteError {
                        package: package_name.clone(),
                        source: e,
                    }
                })?;
            }
        }
    }

    Ok(())
}

/// Download manifest of the package `<package-name>/manifest.json`
/// Resolve to `fastn_core::Manifest` struct
async fn get_manifest(package_name: String) -> Result<fastn_core::Manifest, ManifestError> {
    let manifest_bytes = fastn_core::http::http_get(&format!(
        "https://{}/{}",
        package_name,
        fastn_core::manifest::MANIFEST_FILE
    ))
    .await
    .map_err(|e| ManifestError::DownloadError {
        package: package_name.clone(),
        source: e,
    })?;
    let manifest = serde_json::de::from_slice(&manifest_bytes).map_err(|e| {
        ManifestError::DeserializationError {
            package: package_name.clone(),
            source: e,
        }
    })?;

    Ok(manifest)
}

pub async fn update(config: &fastn_core::Config) -> fastn_core::Result<()> {
    if let Err(fastn_ds::RemoveError::IOError(e)) =
        config.ds.remove(&config.ds.root().join(".packages")).await
    {
        match e.kind() {
            std::io::ErrorKind::NotFound => {}
            _ => return Err(e.into()),
        }
    };

    let c = fastn_core::Config::read_current(false).await?;
    if c.package.dependencies.is_empty() {
        println!("No dependencies to update.");
        return Ok(());
    }

    let spinner_style =
        indicatif::ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    let current_package = &config.package;
    let pb = indicatif::ProgressBar::new(current_package.dependencies.len() as u64);
    pb.set_style(spinner_style);
    pb.set_prefix("Updating packages");

    if let Err(e) =
        update_dependencies(&config.ds, &config.packages_root, current_package, &pb).await
    {
        pb.set_prefix("Update failed");
        pb.abandon_with_message(e.to_string());

        return Ok(());
    }

    pb.set_prefix("Updated");

    match c.package.dependencies.len() {
        1 => pb.set_message("package dependency"),
        n => pb.set_message(format!("{} dependencies.", n)),
    }

    pb.finish();

    Ok(())
}
