#[derive(serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub files: Vec<String>,
    pub zip_url: String,
}

impl Manifest {
    pub fn new(files: Vec<String>, zip_url: String) -> Self {
        Manifest { files, zip_url }
    }
}
