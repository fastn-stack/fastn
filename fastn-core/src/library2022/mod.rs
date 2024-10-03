pub(crate) mod processor;
pub(crate) mod utils;

#[derive(Default, Debug, serde::Serialize)]
pub struct KeyValueData {
    pub key: String,
    pub value: String,
}

impl KeyValueData {
    #[allow(dead_code)]
    pub fn from(key: String, value: String) -> Self {
        Self { key, value }
    }
}

pub type Library2022 = fastn_core::RequestConfig;

impl Library2022 {
    pub async fn get_with_result(
        &mut self,
        name: &str,
        current_processing_module: &str,
        session_id: &Option<String>,
    ) -> ftd_p1::Result<(String, String, usize)> {
        match self.get(name, current_processing_module, session_id).await {
            Ok(v) => Ok(v),
            Err(e) => ftd_p1::utils::parse_error(e.to_string(), "", 0),
        }
    }

    pub(crate) fn get_current_package(
        &self,
        current_processing_module: &str,
    ) -> ftd_p1::Result<fastn_core::Package> {
        let current_package_name = self
            .module_package_map
            .get(current_processing_module.trim_matches('/'))
            .ok_or_else(|| ftd_p1::Error::ParseError {
                message: "The processing document stack is empty: Can't find module in any package"
                    .to_string(),
                doc_id: current_processing_module.to_string(),
                line_number: 0,
            })?;

        self.config
            .all_packages
            .get(current_package_name)
            .map(|p| p.get().to_owned())
            .ok_or_else(|| ftd_p1::Error::ParseError {
                message: format!("Can't find current package: {}", current_package_name),
                doc_id: "".to_string(),
                line_number: 0,
            })
    }

    pub async fn get(
        &mut self,
        name: &str,
        current_processing_module: &str,
        session_id: &Option<String>,
    ) -> fastn_core::Result<(String, String, usize)> {
        if name == "fastn" {
            if self.config.test_command_running {
                return Ok((
                    fastn_core::commands::test::test_fastn_ftd().to_string(),
                    "$fastn$/fastn.ftd".to_string(),
                    0,
                ));
            } else {
                return Ok((
                    fastn_core::library::fastn_dot_ftd::get2022(self).await,
                    "$fastn$/fastn.ftd".to_string(),
                    0,
                ));
            }
        }

        return get_for_package(
            format!("{}/", name.trim_end_matches('/')).as_str(),
            self,
            current_processing_module,
            session_id,
        )
        .await;

        async fn get_for_package(
            name: &str,
            lib: &mut fastn_core::Library2022,
            current_processing_module: &str,
            session_id: &Option<String>,
        ) -> fastn_core::Result<(String, String, usize)> {
            let package = lib.get_current_package(current_processing_module)?;
            if name.starts_with(package.name.as_str()) {
                if let Some((content, size)) =
                    get_data_from_package(name, &package, lib, session_id).await?
                {
                    return Ok((content, name.to_string(), size));
                }
            }
            // Self package referencing
            if package.name.ends_with(name.trim_end_matches('/')) {
                let package_index = format!("{}/", package.name.as_str());
                if let Some((content, size)) =
                    get_data_from_package(package_index.as_str(), &package, lib, session_id).await?
                {
                    return Ok((content, format!("{package_index}index.ftd"), size));
                }
            }

            for (alias, package) in package.aliases() {
                lib.push_package_under_process(name, package, session_id)
                    .await?;
                if name.starts_with(alias) {
                    let name = name.replacen(alias, &package.name, 1);
                    if let Some((content, size)) =
                        get_data_from_package(name.as_str(), package, lib, session_id).await?
                    {
                        return Ok((content, name.to_string(), size));
                    }
                }
            }

            /*let translation_of = match package.translation_of.as_ref() {
                Some(translation_of) => translation_of.to_owned(),
                None => return None,
            };

            let name = name.replacen(package.name.as_str(), translation_of.name.as_str(), 1);
            if name.starts_with(translation_of.name.as_str()) {
                if let Some((content, size)) =
                    get_data_from_package(name.as_str(), &translation_of, lib).await
                {
                    return Some((content, name.to_string(), size));
                }
            }

            for (alias, package) in translation_of.aliases() {
                if name.starts_with(alias) {
                    let name = name.replacen(alias, &package.name, 1);
                    if let Some((content, size)) =
                        get_data_from_package(name.as_str(), package, lib).await
                    {
                        return Some((content, name.to_string(), size));
                    }
                }
            }*/

            fastn_core::usage_error(format!("library not found: {}", name))
        }

        // TODO: This function is too long. Break it down.
        #[allow(clippy::await_holding_refcell_ref)]
        async fn get_data_from_package(
            name: &str,
            package: &fastn_core::Package,
            lib: &mut fastn_core::Library2022,
            session_id: &Option<String>,
        ) -> fastn_core::Result<Option<(String, usize)>> {
            lib.push_package_under_process(name, package, session_id)
                .await?;
            let package = lib
                .config
                .find_package_else_default(package.name.as_str(), Some(package.to_owned()));
            // Explicit check for the current package.
            let name = format!("{}/", name.trim_end_matches('/'));
            if !name.starts_with(format!("{}/", package.name.as_str()).as_str()) {
                return Ok(None);
            }
            let new_name = name.replacen(package.name.as_str(), "", 1);
            let (file_path, data) = package
                .resolve_by_id(
                    new_name.as_str(),
                    None,
                    lib.config.package.name.as_str(),
                    &lib.config.ds,
                    session_id,
                )
                .await?;
            if !file_path.ends_with(".ftd") {
                return Ok(None);
            }
            Ok(String::from_utf8(data).ok().map(|body| {
                let body_with_prefix =
                    package.get_prefixed_body(body.as_str(), name.as_str(), true);
                let line_number = body_with_prefix.split('\n').count() - body.split('\n').count();
                (body_with_prefix, line_number)
            }))
        }
    }

    #[cfg(feature = "use-config-json")]
    pub(crate) async fn push_package_under_process(
        &mut self,
        module: &str,
        package: &fastn_core::Package,
        _session_id: &Option<String>,
    ) -> ftd::ftd2021::p1::Result<()> {
        self.module_package_map.insert(
            module.trim_matches('/').to_string(),
            package.name.to_string(),
        );
        if !self.config.all_packages.contains(package.name.as_str()) {
            return Err(ftd::ftd2021::p1::Error::ParseError {
                message: format!("Cannot resolve the package: {}", package.name),
                doc_id: self.document_id.to_string(),
                line_number: 0,
            });
        }

        Ok(())
    }

    #[cfg(not(feature = "use-config-json"))]
    pub(crate) async fn push_package_under_process(
        &mut self,
        module: &str,
        package: &fastn_core::Package,
        session_id: &Option<String>,
    ) -> ftd::ftd2021::p1::Result<()> {
        self.module_package_map.insert(
            module.trim_matches('/').to_string(),
            package.name.to_string(),
        );
        if self.config.all_packages.contains(package.name.as_str()) {
            return Ok(());
        }

        let package = self
            .config
            .resolve_package(package, session_id)
            .await
            .map_err(|e| ftd::ftd2021::p1::Error::ParseError {
                message: format!("Cannot resolve the package: {}, Error: {}", package.name, e),
                doc_id: self.document_id.to_string(),
                line_number: 0,
            })?;

        fastn_ds::insert_or_update(&self.config.all_packages, package.name.clone(), package);

        Ok(())
    }

    /// process the $processor$ and return the processor's output
    pub async fn process<'a>(
        &'a mut self,
        ast: ftd_ast::Ast,
        processor: String,
        doc: &'a mut ftd::interpreter::TDoc<'a>,
        preview_session_id: &Option<String>,
    ) -> ftd::interpreter::Result<ftd::interpreter::Value> {
        tracing::info!(
            msg = "stuck-on-processor",
            doc = doc.name,
            processor = processor
        );
        let line_number = ast.line_number();
        let (_processor, variable_name, value, kind) = get_processor_data(ast, doc)?;
        match processor.as_str() {
            "figma-typo-token" => {
                processor::figma_typography_tokens::process_typography_tokens(value, kind, doc)
            }
            "figma-cs-token" => processor::figma_tokens::process_figma_tokens(value, kind, doc),
            "figma-cs-token-old" => {
                processor::figma_tokens::process_figma_tokens_old(value, kind, doc)
            }
            "http" => processor::http::process(value, kind, doc, self).await,
            "translation-info" => processor::lang_details::process(value, kind, doc, self).await,
            "current-language" => processor::lang::process(value, kind, doc, self).await,
            "toc" => processor::toc::process(value, kind, doc),
            "get-data" => processor::get_data::process(value, kind, doc, self),
            "sitemap" => processor::sitemap::process(value, kind, doc, self),
            "full-sitemap" => processor::sitemap::full_sitemap_process(value, kind, doc, self),
            "request-data" => {
                processor::request_data::process(variable_name, value, kind, doc, self)
            }
            "document-readers" => processor::document::process_readers(
                value,
                kind,
                doc,
                self,
                self.document_id.as_str(),
            ),
            "document-writers" => processor::document::process_writers(
                value,
                kind,
                doc,
                self,
                self.document_id.as_str(),
            ),
            "user-groups" => processor::user_group::process(value, kind, doc, self),
            "user-group-by-id" => processor::user_group::process_by_id(value, kind, doc, self),
            "get-identities" => processor::user_group::get_identities(value, kind, doc, self).await,
            "document-id" => processor::document::document_id(value, kind, doc, self),
            "current-url" => processor::document::current_url(self),
            "document-full-id" => processor::document::document_full_id(value, kind, doc, self),
            "document-suffix" => processor::document::document_suffix(value, kind, doc, self),
            "document-name" => {
                processor::document::document_name(value, kind, doc, self, preview_session_id).await
            }
            "fetch-file" => {
                processor::fetch_file::fetch_files(value, kind, doc, self, preview_session_id).await
            }
            "user-details" => processor::user_details::process(value, kind, doc, self).await,
            "fastn-apps" => processor::apps::process(value, kind, doc, self),
            "is-reader" => processor::user_group::is_reader(value, kind, doc, self).await,
            // send empty result when the request is for IDE previews
            // FIXME: If the user asking for preview has write access to this site then we should not
            // block their request.
            "sql-query" => {
                processor::sql::process(
                    value,
                    kind,
                    doc,
                    self,
                    "sql-query",
                    preview_session_id.is_none(),
                )
                .await
            }
            "sql-execute" => {
                processor::sql::process(
                    value,
                    kind,
                    doc,
                    self,
                    "sql-execute",
                    preview_session_id.is_none(),
                )
                .await
            }
            "sql-batch" => {
                processor::sql::process(
                    value,
                    kind,
                    doc,
                    self,
                    "sql-batch",
                    preview_session_id.is_none(),
                )
                .await
            }
            // "package-query" => processor::package_query::process(value, kind, doc, self).await,
            // "pg" => processor::pg::process(value, kind, doc, self).await,
            "query" => processor::query::process(value, kind, doc, self, preview_session_id).await,
            t => Err(ftd::interpreter::Error::ParseError {
                doc_id: self.document_id.to_string(),
                line_number,
                message: format!("fastn-Error: No such processor: {}", t),
            }),
        }
    }
}

fn get_processor_data(
    ast: ftd_ast::Ast,
    doc: &mut ftd::interpreter::TDoc,
) -> ftd::interpreter::Result<(
    String,
    String,
    ftd_ast::VariableValue,
    ftd::interpreter::Kind,
)> {
    let line_number = ast.line_number();
    let ast_name = ast.name();
    if let Ok(variable_definition) = ast.clone().get_variable_definition(doc.name) {
        let kind = ftd::interpreter::KindData::from_ast_kind(
            variable_definition.kind,
            &Default::default(),
            doc,
            variable_definition.line_number,
        )?
        .into_optional()
        .ok_or(ftd::interpreter::Error::ValueNotFound {
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
                .ok_or(ftd::interpreter::Error::ParseError {
                    message: format!("No processor found for `{}`", ast_name),
                    doc_id: doc.name.to_string(),
                    line_number,
                })?;
        Ok((
            processor,
            variable_definition.name.to_string(),
            variable_definition.value,
            kind.kind,
        ))
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
                .ok_or(ftd::interpreter::Error::ParseError {
                    message: format!("No processor found for `{}`", ast_name),
                    doc_id: doc.name.to_string(),
                    line_number,
                })?;
        Ok((
            processor,
            variable_invocation.name.to_string(),
            variable_invocation.value,
            kind.kind,
        ))
    }
}
