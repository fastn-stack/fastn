impl fastn_router::Router {
    // /foo.png
    // /-/ds.ft.com/foo.png
    pub fn route(&self, _path: &str, _method: fastn_router::Method) -> fastn_router::Route {
        fastn_router::Route::Document(fastn_router::Document {
            path: "index.ftd".to_string(),
            keys: vec![],
            partial: serde_json::Value::Null,
        })
    }
}
