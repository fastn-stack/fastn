impl fastn_core::Config {
    pub async fn read(
        _fastn_ftd: fastn_section::Document,
        _arena: &mut fastn_unresolved::Arena,
    ) -> Result<Self, ReadError> {
        Ok(fastn_core::Config {
            sitemap: fastn_core::Sitemap {},
            auto_imports: None,
            redirects: vec![],
            dynamic_routes: vec![],
        })
    }
}

#[derive(Debug)]
pub enum ReadError {
    SourceError(fastn_section::Error),
}
