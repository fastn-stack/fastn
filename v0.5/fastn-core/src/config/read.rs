impl fastn_core::Config {
    pub async fn read(_fastn_ftd_sections: Vec<fastn_section::Section>) -> Result<Self, ReadError> {
        Ok(fastn_core::Config {
            sitemap: fastn_core::Sitemap {},
            auto_imports: vec![],
            redirects: vec![],
            dynamic_routes: vec![],
        })
    }
}

#[derive(Debug)]
pub enum ReadError {
    SourceError(fastn_section::Error),
}
