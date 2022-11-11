#[derive(Debug, Clone)]
pub struct App {
    pub name: Option<String>,
    pub package: fpm::Dependency,
    pub mount_point: String,
    pub end_point: Option<String>,
    pub config: std::collections::HashMap<String, String>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct AppTemp {
    pub name: Option<String>,
    pub package: String,
    #[serde(rename = "mount-point")]
    pub mount_point: String,
    #[serde(rename = "end-point")]
    pub end_point: Option<String>,
    pub config: Vec<String>,
}

impl AppTemp {
    fn parse_config(config: &[String]) -> fpm::Result<std::collections::HashMap<String, String>> {
        let mut hm = std::collections::HashMap::new();
        for key_value in config.iter() {
            // <key>=<value>
            let (key, value): (&str, &str) = match key_value.trim().split_once('=') {
                Some(x) => x,
                None => {
                    return Err(fpm::Error::PackageError {
                        message: format!(
                            "package-config-error, wrong header in an fpm app, format is <key>=<value>, config: {}",
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
                    None => return Err(fpm::Error::PackageError {
                        message: format!(
                            "package-config-error, wrong $ENV in an fpm app, format is <key>=$ENV.env_var_name, key: {}, value: {}",
                            key, value
                        ),
                    }),
                };

                let value =
                    std::env::var(env_var_name).map_err(|err| fpm::Error::PackageError {
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

    pub fn into_app(self) -> fpm::Result<App> {
        let package = fpm::Dependency {
            package: fpm::Package::new(self.package.trim().trim_matches('/')),
            version: None,
            notes: None,
            alias: None,
            implements: Vec::new(),
            endpoint: None,
            mountpoint: None,
        };

        Ok(App {
            name: self.name,
            package,
            mount_point: self.mount_point,
            end_point: self.end_point,
            config: Self::parse_config(&self.config)?,
        })
    }
}
