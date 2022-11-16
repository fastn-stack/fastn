#[derive(Debug, Clone)]
pub struct App {
    pub name: String,
    // TODO: Dependency or package??
    pub package: fpm::Package,
    pub mount_point: String,
    pub end_point: Option<String>,
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
    pub config: Vec<String>,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
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

    pub async fn into_app(self, config: &fpm::Config) -> fpm::Result<App> {
        let package = config
            .resolve_package(&fpm::Package::new(self.package.trim().trim_matches('/')))
            .await?;

        Ok(App {
            name: self.name,
            package,
            mount_point: self.mount_point,
            end_point: self.end_point,
            config: Self::parse_config(&self.config)?,
            readers: self.readers,
            writers: self.writers,
        })
    }
}

pub fn processor<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    use itertools::Itertools;
    #[derive(Debug, serde::Serialize)]
    struct UiApp {
        name: String,
        package: String,
        #[serde(rename = "url")]
        url: String,
        icon: Option<String>,
    }

    let apps = config
        .package
        .apps
        .iter()
        .map(|a| UiApp {
            name: a.name.clone(),
            package: a.package.name.clone(),
            url: a.mount_point.to_string(),
            icon: a.package.icon.clone(),
        })
        .collect_vec();

    let indexy_apps = fpm::ds::LengthList::from_owned(apps);
    doc.from_json(&indexy_apps, section)
}

// Takes the path /-/<package-name>/<remaining>/ or /mount-point/<remaining>/
pub async fn can_read(config: &fpm::Config, path: &str) -> fpm::Result<bool> {
    use itertools::Itertools;
    // first get the app
    let readers_groups = if let Some((_, _, _, Some(app))) =
        config.get_mountpoint_sanitized_path(&config.package, path)
    {
        app.readers.clone()
    } else {
        vec![]
    };

    dbg!("app readers");
    dbg!(&readers_groups);
    if readers_groups.is_empty() {
        return Ok(true);
    }

    let user_groups = config
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
        app_identities.extend(ug.get_identities(config)?)
    }

    let auth_identities = fpm::auth::get_auth_identities(
        config.request.as_ref().unwrap().cookies(),
        app_identities.as_slice(),
    )
    .await?;

    return fpm::user_group::belongs_to(
        config,
        user_groups.as_slice(),
        auth_identities.iter().collect_vec().as_slice(),
    );

    // get the app readers
    // get the groups
    // the the access identities
    // check belongs _to
    //return Ok(true);
}
