#[derive(serde::Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub about: Option<String>,
    pub domain: Option<String>,
}

impl Package {
    pub fn parse(b: &ftd::p2::Document) -> Package {
        // TODO(main): Error handling
        b.to_owned()
            .only_instance::<Package>("fpm#package")
            .unwrap()
            .unwrap()
    }
}
