mod read;

#[allow(dead_code)]
pub struct Config {
    sitemap: Sitemap,
    pub auto_import_scope: fastn_unresolved::SFId,
    redirects: Vec<Redirect>,
    dynamic_routes: Vec<DynamicRoute>,
}

pub struct AutoImport {}
pub struct DynamicRoute {}
pub struct Redirect {}
pub struct Sitemap {}
