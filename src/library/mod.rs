mod cr_meta;
pub(crate) mod document;
mod fetch_file;
mod fpm_dot_ftd;
pub(crate) mod full_sitemap;
mod get_data;
mod get_version_data;
pub(crate) mod http;
mod include;
mod package_tree;
mod sitemap;
mod sqlite;
mod toc;

pub use document::convert_to_document_id;
pub use full_sitemap::KeyValueData;

#[derive(Debug)]
pub struct Library {
    pub config: fpm::Config,
    /// If the current module being parsed is a markdown file, `.markdown` contains the name and
    /// content of that file
    pub markdown: Option<(String, String)>,
    pub document_id: String,
    pub translated_data: fpm::TranslationData,
    /// Hashmap that contains the information about the assets document for the current build
    /// It'll contain a map of <package_name> corresponding to the asset doc for that package
    pub asset_documents: std::collections::HashMap<String, String>,
    pub base_url: String,
}

impl Library {
    // TODO: async
    pub async fn get_with_result(
        &self,
        name: &str,
        packages: &mut Vec<fpm::Package>,
    ) -> ftd::p1::Result<String> {
        match self.get(name, packages).await {
            Some(v) => Ok(v),
            None => ftd::p2::utils::e2(format!("library not found: {}", name), "", 0),
        }
    }

    pub async fn get(&self, name: &str, packages: &mut Vec<fpm::Package>) -> Option<String> {
        if name == "fpm" {
            packages.push(packages.last()?.clone());
            return Some(fpm_dot_ftd::get(self).await);
        }
        if name == "fpm-lib" {
            packages.push(packages.last()?.clone());
            return Some(fpm::fpm_lib_ftd().to_string());
        }

        return get_for_package(name, packages, self).await;

        async fn get_for_package(
            name: &str,
            packages: &mut Vec<fpm::Package>,
            lib: &fpm::Library,
        ) -> Option<String> {
            let package = packages.last()?;
            if name.starts_with(package.name.as_str()) {
                if let Some(r) = get_data_from_package(name, package, lib).await {
                    packages.push(packages.last()?.clone());
                    return Some(r);
                }
            }

            for (alias, package) in package.to_owned().aliases() {
                if name.starts_with(alias) {
                    let package = lib.config.resolve_package(package).await.ok()?;
                    if let Some(r) = get_data_from_package(
                        name.replacen(&alias, &package.name, 1).as_str(),
                        &package,
                        lib,
                    )
                    .await
                    {
                        packages.push(package);
                        return Some(r);
                    }
                }
            }

            let translation_of = match package.translation_of.as_ref() {
                Some(translation_of) => translation_of.to_owned(),
                None => return None,
            };

            let name = name.replacen(package.name.as_str(), translation_of.name.as_str(), 1);
            if name.starts_with(translation_of.name.as_str()) {
                if let Some(r) = get_data_from_package(name.as_str(), &translation_of, lib).await {
                    packages.push(packages.last()?.clone());
                    return Some(r);
                }
            }

            for (alias, package) in translation_of.aliases() {
                if name.starts_with(alias) {
                    if let Some(r) = get_data_from_package(
                        name.replacen(&alias, &package.name, 1).as_str(),
                        package,
                        lib,
                    )
                    .await
                    {
                        packages.push(package.clone());
                        return Some(r);
                    }
                }
            }

            None
        }

        fn get_file_from_location(base_path: &camino::Utf8PathBuf, name: &str) -> Option<String> {
            let os_name = name
                .trim_start_matches('/')
                .trim_end_matches('/')
                .replace('/', std::path::MAIN_SEPARATOR.to_string().as_str());
            if let Ok(v) = std::fs::read_to_string(base_path.join(format!("{}.ftd", os_name))) {
                return Some(v);
            }
            if let Ok(v) = std::fs::read_to_string(base_path.join(os_name).join("index.ftd")) {
                return Some(v);
            }
            None
        }

        async fn get_data_from_package(
            name: &str,
            package: &fpm::Package,
            lib: &Library,
        ) -> Option<String> {
            let path = lib.config.get_root_for_package(package);
            fpm::Config::download_required_file(&lib.config.root, name, package)
                .await
                .ok()?;
            // Explicit check for the current package.
            if name.starts_with(&package.name.as_str()) {
                let new_name = name.replacen(&package.name.as_str(), "", 1);
                if new_name.as_str().trim_start_matches('/') == "assets" {
                    // Virtual document for getting the assets
                    if let Some(asset_doc) = lib.asset_documents.get(&package.name.clone()) {
                        return Some(asset_doc.to_owned());
                    } else {
                        panic!("Expected assets doc to be initialized")
                    }
                    // return Some(package.get_assets_doc());
                } else if let Some(body) = get_file_from_location(&path, new_name.as_str()) {
                    return Some(package.get_prefixed_body(body.as_str(), name, true));
                }
            }
            None
        }
    }

    pub async fn process<'a>(
        &'a self,
        section: &ftd::p1::Section,
        doc: &'a ftd::p2::TDoc<'a>,
    ) -> ftd::p1::Result<ftd::Value> {
        match section
            .header
            .str(doc.name, section.line_number, "$processor$")?
        {
            // These processors are implemented both in Rust and Python
            "http" => fpm::library::http::processor(section, doc, &self.config).await,
            "package-query" => fpm::library::sqlite::processor(section, doc, &self.config).await,
            "fetch-file" => fpm::library::fetch_file::processor(section, doc, &self.config).await,
            "package-tree" => {
                fpm::library::package_tree::processor(section, doc, &self.config).await
            }
            "get-version-data" => {
                fpm::library::get_version_data::processor(
                    section,
                    doc,
                    &self.config,
                    self.document_id.as_str(),
                    self.base_url.as_str(),
                )
                .await
            }
            "document-name" => document::processor::document_name(section, doc, &self.config).await,
            _ => process_sync(&self.config, section, self.document_id.as_str(), doc),
        }
    }
}

/// process_sync implements a bunch of processors that are called from Python. We want sync
/// API to expose to outside world and async functions do not work so well with them.
pub fn process_sync<'a>(
    config: &fpm::Config,
    section: &ftd::p1::Section,
    document_id: &str,
    doc: &'a ftd::p2::TDoc<'a>,
) -> ftd::p1::Result<ftd::Value> {
    match section
        .header
        .str(doc.name, section.line_number, "$processor$")?
    {
        "toc" => fpm::library::toc::processor(section, doc, config),
        "include" => fpm::library::include::processor(section, doc, config),
        "get-data" => fpm::library::get_data::processor(section, doc, config),
        "sitemap" => fpm::library::sitemap::processor(section, doc, config),
        "full-sitemap" => fpm::library::full_sitemap::processor(section, doc, config),
        "document-readers" => {
            fpm::library::sitemap::document_readers(section, document_id, doc, config)
        }
        "document-writers" => {
            fpm::library::sitemap::document_writers(section, document_id, doc, config)
        }
        "user-groups" => fpm::user_group::processor::user_groups(section, doc, config),
        "user-group-by-id" => fpm::user_group::processor::user_group_by_id(section, doc, config),
        "package-query" => fpm::library::sqlite::processor_(section, doc, config),
        "fetch-file" => fpm::library::fetch_file::processor_sync(section, doc, config),
        "package-tree" => fpm::library::package_tree::processor_sync(section, doc, config),
        "document-id" => document::processor::document_id(section, doc, config),
        "document-full-id" => document::processor::document_full_id(section, doc, config),
        "document-suffix" => document::processor::document_suffix(section, doc, config),
        "package-id" => Ok(ftd::Value::String {
            text: config.package.name.clone(),
            source: ftd::TextSource::Default,
        }),
        "get-identities" => fpm::user_group::processor::get_identities(section, doc, config),
        "request-data" => fpm::library::http::request_data_processor(section, doc, config),

        t => Err(ftd::p1::Error::NotFound {
            doc_id: document_id.to_string(),
            line_number: section.line_number,
            key: format!("FPM-Error: No such processor: {}", t),
        }),
    }
}

#[derive(Debug)]
pub struct Library2 {
    pub config: fpm::Config,
    /// If the current module being parsed is a markdown file, `.markdown` contains the name and
    /// content of that file
    pub markdown: Option<(String, String)>,
    pub document_id: String,
    pub translated_data: fpm::TranslationData,
    pub base_url: String,
    pub packages_under_process: Vec<String>,
}

impl Library2 {
    pub(crate) async fn push_package_under_process(
        &mut self,
        package: &fpm::Package,
    ) -> ftd::p1::Result<()> {
        self.packages_under_process.push(package.name.to_string());
        if self
            .config
            .all_packages
            .borrow()
            .contains_key(package.name.as_str())
        {
            return Ok(());
        }

        let package =
            self.config
                .resolve_package(package)
                .await
                .map_err(|_| ftd::p1::Error::ParseError {
                    message: format!("Cannot resolve the package: {}", package.name),
                    doc_id: self.document_id.to_string(),
                    line_number: 0,
                })?;

        self.config
            .all_packages
            .borrow_mut()
            .insert(package.name.to_string(), package);
        Ok(())
    }

    pub(crate) fn get_current_package(&self) -> ftd::p1::Result<fpm::Package> {
        let current_package_name =
            self.packages_under_process
                .last()
                .ok_or_else(|| ftd::p1::Error::ParseError {
                    message: "The processing document stack is empty".to_string(),
                    doc_id: "".to_string(),
                    line_number: 0,
                })?;

        self.config
            .all_packages
            .borrow()
            .get(current_package_name)
            .map(|p| p.to_owned())
            .ok_or_else(|| ftd::p1::Error::ParseError {
                message: format!("Can't find current package: {}", current_package_name),
                doc_id: "".to_string(),
                line_number: 0,
            })
    }
    // TODO: async
    pub async fn get_with_result(&mut self, name: &str) -> ftd::p1::Result<String> {
        match self.get(name).await {
            Some(v) => Ok(v),
            None => ftd::p2::utils::e2(format!("library not found: {}", name), "", 0),
        }
    }

    pub async fn get(&mut self, name: &str) -> Option<String> {
        if name == "fpm" {
            self.packages_under_process
                .push(self.get_current_package().ok()?.name);
            return Some(fpm_dot_ftd::get2(self).await);
        }
        if name == "fpm-lib" {
            self.packages_under_process
                .push(self.get_current_package().ok()?.name);
            return Some(fpm::fpm_lib_ftd().to_string());
        }

        return get_for_package(format!("{}/", name.trim_end_matches('/')).as_str(), self).await;

        async fn get_for_package(name: &str, lib: &mut fpm::Library2) -> Option<String> {
            let package = lib.get_current_package().ok()?;
            if name.starts_with(package.name.as_str()) {
                if let Some(r) = get_data_from_package(name, &package, lib).await {
                    return Some(r);
                }
            }

            for (alias, package) in package.aliases() {
                if name.starts_with(alias) {
                    if let Some(r) = get_data_from_package(
                        name.replacen(&alias, &package.name, 1).as_str(),
                        package,
                        lib,
                    )
                    .await
                    {
                        return Some(r);
                    }
                }
            }

            let translation_of = match package.translation_of.as_ref() {
                Some(translation_of) => translation_of.to_owned(),
                None => return None,
            };

            let name = name.replacen(package.name.as_str(), translation_of.name.as_str(), 1);
            if name.starts_with(translation_of.name.as_str()) {
                if let Some(r) = get_data_from_package(name.as_str(), &translation_of, lib).await {
                    return Some(r);
                }
            }

            for (alias, package) in translation_of.aliases() {
                if name.starts_with(alias) {
                    if let Some(r) = get_data_from_package(
                        name.replacen(&alias, &package.name, 1).as_str(),
                        package,
                        lib,
                    )
                    .await
                    {
                        return Some(r);
                    }
                }
            }

            None
        }

        #[allow(clippy::await_holding_refcell_ref)]
        async fn get_data_from_package(
            name: &str,
            package: &fpm::Package,
            lib: &mut Library2,
        ) -> Option<String> {
            lib.push_package_under_process(package).await.ok()?;
            let packages = lib.config.all_packages.borrow();
            let package = packages.get(package.name.as_str()).unwrap_or(package);
            // Explicit check for the current package.
            if !name.starts_with(&package.name.as_str()) {
                return None;
            }
            let new_name = name.replacen(&package.name.as_str(), "", 1);
            let (file_path, data) = package.resolve_by_id(new_name.as_str(), None).await.ok()?;
            if !file_path.ends_with(".ftd") {
                return None;
            }
            String::from_utf8(data)
                .ok()
                .map(|body| package.get_prefixed_body(body.as_str(), name, true))
        }
    }

    /// checks if the current processor is a lazy processor
    /// or not
    ///
    /// for more details
    ///
    /// visit www.fpm.dev/glossary/#lazy-processor
    pub fn is_lazy_processor(
        &self,
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<bool> {
        Ok(section
            .header
            .str(doc.name, section.line_number, "$processor$")?
            .eq("page-headings"))
    }

    pub async fn process<'a>(
        &'a self,
        section: &ftd::p1::Section,
        doc: &'a ftd::p2::TDoc<'a>,
    ) -> ftd::p1::Result<ftd::Value> {
        match section
            .header
            .str(doc.name, section.line_number, "$processor$")?
        {
            // "toc" => fpm::library::toc::processor(section, doc),
            "http" => fpm::library::http::processor(section, doc, &self.config).await,
            "package-query" => fpm::library::sqlite::processor(section, doc, &self.config).await,
            "toc" => fpm::library::toc::processor(section, doc, &self.config),
            "include" => fpm::library::include::processor(section, doc, &self.config),
            "get-data" => fpm::library::get_data::processor(section, doc, &self.config),
            "sitemap" => fpm::library::sitemap::processor(section, doc, &self.config),
            "full-sitemap" => fpm::library::full_sitemap::processor(section, doc, &self.config),
            "user-groups" => fpm::user_group::processor::user_groups(section, doc, &self.config),
            "document-readers" => fpm::library::sitemap::document_readers(
                section,
                self.document_id.as_str(),
                doc,
                &self.config,
            ),
            "document-writers" => fpm::library::sitemap::document_writers(
                section,
                self.document_id.as_str(),
                doc,
                &self.config,
            ),
            "user-group-by-id" => {
                fpm::user_group::processor::user_group_by_id(section, doc, &self.config)
            }
            "get-identities" => {
                fpm::user_group::processor::get_identities(section, doc, &self.config)
            }
            "document-id" => document::processor::document_id(section, doc, &self.config),
            "document-full-id" => document::processor::document_full_id(section, doc, &self.config),
            "document-name" => document::processor::document_name(section, doc, &self.config).await,
            "document-suffix" => document::processor::document_suffix(section, doc, &self.config),
            "package-id" => Ok(ftd::Value::String {
                text: self.config.package.name.clone(),
                source: ftd::TextSource::Default,
            }),
            "package-tree" => {
                fpm::library::package_tree::processor(section, doc, &self.config).await
            }
            "fetch-file" => fpm::library::fetch_file::processor(section, doc, &self.config).await,
            "get-version-data" => {
                fpm::library::get_version_data::processor(
                    section,
                    doc,
                    &self.config,
                    self.document_id.as_str(),
                    self.base_url.as_str(),
                )
                .await
            }
            "cr-meta" => fpm::library::cr_meta::processor(section, doc, &self.config).await,
            "request-data" => {
                fpm::library::http::request_data_processor(section, doc, &self.config)
            }
            t => Err(ftd::p1::Error::NotFound {
                doc_id: self.document_id.to_string(),
                line_number: section.line_number,
                key: format!("FPM-Error: No such processor: {}", t),
            }),
        }
    }
}

#[derive(Default)]
pub struct FPMLibrary {}

impl FPMLibrary {
    pub fn get(&self, name: &str, _doc: &ftd::p2::TDoc) -> Option<String> {
        if name == "fpm" {
            Some(format!(
                "{}\n\n-- optional package-data package:\n",
                fpm::fpm_ftd()
            ))
        } else if name == "env" {
            Some(fpm::get_env_ftd_file())
        } else {
            // Note: currently we do not allow users to import other modules from FPM.ftd
            eprintln!("FPM.ftd can only import `fpm` and `env` module");
            None
        }
    }

    pub fn process(
        &self,
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<ftd::Value> {
        ftd::p2::utils::unknown_processor_error(
            format!("unimplemented for section {:?} and doc {:?}", section, doc),
            doc.name.to_string(),
            section.line_number,
        )
    }

    pub fn get_with_result(&self, name: &str, doc: &ftd::p2::TDoc) -> ftd::p1::Result<String> {
        match self.get(name, doc) {
            Some(v) => Ok(v),
            None => ftd::p2::utils::e2(format!("library not found: {}", name), "", 0),
        }
    }
}
