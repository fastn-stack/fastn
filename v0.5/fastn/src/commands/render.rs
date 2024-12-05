impl fastn::commands::Render {
    pub async fn run(self, config: &mut fastn_core::Config) {
        let route = config.resolve(self.path.as_str()).await;
        match route {
            fastn_core::Route::Document(path, data) => {
                let html = fastn_runtime::render_2024_document(
                    Box::new(fastn::Symbols {}),
                    config.auto_imports.clone(),
                    path.as_str(),
                    data,
                    self.strict,
                )
                .await;
                std::fs::write(path.replace(".ftd", ".html"), html).unwrap();
            }
            _ => todo!(),
        };
    }
}
