pub const MANIFEST_FILE: &str = "manifest.json";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub files: std::collections::HashMap<String, File>,
    pub zip_url: String,
    pub checksum: String,
}

impl Manifest {
    pub fn new(
        files: std::collections::HashMap<String, File>,
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

#[derive(serde::Serialize, serde::Deserialize)]
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
