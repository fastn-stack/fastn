#[derive(serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub packages: Vec<Package>,
}

impl Manifest {
    pub fn new(packages: Vec<Package>) -> Self {
        Manifest { packages }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Package {
    pub name: String,
    pub version: Option<String>,
    pub source: String,
    pub checksum: String,
    pub dependencies: Vec<String>,
}

impl Package {
    pub fn new(
        name: String,
        version: Option<String>,
        source: String,
        checksum: String,
        dependencies: Vec<String>,
    ) -> Self {
        Package {
            name,
            version,
            source,
            checksum,
            dependencies,
        }
    }
}
