extern crate self as fastn_update;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub packages: Vec<Package>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Package {
    pub name: String,
    pub version: Option<String>,
    pub source: String,
    pub checksum: String,
    pub dependencies: Vec<String>,
}

pub async fn update(config: &fastn_core::Config) -> fastn_core::Result<()> {
    if let Err(e) = std::fs::remove_dir_all(config.root.join(".packages")) {
        match e.kind() {
            std::io::ErrorKind::NotFound => {}
            _ => return Err(e.into()),
        }
    };

    let c = fastn_core::Config::read_current(false).await?;
    if c.package.dependencies.is_empty() {
        println!("No dependencies to update.")
    } else if c.package.dependencies.len() == 1 {
        println!("Updated the package dependency.")
    } else {
        println!("Updated {} dependencies.", c.package.dependencies.len())
    }

    Ok(())
}

// https://api.github.com/repos/User/repo/:archive_format/:ref
// https://stackoverflow.com/questions/8377081/github-api-download-zip-or-tarball-link
