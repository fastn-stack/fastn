#[derive(serde::Deserialize, Debug)]
pub struct FPMConfig {
    pub package: String,
    pub base_dir: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct ConfigFromFile {
    package: String,
}

impl FPMConfig {
    pub fn parse(base_dir: String) -> FPMConfig {
        let lib = fpm::Library {};
        let id = "fpm".to_string();
        let doc = std::fs::read_to_string(format!("{}/.FPM.ftd", base_dir.as_str()))
            .expect("cant read file");
        let b = match ftd::p2::Document::from(id.as_str(), doc.as_str(), &lib) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("failed to parse {}: {:?}", id, &e);
                todo!();
            }
        };

        let config = {
            match b.only_instance::<ConfigFromFile>("fpm#config").expect("") {
                Some(v) => Some(v),
                None => None,
            }
        };
        let config = config.expect("Unable to find fpm#config");
        FPMConfig {
            package: config.package,
            base_dir,
        }
    }
}
