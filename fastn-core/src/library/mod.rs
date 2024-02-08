pub(crate) mod document;
pub(crate) mod fastn_dot_ftd;
pub use document::convert_to_document_id;
pub(crate) mod toc;
pub use fastn_core::Library2022;

#[derive(Debug)]
pub struct Library {
    pub config: fastn_core::RequestConfig,
    /// If the current module being parsed is a markdown file, `.markdown` contains the name and
    /// content of that file
    pub markdown: Option<(String, String)>,
    pub document_id: String,
    pub translated_data: fastn_core::TranslationData,
    /// Hashmap that contains the information about the assets document for the current build
    /// It'll contain a map of <package_name> corresponding to the asset doc for that package
    pub asset_documents: std::collections::HashMap<String, String>,
    pub base_url: String,
}

impl Library {
    pub async fn get_with_result(
        &self,
        name: &str,
        packages: &mut Vec<fastn_core::Package>,
    ) -> ftd::ftd2021::p1::Result<String> {
        match self.get(name, packages).await {
            Some(v) => Ok(v),
            None => ftd::ftd2021::p2::utils::e2(format!("library not found: {}", name), "", 0),
        }
    }

    pub async fn get(&self, name: &str, packages: &mut Vec<fastn_core::Package>) -> Option<String> {
        if name == "fastn" {
            packages.push(packages.last()?.clone());
            return Some(fastn_dot_ftd::get(self).await);
        }
        if name == "fastn-lib" {
            packages.push(packages.last()?.clone());
            return Some(fastn_core::fastn_lib_ftd().to_string());
        }

        return get_for_package(name, packages, self).await;

        async fn get_for_package(
            name: &str,
            packages: &mut Vec<fastn_core::Package>,
            lib: &fastn_core::Library,
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
                    let package = lib.config.config.resolve_package(package).await.ok()?;
                    if let Some(r) = get_data_from_package(
                        name.replacen(alias, &package.name, 1).as_str(),
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
                        name.replacen(alias, &package.name, 1).as_str(),
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

        async fn get_file_from_location(
            base_path: &fastn_ds::Path,
            name: &str,
            ds: &fastn_ds::DocumentStore,
        ) -> Option<String> {
            let os_name = name
                .trim_start_matches('/')
                .trim_end_matches('/')
                .replace('/', std::path::MAIN_SEPARATOR.to_string().as_str());
            if let Ok(v) = ds
                .read_to_string(&base_path.join(format!("{}.ftd", os_name)))
                .await
            {
                return Some(v);
            }
            if let Ok(v) = ds
                .read_to_string(&base_path.join(os_name).join("index.ftd"))
                .await
            {
                return Some(v);
            }
            None
        }

        async fn get_data_from_package(
            name: &str,
            package: &fastn_core::Package,
            lib: &Library,
        ) -> Option<String> {
            let path = lib.config.config.get_root_for_package(package);
            fastn_core::Config::download_required_file(
                &lib.config.config.ds.root(),
                name,
                package,
                &lib.config.config.ds,
            )
            .await
            .ok()?;
            // Explicit check for the current package.
            if name.starts_with(package.name.as_str()) {
                let new_name = name.replacen(package.name.as_str(), "", 1);
                if new_name.as_str().trim_start_matches('/') == "assets" {
                    // Virtual document for getting the assets
                    if let Some(asset_doc) = lib.asset_documents.get(&package.name.clone()) {
                        return Some(asset_doc.to_owned());
                    } else {
                        panic!("Expected assets doc to be initialized")
                    }
                    // return Some(package.get_assets_doc());
                } else if let Some(body) =
                    get_file_from_location(&path, new_name.as_str(), &lib.config.config.ds).await
                {
                    return Some(package.get_prefixed_body(body.as_str(), name, true));
                }
            }
            None
        }
    }
}

#[derive(Debug)]
pub struct Library2 {
    pub config: fastn_core::RequestConfig,
    /// If the current module being parsed is a markdown file, `.markdown` contains the name and
    /// content of that file
    pub markdown: Option<(String, String)>,
    pub document_id: String,
    pub translated_data: fastn_core::TranslationData,
    pub base_url: String,
    pub packages_under_process: Vec<String>,
}

impl Library2 {
    pub(crate) fn push_package_under_process(
        &mut self,
        package: &fastn_core::Package,
    ) -> ftd::ftd2021::p1::Result<()> {
        self.packages_under_process.push(package.name.to_string());
        if !self
            .config
            .config
            .all_packages
            .contains_key(package.name.as_str())
        {
            return Err(ftd::ftd2021::p1::Error::ParseError {
                message: format!("Cannot resolve the package: {}", package.name),
                doc_id: self.document_id.to_string(),
                line_number: 0,
            });
        }

        Ok(())
    }

    pub(crate) fn get_current_package(&self) -> ftd::ftd2021::p1::Result<fastn_core::Package> {
        let current_package_name = self.packages_under_process.last().ok_or_else(|| {
            ftd::ftd2021::p1::Error::ParseError {
                message: "The processing document stack is empty".to_string(),
                doc_id: "".to_string(),
                line_number: 0,
            }
        })?;

        self.config
            .config
            .all_packages
            .get(current_package_name)
            .map(|p| p.to_owned())
            .ok_or_else(|| ftd::ftd2021::p1::Error::ParseError {
                message: format!("Can't find current package: {}", current_package_name),
                doc_id: "".to_string(),
                line_number: 0,
            })
    }
    // TODO: async
    pub async fn get_with_result(&mut self, name: &str) -> ftd::ftd2021::p1::Result<String> {
        match self.get(name).await {
            Some(v) => Ok(v),
            None => ftd::ftd2021::p2::utils::e2(format!("library not found: {}", name), "", 0),
        }
    }

    pub async fn get(&mut self, name: &str) -> Option<String> {
        if name == "fastn" {
            self.packages_under_process
                .push(self.get_current_package().ok()?.name);
            return Some(fastn_dot_ftd::get2(self).await);
        }
        if name == "fastn-lib" {
            self.packages_under_process
                .push(self.get_current_package().ok()?.name);
            return Some(fastn_core::fastn_lib_ftd().to_string());
        }

        return get_for_package(format!("{}/", name.trim_end_matches('/')).as_str(), self).await;

        async fn get_for_package(name: &str, lib: &mut fastn_core::Library2) -> Option<String> {
            let package = lib.get_current_package().ok()?;
            if name.starts_with(package.name.as_str()) {
                if let Some(r) = get_data_from_package(name, &package, lib).await {
                    return Some(r);
                }
            }

            for (alias, package) in package.aliases() {
                if name.starts_with(alias) {
                    if let Some(r) = get_data_from_package(
                        name.replacen(alias, &package.name, 1).as_str(),
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
                        name.replacen(alias, &package.name, 1).as_str(),
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
            package: &fastn_core::Package,
            lib: &mut Library2,
        ) -> Option<String> {
            lib.push_package_under_process(package).ok()?;
            let packages = &lib.config.config.all_packages;
            let package = packages.get(package.name.as_str()).unwrap_or(package);
            // Explicit check for the current package.
            if !name.starts_with(package.name.as_str()) {
                return None;
            }
            let new_name = name.replacen(package.name.as_str(), "", 1);
            let (file_path, data) = package
                .resolve_by_id(
                    new_name.as_str(),
                    None,
                    lib.config.config.package.name.as_str(),
                    &lib.config.config.ds,
                )
                .await
                .ok()?;
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
    /// visit www.fastn.dev/glossary/#lazy-processor
    pub fn is_lazy_processor(
        &self,
        section: &ftd::ftd2021::p1::Section,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<bool> {
        Ok(section
            .header
            .str(doc.name, section.line_number, ftd::PROCESSOR_MARKER)?
            .eq("page-headings"))
    }

    pub async fn process<'a>(
        &'a self,
        _section: &ftd::ftd2021::p1::Section,
        _doc: &'a ftd::ftd2021::p2::TDoc<'a>,
    ) -> ftd::ftd2021::p1::Result<ftd::Value> {
        unimplemented!("we are removing support for 0.2, migrate to 0.3 please")
    }
}

#[derive(Default)]
pub struct FastnLibrary {}

impl FastnLibrary {
    pub fn get(&self, name: &str, _doc: &ftd::ftd2021::p2::TDoc) -> Option<String> {
        if name == "fastn" {
            Some(fastn_package::old_fastn::fastn_ftd_2021().to_string())
        } else {
            // Note: currently we do not allow users to import other modules from FASTN.ftd
            eprintln!("FASTN.ftd can only import `fastn` module");
            None
        }
    }

    pub fn get_with_result(
        &self,
        name: &str,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<String> {
        match self.get(name, doc) {
            Some(v) => Ok(v),
            None => ftd::ftd2021::p2::utils::e2(format!("library not found: {}", name), "", 0),
        }
    }
}
