use std::io::Read;

extern crate self as fastn_update;

static GITHUB_PAGES_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"([^/]+)\.github\.io/([^/]+)").unwrap());

fn extract_github_details(pages_url: &str) -> Option<(String, String)> {
    if let Some(captures) = GITHUB_PAGES_REGEX.captures(pages_url) {
        let username = captures.get(1).unwrap().as_str().to_string();
        let repository = captures.get(2).unwrap().as_str().to_string();
        Some((username, repository))
    } else {
        None
    }
}

// https://api.github.com/repos/User/repo/:archive_format/:ref
// https://stackoverflow.com/questions/8377081/github-api-download-zip-or-tarball-link
async fn resolve_dependency_from_gh(
    username: &str,
    repository: &str,
) -> fastn_core::Result<Vec<u8>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/zipball",
        username, repository
    );
    let tar_gz = fastn_core::http::http_get(&url).await?;
    Ok(tar_gz)
}

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

pub async fn unpack_package_archive<R>(
    config: &fastn_core::Config,
    target_dir: camino::Utf8PathBuf,
    mut archive: tar::Archive<flate2::read::GzDecoder<R>>,
) -> fastn_core::Result<()>
where
    R: std::io::Read,
{
    while let Some(entry) = archive.entries()?.next() {
        let mut entry = entry?;
        let entry_path = {
            let entry_path = entry.path()?;
            let path_str = entry_path.to_string_lossy();
            format!("{}/{}", &target_dir, path_str)
        };
        let mut buffer = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut buffer)?;
        config.ds.write_content(&entry_path, buffer).await?;
    }

    Ok(())
}

pub async fn resolve_dependencies(config: &fastn_core::Config) -> fastn_core::Result<()> {
    use std::io::{Seek, Write};

    for dependency in &config.package.dependencies {
        if let Some((username, repo)) = extract_github_details(dependency.package.name.as_str()) {
            let zipball = resolve_dependency_from_gh(username.as_str(), repo.as_str()).await?;
            let mut temp_file = tempfile::tempfile()?;
            temp_file.write_all(&zipball)?;
            temp_file.seek(std::io::SeekFrom::Start(0))?;
            let mut archive = zip::ZipArchive::new(temp_file)?;
            for i in 0..archive.len() {
                let mut entry = archive.by_index(i)?;

                if entry.is_file() {
                    let mut buffer = Vec::new();
                    entry.read_to_end(&mut buffer)?;
                    config
                        .ds
                        .write_content(&config.packages_root.join(entry.name()), buffer)
                        .await?;
                }
            }
        }
    }

    Ok(())
}

pub async fn update(config: &fastn_core::Config) -> fastn_core::Result<()> {
    if let Err(e) = std::fs::remove_dir_all(config.ds.root().join(".packages")) {
        match e.kind() {
            std::io::ErrorKind::NotFound => {}
            _ => return Err(e.into()),
        }
    };

    let c = fastn_core::Config::read_current(false).await?;
    if c.package.dependencies.is_empty() {
        println!("No dependencies to update.");
        return Ok(());
    }

    resolve_dependencies(config).await?;

    if c.package.dependencies.len() == 1 {
        println!("Updated the package dependency.");
    } else {
        println!("Updated {} dependencies.", c.package.dependencies.len())
    }

    Ok(())
}
