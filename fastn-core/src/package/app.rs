#[derive(Debug, Clone)]
pub struct App {
    pub name: String,
    // TODO: Dependency or package??
    pub package: fastn_core::Package,
    pub mount_point: String,
    pub end_point: Option<String>,
    pub user_id: Option<String>,
    pub config: std::collections::HashMap<String, String>,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct AppTemp {
    pub name: String,
    pub package: String,
    #[serde(rename = "mount-point")]
    pub mount_point: String,
    #[serde(rename = "end-point")]
    pub end_point: Option<String>,
    #[serde(rename = "user-id")]
    pub user_id: Option<String>,
    pub config: Vec<String>,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
}

impl AppTemp {
    async fn parse_config(
        config: &[String],
    ) -> fastn_core::Result<std::collections::HashMap<String, String>> {
        let mut hm = std::collections::HashMap::new();
        for key_value in config.iter() {
            // <key>=<value>
            let (key, value): (&str, &str) = match key_value.trim().split_once('=') {
                Some(x) => x,
                None => {
                    return Err(fastn_core::Error::PackageError {
                        message: format!(
                            "package-config-error, wrong header in an fastn app, format is <key>=<value>, config: {}",
                            key_value
                        ),
                    });
                }
            };
            // if value = $ENV.env_var_name
            // so read env_var_name from std::env
            let value = value.trim();
            if value.starts_with("$ENV") {
                let (_, env_var_name) = match value.trim().split_once('.') {
                    Some(x) => x,
                    None => return Err(fastn_core::Error::PackageError {
                        message: format!(
                            "package-config-error, wrong $ENV in an fastn app, format is <key>=$ENV.env_var_name, key: {}, value: {}",
                            key, value
                        ),
                    }),
                };

                let value =
                    std::env::var(env_var_name).map_err(|err| fastn_core::Error::PackageError {
                        message: format!(
                            "package-config-error,$ENV {} variable is not set for {}, err: {}",
                            env_var_name, value, err
                        ),
                    })?;
                hm.insert(key.to_string(), value.to_string());
            } else {
                hm.insert(key.to_string(), value.to_string());
            }
        }
        Ok(hm)
    }

    pub async fn into_app(self, config: &fastn_core::Config) -> fastn_core::Result<App> {
        let package = config
            .resolve_package(&fastn_core::Package::new(
                self.package.trim().trim_matches('/'),
            ))
            .await?;

        Ok(App {
            name: self.name,
            package,
            mount_point: self.mount_point,
            end_point: self.end_point,
            user_id: self.user_id,
            config: Self::parse_config(&self.config).await?,
            readers: self.readers,
            writers: self.writers,
        })
    }
}
