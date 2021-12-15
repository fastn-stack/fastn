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
