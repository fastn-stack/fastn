use crate::utils::HasElements;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Dependency {
    pub package: fpm::Package,
    pub version: Option<String>,
    pub repo: String,
    pub notes: Option<String>,
}

pub fn ensure(
    base_dir: &camino::Utf8PathBuf,
    deps: Vec<fpm::Dependency>,
    package: &mut fpm::Package,
) -> fpm::Result<()> {
    /*futures::future::join_all(
        deps.into_iter()
            .map(|x| (x, base_dir.clone()))
            .map(|(x, base_dir)| {
                tokio::spawn(async move { x.package.process(base_dir, x.repo.as_str()).await })
            })
            .collect::<Vec<tokio::task::JoinHandle<_>>>(),
    )
    .await;*/
    // TODO: To convert it back to async. Not sure we can or should do it as `downloaded_package` would be
    //  referred and updated by all the dep.package.process. To make it async we have change this
    //  function to unsafe and downloaded_package as global static variable to have longer lifetime

    let mut downloaded_package = vec![package.name.clone()];
    for dep in deps {
        dep.package.process(
            base_dir,
            dep.repo.as_str(),
            &mut downloaded_package,
            false,
            true,
        )?;
    }

    if package.translations.has_elements() && package.translation_of.is_some() {
        return Err(fpm::Error::UsageError {
            message: "Package cannot be both original and translation package. \
            suggestion: Remove either `translation-of` or `translation` from FPM.ftd"
                .to_string(),
        });
    }

    for translation in package.translations.iter_mut() {
        if package.lang.is_none() {
            return Err(fpm::Error::UsageError {
                message: "Package needs to declare the language".to_string(),
            });
        }
        let package =
            translation.process(base_dir, "github", &mut downloaded_package, false, false)?;
        if let Some(package) = package.first() {
            *translation = package.to_owned();
        }
    }

    if let Some(translation_of) = package.translation_of.as_mut() {
        if package.lang.is_none() {
            return Err(fpm::Error::UsageError {
                message: "Translation package needs to declare the language".to_string(),
            });
        }
        let translation_packages =
            translation_of.process(base_dir, "github", &mut downloaded_package, true, true)?;
        translation_of.translations = translation_packages;
    }
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
    pub fn process(
        &self,
        base_dir: &camino::Utf8PathBuf,
        repo: &str,
        downloaded_package: &mut Vec<String>,
        download_translations: bool,
        download_dependencies: bool,
    ) -> fpm::Result<Vec<fpm::Package>> {
        use std::io::Write;
        // TODO: in future we will check if we have a new version in the package's repo.
        //       for now assume if package exists we have the latest package and if you
        //       want to update a package, delete the corresponding folder and latest
        //       version will get downloaded.

        if downloaded_package.contains(&self.name) {
            return Ok(Default::default());
        }

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

                let response = futures::executor::block_on(reqwest::get(download_url))?;
                let mut file = std::fs::File::create(&path)?;
                // TODO: instead of reading the whole thing in memory use tokio::io::copy() somehow?
                let content = futures::executor::block_on(response.bytes())?;
                file.write_all(&content)?;
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
            downloaded_package,
            self.name.as_str(),
            download_translations,
            download_dependencies,
        )
    }

    /// This function is called by `process()` or recursively called by itself.
    /// It checks the `FPM.ftd` file of dependent package and find out all the dependency packages.
    /// If dependent package is not available, it calls `process()` to download it inside `.packages` directory
    /// and if dependent package is available, it copies it to `.packages` directory
    /// At the end of both cases, `process_fpm()` is called again
    ///
    /// `process_fpm()`, together with `process()`, recursively make dependency packages available inside
    /// `.packages` directory
    ///
    // #[async_recursion::async_recursion]
    fn process_fpm(
        root: camino::Utf8PathBuf,
        base_path: &camino::Utf8PathBuf,
        downloaded_package: &mut Vec<String>,
        expected_package_name: &str,
        download_translations: bool,
        download_dependencies: bool,
    ) -> fpm::Result<Vec<fpm::Package>> {
        let root = match fpm::config::find_package_root(&root) {
            Some(b) => b,
            None => {
                return Ok(Default::default());
            }
        };
        let ftd_document = {
            let doc = std::fs::read_to_string(root.join("FPM.ftd"))?;
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

        if !package.name.eq(expected_package_name) {
            warning!(format!(
                "Could not able to download `{}` package. \
                Because in `{}/FPM.ftd` file, the package name is given as `{}`, \
                while expected name is `{}`",
                expected_package_name, expected_package_name, package.name, expected_package_name
            ));
            std::fs::remove_dir_all(root)?;
            return Ok(Default::default());
        }
        downloaded_package.push(expected_package_name.to_string());

        let deps = {
            let temp_deps: Vec<fpm::dependency::DependencyTemp> =
                ftd_document.get("fpm#dependency")?;
            temp_deps
                .into_iter()
                .map(|v| v.into_dependency())
                .collect::<Vec<fpm::Dependency>>()
        };

        if download_dependencies {
            for dep in deps {
                let dep_path = root.join(".packages").join(dep.package.name.as_str());
                if downloaded_package.contains(&dep.package.name) {
                    continue;
                }
                if dep_path.exists() {
                    let dst = base_path.join(".packages").join(dep.package.name.as_str());
                    if !dst.exists() {
                        futures::executor::block_on(fpm::copy_dir_all(dep_path, dst.clone()))?;
                    }
                    fpm::Package::process_fpm(
                        dst,
                        base_path,
                        downloaded_package,
                        dep.package.name.as_str(),
                        false,
                        true,
                    )?;
                }
                dep.package.process(
                    base_path,
                    dep.repo.as_str(),
                    downloaded_package,
                    false,
                    true,
                )?;
            }
        }

        if download_translations {
            if let Some(translation_of) = package.translation_of.as_ref() {
                return Err(fpm::Error::PackageError {
                    message: format!(
                        "Cannot translated a translation package. \
                    suggestion: Translated the original package instead. \
                    Looks like `{}` is an original package",
                        translation_of.name
                    ),
                });
            }
            let mut translation_packages = vec![];
            for translation in package.translations.iter() {
                let original_path = root.join(".packages").join(translation.name.as_str());
                if downloaded_package.contains(&translation.name) {
                    continue;
                }
                if original_path.exists() {
                    let dst = base_path.join(".packages").join(translation.name.as_str());
                    if !dst.exists() {
                        futures::executor::block_on(fpm::copy_dir_all(original_path, dst.clone()))?;
                    }
                    translation_packages.extend(fpm::Package::process_fpm(
                        dst,
                        base_path,
                        downloaded_package,
                        translation.name.as_str(),
                        false,
                        false,
                    )?);
                } else {
                    translation_packages.extend(translation.process(
                        base_path,
                        "github",
                        downloaded_package,
                        false,
                        false,
                    )?);
                }
            }
        }
        Ok(vec![package])
    }
}
