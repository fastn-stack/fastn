extern crate self as fastn_update;

mod utils;

pub async fn resolve_dependencies_(
    ds: &fastn_ds::DocumentStore,
    packages_root: &fastn_ds::Path,
    current_package: &fastn_core::Package,
) -> fastn_core::Result<()> {
    let mut stack = vec![current_package.clone()];
    let mut resolved = std::collections::HashSet::new();
    resolved.insert(current_package.name.to_string());
    while let Some(package) = stack.pop() {
        for dependency in package.dependencies {
            if resolved.contains(&dependency.package.name) {
                continue;
            }
            let dependency_path = &packages_root.join(&dependency.package.name);
            let manifest = get_manifest(ds, dependency_path).await?;
            download_zip(&ds, packages_root, &manifest).await?;
            let dep_package = {
                let mut dep_package = dependency.package.clone();
                dep_package
                    .resolve(&dep_package.fastn_path.clone().unwrap(), ds)
                    .await?;
                dep_package
            };
            resolved.insert(dependency.package.name.to_string());
            stack.push(dep_package);
        }
    }
    Ok(())
}

async fn download_zip(
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
                let path_without_prefix = path
                    .to_str()
                    .unwrap()
                    .split_once(std::path::MAIN_SEPARATOR)
                    .unwrap()
                    .1;
                let output_path = &dependency_path.join(path_without_prefix);
                ds.write_content(&output_path, buffer).await?;
            }
        }
    }

    Ok(())
}

/// Download manifest of the package `<package-name>/.fastn/manifest.json`
/// Resolve to `fastn_core::Manifest` struct
async fn get_manifest(
    ds: &fastn_ds::DocumentStore,
    dependency_path: &fastn_ds::Path,
) -> fastn_core::Result<fastn_core::Manifest> {
    let dot_fastn_folder = dependency_path.join(".fastn");
    let manifest_path = dot_fastn_folder.join("manifest.json");
    let manifest = serde_json::de::from_slice(&ds.read_content(&manifest_path).await?)?;

    Ok(manifest)
}

pub async fn process(config: &fastn_core::Config) -> fastn_core::Result<()> {
    resolve_dependencies_(&config.ds, &config.packages_root, &config.package).await?;

    Ok(())
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

    process(config).await?;

    if c.package.dependencies.len() == 1 {
        println!("Updated the package dependency.");
    } else {
        println!("Updated {} dependencies.", c.package.dependencies.len())
    }

    Ok(())
}
