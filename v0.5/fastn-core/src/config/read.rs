impl fastn_core::Config {
    pub fn read(ds: Box<dyn fastn_lang::DS>) -> Self {
        fastn_core::Config {
            ds,
            sitemap: fastn_core::Sitemap {},
            auto_imports: vec![],
            redirects: vec![],
            dynamic_routes: vec![],
        }
    }
}
