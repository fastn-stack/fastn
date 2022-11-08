pub mod dependency;
pub mod package_doc;
pub mod user_group;

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
    pub dependencies: Vec<dependency::Dependency>,
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

    pub dynamic_urls: Option<fpm::sitemap::DynamicUrls>,
    pub dynamic_urls_temp: Option<fpm::sitemap::DynamicUrlsTemp>,

    /// Optional path for favicon icon to be used.
    ///
    /// By default if any file favicon.* is present in package and favicon is not specified
    /// in FPM.ftd, that file will be used.
    ///
    /// If more than one favicon.* file is present, we will use them
    /// in following priority: .ico > .svg > .png > .jpg.
    pub favicon: Option<String>,

    /// endpoint for proxy service
    pub endpoint: Option<String>,

    /// Attribute to define the usage of a WASM backend
    pub backend: bool,

    /// Headers for the WASM backend
    pub backend_headers: Option<Vec<BackendHeader>>,

    /// Installed Apps
    pub apps: Vec<App>,
}

#[derive(Debug, Clone)]
pub struct App {
    pub name: Option<String>,
    pub package: fpm::Dependency,
    pub mount_point: String,
    pub end_point: Option<String>,
    pub config: std::collections::HashMap<String, String>,
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
            dynamic_urls: None,
            dynamic_urls_temp: None,
            favicon: None,
            endpoint: None,
            backend: false,
            backend_headers: None,
            apps: vec![],
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
                pre.unwrap_or_default(),
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
            let temp_package: PackageTemp = fpm_document.get("fpm#package")?;
            temp_package.into_package()
        };
        package.translation_status_summary = fpm_document.get("fpm#translation-status-summary")?;
        package.fpm_path = Some(fpm_path.to_owned());
        package.dependencies = fpm_document
            .get::<Vec<dependency::DependencyTemp>>("fpm#dependency")?
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
            std::fs::create_dir_all(package_root)?;
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

    pub fn from_fpm_doc(
        root: &camino::Utf8Path,
        fpm_doc: &ftd::p2::Document,
    ) -> fpm::Result<Package> {
        let temp_package: Option<PackageTemp> = fpm_doc.get("fpm#package")?;

        let mut package = match temp_package {
            Some(v) => v.into_package(),
            None => {
                return Err(fpm::Error::PackageError {
                    message: "FPM.ftd does not contain package definition".to_string(),
                })
            }
        };

        // reading dependencies
        let mut deps = {
            let temp_deps: Vec<fpm::package::dependency::DependencyTemp> =
                fpm_doc.get("fpm#dependency")?;
            temp_deps
                .into_iter()
                .map(|v| v.into_dependency())
                .collect::<Vec<fpm::Result<fpm::Dependency>>>()
                .into_iter()
                .collect::<fpm::Result<Vec<fpm::Dependency>>>()?
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
                endpoint: None,
                mountpoint: None,
            });
        };
        // setting dependencies
        package.dependencies = deps;
        package.fpm_path = Some(root.join("FPM.ftd"));

        package.auto_import = fpm_doc
            .get::<Vec<String>>("fpm#auto-import")?
            .iter()
            .map(|f| fpm::AutoImport::from_string(f.as_str()))
            .collect();

        package.ignored_paths = fpm_doc.get::<Vec<String>>("fpm#ignore")?;
        package.fonts = fpm_doc.get("fpm#font")?;
        package.sitemap_temp = fpm_doc.get("fpm#sitemap")?;
        package.dynamic_urls_temp = fpm_doc.get("fpm#dynamic-urls")?;

        // TODO: resolve group dependent packages, there may be imported group from foreign package
        //   We need to make sure to resolve that package as well before moving ahead
        //   Because in `UserGroup::get_identities` we have to resolve identities of a group
        let user_groups: Vec<crate::user_group::UserGroupTemp> = fpm_doc.get("fpm#user-group")?;
        let groups = crate::user_group::UserGroupTemp::user_groups(user_groups)?;
        package.groups = groups;

        // validation logic TODO: It should be ordered
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

        // fpm installed Apps
        let apps: Vec<fpm::package::AppTemp> = fpm_doc.get("fpm#app")?;
        package.apps = apps
            .into_iter()
            .map(|x| x.into_app())
            .collect::<fpm::Result<_>>()?;

        dbg!(&package.apps);

        Ok(package)
    }

    // Dependencies with mount point and end point
    // Output: Package Dependencies
    // [Package, endpoint, mount-point]
    pub fn dep_with_ep_and_mp(&self) -> Vec<(&Package, &str, &str)> {
        self.dependencies
            .iter()
            .fold(&mut vec![], |accumulator, dep| {
                if let Some(ep) = &dep.endpoint {
                    if let Some(mp) = &dep.mountpoint {
                        accumulator.push((&dep.package, ep.as_str(), mp.as_str()))
                    }
                }

                accumulator
            })
            .to_owned()
    }

    // Output: Package's dependency which contains mount-point and endpoint
    // where request path starts-with dependency mount-point.
    // (endpoint, sanitized request path from mount-point)
    pub fn get_dep_endpoint<'a>(&'a self, path: &'a str) -> Option<(&'a str, &'a str)> {
        fn dep_endpoint<'a>(package: &'a Package, path: &'a str) -> Option<(&'a str, &'a str)> {
            let dependencies = package.dep_with_ep_and_mp();
            for (_, ep, mp) in dependencies {
                if path.starts_with(mp.trim_matches('/')) {
                    let path_without_mp = path.trim_start_matches(mp.trim_start_matches('/'));
                    return Some((ep, path_without_mp));
                }
            }
            None
        }

        match dep_endpoint(self, path) {
            Some((ep, r)) => Some((ep, r)),
            // TODO: should it refer to default package or not?
            None => self.endpoint.as_ref().map(|ep| (ep.as_str(), path)),
        }
    }
}

/// Backend Header is a struct that is used to read and store the backend-header from the FPM.ftd file
#[derive(serde::Deserialize, Debug, Clone)]
pub struct BackendHeader {
    #[serde(rename = "header-key")]
    pub header_key: String,
    #[serde(rename = "header-value")]
    pub header_value: String,
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
    #[serde(rename = "endpoint")]
    pub endpoint: Option<String>,
    #[serde(rename = "backend")]
    pub backend: bool,
    #[serde(rename = "backend-headers")]
    pub backend_headers: Option<Vec<BackendHeader>>,
}

impl PackageTemp {
    pub fn into_package(self) -> Package {
        // TODO: change this method to: `validate(self) -> fpm::Result<fpm::Package>` and do all
        //       validations in it. Like a package must not have both translation-of and
        //       `translations` set.
        let translation_of = self.translation_of.as_ref().map(|v| fpm::Package::new(v));
        let translations = self
            .translations
            .clone()
            .into_iter()
            .map(|v| Package::new(&v))
            .collect::<Vec<Package>>();

        Package {
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
            dynamic_urls: None,
            dynamic_urls_temp: None,
            favicon: self.favicon,
            endpoint: self.endpoint,
            backend: self.backend,
            backend_headers: self.backend_headers,
            apps: vec![], //self.apps.into_iter().map(|x| x.into_app()).collect_vec(),
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct AppTemp {
    pub name: Option<String>,
    pub package: String,
    #[serde(rename = "mount-point")]
    pub mount_point: String,
    #[serde(rename = "end-point")]
    pub end_point: Option<String>,
    pub config: Vec<String>,
}

impl AppTemp {
    fn parse_config(config: &[String]) -> fpm::Result<std::collections::HashMap<String, String>> {
        let mut hm = std::collections::HashMap::new();
        for key_value in config.iter() {
            // <key>=<value>
            let (key, value): (&str, &str) = match key_value.trim().split_once('=') {
                Some(x) => x,
                None => {
                    return Err(fpm::Error::PackageError {
                        message: format!(
                            "package-config-error, wrong header in an fpm app, format is <key>=<value>, config: {}",
                            key_value
                        ),
                    });
                }
            };
            // if value = $ENV.env_var_name
            // so read env_var_name from std::env
            let value = value.trim();
            if value.starts_with("$ENV") {
                let (_, env_var_name) = match value.trim().split_once('.') {
                    Some(x) => x,
                    None => return Err(fpm::Error::PackageError {
                        message: format!(
                            "package-config-error, wrong $ENV in an fpm app, format is <key>=$ENV.env_var_name, key: {}, value: {}",
                            key, value
                        ),
                    }),
                };

                let value =
                    std::env::var(env_var_name).map_err(|err| fpm::Error::PackageError {
                        message: format!(
                            "package-config-error,$ENV {} variable is not set for {}, err: {}",
                            env_var_name, value, err
                        ),
                    })?;
                hm.insert(key.to_string(), value.to_string());
            } else {
                hm.insert(key.to_string(), value.to_string());
            }
        }
        Ok(hm)
    }

    pub fn into_app(self) -> fpm::Result<App> {
        let package = fpm::Dependency {
            package: fpm::Package::new(self.package.trim().trim_matches('/')),
            version: None,
            notes: None,
            alias: None,
            implements: Vec::new(),
            endpoint: None,
            mountpoint: None,
        };

        Ok(App {
            name: self.name,
            package,
            mount_point: self.mount_point,
            end_point: self.end_point,
            config: Self::parse_config(&self.config)?,
        })
    }
}
