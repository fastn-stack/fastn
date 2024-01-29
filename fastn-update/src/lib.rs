use snafu::prelude::*;

extern crate self as fastn_update;

mod utils;

#[derive(Snafu, Debug)]
pub enum ManifestError {
    #[snafu(display("Failed to download manifest.json for package '{package}'"))]
    DownloadManifest {
        package: String,
        source: fastn_core::Error,
    },
    #[snafu(display("Missing archive url in manifest.json for package '{package}'"))]
    NoZipUrl { package: String },
    #[snafu(display("Failed to deserialize manifest.json for package '{package}'"))]
    DeserializeManifest {
        package: String,
        source: serde_json::Error,
    },
    #[snafu(display("Failed to read manifest content for package '{package}'"))]
    ReadManifest {
        package: String,
        source: fastn_ds::ReadError,
    },
}

#[derive(Snafu, Debug)]
pub enum ArchiveError {
    #[snafu(display("Failed to read archive for package '{package}'"))]
    ReadArchive {
        package: String,
        source: std::io::Error,
    },
    #[snafu(display(
        "Failed to read the archive entry path for the entry '{name}' in the package '{package}'"
    ))]
    ArchiveEntryPathError { package: String, name: String },
    #[snafu(display("Failed to unpack archive for package '{package}'"))]
    ArchiveEntryRead {
        package: String,
        source: zip::result::ZipError,
    },
    #[snafu(display("Failed to download archive for package '{package}'"))]
    DownloadArchive {
        package: String,
        source: fastn_core::Error,
    },
    #[snafu(display("Failed to write archive content for package '{package}'"))]
    WriteArchiveContent {
        package: String,
        source: fastn_ds::WriteError,
    },
}

#[derive(Snafu, Debug)]
pub enum DependencyError {
    #[snafu(display("Failed to resolve dependency '{package}'"))]
    ResolveDependency {
        package: String,
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
    package_name: &str,
    dependency_path: &fastn_ds::Path,
) -> Result<fastn_core::Package, DependencyError> {
    let fastn_path = dependency_path.join("FASTN.ftd");
    let dep_package = from_fastn_doc(ds, &fastn_path)
        .await
        .context(ResolveDependencySnafu {
            package: package_name,
        })?;
    Ok(dep_package)
}

async fn update_dependencies(
    ds: &fastn_ds::DocumentStore,
    packages_root: fastn_ds::Path,
    current_package: &fastn_core::Package,
    pb: &indicatif::ProgressBar,
    offline: bool,
) -> Result<(), UpdateError> {
    let mut stack = vec![current_package.clone()];
    let mut resolved = std::collections::HashSet::new();
    resolved.insert(current_package.name.to_string());

    while let Some(package) = stack.pop() {
        for dependency in package.dependencies {
            if resolved.contains(&dependency.package) {
                continue;
            }

            let package_name = dependency.package.clone();
            let dependency_path = &packages_root.join(&package_name);

            if offline {
                if package_name.eq(&fastn_core::FASTN_UI_INTERFACE) {
                    resolved.insert(package_name.to_string());
                    continue;
                }

                let dep_package =
                    resolve_dependency_package(ds, &package_name, dependency_path).await?;
                resolved.insert(package_name.to_string());
                pb.inc_length(1);
                stack.push(dep_package);
                continue;
            }

            pb.set_message(format!("Resolving {}/manifest.json", &package_name));

            let (manifest, manifest_bytes) = get_manifest(&package_name).await?;

            let manifest_path = dependency_path.join(fastn_core::manifest::MANIFEST_FILE);

            // Download the archive if:
            // 1. The package does not already exist
            // 2. The checksums of the downloaded package manifest and the existing manifest do not match
            let should_download_archive = if !ds.exists(dependency_path).await {
                true
            } else {
                let existing_manifest_bytes =
                    ds.read_content(&manifest_path)
                        .await
                        .context(ReadManifestSnafu {
                            package: package_name.clone(),
                        })?;
                let existing_manifest: fastn_core::Manifest =
                    read_manifest(&existing_manifest_bytes, &package_name)?;

                existing_manifest.checksum.ne(manifest.checksum.as_str())
            };

            if !should_download_archive {
                pb.set_message(format!(
                    "Skipping download for package \"{}\" as it already exists.",
                    &package_name
                ));
            } else {
                pb.set_message(format!("Downloading {} archive", &package_name));

                download_and_unpack_zip(
                    ds,
                    &packages_root.join(&package_name),
                    &manifest,
                    &package_name,
                    pb,
                )
                .await?;

                ds.write_content(&manifest_path, manifest_bytes)
                    .await
                    .context(WriteArchiveContentSnafu {
                        package: package_name.clone(),
                    })?
            }

            if package_name.eq(&fastn_core::FASTN_UI_INTERFACE) {
                resolved.insert(package_name.to_string());
                continue;
            }

            let dep_package =
                resolve_dependency_package(ds, &package_name, dependency_path).await?;
            resolved.insert(package_name.to_string());
            pb.inc_length(1);
            stack.push(dep_package);
        }

        pb.inc(1);
    }
    Ok(())
}

async fn download_and_unpack_zip(
    ds: &fastn_ds::DocumentStore,
    dependency_path: &fastn_ds::Path,
    manifest: &fastn_core::Manifest,
    package_name: &str,
    pb: &indicatif::ProgressBar,
) -> Result<(), ArchiveError> {
    let mut archive = fastn_update::utils::download_archive(manifest.zip_url.clone())
        .await
        .context(DownloadArchiveSnafu {
            package: package_name,
        })?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).context(ArchiveEntryReadSnafu {
            package: package_name,
        })?;

        if entry.is_file() {
            let mut buffer = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut buffer).context(ReadArchiveSnafu {
                package: package_name,
            })?;
            let path = entry.enclosed_name().context(ArchiveEntryPathSnafu {
                package: package_name,
                name: entry.name(),
            })?;
            let path = path.to_str().unwrap();
            let path_without_prefix = match path.split_once(std::path::MAIN_SEPARATOR) {
                Some((_, path)) => path,
                None => path,
            };
            if manifest.files.get(path_without_prefix).is_none() {
                continue;
            }
            let output_path = &dependency_path.join(path_without_prefix);
            ds.write_content(output_path, buffer)
                .await
                .context(WriteArchiveContentSnafu {
                    package: package_name,
                })?;
            pb.tick();
        }
    }

    Ok(())
}

fn read_manifest(bytes: &[u8], package_name: &str) -> Result<fastn_core::Manifest, ManifestError> {
    let manifest: fastn_core::Manifest =
        serde_json::de::from_slice(bytes).context(DeserializeManifestSnafu {
            package: package_name,
        })?;

    Ok(manifest)
}

/// Download manifest of the package `<package-name>/manifest.json`
/// Resolve to `fastn_core::Manifest` struct
async fn get_manifest(
    package_name: &str,
) -> Result<(fastn_core::Manifest, Vec<u8>), ManifestError> {
    let manifest_bytes = fastn_core::http::http_get(&format!(
        "https://{}/{}",
        package_name,
        fastn_core::manifest::MANIFEST_FILE
    ))
    .await
    .context(DownloadManifestSnafu {
        package: package_name,
    })?;
    let manifest = read_manifest(&manifest_bytes, package_name)?;

    Ok((manifest, manifest_bytes))
}

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

#[tracing::instrument(skip_all)]
pub async fn update(ds: &fastn_ds::DocumentStore, offline: bool) -> fastn_core::Result<()> {
    let packages_root = ds.root().join(".packages");
    let current_package = read_current_package(ds).await?;

    if current_package.dependencies.is_empty() {
        println!("No dependencies to update.");
        return Ok(());
    }

    let spinner_style =
        indicatif::ProgressStyle::with_template("{prefix:.bold.dim} [{pos}/{len}] {spinner} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");

    let pb = indicatif::ProgressBar::new(current_package.dependencies.len() as u64);
    pb.set_style(spinner_style);
    pb.set_prefix("Updating dependencies");

    if let Err(e) = update_dependencies(ds, packages_root, &current_package, &pb, offline).await {
        return Err(fastn_core::Error::UpdateError {
            message: e.to_string(),
        });
    }

    pb.finish_and_clear();

    match current_package.dependencies.len() {
        1 => println!("Updated package dependency"),
        n => println!("Updated {} dependencies.", n),
    }

    Ok(())
}
