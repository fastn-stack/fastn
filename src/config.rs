#[derive(serde::Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub about: Option<String>,
    pub domain: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub repo: String,
    pub notes: Option<String>,
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

impl Dependency {
    pub fn parse(b: &ftd::p2::Document) -> Vec<Dependency> {
        b.to_owned().instances("fpm#dependency").unwrap()
    }
}
