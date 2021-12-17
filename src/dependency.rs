#[derive(serde::Deserialize, Debug, Clone)]
pub struct Dependency {
    pub package: fpm::Package,
    pub version: Option<String>,
    pub repo: String,
    pub notes: Option<String>,
}

impl Dependency {
    pub async fn process(&self, base_dir: camino::Utf8PathBuf) -> fpm::Result<()> {
        self.package.process(base_dir, self.repo.as_str()).await
    }
}

pub async fn ensure(base_dir: camino::Utf8PathBuf, deps: Vec<fpm::Dependency>) -> fpm::Result<()> {
    futures::future::join_all(
        deps.into_iter()
            .map(|x| (x, base_dir.clone()))
            .map(|(x, base_dir)| {
                tokio::spawn(async move { x.package.process(base_dir, x.repo.as_str()).await })
            })
            .collect::<Vec<tokio::task::JoinHandle<_>>>(),
    )
    .await;
    Ok(())
}

#[derive(serde::Deserialize, Debug, Clone)]
pub(crate) struct DependencyTemp {
    pub name: String,
    pub version: Option<String>,
    pub repo: String,
    pub notes: Option<String>,
}

impl DependencyTemp {
    pub(crate) fn into_dependency(self) -> fpm::Dependency {
        fpm::Dependency {
            package: fpm::Package::new(self.name.as_str()),
            version: self.version,
            repo: self.repo,
            notes: self.notes,
        }
    }
}

impl fpm::Package {
    /// `process()` checks the package exists in `.packages` or `FPM_HOME` folder (`FPM_HOME` not
    /// yet implemented), and if not downloads and unpacks the method.
    /// It then calls `process_fpm()` which checks the dependencies of the downloaded packages and
    /// then again call `process()` if dependent package is not downloaded or available
    pub async fn process(&self, base_dir: camino::Utf8PathBuf, repo: &str) -> fpm::Result<()> {
        use tokio::io::AsyncWriteExt;
        // TODO: in future we will check if we have a new version in the package's repo.
        //       for now assume if package exists we have the latest package and if you
        //       want to update a package, delete the corresponding folder and latest
        //       version will get downloaded.

        if !base_dir.join(".packages").join(self.name.as_str()).exists() {
            let path =
                camino::Utf8PathBuf::from(format!("/tmp/{}.zip", self.name.replace("/", "__")));

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
        }

        fpm::Package::process_fpm(
            base_dir.join(".packages").join(self.name.as_str()),
            base_dir,
        )
        .await
    }

    /// This function is called by `process()` or recursively called by itself.
    /// It checks the `FPM.ftd` file of dependent package and find out all the dependency packages.
    /// If dependent package is not available, it calls `process()` to download it inside `.packages` directory
    /// and if dependent package is available, it copies it to `.packages` directory
    /// At the end of both cases, `process_fpm()` is called again
    ///
    /// `process_fpm()`, together with `process()`, recursively make dependency packages available inside
    /// `.packages` directory
    #[async_recursion::async_recursion]
    async fn process_fpm(
        root: camino::Utf8PathBuf,
        base_path: camino::Utf8PathBuf,
    ) -> fpm::Result<()> {
        let root = match fpm::config::find_package_root(&root) {
            Some(b) => b,
            None => {
                return Ok(());
            }
        };
        let ftd_document = {
            let doc = tokio::fs::read_to_string(root.join("FPM.ftd")).await?;
            let lib = fpm::FPMLibrary::default();
            match ftd::p2::Document::from("FPM", doc.as_str(), &lib) {
                Ok(v) => v,
                Err(e) => {
                    return Err(fpm::Error::PackageError {
                        message: format!("failed to parse FPM.ftd: {:?}", &e),
                    });
                }
            }
        };
        let package = {
            let temp_package: fpm::config::PackageTemp = ftd_document.get("fpm#package")?;
            temp_package.into_package()
        };
        let deps = {
            let temp_deps: Vec<fpm::dependency::DependencyTemp> =
                ftd_document.get("fpm#dependency")?;
            temp_deps
                .into_iter()
                .map(|v| v.into_dependency())
                .collect::<Vec<fpm::Dependency>>()
        };

        for dep in deps {
            let dep_path = root.join(".packages").join(dep.package.name.as_str());
            if dep_path.exists() {
                let dst = base_path.join(".packages").join(dep.package.name.as_str());
                if !dst.exists() {
                    futures::executor::block_on(fpm::copy_dir_all(dep_path, dst.clone()))?;
                }
                fpm::Package::process_fpm(dst, base_path.clone()).await?;
            }
            dep.package
                .process(base_path.clone(), dep.repo.as_str())
                .await?;
        }

        if let Some(translation_of) = package.translation_of.as_ref() {
            let original_path = root.join(".packages").join(translation_of.name.as_str());
            if original_path.exists() {
                let dst = base_path
                    .join(".packages")
                    .join(translation_of.name.as_str());
                if !dst.exists() {
                    futures::executor::block_on(fpm::copy_dir_all(original_path, dst.clone()))?;
                }
                fpm::Package::process_fpm(dst, base_path.clone()).await?;
            } else {
                translation_of.process(base_path.clone(), "github").await?;
            }
        }
        Ok(())
    }
}
