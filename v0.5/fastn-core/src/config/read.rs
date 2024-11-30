impl fastn_core::Config {
    pub async fn read(
        _fastn_ftd: fastn_section::Document,
        _arena: &mut fastn_unresolved::Arena,
    ) -> Result<Self, ReadError> {
        Ok(fastn_core::Config {
            sitemap: fastn_core::Sitemap {},
            auto_import_scope: desugar_auto_imports(&[]),
            redirects: vec![],
            dynamic_routes: vec![],
        })
    }
}

#[derive(Debug)]
pub enum ReadError {
    SourceError(fastn_section::Error),
}

fn desugar_auto_imports(
    _auto_imports: &[fastn_core::config::AutoImport],
) -> fastn_unresolved::SFId {
    todo!()
}
