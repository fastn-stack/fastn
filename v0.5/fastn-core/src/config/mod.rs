mod read;

#[allow(dead_code)]
pub struct Config {
    sitemap: Sitemap,
    pub auto_imports: Option<fastn_unresolved::SFId>,
    redirects: Vec<Redirect>,
    dynamic_routes: Vec<DynamicRoute>,
}

pub struct AutoImport {}
pub struct DynamicRoute {}
pub struct Redirect {}
pub struct Sitemap {}

#[expect(unused)]
fn desugar_auto_imports(_auto_imports: &[AutoImport]) -> fastn_unresolved::SFId {
    todo!()
}
