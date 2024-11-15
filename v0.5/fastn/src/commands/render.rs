impl fastn::commands::Render {
    pub async fn run(self, config: &mut fastn_core::Config) {
        let route = config.resolve(self.path.as_str()).await;
        match route {
            fastn_core::Route::Document(path, data) => {
                render_document(config, path.as_str(), data, self.strict).await
            }
            _ => todo!(),
        };
    }
}

async fn render_document(
    _config: &mut fastn_core::Config,
    _path: &str,
    _data: serde_json::Value,
    _strict: bool,
) {
    // let _js = match config.document_js(path) {
    //     Some(v) => v,
    //     None => fastn_compiler::compile(config, path, strict).await,
    // };

    todo!()
}
