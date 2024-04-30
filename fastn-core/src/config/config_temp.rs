#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not write config: {0}")]
    FailedToWrite(#[from] fastn_ds::WriteError),

    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("config.json not found. Help: Try running `fastn update`. Source: {0}")]
    NotFound(#[source] fastn_ds::ReadError),

    #[error("Failed to read config.json: {0}")]
    FailedToRead(#[from] fastn_ds::ReadError),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ConfigTemp {
    #[serde(rename = "package")]
    pub package_name: String,
    pub all_packages: std::collections::BTreeMap<String, fastn_core::Manifest>,
}

impl ConfigTemp {
    pub fn new(
        package_name: String,
        all_packages: std::collections::BTreeMap<String, fastn_core::Manifest>,
    ) -> Self {
        ConfigTemp {
            package_name,
            all_packages,
        }
    }

    pub async fn write(
        ds: &fastn_ds::DocumentStore,
        package_name: String,
        all_packages: std::collections::BTreeMap<String, fastn_core::Manifest>,
    ) -> Result<(), Error> {
        let dot_fastn = ds.root().join(".fastn");
        let config_json_path = dot_fastn.join("config.json");
        let config_temp = ConfigTemp::new(package_name, all_packages);

        ds.write_content(
            &config_json_path,
            &serde_json::ser::to_vec_pretty(&config_temp)?,
        )
        .await?;

        Ok(())
    }

    pub async fn read(ds: &fastn_ds::DocumentStore) -> Result<ConfigTemp, Error> {
        let dot_fastn = ds.root().join(".fastn");
        let config_json_path = dot_fastn.join("config.json");
        let bytes = match ds.read_content(&config_json_path).await {
            Ok(v) => v,
            Err(e) => {
                if let fastn_ds::ReadError::NotFound = e {
                    return Err(Error::NotFound(e));
                }

                return Err(Error::FailedToRead(e));
            }
        };
        let config_temp: ConfigTemp = serde_json::de::from_slice(&bytes)?;

        Ok(config_temp)
    }

    pub async fn get_all_packages(
        &self,
        ds: &fastn_ds::DocumentStore,
        package: &mut fastn_core::Package,
        package_root: &fastn_ds::Path,
    ) -> fastn_core::Result<scc::HashMap<String, fastn_core::Package>> {
        let all_packages = scc::HashMap::new();

        for (package_name, manifest) in &self.all_packages {
            let mut current_package = manifest
                .to_package(package_root, package_name, ds, package)
                .await?;
            ConfigTemp::check_dependencies_provided(package, &mut current_package)?;
            fastn_ds::insert_or_update(&all_packages, package_name.clone(), current_package);
        }

        Ok(all_packages)
    }

    fn get_package_name_for_module(
        package: &fastn_core::Package,
        module_name: &str,
    ) -> fastn_core::Result<String> {
        if module_name.starts_with(format!("{}/", &package.name).as_str())
            || module_name.eq(&package.name)
        {
            Ok(package.name.clone())
        } else if let Some(package_dependency) = package.dependencies.iter().find(|v| {
            module_name.starts_with(format!("{}/", &v.package.name).as_str())
                || module_name.eq(&v.package.name)
        }) {
            Ok(package_dependency.package.name.clone())
        } else {
            fastn_core::usage_error(format!("Can't find package for module {}", module_name))
        }
    }

    pub(crate) fn check_dependencies_provided(
        package: &fastn_core::Package,
        current_package: &mut fastn_core::Package,
    ) -> fastn_core::Result<()> {
        let mut auto_imports = vec![];
        for dependency in current_package.dependencies.iter_mut() {
            if let Some(ref required_as) = dependency.required_as {
                if let Some(provided_via) = package.dependencies.iter().find_map(|v| {
                    if v.package.name.eq(&dependency.package.name) {
                        v.provided_via.clone()
                    } else {
                        None
                    }
                }) {
                    let package_name =
                        ConfigTemp::get_package_name_for_module(package, provided_via.as_str())?;
                    dependency.package.name = package_name;
                    auto_imports.push(fastn_core::AutoImport {
                        path: provided_via.to_string(),
                        alias: Some(required_as.clone()),
                        exposing: vec![],
                    });
                } else {
                    /*auto_imports.push(fastn_core::AutoImport {
                        path: dependency.package.name.to_string(),
                        alias: Some(required_as.clone()),
                        exposing: vec![],
                    });*/
                    dbg!("Dependency needs to be provided.", &dependency.package.name);
                    return fastn_core::usage_error(format!(
                        "Dependency {} needs to be provided.",
                        dependency.package.name
                    ));
                }
            }
        }
        current_package.auto_import.extend(auto_imports);
        if let Some(ref package_alias) = current_package.system {
            if current_package.system_is_confidential.unwrap_or(true) {
                return fastn_core::usage_error(format!("system-is-confidential is needed for system package {} and currently only false is supported.", current_package.name));
            }
            if let Some(provided_via) = package.dependencies.iter().find_map(|v| {
                if v.package.name.eq(&current_package.name) {
                    v.provided_via.clone()
                } else {
                    None
                }
            }) {
                let package_name =
                    ConfigTemp::get_package_name_for_module(package, provided_via.as_str())?;
                current_package.dependencies.push(fastn_core::Dependency {
                    package: fastn_core::Package::new(package_name.as_str()),
                    version: None,
                    notes: None,
                    alias: None,
                    implements: vec![],
                    provided_via: None,
                    required_as: None,
                });
                current_package.auto_import.push(fastn_core::AutoImport {
                    path: provided_via.to_string(),
                    alias: Some(package_alias.clone()),
                    exposing: vec![],
                });
            } else {
                current_package.auto_import.push(fastn_core::AutoImport {
                    path: current_package.name.to_string(),
                    alias: Some(package_alias.clone()),
                    exposing: vec![],
                });
            }
        }
        Ok(())
    }
}
