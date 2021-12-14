#[derive(Debug, Clone)]
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

    pub fn is_translation_package(&self) -> bool {
        self.package.translation_of.is_some()
    }

    pub async fn get_translation_snapshots(
        &self,
    ) -> fpm::Result<std::collections::BTreeMap<String, u128>> {
        if let Some(ref original) = self.package.translation_of.as_ref() {
            let original_path = self.root.join(".packages").join(original.name.as_str());
            return fpm::snapshot::get_latest_snapshots(&original_path).await;
        }
        // not sure if error should be returned
        Ok(std::collections::BTreeMap::new())
    }

    pub async fn get_translation_documents(&self) -> fpm::Result<Vec<fpm::File>> {
        if let Some(ref original) = self.package.translation_of.as_ref() {
            let src = self.root.join(".packages").join(original.name.as_str());
            let dst = self.root.join(".packages/.tmp");
            fpm::utils::copy_dir_all(&src, &dst)?;
            let tmp_package = {
                let mut tmp = original.clone();
                tmp.name = dst.as_str().to_string();
                tmp
            };
            return tmp_package.get_documents(self).await;
        }
        // not sure if error should be returned
        Ok(vec![])
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

    pub async fn read_translation(&self) -> fpm::Result<Config> {
        if let Some(ref original) = self.package.translation_of.as_ref() {
            let root = self.root.join(".packages").join(original.name.as_str());
            return Config::read_by_path(root, self.root.clone()).await;
        }
        Err(fpm::Error::ConfigurationError {
            message: "not a translation package".to_string(),
        })
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
        Config::read_by_path(root.clone(), root).await
    }

    pub async fn read_by_path(
        path: camino::Utf8PathBuf,
        root: camino::Utf8PathBuf,
    ) -> fpm::Result<Config> {
        let original_directory: camino::Utf8PathBuf =
            std::env::current_dir()?.canonicalize()?.try_into()?;
        let b = {
            let doc = tokio::fs::read_to_string(path.join("FPM.ftd")).await?;
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
        let package = {
            let temp_package: PackageTemp = b.get("fpm#package")?;
            temp_package.into_package()
        };
        let deps = {
            let temp_deps: Vec<fpm::dependency::DependencyTemp> = b.get("fpm#dependency")?;
            temp_deps
                .into_iter()
                .map(|v| v.into_dependency())
                .collect::<Vec<fpm::Dependency>>()
        };

        let fonts: Vec<fpm::Font> = b.get("fpm#font")?;

        if path.file_name() != Some(package.name.as_str()) {
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
        if let Some(translation_of) = package.translation_of.as_ref() {
            translation_of.process(root.clone(), "github").await?;
        }

        Ok(Config {
            package,
            root: path,
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

#[derive(serde::Deserialize, Debug, Clone)]
struct PackageTemp {
    pub name: String,
    #[serde(rename = "translation-of")]
    pub translation_of: Option<String>,
    pub translations: Vec<String>,
    pub lang: Option<String>,
    pub about: Option<String>,
    pub domain: Option<String>,
}

impl PackageTemp {
    pub fn into_package(self) -> fpm::Package {
        let translation_of = self.translation_of.as_ref().map(|v| fpm::Package::new(v));
        let translations = self
            .translations
            .clone()
            .into_iter()
            .map(|v| fpm::Package::new(&v))
            .collect::<Vec<fpm::Package>>();

        fpm::Package {
            name: self.name,
            translation_of: Box::new(translation_of),
            translations,
            lang: self.lang,
            about: self.about,
            domain: self.domain,
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Package {
    pub name: String,
    pub translation_of: Box<Option<Package>>,
    pub translations: Vec<Package>,
    pub lang: Option<String>,
    pub about: Option<String>,
    pub domain: Option<String>,
}

impl Package {
    pub fn new(name: &str) -> fpm::Package {
        fpm::Package {
            name: name.to_string(),
            translation_of: Box::new(None),
            translations: vec![],
            lang: None,
            about: None,
            domain: None,
        }
    }

    pub(crate) async fn get_documents(&self, config: &fpm::Config) -> fpm::Result<Vec<fpm::File>> {
        let path = config.root.join(".packages").join(self.name.as_str());
        fpm::get_documents(&ignore::WalkBuilder::new(path), config).await
    }

    pub async fn process(&self, base_dir: camino::Utf8PathBuf, repo: &str) -> fpm::Result<()> {
        use tokio::io::AsyncWriteExt;
        if base_dir.join(".packages").join(self.name.as_str()).exists() {
            // TODO: in future we will check if we have a new version in the package's repo.
            //       for now assume if package exists we have the latest package and if you
            //       want to update a package, delete the corresponding folder and latest
            //       version will get downloaded.
            return Ok(());
        }

        let path = camino::Utf8PathBuf::from(format!("/tmp/{}.zip", self.name.replace("/", "__")));

        {
            let download_url = match repo {
                "github" => {
                    format!(
                        "https://github.com/{}/archive/refs/heads/main.zip",
                        self.name
                    )
                }
                k => k.to_string(),
            };
            let response = reqwest::get(download_url).await?;
            let mut file = tokio::fs::File::create(&path).await?;
            // TODO: instead of reading the whole thing in memory use tokio::io::copy() somehow?
            let content = response.bytes().await?;
            file.write_all(&content).await?;
        }

        let file = std::fs::File::open(&path)?;
        // TODO: switch to async_zip crate
        let mut archive = zip::ZipArchive::new(file)?;
        for i in 0..archive.len() {
            let mut c_file = archive.by_index(i).unwrap();
            let out_path = match c_file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };
            let out_path_without_folder = out_path.to_str().unwrap().split_once("/").unwrap().1;
            let file_extract_path = base_dir
                .join(".packages")
                .join(self.name.as_str())
                .join(out_path_without_folder);
            if (&*c_file.name()).ends_with('/') {
                std::fs::create_dir_all(&file_extract_path)?;
            } else {
                if let Some(p) = file_extract_path.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                // Note: we will be able to use tokio::io::copy() with async_zip
                let mut outfile = std::fs::File::create(file_extract_path)?;
                std::io::copy(&mut c_file, &mut outfile)?;
            }
        }
        Ok(())
    }
}
