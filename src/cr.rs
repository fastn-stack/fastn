#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct CRAbout {
    pub title: String, // relative file name with respect to package root
    pub description: Option<String>,
    #[serde(rename = "cr-number")]
    pub cr_number: usize,
    pub open: bool,
}

impl CRAbout {
    pub(crate) fn unset_open(self) -> CRAbout {
        CRAbout {
            title: self.title,
            description: self.description,
            cr_number: self.cr_number,
            open: false,
        }
    }
}

pub(crate) async fn get_cr_about(
    config: &fpm::Config,
    cr_number: usize,
) -> fpm::Result<fpm::cr::CRAbout> {
    let cr_about_path = config.cr_path(cr_number).join("-/about.ftd");
    if !cr_about_path.exists() {
        return fpm::usage_error(format!("CR#{} doesn't exists", cr_number));
    }

    let doc = std::fs::read_to_string(&cr_about_path)?;
    resolve_cr_about(&doc, cr_number).await
}

pub(crate) async fn resolve_cr_about(
    content: &str,
    cr_number: usize,
) -> fpm::Result<fpm::cr::CRAbout> {
    #[derive(serde::Deserialize)]
    struct CRAboutTemp {
        pub title: String,
        pub description: Option<String>,
        pub open: Option<bool>,
    }

    impl CRAboutTemp {
        fn into_cr_about(self, cr_number: usize) -> CRAbout {
            CRAbout {
                title: self.title,
                description: self.description,
                cr_number,
                open: self.open.unwrap_or(true),
            }
        }
    }

    if content.trim().is_empty() {
        return Err(fpm::Error::UsageError {
            message: "Content is empty in cr about".to_string(),
        });
    }
    let lib = fpm::FPMLibrary::default();
    let b = match fpm::doc::parse_ftd(".about.ftd", content, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse .latest.ftd: {:?}", &e);
            todo!();
        }
    };

    Ok(b.get::<CRAboutTemp>("fpm#cr-about")?
        .into_cr_about(cr_number))
}

pub(crate) async fn create_cr_about(
    config: &fpm::Config,
    cr_about: &fpm::cr::CRAbout,
) -> fpm::Result<()> {
    let about_content = generate_cr_about_content(cr_about);
    fpm::utils::update(
        &config.cr_about_path(cr_about.cr_number),
        about_content.as_bytes(),
    )
    .await?;
    Ok(())
}

pub(crate) fn generate_cr_about_content(cr_about: &fpm::cr::CRAbout) -> String {
    let mut about_content = format!("-- import: fpm\n\n\n-- fpm.cr-about: {}", cr_about.title,);
    if !cr_about.open {
        about_content = format!("{}\n{}", about_content, cr_about.open);
    }
    if let Some(ref description) = cr_about.description {
        about_content = format!("{}\n\n{}", about_content, description);
    }
    format!("{about_content}\n")
}

pub(crate) async fn is_open_cr_exists(config: &fpm::Config, cr_number: usize) -> fpm::Result<bool> {
    get_cr_about(config, cr_number).await.map(|v| v.open)
}
