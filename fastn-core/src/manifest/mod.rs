pub const MANIFEST_FILE: &str = "manifest.json";

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Manifest {
    pub files: std::collections::HashMap<String, File>,
    pub zip_url: String,
    pub checksum: String,
}

impl Manifest {
    pub fn new(
        files: std::collections::HashMap<String, File>,
        zip_url: &str,
        checksum: String,
    ) -> Self {
        Manifest {
            files,
            zip_url: zip_url.to_string(),
            checksum,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct File {
    pub name: String,
    pub hash: String,
    pub size: usize,
}

impl File {
    pub fn new(name: String, hash: String, size: usize) -> Self {
        File { name, hash, size }
    }
}
