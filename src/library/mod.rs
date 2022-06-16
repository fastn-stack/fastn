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
    pub fn get_with_result(
        &self,
        name: &str,
        packages: &mut Vec<&fpm::Package>,
    ) -> ftd::p1::Result<String> {
        match self.get(name, packages) {
            Some(v) => Ok(v),
            None => ftd::e2(format!("library not found: {}", name), "", 0),
        }
    }

    pub fn get(&self, name: &str, packages: &mut Vec<&fpm::Package>) -> Option<String> {
        if name == "fpm" {
            packages.push(packages.last()?);
            return Some(fpm_dot_ftd::get(self));
        }
        if name == "fpm-lib" {
            packages.push(packages.last()?);
            return Some(fpm::fpm_lib_ftd().to_string());
        }

        return get_for_package(name, packages, self);

        fn get_for_package(
            name: &str,
            packages: &mut Vec<&fpm::Package>,
            lib: &fpm::Library,
        ) -> Option<String> {
            let package = packages.last()?;
            if name.starts_with(package.name.as_str()) {
                if let Some(r) = get_data_from_package(name, package, lib) {
                    packages.push(packages.last()?);
                    return Some(r);
                }
            }

            for (alias, package) in package.aliases() {
                if name.starts_with(alias) {
                    if let Some(r) = get_data_from_package(
                        name.replacen(&alias, &package.name, 1).as_str(),
                        package,
                        lib,
                    ) {
                        packages.push(package);
                        return Some(r);
                    }
                }
            }

            if let Some(translation_of) = package.translation_of.as_ref() {
                let name = name.replacen(package.name.as_str(), translation_of.name.as_str(), 1);
                if name.starts_with(translation_of.name.as_str()) {
                    if let Some(r) = get_data_from_package(name.as_str(), translation_of, lib) {
                        packages.push(packages.last()?);
                        return Some(r);
                    }
                }

                for (alias, package) in translation_of.aliases() {
                    if name.starts_with(alias) {
                        if let Some(r) = get_data_from_package(
                            name.replacen(&alias, &package.name, 1).as_str(),
                            package,
                            lib,
                        ) {
                            packages.push(package);
                            return Some(r);
                        }
                    }
                }
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
                    return Some(package.get_prefixed_body(body.as_str(), name, true));
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
