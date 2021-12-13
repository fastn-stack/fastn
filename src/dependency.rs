#[derive(serde::Deserialize, Debug, Clone)]
pub struct Dependency {
    pub package: fpm::Package,
    pub version: Option<String>,
    pub repo: String,
    pub notes: Option<String>,
}

impl Dependency {
    pub async fn process(&self, base_dir: camino::Utf8PathBuf) -> fpm::Result<()> {
        use tokio::io::AsyncWriteExt;
        if base_dir
            .join(".packages")
            .join(self.package.name.as_str())
            .exists()
        {
            // TODO: in future we will check if we have a new version in the package's repo.
            //       for now assume if package exists we have the latest package and if you
            //       want to update a package, delete the corresponding folder and latest
            //       version will get downloaded.
            return Ok(());
        }

        let path =
            camino::Utf8PathBuf::from(format!("/tmp/{}.zip", self.package.name.replace("/", "__")));

        {
            let download_url = match self.repo.as_str() {
                "github" => {
                    format!(
                        "https://github.com/{}/archive/refs/heads/main.zip",
                        self.package.name
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
                .join(self.package.name.as_str())
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

pub async fn ensure(base_dir: camino::Utf8PathBuf, deps: Vec<fpm::Dependency>) -> fpm::Result<()> {
    futures::future::join_all(
        deps.into_iter()
            .map(|x| (x, base_dir.clone()))
            .map(|(x, base_dir)| tokio::spawn(async move { x.process(base_dir).await }))
            .collect::<Vec<tokio::task::JoinHandle<_>>>(),
    )
    .await;
    Ok(())
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct DependencyTemp {
    pub name: String,
    pub version: Option<String>,
    pub repo: String,
    pub notes: Option<String>,
}

impl DependencyTemp {
    pub fn to_dependency(&self) -> fpm::Dependency {
        fpm::Dependency {
            package: fpm::Package::new(self.name.as_str()),
            version: self.version.to_owned(),
            repo: self.repo.to_owned(),
            notes: self.notes.to_owned(),
        }
    }
}
