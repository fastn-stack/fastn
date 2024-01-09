extern crate self as fastn_update;

mod utils;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("Failed to resolve manifest.json for package '{package}'")]
    ManifestResolution {
        package: String,
        #[source]
        error: fastn_core::Error,
    },
    #[error("Failed to download and unpack archive for package '{package}'")]
    ArchiveDownload {
        package: String,
        #[source]
        error: fastn_core::Error,
    },
    #[error("Failed to resolve dependency '{package}'")]
    DependencyResolution {
        package: String,
        #[source]
        error: fastn_core::Error,
    },
}

type Result<T> = std::result::Result<T, Error>;

async fn resolve_dependencies(
    ds: &fastn_ds::DocumentStore,
    packages_root: &fastn_ds::Path,
    current_package: &fastn_core::Package,
    pb: &indicatif::ProgressBar,
) -> fastn_update::Result<()> {
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

            let manifest = match get_manifest(package_name.clone()).await {
                Ok(manifest) => manifest,
                Err(e) => {
                    return Err(Error::ManifestResolution {
                        package: package_name.clone(),
                        error: e,
                    });
                }
            };

            pb.set_message(format!("Downloading {} archive", &package_name));

            if let Err(e) =
                download_and_unpack_zip(ds, &packages_root.join(&package_name), &manifest).await
            {
                return Err(Error::ArchiveDownload {
                    package: package_name.clone(),
                    error: e,
                });
            }

            let dep_package = {
                let mut dep_package = dependency.package.clone();
                let fastn_path = dependency_path.join("FASTN.ftd");
                if let Err(e) = dep_package.resolve(&fastn_path, ds).await {
                    return Err(Error::DependencyResolution {
                        package: package_name.clone(),
                        error: e,
                    });
                };
                dep_package
            };
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
) -> fastn_core::Result<()> {
    let mut archive = fastn_update::utils::download_archive(manifest.zip_url.clone()).await?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;

        if entry.is_file() {
            let mut buffer = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut buffer)?;
            if let Some(path) = entry.enclosed_name() {
                let path = path.to_str().unwrap();
                let path_without_prefix = match path.split_once(std::path::MAIN_SEPARATOR) {
                    Some((_, path)) => path,
                    None => path,
                };
                let output_path = &dependency_path.join(path_without_prefix);
                ds.write_content(output_path, buffer).await?;
            }
        }
    }

    Ok(())
}

/// Download manifest of the package `<package-name>/manifest.json`
/// Resolve to `fastn_core::Manifest` struct
async fn get_manifest(package: String) -> fastn_core::Result<fastn_core::Manifest> {
    let manifest_bytes = fastn_core::http::http_get(&format!(
        "https://{}/{}",
        package,
        fastn_core::manifest::MANIFEST_JSON
    ))
    .await?;
    let manifest = serde_json::de::from_slice(&manifest_bytes)?;

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
        resolve_dependencies(&config.ds, &config.packages_root, current_package, &pb).await
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
