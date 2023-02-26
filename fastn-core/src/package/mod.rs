pub mod app;
pub mod dependency;
pub mod package_doc;
pub mod user_group;

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    /// The `versioned` stores the boolean value storing of the fastn package is versioned or not
    pub versioned: bool,
    pub translation_of: Box<Option<Package>>,
    pub translations: Vec<Package>,
    pub language: Option<String>,
    pub about: Option<String>,
    pub zip: Option<String>,
    pub download_base_url: Option<String>,
    pub translation_status_summary: Option<fastn_core::translation::TranslationStatusSummary>,
    pub canonical_url: Option<String>,
    /// `dependencies` keeps track of direct dependencies of a given package. This too should be
    /// moved to `fastn_core::Package` to support recursive dependencies etc.
    pub dependencies: Vec<dependency::Dependency>,
    /// `auto_import` keeps track of the global auto imports in the package.
    pub auto_import: Vec<fastn_core::AutoImport>,
    /// `fastn_path` contains the fastn package root. This value is found in `FASTN.ftd` or
    /// `fastn.manifest.ftd` file.
    pub fastn_path: Option<camino::Utf8PathBuf>,
    /// `ignored` keeps track of files that are to be ignored by `fastn build`, `fastn sync` etc.
    pub ignored_paths: Vec<String>,
    /// `fonts` keeps track of the fonts used by the package.
    ///
    /// Note that this too is kind of bad design, we will move fonts to `fastn_core::Package` struct soon.
    pub fonts: Vec<fastn_core::Font>,
    pub import_auto_imports_from_original: bool,

    pub groups: std::collections::BTreeMap<String, crate::user_group::UserGroup>,

    /// sitemap stores the structure of the package. The structure includes sections, subsections
    /// and table of content (`toc`). This automatically converts the documents in package into the
    /// corresponding to structure.
    pub sitemap: Option<fastn_core::sitemap::Sitemap>,
    pub sitemap_temp: Option<fastn_core::sitemap::SitemapTemp>,

    pub dynamic_urls: Option<fastn_core::sitemap::DynamicUrls>,
    pub dynamic_urls_temp: Option<fastn_core::sitemap::DynamicUrlsTemp>,

    /// Optional path for favicon icon to be used.
    ///
    /// By default if any file favicon.* is present in package and favicon is not specified
    /// in FASTN.ftd, that file will be used.
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
    pub apps: Vec<app::App>,

    /// Package Icon
    pub icon: Option<ftd::ImageSrc>,
}

impl Package {
    pub fn new(name: &str) -> fastn_core::Package {
        fastn_core::Package {
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
            fastn_path: None,
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
            icon: None,
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

    pub fn with_base(mut self, base: String) -> fastn_core::Package {
        self.download_base_url = Some(base);
        self
    }

    pub fn get_dependency_for_interface(&self, interface: &str) -> Option<&fastn_core::Dependency> {
        self.dependencies
            .iter()
            .find(|dep| dep.implements.contains(&interface.to_string()))
    }

    pub fn get_flattened_dependencies(&self) -> Vec<fastn_core::Dependency> {
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
    /// For ftd files apart from FASTN.ftd
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
                // Ignore the fastn document as that path won't exist in the reference website
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
    pub fn aliases(&self) -> std::collections::BTreeMap<&str, &fastn_core::Package> {
        let mut resp = std::collections::BTreeMap::new();
        for d in &self.dependencies {
            if let Some(a) = &d.alias {
                resp.insert(a.as_str(), &d.package);
            }
            resp.insert(&d.package.name, &d.package);
        }
        resp
    }

    pub async fn get_assets_doc(&self) -> fastn_core::Result<String> {
        // Virtual document that contains the asset information about the package
        Ok(self.get_font_ftd().unwrap_or_default())
    }

    pub(crate) async fn get_fastn(&self) -> fastn_core::Result<String> {
        crate::http::construct_url_and_get_str(format!("{}/FASTN.ftd", self.name).as_str()).await
    }

    #[tracing::instrument(skip_all)]
    pub(crate) async fn resolve(
        &mut self,
        fastn_path: &camino::Utf8PathBuf,
    ) -> fastn_core::Result<()> {
        tracing::info!(path = fastn_path.as_str());
        let fastn_document = {
            let doc = tokio::fs::read_to_string(fastn_path).await?;
            let lib = fastn_core::FastnLibrary::default();
            match fastn_core::doc::parse_ftd("fastn", doc.as_str(), &lib) {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!(
                        msg = "failed to pare FASTN.ftd file",
                        path = fastn_path.as_str()
                    );
                    return Err(fastn_core::Error::PackageError {
                        message: format!("failed to parse FASTN.ftd: {:?}", &e),
                    });
                }
            }
        };
        let mut package = {
            let temp_package: PackageTemp = fastn_document.get("fastn#package")?;
            temp_package.into_package()
        };
        package.translation_status_summary =
            fastn_document.get("fastn#translation-status-summary")?;
        package.fastn_path = Some(fastn_path.to_owned());
        package.dependencies = fastn_document
            .get::<Vec<dependency::DependencyTemp>>("fastn#dependency")?
            .into_iter()
            .map(|v| v.into_dependency())
            .collect::<Vec<fastn_core::Result<fastn_core::Dependency>>>()
            .into_iter()
            .collect::<fastn_core::Result<Vec<fastn_core::Dependency>>>()?;

        let user_groups: Vec<crate::user_group::UserGroupTemp> =
            fastn_document.get("fastn#user-group")?;
        let groups = crate::user_group::UserGroupTemp::user_groups(user_groups)?;
        package.groups = groups;
        package.auto_import = fastn_document
            .get::<Vec<String>>("fastn#auto-import")?
            .iter()
            .map(|f| fastn_core::AutoImport::from_string(f.as_str()))
            .collect();
        package.fonts = fastn_document.get("fastn#font")?;
        package.sitemap_temp = fastn_document.get("fastn#sitemap")?;
        *self = package;
        Ok(())
    }

    pub(crate) async fn get_and_resolve(
        &self,
        package_root: &camino::Utf8PathBuf,
    ) -> fastn_core::Result<fastn_core::Package> {
        use tokio::io::AsyncWriteExt;

        let file_extract_path = package_root.join("FASTN.ftd");
        if !file_extract_path.exists() {
            std::fs::create_dir_all(package_root)?;
            let fastn_string = self.get_fastn().await?;
            tokio::fs::File::create(&file_extract_path)
                .await?
                .write_all(fastn_string.as_bytes())
                .await?;
        }

        let mut package = self.clone();
        package.resolve(&file_extract_path).await?;
        Ok(package)
    }

    pub fn from_fastn_doc(
        root: &camino::Utf8Path,
        fastn_doc: &ftd::p2::Document,
    ) -> fastn_core::Result<Package> {
        let temp_package: Option<PackageTemp> = fastn_doc.get("fastn#package")?;

        let mut package = match temp_package {
            Some(v) => v.into_package(),
            None => {
                return Err(fastn_core::Error::PackageError {
                    message: "FASTN.ftd does not contain package definition".to_string(),
                })
            }
        };

        // reading dependencies
        let mut deps = {
            let temp_deps: Vec<fastn_core::package::dependency::DependencyTemp> =
                fastn_doc.get("fastn#dependency")?;
            temp_deps
                .into_iter()
                .map(|v| v.into_dependency())
                .collect::<Vec<fastn_core::Result<fastn_core::Dependency>>>()
                .into_iter()
                .collect::<fastn_core::Result<Vec<fastn_core::Dependency>>>()?
        };

        if package.name != fastn_core::FASTN_UI_INTERFACE
            && !deps.iter().any(|dep| {
                dep.implements
                    .contains(&fastn_core::FASTN_UI_INTERFACE.to_string())
            })
        {
            deps.push(fastn_core::Dependency {
                package: fastn_core::Package::new(fastn_core::FASTN_UI_INTERFACE),
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
        package.fastn_path = Some(root.join("FASTN.ftd"));

        package.auto_import = fastn_doc
            .get::<Vec<String>>("fastn#auto-import")?
            .iter()
            .map(|f| fastn_core::AutoImport::from_string(f.as_str()))
            .collect();

        package.ignored_paths = fastn_doc.get::<Vec<String>>("fastn#ignore")?;
        package.fonts = fastn_doc.get("fastn#font")?;
        package.sitemap_temp = fastn_doc.get("fastn#sitemap")?;
        package.dynamic_urls_temp = fastn_doc.get("fastn#dynamic-urls")?;

        // TODO: resolve group dependent packages, there may be imported group from foreign package
        //   We need to make sure to resolve that package as well before moving ahead
        //   Because in `UserGroup::get_identities` we have to resolve identities of a group
        let user_groups: Vec<crate::user_group::UserGroupTemp> =
            fastn_doc.get("fastn#user-group")?;
        let groups = crate::user_group::UserGroupTemp::user_groups(user_groups)?;
        package.groups = groups;

        // validation logic TODO: It should be ordered
        fastn_core::utils::validate_base_url(&package)?;

        if package.import_auto_imports_from_original {
            if let Some(ref original_package) = *package.translation_of {
                if !package.auto_import.is_empty() {
                    return Err(fastn_core::Error::PackageError {
                        message: format!("Can't use `inherit-auto-imports-from-original` along with auto-imports defined for the translation package. Either set `inherit-auto-imports-from-original` to false or remove `fastn.auto-import` from the {package_name}/FASTN.ftd file", package_name=package.name.as_str()),
                    });
                } else {
                    package.auto_import = original_package.auto_import.clone()
                }
            }
        }

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

/// Backend Header is a struct that is used to read and store the backend-header from the FASTN.ftd file
#[derive(serde::Deserialize, Debug, Clone)]
pub struct BackendHeader {
    #[serde(rename = "header-key")]
    pub header_key: String,
    #[serde(rename = "header-value")]
    pub header_value: String,
}
/// PackageTemp is a struct that is used for mapping the `fastn.package` data in FASTN.ftd file. It is
/// not used elsewhere in program, it is immediately converted to `fastn_core::Package` struct during
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
    #[serde(rename = "icon")]
    pub icon: Option<ftd::ImageSrc>,
}

impl PackageTemp {
    pub fn into_package(self) -> Package {
        // TODO: change this method to: `validate(self) -> fastn_core::Result<fastn_core::Package>` and do all
        //       validations in it. Like a package must not have both translation-of and
        //       `translations` set.
        let translation_of = self
            .translation_of
            .as_ref()
            .map(|v| fastn_core::Package::new(v));
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
            fastn_path: None,
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
            apps: vec![],
            icon: self.icon,
        }
    }
}
