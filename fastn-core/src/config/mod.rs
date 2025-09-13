pub mod config_temp;
pub(crate) mod utils;

pub use config_temp::ConfigTemp;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FTDEdition {
    FTD2022,
    #[default]
    FTD2023,
}

impl FTDEdition {
    pub(crate) fn from_string(s: &str) -> fastn_core::Result<FTDEdition> {
        match s {
            "2022" => Ok(FTDEdition::FTD2022),
            "2023" => Ok(FTDEdition::FTD2023),
            t => fastn_core::usage_error(format!("Unknown edition `{t}`. Help use `2022` instead")),
        }
    }
    pub(crate) fn is_2023(&self) -> bool {
        matches!(self, fastn_core::FTDEdition::FTD2023)
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    // Global Information
    pub ds: fastn_ds::DocumentStore,
    pub package: fastn_core::Package,
    pub packages_root: fastn_ds::Path,
    pub original_directory: fastn_ds::Path,
    pub all_packages: scc::HashMap<String, fastn_core::Package>,
    pub global_ids: std::collections::HashMap<String, String>,
    pub ftd_edition: FTDEdition,
    pub ftd_external_js: Vec<String>,
    pub ftd_inline_js: Vec<String>,
    pub ftd_external_css: Vec<String>,
    pub ftd_inline_css: Vec<String>,
    pub test_command_running: bool,
    pub enable_cache: bool,
}

#[derive(Debug, Clone)]
pub struct RequestConfig {
    pub named_parameters: Vec<(String, ftd::Value)>,
    pub extra_data: std::collections::BTreeMap<String, String>,
    pub downloaded_assets: std::collections::BTreeMap<String, String>,
    pub current_document: Option<String>,
    pub dependencies_during_render: Vec<String>,
    pub request: fastn_core::http::Request,
    pub config: Config,
    /// If the current module being parsed is a markdown file, `.markdown` contains the name and
    /// content of that file
    pub markdown: Option<(String, String)>,
    pub document_id: String,
    pub translated_data: fastn_core::TranslationData,
    pub base_url: String,
    pub module_package_map: std::collections::BTreeMap<String, String>,
    /// each string is the value of Set-Cookie header
    pub processor_set_cookies: Vec<String>,
    pub processor_set_response: Option<ft_sys_shared::Request>,
    /// we use this to determine if the response is cacheable or not
    pub response_is_cacheable: bool,
}

impl RequestConfig {
    pub fn url(&self) -> String {
        self.request.uri.clone()
    }

    /// https://www.example.com/test/ -> https://www.example.com
    pub fn url_prefix(&self) -> String {
        format!(
            "{}://{}",
            self.request.connection_info.scheme(),
            self.request.host(),
        )
    }

    pub fn current_language(&self) -> Option<String> {
        self.config.package.selected_language.clone()
    }

    #[tracing::instrument(skip_all)]
    pub fn new(
        config: &Config,
        request: &fastn_core::http::Request,
        document_id: &str,
        base_url: &str,
    ) -> Self {
        RequestConfig {
            named_parameters: vec![],
            extra_data: Default::default(),
            downloaded_assets: Default::default(),
            current_document: None,
            dependencies_during_render: vec![],
            request: request.clone(),
            config: config.clone(),
            markdown: None,
            document_id: document_id.to_string(),
            translated_data: Default::default(),
            base_url: base_url.to_string(),
            module_package_map: Default::default(),
            processor_set_cookies: Default::default(),
            processor_set_response: None,
            response_is_cacheable: true,
        }
    }

    pub fn doc_id(&self) -> Option<String> {
        self.current_document
            .clone()
            .map(|v| fastn_core::utils::id_to_path(v.as_str()))
            .map(|v| v.trim().replace(std::path::MAIN_SEPARATOR, "/"))
    }

    /// document_name_with_default("index.ftd") -> /
    /// document_name_with_default("foo/index.ftd") -> /foo/
    /// document_name_with_default("foo/abc") -> /foo/abc/
    /// document_name_with_default("/foo/abc.ftd") -> /foo/abc/
    #[allow(dead_code)]
    pub(crate) fn document_name_with_default(&self, document_path: &str) -> String {
        let name = self
            .doc_id()
            .unwrap_or_else(|| document_path.to_string())
            .trim_matches('/')
            .to_string();
        if name.is_empty() {
            "/".to_string()
        } else {
            format!("/{name}/")
        }
    }

    // -/kameri-app.herokuapp.com/
    // .packages/kameri-app.heroku.com/index.ftd
    #[tracing::instrument(skip(self))]
    pub async fn get_file_and_package_by_id(
        &mut self,
        path: &str,
        preview_session_id: &Option<String>,
    ) -> fastn_core::Result<fastn_core::File> {
        // This function will return file and package by given path
        // path can be mounted(mount-point) with other dependencies
        //
        // Sanitize the mountpoint request.
        // Get the package and sanitized path
        let package1;
        let new_path1;

        // TODO: The shitty code written by me ever
        let (path_with_package_name, document, path_params, extra_data) =
            if fastn_core::file::is_static(path)? {
                (path, None, vec![], Default::default())
            } else {
                let (path_with_package_name, sanitized_package, sanitized_path) =
                    match self.config.get_mountpoint_sanitized_path(path) {
                        Some((new_path, package, remaining_path, app)) => {
                            // Update the sitemap of the package, if it does not contain the sitemap information
                            if let Some(app) = app {
                                let mut headers: std::collections::HashMap<String, String> =
                                    Default::default();
                                headers.insert(
                                    fastn_wasm::FASTN_APP_URL_HEADER.to_string(),
                                    app.mount_point.to_string(),
                                );
                                self.request.set_headers(&headers);
                            }
                            new_path1 = new_path;
                            if package.name != self.config.package.name {
                                package1 = self
                                    .config
                                    .update_sitemap(package, preview_session_id)
                                    .await?;
                                (new_path1.as_ref(), &package1, remaining_path)
                            } else {
                                (new_path1.as_ref(), package, remaining_path)
                            }
                        }
                        None => (path, &self.config.package, path.to_string()),
                    };

                // Getting `document` with dynamic parameters, if exists
                // It will first resolve in the sitemap.
                // If not found, resolve in the dynamic urls.
                let (document, path_params, extra_data) =
                    fastn_core::sitemap::resolve(sanitized_package, &sanitized_path)?;

                // document with package-name prefix
                let document = document.map(|doc| {
                    format!(
                        "-/{}/{}",
                        sanitized_package.name.trim_matches('/'),
                        doc.trim_matches('/')
                    )
                });
                (path_with_package_name, document, path_params, extra_data)
            };

        let path = path_with_package_name;

        tracing::info!("resolved path: {path}");
        tracing::info!(
            "document: {document:?}, path_params: {path_params:?}, extra_data: {extra_data:?}"
        );

        if let Some(id) = document {
            let file_name = self
                .config
                .get_file_path_and_resolve(id.as_str(), preview_session_id)
                .await?;
            let package = self.config.find_package_by_id(id.as_str()).await?.1;
            let file = fastn_core::get_file(
                &self.config.ds,
                package.name.to_string(),
                &self.config.ds.root().join(file_name),
                &self.config.get_root_for_package(&package),
                preview_session_id,
            )
            .await?;
            self.current_document = Some(path.to_string());
            self.named_parameters = path_params;
            self.extra_data = extra_data;
            Ok(file)
        } else {
            // -/fifthtry.github.io/todos/add-todo/
            // -/fifthtry.github.io/doc-site/add-todo/
            let file_name = self
                .config
                .get_file_path_and_resolve(path, preview_session_id)
                .await?;
            // .packages/todos/add-todo.ftd
            // .packages/fifthtry.github.io/doc-site/add-todo.ftd

            let package = self.config.find_package_by_id(path).await?.1;
            let mut file = fastn_core::get_file(
                &self.config.ds,
                package.name.to_string(),
                &self
                    .config
                    .ds
                    .root()
                    .join(file_name.trim_start_matches('/')),
                &self.config.get_root_for_package(&package),
                preview_session_id,
            )
            .await?;

            if path.contains("-/") {
                let url = path.trim_end_matches("/index.html").trim_matches('/');
                let extension = if matches!(file, fastn_core::File::Markdown(_)) {
                    "/index.md".to_string()
                } else if matches!(file, fastn_core::File::Ftd(_)) {
                    "/index.ftd".to_string()
                } else {
                    "".to_string()
                };
                file.set_id(format!("{url}{extension}").as_str());
            }
            self.current_document = Some(file.get_id().to_string());
            Ok(file)
        }
    }

    // Authenticated user's session id
    pub(crate) fn session_id(&self) -> Option<String> {
        self.request.cookie(fastn_core::http::SESSION_COOKIE_NAME)
    }
}

impl Config {
    /// `build_dir` is where the static built files are stored. `fastn build` command creates this
    /// folder and stores its output here.
    pub fn build_dir(&self) -> fastn_ds::Path {
        self.ds.root().join(".build")
    }

    pub fn clone_dir(&self) -> fastn_ds::Path {
        self.ds.root().join(".clone-state")
    }

    pub fn workspace_file(&self) -> fastn_ds::Path {
        self.clone_dir().join("workspace.ftd")
    }

    pub fn path_without_root(&self, path: &fastn_ds::Path) -> fastn_core::Result<String> {
        Ok(path
            .strip_prefix(&self.ds.root())
            .ok_or(fastn_core::Error::UsageError {
                message: format!("Can't find prefix `{}` in `{}`", self.ds.root(), path),
            })?
            .to_string())
    }

    pub fn track_path(&self, path: &fastn_ds::Path) -> fastn_ds::Path {
        let path_without_root = self
            .path_without_root(path)
            .unwrap_or_else(|_| path.to_string());
        let track_path = format!("{path_without_root}.track");
        self.track_dir().join(track_path)
    }

    pub(crate) fn package_info_package(&self) -> &str {
        match self
            .package
            .get_dependency_for_interface(fastn_core::FASTN_UI_INTERFACE)
            .or_else(|| {
                self.package
                    .get_dependency_for_interface(fastn_core::PACKAGE_THEME_INTERFACE)
            }) {
            Some(dep) => dep.package.name.as_str(),
            None => fastn_core::FASTN_UI_INTERFACE,
        }
    }

    pub fn remote_dir(&self) -> fastn_ds::Path {
        self.ds.root().join(".remote-state")
    }

    pub fn remote_history_dir(&self) -> fastn_ds::Path {
        self.remote_dir().join("history")
    }

    /// location that stores lowest available cr number
    pub fn remote_cr(&self) -> fastn_ds::Path {
        self.remote_dir().join("cr")
    }

    pub fn history_file(&self) -> fastn_ds::Path {
        self.remote_dir().join("history.ftd")
    }

    /// history of a fastn package is stored in `.history` folder.
    ///
    /// Current design is wrong, we should move this helper to `fastn_core::Package` maybe.
    ///
    /// History of a package is considered part of the package, and when a package is downloaded we
    /// have to chose if we want to download its history as well. For now we do not. Eventually in
    /// we will be able to say download the history also for some package.
    ///
    /// ```ftd
    /// -- fastn.dependency: django
    ///  with-history: true
    /// ```
    ///     
    /// `.history` file is created or updated by `fastn sync` command only, no one else should edit
    /// anything in it.
    pub fn history_dir(&self) -> fastn_ds::Path {
        self.ds.root().join(".history")
    }

    pub fn fastn_dir(&self) -> fastn_ds::Path {
        self.ds.root().join(".fastn")
    }

    pub fn conflicted_dir(&self) -> fastn_ds::Path {
        self.fastn_dir().join("conflicted")
    }

    /// every package's `.history` contains a file `.latest.ftd`. It looks a bit link this:
    ///
    /// ```ftd
    /// -- import: fastn
    ///
    /// -- fastn.snapshot: FASTN.ftd
    /// timestamp: 1638706756293421000
    ///
    /// -- fastn.snapshot: blog.ftd
    /// timestamp: 1638706756293421000
    /// ```
    ///
    /// One `fastn.snapshot` for every file that is currently part of the package.
    pub fn latest_ftd(&self) -> fastn_ds::Path {
        self.ds.root().join(".history/.latest.ftd")
    }

    /// track_dir returns the directory where track files are stored. Tracking information as well
    /// is considered part of a package, but it is not downloaded when a package is downloaded as
    /// a dependency of another package.
    pub fn track_dir(&self) -> fastn_ds::Path {
        self.ds.root().join(".tracks")
    }

    /// `is_translation_package()` is a helper to tell you if the current package is a translation
    /// of another package. We may delete this helper soon.
    pub fn is_translation_package(&self) -> bool {
        self.package.translation_of.is_some()
    }

    /// original_path() returns the path of the original package if the current package is a
    /// translation package. it returns the path in `.packages` folder where the
    pub fn original_path(&self) -> fastn_core::Result<fastn_ds::Path> {
        let o = match self.package.translation_of {
            Some(ref o) => o,
            None => {
                return Err(fastn_core::Error::UsageError {
                    message: "This package is not a translation package".to_string(),
                });
            }
        };
        match &o.fastn_path {
            Some(fastn_path) => Ok(fastn_path
                .parent()
                .expect("Expect fastn_path parent. Panic!")),
            _ => Err(fastn_core::Error::UsageError {
                message: format!("Unable to find `fastn_path` of the package {}", o.name),
            }),
        }
    }

    /*/// aliases() returns the list of the available aliases at the package level.
    pub fn aliases(&self) -> fastn_core::Result<std::collections::BTreeMap<&str, &fastn_core::Package>> {
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
        let mut generated_style = String::new();
        let mut entry = self.all_packages.first_entry();
        while let Some(package) = entry {
            generated_style.push_str(package.get().get_font_html().as_str());
            generated_style.push('\n');
            entry = package.next();
        }
        match generated_style.trim().is_empty() {
            false => format!("<style>{generated_style}</style>"),
            _ => "".to_string(),
        }
    }

    pub(crate) async fn download_fonts(
        &self,
        session_id: &Option<String>,
    ) -> fastn_core::Result<()> {
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

        fonts.extend(self.get_fonts_from_all_packages());

        for font in fonts.iter() {
            if let Some(url) = font.get_url() {
                if fastn_core::config::utils::is_http_url(&url) {
                    continue;
                }
                let start = std::time::Instant::now();
                print!("Processing {url} ... ");
                let content = self.get_file_and_resolve(url.as_str(), session_id).await?.1;
                fastn_core::utils::update(
                    &self.build_dir().join(&url),
                    content.as_slice(),
                    &self.ds,
                )
                .await?;
                fastn_core::utils::print_end(format!("Processed {url}").as_str(), start);
            }
        }

        Ok(())
    }

    fn get_fonts_from_all_packages(&self) -> Vec<fastn_core::Font> {
        let mut fonts = vec![];
        let mut entry = self.all_packages.first_entry();
        while let Some(package) = entry {
            fonts.extend(package.get().fonts.clone());
            entry = package.next();
        }
        fonts
    }

    /*pub(crate) async fn get_versions(
        &self,
        package: &fastn_core::Package,
    ) -> fastn_core::Result<std::collections::HashMap<fastn_core::Version, Vec<fastn_core::File>>>
    {
        let path = self.get_root_for_package(package);
        let mut hash: std::collections::HashMap<fastn_core::Version, Vec<fastn_core::File>> =
            std::collections::HashMap::new();

        let all_files = self.get_all_file_paths(package)?;

        for file in all_files {
            let version = get_version(&file, &path).await?;
            let file = fastn_core::get_file(
                &self.ds,
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
            x: &fastn_ds::Path,
            path: &fastn_ds::Path,
        ) -> fastn_core::Result<fastn_core::Version> {
            let path_str = path
                .to_string()
                .trim_end_matches(std::path::MAIN_SEPARATOR)
                .to_string();
            let id = if x.to_string().contains(&path_str) {
                x.to_string()
                    .trim_start_matches(path_str.as_str())
                    .trim_start_matches(std::path::MAIN_SEPARATOR)
                    .to_string()
            } else {
                return Err(fastn_core::Error::UsageError {
                    message: format!("{:?} should be a file", x),
                });
            };

            if let Some((v, _)) = id.split_once('/') {
                fastn_core::Version::parse(v)
            } else {
                Ok(fastn_core::Version::base())
            }
        }
    }*/

    pub(crate) fn get_root_for_package(&self, package: &fastn_core::Package) -> fastn_ds::Path {
        if let Some(package_fastn_path) = &package.fastn_path {
            // TODO: Unwrap?
            package_fastn_path.parent().unwrap().to_owned()
        } else if package.name.eq(&self.package.name) {
            self.ds.root().clone()
        } else {
            self.packages_root.clone().join(package.name.as_str())
        }
    }

    pub(crate) async fn get_files(
        &self,
        package: &fastn_core::Package,
        session_id: &Option<String>,
    ) -> fastn_core::Result<Vec<fastn_core::File>> {
        let path = self.get_root_for_package(package);
        let all_files = self.get_all_file_paths(package).await?;
        let mut documents = fastn_core::paths_to_files(
            &self.ds,
            package.name.as_str(),
            all_files,
            &path,
            session_id,
        )
        .await?;
        documents.sort_by_key(|v| v.get_id().to_string());

        Ok(documents)
    }

    pub(crate) async fn get_all_file_paths(
        &self,
        package: &fastn_core::Package,
    ) -> fastn_core::Result<Vec<fastn_ds::Path>> {
        let mut ignored_files = vec![
            ".history".to_string(),
            ".packages".to_string(),
            ".tracks".to_string(),
            "fastn".to_string(),
            "rust-toolchain".to_string(),
            ".build".to_string(),
            "_tests".to_string(),
        ];
        ignored_files.extend(package.ignored_paths.clone());
        Ok(self
            .ds
            .get_all_file_path(
                &self.get_root_for_package(package),
                ignored_files.as_slice(),
            )
            .await)
    }

    // Input
    // path: /todos/add-todo/
    // mount-point: /todos/
    // Output
    // -/<todos-package-name>/add-todo/, <todos-package-name>, /add-todo/
    #[tracing::instrument(skip(self))]
    pub fn get_mountpoint_sanitized_path<'a>(
        &'a self,
        path: &'a str,
    ) -> Option<(
        std::borrow::Cow<'a, str>,
        &'a fastn_core::Package,
        String,
        Option<&'a fastn_core::package::app::App>,
    )> {
        // Problem for recursive dependency is that only current package contains dependency,
        // dependent package does not contain dependency

        // For similar package
        // tracing::info!(package = package.name, path = path);
        let dash_path = self.package.dash_path();
        if path.starts_with(dash_path.as_str()) {
            tracing::info!("path is similar. path: {path}, dash_path: {dash_path}");
            let path_without_package_name = path.trim_start_matches(dash_path.as_str());
            return Some((
                std::borrow::Cow::from(path),
                &self.package,
                path_without_package_name.to_string(),
                None,
            ));
        }

        for (mp, dep, app) in self
            .package
            .apps
            .iter()
            .map(|x| (&x.mount_point, &x.package, x))
        {
            let dash_path = dep.dash_path();
            if path.starts_with(mp.trim_matches('/')) {
                // TODO: Need to handle for recursive dependencies mount-point
                // Note: Currently not working because dependency of package does not contain dependencies
                let package_name = dep.name.trim_matches('/');
                let sanitized_path = path.trim_start_matches(mp.trim_start_matches('/'));

                // If `fastn.app`'s mount-point is '/' then and the request comes on '/' then we
                // end up creating a '//' path (see below line using format!). To avoid this, we
                // set it to "" to form a valid path.
                let sanitized_path = if sanitized_path == "/" {
                    ""
                } else {
                    sanitized_path
                };

                let ret_path = std::borrow::Cow::from(format!("-/{package_name}/{sanitized_path}"));
                tracing::info!(
                    "path is consume by `fastn.app`. path: {path}, mount-point: {mp}, dash_path: {dash_path}"
                );
                tracing::info!(
                    "Returning: path: {ret_path}, sanitized path: {sanitized_path}, app: {app_name}",
                    app_name = app.name
                );
                return Some((ret_path, dep, sanitized_path.to_string(), Some(app)));
            } else if path.starts_with(dash_path.as_str()) {
                tracing::info!(
                    "path is not consumed by any `fastn.app`. path: {path}, dash_path: {dash_path}"
                );
                tracing::info!("Returning: {path}");
                let path_without_package_name = path.trim_start_matches(dash_path.as_str());
                return Some((
                    std::borrow::Cow::from(path),
                    dep,
                    path_without_package_name.to_string(),
                    Some(app),
                ));
            }
        }
        None
    }

    pub async fn update_sitemap(
        &self,
        package: &fastn_core::Package,
        session_id: &Option<String>,
    ) -> fastn_core::Result<fastn_core::Package> {
        let fastn_path = &self.packages_root.join(&package.name).join("FASTN.ftd");

        let fastn_doc = utils::fastn_doc(&self.ds, fastn_path, session_id).await?;

        let mut package = package.clone();

        package.migrations = fastn_core::package::get_migration_data(&fastn_doc)?;

        package.sitemap_temp = fastn_doc.get("fastn#sitemap")?;
        package.dynamic_urls_temp = fastn_doc.get("fastn#dynamic-urls")?;

        package.sitemap = match package.sitemap_temp.as_ref() {
            Some(sitemap_temp) => {
                let mut s = fastn_core::sitemap::Sitemap::parse(
                    sitemap_temp.body.as_str(),
                    &package,
                    self,
                    false,
                    session_id,
                )
                .await?;
                s.readers.clone_from(&sitemap_temp.readers);
                s.writers.clone_from(&sitemap_temp.writers);
                Some(s)
            }
            None => None,
        };

        // Handling of `-- fastn.dynamic-urls:`
        package.dynamic_urls = {
            match &package.dynamic_urls_temp {
                Some(urls_temp) => Some(fastn_core::sitemap::DynamicUrls::parse(
                    &self.global_ids,
                    &package.name,
                    urls_temp.body.as_str(),
                )?),
                None => None,
            }
        };
        Ok(package)
    }

    pub async fn get_file_path(
        &self,
        id: &str,
        session_id: &Option<String>,
    ) -> fastn_core::Result<String> {
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
            package
                .resolve_by_id(id, None, self.package.name.as_str(), &self.ds, session_id)
                .await?
                .0
        ))
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn get_file_path_and_resolve(
        &self,
        id: &str,
        session_id: &Option<String>,
    ) -> fastn_core::Result<String> {
        Ok(self.get_file_and_resolve(id, session_id).await?.0)
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn get_file_and_resolve(
        &self,
        id: &str,
        session_id: &Option<String>,
    ) -> fastn_core::Result<(String, Vec<u8>)> {
        let (package_name, package) = self.find_package_by_id(id).await?;

        let package = self.resolve_package(&package, session_id).await?;
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

        let (file_name, content) = package
            .resolve_by_id(id, None, self.package.name.as_str(), &self.ds, session_id)
            .await?;

        tracing::info!("file: {file_name}");

        Ok((format!("{add_packages}{file_name}"), content))
    }

    /// Return (package name or alias, package)
    pub(crate) async fn find_package_by_id(
        &self,
        id: &str,
    ) -> fastn_core::Result<(String, fastn_core::Package)> {
        let sanitized_id = self
            .get_mountpoint_sanitized_path(id)
            .map(|(x, _, _, _)| x)
            .unwrap_or_else(|| std::borrow::Cow::Borrowed(id));

        let id = sanitized_id.as_ref();
        let id = if let Some(id) = id.strip_prefix("-/") {
            id
        } else {
            return Ok((self.package.name.to_string(), self.package.to_owned()));
        };

        if id.starts_with(self.package.name.as_str()) {
            return Ok((self.package.name.to_string(), self.package.to_owned()));
        }

        if let Some(package) = self.package.aliases().iter().rev().find_map(|(alias, d)| {
            if id.starts_with(alias) {
                Some((alias.to_string(), (*d).to_owned()))
            } else {
                None
            }
        }) {
            return Ok(package);
        }

        if let Some(value) = self.find_package_id_in_all_packages(id) {
            return Ok(value);
        }

        Ok((self.package.name.to_string(), self.package.to_owned()))
    }

    fn find_package_id_in_all_packages(&self, id: &str) -> Option<(String, fastn_core::Package)> {
        let mut item = self.all_packages.first_entry();
        while let Some(package) = item {
            let package_name = package.key();
            if id.starts_with(format!("{package_name}/").as_str()) || id.eq(package_name) {
                return Some((package_name.to_string(), package.get().to_owned()));
            }
            item = package.next();
        }
        None
    }

    pub(crate) async fn get_file_name(
        root: &fastn_ds::Path,
        id: &str,
        ds: &fastn_ds::DocumentStore,
        session_id: &Option<String>,
    ) -> fastn_core::Result<String> {
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
            if ds
                .exists(&root.join(format!("{add_packages}index.ftd")), session_id)
                .await
            {
                return Ok(format!("{add_packages}index.ftd"));
            }
            if ds
                .exists(&root.join(format!("{add_packages}README.md")), session_id)
                .await
            {
                return Ok(format!("{add_packages}README.md"));
            }
            return Err(fastn_core::Error::UsageError {
                message: "File not found".to_string(),
            });
        }
        id = id.trim_matches('/').to_string();
        if ds
            .exists(&root.join(format!("{add_packages}{id}.ftd")), session_id)
            .await
        {
            return Ok(format!("{add_packages}{id}.ftd"));
        }
        if ds
            .exists(
                &root.join(format!("{add_packages}{id}/index.ftd")),
                session_id,
            )
            .await
        {
            return Ok(format!("{add_packages}{id}/index.ftd"));
        }
        if ds
            .exists(&root.join(format!("{add_packages}{id}.md")), session_id)
            .await
        {
            return Ok(format!("{add_packages}{id}.md"));
        }
        if ds
            .exists(
                &root.join(format!("{add_packages}{id}/README.md")),
                session_id,
            )
            .await
        {
            return Ok(format!("{add_packages}{id}/README.md"));
        }
        Err(fastn_core::Error::UsageError {
            message: "File not found".to_string(),
        })
    }

    #[allow(dead_code)]
    async fn get_root_path(
        directory: &fastn_ds::Path,
        ds: &fastn_ds::DocumentStore,
        session_id: &Option<String>,
    ) -> fastn_core::Result<fastn_ds::Path> {
        if let Some(fastn_ftd_root) =
            utils::find_root_for_file(directory, "FASTN.ftd", ds, session_id).await
        {
            return Ok(fastn_ftd_root);
        }
        let fastn_manifest_path = match utils::find_root_for_file(
            directory,
            "fastn.manifest.ftd",
            ds,
            session_id,
        )
        .await
        {
            Some(fastn_manifest_path) => fastn_manifest_path,
            None => {
                return Err(fastn_core::Error::UsageError {
                    message: "FASTN.ftd or fastn.manifest.ftd not found in any parent directory"
                        .to_string(),
                });
            }
        };

        let doc = ds
            .read_to_string(&fastn_manifest_path.join("fastn.manifest.ftd"), session_id)
            .await?;
        let lib = fastn_core::FastnLibrary::default();
        let fastn_manifest_processed =
            match fastn_core::doc::parse_ftd("fastn.manifest", doc.as_str(), &lib) {
                Ok(fastn_manifest_processed) => fastn_manifest_processed,
                Err(e) => {
                    return Err(fastn_core::Error::PackageError {
                        message: format!("failed to parse fastn.manifest.ftd: {:?}", &e),
                    });
                }
            };

        let new_package_root = fastn_manifest_processed
            .get::<String>("fastn.manifest#package-root")?
            .as_str()
            .split('/')
            .fold(fastn_manifest_path, |accumulator, part| {
                accumulator.join(part)
            });

        if ds
            .exists(&new_package_root.join("FASTN.ftd"), session_id)
            .await
        {
            Ok(new_package_root)
        } else {
            Err(fastn_core::Error::PackageError {
                message: "Can't find FASTN.ftd. The path specified in fastn.manifest.ftd doesn't contain the FASTN.ftd file".to_string(),
            })
        }
    }

    pub fn add_edition(self, edition: Option<String>) -> fastn_core::Result<Self> {
        match edition {
            Some(e) => {
                let mut config = self;
                config.ftd_edition = FTDEdition::from_string(e.as_str())?;
                Ok(config)
            }
            None => Ok(self),
        }
    }

    pub fn add_external_js(self, external_js: Vec<String>) -> Self {
        let mut config = self;
        config.ftd_external_js = external_js;
        config
    }

    pub fn add_inline_js(self, inline_js: Vec<String>) -> Self {
        let mut config = self;
        config.ftd_inline_js = inline_js;
        config
    }

    pub fn add_external_css(self, external_css: Vec<String>) -> Self {
        let mut config = self;
        config.ftd_external_css = external_css;
        config
    }

    pub fn add_inline_css(self, inline_css: Vec<String>) -> Self {
        let mut config = self;
        config.ftd_inline_css = inline_css;
        config
    }

    pub fn set_test_command_running(self) -> Self {
        let mut config = self;
        config.test_command_running = true;
        config
    }

    pub fn set_enable_cache(self, enable_cache: bool) -> Self {
        let mut config = self;
        config.enable_cache = enable_cache;
        config
    }

    /// `read()` is the way to read a Config.
    #[tracing::instrument(name = "Config::read", skip_all)]
    pub async fn read(
        ds: fastn_ds::DocumentStore,
        resolve_sitemap: bool,
        session_id: &Option<String>,
    ) -> fastn_core::Result<fastn_core::Config> {
        let original_directory = fastn_ds::Path::new(std::env::current_dir()?.to_str().unwrap()); // todo: remove unwrap()
        let fastn_doc =
            utils::fastn_doc(&ds, &fastn_ds::Path::new("FASTN.ftd"), session_id).await?;
        let mut package = fastn_core::Package::from_fastn_doc(&ds, &fastn_doc)?;
        let package_root = ds.root().join(".packages");
        let all_packages = get_all_packages(&mut package, &package_root, &ds, session_id).await?;
        let mut config = Config {
            package: package.clone(),
            packages_root: package_root.clone(),
            original_directory,
            all_packages,
            global_ids: Default::default(),
            ftd_edition: FTDEdition::default(),
            ftd_external_js: Default::default(),
            ftd_inline_js: Default::default(),
            ftd_external_css: Default::default(),
            ftd_inline_css: Default::default(),
            test_command_running: false,
            enable_cache: false,
            ds,
        };
        // Update global_ids map from the current package files
        // config.update_ids_from_package().await?;

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
                    let mut s = fastn_core::sitemap::Sitemap::parse(
                        sitemap_temp.body.as_str(),
                        &package,
                        &config,
                        resolve_sitemap,
                        session_id,
                    )
                    .await?;
                    s.readers.clone_from(&sitemap_temp.readers);
                    s.writers.clone_from(&sitemap_temp.writers);
                    Some(s)
                }
                None => None,
            }
        };

        // Handling of `-- fastn.dynamic-urls:`
        config.package.dynamic_urls = {
            match &package.dynamic_urls_temp {
                Some(urls_temp) => Some(fastn_core::sitemap::DynamicUrls::parse(
                    &config.global_ids,
                    &package.name,
                    urls_temp.body.as_str(),
                )?),
                None => None,
            }
        };

        // fastn installed Apps
        config.package.apps = {
            let apps_temp: Vec<fastn_core::package::app::AppTemp> = fastn_doc.get("fastn#app")?;
            let mut apps: Vec<fastn_core::package::app::App> = vec![];

            for app in apps_temp.into_iter() {
                let new_app_package = app.package.clone();
                let new_app = app.into_app(&config, session_id).await?;

                if let Some(found_app) = apps.iter().find(|a| a.package.name.eq(&new_app_package)) {
                    return Err(fastn_core::Error::PackageError {
                        message: format!(
                            "Mounting the same package twice is not yet allowed. Tried mounting `{}` which is aready mounted at `{}`",
                            new_app_package, found_app.mount_point
                        ),
                    });
                }

                apps.push(new_app);
            }
            apps
        };

        config.package.endpoints = {
            for endpoint in &mut config.package.endpoints {
                endpoint.endpoint =
                    fastn_core::utils::interpolate_env_vars(&config.ds, &endpoint.endpoint).await?;
            }

            config.package.endpoints
        };

        fastn_wasm::insert_or_update(
            &config.all_packages,
            package.name.to_string(),
            config.package.to_owned(),
        );

        fastn_core::migrations::migrate(&config).await?;

        Ok(config)
    }

    #[cfg(feature = "use-config-json")]
    pub(crate) async fn resolve_package(
        &self,
        package: &fastn_core::Package,
        _session_id: &Option<String>,
    ) -> fastn_core::Result<fastn_core::Package> {
        match self.all_packages.get(&package.name) {
            Some(package) => Ok(package.get().clone()),
            None => Err(fastn_core::Error::PackageError {
                message: format!("Could not resolve package {}", &package.name),
            }),
        }
    }

    #[cfg(not(feature = "use-config-json"))]
    pub(crate) async fn resolve_package(
        &self,
        package: &fastn_core::Package,
        session_id: &Option<String>,
    ) -> fastn_core::Result<fastn_core::Package> {
        if self.package.name.eq(package.name.as_str()) {
            return Ok(self.package.clone());
        }

        if let Some(package) = { self.all_packages.get(package.name.as_str()) } {
            return Ok(package.get().clone());
        }

        let mut package = package
            .get_and_resolve(&self.get_root_for_package(package), &self.ds, session_id)
            .await?;

        ConfigTemp::check_dependencies_provided(&self.package, &mut package)?;
        package.auto_import_language(
            self.package.requested_language.clone(),
            self.package.selected_language.clone(),
        )?;
        self.add_package(&package);
        Ok(package)
    }

    #[cfg(not(feature = "use-config-json"))]
    pub(crate) fn add_package(&self, package: &fastn_core::Package) {
        fastn_wasm::insert_or_update(
            &self.all_packages,
            package.name.to_string(),
            package.to_owned(),
        );
    }

    #[cfg(feature = "use-config-json")]
    pub(crate) fn find_package_else_default(
        &self,
        package_name: &str,
        default: Option<fastn_core::Package>,
    ) -> fastn_core::Package {
        if let Some(package) = self.all_packages.get(package_name) {
            package.get().to_owned()
        } else if let Some(package) = default {
            package.to_owned()
        } else {
            self.package.to_owned()
        }
    }

    #[cfg(not(feature = "use-config-json"))]
    pub(crate) fn find_package_else_default(
        &self,
        package_name: &str,
        default: Option<fastn_core::Package>,
    ) -> fastn_core::Package {
        if let Some(package) = self.all_packages.get(package_name) {
            package.get().to_owned()
        } else if let Some(package) = default {
            package.to_owned()
        } else {
            self.package.to_owned()
        }
    }

    pub(crate) async fn get_db_url(&self) -> String {
        match self.ds.env("FASTN_DB_URL").await {
            Ok(db_url) => db_url,
            Err(_) => self
                .ds
                .env("DATABASE_URL")
                .await
                .unwrap_or_else(|_| "sqlite:///fastn.sqlite".to_string()),
        }
    }

    /// Get mounted apps (package's system name, mount point)
    ///
    /// ```ftd
    /// ;; FASTN.ftd
    /// -- fastn.app: Auth App
    /// package: lets-auth.fifthtry.site
    /// mount-point: /-/auth/
    ///
    /// -- fastn.app: Let's Talk App
    /// package: lets-talk.fifthtry.site
    /// mount-point: /talk/
    /// ```
    ///
    /// Then the value will be a json string:
    ///
    /// ```json
    /// { "lets-auth": "/-/auth/", "lets-talk": "/talk/" }
    /// ```
    ///
    /// Keys `lets-auth` and `lets-talk` are `system` names of the associated packages.
    pub fn app_mounts(&self) -> fastn_core::Result<std::collections::HashMap<String, String>> {
        let mut mounts = std::collections::HashMap::new();

        for a in &self.package.apps {
            if a.package.system.is_none() {
                return fastn_core::usage_error(format!(
                    "Package {} used for app {} is not a system package",
                    a.package.name, a.name
                ));
            }

            mounts.insert(
                a.package.system.clone().expect("already checked for None"),
                a.mount_point.clone(),
            );
        }

        Ok(mounts)
    }
}

#[cfg(feature = "use-config-json")]
async fn get_all_packages(
    package: &mut fastn_core::Package,
    package_root: &fastn_ds::Path,
    ds: &fastn_ds::DocumentStore,
    session_id: &Option<String>,
) -> fastn_core::Result<scc::HashMap<String, fastn_core::Package>> {
    let all_packages = scc::HashMap::new();
    fastn_wasm::insert_or_update(&all_packages, package.name.to_string(), package.to_owned());
    let config_temp = config_temp::ConfigTemp::read(ds, session_id).await?;
    let other = config_temp
        .get_all_packages(ds, package, package_root, session_id)
        .await?;
    let mut entry = other.first_entry();
    while let Some(package) = entry {
        all_packages
            .insert(package.key().to_string(), package.get().to_owned())
            .unwrap();
        entry = package.next();
    }
    Ok(all_packages)
}

#[cfg(not(feature = "use-config-json"))]
async fn get_all_packages(
    package: &mut fastn_core::Package,
    _package_root: &fastn_ds::Path,
    _ds: &fastn_ds::DocumentStore,
    _session_id: &Option<String>,
) -> fastn_core::Result<scc::HashMap<String, fastn_core::Package>> {
    let all_packages = scc::HashMap::new();
    fastn_wasm::insert_or_update(&all_packages, package.name.to_string(), package.to_owned());
    Ok(all_packages)
}
