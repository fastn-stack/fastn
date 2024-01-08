extern crate self as fastn_update;

mod utils;

async fn resolve_dependencies(
    ds: &fastn_ds::DocumentStore,
    packages_root: &fastn_ds::Path,
    current_package: &fastn_core::Package,
) -> fastn_core::Result<()> {
    let mut stack = vec![current_package.clone()];
    let mut resolved = std::collections::HashSet::new();
    resolved.insert(current_package.name.to_string());

    let spinner_style =
        indicatif::ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    let pb = indicatif::ProgressBar::new(current_package.dependencies.len() as u64);
    pb.set_style(spinner_style);
    while let Some(package) = stack.pop() {
        pb.set_prefix(format!("Resolving package {}", &package.name));
        for dependency in package.dependencies {
            if resolved.contains(&dependency.package.name)
                || dependency.package.name.eq(&fastn_core::FASTN_UI_INTERFACE)
            {
                continue;
            }
            let dependency_path = &packages_root.join(&dependency.package.name);
            pb.set_message("Downloading manifest.json");
            let manifest = match get_manifest(dependency.package.name.clone()).await {
                Ok(manifest) => manifest,
                Err(e) => {
                    pb.set_message(format!(
                        "Failed to resolve manifest.json for {}",
                        &dependency.package.name
                    ));
                    pb.finish();

                    return Err(e);
                }
            };
            pb.set_message("Downloading package archive");
            download_and_unpack_zip(ds, &packages_root.join(&dependency.package.name), &manifest)
                .await?;
            pb.set_message("Downloaded and unpacked zip archive");
            let dep_package = {
                let mut dep_package = dependency.package.clone();
                let fastn_path = dependency_path.join("FASTN.ftd");
                dep_package.resolve(&fastn_path, ds).await?;
                dep_package
            };
            resolved.insert(dependency.package.name.to_string());
            stack.push(dep_package);
            pb.inc(1);
            pb.set_message("Resolved");
        }
    }
    pb.finish();
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

/// Download manifest of the package `<package-name>/.fastn/manifest.json`
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

async fn process(config: &fastn_core::Config) -> fastn_core::Result<()> {
    resolve_dependencies(&config.ds, &config.packages_root, &config.package).await
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

    if process(config).await.is_err() {
        eprintln!("Update failed.");

        return Ok(());
    }

    if c.package.dependencies.len() == 1 {
        println!("Updated the package dependency.");
    } else {
        println!("Updated {} dependencies.", c.package.dependencies.len())
    }

    Ok(())
}
