mod read;

#[allow(dead_code)]
pub struct Config {
    sitemap: Sitemap,
    pub auto_imports: Vec<AutoImport>,
    redirects: Vec<Redirect>,
    dynamic_routes: Vec<DynamicRoute>,
}

pub struct AutoImport {}
pub struct DynamicRoute {}
pub struct Redirect {}
pub struct Sitemap {}

pub fn desugar_auto_imports(_auto_imports: &[AutoImport]) -> Vec<fastn_unresolved::URD> {
    // todo!()
    vec![]
}
