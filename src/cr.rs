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
        return fpm::usage_error(format!("CR#{} doesn't exist", cr_number));
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
            eprintln!("failed to parse .about.ftd for CR#{}: {:?}", cr_number, &e);
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct CRDeleted {
    pub filename: String,
    pub version: i32,
}

impl CRDeleted {
    pub(crate) fn new(filename: &str, version: i32) -> CRDeleted {
        CRDeleted {
            filename: filename.to_string(),
            version,
        }
    }
}

pub(crate) async fn get_deleted_files(
    config: &fpm::Config,
    cr_number: usize,
) -> fpm::Result<Vec<CRDeleted>> {
    if !config.cr_path(cr_number).exists() {
        return fpm::usage_error(format!("CR#{} doesn't exist", cr_number));
    }
    let deleted_files_path = config.cr_deleted_file_path(cr_number);
    if !deleted_files_path.exists() {
        return Ok(vec![]);
    }
    let deleted_files_content = tokio::fs::read_to_string(&deleted_files_path).await?;
    resolve_cr_deleted(deleted_files_content.as_str(), cr_number).await
}

pub(crate) async fn resolve_cr_deleted(
    content: &str,
    cr_number: usize,
) -> fpm::Result<Vec<CRDeleted>> {
    if content.trim().is_empty() {
        return Ok(vec![]);
    }
    let lib = fpm::FPMLibrary::default();
    let b = match fpm::doc::parse_ftd("deleted.ftd", content, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse deleted.ftd for CR#{}: {:?}", cr_number, &e);
            todo!();
        }
    };

    Ok(b.get("fpm#cr-deleted")?)
}

pub(crate) async fn create_deleted_files(
    config: &fpm::Config,
    cr_number: usize,
    cr_deleted: &[CRDeleted],
) -> fpm::Result<()> {
    let cr_deleted_content = generate_deleted_files_content(cr_deleted);
    fpm::utils::update(
        &config.cr_deleted_file_path(cr_number),
        cr_deleted_content.as_bytes(),
    )
    .await?;
    Ok(())
}

pub(crate) fn generate_deleted_files_content(cr_deleted_files: &[CRDeleted]) -> String {
    let mut deleted_files_content = vec!["-- import: fpm".to_string()];

    for cr_deleted_file in cr_deleted_files {
        let content = format!(
            "-- fpm.cr-deleted: {}\nversion: {}",
            cr_deleted_file.filename, cr_deleted_file.version
        );
        deleted_files_content.push(content)
    }

    let content = deleted_files_content.join("\n\n");
    format!("{content}\n")
}

impl fpm::Config {
    pub(crate) fn cr_path_to_file_name(
        &self,
        cr_number: usize,
        cr_file_path: &str,
    ) -> fpm::Result<String> {
        let cr_path = self.cr_path(cr_number);
        let cr_path_without_root = cr_path.strip_prefix(&self.root)?;
        Ok(cr_file_path
            .replace(cr_path_without_root.to_string().as_str(), "")
            .trim_matches('/')
            .to_string())
    }

    #[allow(dead_code)]
    pub(crate) async fn get_cr_tracking_info(
        &self,
        cr_number: usize,
    ) -> fpm::Result<Vec<fpm::track::TrackingInfo>> {
        let cr_track_paths = ignore::WalkBuilder::new(self.cr_track_dir(cr_number))
            .build()
            .into_iter()
            .flatten()
            .map(|x| camino::Utf8PathBuf::from_path_buf(x.into_path()).unwrap())
            .filter(|x| x.is_file() && x.extension().map(|v| v.eq("track")).unwrap_or(false))
            .collect::<Vec<camino::Utf8PathBuf>>();
        let mut tracking_infos = vec![];

        for cr_track_path in cr_track_paths {
            let tracked_file = cr_track_path.strip_prefix(self.track_dir())?;
            let tracked_file_str = self.cr_path_to_file_name(cr_number, tracked_file.as_str())?;
            let cr_tracking_infos = fpm::track::get_tracking_info_(&cr_track_path).await?;
            if let Some(tracking_info) = cr_tracking_infos
                .into_iter()
                .find(|v| tracked_file_str.eq(&v.filename))
            {
                tracking_infos.push(tracking_info);
            }
        }
        Ok(tracking_infos)
    }
}

pub(crate) fn cr_path(cr_number: usize) -> String {
    format!("-/{}", cr_number)
}
