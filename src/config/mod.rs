// Document: https://fpm.dev/crate/config/
// Document: https://fpm.dev/crate/package/

pub(crate) mod utils;

#[derive(Debug, Clone)]
pub struct Config {
    // Global Information
    pub package: fpm::Package,
    pub root: camino::Utf8PathBuf,
    pub packages_root: camino::Utf8PathBuf,
    pub original_directory: camino::Utf8PathBuf,
    pub all_packages: std::cell::RefCell<std::collections::BTreeMap<String, fpm::Package>>,
    pub downloaded_assets: std::collections::BTreeMap<String, String>,
    pub global_ids: std::collections::HashMap<String, String>,
    // Related to current request, or per request
    pub extra_data: serde_json::Map<String, serde_json::Value>,
    pub path_parameters: Vec<(String, ftd::Value)>,
    pub current_document: Option<String>,
    pub request: Option<fpm::http::Request>, // TODO: It should only contain reference
}

impl Config {
    /// `build_dir` is where the static built files are stored. `fpm build` command creates this
    /// folder and stores its output here.
    pub fn build_dir(&self) -> camino::Utf8PathBuf {
        self.root.join(".build")
    }

    pub fn clone_dir(&self) -> camino::Utf8PathBuf {
        self.root.join(".clone-state")
    }

    pub fn workspace_file(&self) -> camino::Utf8PathBuf {
        self.clone_dir().join("workspace.ftd")
    }

    pub fn clone_available_crs_path(&self) -> camino::Utf8PathBuf {
        self.clone_dir().join("cr")
    }

    pub fn cr_path(&self, cr_number: usize) -> camino::Utf8PathBuf {
        self.root.join(fpm::cr::cr_path(cr_number))
    }

    pub fn path_without_root(&self, path: &camino::Utf8PathBuf) -> fpm::Result<String> {
        Ok(path.strip_prefix(&self.root)?.to_string())
    }

    pub fn cr_deleted_file_path(&self, cr_number: usize) -> camino::Utf8PathBuf {
        self.cr_path(cr_number).join("-/deleted.ftd")
    }

    pub fn track_path(&self, path: &camino::Utf8PathBuf) -> camino::Utf8PathBuf {
        let path_without_root = self
            .path_without_root(path)
            .unwrap_or_else(|_| path.to_string());
        let track_path = format!("{}.track", path_without_root);
        self.track_dir().join(track_path)
    }

    pub fn cr_track_dir(&self, cr_number: usize) -> camino::Utf8PathBuf {
        self.track_dir().join(fpm::cr::cr_path(cr_number))
    }

    pub fn cr_track_path(
        &self,
        path: &camino::Utf8PathBuf,
        cr_number: usize,
    ) -> camino::Utf8PathBuf {
        let path_without_root = self
            .cr_path(cr_number)
            .join(path)
            .to_string()
            .replace(self.root.to_string().as_str(), "");
        let track_path = format!("{}.track", path_without_root);
        self.track_dir().join(track_path)
    }

    pub fn cr_about_path(&self, cr_number: usize) -> camino::Utf8PathBuf {
        self.cr_path(cr_number).join("-/about.ftd")
    }

    pub fn cr_meta_path(&self, cr_number: usize) -> camino::Utf8PathBuf {
        self.cr_path(cr_number).join("-/meta.ftd")
    }

    pub fn remote_dir(&self) -> camino::Utf8PathBuf {
        self.root.join(".remote-state")
    }

    pub fn remote_history_dir(&self) -> camino::Utf8PathBuf {
        self.remote_dir().join("history")
    }

    /// location that stores lowest available cr number
    pub fn remote_cr(&self) -> camino::Utf8PathBuf {
        self.remote_dir().join("cr")
    }

    pub fn history_file(&self) -> camino::Utf8PathBuf {
        self.remote_dir().join("history.ftd")
    }

    pub(crate) fn history_path(&self, id: &str, version: i32) -> camino::Utf8PathBuf {
        let id_with_timestamp_extension = fpm::utils::snapshot_id(id, &(version as u128));
        self.remote_history_dir().join(id_with_timestamp_extension)
    }

    /// document_name_with_default("index.ftd") -> /
    /// document_name_with_default("foo/index.ftd") -> /foo/
    /// document_name_with_default("foo/abc") -> /foo/abc/
    /// document_name_with_default("/foo/abc.ftd") -> /foo/abc/
    pub(crate) fn document_name_with_default(&self, document_path: &str) -> String {
        let name = self
            .doc_id()
            .unwrap_or_else(|| document_path.to_string())
            .trim_matches('/')
            .to_string();
        if name.is_empty() {
            "/".to_string()
        } else {
            format!("/{}/", name)
        }
    }

    /// history of a fpm package is stored in `.history` folder.
    ///
    /// Current design is wrong, we should move this helper to `fpm::Package` maybe.
    ///
    /// History of a package is considered part of the package, and when a package is downloaded we
    /// have to chose if we want to download its history as well. For now we do not. Eventually in
    /// we will be able to say download the history also for some package.
    ///
    /// ```ftd
    /// -- fpm.dependency: django
    ///  with-history: true
    /// ```
    ///     
    /// `.history` file is created or updated by `fpm sync` command only, no one else should edit
    /// anything in it.
    pub fn history_dir(&self) -> camino::Utf8PathBuf {
        self.root.join(".history")
    }

    pub fn fpm_dir(&self) -> camino::Utf8PathBuf {
        self.root.join(".fpm")
    }

    pub fn conflicted_dir(&self) -> camino::Utf8PathBuf {
        self.fpm_dir().join("conflicted")
    }

    /// every package's `.history` contains a file `.latest.ftd`. It looks a bit link this:
    ///
    /// ```ftd
    /// -- import: fpm
    ///
    /// -- fpm.snapshot: FPM.ftd
    /// timestamp: 1638706756293421000
    ///
    /// -- fpm.snapshot: blog.ftd
    /// timestamp: 1638706756293421000
    /// ```
    ///
    /// One `fpm.snapshot` for every file that is currently part of the package.
    pub fn latest_ftd(&self) -> camino::Utf8PathBuf {
        self.root.join(".history/.latest.ftd")
    }

    /// track_dir returns the directory where track files are stored. Tracking information as well
    /// is considered part of a package, but it is not downloaded when a package is downloaded as
    /// a dependency of another package.
    pub fn track_dir(&self) -> camino::Utf8PathBuf {
        self.root.join(".tracks")
    }

    /// `is_translation_package()` is a helper to tell you if the current package is a translation
    /// of another package. We may delete this helper soon.
    pub fn is_translation_package(&self) -> bool {
        self.package.translation_of.is_some()
    }

    /// original_path() returns the path of the original package if the current package is a
    /// translation package. it returns the path in `.packages` folder where the
    pub fn original_path(&self) -> fpm::Result<camino::Utf8PathBuf> {
        let o = match self.package.translation_of.as_ref() {
            Some(ref o) => o,
            None => {
                return Err(fpm::Error::UsageError {
                    message: "This package is not a translation package".to_string(),
                });
            }
        };
        match &o.fpm_path {
            Some(fpm_path) => Ok(fpm_path
                .parent()
                .expect("Expect fpm_path parent. Panic!")
                .to_owned()),
            _ => Err(fpm::Error::UsageError {
                message: format!("Unable to find `fpm_path` of the package {}", o.name),
            }),
        }
    }

    /*/// aliases() returns the list of the available aliases at the package level.
    pub fn aliases(&self) -> fpm::Result<std::collections::BTreeMap<&str, &fpm::Package>> {
        let mut resp = std::collections::BTreeMap::new();
        self.package
            .dependencies
            .iter()
            .filter(|d| d.alias.is_some())
            .for_each(|d| {
                resp.insert(d.alias.as_ref().unwrap().as_str(), &d.package);
            });
        Ok(resp)
    }*/

    /// `get_font_style()` returns the HTML style tag which includes all the fonts used by any
    /// ftd document. Currently this function does not check for fonts in package dependencies
    /// nor it tries to avoid fonts that are configured but not needed in current document.
    pub fn get_font_style(&self) -> String {
        use itertools::Itertools;
        // TODO: accept list of actual fonts used in the current document. each document accepts
        //       a different list of fonts and only fonts used by a given document should be
        //       included in the HTML produced by that font
        // TODO: fetch fonts from package dependencies as well (ideally this function should fail
        //       if one of the fonts used by any ftd document is not found

        let generated_style = {
            let mut generated_style = self
                .package
                .get_flattened_dependencies()
                .into_iter()
                .unique_by(|dep| dep.package.name.clone())
                .collect_vec()
                .iter()
                .fold(self.package.get_font_html(), |accumulator, dep| {
                    format!(
                        "{pre}\n{new}",
                        pre = accumulator,
                        new = dep.package.get_font_html()
                    )
                });
            generated_style = self.all_packages.borrow().values().fold(
                generated_style,
                |accumulator, package| {
                    format!(
                        "{pre}\n{new}",
                        pre = accumulator,
                        new = package.get_font_html()
                    )
                },
            );
            generated_style
        };
        return match generated_style.trim().is_empty() {
            false => format!("<style>{}</style>", generated_style),
            _ => "".to_string(),
        };
    }

    pub(crate) async fn download_fonts(&self) -> fpm::Result<()> {
        use itertools::Itertools;

        let mut fonts = vec![];
        for dep in self
            .package
            .get_flattened_dependencies()
            .into_iter()
            .unique_by(|dep| dep.package.name.clone())
        {
            fonts.extend(dep.package.fonts);
        }

        for package in self.all_packages.borrow().values() {
            fonts.extend(package.fonts.clone());
        }

        for font in fonts.iter() {
            if let Some(url) = font.get_url() {
                let start = std::time::Instant::now();
                print!("Processing {} ... ", url);
                let content = self.get_file_and_resolve(url.as_str()).await?.1;
                fpm::utils::update(&self.build_dir().join(&url), content.as_slice()).await?;
                fpm::utils::print_end(format!("Processed {}", url).as_str(), start);
            }
        }

        Ok(())
    }

    /// `attach_data_string()` sets the value of extra data in fpm::Config,
    /// provided as `data` paramater of type `&str`
    pub fn attach_data_string(&mut self, data: &str) -> fpm::Result<()> {
        self.attach_data(serde_json::from_str(data)?)
    }

    /// `attach_data()` sets the value of extra data in fpm::Config,
    /// provided as `data` paramater of type `serde_json::Value`
    pub fn attach_data(&mut self, data: serde_json::Value) -> fpm::Result<()> {
        let data = match data {
            serde_json::Value::Object(o) => o,
            t => {
                return Err(fpm::Error::UsageError {
                    message: format!("Expected object type, found: `{:?}`", t),
                })
            }
        };
        self.extra_data = data;
        Ok(())
    }

    /// update the config.global_ids map from the contents of a file
    /// in case the user defines the id for any component in the document
    pub async fn update_global_ids_from_file(
        &mut self,
        doc_id: &str,
        data: &str,
    ) -> fpm::Result<()> {
        /// updates the config.global_ids map
        ///
        /// mapping from [id -> link]
        ///
        /// link: <document-id>#<slugified-id>
        fn update_id_map(
            global_ids: &mut std::collections::HashMap<String, String>,
            id_string: &str,
            doc_name: &str,
            line_number: usize,
        ) -> fpm::Result<()> {
            // returns doc-id from link as String
            fn fetch_doc_id_from_link(link: &str) -> fpm::Result<String> {
                // link = <document-id>#<slugified-id>
                let doc_id = link.split_once('#').map(|s| s.0);
                match doc_id {
                    Some(id) => Ok(id.to_string()),
                    None => Err(fpm::Error::PackageError {
                        message: format!("Invalid link format {}", link),
                    }),
                }
            }

            let (_header, value) = ftd::p2::utils::split_once(id_string, doc_name, line_number)?;
            let document_id = fpm::library::convert_to_document_id(doc_name);

            if let Some(id) = value {
                // check if the current id already exists in the map
                // if it exists then throw error
                if global_ids.contains_key(&id) {
                    return Err(fpm::Error::UsageError {
                        message: format!(
                            "conflicting id: \'{}\' used in doc: \'{}\' and doc: \'{}\'",
                            id,
                            fetch_doc_id_from_link(&global_ids[&id])?,
                            document_id
                        ),
                    });
                }

                // mapping id -> <document-id>#<slugified-id>
                let link = format!("{}#{}", document_id, slug::slugify(&id));
                global_ids.insert(id, link);
            }

            Ok(())
        }

        // Vec<captured_id, line_number>
        let captured_global_ids: Vec<(String, usize)> = ftd::p1::parse_file_for_global_ids(data);
        for (captured_id, ln) in captured_global_ids.iter() {
            update_id_map(&mut self.global_ids, captured_id.as_str(), doc_id, *ln)?;
        }

        Ok(())
    }

    pub(crate) async fn get_versions(
        &self,
        package: &fpm::Package,
    ) -> fpm::Result<std::collections::HashMap<fpm::Version, Vec<fpm::File>>> {
        let path = self.get_root_for_package(package);
        let mut hash: std::collections::HashMap<fpm::Version, Vec<fpm::File>> =
            std::collections::HashMap::new();

        let all_files = self.get_all_file_paths1(package, true)?;

        for file in all_files {
            if file.is_dir() {
                continue;
            }
            let version = get_version(&file, &path).await?;
            let file = fpm::get_file(
                package.name.to_string(),
                &file,
                &(if version.original.eq("BASE_VERSION") {
                    path.to_owned()
                } else {
                    path.join(&version.original)
                }),
            )
            .await?;
            if let Some(files) = hash.get_mut(&version) {
                files.push(file)
            } else {
                hash.insert(version, vec![file]);
            }
        }
        return Ok(hash);

        async fn get_version(
            x: &camino::Utf8PathBuf,
            path: &camino::Utf8PathBuf,
        ) -> fpm::Result<fpm::Version> {
            let id = match tokio::fs::canonicalize(x)
                .await?
                .to_str()
                .unwrap()
                .rsplit_once(
                    if path.as_str().ends_with(std::path::MAIN_SEPARATOR) {
                        path.as_str().to_string()
                    } else {
                        format!("{}{}", path, std::path::MAIN_SEPARATOR)
                    }
                    .as_str(),
                ) {
                Some((_, id)) => id.to_string(),
                None => {
                    return Err(fpm::Error::UsageError {
                        message: format!("{:?} should be a file", x),
                    });
                }
            };
            if let Some((v, _)) = id.split_once('/') {
                fpm::Version::parse(v)
            } else {
                Ok(fpm::Version::base())
            }
        }
    }

    pub(crate) fn get_root_for_package(&self, package: &fpm::Package) -> camino::Utf8PathBuf {
        if let Some(package_fpm_path) = &package.fpm_path {
            // TODO: Unwrap?
            package_fpm_path.parent().unwrap().to_owned()
        } else if package.name.eq(&self.package.name) {
            self.root.clone()
        } else {
            self.packages_root.clone().join(package.name.as_str())
        }
    }

    pub(crate) async fn get_files(&self, package: &fpm::Package) -> fpm::Result<Vec<fpm::File>> {
        let path = self.get_root_for_package(package);
        let all_files = self.get_all_file_paths1(package, true)?;
        // TODO: Unwrap?
        let mut documents = fpm::paths_to_files(package.name.as_str(), all_files, &path).await?;
        documents.sort_by_key(|v| v.get_id());

        Ok(documents)
    }

    /// updates the terms map from the files of the current package
    async fn update_ids_from_package(&mut self) -> fpm::Result<()> {
        let path = self.get_root_for_package(&self.package);
        let all_files_path = self.get_all_file_paths1(&self.package, true)?;

        let documents =
            fpm::paths_to_files(self.package.name.as_str(), all_files_path, &path).await?;
        for document in documents.iter() {
            if let fpm::File::Ftd(doc) = document {
                // Ignore fetching id's from FPM.ftd since
                // id's would be used to link inside sitemap
                if doc.id.eq("FPM.ftd") {
                    continue;
                }
                self.update_global_ids_from_file(&doc.id, &doc.content)
                    .await?;
            }
        }
        Ok(())
    }

    pub(crate) fn get_all_file_paths1(
        &self,
        package: &fpm::Package,
        ignore_history: bool,
    ) -> fpm::Result<Vec<camino::Utf8PathBuf>> {
        let path = self.get_root_for_package(package);
        let mut ignore_paths = ignore::WalkBuilder::new(&path);
        // ignore_paths.hidden(false); // Allow the linux hidden files to be evaluated
        ignore_paths.overrides(fpm::file::package_ignores(package, &path, ignore_history)?);
        Ok(ignore_paths
            .build()
            .into_iter()
            .flatten()
            .map(|x| camino::Utf8PathBuf::from_path_buf(x.into_path()).unwrap()) //todo: improve error message
            .collect::<Vec<camino::Utf8PathBuf>>())
    }

    pub(crate) fn get_all_file_path(
        &self,
        package: &fpm::Package,
        ignore_paths: Vec<String>,
    ) -> fpm::Result<Vec<camino::Utf8PathBuf>> {
        let path = self.get_root_for_package(package);
        let mut ignore_paths_build = ignore::WalkBuilder::new(&path);
        ignore_paths_build.hidden(false);
        ignore_paths_build.overrides(fpm::file::ignore_path(package, &path, ignore_paths)?);
        Ok(ignore_paths_build
            .build()
            .into_iter()
            .flatten()
            .map(|x| camino::Utf8PathBuf::from_path_buf(x.into_path()).unwrap()) //todo: improve error message
            .collect::<Vec<camino::Utf8PathBuf>>())
    }

    pub async fn get_file_by_id(&self, id: &str, package: &fpm::Package) -> fpm::Result<fpm::File> {
        let file_name = fpm::Config::get_file_name(&self.root, id)?;
        self.get_files(package)
            .await?
            .into_iter()
            .find(|v| v.get_id().eq(file_name.as_str()))
            .ok_or_else(|| fpm::Error::UsageError {
                message: format!("No such file found: {}", id),
            })
    }

    pub(crate) async fn get_file_and_package_by_cr_id(
        &mut self,
        id: &str,
        cr_number: usize,
    ) -> fpm::Result<fpm::File> {
        let file_name = self.get_cr_file_and_resolve(id, cr_number).await?.0;
        let id_without_cr_prefix = fpm::cr::get_id_from_cr_id(id, cr_number)?;
        let package = self
            .find_package_by_id(id_without_cr_prefix.as_str())
            .await?
            .1;

        let mut file = fpm::get_file(
            package.name.to_string(),
            &self.root.join(file_name),
            &self.get_root_for_package(&package),
        )
        .await?;

        if id_without_cr_prefix.contains("-/") && !id_without_cr_prefix.contains("-/about") {
            let url = id_without_cr_prefix
                .trim_end_matches("/index.html")
                .trim_matches('/');
            let extension = if matches!(file, fpm::File::Markdown(_)) {
                "/index.md".to_string()
            } else if matches!(file, fpm::File::Ftd(_)) {
                "/index.ftd".to_string()
            } else {
                "".to_string()
            };
            file.set_id(format!("{}{}", url, extension).as_str());
        }
        Ok(file)
    }

    // Input
    // path: /todos/add-todo/
    // mount-point: /todos/
    // Output
    // -/<todos-package-name>/add-todo/, <todos-package-name>, /add-todo/
    pub fn get_mountpoint_sanitized_path<'a>(
        &'a self,
        package: &'a fpm::Package,
        path: &'a str,
    ) -> Option<(
        String,
        &'a fpm::Package,
        String,
        Option<&fpm::package::app::App>,
    )> {
        // Problem for recursive dependency is that only current package contains dependency,
        // dependent package does not contain dependency

        // For similar package
        if path.starts_with(format!("-/{}", package.name.trim_matches('/')).as_str()) {
            let path_without_package_name =
                path.trim_start_matches(format!("-/{}", package.name.trim_matches('/')).as_str());
            return Some((
                path.to_string(),
                package,
                path_without_package_name.to_string(),
                None,
            ));
        }

        for (mp, dep, app) in package
            .apps
            .iter()
            .map(|x| (&x.mount_point, &x.package.package, x))
        {
            if path.starts_with(mp.trim_matches('/')) {
                // TODO: Need to handle for recursive dependencies mount-point
                // Note: Currently not working because dependency of package does not contain dependencies
                let package_name = dep.name.trim_matches('/');
                let sanitized_path = path.trim_start_matches(mp.trim_start_matches('/'));
                return Some((
                    format!("-/{package_name}/{sanitized_path}"),
                    dep,
                    sanitized_path.to_string(),
                    Some(app),
                ));
            } else if path.starts_with(format!("-/{}", dep.name.trim_matches('/')).as_str()) {
                let path_without_package_name =
                    path.trim_start_matches(format!("-/{}", dep.name.trim_matches('/')).as_str());
                return Some((
                    path.to_string(),
                    dep,
                    path_without_package_name.to_string(),
                    Some(app),
                ));
            }
        }
        None
    }

    pub async fn update_sitemap(&self, package: &fpm::Package) -> fpm::Result<fpm::Package> {
        let fpm_path = &self.packages_root.join(&package.name).join("FPM.ftd");

        if !fpm_path.exists() {
            let package = self.resolve_package(package).await?;
            self.add_package(&package);
        }

        dbg!(&fpm_path);

        let fpm_doc = utils::fpm_doc(fpm_path).await?;

        let mut package = package.clone();

        package.sitemap_temp = fpm_doc.get("fpm#sitemap")?;
        package.dynamic_urls_temp = fpm_doc.get("fpm#dynamic-urls")?;

        package.sitemap = match package.sitemap_temp.as_ref() {
            Some(sitemap_temp) => {
                let mut s = fpm::sitemap::Sitemap::parse(
                    sitemap_temp.body.as_str(),
                    &package,
                    &mut self.clone(), //TODO: totally wrong
                    false,
                )
                .await?;
                s.readers = sitemap_temp.readers.clone();
                s.writers = sitemap_temp.writers.clone();
                Some(s)
            }
            None => None,
        };

        // Handling of `-- fpm.dynamic-urls:`
        package.dynamic_urls = {
            match &package.dynamic_urls_temp {
                Some(urls_temp) => Some(fpm::sitemap::DynamicUrls::parse(
                    &self.global_ids,
                    &package.name,
                    urls_temp.body.as_str(),
                )?),
                None => None,
            }
        };
        Ok(package)
    }

    // -/kameri-app.herokuapp.com/
    // .packages/kameri-app.heroku.com/index.ftd
    pub async fn get_file_and_package_by_id(&mut self, path: &str) -> fpm::Result<fpm::File> {
        // This function will return file and package by given path
        // path can be mounted(mount-point) with other dependencies
        //
        // Sanitize the mountpoint request.
        // Get the package and sanitized path
        dbg!(&path);
        let package1;
        let (path_with_package_name, sanitized_package, sanitized_path) =
            match self.get_mountpoint_sanitized_path(&self.package, path) {
                Some((new_path, package, remaining_path, _)) => {
                    // Update the sitemap of the package, if it does ot contain the sitemap information
                    dbg!(&new_path, &package.name, &remaining_path);
                    if package.name != self.package.name {
                        package1 = self.update_sitemap(package).await?;
                        (new_path, &package1, remaining_path)
                    } else {
                        (new_path, package, remaining_path)
                    }
                }
                None => (path.to_string(), &self.package, path.to_string()),
            };

        dbg!(
            &path_with_package_name,
            &sanitized_package.name,
            &sanitized_path
        );

        // Getting `document` with dynamic parameters, if exists
        let (document, path_params) = match sanitized_package.sitemap.as_ref() {
            //1. First resolve document in sitemap
            Some(sitemap) => match sitemap.resolve_document(sanitized_path.as_str()) {
                Some(document) => (Some(document), vec![]),
                //2.  Else resolve document in dynamic urls
                None => match sanitized_package.dynamic_urls.as_ref() {
                    Some(dynamic_urls) => dynamic_urls.resolve_document(sanitized_path.as_str())?,
                    None => (None, vec![]),
                },
            },
            None => (None, vec![]),
        };

        // document with package-name prefix
        let document = document.map(|doc| {
            format!(
                "-/{}/{}",
                sanitized_package.name.trim_matches('/'),
                doc.trim_matches('/')
            )
        });

        dbg!(&document);

        let path = path_with_package_name.as_str();
        if let Some(id) = document {
            let file_name = self.get_file_path_and_resolve(id.as_str()).await?;
            let package = self.find_package_by_id(id.as_str()).await?.1;
            let file = fpm::get_file(
                package.name.to_string(),
                &self.root.join(file_name),
                &self.get_root_for_package(&package),
            )
            .await?;
            self.current_document = Some(path.to_string());
            self.path_parameters = path_params;
            Ok(file)
        } else {
            // -/fifthtry.github.io/todos/add-todo/
            // -/fifthtry.github.io/doc-site/add-todo/
            dbg!(path);
            let file_name = self.get_file_path_and_resolve(path).await?;
            // .packages/todos/add-todo.ftd
            // .packages/fifthtry.github.io/doc-site/add-todo.ftd
            dbg!(&file_name);

            let package = self.find_package_by_id(path).await?.1;
            let mut file = fpm::get_file(
                package.name.to_string(),
                &self.root.join(file_name),
                &self.get_root_for_package(&package),
            )
            .await?;

            if path.contains("-/") {
                let url = path.trim_end_matches("/index.html").trim_matches('/');
                let extension = if matches!(file, fpm::File::Markdown(_)) {
                    "/index.md".to_string()
                } else if matches!(file, fpm::File::Ftd(_)) {
                    "/index.ftd".to_string()
                } else {
                    "".to_string()
                };
                file.set_id(format!("{}{}", url, extension).as_str());
            }
            dbg!(&file.get_id());
            self.current_document = Some(file.get_id());
            Ok(file)
        }
    }

    pub fn doc_id(&self) -> Option<String> {
        self.current_document
            .clone()
            .map(|v| fpm::utils::id_to_path(v.as_str()))
            .map(|v| v.trim().replace(std::path::MAIN_SEPARATOR, "/"))
    }

    pub async fn get_file_path(&self, id: &str) -> fpm::Result<String> {
        let (package_name, package) = self.find_package_by_id(id).await?;
        let mut id = id.to_string();
        let mut add_packages = "".to_string();
        if let Some(new_id) = id.strip_prefix("-/") {
            // Check if the id is alias for index.ftd. eg: `/-/bar/`
            if new_id.starts_with(&package_name) || !package.name.eq(self.package.name.as_str()) {
                id = new_id.to_string();
            }
            if !package.name.eq(self.package.name.as_str()) {
                add_packages = format!(".packages/{}/", package.name);
            }
        }
        let id = {
            let mut id = id
                .split_once("-/")
                .map(|(id, _)| id)
                .unwrap_or_else(|| id.as_str())
                .trim()
                .trim_start_matches(package_name.as_str());
            if id.is_empty() {
                id = "/";
            }
            id
        };
        Ok(format!(
            "{}{}",
            add_packages,
            package.resolve_by_id(id, None).await?.0
        ))
    }

    pub(crate) async fn get_file_path_and_resolve(&self, id: &str) -> fpm::Result<String> {
        Ok(self.get_file_and_resolve(id).await?.0)
    }

    pub(crate) async fn get_file_and_resolve(&self, id: &str) -> fpm::Result<(String, Vec<u8>)> {
        let (package_name, package) = self.find_package_by_id(id).await?;
        let package = self.resolve_package(&package).await?;
        self.add_package(&package);
        let mut id = id.to_string();
        let mut add_packages = "".to_string();
        if let Some(new_id) = id.strip_prefix("-/") {
            // Check if the id is alias for index.ftd. eg: `/-/bar/`
            if new_id.starts_with(&package_name) || !package.name.eq(self.package.name.as_str()) {
                id = new_id.to_string();
            }
            if !package.name.eq(self.package.name.as_str()) {
                add_packages = format!(".packages/{}/", package.name);
            }
        }
        let id = {
            let mut id = id
                .split_once("-/")
                .map(|(id, _)| id)
                .unwrap_or_else(|| id.as_str())
                .trim()
                .trim_start_matches(package_name.as_str());
            if id.is_empty() {
                id = "/";
            }
            id
        };

        let (file_name, content) = package.resolve_by_id(id, None).await?;

        Ok((format!("{}{}", add_packages, file_name), content))
    }

    pub(crate) async fn get_cr_file_and_resolve(
        &self,
        cr_id: &str,
        cr_number: usize,
    ) -> fpm::Result<(String, Vec<u8>)> {
        let id_without_cr_prefix = fpm::cr::get_id_from_cr_id(cr_id, cr_number)?;
        let (package_name, package) = self
            .find_package_by_id(id_without_cr_prefix.as_str())
            .await?;
        let package = self.resolve_package(&package).await?;
        self.add_package(&package);
        let mut new_id = id_without_cr_prefix.to_string();
        let mut add_packages = "".to_string();
        if let Some(id) = new_id.strip_prefix("-/") {
            // Check if the id is alias for index.ftd. eg: `/-/bar/`
            if id.starts_with(&package_name) || !package.name.eq(self.package.name.as_str()) {
                new_id = id.to_string();
            }
            if !package.name.eq(self.package.name.as_str()) {
                add_packages = format!(".packages/{}/", package.name);
            }
        }
        let id = {
            let mut id = match new_id.split_once("-/") {
                Some((p1, p2))
                    if !(package_name.eq(self.package.name.as_str())
                        && fpm::utils::ids_matches(p2, "about")) =>
                // full id in case of about page as it's a special page
                {
                    p1.to_string()
                }
                _ => new_id,
            }
            .trim()
            .trim_start_matches(package_name.as_str())
            .to_string();
            if id.is_empty() {
                id = "/".to_string();
            }
            id
        };

        if package.name.eq(self.package.name.as_str()) {
            let file_info_map = fpm::cr::cr_clone_file_info(self, cr_number).await?;
            let file_info = fpm::package::package_doc::file_id_to_names(id.as_str())
                .into_iter()
                .find_map(|id| file_info_map.get(&id))
                .ok_or_else(|| fpm::Error::UsageError {
                    message: format!("{} is not found", cr_id),
                })?;

            return Ok((
                format!("{}{}", add_packages, file_info.path),
                file_info.content.to_owned(),
            ));
        }

        let (file_name, content) = package.resolve_by_id(id.as_str(), None).await?;

        Ok((format!("{}{}", add_packages, file_name), content))
    }

    /// Return (package name or alias, package)
    pub(crate) async fn find_package_by_id(&self, id: &str) -> fpm::Result<(String, fpm::Package)> {
        let sanitized_id = self
            .get_mountpoint_sanitized_path(&self.package, id)
            .map(|(x, _, _, _)| x)
            .unwrap_or_else(|| id.to_string());

        let id = sanitized_id.as_str();
        let id = if let Some(id) = id.strip_prefix("-/") {
            id
        } else {
            return Ok((self.package.name.to_string(), self.package.to_owned()));
        };

        if let Some(package) = self.package.aliases().iter().find_map(|(alias, d)| {
            if id.starts_with(alias) {
                Some((alias.to_string(), (*d).to_owned()))
            } else {
                None
            }
        }) {
            return Ok(package);
        }

        for (package_name, package) in self.all_packages.borrow().iter() {
            if id.starts_with(package_name) {
                return Ok((package_name.to_string(), package.to_owned()));
            }
        }

        if let Some(package_root) =
            utils::find_root_for_file(&self.packages_root.join(id), "FPM.ftd")
        {
            let mut package = fpm::Package::new("unknown-package");
            package.resolve(&package_root.join("FPM.ftd")).await?;
            self.add_package(&package);
            return Ok((package.name.to_string(), package));
        }

        Ok((self.package.name.to_string(), self.package.to_owned()))
    }

    pub(crate) async fn download_required_file(
        root: &camino::Utf8PathBuf,
        id: &str,
        package: &fpm::Package,
    ) -> fpm::Result<String> {
        use tokio::io::AsyncWriteExt;

        let id = id.trim_start_matches(package.name.as_str());

        let base = package
            .download_base_url
            .clone()
            .ok_or_else(|| fpm::Error::PackageError {
                message: "package base not found".to_string(),
            })?;

        if id.eq("/") {
            if let Ok(string) = crate::http::http_get_str(
                format!("{}/index.ftd", base.trim_end_matches('/')).as_str(),
            )
            .await
            {
                let base = root.join(".packages").join(package.name.as_str());
                tokio::fs::create_dir_all(&base).await?;
                tokio::fs::File::create(base.join("index.ftd"))
                    .await?
                    .write_all(string.as_bytes())
                    .await?;
                return Ok(format!(".packages/{}/index.ftd", package.name));
            }
            if let Ok(string) = crate::http::http_get_str(
                format!("{}/README.md", base.trim_end_matches('/')).as_str(),
            )
            .await
            {
                let base = root.join(".packages").join(package.name.as_str());
                tokio::fs::create_dir_all(&base).await?;
                tokio::fs::File::create(base.join("README.md"))
                    .await?
                    .write_all(string.as_bytes())
                    .await?;
                return Ok(format!(".packages/{}/README.md", package.name));
            }
            return Err(fpm::Error::UsageError {
                message: "File not found".to_string(),
            });
        }

        let id = id.trim_matches('/').to_string();
        if let Ok(string) =
            crate::http::http_get_str(format!("{}/{}.ftd", base.trim_end_matches('/'), id).as_str())
                .await
        {
            let (prefix, id) = match id.rsplit_once('/') {
                Some((prefix, id)) => (format!("/{}", prefix), id.to_string()),
                None => ("".to_string(), id),
            };
            let base = root
                .join(".packages")
                .join(format!("{}{}", package.name.as_str(), prefix));
            tokio::fs::create_dir_all(&base).await?;
            let file_path = base.join(format!("{}.ftd", id));
            tokio::fs::File::create(&file_path)
                .await?
                .write_all(string.as_bytes())
                .await?;
            return Ok(file_path.to_string());
        }
        if let Ok(string) = crate::http::http_get_str(
            format!("{}/{}/index.ftd", base.trim_end_matches('/'), id).as_str(),
        )
        .await
        {
            let base = root.join(".packages").join(package.name.as_str()).join(id);
            tokio::fs::create_dir_all(&base).await?;
            let file_path = base.join("index.ftd");
            tokio::fs::File::create(&file_path)
                .await?
                .write_all(string.as_bytes())
                .await?;
            return Ok(file_path.to_string());
        }
        if let Ok(string) =
            crate::http::http_get_str(format!("{}/{}.md", base.trim_end_matches('/'), id).as_str())
                .await
        {
            let base = root.join(".packages").join(package.name.as_str());
            tokio::fs::create_dir_all(&base).await?;
            tokio::fs::File::create(base.join(format!("{}.md", id)))
                .await?
                .write_all(string.as_bytes())
                .await?;
            return Ok(format!(".packages/{}/{}.md", package.name, id));
        }
        if let Ok(string) = crate::http::http_get_str(
            format!("{}/{}/README.md", base.trim_end_matches('/'), id).as_str(),
        )
        .await
        {
            let base = root.join(".packages").join(package.name.as_str());
            tokio::fs::create_dir_all(&base).await?;
            tokio::fs::File::create(base.join(format!("{}/README.md", id)))
                .await?
                .write_all(string.as_bytes())
                .await?;
            return Ok(format!(".packages/{}/{}/README.md", package.name, id));
        }
        Err(fpm::Error::UsageError {
            message: "File not found".to_string(),
        })
    }

    pub(crate) fn get_file_name(root: &camino::Utf8PathBuf, id: &str) -> fpm::Result<String> {
        let mut id = id.to_string();
        let mut add_packages = "".to_string();
        if let Some(new_id) = id.strip_prefix("-/") {
            id = new_id.to_string();
            add_packages = ".packages/".to_string()
        }
        let mut id = id
            .split_once("-/")
            .map(|(id, _)| id)
            .unwrap_or_else(|| id.as_str())
            .trim()
            .replace("/index.html", "/")
            .replace("index.html", "/");
        if id.eq("/") {
            if root.join(format!("{}index.ftd", add_packages)).exists() {
                return Ok(format!("{}index.ftd", add_packages));
            }
            if root.join(format!("{}README.md", add_packages)).exists() {
                return Ok(format!("{}README.md", add_packages));
            }
            return Err(fpm::Error::UsageError {
                message: "File not found".to_string(),
            });
        }
        id = id.trim_matches('/').to_string();
        if root.join(format!("{}{}.ftd", add_packages, id)).exists() {
            return Ok(format!("{}{}.ftd", add_packages, id));
        }
        if root
            .join(format!("{}{}/index.ftd", add_packages, id))
            .exists()
        {
            return Ok(format!("{}{}/index.ftd", add_packages, id));
        }
        if root.join(format!("{}{}.md", add_packages, id)).exists() {
            return Ok(format!("{}{}.md", add_packages, id));
        }
        if root
            .join(format!("{}{}/README.md", add_packages, id))
            .exists()
        {
            return Ok(format!("{}{}/README.md", add_packages, id));
        }
        Err(fpm::Error::UsageError {
            message: "File not found".to_string(),
        })
    }

    pub(crate) async fn get_assets(
        &self,
    ) -> fpm::Result<std::collections::HashMap<String, String>> {
        use itertools::Itertools;

        let mut asset_documents = std::collections::HashMap::new();
        asset_documents.insert(
            self.package.name.clone(),
            self.package.get_assets_doc().await?,
        );

        let dependencies = if let Some(package) = self.package.translation_of.as_ref() {
            let mut deps = package
                .get_flattened_dependencies()
                .into_iter()
                .unique_by(|dep| dep.package.name.clone())
                .collect_vec();
            deps.extend(
                self.package
                    .get_flattened_dependencies()
                    .into_iter()
                    .unique_by(|dep| dep.package.name.clone()),
            );
            deps
        } else {
            self.package
                .get_flattened_dependencies()
                .into_iter()
                .unique_by(|dep| dep.package.name.clone())
                .collect_vec()
        };
        for dep in &dependencies {
            asset_documents.insert(
                dep.package.name.clone(),
                dep.package.get_assets_doc().await?,
            );
        }
        Ok(asset_documents)
    }

    async fn get_root_path(directory: &camino::Utf8PathBuf) -> fpm::Result<camino::Utf8PathBuf> {
        if let Some(fpm_ftd_root) = utils::find_root_for_file(directory, "FPM.ftd") {
            return Ok(fpm_ftd_root);
        }
        let fpm_manifest_path = match utils::find_root_for_file(directory, "FPM.manifest.ftd") {
            Some(fpm_manifest_path) => fpm_manifest_path,
            None => {
                return Err(fpm::Error::UsageError {
                    message: "FPM.ftd or FPM.manifest.ftd not found in any parent directory"
                        .to_string(),
                });
            }
        };

        let doc = tokio::fs::read_to_string(fpm_manifest_path.join("FPM.manifest.ftd"));
        let lib = fpm::FPMLibrary::default();
        let fpm_manifest_processed =
            match fpm::doc::parse_ftd("FPM.manifest", doc.await?.as_str(), &lib) {
                Ok(fpm_manifest_processed) => fpm_manifest_processed,
                Err(e) => {
                    return Err(fpm::Error::PackageError {
                        message: format!("failed to parse FPM.manifest.ftd: {:?}", &e),
                    });
                }
            };

        let new_package_root = fpm_manifest_processed
            .get::<String>("FPM.manifest#package-root")?
            .as_str()
            .split('/')
            .fold(fpm_manifest_path, |accumulator, part| {
                accumulator.join(part)
            });

        if new_package_root.join("FPM.ftd").exists() {
            Ok(new_package_root)
        } else {
            Err(fpm::Error::PackageError {
                message: "Can't find FPM.ftd. The path specified in FPM.manifest.ftd doesn't contain the FPM.ftd file".to_string(),
            })
        }
    }

    /// `read()` is the way to read a Config.
    pub async fn read(
        root: Option<String>,
        resolve_sitemap: bool,
        req: Option<&fpm::http::Request>,
    ) -> fpm::Result<fpm::Config> {
        let (root, original_directory) = match root {
            Some(r) => {
                let root: camino::Utf8PathBuf = tokio::fs::canonicalize(r.as_str())
                    .await?
                    .to_str()
                    .map_or_else(|| r, |r| r.to_string())
                    .into();
                (root.clone(), root)
            }
            None => {
                let original_directory: camino::Utf8PathBuf =
                    tokio::fs::canonicalize(std::env::current_dir()?)
                        .await?
                        .try_into()?;
                (
                    fpm::Config::get_root_path(&original_directory).await?,
                    original_directory,
                )
            }
        };
        let fpm_doc = utils::fpm_doc(&root.join("FPM.ftd")).await?;
        let package = fpm::Package::from_fpm_doc(&root, &fpm_doc)?;
        let mut config = Config {
            package: package.clone(),
            packages_root: root.clone().join(".packages"),
            root,
            original_directory,
            extra_data: Default::default(),
            current_document: None,
            all_packages: Default::default(),
            downloaded_assets: Default::default(),
            global_ids: Default::default(),
            request: req.map(ToOwned::to_owned),
            path_parameters: vec![],
        };

        // Update global_ids map from the current package files
        config.update_ids_from_package().await?;

        // TODO: Major refactor, while parsing sitemap of a package why do we need config in it?
        config.package.sitemap = {
            let sitemap = match package.translation_of.as_ref() {
                Some(translation) => translation,
                None => &package,
            }
            .sitemap_temp
            .as_ref();

            match sitemap {
                Some(sitemap_temp) => {
                    let mut s = fpm::sitemap::Sitemap::parse(
                        sitemap_temp.body.as_str(),
                        &package,
                        &mut config,
                        resolve_sitemap,
                    )
                    .await?;
                    s.readers = sitemap_temp.readers.clone();
                    s.writers = sitemap_temp.writers.clone();
                    Some(s)
                }
                None => None,
            }
        };

        // Handling of `-- fpm.dynamic-urls:`
        config.package.dynamic_urls = {
            match &package.dynamic_urls_temp {
                Some(urls_temp) => Some(fpm::sitemap::DynamicUrls::parse(
                    &config.global_ids,
                    &package.name,
                    urls_temp.body.as_str(),
                )?),
                None => None,
            }
        };

        config.add_package(&package);

        Ok(config)
    }

    pub fn set_request(mut self, req: fpm::http::Request) -> Self {
        self.request = Some(req);
        self
    }

    pub(crate) async fn resolve_package(
        &self,
        package: &fpm::Package,
    ) -> fpm::Result<fpm::Package> {
        if self.package.name.eq(package.name.as_str()) {
            return Ok(self.package.clone());
        }

        if let Some(package) = { self.all_packages.borrow().get(package.name.as_str()) } {
            return Ok(package.clone());
        }

        let package = package
            .get_and_resolve(&self.get_root_for_package(package))
            .await?;

        self.add_package(&package);
        Ok(package)
    }

    pub(crate) fn add_package(&self, package: &fpm::Package) {
        self.all_packages
            .borrow_mut()
            .insert(package.name.to_string(), package.to_owned());
    }

    #[allow(dead_code)]
    pub(crate) fn get_fpm_document(&self, package_name: &str) -> fpm::Result<ftd::p2::Document> {
        let package = fpm::Package::new(package_name);
        let root = self.get_root_for_package(&package);
        let package_fpm_path = root.join("FPM.ftd");
        let doc = std::fs::read_to_string(package_fpm_path)?;
        let lib = fpm::FPMLibrary::default();
        Ok(fpm::doc::parse_ftd("FPM", doc.as_str(), &lib)?)
    }

    pub(crate) async fn get_reserved_crs(
        &self,
        number_of_crs_to_reserve: Option<usize>,
    ) -> fpm::Result<Vec<i32>> {
        let number_of_crs_to_reserve =
            if let Some(number_of_crs_to_reserve) = number_of_crs_to_reserve {
                number_of_crs_to_reserve
            } else {
                fpm::NUMBER_OF_CRS_TO_RESERVE
            };
        if !cfg!(feature = "remote") {
            return fpm::usage_error("Can be used by remote only".to_string());
        }
        let value = fpm::cache::update(
            self.remote_cr().to_string().as_str(),
            number_of_crs_to_reserve,
        )
        .await? as i32;

        Ok(Vec::from_iter(
            (value - (number_of_crs_to_reserve as i32))..value,
        ))
    }

    pub(crate) async fn can_read(
        &self,
        req: &fpm::http::Request,
        document_path: &str,
    ) -> fpm::Result<bool> {
        use itertools::Itertools;
        let document_name = self.document_name_with_default(document_path);
        if let Some(sitemap) = &self.package.sitemap {
            // TODO: This can be buggy in case of: if groups are used directly in sitemap are foreign groups
            let document_readers = sitemap.readers(document_name.as_str(), &self.package.groups);
            dbg!(&document_readers);
            if document_readers.is_empty() {
                return Ok(true);
            }
            let access_identities =
                fpm::user_group::access_identities(self, req, &document_name, true).await?;
            return fpm::user_group::belongs_to(
                self,
                document_readers.as_slice(),
                access_identities.iter().collect_vec().as_slice(),
            );
        }
        Ok(true)
    }

    pub(crate) async fn can_write(
        &self,
        req: &fpm::http::Request,
        document_path: &str,
    ) -> fpm::Result<bool> {
        use itertools::Itertools;
        let document_name = self.document_name_with_default(document_path);
        if let Some(sitemap) = &self.package.sitemap {
            // TODO: This can be buggy in case of: if groups are used directly in sitemap are foreign groups
            let document_writers = sitemap.writers(document_name.as_str(), &self.package.groups);
            let access_identities =
                fpm::user_group::access_identities(self, req, &document_name, false).await?;

            return fpm::user_group::belongs_to(
                self,
                document_writers.as_slice(),
                access_identities.iter().collect_vec().as_slice(),
            );
        }

        Ok(false)
    }
}
