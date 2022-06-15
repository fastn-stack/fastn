mod fpm_dot_ftd;
mod get_data;
mod get_version_data;
pub(crate) mod http;
mod include;
mod sitemap;
mod sqlite;
mod toc;

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
    pub fn get_with_result(&self, name: &str, doc: &ftd::p2::TDoc) -> ftd::p1::Result<String> {
        match self.get(name, doc) {
            Some(v) => Ok(v),
            None => ftd::e2(format!("library not found: {}", name), "", 0),
        }
    }

    pub fn get(&self, name: &str, doc: &ftd::p2::TDoc) -> Option<String> {
        if name == "fpm" {
            return Some(fpm_dot_ftd::get(self));
        }
        if name == "fpm-lib" {
            return Some(fpm::fpm_lib_ftd().to_string());
        }
        return if let Some(r) = get_for_package_config(name, &self.config.package, self) {
            Some(r)
        } else {
            for package in &get_root_package_for_path(doc.name, &self.config.package, false) {
                if let Some(resp) = get_for_package_config(name, package, self) {
                    return Some(resp);
                };
            }
            None
        };

        fn get_for_package_config(
            name: &str,
            package: &fpm::Package,
            lib: &fpm::Library,
        ) -> Option<String> {
            if name.starts_with(package.name.as_str()) {
                if let Some(r) = get_data_from_package(name, package, lib) {
                    return Some(r);
                }
            }
            // If package == lib.config.package => Current iteration for the top most package
            // Root package evaluation
            if package.name == lib.config.package.name {
                if let Some(translation_of_package) = lib.config.package.translation_of.as_ref() {
                    // Index document can be accessed from the package name directly. For others
                    // `/` is required to be certain. foo-hi -> foo-hi-hi This is wrong. That's why
                    // we ensure a strict `/` check or a full name match
                    let new_name = if translation_of_package.name.as_str().eq(name) {
                        package.name.clone()
                    } else {
                        name.replacen(
                            format!("{}/", translation_of_package.name.as_str()).as_str(),
                            format!("{}/", package.name.as_str()).as_str(),
                            1,
                        )
                    };

                    if let Some(resp) = get_data_from_package(new_name.as_str(), package, lib) {
                        return Some(resp);
                    }
                }
            }

            // Check the translation of the package
            if let Some(translation_of_package) = package.translation_of.as_ref() {
                if let Some(resp) = get_for_package_config(
                    name.replacen(
                        package.name.as_str(),
                        translation_of_package.name.as_str(),
                        1,
                    )
                    .as_str(),
                    translation_of_package,
                    lib,
                ) {
                    return Some(resp);
                }
            }

            if let Some(r) = get_from_all_dependencies(name, package, lib) {
                return Some(r);
            }
            None
        }
        fn get_root_package_for_path(
            name: &str,
            package: &fpm::Package,
            include_self: bool,
        ) -> Vec<fpm::Package> {
            if name.starts_with(package.name.as_str()) {
                if include_self {
                    vec![package.to_owned()]
                } else {
                    vec![]
                }
            } else {
                let mut resp = vec![];
                for dep in &package.dependencies {
                    if let Some(unaliased_name) = dep.unaliased_name(name) {
                        resp.extend(get_root_package_for_path(
                            unaliased_name.as_str(),
                            &dep.package,
                            false,
                        ));
                        resp.push(dep.package.clone())
                    }
                }
                resp
            }
        }

        fn get_from_all_dependencies(
            name: &str,
            package: &fpm::Package,
            lib: &fpm::Library,
            // evaluated_packages: &mut Vec<String>,
        ) -> Option<String> {
            for dep in &package.get_flattened_dependencies() {
                if let Some(non_aliased_name) = dep.unaliased_name(name) {
                    if non_aliased_name.starts_with(dep.package.name.as_str()) {
                        if let Some(resp) =
                            get_from_dependency(non_aliased_name.as_str(), &dep.package, lib)
                        {
                            return Some(resp);
                        };
                    }
                }
            }
            None
        }
        fn get_from_dependency(
            name: &str,
            from_package: &fpm::Package,
            lib: &fpm::Library,
        ) -> Option<String> {
            // TODO: Here the library needs to be evaluated for this particular package
            // Right now the solution works by recursively looking for the package in the dependency tree
            // Ideally we should also know the library definition of a particular package
            if let Some(resp_body) = get_data_from_package(name, from_package, lib) {
                return Some(resp_body);
            }
            None
        }

        fn get_file_from_location(base_path: &camino::Utf8PathBuf, name: &str) -> Option<String> {
            let os_name = name
                .trim_start_matches('/')
                .trim_end_matches('/')
                .replace("/", std::path::MAIN_SEPARATOR.to_string().as_str());
            if let Ok(v) = std::fs::read_to_string(base_path.join(format!("{}.ftd", os_name))) {
                return Some(v);
            }
            if let Ok(v) = std::fs::read_to_string(base_path.join(os_name).join("index.ftd")) {
                return Some(v);
            }
            None
        }

        fn get_data_from_package(
            name: &str,
            package: &fpm::Package,
            lib: &Library,
        ) -> Option<String> {
            let path = lib.config.get_root_for_package(package);
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
                    return Some(package.get_prefixed_body(body.as_str(), name, false));
                }
            }
            None
        }
    }

    // TODO: async
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
            "http" => fpm::library::http::processor(section, doc).await,
            "package-query" => fpm::library::sqlite::processor(section, doc, &self.config).await,
            "toc" => fpm::library::toc::processor(section, doc, &self.config),
            "include" => fpm::library::include::processor(section, doc, &self.config),
            "get-data" => fpm::library::get_data::processor(section, doc, &self.config),
            "sitemap" => fpm::library::sitemap::processor(section, doc, &self.config),
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
            t => unimplemented!("No such processor: {}", t),
        }
    }
}

#[derive(Default)]
pub struct FPMLibrary {}

impl FPMLibrary {
    pub fn get(&self, name: &str, _doc: &ftd::p2::TDoc) -> Option<String> {
        if name == "fpm" {
            return Some(format!(
                "{}\n\n-- optional package-data package:\n",
                fpm::fpm_ftd()
            ));
        } else {
            // Note: currently we do not allow users to import other modules from FPM.ftd
            eprintln!("FPM.ftd can only import `fpm` module");
            None
        }
    }

    pub fn process(
        &self,
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<ftd::Value> {
        ftd::unknown_processor_error(
            format!("unimplemented for section {:?} and doc {:?}", section, doc),
            doc.name.to_string(),
            section.line_number,
        )
    }

    pub fn get_with_result(&self, name: &str, doc: &ftd::p2::TDoc) -> ftd::p1::Result<String> {
        match self.get(name, doc) {
            Some(v) => Ok(v),
            None => ftd::e2(format!("library not found: {}", name), "", 0),
        }
    }
}
