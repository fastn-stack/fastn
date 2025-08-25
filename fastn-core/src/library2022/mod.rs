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
    #[tracing::instrument(skip(self))]
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

    #[tracing::instrument(skip(self))]
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
                message: format!("Can't find current package: {current_package_name}"),
                doc_id: "".to_string(),
                line_number: 0,
            })
    }

    #[tracing::instrument(skip(self))]
    pub async fn get(
        &mut self,
        name: &str,
        current_processing_module: &str,
        session_id: &Option<String>,
    ) -> fastn_core::Result<(String, String, usize)> {
        if name == "fastn" {
            tracing::info!("fastn.ftd requested");
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

        #[tracing::instrument(skip(lib, session_id))]
        async fn get_for_package(
            name: &str,
            lib: &mut fastn_core::Library2022,
            current_processing_module: &str,
            session_id: &Option<String>,
        ) -> fastn_core::Result<(String, String, usize)> {
            let package = lib.get_current_package(current_processing_module)?;

            tracing::info!(
                "getting data for {name} in current package {}",
                package.name
            );

            let main_package = lib.config.package.name.to_string();
            // Check for app possibility
            if current_processing_module.contains("/-/") && main_package == package.name {
                let package_name = current_processing_module.split_once("/-/").unwrap().0;
                if let Some(app) = package
                    .apps
                    .iter()
                    .find(|app| app.package.name == package_name)
                    && let Some(val) = get_for_package_(name, lib, &app.package, session_id).await?
                    {
                        return Ok(val);
                    }
            }

            if let Some(val) = get_for_package_(name, lib, &package, session_id).await? {
                return Ok(val);
            }

            if name.starts_with("inherited-") {
                // The inherited- prefix is added to every dependency that is `auto-import`ed
                // and has a provided-via in the main package's FASTN.ftd
                let new_name = name.trim_start_matches("inherited-");
                // We only check the main package
                let main_package = lib.config.package.clone();
                if let Some(provided_via) = main_package.dependencies.iter().find_map(|d| {
                    if d.package.name == new_name.trim_end_matches('/') && d.provided_via.is_some()
                    {
                        d.provided_via.clone()
                    } else {
                        None
                    }
                }) {
                    tracing::error!("using provided-via: {provided_via} for {name}");
                    if let Some((content, size)) =
                        get_data_from_package(&provided_via, &main_package, lib, session_id).await?
                    {
                        // NOTE: we still return `name`. This way, we use source of provided-via's
                        // module but act as if the source is from `name`.
                        // Also note that this only applies to modules starting with "inherited-"
                        let name = format!("{}/", name.trim_end_matches('/'));
                        tracing::info!(?content, ?name);
                        return Ok((content, name, size));
                    }
                }
            }

            fastn_core::usage_error(format!("library not found 1: {name}: {package:?}"))
        }

        #[tracing::instrument(skip(lib, package))]
        async fn get_for_package_(
            name: &str,
            lib: &mut fastn_core::Library2022,
            package: &fastn_core::Package,
            session_id: &Option<String>,
        ) -> fastn_core::Result<Option<(String, String, usize)>> {
            tracing::info!("getting data for {name} in package {}", package.name);
            if name.starts_with(package.name.as_str()) {
                tracing::info!("found {name} in package {}", package.name);
                if let Some((content, size)) =
                    get_data_from_package(name, package, lib, session_id).await?
                {
                    return Ok(Some((content, name.to_string(), size)));
                }
            }
            // Self package referencing
            if package.name.ends_with(name.trim_end_matches('/')) {
                tracing::info!(
                    "self package referencing {name} in package {}",
                    package.name
                );
                let package_index = format!("{}/", package.name.as_str());
                if let Some((content, size)) =
                    get_data_from_package(package_index.as_str(), package, lib, session_id).await?
                {
                    return Ok(Some((content, format!("{package_index}index.ftd"), size)));
                }
            }

            for (alias, package) in package.aliases() {
                tracing::info!(
                    "checking alias {alias} for {name} in package {}",
                    package.name
                );
                lib.push_package_under_process(name, package, session_id)
                    .await?;
                if name.starts_with(alias) {
                    let name = name.replacen(alias, &package.name, 1);
                    if let Some((content, size)) =
                        get_data_from_package(name.as_str(), package, lib, session_id).await?
                    {
                        return Ok(Some((content, name.to_string(), size)));
                    }
                }
            }

            Ok(None)
        }

        #[allow(clippy::await_holding_refcell_ref)]
        #[tracing::instrument(skip(lib, package))]
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
            tracing::info!("checking package: {}", package.name);
            // Explicit check for the current package.
            let name = format!("{}/", name.trim_end_matches('/'));
            if !name.starts_with(format!("{}/", package.name.as_str()).as_str()) {
                return Ok(None);
            }

            let id = name.replacen(package.name.as_str(), "", 1);

            tracing::info!("checking sitemap for {id}");

            let resolved_id = package
                .sitemap
                .as_ref()
                .and_then(|sitemap| {
                    sitemap
                        .resolve_document(&id)
                        .map(|(sitemap_id, _)| sitemap_id)
                })
                .unwrap_or(id);

            tracing::info!("found id in sitemap: {resolved_id}");

            let (file_path, data) = package
                .resolve_by_id(
                    resolved_id.as_str(),
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
                let body_with_prefix = package.get_prefixed_body(
                    &lib.config.package,
                    body.as_str(),
                    name.as_str(),
                    true,
                );
                let line_number = body_with_prefix.split('\n').count() - body.split('\n').count();
                (body_with_prefix, line_number)
            }))
        }
    }

    #[cfg(feature = "use-config-json")]
    #[tracing::instrument(skip(self, package))]
    pub(crate) async fn push_package_under_process(
        &mut self,
        module: &str,
        package: &fastn_core::Package,
        _session_id: &Option<String>,
    ) -> ftd::ftd2021::p1::Result<()> {
        tracing::info!("{:?}", package.name);

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

        fastn_wasm::insert_or_update(&self.config.all_packages, package.name.clone(), package);

        Ok(())
    }

    /// process the $processor$ and return the processor's output
    pub async fn process<'a>(
        &'a mut self,
        ast: ftd_ast::Ast,
        processor: String,
        doc: &'a mut ftd::interpreter::TDoc<'a>,
        preview_session_id: &Option<String>,
    ) -> ftd::interpreter::Result<fastn_resolved::Value> {
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
            "sql-query" | "sql-execute" | "sql-batch" if preview_session_id.is_some() => {
                // send empty result when the request is for IDE previews
                // FIXME: If the user asking for preview has write access to this site then we should not
                // block their request.
                processor::sqlite::result_to_value(Default::default(), kind, doc, &value)
            }
            "sql-query" => processor::sql::process(value, kind, doc, self, "sql-query").await,
            "sql-execute" => processor::sql::process(value, kind, doc, self, "sql-execute").await,
            "sql-batch" => processor::sql::process(value, kind, doc, self, "sql-batch").await,
            // "package-query" => processor::package_query::process(value, kind, doc, self).await,
            // "pg" => processor::pg::process(value, kind, doc, self).await,
            "query" => processor::query::process(value, kind, doc, self, preview_session_id).await,
            t => Err(ftd::interpreter::Error::ParseError {
                doc_id: self.document_id.to_string(),
                line_number,
                message: format!("fastn-Error: No such processor: {t}"),
            }),
        }
    }
}

fn get_processor_data(
    ast: ftd_ast::Ast,
    doc: &mut ftd::interpreter::TDoc,
) -> ftd::interpreter::Result<(String, String, ftd_ast::VariableValue, fastn_resolved::Kind)> {
    use ftd::interpreter::KindDataExt;

    let line_number = ast.line_number();
    let ast_name = ast.name();
    if let Ok(variable_definition) = ast.clone().get_variable_definition(doc.name) {
        let kind = fastn_resolved::KindData::from_ast_kind(
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
                    message: format!("No processor found for `{ast_name}`"),
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
                    message: format!("No processor found for `{ast_name}`"),
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
