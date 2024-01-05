extern crate self as fastn_update;

mod types;
mod utils;

#[async_recursion::async_recursion(?Send)]
pub async fn resolve_dependencies(
    ds: &fastn_ds::DocumentStore,
    packages_root: &fastn_ds::Path,
    dependencies: &Vec<fastn_core::package::dependency::Dependency>,
    packages: &mut Vec<fastn_update::types::Package>,
    visited: &mut std::collections::HashSet<String>,
    processing_set: &mut std::collections::HashSet<String>,
) -> fastn_core::Result<()> {
    use fastn_core::package::PackageTempIntoPackage;
    use std::io::Read;

    for dependency in dependencies {
        let package_name = &dependency.package.name;

        println!("Currently resolving {}", &package_name);

        // Check for circular dependency
        if processing_set.contains(package_name) || package_name.eq(fastn_core::FASTN_UI_INTERFACE)
        {
            continue;
        }

        processing_set.insert(package_name.clone());

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

                // read FASTN.ftd
                let fastn_path = dependency_path.join("FASTN.ftd");
                let ftd_document = {
                    let doc = ds.read_to_string(&fastn_path).await?;
                    let lib = fastn_core::FastnLibrary::default();
                    match fastn_core::doc::parse_ftd("fastn", doc.as_str(), &lib) {
                        Ok(v) => v,
                        Err(e) => {
                            return Err(fastn_core::Error::PackageError {
                                message: format!("failed to parse FASTN.ftd 2: {:?}", &e),
                            });
                        }
                    }
                };
                let mut package = {
                    let temp_package: fastn_package::old_fastn::PackageTemp =
                        ftd_document.get("fastn#package")?;
                    temp_package.into_package()
                };

                package.translation_status_summary =
                    ftd_document.get("fastn#translation-status-summary")?;

                package.fastn_path = Some(fastn_path.clone());
                package.dependencies = {
                    let temp_deps: Vec<fastn_core::package::dependency::DependencyTemp> =
                        ftd_document.get("fastn#dependency")?;
                    temp_deps
                        .into_iter()
                        .map(|v| v.into_dependency())
                        .collect::<Vec<fastn_core::Result<fastn_core::package::dependency::Dependency>>>()
                        .into_iter()
                        .collect::<fastn_core::Result<Vec<fastn_core::package::dependency::Dependency>>>()?
                };

                let auto_imports: Vec<fastn_core::package::dependency::AutoImportTemp> =
                    ftd_document.get("fastn#auto-import")?;
                let auto_import = auto_imports
                    .into_iter()
                    .map(|f| f.into_auto_import())
                    .collect();
                package.auto_import = auto_import;
                package.fonts = ftd_document.get("fastn#font")?;
                package.sitemap_temp = ftd_document.get("fastn#sitemap")?;

                println!("It came here");

                if !visited.contains(package_name) {
                    processing_set.insert(package_name.clone());

                    println!(
                        "Currently processing dependencies of package: {}",
                        &package_name
                    );

                    resolve_dependencies(
                        ds,
                        &dependency_path.join(".packages"),
                        &package.dependencies,
                        &mut Vec::<fastn_update::types::Package>::new(),
                        visited,
                        processing_set,
                    )
                    .await?;

                    processing_set.remove(package_name);
                    visited.insert(package_name.clone());

                    processing_set.remove(package_name);

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
    let mut processing_set = std::collections::HashSet::new();
    let mut visited = std::collections::HashSet::new();

    // 1st PASS: download the direct dependencies
    resolve_dependencies(
        &config.ds,
        &config.packages_root,
        &config.package.dependencies,
        &mut packages,
        &mut visited,
        &mut processing_set,
    )
    .await?;

    let manifest = fastn_update::types::Manifest::new(packages);
    let dot_fastn_dir = config.ds.root().join(".fastn");

    config
        .ds
        .write_content(
            &dot_fastn_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest)?,
        )
        .await?;

    println!("Wrote manifest.json");

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
