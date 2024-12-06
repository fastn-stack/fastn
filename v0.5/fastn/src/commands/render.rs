impl fastn::commands::Render {
    pub async fn run(self, config: &mut fastn_core::Config) {
        let route = config.resolve(self.path.as_str()).await;
        match route {
            fastn_core::Route::Document(path, data) => {
                let html = fastn::commands::render::render_document(
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

pub async fn render_document(
    global_aliases: fastn_unresolved::AliasesSimple,
    path: &str,
    _data: serde_json::Value,
    _strict: bool,
) -> String {
    let source = std::fs::File::open(path)
        .and_then(std::io::read_to_string)
        .unwrap();
    let mut cs = fastn_compiler::compile(&source, "main", None, global_aliases);
    let mut symbol_store = fastn::Symbols {};

    let o = loop {
        match cs {
            fastn_compiler::CompilerState::StuckOnSymbols(mut c, symbols) => {
                let o = symbol_store.lookup(&mut c, &symbols).await;
                cs = c.continue_with_definitions(o);
            }
            fastn_compiler::CompilerState::Done(c) => {
                break c;
            }
        }
    };

    let h = fastn_runtime::HtmlData::from_cd(o.unwrap());
    h.to_test_html()
}
