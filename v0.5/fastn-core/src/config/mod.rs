mod read;

#[allow(dead_code)]
pub struct Config {
    sitemap: Sitemap,
    pub auto_imports: Vec<fastn_section::AutoImport>,
    redirects: Vec<Redirect>,
    dynamic_routes: Vec<DynamicRoute>,
}

pub struct DynamicRoute {}
pub struct Redirect {}

pub struct Sitemap {}
