#[derive(Debug)]
pub struct Export {
    pub name: String,
    pub desc: fastn_wasm::ExportDesc,
}

pub fn func1(
    name: &str,
    arg0: fastn_wasm::PL,
    body: Vec<fastn_wasm::Expression>,
) -> fastn_wasm::Ast {
    fastn_wasm::Ast::Func(fastn_wasm::Func {
        export: Some(name.to_string()),
        params: vec![arg0],
        body,
        ..Default::default()
    })
}

impl Export {
    pub fn to_doc(&self) -> pretty::RcDoc<'static> {
        fastn_wasm::group(
            "export".to_string(),
            Some(pretty::RcDoc::text(format!("\"{}\"", self.name))),
            self.desc.to_doc(),
        )
    }
}

#[derive(Debug)]
pub enum ExportDesc {
    Func { index: fastn_wasm::Index },
}

impl ExportDesc {
    pub fn to_doc(&self) -> pretty::RcDoc<'static> {
        match self {
            ExportDesc::Func { index } => fastn_wasm::named("func", Some(index.to_doc())),
        }
    }
}

#[cfg(test)]
mod test {
    #[track_caller]
    fn e(f: fastn_wasm::Export, s: &str) {
        let g = fastn_wasm::encode(&vec![fastn_wasm::Ast::Export(f)]);
        println!("got: {}", g);
        println!("expected: {}", s);
        assert_eq!(g, s);
    }

    #[test]
    fn test() {
        e(
            fastn_wasm::Export {
                name: "add".to_string(),
                desc: fastn_wasm::ExportDesc::Func {
                    index: "add".into(),
                },
            },
            r#"(module (export "add" (func $add)))"#,
        );
    }
}
