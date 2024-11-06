mod read;

#[allow(dead_code)]
pub struct Config {
    sitemap: Sitemap,
    auto_imports: Vec<AutoImport>,
    redirects: Vec<Redirect>,
    dynamic_routes: Vec<DynamicRoute>,
}

pub struct DynamicRoute {}
pub struct Redirect {}
pub struct AutoImport {}
pub struct Sitemap {}
