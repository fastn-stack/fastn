pub struct Config {
    pub package: fpm::Package,
    pub root: String,
    pub fonts: Vec<fpm::Font>,
    pub dependencies: Vec<fpm::Dependency>,
}

impl Config {
    pub fn get_font_style(&self) -> String {
        let generated_style = self
            .fonts
            .iter()
            .fold("".to_string(), |c, f| format!("{}\n{}", c, f.to_html()));
        return match generated_style.is_empty() {
            false => format!("<style>{}</style>", generated_style),
            _ => format!(""),
        };
    }

    pub async fn process_dependencies(&self) -> fpm::Result<()> {
        Ok(())
    }

    pub async fn read() -> fpm::Result<Config> {
        let root_dir = std::env::current_dir()
            .expect("Panic1")
            .to_str()
            .expect("panic")
            .to_string();
        let (_, package_folder_name) = root_dir.as_str().rsplit_once("/").expect("");
        let (_is_okay, base_dir) = find_fpm_file(root_dir.clone());

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
        // fpm::Error::ConfigurationParseError {
        //     message: "".to_string,
        //     line_number: 1,
        // }
        let package =
            fpm::Package::parse(&b)?.ok_or_else(|| fpm::Error::ConfigurationParseError {
                message: "".to_string(),
                line_number: 1,
            })?;
        let dep = fpm::Dependency::parse(&b)?;

        let fonts = fpm::Font::parse(&b);
        // futures::future::join_all(dep).await;

        if package_folder_name != package.name {
            todo!("package directory name mismatch")
        }

        let c = Config {
            package,
            root: base_dir,
            fonts,
            dependencies: dep,
        };
        c.process_dependencies().await?;

        Ok(c)
    }
}

fn find_fpm_file(dir: String) -> (bool, String) {
    if std::path::Path::new(format!("{}/FPM.ftd", dir).as_str()).exists() {
        (true, dir)
    } else {
        if let Some((parent_dir, _)) = dir.rsplit_once("/") {
            return find_fpm_file(parent_dir.to_string());
        };
        (false, "".to_string())
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub about: Option<String>,
    pub domain: Option<String>,
}

impl Package {
    pub fn parse(b: &ftd::p2::Document) -> fpm::Result<Option<Package>> {
        Ok(b.to_owned().only_instance::<Package>("fpm#package")?)
    }
}
