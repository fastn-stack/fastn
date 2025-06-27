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

#[macro_export]
macro_rules! mprint {
    ($($arg:tt)*) => {
        if !fastn_core::utils::is_test() {
            print!($($arg)*);
        }
    }
}

// macro called mred that prints in red a message, using ansi colors for red,
macro_rules! mdone {
    ($red: expr, $($arg:tt)*) => {
        use colored::Colorize;

        if !fastn_core::utils::is_test() {
            let msg = format!($($arg)*);
            if $red {
                println!("{}", msg.red());
            } else {
                println!("{}", msg.green());
            }
        }
    }
}

async fn update_dependencies(
    ds: &fastn_ds::DocumentStore,
    packages_root: fastn_ds::Path,
    current_package: &fastn_core::Package,
    check: bool,
) -> Result<(usize, usize), UpdateError> {
    use colored::Colorize;
    mprint!("Checking dependencies for {}.\n", current_package.name);

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
            mprint!("Checking {}: ", dependency.package.name.blue());
            let dep_package = &dependency.package;
            let package_name = dep_package.name.clone();
            let dependency_path = packages_root.join(&package_name);
            let updated = if ds.exists(&dependency_path.join(".is-local"), &None).await {
                mdone!(true, "Local package");
                all_packages.push((
                    package_name.to_string(),
                    update_local_package_manifest(&dependency_path).await?,
                ));
                false
            } else if is_fifthtry_site_package(package_name.as_str()) {
                update_fifthtry_dependency(
                    &dependency,
                    ds,
                    packages_root.clone(),
                    &mut all_packages,
                    check,
                )
                .await?
            } else {
                update_github_dependency(
                    &dependency,
                    ds,
                    packages_root.clone(),
                    &mut all_packages,
                    check,
                )
                .await?
            };

            if updated {
                updated_packages += 1;
            }

            // TODO: why are we not updating FASTN_UI_INTERFACE package?
            if package_name.eq(&fastn_core::FASTN_UI_INTERFACE) {
                resolved.insert(package_name.to_string());
                continue;
            }

            let dep_package =
                utils::resolve_dependency_package(ds, &dependency, &dependency_path).await?;
            resolved.insert(package_name.to_string());
            stack.push(dep_package);
        }
    }

    let total_packages = all_packages.len();
    fastn_core::ConfigTemp::write(
        ds,
        current_package.name.clone(),
        all_packages.into_iter().collect(),
    )
    .await?;

    Ok((updated_packages, total_packages))
}

async fn update_github_dependency(
    dependency: &fastn_core::package::dependency::Dependency,
    ds: &fastn_ds::DocumentStore,
    packages_root: fastn_ds::Path,
    all_packages: &mut Vec<(String, fastn_core::Manifest)>,
    check: bool,
) -> Result<bool, fastn_update::UpdateError> {
    let dep_package = &dependency.package;
    let package_name = dep_package.name.clone();
    let dependency_path = &packages_root.join(&package_name);

    if is_fifthtry_site_package(package_name.as_str()) {
        return Err(fastn_update::UpdateError::InvalidPackage(format!(
            "{package_name} is a fifthtry site."
        )));
    }

    let (manifest, updated) = download_unpack_zip_and_get_manifest(
        dependency_path,
        fastn_core::manifest::utils::get_zipball_url(package_name.as_str())
            .unwrap()
            .as_str(),
        ds,
        package_name.as_str(),
        true,
        check,
    )
    .await?;

    all_packages.push((package_name.to_string(), manifest));
    Ok(updated)
}

async fn update_fifthtry_dependency(
    dependency: &fastn_core::package::dependency::Dependency,
    ds: &fastn_ds::DocumentStore,
    packages_root: fastn_ds::Path,
    all_packages: &mut Vec<(String, fastn_core::Manifest)>,
    check: bool,
) -> Result<bool, fastn_update::UpdateError> {
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

    let (manifest, updated) = download_unpack_zip_and_get_manifest(
        dependency_path,
        site_zip_url.as_str(),
        ds,
        package_name.as_str(),
        false,
        check,
    )
    .await?;

    all_packages.push((package_name.to_string(), manifest));

    // todo: return true only if package is updated
    Ok(updated)
}

async fn update_local_package_manifest(
    _path: &fastn_ds::Path,
) -> Result<fastn_core::Manifest, fastn_update::UpdateError> {
    Ok(fastn_core::Manifest {
        files: Default::default(),
        zip_url: "".to_string(),
        checksum: "".to_string(),
    })
}

async fn download_unpack_zip_and_get_manifest(
    dependency_path: &fastn_ds::Path,
    zip_url: &str,
    ds: &fastn_ds::DocumentStore,
    package_name: &str,
    is_github_package: bool,
    check: bool,
) -> Result<(fastn_core::Manifest, bool), fastn_update::UpdateError> {
    let etag_file = dependency_path.join(".etag");

    let start = std::time::Instant::now();
    let resp = utils::download_archive(ds, zip_url, &etag_file)
        .await
        .context(DownloadArchiveSnafu {
            package: package_name,
        })?;

    let elapsed = start.elapsed();
    let elapsed_secs = elapsed.as_secs();
    let elapsed_millis = elapsed.subsec_millis();

    let red = elapsed_secs > 0 || elapsed_millis > 200;

    let (etag, mut archive) = match resp {
        Some(x) => {
            mdone!(red, "downloaded in {}.{:03}s", elapsed_secs, elapsed_millis);
            x
        }
        None => {
            mdone!(red, "checked in {}.{:03}s", elapsed_secs, elapsed_millis);
            return Ok((update_local_package_manifest(dependency_path).await?, false));
        }
    };

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

            // For package like `fifthtry.github.io/package-doc`, github zip is a folder called
            // `package-doc-<commit-id>` which contains all files, so path for `FASTN.ftd` becomes
            // `package-doc-<commit-id>/FASTN.ftd` while fifthtry package zip doesn't have any
            // such folder, so path becomes `FASTN.ftd`
            let path_without_prefix = if is_github_package {
                match path_normalized.split_once('/') {
                    Some((_, path)) => path,
                    None => &path_normalized,
                }
            } else {
                // For fifthtry packages
                &path_normalized
            };

            let output_path = &dependency_path.join(path_without_prefix);
            write_archive_content(ds, output_path, &buffer, package_name, check).await?;
        }
    }

    ds.write_content(&etag_file, etag.as_bytes())
        .await
        .inspect_err(|e| eprintln!("failed to write etag file for {package_name}: {e}"))
        .unwrap_or(());

    Ok((update_local_package_manifest(dependency_path).await?, true))
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
        println!("No dependencies in {}.", current_package.name);

        // Creating Empty config file for packages with no dependencies
        fastn_core::ConfigTemp::write(ds, current_package.name.clone(), Default::default()).await?;

        return Ok(());
    }

    let (updated_packages, total_packages) =
        match update_dependencies(ds, packages_root, &current_package, check).await {
            Ok(n) => n,
            Err(UpdateError::Check(e)) => {
                eprintln!("{e}");
                std::process::exit(7);
            }
            Err(e) => {
                return Err(fastn_core::Error::UpdateError {
                    message: e.to_string(),
                });
            }
        };

    match updated_packages {
        _ if fastn_core::utils::is_test() => println!("Updated N dependencies."),
        0 => println!("All the {total_packages} packages are up to date."),
        1 => println!("Updated 1/{total_packages} dependency."),
        _ => println!("Updated {updated_packages}/{total_packages} dependencies."),
    }

    Ok(())
}
