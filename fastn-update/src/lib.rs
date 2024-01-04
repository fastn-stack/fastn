use std::io::Read;

extern crate self as fastn_update;

mod types;
mod utils;

pub async fn resolve_dependencies(
    config: &fastn_core::Config,
    packages_root: &camino::Utf8PathBuf,
    dependencies: &Vec<fastn_core::package::dependency::Dependency>,
    packages: &mut Vec<fastn_update::types::Package>,
) -> fastn_core::Result<()> {
    for dependency in dependencies {
        match fastn_update::utils::get_package_source_url(&dependency.package) {
            Some(source) => {
                let (mut archive, checksum) =
                    fastn_update::utils::download_archive(source.clone()).await?;
                let dependency_path = &packages_root.join(&dependency.package.name);

                for i in 0..archive.len() {
                    let mut entry = archive.by_index(i)?;

                    if entry.is_file() {
                        let mut buffer = Vec::new();
                        entry.read_to_end(&mut buffer)?;
                        if let Some(path) = entry.enclosed_name() {
                            if let Some(name) = path.to_str() {
                                let path = &dependency_path.join(name);
                                config.ds.write_content(&path, buffer).await?;
                            }
                        }
                    }
                }

                let package = fastn_update::types::Package::new(
                    dependency.package.name.clone(),
                    None, // todo: fix this when versioning is available
                    source,
                    checksum,
                    dependency
                        .package
                        .dependencies
                        .iter()
                        .map(|d| d.package.name.clone())
                        .collect(),
                );

                packages.push(package);
            }
            None => {
                return Err(fastn_core::Error::PackageError {
                    message: format!(
                        "Could not download package: {}, no source found.",
                        dependency.package.name
                    ),
                })
            }
        }
    }

    Ok(())
}

pub async fn process(config: &fastn_core::Config) -> fastn_core::Result<()> {
    let mut packages: Vec<fastn_update::types::Package> = vec![];

    // 1st PASS: download the direct dependencies
    resolve_dependencies(
        config,
        &config.packages_root,
        &config.package.dependencies,
        &mut packages,
    )
    .await?;

    // 2nd PASS: download all other dependencies
    //

    let manifest = fastn_update::types::Manifest::new(packages);
    let dot_fastn_dir = config.ds.root().join(".fastn");

    config
        .ds
        .write_content(
            dot_fastn_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest)?,
        )
        .await?;

    println!("Wrote manifest.json");

    Ok(())
}

pub async fn update(config: &fastn_core::Config) -> fastn_core::Result<()> {
    if let Err(e) = std::fs::remove_dir_all(config.ds.root().join(".packages")) {
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
