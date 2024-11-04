mod read;

#[allow(dead_code)]
pub struct Config {
    ds: Box<dyn fastn_lang::DS>,
    sitemap: Sitemap,
    auto_imports: Vec<AutoImport>,
    redirects: Vec<Redirect>,
    dynamic_routes: Vec<DynamicRoute>,
}

impl std::ops::Deref for Config {
    type Target = Box<dyn fastn_lang::DS>;
    fn deref(&self) -> &Self::Target {
        &self.ds
    }
}

impl std::ops::DerefMut for Config {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ds
    }
}

pub struct DynamicRoute {}
pub struct Redirect {}
pub struct AutoImport {}
pub struct Sitemap {}
