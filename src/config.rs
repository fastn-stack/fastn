#[derive(Debug)]
pub struct Config {
    pub package: fpm::Package,
    pub root: camino::Utf8PathBuf,
    pub original_directory: camino::Utf8PathBuf,
    pub fonts: Vec<fpm::Font>,
    pub dependencies: Vec<fpm::Dependency>,
    pub ignored: ignore::overrides::Override,
    pub is_translation_package: bool,
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
        let package = {
            let temp_package: PackageTemp = b.get("fpm#package")?;
            temp_package.to_package()
        };
        let deps = {
            let temp_deps: Vec<fpm::dependency::DependencyTemp> = b.get("fpm#dependency")?;
            temp_deps
                .into_iter()
                .map(|v| v.to_dependency())
                .collect::<Vec<fpm::Dependency>>()
        };

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
        if let Some(translation_of) = package.translation_of.as_ref() {
            translation_of.process(root.clone()).await?;
        }

        Ok(Config {
            package: package.clone(),
            root,
            original_directory,
            fonts,
            dependencies: deps,
            ignored,
            is_translation_package: package.translation_of.is_some(),
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
pub struct PackageTemp {
    pub name: String,
    #[serde(rename = "translation-of")]
    pub translation_of: Option<String>,
    pub translations: Vec<String>,
    pub lang: Option<String>,
    pub about: Option<String>,
    pub domain: Option<String>,
}

impl PackageTemp {
    pub fn to_package(&self) -> fpm::Package {
        let translation_of = self.translation_of.as_ref().map(|v| fpm::Package::new(v));
        let translations = self
            .translations
            .clone()
            .into_iter()
            .map(|v| fpm::Package::new(&v))
            .collect::<Vec<fpm::Package>>();

        fpm::Package {
            name: self.name.to_owned(),
            translation_of: Box::new(translation_of),
            translations,
            lang: self.lang.to_owned(),
            about: self.about.to_owned(),
            domain: self.domain.to_owned(),
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
        let mut ignore_paths = ignore::WalkBuilder::new(path);
        ignore_paths.standard_filters(true);
        ignore_paths.overrides(config.ignored.clone());
        let all_files = ignore_paths
            .build()
            .into_iter()
            .flatten()
            .map(|x| x.into_path())
            .collect::<Vec<std::path::PathBuf>>();
        let mut documents =
            fpm::paths_to_files(all_files, config.root.join(".packages").as_str()).await?;
        documents.sort_by_key(|v| v.get_id());

        Ok(documents)
    }

    pub async fn process(&self, base_dir: camino::Utf8PathBuf) -> fpm::Result<()> {
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
            let download_url = format!(
                "https://github.com/{}/archive/refs/heads/main.zip",
                self.name
            );
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
