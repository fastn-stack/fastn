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
            .contains(package.name.as_str())
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
            .map(|p| p.get().to_owned())
            .ok_or_else(|| ftd::ftd2021::p1::Error::ParseError {
                message: format!("Can't find current package: {}", current_package_name),
                doc_id: "".to_string(),
                line_number: 0,
            })
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
