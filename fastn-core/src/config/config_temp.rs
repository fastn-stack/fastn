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
            serde_json::ser::to_vec_pretty(&config_temp)?,
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
        package_root: &fastn_ds::Path,
    ) -> fastn_core::Result<std::collections::BTreeMap<String, fastn_core::Package>> {
        let mut all_packages = std::collections::BTreeMap::new();

        for (package_name, manifest) in &self.all_packages {
            let package = manifest
                .to_package(package_root, package_name, ds)
                .await?;
            all_packages.insert(package_name.clone(), package);
        }

        Ok(all_packages)
    }
}
