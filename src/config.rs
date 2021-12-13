#[derive(Debug)]
pub struct Config {
    pub package: fpm::Package,
    pub root: camino::Utf8PathBuf,
    pub original_directory: camino::Utf8PathBuf,
    pub fonts: Vec<fpm::Font>,
    pub dependencies: Vec<fpm::Dependency>,
    pub ignored: ignore::overrides::Override,
}

impl Config {
    pub fn build_dir(&self) -> camino::Utf8PathBuf {
        self.root.join(".build")
    }

    pub fn history_dir(&self) -> camino::Utf8PathBuf {
        self.root.join(".history")
    }

    pub fn track_dir(&self) -> camino::Utf8PathBuf {
        self.root.join(".tracks")
    }

    pub fn latest_ftd(&self) -> camino::Utf8PathBuf {
        self.root.join(".history/.latest.ftd")
    }

    pub fn get_font_style(&self) -> String {
        let generated_style = self
            .fonts
            .iter()
            .fold("".to_string(), |c, f| format!("{}\n{}", c, f.to_html()));
        return match generated_style.is_empty() {
            false => format!("<style>{}</style>", generated_style),
            _ => format!(""),
        };
    }

    pub async fn read() -> fpm::Result<Config> {
        let original_directory: camino::Utf8PathBuf =
            std::env::current_dir()?.canonicalize()?.try_into()?;
        let root = match find_package_root(&original_directory) {
            Some(b) => b,
            None => {
                return Err(fpm::Error::ConfigurationError {
                    message: "FPM.ftd not found in any parent directory".to_string(),
                });
            }
        };

        let b = {
            let doc = tokio::fs::read_to_string(root.join("FPM.ftd")).await?;
            let lib = fpm::Library::default();
            match ftd::p2::Document::from("FPM", doc.as_str(), &lib) {
                Ok(v) => v,
                Err(e) => {
                    return Err(fpm::Error::ConfigurationError {
                        message: format!("failed to parse FPM.ftd: {:?}", &e),
                    });
                }
            }
        };
        let package: fpm::Package = b.get("fpm#package")?;
        let deps: Vec<fpm::Dependency> = b.get("fpm#dependency")?;
        let fonts: Vec<fpm::Font> = b.get("fpm#font")?;

        if root.file_name() != Some(package.name.as_str()) {
            return Err(fpm::Error::ConfigurationError {
                message: "package name and folder name must match".to_string(),
            });
        }

        let ignored = {
            let mut overrides = ignore::overrides::OverrideBuilder::new("./");
            for ig in b.get::<Vec<String>>("fpm#ignore")? {
                if let Err(e) = overrides.add(format!("!{}", ig.as_str()).as_str()) {
                    return Err(fpm::Error::ConfigurationError {
                        message: format!("failed parse fpm.ignore: {} => {:?}", ig, e),
                    });
                }
            }

            match overrides.build() {
                Ok(v) => v,
                Err(e) => {
                    return Err(fpm::Error::ConfigurationError {
                        message: format!("failed parse fpm.ignore: {:?}", e),
                    });
                }
            }
        };

        fpm::dependency::ensure(root.clone(), deps.clone()).await?;

        Ok(Config {
            package,
            root,
            original_directory,
            fonts,
            dependencies: deps,
            ignored,
        })
    }
}

fn find_package_root(dir: &camino::Utf8Path) -> Option<camino::Utf8PathBuf> {
    if dir.join("FPM.ftd").exists() {
        Some(dir.into())
    } else {
        if let Some(p) = dir.parent() {
            return find_package_root(p);
        };
        None
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub about: Option<String>,
    pub domain: Option<String>,
}
