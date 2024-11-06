#![allow(dead_code)]
// the router will depend on fastn-section.
pub enum Route {
    NotFound,
    // String contains the path, the data may contain more than that was passed to route, e.g., it
    // can extract some extra path-specific data from FASTN.ftd file
    Document(String, serde_json::Value),
    Wasm(String, serde_json::Value),
    Redirect(String),
    Static(String),
}

impl fastn_core::Config {
    pub async fn resolve(&self, _path: &str) -> Route {
        todo!()
    }
}
