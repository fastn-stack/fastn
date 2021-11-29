#[derive(serde::Deserialize, Debug)]
pub struct Config {
    pub package: String,
    pub base_dir: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct ConfigFromFile {
    package: String,
}

impl Config {
    pub fn parse(base_dir: String) -> Config {
        let lib = fpm::Library {};
        let id = "fpm".to_string();
        let doc = std::fs::read_to_string(format!("{}/FPM.ftd", base_dir.as_str()))
            .expect(format!("cant read file. {}/FPM.ftd", base_dir.as_str()).as_str());
        let b = match ftd::p2::Document::from(id.as_str(), doc.as_str(), &lib) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("failed to parse {}: {:?}", id, &e);
                todo!();
            }
        };

        let config = { b.only_instance::<ConfigFromFile>("fpm#config").expect("") };
        let config = config.unwrap_or_else(|| {
            panic!(
                "Parsing of {} failed. Unable to find fpm#config",
                format!("{}/FPM.ftd", base_dir.as_str())
            )
        });
        Config {
            package: config.package,
            base_dir,
        }
    }
}
