impl fastn::commands::Render {
    pub async fn run(self, _package: &mut fastn_package::Package, router: fastn_router::Router) {
        let route = router.route("/", fastn_router::Method::Get);
        match route {
            fastn_router::Route::Document(doc) => {
                let (path, data) = doc.with_data(&[]).unwrap();
                let html =
                    fastn::commands::render::render_document(path.as_str(), data, self.strict)
                        .await;
                std::fs::write(path.replace(".ftd", ".html"), html).unwrap();
            }
            _ => todo!(),
        };
    }
}

pub async fn render_document(
    path: &str,
    _data: serde_json::Map<String, serde_json::Value>,
    _strict: bool,
) -> String {
    let source = std::fs::File::open(path)
        .and_then(std::io::read_to_string)
        .unwrap();
    let o = fastn_compiler::compile(&source, "main", None)
        .consume_with_fn(fastn::definition_provider::lookup);
    let h = fastn_runtime::HtmlData::from_cd(o.unwrap());
    h.to_test_html()
}
