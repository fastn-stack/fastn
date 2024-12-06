impl fastn_router::Router {
    pub fn route(
        &self,
        _path: &str,
        _method: fastn_router::Method,
        _data: &[u8],
    ) -> fastn_router::Route {
        fastn_router::Route::Document("index.ftd".to_string(), serde_json::Value::Null)
    }
}
