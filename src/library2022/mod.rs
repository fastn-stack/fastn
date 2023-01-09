pub(crate) mod get_data_processor;
pub(crate) mod http_processor;
pub(crate) mod processor;
pub(crate) mod toc_processor;

#[derive(Default, Debug, serde::Serialize)]
pub struct KeyValueData {
    pub key: String,
    pub value: String,
}

impl KeyValueData {
    pub fn from(key: String, value: String) -> Self {
        Self { key, value }
    }
}

#[derive(Debug)]
pub struct Library2022 {
    pub config: fpm::Config,
    /// If the current module being parsed is a markdown file, `.markdown` contains the name and
    /// content of that file
    pub markdown: Option<(String, String)>,
    pub document_id: String,
    pub translated_data: fpm::TranslationData,
    pub base_url: String,
    pub module_package_map: std::collections::BTreeMap<String, String>,
}

impl Library2022 {
    pub async fn get_with_result(
        &mut self,
        name: &str,
        current_processing_module: &str,
    ) -> ftd::p11::Result<String> {
        match self.get(name, current_processing_module).await {
            Some(v) => Ok(v),
            None => ftd::p11::utils::parse_error(format!("library not found: {}", name), "", 0),
        }
    }

    pub(crate) fn get_current_package(
        &self,
        current_processing_module: &str,
    ) -> ftd::p11::Result<fpm::Package> {
        let current_package_name = self
            .module_package_map
            .get(current_processing_module.trim_matches('/'))
            .ok_or_else(|| ftd::p11::Error::ParseError {
                message: "The processing document stack is empty: Can't find module in any package"
                    .to_string(),
                doc_id: current_processing_module.to_string(),
                line_number: 0,
            })?;

        self.config
            .all_packages
            .borrow()
            .get(current_package_name)
            .map(|p| p.to_owned())
            .ok_or_else(|| ftd::p11::Error::ParseError {
                message: format!("Can't find current package: {}", current_package_name),
                doc_id: "".to_string(),
                line_number: 0,
            })
    }

    pub async fn get(&mut self, name: &str, current_processing_module: &str) -> Option<String> {
        if name == "fpm" {
            return Some(fpm::library::fpm_dot_ftd::get2022(self).await);
        }

        if name == "fpm-lib" {
            return Some(fpm::fpm_lib_ftd().to_string());
        }

        return get_for_package(
            format!("{}/", name.trim_end_matches('/')).as_str(),
            self,
            current_processing_module,
        )
        .await;

        async fn get_for_package(
            name: &str,
            lib: &mut fpm::Library2022,
            current_processing_module: &str,
        ) -> Option<String> {
            let package = lib.get_current_package(current_processing_module).ok()?;
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
            package: &fpm::Package,
            lib: &mut fpm::Library2022,
        ) -> Option<String> {
            lib.push_package_under_process(name, package).await.ok()?;
            let packages = lib.config.all_packages.borrow();
            let package = packages.get(package.name.as_str()).unwrap_or(package);
            // Explicit check for the current package.
            if !name.starts_with(package.name.as_str()) {
                return None;
            }
            let new_name = name.replacen(package.name.as_str(), "", 1);
            let (file_path, data) = package.resolve_by_id(new_name.as_str(), None).await.ok()?;
            if !file_path.ends_with(".ftd") {
                return None;
            }
            String::from_utf8(data)
                .ok()
                .map(|body| package.get_prefixed_body(body.as_str(), name, true))
        }
    }

    pub(crate) async fn push_package_under_process(
        &mut self,
        module: &str,
        package: &fpm::Package,
    ) -> ftd::p1::Result<()> {
        self.module_package_map.insert(
            module.trim_matches('/').to_string(),
            package.name.to_string(),
        );
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

    /// process the processor and return the processor's output
    pub async fn process<'a>(
        &'a self,
        ast: ftd::ast::AST,
        processor: String,
        doc: &'a mut ftd::interpreter2::TDoc<'a>,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
        let line_number = ast.line_number();
        let (_processor, value, kind) = get_processor_data(ast, doc)?;
        match processor.as_str() {
            "http" => http_processor::process(value, kind, doc, &self.config).await,
            "toc" => toc_processor::process(value, kind, doc, &self.config),
            "get-data" => get_data_processor::process(value, kind, doc, &self.config),
            "sitemap" => processor::sitemap::process(value, kind, doc, &self.config),
            "full-sitemap" => {
                processor::sitemap::full_sitemap_process(value, kind, doc, &self.config)
            }
            t => Err(ftd::interpreter2::Error::ParseError {
                doc_id: self.document_id.to_string(),
                line_number,
                message: format!("FPM-Error: No such processor: {}", t),
            }),
        }
    }
}

fn get_processor_data(
    ast: ftd::ast::AST,
    doc: &mut ftd::interpreter2::TDoc,
) -> ftd::interpreter2::Result<(String, ftd::ast::VariableValue, ftd::interpreter2::Kind)> {
    let line_number = ast.line_number();
    let ast_name = ast.name();
    if let Ok(variable_definition) = ast.clone().get_variable_definition(doc.name) {
        let kind = ftd::interpreter2::KindData::from_ast_kind(
            variable_definition.kind,
            &Default::default(),
            doc,
            variable_definition.line_number,
        )?
        .into_optional()
        .ok_or(ftd::interpreter2::Error::ValueNotFound {
            doc_id: doc.name.to_string(),
            line_number,
            message: format!(
                "Cannot find kind for `{}`",
                variable_definition.name.as_str(),
            ),
        })?;
        let processor =
            variable_definition
                .processor
                .ok_or(ftd::interpreter2::Error::ParseError {
                    message: format!("No processor found for `{}`", ast_name),
                    doc_id: doc.name.to_string(),
                    line_number,
                })?;
        Ok((processor, variable_definition.value, kind.kind))
    } else {
        let variable_invocation = ast.get_variable_invocation(doc.name)?;
        let kind = doc
            .get_variable(
                variable_invocation.name.as_str(),
                variable_invocation.line_number,
            )?
            .kind;
        let processor =
            variable_invocation
                .processor
                .ok_or(ftd::interpreter2::Error::ParseError {
                    message: format!("No processor found for `{}`", ast_name),
                    doc_id: doc.name.to_string(),
                    line_number,
                })?;
        Ok((processor, variable_invocation.value, kind.kind))
    }
}
