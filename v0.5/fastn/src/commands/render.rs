impl fastn::commands::Render {
    pub async fn run(self, _package: &mut fastn_package::Package, _router: fastn_router::Router) {
        let route = fastn_router::Router::reader().consume(&self).route(
            "/",
            fastn_router::Method::Get,
            &[],
        );
        match route {
            fastn_router::Route::Document(path, data) => {
                let html =
                    fastn::commands::render::render_document(path.as_str(), data, self.strict)
                        .await;
                std::fs::write(path.replace(".ftd", ".html"), html).unwrap();
            }
            _ => todo!(),
        };
    }
}

impl fastn_continuation::Provider for &fastn::commands::Render {
    type Input = Vec<String>;
    type Output = Vec<(String, Option<String>)>;

    fn provide(&self, _input: Self::Input) -> Self::Output {
        todo!()
    }
}

pub async fn render_document(path: &str, _data: serde_json::Value, _strict: bool) -> String {
    let source = std::fs::File::open(path)
        .and_then(std::io::read_to_string)
        .unwrap();
    let o = fastn_compiler::compile(&source, "main", None).consume_with(fastn::symbols::lookup);
    let h = fastn_runtime::HtmlData::from_cd(o.unwrap());
    h.to_test_html()
}
