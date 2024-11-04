#![allow(dead_code)]
// the router will depend on fastn-lang, but
pub enum Route {
    NotFound,
    // String contains path, the data may contain more than that was passed to route, eg it
    // can extract some extra path specific data from FASTN.ftd file
    Document(String, serde_json::Value),
    Wasm(String, serde_json::Value),
    Redirect(String),
    Static(String),
}

impl fastn_core::Config {
    pub async fn resolve(&mut self, _path: &str) -> Route {
        todo!()
    }
}
