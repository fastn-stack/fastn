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
    fn parse_config(
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
            config: Self::parse_config(&self.config)?,
            readers: self.readers,
            writers: self.writers,
        })
    }
}

// Takes the path /-/<package-name>/<remaining>/ or /mount-point/<remaining>/
pub async fn can_read(config: &fastn_core::RequestConfig, path: &str) -> fastn_core::Result<bool> {
    use itertools::Itertools;
    // first get the app
    let readers_groups = if let Some((_, _, _, Some(app))) = config
        .config
        .get_mountpoint_sanitized_path(&config.config.package, path)
    {
        app.readers.clone()
    } else {
        vec![]
    };

    if readers_groups.is_empty() {
        return Ok(true);
    }

    let user_groups = config
        .config
        .package
        .groups
        .iter()
        .filter_map(|(id, g)| {
            if readers_groups.contains(id) {
                Some(g)
            } else {
                None
            }
        })
        .collect_vec();

    let mut app_identities = vec![];
    for ug in user_groups.iter() {
        app_identities.extend(ug.get_identities(&config.config).await?)
    }

    let auth_identities = {
        #[cfg(feature = "auth")]
        let i = fastn_core::auth::get_auth_identities(
            config.request.cookies(),
            app_identities.as_slice(),
        )
        .await?;

        #[cfg(not(feature = "auth"))]
        let i = vec![];

        i
    };

    return fastn_core::user_group::belongs_to(
        &config.config,
        user_groups.as_slice(),
        auth_identities.iter().collect_vec().as_slice(),
    )
    .await;

    // get the app readers
    // get the groups
    // the the access identities
    // check belongs _to
    //return Ok(true);
}
