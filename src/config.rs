// Document: https://fpm.dev/crate/config/
// Document: https://fpm.dev/crate/package/
use std::convert::TryInto;
use std::iter::FromIterator;

#[derive(Debug, Clone)]
pub struct Config {
    pub package: Package,
    pub root: camino::Utf8PathBuf,
    pub packages_root: camino::Utf8PathBuf,
    pub original_directory: camino::Utf8PathBuf,
    pub extra_data: serde_json::Map<String, serde_json::Value>,
    pub current_document: Option<String>,
    pub all_packages: std::cell::RefCell<std::collections::BTreeMap<String, Package>>,
    pub downloaded_assets: std::collections::BTreeMap<String, String>,
    pub global_ids: std::collections::HashMap<String, String>,
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

            let (_header, value) =
                ftd::identifier::segregate_key_value(id_string, doc_name, line_number)?;
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

    pub async fn get_file_and_package_by_id(&mut self, id: &str) -> fpm::Result<fpm::File> {
        let file_name = self.get_file_path_and_resolve(id).await?;
        let package = self.find_package_by_id(id).await?.1;
        let mut file = fpm::get_file(
            package.name.to_string(),
            &self.root.join(file_name),
            &self.get_root_for_package(&package),
        )
        .await?;
        if id.contains("-/") {
            let url = id.trim_end_matches("/index.html").trim_matches('/');
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
            let file_info = fpm::package_doc::file_id_to_names(id.as_str())
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

        if let Some(package_root) = find_root_for_file(&self.packages_root.join(id), "FPM.ftd") {
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
        if let Some(fpm_ftd_root) = find_root_for_file(directory, "FPM.ftd") {
            return Ok(fpm_ftd_root);
        }
        let fpm_manifest_path = match find_root_for_file(directory, "FPM.manifest.ftd") {
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
    pub async fn read(root: Option<String>, resolve_sitemap: bool) -> fpm::Result<fpm::Config> {
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

        let fpm_doc = {
            let doc = tokio::fs::read_to_string(root.join("FPM.ftd"));
            let lib = fpm::FPMLibrary::default();
            match fpm::doc::parse_ftd("FPM", doc.await?.as_str(), &lib) {
                Ok(v) => v,
                Err(e) => {
                    return Err(fpm::Error::PackageError {
                        message: format!("failed to parse FPM.ftd 3: {:?}", &e),
                    });
                }
            }
        };

        let mut deps = {
            let temp_deps: Vec<fpm::dependency::DependencyTemp> = fpm_doc.get("fpm#dependency")?;
            temp_deps
                .into_iter()
                .map(|v| v.into_dependency())
                .collect::<Vec<fpm::Result<fpm::Dependency>>>()
                .into_iter()
                .collect::<fpm::Result<Vec<fpm::Dependency>>>()?
        };

        let mut package = {
            let temp_package: Option<PackageTemp> = fpm_doc.get("fpm#package")?;
            let mut package = match temp_package {
                Some(v) => v.into_package(),
                None => {
                    return Err(fpm::Error::PackageError {
                        message: "FPM.ftd does not contain package definition".to_string(),
                    })
                }
            };

            if package.name != fpm::FPM_UI_INTERFACE
                && !deps
                    .iter()
                    .any(|dep| dep.implements.contains(&fpm::FPM_UI_INTERFACE.to_string()))
            {
                deps.push(fpm::Dependency {
                    package: fpm::Package::new(fpm::FPM_UI_INTERFACE),
                    version: None,
                    notes: None,
                    alias: None,
                    implements: Vec::new(),
                });
            };
            package.fpm_path = Some(root.join("FPM.ftd"));

            package.dependencies = deps;

            package.auto_import = fpm_doc
                .get::<Vec<String>>("fpm#auto-import")?
                .iter()
                .map(|f| fpm::AutoImport::from_string(f.as_str()))
                .collect();

            package.ignored_paths = fpm_doc.get::<Vec<String>>("fpm#ignore")?;
            package.fonts = fpm_doc.get("fpm#font")?;
            package.sitemap_temp = fpm_doc.get("fpm#sitemap")?;
            package
        };

        fpm::utils::validate_base_url(&package)?;

        if package.import_auto_imports_from_original {
            if let Some(ref original_package) = *package.translation_of {
                if !package.auto_import.is_empty() {
                    return Err(fpm::Error::PackageError {
                        message: format!("Can't use `inherit-auto-imports-from-original` along with auto-imports defined for the translation package. Either set `inherit-auto-imports-from-original` to false or remove `fpm.auto-import` from the {package_name}/FPM.ftd file", package_name=package.name.as_str()),
                    });
                } else {
                    package.auto_import = original_package.auto_import.clone()
                }
            }
        }

        // TODO: resolve group dependent packages, there may be imported group from foreign package
        // TODO: We need to make sure to resolve that package as well before moving ahead
        // TODO: Because in `UserGroup::get_identities` we have to resolve identities of a group

        let user_groups: Vec<crate::user_group::UserGroupTemp> = fpm_doc.get("fpm#user-group")?;
        let groups = crate::user_group::UserGroupTemp::user_groups(user_groups)?;
        package.groups = groups;
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
        };

        let asset_documents = config.get_assets().await?;

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
                        &asset_documents,
                        "/",
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

        config.add_package(&package);

        // Update terms map from the current package files
        config.update_ids_from_package().await?;

        Ok(config)
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
        let package = Package::new(package_name);
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
        req: &actix_web::HttpRequest,
        document_path: &str,
    ) -> fpm::Result<bool> {
        self.can_read_(req, document_path).await
    }

    async fn can_read_(
        &self,
        req: &actix_web::HttpRequest,
        document_path: &str,
    ) -> fpm::Result<bool> {
        use itertools::Itertools;
        let document_name = self.document_name_with_default(document_path);
        let access_identities =
            fpm::user_group::access_identities(self, req, &document_name, true).await?;
        if let Some(sitemap) = &self.package.sitemap {
            // TODO: This can be buggy in case of: if groups are used directly in sitemap are foreign groups
            let document_readers = sitemap.readers(document_name.as_str(), &self.package.groups);
            if document_readers.is_empty() {
                return Ok(true);
            }
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
        req: &actix_web::HttpRequest,
        document_path: &str,
    ) -> fpm::Result<bool> {
        use itertools::Itertools;
        let document_name = self.document_name_with_default(document_path);
        let access_identities =
            fpm::user_group::access_identities(self, req, &document_name, false).await?;

        if let Some(sitemap) = &self.package.sitemap {
            // TODO: This can be buggy in case of: if groups are used directly in sitemap are foreign groups
            let document_writers = sitemap.writers(document_name.as_str(), &self.package.groups);
            return fpm::user_group::belongs_to(
                self,
                document_writers.as_slice(),
                access_identities.iter().collect_vec().as_slice(),
            );
        }

        Ok(false)
    }
}

/// `find_root_for_file()` starts with the given path, which is the current directory where the
/// application started in, and goes up till it finds a folder that contains `FPM.ftd` file.
/// TODO: make async
pub(crate) fn find_root_for_file(
    dir: &camino::Utf8Path,
    file_name: &str,
) -> Option<camino::Utf8PathBuf> {
    if dir.join(file_name).exists() {
        Some(dir.into())
    } else {
        if let Some(p) = dir.parent() {
            return find_root_for_file(p, file_name);
        };
        None
    }
}

/// PackageTemp is a struct that is used for mapping the `fpm.package` data in FPM.ftd file. It is
/// not used elsewhere in program, it is immediately converted to `fpm::Package` struct during
/// deserialization process
#[derive(serde::Deserialize, Debug, Clone)]
pub(crate) struct PackageTemp {
    pub name: String,
    pub versioned: bool,
    #[serde(rename = "translation-of")]
    pub translation_of: Option<String>,
    #[serde(rename = "translation")]
    pub translations: Vec<String>,
    #[serde(rename = "language")]
    pub language: Option<String>,
    pub about: Option<String>,
    pub zip: Option<String>,
    #[serde(rename = "download-base-url")]
    pub download_base_url: Option<String>,
    #[serde(rename = "canonical-url")]
    pub canonical_url: Option<String>,
    #[serde(rename = "inherit-auto-imports-from-original")]
    pub import_auto_imports_from_original: bool,
    #[serde(rename = "favicon")]
    pub favicon: Option<String>,
}

impl PackageTemp {
    pub fn into_package(self) -> fpm::Package {
        // TODO: change this method to: `validate(self) -> fpm::Result<fpm::Package>` and do all
        //       validations in it. Like a package must not have both translation-of and
        //       `translations` set.
        let translation_of = self.translation_of.as_ref().map(|v| fpm::Package::new(v));
        let translations = self
            .translations
            .clone()
            .into_iter()
            .map(|v| fpm::Package::new(&v))
            .collect::<Vec<fpm::Package>>();

        fpm::Package {
            name: self.name,
            versioned: self.versioned,
            translation_of: Box::new(translation_of),
            translations,
            language: self.language,
            about: self.about,
            zip: self.zip,
            download_base_url: self.download_base_url,
            translation_status_summary: None,
            canonical_url: self.canonical_url,
            dependencies: vec![],
            auto_import: vec![],
            fpm_path: None,
            ignored_paths: vec![],
            fonts: vec![],
            import_auto_imports_from_original: self.import_auto_imports_from_original,
            groups: std::collections::BTreeMap::new(),
            sitemap: None,
            sitemap_temp: None,
            favicon: self.favicon,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    /// The `versioned` stores the boolean value storing of the fpm package is versioned or not
    pub versioned: bool,
    pub translation_of: Box<Option<Package>>,
    pub translations: Vec<Package>,
    pub language: Option<String>,
    pub about: Option<String>,
    pub zip: Option<String>,
    pub download_base_url: Option<String>,
    pub translation_status_summary: Option<fpm::translation::TranslationStatusSummary>,
    pub canonical_url: Option<String>,
    /// `dependencies` keeps track of direct dependencies of a given package. This too should be
    /// moved to `fpm::Package` to support recursive dependencies etc.
    pub dependencies: Vec<fpm::Dependency>,
    /// `auto_import` keeps track of the global auto imports in the package.
    pub auto_import: Vec<fpm::AutoImport>,
    /// `fpm_path` contains the fpm package root. This value is found in `FPM.ftd` or
    /// `FPM.manifest.ftd` file.
    pub fpm_path: Option<camino::Utf8PathBuf>,
    /// `ignored` keeps track of files that are to be ignored by `fpm build`, `fpm sync` etc.
    pub ignored_paths: Vec<String>,
    /// `fonts` keeps track of the fonts used by the package.
    ///
    /// Note that this too is kind of bad design, we will move fonts to `fpm::Package` struct soon.
    pub fonts: Vec<fpm::Font>,
    pub import_auto_imports_from_original: bool,

    pub groups: std::collections::BTreeMap<String, crate::user_group::UserGroup>,

    /// sitemap stores the structure of the package. The structure includes sections, subsections
    /// and table of content (`toc`). This automatically converts the documents in package into the
    /// corresponding to structure.
    pub sitemap: Option<fpm::sitemap::Sitemap>,
    pub sitemap_temp: Option<fpm::sitemap::SitemapTemp>,
    /// Optional path for favicon icon to be used.
    ///
    /// By default if any file favicon.* is present in package and favicon is not specified
    /// in FPM.ftd, that file will be used.
    ///
    /// If more than one favicon.* file is present, we will use them
    /// in following priority: .ico > .svg > .png > .jpg.
    pub favicon: Option<String>,
}

impl Package {
    pub fn new(name: &str) -> fpm::Package {
        fpm::Package {
            name: name.to_string(),
            versioned: false,
            translation_of: Box::new(None),
            translations: vec![],
            language: None,
            about: None,
            zip: None,
            download_base_url: None,
            translation_status_summary: None,
            canonical_url: None,
            dependencies: vec![],
            auto_import: vec![],
            fpm_path: None,
            ignored_paths: vec![],
            fonts: vec![],
            import_auto_imports_from_original: true,
            groups: std::collections::BTreeMap::new(),
            sitemap_temp: None,
            sitemap: None,
            favicon: None,
        }
    }

    pub fn get_font_ftd(&self) -> Option<String> {
        use itertools::Itertools;
        if self.fonts.is_empty() {
            None
        } else {
            let (font_record, fonts) = self
                .fonts
                .iter()
                .unique_by(|font| font.name.as_str())
                .collect_vec()
                .iter()
                .fold(
                    (
                        String::from("-- record font:"),
                        String::from("-- font fonts:"),
                    ),
                    |(record_accumulator, instance_accumulator), font| {
                        (
                            format!(
                                "{pre}\nstring {font_var_name}:",
                                pre = record_accumulator,
                                font_var_name = font.name.as_str(),
                            ),
                            format!(
                                "{pre}\n{font_var_name}: {font_var_val}",
                                pre = instance_accumulator,
                                font_var_name = font.name.as_str(),
                                font_var_val = font.html_name(self.name.as_str())
                            ),
                        )
                    },
                );
            Some(format!(
                indoc::indoc! {"
                            {font_record}
                            {fonts}
                        "},
                font_record = font_record,
                fonts = fonts
            ))
        }
    }

    pub fn with_base(mut self, base: String) -> fpm::Package {
        self.download_base_url = Some(base);
        self
    }

    pub fn get_dependency_for_interface(&self, interface: &str) -> Option<&fpm::Dependency> {
        self.dependencies
            .iter()
            .find(|dep| dep.implements.contains(&interface.to_string()))
    }

    pub fn get_flattened_dependencies(&self) -> Vec<fpm::Dependency> {
        self.dependencies
            .clone()
            .into_iter()
            .fold(&mut vec![], |old_val, dep| {
                old_val.extend(dep.package.get_flattened_dependencies());
                old_val.push(dep);
                old_val
            })
            .to_owned()
    }

    pub fn get_font_html(&self) -> String {
        self.fonts.iter().fold(String::new(), |accumulator, font| {
            format!(
                "{pre}{new}\n",
                pre = accumulator,
                new = font.to_html(self.name.as_str())
            )
        })
    }

    pub fn generate_prefix_string(&self, with_alias: bool) -> Option<String> {
        self.auto_import.iter().fold(None, |pre, ai| {
            let mut import_doc_path = ai.path.clone();
            if !with_alias {
                // Check for the aliases and map them to the full path
                for dependency in &self.dependencies {
                    if let Some(alias) = &dependency.alias {
                        if alias.as_str().eq(ai.path.as_str())
                            || ai.path.starts_with(format!("{}/", alias).as_str())
                        {
                            import_doc_path = ai.path.replacen(
                                dependency.alias.as_ref()?.as_str(),
                                dependency.package.name.as_str(),
                                1,
                            );
                        }
                    }
                }
            }
            Some(format!(
                "{}\n-- import: {}{}",
                pre.unwrap_or_else(|| "".to_string()),
                &import_doc_path,
                match &ai.alias {
                    Some(a) => format!(" as {}", a),
                    None => String::new(),
                }
            ))
        })
    }

    /// returns the full path of the import from its alias if valid
    /// otherwise returns None
    pub fn get_full_path_from_alias(&self, alias: &str) -> Option<String> {
        let mut full_path: Option<String> = None;

        for dependency in &self.dependencies {
            if let Some(dep_alias) = &dependency.alias {
                if dep_alias.as_str().eq(alias) {
                    full_path = Some(dependency.package.name.clone());
                }
            }
        }

        full_path
    }

    /// returns expanded import path given Type-1 aliased import content
    pub fn fix_aliased_import_type1(
        &self,
        import_content: &str,
        id: &str,
        line_number: usize,
        with_alias: bool,
    ) -> ftd::p1::Result<String> {
        let mut parts = import_content.splitn(2, '/');
        match (parts.next(), parts.next()) {
            (Some(front), Some(rem)) => {
                // case 1: -- import alias/x..
                // front = alias, rem = x..

                let extended_front = self.get_full_path_from_alias(front);
                match extended_front {
                    Some(ext_front) => Ok(format!("{}/{}", ext_front, rem)),
                    None => Ok(format!("{}/{}", front, rem)),
                }
            }
            (Some(front), None) => {
                // case 2: -- import alias
                // front = alias

                let extended_front = self.get_full_path_from_alias(front);
                match extended_front {
                    Some(ext_front) => match with_alias {
                        true => Ok(format!("{} as {}", ext_front, front)),
                        false => Ok(ext_front),
                    },
                    None => Ok(front.to_string()),
                }
            }
            _ => {
                // Throw error for unknown type-1 import
                Err(ftd::p1::Error::ParseError {
                    message: "invalid aliased import !! (Type-1)".to_string(),
                    doc_id: id.to_string(),
                    line_number,
                })
            }
        }
    }

    /// returns expanded import path given Type-2 aliased import content
    pub fn fix_aliased_import_type2(
        &self,
        import_content: &str,
        id: &str,
        line_number: usize,
    ) -> ftd::p1::Result<String> {
        let mut parts = import_content.splitn(2, " as ");

        match (parts.next(), parts.next()) {
            (Some(front), Some(alias)) => {
                // case 1: -- import alias/x.. as alias_2
                // case 2: -- import alias as alias_2
                // front = alias/x or alias, alias = alias_2

                let extended_front =
                    self.fix_aliased_import_type1(front, id, line_number, false)?;
                Ok(format!("{} as {}", extended_front, alias))
            }
            _ => {
                // Throw error for unknown type-2 import
                Err(ftd::p1::Error::ParseError {
                    message: "invalid aliased import !! (Type-2)".to_string(),
                    doc_id: id.to_string(),
                    line_number,
                })
            }
        }
    }

    /// will map aliased imports to full path in the actual body of the document
    /// and return the new document body as string
    ///
    /// For ftd files apart from FPM.ftd
    ///
    /// If aliased imports of Type-1 and Type-2 are used
    /// then those will be mapped to its corresponding full import paths
    ///
    /// [`Type-1`] aliased imports
    ///
    /// case 1: -- import alias
    ///
    /// map:    -- import full_path_of_alias as alias
    ///
    /// case 2: -- import alias/x..
    ///
    /// map:    -- import full_path_of_alias/x..
    ///
    /// [`Type-2`] aliased imports
    ///
    /// case 1: -- import alias/x.. as alias_2
    ///
    /// map:    -- import full_path_of_alias/x.. as alias_2
    ///
    /// case 2: -- import alias as alias_2
    ///
    /// map:    -- import full_path_of_alias as alias_2
    ///
    pub fn fix_imports_in_body(&self, body: &str, id: &str) -> ftd::p1::Result<String> {
        let mut new_body = String::new();
        let mut ln = 1;

        for line in body.lines() {
            let line_string = line.trim();

            let final_line = {
                if line_string.starts_with("-- import") {
                    // Split [-- import | content]
                    let import_tokens: Vec<&str> = line_string.split(':').collect();
                    if import_tokens.len() <= 1 {
                        return Err(ftd::p1::Error::ParseError {
                            message: "Import content missing !!".to_string(),
                            doc_id: id.to_string(),
                            line_number: ln,
                        });
                    }

                    // Initial import content from the doc
                    let mut import_content = String::from(import_tokens[1].trim());

                    import_content = match import_content.contains(" as ") {
                        true => self.fix_aliased_import_type2(import_content.as_str(), id, ln)?,
                        false => {
                            self.fix_aliased_import_type1(import_content.as_str(), id, ln, true)?
                        }
                    };

                    format!("-- import: {}", &import_content)
                } else {
                    // No change in line push as it is
                    line.to_string()
                }
            };

            new_body.push_str(&final_line);
            new_body.push('\n');

            ln += 1;
        }

        Ok(new_body)
    }

    pub fn get_prefixed_body(&self, body: &str, id: &str, with_alias: bool) -> String {
        if id.contains("FPM/") {
            return body.to_string();
        };
        match self.generate_prefix_string(with_alias) {
            Some(s) => format!("{}\n\n{}", s.trim(), body),
            None => body.to_string(),
        }
    }

    pub fn eval_auto_import(&self, name: &str) -> Option<&str> {
        for x in &self.auto_import {
            let matching_string = match &x.alias {
                Some(a) => a.as_str(),
                None => x.path.as_str(),
            };
            if matching_string == name {
                return Some(&x.path);
            };
        }
        None
    }

    pub fn generate_canonical_url(&self, path: &str) -> String {
        if let Some(path) = path.strip_prefix("-/") {
            let mut url = path
                .split_once("-/")
                .map(|(v, _)| v.trim_matches('/'))
                .unwrap_or_else(|| path.trim_matches('/'))
                .to_string();
            if !url.ends_with(".html") {
                url = format!("{}/", url);
            }

            return format!("\n<link rel=\"canonical\" href=\"{url}\" />", url = url);
        }

        if path.starts_with("-/") {
            return "".to_string();
        }
        let (path, canonical_url) = path
            .split_once("-/")
            .map(|(v, _)| {
                (
                    v.trim_matches('/'),
                    Some(
                        self.canonical_url
                            .clone()
                            .unwrap_or_else(|| self.name.to_string()),
                    ),
                )
            })
            .unwrap_or((path.trim_matches('/'), self.canonical_url.clone()));
        match canonical_url {
            Some(url) => {
                let url = if !url.ends_with('/') {
                    format!("{}/", url)
                } else {
                    url
                };
                // Ignore the FPM document as that path won't exist in the reference website
                format!(
                    "\n<link rel=\"canonical\" href=\"{canonical_base}{path}\" />",
                    canonical_base = url,
                    path = path
                )
            }
            None => "".to_string(),
        }
    }

    /// aliases() returns the list of the available aliases at the package level.
    pub fn aliases(&self) -> std::collections::BTreeMap<&str, &fpm::Package> {
        let mut resp = std::collections::BTreeMap::new();
        for d in &self.dependencies {
            if let Some(a) = &d.alias {
                resp.insert(a.as_str(), &d.package);
            }
            resp.insert(&d.package.name, &d.package);
        }
        resp
    }

    pub async fn get_assets_doc(&self) -> fpm::Result<String> {
        // Virtual document that contains the asset information about the package
        Ok(self.get_font_ftd().unwrap_or_default())
    }

    pub(crate) async fn get_fpm(&self) -> fpm::Result<String> {
        crate::http::construct_url_and_get_str(format!("{}/FPM.ftd", self.name).as_str()).await
    }

    pub(crate) async fn resolve(&mut self, fpm_path: &camino::Utf8PathBuf) -> fpm::Result<()> {
        let fpm_document = {
            let doc = tokio::fs::read_to_string(fpm_path).await?;
            let lib = fpm::FPMLibrary::default();
            match fpm::doc::parse_ftd("FPM", doc.as_str(), &lib) {
                Ok(v) => v,
                Err(e) => {
                    return Err(fpm::Error::PackageError {
                        message: format!("failed to parse FPM.ftd: {:?}", &e),
                    });
                }
            }
        };
        let mut package = {
            let temp_package: fpm::config::PackageTemp = fpm_document.get("fpm#package")?;
            temp_package.into_package()
        };
        package.translation_status_summary = fpm_document.get("fpm#translation-status-summary")?;
        package.fpm_path = Some(fpm_path.to_owned());
        package.dependencies = fpm_document
            .get::<Vec<fpm::dependency::DependencyTemp>>("fpm#dependency")?
            .into_iter()
            .map(|v| v.into_dependency())
            .collect::<Vec<fpm::Result<fpm::Dependency>>>()
            .into_iter()
            .collect::<fpm::Result<Vec<fpm::Dependency>>>()?;

        let user_groups: Vec<crate::user_group::UserGroupTemp> =
            fpm_document.get("fpm#user-group")?;
        let groups = crate::user_group::UserGroupTemp::user_groups(user_groups)?;
        package.groups = groups;
        package.auto_import = fpm_document
            .get::<Vec<String>>("fpm#auto-import")?
            .iter()
            .map(|f| fpm::AutoImport::from_string(f.as_str()))
            .collect();
        package.fonts = fpm_document.get("fpm#font")?;
        package.sitemap_temp = fpm_document.get("fpm#sitemap")?;
        *self = package;
        Ok(())
    }

    pub(crate) async fn get_and_resolve(
        &self,
        package_root: &camino::Utf8PathBuf,
    ) -> fpm::Result<fpm::Package> {
        use tokio::io::AsyncWriteExt;

        let file_extract_path = package_root.join("FPM.ftd");
        if !file_extract_path.exists() {
            std::fs::create_dir_all(&package_root)?;
            let fpm_string = self.get_fpm().await?;
            tokio::fs::File::create(&file_extract_path)
                .await?
                .write_all(fpm_string.as_bytes())
                .await?;
        }

        let mut package = self.clone();
        package.resolve(&file_extract_path).await?;
        Ok(package)
    }
}
