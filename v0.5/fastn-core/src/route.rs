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

pub fn route(_config: &fastn_core::Config, _path: &str, _data: serde_json::Value) -> Route {
    // unresolved the fastn_ftd_source, and extract
    todo!()
}
