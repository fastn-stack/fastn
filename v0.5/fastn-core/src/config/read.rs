impl fastn_core::Config {
    pub async fn read(mut ds: Box<dyn fastn_lang::DS>) -> Self {
        ds.source("src/config.toml").await.unwrap();
        fastn_core::Config {
            ds,
            sitemap: fastn_core::Sitemap {},
            auto_imports: vec![],
            redirects: vec![],
            dynamic_routes: vec![],
        }
    }
}
