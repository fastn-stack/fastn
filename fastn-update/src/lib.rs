#![deny(unused_crate_dependencies)]

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

#[derive(Debug)]
pub enum CheckError {
    WriteDuringCheck { package: String, file: String },
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::*;

        match self {
            CheckError::WriteDuringCheck { package, file } => {
                write!(
                    f,
                    "{}\n\nThe package '{}' is out of sync with the FASTN.ftd file.\n\nFile: '{}'\nOperation: {}",
                    "Error: Out of Sync Package".red().bold(),
                    package,
                    file,
                    "Write Attempt".yellow()
                )
            }
        }
    }
}

impl std::error::Error for CheckError {}

#[derive(thiserror::Error, Debug)]
pub enum UpdateError {
    #[error("Manifest error: {0}")]
    Manifest(#[from] ManifestError),
    #[error("Archive error: {0}")]
    Archive(#[from] ArchiveError),
    #[error("Dependency error: {0}")]
    Dependency(#[from] DependencyError),
    #[error("Check error: {0}")]
    Check(#[from] CheckError),
    #[error("Config error: {0}")]
    Config(#[from] fastn_core::config_temp::Error),
    #[error("Invalid package {0}")]
    InvalidPackage(String),
}

async fn update_dependencies(
    ds: &fastn_ds::DocumentStore,
    packages_root: fastn_ds::Path,
    current_package: &fastn_core::Package,
    pb: &indicatif::ProgressBar,
    check: bool,
) -> Result<usize, UpdateError> {
    let mut stack = vec![current_package.clone()];
    let mut resolved = std::collections::HashSet::new();
    resolved.insert(current_package.name.to_string());
    let mut all_packages: Vec<(String, fastn_core::Manifest)> = vec![];
    let mut updated_packages: usize = 0;

    while let Some(package) = stack.pop() {
        for dependency in package.dependencies {
            if resolved.contains(&dependency.package.name) {
                continue;
            }

            let dep_package = &dependency.package;
            let package_name = dep_package.name.clone();
            let dependency_path = &packages_root.join(&package_name);

            if is_fifthtry_site_package(package_name.as_str()) {
                update_fifthtry_site_dependency(
                    &dependency,
                    ds,
                    packages_root.clone(),
                    pb,
                    &mut updated_packages,
                    &mut all_packages,
                    check,
                )
                .await?;
            } else {
                update_github_dependency(
                    &dependency,
                    ds,
                    packages_root.clone(),
                    pb,
                    &mut updated_packages,
                    &mut all_packages,
                    check,
                )
                .await?;
            }

            if package_name.eq(&fastn_core::FASTN_UI_INTERFACE) {
                resolved.insert(package_name.to_string());
                continue;
            }

            let dep_package =
                utils::resolve_dependency_package(ds, &dependency, dependency_path).await?;
            resolved.insert(package_name.to_string());
            pb.inc_length(1);
            stack.push(dep_package);
        }

        pb.inc(1);
    }

    fastn_core::ConfigTemp::write(
        ds,
        current_package.name.clone(),
        all_packages.into_iter().collect(),
    )
    .await?;

    Ok(updated_packages)
}

async fn update_github_dependency(
    dependency: &fastn_core::package::dependency::Dependency,
    ds: &fastn_ds::DocumentStore,
    packages_root: fastn_ds::Path,
    pb: &indicatif::ProgressBar,
    updated_packages: &mut usize,
    all_packages: &mut Vec<(String, fastn_core::Manifest)>,
    check: bool,
) -> Result<(), fastn_update::UpdateError> {
    let dep_package = &dependency.package;
    let package_name = dep_package.name.clone();
    let dependency_path = &packages_root.join(&package_name);

    if is_fifthtry_site_package(package_name.as_str()) {
        return Err(fastn_update::UpdateError::InvalidPackage(format!(
            "{package_name} is a fifthtry site."
        )));
    }

    pb.set_message(format!("Resolving {}/manifest.json", &package_name));
    let (manifest, manifest_bytes) = utils::get_manifest(ds, &package_name).await?;

    let manifest_path = dependency_path.join(fastn_core::manifest::MANIFEST_FILE);

    // Download the archive if:
    // 1. The package does not yet exist
    // 2. The checksums of the downloaded package manifest
    //    and the existing manifest does not match
    let should_download_archive = if !ds.exists(dependency_path, &None).await {
        true
    } else {
        let existing_manifest_bytes =
            ds.read_content(&manifest_path, &None)
                .await
                .context(ReadManifestSnafu {
                    package: package_name.clone(),
                })?;
        let existing_manifest: fastn_core::Manifest =
            utils::read_manifest(&existing_manifest_bytes, &package_name)?;

        existing_manifest.checksum.ne(manifest.checksum.as_str())
    };

    if !should_download_archive {
        pb.set_message(format!(
            "Skipping download for package \"{}\" as it already exists.",
            &package_name
        ));
    } else {
        pb.set_message(format!("Downloading {} archive", &package_name));

        download_unpack_zip_and_get_manifest(
            dependency_path.clone(),
            manifest.zip_url.as_str(),
            ds,
            package_name.as_str(),
            Some(&manifest),
            pb,
            check,
        )
        .await?;

        write_archive_content(ds, &manifest_path, &manifest_bytes, &package_name, check).await?;

        *updated_packages += 1;
    }

    all_packages.push((package_name.to_string(), manifest));
    Ok(())
}

async fn update_fifthtry_site_dependency(
    dependency: &fastn_core::package::dependency::Dependency,
    ds: &fastn_ds::DocumentStore,
    packages_root: fastn_ds::Path,
    pb: &indicatif::ProgressBar,
    updated_packages: &mut usize,
    all_packages: &mut Vec<(String, fastn_core::Manifest)>,
    check: bool,
) -> Result<(), fastn_update::UpdateError> {
    let dep_package = &dependency.package;
    let package_name = dep_package.name.clone();

    if !is_fifthtry_site_package(package_name.as_str()) {
        return Err(fastn_update::UpdateError::InvalidPackage(format!(
            "{package_name} is not a fifthtry site."
        )));
    }

    let site_slug = package_name.trim_end_matches(".fifthtry.site");
    let dependency_path = &packages_root.join(&package_name);
    let site_zip_url = fastn_core::utils::fifthtry_site_zip_url(site_slug);

    let manifest = download_unpack_zip_and_get_manifest(
        dependency_path.clone(),
        site_zip_url.as_str(),
        ds,
        package_name.as_str(),
        None,
        pb,
        check,
    )
    .await?;

    all_packages.push((package_name.to_string(), manifest));
    *updated_packages += 1;
    Ok(())
}

async fn download_unpack_zip_and_get_manifest(
    dependency_path: fastn_ds::Path,
    zip_url: &str,
    ds: &fastn_ds::DocumentStore,
    package_name: &str,
    manifest: Option<&fastn_core::Manifest>,
    pb: &indicatif::ProgressBar,
    check: bool,
) -> Result<fastn_core::Manifest, fastn_update::UpdateError> {
    use sha2::digest::FixedOutput;
    use sha2::Digest;

    let mut files: std::collections::BTreeMap<String, fastn_core::manifest::File> = match manifest {
        Some(manifest) => manifest.files.clone(),
        None => Default::default(),
    };

    let mut archive = utils::download_archive(ds, zip_url.to_string())
        .await
        .context(DownloadArchiveSnafu {
            package: package_name,
        })?;

    let mut hasher = sha2::Sha256::new();
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
            let path_string = path.to_string_lossy().into_owned();
            let path_normalized = path_string.replace('\\', "/");

            let file_name = path_normalized.clone();
            let file_hash = fastn_core::utils::generate_hash(buffer.clone());
            let file_size = buffer.len();

            // For package like `fifthtry.github.io/package-doc`, github zip is a folder called
            // `package-doc-<commit-id>` which contains all files, so path for `FASTN.ftd` becomes
            // `package-doc-<commit-id>/FASTN.ftd` while fifthtry package zip doesn't have any
            // such folder, so path becomes `FASTN.ftd`
            let path_without_prefix = if manifest.is_some() {
                // For github
                match path_normalized.split_once('/') {
                    Some((_, path)) => path,
                    None => &path_normalized,
                }
            } else {
                // For fifthtry packages
                &path_normalized
            };

            if manifest.is_some() {
                if !files.contains_key(path_without_prefix) {
                    continue;
                }
            } else {
                hasher.update(&buffer);
                // Creating file entry for manifest using archive files
                files.insert(
                    file_name.clone(),
                    fastn_core::manifest::File::new(file_name, file_hash, file_size),
                );
            }
            let output_path = &dependency_path.join(path_without_prefix);
            write_archive_content(ds, output_path, &buffer, package_name, check).await?;
            pb.tick();
        }
    }

    let manifest = match manifest {
        Some(manifest) => manifest.clone(),
        None => {
            let checksum = format!("{:X}", hasher.finalize_fixed());
            fastn_core::Manifest::new(files, zip_url.to_string(), checksum)
        }
    };

    Ok(manifest)
}

fn is_fifthtry_site_package(package_name: &str) -> bool {
    package_name.ends_with(".fifthtry.site")
}

async fn write_archive_content(
    ds: &fastn_ds::DocumentStore,
    output_path: &fastn_ds::Path,
    buffer: &[u8],
    package_name: &str,
    check: bool,
) -> Result<(), UpdateError> {
    if check {
        return Err(UpdateError::Check(CheckError::WriteDuringCheck {
            package: package_name.to_string(),
            file: output_path.to_string(),
        }));
    }

    Ok(ds
        .write_content(output_path, buffer)
        .await
        .context(WriteArchiveContentSnafu {
            package: package_name,
        })?)
}

#[tracing::instrument(skip_all)]
pub async fn update(ds: &fastn_ds::DocumentStore, check: bool) -> fastn_core::Result<()> {
    let packages_root = ds.root().join(".packages");
    let current_package = utils::read_current_package(ds).await?;

    if current_package.dependencies.is_empty() {
        println!("No dependencies to update.");

        // Creating Empty config file for packages with no dependencies
        fastn_core::ConfigTemp::write(ds, current_package.name.clone(), Default::default()).await?;

        return Ok(());
    }

    let spinner_style =
        indicatif::ProgressStyle::with_template("{prefix:.bold.dim} [{pos}/{len}] {spinner} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");

    let pb = indicatif::ProgressBar::new(current_package.dependencies.len() as u64);
    pb.set_style(spinner_style);
    pb.set_prefix("Updating dependencies");

    let updated_packages =
        match update_dependencies(ds, packages_root, &current_package, &pb, check).await {
            Ok(n) => n,
            Err(UpdateError::Check(e)) => {
                eprintln!("{}", e);
                std::process::exit(7);
            }
            Err(e) => {
                return Err(fastn_core::Error::UpdateError {
                    message: e.to_string(),
                });
            }
        };

    pb.finish_and_clear();

    match updated_packages {
        0 => println!("No packages updated."),
        1 => println!("Updated package dependency."),
        n => println!("Updated {} dependencies.", n),
    }

    Ok(())
}
