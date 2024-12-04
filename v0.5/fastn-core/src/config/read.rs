impl fastn_core::Config {
    pub async fn read(_fastn_ftd: fastn_section::Document) -> Result<Self, ReadError> {
        Ok(fastn_core::Config {
            sitemap: fastn_core::Sitemap {},
            auto_imports: desugar_auto_imports(&[]),
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
) -> fastn_unresolved::AliasesSimple {
    let mut aliases = fastn_unresolved::AliasesSimple::new();
    let ftd = fastn_unresolved::SoMBase::Module("ftd".to_string());
    aliases.insert("ftd".to_string(), ftd);
    aliases
}
