mod manifest_to_package;
pub mod utils;

pub const MANIFEST_FILE: &str = "manifest.json";

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Manifest {
    pub files: std::collections::BTreeMap<String, File>,
    pub zip_url: String,
    pub checksum: String,
}

impl Manifest {
    pub fn new(
        files: std::collections::BTreeMap<String, File>,
        zip_url: String,
        checksum: String,
    ) -> Self {
        Manifest {
            files,
            zip_url,
            checksum,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct File {
    pub name: String,
    pub checksum: String,
    pub size: usize,
}

impl File {
    pub fn new(name: String, checksum: String, size: usize) -> Self {
        File {
            name,
            checksum,
            size,
        }
    }
}

pub async fn write_manifest_file(
    config: &fastn_core::Config,
    build_dir: &fastn_ds::Path,
    zip_url: Option<String>,
) -> fastn_core::Result<()> {
    use sha2::digest::FixedOutput;
    use sha2::Digest;

    let start = std::time::Instant::now();

    print!(
        "Processing {}/{} ... ",
        &config.package.name.as_str(),
        fastn_core::manifest::MANIFEST_FILE
    );

    let zip_url = match zip_url {
        Some(zip_url) => zip_url,
        None => {
            match fastn_core::manifest::utils::get_gh_zipball_url(config.package.name.clone()) {
                Some(gh_zip_url) => gh_zip_url,
                None => {
                    return Err(fastn_core::error::Error::UsageError {
                        message: format!(
                            "Could not find zip url for package \"{}\".",
                            &config.package.name,
                        ),
                    });
                }
            }
        }
    };

    let mut hasher = sha2::Sha256::new();
    let mut files: std::collections::BTreeMap<String, fastn_core::manifest::File> =
        std::collections::BTreeMap::new();

    for file in config.get_files(&config.package).await? {
        if file.get_id().eq(fastn_core::manifest::MANIFEST_FILE) {
            continue;
        }

        let name = file.get_id().to_string();
        let content = &config.ds.read_content(&file.get_full_path()).await?;
        let hash = fastn_core::utils::generate_hash(content);
        let size = content.len();

        hasher.update(content);

        files.insert(
            name.clone(),
            fastn_core::manifest::File::new(name, hash, size),
        );
    }

    let checksum = format!("{:X}", hasher.finalize_fixed());

    let manifest = fastn_core::Manifest::new(files, zip_url, checksum);

    let mut serialized_manifest = serde_json::ser::to_vec_pretty(&manifest)?;
    // Append newline character
    serialized_manifest.push(b'\n');

    config
        .ds
        .write_content(
            &build_dir.join(fastn_core::manifest::MANIFEST_FILE),
            serialized_manifest,
        )
        .await?;

    fastn_core::utils::print_end(
        format!(
            "Processed {}/{}",
            &config.package.name.as_str(),
            fastn_core::manifest::MANIFEST_FILE
        )
        .as_str(),
        start,
    );

    Ok(())
}
