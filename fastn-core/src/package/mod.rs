pub mod app;
pub mod dependency;
pub mod package_doc;
pub mod redirects;

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    /// The `versioned` stores the boolean value storing of the fastn package is versioned or not
    pub files: Vec<String>,
    pub versioned: bool,
    pub translation_of: Option<Box<Package>>,
    pub translations: Vec<Package>,
    pub requested_language: Option<String>,
    pub selected_language: Option<String>,
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
    pub fastn_path: Option<fastn_ds::Path>,
    /// `ignored` keeps track of files that are to be ignored by `fastn build`, `fastn sync` etc.
    pub ignored_paths: Vec<String>,
    /// `fonts` keeps track of the fonts used by the package.
    ///
    /// Note that this too is kind of bad design, we will move fonts to `fastn_core::Package` struct soon.
    pub fonts: Vec<fastn_core::Font>,
    pub import_auto_imports_from_original: bool,

    // TODO: this needs to be moved to another fastn + wasm package or would require a redesign
    // if we move this: think about how we can design it mostly in ftd land
    // pub groups: std::collections::BTreeMap<String, crate::user_group::UserGroup>,
    /// sitemap stores the structure of the package. The structure includes sections, sub_sections
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

    /// endpoints for proxy service
    pub endpoints: Vec<fastn_package::old_fastn::EndpointData>,

    /// Installed Apps
    pub apps: Vec<app::App>,

    /// Package Icon
    pub icon: Option<ftd::ImageSrc>,

    /// Redirect URLs
    pub redirects: Option<ftd::Map<String>>,
    pub system: Option<String>,
    pub system_is_confidential: Option<bool>,

    pub lang: Option<Lang>,

    /// Migrations
    pub migrations: Vec<MigrationData>,
}

impl Package {
    pub fn dash_path(&self) -> String {
        format!("-/{}", self.name.trim_matches('/'))
    }

    pub fn new(name: &str) -> fastn_core::Package {
        fastn_core::Package {
            name: name.to_string(),
            files: vec![],
            versioned: false,
            translation_of: None,
            translations: vec![],
            requested_language: None,
            selected_language: None,
            lang: None,
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
            sitemap_temp: None,
            sitemap: None,
            dynamic_urls: None,
            dynamic_urls_temp: None,
            favicon: None,
            endpoints: vec![],
            apps: vec![],
            icon: None,
            redirects: None,
            system: None,
            system_is_confidential: None,
            migrations: vec![],
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn get_font_ftd(&self) -> Option<String> {
        use itertools::Itertools;
        if self.fonts.is_empty() {
            return None;
        }
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
        Some(format!("{font_record}\n{fonts}"))
    }

    pub fn with_base(mut self, base: String) -> fastn_core::Package {
        self.download_base_url = Some(base);
        self
    }

    pub fn current_language_meta(
        &self,
    ) -> ftd::interpreter::Result<fastn_core::library2022::processor::lang_details::LanguageMeta>
    {
        let default_language = "en".to_string();
        let current_language = self
            .requested_language
            .as_ref()
            .unwrap_or(self.selected_language.as_ref().unwrap_or(&default_language));

        let lang = realm_lang::Language::from_2_letter_code(current_language).map_err(
            |realm_lang::Error::InvalidCode { ref found }| ftd::interpreter::Error::ParseError {
                message: found.clone(),
                doc_id: format!("{}/FASTN.ftd", self.name.as_str()),
                line_number: 0,
            },
        )?;

        Ok(
            fastn_core::library2022::processor::lang_details::LanguageMeta {
                id: lang.to_2_letter_code().to_string(),
                id3: lang.to_3_letter_code().to_string(),
                human: lang.human(),
                is_current: true,
            },
        )
    }

    pub fn available_languages_meta(
        &self,
    ) -> ftd::interpreter::Result<Vec<fastn_core::library2022::processor::lang_details::LanguageMeta>>
    {
        let current_language = self.selected_language.clone();
        let mut available_languages = vec![];

        if let Some(ref lang) = self.lang {
            for lang_id in lang.available_languages.keys() {
                let language = realm_lang::Language::from_2_letter_code(lang_id).map_err(
                    |realm_lang::Error::InvalidCode { ref found }| {
                        ftd::interpreter::Error::ParseError {
                            message: found.clone(),
                            doc_id: format!("{}/FASTN.ftd", self.name.as_str()),
                            line_number: 0,
                        }
                    },
                )?;
                available_languages.push(
                    fastn_core::library2022::processor::lang_details::LanguageMeta {
                        id: language.to_2_letter_code().to_string(),
                        id3: language.to_3_letter_code().to_string(),
                        human: language.human(),
                        is_current: is_active_language(
                            &current_language,
                            &language,
                            self.name.as_str(),
                        )?,
                    },
                );
            }
        }

        return Ok(available_languages);

        fn is_active_language(
            current: &Option<String>,
            other: &realm_lang::Language,
            package_name: &str,
        ) -> ftd::interpreter::Result<bool> {
            if let Some(current) = current {
                let current = realm_lang::Language::from_2_letter_code(current.as_str()).map_err(
                    |realm_lang::Error::InvalidCode { ref found }| {
                        ftd::interpreter::Error::ParseError {
                            message: found.clone(),
                            doc_id: format!("{package_name}/FASTN.ftd"),
                            line_number: 0,
                        }
                    },
                )?;
                return Ok(current.eq(other));
            }
            Ok(false)
        }
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
                "{accumulator}{new}\n",
                new = font.to_html(self.name.as_str())
            )
        })
    }

    #[tracing::instrument(skip(self, current_package))]
    pub fn generate_prefix_string(
        &self,
        current_package: &Package,
        with_alias: bool,
    ) -> Option<String> {
        self.auto_import.iter().fold(None, |pre, ai| {
            let mut import_doc_path = ai.path.clone();
            if !with_alias {
                // Check for the aliases and map them to the full path
                for dependency in &self.dependencies {
                    if let Some(alias) = &dependency.alias
                        && (alias.as_str().eq(ai.path.as_str())
                            || ai.path.starts_with(format!("{alias}/").as_str()))
                    {
                        import_doc_path = ai.path.replacen(
                            dependency.alias.as_ref()?.as_str(),
                            dependency.package.name.as_str(),
                            1,
                        );
                    }
                }
            }

            tracing::info!(?import_doc_path, ?ai.alias, ?ai.exposing);

            let import_doc_path = if let Some(provided_via) =
                current_package.dependencies.iter().find_map(|d| {
                    if d.package.name == import_doc_path && d.provided_via.is_some() {
                        d.provided_via.clone()
                    } else {
                        None
                    }
                }) {
                tracing::info!(
                    ?import_doc_path,
                    ?provided_via,
                    "Prefixing auto-import inherited- because it's a provided-via the main package"
                );
                format!("inherited-{import_doc_path}")
            } else {
                import_doc_path
            };

            tracing::info!(?import_doc_path, "import_doc_path has changed");

            Some(format!(
                "{}\n-- import: {}{}{}",
                pre.unwrap_or_default(),
                &import_doc_path,
                match &ai.alias {
                    Some(a) => format!(" as {a}"),
                    None => String::new(),
                },
                if ai.exposing.is_empty() {
                    "".to_string()
                } else {
                    format!("\nexposing: {}\n", ai.exposing.join(","))
                }
            ))
        })
    }

    /// returns the full path of the import from its alias if valid
    /// otherwise returns None
    pub fn get_full_path_from_alias(&self, alias: &str) -> Option<String> {
        let mut full_path: Option<String> = None;

        for dependency in &self.dependencies {
            if let Some(dep_alias) = &dependency.alias
                && dep_alias.as_str().eq(alias)
            {
                full_path = Some(dependency.package.name.clone());
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
    ) -> ftd::ftd2021::p1::Result<String> {
        let mut parts = import_content.splitn(2, '/');
        match (parts.next(), parts.next()) {
            (Some(front), Some(rem)) => {
                // case 1: -- import alias/x..
                // front = alias, rem = x..

                let extended_front = self.get_full_path_from_alias(front);
                match extended_front {
                    Some(ext_front) => Ok(format!("{ext_front}/{rem}")),
                    None => Ok(format!("{front}/{rem}")),
                }
            }
            (Some(front), None) => {
                // case 2: -- import alias
                // front = alias

                let extended_front = self.get_full_path_from_alias(front);
                match extended_front {
                    Some(ext_front) => match with_alias {
                        true => Ok(format!("{ext_front} as {front}")),
                        false => Ok(ext_front),
                    },
                    None => Ok(front.to_string()),
                }
            }
            _ => {
                // Throw error for unknown type-1 import
                Err(ftd::ftd2021::p1::Error::ParseError {
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
    ) -> ftd::ftd2021::p1::Result<String> {
        let mut parts = import_content.splitn(2, " as ");

        match (parts.next(), parts.next()) {
            (Some(front), Some(alias)) => {
                // case 1: -- import alias/x.. as alias_2
                // case 2: -- import alias as alias_2
                // front = alias/x or alias, alias = alias_2

                let extended_front =
                    self.fix_aliased_import_type1(front, id, line_number, false)?;
                Ok(format!("{extended_front} as {alias}"))
            }
            _ => {
                // Throw error for unknown type-2 import
                Err(ftd::ftd2021::p1::Error::ParseError {
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
    pub fn fix_imports_in_body(&self, body: &str, id: &str) -> ftd::ftd2021::p1::Result<String> {
        let mut new_body = String::new();
        let mut ln = 1;

        for line in body.lines() {
            let line_string = line.trim();

            let final_line = {
                if line_string.starts_with("-- import") {
                    // Split [-- import | content]
                    let import_tokens: Vec<&str> = line_string.split(':').collect();
                    if import_tokens.len() <= 1 {
                        return Err(ftd::ftd2021::p1::Error::ParseError {
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

    pub fn get_prefixed_body(
        &self,
        current_package: &Package,
        body: &str,
        id: &str,
        with_alias: bool,
    ) -> String {
        if id.contains("FPM/") {
            return body.to_string();
        };
        match self.generate_prefix_string(current_package, with_alias) {
            Some(s) => {
                let t = format!("{s}\n\n{body}");
                self.fix_imports_in_body(t.as_str(), id).ok().unwrap_or(t)
            }
            None => self
                .fix_imports_in_body(body, id)
                .ok()
                .unwrap_or(body.to_string()),
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
                url = format!("{url}/");
            }

            return format!(
                "\n<link rel=\"canonical\" href=\"{url}\" /><meta property=\"og:url\" content=\"{url}\" />"
            );
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
                let url = if url.ends_with('/') {
                    url
                } else {
                    format!("{url}/")
                };
                // Ignore the fastn document as that path won't exist in the reference website
                format!(
                    "\n<link rel=\"canonical\" href=\"{url}{path}\" /><meta property=\"og:url\" content=\"{url}{path}\" />"
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

    // pub(crate) async fn get_fastn(&self) -> fastn_core::Result<String> {
    //     crate::http::construct_url_and_get_str(format!("{}/FASTN.ftd", self.name).as_str()).await
    // }

    pub async fn resolve(
        &mut self,
        fastn_path: &fastn_ds::Path,
        ds: &fastn_ds::DocumentStore,
        session_id: &Option<String>,
    ) -> fastn_core::Result<()> {
        let fastn_document = {
            let doc = ds.read_to_string(fastn_path, session_id).await?;
            let lib = fastn_core::FastnLibrary::default();
            match fastn_core::doc::parse_ftd("fastn", doc.as_str(), &lib) {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!(
                        msg = "failed to pare FASTN.ftd file",
                        path = fastn_path.to_string()
                    );
                    return Err(fastn_core::Error::PackageError {
                        message: format!("failed to parse FASTN.ftd: {:?}", &e),
                    });
                }
            }
        };
        let mut package = {
            let temp_package: fastn_package::old_fastn::PackageTemp =
                fastn_document.get("fastn#package")?;
            temp_package.into_package()
        };

        let url_mappings = {
            let url_mappings_temp: Option<redirects::UrlMappingsTemp> =
                fastn_document.get("fastn#url-mappings")?;
            if let Some(url_mappings) = url_mappings_temp {
                let result = url_mappings
                    .url_mappings_from_body()
                    .map_err(|e| fastn_core::Error::GenericError(e.to_string()))?;
                Some(result)
            } else {
                None
            }
        };

        if let Some(url_mappings) = url_mappings {
            package.redirects = Some(url_mappings.redirects);
            package.endpoints = url_mappings.endpoints;
        }

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

        package.auto_import = fastn_document
            .get::<Vec<fastn_core::package::dependency::AutoImportTemp>>("fastn#auto-import")?
            .into_iter()
            .map(|f| f.into_auto_import())
            .collect();

        // Todo: Add `package.files` and fix `fs_fetch_by_id` to check if file is present
        package.fonts = fastn_document.get("fastn#font")?;
        package.sitemap_temp = fastn_document.get("fastn#sitemap")?;

        package.migrations = get_migration_data(&fastn_document)?;
        *self = package;
        Ok(())
    }

    #[cfg(not(feature = "use-config-json"))]
    #[tracing::instrument(skip(self, ds))]
    pub(crate) async fn get_and_resolve(
        &self,
        package_root: &fastn_ds::Path,
        ds: &fastn_ds::DocumentStore,
        session_id: &Option<String>,
    ) -> fastn_core::Result<fastn_core::Package> {
        let file_extract_path = package_root.join("FASTN.ftd");
        let mut package = self.clone();
        package.resolve(&file_extract_path, ds, session_id).await?;
        Ok(package)
    }

    pub fn from_fastn_doc(
        ds: &fastn_ds::DocumentStore,
        fastn_doc: &ftd::ftd2021::p2::Document,
    ) -> fastn_core::Result<Package> {
        let temp_package: Option<fastn_package::old_fastn::PackageTemp> =
            fastn_doc.get("fastn#package")?;

        let mut package = match temp_package {
            Some(v) => v.into_package(),
            None => {
                return Err(fastn_core::Error::PackageError {
                    message: "FASTN.ftd does not contain package definition".to_string(),
                });
            }
        };

        let url_mappings = {
            let url_mappings_temp: Option<redirects::UrlMappingsTemp> =
                fastn_doc.get("fastn#url-mappings")?;
            if let Some(url_mappings) = url_mappings_temp {
                let result = url_mappings
                    .url_mappings_from_body()
                    .map_err(|e| fastn_core::Error::GenericError(e.to_string()))?;
                Some(result)
            } else {
                None
            }
        };

        if let Some(url_mappings) = url_mappings {
            package.redirects = Some(url_mappings.redirects);
            package.endpoints = url_mappings.endpoints;
        }

        // reading dependencies
        let deps = {
            let temp_deps: Vec<fastn_core::package::dependency::DependencyTemp> =
                fastn_doc.get("fastn#dependency")?;

            temp_deps
                .into_iter()
                .map(|v| v.into_dependency())
                .collect::<Vec<fastn_core::Result<fastn_core::Dependency>>>()
                .into_iter()
                .collect::<fastn_core::Result<Vec<fastn_core::Dependency>>>()?
        };

        // setting dependencies
        package.dependencies = deps;
        // package.resolve_system_dependencies()?;

        package.fastn_path = Some(ds.root().join("FASTN.ftd"));

        package.auto_import = fastn_doc
            .get::<Vec<fastn_core::package::dependency::AutoImportTemp>>("fastn#auto-import")?
            .into_iter()
            .map(|f| f.into_auto_import())
            .collect();

        if let Some(ref system_alias) = package.system {
            if package.system_is_confidential.unwrap_or(true) {
                return fastn_core::usage_error(format!(
                    "system-is-confidential is needed for system package {} and currently only false is supported.",
                    package.name
                ));
            }
            package.auto_import.push(fastn_core::AutoImport {
                path: package.name.clone(),
                alias: Some(system_alias.clone()),
                exposing: vec![],
            });
        }

        package.auto_import_language(None, None)?;
        package.ignored_paths = fastn_doc.get::<Vec<String>>("fastn#ignore")?;
        package.fonts = fastn_doc.get("fastn#font")?;
        package.sitemap_temp = fastn_doc.get("fastn#sitemap")?;
        package.dynamic_urls_temp = fastn_doc.get("fastn#dynamic-urls")?;
        package.migrations = get_migration_data(fastn_doc)?;

        // validation logic TODO: It should be ordered
        fastn_core::utils::validate_base_url(&package)?;

        if package.import_auto_imports_from_original
            && let Some(ref original_package) = package.translation_of
        {
            if package.auto_import.is_empty() {
                package
                    .auto_import
                    .clone_from(&original_package.auto_import)
            } else {
                return Err(fastn_core::Error::PackageError {
                    message: format!(
                        "Can't use `inherit-auto-imports-from-original` along with auto-imports defined for the translation package. Either set `inherit-auto-imports-from-original` to false or remove `fastn.auto-import` from the {package_name}/FASTN.ftd file",
                        package_name = package.name.as_str()
                    ),
                });
            }
        }

        Ok(package)
    }

    pub fn auto_import_language(
        &mut self,
        req_lang: Option<String>,
        main_package_selected_language: Option<String>,
    ) -> fastn_core::Result<()> {
        let lang = if let Some(lang) = &self.lang {
            lang
        } else {
            return Ok(());
        };
        let mut lang_module_path_with_language = None;

        if let Some(request_lang) = req_lang.as_ref() {
            lang_module_path_with_language = lang
                .available_languages
                .get(request_lang)
                .map(|module| (module, request_lang.to_string()));
        }

        if lang_module_path_with_language.is_none()
            && !main_package_selected_language.eq(&req_lang)
            && let Some(main_package_selected_language) = main_package_selected_language.as_ref()
        {
            lang_module_path_with_language = lang
                .available_languages
                .get(main_package_selected_language)
                .map(|module| (module, main_package_selected_language.to_string()));
        }

        if lang_module_path_with_language.is_none() {
            lang_module_path_with_language = lang
                .available_languages
                .get(&lang.default_lang)
                .map(|v| (v, lang.default_lang.to_string()));
        }

        let (lang_module_path, language) = match lang_module_path_with_language {
            Some(v) => v,
            None => {
                return fastn_core::usage_error(format!(
                    "Module corresponding to `default-language: {}` is not provided in FASTN.ftd of {}",
                    lang.default_lang, self.name
                ));
            }
        };

        self.auto_import.push(fastn_core::AutoImport {
            path: lang_module_path.to_string(),
            alias: Some("lang".to_string()),
            exposing: vec![],
        });

        self.requested_language = req_lang;
        self.selected_language = Some(language);
        Ok(())
    }
}

pub(crate) fn get_migration_data(
    doc: &ftd::ftd2021::p2::Document,
) -> fastn_core::Result<Vec<MigrationData>> {
    let migration_data = doc.get::<Vec<MigrationDataTemp>>("fastn#migration")?;
    let mut migrations = vec![];
    for (number, migration) in migration_data.into_iter().rev().enumerate() {
        migrations.push(migration.into_migration(number as i64));
    }
    Ok(migrations)
}

#[derive(Debug, Clone)]
pub struct Lang {
    pub default_lang: String,
    pub available_languages: std::collections::HashMap<String, String>,
}

trait PackageTempIntoPackage {
    fn into_package(self) -> Package;
}

impl PackageTempIntoPackage for fastn_package::old_fastn::PackageTemp {
    fn into_package(self) -> Package {
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

        // Currently supported languages
        // English - en
        // Hindi- hi
        // Chinese - zh
        // Spanish - es
        // Arabic - ar
        // Portuguese - pt
        // Russian - ru
        // French - fr
        // German - de
        // Japanese - ja
        // Bengali - bn
        // Urdu - ur
        // Indonesian - id
        // Turkish - tr
        // Vietnamese - vi
        // Italian - it
        // Polish - pl
        // Thai - th
        // Dutch - nl
        // Korean - ko
        let lang = if let Some(default_lang) = &self.default_language {
            let mut available_languages = std::collections::HashMap::new();

            if let Some(lang_en) = self.translation_en {
                available_languages.insert("en".to_string(), lang_en);
            }

            if let Some(lang_hi) = self.translation_hi {
                available_languages.insert("hi".to_string(), lang_hi);
            }

            if let Some(lang_zh) = self.translation_zh {
                available_languages.insert("zh".to_string(), lang_zh);
            }

            if let Some(lang_es) = self.translation_es {
                available_languages.insert("es".to_string(), lang_es);
            }

            if let Some(lang_ar) = self.translation_ar {
                available_languages.insert("ar".to_string(), lang_ar);
            }

            if let Some(lang_pt) = self.translation_pt {
                available_languages.insert("pt".to_string(), lang_pt);
            }

            if let Some(lang_ru) = self.translation_ru {
                available_languages.insert("ru".to_string(), lang_ru);
            }

            if let Some(lang_fr) = self.translation_fr {
                available_languages.insert("fr".to_string(), lang_fr);
            }

            if let Some(lang_de) = self.translation_de {
                available_languages.insert("de".to_string(), lang_de);
            }

            if let Some(lang_ja) = self.translation_ja {
                available_languages.insert("ja".to_string(), lang_ja);
            }

            if let Some(lang_bn) = self.translation_bn {
                available_languages.insert("bn".to_string(), lang_bn);
            }

            if let Some(lang_ur) = self.translation_ur {
                available_languages.insert("ur".to_string(), lang_ur);
            }

            if let Some(lang_id) = self.translation_id {
                available_languages.insert("id".to_string(), lang_id);
            }

            if let Some(lang_tr) = self.translation_tr {
                available_languages.insert("tr".to_string(), lang_tr);
            }

            if let Some(lang_vi) = self.translation_vi {
                available_languages.insert("vi".to_string(), lang_vi);
            }

            if let Some(lang_it) = self.translation_it {
                available_languages.insert("it".to_string(), lang_it);
            }

            if let Some(lang_pl) = self.translation_pl {
                available_languages.insert("pl".to_string(), lang_pl);
            }

            if let Some(lang_th) = self.translation_th {
                available_languages.insert("th".to_string(), lang_th);
            }

            if let Some(lang_nl) = self.translation_nl {
                available_languages.insert("nl".to_string(), lang_nl);
            }

            if let Some(lang_ko) = self.translation_ko {
                available_languages.insert("ko".to_string(), lang_ko);
            }

            Some(Lang {
                default_lang: default_lang.to_string(),
                available_languages,
            })
        } else {
            None
        };

        Package {
            name: self.name.clone(),
            files: vec![],
            versioned: self.versioned,
            translation_of: translation_of.map(Box::new),
            translations,
            requested_language: None,
            selected_language: None,
            lang,
            about: self.about,
            zip: self.zip,
            download_base_url: self.download_base_url.or(Some(self.name)),
            translation_status_summary: None,
            canonical_url: self.canonical_url,
            dependencies: vec![],
            auto_import: vec![],
            fastn_path: None,
            ignored_paths: vec![],
            fonts: vec![],
            import_auto_imports_from_original: self.import_auto_imports_from_original,
            sitemap: None,
            sitemap_temp: None,
            dynamic_urls: None,
            dynamic_urls_temp: None,
            favicon: self.favicon,
            endpoints: self.endpoint,
            apps: vec![],
            icon: self.icon,
            redirects: None,
            system: self.system,
            system_is_confidential: self.system_is_confidential,
            migrations: vec![],
        }
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct MigrationData {
    pub number: i64,
    pub name: String,
    pub content: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct MigrationDataTemp {
    pub name: String,
    pub content: String,
}

impl MigrationDataTemp {
    pub(crate) fn into_migration(self, number: i64) -> MigrationData {
        MigrationData {
            number,
            name: self.name,
            content: self.content,
        }
    }
}
