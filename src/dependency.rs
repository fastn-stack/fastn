use std::io::Write;

pub trait DependencyProvider {
    fn download(&self) -> bool;
}

#[derive(serde::Deserialize, Debug)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub repo: String,
    pub notes: Option<String>,
}

impl Dependency {
    pub fn parse(b: &ftd::p2::Document) -> Vec<Dependency> {
        b.to_owned().instances("fpm#dependency").unwrap()
    }

    pub async fn download_zip(&self, download_url: &str) -> Result<std::fs::File, std::io::Error> {
        let response = reqwest::get(download_url);

        let path = std::path::Path::new("./download.zip");
        dbg!(">>>>>>");

        let mut file = match std::fs::File::create(&path) {
            Err(why) => panic!("couldn't create {}", why),
            Ok(file) => file,
        };
        let content = response.await.expect("").bytes().await.expect("");

        file.write_all(&content).expect("");
        Ok(file)
    }

    pub async fn download(&self) -> bool {
        self.download_zip(self.repo.as_str()).await.is_ok()
    }
}
