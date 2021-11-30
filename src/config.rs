#[derive(serde::Deserialize, Debug)]
pub struct Package {
    pub name: String,
}

impl Package {
    pub fn parse(base_dir: String) -> Package {
        let lib = fpm::Library {};
        let id = "fpm".to_string();
        let doc = std::fs::read_to_string(format!("{}/FPM.ftd", base_dir.as_str()))
            .unwrap_or_else(|_| panic!("cant read file. {}/FPM.ftd", base_dir.as_str()));
        let b = match ftd::p2::Document::from(id.as_str(), doc.as_str(), &lib) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("failed to parse {}: {:?}", id, &e);
                todo!();
            }
        };

        // TODO(main): Error handling
        b.only_instance::<Package>("fpm#package").unwrap().unwrap()
    }
}
