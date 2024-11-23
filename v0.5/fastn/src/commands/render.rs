impl fastn::commands::Render {
    pub async fn run(self, config: &mut fastn_core::Config) {
        let route = config.resolve(self.path.as_str()).await;
        match route {
            fastn_core::Route::Document(path, data) => {
                render_document(config, path.as_str(), data, self.strict)
            }
            _ => todo!(),
        };
    }
}

fn render_document(
    config: &fastn_core::Config,
    path: &str,
    _data: serde_json::Value,
    _strict: bool,
) {
    // let _js = match config.document_js(path) {
    //     Some(v) => v,
    //     None => fastn_compiler::compile(config, path, strict).await,
    // };

    let source = std::fs::File::open(path)
        .and_then(std::io::read_to_string)
        .unwrap();
    let _o = fastn_compiler::compile(
        Box::new(fastn::Symbols {
            auto_imports: &config.auto_imports,
        }),
        &source,
    );
    todo!()
}
